// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Derived describe reports and normalized event extraction for troubleshooting flows.
//!
//! This module is the read-model layer between raw admin API status payloads
//! and operator-facing reports. It computes group and pipeline summaries,
//! flattens per-core lifecycle events into one sortable event stream, and
//! builds progress snapshots used by watch commands without changing the
//! underlying SDK response types.

use super::models::{
    GroupShutdownWatchPipeline, GroupShutdownWatchSnapshot, GroupsDescribeReport, GroupsSummary,
    NormalizedEvent, NormalizedEventKind, PipelineDescribeReport,
};
use otap_df_admin_api::{groups, pipelines};
use serde_json::Value;
use std::time::SystemTime;

/// Builds the derived `groups describe` report from raw group status.
pub fn describe_groups(status: groups::Status) -> GroupsDescribeReport {
    let recent_events = extract_events_from_group_status(&status, None);
    let mut running_pipelines = 0;
    let mut ready_pipelines = 0;
    let mut terminal_pipelines = 0;
    let mut non_ready_pipelines = Vec::new();
    let mut non_terminal_pipelines = Vec::new();

    for (name, pipeline) in &status.pipelines {
        if pipeline.running_cores > 0 {
            running_pipelines += 1;
        }
        if pipeline_is_ready(pipeline) {
            ready_pipelines += 1;
        } else {
            non_ready_pipelines.push(name.clone());
        }
        if pipeline_is_terminal(pipeline) {
            terminal_pipelines += 1;
        } else {
            non_terminal_pipelines.push(name.clone());
        }
    }

    GroupsDescribeReport {
        summary: GroupsSummary {
            total_pipelines: status.pipelines.len(),
            running_pipelines,
            ready_pipelines,
            terminal_pipelines,
            non_ready_pipelines,
            non_terminal_pipelines,
        },
        status,
        recent_events,
    }
}

/// Builds the derived `pipelines describe` report from raw pipeline details and probes.
pub fn describe_pipeline(
    details: pipelines::PipelineDetails,
    status: pipelines::Status,
    livez: pipelines::ProbeResult,
    readyz: pipelines::ProbeResult,
) -> PipelineDescribeReport {
    let recent_events = extract_events_from_pipeline_status(
        &details.pipeline_group_id,
        &details.pipeline_id,
        &status,
        None,
    );
    PipelineDescribeReport {
        details,
        status,
        livez,
        readyz,
        recent_events,
    }
}

/// Extracts normalized recent events from every pipeline in group status.
pub fn extract_events_from_group_status(
    status: &groups::Status,
    filters: Option<&super::models::EventFilters>,
) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();
    for (name, pipeline) in &status.pipelines {
        let (pipeline_group_id, pipeline_id) = name.split_once(':').unwrap_or((name.as_str(), ""));
        events.extend(extract_events_from_pipeline_status(
            pipeline_group_id,
            pipeline_id,
            pipeline,
            filters,
        ));
    }
    sort_events(&mut events);
    events
}

/// Extracts normalized recent events from one pipeline status response.
pub fn extract_events_from_pipeline_status(
    pipeline_group_id: &str,
    pipeline_id: &str,
    status: &pipelines::Status,
    filters: Option<&super::models::EventFilters>,
) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();
    for (core_id, core) in &status.cores {
        let Some(recent_events) = &core.recent_events else {
            continue;
        };
        for event in recent_events {
            let normalized = normalize_event(pipeline_group_id, pipeline_id, *core_id, event);
            if filters.is_none_or(|value| super::filter::event_matches(&normalized, value)) {
                events.push(normalized);
            }
        }
    }
    sort_events(&mut events);
    events
}

/// Keeps only the newest `tail` events when a tail limit is configured.
pub fn tail_events(events: Vec<NormalizedEvent>, tail: Option<usize>) -> Vec<NormalizedEvent> {
    let Some(tail) = tail else {
        return events;
    };
    let keep_from = events.len().saturating_sub(tail);
    events.into_iter().skip(keep_from).collect()
}

