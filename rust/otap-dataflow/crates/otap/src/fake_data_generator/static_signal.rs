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

/// Static log attributes for load testing.
/// Sized to produce approximately 1 KB per log record.
fn static_log_attributes() -> Vec<KeyValue> {
    vec![
        KeyValue::new(
            "cloud.resource_id",
            AnyValue::new_string(
                "/subscriptions/12345678-1234-1234-1234-123456789abc/resourceGroups/my-resource-group/providers/Microsoft.Web/sites/my-function-app",
            ),
        ),
        KeyValue::new(
            "service.instance.id",
            AnyValue::new_string("i-1234567890abcdef0"),
        ),
        KeyValue::new(
            "service.namespace",
            AnyValue::new_string("production-us-west-2"),
        ),
        KeyValue::new(
            "host.name",
            AnyValue::new_string("ip-172-31-24-56.us-west-2.compute.internal"),
        ),
        KeyValue::new("host.ip", AnyValue::new_string("172.31.24.56")),
        KeyValue::new(
            "container.id",
            AnyValue::new_string(
                "3f4b8c2d1a9e5f6b7c8d9e0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c",
            ),
        ),
        KeyValue::new(
            "container.image.name",
            AnyValue::new_string("myregistry.azurecr.io/services/order-processor"),
        ),
        KeyValue::new(
            "container.image.tag",
            AnyValue::new_string("v2.15.3-alpine"),
        ),
        KeyValue::new("process.pid", AnyValue::new_int(28547)),
        KeyValue::new(
            "process.executable.name",
            AnyValue::new_string("order-processor"),
        ),
        KeyValue::new("thread.id", AnyValue::new_int(14073)),
        KeyValue::new("thread.name", AnyValue::new_string("async-worker-pool-7")),
        KeyValue::new(
            "code.filepath",
            AnyValue::new_string("/app/src/services/order_processor.rs"),
        ),
        KeyValue::new(
            "code.function",
            AnyValue::new_string("process_incoming_order"),
        ),
        KeyValue::new("code.lineno", AnyValue::new_int(247)),
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

/// Generate static log records for load testing.
/// Produces approximately 1 KB per log record.
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
                .body(AnyValue::new_string(
                    "Successfully processed order #ORD-2024-0847291 for customer cust_8a7b6c5d4e3f2a1b. \
                     Items: 3, Total: $142.99 USD. Payment method: credit_card ending in 4242. \
                     Shipping address validated and inventory reserved. \
                     Estimated delivery: 2024-01-25. Fulfillment center: FC-SEA-03. \
                     Correlation ID: corr-f8e7d6c5-b4a3-9281-7654-321fedcba098."
                ))
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
