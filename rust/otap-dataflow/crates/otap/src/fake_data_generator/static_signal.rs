// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Static hardcoded signal generators for lightweight load testing.
//!
//! These generators produce minimal OTLP signals without requiring the
//! semantic conventions registry, making them ideal for high-throughput
//! load testing where startup time and per-signal CPU cost matter.

use crate::fake_data_generator::fake_data::{
    current_time, delay, gen_span_id, gen_trace_id, get_scope_name, get_scope_version,
};
use otap_df_pdata::proto::opentelemetry::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
    metrics::v1::{
        AggregationTemporality, Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
        ScopeMetrics, Sum,
    },
    resource::v1::Resource,
    trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData, span::SpanKind},
};

/// Static resource attributes
fn static_resource_attributes() -> Vec<KeyValue> {
    vec![
        KeyValue::new("service.name", AnyValue::new_string("load-generator")),
        KeyValue::new("service.version", AnyValue::new_string("1.0.0")),
        KeyValue::new("service.instance.id", AnyValue::new_string("instance-001")),
    ]
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

/// Static metric attributes
fn static_metric_attributes() -> Vec<KeyValue> {
    vec![
        KeyValue::new("http.method", AnyValue::new_string("GET")),
        KeyValue::new("http.route", AnyValue::new_string("/api")),
        KeyValue::new("http.status_code", AnyValue::new_int(200)),
    ]
}

/// Static log attributes
fn static_log_attributes() -> Vec<KeyValue> {
    vec![
        KeyValue::new("log.level", AnyValue::new_string("INFO")),
        KeyValue::new("code.function", AnyValue::new_string("handle_request")),
        KeyValue::new("code.filepath", AnyValue::new_string("src/handler.rs")),
        KeyValue::new("code.lineno", AnyValue::new_int(42)),
    ]
}

/// Generates TracesData with static hardcoded spans
#[must_use]
pub fn static_otlp_traces(signal_count: usize) -> TracesData {
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
            .attributes(static_resource_attributes())
            .finish(),
        scopes,
    )];

    TracesData::new(resources)
}

/// Generates LogsData with static hardcoded log records
#[must_use]
pub fn static_otlp_logs(signal_count: usize) -> LogsData {
    let logs = static_logs(signal_count);

    let scopes = vec![ScopeLogs::new(
        InstrumentationScope::build()
            .name(get_scope_name())
            .version(get_scope_version())
            .finish(),
        logs,
    )];

    let resources = vec![ResourceLogs::new(
        Resource::build()
            .attributes(static_resource_attributes())
            .finish(),
        scopes,
    )];

    LogsData::new(resources)
}

/// Generates MetricsData with static hardcoded metrics
#[must_use]
pub fn static_otlp_metrics(signal_count: usize) -> MetricsData {
    let metrics = static_metrics(signal_count);

    let scopes = vec![ScopeMetrics::new(
        InstrumentationScope::build()
            .name(get_scope_name())
            .version(get_scope_version())
            .finish(),
        metrics,
    )];

    let resources = vec![ResourceMetrics::new(
        Resource::build()
            .attributes(static_resource_attributes())
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

/// Generate static metrics (alternating between counter and gauge)
fn static_metrics(signal_count: usize) -> Vec<Metric> {
    let attributes = static_metric_attributes();

    (0..signal_count)
        .map(|i| {
            let timestamp = current_time();
            let datapoints = vec![
                NumberDataPoint::build()
                    .time_unix_nano(timestamp)
                    .value_double(1.0)
                    .attributes(attributes.clone())
                    .finish(),
            ];

            if i % 2 == 0 {
                // Counter (monotonic sum)
                Metric::build()
                    .name("http.server.request.duration")
                    .description("Duration of HTTP server requests")
                    .unit("ms")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        datapoints,
                    ))
                    .finish()
            } else {
                // Gauge
                Metric::build()
                    .name("http.server.active_requests")
                    .description("Number of active HTTP requests")
                    .unit("{request}")
                    .data_gauge(Gauge::new(datapoints))
                    .finish()
            }
        })
        .collect()
}

/// Generate static log records
fn static_logs(signal_count: usize) -> Vec<LogRecord> {
    let attributes = static_log_attributes();

    (0..signal_count)
        .map(|_| {
            let timestamp = current_time();

            LogRecord::build()
                .time_unix_nano(timestamp)
                .observed_time_unix_nano(timestamp)
                .severity_number(SeverityNumber::Info)
                .severity_text("INFO")
                .body(AnyValue::new_string("Request processed successfully"))
                .attributes(attributes.clone())
                .finish()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_traces() {
        let traces = static_otlp_traces(10);
        assert_eq!(traces.resource_spans.len(), 1);
        assert_eq!(traces.resource_spans[0].scope_spans.len(), 1);
        assert_eq!(traces.resource_spans[0].scope_spans[0].spans.len(), 10);
    }

    #[test]
    fn test_static_metrics() {
        let metrics = static_otlp_metrics(10);
        assert_eq!(metrics.resource_metrics.len(), 1);
        assert_eq!(metrics.resource_metrics[0].scope_metrics.len(), 1);
        assert_eq!(
            metrics.resource_metrics[0].scope_metrics[0].metrics.len(),
            10
        );
    }

    #[test]
    fn test_static_logs() {
        let logs = static_otlp_logs(10);
        assert_eq!(logs.resource_logs.len(), 1);
        assert_eq!(logs.resource_logs[0].scope_logs.len(), 1);
        assert_eq!(logs.resource_logs[0].scope_logs[0].log_records.len(), 10);
    }
}
