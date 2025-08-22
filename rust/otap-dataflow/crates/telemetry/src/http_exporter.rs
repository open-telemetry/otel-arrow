// SPDX-License-Identifier: Apache-2.0

//! HTTP server for exposing telemetry endpoints.

use axum::{
    extract::{State, Query},
    http::{StatusCode, header},
    response::{Json, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;

use crate::config::HttpServerConfig;
use crate::registry::MetricsRegistryHandle;
use crate::semconv::SemConvRegistry;
use crate::attributes::AttributeValue;
use crate::descriptor::Instrument;
use std::fmt::Write as _;

/// Shared state for the HTTP server.
#[derive(Clone)]
struct AppState {
    metrics_registry: MetricsRegistryHandle,
}

/// JSON representation of aggregated metrics.
#[derive(Serialize)]
struct MetricsResponse {
    /// Timestamp when the metrics were collected.
    timestamp: String,
    /// List of metric sets with their values.
    metric_sets: Vec<MetricSetJson>,
}

/// JSON representation of a metric set.
#[derive(Serialize)]
struct MetricSetJson {
    /// Name of the metric set.
    name: String,
    /// Description of the metric set.
    description: String,
    /// Attributes associated with this metric set.
    attributes: HashMap<String, Value>,
    /// Individual metrics within this set.
    metrics: Vec<MetricJson>,
}

/// JSON representation of an individual metric.
#[derive(Serialize)]
struct MetricJson {
    /// Name of the metric.
    name: String,
    /// Description of the metric.
    description: String,
    /// Unit of measurement.
    unit: String,
    /// Current value.
    value: u64,
    /// Type of instrument (counter, gauge, histogram, etc.).
    instrument_type: String,
}

/// Compact JSON representation of aggregated metrics (no metadata).
#[derive(Serialize)]
struct CompactMetricsResponse {
    timestamp: String,
    metric_sets: Vec<CompactMetricSetJson>,
}

#[derive(Serialize)]
struct CompactMetricSetJson {
    name: String,
    attributes: HashMap<String, Value>,
    metrics: HashMap<String, u64>,
}

/// Query parameters for /telemetry/metrics
#[derive(Debug, Default, Deserialize)]
struct MetricsQuery {
    /// When true, reset metrics after reading. Default: true.
    #[serde(default = "default_true")]
    reset: bool,
    /// Output format: json (default), line, prometheus
    #[serde(default)]
    format: Option<String>,
}

/// Query parameters for /telemetry/metrics/aggregate
#[derive(Debug, Default, Deserialize)]
struct AggregateQuery {
    /// When true, reset metrics after reading. Default: true.
    #[serde(default = "default_true")]
    reset: bool,
    /// Comma-separated list of attribute names to group by (in addition to metric set name).
    #[serde(default)]
    attrs: Option<String>,
    /// Output format: json (default), compact, line_protocol, prometheus
    #[serde(default)]
    format: Option<String>,
}

#[inline]
fn default_true() -> bool { true }

/// Starts the HTTP server with the given configuration and metrics registry.
pub async fn start_server(
    config: HttpServerConfig,
    metrics_registry: MetricsRegistryHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app_state = AppState { metrics_registry };

    let app = Router::new()
        .route("/telemetry/schema", get(get_semconv))
        .route("/telemetry/metrics", get(get_metrics))
        .route("/telemetry/metrics/aggregate", get(get_metrics_aggregate))
        .route("/health", get(health_check))
        .layer(ServiceBuilder::new())
        .with_state(app_state);

    let addr: SocketAddr = config.bind_address.parse()?;
    let listener = TcpListener::bind(&addr).await?;

    println!(
        "HTTP admin endpoints listening on {}\n  GET /health\n  GET /telemetry/schema\n  GET /telemetry/metrics?reset=true|false&format=json|json_compact|line_protocol|prometheus\n  GET /telemetry/metrics/aggregate?reset=true|false&attrs=attr1,attr2&format=json|json_compact|line_protocol|prometheus",
        addr
    );

    axum::serve(listener, app).await?;
    Ok(())
}

/// Handler for the /semconv endpoint - returns semantic convention registry.
async fn get_semconv(
    State(state): State<AppState>,
) -> Result<Json<SemConvRegistry>, StatusCode> {
    let semconv = state.metrics_registry.generate_semconv_registry();
    Ok(Json(semconv))
}

/// Handler for the /metrics endpoint - returns aggregated metrics in JSON or text format.
async fn get_metrics(
    State(state): State<AppState>,
    Query(q): Query<MetricsQuery>,
) -> Result<Response, StatusCode> {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();

    // Normalize format string
    let fmt = q.format.as_deref().unwrap_or("json").to_ascii_lowercase();

    match fmt.as_str() {
        "json" => {
            // Snapshot with optional reset
            let metric_sets = if q.reset {
                collect_metrics_snapshot_and_reset(&state.metrics_registry)
            } else {
                collect_metrics_snapshot(&state.metrics_registry)
            };

            let response = MetricsResponse {
                timestamp,
                metric_sets,
            };

            Ok(Json(response).into_response())
        }
        "json_compact" | "compact" => {
            let metric_sets = if q.reset {
                collect_compact_snapshot_and_reset(&state.metrics_registry)
            } else {
                collect_compact_snapshot(&state.metrics_registry)
            };

            let response = CompactMetricsResponse { timestamp, metric_sets };
            Ok(Json(response).into_response())
        }
        "line" | "line_protocol" | "lp" => {
            let body = if q.reset {
                format_line_protocol(&state.metrics_registry, true, Some(now.timestamp_millis()))
            } else {
                format_line_protocol(&state.metrics_registry, false, Some(now.timestamp_millis()))
            };
            let mut resp = body.into_response();
            let _ = resp.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/plain; charset=utf-8"));
            Ok(resp)
        }
        "prometheus" | "prom" => {
            let body = if q.reset {
                format_prometheus_text(&state.metrics_registry, true, Some(now.timestamp_millis()))
            } else {
                format_prometheus_text(&state.metrics_registry, false, Some(now.timestamp_millis()))
            };
            let mut resp = body.into_response();
            // Prometheus text exposition format 0.0.4
            let _ = resp.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"));
            Ok(resp)
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

/// Handler for the /telemetry/metrics/aggregate endpoint.
/// Aggregates metrics by metric set name and optionally by a list of attributes.
async fn get_metrics_aggregate(
    State(state): State<AppState>,
    Query(q): Query<AggregateQuery>,
) -> Result<Response, StatusCode> {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();

    // Parse attribute list (comma-separated), trim whitespace, drop empties
    let group_attrs: Vec<String> = q
        .attrs
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    // Aggregate groups with or without reset
    let groups = aggregate_metric_groups(&state.metrics_registry, q.reset, &group_attrs);

    // Normalize format string
    let fmt = q.format.as_deref().unwrap_or("json").to_ascii_lowercase();

    match fmt.as_str() {
        "json" => {
            let response = MetricsResponse {
                timestamp,
                metric_sets: groups_to_full_json(&groups),
            };
            Ok(Json(response).into_response())
        }
        "json_compact" | "compact" => {
            let response = CompactMetricsResponse {
                timestamp,
                metric_sets: groups_to_compact_json(&groups),
            };
            Ok(Json(response).into_response())
        }
        "line" | "line_protocol" | "lp" => {
            let body = format_aggregate_line_protocol(&groups, Some(now.timestamp_millis()));
            let mut resp = body.into_response();
            let _ = resp.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/plain; charset=utf-8"));
            Ok(resp)
        }
        "prometheus" | "prom" => {
            let body = format_aggregate_prometheus_text(&groups, Some(now.timestamp_millis()));
            let mut resp = body.into_response();
            let _ = resp.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"));
            Ok(resp)
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

// --- Aggregation helpers for groups ---

/// Internal representation of an aggregated group.
struct AggregateGroup {
    /// Metric set name (descriptor name)
    name: String,
    /// Descriptor for retrieving metric metadata
    descriptor: &'static crate::descriptor::MetricsDescriptor,
    /// Selected attributes for this group
    attributes: HashMap<String, Value>,
    /// Aggregated metrics by field name
    metrics: HashMap<String, u64>,
}

fn aggregate_metric_groups(
    registry: &MetricsRegistryHandle,
    reset: bool,
    group_attrs: &[String],
) -> Vec<AggregateGroup> {
    use std::collections::hash_map::Entry;

    // Aggregation map keyed by (set name, sorted list of (attr, Option<val_string>))
    type GroupKey = (String, Vec<(String, Option<String>)>);
    let mut agg: HashMap<GroupKey, (HashMap<String, Value>, HashMap<String, u64>, &'static crate::descriptor::MetricsDescriptor)> = HashMap::new();

    let mut visit = |descriptor: &'static crate::descriptor::MetricsDescriptor,
                     attributes: &dyn crate::attributes::AttributeSetHandler,
                     metrics_iter: crate::registry::NonZeroMetrics<'_>| {
        // Build key attributes vector
        let mut key_attrs: Vec<(String, Option<String>)> = Vec::new();
        if !group_attrs.is_empty() {
            let mut amap: HashMap<&str, String> = HashMap::new();
            for (k, v) in attributes.iter_attributes() {
                let _ = amap.insert(k, v.to_string_value());
            }
            for gk in group_attrs {
                let val_opt = amap.get(gk.as_str()).cloned();
                key_attrs.push((gk.clone(), val_opt));
            }
        }
        key_attrs.sort_by(|a, b| a.0.cmp(&b.0));

        // Prepare group entry
        let attrs_and_metrics = match agg.entry((descriptor.name.to_string(), key_attrs)) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let mut attrs_map = HashMap::new();
                if !group_attrs.is_empty() {
                    let mut avals: HashMap<&str, &AttributeValue> = HashMap::new();
                    for (k, val) in attributes.iter_attributes() {
                        let _ = avals.insert(k, val);
                    }
                    for gk in group_attrs {
                        if let Some(v) = avals.get(gk.as_str()) {
                            let _ = attrs_map.insert(gk.clone(), attribute_value_to_json(*v));
                        }
                    }
                }
                v.insert((attrs_map, HashMap::new(), descriptor))
            }
        };

        let (_attrs_map, metrics_map, _desc) = attrs_and_metrics;

        // Accumulate metrics
        for (field, value) in metrics_iter {
            match metrics_map.entry(field.name.to_string()) {
                Entry::Occupied(mut e) => {
                    let _ = e.insert(e.get().saturating_add(value));
                }
                Entry::Vacant(e) => {
                    let _ = e.insert(value);
                }
            }
        }
    };

    if reset {
        registry.visit_non_zero_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        registry.visit_current_metrics(|d, a, m| visit(d, a, m));
    }

    // Convert to vector
    let mut groups: Vec<AggregateGroup> = Vec::with_capacity(agg.len());
    for ((set_name, _), (attrs_map, metrics_map, desc)) in agg.into_iter() {
        groups.push(AggregateGroup {
            name: set_name,
            descriptor: desc,
            attributes: attrs_map,
            metrics: metrics_map,
        });
    }

    // Stable sort by set name then number of metrics
    groups.sort_by(|a, b| {
        let ord = a.name.cmp(&b.name);
        if ord == std::cmp::Ordering::Equal {
            a.metrics.len().cmp(&b.metrics.len())
        } else {
            ord
        }
    });

    groups
}

fn groups_to_full_json(groups: &[AggregateGroup]) -> Vec<MetricSetJson> {
    let mut out = Vec::with_capacity(groups.len());
    for g in groups {
        // Build metrics vector using descriptor metadata where available
        let mut metrics = Vec::with_capacity(g.metrics.len());
        for field in g.descriptor.metrics.iter() {
            if let Some(val) = g.metrics.get(field.name) {
                metrics.push(MetricJson {
                    name: field.name.to_string(),
                    description: field.brief.to_string(),
                    unit: field.unit.to_string(),
                    value: *val,
                    instrument_type: format!("{:?}", field.instrument),
                });
            }
        }
        if !metrics.is_empty() {
            out.push(MetricSetJson {
                name: g.name.clone(),
                description: "".to_string(),
                attributes: g.attributes.clone(),
                metrics,
            });
        }
    }
    out
}

fn groups_to_compact_json(groups: &[AggregateGroup]) -> Vec<CompactMetricSetJson> {
    let mut out = Vec::with_capacity(groups.len());
    for g in groups {
        out.push(CompactMetricSetJson {
            name: g.name.clone(),
            attributes: g.attributes.clone(),
            metrics: g.metrics.clone(),
        });
    }
    out
}

fn format_aggregate_line_protocol(groups: &[AggregateGroup], timestamp_millis: Option<i64>) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis.map(|ms| format!(" {}", ms)).unwrap_or_default();

    for g in groups {
        let measurement = escape_lp_measurement(&g.name);
        // Tags from grouped attributes only
        let mut tags = String::new();
        for (k, v) in &g.attributes {
            let _ = write!(
                &mut tags,
                ",{}={}",
                escape_lp_tag_key(k),
                escape_lp_tag_value(&v.to_string())
            );
        }
        let mut fields = String::new();
        let mut first = true;
        for (fname, val) in &g.metrics {
            if first { first = false; } else { fields.push(','); }
            let _ = write!(&mut fields, "{}={}i", escape_lp_field_key(fname), val);
        }
        if !fields.is_empty() {
            let _ = writeln!(&mut out, "{}{} {}{}", measurement, tags, fields, ts_suffix);
        }
    }

    out
}

fn format_aggregate_prometheus_text(groups: &[AggregateGroup], timestamp_millis: Option<i64>) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis.map(|ms| format!(" {}", ms)).unwrap_or_default();
    let mut seen: HashSet<String> = HashSet::new();

    for g in groups {
        // Base labels include set name and selected attributes
        let mut base_labels = String::new();
        if !g.name.is_empty() {
            let _ = write!(&mut base_labels, "set=\"{}\"", escape_prom_label_value(&g.name));
        }
        // ensure deterministic order of attributes in output
        let mut attrs: Vec<(&String, &Value)> = g.attributes.iter().collect();
        attrs.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in attrs {
            if !base_labels.is_empty() { base_labels.push(','); }
            let _ = write!(
                &mut base_labels,
                "{}=\"{}\"",
                sanitize_prom_label_key(k),
                escape_prom_label_value(&v.to_string())
            );
        }

        // Emit metrics for this group
        for field in g.descriptor.metrics.iter() {
            if let Some(value) = g.metrics.get(field.name) {
                let metric_name = sanitize_prom_metric_name(field.name);
                if seen.insert(metric_name.clone()) {
                    if !field.brief.is_empty() {
                        let _ = writeln!(&mut out, "# HELP {} {}", metric_name, escape_prom_help(field.brief));
                    }
                    let prom_type = match field.instrument {
                        Instrument::Counter => "counter",
                        Instrument::UpDownCounter => "gauge",
                        Instrument::Gauge => "gauge",
                        Instrument::Histogram => "gauge",
                    };
                    let _ = writeln!(&mut out, "# TYPE {} {}", metric_name, prom_type);
                }
                if base_labels.is_empty() {
                    let _ = writeln!(&mut out, "{} {}{}", metric_name, value, ts_suffix);
                } else {
                    let _ = writeln!(&mut out, "{}{{{}}} {}{}", metric_name, base_labels, value, ts_suffix);
                }
            }
        }
    }

    out
}

// ---- Backward-compatible helpers used by /telemetry/metrics ----

/// Handler for the /health endpoint - simple health check.
async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "telemetry-server"
    }))
}

