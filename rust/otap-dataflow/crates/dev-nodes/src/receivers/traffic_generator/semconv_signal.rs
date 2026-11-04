// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! The semconv signal module provides methods for generating OTLP signals based on
//! the OpenTelemetry Semantic Conventions registry.
//!
//! Values are driven by `traffic-generator` annotations on the registry groups
//! and attributes. The registry is compiled once into a [`SemconvModel`]
//! (annotations parsed, samplers built); generation only draws from the
//! compiled templates. Groups or attributes without annotations fall back to
//! enum members, examples, and type defaults.
//!
//! ToDo: Add profile signal support -> update the builder lib.rs to work on profile object
//!
//! [`SemconvModel`]: crate::receivers::traffic_generator::semconv_model::SemconvModel

use crate::receivers::traffic_generator::attributes::AttributeGen;
use crate::receivers::traffic_generator::fake_data::{
    current_time, delay, gen_span_id, gen_trace_id, get_scope_name, get_scope_version,
};
use crate::receivers::traffic_generator::semconv_model::{
    EventTemplate, MetricTemplate, SemconvModel, SpanTemplate,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
    metrics::v1::{
        AggregationTemporality, Gauge, Histogram, HistogramDataPoint, Metric, MetricsData,
        NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    },
    resource::v1::Resource,
    trace::v1::{ResourceSpans, ScopeSpans, Span, Status, TracesData, span::Event, span::SpanKind},
};
use rand::Rng;
use weaver_semconv::v1::group::{InstrumentSpec, SpanKindSpec};

/// Value emitted for metrics without a `value` annotation.
const DEFAULT_METRIC_VALUE: f64 = 1.0;

fn resource() -> Resource {
    Resource::build()
        .attributes(vec![KeyValue::new(
            "traffic_generator",
            AnyValue::new_string("v1"),
        )])
        .finish()
}

fn scope() -> InstrumentationScope {
    InstrumentationScope::build()
        .name(get_scope_name())
        .version(get_scope_version())
        .finish()
}

/// Generates TracesData with the specified resource/scope count and defined spans in the registry
#[must_use]
pub fn semconv_otlp_traces(signal_count: usize, model: &SemconvModel) -> TracesData {
    semconv_otlp_traces_with_rng(signal_count, model, &mut rand::rng())
}

/// Generates LogsData with the specified resource/scope count and defined events (structured logs) in the registry
#[must_use]
pub fn semconv_otlp_logs(signal_count: usize, model: &SemconvModel) -> LogsData {
    semconv_otlp_logs_with_rng(signal_count, model, &mut rand::rng())
}

/// Generates MetricsData with the specified resource/scope count and defined metrics in the registry
#[must_use]
pub fn semconv_otlp_metrics(signal_count: usize, model: &SemconvModel) -> MetricsData {
    semconv_otlp_metrics_with_rng(signal_count, model, &mut rand::rng())
}

/// Like [`semconv_otlp_traces`], drawing random values from `rng`.
pub fn semconv_otlp_traces_with_rng<R: Rng + ?Sized>(
    signal_count: usize,
    model: &SemconvModel,
    rng: &mut R,
) -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        resource(),
        vec![ScopeSpans::new(scope(), spans(signal_count, model, rng))],
    )])
}

/// Like [`semconv_otlp_logs`], drawing random values from `rng`.
pub fn semconv_otlp_logs_with_rng<R: Rng + ?Sized>(
    signal_count: usize,
    model: &SemconvModel,
    rng: &mut R,
) -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        resource(),
        vec![ScopeLogs::new(scope(), logs(signal_count, model, rng))],
    )])
}

/// Like [`semconv_otlp_metrics`], drawing random values from `rng`.
pub fn semconv_otlp_metrics_with_rng<R: Rng + ?Sized>(
    signal_count: usize,
    model: &SemconvModel,
    rng: &mut R,
) -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        resource(),
        vec![ScopeMetrics::new(
            scope(),
            metrics(signal_count, model, rng),
        )],
    )])
}

fn attributes<R: Rng + ?Sized>(attributes: &[AttributeGen], rng: &mut R) -> Vec<KeyValue> {
    attributes.iter().map(|a| a.sample(rng)).collect()
}

// ---------------------------------------------------------------------------
// Signal-level sampling
// ---------------------------------------------------------------------------

/// Span duration in nanoseconds (annotation `duration` is in milliseconds).
fn span_duration_ns<R: Rng + ?Sized>(span: &SpanTemplate, rng: &mut R) -> u64 {
    match &span.duration_ms {
        Some(d) => (d.sample_f64(rng) * 1_000_000.0).round().max(0.0) as u64,
        None => delay(),
    }
}