/// Builds one group shutdown progress snapshot for watch output.
pub fn group_shutdown_snapshot(
    request_status: groups::ShutdownStatus,
    status: &groups::Status,
    started_at: SystemTime,
) -> GroupShutdownWatchSnapshot {
    let pipelines = status
        .pipelines
        .iter()
        .map(|(pipeline, value)| GroupShutdownWatchPipeline {
            pipeline: pipeline.clone(),
            running_cores: value.running_cores,
            total_cores: value.total_cores,
            terminal: pipeline_is_terminal(value),
            phases: pipeline_phases(value),
        })
        .collect::<Vec<_>>();
    let terminal_pipelines = pipelines.iter().filter(|value| value.terminal).count();

    GroupShutdownWatchSnapshot {
        started_at: humantime::format_rfc3339_seconds(started_at).to_string(),
        generated_at: status.generated_at.clone(),
        request_status: format!("{request_status:?}").to_ascii_lowercase(),
        elapsed_ms: started_at
            .elapsed()
            .map_or(0, |value| value.as_millis() as u64),
        total_pipelines: pipelines.len(),
        terminal_pipelines,
        all_terminal: terminal_pipelines == pipelines.len(),
        pipelines,
    }
}

/// Returns true when all selected pipeline cores have reached a terminal phase.
pub fn pipeline_is_terminal(status: &pipelines::Status) -> bool {
    let phases_terminal = if let Some(instances) = &status.instances {
        !instances.is_empty()
            && instances
                .iter()
                .all(|instance| phase_is_terminal(&instance.status.phase))
    } else {
        !status.cores.is_empty()
            && status
                .cores
                .values()
                .all(|core| phase_is_terminal(&core.phase))
    };
    phases_terminal && status.running_cores == 0
}

/// Returns true when the pipeline has an aggregate ready condition set to true.
pub fn pipeline_is_ready(status: &pipelines::Status) -> bool {
    status.conditions.iter().any(|condition| {
        condition.kind == pipelines::ConditionKind::Ready
            && condition.status == pipelines::ConditionStatus::True
    })
}

/// Returns true when a core phase is terminal for shutdown/progress purposes.
pub fn phase_is_terminal(phase: &pipelines::Phase) -> bool {
    matches!(
        phase,
        pipelines::Phase::Stopped
            | pipelines::Phase::Deleted
            | pipelines::Phase::Failed(_)
            | pipelines::Phase::Rejected(_)
    )
}

fn normalize_event(
    fallback_pipeline_group_id: &str,
    fallback_pipeline_id: &str,
    fallback_core_id: usize,
    event: &pipelines::ObservedEvent,
) -> NormalizedEvent {
    match event {
        pipelines::ObservedEvent::Engine(event) => {
            let (kind, name, detail) = match &event.r#type {
                pipelines::EventType::Request(kind) => {
                    (NormalizedEventKind::Request, format!("{kind:?}"), None)
                }
                pipelines::EventType::Success(kind) => {
                    (NormalizedEventKind::Success, format!("{kind:?}"), None)
                }
                pipelines::EventType::Error(kind) => normalize_error_event(kind),
            };
            NormalizedEvent {
                time: event.time.clone(),
                kind,
                name,
                pipeline_group_id: event.key.pipeline_group_id.clone(),
                pipeline_id: event.key.pipeline_id.clone(),
                core_id: event.key.core_id,
                deployment_generation: event.key.deployment_generation,
                node_id: event.node_id.clone(),
                node_kind: event
                    .node_kind
                    .map(|value| format!("{value:?}").to_ascii_lowercase()),
                message: event.message.clone(),
                detail,
                record: None,
            }
        }
        pipelines::ObservedEvent::Log(event) => NormalizedEvent {
            time: event.time.clone(),
            kind: NormalizedEventKind::Log,
            name: "log".to_string(),
            pipeline_group_id: fallback_pipeline_group_id.to_string(),
            pipeline_id: fallback_pipeline_id.to_string(),
            core_id: fallback_core_id,
            deployment_generation: None,
            node_id: None,
            node_kind: None,
            message: Some(log_record_summary(&event.record)),
            detail: None,
            record: Some(event.record.clone()),
        },
    }
}