/// Collects a snapshot of current metrics without resetting them.
fn collect_metrics_snapshot(registry: &MetricsRegistryHandle) -> Vec<MetricSetJson> {
    let mut metric_sets = Vec::new();

    registry.visit_current_metrics(|descriptor, attributes, metrics_iter| {
        let mut metrics = Vec::new();

        for (field, value) in metrics_iter {
            metrics.push(MetricJson {
                name: field.name.to_string(),
                description: field.brief.to_string(),
                unit: field.unit.to_string(),
                value,
                instrument_type: format!("{:?}", field.instrument),
            });
        }

        if !metrics.is_empty() {
            // Convert attributes to HashMap using the iterator
            let mut attrs_map = HashMap::new();
            for (key, value) in attributes.iter_attributes() {
                let json_value = attribute_value_to_json(value);
                let _ = attrs_map.insert(key.to_string(), json_value);
            }

            metric_sets.push(MetricSetJson {
                name: descriptor.name.to_string(),
                description: "".to_string(), // MetricsDescriptor doesn't have description field
                attributes: attrs_map,
                metrics,
            });
        }
    });

    metric_sets
}

/// Collects a snapshot of current metrics and resets them afterwards.
fn collect_metrics_snapshot_and_reset(registry: &MetricsRegistryHandle) -> Vec<MetricSetJson> {
    let mut metric_sets = Vec::new();

    registry.visit_non_zero_metrics_and_reset(|descriptor, attributes, metrics_iter| {
        let mut metrics = Vec::new();

        for (field, value) in metrics_iter {
            metrics.push(MetricJson {
                name: field.name.to_string(),
                description: field.brief.to_string(),
                unit: field.unit.to_string(),
                value,
                instrument_type: format!("{:?}", field.instrument),
            });
        }

        if !metrics.is_empty() {
            let mut attrs_map = HashMap::new();
            for (key, value) in attributes.iter_attributes() {
                let json_value = attribute_value_to_json(value);
                let _ = attrs_map.insert(key.to_string(), json_value);
            }

            metric_sets.push(MetricSetJson {
                name: descriptor.name.to_string(),
                description: "".to_string(),
                attributes: attrs_map,
                metrics,
            });
        }
    });

    metric_sets
}

