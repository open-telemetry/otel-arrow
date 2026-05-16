// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry endpoints.
//!
//! - /api/v1/telemetry/live-schema - current semantic conventions registry
//! - /api/v1/telemetry/logs - retained internal logs from the in-memory log tap
//! - /api/v1/telemetry/logs/stream - live internal log stream over WebSocket
//! - /api/v1/telemetry/metrics - current aggregated metrics in JSON, line protocol, or Prometheus text format
//! - /api/v1/telemetry/metrics/aggregate - aggregated metrics grouped by metric set name and optional attributes

use crate::AppState;
use crate::convert::json_shape;
#[cfg(feature = "live-logs-ws")]
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use otap_df_admin_types::telemetry as api;
use otap_df_config::pipeline::telemetry::AttributeValue as ResourceAttributeValue;
use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
use otap_df_telemetry::descriptor::{Instrument, MetricValueType, MetricsDescriptor, MetricsField};
use otap_df_telemetry::event::LogEvent;
use otap_df_telemetry::log_tap::{LogQuery, LogQueryResult, RetainedLogEvent};
use otap_df_telemetry::metrics::{MetricValue, MetricsIterator};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::self_tracing::format_log_record_to_string;
use otap_df_telemetry::semconv::SemConvRegistry;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
#[cfg(feature = "live-logs-ws")]
use std::sync::Arc;
#[cfg(feature = "live-logs-ws")]
use tokio::sync::broadcast;

/// All the routes for telemetry.
pub(crate) fn routes() -> Router<AppState> {
    let router = Router::new()
        .route("/telemetry/live-schema", get(get_live_schema))
        .route("/telemetry/logs", get(get_logs))
        .route("/telemetry/metrics", get(get_metrics))
        .route("/telemetry/metrics/aggregate", get(get_metrics_aggregate))
        .route("/metrics", get(get_metrics));

    #[cfg(feature = "live-logs-ws")]
    let router = router.route("/telemetry/logs/stream", get(ws_logs_stream));

    router
}

/// All metric sets.
#[derive(Serialize)]
struct MetricsWithMetadata {
    /// Timestamp when the metrics were collected.
    timestamp: String,
    /// List of metric sets with their values.
    metric_sets: Vec<MetricSetWithMetadata>,
}

/// Metric set (aka multivariate metrics).
#[derive(Serialize)]
struct MetricSetWithMetadata {
    /// Name of the metric set.
    name: String,
    /// Brief of the metric set.
    brief: String,
    /// Attributes associated with this metric set.
    attributes: HashMap<String, AttributeValue>,
    /// Individual metrics within this set.
    metrics: Vec<MetricDataPointWithMetadata>,
}

/// Metric data point with metadata.
#[derive(Serialize)]
struct MetricDataPointWithMetadata {
    /// Descriptor for retrieving metric metadata
    #[serde(flatten)]
    metadata: MetricsField,
    /// Current value.
    value: MetricValue,
}

/// Container of all aggregated metrics (no metadata).
#[derive(Serialize)]
struct AllMetrics {
    timestamp: String,
    metric_sets: Vec<MetricSet>,
}

#[derive(Serialize)]
struct MetricSet {
    name: String,
    attributes: HashMap<String, AttributeValue>,
    metrics: HashMap<String, MetricValue>,
}

type LogsQuery = api::LogsQuery;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OutputFormat {
    Json,
    JsonCompact,
    LineProtocol,
    #[default]
    Prometheus,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct MetricsQuery {
    #[serde(default)]
    reset: bool,
    #[serde(default)]
    format: Option<OutputFormat>,
    #[serde(default)]
    keep_all_zeroes: bool,
}

impl MetricsQuery {
    #[must_use]
    fn output_format(&self) -> OutputFormat {
        self.format.unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct AggregateQuery {
    #[serde(default)]
    reset: bool,
    #[serde(default)]
    attrs: Option<String>,
    #[serde(default)]
    format: Option<OutputFormat>,
}

impl AggregateQuery {
    #[must_use]
    fn output_format(&self) -> OutputFormat {
        self.format.unwrap_or_default()
    }
}

/// Internal representation of an aggregated group.
struct AggregateGroup {
    /// Metric set name (descriptor name)
    name: String,
    /// Descriptor for retrieving metric metadata
    brief: &'static MetricsDescriptor,
    /// Selected attributes for this group
    attributes: HashMap<String, AttributeValue>,
    /// Aggregated metrics by field name
    metrics: HashMap<String, MetricValue>,
}

#[derive(Serialize)]
pub(crate) struct LogsResponse {
    oldest_seq: Option<u64>,
    newest_seq: Option<u64>,
    next_seq: u64,
    truncated_before_seq: Option<u64>,
    dropped_on_ingest: u64,
    dropped_on_retention: u64,
    retained_bytes: usize,
    logs: Vec<LogEntry>,
}

#[derive(Serialize)]
struct LogEntry {
    seq: u64,
    timestamp: String,
    level: String,
    target: String,
    event_name: String,
    file: Option<String>,
    line: Option<u32>,
    rendered: String,
    contexts: Vec<ResolvedLogContext>,
}

#[derive(Serialize)]
struct ResolvedLogContext {
    entity_key: String,
    schema_name: Option<String>,
    attributes: HashMap<String, AttributeValue>,
}

fn logs_response(registry: &TelemetryRegistryHandle, result: LogQueryResult) -> LogsResponse {
    LogsResponse {
        oldest_seq: result.oldest_seq,
        newest_seq: result.newest_seq,
        next_seq: result.next_seq,
        truncated_before_seq: result.truncated_before_seq,
        dropped_on_ingest: result.dropped_on_ingest,
        dropped_on_retention: result.dropped_on_retention,
        retained_bytes: result.retained_bytes,
        logs: result
            .logs
            .iter()
            .map(|entry| render_log_entry(registry, entry))
            .collect(),
    }
}

fn render_log_entry(registry: &TelemetryRegistryHandle, entry: &RetainedLogEvent) -> LogEntry {
    let callsite = entry.event.record.callsite();
    LogEntry {
        seq: entry.seq,
        timestamp: chrono::DateTime::<chrono::Utc>::from(entry.event.time).to_rfc3339(),
        level: callsite.level().to_string(),
        target: callsite.target().to_string(),
        event_name: callsite.name().to_string(),
        file: callsite.file().map(str::to_string),
        line: callsite.line(),
        rendered: render_log_message(&entry.event),
        contexts: resolve_log_contexts(registry, &entry.event),
    }
}

fn render_log_message(event: &LogEvent) -> String {
    format_log_record_to_string(Some(event.time), &event.record)
}

fn resolve_log_contexts(
    registry: &TelemetryRegistryHandle,
    event: &LogEvent,
) -> Vec<ResolvedLogContext> {
    event
        .record
        .context
        .iter()
        .map(|entity_key| {
            registry
                .visit_entity(*entity_key, |attrs| ResolvedLogContext {
                    entity_key: format!("{entity_key:?}"),
                    schema_name: Some(attrs.schema_name().to_string()),
                    attributes: attrs
                        .iter_attributes()
                        .map(|(key, value)| (key.to_string(), value.clone()))
                        .collect(),
                })
                .unwrap_or_else(|| ResolvedLogContext {
                    entity_key: format!("{entity_key:?}"),
                    schema_name: None,
                    attributes: HashMap::new(),
                })
        })
        .collect()
}

/// Handler for the /live-schema endpoint.
///
/// This reflects the current live schema of the metrics registry.
pub async fn get_live_schema(
    State(state): State<AppState>,
) -> Result<Json<SemConvRegistry>, StatusCode> {
    Ok(Json(state.metrics_registry.generate_semconv_registry()))
}

/// Handler for the `/telemetry/logs` endpoint.
pub async fn get_logs(
    State(state): State<AppState>,
    Query(q): Query<LogsQuery>,
) -> Result<Json<api::LogsResponse>, StatusCode> {
    let Some(log_tap) = state.log_tap.as_ref() else {
        return Err(StatusCode::NOT_FOUND);
    };

    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let result = log_tap.query(LogQuery {
        after: q.after,
        limit,
    });
    Ok(Json(json_shape(&logs_response(
        &state.metrics_registry,
        result,
    ))))
}

/// Handler for the `/api/v1/telemetry/metrics` endpoint.
/// Supports multiple output formats and optional reset.
///
/// Query parameters:
/// - `reset` (bool, default false): whether to reset metrics after reading.
/// - `format` (string, default "prometheus"): output format, one of "json", "json_compact", "line_protocol", "prometheus".
async fn get_metrics(
    State(state): State<AppState>,
    Query(q): Query<MetricsQuery>,
) -> Result<Response, StatusCode> {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();

    let fmt = q.output_format();

    match fmt {
        OutputFormat::Json => {
            // Snapshot with optional reset
            let metric_sets = if q.reset {
                collect_metrics_snapshot_and_reset(&state.metrics_registry, q.keep_all_zeroes)
            } else {
                collect_metrics_snapshot(&state.metrics_registry, q.keep_all_zeroes)
            };

            let response = MetricsWithMetadata {
                timestamp,
                metric_sets,
            };

            Ok(Json(json_shape::<_, api::MetricsResponse>(&response)).into_response())
        }
        OutputFormat::JsonCompact => {
            let metric_sets = if q.reset {
                collect_compact_snapshot_and_reset(&state.metrics_registry)
            } else {
                collect_compact_snapshot(&state.metrics_registry)
            };

            let response = AllMetrics {
                timestamp,
                metric_sets,
            };
            Ok(Json(json_shape::<_, api::CompactMetricsResponse>(&response)).into_response())
        }
        OutputFormat::LineProtocol => {
            let body = if q.reset {
                format_line_protocol(&state.metrics_registry, true, Some(now.timestamp_millis()))
            } else {
                format_line_protocol(&state.metrics_registry, false, Some(now.timestamp_millis()))
            };
            let mut resp = body.into_response();
            let _ = resp.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            Ok(resp)
        }
        OutputFormat::Prometheus => {
            let body = if q.reset {
                format_prometheus_text(
                    &state.metrics_registry,
                    true,
                    Some(now.timestamp_millis()),
                    &state.target_info,
                )
            } else {
                format_prometheus_text(
                    &state.metrics_registry,
                    false,
                    Some(now.timestamp_millis()),
                    &state.target_info,
                )
            };
            let mut resp = body.into_response();
            // Prometheus text exposition format 0.0.4
            let _ = resp.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
            );
            Ok(resp)
        }
    }
}

/// Handler for the /api/v1/telemetry/metrics/aggregate endpoint.
/// Aggregates metrics by metric set name and optionally by a list of attributes.
///
/// Query parameters:
/// - `reset` (bool, default false): whether to reset metrics after reading.
/// - `attrs` (string, optional): comma-separated list of attribute names to group by.
/// - `format` (string, default "json"): output format, one of "json", "json_compact", "line_protocol", "prometheus".
pub async fn get_metrics_aggregate(
    State(state): State<AppState>,
    Query(q): Query<AggregateQuery>,
) -> Result<Response, StatusCode> {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();

    // Parse attribute list (comma-separated), trim whitespace, drop empties
    let group_attrs: Vec<_> = q
        .attrs
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Aggregate groups with or without reset
    let groups = aggregate_metric_groups(&state.metrics_registry, q.reset, &group_attrs);

    let fmt = q.output_format();

    match fmt {
        OutputFormat::Json => {
            let response = MetricsWithMetadata {
                timestamp,
                metric_sets: groups_with_metadata(&groups),
            };
            Ok(Json(json_shape::<_, api::MetricsResponse>(&response)).into_response())
        }
        OutputFormat::JsonCompact => {
            let response = AllMetrics {
                timestamp,
                metric_sets: groups_without_metadata(&groups),
            };
            Ok(Json(json_shape::<_, api::CompactMetricsResponse>(&response)).into_response())
        }
        OutputFormat::LineProtocol => {
            let body = agg_line_protocol_text(&groups, Some(now.timestamp_millis()));
            let mut resp = body.into_response();
            let _ = resp.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            Ok(resp)
        }
        OutputFormat::Prometheus => {
            let body =
                agg_prometheus_text(&groups, Some(now.timestamp_millis()), &state.target_info);
            let mut resp = body.into_response();
            let _ = resp.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
            );
            Ok(resp)
        }
    }
}

fn aggregate_metric_groups(
    telemetry_registry: &TelemetryRegistryHandle,
    reset: bool,
    group_attrs: &[&str],
) -> Vec<AggregateGroup> {
    // Aggregation map keyed by (set name, sorted list of (attr, Option<val_string>))
    type GroupKey = (String, Vec<(String, Option<String>)>);
    let mut agg: HashMap<
        GroupKey,
        (
            HashMap<String, AttributeValue>,
            HashMap<String, MetricValue>,
            &'static MetricsDescriptor,
        ),
    > = HashMap::new();

    // Build a filter set once (if grouping is requested)
    let group_filter: Option<HashSet<&str>> = if group_attrs.is_empty() {
        None
    } else {
        Some(group_attrs.iter().copied().collect())
    };

    let mut visit = |descriptor: &'static MetricsDescriptor,
                     attributes: &dyn AttributeSetHandler,
                     metrics_iter: MetricsIterator<'_>| {
        // Single-pass collection of only the grouped attributes
        let mut selected: HashMap<&str, (String, AttributeValue)> = HashMap::new();
        if let Some(filter) = &group_filter {
            for (k, v) in attributes.iter_attributes() {
                if filter.contains(k) {
                    let _ = selected.insert(k, (v.to_string_value(), v.clone()));
                }
            }
        }

        // Build key attributes vector (name + optional string value)
        let mut key_attrs: Vec<(String, Option<String>)> = Vec::new();
        if !group_attrs.is_empty() {
            key_attrs.reserve(group_attrs.len());
            for gk in group_attrs {
                let val_opt = selected.get(gk).map(|(s, _)| s.clone());
                key_attrs.push(((*gk).to_string(), val_opt));
            }
            key_attrs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        }

        // Prepare group entry
        let attrs_and_metrics = match agg.entry((descriptor.name.to_string(), key_attrs)) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                // Build attributes map for this group using the already-selected values
                let mut attrs_map = HashMap::new();
                if !group_attrs.is_empty() {
                    attrs_map.reserve(selected.len());
                    for gk in group_attrs {
                        if let Some((_, attr_val)) = selected.get(gk) {
                            let _ = attrs_map.insert((*gk).to_string(), attr_val.clone());
                        }
                    }
                }
                v.insert((attrs_map, HashMap::new(), descriptor))
            }
        };

        let (_attrs_map, metrics_map, _desc) = attrs_and_metrics;

        // Accumulate metrics
        for (field, value) in metrics_iter {
            let _ = metrics_map
                .entry(field.name.to_string())
                .and_modify(|existing| existing.add_in_place(value))
                .or_insert(value);
        }
    };

    if reset {
        telemetry_registry.visit_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        telemetry_registry.visit_current_metrics(|d, a, m| visit(d, a, m));
    }

    // Convert to vector
    let mut groups: Vec<AggregateGroup> = Vec::with_capacity(agg.len());
    for ((set_name, _), (attrs_map, metrics_map, desc)) in agg {
        groups.push(AggregateGroup {
            name: set_name,
            brief: desc,
            attributes: attrs_map,
            metrics: metrics_map,
        });
    }

    // Unstable sort by set name, then number of metrics
    groups.sort_unstable_by(|a, b| {
        let ord = a.name.cmp(&b.name);
        if ord == std::cmp::Ordering::Equal {
            a.metrics.len().cmp(&b.metrics.len())
        } else {
            ord
        }
    });

    groups
}