/// Span status from the `status` annotation, or `None` when absent.
fn span_status<R: Rng + ?Sized>(span: &SpanTemplate, rng: &mut R) -> Option<Status> {
    span.status
        .as_ref()
        .map(|s| Status::new(*s.sample(rng), ""))
}

/// Log severity number and text from the `severity` annotation (INFO with no
/// text when absent).
fn log_severity<R: Rng + ?Sized>(event: &EventTemplate, rng: &mut R) -> (i32, &'static str) {
    match &event.severity {
        Some(s) => *s.sample(rng),
        None => (SeverityNumber::Info as i32, ""),
    }
}

/// Metric data point value from the `value` annotation (1.0 when absent).
fn metric_value<R: Rng + ?Sized>(metric: &MetricTemplate, rng: &mut R) -> f64 {
    metric
        .value
        .as_ref()
        .map_or(DEFAULT_METRIC_VALUE, |v| v.sample_f64(rng))
}

/// Number of data points per metric from the `data_points` annotation (1 when absent).
fn data_point_count<R: Rng + ?Sized>(metric: &MetricTemplate, rng: &mut R) -> usize {
    metric
        .data_points
        .as_ref()
        .map_or(1, |d| d.sample_count(rng).max(1))
}

/// Map a severity short name to its SeverityNumber value (1..=24).
#[must_use]
pub fn severity_number(name: &str) -> Option<i32> {
    crate::receivers::traffic_generator::semconv_model::severity_entry(name).map(|(n, _)| n)
}

/// Map a status name to its StatusCode value.
#[must_use]
pub fn status_code(name: &str) -> Option<i32> {
    crate::receivers::traffic_generator::semconv_model::status_entry(name)
}

// ---------------------------------------------------------------------------
// Signals
// ---------------------------------------------------------------------------

/// Output capacity: `signal_count`, or 0 when there are no groups to cycle.
const fn capacity(signal_count: usize, groups: usize) -> usize {
    if groups == 0 { 0 } else { signal_count }
}

/// generate each span defined in the resolved registry
fn spans<R: Rng + ?Sized>(signal_count: usize, model: &SemconvModel, rng: &mut R) -> Vec<Span> {
    let mut spans = Vec::with_capacity(capacity(signal_count, model.spans.len()));
    for span in model.spans.iter().cycle().take(signal_count) {
        let start_time = current_time();
        let end_time = start_time + span_duration_ns(span, rng);
        let builder = Span::build()
            .trace_id(gen_trace_id())
            .span_id(gen_span_id())
            .name(span.name.clone())
            .start_time_unix_nano(start_time)
            .attributes(attributes(&span.attributes, rng))
            .events(
                span.events
                    .iter()
                    .map(|event_name| {
                        Event::build()
                            .name(event_name.clone())
                            .time_unix_nano(current_time())
                            .finish()
                    })
                    .collect::<Vec<_>>(),
            )
            .kind(otel_span_kind(span.kind.as_ref()))
            .end_time_unix_nano(end_time);
        spans.push(match span_status(span, rng) {
            Some(status) => builder.status(status).finish(),
            None => builder.finish(),
        });
    }
    spans
}

/// generate each metric defined in the resolved registry
fn metrics<R: Rng + ?Sized>(signal_count: usize, model: &SemconvModel, rng: &mut R) -> Vec<Metric> {
    let mut metrics = Vec::with_capacity(capacity(signal_count, model.metrics.len()));
    // Metric groups without an instrument occupy a slot and are skipped.
    for metric in model.metrics.iter().cycle().take(signal_count).flatten() {
        let point_count = data_point_count(metric, rng);
        let now = current_time();
        let builder = Metric::build()
            .name(metric.name.clone())
            .description(metric.description.clone())
            .unit(metric.unit.clone());
        let built = match metric.instrument {
            InstrumentSpec::Histogram => {
                let points = (0..point_count)
                    .map(|_| histogram_point(metric, now, rng))
                    .collect::<Vec<_>>();
                builder
                    .data_histogram(Histogram::new(AggregationTemporality::Delta, points))
                    .finish()
            }
            InstrumentSpec::Gauge => builder
                .data_gauge(Gauge::new(number_points(metric, point_count, now, rng)))
                .finish(),
            // Counters are per-interval increments with delta temporality, so
            // each sample is independent and replayed batches stay valid.
            InstrumentSpec::Counter => builder
                .data_sum(Sum::new(
                    AggregationTemporality::Delta,
                    true,
                    number_points(metric, point_count, now, rng),
                ))
                .finish(),
            InstrumentSpec::UpDownCounter => builder
                .data_sum(Sum::new(
                    AggregationTemporality::Delta,
                    false,
                    number_points(metric, point_count, now, rng),
                ))
                .finish(),
        };
        metrics.push(built);
    }
    metrics
}