fn normalize_error_event(
    event: &pipelines::ErrorEvent,
) -> (NormalizedEventKind, String, Option<String>) {
    match event {
        pipelines::ErrorEvent::AdmissionError(summary) => (
            NormalizedEventKind::Error,
            "AdmissionError".to_string(),
            Some(error_summary(summary)),
        ),
        pipelines::ErrorEvent::ConfigRejected(summary) => (
            NormalizedEventKind::Error,
            "ConfigRejected".to_string(),
            Some(error_summary(summary)),
        ),
        pipelines::ErrorEvent::UpdateFailed(summary) => (
            NormalizedEventKind::Error,
            "UpdateFailed".to_string(),
            Some(error_summary(summary)),
        ),
        pipelines::ErrorEvent::RollbackFailed(summary) => (
            NormalizedEventKind::Error,
            "RollbackFailed".to_string(),
            Some(error_summary(summary)),
        ),
        pipelines::ErrorEvent::DrainError(summary) => (
            NormalizedEventKind::Error,
            "DrainError".to_string(),
            Some(error_summary(summary)),
        ),
        pipelines::ErrorEvent::DrainDeadlineReached => (
            NormalizedEventKind::Error,
            "DrainDeadlineReached".to_string(),
            None,
        ),
        pipelines::ErrorEvent::RuntimeError(summary) => (
            NormalizedEventKind::Error,
            "RuntimeError".to_string(),
            Some(error_summary(summary)),
        ),
        pipelines::ErrorEvent::DeleteError(summary) => (
            NormalizedEventKind::Error,
            "DeleteError".to_string(),
            Some(error_summary(summary)),
        ),
    }
}

fn error_summary(summary: &pipelines::ErrorSummary) -> String {
    match summary {
        pipelines::ErrorSummary::Pipeline {
            error_kind,
            message,
            source,
        } => source.as_ref().map_or_else(
            || format!("{error_kind}: {message}"),
            |source| format!("{error_kind}: {message} ({source})"),
        ),
        pipelines::ErrorSummary::Node {
            node,
            node_kind,
            error_kind,
            message,
            source,
        } => source.as_ref().map_or_else(
            || format!("{node_kind:?}:{node} {error_kind}: {message}"),
            |source| format!("{node_kind:?}:{node} {error_kind}: {message} ({source})"),
        ),
    }
}

/// Returns the distinct lower-case phases currently represented by a pipeline status.
pub(super) fn pipeline_phases(status: &pipelines::Status) -> Vec<String> {
    if let Some(instances) = &status.instances {
        instances
            .iter()
            .map(|instance| format!("{:?}", instance.status.phase).to_ascii_lowercase())
            .collect()
    } else {
        status
            .cores
            .values()
            .map(|core| format!("{:?}", core.phase).to_ascii_lowercase())
            .collect()
    }
}

fn sort_events(events: &mut [NormalizedEvent]) {
    events.sort_by(|left, right| {
        (
            &left.time,
            &left.pipeline_group_id,
            &left.pipeline_id,
            left.core_id,
            &left.name,
            left.node_id.as_deref(),
        )
            .cmp(&(
                &right.time,
                &right.pipeline_group_id,
                &right.pipeline_id,
                right.core_id,
                &right.name,
                right.node_id.as_deref(),
            ))
    });
}

fn log_record_summary(record: &Value) -> String {
    serde_json::to_string(record).unwrap_or_else(|_| record.to_string())
}
