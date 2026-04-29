// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Scenario tests for the troubleshoot describe, filter, and diagnosis helpers.
//!
//! The tests in this module document the behavior expected from client-side
//! troubleshooting evidence: filters must preserve metadata, describe helpers
//! must normalize status consistently, and diagnosis heuristics must produce
//! stable findings that are useful to CLI users, the TUI, and automation.

use super::*;
use serde_json::json;
use std::collections::BTreeMap;

fn sample_event(name: &str) -> NormalizedEvent {
    NormalizedEvent {
        time: "2026-01-01T00:00:00Z".to_string(),
        kind: NormalizedEventKind::Error,
        name: name.to_string(),
        pipeline_group_id: "group-a".to_string(),
        pipeline_id: "pipeline-a".to_string(),
        core_id: 1,
        deployment_generation: Some(7),
        node_id: Some("receiver".to_string()),
        node_kind: Some("receiver".to_string()),
        message: Some("message".to_string()),
        detail: Some("detail".to_string()),
        record: None,
    }
}

/// Scenario: event filtering is applied to one normalized runtime event.
/// Guarantees: scope, kind, node, and substring matching all participate in the
/// filter decision.
#[test]
fn event_filter_matches_scope_kind_and_contains() {
    let event = sample_event("DrainDeadlineReached");
    let filters = EventFilters {
        kinds: vec![NormalizedEventKind::Error],
        pipeline_group_id: Some("group-a".to_string()),
        pipeline_id: Some("pipeline-a".to_string()),
        node_id: Some("receiver".to_string()),
        contains: Some("deadline".to_string()),
    };

    assert!(filter::event_matches(&event, &filters));
}

/// Scenario: retained admin logs are filtered by level, target, event name, and
/// scoped context attributes.
/// Guarantees: client-side log filtering preserves only entries that satisfy
/// every requested filter clause.
#[test]
fn log_filter_matches_context_attributes_and_contains() {
    let response = otap_df_admin_api::telemetry::LogsResponse {
        oldest_seq: Some(1),
        newest_seq: Some(1),
        next_seq: 2,
        truncated_before_seq: None,
        dropped_on_ingest: 0,
        dropped_on_retention: 0,
        retained_bytes: 32,
        logs: vec![otap_df_admin_api::telemetry::LogEntry {
            seq: 1,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            level: "WARN".to_string(),
            target: "controller".to_string(),
            event_name: "controller.pipeline_runtime_failed".to_string(),
            file: None,
            line: None,
            rendered: "channel is closed and the message could not be sent".to_string(),
            contexts: vec![otap_df_admin_api::telemetry::ResolvedLogContext {
                entity_key: "EntityKey(1)".to_string(),
                schema_name: Some("node.attrs".to_string()),
                attributes: BTreeMap::from([
                    (
                        "pipeline.group.id".to_string(),
                        otap_df_admin_api::telemetry::AttributeValue::String("group-a".to_string()),
                    ),
                    (
                        "pipeline.id".to_string(),
                        otap_df_admin_api::telemetry::AttributeValue::String(
                            "pipeline-a".to_string(),
                        ),
                    ),
                    (
                        "node.id".to_string(),
                        otap_df_admin_api::telemetry::AttributeValue::String(
                            "receiver".to_string(),
                        ),
                    ),
                ]),
            }],
        }],
    };
    let filtered = filter_logs(
        &response,
        &LogFilters {
            level: Some("warn".to_string()),
            target: Some("control".to_string()),
            event: Some("runtime_failed".to_string()),
            pipeline_group_id: Some("group-a".to_string()),
            pipeline_id: Some("pipeline-a".to_string()),
            node_id: Some("receiver".to_string()),
            contains: Some("closed".to_string()),
        },
    );

    assert_eq!(filtered.logs.len(), 1);
}

