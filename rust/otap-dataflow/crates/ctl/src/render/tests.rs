// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::human::{
    render_engine_status, render_group_shutdown_watch, render_metrics_compact,
    render_pipeline_status,
};
use super::table::{TableColumn, render_table, visible_width};
use crate::args::ColorChoice;
use crate::style::HumanStyle;
use crate::troubleshoot::{GroupShutdownWatchPipeline, GroupShutdownWatchSnapshot};
use otap_df_admin_api::{engine, pipelines, telemetry};
use serde_json::json;

fn no_color_style() -> HumanStyle {
    HumanStyle::resolve(ColorChoice::Never, false)
}

/// Scenario: the human table renderer is asked to align colored cells.
/// Guarantees: visible-width calculation ignores ANSI escape sequences so every
/// rendered row keeps the same visual width.
#[test]
fn table_alignment_ignores_ansi_sequences() {
    let style = HumanStyle::resolve(ColorChoice::Always, true);
    let rendered = render_table(
        &style,
        &[TableColumn::left("name"), TableColumn::left("state")],
        vec![
            vec!["alpha".to_string(), style.state("succeeded")],
            vec!["beta-long".to_string(), style.state("failed")],
        ],
    );

    let widths = rendered.lines().map(visible_width).collect::<Vec<_>>();
    assert!(widths.windows(2).all(|pair| pair[0] == pair[1]));
}

/// Scenario: engine status is rendered for humans with at least one pipeline in
/// the snapshot.
/// Guarantees: the renderer uses the tabular pipeline summary layout rather
/// than the older repeated key-value line format.
#[test]
fn engine_status_renders_pipeline_table() {
    let status: engine::Status = serde_json::from_value(json!({
        "generatedAt": "2026-01-01T00:00:00Z",
        "pipelines": {
            "tenant-a:ingest": {
                "conditions": [],
                "totalCores": 2,
                "runningCores": 1,
                "cores": {},
                "activeGeneration": 7,
                "rollout": {
                    "rolloutId": "rollout-1",
                    "state": "running",
                    "targetGeneration": 8,
                    "startedAt": "2026-01-01T00:00:00Z",
                    "updatedAt": "2026-01-01T00:00:00Z"
                }
            }
        }
    }))
    .expect("status");

    let rendered = render_engine_status(&no_color_style(), &status);
    assert!(rendered.contains("pipeline"));
    assert!(rendered.contains("active_generation"));
    assert!(rendered.contains("tenant-a:ingest"));
    assert!(rendered.contains("1/2"));
    assert!(!rendered.contains("pipeline: tenant-a:ingest"));
}

/// Scenario: pipeline status is rendered for humans with one running core.
/// Guarantees: the renderer emits the core table layout instead of repeating a
/// legacy per-core key-value line representation.
#[test]
fn pipeline_status_renders_core_table() {
    let status: pipelines::Status = serde_json::from_value(json!({
        "conditions": [],
        "totalCores": 1,
        "runningCores": 1,
        "cores": {
            "0": {
                "phase": "running",
                "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                "conditions": [],
                "deletePending": false
            }
        },
        "activeGeneration": 3
    }))
    .expect("status");

    let rendered = render_pipeline_status(&no_color_style(), &status);
    assert!(rendered.contains("core"));
    assert!(rendered.contains("last_heartbeat_time"));
    assert!(rendered.contains("Running"));
    assert!(!rendered.contains("core: 0"));
}

/// Scenario: the groups shutdown watch heuristic renders a mixed-terminal
/// pipeline snapshot.
/// Guarantees: the output uses the structured table layout and carries the
/// pipeline phase summary needed for operators to inspect progress.
#[test]
fn group_shutdown_watch_renders_pipeline_table() {
    let snapshot = GroupShutdownWatchSnapshot {
        started_at: "2026-01-01T00:00:00Z".to_string(),
        generated_at: "2026-01-01T00:00:05Z".to_string(),
        request_status: "accepted".to_string(),
        elapsed_ms: 5000,
        total_pipelines: 2,
        terminal_pipelines: 1,
        all_terminal: false,
        pipelines: vec![
            GroupShutdownWatchPipeline {
                pipeline: "tenant-a:ingest".to_string(),
                running_cores: 0,
                total_cores: 1,
                terminal: true,
                phases: vec!["stopped".to_string()],
            },
            GroupShutdownWatchPipeline {
                pipeline: "tenant-a:egress".to_string(),
                running_cores: 0,
                total_cores: 1,
                terminal: false,
                phases: vec!["draining".to_string()],
            },
        ],
    };

    let rendered = render_group_shutdown_watch(&no_color_style(), &snapshot);
    assert!(rendered.contains("terminal"));
    assert!(rendered.contains("tenant-a:egress"));
    assert!(rendered.contains("draining"));
    assert!(!rendered.contains("pipeline: tenant-a:egress"));
}

/// Scenario: compact metrics are rendered for a human reader.
/// Guarantees: the renderer includes the metric-set header, scoped attributes,
/// and the metric/value table.
#[test]
fn metrics_compact_renders_metric_table() {
    let response: telemetry::CompactMetricsResponse = serde_json::from_value(json!({
        "timestamp": "2026-01-01T00:00:00Z",
        "metric_sets": [
            {
                "name": "engine.runtime",
                "attributes": {
                    "pipeline.id": { "String": "ingest" }
                },
                "metrics": {
                    "pipelines": 3,
                    "ready": 1
                }
            }
        ]
    }))
    .expect("metrics");

    let rendered = render_metrics_compact(&no_color_style(), &response);
    assert!(rendered.contains("metric_set: engine.runtime"));
    assert!(rendered.contains("attributes: pipeline.id=ingest"));
    assert!(rendered.contains("metric"));
    assert!(rendered.contains("value"));
    assert!(rendered.contains("pipelines"));
}
