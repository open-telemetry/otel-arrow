// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use related_data::RelatedData;
use snafu::ensure;

use crate::arrays::NullableArrayAccessor;
use crate::error::{self, Result, SpanRecordNotFoundSnafu};
use crate::otap::OtapBatch;
use crate::otlp::common::{ResourceArrays, ScopeArrays};
use crate::otlp::metrics::AppendAndGet;
use crate::otlp::traces::spans_arrays::SpansArrays;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;

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
pub fn traces_from(traces_otap_batch: OtapBatch) -> Result<ExportTraceServiceRequest> {
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
        .ok_or(SpanRecordNotFoundSnafu.build())?;

    // Parse all related data (attributes, events, links)
    let mut related_data = RelatedData::try_from(&traces_otap_batch)?;

    // Extract the three main components from the record batch:
    // 1. Resource information (attributes, schema URLs)
    // 2. Scope information (instrumentation scope details)
    // 3. Span data (core span fields)
    let resource_arrays = ResourceArrays::try_from(rb)?;
    let scope_arrays = ScopeArrays::try_from(rb)?;
    let spans_arrays = SpansArrays::try_from(rb)?;

    for idx in 0..rb.num_rows() {
        let res_delta_id = resource_arrays.id.value_at_or_default(idx);
        res_id += res_delta_id;

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
        let scope_delta_id = scope_arrays.id.value_at_or_default(idx);
        scope_id += scope_delta_id;

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
        let delta_id = spans_arrays.id.value_at_or_default(idx);
        let span_id = related_data.span_id_from_delta(delta_id);

        if let Some(trace_id_bytes) = spans_arrays.trace_id.value_at(idx) {
            ensure!(
                trace_id_bytes.len() == 16,
                error::InvalidTraceIdSnafu {
                    message: format!(
                        "span_id = {}, index = {}, trace_id = {:?}",
                        span_id, idx, trace_id_bytes
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
                        "span_id = {}, index = {}, span_id = {:?}",
                        span_id, idx, span_id_bytes
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
                        "span_id = {}, index = {}, parent_span_id = {:?}",
                        span_id, idx, parent_span_id_bytes
                    ),
                }
            );
            current_span.parent_span_id = parent_span_id_bytes;
        }

        current_span.name = spans_arrays.name.value_at_or_default(idx);
        current_span.kind = spans_arrays
            .kind
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

        if let Some(attrs) = related_data
            .span_attr_map_store
            .as_mut()
            .and_then(|store| store.attribute_by_delta_id(delta_id))
        {
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