/// Scenario: compact metrics are filtered by metric-set identity, metric name,
/// and scoped attributes.
/// Guarantees: unrelated metrics are pruned while matching scoped metrics stay
/// available to downstream diagnosis and rendering code.
#[test]
fn metrics_filters_prune_sets_and_metric_names() {
    let response = otap_df_admin_api::telemetry::CompactMetricsResponse {
        timestamp: "2026-01-01T00:00:00Z".to_string(),
        metric_sets: vec![otap_df_admin_api::telemetry::MetricSet {
            name: "engine.pipeline".to_string(),
            attributes: BTreeMap::from([
                (
                    "pipeline.group.id".to_string(),
                    otap_df_admin_api::telemetry::AttributeValue::String("group-a".to_string()),
                ),
                (
                    "pipeline.id".to_string(),
                    otap_df_admin_api::telemetry::AttributeValue::String("pipeline-a".to_string()),
                ),
                (
                    "node.id".to_string(),
                    otap_df_admin_api::telemetry::AttributeValue::String("receiver".to_string()),
                ),
            ]),
            metrics: BTreeMap::from([
                (
                    "pending.sends".to_string(),
                    otap_df_admin_api::telemetry::MetricValue::U64(4),
                ),
                (
                    "processed".to_string(),
                    otap_df_admin_api::telemetry::MetricValue::U64(9),
                ),
            ]),
        }],
    };

    let filtered = filter_metrics_compact(
        &response,
        &MetricsFilters {
            metric_sets: vec!["engine.pipeline".to_string()],
            metric_names: vec!["pending.sends".to_string()],
            pipeline_group_id: Some("group-a".to_string()),
            pipeline_id: Some("pipeline-a".to_string()),
            core_id: None,
            node_id: Some("receiver".to_string()),
        },
    );

    assert_eq!(filtered.metric_sets.len(), 1);
    assert_eq!(filtered.metric_sets[0].metrics.len(), 1);
    assert!(
        filtered.metric_sets[0]
            .metrics
            .contains_key("pending.sends")
    );
}

/// Scenario: group shutdown diagnosis sees known buffered-send and blocked-drop
/// signatures in logs and metrics.
/// Guarantees: the diagnosis report surfaces both findings so operators can
/// distinguish between channel backlog and topic-forward drops.
#[test]
fn diagnosis_surfaces_drain_deadline_and_drop_signatures() {
    let report = diagnose_group_shutdown(
        &otap_df_admin_api::groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::new(),
        },
        &otap_df_admin_api::telemetry::LogsResponse {
            oldest_seq: Some(1),
            newest_seq: Some(2),
            next_seq: 3,
            truncated_before_seq: None,
            dropped_on_ingest: 0,
            dropped_on_retention: 0,
            retained_bytes: 32,
            logs: vec![otap_df_admin_api::telemetry::LogEntry {
                seq: 2,
                timestamp: "2026-01-01T00:00:01Z".to_string(),
                level: "ERROR".to_string(),
                target: "topic_receiver".to_string(),
                event_name: "topic_receiver.drain_ingress_drop_pending_forward".to_string(),
                file: None,
                line: None,
                rendered: "blocked pending forward".to_string(),
                contexts: vec![],
            }],
        },
        &otap_df_admin_api::telemetry::CompactMetricsResponse {
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            metric_sets: vec![otap_df_admin_api::telemetry::MetricSet {
                name: "engine.pipeline".to_string(),
                attributes: BTreeMap::new(),
                metrics: BTreeMap::from([(
                    "pending.sends".to_string(),
                    otap_df_admin_api::telemetry::MetricValue::U64(1),
                )]),
            }],
        },
    );

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "blocked_pending_forward_drop")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "pending_sends_buffered")
    );
}

/// Scenario: a logged observed event is normalized into the shared event model.
/// Guarantees: the structured log record is preserved so downstream renderers
/// and filters can still inspect the original payload.
#[test]
fn normalized_log_event_preserves_record() {
    let event = extract_events_from_pipeline_status(
        "group-a",
        "pipeline-a",
        &otap_df_admin_api::pipelines::Status {
            conditions: vec![],
            total_cores: 1,
            running_cores: 1,
            cores: BTreeMap::from([(
                0,
                serde_json::from_value(json!({
                    "phase": "running",
                    "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                    "conditions": [],
                    "deletePending": false,
                    "recentEvents": [{
                        "Log": {
                            "time": "2026-01-01T00:00:00Z",
                            "record": { "message": "hello" }
                        }
                    }]
                }))
                .expect("core status"),
            )]),
            instances: None,
            active_generation: None,
            serving_generations: None,
            rollout: None,
        },
        None,
    );

    assert_eq!(event[0].kind, NormalizedEventKind::Log);
    assert_eq!(event[0].record, Some(json!({"message": "hello"})));
}
