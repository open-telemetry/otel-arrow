// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Row builders and tone helpers shared by TUI panes.
//!
//! This module normalizes admin API and troubleshooting records into compact
//! table rows with precomputed operational tones. It keeps the status
//! classification rules used by list and table rendering in one place so the
//! same pipeline, event, probe, rollout, and diagnosis states look consistent
//! across the TUI.

use super::*;
use crate::style::terminal_safe;

/// Convert normalized engine events into timeline rows that can be sorted and styled consistently.
pub(super) fn event_rows(events: &[NormalizedEvent]) -> Vec<TimelineRow> {
    events
        .iter()
        .map(|event| TimelineRow {
            time: terminal_safe(&event.time),
            kind: format!("{:?}", event.kind).to_ascii_lowercase(),
            scope: event_scope(event),
            message: event_message(event),
            tone: event_tone(event.kind),
        })
        .collect()
}

/// Convert captured log entries into terminal-safe TUI rows.
pub(super) fn log_rows(entries: &[telemetry::LogEntry]) -> Vec<LogRow> {
    entries
        .iter()
        .map(|entry| LogRow {
            time: terminal_safe(&entry.timestamp),
            level: terminal_safe(&entry.level),
            target: terminal_safe(&entry.target),
            message: terminal_safe(&entry.rendered),
            tone: state_tone(&entry.level),
        })
        .collect()
}

/// Flatten compact metric sets into one display row per metric value.
pub(super) fn metric_rows(metrics: &telemetry::CompactMetricsResponse) -> Vec<MetricRow> {
    let mut rows = Vec::new();
    for metric_set in &metrics.metric_sets {
        let set = metric_set_label(metric_set);
        for (metric, value) in &metric_set.metrics {
            rows.push(MetricRow {
                metric_set: terminal_safe(&set),
                metric: terminal_safe(metric),
                instrument: String::new(),
                unit: String::new(),
                value: metric_value_string(value),
            });
        }
    }
    rows
}

/// Convert readiness/liveness conditions into status rows with precomputed tones.
pub(super) fn condition_rows(conditions: &[pipelines::Condition]) -> Vec<ConditionRow> {
    conditions
        .iter()
        .map(|condition| ConditionRow {
            kind: format!("{:?}", condition.kind).to_ascii_lowercase(),
            status: format!("{:?}", condition.status).to_ascii_lowercase(),
            reason: condition
                .reason
                .as_ref()
                .map(|value| value.as_str().to_ascii_lowercase())
                .unwrap_or_default(),
            message: terminal_safe(condition.message.clone().unwrap_or_default()),
            tone: condition_tone(condition),
        })
        .collect()
}

/// Build core rows from the most detailed runtime instance view available in the status payload.
pub(super) fn core_rows(status: &pipelines::Status) -> Vec<CoreRow> {
    if let Some(instances) = &status.instances {
        instances
            .iter()
            .map(|instance| CoreRow {
                core: instance.core_id.to_string(),
                generation: instance.deployment_generation.to_string(),
                phase: phase_label(&instance.status.phase),
                heartbeat: terminal_safe(&instance.status.last_heartbeat_time),
                delete_pending: bool_label(instance.status.delete_pending),
                tone: phase_tone(&instance.status.phase),
            })
            .collect()
    } else {
        status
            .cores
            .iter()
            .map(|(core_id, core)| CoreRow {
                core: core_id.to_string(),
                generation: status
                    .active_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "active".to_string()),
                phase: phase_label(&core.phase),
                heartbeat: terminal_safe(&core.last_heartbeat_time),
                delete_pending: bool_label(core.delete_pending),
                tone: phase_tone(&core.phase),
            })
            .collect()
    }
}

/// Build the pipeline inventory shown by group and engine views.
pub(super) fn pipeline_inventory_rows(
    pipelines: &BTreeMap<String, pipelines::Status>,
    include_group_prefix: bool,
) -> Vec<PipelineInventoryRow> {
    pipelines
        .iter()
        .map(|(name, status)| {
            let pipeline = if include_group_prefix {
                name.clone()
            } else {
                name.split_once(':')
                    .map_or_else(|| name.clone(), |(_, pipeline)| pipeline.to_string())
            };
            let (_, tone) = classify_pipeline(status);
            PipelineInventoryRow {
                pipeline: terminal_safe(pipeline),
                running: format!("{}/{}", status.running_cores, status.total_cores),
                ready: bool_label(pipeline_is_ready(status)),
                active_generation: status
                    .active_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                rollout: status
                    .rollout
                    .as_ref()
                    .map(|value| format!("{:?}", value.state).to_ascii_lowercase())
                    .unwrap_or_else(|| "none".to_string()),
                tone,
            }
        })
        .collect()
}

/// Convert a diagnosis finding into a compact table row.
pub(super) fn finding_row(finding: &DiagnosisFinding) -> FindingRow {
    FindingRow {
        severity: format!("{:?}", finding.severity).to_ascii_lowercase(),
        code: terminal_safe(&finding.code),
        summary: terminal_safe(&finding.summary),
        tone: match finding.severity {
            FindingSeverity::Info => Tone::Accent,
            FindingSeverity::Warning => Tone::Warning,
            FindingSeverity::Error => Tone::Failure,
        },
    }
}

