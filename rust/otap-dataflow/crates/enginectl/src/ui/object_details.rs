// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Object-details pane assembly for the Engine, Groups, and Pipelines views.
//!
//! This module turns refreshed engine, group, and pipeline objects into the
//! structured detail panes shown by the TUI. It centralizes the selection of
//! summary cards, status chips, and field/value/detail rows so object detail
//! rendering stays consistent across top-level views.

use super::*;

/// Build the pipeline object detail pane from the current describe, rollout, and shutdown state.
pub(super) fn build_pipeline_details_pane(
    describe: &PipelineDescribeReport,
    header: DetailHeader,
    rollout_status: Option<&pipelines::RolloutStatus>,
    shutdown_status: Option<&pipelines::ShutdownStatus>,
) -> ObjectDetailsPane {
    let (status_badge, status_tone) = classify_pipeline(&describe.status);
    let rollout_state = describe
        .status
        .rollout
        .as_ref()
        .map(|rollout| format!("{:?}", rollout.state).to_ascii_lowercase())
        .unwrap_or_else(|| "none".to_string());
    let rollout_state_tone = describe
        .status
        .rollout
        .as_ref()
        .map(|rollout| rollout_tone(rollout.state))
        .or_else(|| rollout_status.map(|status| rollout_tone(status.state)))
        .unwrap_or(Tone::Muted);
    let rollout_id = describe
        .status
        .rollout
        .as_ref()
        .map(|rollout| rollout.rollout_id.clone())
        .or_else(|| rollout_status.map(|status| status.rollout_id.clone()))
        .unwrap_or_else(|| "none".to_string());
    let shutdown_id = shutdown_status
        .map(|status| status.shutdown_id.clone())
        .unwrap_or_else(|| "none".to_string());

    ObjectDetailsPane {
        header: Some(add_header_chip(
            header,
            chip("object", "pipeline", Tone::Muted),
        )),
        stats: vec![
            card("Status", status_badge.to_string(), status_tone),
            card(
                "Running",
                format!(
                    "{}/{}",
                    describe.status.running_cores, describe.status.total_cores
                ),
                if describe.status.running_cores == describe.status.total_cores {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            card(
                "Ready",
                format!("{:?}", describe.readyz.status).to_ascii_lowercase(),
                probe_tone(describe.readyz.status),
            ),
            card(
                "Events",
                describe.recent_events.len().to_string(),
                Tone::Accent,
            ),
        ],
        rows: vec![
            detail_row(
                "Group",
                describe.details.pipeline_group_id.to_string(),
                "pipeline group id",
                Tone::Muted,
            ),
            detail_row(
                "Pipeline",
                describe.details.pipeline_id.to_string(),
                "pipeline id",
                Tone::Muted,
            ),
            detail_row(
                "Active generation",
                describe
                    .status
                    .active_generation
                    .or(describe.details.active_generation)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                "currently serving deployment generation",
                Tone::Accent,
            ),
            detail_row(
                "Livez",
                format!("{:?}", describe.livez.status).to_ascii_lowercase(),
                "liveness probe result",
                probe_tone(describe.livez.status),
            ),
            detail_row(
                "Readyz",
                format!("{:?}", describe.readyz.status).to_ascii_lowercase(),
                "readiness probe result",
                probe_tone(describe.readyz.status),
            ),
            detail_row(
                "Runtime instances",
                describe
                    .status
                    .instances
                    .as_ref()
                    .map(|instances| instances.len())
                    .unwrap_or(describe.status.cores.len())
                    .to_string(),
                "core-level runtime rows visible in status",
                Tone::Muted,
            ),
            detail_row(
                "Conditions",
                describe.status.conditions.len().to_string(),
                "pipeline conditions reported by the engine",
                Tone::Muted,
            ),
            detail_row(
                "Rollout state",
                rollout_state,
                format!("rollout id: {rollout_id}"),
                rollout_state_tone,
            ),
            detail_row(
                "Shutdown",
                shutdown_status
                    .map(|status| status.state.clone())
                    .unwrap_or_else(|| "none".to_string()),
                format!("shutdown id: {shutdown_id}"),
                shutdown_status
                    .map(|status| state_tone(&status.state))
                    .unwrap_or(Tone::Muted),
            ),
        ],
        empty_message: "No pipeline details are available.".to_string(),
    }
}

/// Build the group object detail pane from the current group summary snapshot.
pub(super) fn build_group_details_pane(
    group_id: &str,
    describe: &GroupsDescribeReport,
    header: DetailHeader,
) -> ObjectDetailsPane {
    let pipelines = pipeline_inventory_rows(&describe.status.pipelines, false);
    let attention = pipelines
        .iter()
        .filter(|row| matches!(row.tone, Tone::Warning | Tone::Failure | Tone::Accent))
        .count();

    ObjectDetailsPane {
        header: Some(add_header_chip(
            header,
            chip("object", "group", Tone::Muted),
        )),
        stats: vec![
            card(
                "Pipelines",
                describe.summary.total_pipelines.to_string(),
                Tone::Accent,
            ),
            card(
                "Running",
                describe.summary.running_pipelines.to_string(),
                Tone::Success,
            ),
            card(
                "Ready",
                describe.summary.ready_pipelines.to_string(),
                if describe.summary.ready_pipelines == describe.summary.total_pipelines {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            card(
                "Attention",
                attention.to_string(),
                if attention == 0 {
                    Tone::Muted
                } else {
                    Tone::Warning
                },
            ),
        ],
        rows: vec![
            detail_row("Group", group_id, "pipeline group id", Tone::Muted),
            detail_row(
                "Generated at",
                describe.status.generated_at.clone(),
                "source status snapshot time",
                Tone::Muted,
            ),
            detail_row(
                "Terminal pipelines",
                describe.summary.terminal_pipelines.to_string(),
                "pipelines with all runtime instances terminal",
                Tone::Muted,
            ),
            detail_row(
                "Non-ready",
                describe.summary.non_ready_pipelines.len().to_string(),
                describe.summary.non_ready_pipelines.join(", "),
                if describe.summary.non_ready_pipelines.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Warning
                },
            ),
            detail_row(
                "Non-terminal",
                describe.summary.non_terminal_pipelines.len().to_string(),
                describe.summary.non_terminal_pipelines.join(", "),
                if describe.summary.non_terminal_pipelines.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Warning
                },
            ),
            detail_row(
                "Recent events",
                describe
                    .recent_events
                    .iter()
                    .filter(|event| event.pipeline_group_id == group_id)
                    .count()
                    .to_string(),
                "events retained for this group in the current snapshot",
                Tone::Accent,
            ),
        ],
        empty_message: "No group details are available.".to_string(),
    }
}

/// Build the engine object detail pane, including probe and process-vital summaries.
pub(super) fn build_engine_details_pane(
    status: &engine::Status,
    livez: &engine::ProbeResponse,
    readyz: &engine::ProbeResponse,
    vitals: &EngineVitals,
    header: DetailHeader,
) -> ObjectDetailsPane {
    let ready_pipelines = status
        .pipelines
        .values()
        .filter(|pipeline| pipeline_is_ready(pipeline))
        .count();
    let terminal_pipelines = status
        .pipelines
        .values()
        .filter(|pipeline| pipeline_is_terminal(pipeline))
        .count();
    let running_pipelines = status
        .pipelines
        .values()
        .filter(|pipeline| pipeline.running_cores > 0)
        .count();

    ObjectDetailsPane {
        header: Some(add_header_chip(
            header,
            chip("object", "engine", Tone::Muted),
        )),
        stats: vec![
            card(
                "Pipelines",
                status.pipelines.len().to_string(),
                Tone::Accent,
            ),
            card("Running", running_pipelines.to_string(), Tone::Success),
            card(
                "Ready",
                ready_pipelines.to_string(),
                probe_tone_engine(readyz.status),
            ),
            card(
                "Failing",
                readyz.failing.len().to_string(),
                if readyz.failing.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Failure
                },
            ),
        ],
        rows: vec![
            detail_row(
                "Generated at",
                status.generated_at.clone(),
                "engine status snapshot time",
                Tone::Muted,
            ),
            detail_row(
                "Livez",
                format!("{:?}", livez.status).to_ascii_lowercase(),
                "engine liveness probe",
                probe_tone_engine(livez.status),
            ),
            detail_row(
                "Readyz",
                format!("{:?}", readyz.status).to_ascii_lowercase(),
                "engine readiness probe",
                probe_tone_engine(readyz.status),
            ),
            detail_row(
                "Terminal pipelines",
                terminal_pipelines.to_string(),
                "pipelines with no running cores",
                Tone::Muted,
            ),
            detail_row(
                "CPU",
                vitals.cpu_utilization.clone(),
                if vitals.stale {
                    "stale metrics snapshot".to_string()
                } else {
                    "latest compact engine metric".to_string()
                },
                vitals.cpu_tone,
            ),
            detail_row(
                "RSS",
                vitals.memory_rss.clone(),
                vitals
                    .pressure_detail
                    .clone()
                    .unwrap_or_else(|| "resident memory from engine metrics".to_string()),
                vitals.memory_tone,
            ),
            detail_row(
                "Memory pressure",
                vitals.pressure_state.clone(),
                vitals
                    .pressure_detail
                    .clone()
                    .unwrap_or_else(|| "no pressure detail available".to_string()),
                vitals.pressure_tone,
            ),
        ],
        empty_message: "No engine details are available.".to_string(),
    }
}

fn detail_row(
    field: impl Into<String>,
    value: impl Into<String>,
    detail: impl Into<String>,
    tone: Tone,
) -> ObjectDetailRow {
    ObjectDetailRow {
        field: field.into(),
        value: value.into(),
        detail: detail.into(),
        tone,
    }
}