/// Compact snapshot without resetting.
fn collect_compact_snapshot(registry: &MetricsRegistryHandle) -> Vec<CompactMetricSetJson> {
    let mut metric_sets = Vec::new();

    registry.visit_current_metrics(|descriptor, attributes, metrics_iter| {
        let mut metrics = HashMap::new();
        for (field, value) in metrics_iter {
            let _ = metrics.insert(field.name.to_string(), value);
        }

        if !metrics.is_empty() {
            // include attributes in compact format
            let mut attrs_map = HashMap::new();
            for (key, value) in attributes.iter_attributes() {
                let json_value = attribute_value_to_json(value);
                let _ = attrs_map.insert(key.to_string(), json_value);
            }

            metric_sets.push(CompactMetricSetJson {
                name: descriptor.name.to_string(),
                attributes: attrs_map,
                metrics,
            });
        }
    });

    metric_sets
}

/// Compact snapshot with resetting.
fn collect_compact_snapshot_and_reset(registry: &MetricsRegistryHandle) -> Vec<CompactMetricSetJson> {
    let mut metric_sets = Vec::new();

    registry.visit_non_zero_metrics_and_reset(|descriptor, attributes, metrics_iter| {
        let mut metrics = HashMap::new();
        for (field, value) in metrics_iter {
            let _ = metrics.insert(field.name.to_string(), value);
        }

        if !metrics.is_empty() {
            let mut attrs_map = HashMap::new();
            for (key, value) in attributes.iter_attributes() {
                let json_value = attribute_value_to_json(value);
                let _ = attrs_map.insert(key.to_string(), json_value);
            }

            metric_sets.push(CompactMetricSetJson {
                name: descriptor.name.to_string(),
                attributes: attrs_map,
                metrics,
            });
        }
    });

    metric_sets
}

