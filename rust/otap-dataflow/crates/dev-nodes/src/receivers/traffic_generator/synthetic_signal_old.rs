// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Previous prost-based synthetic signal generators.
//!
//! The traffic generator now builds synthetic data as OTAP record batches (see
//! [`super::synthetic_signal`]). This module keeps the original prost
//! implementation so the two can be compared in equivalence tests and
//! benchmarks. It shares the data pools with the new implementation.

use super::synthetic_signal::{
    ATTR_NAMES, ATTR_VALUE_POOL, DEFAULT_DATA_POINTS_PER_METRIC, DEFAULT_LOG_ATTRIBUTE_COUNT,
    DEFAULT_METRIC_ATTRIBUTE_COUNT, HISTOGRAM_BOUNDS, METRIC_ATTR_HOSTS, METRIC_ATTR_METHODS,
    METRIC_ATTR_NAMES, METRIC_ATTR_ROUTES, METRIC_DEFS, MetricKind, SUMMARY_QUANTILES,
    THREAD_NAMES, build_body_pool,
};
use crate::receivers::traffic_generator::fake_data::{
    current_time, delay, gen_span_id, gen_trace_id, get_scope_name, get_scope_version,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
    metrics::v1::{
        AggregationTemporality, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge,
        Histogram, HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
        ScopeMetrics, Sum, Summary, SummaryDataPoint, exponential_histogram_data_point::Buckets,
        summary_data_point::ValueAtQuantile,
    },
    resource::v1::Resource,
    trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData, span::SpanKind},
};
use std::collections::HashMap;

/// Static resource attributes, with optional user-supplied extras merged in.
fn build_resource_attributes(extra: Option<&HashMap<String, String>>) -> Vec<KeyValue> {
    let mut attrs = vec![
        KeyValue::new("service.name", AnyValue::new_string("load-generator")),
        KeyValue::new("service.version", AnyValue::new_string("1.0.0")),
        KeyValue::new("service.instance.id", AnyValue::new_string("instance-001")),
    ];
    if let Some(extra) = extra {
        for (k, v) in extra {
            attrs.push(KeyValue::new(k.as_str(), AnyValue::new_string(v.as_str())));
        }
    }
    attrs
}

/// Static span attributes for HTTP server spans
fn static_span_attributes() -> Vec<KeyValue> {
    vec![
        KeyValue::new("http.method", AnyValue::new_string("GET")),
        KeyValue::new("http.url", AnyValue::new_string("http://example.com/api")),
        KeyValue::new("http.status_code", AnyValue::new_int(200)),
        KeyValue::new("http.route", AnyValue::new_string("/api")),
    ]
}

/// Generate `count` metric attributes with realistic names and per-data-point
/// value variance.
fn build_metric_attributes(count: usize, point_index: usize) -> Vec<KeyValue> {
    (0..count)
        .map(|i| {
            let name = if i < METRIC_ATTR_NAMES.len() {
                METRIC_ATTR_NAMES[i].to_string()
            } else {
                format!("metric_attr_{i}")
            };
            let value = match METRIC_ATTR_NAMES.get(i) {
                Some(&"http.method") => AnyValue::new_string(
                    METRIC_ATTR_METHODS[point_index % METRIC_ATTR_METHODS.len()],
                ),
                Some(&"http.route") => {
                    AnyValue::new_string(METRIC_ATTR_ROUTES[point_index % METRIC_ATTR_ROUTES.len()])
                }
                Some(&"http.status_code") | Some(&"http.response.status_code") => {
                    AnyValue::new_int(
                        [200, 200, 200, 201, 204, 301, 400, 403, 404, 500][point_index % 10],
                    )
                }
                Some(&"server.port") | Some(&"client.port") | Some(&"net.peer.port") => {
                    AnyValue::new_int([8080, 8443, 3000, 4317, 9090, 5432][point_index % 6])
                }
                Some(&"server.address") | Some(&"host.name") | Some(&"net.peer.name") => {
                    AnyValue::new_string(METRIC_ATTR_HOSTS[point_index % METRIC_ATTR_HOSTS.len()])
                }
                _ => {
                    let pool_idx = (i.wrapping_mul(7) + point_index) % ATTR_VALUE_POOL.len();
                    AnyValue::new_string(ATTR_VALUE_POOL[pool_idx])
                }
            };
            KeyValue::new(name, value)
        })
        .collect()
}

