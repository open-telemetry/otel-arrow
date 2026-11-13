// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared OTAP batch construction for the traffic generator data sources.
//!
//! Both the synthetic and the semantic-conventions generators build their
//! signals as OTAP record batches through the batch types here and encode
//! them to OTLP with [`OtlpEncoder`]. The batch types own the record batch
//! builders, id bookkeeping, and the columns that are the same for every
//! generated row (one resource, one scope, no schema url, no dropped counts),
//! so the generators only supply what actually varies.

use crate::receivers::traffic_generator::fake_data::{
    SCOPE_NAME, SCOPE_VERSION, current_time, delay, gen_span_id_bytes, gen_trace_id_bytes,
};
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::arrow::array::RecordBatch;
use otel_arrow_dfe_pdata::arrow::datatypes::ArrowPrimitiveType;
use otel_arrow_dfe_pdata::encode::append_attribute_value;
use otel_arrow_dfe_pdata::encode::record::attributes::{
    AttributesRecordBatchBuilder, AttributesRecordBatchBuilderConstructorHelper,
};
use otel_arrow_dfe_pdata::encode::record::logs::{
    LogsRecordBatchBuilder, ResourceBuilder, ScopeBuilder,
};
use otel_arrow_dfe_pdata::encode::record::metrics::{
    ExponentialHistogramDataPointsRecordBatchBuilder, HistogramDataPointsRecordBatchBuilder,
    MetricsRecordBatchBuilder, NumberDataPointsRecordBatchBuilder,
    SummaryDataPointsRecordBatchBuilder,
};
use otel_arrow_dfe_pdata::encode::record::traces::{
    EventsRecordBatchBuilder, TracesRecordBatchBuilder,
};
use otel_arrow_dfe_pdata::encode::{Error, Result};
use otel_arrow_dfe_pdata::otap::{Logs, Metrics, OtapArrowRecords, Traces};
use otel_arrow_dfe_pdata::otlp::attributes::parent_id::ParentId;
use otel_arrow_dfe_pdata::otlp::logs::LogsProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::metrics::{MetricType, MetricsProtoBytesEncoder};
use otel_arrow_dfe_pdata::otlp::traces::TracesProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::{ProtoBuffer, ProtoBytesEncoder};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::KeyValue;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::SeverityNumber;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::AggregationTemporality;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::span::SpanKind;
use otel_arrow_dfe_pdata::views::otlp::proto::common::ObjKeyValue;
use otel_arrow_dfe_pdata::views::otlp::proto::wrappers::GenericObj;

/// Encodes generated OTAP batches into OTLP proto bytes.
///
/// Holds the per-signal encoders and a scratch buffer so they are reused
/// across batches instead of being re-created for every payload.
pub struct OtlpEncoder {
    logs: LogsProtoBytesEncoder,
    traces: TracesProtoBytesEncoder,
    metrics: MetricsProtoBytesEncoder,
    buffer: ProtoBuffer,
}

impl Default for OtlpEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl OtlpEncoder {
    /// Create a new encoder with empty scratch state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            logs: LogsProtoBytesEncoder::new(),
            traces: TracesProtoBytesEncoder::new(),
            metrics: MetricsProtoBytesEncoder::new(),
            buffer: ProtoBuffer::default(),
        }
    }

    /// Encode an OTAP batch into the matching OTLP export request bytes.
    pub fn encode(
        &mut self,
        mut records: OtapArrowRecords,
    ) -> otel_arrow_dfe_pdata::error::Result<OtlpProtoBytes> {
        self.buffer.clear();
        match records {
            OtapArrowRecords::Logs(_) => self.logs.encode(&mut records, &mut self.buffer)?,
            OtapArrowRecords::Traces(_) => self.traces.encode(&mut records, &mut self.buffer)?,
            OtapArrowRecords::Metrics(_) => self.metrics.encode(&mut records, &mut self.buffer)?,
        }

        // The encoded bytes are handed off with the payload, so the buffer
        // allocation goes with them. Re-reserve the same capacity so the next
        // batch of similar size does not have to grow the buffer again.
        let (bytes, capacity) = self.buffer.take_into_bytes();
        self.buffer.ensure_capacity(capacity);

        Ok(match records {
            OtapArrowRecords::Logs(_) => OtlpProtoBytes::ExportLogsRequest(bytes),
            OtapArrowRecords::Traces(_) => OtlpProtoBytes::ExportTracesRequest(bytes),
            OtapArrowRecords::Metrics(_) => OtlpProtoBytes::ExportMetricsRequest(bytes),
        })
    }
}