fn groups_with_metadata(groups: &[AggregateGroup]) -> Vec<MetricSetWithMetadata> {
    let mut out = Vec::with_capacity(groups.len());
    for g in groups {
        // Build metrics vector using descriptor metadata where available
        let mut metrics = Vec::with_capacity(g.metrics.len());
        for field in g.brief.metrics.iter() {
            if let Some(val) = g.metrics.get(field.name) {
                metrics.push(MetricDataPointWithMetadata {
                    metadata: *field,
                    value: *val,
                });
            }
        }
        if !metrics.is_empty() {
            out.push(MetricSetWithMetadata {
                name: g.name.clone(),
                brief: String::new(),
                attributes: g.attributes.clone(),
                metrics,
            });
        }
    }
    out
}

fn groups_without_metadata(groups: &[AggregateGroup]) -> Vec<MetricSet> {
    let mut out = Vec::with_capacity(groups.len());
    for g in groups {
        out.push(MetricSet {
            name: g.name.clone(),
            attributes: g.attributes.clone(),
            metrics: g.metrics.clone(),
        });
    }
    out
}

fn format_lp_value(value: MetricValue, value_type: Option<MetricValueType>) -> String {
    match value {
        MetricValue::U64(_) | MetricValue::F64(_) => {
            let vtype = value_type.unwrap_or(match value {
                MetricValue::U64(_) => MetricValueType::U64,
                MetricValue::F64(_) => MetricValueType::F64,
                MetricValue::Mmsc(_) => unreachable!(),
            });
            match vtype {
                MetricValueType::U64 => {
                    let int_val = match value {
                        MetricValue::U64(v) => v,
                        MetricValue::F64(v) => v as u64,
                        MetricValue::Mmsc(_) => unreachable!(),
                    };
                    format!("{int_val}i")
                }
                MetricValueType::F64 => value.to_f64().to_string(),
            }
        }
        // MMSC values are expanded into multiple fields at the call site;
        // this arm should not be reached.
        MetricValue::Mmsc(_) => unreachable!("MMSC values must be expanded at the call site"),
    }
}

/// Metadata for a Prometheus metric family (HELP, UNIT, TYPE directives).
struct PromMetricMetadata {
    help: Option<String>,
    unit: Option<String>,
    prom_type: String,
}

/// A single metric family group: metadata + collected sample lines.
struct PromMetricGroup {
    metadata: PromMetricMetadata,
    samples: Vec<String>,
}

/// Collects metric samples grouped by metric name, preserving insertion order.
/// After all entities are visited, `emit` writes contiguous metric families.
struct PromGroupedMetrics {
    /// Metric names in insertion order.
    order: Vec<String>,
    /// Metric name → group.
    groups: HashMap<String, PromMetricGroup>,
}

impl PromGroupedMetrics {
    fn new() -> Self {
        Self {
            order: Vec::new(),
            groups: HashMap::new(),
        }
    }

    /// Returns a mutable reference to the group for `metric_name`, creating it
    /// with the given metadata factory if it doesn't exist yet.
    fn get_or_insert(
        &mut self,
        metric_name: &str,
        metadata_fn: impl FnOnce() -> PromMetricMetadata,
    ) -> &mut PromMetricGroup {
        match self.groups.entry(metric_name.to_string()) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                self.order.push(metric_name.to_string());
                e.insert(PromMetricGroup {
                    metadata: metadata_fn(),
                    samples: Vec::new(),
                })
            }
        }
    }

    /// Emits all collected metrics as contiguous Prometheus text families.
    fn emit(self, out: &mut String) {
        for name in &self.order {
            if let Some(group) = self.groups.get(name) {
                if let Some(ref help) = group.metadata.help {
                    let _ = writeln!(out, "# HELP {name} {help}");
                }
                if let Some(ref unit) = group.metadata.unit {
                    let _ = writeln!(out, "# UNIT {name} {unit}");
                }
                let _ = writeln!(out, "# TYPE {name} {}", group.metadata.prom_type);
                for sample in &group.samples {
                    out.push_str(sample);
                }
            }
        }
    }
}

fn format_prom_value(value: MetricValue, value_type: Option<MetricValueType>) -> String {
    match value {
        MetricValue::U64(_) | MetricValue::F64(_) => {
            let vtype = value_type.unwrap_or(match value {
                MetricValue::U64(_) => MetricValueType::U64,
                MetricValue::F64(_) => MetricValueType::F64,
                MetricValue::Mmsc(_) => unreachable!(),
            });
            match vtype {
                MetricValueType::U64 => match value {
                    MetricValue::U64(v) => v.to_string(),
                    MetricValue::F64(v) => (v as u64).to_string(),
                    MetricValue::Mmsc(_) => unreachable!(),
                },
                MetricValueType::F64 => value.to_f64().to_string(),
            }
        }
        // MMSC values are expanded into summary lines at the call site;
        // this arm should not be reached.
        MetricValue::Mmsc(_) => unreachable!("MMSC values must be expanded at the call site"),
    }
}

fn agg_line_protocol_text(groups: &[AggregateGroup], timestamp_millis: Option<i64>) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {ms}"))
        .unwrap_or_default();

    for g in groups {
        let measurement = escape_lp_measurement(&g.name);
        // Tags from grouped attributes only
        let mut tags = String::new();
        for (k, v) in &g.attributes {
            let _ = write!(
                &mut tags,
                ",{}={}",
                escape_lp_tag_key(k),
                escape_lp_tag_value(&v.to_string_value())
            );
        }
        let mut fields = String::new();
        let mut first = true;
        for (fname, val) in &g.metrics {
            let field_type = g
                .brief
                .metrics
                .iter()
                .find(|f| f.name == fname.as_str())
                .map(|f| f.value_type);
            match val {
                MetricValue::U64(_) | MetricValue::F64(_) => {
                    if !first {
                        fields.push(',');
                    }
                    first = false;
                    let _ = write!(
                        &mut fields,
                        "{}={}",
                        escape_lp_field_key(fname),
                        format_lp_value(*val, field_type)
                    );
                }
                MetricValue::Mmsc(s) => {
                    if s.count == 0 {
                        continue;
                    }
                    for (suffix, fval) in [("_min", s.min), ("_max", s.max), ("_sum", s.sum)] {
                        if !first {
                            fields.push(',');
                        }
                        first = false;
                        let _ = write!(
                            &mut fields,
                            "{}{}={}",
                            escape_lp_field_key(fname),
                            suffix,
                            fval
                        );
                    }
                    if !first {
                        fields.push(',');
                    }
                    first = false;
                    let _ = write!(
                        &mut fields,
                        "{}_count={}i",
                        escape_lp_field_key(fname),
                        s.count
                    );
                }
            }
        }
        if !fields.is_empty() {
            let _ = writeln!(&mut out, "{measurement}{tags} {fields}{ts_suffix}");
        }
    }

    out
}

/// Collects a single scalar metric sample into the grouped buffer.
fn collect_scalar_metric(
    groups: &mut PromGroupedMetrics,
    field: &MetricsField,
    value: MetricValue,
    base_labels: &str,
    ts_suffix: &str,
) {
    let metric_name = build_prom_metric_name(field.name, field.unit, field.instrument);

    let group = groups.get_or_insert(&metric_name, || {
        let prom_type = match field.instrument {
            Instrument::Counter => "counter",
            Instrument::UpDownCounter => "gauge",
            Instrument::Gauge => "gauge",
            // `Instrument::Histogram` reaches this path with a scalar
            // `U64`/`F64` value because the telemetry registry does not yet
            // store pre-aggregated bucket data — see the matching TODO in the
            // dispatcher's `add_opentelemetry_metric`. The stored scalar is
            // a single observation (whatever the metric set's
            // `snapshot_values()` returns); buckets/sum/count exist only
            // downstream inside the OTel SDK, not here.
            //
            // Rendering as the Prometheus histogram family
            // (`_bucket{le=...}`/`_sum`/`_count`) would require fabricating
            // bucket data we don't have, so we emit a `gauge` reflecting the
            // raw stored scalar. This is a known limitation: not spec-compliant
            // for OTel Histograms (the spec mandates the histogram family) and
            // potentially misleading because the gauge value's meaning depends
            // on what the producer chose to put in `snapshot_values()`.
            // Proper handling requires extending `MetricValue` with a variant
            // carrying buckets/sum/count.
            Instrument::Histogram => "gauge",
            Instrument::Mmsc => unreachable!("MMSC is not a scalar"),
        };
        PromMetricMetadata {
            help: if field.brief.is_empty() {
                None
            } else {
                Some(escape_prom_help(field.brief))
            },
            unit: ucum_to_prometheus_unit(field.unit).map(|u| u.to_string()),
            prom_type: prom_type.to_string(),
        }
    });
    let mut sample = String::new();
    let value_str = format_prom_value(value, Some(field.value_type));
    emit_sample_line(
        &mut sample,
        &metric_name,
        base_labels,
        &value_str,
        ts_suffix,
    );
    group.samples.push(sample);
}

/// Collects MMSC (min/max/sum/count) sub-metrics into the grouped buffer.
fn collect_mmsc_metric(
    groups: &mut PromGroupedMetrics,
    field: &MetricsField,
    s: &otap_df_telemetry::instrument::MmscSnapshot,
    base_labels: &str,
    ts_suffix: &str,
) {
    if s.count == 0 {
        return;
    }
    let base_metric_name = build_prom_metric_name(field.name, field.unit, Instrument::Gauge);
    let brief = if field.brief.is_empty() {
        None
    } else {
        Some(escape_prom_help(field.brief))
    };
    let unit_word = ucum_to_prometheus_unit(field.unit).map(|u| u.to_string());

    // _min and _max as gauges
    for (suffix, prom_type, val) in [("_min", "gauge", s.min), ("_max", "gauge", s.max)] {
        let sub_name = format!("{base_metric_name}{suffix}");
        let group = groups.get_or_insert(&sub_name, || PromMetricMetadata {
            help: brief.clone(),
            unit: unit_word.clone(),
            prom_type: prom_type.to_string(),
        });
        let mut sample = String::new();
        emit_sample_line(
            &mut sample,
            &sub_name,
            base_labels,
            &format!("{val}"),
            ts_suffix,
        );
        group.samples.push(sample);
    }

    // _sum uses same unit-bearing base name (histogram-family convention)
    let sum_name = format!("{base_metric_name}_sum");
    {
        let group = groups.get_or_insert(&sum_name, || PromMetricMetadata {
            help: brief.clone(),
            unit: unit_word.clone(),
            prom_type: "counter".to_string(),
        });
        let mut sample = String::new();
        emit_sample_line(
            &mut sample,
            &sum_name,
            base_labels,
            &format!("{}", s.sum),
            ts_suffix,
        );
        group.samples.push(sample);
    }

    // _count uses same unit-bearing base name (histogram-family convention).
    let count_name = format!("{base_metric_name}_count");
    {
        let group = groups.get_or_insert(&count_name, || PromMetricMetadata {
            help: brief,
            unit: unit_word,
            prom_type: "counter".to_string(),
        });
        let mut sample = String::new();
        emit_sample_line(
            &mut sample,
            &count_name,
            base_labels,
            &format!("{}", s.count),
            ts_suffix,
        );
        group.samples.push(sample);
    }
}

/// Writes a single sample line with optional labels and timestamp suffix.
fn emit_sample_line(
    out: &mut String,
    metric_name: &str,
    labels: &str,
    value: &str,
    ts_suffix: &str,
) {
    if labels.is_empty() {
        let _ = writeln!(out, "{metric_name} {value}{ts_suffix}");
    } else {
        let _ = writeln!(out, "{metric_name}{{{labels}}} {value}{ts_suffix}");
    }
}

/// Renders the `target_info` gauge block from resource attributes into a
/// reusable string. Returns an empty string when `resource_attributes` is
/// empty (per OTel→Prometheus spec, `target_info` is only emitted when there
/// is metadata to expose). Intended to be called once at server startup; the
/// resulting string is then prepended verbatim to every Prometheus scrape.
///
/// Accepts `AttributeValue` (the same type used in the engine telemetry
/// config) so callers do not need to pre-flatten typed values to strings.
pub(crate) fn render_target_info(
    resource_attributes: &HashMap<String, ResourceAttributeValue>,
) -> String {
    if resource_attributes.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let _ = writeln!(&mut out, "# HELP target_info Target metadata");
    let _ = writeln!(&mut out, "# TYPE target_info gauge");
    let merged = sanitize_and_merge_label_pairs(
        resource_attributes
            .iter()
            .map(|(k, v)| (k.as_str(), v.to_string_value())),
        // No reserved keys: `target_info` is a separate metric line and
        // does not carry `otel_scope_*` labels, so no collision is possible.
        &[],
    );
    let mut labels = String::new();
    for (key, value) in &merged {
        if !labels.is_empty() {
            labels.push(',');
        }
        let _ = write!(
            &mut labels,
            "{}=\"{}\"",
            key,
            escape_prom_label_value(value)
        );
    }
    let _ = writeln!(&mut out, "target_info{{{labels}}} 1");
    out
}

fn agg_prometheus_text(
    groups: &[AggregateGroup],
    timestamp_millis: Option<i64>,
    target_info: &str,
) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {ms}"))
        .unwrap_or_default();
    let mut prom_groups = PromGroupedMetrics::new();

    out.push_str(target_info);

    for g in groups {
        // Base labels: `otel_scope_name` plus the merged sanitized attributes.
        // `otel_scope_version` is emitted only when a non-empty version is
        // available; the current `MetricsDescriptor` does not carry one, so
        // the label is omitted entirely (per OTel→Prometheus spec: only
        // labels with values are emitted).
        let mut base_labels = String::new();
        if !g.name.is_empty() {
            let _ = write!(
                &mut base_labels,
                "otel_scope_name=\"{}\"",
                escape_prom_label_value(&g.name)
            );
        }
        // Merge values for keys that collide after sanitization (per
        // OTel→Prometheus spec). Emission order is unspecified — Prometheus
        // treats labels as an unordered set. Drop attribute keys that
        // sanitize to the reserved `otel_scope_*` names already emitted
        // above to avoid duplicate-label rejection by Prometheus.
        let merged = sanitize_and_merge_label_pairs(
            g.attributes
                .iter()
                .map(|(k, v)| (k.as_str(), v.to_string_value())),
            RESERVED_SCOPE_LABEL_KEYS,
        );
        for (k, v) in &merged {
            if !base_labels.is_empty() {
                base_labels.push(',');
            }
            let _ = write!(&mut base_labels, "{}=\"{}\"", k, escape_prom_label_value(v));
        }

        // Collect metrics for this group
        for field in g.brief.metrics.iter() {
            if let Some(value) = g.metrics.get(field.name) {
                match value {
                    MetricValue::U64(_) | MetricValue::F64(_) => {
                        collect_scalar_metric(
                            &mut prom_groups,
                            field,
                            *value,
                            &base_labels,
                            &ts_suffix,
                        );
                    }
                    MetricValue::Mmsc(s) => {
                        collect_mmsc_metric(&mut prom_groups, field, s, &base_labels, &ts_suffix);
                    }
                }
            }
        }
    }

    // Emit all metric families as contiguous groups (Prometheus spec requirement).
    prom_groups.emit(&mut out);

    out
}