/// Converts an AttributeValue to a JSON Value.
fn attribute_value_to_json(attr_value: &AttributeValue) -> Value {
    match attr_value {
        AttributeValue::String(s) => Value::String(s.clone()),
        AttributeValue::Int(i) => Value::Number((*i).into()),
        AttributeValue::UInt(u) => Value::Number((*u).into()),
        AttributeValue::Double(f) => {
            match serde_json::Number::from_f64(*f) {
                Some(num) => Value::Number(num),
                None => Value::Null, // Handle NaN or infinite values
            }
        }
        AttributeValue::Boolean(b) => Value::Bool(*b),
    }
}

// ---- Text formatters for /telemetry/metrics ----

fn format_line_protocol(
    registry: &MetricsRegistryHandle,
    reset: bool,
    timestamp_millis: Option<i64>,
) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis.map(|ms| format!(" {}", ms)).unwrap_or_default();

    let mut visit = |descriptor: &'static crate::descriptor::MetricsDescriptor,
                     attributes: &dyn crate::attributes::AttributeSetHandler,
                     metrics_iter: crate::registry::NonZeroMetrics<'_>| {
        // Measurement is the metric set name when available; fallback to "metrics".
        let measurement_name = if descriptor.name.is_empty() { "metrics" } else { descriptor.name };
        let measurement = escape_lp_measurement(measurement_name);

        // Tags from attributes only.
        let mut tags = String::new();
        for (key, value) in attributes.iter_attributes() {
            let _ = write!(
                &mut tags,
                ",{}={}",
                escape_lp_tag_key(key),
                escape_lp_tag_value(&value.to_string_value())
            );
        }

        // Collect fields for this metric set into a single line
        let mut fields = String::new();
        let mut first = true;
        for (field, value) in metrics_iter {
            if first { first = false; } else { fields.push(','); }
            let _ = write!(
                &mut fields,
                "{}={}i",
                escape_lp_field_key(field.name),
                value
            );
        }

        if !fields.is_empty() {
            let _ = writeln!(
                &mut out,
                "{}{} {}{}",
                measurement,
                tags,
                fields,
                ts_suffix
            );
        }
    };

    if reset {
        registry.visit_non_zero_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        registry.visit_current_metrics(|d, a, m| visit(d, a, m));
    }

    out
}