/// Columns of one generated log record that vary by data source.
pub(super) struct LogRecordRow<'a> {
    pub(super) severity_number: SeverityNumber,
    pub(super) severity_text: Option<&'a str>,
    pub(super) event_name: Option<&'a str>,
    /// Populate random trace and span ids.
    pub(super) trace_context: bool,
    pub(super) body: Option<&'a str>,
    /// Whether attributes will be appended for this record. Records without
    /// attributes get a null id since ids only link attributes.
    pub(super) has_attributes: bool,
}

/// Builds a logs batch under one resource and one scope.
pub(super) struct LogsBatch {
    logs: LogsRecordBatchBuilder,
    /// Attributes of the single resource (parent id 0).
    pub(super) resource_attrs: AttributesRecordBatchBuilder<u16>,
    /// Log record attributes, keyed by the id returned from `append_log`.
    pub(super) log_attrs: AttributesRecordBatchBuilder<u16>,
    count: usize,
}

impl LogsBatch {
    pub(super) fn new() -> Self {
        Self {
            logs: LogsRecordBatchBuilder::new(),
            resource_attrs: AttributesRecordBatchBuilder::new(),
            log_attrs: AttributesRecordBatchBuilder::new(),
            count: 0,
        }
    }

    /// Append one log record stamped with the current time and return its id.
    pub(super) fn append_log(&mut self, row: LogRecordRow<'_>) -> Result<u16> {
        let id = next_u16_id(self.count)?;
        self.count += 1;

        let timestamp = current_time() as i64;
        self.logs.append_time_unix_nano(timestamp);
        self.logs.append_observed_time_unix_nano(timestamp);
        self.logs
            .append_severity_number(Some(row.severity_number as i32));
        self.logs
            .append_severity_text(row.severity_text.map(str::as_bytes));
        self.logs
            .append_event_name(row.event_name.map(str::as_bytes));
        if row.trace_context {
            self.logs.append_trace_id(Some(&gen_trace_id_bytes()))?;
            self.logs.append_span_id(Some(&gen_span_id_bytes()))?;
        } else {
            self.logs.append_trace_id(None)?;
            self.logs.append_span_id(None)?;
        }
        match row.body {
            Some(body) => self.logs.body.append_str(body.as_bytes()),
            None => self.logs.body.append_null(),
        }
        self.logs.append_id(row.has_attributes.then_some(id));
        Ok(id)
    }

    pub(super) fn finish(mut self) -> Result<OtapArrowRecords> {
        if self.count == 0 {
            return Ok(OtapArrowRecords::Logs(Logs::default()));
        }
        let n = self.count;
        append_single_resource_scope(&mut self.logs.resource, &mut self.logs.scope, n);
        self.logs.append_schema_url_n(None, n);
        self.logs.append_dropped_attributes_count_n(0, n);
        self.logs.append_flags_n(None, n);

        let mut records = OtapArrowRecords::Logs(Logs::default());
        set_payloads(
            &mut records,
            [
                (self.logs.finish()?, ArrowPayloadType::Logs),
                (self.log_attrs.finish()?, ArrowPayloadType::LogAttrs),
                (
                    self.resource_attrs.finish()?,
                    ArrowPayloadType::ResourceAttrs,
                ),
            ],
        )?;
        Ok(records)
    }
}

