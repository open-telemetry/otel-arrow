// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry endpoints.
//!
//! - /telemetry/live-schema - current semantic conventions registry
//! - /telemetry/metrics - current aggregated metrics in JSON, line protocol, or Prometheus text format
//! - /telemetry/metrics/aggregate - aggregated metrics grouped by metric set name and optional attributes

use crate::AppState;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
use otap_df_telemetry::descriptor::{Instrument, MetricsDescriptor, MetricsField};
use otap_df_telemetry::registry::{MetricsIterator, MetricsRegistryHandle};
use otap_df_telemetry::semconv::SemConvRegistry;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

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
    value: u64,
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
    metrics: HashMap<String, u64>,
}

/// Output format for telemetry endpoints.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Json,
    JsonCompact,
    LineProtocol,
    Prometheus,
}

/// Query parameters for /telemetry/metrics
#[derive(Debug, Default, Deserialize)]
pub struct MetricsQuery {
    /// When true, reset metrics after reading. Default: true.
    #[serde(default = "default_true")]
    reset: bool,
    /// Output format: json (default), json_compact, line_protocol, prometheus
    #[serde(default)]
    format: Option<OutputFormat>,
}

/// Query parameters for /telemetry/metrics/aggregate
#[derive(Debug, Default, Deserialize)]
pub struct AggregateQuery {
    /// When true, reset metrics after reading. Default: true.
    #[serde(default = "default_true")]
    reset: bool,
    /// Comma-separated list of attribute names to group by (in addition to metric set name).
    #[serde(default)]
    attrs: Option<String>,
    /// Output format: json (default), json_compact, line_protocol, prometheus
    #[serde(default)]
    format: Option<OutputFormat>,
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
    metrics: HashMap<String, u64>,
}

#[inline]
const fn default_true() -> bool {
    true
}

/// Handler for the /live-schema endpoint.
///
/// This reflects the current live schema of the metrics registry.
pub async fn get_live_schema(
    State(state): State<AppState>,
) -> Result<Json<SemConvRegistry>, StatusCode> {
    Ok(Json(state.metrics_registry.generate_semconv_registry()))
}

/// Handler for the `/metrics` endpoint.
/// Supports multiple output formats and optional reset.
///
/// Query parameters:
/// - `reset` (bool, default true): whether to reset metrics after reading.
/// - `format` (string, default "json"): output format, one of "json", "json_compact", "line_protocol", "prometheus".
pub async fn get_metrics(
    State(state): State<AppState>,
    Query(q): Query<MetricsQuery>,
) -> Result<Response, StatusCode> {
    let now = chrono::Utc::now();
    let timestamp = now.to_rfc3339();

    // Resolve format (default json)
    let fmt = q.format.unwrap_or_default();

    match fmt {
        OutputFormat::Json => {
            // Snapshot with optional reset
            let metric_sets = if q.reset {
                collect_metrics_snapshot_and_reset(&state.metrics_registry)
            } else {
                collect_metrics_snapshot(&state.metrics_registry)
            };

            let response = MetricsWithMetadata {
                timestamp,
                metric_sets,
            };

            Ok(Json(response).into_response())
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
            Ok(Json(response).into_response())
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

/// Handler for the /telemetry/metrics/aggregate endpoint.
/// Aggregates metrics by metric set name and optionally by a list of attributes.
///
/// Query parameters:
/// - `reset` (bool, default true): whether to reset metrics after reading.
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

    // Resolve format (default json)
    let fmt = q.format.unwrap_or_default();

    match fmt {
        OutputFormat::Json => {
            let response = MetricsWithMetadata {
                timestamp,
                metric_sets: groups_with_metadata(&groups),
            };
            Ok(Json(response).into_response())
        }
        OutputFormat::JsonCompact => {
            let response = AllMetrics {
                timestamp,
                metric_sets: groups_without_metadata(&groups),
            };
            Ok(Json(response).into_response())
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
    registry: &MetricsRegistryHandle,
    reset: bool,
    group_attrs: &[&str],
) -> Vec<AggregateGroup> {
    // Aggregation map keyed by (set name, sorted list of (attr, Option<val_string>))
    type GroupKey = (String, Vec<(String, Option<String>)>);
    let mut agg: HashMap<
        GroupKey,
        (
            HashMap<String, AttributeValue>,
            HashMap<String, u64>,
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
            let entry = metrics_map.entry(field.name.to_string()).or_insert(0);
            *entry = entry.saturating_add(value);
        }
    };

    if reset {
        registry.visit_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        registry.visit_current_metrics(|d, a, m| visit(d, a, m));
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
                brief: "".to_string(),
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

fn agg_line_protocol_text(groups: &[AggregateGroup], timestamp_millis: Option<i64>) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {}", ms))
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
            if first {
                first = false;
            } else {
                fields.push(',');
            }
            let _ = write!(&mut fields, "{}={}i", escape_lp_field_key(fname), val);
        }
        if !fields.is_empty() {
            let _ = writeln!(&mut out, "{}{} {}{}", measurement, tags, fields, ts_suffix);
        }
    }

    out
}

fn agg_prometheus_text(groups: &[AggregateGroup], timestamp_millis: Option<i64>) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {}", ms))
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
                        Instrument::Histogram => "histogram",
                    };
                    let _ = writeln!(&mut out, "# TYPE {} {}", metric_name, prom_type);
                }
                if base_labels.is_empty() {
                    let _ = writeln!(&mut out, "{} {}{}", metric_name, value, ts_suffix);
                } else {
                    let _ = writeln!(
                        &mut out,
                        "{}{{{}}} {}{}",
                        metric_name, base_labels, value, ts_suffix
                    );
                }
            }
        }
    }

    out
}

