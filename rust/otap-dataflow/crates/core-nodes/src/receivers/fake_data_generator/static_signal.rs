// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Static hardcoded signal generators for lightweight load testing.
//!
//! These generators produce minimal OTLP signals without requiring the
//! semantic conventions registry, making them ideal for high-throughput
//! load testing where startup time and per-signal CPU cost matter.

use crate::receivers::fake_data_generator::fake_data::{
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

/// Static metric attributes
fn static_metric_attributes() -> Vec<KeyValue> {
    vec![
        KeyValue::new("http.method", AnyValue::new_string("GET")),
        KeyValue::new("http.route", AnyValue::new_string("/api")),
        KeyValue::new("http.status_code", AnyValue::new_int(200)),
    ]
}

/// Static log attributes (default when `num_log_attributes` is not configured).
fn static_log_attributes() -> Vec<KeyValue> {
    vec![
        KeyValue::new("thread.id", AnyValue::new_int(1)),
        KeyValue::new("thread.name", AnyValue::new_string("main")),
    ]
}

/// Generates TracesData with static hardcoded spans
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

/// Generates LogsData with static hardcoded log records
#[must_use]
pub fn static_otlp_logs(
    signal_count: usize,
    extra_attrs: Option<&HashMap<String, String>>,
) -> LogsData {
    static_otlp_logs_with_config(signal_count, None, None, extra_attrs)
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
    extra_attrs: Option<&HashMap<String, String>>,
) -> LogsData {
    let logs = static_logs(signal_count, log_body_size_bytes, num_log_attributes);

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

/// Generates MetricsData with static hardcoded metrics
#[must_use]
pub fn static_otlp_metrics(
    signal_count: usize,
    extra_attrs: Option<&HashMap<String, String>>,
) -> MetricsData {
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

/// Generate a log body string of the specified size in bytes.
/// Produces a repeating pattern of printable ASCII characters.
fn generate_body(size_bytes: usize) -> String {
    const PATTERN: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyz";
    let mut body = String::with_capacity(size_bytes);
    for i in 0..size_bytes {
        body.push(PATTERN[i % PATTERN.len()] as char);
    }
    body
}

/// Generate synthetic log attributes with the specified count.
/// Each attribute is a string key-value pair like `attr_0 = "value_0"`.
fn generate_log_attributes(count: usize) -> Vec<KeyValue> {
    (0..count)
        .map(|i| KeyValue::new(format!("attr_{i}"), AnyValue::new_string(format!("value_{i}"))))
        .collect()
}

/// Generate static log records for load testing.
///
/// When `log_body_size_bytes` or `num_log_attributes` are `None`, the
/// function falls back to the original hardcoded defaults.
fn static_logs(
    signal_count: usize,
    log_body_size_bytes: Option<usize>,
    num_log_attributes: Option<usize>,
) -> Vec<LogRecord> {
    let attributes = match num_log_attributes {
        Some(n) => generate_log_attributes(n),
        None => static_log_attributes(),
    };

    let body_str = match log_body_size_bytes {
        Some(n) => generate_body(n),
        None => "Order processed successfully".to_string(),
    };

    (0..signal_count)
        .map(|_| {
            let timestamp = current_time();

            LogRecord::build()
                .time_unix_nano(timestamp)
                .observed_time_unix_nano(timestamp)
                .severity_number(SeverityNumber::Info)
                .severity_text("INFO")
                .body(AnyValue::new_string(&body_str))
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
    fn test_static_logs() {
        let logs = static_otlp_logs(10, None);
        assert_eq!(logs.resource_logs.len(), 1);
        assert_eq!(logs.resource_logs[0].scope_logs.len(), 1);
        assert_eq!(logs.resource_logs[0].scope_logs[0].log_records.len(), 10);
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
        let logs = static_otlp_logs_with_config(5, Some(1024), None, None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 5);
        // Check body is the expected size
        if let Some(body) = &records[0].body {
            if let Some(ref value) = body.value {
                match value {
                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => {
                        assert_eq!(s.len(), 1024);
                    }
                    _ => panic!("Expected string body"),
                }
            }
        }
        // Default attributes (2) should be used
        assert_eq!(records[0].attributes.len(), 2);
    }

    #[test]
    fn test_static_logs_with_custom_attributes() {
        let logs = static_otlp_logs_with_config(3, None, Some(5), None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].attributes.len(), 5);
        assert_eq!(records[0].attributes[0].key, "attr_0");
        assert_eq!(records[0].attributes[4].key, "attr_4");
    }

    #[test]
    fn test_static_logs_with_both_custom() {
        let logs = static_otlp_logs_with_config(2, Some(512), Some(10), None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].attributes.len(), 10);
    }

    #[test]
    fn test_static_logs_zero_body_size() {
        let logs = static_otlp_logs_with_config(1, Some(0), None, None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        if let Some(body) = &records[0].body {
            if let Some(ref value) = body.value {
                match value {
                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => {
                        assert!(s.is_empty());
                    }
                    _ => panic!("Expected string body"),
                }
            }
        }
    }

    #[test]
    fn test_static_logs_zero_attributes() {
        let logs = static_otlp_logs_with_config(1, None, Some(0), None);
        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert!(records[0].attributes.is_empty());
    }
}