fn format_prometheus_text(
    registry: &MetricsRegistryHandle,
    reset: bool,
    timestamp_millis: Option<i64>,
) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis.map(|ms| format!(" {}", ms)).unwrap_or_default();
    let mut seen: HashSet<String> = HashSet::new();

    let mut visit = |descriptor: &'static crate::descriptor::MetricsDescriptor,
                     attributes: &dyn crate::attributes::AttributeSetHandler,
                     metrics_iter: crate::registry::NonZeroMetrics<'_>| {
        // Render labels from attributes + set label
        let mut base_labels = String::new();
        if !descriptor.name.is_empty() {
            let _ = write!(
                &mut base_labels,
                "set=\"{}\"",
                escape_prom_label_value(descriptor.name)
            );
        }
        for (key, value) in attributes.iter_attributes() {
            if !base_labels.is_empty() { base_labels.push(','); }
            let _ = write!(
                &mut base_labels,
                "{}=\"{}\"",
                sanitize_prom_label_key(key),
                escape_prom_label_value(&value.to_string_value())
            );
        }

        for (field, value) in metrics_iter {
            let metric_name = sanitize_prom_metric_name(field.name);

            // HELP/TYPE once per metric name
            if seen.insert(metric_name.clone()) {
                if !field.brief.is_empty() {
                    let _ = writeln!(&mut out, "# HELP {} {}", metric_name, escape_prom_help(field.brief));
                }
                let prom_type = match field.instrument {
                    Instrument::Counter => "counter",
                    Instrument::UpDownCounter => "gauge",
                    Instrument::Gauge => "gauge",
                    Instrument::Histogram => "gauge",
                };
                let _ = writeln!(&mut out, "# TYPE {} {}", metric_name, prom_type);
            }

            if base_labels.is_empty() {
                let _ = writeln!(&mut out, "{} {}{}", metric_name, value, ts_suffix);
            } else {
                let _ = writeln!(&mut out, "{}{{{}}} {}{}", metric_name, base_labels, value, ts_suffix);
            }
        }
    };

    if reset {
        registry.visit_non_zero_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        registry.visit_current_metrics(|d, a, m| visit(d, a, m));
    }

    out
}