/// Generate `count` log attributes with realistic names and per-record value variance.
///
/// Names come from [`ATTR_NAMES`]; if `count` exceeds the pool size, overflow
/// names are `attr_N`. Values vary by `record_index` with realistic cardinality.
fn build_log_attributes(count: usize, record_index: usize) -> Vec<KeyValue> {
    (0..count)
        .map(|i| {
            let name = if i < ATTR_NAMES.len() {
                ATTR_NAMES[i].to_string()
            } else {
                format!("attr_{i}")
            };
            // Special-case a few names for more realistic value shapes
            let value = match ATTR_NAMES.get(i) {
                Some(&"thread.id") => AnyValue::new_int((record_index % 16) as i64),
                Some(&"thread.name") => {
                    AnyValue::new_string(THREAD_NAMES[record_index % THREAD_NAMES.len()])
                }
                Some(&"code.lineno") | Some(&"server.port") | Some(&"client.port") => {
                    AnyValue::new_int((record_index % 500) as i64)
                }
                Some(&"http.response.status_code") => AnyValue::new_int(
                    [200, 200, 200, 201, 204, 301, 400, 403, 404, 500][record_index % 10],
                ),
                _ => {
                    // Pick from value pool using both attr index and record index
                    // to avoid all attributes on the same record getting the same value.
                    let pool_idx = (i.wrapping_mul(7) + record_index) % ATTR_VALUE_POOL.len();
                    AnyValue::new_string(ATTR_VALUE_POOL[pool_idx])
                }
            };
            KeyValue::new(name, value)
        })
        .collect()
}

/// Generates TracesData with Synthetic hardcoded spans
#[must_use]
pub fn static_otlp_traces(
    signal_count: usize,
    extra_attrs: Option<&HashMap<String, String>>,
) -> TracesData {
    let spans = static_spans(signal_count);

    let scopes = vec![ScopeSpans::new(
        InstrumentationScope::build()
            .name(get_scope_name())
            .version(get_scope_version())
            .finish(),
        spans,
    )];

    let resources = vec![ResourceSpans::new(
        Resource::build()
            .attributes(build_resource_attributes(extra_attrs))
            .finish(),
        scopes,
    )];

    TracesData::new(resources)
}

/// Generates LogsData with Synthetic hardcoded log records
#[must_use]
pub fn static_otlp_logs(
    signal_count: usize,
    extra_attrs: Option<&HashMap<String, String>>,
) -> LogsData {
    static_otlp_logs_with_config(signal_count, None, None, false, extra_attrs)
}

/// Generates LogsData with configurable body size, attribute count, and
/// optional extra resource attributes.
///
/// - `log_body_size_bytes`: When `Some(n)`, generates a log body of approximately `n` bytes.
///   When `None`, uses the default body ("Order processed successfully").
/// - `num_log_attributes`: When `Some(n)`, generates `n` key-value string attributes.
///   When `None`, uses the default 2 attributes (thread.id, thread.name).
/// - `extra_attrs`: Optional extra key-value pairs merged into the resource attributes.
#[must_use]
pub fn static_otlp_logs_with_config(
    signal_count: usize,
    log_body_size_bytes: Option<usize>,
    num_log_attributes: Option<usize>,
    use_trace_context: bool,
    extra_attrs: Option<&HashMap<String, String>>,
) -> LogsData {
    let logs = static_logs(
        signal_count,
        log_body_size_bytes,
        num_log_attributes,
        use_trace_context,
    );

    let scopes = vec![ScopeLogs::new(
        InstrumentationScope::build()
            .name(get_scope_name())
            .version(get_scope_version())
            .finish(),
        logs,
    )];

    let resources = vec![ResourceLogs::new(
        Resource::build()
            .attributes(build_resource_attributes(extra_attrs))
            .finish(),
        scopes,
    )];

    LogsData::new(resources)
}