/// Collects a snapshot of current metrics without resetting them.
fn collect_metrics_snapshot(
    telemetry_registry: &TelemetryRegistryHandle,
    keep_all_zeroes: bool,
) -> Vec<MetricSetWithMetadata> {
    let mut metric_sets = Vec::new();

    telemetry_registry.visit_current_metrics_with_zeroes(
        |descriptor, attributes, metrics_iter| {
            let mut metrics = Vec::new();

            for (field, value) in metrics_iter {
                metrics.push(MetricDataPointWithMetadata {
                    metadata: *field,
                    value,
                });
            }

            if !metrics.is_empty() {
                // Convert attributes to HashMap using the iterator
                let mut attrs_map = HashMap::new();
                for (key, value) in attributes.iter_attributes() {
                    let _ = attrs_map.insert(key.to_string(), value.clone());
                }

                metric_sets.push(MetricSetWithMetadata {
                    name: descriptor.name.to_owned(),
                    brief: String::new(), // MetricsDescriptor doesn't have description field
                    attributes: attrs_map,
                    metrics,
                });
            }
        },
        keep_all_zeroes,
    );

    metric_sets
}

/// Collects a snapshot of current metrics and resets them afterwards.
fn collect_metrics_snapshot_and_reset(
    telemetry_registry: &TelemetryRegistryHandle,
    keep_all_zeroes: bool,
) -> Vec<MetricSetWithMetadata> {
    let mut metric_sets = Vec::new();

    telemetry_registry.visit_metrics_and_reset_with_zeroes(
        |descriptor, attributes, metrics_iter| {
            let mut metrics = Vec::new();

            for (field, value) in metrics_iter {
                metrics.push(MetricDataPointWithMetadata {
                    metadata: *field,
                    value,
                });
            }

            if !metrics.is_empty() {
                let mut attrs_map = HashMap::new();
                for (key, value) in attributes.iter_attributes() {
                    let _ = attrs_map.insert(key.to_string(), value.clone());
                }

                metric_sets.push(MetricSetWithMetadata {
                    name: descriptor.name.to_owned(),
                    brief: "".to_owned(),
                    attributes: attrs_map,
                    metrics,
                });
            }
        },
        keep_all_zeroes,
    );

    metric_sets
}

/// Compact snapshot without resetting.
fn collect_compact_snapshot(telemetry_registry: &TelemetryRegistryHandle) -> Vec<MetricSet> {
    let mut metric_sets = Vec::new();

    telemetry_registry.visit_current_metrics(|descriptor, attributes, metrics_iter| {
        let mut metrics = HashMap::new();
        for (field, value) in metrics_iter {
            let _ = metrics.insert(field.name.to_string(), value);
        }

        if !metrics.is_empty() {
            // include attributes in compact format
            let mut attrs_map = HashMap::new();
            for (key, value) in attributes.iter_attributes() {
                let _ = attrs_map.insert(key.to_string(), value.clone());
            }

            metric_sets.push(MetricSet {
                name: descriptor.name.to_string(),
                attributes: attrs_map,
                metrics,
            });
        }
    });

    metric_sets
}

/// Compact snapshot with resetting.
fn collect_compact_snapshot_and_reset(
    telemetry_registry: &TelemetryRegistryHandle,
) -> Vec<MetricSet> {
    let mut metric_sets = Vec::new();

    telemetry_registry.visit_metrics_and_reset(|descriptor, attributes, metrics_iter| {
        let mut metrics = HashMap::new();
        for (field, value) in metrics_iter {
            let _ = metrics.insert(field.name.to_string(), value);
        }

        if !metrics.is_empty() {
            let mut attrs_map = HashMap::new();
            for (key, value) in attributes.iter_attributes() {
                let _ = attrs_map.insert(key.to_string(), value.clone());
            }

            metric_sets.push(MetricSet {
                name: descriptor.name.to_string(),
                attributes: attrs_map,
                metrics,
            });
        }
    });

    metric_sets
}

fn format_line_protocol(
    telemetry_registry: &TelemetryRegistryHandle,
    reset: bool,
    timestamp_millis: Option<i64>,
) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {ms}"))
        .unwrap_or_default();

    let mut visit = |descriptor: &'static MetricsDescriptor,
                     attributes: &dyn AttributeSetHandler,
                     metrics_iter: MetricsIterator<'_>| {
        // Measurement is the metric set name when available; fallback to "metrics".
        let measurement_name = if descriptor.name.is_empty() {
            "metrics"
        } else {
            descriptor.name
        };
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
            match value {
                MetricValue::U64(_) | MetricValue::F64(_) => {
                    if !first {
                        fields.push(',');
                    }
                    first = false;
                    let _ = write!(
                        &mut fields,
                        "{}={}",
                        escape_lp_field_key(field.name),
                        format_lp_value(value, Some(field.value_type))
                    );
                }
                MetricValue::Mmsc(s) => {
                    if s.count == 0 {
                        continue;
                    }
                    for (suffix, fval) in [("_min", s.min), ("_max", s.max), ("_sum", s.sum)] {
                        if !first {
                            fields.push(',');
                        }
                        first = false;
                        let _ = write!(
                            &mut fields,
                            "{}{}={}",
                            escape_lp_field_key(field.name),
                            suffix,
                            fval
                        );
                    }
                    if !first {
                        fields.push(',');
                    }
                    first = false;
                    let _ = write!(
                        &mut fields,
                        "{}_count={}i",
                        escape_lp_field_key(field.name),
                        s.count
                    );
                }
            }
        }

        if !fields.is_empty() {
            let _ = writeln!(&mut out, "{measurement}{tags} {fields}{ts_suffix}");
        }
    };

    if reset {
        telemetry_registry.visit_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        telemetry_registry.visit_current_metrics(|d, a, m| visit(d, a, m));
    }

    out
}

fn format_prometheus_text(
    telemetry_registry: &TelemetryRegistryHandle,
    reset: bool,
    timestamp_millis: Option<i64>,
    target_info: &str,
) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {ms}"))
        .unwrap_or_default();
    let mut groups = PromGroupedMetrics::new();

    out.push_str(target_info);

    let mut visit = |descriptor: &'static MetricsDescriptor,
                     attributes: &dyn AttributeSetHandler,
                     metrics_iter: MetricsIterator<'_>| {
        // Base labels: `otel_scope_name` plus the merged sanitized attributes.
        // `otel_scope_version` is emitted only when a non-empty version is
        // available; the current `MetricsDescriptor` does not carry one, so
        // the label is omitted entirely (per OTel→Prometheus spec: only
        // labels with values are emitted).
        let mut base_labels = String::new();
        if !descriptor.name.is_empty() {
            let _ = write!(
                &mut base_labels,
                "otel_scope_name=\"{}\"",
                escape_prom_label_value(descriptor.name)
            );
        }
        // Merge values for keys that collide after sanitization (per
        // OTel→Prometheus spec). Emission order is unspecified. Drop
        // attribute keys that sanitize to the reserved `otel_scope_*`
        // names already emitted above.
        let merged = sanitize_and_merge_label_pairs(
            attributes
                .iter_attributes()
                .map(|(k, v)| (k, v.to_string_value())),
            RESERVED_SCOPE_LABEL_KEYS,
        );
        for (key, value) in &merged {
            if !base_labels.is_empty() {
                base_labels.push(',');
            }
            let _ = write!(
                &mut base_labels,
                "{}=\"{}\"",
                key,
                escape_prom_label_value(value)
            );
        }

        for (field, value) in metrics_iter {
            match value {
                MetricValue::U64(_) | MetricValue::F64(_) => {
                    collect_scalar_metric(&mut groups, field, value, &base_labels, &ts_suffix);
                }
                MetricValue::Mmsc(ref s) => {
                    collect_mmsc_metric(&mut groups, field, s, &base_labels, &ts_suffix);
                }
            }
        }
    };

    if reset {
        telemetry_registry.visit_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        telemetry_registry.visit_current_metrics(|d, a, m| visit(d, a, m));
    }

    // Emit all metric families as contiguous groups (Prometheus spec requirement).
    groups.emit(&mut out);

    out
}

fn escape_lp_measurement(s: &str) -> String {
    // Fast path: no escaping needed
    if !s.as_bytes().iter().any(|&b| b == b',' || b == b' ') {
        return s.to_string();
    }
    // Single-pass escape
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ',' | ' ' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn escape_lp_tag_key(s: &str) -> String {
    // Fast path: no escaping needed
    if !s
        .as_bytes()
        .iter()
        .any(|&b| b == b',' || b == b' ' || b == b'=')
    {
        return s.to_string();
    }
    // Single-pass escape
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ',' | ' ' | '=' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn escape_lp_tag_value(s: &str) -> String {
    // Same escaping rules as tag key for spaces/commas/equals
    escape_lp_tag_key(s)
}

fn escape_lp_field_key(s: &str) -> String {
    // Same escaping rules as tag key for spaces/commas/equals
    escape_lp_tag_key(s)
}

fn sanitize_prom_metric_name(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, ch) in s.chars().enumerate() {
        let ok = matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':');
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
    if out.is_empty() {
        return "metric".to_string();
    }
    // Collapse multiple consecutive underscores into a single underscore.
    let mut collapsed = String::with_capacity(out.len());
    let mut prev_underscore = false;
    for ch in out.chars() {
        if ch == '_' {
            if !prev_underscore {
                collapsed.push('_');
            }
            prev_underscore = true;
        } else {
            collapsed.push(ch);
            prev_underscore = false;
        }
    }
    // Strip a trailing underscore so callers that append unit / `_total`
    // suffixes don't produce double underscores (e.g. `foo.` → `foo_` →
    // `foo__bytes`). If stripping leaves an empty string, fall back to
    // the placeholder name used for fully-invalid inputs.
    if collapsed.ends_with('_') {
        let _ = collapsed.pop();
    }
    if collapsed.is_empty() {
        return "metric".to_string();
    }
    collapsed
}

/// Maps a simple UCUM unit abbreviation to its Prometheus unit word.
fn ucum_simple_unit(unit: &str) -> Option<&'static str> {
    match unit {
        "d" => Some("days"),
        "h" => Some("hours"),
        "min" => Some("minutes"),
        "s" => Some("seconds"),
        "ms" => Some("milliseconds"),
        "us" => Some("microseconds"),
        "ns" => Some("nanoseconds"),
        "By" => Some("bytes"),
        "KiBy" => Some("kibibytes"),
        "MiBy" => Some("mebibytes"),
        "GiBy" => Some("gibibytes"),
        "TiBy" => Some("tebibytes"),
        "kBy" => Some("kilobytes"),
        "MBy" => Some("megabytes"),
        "GBy" => Some("gigabytes"),
        "TBy" => Some("terabytes"),
        "m" => Some("meters"),
        "V" => Some("volts"),
        "A" => Some("amperes"),
        "J" => Some("joules"),
        "W" => Some("watts"),
        "g" => Some("grams"),
        "Cel" => Some("celsius"),
        "Hz" => Some("hertz"),
        "%" => Some("percent"),
        _ => None,
    }
}

/// Maps UCUM unit strings to Prometheus unit words per the OTel spec.
///
/// Handles:
/// - Simple units: `"By"` → `"bytes"`
/// - Dimensionless `"1"` and empty → `None`
/// - Bracketed annotations are stripped: `"{packet}/s"` → `"per_second"`
/// - Compound rate units: `"By/s"` → `"bytes_per_second"`
fn ucum_to_prometheus_unit(unit: &str) -> Option<&'static str> {
    if unit.is_empty() || unit == "1" {
        return None;
    }

    // Strip bracketed annotation portions (e.g., `{packet}` → ``).
    let stripped = strip_curly_braces(unit);
    let stripped = stripped.trim();
    if stripped.is_empty() {
        return None;
    }

    // Try as a simple unit first.
    if let Some(w) = ucum_simple_unit(stripped) {
        return Some(w);
    }

    // Handle compound rate units: `<numerator>/<denominator>`
    if let Some(pos) = stripped.find('/') {
        let numer = stripped[..pos].trim();
        let denom = stripped[pos + 1..].trim();
        return compound_rate_unit(numer, denom);
    }

    None
}

