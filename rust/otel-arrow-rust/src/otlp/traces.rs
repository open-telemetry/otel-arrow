// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::Array;
use snafu::OptionExt;

use crate::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use crate::error::{self, Error, Result};
use crate::otap::OtapArrowRecords;
use crate::otlp::attributes::{Attribute16Arrays, Attribute32Arrays, encode_key_value};
use crate::otlp::common::{
    BatchSorter, ChildIndexIter, ResourceArrays, ScopeArrays, SortedBatchCursor,
    proto_encode_instrumentation_scope, proto_encode_resource,
};
use crate::otlp::traces::span_event::{SpanEventArrays, encode_span_event};
use crate::otlp::traces::span_link::{SpanLinkArrays, encode_span_link};
use crate::otlp::traces::spans_arrays::SpansArrays;
use crate::otlp::traces::spans_status_arrays::SpanStatusArrays;
use crate::otlp::{ProtoBuffer, ProtoBytesEncoder};
use crate::proto::consts::field_num::traces::{
    RESOURCE_SPANS_RESOURCE, RESOURCE_SPANS_SCHEMA_URL, RESOURCE_SPANS_SCOPE_SPANS,
    SCOPE_SPANS_SCHEMA_URL, SCOPE_SPANS_SCOPE, SCOPE_SPANS_SPANS, SPAN_ATTRIBUTES,
    SPAN_DROPPED_ATTRIBUTES_COUNT, SPAN_DROPPED_EVENTS_COUNT, SPAN_DROPPED_LINKS_COUNT,
    SPAN_END_TIME_UNIX_NANO, SPAN_EVENTS, SPAN_FLAGS, SPAN_KIND, SPAN_LINKS, SPAN_NAME,
    SPAN_PARENT_SPAN_ID, SPAN_SPAN_ID, SPAN_START_TIME_UNIX_NANO, SPAN_STATUS, SPAN_STATUS_CODE,
    SPAN_STATUS_MESSAGE, SPAN_TRACE_ID, SPAN_TRACE_STATE, TRACES_DATA_RESOURCE_SPANS,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::proto_encode_len_delimited_unknown_size;

pub mod delta_decoder;
mod span_event;
mod span_link;
mod spans_arrays;
mod spans_status_arrays;

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

impl Default for TracesProtoBytesEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtoBytesEncoder for TracesProtoBytesEncoder {
    /// encode the OTAP batch into a proto serialized `TracesData` / `ExportTraceServiceRequest`
    /// message into the target buffer
    fn encode(
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
                &res_attrs.parent_id,
                &mut self.resource_attrs_cursor,
            );
        }

        // init cursors for visiting child attributes in the correct order
        if let Some(scope_attrs) = traces_data_arrays.scope_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &scope_attrs.parent_id,
                &mut self.scope_attrs_cursor,
            );
        }
        if let Some(log_attrs) = traces_data_arrays.span_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(&log_attrs.parent_id, &mut self.spans_attrs_cursor);
        }
        if let Some(span_events) = traces_data_arrays.span_events.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &MaybeDictArrayAccessor::Native(span_events.parent_id),
                &mut self.span_events_cursor,
            );
        }
        if let Some(span_event_attrs) = traces_data_arrays.span_event_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &span_event_attrs.parent_id,
                &mut self.span_events_attrs_cursor,
            );
        }

        if let Some(span_links) = traces_data_arrays.span_links.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                &MaybeDictArrayAccessor::Native(span_links.parent_id),
                &mut self.span_links_cursor,
            );
        }
        if let Some(span_link_attrs) = traces_data_arrays.span_link_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u32_id_column(
                &span_link_attrs.parent_id,
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
                    ChildIndexIter::new(id, &span_attrs.parent_id, &mut self.spans_attrs_cursor);
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
                let parent_ids = MaybeDictArrayAccessor::Native(span_events.parent_id);
                let events_index_iter =
                    ChildIndexIter::new(id, &parent_ids, &mut self.span_events_cursor);
                for event_index in events_index_iter {
                    proto_encode_len_delimited_unknown_size!(
                        SPAN_EVENTS,
                        encode_span_event(
                            event_index,
                            span_events,
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
                let parent_ids = MaybeDictArrayAccessor::Native(span_links.parent_id);
                let links_index_iter =
                    ChildIndexIter::new(id, &parent_ids, &mut self.span_links_cursor);
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

        if let Some(status) = &span_arrays.status {
            if status.status.is_valid(index) {
                proto_encode_len_delimited_unknown_size!(
                    SPAN_STATUS,
                    self.encode_span_status(index, status, result_buf),
                    result_buf
                );
            }
        }

        self.root_cursor.advance();

        Ok(())
    }

    fn encode_span_status(
        &mut self,
        index: usize,
        status_arrays: &SpanStatusArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) {
        if let Some(col) = &status_arrays.message {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_string(SPAN_STATUS_MESSAGE, val);
            }
        }

        if let Some(col) = &status_arrays.code {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(SPAN_STATUS_CODE, wire_types::VARINT);
                result_buf.encode_varint(val as u64);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{
        DurationNanosecondArray, FixedSizeBinaryArray, Int32Array, RecordBatch, StringArray,
        StructArray, TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::buffer::NullBuffer;
    use arrow::datatypes::{DataType, Field, Fields, Schema, TimeUnit};
    use pretty_assertions::assert_eq;
    use prost::Message;
    use std::sync::Arc;

    use crate::otap::Traces;
    use crate::otlp::attributes::AttributeValueType;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::span::{Event, Link};
    use crate::proto::opentelemetry::trace::v1::status::StatusCode;
    use crate::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, TracesData,
    };
    use crate::schema::{FieldExt, consts};

    #[test]
    pub fn smoke_test_proto_encoding() {
        // simple smoke test for proto encoding. This doesn't test every field, but those are
        // tested in other test suites in this project that encode/decode OTAP -> OTLP

        let res_struct_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
        ]);
        let scope_struct_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
            Field::new(consts::VERSION, DataType::Utf8, true),
        ]);
        let span_status_fields = Fields::from(vec![
            Field::new(consts::STATUS_MESSAGE, DataType::Utf8, true),
            Field::new(consts::STATUS_CODE, DataType::Int32, false),
        ]);

        let spans_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(res_struct_fields.clone()),
                    true,
                ),
                Field::new(
                    consts::SCOPE,
                    DataType::Struct(scope_struct_fields.clone()),
                    true,
                ),
                Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
                Field::new(
                    consts::START_TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(
                    consts::DURATION_TIME_UNIX_NANO,
                    DataType::Duration(TimeUnit::Nanosecond),
                    true,
                ),
                Field::new(
                    consts::STATUS,
                    DataType::Struct(span_status_fields.clone()),
                    true,
                ),
            ])),
            vec![
                Arc::new(StructArray::new(
                    res_struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values([0, 1, 1]))],
                    None,
                )),
                Arc::new(StructArray::new(
                    scope_struct_fields.clone(),
                    vec![
                        Arc::new(UInt16Array::from_iter_values([0, 1, 2])),
                        Arc::new(StringArray::from_iter_values(vec![
                            "scopev0", "scopev1", "scopev2",
                        ])),
                    ],
                    None,
                )),
                Arc::new(UInt16Array::from_iter_values([0, 1, 2])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![
                            4u128.to_le_bytes().to_vec(),
                            1u128.to_le_bytes().to_vec(),
                            8u128.to_le_bytes().to_vec(),
                        ]
                        .into_iter(),
                    )
                    .unwrap(),
                ),
                Arc::new(TimestampNanosecondArray::from_iter_values([
                    1i64, 5i64, 8i64,
                ])),
                Arc::new(DurationNanosecondArray::from_iter_values([
                    1i64, 2i64, 1i64,
                ])),
                Arc::new(StructArray::new(
                    span_status_fields.clone(),
                    vec![
                        Arc::new(StringArray::from_iter_values(["statusa", "statusb", ""])),
                        Arc::new(Int32Array::from_iter_values([1, 2, 0])),
                    ],
                    Some(NullBuffer::from_iter([true, true, false])),
                )),
            ],
        )
        .unwrap();

        let span_events_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, false),
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(consts::NAME, DataType::Utf8, false),
                Field::new(consts::DROPPED_ATTRIBUTES_COUNT, DataType::UInt32, true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values([0, 1])),
                Arc::new(UInt16Array::from_iter_values([1, 1])),
                Arc::new(TimestampNanosecondArray::from_iter_values([1i64, 2i64])),
                Arc::new(StringArray::from_iter_values(["sea", "seb"])),
                Arc::new(UInt32Array::from_iter_values([0, 2])),
            ],
        )
        .unwrap();

        let span_links_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, false),
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), false),
                Field::new(consts::SPAN_ID, DataType::FixedSizeBinary(8), false),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values([0])),
                Arc::new(UInt16Array::from_iter_values([2])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![16u128.to_le_bytes().to_vec()].into_iter(),
                    )
                    .unwrap(),
                ),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![32u64.to_le_bytes().to_vec()].into_iter(),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        let attr_32_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt32, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values([0, 0])),
                Arc::new(StringArray::from_iter_values(["ake1", "bke1"])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter_values(vec!["aval", "bval"])),
            ],
        )
        .unwrap();

        let attr_16_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values([0, 1])),
                Arc::new(StringArray::from_iter_values(["ake2", "bke2"])),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(StringArray::from_iter_values(vec!["aval", "bval"])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Traces(Traces::default());
        otap_batch.set(ArrowPayloadType::Spans, spans_rb);
        otap_batch.set(ArrowPayloadType::ResourceAttrs, attr_16_rb.clone());
        otap_batch.set(ArrowPayloadType::ScopeAttrs, attr_16_rb.clone());
        otap_batch.set(ArrowPayloadType::SpanAttrs, attr_16_rb.clone());
        otap_batch.set(ArrowPayloadType::SpanEvents, span_events_rb);
        otap_batch.set(ArrowPayloadType::SpanLinks, span_links_rb);
        otap_batch.set(ArrowPayloadType::SpanEventAttrs, attr_32_rb.clone());
        otap_batch.set(ArrowPayloadType::SpanLinkAttrs, attr_32_rb.clone());

        let mut result_buf = ProtoBuffer::new();
        let mut encoder = TracesProtoBytesEncoder::new();
        encoder.encode(&mut otap_batch, &mut result_buf).unwrap();

        let result = TracesData::decode(result_buf.as_ref()).unwrap();

        let expected = TracesData::new(vec![
            ResourceSpans::build(Resource {
                attributes: vec![KeyValue::new("ake2", AnyValue::new_string("aval"))],
                ..Default::default()
            })
            .scope_spans(vec![
                ScopeSpans::build(InstrumentationScope {
                    version: "scopev0".to_string(),
                    attributes: vec![KeyValue::new("ake2", AnyValue::new_string("aval"))],
                    ..Default::default()
                })
                .spans(vec![Span {
                    trace_id: 4u128.to_le_bytes().to_vec(),
                    start_time_unix_nano: 1,
                    end_time_unix_nano: 2,
                    status: Some(Status {
                        message: "statusa".to_string(),
                        code: StatusCode::Ok as i32,
                    }),
                    attributes: vec![KeyValue::new("ake2", AnyValue::new_string("aval"))],
                    ..Default::default()
                }])
                .finish(),
            ])
            .finish(),
            ResourceSpans::build(Resource {
                attributes: vec![KeyValue::new("bke2", AnyValue::new_string("bval"))],
                ..Default::default()
            })
            .scope_spans(vec![
                ScopeSpans::build(InstrumentationScope {
                    version: "scopev1".to_string(),
                    attributes: vec![KeyValue::new("bke2", AnyValue::new_string("bval"))],
                    ..Default::default()
                })
                .spans(vec![Span {
                    trace_id: 1u128.to_le_bytes().to_vec(),
                    start_time_unix_nano: 5,
                    end_time_unix_nano: 7,
                    attributes: vec![KeyValue::new("bke2", AnyValue::new_string("bval"))],
                    status: Some(Status {
                        message: "statusb".to_string(),
                        code: StatusCode::Error as i32,
                    }),
                    events: vec![
                        Event {
                            time_unix_nano: 1,
                            name: "sea".to_string(),
                            dropped_attributes_count: 0,
                            attributes: vec![
                                KeyValue::new("ake1", AnyValue::new_string("aval")),
                                KeyValue::new("bke1", AnyValue::new_string("bval")),
                            ],
                        },
                        Event {
                            time_unix_nano: 2,
                            name: "seb".to_string(),
                            dropped_attributes_count: 2,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }])
                .finish(),
                ScopeSpans::build(InstrumentationScope {
                    version: "scopev2".to_string(),
                    ..Default::default()
                })
                .spans(vec![Span {
                    trace_id: 8u128.to_le_bytes().to_vec(),
                    start_time_unix_nano: 8,
                    end_time_unix_nano: 9,
                    links: vec![Link {
                        trace_id: 16u128.to_le_bytes().to_vec(),
                        span_id: 32u64.to_le_bytes().to_vec(),
                        attributes: vec![
                            KeyValue::new("ake1", AnyValue::new_string("aval")),
                            KeyValue::new("bke1", AnyValue::new_string("bval")),
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                }])
                .finish(),
            ])
            .finish(),
        ]);

        assert_eq!(result, expected);
    }
}