/// Generates MetricsData with Synthetic hardcoded metrics
#[must_use]
pub fn static_otlp_metrics(
    signal_count: usize,
    extra_attrs: Option<&HashMap<String, String>>,
) -> MetricsData {
    static_otlp_metrics_with_config(signal_count, None, None, extra_attrs)
}

/// Generates MetricsData with configurable attribute count, data point count,
/// and optional extra resource attributes.
///
/// - `num_metric_attributes`: When `Some(n)`, generates `n` key-value attributes per data point.
///   When `None`, uses the default 3 attributes (http.method, http.route, http.status_code).
/// - `num_data_points`: When `Some(n)`, generates `n` data points per metric.
///   When `None`, uses the default of 1 data point per metric.
/// - `extra_attrs`: Optional extra key-value pairs merged into the resource attributes.
#[must_use]
pub fn static_otlp_metrics_with_config(
    signal_count: usize,
    num_metric_attributes: Option<usize>,
    num_data_points: Option<usize>,
    extra_attrs: Option<&HashMap<String, String>>,
) -> MetricsData {
    let metrics = static_metrics_elaborate(signal_count, num_metric_attributes, num_data_points);

    let scopes = vec![ScopeMetrics::new(
        InstrumentationScope::build()
            .name(get_scope_name())
            .version(get_scope_version())
            .finish(),
        metrics,
    )];

    let resources = vec![ResourceMetrics::new(
        Resource::build()
            .attributes(build_resource_attributes(extra_attrs))
            .finish(),
        scopes,
    )];

    MetricsData::new(resources)
}

/// Generate static spans
fn static_spans(signal_count: usize) -> Vec<Span> {
    let attributes = static_span_attributes();

    (0..signal_count)
        .map(|_| {
            let start_time = current_time();
            let end_time = start_time + delay();

            Span::build()
                .trace_id(gen_trace_id())
                .span_id(gen_span_id())
                .name("HTTP GET")
                .start_time_unix_nano(start_time)
                .end_time_unix_nano(end_time)
                .kind(SpanKind::Server)
                .attributes(attributes.clone())
                .finish()
        })
        .collect()
}

/// Generate static metrics covering all five OTLP metric types with realistic
/// names, attributes, and per-data-point variance.
///
/// Metrics cycle through [`METRIC_DEFS`] which includes Counters, UpDownCounters,
/// Gauges, Histograms, ExponentialHistograms, and Summaries.
fn static_metrics_elaborate(
    signal_count: usize,
    num_metric_attributes: Option<usize>,
    num_data_points: Option<usize>,
) -> Vec<Metric> {
    let attr_count = num_metric_attributes.unwrap_or(DEFAULT_METRIC_ATTRIBUTE_COUNT);
    let dp_count = num_data_points.unwrap_or(DEFAULT_DATA_POINTS_PER_METRIC);
    let defs_len = METRIC_DEFS.len();

    (0..signal_count)
        .map(|i| {
            let def = &METRIC_DEFS[i % defs_len];
            match def.kind {
                MetricKind::Counter => {
                    let datapoints = build_number_data_points(dp_count, i, attr_count, true);
                    Metric::build()
                        .name(def.name)
                        .description(def.description)
                        .unit(def.unit)
                        .data_sum(Sum::new(
                            AggregationTemporality::Cumulative,
                            true,
                            datapoints,
                        ))
                        .finish()
                }
                MetricKind::UpDownCounter => {
                    let datapoints = build_number_data_points(dp_count, i, attr_count, false);
                    Metric::build()
                        .name(def.name)
                        .description(def.description)
                        .unit(def.unit)
                        .data_sum(Sum::new(AggregationTemporality::Delta, false, datapoints))
                        .finish()
                }
                MetricKind::Gauge => {
                    let datapoints = build_number_data_points(dp_count, i, attr_count, false);
                    Metric::build()
                        .name(def.name)
                        .description(def.description)
                        .unit(def.unit)
                        .data_gauge(Gauge::new(datapoints))
                        .finish()
                }
                MetricKind::Histogram => {
                    let datapoints = build_histogram_data_points(dp_count, i, attr_count);
                    Metric::build()
                        .name(def.name)
                        .description(def.description)
                        .unit(def.unit)
                        .data_histogram(Histogram::new(AggregationTemporality::Delta, datapoints))
                        .finish()
                }
                MetricKind::ExponentialHistogram => {
                    let datapoints = build_exp_histogram_data_points(dp_count, i, attr_count);
                    Metric::build()
                        .name(def.name)
                        .description(def.description)
                        .unit(def.unit)
                        .data_exponential_histogram(ExponentialHistogram::new(
                            AggregationTemporality::Delta,
                            datapoints,
                        ))
                        .finish()
                }
                MetricKind::Summary => {
                    let datapoints = build_summary_data_points(dp_count, i, attr_count);
                    Metric::build()
                        .name(def.name)
                        .description(def.description)
                        .unit(def.unit)
                        .data_summary(Summary::new(datapoints))
                        .finish()
                }
            }
        })
        .collect()
}