/// Convert a diagnosis evidence excerpt into a compact table row.
pub(super) fn evidence_row(evidence: &EvidenceExcerpt) -> EvidenceRow {
    EvidenceRow {
        source: terminal_safe(&evidence.source),
        time: terminal_safe(evidence.time.clone().unwrap_or_default()),
        message: terminal_safe(&evidence.message),
    }
}

/// Render the stable scope label used by event timelines.
pub(super) fn event_scope(event: &NormalizedEvent) -> String {
    let mut parts = vec![format!(
        "{}/{} c{}",
        terminal_safe(&event.pipeline_group_id),
        terminal_safe(&event.pipeline_id),
        event.core_id
    )];
    if let Some(node_id) = &event.node_id {
        parts.push(format!("node={}", terminal_safe(node_id)));
    }
    parts.join(" ")
}

/// Compose the event name, message, and detail into the timeline message cell.
pub(super) fn event_message(event: &NormalizedEvent) -> String {
    let mut parts = vec![terminal_safe(&event.name)];
    if let Some(message) = &event.message {
        parts.push(terminal_safe(message));
    }
    if let Some(detail) = &event.detail {
        parts.push(terminal_safe(detail));
    }
    parts.join(" - ")
}

/// Render a metric set name with a small attribute preview for disambiguation.
pub(super) fn metric_set_label(metric_set: &telemetry::MetricSet) -> String {
    let attrs = metric_set
        .attributes
        .iter()
        .take(3)
        .map(|(key, value)| format!("{}={}", terminal_safe(key), attribute_value_string(value)))
        .collect::<Vec<_>>()
        .join(" ");
    if attrs.is_empty() {
        terminal_safe(&metric_set.name)
    } else {
        format!("{} [{}]", terminal_safe(&metric_set.name), attrs)
    }
}

/// Render a telemetry attribute value into a terminal-safe table cell.
pub(super) fn attribute_value_string(value: &telemetry::AttributeValue) -> String {
    match value {
        telemetry::AttributeValue::String(value) => terminal_safe(value),
        telemetry::AttributeValue::Int(value) => value.to_string(),
        telemetry::AttributeValue::UInt(value) => value.to_string(),
        telemetry::AttributeValue::Double(value) => value.to_string(),
        telemetry::AttributeValue::Boolean(value) => value.to_string(),
        telemetry::AttributeValue::Map(value) => {
            terminal_safe(serde_json::to_string(value).unwrap_or_default())
        }
    }
}

/// Render a metric value into the compact TUI representation.
pub(super) fn metric_value_string(value: &telemetry::MetricValue) -> String {
    match value {
        telemetry::MetricValue::U64(value) => value.to_string(),
        telemetry::MetricValue::F64(value) => format!("{value:.3}"),
        telemetry::MetricValue::Mmsc(value) => format!(
            "min={:.3} max={:.3} sum={:.3} count={}",
            value.min, value.max, value.sum, value.count
        ),
    }
}

/// Classify a pipeline into the short operational badge and tone used across TUI lists.
pub(super) fn classify_pipeline(status: &pipelines::Status) -> (&'static str, Tone) {
    if status
        .rollout
        .as_ref()
        .is_some_and(|rollout| !rollout_is_terminal(rollout.state))
    {
        return ("roll", Tone::Accent);
    }
    let has_failure = status.cores.values().any(|core| {
        matches!(
            core.phase,
            pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_)
        )
    }) || status.instances.as_ref().is_some_and(|instances| {
        instances.iter().any(|instance| {
            matches!(
                instance.status.phase,
                pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_)
            )
        })
    });
    if has_failure {
        return ("fail", Tone::Failure);
    }
    if pipeline_is_terminal(status) {
        return ("stop", Tone::Muted);
    }
    if !pipeline_is_ready(status) || status.running_cores < status.total_cores {
        return ("warn", Tone::Warning);
    }
    ("ok", Tone::Success)
}

