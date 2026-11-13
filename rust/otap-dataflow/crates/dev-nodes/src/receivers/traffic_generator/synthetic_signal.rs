// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Synthetic hardcoded signal generators for lightweight load testing.
//!
//! These generators produce minimal signals without requiring the semantic
//! conventions registry, making them ideal for high-throughput load testing
//! where startup time and per-signal CPU cost matter.
//!
//! Signals are built directly as OTAP record batches and then encoded to OTLP
//! bytes with the OTAP->OTLP encoder (see [`OtlpEncoder`]). This gives the
//! generated OTLP payloads the same wire layout that the engine itself
//! produces, instead of the layout of the prost encoder.

use crate::receivers::traffic_generator::fake_data::{
    SCOPE_NAME, SCOPE_VERSION, current_time, delay, gen_span_id_bytes, gen_trace_id_bytes,
};
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::arrow::array::RecordBatch;
use otel_arrow_dfe_pdata::arrow::datatypes::ArrowPrimitiveType;
use otel_arrow_dfe_pdata::encode::record::attributes::{
    AttributesRecordBatchBuilder, AttributesRecordBatchBuilderConstructorHelper,
};
use otel_arrow_dfe_pdata::encode::record::logs::LogsRecordBatchBuilder;
use otel_arrow_dfe_pdata::encode::record::metrics::{
    ExponentialHistogramDataPointsRecordBatchBuilder, HistogramDataPointsRecordBatchBuilder,
    MetricsRecordBatchBuilder, NumberDataPointsRecordBatchBuilder,
    SummaryDataPointsRecordBatchBuilder,
};
use otel_arrow_dfe_pdata::encode::record::traces::TracesRecordBatchBuilder;
use otel_arrow_dfe_pdata::encode::{Error, Result};
use otel_arrow_dfe_pdata::otap::{Logs, Metrics, OtapArrowRecords, Traces};
use otel_arrow_dfe_pdata::otlp::attributes::parent_id::ParentId;
use otel_arrow_dfe_pdata::otlp::logs::LogsProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::metrics::{MetricType, MetricsProtoBytesEncoder};
use otel_arrow_dfe_pdata::otlp::traces::TracesProtoBytesEncoder;
use otel_arrow_dfe_pdata::otlp::{ProtoBuffer, ProtoBytesEncoder};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::SeverityNumber;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::AggregationTemporality;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::span::SpanKind;
use std::borrow::Cow;
use std::collections::HashMap;

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

/// Generates an OTAP traces batch with `signal_count` synthetic HTTP server spans.
pub fn static_otap_traces(
    signal_count: usize,
    extra_attrs: Option<&HashMap<String, String>>,
) -> Result<OtapArrowRecords> {
    if signal_count == 0 {
        return Ok(OtapArrowRecords::Traces(Traces::default()));
    }
    let span_count = u16::try_from(signal_count).map_err(|_| Error::U16OverflowError)?;

    let mut resource_attrs = AttributesRecordBatchBuilder::<u16>::new();
    append_resource_attributes(&mut resource_attrs, extra_attrs);

    let mut spans = TracesRecordBatchBuilder::new();
    let mut span_attrs = AttributesRecordBatchBuilder::<u16>::new();

    spans.resource.append_id_n(0, signal_count);
    spans.resource.append_schema_url_n(None, signal_count);
    spans
        .resource
        .append_dropped_attributes_count_n(0, signal_count);
    spans.scope.append_id_n(0, signal_count);
    spans
        .scope
        .append_name_n(Some(SCOPE_NAME.as_bytes()), signal_count);
    spans
        .scope
        .append_version_n(Some(SCOPE_VERSION.as_bytes()), signal_count);
    spans
        .scope
        .append_dropped_attributes_count_n(0, signal_count);
    spans.append_schema_url_n(None, signal_count);

    for span_id in 0..span_count {
        let start_time = current_time();
        let end_time = start_time + delay();

        spans.append_id(Some(span_id));
        spans.append_start_time_unix_nano(start_time as i64);
        spans.append_duration_time_unix_nano(end_time as i64 - start_time as i64);
        spans.append_trace_id(gen_trace_id_bytes())?;
        spans.append_span_id(gen_span_id_bytes())?;
        spans.append_trace_state(None);
        spans.append_parent_span_id(None)?;
        spans.append_flags(None);
        spans.append_name(b"HTTP GET");
        spans.append_kind(Some(SpanKind::Server as i32));
        spans.append_dropped_attributes_count(Some(0));
        spans.append_dropped_events_count(Some(0));
        spans.append_dropped_links_count(Some(0));
        spans.status.append_code(None);
        spans.status.append_status_message(None);

        append_span_attributes(&mut span_attrs, span_id);
    }

    let mut records = OtapArrowRecords::Traces(Traces::default());
    set_payloads(
        &mut records,
        [
            (spans.finish()?, ArrowPayloadType::Spans),
            (span_attrs.finish()?, ArrowPayloadType::SpanAttrs),
            (resource_attrs.finish()?, ArrowPayloadType::ResourceAttrs),
        ],
    )?;
    Ok(records)
}

/// Generates an OTAP logs batch with configurable body size, attribute count,
/// trace context, and optional extra resource attributes.
///
/// - `log_body_size_bytes`: When `Some(n)`, generates a log body of approximately `n` bytes.
///   When `None`, cycles through the default body templates.
/// - `num_log_attributes`: When `Some(n)`, generates `n` attributes per record.
///   When `None`, uses the default 2 attributes (thread.id, thread.name).
/// - `use_trace_context`: When true, populates random trace_id/span_id on every record.
/// - `extra_attrs`: Optional extra key-value pairs merged into the resource attributes.
pub fn static_otap_logs(
    signal_count: usize,
    log_body_size_bytes: Option<usize>,
    num_log_attributes: Option<usize>,
    use_trace_context: bool,
    extra_attrs: Option<&HashMap<String, String>>,
) -> Result<OtapArrowRecords> {
    if signal_count == 0 {
        return Ok(OtapArrowRecords::Logs(Logs::default()));
    }
    let log_count = u16::try_from(signal_count).map_err(|_| Error::U16OverflowError)?;

    let body_pool = build_body_pool(log_body_size_bytes);
    let attr_count = num_log_attributes.unwrap_or(DEFAULT_LOG_ATTRIBUTE_COUNT);
    let attr_names = attribute_names(ATTR_NAMES, "attr", attr_count);

    let mut resource_attrs = AttributesRecordBatchBuilder::<u16>::new();
    append_resource_attributes(&mut resource_attrs, extra_attrs);

    let mut logs = LogsRecordBatchBuilder::new();
    let mut log_attrs = AttributesRecordBatchBuilder::<u16>::new();

    logs.resource.append_id_n(0, signal_count);
    logs.resource.append_schema_url_n(None, signal_count);
    logs.resource
        .append_dropped_attributes_count_n(0, signal_count);
    logs.scope.append_id_n(0, signal_count);
    logs.scope
        .append_name_n(Some(SCOPE_NAME.as_bytes()), signal_count);
    logs.scope
        .append_version_n(Some(SCOPE_VERSION.as_bytes()), signal_count);
    logs.scope
        .append_dropped_attributes_count_n(0, signal_count);
    logs.append_schema_url_n(None, signal_count);
    logs.append_dropped_attributes_count_n(0, signal_count);
    logs.append_flags_n(None, signal_count);

    for log_id in 0..log_count {
        let record_index = log_id as usize;
        let timestamp = current_time() as i64;
        let (severity_number, severity_text) = log_severity(record_index);

        logs.append_time_unix_nano(timestamp);
        logs.append_observed_time_unix_nano(timestamp);
        logs.append_severity_number(Some(severity_number as i32));
        logs.append_severity_text(Some(severity_text.as_bytes()));
        logs.append_event_name(None);

        if use_trace_context {
            logs.append_trace_id(Some(&gen_trace_id_bytes()))?;
            logs.append_span_id(Some(&gen_span_id_bytes()))?;
        } else {
            logs.append_trace_id(None)?;
            logs.append_span_id(None)?;
        }

        if body_pool.is_empty() {
            logs.body.append_null();
        } else {
            logs.body
                .append_str(body_pool[record_index % body_pool.len()].as_bytes());
        }

        // Log ids only exist to link attributes, so records without
        // attributes get a null id.
        if attr_names.is_empty() {
            logs.append_id(None);
            continue;
        }

        logs.append_id(Some(log_id));
        for (i, name) in attr_names.iter().enumerate() {
            append_attribute(
                &mut log_attrs,
                log_id,
                name,
                log_attribute_value(i, record_index),
            );
        }
    }

    let mut records = OtapArrowRecords::Logs(Logs::default());
    set_payloads(
        &mut records,
        [
            (logs.finish()?, ArrowPayloadType::Logs),
            (log_attrs.finish()?, ArrowPayloadType::LogAttrs),
            (resource_attrs.finish()?, ArrowPayloadType::ResourceAttrs),
        ],
    )?;
    Ok(records)
}

