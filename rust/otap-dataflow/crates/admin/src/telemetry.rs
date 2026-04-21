// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry endpoints.
//!
//! - /api/v1/telemetry/live-schema - current semantic conventions registry
//! - /api/v1/telemetry/logs - retained internal logs from the in-memory log tap
//! - /api/v1/telemetry/metrics - current aggregated metrics in JSON, line protocol, or Prometheus text format
//! - /api/v1/telemetry/metrics/aggregate - aggregated metrics grouped by metric set name and optional attributes

use crate::AppState;
use crate::convert::{convert_attribute_value, json_shape};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use otap_df_admin_types::telemetry as api;
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
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::Arc;
use tokio::sync::broadcast;

/// All the routes for telemetry.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/telemetry/live-schema", get(get_live_schema))
        .route("/telemetry/logs", get(get_logs))
        .route("/telemetry/logs/stream", get(ws_logs_stream))
        .route("/telemetry/metrics", get(get_metrics))
        .route("/telemetry/metrics/aggregate", get(get_metrics_aggregate))
        .route("/metrics", get(get_metrics))
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

fn logs_response(registry: &TelemetryRegistryHandle, result: LogQueryResult) -> api::LogsResponse {
    api::LogsResponse {
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

fn render_log_entry(registry: &TelemetryRegistryHandle, entry: &RetainedLogEvent) -> api::LogEntry {
    let callsite = entry.event.record.callsite();
    api::LogEntry {
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
) -> Vec<api::ResolvedLogContext> {
    event
        .record
        .context
        .iter()
        .map(|entity_key| {
            registry
                .visit_entity(*entity_key, |attrs| api::ResolvedLogContext {
                    entity_key: format!("{entity_key:?}"),
                    schema_name: Some(attrs.schema_name().to_string()),
                    attributes: attrs
                        .iter_attributes()
                        .map(|(key, value)| (key.to_string(), convert_attribute_value(value)))
                        .collect(),
                })
                .unwrap_or_else(|| api::ResolvedLogContext {
                    entity_key: format!("{entity_key:?}"),
                    schema_name: None,
                    attributes: BTreeMap::new(),
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
    Ok(Json(logs_response(&state.metrics_registry, result)))
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
                format_prometheus_text(&state.metrics_registry, true, Some(now.timestamp_millis()))
            } else {
                format_prometheus_text(&state.metrics_registry, false, Some(now.timestamp_millis()))
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
            let body = agg_prometheus_text(&groups, Some(now.timestamp_millis()));
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

fn agg_prometheus_text(groups: &[AggregateGroup], timestamp_millis: Option<i64>) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {ms}"))
        .unwrap_or_default();
    let mut seen: HashSet<String> = HashSet::new();

    for g in groups {
        // Base labels include set name and selected attributes
        let mut base_labels = String::new();
        if !g.name.is_empty() {
            let _ = write!(
                &mut base_labels,
                "set=\"{}\"",
                escape_prom_label_value(&g.name)
            );
        }
        // ensure deterministic order of attributes in output
        let mut attrs: Vec<(&String, &AttributeValue)> = g.attributes.iter().collect();
        attrs.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in attrs {
            if !base_labels.is_empty() {
                base_labels.push(',');
            }
            let _ = write!(
                &mut base_labels,
                "{}=\"{}\"",
                sanitize_prom_label_key(k),
                escape_prom_label_value(&v.to_string_value())
            );
        }

        // Emit metrics for this group
        for field in g.brief.metrics.iter() {
            if let Some(value) = g.metrics.get(field.name) {
                let metric_name = sanitize_prom_metric_name(field.name);
                match value {
                    MetricValue::U64(_) | MetricValue::F64(_) => {
                        if seen.insert(metric_name.clone()) {
                            if !field.brief.is_empty() {
                                let _ = writeln!(
                                    &mut out,
                                    "# HELP {} {}",
                                    metric_name,
                                    escape_prom_help(field.brief)
                                );
                            }
                            let prom_type = match field.instrument {
                                Instrument::Counter => "counter",
                                Instrument::UpDownCounter => "gauge",
                                Instrument::Gauge => "gauge",
                                Instrument::Histogram => "gauge",
                                Instrument::Mmsc => unreachable!("MMSC is not a scalar"),
                            };
                            let _ = writeln!(&mut out, "# TYPE {metric_name} {prom_type}");
                        }
                        let value_str = format_prom_value(*value, Some(field.value_type));
                        if base_labels.is_empty() {
                            let _ = writeln!(&mut out, "{metric_name} {value_str}{ts_suffix}");
                        } else {
                            let _ = writeln!(
                                &mut out,
                                "{metric_name}{{{base_labels}}} {value_str}{ts_suffix}"
                            );
                        }
                    }
                    MetricValue::Mmsc(s) => {
                        if s.count == 0 {
                            continue;
                        }
                        let brief = escape_prom_help(field.brief);
                        for (suffix, prom_type, val) in [
                            ("_min", "gauge", s.min),
                            ("_max", "gauge", s.max),
                            ("_sum", "counter", s.sum),
                        ] {
                            let sub_name = format!("{metric_name}{suffix}");
                            if seen.insert(sub_name.clone()) {
                                if !field.brief.is_empty() {
                                    let _ = writeln!(&mut out, "# HELP {sub_name} {brief}");
                                }
                                let _ = writeln!(&mut out, "# TYPE {sub_name} {prom_type}");
                            }
                            if base_labels.is_empty() {
                                let _ = writeln!(&mut out, "{sub_name} {val}{ts_suffix}");
                            } else {
                                let _ = writeln!(
                                    &mut out,
                                    "{sub_name}{{{base_labels}}} {val}{ts_suffix}"
                                );
                            }
                        }
                        // _count as counter with integer value
                        let count_name = format!("{metric_name}_count");
                        if seen.insert(count_name.clone()) {
                            if !field.brief.is_empty() {
                                let _ = writeln!(&mut out, "# HELP {count_name} {brief}");
                            }
                            let _ = writeln!(&mut out, "# TYPE {count_name} counter");
                        }
                        if base_labels.is_empty() {
                            let _ = writeln!(&mut out, "{count_name} {}{ts_suffix}", s.count);
                        } else {
                            let _ = writeln!(
                                &mut out,
                                "{count_name}{{{base_labels}}} {}{ts_suffix}",
                                s.count
                            );
                        }
                    }
                }
            }
        }
    }

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
) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {ms}"))
        .unwrap_or_default();
    let mut seen: HashSet<String> = HashSet::new();

    let mut visit = |descriptor: &'static MetricsDescriptor,
                     attributes: &dyn AttributeSetHandler,
                     metrics_iter: MetricsIterator<'_>| {
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
            if !base_labels.is_empty() {
                base_labels.push(',');
            }
            let _ = write!(
                &mut base_labels,
                "{}=\"{}\"",
                sanitize_prom_label_key(key),
                escape_prom_label_value(&value.to_string_value())
            );
        }

        for (field, value) in metrics_iter {
            let metric_name = sanitize_prom_metric_name(field.name);

            match value {
                MetricValue::U64(_) | MetricValue::F64(_) => {
                    // HELP/TYPE once per metric name
                    if seen.insert(metric_name.clone()) {
                        if !field.brief.is_empty() {
                            let _ = writeln!(
                                &mut out,
                                "# HELP {} {}",
                                metric_name,
                                escape_prom_help(field.brief)
                            );
                        }
                        let prom_type = match field.instrument {
                            Instrument::Counter => "counter",
                            Instrument::UpDownCounter => "gauge",
                            Instrument::Gauge => "gauge",
                            Instrument::Histogram => "gauge",
                            Instrument::Mmsc => unreachable!("MMSC is not a scalar"),
                        };
                        let _ = writeln!(&mut out, "# TYPE {metric_name} {prom_type}");
                    }
                    let value_str = format_prom_value(value, Some(field.value_type));
                    if base_labels.is_empty() {
                        let _ = writeln!(&mut out, "{metric_name} {value_str}{ts_suffix}");
                    } else {
                        let _ = writeln!(
                            &mut out,
                            "{metric_name}{{{base_labels}}} {value_str}{ts_suffix}"
                        );
                    }
                }
                MetricValue::Mmsc(s) => {
                    if s.count == 0 {
                        continue;
                    }
                    let brief = escape_prom_help(field.brief);
                    for (suffix, prom_type, val) in [
                        ("_min", "gauge", s.min),
                        ("_max", "gauge", s.max),
                        ("_sum", "counter", s.sum),
                    ] {
                        let sub_name = format!("{metric_name}{suffix}");
                        if seen.insert(sub_name.clone()) {
                            if !field.brief.is_empty() {
                                let _ = writeln!(&mut out, "# HELP {sub_name} {brief}");
                            }
                            let _ = writeln!(&mut out, "# TYPE {sub_name} {prom_type}");
                        }
                        if base_labels.is_empty() {
                            let _ = writeln!(&mut out, "{sub_name} {val}{ts_suffix}");
                        } else {
                            let _ =
                                writeln!(&mut out, "{sub_name}{{{base_labels}}} {val}{ts_suffix}");
                        }
                    }
                    // _count as counter with integer value
                    let count_name = format!("{metric_name}_count");
                    if seen.insert(count_name.clone()) {
                        if !field.brief.is_empty() {
                            let _ = writeln!(&mut out, "# HELP {count_name} {brief}");
                        }
                        let _ = writeln!(&mut out, "# TYPE {count_name} counter");
                    }
                    if base_labels.is_empty() {
                        let _ = writeln!(&mut out, "{count_name} {}{ts_suffix}", s.count);
                    } else {
                        let _ = writeln!(
                            &mut out,
                            "{count_name}{{{base_labels}}} {}{ts_suffix}",
                            s.count
                        );
                    }
                }
            }
        }
    };

    if reset {
        telemetry_registry.visit_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        telemetry_registry.visit_current_metrics(|d, a, m| visit(d, a, m));
    }

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
        "metric".to_string()
    } else {
        out
    }
}

fn sanitize_prom_label_key(s: &str) -> String {
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
        "label".to_string()
    } else {
        out
    }
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

// ---------------------------------------------------------------------------
// WebSocket live log stream  (/api/v1/telemetry/logs/stream)
// ---------------------------------------------------------------------------

/// Map a level string to a numeric severity (TRACE=0 … ERROR=4).
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

impl LogFilter {
    /// Returns `true` when the rendered log entry passes all active criteria.
    fn matches(&self, entry: &api::LogEntry) -> bool {
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
    /// events that would be rejected anyway.  `search_query` is intentionally
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

/// Client → server WebSocket messages.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum WsClientMsg {
    /// Begin streaming.  Sends an initial retained-log snapshot, then follows
    /// with live events.
    Subscribe {
        /// Cursor: only include retained entries strictly newer than this seq.
        after: Option<u64>,
        /// Maximum retained entries in the initial snapshot (clamped 1–1000).
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

/// Server → client WebSocket messages.
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
        logs: Vec<api::LogEntry>,
    },
    /// Single live log entry pushed by the server.
    Log {
        #[serde(flatten)]
        entry: api::LogEntry,
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
async fn ws_logs_stream(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_ws_logs(socket, state))
}

/// Send a serialized `WsServerMsg` as a WebSocket text frame.
///
/// Returns `false` if the socket is closed and the caller should stop sending.
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
///    socket.  While paused the server still drains the broadcast channel so
///    that the producer is never slowed by this client.
/// 4. On `backfill` the server re-queries the retained ring buffer and sends a
///    `snapshot`.  The cursor is updated so subsequent live events do not
///    duplicate.
/// 5. If the client falls more than `SUBSCRIBER_CHANNEL_CAPACITY` events
///    behind, the broadcast channel drops the overflow; the server notifies the
///    client with an `error` message so it can issue a `backfill`.
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
    // deliberately skipped while paused).  Used in `state` replies so the
    // client knows where the live cursor stands.
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
                                    // Only advance cursor — never move it backward.  A
                                    // client may request an older `after` (e.g. a lag
                                    // gap backfill) while the live stream has already
                                    // moved the cursor forward; preserving the maximum
                                    // keeps the dedup guard in the live event arm sound.
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
                        Some(Ok(_)) => {} // binary / ping frames — ignore
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
                                // Discard silently — already in the snapshot.
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
                            // receiver slot.  `cursor` here is the last seq we
                            // successfully delivered — the client can use it as
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
                        // the query and the first receive.  Live events with
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
    use axum::body::to_bytes;
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_engine::memory_limiter::MemoryPressureState;
    use otap_df_state::store::ObservedStateStore;
    use otap_df_telemetry::descriptor::{Instrument, MetricsField, Temporality};
    use std::sync::Arc;
    use tokio::sync::Mutex;

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
            log_tap: None,
            ctrl_msg_senders: Arc::new(Mutex::new(Vec::new())),
            memory_pressure_state: MemoryPressureState::default(),
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

        let output = agg_prometheus_text(&groups, Some(1000));

        // Each sub-metric should have its own HELP and TYPE
        assert!(output.contains("# HELP request_duration_min Request duration\n"));
        assert!(output.contains("# TYPE request_duration_min gauge\n"));
        assert!(output.contains("request_duration_min{set=\"latency_metrics\"} 1.5 1000\n"));

        assert!(output.contains("# HELP request_duration_max Request duration\n"));
        assert!(output.contains("# TYPE request_duration_max gauge\n"));
        assert!(output.contains("request_duration_max{set=\"latency_metrics\"} 100 1000\n"));

        assert!(output.contains("# HELP request_duration_sum Request duration\n"));
        assert!(output.contains("# TYPE request_duration_sum counter\n"));
        assert!(output.contains("request_duration_sum{set=\"latency_metrics\"} 250.5 1000\n"));

        assert!(output.contains("# HELP request_duration_count Request duration\n"));
        assert!(output.contains("# TYPE request_duration_count counter\n"));
        assert!(output.contains("request_duration_count{set=\"latency_metrics\"} 10 1000\n"));

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

    fn make_log_entry(rendered: &str, level: &str, target: &str, timestamp: &str) -> api::LogEntry {
        api::LogEntry {
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

        // The filter must pass the matching entry and reject the other.
        assert!(filter.matches(&match_entry));
        assert!(!filter.matches(&no_match_entry));

        // Simulate the retain() call used in ws_send_snapshot.
        let mut logs = vec![match_entry, no_match_entry];
        logs.retain(|e| filter.matches(e));
        assert_eq!(logs.len(), 1);
        assert!(logs[0].rendered.contains("important"));
    }

    #[test]
    fn level_severity_ordering_is_correct() {
        // TRACE < DEBUG < INFO < WARN < ERROR
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
        // Both constraints must pass.
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
    // WebSocket ↔ HTTP schema alignment tests
    // ---------------------------------------------------------------------------

    #[test]
    fn ws_log_msg_serializes_same_fields_as_api_log_entry() {
        let entry = make_log_entry("hello", "INFO", "admin", "2026-01-01T00:00:00Z");
        let msg = WsServerMsg::Log {
            entry: entry.clone(),
        };
        let json: serde_json::Value = serde_json::to_value(&msg).unwrap();
        let obj = json.as_object().unwrap();

        // The flattened entry must carry the same fields as api::LogEntry
        // plus the discriminator tag.
        assert_eq!(obj.get("type").unwrap(), "log");
        assert_eq!(obj.get("seq").unwrap(), entry.seq);
        assert_eq!(obj.get("timestamp").unwrap(), &entry.timestamp);
        assert_eq!(obj.get("level").unwrap(), &entry.level);
        assert_eq!(obj.get("target").unwrap(), &entry.target);
        assert_eq!(obj.get("event_name").unwrap(), &entry.event_name);
        assert_eq!(obj.get("rendered").unwrap(), &entry.rendered);
        assert!(obj.contains_key("contexts"));
    }

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

        // Each log in the snapshot must deserialize as a valid api::LogEntry.
        let roundtrip: api::LogEntry = serde_json::from_value(logs[0].clone())
            .expect("snapshot log should match api::LogEntry");
        assert_eq!(roundtrip.seq, 1);
        assert_eq!(roundtrip.rendered, "hello");
    }
}