fn number_points<R: Rng + ?Sized>(
    metric: &MetricTemplate,
    count: usize,
    now: u64,
    rng: &mut R,
) -> Vec<NumberDataPoint> {
    (0..count)
        .map(|_| {
            let attrs = attributes(&metric.attributes, rng);
            let value = metric_value(metric, rng);
            let b = NumberDataPoint::build()
                .time_unix_nano(now)
                .attributes(attrs);
            if metric.int_value {
                b.value_int(value.round() as i64).finish()
            } else {
                b.value_double(value).finish()
            }
        })
        .collect()
}

/// Index of the explicit bucket an observation falls into.
///
/// Bucket `i` counts observations `x` with `bounds[i-1] < x <= bounds[i]`;
/// the last bucket (`bounds.len()`) counts `x > bounds[last]` (OTLP explicit
/// bucket semantics).
#[must_use]
pub fn bucket_index(bounds: &[f64], x: f64) -> usize {
    bounds.partition_point(|b| *b < x)
}

/// Aggregate sampled observations into explicit bucket counts.
///
/// Bucket `i` counts observations `x` with `bounds[i-1] < x <= bounds[i]`;
/// the last bucket counts `x > bounds[last]` (OTLP explicit bucket semantics).
#[must_use]
pub fn bucketize(observations: &[f64], bounds: &[f64]) -> Vec<u64> {
    let mut counts = vec![0u64; bounds.len() + 1];
    for x in observations {
        counts[bucket_index(bounds, *x)] += 1;
    }
    counts
}

fn histogram_point<R: Rng + ?Sized>(
    metric: &MetricTemplate,
    now: u64,
    rng: &mut R,
) -> HistogramDataPoint {
    let attrs = attributes(&metric.attributes, rng);
    let n = metric
        .observations
        .as_ref()
        .map_or(1, |o| o.sample_count(rng).max(1));
    // Aggregate on the fly instead of materializing the observations.
    let mut bucket_counts = vec![0u64; metric.bounds.len() + 1];
    let (mut sum, mut min, mut max) = (0.0, f64::INFINITY, f64::NEG_INFINITY);
    for _ in 0..n {
        let x = metric_value(metric, rng);
        bucket_counts[bucket_index(&metric.bounds, x)] += 1;
        sum += x;
        min = min.min(x);
        max = max.max(x);
    }
    HistogramDataPoint::build()
        .time_unix_nano(now)
        .count(n as u64)
        .sum(sum)
        .min(min)
        .max(max)
        .bucket_counts(bucket_counts)
        .explicit_bounds(metric.bounds.clone())
        .attributes(attrs)
        .finish()
}

/// generate each event (structured log) defined in the resolved registry
fn logs<R: Rng + ?Sized>(signal_count: usize, model: &SemconvModel, rng: &mut R) -> Vec<LogRecord> {
    let mut log_records = Vec::with_capacity(capacity(signal_count, model.events.len()));
    for event in model.events.iter().cycle().take(signal_count) {
        // events are structured logs
        let timestamp = current_time();
        let (number, text) = log_severity(event, rng);
        let builder = LogRecord::build()
            .time_unix_nano(timestamp)
            .severity_number(number)
            .event_name(event.name.clone())
            .attributes(attributes(&event.attributes, rng))
            .body(AnyValue::new_string(event.body.clone()))
            .observed_time_unix_nano(timestamp);
        log_records.push(if text.is_empty() {
            builder.finish()
        } else {
            builder.severity_text(text).finish()
        });
    }
    log_records
}

/// map a SpanKindSpec to a SpanKind
#[must_use]
const fn otel_span_kind(span_kind: Option<&SpanKindSpec>) -> SpanKind {
    match span_kind {
        Some(SpanKindSpec::Client) => SpanKind::Client,
        Some(SpanKindSpec::Server) => SpanKind::Server,
        Some(SpanKindSpec::Producer) => SpanKind::Producer,
        Some(SpanKindSpec::Consumer) => SpanKind::Consumer,
        Some(SpanKindSpec::Internal) | None => SpanKind::Internal,
    }
}