/// Generates an OTAP metrics batch covering all five OTLP metric types with
/// configurable attribute count, data point count, and optional extra
/// resource attributes.
///
/// - `num_metric_attributes`: When `Some(n)`, generates `n` attributes per data point.
///   When `None`, uses the default 3 attributes (http.method, http.route, http.status_code).
/// - `num_data_points`: When `Some(n)`, generates `n` data points per metric.
///   When `None`, uses the default of 1 data point per metric.
/// - `extra_attrs`: Optional extra key-value pairs merged into the resource attributes.
///
/// Metrics cycle through [`METRIC_DEFS`], which includes Counters,
/// UpDownCounters, Gauges, Histograms, ExponentialHistograms, and Summaries.
pub fn static_otap_metrics(
    signal_count: usize,
    num_metric_attributes: Option<usize>,
    num_data_points: Option<usize>,
    extra_attrs: Option<&HashMap<String, String>>,
) -> Result<OtapArrowRecords> {
    if signal_count == 0 {
        return Ok(OtapArrowRecords::Metrics(Metrics::default()));
    }
    let metric_count = u16::try_from(signal_count).map_err(|_| Error::U16OverflowError)?;
    let attr_count = num_metric_attributes.unwrap_or(DEFAULT_METRIC_ATTRIBUTE_COUNT);
    let dp_count = num_data_points.unwrap_or(DEFAULT_DATA_POINTS_PER_METRIC);
    // Data point ids are u32 and are shared by all points of one type, so the
    // total across all metrics must fit.
    _ = signal_count
        .checked_mul(dp_count)
        .and_then(|total| u32::try_from(total).ok())
        .ok_or(Error::U32OverflowError)?;

    let mut b = MetricsBuilders::new(attr_count);
    append_resource_attributes(&mut b.resource_attrs, extra_attrs);

    b.metrics.resource.append_id_n(0, signal_count);
    b.metrics.resource.append_schema_url_n(None, signal_count);
    b.metrics
        .resource
        .append_dropped_attributes_count_n(0, signal_count);
    b.metrics.scope.append_id_n(0, signal_count);
    b.metrics
        .scope
        .append_name_n(Some(SCOPE_NAME.as_bytes()), signal_count);
    b.metrics
        .scope
        .append_version_n(Some(SCOPE_VERSION.as_bytes()), signal_count);
    b.metrics
        .scope
        .append_dropped_attributes_count_n(0, signal_count);
    b.metrics.append_scope_schema_url_n(b"", signal_count);

    for metric_id in 0..metric_count {
        let metric_index = metric_id as usize;
        let def = &METRIC_DEFS[metric_index % METRIC_DEFS.len()];
        b.append_metric(metric_id, def, dp_count);
    }

    b.finish()
}

/// Static resource attributes, with optional user-supplied extras merged in.
fn append_resource_attributes(
    builder: &mut AttributesRecordBatchBuilder<u16>,
    extra: Option<&HashMap<String, String>>,
) {
    append_attribute(builder, 0, "service.name", AttrValue::Str("load-generator"));
    append_attribute(builder, 0, "service.version", AttrValue::Str("1.0.0"));
    append_attribute(
        builder,
        0,
        "service.instance.id",
        AttrValue::Str("instance-001"),
    );
    let Some(extra) = extra else {
        return;
    };
    for (k, v) in extra {
        append_attribute(builder, 0, k, AttrValue::Str(v));
    }
}

/// Static span attributes for HTTP server spans.
fn append_span_attributes(builder: &mut AttributesRecordBatchBuilder<u16>, span_id: u16) {
    append_attribute(builder, span_id, "http.method", AttrValue::Str("GET"));
    append_attribute(
        builder,
        span_id,
        "http.url",
        AttrValue::Str("http://example.com/api"),
    );
    append_attribute(builder, span_id, "http.status_code", AttrValue::Int(200));
    append_attribute(builder, span_id, "http.route", AttrValue::Str("/api"));
}

/// Severity rotation giving ~80% INFO, ~15% WARN, ~5% ERROR.
const fn log_severity(record_index: usize) -> (SeverityNumber, &'static str) {
    match record_index % 20 {
        0..=15 => (SeverityNumber::Info, "INFO"),
        16..=18 => (SeverityNumber::Warn, "WARN"),
        _ => (SeverityNumber::Error, "ERROR"),
    }
}

/// Value of the `i`-th log attribute for record `record_index`.
///
/// A few well-known names get realistic value shapes; everything else draws
/// from [`ATTR_VALUE_POOL`] using both the attribute and record index so that
/// attributes on the same record get different values.
fn log_attribute_value(i: usize, record_index: usize) -> AttrValue<'static> {
    match ATTR_NAMES.get(i) {
        Some(&"thread.id") => AttrValue::Int((record_index % 16) as i64),
        Some(&"thread.name") => AttrValue::Str(THREAD_NAMES[record_index % THREAD_NAMES.len()]),
        Some(&"code.lineno") | Some(&"server.port") | Some(&"client.port") => {
            AttrValue::Int((record_index % 500) as i64)
        }
        Some(&"http.response.status_code") => AttrValue::Int(STATUS_CODES[record_index % 10]),
        _ => {
            let pool_idx = (i.wrapping_mul(7) + record_index) % ATTR_VALUE_POOL.len();
            AttrValue::Str(ATTR_VALUE_POOL[pool_idx])
        }
    }
}

/// Value of the `i`-th metric attribute for data point `point_index`.
fn metric_attribute_value(i: usize, point_index: usize) -> AttrValue<'static> {
    match METRIC_ATTR_NAMES.get(i) {
        Some(&"http.method") => {
            AttrValue::Str(METRIC_ATTR_METHODS[point_index % METRIC_ATTR_METHODS.len()])
        }
        Some(&"http.route") => {
            AttrValue::Str(METRIC_ATTR_ROUTES[point_index % METRIC_ATTR_ROUTES.len()])
        }
        Some(&"http.status_code") | Some(&"http.response.status_code") => {
            AttrValue::Int(STATUS_CODES[point_index % 10])
        }
        Some(&"server.port") | Some(&"client.port") | Some(&"net.peer.port") => {
            AttrValue::Int(PORTS[point_index % PORTS.len()])
        }
        Some(&"server.address") | Some(&"host.name") | Some(&"net.peer.name") => {
            AttrValue::Str(METRIC_ATTR_HOSTS[point_index % METRIC_ATTR_HOSTS.len()])
        }
        _ => {
            let pool_idx = (i.wrapping_mul(7) + point_index) % ATTR_VALUE_POOL.len();
            AttrValue::Str(ATTR_VALUE_POOL[pool_idx])
        }
    }
}