/// Strips `{...}` annotation blocks from a unit string.
fn strip_curly_braces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth = 0u32;
    for ch in s.chars() {
        match ch {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            // Unbalanced closing brace: silently drop.
            '}' => {}
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Returns the Prometheus unit word for compound rate units like `By/s`.
///
/// Looks up the result in [`COMPOUND_RATE_CACHE`], which is generated by
/// [`rate_entries!`] from the same word list as [`ucum_simple_unit`] — so
/// every simple unit automatically supports second/minute/hour rates
/// (e.g. `KiBy/h` → `kibibytes_per_hour`, `Hz/s` → `hertz_per_second`).
///
/// Note: the denominator only accepts time-division units (`s`, `min`, `h`).
/// A denominator of `"m"` is the UCUM code for *meters*, not minutes
/// (`"min"` is minutes), so `By/m` intentionally returns `None`.
fn compound_rate_unit(numerator: &str, denominator: &str) -> Option<&'static str> {
    let denom_word = match denominator {
        "s" => "second",
        "min" => "minute",
        "h" => "hour",
        _ => return None,
    };

    if numerator.is_empty() {
        // Pure rate like `/s` or `{packet}/s` after stripping brackets.
        return match denom_word {
            "second" => Some("per_second"),
            "minute" => Some("per_minute"),
            "hour" => Some("per_hour"),
            _ => None,
        };
    }

    let numer_word = ucum_simple_unit(numerator)?;
    COMPOUND_RATE_CACHE
        .iter()
        .find(|(n, d, _)| *n == numer_word && *d == denom_word)
        .map(|(_, _, result)| *result)
}

/// Pre-computed compound rate unit strings, keyed by `(numerator_word,
/// denominator_word)`. Generated by [`rate_entries!`] from the list of
/// simple-unit words in [`ucum_simple_unit`] so every simple unit
/// automatically gains second/minute/hour rate forms (e.g. `Hz/h` →
/// `hertz_per_hour`). Each value is a `&'static str` produced by `concat!`
/// at compile time — no heap allocation on the scrape path.
///
/// To support a new simple unit, add the word to the list below *and* the
/// matching UCUM mapping to [`ucum_simple_unit`].
macro_rules! rate_entries {
    ($($numer:literal),* $(,)?) => {
        &[
            $(
                ($numer, "second", concat!($numer, "_per_second")),
                ($numer, "minute", concat!($numer, "_per_minute")),
                ($numer, "hour",   concat!($numer, "_per_hour")),
            )*
        ]
    };
}

static COMPOUND_RATE_CACHE: &[(&str, &str, &str)] = rate_entries![
    // Time
    "days",
    "hours",
    "minutes",
    "seconds",
    "milliseconds",
    "microseconds",
    "nanoseconds",
    // Bytes (binary)
    "bytes",
    "kibibytes",
    "mebibytes",
    "gibibytes",
    "tebibytes",
    // Bytes (SI)
    "kilobytes",
    "megabytes",
    "gigabytes",
    "terabytes",
    // Distance / mass / energy / power
    "meters",
    "grams",
    "joules",
    "watts",
    // Electrical / thermal / frequency / dimensionless
    "volts",
    "amperes",
    "celsius",
    "hertz",
    "percent",
];

/// Builds a Prometheus metric name with proper unit and type suffixes per OTel spec.
///
/// Ordering for counters is `<base>_<unit>_total` per the spec. If `base_name`
/// itself already ends in `_total`, the suffix is temporarily stripped so the
/// unit can be inserted between the base and `_total` (otherwise a counter
/// named `errors_total` with unit `By` would render as
/// `errors_total_bytes_total`).
fn build_prom_metric_name(base_name: &str, unit: &str, instrument: Instrument) -> String {
    let mut name = sanitize_prom_metric_name(base_name);
    let is_counter = matches!(instrument, Instrument::Counter);

    // For counters, strip any existing `_total` suffix so the unit suffix is
    // placed before it. Re-appended unconditionally below.
    if is_counter && has_total_suffix(&name) {
        if name.eq_ignore_ascii_case("total") {
            name.clear();
        } else {
            // `_total` is 6 ASCII bytes; the suffix check above already
            // confirmed length and the underscore separator.
            name.truncate(name.len() - 6);
        }
    }

    // Append unit suffix if applicable and not already present.
    // The check is done byte-wise to avoid allocating a temporary String:
    // both `name` (post-sanitization) and `unit_word` are ASCII.
    if let Some(unit_word) = ucum_to_prometheus_unit(unit)
        && !ends_with_underscore_word(&name, unit_word)
    {
        if !name.is_empty() {
            name.push('_');
        }
        name.push_str(unit_word);
    }

    // Counters always end in `_total`.
    if is_counter {
        if name.is_empty() {
            name.push_str("total");
        } else {
            name.push_str("_total");
        }
    }

    name
}

/// Returns true if `name` ends with `_<word>`. `word` is compared byte-wise
/// (ASCII). Avoids allocating a temporary `String` for the suffix check.
fn ends_with_underscore_word(name: &str, word: &str) -> bool {
    name.len() > word.len()
        && name.ends_with(word)
        && name.as_bytes()[name.len() - word.len() - 1] == b'_'
}

/// Returns true if `name` already ends with `_total` as a proper suffix
/// (preceded by `_`, or the entire name is `"total"`). The comparison is
/// case-insensitive: `Foo_Total`, `FOO_TOTAL`, and `foo_total` all match.
fn has_total_suffix(name: &str) -> bool {
    if name.eq_ignore_ascii_case("total") {
        return true;
    }
    let bytes = name.as_bytes();
    bytes.len() >= 6
        && bytes[bytes.len() - 6] == b'_'
        && bytes[bytes.len() - 5..].eq_ignore_ascii_case(b"total")
}

fn sanitize_prom_label_key(s: &str) -> String {
    // Sanitize each char and collapse runs of `_` inline
    // (per OTel spec §Metric Attributes). No intermediate allocation.
    let mut out = String::with_capacity(s.len());
    let mut prev_underscore = false;
    let mut first = true;
    for ch in s.chars() {
        let mapped = match ch {
            'a'..='z' | 'A'..='Z' | '_' | ':' => ch,
            '0'..='9' => {
                if first {
                    // Leading digit: prepend `_` then keep the digit.
                    out.push('_');
                    prev_underscore = true;
                }
                ch
            }
            _ => '_',
        };
        if mapped == '_' {
            if !prev_underscore {
                out.push('_');
                prev_underscore = true;
            }
        } else {
            out.push(mapped);
            prev_underscore = false;
        }
        first = false;
    }
    if out.is_empty() {
        return "label".to_string();
    }
    out
}

fn escape_prom_label_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => {
                out.push('\\');
                out.push('\\');
            }
            '"' => {
                out.push('\\');
                out.push('"');
            }
            '\n' => {
                out.push('\\');
                out.push('n');
            }
            _ => out.push(ch),
        }
    }
    out
}

fn escape_prom_help(s: &str) -> String {
    // Similar escaping to label value per Prometheus recommendations
    escape_prom_label_value(s)
}

/// Sanitizes label keys and merges values that collide after sanitization
/// into a single entry separated by `;`, per the OTel→Prometheus spec.
///
/// Per spec: "OpenTelemetry keys [that] map to the same Prometheus key …
/// MUST be concatenated together, separated by `;`, and ordered by the
/// lexicographical order of the original keys." We collect and sort by
/// original key before merging so the joined value is deterministic
/// regardless of caller iteration order (e.g. `HashMap::iter`, which has
/// no defined order).
///
/// `reserved_keys` lists already-emitted labels (post-sanitization) that
/// must not appear in the merged map. Per the spec, the scope-derived
/// `otel_scope_name` / `otel_scope_version` labels are emitted separately
/// from per-metric attributes; if a metric attribute key sanitizes to one
/// of those reserved names, the conflicting attribute is dropped (Prometheus
/// rejects duplicate label names on a single sample). Pass `&[]` when no
/// reservation applies (e.g. when rendering `target_info` from resource
/// attributes).
///
/// Iteration order over the returned map is not specified — Prometheus
/// treats labels as an unordered set.
fn sanitize_and_merge_label_pairs<'a, I>(
    attrs: I,
    reserved_keys: &[&str],
) -> HashMap<String, String>
where
    I: IntoIterator<Item = (&'a str, String)>,
{
    // Collect first so we can sort by original key. Sorting before the
    // sanitize/merge pass guarantees that for collisions like
    // `service.name="a"` + `service_name="b"`, the joined output is
    // always `"a;b"` (lex-ordered by raw key), independent of how the
    // caller iterates its source container.
    let mut entries: Vec<(&'a str, String)> = attrs.into_iter().collect();
    entries.sort_by_key(|(k, _)| *k);

    let mut merged: HashMap<String, String> = HashMap::with_capacity(entries.len());
    for (key, value) in entries {
        let sanitized = sanitize_prom_label_key(key);
        // Skip keys that collide with separately-emitted scope/reserved
        // labels. Comparison is on the sanitized form to catch inputs like
        // `otel.scope.name` that map to the reserved name.
        if reserved_keys.contains(&sanitized.as_str()) {
            continue;
        }
        let _ = merged
            .entry(sanitized)
            .and_modify(|existing| {
                existing.push(';');
                existing.push_str(&value);
            })
            .or_insert(value);
    }
    merged
}

/// Reserved Prometheus label keys for OTel scope identity. These are emitted
/// separately from per-metric attributes; per-metric attributes whose keys
/// sanitize to one of these names are dropped to avoid duplicate-label
/// scrape errors.
///
/// `otel_scope_version` is intentionally omitted: the current
/// `MetricsDescriptor` does not carry a version, so we never emit
/// `otel_scope_version` ourselves and a user attribute with that name is
/// not a collision. Add it here when version emission is implemented.
const RESERVED_SCOPE_LABEL_KEYS: &[&str] = &["otel_scope_name"];

// ---------------------------------------------------------------------------
// WebSocket live log stream  (/api/v1/telemetry/logs/stream)
// ---------------------------------------------------------------------------

/// Map a level string to a numeric severity (TRACE=0 through ERROR=4).
/// Unknown levels are treated as TRACE (lowest severity).
///
/// Uses ASCII-only comparison to avoid allocating a temporary uppercase string.
fn level_severity(level: &str) -> u8 {
    if level.eq_ignore_ascii_case("ERROR") {
        4
    } else if level.eq_ignore_ascii_case("WARN") {
        3
    } else if level.eq_ignore_ascii_case("INFO") {
        2
    } else if level.eq_ignore_ascii_case("DEBUG") {
        1
    } else {
        0 // TRACE and anything unknown
    }
}

/// Map a `tracing::Level` to the same severity scale used by [`level_severity`].
#[cfg_attr(not(feature = "live-logs-ws"), allow(dead_code))]
fn tracing_level_severity(level: &otap_df_telemetry::Level) -> u8 {
    match *level {
        otap_df_telemetry::Level::ERROR => 4,
        otap_df_telemetry::Level::WARN => 3,
        otap_df_telemetry::Level::INFO => 2,
        otap_df_telemetry::Level::DEBUG => 1,
        _ => 0, // TRACE
    }
}

/// Optional text filter applied to rendered log entries on the server side.
///
/// Applied after rendering so that `search_query` can match against the fully
/// formatted message text as well as metadata fields (level, target, etc.).
#[derive(Default)]
#[cfg_attr(not(feature = "live-logs-ws"), allow(dead_code))]
struct LogFilter {
    /// Case-insensitive substring matched against: rendered message, level,
    /// target, event_name, and file path.
    search_query: Option<String>,
    /// Discard entries whose timestamp is strictly before this instant.
    minimum_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Numeric minimum severity (TRACE=0, DEBUG=1, INFO=2, WARN=3, ERROR=4).
    /// Entries with a lower severity are discarded.
    minimum_level: Option<u8>,
}

#[cfg_attr(not(feature = "live-logs-ws"), allow(dead_code))]
impl LogFilter {
    /// Returns `true` when the rendered log entry passes all active criteria.
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_ts) = &self.minimum_timestamp {
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                if ts.with_timezone(&chrono::Utc) < *min_ts {
                    return false;
                }
            }
        }
        if let Some(min_level) = self.minimum_level {
            if level_severity(&entry.level) < min_level {
                return false;
            }
        }
        if let Some(q) = &self.search_query {
            // `q` is already lowercased at construction time; only the entry
            // fields need to be lowercased per event.
            //
            // TODO(perf): each `to_lowercase()` allocates a temporary String.
            // If filter throughput ever becomes a concern, consider a
            // case-insensitive substring search (e.g. `memchr` + manual ASCII
            // comparison) to avoid per-field heap allocation.
            let matched = entry.rendered.to_lowercase().contains(q.as_str())
                || entry.level.to_lowercase().contains(q.as_str())
                || entry.target.to_lowercase().contains(q.as_str())
                || entry.event_name.to_lowercase().contains(q.as_str())
                || entry
                    .file
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(q.as_str());
            if !matched {
                return false;
            }
        }
        true
    }

    /// Cheap pre-filter on the raw (unrendered) log event.
    ///
    /// Checks `minimum_level` and `minimum_timestamp` without rendering the
    /// entry, so we can skip the more expensive `render_log_entry()` call for
    /// events that would be rejected anyway. `search_query` is intentionally
    /// not checked here because it operates on the rendered text.
    fn prefilter_raw(&self, event: &RetainedLogEvent) -> bool {
        if let Some(min_ts) = &self.minimum_timestamp {
            let event_ts = chrono::DateTime::<chrono::Utc>::from(event.event.time);
            if event_ts < *min_ts {
                return false;
            }
        }
        if let Some(min_level) = self.minimum_level {
            let callsite = event.event.record.callsite();
            if tracing_level_severity(callsite.level()) < min_level {
                return false;
            }
        }
        true
    }

    /// Build a `LogFilter` from optional client-supplied strings.
    fn from_params(
        search_query: Option<String>,
        minimum_timestamp: Option<String>,
        minimum_level: Option<String>,
    ) -> Self {
        let minimum_timestamp = minimum_timestamp.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        Self {
            // Pre-lowercase so `matches()` doesn't allocate a lowercase string
            // on every log event that passes through the filter.
            search_query: search_query.map(|s| s.to_lowercase()),
            minimum_timestamp,
            minimum_level: minimum_level
                .as_deref()
                .filter(|s| !s.eq_ignore_ascii_case("all") && !s.is_empty())
                .map(level_severity),
        }
    }
}

/// Client to server WebSocket messages.
#[cfg(feature = "live-logs-ws")]
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum WsClientMsg {
    /// Begin streaming. Sends an initial retained-log snapshot, then follows
    /// with live events.
    Subscribe {
        /// Cursor: only include retained entries strictly newer than this seq.
        after: Option<u64>,
        /// Maximum retained entries in the initial snapshot (clamped 1-1000).
        limit: Option<usize>,
        /// Case-insensitive text filter (applied server-side).
        #[serde(rename = "searchQuery")]
        search_query: Option<String>,
        /// RFC 3339 minimum timestamp filter.
        #[serde(rename = "minimumTimestamp")]
        minimum_timestamp: Option<String>,
        /// Minimum log level (TRACE/DEBUG/INFO/WARN/ERROR or ALL to disable).
        #[serde(rename = "minimumLevel")]
        minimum_level: Option<String>,
        /// Start paused (no live events sent until `resume`).
        paused: Option<bool>,
    },
    /// Stop forwarding live events (WebSocket stays open; server drains
    /// broadcast to avoid lagging, but does not accumulate a client backlog).
    Pause,
    /// Resume forwarding live events from the current live position.
    Resume,
    /// Update the server-side text/timestamp/level filter.
    SetFilter {
        #[serde(rename = "searchQuery")]
        search_query: Option<String>,
        #[serde(rename = "minimumTimestamp")]
        minimum_timestamp: Option<String>,
        #[serde(rename = "minimumLevel")]
        minimum_level: Option<String>,
    },
    /// Request a retained-log snapshot (same as the HTTP query endpoint).
    Backfill {
        after: Option<u64>,
        limit: Option<usize>,
    },
    /// Keep-alive; server replies with `pong`.
    Ping,
}

/// Server to client WebSocket messages.
#[cfg(feature = "live-logs-ws")]
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsServerMsg {
    /// Initial or backfill snapshot of retained logs.
    Snapshot {
        oldest_seq: Option<u64>,
        newest_seq: Option<u64>,
        next_seq: u64,
        truncated_before_seq: Option<u64>,
        dropped_on_ingest: u64,
        dropped_on_retention: u64,
        retained_bytes: usize,
        logs: Vec<LogEntry>,
    },
    /// Single live log entry pushed by the server.
    Log {
        #[serde(flatten)]
        entry: LogEntry,
    },
    /// Current pause state and cursor position.
    State { paused: bool, next_seq: u64 },
    /// Server-side error notification (e.g. subscriber lagged and dropped events).
    /// `lag_before_seq` is set on lag errors: it is the cursor value from just
    /// before the gap so the client can issue a targeted backfill.
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        lag_before_seq: Option<u64>,
    },
    /// Reply to a client `ping`.
    Pong,
}

/// Upgrade handler for `GET /api/v1/telemetry/logs/stream`.
#[cfg(feature = "live-logs-ws")]
async fn ws_logs_stream(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_ws_logs(socket, state))
}

/// Send a serialized `WsServerMsg` as a WebSocket text frame.
///
/// Returns `false` if the socket is closed and the caller should stop sending.
#[cfg(feature = "live-logs-ws")]
async fn ws_send(ws: &mut WebSocket, msg: &WsServerMsg) -> bool {
    match serde_json::to_string(msg) {
        Ok(text) => ws.send(Message::Text(text.into())).await.is_ok(),
        Err(_) => false,
    }
}