/// Builds a traces batch under one resource and one scope.
pub(super) struct TracesBatch {
    spans: TracesRecordBatchBuilder,
    events: EventsRecordBatchBuilder,
    /// Attributes of the single resource (parent id 0).
    pub(super) resource_attrs: AttributesRecordBatchBuilder<u16>,
    /// Span attributes, keyed by the id returned from `append_span`.
    pub(super) span_attrs: AttributesRecordBatchBuilder<u16>,
    count: usize,
    next_event_id: u32,
}

impl TracesBatch {
    pub(super) fn new() -> Self {
        Self {
            spans: TracesRecordBatchBuilder::new(),
            events: EventsRecordBatchBuilder::new(),
            resource_attrs: AttributesRecordBatchBuilder::new(),
            span_attrs: AttributesRecordBatchBuilder::new(),
            count: 0,
            next_event_id: 0,
        }
    }

    /// Append one root span starting now with random trace/span ids and
    /// return its id.
    pub(super) fn append_span(&mut self, name: &str, kind: SpanKind) -> Result<u16> {
        let id = next_u16_id(self.count)?;
        self.count += 1;

        let start_time = current_time();
        let end_time = start_time + delay();
        let spans = &mut self.spans;
        spans.append_id(Some(id));
        spans.append_start_time_unix_nano(start_time as i64);
        spans.append_duration_time_unix_nano(end_time as i64 - start_time as i64);
        spans.append_trace_id(gen_trace_id_bytes())?;
        spans.append_span_id(gen_span_id_bytes())?;
        spans.append_trace_state(None);
        spans.append_parent_span_id(None)?;
        spans.append_flags(None);
        spans.append_name(name.as_bytes());
        spans.append_kind(Some(kind as i32));
        spans.append_dropped_attributes_count(Some(0));
        spans.append_dropped_events_count(Some(0));
        spans.append_dropped_links_count(Some(0));
        spans.status.append_code(None);
        spans.status.append_status_message(None);
        Ok(id)
    }

    /// Append an attribute-less event at the current time to span `span_id`.
    pub(super) fn append_event(&mut self, span_id: u16, name: &str) -> Result<()> {
        let id = self.next_event_id;
        self.next_event_id = id.checked_add(1).ok_or(Error::U32OverflowError)?;
        self.events.append_id(Some(id));
        self.events.append_parent_id(span_id);
        self.events
            .append_time_unix_nano(Some(current_time() as i64));
        self.events.append_name(name.as_bytes());
        self.events.append_dropped_attributes_count(Some(0));
        Ok(())
    }

    pub(super) fn finish(mut self) -> Result<OtapArrowRecords> {
        if self.count == 0 {
            return Ok(OtapArrowRecords::Traces(Traces::default()));
        }
        let n = self.count;
        append_single_resource_scope(&mut self.spans.resource, &mut self.spans.scope, n);
        self.spans.append_schema_url_n(None, n);

        let mut records = OtapArrowRecords::Traces(Traces::default());
        set_payloads(
            &mut records,
            [
                (self.spans.finish()?, ArrowPayloadType::Spans),
                (self.events.finish()?, ArrowPayloadType::SpanEvents),
                (self.span_attrs.finish()?, ArrowPayloadType::SpanAttrs),
                (
                    self.resource_attrs.finish()?,
                    ArrowPayloadType::ResourceAttrs,
                ),
            ],
        )?;
        Ok(records)
    }
}

/// Columns of one generated metric.
pub(super) struct MetricRow<'a> {
    pub(super) metric_type: MetricType,
    pub(super) name: &'a str,
    pub(super) description: &'a str,
    pub(super) unit: &'a str,
    pub(super) temporality: Option<AggregationTemporality>,
    pub(super) is_monotonic: Option<bool>,
}

/// A number data point (gauge or sum).
pub(super) struct NumberPoint {
    pub(super) start_time: Option<u64>,
    pub(super) time: u64,
    pub(super) value: f64,
}