/// Collects a snapshot of current metrics without resetting them.
fn collect_metrics_snapshot(registry: &MetricsRegistryHandle) -> Vec<MetricSetWithMetadata> {
    let mut metric_sets = Vec::new();

    registry.visit_current_metrics(|descriptor, attributes, metrics_iter| {
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
                brief: "".to_string(), // MetricsDescriptor doesn't have description field
                attributes: attrs_map,
                metrics,
            });
        }
    });

    metric_sets
}

/// Collects a snapshot of current metrics and resets them afterwards.
fn collect_metrics_snapshot_and_reset(
    registry: &MetricsRegistryHandle,
) -> Vec<MetricSetWithMetadata> {
    let mut metric_sets = Vec::new();

    registry.visit_metrics_and_reset(|descriptor, attributes, metrics_iter| {
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
    });

    metric_sets
}

/// Compact snapshot without resetting.
fn collect_compact_snapshot(registry: &MetricsRegistryHandle) -> Vec<MetricSet> {
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
fn collect_compact_snapshot_and_reset(registry: &MetricsRegistryHandle) -> Vec<MetricSet> {
    let mut metric_sets = Vec::new();

    registry.visit_metrics_and_reset(|descriptor, attributes, metrics_iter| {
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
    registry: &MetricsRegistryHandle,
    reset: bool,
    timestamp_millis: Option<i64>,
) -> String {
    let mut out = String::new();
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {}", ms))
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
            if first {
                first = false;
            } else {
                fields.push(',');
            }
            let _ = write!(
                &mut fields,
                "{}={}i",
                escape_lp_field_key(field.name),
                value
            );
        }

        if !fields.is_empty() {
            let _ = writeln!(&mut out, "{}{} {}{}", measurement, tags, fields, ts_suffix);
        }
    };

    if reset {
        registry.visit_metrics_and_reset(|d, a, m| visit(d, a, m));
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
    let ts_suffix = timestamp_millis
        .map(|ms| format!(" {}", ms))
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
                };
                let _ = writeln!(&mut out, "# TYPE {} {}", metric_name, prom_type);
            }

            if base_labels.is_empty() {
                let _ = writeln!(&mut out, "{} {}{}", metric_name, value, ts_suffix);
            } else {
                let _ = writeln!(
                    &mut out,
                    "{}{{{}}} {}{}",
                    metric_name, base_labels, value, ts_suffix
                );
            }
        }
    };

    if reset {
        registry.visit_metrics_and_reset(|d, a, m| visit(d, a, m));
    } else {
        registry.visit_current_metrics(|d, a, m| visit(d, a, m));
    }

    out
}

fn escape_lp_measurement(s: &str) -> String {
    s.replace(',', "\\,").replace(' ', "\\ ")
}

fn escape_lp_tag_key(s: &str) -> String {
    s.replace(',', "\\,")
        .replace(' ', "\\ ")
        .replace('=', "\\=")
}