/// Build NumberDataPoints with varying values and attributes.
fn build_number_data_points(
    count: usize,
    metric_index: usize,
    attr_count: usize,
    monotonic: bool,
) -> Vec<NumberDataPoint> {
    (0..count)
        .map(|j| {
            let point_idx = metric_index.wrapping_mul(31) + j;
            let timestamp = current_time();
            let start_time = timestamp - delay();
            let attributes = build_metric_attributes(attr_count, point_idx);
            let value = if monotonic {
                // Monotonic counter: increasing values
                ((point_idx + 1) * 42) as f64 + (j as f64 * 1.5)
            } else {
                // Gauge/up-down: varied values centered around typical ranges
                let base_values = [0.73, 1.2, 45.8, 128.0, 0.95, 2048.0, 0.12, 89.5];
                base_values[point_idx % base_values.len()]
            };
            NumberDataPoint::build()
                .time_unix_nano(timestamp)
                .start_time_unix_nano(start_time)
                .value_double(value)
                .attributes(attributes)
                .finish()
        })
        .collect()
}

/// Build HistogramDataPoints with realistic bucket distributions.
fn build_histogram_data_points(
    count: usize,
    metric_index: usize,
    attr_count: usize,
) -> Vec<HistogramDataPoint> {
    // Pre-computed bucket count distributions (16 buckets = 15 bounds + 1 overflow)
    // representing different latency profiles.
    // Bounds: [0, 5, 10, 25, 50, 75, 100, 250, 500, 750, 1000, 2500, 5000, 7500, 10000, +inf]
    const DISTRIBUTIONS: &[&[u64]] = &[
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

    (0..count)
        .map(|j| {
            let point_idx = metric_index.wrapping_mul(31) + j;
            let timestamp = current_time();
            let start_time = timestamp - delay();
            let attributes = build_metric_attributes(attr_count, point_idx);
            let dist = DISTRIBUTIONS[point_idx % DISTRIBUTIONS.len()];
            let total_count: u64 = dist.iter().sum();
            // Midpoints of each bucket: [(-inf,0], (0,5], (5,10], ... (10000,+inf)]
            let midpoints = [
                0.0, 2.5, 7.5, 17.5, 37.5, 62.5, 87.5, 175.0, 375.0, 625.0, 875.0, 1750.0, 3750.0,
                6250.0, 8750.0, 15000.0,
            ];
            let sum: f64 = dist
                .iter()
                .zip(midpoints.iter())
                .map(|(&c, &m)| c as f64 * m)
                .sum();
            // Derive min/max from the first and last non-zero bucket midpoints
            let min_val = dist
                .iter()
                .zip(midpoints.iter())
                .find(|(c, _)| **c > 0)
                .map_or(0.0, |(_, &m)| m);
            let max_val = dist
                .iter()
                .zip(midpoints.iter())
                .rev()
                .find(|(c, _)| **c > 0)
                .map_or(0.0, |(_, &m)| m);

            HistogramDataPoint::build()
                .time_unix_nano(timestamp)
                .start_time_unix_nano(start_time)
                .count(total_count)
                .sum(sum)
                .bucket_counts(dist.to_vec())
                .explicit_bounds(HISTOGRAM_BOUNDS.to_vec())
                .min(min_val)
                .max(max_val)
                .attributes(attributes)
                .finish()
        })
        .collect()
}

/// Build ExponentialHistogramDataPoints with realistic exponential bucket distributions.
fn build_exp_histogram_data_points(
    count: usize,
    metric_index: usize,
    attr_count: usize,
) -> Vec<ExponentialHistogramDataPoint> {
    (0..count)
        .map(|j| {
            let point_idx = metric_index.wrapping_mul(31) + j;
            let timestamp = current_time();
            let start_time = timestamp - delay();
            let attributes = build_metric_attributes(attr_count, point_idx);

            // Vary the bucket distribution per point
            let positive_buckets = match point_idx % 4 {
                0 => Buckets::new(0, vec![100, 80, 40, 15, 5, 2, 1]),
                1 => Buckets::new(1, vec![20, 50, 120, 80, 30, 10]),
                2 => Buckets::new(-1, vec![5, 15, 40, 90, 60, 25, 8, 3]),
                _ => Buckets::new(0, vec![30, 45, 60, 45, 30, 15, 5]),
            };
            let total_count: u64 = positive_buckets.bucket_counts.iter().sum();
            let sum = total_count as f64 * 12.5;
            let zeros = 2u64;

            ExponentialHistogramDataPoint::build()
                .time_unix_nano(timestamp)
                .start_time_unix_nano(start_time)
                .count(total_count + zeros)
                .sum(sum)
                .scale(4)
                .zero_count(zeros)
                .positive(positive_buckets)
                .min(0.1)
                .max(sum / total_count as f64 * 3.0)
                .attributes(attributes)
                .finish()
        })
        .collect()
}

/// Build SummaryDataPoints with realistic quantile values.
fn build_summary_data_points(
    count: usize,
    metric_index: usize,
    attr_count: usize,
) -> Vec<SummaryDataPoint> {
    // Different latency profiles for quantile values
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

    (0..count)
        .map(|j| {
            let point_idx = metric_index.wrapping_mul(31) + j;
            let timestamp = current_time();
            let start_time = timestamp - delay();
            let attributes = build_metric_attributes(attr_count, point_idx);
            let profile = QUANTILE_PROFILES[point_idx % QUANTILE_PROFILES.len()];
            let total_count = ((point_idx + 1) * 1000) as u64;
            let sum = profile[3] * total_count as f64 * 0.8;

            let quantile_values: Vec<ValueAtQuantile> = SUMMARY_QUANTILES
                .iter()
                .zip(profile.iter())
                .map(|(&q, &v)| ValueAtQuantile::new(q, v))
                .collect();

            SummaryDataPoint::build()
                .time_unix_nano(timestamp)
                .start_time_unix_nano(start_time)
                .count(total_count)
                .sum(sum)
                .quantile_values(quantile_values)
                .attributes(attributes)
                .finish()
        })
        .collect()
}

/// Generate static log records for load testing.
///
/// Each record within a batch varies to produce realistic payloads:
/// - **Body**: cycles through 50 distinct templates
/// - **Attributes**: values drawn from a pool of realistic strings (keys stay fixed)
/// - **Severity**: rotates through a realistic distribution
///   (~=80% INFO, ~=15% WARN, ~=5% ERROR)
/// - **TraceID / SpanID**: unique random IDs per record (when `use_trace_context` is true)
///
/// When `log_body_size_bytes` or `num_log_attributes` are `None`, the
/// function falls back to the original hardcoded defaults.
fn static_logs(
    signal_count: usize,
    log_body_size_bytes: Option<usize>,
    num_log_attributes: Option<usize>,
    use_trace_context: bool,
) -> Vec<LogRecord> {
    let body_pool = build_body_pool(log_body_size_bytes);

    (0..signal_count)
        .map(|i| {
            let timestamp = current_time();
            let (severity_number, severity_text) = match i % 20 {
                0..=15 => (SeverityNumber::Info, "INFO"),
                16..=18 => (SeverityNumber::Warn, "WARN"),
                _ => (SeverityNumber::Error, "ERROR"),
            };

            let attributes =
                build_log_attributes(num_log_attributes.unwrap_or(DEFAULT_LOG_ATTRIBUTE_COUNT), i);

            let mut builder = LogRecord::build()
                .time_unix_nano(timestamp)
                .observed_time_unix_nano(timestamp)
                .severity_number(severity_number)
                .severity_text(severity_text)
                .attributes(attributes);

            if use_trace_context {
                builder = builder.trace_id(gen_trace_id()).span_id(gen_span_id());
            }

            if !body_pool.is_empty() {
                builder = builder.body(AnyValue::new_string(&body_pool[i % body_pool.len()]));
            }

            builder.finish()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_traces() {
        let traces = static_otlp_traces(10, None);
        assert_eq!(traces.resource_spans.len(), 1);
        assert_eq!(traces.resource_spans[0].scope_spans.len(), 1);
        assert_eq!(traces.resource_spans[0].scope_spans[0].spans.len(), 10);
    }

    #[test]
    fn test_static_metrics() {
        let metrics = static_otlp_metrics(10, None);
        assert_eq!(metrics.resource_metrics.len(), 1);
        assert_eq!(metrics.resource_metrics[0].scope_metrics.len(), 1);
        assert_eq!(
            metrics.resource_metrics[0].scope_metrics[0].metrics.len(),
            10
        );
    }

    #[test]
    fn test_static_metrics_all_types() {
        // Generate enough metrics to cycle through all METRIC_DEFS (22 entries)
        let metrics = static_otlp_metrics(22, None);
        let metric_list = &metrics.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(metric_list.len(), 22);

        // Verify we have all metric types
        use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data;
        let mut has_sum = false;
        let mut has_gauge = false;
        let mut has_histogram = false;
        let mut has_exp_histogram = false;
        let mut has_summary = false;
        for m in metric_list {
            match &m.data {
                Some(Data::Sum(_)) => has_sum = true,
                Some(Data::Gauge(_)) => has_gauge = true,
                Some(Data::Histogram(_)) => has_histogram = true,
                Some(Data::ExponentialHistogram(_)) => has_exp_histogram = true,
                Some(Data::Summary(_)) => has_summary = true,
                None => panic!("Metric should have data"),
            }
        }
        assert!(has_sum, "should generate Sum metrics");
        assert!(has_gauge, "should generate Gauge metrics");
        assert!(has_histogram, "should generate Histogram metrics");
        assert!(
            has_exp_histogram,
            "should generate ExponentialHistogram metrics"
        );
        assert!(has_summary, "should generate Summary metrics");
    }

    #[test]
    fn test_static_metrics_with_custom_attributes() {
        let metrics = static_otlp_metrics_with_config(5, Some(6), None, None);
        let metric_list = &metrics.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(metric_list.len(), 5);
    }

    #[test]
    fn test_static_metrics_with_custom_data_points() {
        let metrics = static_otlp_metrics_with_config(3, None, Some(4), None);
        let metric_list = &metrics.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(metric_list.len(), 3);
        // First metric is a counter (Sum), check it has 4 data points
        use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::metric::Data;
        if let Some(Data::Sum(sum)) = &metric_list[0].data {
            assert_eq!(sum.data_points.len(), 4);
        } else {
            panic!("First metric should be a Sum");
        }
    }

    #[test]
    fn test_static_metrics_with_extra_resource_attrs() {
        let mut extra = HashMap::new();
        _ = extra.insert("tenant.id".to_string(), "prod".to_string());
        let metrics = static_otlp_metrics(5, Some(&extra));
        let attrs = &metrics.resource_metrics[0]
            .resource
            .as_ref()
            .unwrap()
            .attributes;
        assert!(attrs.iter().any(|kv| kv.key == "tenant.id"));
    }

    /// Verify that generated metric batches achieve a realistic compression ratio.
    ///
    /// Generates 500 metrics covering all five OTLP metric types with 6
    /// attributes and 3 data points per metric, then checks the zstd
    /// compression ratio stays within a realistic range.
    ///
    /// Run with:
    /// ```sh
    /// cargo test -p otel-arrow-dfe-core-nodes --features dev-tools -- test_metrics_compression_ratio --nocapture
    /// ```
    #[test]
    fn test_metrics_compression_ratio_is_realistic() {
        use prost::Message;

        let metrics = static_otlp_metrics_with_config(500, Some(6), Some(3), None);
        let raw = metrics.encode_to_vec();
        let raw_size = raw.len();

        let compressed = zstd::bulk::compress(&raw, 3).expect("zstd compression failed");
        let compressed_size = compressed.len();

        let ratio = raw_size as f64 / compressed_size as f64;

        println!(
            "Metrics compression: raw={raw_size} bytes, compressed={compressed_size} bytes, \
             ratio={ratio:.1}:1"
        );

        assert!(
            (3.0..=45.0).contains(&ratio),
            "compression ratio {ratio:.1}:1 is outside acceptable range (3:1 - 45:1); \
             raw={raw_size} bytes, compressed={compressed_size} bytes"
        );
    }

    #[test]
    fn test_static_logs() {
        let logs = static_otlp_logs(10, None);
        assert_eq!(logs.resource_logs.len(), 1);
        assert_eq!(logs.resource_logs[0].scope_logs.len(), 1);
        assert_eq!(logs.resource_logs[0].scope_logs[0].log_records.len(), 10);
        // Bodies cycle through the default template pool
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert!(records[0].body.is_some());
        assert!(records[1].body.is_some());
    }

    #[test]
    fn test_static_logs_with_extra_attrs() {
        let mut extra = HashMap::new();
        _ = extra.insert("tenant.id".to_string(), "prod".to_string());
        let logs = static_otlp_logs(5, Some(&extra));
        let attrs = &logs.resource_logs[0].resource.as_ref().unwrap().attributes;
        assert!(attrs.iter().any(|kv| kv.key == "tenant.id"));
    }

    #[test]
    fn test_static_logs_with_custom_body_size() {
        let logs = static_otlp_logs_with_config(5, Some(1024), None, false, None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 5);
        if let Some(body) = &records[0].body
            && let Some(ref value) = body.value
        {
            match value {
                otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => {
                    assert_eq!(s.len(), 1024);
                }
                _ => panic!("Expected string body"),
            }
        }
        // Default attributes (2) should be used
        assert_eq!(records[0].attributes.len(), 2);
    }

    #[test]
    fn test_static_logs_with_custom_attributes() {
        let logs = static_otlp_logs_with_config(3, None, Some(5), false, None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].attributes.len(), 5);
        // First 5 names come from the ATTR_NAMES pool
        assert_eq!(records[0].attributes[0].key, "thread.id");
        assert_eq!(records[0].attributes[1].key, "thread.name");
        assert_eq!(records[0].attributes[2].key, "code.function");
        assert_eq!(records[0].attributes[3].key, "code.namespace");
        assert_eq!(records[0].attributes[4].key, "code.filepath");
        // Values vary per record
        let val0 = &records[0].attributes[2].value;
        let val1 = &records[1].attributes[2].value;
        assert_ne!(val0, val1);
    }

    #[test]
    fn test_static_logs_with_both_custom() {
        let logs = static_otlp_logs_with_config(2, Some(512), Some(10), false, None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].attributes.len(), 10);
    }

    #[test]
    fn test_static_logs_zero_body_size() {
        let logs = static_otlp_logs_with_config(1, Some(0), None, false, None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert!(
            records[0].body.is_none(),
            "body should be omitted when size is 0"
        );
    }

    #[test]
    fn test_static_logs_zero_attributes() {
        let logs = static_otlp_logs_with_config(1, None, Some(0), false, None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert!(records[0].attributes.is_empty());
    }
}