/// An explicit-bucket histogram data point.
pub(super) struct HistogramPoint<'a> {
    pub(super) start_time: u64,
    pub(super) time: u64,
    pub(super) count: u64,
    pub(super) sum: Option<f64>,
    pub(super) bucket_counts: &'a [u64],
    pub(super) explicit_bounds: &'a [f64],
    pub(super) min: Option<f64>,
    pub(super) max: Option<f64>,
}

/// A base-2 exponential histogram data point with positive buckets only.
pub(super) struct ExpHistogramPoint<'a> {
    pub(super) start_time: u64,
    pub(super) time: u64,
    pub(super) count: u64,
    pub(super) sum: Option<f64>,
    pub(super) scale: i32,
    pub(super) zero_count: u64,
    pub(super) min: Option<f64>,
    pub(super) max: Option<f64>,
    pub(super) positive_offset: i32,
    pub(super) positive_counts: &'a [u64],
}

/// A summary data point. `quantiles` and `values` are parallel.
pub(super) struct SummaryPoint<'a> {
    pub(super) start_time: u64,
    pub(super) time: u64,
    pub(super) count: u64,
    pub(super) sum: f64,
    pub(super) quantiles: &'a [f64],
    pub(super) values: &'a [f64],
}

/// Builds a metrics batch under one resource and one scope.
///
/// Each `append_*_point` returns the data point id to use as the parent id
/// when appending to the matching `*_attrs` builder.
pub(super) struct MetricsBatch {
    metrics: MetricsRecordBatchBuilder,
    ndp: NumberDataPointsRecordBatchBuilder,
    hdp: HistogramDataPointsRecordBatchBuilder,
    ehdp: ExponentialHistogramDataPointsRecordBatchBuilder,
    sdp: SummaryDataPointsRecordBatchBuilder,
    /// Attributes of the single resource (parent id 0).
    pub(super) resource_attrs: AttributesRecordBatchBuilder<u16>,
    pub(super) ndp_attrs: AttributesRecordBatchBuilder<u32>,
    pub(super) hdp_attrs: AttributesRecordBatchBuilder<u32>,
    pub(super) ehdp_attrs: AttributesRecordBatchBuilder<u32>,
    pub(super) sdp_attrs: AttributesRecordBatchBuilder<u32>,
    count: usize,
    next_ndp_id: u32,
    next_hdp_id: u32,
    next_ehdp_id: u32,
    next_sdp_id: u32,
}

impl MetricsBatch {
    pub(super) fn new() -> Self {
        Self {
            metrics: MetricsRecordBatchBuilder::new(),
            ndp: NumberDataPointsRecordBatchBuilder::new(),
            hdp: HistogramDataPointsRecordBatchBuilder::new(),
            ehdp: ExponentialHistogramDataPointsRecordBatchBuilder::new(),
            sdp: SummaryDataPointsRecordBatchBuilder::new(),
            resource_attrs: AttributesRecordBatchBuilder::new(),
            ndp_attrs: AttributesRecordBatchBuilder::new(),
            hdp_attrs: AttributesRecordBatchBuilder::new(),
            ehdp_attrs: AttributesRecordBatchBuilder::new(),
            sdp_attrs: AttributesRecordBatchBuilder::new(),
            count: 0,
            next_ndp_id: 0,
            next_hdp_id: 0,
            next_ehdp_id: 0,
            next_sdp_id: 0,
        }
    }

    /// Append one metric and return its id for its data points.
    pub(super) fn append_metric(&mut self, row: MetricRow<'_>) -> Result<u16> {
        let id = next_u16_id(self.count)?;
        self.count += 1;

        self.metrics.append_id(id);
        self.metrics.append_metric_type(row.metric_type as u8);
        self.metrics.append_name(row.name.as_bytes());
        self.metrics.append_description(row.description.as_bytes());
        self.metrics.append_unit(row.unit.as_bytes());
        self.metrics
            .append_aggregation_temporality(row.temporality.map(|t| t as i32));
        self.metrics.append_is_monotonic(row.is_monotonic);
        Ok(id)
    }