fn escape_lp_tag_value(s: &str) -> String {
    s.replace(',', "\\,")
        .replace(' ', "\\ ")
        .replace('=', "\\=")
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

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_telemetry::descriptor::{Instrument, MetricsField};

    static TEST_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test_metrics",
        metrics: &[
            MetricsField {
                name: "requests_total",
                unit: "1",
                instrument: Instrument::Counter,
                brief: "Total number of requests",
            },
            MetricsField {
                name: "errors_total",
                unit: "1",
                instrument: Instrument::Counter,
                brief: "Total number of errors",
            },
        ],
    };

    static TEST_METRICS_DESCRIPTOR_2: MetricsDescriptor = MetricsDescriptor {
        name: "database_metrics",
        metrics: &[MetricsField {
            name: "connections_active",
            unit: "1",
            instrument: Instrument::Gauge,
            brief: "Active database connections",
        }],
    };

    #[test]
    fn test_aggregate_metric_groups_sorting_logic() {
        // Test the sorting logic with mock AggregateGroup structs
        let mut groups = vec![
            AggregateGroup {
                name: "zebra_metrics".to_string(),
                brief: &TEST_METRICS_DESCRIPTOR,
                attributes: HashMap::new(),
                metrics: {
                    let mut m = HashMap::new();
                    let _ = m.insert("metric1".to_string(), 10);
                    m
                },
            },
            AggregateGroup {
                name: "alpha_metrics".to_string(),
                brief: &TEST_METRICS_DESCRIPTOR_2,
                attributes: HashMap::new(),
                metrics: {
                    let mut m = HashMap::new();
                    let _ = m.insert("metric1".to_string(), 5);
                    let _ = m.insert("metric2".to_string(), 15);
                    m
                },
            },
            AggregateGroup {
                name: "alpha_metrics".to_string(),
                brief: &TEST_METRICS_DESCRIPTOR,
                attributes: HashMap::new(),
                metrics: {
                    let mut m = HashMap::new();
                    let _ = m.insert("metric1".to_string(), 8);
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
            fn snapshot_values(&self) -> Vec<u64> {
                vec![10, 1]
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
            fn snapshot_values(&self) -> Vec<u64> {
                vec![5, 0]
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
            fn snapshot_values(&self) -> Vec<u64> {
                vec![5, 4]
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
            fn snapshot_values(&self) -> Vec<u64> {
                vec![2, 2]
            } // requests_total=2, errors_total=2
            fn clear_values(&mut self) {}
            fn needs_flush(&self) -> bool {
                true
            }
        }

        // Build registry with two entries for the same metric set but different attributes
        let reg = MetricsRegistryHandle::new();
        let _m1: otap_df_telemetry::metrics::MetricSet<MetricSetA> =
            reg.register(MockAttrSet::new("prod", "us"));
        let _m2: otap_df_telemetry::metrics::MetricSet<MetricSetB> =
            reg.register(MockAttrSet::new("dev", "eu"));
        let _m3: otap_df_telemetry::metrics::MetricSet<MetricSetC> =
            reg.register(MockAttrSet::new("prod", "us"));
        let _m4: otap_df_telemetry::metrics::MetricSet<MetricSetD> =
            reg.register(MockAttrSet::new("dev", "us"));

        // Group by the "env" attribute and do not reset
        let groups = aggregate_metric_groups(&reg, false, &["env"]);

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
                    assert_eq!(g.metrics.get("requests_total"), Some(&(10 + 5)));
                    assert_eq!(g.metrics.get("errors_total"), Some(&(1 + 4)));
                    // Only grouped attribute should be present (env), not region
                    assert!(g.attributes.contains_key("env"));
                    assert!(!g.attributes.contains_key("region"));
                }
                Some("dev") => {
                    dev_found = true;
                    assert_eq!(g.metrics.get("requests_total"), Some(&(5 + 2)));
                    assert_eq!(g.metrics.get("errors_total"), Some(&(2)));
                    assert!(g.attributes.contains_key("env"));
                    assert!(!g.attributes.contains_key("region"));
                }
                _ => panic!("unexpected env attribute in group"),
            }
        }
        assert!(prod_found && dev_found);

        // Group by the "env" and region attributes and do not reset
        let groups = aggregate_metric_groups(&reg, false, &["env", "region"]);

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
                    assert_eq!(g.metrics.get("requests_total"), Some(&(10 + 5)));
                    assert_eq!(g.metrics.get("errors_total"), Some(&(1 + 4)));
                    assert!(g.attributes.contains_key("env"));
                    assert!(g.attributes.contains_key("region"));
                }
                (Some("dev"), Some("eu")) => {
                    dev_eu_found = true;
                    assert_eq!(g.metrics.get("requests_total"), Some(&(5)));
                    assert_eq!(g.metrics.get("errors_total"), Some(&(0)));
                    assert!(g.attributes.contains_key("env"));
                    assert!(g.attributes.contains_key("region"));
                }
                (Some("dev"), Some("us")) => {
                    dev_us_found = true;
                    assert_eq!(g.metrics.get("requests_total"), Some(&(2)));
                    assert_eq!(g.metrics.get("errors_total"), Some(&(2)));
                    assert!(g.attributes.contains_key("env"));
                    assert!(g.attributes.contains_key("region"));
                }
                _ => panic!("unexpected env, region attributes in group"),
            }
        }
        assert!(prod_us_found && dev_us_found && dev_eu_found);
    }
}
