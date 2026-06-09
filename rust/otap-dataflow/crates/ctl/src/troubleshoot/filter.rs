// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Client-side filtering helpers for troubleshooting events, retained logs, and metrics.
//!
//! The admin API returns broad status and telemetry snapshots, while CLI users
//! often ask for a narrower pipeline, node, event kind, or metric subset. This
//! module applies those scopes after retrieval while preserving response
//! metadata such as cursors and timestamps, keeping filtering behavior shared
//! across snapshot commands, watch commands, bundles, diagnosis, and the TUI.

use super::models::{EventFilters, LogFilters, MetricsFilters, NormalizedEvent};
use otap_df_admin_api::telemetry;
use std::collections::BTreeMap;

/// Applies retained-log filters while preserving response cursor metadata.
pub fn filter_logs(
    response: &telemetry::LogsResponse,
    filters: &LogFilters,
) -> telemetry::LogsResponse {
    telemetry::LogsResponse {
        oldest_seq: response.oldest_seq,
        newest_seq: response.newest_seq,
        next_seq: response.next_seq,
        truncated_before_seq: response.truncated_before_seq,
        dropped_on_ingest: response.dropped_on_ingest,
        dropped_on_retention: response.dropped_on_retention,
        retained_bytes: response.retained_bytes,
        logs: response
            .logs
            .iter()
            .filter(|entry| log_matches(entry, filters))
            .cloned()
            .collect(),
    }
}

/// Applies metric-set and metric filters to a compact metrics response.
pub fn filter_metrics_compact(
    response: &telemetry::CompactMetricsResponse,
    filters: &MetricsFilters,
) -> telemetry::CompactMetricsResponse {
    telemetry::CompactMetricsResponse {
        timestamp: response.timestamp.clone(),
        metric_sets: response
            .metric_sets
            .iter()
            .filter_map(|metric_set| filter_compact_metric_set(metric_set, filters))
            .collect(),
    }
}

/// Applies metric-set and metric filters to a full metrics response.
pub fn filter_metrics_full(
    response: &telemetry::MetricsResponse,
    filters: &MetricsFilters,
) -> telemetry::MetricsResponse {
    telemetry::MetricsResponse {
        timestamp: response.timestamp.clone(),
        metric_sets: response
            .metric_sets
            .iter()
            .filter_map(|metric_set| filter_full_metric_set(metric_set, filters))
            .collect(),
    }
}

/// Returns true when a normalized event satisfies all configured event filters.
pub(super) fn event_matches(event: &NormalizedEvent, filters: &EventFilters) -> bool {
    if !filters.kinds.is_empty() && !filters.kinds.contains(&event.kind) {
        return false;
    }
    if !string_option_matches(&filters.pipeline_group_id, &event.pipeline_group_id) {
        return false;
    }
    if !string_option_matches(&filters.pipeline_id, &event.pipeline_id) {
        return false;
    }
    if !option_eq(&filters.node_id, event.node_id.as_deref()) {
        return false;
    }
    if let Some(contains) = &filters.contains {
        let contains = contains.to_ascii_lowercase();
        if ![
            event.name.to_ascii_lowercase(),
            event
                .message
                .as_ref()
                .map(|value| value.to_ascii_lowercase())
                .unwrap_or_default(),
            event
                .detail
                .as_ref()
                .map(|value| value.to_ascii_lowercase())
                .unwrap_or_default(),
            event
                .record
                .as_ref()
                .map(log_record_summary)
                .unwrap_or_default()
                .to_ascii_lowercase(),
        ]
        .iter()
        .any(|value| value.contains(&contains))
        {
            return false;
        }
    }
    true
}

fn log_matches(entry: &telemetry::LogEntry, filters: &LogFilters) -> bool {
    if !option_eq_ci(&filters.level, Some(&entry.level)) {
        return false;
    }
    if !option_contains_ci(&filters.target, &entry.target) {
        return false;
    }
    if !option_contains_ci(&filters.event, &entry.event_name) {
        return false;
    }
    if !context_attr_matches(
        &entry.contexts,
        &filters.pipeline_group_id,
        &["pipeline.group.id", "pipeline_group_id"],
    ) {
        return false;
    }
    if !context_attr_matches(
        &entry.contexts,
        &filters.pipeline_id,
        &["pipeline.id", "pipeline_id"],
    ) {
        return false;
    }
    if !context_attr_matches(&entry.contexts, &filters.node_id, &["node.id", "node_id"]) {
        return false;
    }
    if let Some(contains) = &filters.contains {
        let contains = contains.to_ascii_lowercase();
        let context_blob = entry
            .contexts
            .iter()
            .map(context_summary)
            .collect::<Vec<_>>()
            .join(" ");
        if ![
            entry.rendered.to_ascii_lowercase(),
            entry.target.to_ascii_lowercase(),
            entry.event_name.to_ascii_lowercase(),
            context_blob.to_ascii_lowercase(),
        ]
        .iter()
        .any(|value| value.contains(&contains))
        {
            return false;
        }
    }
    true
}