    pub(super) fn append_number_point(&mut self, metric_id: u16, p: NumberPoint) -> Result<u32> {
        let id = next_u32_id(&mut self.next_ndp_id)?;
        self.ndp.append_id(id);
        self.ndp.append_parent_id(metric_id);
        self.ndp
            .append_start_time_unix_nano(p.start_time.map(|t| t as i64));
        self.ndp.append_time_unix_nano(p.time as i64);
        self.ndp.append_double_value(Some(p.value));
        self.ndp.append_int_value(None);
        self.ndp.append_flags(0);
        Ok(id)
    }

    pub(super) fn append_histogram_point(
        &mut self,
        metric_id: u16,
        p: HistogramPoint<'_>,
    ) -> Result<u32> {
        let id = next_u32_id(&mut self.next_hdp_id)?;
        self.hdp.append_id(id);
        self.hdp.append_parent_id(metric_id);
        self.hdp.append_start_time_unix_nano(p.start_time as i64);
        self.hdp.append_time_unix_nano(p.time as i64);
        self.hdp.append_count(p.count);
        self.hdp
            .append_bucket_counts(p.bucket_counts.iter().copied());
        self.hdp
            .append_explicit_bounds(p.explicit_bounds.iter().copied());
        self.hdp.append_sum(p.sum);
        self.hdp.append_flags(0);
        self.hdp.append_min(p.min);
        self.hdp.append_max(p.max);
        Ok(id)
    }

    pub(super) fn append_exp_histogram_point(
        &mut self,
        metric_id: u16,
        p: ExpHistogramPoint<'_>,
    ) -> Result<u32> {
        let id = next_u32_id(&mut self.next_ehdp_id)?;
        self.ehdp.append_id(id);
        self.ehdp.append_parent_id(metric_id);
        self.ehdp.append_start_time_unix_nano(p.start_time as i64);
        self.ehdp.append_time_unix_nano(p.time as i64);
        self.ehdp.append_count(p.count);
        self.ehdp.append_sum(p.sum);
        self.ehdp.append_scale(p.scale);
        self.ehdp.append_zero_count(p.zero_count);
        self.ehdp.append_flags(0);
        self.ehdp.append_min(p.min);
        self.ehdp.append_max(p.max);
        self.ehdp.append_zero_threshold(0.0);
        self.ehdp
            .positive
            .append(Some((p.positive_offset, p.positive_counts.iter().copied())));
        self.ehdp
            .negative
            .append(None::<(i32, std::iter::Empty<u64>)>);
        Ok(id)
    }

    pub(super) fn append_summary_point(
        &mut self,
        metric_id: u16,
        p: SummaryPoint<'_>,
    ) -> Result<u32> {
        debug_assert_eq!(p.quantiles.len(), p.values.len());
        let id = next_u32_id(&mut self.next_sdp_id)?;
        self.sdp.append_id(id);
        self.sdp.append_parent_id(metric_id);
        self.sdp.append_start_time_unix_nano(p.start_time as i64);
        self.sdp.append_time_unix_nano(p.time as i64);
        self.sdp.append_count(p.count);
        self.sdp.append_sum(p.sum);
        self.sdp.append_flags(0);
        self.sdp
            .quantile_values
            .append(p.quantiles.iter().copied().zip(p.values.iter().copied()));
        Ok(id)
    }