/// Render a boolean as a human-readable table cell.
pub(super) fn bool_label(value: bool) -> String {
    if value {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

/// Map an in-progress group shutdown pipeline snapshot to the tone used by the watch pane.
pub(super) fn group_shutdown_tone(
    pipeline: &crate::troubleshoot::GroupShutdownWatchPipeline,
) -> Tone {
    if pipeline
        .phases
        .iter()
        .any(|phase| phase.contains("failed") || phase.contains("rejected"))
    {
        Tone::Failure
    } else if pipeline.terminal {
        Tone::Muted
    } else if pipeline.running_cores > 0 || !pipeline.phases.is_empty() {
        Tone::Warning
    } else {
        Tone::Neutral
    }
}

/// Render engine phase variants in the lowercase style used by CLI and TUI tables.
pub(super) fn phase_label(phase: &pipelines::Phase) -> String {
    format!("{phase:?}").to_ascii_lowercase()
}

/// Map pipeline conditions to a severity tone.
pub(super) fn condition_tone(condition: &pipelines::Condition) -> Tone {
    match condition.status {
        pipelines::ConditionStatus::True => Tone::Success,
        pipelines::ConditionStatus::False => Tone::Failure,
        pipelines::ConditionStatus::Unknown => Tone::Warning,
    }
}

/// Map runtime phases to the TUI severity tone.
pub(super) fn phase_tone(phase: &pipelines::Phase) -> Tone {
    match phase {
        pipelines::Phase::Running | pipelines::Phase::Stopped | pipelines::Phase::Deleted => {
            Tone::Success
        }
        pipelines::Phase::Pending
        | pipelines::Phase::Starting
        | pipelines::Phase::Draining
        | pipelines::Phase::Updating
        | pipelines::Phase::RollingBack
        | pipelines::Phase::Deleting(_) => Tone::Warning,
        pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_) => Tone::Failure,
    }
}

/// Map normalized event kinds to the tone used in event tables.
pub(super) fn event_tone(kind: NormalizedEventKind) -> Tone {
    match kind {
        NormalizedEventKind::Request => Tone::Accent,
        NormalizedEventKind::Success => Tone::Success,
        NormalizedEventKind::Error => Tone::Failure,
        NormalizedEventKind::Log => Tone::Muted,
    }
}

/// Map rollout states to the tone used in operation panes.
pub(super) fn rollout_tone(state: pipelines::PipelineRolloutState) -> Tone {
    match state {
        pipelines::PipelineRolloutState::Pending | pipelines::PipelineRolloutState::Running => {
            Tone::Warning
        }
        pipelines::PipelineRolloutState::Succeeded => Tone::Success,
        pipelines::PipelineRolloutState::Failed
        | pipelines::PipelineRolloutState::RollbackFailed => Tone::Failure,
        pipelines::PipelineRolloutState::RollingBack => Tone::Warning,
    }
}

/// Return whether a rollout state is final from the operator perspective.
pub(super) fn rollout_is_terminal(state: pipelines::PipelineRolloutState) -> bool {
    matches!(
        state,
        pipelines::PipelineRolloutState::Succeeded
            | pipelines::PipelineRolloutState::Failed
            | pipelines::PipelineRolloutState::RollbackFailed
    )
}

/// Map diagnosis status to the TUI severity tone.
pub(super) fn diagnosis_tone(status: DiagnosisStatus) -> Tone {
    match status {
        DiagnosisStatus::Healthy => Tone::Success,
        DiagnosisStatus::InProgress | DiagnosisStatus::Blocked => Tone::Warning,
        DiagnosisStatus::Failed => Tone::Failure,
        DiagnosisStatus::Unknown => Tone::Muted,
    }
}

/// Map pipeline probe status to the TUI severity tone.
pub(super) fn probe_tone(status: pipelines::ProbeStatus) -> Tone {
    match status {
        pipelines::ProbeStatus::Ok => Tone::Success,
        pipelines::ProbeStatus::Failed => Tone::Failure,
    }
}

/// Map engine probe status to the TUI severity tone.
pub(super) fn probe_tone_engine(status: engine::ProbeStatus) -> Tone {
    match status {
        engine::ProbeStatus::Ok => Tone::Success,
        engine::ProbeStatus::Failed => Tone::Failure,
    }
}

/// Heuristically classify free-form status text into a display tone.
pub(super) fn state_tone(value: &str) -> Tone {
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("fail") || lowered.contains("error") || lowered.contains("reject") {
        Tone::Failure
    } else if lowered.contains("warn")
        || lowered.contains("pending")
        || lowered.contains("drain")
        || lowered.contains("running")
        || lowered.contains("wait")
        || lowered.contains("rollback")
    {
        Tone::Warning
    } else if lowered.contains("ok")
        || lowered.contains("ready")
        || lowered.contains("success")
        || lowered.contains("succeeded")
        || lowered.contains("completed")
    {
        Tone::Success
    } else {
        Tone::Muted
    }
}

/// Return whether the pipeline Ready condition is currently true.
pub(super) fn pipeline_is_ready(status: &pipelines::Status) -> bool {
    status.conditions.iter().any(|condition| {
        condition.kind == pipelines::ConditionKind::Ready
            && condition.status == pipelines::ConditionStatus::True
    })
}

/// Return whether all known runtime instances have reached a terminal phase.
pub(super) fn pipeline_is_terminal(status: &pipelines::Status) -> bool {
    let phases_terminal = if let Some(instances) = &status.instances {
        !instances.is_empty()
            && instances.iter().all(|instance| {
                matches!(
                    instance.status.phase,
                    pipelines::Phase::Stopped
                        | pipelines::Phase::Deleted
                        | pipelines::Phase::Failed(_)
                        | pipelines::Phase::Rejected(_)
                )
            })
    } else {
        !status.cores.is_empty()
            && status.cores.values().all(|core| {
                matches!(
                    core.phase,
                    pipelines::Phase::Stopped
                        | pipelines::Phase::Deleted
                        | pipelines::Phase::Failed(_)
                        | pipelines::Phase::Rejected(_)
                )
            })
    };
    phases_terminal && status.running_cores == 0
}