/// Build and send a `snapshot` message from a `LogQueryResult`.
///
/// Applies `filter` to the rendered entries so that snapshot and backfill
/// responses are consistent with what the live stream sends for the same filter.
#[cfg(feature = "live-logs-ws")]
async fn ws_send_snapshot(
    ws: &mut WebSocket,
    registry: &TelemetryRegistryHandle,
    result: LogQueryResult,
    filter: &LogFilter,
) -> bool {
    let mut resp = logs_response(registry, result);
    resp.logs.retain(|entry| filter.matches(entry));
    let msg = WsServerMsg::Snapshot {
        oldest_seq: resp.oldest_seq,
        newest_seq: resp.newest_seq,
        next_seq: resp.next_seq,
        truncated_before_seq: resp.truncated_before_seq,
        dropped_on_ingest: resp.dropped_on_ingest,
        dropped_on_retention: resp.dropped_on_retention,
        retained_bytes: resp.retained_bytes,
        logs: resp.logs,
    };
    ws_send(ws, &msg).await
}

/// Core WebSocket session loop for the live log stream.
///
/// # Protocol summary
///
/// 1. The client sends `subscribe` first.
/// 2. The server sends the initial retained-log snapshot, then streams live
///    events via `log` messages.
/// 3. `pause` / `resume` toggle server-side forwarding without closing the
///    socket. While paused the server still drains the broadcast channel so
///    that the producer is never slowed by this client.
/// 4. On `backfill` the server re-queries the retained ring buffer and sends a
///    `snapshot`. The cursor is updated so subsequent live events do not
///    duplicate.
/// 5. If the client falls more than `SUBSCRIBER_CHANNEL_CAPACITY` events
///    behind, the broadcast channel drops the overflow; the server notifies the
///    client with an `error` message so it can issue a `backfill`.
#[cfg(feature = "live-logs-ws")]
async fn handle_ws_logs(mut ws: WebSocket, state: AppState) {
    let Some(log_tap) = state.log_tap.as_ref() else {
        let _ = ws_send(
            &mut ws,
            &WsServerMsg::Error {
                message: "log tap is not enabled".to_string(),
                lag_before_seq: None,
            },
        )
        .await;
        return;
    };

    let registry = &state.metrics_registry;

    // Live broadcast receiver; set once on `subscribe`.
    let mut live_rx: Option<broadcast::Receiver<Arc<RetainedLogEvent>>> = None;
    let mut paused = false;
    let mut filter = LogFilter::default();
    // Tracks the sequence number of the last event we acknowledged (sent or
    // deliberately skipped while paused). Used in `state` replies so the client
    // knows where the live cursor stands.
    let mut cursor: u64 = 0;

    loop {
        if let Some(rx) = live_rx.as_mut() {
            // Subscribed: multiplex client messages and live log events.
            tokio::select! {
                biased; // prioritise client control messages

                client_raw = ws.recv() => {
                    match client_raw {
                        Some(Ok(Message::Text(text))) => {
                            match serde_json::from_str::<WsClientMsg>(&text) {
                                Ok(WsClientMsg::Pause) => {
                                    paused = true;
                                    if !ws_send(&mut ws, &WsServerMsg::State { paused, next_seq: cursor }).await {
                                        break;
                                    }
                                }
                                Ok(WsClientMsg::Resume) => {
                                    paused = false;
                                    // Send current cursor so client can decide to backfill.
                                    if !ws_send(&mut ws, &WsServerMsg::State { paused, next_seq: cursor }).await {
                                        break;
                                    }
                                }
                                Ok(WsClientMsg::SetFilter { search_query, minimum_timestamp, minimum_level }) => {
                                    filter = LogFilter::from_params(search_query, minimum_timestamp, minimum_level);
                                }
                                Ok(WsClientMsg::Backfill { after, limit }) => {
                                    let limit = limit.unwrap_or(100).clamp(1, 1000);
                                    let result = log_tap.query(LogQuery { after, limit });
                                    // Only advance cursor; never move it backward. A client may
                                    // request an older `after` (e.g. a lag gap backfill) while the
                                    // live stream has already moved the cursor forward.
                                    cursor = cursor.max(result.next_seq);
                                    if !ws_send_snapshot(&mut ws, registry, result, &filter).await {
                                        break;
                                    }
                                }
                                Ok(WsClientMsg::Ping) => {
                                    if !ws_send(&mut ws, &WsServerMsg::Pong).await {
                                        break;
                                    }
                                }
                                Ok(WsClientMsg::Subscribe { .. }) => {
                                    // Already subscribed; ignore duplicate.
                                }
                                Err(_) => {
                                    // Unknown or malformed message; keep the session open.
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Ok(_)) => {} // binary / ping frames; ignore
                        Some(Err(_)) => break,
                    }
                }

                live_event = rx.recv() => {
                    match live_event {
                        Ok(entry) => {
                            let entry_seq = entry.seq;

                            // Skip entries whose seq is at or below cursor: they
                            // were already delivered in the most recent snapshot
                            // or backfill (the subscribe-before-query race window).
                            if entry_seq <= cursor {
                                // Discard silently; already in the snapshot.
                            } else {
                                // Advance cursor so `state` replies are accurate
                                // even when paused or filtered.
                                cursor = entry_seq;

                                if !paused {
                                    // Cheap pre-filter on level/timestamp before
                                    // the more expensive render + search match.
                                    if filter.prefilter_raw(&entry) {
                                        let rendered = render_log_entry(registry, &entry);
                                        if filter.matches(&rendered) {
                                            let msg = WsServerMsg::Log {
                                                entry: rendered,
                                            };
                                            if !ws_send(&mut ws, &msg).await {
                                                break;
                                            }
                                        }
                                    }
                                }
                                // If paused: drain the channel to avoid lagging the
                                // producer, but send nothing to the client.
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            // The client was too slow; events were dropped from its
                            // receiver slot. `cursor` here is the last seq we
                            // successfully delivered; the client can use it as
                            // the `after` param for a backfill to recover the gap.
                            let msg = WsServerMsg::Error {
                                message: format!(
                                    "dropped {n} log event(s) due to slow consumption; \
                                     send backfill to resync"
                                ),
                                lag_before_seq: Some(cursor),
                            };
                            if !ws_send(&mut ws, &msg).await {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        } else {
            // Not yet subscribed: wait for a `subscribe` message only.
            let client_raw = ws.recv().await;
            match client_raw {
                Some(Ok(Message::Text(text))) => {
                    if let Ok(WsClientMsg::Subscribe {
                        after,
                        limit,
                        search_query,
                        minimum_timestamp,
                        minimum_level,
                        paused: start_paused,
                    }) = serde_json::from_str::<WsClientMsg>(&text)
                    {
                        // Subscribe to the broadcast channel BEFORE querying
                        // retained logs so we do not miss events recorded between
                        // the query and the first receive. Live events with
                        // seq <= cursor (set from snapshot.next_seq below) are
                        // silently discarded in the live_event arm to prevent
                        // duplicates for that race window.
                        live_rx = Some(log_tap.subscribe());

                        filter =
                            LogFilter::from_params(search_query, minimum_timestamp, minimum_level);
                        paused = start_paused.unwrap_or(false);

                        let limit = limit.unwrap_or(100).clamp(1, 1000);
                        let result = log_tap.query(LogQuery { after, limit });
                        cursor = result.next_seq;

                        if !ws_send_snapshot(&mut ws, registry, result, &filter).await {
                            break;
                        }
                        if !ws_send(
                            &mut ws,
                            &WsServerMsg::State {
                                paused,
                                next_seq: cursor,
                            },
                        )
                        .await
                        {
                            break;
                        }
                    }
                    // Other messages before subscribe are silently ignored.
                }
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {}
                Some(Err(_)) => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ControlPlane, ControlPlaneError, PipelineDetails, ReconfigureRequest, RolloutStatus,
        ShutdownStatus,
    };
    use axum::body::{Body, to_bytes};
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_engine::memory_limiter::MemoryPressureState;
    use otap_df_state::store::ObservedStateStore;
    use otap_df_telemetry::descriptor::{Instrument, MetricsField, Temporality};
    use std::sync::Arc;
    use tower::ServiceExt;

    struct NoopControlPlane;

    impl ControlPlane for NoopControlPlane {
        fn shutdown_all(&self, _timeout_secs: u64) -> Result<(), ControlPlaneError> {
            Err(ControlPlaneError::Internal {
                message: "not used in telemetry tests".to_string(),
            })
        }

        fn shutdown_pipeline(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _timeout_secs: u64,
        ) -> Result<ShutdownStatus, ControlPlaneError> {
            Err(ControlPlaneError::Internal {
                message: "not used in telemetry tests".to_string(),
            })
        }

        fn reconfigure_pipeline(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _request: ReconfigureRequest,
        ) -> Result<RolloutStatus, ControlPlaneError> {
            Err(ControlPlaneError::Internal {
                message: "not used in telemetry tests".to_string(),
            })
        }

        fn pipeline_details(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
        ) -> Result<Option<PipelineDetails>, ControlPlaneError> {
            Ok(None)
        }

        fn rollout_status(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _rollout_id: &str,
        ) -> Result<Option<RolloutStatus>, ControlPlaneError> {
            Ok(None)
        }

        fn shutdown_status(
            &self,
            _pipeline_group_id: &str,
            _pipeline_id: &str,
            _shutdown_id: &str,
        ) -> Result<Option<ShutdownStatus>, ControlPlaneError> {
            Ok(None)
        }
    }

    static TEST_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test_metrics",
        metrics: &[
            MetricsField {
                name: "requests_total",
                unit: "1",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                brief: "Total number of requests",
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "errors_total",
                unit: "1",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                brief: "Total number of errors",
                value_type: MetricValueType::U64,
            },
        ],
    };

    static TEST_METRICS_DESCRIPTOR_2: MetricsDescriptor = MetricsDescriptor {
        name: "database_metrics",
        metrics: &[MetricsField {
            name: "connections_active",
            unit: "1",
            instrument: Instrument::Gauge,
            temporality: None,
            brief: "Active database connections",
            value_type: MetricValueType::U64,
        }],
    };

    fn test_app_state() -> AppState {
        let metrics_registry = TelemetryRegistryHandle::new();
        let observed_state_store =
            ObservedStateStore::new(&ObservedStateSettings::default(), metrics_registry.clone());

        AppState {
            observed_state_store: observed_state_store.handle(),
            metrics_registry,
            controller: Arc::new(NoopControlPlane),
            log_tap: None,
            memory_pressure_state: MemoryPressureState::default(),
            target_info: Arc::from(""),
        }
    }

    #[tokio::test]
    async fn metrics_handler_keeps_prometheus_text_format() {
        let response = get_metrics(
            State(test_app_state()),
            Query(MetricsQuery {
                format: Some(OutputFormat::Prometheus),
                ..Default::default()
            }),
        )
        .await
        .expect("prometheus metrics should render");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE),
            Some(&header::HeaderValue::from_static(
                "text/plain; version=0.0.4; charset=utf-8"
            ))
        );

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("metrics body should collect");
        assert!(
            std::str::from_utf8(&body)
                .expect("prometheus body should be utf-8")
                .is_empty()
        );
    }

    #[cfg(feature = "live-logs-ws")]
    #[tokio::test]
    async fn telemetry_routes_include_logs_stream_websocket_endpoint() {
        let response = routes()
            .with_state(test_app_state())
            .oneshot(
                axum::http::Request::builder()
                    .uri("/telemetry/logs/stream")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");

        assert_ne!(response.status(), StatusCode::NOT_FOUND);
    }

    #[cfg(not(feature = "live-logs-ws"))]
    #[tokio::test]
    async fn telemetry_routes_exclude_logs_stream_websocket_endpoint_without_feature() {
        let response = routes()
            .with_state(test_app_state())
            .oneshot(
                axum::http::Request::builder()
                    .uri("/telemetry/logs/stream")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Ensures aggregate group ordering is deterministic: metric-set name first,
    /// then metric count when names are equal.
    #[test]
    fn test_aggregate_metric_groups_sorting_logic() {
        // Test the sorting logic with mock AggregateGroup structs
        let mut groups = [
            AggregateGroup {
                name: "zebra_metrics".to_string(),
                brief: &TEST_METRICS_DESCRIPTOR,
                attributes: HashMap::new(),
                metrics: {
                    let mut m = HashMap::new();
                    let _ = m.insert("metric1".to_string(), MetricValue::from(10u64));
                    m
                },
            },
            AggregateGroup {
                name: "alpha_metrics".to_string(),
                brief: &TEST_METRICS_DESCRIPTOR_2,
                attributes: HashMap::new(),
                metrics: {
                    let mut m = HashMap::new();
                    let _ = m.insert("metric1".to_string(), MetricValue::from(5u64));
                    let _ = m.insert("metric2".to_string(), MetricValue::from(15u64));
                    m
                },
            },
            AggregateGroup {
                name: "alpha_metrics".to_string(),
                brief: &TEST_METRICS_DESCRIPTOR,
                attributes: HashMap::new(),
                metrics: {
                    let mut m = HashMap::new();
                    let _ = m.insert("metric1".to_string(), MetricValue::from(8u64));
                    m
                },
            },
        ];

        // Apply the same sorting logic as in the function
        groups.sort_by(|a, b| {
            let ord = a.name.cmp(&b.name);
            if ord == std::cmp::Ordering::Equal {
                a.metrics.len().cmp(&b.metrics.len())
            } else {
                ord
            }
        });

        // Verify sorting: first by name alphabetically, then by number of metrics
        assert_eq!(groups[0].name, "alpha_metrics");
        assert_eq!(groups[0].metrics.len(), 1); // smaller metrics count first

        assert_eq!(groups[1].name, "alpha_metrics");
        assert_eq!(groups[1].metrics.len(), 2); // larger metrics count second

        assert_eq!(groups[2].name, "zebra_metrics");
        assert_eq!(groups[2].metrics.len(), 1);
    }

    /// Verifies attribute-based grouping splits and aggregates metric sets by the
    /// selected group keys (`env`, `region`) and preserves grouped attributes.
    #[test]
    fn test_aggregate_metric_groups_group_by_attribute() {
        use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
        use otap_df_telemetry::descriptor::{
            AttributeField, AttributeValueType, AttributesDescriptor,
        };
        use otap_df_telemetry::metrics::MetricSetHandler;

        // Mock Attributes: [env, region]
        static MOCK_ATTR_DESC: AttributesDescriptor = AttributesDescriptor {
            name: "test_attrs",
            fields: &[
                AttributeField {
                    key: "env",
                    r#type: AttributeValueType::String,
                    brief: "Environment",
                },
                AttributeField {
                    key: "region",
                    r#type: AttributeValueType::String,
                    brief: "Region",
                },
            ],
        };

        #[derive(Debug)]
        struct MockAttrSet {
            values: Vec<AttributeValue>,
        }
        impl MockAttrSet {
            fn new(env: &str, region: &str) -> Self {
                Self {
                    values: vec![
                        AttributeValue::String(env.to_string()),
                        AttributeValue::String(region.to_string()),
                    ],
                }
            }
        }
        impl AttributeSetHandler for MockAttrSet {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &MOCK_ATTR_DESC
            }
            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        // Three metric set implementations sharing the same descriptor but different default values
        #[derive(Debug, Default)]
        struct MetricSetA;
        #[derive(Debug, Default)]
        struct MetricSetB;
        #[derive(Debug, Default)]
        struct MetricSetC;
        #[derive(Debug, Default)]
        struct MetricSetD;

        impl MetricSetHandler for MetricSetA {
            fn descriptor(&self) -> &'static MetricsDescriptor {
                &TEST_METRICS_DESCRIPTOR
            }
            fn snapshot_values(&self) -> Vec<MetricValue> {
                vec![MetricValue::from(10u64), MetricValue::from(1u64)]
            } // requests_total=10, errors_total=1
            fn clear_values(&mut self) {}
            fn needs_flush(&self) -> bool {
                true
            }
        }
        impl MetricSetHandler for MetricSetB {
            fn descriptor(&self) -> &'static MetricsDescriptor {
                &TEST_METRICS_DESCRIPTOR
            }
            fn snapshot_values(&self) -> Vec<MetricValue> {
                vec![MetricValue::from(5u64), MetricValue::from(0u64)]
            } // requests_total=5, errors_total=0
            fn clear_values(&mut self) {}
            fn needs_flush(&self) -> bool {
                true
            }
        }
        impl MetricSetHandler for MetricSetC {
            fn descriptor(&self) -> &'static MetricsDescriptor {
                &TEST_METRICS_DESCRIPTOR
            }
            fn snapshot_values(&self) -> Vec<MetricValue> {
                vec![MetricValue::from(5u64), MetricValue::from(4u64)]
            } // requests_total=5, errors_total=4
            fn clear_values(&mut self) {}
            fn needs_flush(&self) -> bool {
                true
            }
        }
        impl MetricSetHandler for MetricSetD {
            fn descriptor(&self) -> &'static MetricsDescriptor {
                &TEST_METRICS_DESCRIPTOR
            }
            fn snapshot_values(&self) -> Vec<MetricValue> {
                vec![MetricValue::from(2u64), MetricValue::from(2u64)]
            } // requests_total=2, errors_total=2
            fn clear_values(&mut self) {}
            fn needs_flush(&self) -> bool {
                true
            }
        }

        // Build registry with two entries for the same metric set but different attributes
        let telemetry_registry = TelemetryRegistryHandle::new();
        let _m1: otap_df_telemetry::metrics::MetricSet<MetricSetA> =
            telemetry_registry.register_metric_set(MockAttrSet::new("prod", "us"));
        let _m2: otap_df_telemetry::metrics::MetricSet<MetricSetB> =
            telemetry_registry.register_metric_set(MockAttrSet::new("dev", "eu"));
        let _m3: otap_df_telemetry::metrics::MetricSet<MetricSetC> =
            telemetry_registry.register_metric_set(MockAttrSet::new("prod", "us"));
        let _m4: otap_df_telemetry::metrics::MetricSet<MetricSetD> =
            telemetry_registry.register_metric_set(MockAttrSet::new("dev", "us"));

        // Group by the "env" attribute and do not reset
        let groups = aggregate_metric_groups(&telemetry_registry, false, &["env"]);

        // Expect two groups for the same descriptor name, keyed by env values
        assert_eq!(groups.len(), 2);
        assert!(
            groups
                .iter()
                .all(|g| g.name == TEST_METRICS_DESCRIPTOR.name)
        );

        // Find groups by env
        let mut prod_found = false;
        let mut dev_found = false;
        for g in groups {
            let env = g.attributes.get("env").and_then(|v| match v {
                AttributeValue::String(s) => Some(s.as_str()),
                _ => None,
            });
            match env {
                Some("prod") => {
                    prod_found = true;
                    assert_eq!(
                        g.metrics.get("requests_total"),
                        Some(&MetricValue::from(10u64 + 5))
                    );
                    assert_eq!(
                        g.metrics.get("errors_total"),
                        Some(&MetricValue::from(1u64 + 4))
                    );
                    // Only grouped attribute should be present (env), not region
                    assert!(g.attributes.contains_key("env"));
                    assert!(!g.attributes.contains_key("region"));
                }
                Some("dev") => {
                    dev_found = true;
                    assert_eq!(
                        g.metrics.get("requests_total"),
                        Some(&MetricValue::from(5u64 + 2))
                    );
                    assert_eq!(
                        g.metrics.get("errors_total"),
                        Some(&MetricValue::from(2u64))
                    );
                    assert!(g.attributes.contains_key("env"));
                    assert!(!g.attributes.contains_key("region"));
                }
                _ => panic!("unexpected env attribute in group"),
            }
        }
        assert!(prod_found && dev_found);

        // Group by the "env" and region attributes and do not reset
        let groups = aggregate_metric_groups(&telemetry_registry, false, &["env", "region"]);

        // Expect three groups for the same descriptor name, keyed by env, region values
        assert_eq!(groups.len(), 3);
        assert!(
            groups
                .iter()
                .all(|g| g.name == TEST_METRICS_DESCRIPTOR.name)
        );

        // Find groups by env
        let mut prod_us_found = false;
        let mut dev_us_found = false;
        let mut dev_eu_found = false;
        for g in groups {
            let env = g.attributes.get("env").and_then(|v| match v {
                AttributeValue::String(s) => Some(s.as_str()),
                _ => None,
            });
            let region = g.attributes.get("region").and_then(|v| match v {
                AttributeValue::String(s) => Some(s.as_str()),
                _ => None,
            });
            match (env, region) {
                (Some("prod"), Some("us")) => {
                    prod_us_found = true;
                    assert_eq!(
                        g.metrics.get("requests_total"),
                        Some(&MetricValue::from(10u64 + 5))
                    );
                    assert_eq!(
                        g.metrics.get("errors_total"),
                        Some(&MetricValue::from(1u64 + 4))
                    );
                    assert!(g.attributes.contains_key("env"));
                    assert!(g.attributes.contains_key("region"));
                }
                (Some("dev"), Some("eu")) => {
                    dev_eu_found = true;
                    assert_eq!(
                        g.metrics.get("requests_total"),
                        Some(&MetricValue::from(5u64))
                    );
                    assert_eq!(
                        g.metrics.get("errors_total"),
                        Some(&MetricValue::from(0u64))
                    );
                    assert!(g.attributes.contains_key("env"));
                    assert!(g.attributes.contains_key("region"));
                }
                (Some("dev"), Some("us")) => {
                    dev_us_found = true;
                    assert_eq!(
                        g.metrics.get("requests_total"),
                        Some(&MetricValue::from(2u64))
                    );
                    assert_eq!(
                        g.metrics.get("errors_total"),
                        Some(&MetricValue::from(2u64))
                    );
                    assert!(g.attributes.contains_key("env"));
                    assert!(g.attributes.contains_key("region"));
                }
                _ => panic!("unexpected env, region attributes in group"),
            }
        }
        assert!(prod_us_found && dev_us_found && dev_eu_found);
    }

    /// Validates line-protocol escaping rules for measurement names.
    #[test]
    fn test_escape_lp_measurement() {
        assert_eq!(escape_lp_measurement("cpu, name=avg"), "cpu\\,\\ name=avg");
        assert_eq!(escape_lp_measurement("plain"), "plain");
    }

    /// Validates line-protocol escaping rules for tag keys.
    #[test]
    fn test_escape_lp_tag_key() {
        assert_eq!(
            escape_lp_tag_key("host name,role=primary"),
            "host\\ name\\,role\\=primary"
        );
        assert_eq!(escape_lp_tag_key("plain"), "plain");
    }

    /// Validates line-protocol escaping rules for tag values.
    #[test]
    fn test_escape_lp_tag_value() {
        assert_eq!(
            escape_lp_tag_value("us west,zone=1"),
            "us\\ west\\,zone\\=1"
        );
        assert_eq!(escape_lp_tag_value("plain"), "plain");
    }

    static MMSC_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "latency_metrics",
        metrics: &[MetricsField {
            name: "request_duration",
            unit: "ms",
            instrument: Instrument::Mmsc,
            temporality: Some(Temporality::Delta),
            brief: "Request duration",
            value_type: MetricValueType::F64,
        }],
    };

    /// Ensures Prometheus rendering expands MMSC values into min/max/sum/count
    /// sub-metrics with expected HELP/TYPE lines.
    #[test]
    fn test_agg_prometheus_mmsc_metrics() {
        use otap_df_telemetry::instrument::MmscSnapshot;

        let groups = vec![AggregateGroup {
            name: "latency_metrics".to_string(),
            brief: &MMSC_METRICS_DESCRIPTOR,
            attributes: HashMap::new(),
            metrics: {
                let mut m = HashMap::new();
                let _ = m.insert(
                    "request_duration".to_string(),
                    MetricValue::Mmsc(MmscSnapshot {
                        min: 1.5,
                        max: 100.0,
                        sum: 250.5,
                        count: 10,
                    }),
                );
                m
            },
        }];

        let output = agg_prometheus_text(&groups, Some(1000), "");

        // Each sub-metric should have its own HELP and TYPE.
        // Unit `ms` adds the `_milliseconds` suffix per OTel→Prometheus spec.
        assert!(output.contains("# HELP request_duration_milliseconds_min Request duration\n"));
        assert!(output.contains("# TYPE request_duration_milliseconds_min gauge\n"));
        assert!(output.contains(
            "request_duration_milliseconds_min{otel_scope_name=\"latency_metrics\"} 1.5 1000\n"
        ));

        assert!(output.contains("# HELP request_duration_milliseconds_max Request duration\n"));
        assert!(output.contains("# TYPE request_duration_milliseconds_max gauge\n"));
        assert!(output.contains(
            "request_duration_milliseconds_max{otel_scope_name=\"latency_metrics\"} 100 1000\n"
        ));

        assert!(output.contains("# HELP request_duration_milliseconds_sum Request duration\n"));
        assert!(output.contains("# TYPE request_duration_milliseconds_sum counter\n"));
        assert!(output.contains(
            "request_duration_milliseconds_sum{otel_scope_name=\"latency_metrics\"} 250.5 1000\n"
        ));

        assert!(output.contains("# HELP request_duration_milliseconds_count Request duration\n"));
        assert!(output.contains("# TYPE request_duration_milliseconds_count counter\n"));
        assert!(output.contains(
            "request_duration_milliseconds_count{otel_scope_name=\"latency_metrics\"} 10 1000\n"
        ));

        // Should NOT contain the base metric name without suffix
        assert!(!output.contains("# TYPE request_duration gauge"));
        assert!(!output.contains("# TYPE request_duration counter"));
        assert!(!output.contains("# TYPE request_duration summary"));
        assert!(!output.contains("# TYPE request_duration histogram"));
    }

    /// Ensures line-protocol rendering outputs all MMSC sub-fields for a metric.
    #[test]
    fn test_agg_line_protocol_mmsc_metrics() {
        use otap_df_telemetry::instrument::MmscSnapshot;

        let groups = vec![AggregateGroup {
            name: "latency_metrics".to_string(),
            brief: &MMSC_METRICS_DESCRIPTOR,
            attributes: HashMap::new(),
            metrics: {
                let mut m = HashMap::new();
                let _ = m.insert(
                    "request_duration".to_string(),
                    MetricValue::Mmsc(MmscSnapshot {
                        min: 1.5,
                        max: 100.0,
                        sum: 250.5,
                        count: 10,
                    }),
                );
                m
            },
        }];

        let output = agg_line_protocol_text(&groups, Some(1000));

        // All four sub-fields should appear in a single line
        assert!(output.contains("request_duration_min=1.5"));
        assert!(output.contains("request_duration_max=100"));
        assert!(output.contains("request_duration_sum=250.5"));
        assert!(output.contains("request_duration_count=10i"));
    }

    // ---------------------------------------------------------------------------
    // LogFilter unit tests
    // ---------------------------------------------------------------------------

    fn make_log_entry(rendered: &str, level: &str, target: &str, timestamp: &str) -> LogEntry {
        LogEntry {
            seq: 1,
            timestamp: timestamp.to_string(),
            level: level.to_string(),
            target: target.to_string(),
            event_name: "test.event".to_string(),
            file: Some("src/lib.rs".to_string()),
            line: Some(1),
            rendered: rendered.to_string(),
            contexts: vec![],
        }
    }

    #[test]
    fn log_filter_no_criteria_always_matches() {
        let filter = LogFilter::default();
        let entry = make_log_entry("hello world", "INFO", "admin", "2026-01-01T00:00:00Z");
        assert!(filter.matches(&entry));
    }

    #[test]
    fn log_filter_search_query_matches_rendered() {
        let filter = LogFilter::from_params(Some("hello".to_string()), None, None);
        let yes = make_log_entry("say hello world", "INFO", "admin", "2026-01-01T00:00:00Z");
        let no = make_log_entry("goodbye world", "INFO", "admin", "2026-01-01T00:00:00Z");
        assert!(filter.matches(&yes));
        assert!(!filter.matches(&no));
    }

    #[test]
    fn log_filter_search_query_matches_level() {
        let filter = LogFilter::from_params(Some("WARN".to_string()), None, None);
        let yes = make_log_entry("msg", "WARN", "admin", "2026-01-01T00:00:00Z");
        let no = make_log_entry("msg", "INFO", "admin", "2026-01-01T00:00:00Z");
        assert!(filter.matches(&yes));
        assert!(!filter.matches(&no));
    }

    #[test]
    fn log_filter_search_query_is_case_insensitive() {
        let filter = LogFilter::from_params(Some("HELLO".to_string()), None, None);
        let entry = make_log_entry("say hello world", "INFO", "admin", "2026-01-01T00:00:00Z");
        assert!(filter.matches(&entry));
    }

    #[test]
    fn log_filter_minimum_timestamp_excludes_older_entries() {
        let filter = LogFilter::from_params(None, Some("2026-06-01T00:00:00Z".to_string()), None);
        let old_entry = make_log_entry("old", "INFO", "admin", "2026-01-01T00:00:00Z");
        let new_entry = make_log_entry("new", "INFO", "admin", "2026-07-01T00:00:00Z");
        assert!(!filter.matches(&old_entry));
        assert!(filter.matches(&new_entry));
    }

    #[test]
    fn log_filter_invalid_minimum_timestamp_is_ignored() {
        // A malformed timestamp should not crash; the filter passes without the constraint.
        let filter = LogFilter::from_params(None, Some("not-a-date".to_string()), None);
        let entry = make_log_entry("msg", "INFO", "admin", "2026-01-01T00:00:00Z");
        assert!(filter.matches(&entry));
    }

    /// Verifies that snapshot filtering is applied consistently: a filter that
    /// excludes an entry in the live stream must also exclude it in a snapshot.
    #[test]
    fn log_filter_applied_consistently_to_snapshot_logs() {
        let filter = LogFilter::from_params(Some("important".to_string()), None, None);
        let match_entry = make_log_entry(
            "important event occurred",
            "INFO",
            "admin",
            "2026-01-01T00:00:00Z",
        );
        let no_match_entry = make_log_entry(
            "routine heartbeat",
            "DEBUG",
            "admin",
            "2026-01-01T00:00:01Z",
        );

        assert!(filter.matches(&match_entry));
        assert!(!filter.matches(&no_match_entry));

        let mut logs = vec![match_entry, no_match_entry];
        logs.retain(|e| filter.matches(e));
        assert_eq!(logs.len(), 1);
        assert!(logs[0].rendered.contains("important"));
    }

    #[test]
    fn level_severity_ordering_is_correct() {
        assert!(level_severity("TRACE") < level_severity("DEBUG"));
        assert!(level_severity("DEBUG") < level_severity("INFO"));
        assert!(level_severity("INFO") < level_severity("WARN"));
        assert!(level_severity("WARN") < level_severity("ERROR"));
    }

    #[test]
    fn level_severity_is_case_insensitive() {
        assert_eq!(level_severity("error"), level_severity("ERROR"));
        assert_eq!(level_severity("warn"), level_severity("WARN"));
        assert_eq!(level_severity("info"), level_severity("INFO"));
        assert_eq!(level_severity("debug"), level_severity("DEBUG"));
        assert_eq!(level_severity("trace"), level_severity("TRACE"));
    }

    #[test]
    fn log_filter_minimum_level_excludes_lower_severity() {
        let filter = LogFilter::from_params(None, None, Some("WARN".to_string()));
        let trace = make_log_entry("msg", "TRACE", "t", "2026-01-01T00:00:00Z");
        let debug = make_log_entry("msg", "DEBUG", "t", "2026-01-01T00:00:00Z");
        let info = make_log_entry("msg", "INFO", "t", "2026-01-01T00:00:00Z");
        let warn = make_log_entry("msg", "WARN", "t", "2026-01-01T00:00:00Z");
        let error = make_log_entry("msg", "ERROR", "t", "2026-01-01T00:00:00Z");
        assert!(!filter.matches(&trace));
        assert!(!filter.matches(&debug));
        assert!(!filter.matches(&info));
        assert!(filter.matches(&warn));
        assert!(filter.matches(&error));
    }

    #[test]
    fn log_filter_minimum_level_all_disables_level_filter() {
        let filter = LogFilter::from_params(None, None, Some("ALL".to_string()));
        let trace = make_log_entry("msg", "TRACE", "t", "2026-01-01T00:00:00Z");
        let error = make_log_entry("msg", "ERROR", "t", "2026-01-01T00:00:00Z");
        assert!(filter.matches(&trace));
        assert!(filter.matches(&error));
    }

    #[test]
    fn log_filter_minimum_level_empty_string_disables_level_filter() {
        let filter = LogFilter::from_params(None, None, Some(String::new()));
        let trace = make_log_entry("msg", "TRACE", "t", "2026-01-01T00:00:00Z");
        assert!(filter.matches(&trace));
    }

    #[test]
    fn log_filter_minimum_level_and_search_query_combine() {
        let filter = LogFilter::from_params(
            Some("critical".to_string()),
            None,
            Some("ERROR".to_string()),
        );
        let match_entry = make_log_entry("critical failure", "ERROR", "t", "2026-01-01T00:00:00Z");
        let wrong_level = make_log_entry("critical info", "INFO", "t", "2026-01-01T00:00:00Z");
        let wrong_text = make_log_entry("normal error", "ERROR", "t", "2026-01-01T00:00:00Z");
        assert!(filter.matches(&match_entry));
        assert!(!filter.matches(&wrong_level));
        assert!(!filter.matches(&wrong_text));
    }

    // ---------------------------------------------------------------------------
    // WebSocket / HTTP schema alignment tests
    // ---------------------------------------------------------------------------

    #[cfg(feature = "live-logs-ws")]
    #[test]
    fn ws_log_msg_serializes_same_fields_as_api_log_entry() {
        let entry = make_log_entry("hello", "INFO", "admin", "2026-01-01T00:00:00Z");
        let expected_seq = entry.seq;
        let expected_timestamp = entry.timestamp.clone();
        let expected_level = entry.level.clone();
        let expected_target = entry.target.clone();
        let expected_event_name = entry.event_name.clone();
        let expected_rendered = entry.rendered.clone();
        let msg = WsServerMsg::Log { entry };
        let json: serde_json::Value = serde_json::to_value(&msg).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.get("type").unwrap(), "log");
        assert_eq!(obj.get("seq").unwrap(), expected_seq);
        assert_eq!(obj.get("timestamp").unwrap(), &expected_timestamp);
        assert_eq!(obj.get("level").unwrap(), &expected_level);
        assert_eq!(obj.get("target").unwrap(), &expected_target);
        assert_eq!(obj.get("event_name").unwrap(), &expected_event_name);
        assert_eq!(obj.get("rendered").unwrap(), &expected_rendered);
        assert!(obj.contains_key("contexts"));

        let roundtrip: api::LogEntry =
            serde_json::from_value(json).expect("log message should match api::LogEntry shape");
        assert_eq!(roundtrip.seq, 1);
        assert_eq!(roundtrip.rendered, "hello");
    }

    #[cfg(feature = "live-logs-ws")]
    #[test]
    fn ws_snapshot_logs_use_api_log_entry_shape() {
        let entry = make_log_entry("hello", "INFO", "admin", "2026-01-01T00:00:00Z");
        let msg = WsServerMsg::Snapshot {
            oldest_seq: Some(1),
            newest_seq: Some(1),
            next_seq: 1,
            truncated_before_seq: None,
            dropped_on_ingest: 0,
            dropped_on_retention: 0,
            retained_bytes: 0,
            logs: vec![entry],
        };
        let json: serde_json::Value = serde_json::to_value(&msg).unwrap();
        let logs = json.get("logs").unwrap().as_array().unwrap();
        assert_eq!(logs.len(), 1);

        let roundtrip: api::LogEntry = serde_json::from_value(logs[0].clone())
            .expect("snapshot log should match api::LogEntry");
        assert_eq!(roundtrip.seq, 1);
        assert_eq!(roundtrip.rendered, "hello");
    }

    // ---------------------------------------------------------------------
    // OTel→Prometheus metric name & unit suffix (build_prom_metric_name)
    // ---------------------------------------------------------------------

    #[test]
    fn test_build_prom_metric_name_counter_with_unit() {
        assert_eq!(
            build_prom_metric_name("http_request_duration", "s", Instrument::Counter),
            "http_request_duration_seconds_total"
        );
    }

    #[test]
    fn test_build_prom_metric_name_no_double_suffix() {
        assert_eq!(
            build_prom_metric_name("requests_total", "1", Instrument::Counter),
            "requests_total"
        );
    }

    #[test]
    fn test_build_prom_metric_name_gauge_with_unit() {
        assert_eq!(
            build_prom_metric_name("memory_usage", "By", Instrument::Gauge),
            "memory_usage_bytes"
        );
    }

    #[test]
    fn test_build_prom_metric_name_subtotal_gets_total() {
        // "subtotal" does NOT end with a proper "_total" suffix, so _total should be added.
        assert_eq!(
            build_prom_metric_name("subtotal", "1", Instrument::Counter),
            "subtotal_total"
        );
    }

    #[test]
    fn test_build_prom_metric_name_total_with_unit_orders_correctly() {
        // Per OTel spec: counter name ordering is `<base>_<unit>_total`.
        // A base name that already ends in `_total` must not push the unit
        // suffix after `_total` (would be `errors_total_bytes_total`).
        assert_eq!(
            build_prom_metric_name("errors_total", "By", Instrument::Counter),
            "errors_bytes_total"
        );
        // Already in spec-compliant form: don't duplicate the unit suffix.
        assert_eq!(
            build_prom_metric_name("errors_bytes_total", "By", Instrument::Counter),
            "errors_bytes_total"
        );
        // Case-insensitive `_total` recognition (sanitizer preserves case).
        assert_eq!(
            build_prom_metric_name("Errors_Total", "By", Instrument::Counter),
            "Errors_bytes_total"
        );
    }

    #[test]
    fn test_has_total_suffix_case_insensitive() {
        assert!(has_total_suffix("total"));
        assert!(has_total_suffix("Total"));
        assert!(has_total_suffix("TOTAL"));
        assert!(has_total_suffix("requests_total"));
        assert!(has_total_suffix("Requests_Total"));
        assert!(has_total_suffix("REQUESTS_TOTAL"));
        // Not a proper suffix: must be preceded by `_`.
        assert!(!has_total_suffix("subtotal"));
        assert!(!has_total_suffix("Subtotal"));
        assert!(!has_total_suffix(""));
    }

    #[test]
    fn test_ucum_to_prometheus_unit() {
        assert_eq!(ucum_to_prometheus_unit("By"), Some("bytes"));
        assert_eq!(ucum_to_prometheus_unit("s"), Some("seconds"));
        assert_eq!(ucum_to_prometheus_unit("1"), None);
        assert_eq!(ucum_to_prometheus_unit(""), None);
        assert_eq!(ucum_to_prometheus_unit("{requests}"), None);
    }

    #[test]
    fn test_ucum_to_prometheus_unit_bracketed_units() {
        // Pure annotation: {packet} → None
        assert_eq!(ucum_to_prometheus_unit("{packet}"), None);
        // Annotation with rate: {packet}/s → per_second (brackets stripped)
        assert_eq!(ucum_to_prometheus_unit("{packet}/s"), Some("per_second"));
        // Pure annotation: {requests} → None
        assert_eq!(ucum_to_prometheus_unit("{requests}"), None);
    }

    #[test]
    fn test_ucum_to_prometheus_unit_compound_rate_units() {
        assert_eq!(ucum_to_prometheus_unit("By/s"), Some("bytes_per_second"));
        assert_eq!(ucum_to_prometheus_unit("m/s"), Some("meters_per_second"));
        // "m" in UCUM is meters, not minutes ("min" is minutes).
        // By/m (bytes per meter) is not a meaningful rate, so returns None.
        assert_eq!(ucum_to_prometheus_unit("By/m"), None);
        // "min" is the correct UCUM code for minute
        assert_eq!(ucum_to_prometheus_unit("By/min"), Some("bytes_per_minute"));
        // Unsupported denominator
        assert_eq!(ucum_to_prometheus_unit("By/d"), None);
        // Newly supported via dynamic composition
        assert_eq!(
            ucum_to_prometheus_unit("KiBy/s"),
            Some("kibibytes_per_second")
        );
        assert_eq!(
            ucum_to_prometheus_unit("MiBy/s"),
            Some("mebibytes_per_second")
        );
        assert_eq!(ucum_to_prometheus_unit("g/s"), Some("grams_per_second"));
        assert_eq!(ucum_to_prometheus_unit("By/h"), Some("bytes_per_hour"));
        // Every simple unit gains second/minute/hour rates via rate_entries!.
        // Combinations the previous static table missed:
        assert_eq!(
            ucum_to_prometheus_unit("KiBy/h"),
            Some("kibibytes_per_hour")
        );
        assert_eq!(
            ucum_to_prometheus_unit("MiBy/min"),
            Some("mebibytes_per_minute")
        );
        assert_eq!(ucum_to_prometheus_unit("Hz/s"), Some("hertz_per_second"));
        assert_eq!(
            ucum_to_prometheus_unit("Cel/min"),
            Some("celsius_per_minute")
        );
        assert_eq!(ucum_to_prometheus_unit("V/h"), Some("volts_per_hour"));
        assert_eq!(ucum_to_prometheus_unit("W/s"), Some("watts_per_second"));
        assert_eq!(ucum_to_prometheus_unit("J/h"), Some("joules_per_hour"));
        assert_eq!(
            ucum_to_prometheus_unit("ms/s"),
            Some("milliseconds_per_second")
        );
    }

    #[test]
    fn test_strip_curly_braces() {
        assert_eq!(strip_curly_braces("{packet}/s"), "/s");
        assert_eq!(strip_curly_braces("{requests}"), "");
        assert_eq!(strip_curly_braces("By"), "By");
        assert_eq!(strip_curly_braces("{a}By{b}"), "By");
        // Unbalanced braces
        assert_eq!(strip_curly_braces("{unclosed"), "");
        assert_eq!(strip_curly_braces("extra}close"), "extraclose");
    }

    #[test]
    fn test_sanitize_prom_metric_name_collapses_underscores() {
        assert_eq!(sanitize_prom_metric_name("foo__bar___baz"), "foo_bar_baz");
    }

    #[test]
    fn test_sanitize_prom_metric_name_strips_trailing_underscore() {
        // A trailing non-alphanumeric character (e.g. `.`, `-`) sanitizes to
        // `_`. If left in place, downstream callers that append `_<unit>` or
        // `_total` would produce double underscores (e.g.
        // `foo.` → `foo_` → `foo__bytes`). `sanitize_prom_metric_name` strips
        // the trailing `_` so suffix-appending callers don't have to.
        assert_eq!(sanitize_prom_metric_name("foo."), "foo");
        assert_eq!(sanitize_prom_metric_name("foo___"), "foo");
        assert_eq!(sanitize_prom_metric_name("req.count."), "req_count");
        // After stripping, the unit suffix joins cleanly with a single `_`.
        assert_eq!(
            build_prom_metric_name("foo.", "By", Instrument::Gauge),
            "foo_bytes"
        );
        assert_eq!(
            build_prom_metric_name("req.count.", "By", Instrument::Counter),
            "req_count_bytes_total"
        );
        // All-invalid input still falls back to the placeholder.
        assert_eq!(sanitize_prom_metric_name("..."), "metric");
    }

    // ---------------------------------------------------------------------
    // Label key sanitization & collision merging
    // ---------------------------------------------------------------------

    #[test]
    fn test_sanitize_prom_label_key_collapses_underscores() {
        // Per OTel spec §Metric Attributes: "Multiple consecutive _ characters
        // SHOULD be replaced with a single _ character." This applies to label
        // keys, not just metric names.
        assert_eq!(sanitize_prom_label_key("foo..bar"), "foo_bar");
        assert_eq!(sanitize_prom_label_key("a__b___c"), "a_b_c");
        assert_eq!(sanitize_prom_label_key("trailing__"), "trailing_");
        assert_eq!(sanitize_prom_label_key(""), "label");
    }

    #[test]
    fn test_sanitize_and_merge_label_pairs_collisions_use_semicolon() {
        // Per OTel→Prometheus spec: when two original keys collide after
        // sanitization, their values are concatenated with `;`.
        let merged = sanitize_and_merge_label_pairs(
            vec![
                ("http.method", "GET".to_string()),
                ("http_method", "POST".to_string()),
            ],
            &[],
        );
        // Keys are merged into a single sanitized entry.
        assert_eq!(merged.len(), 1);
        // Values are joined in lexicographical order of the original keys
        // (`http.method` < `http_method` because `.` < `_`).
        assert_eq!(
            merged.get("http_method").map(String::as_str),
            Some("GET;POST")
        );
    }

    #[test]
    fn test_sanitize_and_merge_label_pairs_collision_is_lex_ordered_by_original_key() {
        // Per OTel→Prometheus spec: "values MUST be concatenated together,
        // separated by `;`, and ordered by the lexicographical order of the
        // original keys." This must hold regardless of caller iteration
        // order — including `HashMap::iter()`, which is unspecified.
        //
        // Three keys all sanitize to `service_name`. Lex order of the raw
        // keys is: "service-name" < "service.name" < "service_name".
        // Vary the input order to confirm the output is independent of it.
        let cases = [
            vec![
                ("service.name", "dot".to_string()),
                ("service_name", "underscore".to_string()),
                ("service-name", "dash".to_string()),
            ],
            vec![
                ("service_name", "underscore".to_string()),
                ("service-name", "dash".to_string()),
                ("service.name", "dot".to_string()),
            ],
            vec![
                ("service-name", "dash".to_string()),
                ("service.name", "dot".to_string()),
                ("service_name", "underscore".to_string()),
            ],
        ];
        for input in cases {
            let merged = sanitize_and_merge_label_pairs(input, &[]);
            assert_eq!(merged.len(), 1);
            assert_eq!(
                merged.get("service_name").map(String::as_str),
                Some("dash;dot;underscore"),
                "merge order must be lex-by-original-key, not caller order"
            );
        }

        // HashMap input (genuinely unordered) must also produce the same
        // deterministic output.
        let mut hm = HashMap::new();
        let _ = hm.insert("service.name", "dot".to_string());
        let _ = hm.insert("service_name", "underscore".to_string());
        let _ = hm.insert("service-name", "dash".to_string());
        let merged = sanitize_and_merge_label_pairs(hm.iter().map(|(k, v)| (*k, v.clone())), &[]);
        assert_eq!(
            merged.get("service_name").map(String::as_str),
            Some("dash;dot;underscore")
        );
    }

    #[test]
    fn test_sanitize_and_merge_label_pairs_distinct_keys_unchanged() {
        let merged = sanitize_and_merge_label_pairs(
            vec![("a", "1".to_string()), ("b", "2".to_string())],
            &[],
        );
        assert_eq!(merged.len(), 2);
        assert_eq!(merged.get("a").map(String::as_str), Some("1"));
        assert_eq!(merged.get("b").map(String::as_str), Some("2"));
    }

    #[test]
    fn test_sanitize_and_merge_label_pairs_drops_reserved_keys() {
        // Per OTel→Prometheus spec: `otel_scope_name` is emitted separately
        // from per-metric attributes. If a metric attribute key sanitizes to
        // that reserved name (e.g. raw key `otel.scope.name`), the
        // conflicting attribute is dropped to avoid Prometheus duplicate-
        // label rejection. (`otel_scope_version` is not currently emitted,
        // so user attributes with that name are not reserved.)
        let merged = sanitize_and_merge_label_pairs(
            vec![
                ("otel.scope.name", "user_value".to_string()),
                ("otel_scope_name", "literal_collision".to_string()),
                ("http_method", "GET".to_string()),
            ],
            RESERVED_SCOPE_LABEL_KEYS,
        );
        // Only the non-reserved key survives.
        assert_eq!(merged.len(), 1);
        assert_eq!(merged.get("http_method").map(String::as_str), Some("GET"));
        assert!(!merged.contains_key("otel_scope_name"));
    }

    /// Returns true if `output` contains a line of the shape
    /// `<prefix><labels><suffix>` where `<labels>` (the comma-separated
    /// content between `{` and `}`) is exactly the set in `expected_labels`,
    /// regardless of order. Used by tests that assert label *content* without
    /// pinning down emission order (Prometheus treats labels as unordered).
    fn line_has_labels(output: &str, prefix: &str, suffix: &str, expected_labels: &[&str]) -> bool {
        let expected: HashSet<&str> = expected_labels.iter().copied().collect();
        output.lines().any(|line| {
            if !line.starts_with(prefix) || !line.ends_with(suffix) {
                return false;
            }
            let labels_str = &line[prefix.len()..line.len() - suffix.len()];
            let actual: HashSet<&str> = labels_str.split(',').collect();
            actual == expected
        })
    }

    // -------------------------------------------------------------------
    // End-to-end integration test: format_prometheus_text with real metrics
    // -------------------------------------------------------------------

    use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
    use otap_df_telemetry::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, MetricValueType,
    };
    use otap_df_telemetry::metrics::{MetricSetHandler, MetricValue};

    #[derive(Debug)]
    struct E2eMetricSet {
        values: Vec<MetricValue>,
    }

    impl Default for E2eMetricSet {
        fn default() -> Self {
            Self {
                values: vec![
                    MetricValue::U64(0),   // http_requests counter
                    MetricValue::F64(0.0), // http_request_duration counter
                    MetricValue::U64(0),   // memory_usage gauge
                ],
            }
        }
    }

    static E2E_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "http_server",
        metrics: &[
            MetricsField {
                name: "http_requests",
                unit: "1",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                brief: "Total HTTP requests",
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "http_request_duration",
                unit: "s",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                brief: "Total request duration",
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "memory_usage",
                unit: "By",
                instrument: Instrument::Gauge,
                temporality: None,
                brief: "Current memory usage",
                value_type: MetricValueType::U64,
            },
        ],
    };

    static E2E_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "http_attrs",
        fields: &[AttributeField {
            key: "http.method",
            r#type: AttributeValueType::String,
            brief: "HTTP method",
        }],
    };

    impl MetricSetHandler for E2eMetricSet {
        fn descriptor(&self) -> &'static MetricsDescriptor {
            &E2E_METRICS_DESCRIPTOR
        }

        fn snapshot_values(&self) -> Vec<MetricValue> {
            self.values.clone()
        }

        fn clear_values(&mut self) {
            self.values.iter_mut().for_each(MetricValue::reset);
        }

        fn needs_flush(&self) -> bool {
            self.values.iter().any(|&v| !v.is_zero())
        }
    }

    #[derive(Debug)]
    struct E2eAttributeSet {
        values: Vec<AttributeValue>,
    }

    impl AttributeSetHandler for E2eAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &E2E_ATTRIBUTES_DESCRIPTOR
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    #[test]
    fn test_render_target_info_typed_resource_attributes() {
        // Verifies the typed `AttributeValue` path: numerics, booleans,
        // and arrays are rendered through `to_string_value()` (Display
        // for scalars, bare JSON for arrays) into Prometheus label values.
        let mut attrs: HashMap<String, ResourceAttributeValue> = HashMap::new();
        let _ = attrs.insert(
            "service.name".into(),
            ResourceAttributeValue::String("svc".into()),
        );
        let _ = attrs.insert(
            "service.instance.port".into(),
            ResourceAttributeValue::I64(4317),
        );
        let _ = attrs.insert(
            "deployment.staging".into(),
            ResourceAttributeValue::Bool(true),
        );
        let _ = attrs.insert(
            "service.tags".into(),
            ResourceAttributeValue::Array(
                otap_df_config::pipeline::telemetry::AttributeValueArray::String(vec![
                    "edge".into(),
                    "us-west".into(),
                ]),
            ),
        );

        let out = render_target_info(&attrs);
        assert!(out.contains("# HELP target_info Target metadata"));
        assert!(out.contains("# TYPE target_info gauge"));
        assert!(
            out.contains(r#"service_name="svc""#),
            "missing service_name. Output:\n{out}"
        );
        assert!(
            out.contains(r#"service_instance_port="4317""#),
            "i64 should render as decimal. Output:\n{out}"
        );
        assert!(
            out.contains(r#"deployment_staging="true""#),
            "bool should render as `true`/`false`. Output:\n{out}"
        );
        // JSON array escaping: the inner `"` must be escaped per Prometheus
        // label-value rules so the whole value parses as a single token.
        assert!(
            out.contains(r#"service_tags="[\"edge\",\"us-west\"]""#),
            "array should render as bare JSON with escaped quotes. Output:\n{out}"
        );
    }

    /// Full integration test: registers metrics, accumulates values, then validates
    /// the Prometheus text output follows OTel OTLP-to-Prometheus translation rules.
    #[test]
    fn test_format_prometheus_text_e2e_otel_compliance() {
        let registry = TelemetryRegistryHandle::new();

        // Register a metric set with attributes
        let metric_set = registry.register_metric_set::<E2eMetricSet>(E2eAttributeSet {
            values: vec![AttributeValue::String("GET".to_string())],
        });

        // Accumulate some metric values
        registry.accumulate_metric_set_snapshot(
            metric_set.metric_set_key(),
            &[
                MetricValue::U64(42),   // http_requests counter
                MetricValue::F64(1.25), // http_request_duration counter (seconds)
                MetricValue::U64(1024), // memory_usage gauge (bytes)
            ],
        );

        // Format with resource attributes for target_info
        let mut resource_attrs = HashMap::new();
        let _ = resource_attrs.insert(
            "service.name".to_string(),
            ResourceAttributeValue::String("my-service".to_string()),
        );
        let _ = resource_attrs.insert(
            "service.instance.id".to_string(),
            ResourceAttributeValue::String("host1:8080".to_string()),
        );

        let target_info = render_target_info(&resource_attrs);
        let output = format_prometheus_text(&registry, false, Some(1000), &target_info);

        // --- Validate target_info metric ---
        assert!(
            output.contains("# HELP target_info Target metadata\n"),
            "missing target_info HELP"
        );
        assert!(
            output.contains("# TYPE target_info gauge\n"),
            "missing target_info TYPE"
        );
        // target_info labels: order is unspecified (HashMap iteration).
        assert!(
            line_has_labels(
                &output,
                "target_info{",
                "} 1",
                &[
                    "service_instance_id=\"host1:8080\"",
                    "service_name=\"my-service\"",
                ],
            ),
            "target_info should carry both resource-derived labels. Output:\n{output}"
        );

        // --- Validate counter with _total suffix (dimensionless unit "1") ---
        assert!(
            output.contains("# HELP http_requests_total Total HTTP requests\n"),
            "counter should get _total suffix. Output:\n{output}"
        );
        assert!(
            output.contains("# TYPE http_requests_total counter\n"),
            "counter TYPE should be counter"
        );
        assert!(
            output.contains(
                "http_requests_total{otel_scope_name=\"http_server\",http_method=\"GET\"} 42 1000\n"
            ),
            "counter should have otel_scope_name and (omitted-when-empty) otel_scope_version labels. Output:\n{output}"
        );
        // No UNIT metadata for dimensionless "1"
        assert!(
            !output.contains("# UNIT http_requests_total"),
            "dimensionless counter should not have UNIT"
        );

        // --- Validate counter with unit suffix ---
        assert!(
            output.contains("# HELP http_request_duration_seconds_total Total request duration\n"),
            "counter with unit 's' should get _seconds_total suffix. Output:\n{output}"
        );
        assert!(
            output.contains("# UNIT http_request_duration_seconds_total seconds\n"),
            "should emit UNIT metadata for seconds"
        );
        assert!(
            output.contains("# TYPE http_request_duration_seconds_total counter\n"),
            "TYPE should be counter"
        );
        assert!(
            output.contains("http_request_duration_seconds_total{otel_scope_name=\"http_server\",http_method=\"GET\"} 1.25 1000\n"),
            "should have correct value with labels. Output:\n{output}"
        );

        // --- Validate gauge with unit suffix ---
        assert!(
            output.contains("# HELP memory_usage_bytes Current memory usage\n"),
            "gauge with unit 'By' should get _bytes suffix. Output:\n{output}"
        );
        assert!(
            output.contains("# UNIT memory_usage_bytes bytes\n"),
            "should emit UNIT metadata for bytes"
        );
        assert!(
            output.contains("# TYPE memory_usage_bytes gauge\n"),
            "TYPE should be gauge"
        );
        assert!(
            output.contains("memory_usage_bytes{otel_scope_name=\"http_server\",http_method=\"GET\"} 1024 1000\n"),
            "gauge should have correct value. Output:\n{output}"
        );

        // --- Validate labels ---
        assert!(!output.contains("set=\""), "should not use old 'set' label");
        assert!(
            output.contains("otel_scope_name=\"http_server\""),
            "should use otel_scope_name label"
        );
        assert!(
            !output.contains("otel_scope_version"),
            "otel_scope_version label should be omitted when empty"
        );

        // --- Validate no double _total suffix ---
        assert!(
            !output.contains("_total_total"),
            "should not double-add _total suffix"
        );
    }

    /// Verifies that multi-entity metrics are emitted as contiguous groups
    /// per the Prometheus exposition format spec: all samples for a given
    /// metric name must appear together, preceded by at most one HELP/TYPE.
    #[test]
    fn test_format_prometheus_text_multi_entity_contiguous_grouping() {
        let registry = TelemetryRegistryHandle::new();

        // Register two entities sharing the same metric set (simulates
        // multiple pipeline-thread cores).
        let ms1 = registry.register_metric_set::<E2eMetricSet>(E2eAttributeSet {
            values: vec![AttributeValue::String("core_0".to_string())],
        });
        let ms2 = registry.register_metric_set::<E2eMetricSet>(E2eAttributeSet {
            values: vec![AttributeValue::String("core_1".to_string())],
        });

        registry.accumulate_metric_set_snapshot(
            ms1.metric_set_key(),
            &[
                MetricValue::U64(100),
                MetricValue::F64(1.0),
                MetricValue::U64(512),
            ],
        );
        registry.accumulate_metric_set_snapshot(
            ms2.metric_set_key(),
            &[
                MetricValue::U64(200),
                MetricValue::F64(2.0),
                MetricValue::U64(1024),
            ],
        );

        let output = format_prometheus_text(&registry, false, None, "");

        // For each metric name, verify:
        // 1. Exactly one HELP and one TYPE directive exists
        // 2. All sample lines appear contiguously after the TYPE directive
        for metric_name in [
            "http_requests_total",
            "http_request_duration_seconds_total",
            "memory_usage_bytes",
        ] {
            let help_count = output.matches(&format!("# HELP {metric_name} ")).count();
            let type_count = output.matches(&format!("# TYPE {metric_name} ")).count();
            assert_eq!(
                help_count, 1,
                "expected exactly one HELP for {metric_name}, got {help_count}.\nOutput:\n{output}"
            );
            assert_eq!(
                type_count, 1,
                "expected exactly one TYPE for {metric_name}, got {type_count}.\nOutput:\n{output}"
            );

            // Verify contiguity: collect line indices for this metric's
            // samples and directives; they must form a contiguous block.
            let lines: Vec<(usize, &str)> = output
                .lines()
                .enumerate()
                .filter(|(_, l)| {
                    l.starts_with(&format!("{metric_name}{{"))
                        || l.starts_with(&format!("{metric_name} "))
                        || l.starts_with(&format!("# HELP {metric_name} "))
                        || l.starts_with(&format!("# UNIT {metric_name} "))
                        || l.starts_with(&format!("# TYPE {metric_name} "))
                })
                .collect();
            assert!(
                lines.len() >= 3,
                "expected at least 3 lines (HELP, TYPE, 2 samples) for {metric_name}, got {}.\nOutput:\n{output}",
                lines.len()
            );
            // Check that line indices are contiguous (no gaps).
            for window in lines.windows(2) {
                assert_eq!(
                    window[1].0,
                    window[0].0 + 1,
                    "lines for {metric_name} are not contiguous: line {} '{}' and line {} '{}' have a gap.\nOutput:\n{output}",
                    window[0].0,
                    window[0].1,
                    window[1].0,
                    window[1].1
                );
            }
        }

        // Both core_0 and core_1 samples should be present
        assert!(
            output.contains("core_0") && output.contains("core_1"),
            "output should contain samples from both entities.\nOutput:\n{output}"
        );
    }
}