/// Resolve the names of the first `count` attributes.
///
/// Names come from `pool`; if `count` exceeds the pool size, overflow names are
/// `{prefix}_N`. Resolved once per batch so overflow names are not formatted
/// for every record.
fn attribute_names(pool: &[&'static str], prefix: &str, count: usize) -> Vec<Cow<'static, str>> {
    let mut names = Vec::with_capacity(count);
    for i in 0..count {
        match pool.get(i) {
            Some(name) => names.push(Cow::Borrowed(*name)),
            None => names.push(Cow::Owned(format!("{prefix}_{i}"))),
        }
    }
    names
}

/// A generated attribute value.
#[derive(Clone, Copy)]
enum AttrValue<'a> {
    Str(&'a str),
    Int(i64),
}

/// Append one attribute row (parent id, key, value) to an attributes builder.
fn append_attribute<T>(
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

/// Set every non-empty record batch into the OTAP batch.
pub(super) fn set_payloads<const N: usize>(
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

/// Builders for every record batch that makes up a generated metrics batch.
struct MetricsBuilders {
    attr_names: Vec<Cow<'static, str>>,
    metrics: MetricsRecordBatchBuilder,
    resource_attrs: AttributesRecordBatchBuilder<u16>,
    ndp: NumberDataPointsRecordBatchBuilder,
    ndp_attrs: AttributesRecordBatchBuilder<u32>,
    hdp: HistogramDataPointsRecordBatchBuilder,
    hdp_attrs: AttributesRecordBatchBuilder<u32>,
    ehdp: ExponentialHistogramDataPointsRecordBatchBuilder,
    ehdp_attrs: AttributesRecordBatchBuilder<u32>,
    sdp: SummaryDataPointsRecordBatchBuilder,
    sdp_attrs: AttributesRecordBatchBuilder<u32>,
    next_ndp_id: u32,
    next_hdp_id: u32,
    next_ehdp_id: u32,
    next_sdp_id: u32,
}

impl MetricsBuilders {
    fn new(attr_count: usize) -> Self {
        Self {
            attr_names: attribute_names(METRIC_ATTR_NAMES, "metric_attr", attr_count),
            metrics: MetricsRecordBatchBuilder::new(),
            resource_attrs: AttributesRecordBatchBuilder::new(),
            ndp: NumberDataPointsRecordBatchBuilder::new(),
            ndp_attrs: AttributesRecordBatchBuilder::new(),
            hdp: HistogramDataPointsRecordBatchBuilder::new(),
            hdp_attrs: AttributesRecordBatchBuilder::new(),
            ehdp: ExponentialHistogramDataPointsRecordBatchBuilder::new(),
            ehdp_attrs: AttributesRecordBatchBuilder::new(),
            sdp: SummaryDataPointsRecordBatchBuilder::new(),
            sdp_attrs: AttributesRecordBatchBuilder::new(),
            next_ndp_id: 0,
            next_hdp_id: 0,
            next_ehdp_id: 0,
            next_sdp_id: 0,
        }
    }

    /// Append one metric row and its `dp_count` data points.
    fn append_metric(&mut self, metric_id: u16, def: &MetricDef, dp_count: usize) {
        let (metric_type, temporality, monotonic) = match def.kind {
            MetricKind::Counter => (
                MetricType::Sum,
                Some(AggregationTemporality::Cumulative),
                Some(true),
            ),
            MetricKind::UpDownCounter => (
                MetricType::Sum,
                Some(AggregationTemporality::Delta),
                Some(false),
            ),
            MetricKind::Gauge => (MetricType::Gauge, None, None),
            MetricKind::Histogram => (
                MetricType::Histogram,
                Some(AggregationTemporality::Delta),
                None,
            ),
            MetricKind::ExponentialHistogram => (
                MetricType::ExponentialHistogram,
                Some(AggregationTemporality::Delta),
                None,
            ),
            MetricKind::Summary => (MetricType::Summary, None, None),
        };

        self.metrics.append_id(metric_id);
        self.metrics.append_metric_type(metric_type as u8);
        self.metrics.append_name(def.name.as_bytes());
        self.metrics.append_description(def.description.as_bytes());
        self.metrics.append_unit(def.unit.as_bytes());
        self.metrics
            .append_aggregation_temporality(temporality.map(|t| t as i32));
        self.metrics.append_is_monotonic(monotonic);

        let metric_index = metric_id as usize;
        for j in 0..dp_count {
            let point_idx = metric_index.wrapping_mul(31) + j;
            match def.kind {
                MetricKind::Counter => self.append_number_point(metric_id, point_idx, j, true),
                MetricKind::UpDownCounter | MetricKind::Gauge => {
                    self.append_number_point(metric_id, point_idx, j, false)
                }
                MetricKind::Histogram => self.append_histogram_point(metric_id, point_idx),
                MetricKind::ExponentialHistogram => {
                    self.append_exp_histogram_point(metric_id, point_idx)
                }
                MetricKind::Summary => self.append_summary_point(metric_id, point_idx),
            }
        }
    }

    /// Append a NumberDataPoint with a varying value.
    fn append_number_point(&mut self, metric_id: u16, point_idx: usize, j: usize, monotonic: bool) {
        let timestamp = current_time();
        let start_time = timestamp - delay();
        let value = if monotonic {
            // Monotonic counter: increasing values
            ((point_idx + 1) * 42) as f64 + (j as f64 * 1.5)
        } else {
            // Gauge/up-down: varied values centered around typical ranges
            NUMBER_BASE_VALUES[point_idx % NUMBER_BASE_VALUES.len()]
        };

        let id = self.next_ndp_id;
        self.next_ndp_id += 1;
        self.ndp.append_id(id);
        self.ndp.append_parent_id(metric_id);
        self.ndp
            .append_start_time_unix_nano(Some(start_time as i64));
        self.ndp.append_time_unix_nano(timestamp as i64);
        self.ndp.append_double_value(Some(value));
        self.ndp.append_int_value(None);
        self.ndp.append_flags(0);
        append_metric_attributes(&mut self.ndp_attrs, &self.attr_names, id, point_idx);
    }

    /// Append a HistogramDataPoint with a realistic bucket distribution.
    fn append_histogram_point(&mut self, metric_id: u16, point_idx: usize) {
        let timestamp = current_time();
        let start_time = timestamp - delay();
        let dist = HISTOGRAM_DISTRIBUTIONS[point_idx % HISTOGRAM_DISTRIBUTIONS.len()];
        let total_count: u64 = dist.iter().sum();
        let sum: f64 = dist
            .iter()
            .zip(HISTOGRAM_MIDPOINTS.iter())
            .map(|(&c, &m)| c as f64 * m)
            .sum();
        // Derive min/max from the first and last non-zero bucket midpoints
        let min_val = dist
            .iter()
            .zip(HISTOGRAM_MIDPOINTS.iter())
            .find(|(c, _)| **c > 0)
            .map_or(0.0, |(_, &m)| m);
        let max_val = dist
            .iter()
            .zip(HISTOGRAM_MIDPOINTS.iter())
            .rev()
            .find(|(c, _)| **c > 0)
            .map_or(0.0, |(_, &m)| m);

        let id = self.next_hdp_id;
        self.next_hdp_id += 1;
        self.hdp.append_id(id);
        self.hdp.append_parent_id(metric_id);
        self.hdp.append_start_time_unix_nano(start_time as i64);
        self.hdp.append_time_unix_nano(timestamp as i64);
        self.hdp.append_count(total_count);
        self.hdp.append_bucket_counts(dist.iter().copied());
        self.hdp
            .append_explicit_bounds(HISTOGRAM_BOUNDS.iter().copied());
        self.hdp.append_sum(Some(sum));
        self.hdp.append_flags(0);
        self.hdp.append_min(Some(min_val));
        self.hdp.append_max(Some(max_val));
        append_metric_attributes(&mut self.hdp_attrs, &self.attr_names, id, point_idx);
    }

    /// Append an ExponentialHistogramDataPoint with varying positive buckets.
    fn append_exp_histogram_point(&mut self, metric_id: u16, point_idx: usize) {
        let timestamp = current_time();
        let start_time = timestamp - delay();
        let (offset, counts) = EXP_HISTOGRAM_BUCKETS[point_idx % EXP_HISTOGRAM_BUCKETS.len()];
        let total_count: u64 = counts.iter().sum();
        let sum = total_count as f64 * 12.5;
        let zeros = 2u64;

        let id = self.next_ehdp_id;
        self.next_ehdp_id += 1;
        self.ehdp.append_id(id);
        self.ehdp.append_parent_id(metric_id);
        self.ehdp.append_start_time_unix_nano(start_time as i64);
        self.ehdp.append_time_unix_nano(timestamp as i64);
        self.ehdp.append_count(total_count + zeros);
        self.ehdp.append_sum(Some(sum));
        self.ehdp.append_scale(4);
        self.ehdp.append_zero_count(zeros);
        self.ehdp.append_flags(0);
        self.ehdp.append_min(Some(0.1));
        self.ehdp.append_max(Some(sum / total_count as f64 * 3.0));
        self.ehdp.append_zero_threshold(0.0);
        self.ehdp
            .positive
            .append(Some((offset, counts.iter().copied())));
        self.ehdp
            .negative
            .append(None::<(i32, std::iter::Empty<u64>)>);
        append_metric_attributes(&mut self.ehdp_attrs, &self.attr_names, id, point_idx);
    }

    /// Append a SummaryDataPoint with realistic quantile values.
    fn append_summary_point(&mut self, metric_id: u16, point_idx: usize) {
        let timestamp = current_time();
        let start_time = timestamp - delay();
        let profile = QUANTILE_PROFILES[point_idx % QUANTILE_PROFILES.len()];
        let total_count = ((point_idx + 1) * 1000) as u64;
        let sum = profile[3] * total_count as f64 * 0.8;

        let id = self.next_sdp_id;
        self.next_sdp_id += 1;
        self.sdp.append_id(id);
        self.sdp.append_parent_id(metric_id);
        self.sdp.append_start_time_unix_nano(start_time as i64);
        self.sdp.append_time_unix_nano(timestamp as i64);
        self.sdp.append_count(total_count);
        self.sdp.append_sum(sum);
        self.sdp.append_flags(0);
        self.sdp.quantile_values.append(
            SUMMARY_QUANTILES
                .iter()
                .copied()
                .zip(profile.iter().copied()),
        );
        append_metric_attributes(&mut self.sdp_attrs, &self.attr_names, id, point_idx);
    }

    /// Finish all builders into an OTAP metrics batch.
    fn finish(mut self) -> Result<OtapArrowRecords> {
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

/// Append the attributes for one metric data point.
fn append_metric_attributes(
    builder: &mut AttributesRecordBatchBuilder<u32>,
    names: &[Cow<'static, str>],
    data_point_id: u32,
    point_idx: usize,
) {
    for (i, name) in names.iter().enumerate() {
        append_attribute(
            builder,
            data_point_id,
            name,
            metric_attribute_value(i, point_idx),
        );
    }
}

// ---------------------------------------------------------------------------
// Data pools. Shared with the previous prost implementation in
// `synthetic_signal_old.rs`.
// ---------------------------------------------------------------------------

/// HTTP status codes, weighted towards success.
const STATUS_CODES: [i64; 10] = [200, 200, 200, 201, 204, 301, 400, 403, 404, 500];

/// Common server ports.
const PORTS: [i64; 6] = [8080, 8443, 3000, 4317, 9090, 5432];

/// Gauge/up-down counter values centered around typical ranges.
const NUMBER_BASE_VALUES: [f64; 8] = [0.73, 1.2, 45.8, 128.0, 0.95, 2048.0, 0.12, 89.5];

/// Pre-computed histogram bucket count distributions (16 buckets = 15 bounds +
/// 1 overflow) representing different latency profiles.
/// Bounds: [0, 5, 10, 25, 50, 75, 100, 250, 500, 750, 1000, 2500, 5000, 7500, 10000, +inf]
const HISTOGRAM_DISTRIBUTIONS: &[&[u64]] = &[
    // Mostly fast requests with tail latency
    &[10, 45, 120, 200, 180, 90, 40, 15, 8, 4, 2, 1, 0, 0, 0, 0],
    // Bimodal: fast + slow
    &[5, 20, 80, 150, 60, 20, 10, 5, 30, 80, 40, 10, 3, 1, 0, 0],
    // Uniform-ish spread
    &[8, 12, 18, 25, 30, 35, 32, 28, 22, 15, 10, 6, 3, 1, 1, 0],
    // Very fast (cache hits)
    &[200, 150, 50, 10, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    // Slow service
    &[0, 1, 3, 5, 10, 20, 45, 80, 120, 90, 50, 25, 12, 5, 2, 0],
];

/// Midpoints of each histogram bucket: [(-inf,0], (0,5], (5,10], ... (10000,+inf)]
const HISTOGRAM_MIDPOINTS: [f64; 16] = [
    0.0, 2.5, 7.5, 17.5, 37.5, 62.5, 87.5, 175.0, 375.0, 625.0, 875.0, 1750.0, 3750.0, 6250.0,
    8750.0, 15000.0,
];

/// Positive bucket (offset, counts) layouts for exponential histograms.
const EXP_HISTOGRAM_BUCKETS: [(i32, &[u64]); 4] = [
    (0, &[100, 80, 40, 15, 5, 2, 1]),
    (1, &[20, 50, 120, 80, 30, 10]),
    (-1, &[5, 15, 40, 90, 60, 25, 8, 3]),
    (0, &[30, 45, 60, 45, 30, 15, 5]),
];

/// Latency profiles for summary quantile values (aligned with [`SUMMARY_QUANTILES`]).
const QUANTILE_PROFILES: &[&[f64]] = &[
    // Fast service: p50=2ms, p99=50ms
    &[0.1, 0.8, 2.0, 5.0, 12.0, 25.0, 50.0, 120.0],
    // Medium service: p50=15ms, p99=500ms
    &[1.0, 5.0, 15.0, 40.0, 100.0, 200.0, 500.0, 1200.0],
    // Slow service: p50=100ms, p99=5000ms
    &[5.0, 30.0, 100.0, 300.0, 800.0, 1500.0, 5000.0, 12000.0],
    // Very fast (cache): p50=0.3ms, p99=5ms
    &[0.01, 0.1, 0.3, 0.8, 2.0, 3.5, 5.0, 15.0],
];

/// Default number of metric attributes when `num_metric_attributes` is not configured.
pub(super) const DEFAULT_METRIC_ATTRIBUTE_COUNT: usize = 3;

/// Default number of data points per metric when `num_data_points` is not configured.
pub(super) const DEFAULT_DATA_POINTS_PER_METRIC: usize = 1;

/// Metric attribute names drawn from common OTel semantic conventions for metrics.
pub(super) const METRIC_ATTR_NAMES: &[&str] = &[
    "http.method",
    "http.route",
    "http.status_code",
    "server.address",
    "server.port",
    "network.protocol.name",
    "network.protocol.version",
    "rpc.system",
    "rpc.service",
    "rpc.method",
    "db.system",
    "db.namespace",
    "db.operation.name",
    "messaging.system",
    "messaging.operation.type",
    "messaging.destination.name",
    "cloud.region",
    "cloud.availability_zone",
    "k8s.namespace.name",
    "k8s.deployment.name",
    "container.name",
    "process.runtime.name",
    "deployment.environment",
    "service.namespace",
    "host.name",
    "host.arch",
    "os.type",
    "net.peer.name",
];

/// Metric attribute value pools per attribute type for realistic cardinality.
pub(super) const METRIC_ATTR_ROUTES: &[&str] = &[
    "/api/v1/users",
    "/api/v2/orders",
    "/api/v1/products",
    "/api/v2/payments",
    "/api/v1/auth",
    "/api/v3/search",
    "/api/v1/notifications",
    "/internal/health",
];

pub(super) const METRIC_ATTR_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"];

pub(super) const METRIC_ATTR_HOSTS: &[&str] = &[
    "prod-us-east-1a.internal",
    "prod-us-west-2b.internal",
    "staging-eu-west-1.internal",
    "worker-pool-7f3a.compute",
    "api-gateway-01.internal",
    "cache-redis-03.internal",
    "db-primary-01.internal",
    "queue-broker-12.internal",
];

/// Realistic metric definitions covering all five OTLP metric types.
/// Each entry: (name, description, unit, MetricKind)
#[derive(Clone, Copy)]
pub(super) enum MetricKind {
    /// Monotonic cumulative sum (counter)
    Counter,
    /// Non-monotonic delta sum (up-down counter)
    UpDownCounter,
    /// Gauge
    Gauge,
    /// Explicit-bucket histogram
    Histogram,
    /// Base-2 exponential histogram
    ExponentialHistogram,
    /// Summary (quantile sketch)
    Summary,
}

pub(super) struct MetricDef {
    pub(super) name: &'static str,
    pub(super) description: &'static str,
    pub(super) unit: &'static str,
    pub(super) kind: MetricKind,
}

pub(super) const METRIC_DEFS: &[MetricDef] = &[
    // Counters
    MetricDef {
        name: "http.server.request.count",
        description: "Total number of HTTP server requests",
        unit: "{request}",
        kind: MetricKind::Counter,
    },
    MetricDef {
        name: "http.client.request.count",
        description: "Total number of HTTP client requests",
        unit: "{request}",
        kind: MetricKind::Counter,
    },
    MetricDef {
        name: "rpc.server.request.count",
        description: "Total number of RPC server requests",
        unit: "{request}",
        kind: MetricKind::Counter,
    },
    MetricDef {
        name: "db.client.operation.count",
        description: "Total number of database operations",
        unit: "{operation}",
        kind: MetricKind::Counter,
    },
    MetricDef {
        name: "messaging.publish.count",
        description: "Total messages published",
        unit: "{message}",
        kind: MetricKind::Counter,
    },
    MetricDef {
        name: "http.server.request.body.size",
        description: "Total bytes received in HTTP request bodies",
        unit: "By",
        kind: MetricKind::Counter,
    },
    // Up-down counters
    MetricDef {
        name: "http.server.active_requests",
        description: "Number of active HTTP server requests",
        unit: "{request}",
        kind: MetricKind::UpDownCounter,
    },
    MetricDef {
        name: "db.client.connection.count",
        description: "Number of active database connections",
        unit: "{connection}",
        kind: MetricKind::UpDownCounter,
    },
    // Gauges
    MetricDef {
        name: "process.cpu.utilization",
        description: "CPU utilization ratio",
        unit: "1",
        kind: MetricKind::Gauge,
    },
    MetricDef {
        name: "process.memory.usage",
        description: "Current memory usage",
        unit: "By",
        kind: MetricKind::Gauge,
    },
    MetricDef {
        name: "system.cpu.load_average.1m",
        description: "1-minute load average",
        unit: "1",
        kind: MetricKind::Gauge,
    },
    MetricDef {
        name: "runtime.gc.heap_size",
        description: "Current heap size",
        unit: "By",
        kind: MetricKind::Gauge,
    },
    // Histograms
    MetricDef {
        name: "http.server.request.duration",
        description: "Duration of HTTP server requests",
        unit: "ms",
        kind: MetricKind::Histogram,
    },
    MetricDef {
        name: "http.client.request.duration",
        description: "Duration of HTTP client requests",
        unit: "ms",
        kind: MetricKind::Histogram,
    },
    MetricDef {
        name: "rpc.server.duration",
        description: "Duration of RPC server calls",
        unit: "ms",
        kind: MetricKind::Histogram,
    },
    MetricDef {
        name: "db.client.operation.duration",
        description: "Duration of database operations",
        unit: "ms",
        kind: MetricKind::Histogram,
    },
    MetricDef {
        name: "http.server.response.body.size",
        description: "Size of HTTP response bodies",
        unit: "By",
        kind: MetricKind::Histogram,
    },
    // Exponential histograms
    MetricDef {
        name: "messaging.process.duration",
        description: "Duration of message processing",
        unit: "ms",
        kind: MetricKind::ExponentialHistogram,
    },
    MetricDef {
        name: "rpc.client.duration",
        description: "Duration of RPC client calls",
        unit: "ms",
        kind: MetricKind::ExponentialHistogram,
    },
    // Summaries
    MetricDef {
        name: "runtime.gc.pause_duration",
        description: "GC pause duration quantiles",
        unit: "ms",
        kind: MetricKind::Summary,
    },
    MetricDef {
        name: "http.server.request.duration.summary",
        description: "HTTP request duration quantiles",
        unit: "ms",
        kind: MetricKind::Summary,
    },
];

/// OTel SDK default explicit bucket histogram boundaries.
/// See <https://opentelemetry.io/docs/specs/otel/metrics/sdk/#explicit-bucket-histogram-aggregation>
pub(super) const HISTOGRAM_BOUNDS: &[f64] = &[
    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0, 7500.0,
    10000.0,
];

/// Standard quantiles for summary metrics.
pub(super) const SUMMARY_QUANTILES: &[f64] = &[0.0, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99, 1.0];

/// Pool of realistic log attribute names drawn from common OTel semantic conventions.
/// When generating attributes, names are picked from this pool in order.
/// If more attributes are requested than the pool contains, extra names are
/// generated as `attr_N` (where N is the overflow index).
pub(super) const ATTR_NAMES: &[&str] = &[
    "thread.id",
    "thread.name",
    "code.function",
    "code.namespace",
    "code.filepath",
    "code.lineno",
    "log.record.uid",
    "event.name",
    "exception.type",
    "exception.message",
    "exception.stacktrace",
    "user.id",
    "user.name",
    "user.email",
    "session.id",
    "http.request.method",
    "http.response.status_code",
    "http.route",
    "url.full",
    "url.path",
    "url.scheme",
    "server.address",
    "server.port",
    "client.address",
    "client.port",
    "network.protocol.name",
    "network.protocol.version",
    "network.transport",
    "db.system",
    "db.namespace",
    "db.operation.name",
    "db.query.text",
    "db.collection.name",
    "messaging.system",
    "messaging.operation.type",
    "messaging.destination.name",
    "messaging.message.id",
    "rpc.system",
    "rpc.service",
    "rpc.method",
    "enduser.id",
    "enduser.role",
    "enduser.scope",
    "cloud.provider",
    "cloud.region",
    "cloud.availability_zone",
    "cloud.account.id",
    "container.id",
    "container.name",
    "container.image.name",
    "container.image.tag",
    "k8s.pod.name",
    "k8s.namespace.name",
    "k8s.deployment.name",
    "k8s.node.name",
    "os.type",
    "os.version",
    "process.pid",
    "process.executable.name",
    "process.command_line",
    "process.runtime.name",
    "process.runtime.version",
    "deployment.environment",
    "service.namespace",
    "telemetry.sdk.name",
    "telemetry.sdk.version",
    "telemetry.sdk.language",
    "feature_flag.key",
    "feature_flag.variant",
    "gen_ai.system",
    "gen_ai.request.model",
    "gen_ai.response.model",
    "peer.service",
    "otel.status_code",
    "otel.status_description",
    "error.type",
    "host.name",
    "host.id",
    "host.arch",
    "net.peer.name",
    "net.peer.port",
];

/// Thread names to rotate through for realistic cardinality.
pub(super) const THREAD_NAMES: &[&str] = &[
    "main",
    "worker-1",
    "worker-2",
    "worker-3",
    "tokio-runtime-0",
    "tokio-runtime-1",
    "blocking-0",
    "blocking-1",
];

/// Realistic attribute value pools for generic string attributes.
/// Each pool entry is 20-120 chars, resembling real observability values like
/// paths, IDs, hostnames, URLs, error descriptions, and user agents.
pub(super) const ATTR_VALUE_POOL: &[&str] = &[
    // URL paths
    "/api/v2/orders/checkout/confirm",
    "/api/v1/users/profile/settings",
    "/api/v3/inventory/products/search?category=electronics&page=2&limit=50",
    "/api/v2/payments/process",
    "/api/v1/auth/oauth/callback",
    "/internal/health/readiness",
    "/graphql/query/GetUserOrders",
    "/api/v2/notifications/subscribe",
    "/api/v1/reports/export/csv",
    "/webhooks/stripe/payment-intent",
    // Full URLs
    "https://api.example.com/v2/orders/ord-48291?include=items,shipping",
    "https://auth.example.com/oauth2/authorize?client_id=web-app&response_type=code",
    "https://cdn.example.com/assets/js/checkout-bundle-v3.2.1.min.js",
    "https://hooks.partner.io/events/order.completed",
    "https://storage.googleapis.com/data-lake-prod/exports/2025/03/batch-0042.parquet",
    // Hostnames
    "prod-us-east-1a.internal.example.com",
    "staging-eu-west-2b.internal.example.com",
    "worker-pool-7f3a8b.compute.internal",
    "cache-redis-03.memstore.internal",
    "db-primary-01.postgres.internal",
    "queue-kafka-broker-12.messaging.internal",
    "collector.observability.svc.cluster.local",
    // UUIDs / IDs
    "550e8400-e29b-41d4-a716-446655440000",
    "7c9e6679-7425-40de-944b-e07fc1f90ae7",
    "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "a3bb189e-8bf9-3888-9912-ace4e6543002",
    "req-8c4a2f-b7e1-4d39-9a2c-f8e6d1c5b3a0",
    "sess-a7b3c9d2-e4f5-6789-abcd-0123456789ef",
    // Class / function names
    "org.example.checkout.PaymentProcessor",
    "com.example.auth.TokenValidator",
    "io.example.pipeline.DataTransformer",
    "net.example.grpc.StreamHandler",
    "io.opentelemetry.sdk.trace.SdkSpan",
    "com.example.inventory.StockManager",
    // File paths
    "/usr/src/app/services/payment/handler.rs",
    "/usr/src/app/middleware/auth/validate.rs",
    "/usr/src/app/workers/export/batch.rs",
    "/opt/app/lib/telemetry/exporter.py",
    // Status descriptions / error messages
    "OK",
    "Not Found",
    "Internal Server Error",
    "Bad Gateway",
    "Service Unavailable",
    "timeout waiting for response from upstream service after 30000ms",
    "connection reset by peer during TLS handshake after 30s idle",
    "certificate chain verification failed: self-signed certificate in chain",
    "rate limit exceeded for client tier=free, retry after 60s",
    "circuit breaker open for service=inventory-api, falling back to cached response",
    "retry exhausted after 5 attempts, last error: deadline exceeded on POST /api/v2/orders",
    "request body too large: 10485760 bytes exceeds maximum allowed 5242880 bytes",
    "invalid JSON in request body at line 42 column 15: expected ',' or '}' after object member",
    // User agents
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 Chrome/120.0.6099.144 Mobile Safari/537.36",
    "okhttp/4.12.0 MyApp/3.2.1 (Android 14; sdk=34)",
    "Mozilla/5.0 (Linux; x86_64) OTelCollector/0.92.0",
    // SQL / DB queries
    "SELECT o.id, o.status, o.total FROM orders o WHERE o.user_id = $1 AND o.created_at > $2 ORDER BY o.created_at DESC LIMIT 50",
    "INSERT INTO audit_log (actor, action, resource, timestamp) VALUES ($1, $2, $3, $4) RETURNING id",
    "UPDATE inventory SET quantity = quantity - $1 WHERE product_id = $2 AND warehouse = $3 AND quantity >= $1",
    // Cloud / infra identifiers
    "kubernetes.io/service-account-token",
    "arn:aws:s3:::data-lake-prod/exports/2025/03",
    "projects/my-project/locations/us-central1/clusters/prod-gke",
    // Content types / encodings
    "application/json; charset=utf-8",
    "application/x-protobuf; proto=opentelemetry.proto.collector.logs.v1",
    "gzip;q=1.0, zstd;q=0.9, deflate;q=0.5, identity;q=0.1",
    "TLSv1.3 with cipher TLS_AES_256_GCM_SHA384 (256/256 bits)",
    // Operation names
    "ProcessPayment",
    "ValidateToken",
    "ExportBatchData",
    "RefreshInventoryCache",
    "SendNotification",
    "ResolveGraphQLQuery",
    "HandleWebSocketFrame",
    "DrainMessageQueue",
];

/// Default number of log attributes when `num_log_attributes` is not configured.
pub(super) const DEFAULT_LOG_ATTRIBUTE_COUNT: usize = 2;

/// Default log body templates used when `log_body_size_bytes` is not configured.
/// Records cycle through these to produce realistic dictionary-friendly cardinality.
/// Each template is ~150 chars to provide meaningful entropy when padded to larger sizes.
pub(super) const DEFAULT_BODY_TEMPLATES: &[&str] = &[
    "Order processed successfully for customer account region=us-east-1 warehouse=primary fulfillment_id=ful-29847 shipment_method=express priority=high",
    "User authentication completed via SSO provider=okta domain=corp.example.com session_duration=3600s mfa_method=totp client_ip=10.42.0.17 org=engineering",
    "Payment transaction recorded amount=149.99 currency=USD gateway=stripe merchant_id=acct_9f8e7d processor=visa card_type=credit risk_score=low batch=daily",
    "Database query executed table=orders index=idx_created_at rows_scanned=12847 rows_returned=50 duration_ms=23 connection_pool=primary shard=us-west-2a",
    "Cache entry refreshed key=product_catalog:v2:electronics ttl=1800s size_bytes=45821 backend=redis cluster=cache-prod-03 eviction_policy=lru hit_rate=0.94",
    "Configuration reloaded source=consul/kv path=/app/settings/production version=47 checksum=a3f8b2c1 changed_keys=12 rollback_enabled=true env=production",
    "Health check passed service=payment-gateway endpoint=/healthz latency_ms=2 upstream_status=healthy dependencies=3/3 region=eu-west-1 pod=pay-gw-7f9a3b",
    "Request validation failed path=/api/v2/orders error=missing_field field=shipping_address request_id=req-8c4a2f client=mobile-ios version=3.2.1 code=422",
    "Session expired for user principal=user-38291 session_id=sess-a7b3c9d2 idle_timeout=1800s last_activity=2025-03-15T14:22:00Z provider=internal tenant=acme",
    "Background job completed job_class=DataExportWorker job_id=job-f2e8a4b7 queue=critical duration_s=47.3 records_processed=128450 retries=0 status=success",
    "Rate limit threshold reached client_id=cli-92847 endpoint=/api/search limit=1000/min current=1001 policy=sliding_window action=throttle tier=free quota=used",
    "Connection pool exhausted pool=postgres-primary max_connections=100 active=100 waiting=23 timeout_ms=5000 host=db-prod-01.internal region=us-east-1 alert=p2",
    "Retry attempt initiated operation=s3_upload attempt=3/5 backoff_ms=4000 error=timeout bucket=data-lake-prod key=exports/2025/03/batch-0042.parquet size=128mb",
    "Webhook delivery confirmed target=https://hooks.partner.io/events event_type=order.completed delivery_id=dlv-c8a2f4 response_status=200 latency_ms=340 sig=v1",
    "Data export finished format=parquet destination=s3://analytics-lake/daily rows=2847291 size_mb=1247 duration_s=183 compression=zstd partitions=24 schema=v3",
    "Index rebuild started table=user_events index=idx_timestamp_user strategy=concurrent estimated_rows=48000000 tablespace=fast_ssd scheduled=off_peak priority=low",
    "Notification dispatched channel=email template=order_confirmation recipient_hash=a8f2c4 provider=sendgrid message_id=msg-b3e7f1 campaign=transactional locale=en",
    "File upload processed bucket=user-uploads key=documents/invoice-2025-q1.pdf size_bytes=2847592 content_type=application/pdf scan_status=clean duration_ms=1240",
    "Scheduled task triggered cron=0_*/5_*_*_* task=metric_aggregation last_run=2025-03-15T14:15:00Z next_run=2025-03-15T14:20:00Z lock=acquired node=worker-03",
    "Audit log entry created actor=admin@example.com action=role.update resource=team/engineering change=added_member target_user=dev42 ip=10.0.1.42 approved=auto",
    "Token refresh succeeded grant_type=refresh_token client_id=mobile-app scope=openid+profile+email token_lifetime=3600s issuer=auth.example.com jti=tok-7f2a8b",
    "Inventory sync completed source=erp target=catalog items_synced=4829 items_skipped=12 items_failed=3 duration_s=28 warehouse=us-central mode=incremental delta=47",
    "Email verification sent recipient_hash=c4d8e2 template=verify_email provider=ses region=us-east-1 message_id=msg-a2f8b4 campaign=onboarding locale=en ttl=24h",
    "API key rotated service=payment-gateway key_prefix=pk_live new_version=7 old_version=6 rotation_policy=90d initiated_by=security-bot approved=auto env=production",
    "Backup snapshot created database=orders-prod snapshot_id=snap-2025-03-15-1400 size_gb=247 duration_s=1842 retention_days=30 storage=s3-glacier type=full verified=true",
    "SSL certificate renewed domain=api.example.com issuer=letsencrypt serial=0x3a8f2c valid_until=2025-06-15 key_type=ecdsa-p256 auto_renew=true dns_challenge=cloudflare",
    "Feature flag evaluated flag=new_checkout_flow user_segment=beta result=enabled variant=treatment_b sdk=server-rust version=2.1.0 context=web evaluation_ms=0.3",
    "Message queue drained queue=order-events consumer_group=processors messages=4829 duration_s=12.4 avg_latency_ms=2.6 dlq_count=0 partition=3 offset=892741",
    "Load balancer health ok upstream=api-cluster healthy=8/8 avg_latency_ms=12 p99_latency_ms=45 active_connections=2847 requests_per_sec=1240 protocol=h2 region=us",
    "Deployment rollout started service=checkout-api version=v2.4.7 strategy=canary initial_weight=5 target_replicas=12 cluster=prod-us-east namespace=commerce image=latest",
    "Circuit breaker tripped service=inventory-api failure_rate=0.62 threshold=0.50 state=open half_open_after=30s consecutive_failures=15 last_success=42s_ago fallback=cache",
    "Cache miss for lookup key=user:profile:u-48291 backend=redis cluster=cache-prod-02 ttl_remaining=0 origin_fetch=true origin_latency_ms=45 refill=async size_bytes=2847",
    "DNS resolution completed hostname=api.partner.example.com resolved=203.0.113.42 ttl=300s resolver=internal cache_hit=false lookup_ms=12 record_type=A nameserver=ns1",
    "Compression ratio logged codec=zstd level=3 input_bytes=1048576 output_bytes=142857 ratio=7.3 duration_us=2840 context=grpc_export stream_id=str-a8f2 batch_seq=47",
    "Garbage collection paused generation=2 pause_ms=23 heap_before_mb=2048 heap_after_mb=1247 freed_mb=801 objects_collected=2847291 cpu_time_ms=89 trigger=allocation",
    "Memory threshold exceeded process=data-pipeline current_mb=3847 limit_mb=4096 utilization=0.94 gc_pressure=high action=shed_load shed_percent=20 alert=warning",
    "Disk usage alert cleared mount=/data/warehouse usage_percent=72 previous_percent=91 freed_gb=847 cleanup_type=ttl_expiry retention_policy=30d filesystem=ext4 inode=ok",
    "Span context propagated trace_id=4bf92f3577b34da6a3ce929d0e0e4736 span_id=00f067aa0ba902b7 parent_id=a2fb4a1d1a96d312 sampled=true baggage=tenant:acme propagator=w3c",
    "Trace sampling decision made trace_id=7f3c8a2b rule=rate_limited sampler=probabilistic rate=0.01 decision=drop head_based=true evaluated_rules=3 matched=rate_global",
    "Metric aggregation flushed pipeline=prometheus metric_families=247 datapoints=48291 duration_ms=340 destination=remote_write endpoint=thanos flush_interval=15s errors=0",
    "Log buffer rotated buffer_id=stderr-capture-03 flushed_bytes=4194304 records=8472 destination=otlp-exporter compression=zstd pending_bytes=0 rotation_trigger=size_limit",
    "gRPC stream established service=otel.ArrowService method=ArrowStream peer=10.42.3.17:44892 compression=zstd auth=mtls tls_version=1.3 cipher=aes-256-gcm stream_id=42",
    "HTTP/2 connection opened peer=collector.observability.svc:4318 streams_max=100 window_size=65535 tls=true alpn=h2 keepalive_interval=30s idle_timeout=300s conn_id=c-847",
    "TLS handshake completed peer=10.42.0.89:48291 version=TLSv1.3 cipher_suite=TLS_AES_256_GCM_SHA384 certificate_cn=collector.prod sni=collector.prod duration_us=4200 resumed=false",
    "OAuth token exchanged grant=authorization_code provider=google scope=openid+email token_type=bearer expires_in=3600 client_id=web-app-prod jti=tok-c8a2f4 nonce=verified",
    "SAML assertion validated issuer=idp.corp.example.com subject=user@example.com audience=sp.example.com not_before=2025-03-15T14:00:00Z conditions=valid authn_context=password",
    "Cursor pagination advanced collection=events cursor=eyJ0cyI6MTcxMDUxMjAwMH0 page_size=100 returned=100 has_more=true direction=forward index=ts_desc total_estimate=284700",
    "GraphQL query resolved operation=GetUserOrders complexity=42 depth=4 fields=23 resolver_calls=8 duration_ms=67 cache_hits=5 cache_misses=3 auth=bearer user_id=u-48291",
    "WebSocket frame received connection_id=ws-a8f2b4 frame_type=text size_bytes=2847 channel=live-updates subscription=orders.status client=dashboard-v3 compress=permessage",
    "Batch export succeeded protocol=otlp/grpc destination=collector.prod:4317 spans=0 metrics=0 logs=500 bytes=612394 duration_ms=89 compression=zstd retries=0 stream=arrow",
];

/// Minimum number of bodies in the pool when `log_body_size_bytes` is set.
/// This ensures enough variety within a single batch (default batch_size=512)
/// to produce realistic compression ratios.
pub(super) const MIN_BODY_POOL_SIZE: usize = 512;

/// Build a pool of body strings for log generation.
///
/// When `size_bytes` is specified, generates a pool of at least
/// [`MIN_BODY_POOL_SIZE`] unique bodies. Each body is built by cycling through
/// *different* templates from [`DEFAULT_BODY_TEMPLATES`] (starting at a
/// different offset per entry) and prepending a deterministic unique prefix
/// (`seq=N`) so that no two pool entries are identical. This produces
/// realistic entropy that avoids inflating compression ratios.
///
/// When `None`, the templates are used as-is.
pub(super) fn build_body_pool(size_bytes: Option<usize>) -> Vec<String> {
    match size_bytes {
        Some(0) => Vec::new(),
        Some(n) => {
            let pool_size = MIN_BODY_POOL_SIZE.max(DEFAULT_BODY_TEMPLATES.len());
            (0..pool_size)
                .map(|pool_idx| {
                    let mut body = String::with_capacity(n);
                    // Deterministic unique prefix so no two bodies are identical.
                    let prefix = format!("seq={pool_idx:04} ");
                    body.push_str(&prefix);
                    // Fill by cycling through different templates starting at
                    // a unique offset.
                    let mut tmpl_idx = pool_idx;
                    while body.len() < n {
                        if body.len() > prefix.len() {
                            body.push(' ');
                        }
                        body.push_str(
                            DEFAULT_BODY_TEMPLATES[tmpl_idx % DEFAULT_BODY_TEMPLATES.len()],
                        );
                        tmpl_idx += 1;
                    }
                    body.truncate(n);
                    body
                })
                .collect()
        }
        None => DEFAULT_BODY_TEMPLATES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::synthetic_signal_old as old;
    use super::*;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{MetricsData, metric::Data};
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::TracesData;
    use prost::Message;

    fn encode(records: OtapArrowRecords) -> OtlpProtoBytes {
        OtlpEncoder::new().encode(records).expect("encode to OTLP")
    }

    fn decode_logs(records: OtapArrowRecords) -> LogsData {
        LogsData::decode(encode(records).as_bytes()).expect("decode logs")
    }

    fn decode_traces(records: OtapArrowRecords) -> TracesData {
        TracesData::decode(encode(records).as_bytes()).expect("decode traces")
    }

    fn decode_metrics(records: OtapArrowRecords) -> MetricsData {
        MetricsData::decode(encode(records).as_bytes()).expect("decode metrics")
    }

    /// Zero the fields that are nondeterministic between two generations
    /// (wall clock timestamps and random ids).
    fn normalize_logs(mut logs: LogsData) -> LogsData {
        for rl in &mut logs.resource_logs {
            for sl in &mut rl.scope_logs {
                for lr in &mut sl.log_records {
                    lr.time_unix_nano = 0;
                    lr.observed_time_unix_nano = 0;
                    if !lr.trace_id.is_empty() {
                        lr.trace_id = vec![1; 16];
                    }
                    if !lr.span_id.is_empty() {
                        lr.span_id = vec![1; 8];
                    }
                }
            }
        }
        logs
    }

    fn normalize_traces(mut traces: TracesData) -> TracesData {
        for rs in &mut traces.resource_spans {
            for ss in &mut rs.scope_spans {
                for span in &mut ss.spans {
                    // Keep the duration, which is deterministic.
                    span.end_time_unix_nano -= span.start_time_unix_nano;
                    span.start_time_unix_nano = 0;
                    span.trace_id = vec![1; 16];
                    span.span_id = vec![1; 8];
                    // OTAP has no null bitmap on the span status struct, so
                    // the OTAP->OTLP encoder writes an empty `Status` where
                    // prost omits it. Both mean `Unset`.
                    if span.status == Some(Default::default()) {
                        span.status = None;
                    }
                }
            }
        }
        traces
    }

    fn normalize_metrics(mut metrics: MetricsData) -> MetricsData {
        for rm in &mut metrics.resource_metrics {
            for sm in &mut rm.scope_metrics {
                for m in &mut sm.metrics {
                    match m.data.as_mut().expect("metric data") {
                        Data::Gauge(g) => g.data_points.iter_mut().for_each(|p| {
                            p.start_time_unix_nano = p.time_unix_nano - p.start_time_unix_nano;
                            p.time_unix_nano = 0;
                        }),
                        Data::Sum(s) => s.data_points.iter_mut().for_each(|p| {
                            p.start_time_unix_nano = p.time_unix_nano - p.start_time_unix_nano;
                            p.time_unix_nano = 0;
                        }),
                        Data::Histogram(h) => h.data_points.iter_mut().for_each(|p| {
                            p.start_time_unix_nano = p.time_unix_nano - p.start_time_unix_nano;
                            p.time_unix_nano = 0;
                        }),
                        Data::ExponentialHistogram(h) => h.data_points.iter_mut().for_each(|p| {
                            p.start_time_unix_nano = p.time_unix_nano - p.start_time_unix_nano;
                            p.time_unix_nano = 0;
                            // OTAP buckets have no null bitmap, so the
                            // OTAP->OTLP encoder writes empty `negative`
                            // buckets where prost omits them. Both are empty.
                            if p.negative == Some(Default::default()) {
                                p.negative = None;
                            }
                        }),
                        Data::Summary(s) => s.data_points.iter_mut().for_each(|p| {
                            p.start_time_unix_nano = p.time_unix_nano - p.start_time_unix_nano;
                            p.time_unix_nano = 0;
                        }),
                    }
                }
            }
        }
        metrics
    }

    fn extra_attrs() -> HashMap<String, String> {
        HashMap::from([
            ("tenant.id".to_string(), "prod".to_string()),
            ("deployment.environment".to_string(), "staging".to_string()),
        ])
    }

    #[test]
    fn logs_match_prost_implementation() {
        let extra = extra_attrs();
        // (count, body size, attribute count, trace context, extra resource attrs)
        let cases: [(
            usize,
            Option<usize>,
            Option<usize>,
            bool,
            Option<&HashMap<_, _>>,
        ); 8] = [
            (1, None, None, false, None),
            (100, None, None, false, None),
            (64, Some(1024), None, false, None),
            (10, Some(0), Some(0), false, None),
            (10, None, Some(0), false, None),
            (20, None, Some(90), false, None),
            (50, Some(150), Some(4), true, None),
            (30, None, Some(10), false, Some(&extra)),
        ];

        for (count, body, attrs, tc, extra) in cases {
            let expected = old::static_otlp_logs_with_config(count, body, attrs, tc, extra);
            let actual =
                decode_logs(static_otap_logs(count, body, attrs, tc, extra).expect("build logs"));
            assert_eq!(
                normalize_logs(actual),
                normalize_logs(expected),
                "count={count} body={body:?} attrs={attrs:?} tc={tc} extra={extra:?}"
            );
        }
    }

    #[test]
    fn logs_trace_context_is_random_per_record() {
        let logs = decode_logs(static_otap_logs(4, None, None, true, None).expect("build logs"));
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert!(records.iter().all(|r| r.trace_id.len() == 16));
        assert!(records.iter().all(|r| r.span_id.len() == 8));
        assert_ne!(records[0].trace_id, records[1].trace_id);
    }

    #[test]
    fn traces_match_prost_implementation() {
        let extra = extra_attrs();
        for (count, extra) in [(1, None), (100, None), (10, Some(&extra))] {
            let expected = old::static_otlp_traces(count, extra);
            let actual = decode_traces(static_otap_traces(count, extra).expect("build traces"));
            assert_eq!(
                normalize_traces(actual),
                normalize_traces(expected),
                "count={count} extra={extra:?}"
            );
        }
    }

    #[test]
    fn metrics_match_prost_implementation() {
        let extra = extra_attrs();
        // (count, attribute count, data points per metric, extra resource attrs)
        let cases: [(usize, Option<usize>, Option<usize>, Option<&HashMap<_, _>>); 6] = [
            (1, None, None, None),
            // Enough metrics to cycle through every METRIC_DEFS entry twice.
            (2 * METRIC_DEFS.len(), None, None, None),
            (2 * METRIC_DEFS.len(), Some(6), Some(3), None),
            (25, Some(0), Some(2), None),
            (25, Some(40), Some(1), None),
            (10, None, None, Some(&extra)),
        ];

        for (count, attrs, dps, extra) in cases {
            let expected = old::static_otlp_metrics_with_config(count, attrs, dps, extra);
            let actual = decode_metrics(
                static_otap_metrics(count, attrs, dps, extra).expect("build metrics"),
            );
            assert_eq!(
                normalize_metrics(actual),
                normalize_metrics(expected),
                "count={count} attrs={attrs:?} dps={dps:?} extra={extra:?}"
            );
        }
    }

    #[test]
    fn metrics_cover_all_types() {
        let metrics = decode_metrics(
            static_otap_metrics(METRIC_DEFS.len(), None, None, None).expect("build metrics"),
        );
        let list = &metrics.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(list.len(), METRIC_DEFS.len());
        let has = |f: fn(&Data) -> bool| list.iter().any(|m| m.data.as_ref().is_some_and(f));
        assert!(has(|d| matches!(d, Data::Sum(_))));
        assert!(has(|d| matches!(d, Data::Gauge(_))));
        assert!(has(|d| matches!(d, Data::Histogram(_))));
        assert!(has(|d| matches!(d, Data::ExponentialHistogram(_))));
        assert!(has(|d| matches!(d, Data::Summary(_))));
    }

    #[test]
    fn zero_count_produces_empty_batches() {
        for records in [
            static_otap_logs(0, None, None, false, None).expect("logs"),
            static_otap_traces(0, None).expect("traces"),
            static_otap_metrics(0, None, None, None).expect("metrics"),
        ] {
            assert_eq!(records.num_items(), 0);
        }
    }

    #[test]
    fn oversized_batches_are_rejected() {
        let too_many = u16::MAX as usize + 1;
        assert!(static_otap_logs(too_many, None, Some(0), false, None).is_err());
        assert!(static_otap_traces(too_many, None).is_err());
        assert!(static_otap_metrics(too_many, Some(0), None, None).is_err());
    }

    #[test]
    fn encoder_is_reusable_across_signals() {
        let mut encoder = OtlpEncoder::new();
        for _ in 0..3 {
            let logs = encoder
                .encode(static_otap_logs(8, None, None, false, None).expect("logs"))
                .expect("encode logs");
            assert!(matches!(logs, OtlpProtoBytes::ExportLogsRequest(_)));
            let traces = encoder
                .encode(static_otap_traces(8, None).expect("traces"))
                .expect("encode traces");
            assert!(matches!(traces, OtlpProtoBytes::ExportTracesRequest(_)));
            let metrics = encoder
                .encode(static_otap_metrics(8, None, None, None).expect("metrics"))
                .expect("encode metrics");
            assert!(matches!(metrics, OtlpProtoBytes::ExportMetricsRequest(_)));
        }
    }
}
