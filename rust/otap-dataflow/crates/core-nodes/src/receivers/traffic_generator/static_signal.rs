// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Static hardcoded signal generators for lightweight load testing.
//!
//! These generators produce minimal OTLP signals without requiring the
//! semantic conventions registry, making them ideal for high-throughput
//! load testing where startup time and per-signal CPU cost matter.

use crate::receivers::traffic_generator::fake_data::{
    current_time, delay, gen_span_id, gen_trace_id, get_scope_name, get_scope_version,
};
use otap_df_pdata::proto::opentelemetry::{
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

/// Default number of metric attributes when `num_metric_attributes` is not configured.
const DEFAULT_METRIC_ATTRIBUTE_COUNT: usize = 3;

/// Default number of data points per metric when `num_data_points` is not configured.
const DEFAULT_DATA_POINTS_PER_METRIC: usize = 1;

/// Metric attribute names drawn from common OTel semantic conventions for metrics.
const METRIC_ATTR_NAMES: &[&str] = &[
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
const METRIC_ATTR_ROUTES: &[&str] = &[
    "/api/v1/users",
    "/api/v2/orders",
    "/api/v1/products",
    "/api/v2/payments",
    "/api/v1/auth",
    "/api/v3/search",
    "/api/v1/notifications",
    "/internal/health",
];

const METRIC_ATTR_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"];

const METRIC_ATTR_HOSTS: &[&str] = &[
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
enum MetricKind {
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

struct MetricDef {
    name: &'static str,
    description: &'static str,
    unit: &'static str,
    kind: MetricKind,
}

const METRIC_DEFS: &[MetricDef] = &[
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
const HISTOGRAM_BOUNDS: &[f64] = &[
    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0, 7500.0,
    10000.0,
];

/// Standard quantiles for summary metrics.
const SUMMARY_QUANTILES: &[f64] = &[0.0, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99, 1.0];

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

/// Pool of realistic log attribute names drawn from common OTel semantic conventions.
/// When generating attributes, names are picked from this pool in order.
/// If more attributes are requested than the pool contains, extra names are
/// generated as `attr_N` (where N is the overflow index).
const ATTR_NAMES: &[&str] = &[
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
const THREAD_NAMES: &[&str] = &[
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
/// Each pool entry is 20–120 chars, resembling real observability values like
/// paths, IDs, hostnames, URLs, error descriptions, and user agents.
const ATTR_VALUE_POOL: &[&str] = &[
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

/// Default number of log attributes when `num_log_attributes` is not configured.
const DEFAULT_LOG_ATTRIBUTE_COUNT: usize = 2;

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

/// Generates MetricsData with static hardcoded metrics
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

/// Default log body templates used when `log_body_size_bytes` is not configured.
/// Records cycle through these to produce realistic dictionary-friendly cardinality.
/// Each template is ~150 chars to provide meaningful entropy when padded to larger sizes.
const DEFAULT_BODY_TEMPLATES: &[&str] = &[
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

/// Build a pool of body strings for log generation.
///
/// When `size_bytes` is specified, each template from [`DEFAULT_BODY_TEMPLATES`]
/// is repeated to fill the target size, then truncated to exactly `size_bytes`.
/// When `None`, the templates are used as-is.
fn build_body_pool(size_bytes: Option<usize>) -> Vec<String> {
    match size_bytes {
        Some(0) => Vec::new(),
        Some(n) => DEFAULT_BODY_TEMPLATES
            .iter()
            .map(|template| {
                let mut body = String::with_capacity(n);
                while body.len() < n {
                    if !body.is_empty() {
                        body.push(' ');
                    }
                    body.push_str(template);
                }
                body.truncate(n);
                body
            })
            .collect(),
        None => DEFAULT_BODY_TEMPLATES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

/// Generate static log records for load testing.
///
/// Each record within a batch varies to produce realistic payloads:
/// - **Body**: cycles through 50 distinct templates
/// - **Attributes**: values drawn from a pool of realistic strings (keys stay fixed)
/// - **Severity**: rotates through a realistic distribution
///   (≈80% INFO, ≈15% WARN, ≈5% ERROR)
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
        use otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data;
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
        use otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data;
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
    /// cargo test -p otap-df-core-nodes --features dev-tools -- test_metrics_compression_ratio --nocapture
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

    /// Verify that generated log batches are not trivially compressible.
    ///
    /// Generates 500 logs with ~1KB bodies, 6 attributes, and trace context
    /// enabled, then checks the zstd compression ratio stays within a
    /// realistic range. Before these changes all records were nearly identical
    /// and compressed at ~57:1; with varied bodies, attribute values, severity
    /// rotation, and random trace_id/span_id the ratio drops to ~19:1.
    ///
    /// The assert uses a generous 3:1–45:1 window to avoid flaky failures
    /// across platforms while still catching regressions to the old
    /// all-identical regime (>50:1).
    ///
    /// Run with:
    /// ```sh
    /// cargo test -p otap-df-core-nodes --features dev-tools -- test_compression_ratio --nocapture
    /// ```
    #[test]
    fn test_compression_ratio_is_realistic() {
        use prost::Message;

        let logs = static_otlp_logs_with_config(500, Some(1024), Some(6), true, None);
        let raw = logs.encode_to_vec();
        let raw_size = raw.len();

        let compressed = zstd::bulk::compress(&raw, 3).expect("zstd compression failed");
        let compressed_size = compressed.len();

        let ratio = raw_size as f64 / compressed_size as f64;

        println!(
            "Compression: raw={raw_size} bytes, compressed={compressed_size} bytes, ratio={ratio:.1}:1"
        );

        assert!(
            (3.0..=45.0).contains(&ratio),
            "compression ratio {ratio:.1}:1 is outside acceptable range (3:1 – 45:1); \
             raw={raw_size} bytes, compressed={compressed_size} bytes"
        );
    }
}
