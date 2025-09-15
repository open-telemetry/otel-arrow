// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::StructArray;
use related_data::RelatedData;
use snafu::{OptionExt, ensure};

use crate::arrays::NullableArrayAccessor;
use crate::error::{self, Error, Result};
use crate::otap::OtapArrowRecords;
use crate::otlp::ProtoBuffer;
use crate::otlp::attributes::{Attribute16Arrays, Attribute32Arrays, encode_key_value};
use crate::otlp::common::{
    BatchSorter, ChildIndexIter, ResourceArrays, ScopeArrays, SortedBatchCursor,
    proto_encode_instrumentation_scope, proto_encode_resource,
};
use crate::otlp::metrics::AppendAndGet;
use crate::otlp::traces::span_event::{SpanEventArrays, encode_span_event};
use crate::otlp::traces::span_link::{SpanLinkArrays, encode_span_link};
use crate::otlp::traces::spans_arrays::SpansArrays;
use crate::proto::consts::field_num::traces::{
    RESOURCE_SPANS_RESOURCE, RESOURCE_SPANS_SCHEMA_URL, RESOURCE_SPANS_SCOPE_SPANS,
    SCOPE_SPANS_SCHEMA_URL, SCOPE_SPANS_SCOPE, SCOPE_SPANS_SPANS, SPAN_ATTRIBUTES,
    SPAN_DROPPED_ATTRIBUTES_COUNT, SPAN_DROPPED_EVENTS_COUNT, SPAN_DROPPED_LINKS_COUNT,
    SPAN_END_TIME_UNIX_NANO, SPAN_EVENTS, SPAN_FLAGS, SPAN_KIND, SPAN_LINKS, SPAN_NAME,
    SPAN_PARENT_SPAN_ID, SPAN_SPAN_ID, SPAN_START_TIME_UNIX_NANO, SPAN_TRACE_ID, SPAN_TRACE_STATE,
    TRACES_DATA_RESOURCE_SPANS,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::{consts, is_id_plain_encoded};

pub mod delta_decoder;
mod related_data;
mod span_event;
mod span_event_store;
mod span_link;
mod span_links_store;
mod spans_arrays;
mod spans_status_arrays;

/// Converts an OTAP Arrow batch into OpenTelemetry trace protocol format.
///
/// The function processes the batch row-by-row, handling:
/// - Resource spans grouping (via delta-encoded resource IDs)
/// - Scope spans grouping (via delta-encoded scope IDs)
/// - Individual span conversion with:
///   - Trace/span ID validation
///   - Attribute/event/link attachment from related data
///   - Time/duration calculations
///
/// # Parameters
/// - `traces_otap_batch`: Input batch containing span data in OTAP Arrow format
///
/// # Returns
/// - `Result<ExportTraceServiceRequest>`: Converted trace data in OTLP format
pub fn traces_from(traces_otap_batch: OtapArrowRecords) -> Result<ExportTraceServiceRequest> {
    // Initialize empty trace export request
    let mut traces = ExportTraceServiceRequest::default();

    // Track previous resource and scope IDs for delta decoding
    let mut prev_res_id: Option<u16> = None;
    let mut prev_scope_id: Option<u16> = None;

    // Current resource and scope id
    let mut res_id = 0;
    let mut scope_id = 0;

    let rb = traces_otap_batch
        .get(ArrowPayloadType::Spans)
        .ok_or(error::SpanRecordNotFoundSnafu.build())?;

    // Parse all related data (attributes, events, links)
    let mut related_data = RelatedData::try_from(&traces_otap_batch)?;

    // Extract the three main components from the record batch:
    // 1. Resource information (attributes, schema URLs)
    // 2. Scope information (instrumentation scope details)
    // 3. Span data (core span fields)
    let resource_arrays = ResourceArrays::try_from(rb)?;
    let scope_arrays = ScopeArrays::try_from(rb)?;
    let spans_arrays = SpansArrays::try_from(rb)?;

    let ids_plain_encoded = is_id_plain_encoded(rb);
    let resource_ids_plain_encoded = rb
        .column_by_name(consts::RESOURCE)
        .and_then(|col| col.as_any().downcast_ref::<StructArray>())
        .and_then(|col_struct| col_struct.fields().find(consts::ID))
        .and_then(|(_, field)| field.metadata().get(consts::metadata::COLUMN_ENCODING))
        .map(|encoding| encoding.as_str() == consts::metadata::encodings::PLAIN)
        .unwrap_or(false);

    let scope_ids_plain_encoded = rb
        .column_by_name(consts::SCOPE)
        .and_then(|col| col.as_any().downcast_ref::<StructArray>())
        .and_then(|col_struct| col_struct.fields().find(consts::ID))
        .and_then(|(_, field)| field.metadata().get(consts::metadata::COLUMN_ENCODING))
        .map(|encoding| encoding.as_str() == consts::metadata::encodings::PLAIN)
        .unwrap_or(false);

    for idx in 0..rb.num_rows() {
        let res_maybe_delta_id = resource_arrays.id.value_at(idx).unwrap_or_default();
        if resource_ids_plain_encoded {
            res_id = res_maybe_delta_id;
        } else {
            res_id += res_maybe_delta_id;
        }

        // When resource ID changes, create new ResourceSpans entry
        if prev_res_id != Some(res_id) {
            prev_res_id = Some(res_id);
            let resource_spans = traces.resource_spans.append_and_get();
            prev_scope_id = None;

            // Update the resource field of the current resource spans
            let resource = resource_spans.resource.get_or_insert_default();
            resource.dropped_attributes_count = resource_arrays
                .dropped_attributes_count
                .value_at_or_default(idx);

            if let Some(res_id) = resource_arrays.id.value_at(idx) {
                if let Some(attrs) = related_data
                    .res_attr_map_store
                    .as_mut()
                    .and_then(|store| store.attribute_by_delta_id(res_id))
                {
                    resource.attributes = attrs.to_vec();
                }
            }

            resource_spans.schema_url =
                resource_arrays.schema_url.value_at(idx).unwrap_or_default();
        }

        // Decode scope ID using delta encoding
        let scope_maybe_delta_id_opt = scope_arrays.id.value_at(idx);
        if scope_ids_plain_encoded {
            scope_id = scope_maybe_delta_id_opt.unwrap_or_default();
        } else {
            scope_id += scope_maybe_delta_id_opt.unwrap_or_default();
        }

        // When scope ID changes, create new ScopeSpans entry
        if prev_scope_id != Some(scope_id) {
            prev_scope_id = Some(scope_id);
            // safety: we must have appended at least one resource spans when reach here
            let current_scope_spans_slice = &mut traces
                .resource_spans
                .last_mut()
                .expect("At this stage, we should have at least one resource span.")
                .scope_spans;
            // create new ScopeSpans
            let scope_spans = current_scope_spans_slice.append_and_get();

            let mut scope = scope_arrays.create_instrumentation_scope(idx);
            if let Some(attrs) = related_data
                .scope_attr_map_store
                .as_mut()
                .and_then(|store| store.attribute_by_delta_id(scope_id))
            {
                scope.attributes = attrs.to_vec();
            }

            scope_spans.scope = Some(scope);
            scope_spans.schema_url = spans_arrays.schema_url.value_at(idx).unwrap_or_default();
        }

        // Process individual span data:
        // - Validate trace/span IDs
        // - Set timing information (start + duration = end time)
        // - Attach attributes, events and links from related data
        let current_scope_spans = &mut traces
            .resource_spans
            .last_mut()
            .expect("At this stage, we should have at least one resource span.")
            .scope_spans
            .last_mut()
            .expect("At this stage, we should have added at least one scope span.");

        let current_span = current_scope_spans.spans.append_and_get();
        let maybe_delta_id = spans_arrays.id.value_at_or_default(idx);
        let span_id = if ids_plain_encoded {
            maybe_delta_id
        } else {
            related_data.span_id_from_delta(maybe_delta_id)
        };

        if let Some(trace_id_bytes) = spans_arrays.trace_id.value_at(idx) {
            ensure!(
                trace_id_bytes.len() == 16,
                error::InvalidTraceIdSnafu {
                    message: format!(
                        "span_id = {span_id}, index = {idx}, trace_id = {trace_id_bytes:?}"
                    ),
                }
            );
            current_span.trace_id = trace_id_bytes;
        }

        if let Some(span_id_bytes) = spans_arrays.span_id.value_at(idx) {
            ensure!(
                span_id_bytes.len() == 8,
                error::InvalidSpanIdSnafu {
                    message: format!(
                        "span_id = {span_id}, index = {idx}, span_id = {span_id_bytes:?}"
                    ),
                }
            );
            current_span.span_id = span_id_bytes;
        }

        if let Some(parent_span_id_bytes) = spans_arrays.parent_span_id.value_at(idx) {
            ensure!(
                parent_span_id_bytes.len() == 8,
                error::InvalidSpanIdSnafu {
                    message: format!(
                        "span_id = {span_id}, index = {idx}, parent_span_id = {parent_span_id_bytes:?}"
                    ),
                }
            );
            current_span.parent_span_id = parent_span_id_bytes;
        }

        current_span.name = spans_arrays.name.value_at_or_default(idx);
        current_span.kind = spans_arrays
            .kind
            .as_ref()
            .map(|arr| arr.value_at_or_default(idx))
            .unwrap_or_default();
        current_span.start_time_unix_nano =
            spans_arrays.start_time_unix_nano.value_at_or_default(idx) as u64;
        current_span.end_time_unix_nano = current_span.start_time_unix_nano
            + spans_arrays
                .duration_time_unix_nano
                .value_at_or_default(idx) as u64;
        current_span.dropped_attributes_count = spans_arrays
            .dropped_attributes_count
            .value_at_or_default(idx);
        current_span.dropped_events_count =
            spans_arrays.dropped_events_count.value_at_or_default(idx);
        current_span.dropped_links_count =
            spans_arrays.dropped_links_count.value_at_or_default(idx);

        if let Some(status_val) = spans_arrays.status.value_at(idx) {
            current_span.status = Some(status_val);
        }

        let attrs = related_data.span_attr_map_store.as_mut().and_then(|store| {
            if ids_plain_encoded {
                store.attribute_by_id(span_id)
            } else {
                store.attribute_by_delta_id(span_id)
            }
        });
        if let Some(attrs) = attrs {
            current_span.attributes = attrs.to_vec();
        }

        // Add span events if available
        if let Some(events) = related_data
            .span_events_store
            .as_mut()
            .and_then(|store| store.event_by_id(span_id))
        {
            current_span.events = events.clone();
        }

        // Add span links if available
        if let Some(links) = related_data
            .span_links_store
            .as_mut()
            .and_then(|store| store.link_by_id(span_id))
        {
            current_span.links = links.clone();
        }
    }

    Ok(traces)
}

struct TracesDataArrays<'a> {
    span_arrays: SpansArrays<'a>,
    scope_arrays: ScopeArrays<'a>,
    resource_arrays: ResourceArrays<'a>,
    span_attrs: Option<Attribute16Arrays<'a>>,
    scope_attrs: Option<Attribute16Arrays<'a>>,
    resource_attrs: Option<Attribute16Arrays<'a>>,
    span_events: Option<SpanEventArrays<'a>>,
    span_event_attrs: Option<Attribute32Arrays<'a>>,
    span_links: Option<SpanLinkArrays<'a>>,
    span_link_attrs: Option<Attribute32Arrays<'a>>,
}