fn filter_compact_metric_set(
    metric_set: &telemetry::MetricSet,
    filters: &MetricsFilters,
) -> Option<telemetry::MetricSet> {
    if !metric_set_matches(&metric_set.name, &metric_set.attributes, filters) {
        return None;
    }
    let metrics = metric_set
        .metrics
        .iter()
        .filter(|(name, _)| metric_name_matches(name, &filters.metric_names))
        .map(|(name, value)| (name.clone(), *value))
        .collect::<BTreeMap<_, _>>();
    if !filters.metric_names.is_empty() && metrics.is_empty() {
        return None;
    }
    Some(telemetry::MetricSet {
        name: metric_set.name.clone(),
        attributes: metric_set.attributes.clone(),
        metrics,
    })
}

fn filter_full_metric_set(
    metric_set: &telemetry::MetricSetWithMetadata,
    filters: &MetricsFilters,
) -> Option<telemetry::MetricSetWithMetadata> {
    if !metric_set_matches(&metric_set.name, &metric_set.attributes, filters) {
        return None;
    }
    let metrics = metric_set
        .metrics
        .iter()
        .filter(|point| metric_name_matches(&point.metadata.name, &filters.metric_names))
        .cloned()
        .collect::<Vec<_>>();
    if !filters.metric_names.is_empty() && metrics.is_empty() {
        return None;
    }
    Some(telemetry::MetricSetWithMetadata {
        name: metric_set.name.clone(),
        brief: metric_set.brief.clone(),
        attributes: metric_set.attributes.clone(),
        metrics,
    })
}

fn metric_set_matches(
    metric_set_name: &str,
    attributes: &BTreeMap<String, telemetry::AttributeValue>,
    filters: &MetricsFilters,
) -> bool {
    if !filters.metric_sets.is_empty()
        && !filters
            .metric_sets
            .iter()
            .any(|value| eq_ci(metric_set_name, value))
    {
        return false;
    }
    if !attribute_matches(
        attributes,
        &filters.pipeline_group_id,
        &["pipeline.group.id", "pipeline_group_id"],
    ) {
        return false;
    }
    if !attribute_matches(
        attributes,
        &filters.pipeline_id,
        &["pipeline.id", "pipeline_id"],
    ) {
        return false;
    }
    if !attribute_matches(
        attributes,
        &filters.core_id.map(|value| value.to_string()),
        &["core.id", "core_id"],
    ) {
        return false;
    }
    if !attribute_matches(attributes, &filters.node_id, &["node.id", "node_id"]) {
        return false;
    }
    true
}

fn metric_name_matches(metric_name: &str, filters: &[String]) -> bool {
    filters.is_empty() || filters.iter().any(|value| eq_ci(metric_name, value))
}

fn string_option_matches(expected: &Option<String>, actual: &str) -> bool {
    expected.as_ref().is_none_or(|value| eq_ci(value, actual))
}

fn option_eq(expected: &Option<String>, actual: Option<&str>) -> bool {
    expected
        .as_ref()
        .is_none_or(|value| actual.is_some_and(|actual| eq_ci(value, actual)))
}

fn option_eq_ci(expected: &Option<String>, actual: Option<&str>) -> bool {
    expected
        .as_ref()
        .is_none_or(|value| actual.is_some_and(|actual| eq_ci(value, actual)))
}

fn option_contains_ci(needle: &Option<String>, haystack: &str) -> bool {
    needle
        .as_ref()
        .is_none_or(|value| contains_ci(haystack, value))
}

fn context_attr_matches(
    contexts: &[telemetry::ResolvedLogContext],
    expected: &Option<String>,
    keys: &[&str],
) -> bool {
    expected.as_ref().is_none_or(|expected| {
        contexts.iter().any(|context| {
            keys.iter()
                .filter_map(|key| context.attributes.get(*key))
                .any(|value| eq_ci(expected, &attribute_value_string(value)))
        })
    })
}

fn attribute_matches(
    attributes: &BTreeMap<String, telemetry::AttributeValue>,
    expected: &Option<String>,
    keys: &[&str],
) -> bool {
    expected.as_ref().is_none_or(|expected| {
        keys.iter()
            .filter_map(|key| attributes.get(*key))
            .any(|value| eq_ci(expected, &attribute_value_string(value)))
    })
}

fn attribute_value_string(value: &telemetry::AttributeValue) -> String {
    match value {
        telemetry::AttributeValue::String(value) => value.clone(),
        telemetry::AttributeValue::Int(value) => value.to_string(),
        telemetry::AttributeValue::UInt(value) => value.to_string(),
        telemetry::AttributeValue::Double(value) => value.to_string(),
        telemetry::AttributeValue::Boolean(value) => value.to_string(),
        telemetry::AttributeValue::Map(value) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn context_summary(context: &telemetry::ResolvedLogContext) -> String {
    let attrs = context
        .attributes
        .iter()
        .map(|(key, value)| format!("{key}={}", attribute_value_string(value)))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{} {} {}",
        context.entity_key,
        context.schema_name.as_deref().unwrap_or(""),
        attrs
    )
}

fn log_record_summary(record: &serde_json::Value) -> String {
    serde_json::to_string(record).unwrap_or_else(|_| record.to_string())
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn eq_ci(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}