// ---- Helpers: escaping/sanitization ----

fn escape_lp_measurement(s: &str) -> String {
    s.replace(',', "\\,").replace(' ', "\\ ")
}

fn escape_lp_tag_key(s: &str) -> String {
    s.replace(',', "\\,").replace(' ', "\\ ").replace('=', "\\=")
}

fn escape_lp_tag_value(s: &str) -> String {
    s.replace(',', "\\,").replace(' ', "\\ ").replace('=', "\\=")
}

fn escape_lp_field_key(s: &str) -> String {
    // Same escaping rules as tag key for spaces/commas/equals
    escape_lp_tag_key(s)
}

fn sanitize_prom_metric_name(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, ch) in s.chars().enumerate() {
        let ok = match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':' => true,
            _ => false,
        };
        if ok && !(i == 0 && ch.is_ascii_digit()) {
            out.push(ch);
        } else if ch == '.' || ch == '-' || ch == ' ' {
            out.push('_');
        } else if i == 0 && ch.is_ascii_digit() {
            out.push('_');
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() { "metric".to_string() } else { out }
}

fn sanitize_prom_label_key(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, ch) in s.chars().enumerate() {
        let ok = match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':' => true,
            _ => false,
        };
        if ok && !(i == 0 && ch.is_ascii_digit()) {
            out.push(ch);
        } else if ch == '.' || ch == '-' || ch == ' ' {
            out.push('_');
        } else if i == 0 && ch.is_ascii_digit() {
            out.push('_');
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() { "label".to_string() } else { out }
}

fn escape_prom_label_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => { out.push('\\'); out.push('\\'); }
            '"' => { out.push('\\'); out.push('"'); }
            '\n' => { out.push('\\'); out.push('n'); }
            _ => out.push(ch),
        }
    }
    out
}

fn escape_prom_help(s: &str) -> String {
    // Similar escaping to label value per Prometheus recommendations
    escape_prom_label_value(s)
}