impl<'a> TryFrom<&'a OtapArrowRecords> for TracesDataArrays<'a> {
    type Error = Error;

    fn try_from(otap_batch: &'a OtapArrowRecords) -> Result<Self> {
        let spans_rb = otap_batch
            .get(ArrowPayloadType::Spans)
            .context(error::SpanRecordNotFoundSnafu)?;

        Ok(Self {
            span_arrays: SpansArrays::try_from(spans_rb)?,
            scope_arrays: ScopeArrays::try_from(spans_rb)?,
            resource_arrays: ResourceArrays::try_from(spans_rb)?,
            span_attrs: otap_batch
                .get(ArrowPayloadType::SpanAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            scope_attrs: otap_batch
                .get(ArrowPayloadType::ScopeAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            resource_attrs: otap_batch
                .get(ArrowPayloadType::ResourceAttrs)
                .map(Attribute16Arrays::try_from)
                .transpose()?,
            span_events: otap_batch
                .get(ArrowPayloadType::SpanEvents)
                .map(SpanEventArrays::try_from)
                .transpose()?,
            span_event_attrs: otap_batch
                .get(ArrowPayloadType::SpanEventAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
            span_links: otap_batch
                .get(ArrowPayloadType::SpanLinks)
                .map(SpanLinkArrays::try_from)
                .transpose()?,
            span_link_attrs: otap_batch
                .get(ArrowPayloadType::SpanLinkAttrs)
                .map(Attribute32Arrays::try_from)
                .transpose()?,
        })
    }
}

/// This is used to encode OTAP Batches of Spans data into serialized protobuf
// TODO -- a docs test
pub struct TracesProtoBytesEncoder {
    batch_sorter: BatchSorter,
    root_cursor: SortedBatchCursor,
    resource_attrs_cursor: SortedBatchCursor,
    scope_attrs_cursor: SortedBatchCursor,
    spans_attrs_cursor: SortedBatchCursor,
    span_links_cursor: SortedBatchCursor,
    span_links_attrs_cursor: SortedBatchCursor,
    span_events_cursor: SortedBatchCursor,
    span_events_attrs_cursor: SortedBatchCursor,
}

impl TracesProtoBytesEncoder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            batch_sorter: BatchSorter::new(),
            root_cursor: SortedBatchCursor::new(),
            resource_attrs_cursor: SortedBatchCursor::new(),
            scope_attrs_cursor: SortedBatchCursor::new(),
            spans_attrs_cursor: SortedBatchCursor::new(),
            span_links_cursor: SortedBatchCursor::new(),
            span_links_attrs_cursor: SortedBatchCursor::new(),
            span_events_cursor: SortedBatchCursor::new(),
            span_events_attrs_cursor: SortedBatchCursor::new(),
        }
    }

    pub fn reset(&mut self) {
        self.root_cursor.reset();
        self.resource_attrs_cursor.reset();
        self.scope_attrs_cursor.reset();
        self.spans_attrs_cursor.reset();
        self.span_links_cursor.reset();
        self.span_links_attrs_cursor.reset();
        self.span_events_cursor.reset();
        self.span_events_attrs_cursor.reset();
    }

    /// encode the OTAP batch into a proto serialized `TracesData` / `ExportTraceServiceRequest`
    /// message into the target buffer
    pub fn encode(
        &mut self,
        otap_batch: &mut OtapArrowRecords,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        otap_batch.decode_transport_optimized_ids()?;
        let traces_data_arrays = TracesDataArrays::try_from(&*otap_batch)?;

        self.reset();

        // get the list of indices from the root record batch to visit in order
        let spans_rb = otap_batch
            .get(ArrowPayloadType::Spans)
            .context(error::SpanRecordNotFoundSnafu)?;
        self.batch_sorter
            .init_cursor_for_root_batch(spans_rb, &mut self.root_cursor)?;

        // get the lists of child indices for attributes to visit in oder:
        if let Some(res_attrs) = traces_data_arrays.resource_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                res_attrs.parent_id,
                &mut self.resource_attrs_cursor,
            );
        }

        // init cursors for visiting child attributes in the correct order
        if let Some(scope_attrs) = traces_data_arrays.scope_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(scope_attrs.parent_id, &mut self.scope_attrs_cursor);
        }
        if let Some(log_attrs) = traces_data_arrays.span_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(log_attrs.parent_id, &mut self.spans_attrs_cursor);
        }
        if let Some(span_events) = traces_data_arrays.span_events.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(span_events.parent_id, &mut self.span_events_cursor);
        }
        if let Some(span_event_attrs) = traces_data_arrays.span_event_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                span_event_attrs.parent_id,
                &mut self.span_events_attrs_cursor,
            );
        }
        if let Some(span_links) = traces_data_arrays.span_links.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(span_links.parent_id, &mut self.span_links_cursor);
        }
        if let Some(span_link_attrs) = traces_data_arrays.span_link_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                span_link_attrs.parent_id,
                &mut self.span_links_attrs_cursor,
            );
        }

        // encode all the `ResourceSpan`s for this `TracesData`
        loop {
            proto_encode_len_delimited_unknown_size!(
                TRACES_DATA_RESOURCE_SPANS,
                self.encode_resource_spans(&traces_data_arrays, result_buf)?,
                result_buf
            );

            if self.root_cursor.finished() {
                break;
            }
        }

        Ok(())
    }

    fn encode_resource_spans(
        &mut self,
        traces_data_arrays: &TracesDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()), // no more rows to visit
        };

        // encode the `Resource`
        proto_encode_len_delimited_unknown_size!(
            RESOURCE_SPANS_RESOURCE,
            proto_encode_resource(
                index,
                &traces_data_arrays.resource_arrays,
                traces_data_arrays.resource_attrs.as_ref(),
                &mut self.resource_attrs_cursor,
                result_buf,
            )?,
            result_buf
        );

        // encode all the `ScopeSpan`s for this `ResourceSpan`
        let resource_id = traces_data_arrays.resource_arrays.id.value_at(index);

        loop {
            proto_encode_len_delimited_unknown_size!(
                RESOURCE_SPANS_SCOPE_SPANS,
                self.encode_scope_spans(traces_data_arrays, result_buf)?,
                result_buf
            );

            // break when we've reached the end of the root record batch
            if self.root_cursor.finished() {
                break;
            }

            // safety: we've just checked that the cursor isn't finished
            let next_index = self.root_cursor.curr_index().expect("cursor not finished");

            // check if we've found a new resource ID. If so, break
            if resource_id != traces_data_arrays.resource_arrays.id.value_at(next_index) {
                break;
            }
        }

        // encode the schema url
        if let Some(col) = &traces_data_arrays.resource_arrays.schema_url {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(RESOURCE_SPANS_SCHEMA_URL, val);
            }
        }

        Ok(())
    }

    fn encode_scope_spans(
        &mut self,
        traces_data_arrays: &TracesDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()), // no more rows to visit
        };

        // encode the `InstrumentationScope`
        proto_encode_len_delimited_unknown_size!(
            SCOPE_SPANS_SCOPE,
            proto_encode_instrumentation_scope(
                index,
                &traces_data_arrays.scope_arrays,
                traces_data_arrays.scope_attrs.as_ref(),
                &mut self.scope_attrs_cursor,
                result_buf
            )?,
            result_buf
        );

        // encode all `Span`s for this `ScopeSpan`
        let scope_id = traces_data_arrays.scope_arrays.id.value_at(index);
        loop {
            proto_encode_len_delimited_unknown_size!(
                SCOPE_SPANS_SPANS,
                self.encode_span(traces_data_arrays, result_buf)?,
                result_buf
            );

            // break if we've reached the end of the record batch
            if self.root_cursor.finished() {
                break;
            }

            // Safety: we've just checked above that cursor isn't finished
            let next_index = self.root_cursor.curr_index().expect("cursor not finished");

            // check if we've found a new scope ID. If so, break
            if scope_id != traces_data_arrays.scope_arrays.id.value_at(next_index) {
                break;
            }
        }

        // encode the schema url
        if let Some(col) = &traces_data_arrays.span_arrays.schema_url {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(SCOPE_SPANS_SCHEMA_URL, val);
            }
        }

        Ok(())
    }

    fn encode_span(
        &mut self,
        traces_data_arrays: &TracesDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()), // no more rows to visit
        };

        let span_arrays = &traces_data_arrays.span_arrays;

        if let Some(col) = &span_arrays.trace_id {
            if let Some(val) = col.slice_at(index) {
                result_buf.encode_bytes(SPAN_TRACE_ID, val);
            }
        }

        if let Some(col) = &span_arrays.span_id {
            if let Some(val) = col.slice_at(index) {
                result_buf.encode_bytes(SPAN_SPAN_ID, val);
            }
        }

        if let Some(col) = &span_arrays.trace_state {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(SPAN_TRACE_STATE, val);
            }
        }

        if let Some(col) = &span_arrays.parent_span_id {
            if let Some(val) = col.slice_at(index) {
                result_buf.encode_bytes(SPAN_PARENT_SPAN_ID, val);
            }
        }

        if let Some(col) = &span_arrays.flags {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(SPAN_FLAGS, wire_types::FIXED32);
                result_buf.extend_from_slice(&val.to_le_bytes());
            }
        }

        if let Some(col) = &span_arrays.name {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(SPAN_NAME, val);
            }
        }

        if let Some(col) = &span_arrays.kind {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(SPAN_KIND, wire_types::VARINT);
                result_buf.encode_varint(val as u64);
            }
        }

        if let Some(col) = &span_arrays.start_time_unix_nano {
            if let Some(start_time) = col.value_at(index) {
                result_buf.encode_field_tag(SPAN_START_TIME_UNIX_NANO, wire_types::FIXED64);
                result_buf.extend_from_slice(&start_time.to_le_bytes());

                // encode end time from start + duration
                if let Some(col) = &span_arrays.duration_time_unix_nano {
                    if let Some(duration) = col.value_at(index) {
                        let end_time = start_time + duration;
                        result_buf.encode_field_tag(SPAN_END_TIME_UNIX_NANO, wire_types::FIXED64);
                        result_buf.extend_from_slice(&end_time.to_le_bytes());
                    }
                }
            }
        }

        if let Some(span_attrs) = &traces_data_arrays.span_attrs {
            if let Some(id) = span_arrays.id.value_at(index) {
                let attrs_index_iter =
                    ChildIndexIter::new(id, span_attrs.parent_id, &mut self.spans_attrs_cursor);
                for attr_index in attrs_index_iter {
                    proto_encode_len_delimited_unknown_size!(
                        SPAN_ATTRIBUTES,
                        encode_key_value(span_attrs, attr_index, result_buf)?,
                        result_buf
                    );
                }
            }
        }

        if let Some(col) = span_arrays.dropped_attributes_count {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(SPAN_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT);
                result_buf.encode_varint(val as u64);
            }
        }

        if let Some(span_events) = &traces_data_arrays.span_events {
            if let Some(id) = span_arrays.id.value_at(index) {
                let events_index_iter =
                    ChildIndexIter::new(id, span_events.parent_id, &mut self.span_events_cursor);
                for event_index in events_index_iter {
                    proto_encode_len_delimited_unknown_size!(
                        SPAN_EVENTS,
                        encode_span_event(
                            event_index,
                            &span_events,
                            &mut self.span_events_attrs_cursor,
                            traces_data_arrays.span_event_attrs.as_ref(),
                            result_buf
                        )?,
                        result_buf
                    );
                }
            }
        }

        if let Some(col) = span_arrays.dropped_events_count {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(SPAN_DROPPED_EVENTS_COUNT, wire_types::VARINT);
                result_buf.encode_varint(val as u64);
            }
        }

        if let Some(span_links) = &traces_data_arrays.span_links {
            if let Some(id) = span_arrays.id.value_at(index) {
                let links_index_iter =
                    ChildIndexIter::new(id, span_links.parent_id, &mut self.span_links_cursor);
                for link_index in links_index_iter {
                    proto_encode_len_delimited_unknown_size!(
                        SPAN_LINKS,
                        encode_span_link(
                            link_index,
                            span_links,
                            &mut self.span_links_attrs_cursor,
                            traces_data_arrays.span_link_attrs.as_ref(),
                            result_buf
                        )?,
                        result_buf
                    );
                }
            }
        }

        if let Some(col) = span_arrays.dropped_links_count {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(SPAN_DROPPED_LINKS_COUNT, wire_types::VARINT);
                result_buf.encode_varint(val as u64);
            }
        }

        Ok(())
    }
}