    pub(super) fn finish(mut self) -> Result<OtapArrowRecords> {
        if self.count == 0 {
            return Ok(OtapArrowRecords::Metrics(Metrics::default()));
        }
        let n = self.count;
        append_single_resource_scope(&mut self.metrics.resource, &mut self.metrics.scope, n);
        self.metrics.append_scope_schema_url_n(b"", n);

        let mut records = OtapArrowRecords::Metrics(Metrics::default());
        set_payloads(
            &mut records,
            [
                (self.metrics.finish()?, ArrowPayloadType::UnivariateMetrics),
                (
                    self.resource_attrs.finish()?,
                    ArrowPayloadType::ResourceAttrs,
                ),
                (self.ndp.finish()?, ArrowPayloadType::NumberDataPoints),
                (self.ndp_attrs.finish()?, ArrowPayloadType::NumberDpAttrs),
                (self.hdp.finish()?, ArrowPayloadType::HistogramDataPoints),
                (self.hdp_attrs.finish()?, ArrowPayloadType::HistogramDpAttrs),
                (
                    self.ehdp.finish()?,
                    ArrowPayloadType::ExpHistogramDataPoints,
                ),
                (
                    self.ehdp_attrs.finish()?,
                    ArrowPayloadType::ExpHistogramDpAttrs,
                ),
                (self.sdp.finish()?, ArrowPayloadType::SummaryDataPoints),
                (self.sdp_attrs.finish()?, ArrowPayloadType::SummaryDpAttrs),
            ],
        )?;
        Ok(records)
    }
}

/// A generated attribute value.
#[derive(Clone, Copy)]
pub(super) enum AttrValue<'a> {
    Str(&'a str),
    Int(i64),
}

/// Append one attribute row (parent id, key, value) to an attributes builder.
pub(super) fn append_attribute<T>(
    builder: &mut AttributesRecordBatchBuilder<T>,
    parent_id: <T::ArrayType as ArrowPrimitiveType>::Native,
    key: &str,
    value: AttrValue<'_>,
) where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    builder.append_parent_id(&parent_id);
    builder.append_key(key.as_bytes());
    match value {
        AttrValue::Str(s) => builder.any_values_builder.append_str(s.as_bytes()),
        AttrValue::Int(v) => builder.any_values_builder.append_int(v),
    }
}

/// Append one attribute of any value type, given as a prost `KeyValue`.
pub(super) fn append_key_value<T>(
    builder: &mut AttributesRecordBatchBuilder<T>,
    parent_id: <T::ArrayType as ArrowPrimitiveType>::Native,
    kv: &KeyValue,
) -> Result<()>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    builder.append_parent_id(&parent_id);
    let view = ObjKeyValue::new(&kv.key, kv.value.as_ref().map(|inner| GenericObj { inner }));
    append_attribute_value(builder, &view)
}

/// Fill the resource and scope columns of `n` rows that all belong to the
/// generator's single resource (id 0) and scope (id 0).
fn append_single_resource_scope(
    resource: &mut ResourceBuilder,
    scope: &mut ScopeBuilder,
    n: usize,
) {
    resource.append_id_n(0, n);
    resource.append_schema_url_n(None, n);
    resource.append_dropped_attributes_count_n(0, n);
    scope.append_id_n(0, n);
    scope.append_name_n(Some(SCOPE_NAME.as_bytes()), n);
    scope.append_version_n(Some(SCOPE_VERSION.as_bytes()), n);
    scope.append_dropped_attributes_count_n(0, n);
}

/// The id of the next row given `count` rows so far, if it fits in a u16.
fn next_u16_id(count: usize) -> Result<u16> {
    u16::try_from(count).map_err(|_| Error::U16OverflowError)
}

/// Take the next u32 id from `next`.
fn next_u32_id(next: &mut u32) -> Result<u32> {
    let id = *next;
    *next = id.checked_add(1).ok_or(Error::U32OverflowError)?;
    Ok(id)
}

/// Set every non-empty record batch into the OTAP batch.
fn set_payloads<const N: usize>(
    records: &mut OtapArrowRecords,
    payloads: [(RecordBatch, ArrowPayloadType); N],
) -> Result<()> {
    for (rb, payload_type) in payloads {
        if rb.num_rows() > 0 {
            records.set(payload_type, rb)?;
        }
    }
    Ok(())
}
