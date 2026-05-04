// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Diagnosis heuristics for rollout, shutdown, and readiness troubleshooting flows.
//!
//! This module consumes the normalized evidence prepared by `describe` and
//! `filter` and turns it into findings, evidence excerpts, and recommended
//! next steps. The heuristics stay client-side so `dfctl diagnose` and the TUI
//! can explain likely failure modes even when the admin API only exposes raw
//! status, retained logs, metrics, and operation snapshots.

use super::describe::pipeline_is_terminal;
use super::models::{
    DiagnosisFinding, DiagnosisReport, DiagnosisStatus, EvidenceExcerpt, FindingSeverity,
    NormalizedEvent, PipelineDescribeReport,
};
use otap_df_admin_api::{groups, pipelines, telemetry};
use std::collections::BTreeSet;

/// Produces diagnosis findings for a coordinated group shutdown.
pub fn diagnose_group_shutdown(
    status: &groups::Status,
    logs: &telemetry::LogsResponse,
    metrics: &telemetry::CompactMetricsResponse,
) -> DiagnosisReport {
    let events = super::describe::extract_events_from_group_status(status, None);
    let all_terminal = status.pipelines.values().all(pipeline_is_terminal);
    let findings = build_findings(
        "group_shutdown",
        &events,
        &logs.logs,
        Some(metrics),
        None,
        None,
        None,
        all_terminal,
    );
    finalize_diagnosis("group_shutdown", findings, all_terminal, false, false)
}

/// Produces diagnosis findings for a pipeline rollout.
pub fn diagnose_pipeline_rollout(
    describe: &PipelineDescribeReport,
    rollout_status: Option<&pipelines::RolloutStatus>,
    logs: &telemetry::LogsResponse,
    metrics: &telemetry::CompactMetricsResponse,
) -> DiagnosisReport {
    let operation_failed = rollout_status.is_some_and(|value| {
        matches!(
            value.state,
            pipelines::PipelineRolloutState::Failed
                | pipelines::PipelineRolloutState::RollbackFailed
        )
    });
    let operation_in_progress = rollout_status.is_some_and(|value| {
        matches!(
            value.state,
            pipelines::PipelineRolloutState::Pending
                | pipelines::PipelineRolloutState::Running
                | pipelines::PipelineRolloutState::RollingBack
        )
    });
    let findings = build_findings(
        "pipeline_rollout",
        &describe.recent_events,
        &logs.logs,
        Some(metrics),
        rollout_status,
        None,
        Some(&describe.readyz),
        pipeline_is_terminal(&describe.status),
    );
    finalize_diagnosis(
        "pipeline_rollout",
        findings,
        pipeline_is_terminal(&describe.status),
        operation_in_progress,
        operation_failed || !probe_is_ok(&describe.readyz),
    )
}

/// Produces diagnosis findings for a pipeline shutdown.
pub fn diagnose_pipeline_shutdown(
    describe: &PipelineDescribeReport,
    shutdown_status: Option<&pipelines::ShutdownStatus>,
    logs: &telemetry::LogsResponse,
    metrics: &telemetry::CompactMetricsResponse,
) -> DiagnosisReport {
    let operation_failed = shutdown_status.is_some_and(|value| value.state == "failed");
    let operation_in_progress =
        shutdown_status.is_some_and(|value| !shutdown_is_terminal(&value.state));
    let all_terminal = pipeline_is_terminal(&describe.status);
    let findings = build_findings(
        "pipeline_shutdown",
        &describe.recent_events,
        &logs.logs,
        Some(metrics),
        None,
        shutdown_status,
        Some(&describe.readyz),
        all_terminal,
    );
    finalize_diagnosis(
        "pipeline_shutdown",
        findings,
        all_terminal,
        operation_in_progress,
        operation_failed,
    )
}

fn build_findings(
    scope: &str,
    events: &[NormalizedEvent],
    logs: &[telemetry::LogEntry],
    metrics: Option<&telemetry::CompactMetricsResponse>,
    rollout_status: Option<&pipelines::RolloutStatus>,
    shutdown_status: Option<&pipelines::ShutdownStatus>,
    readyz: Option<&pipelines::ProbeResult>,
    terminal: bool,
) -> Vec<DiagnosisFinding> {
    let mut findings = Vec::new();

    if let Some(finding) = operation_terminal_failure(scope, rollout_status, shutdown_status) {
        findings.push(finding);
    }
    if let Some(finding) = drain_deadline_reached(events, logs) {
        findings.push(finding);
    }
    if let Some(finding) = runtime_error_while_draining(events, logs) {
        findings.push(finding);
    }
    if let Some(finding) = blocked_pending_forward_drop(logs) {
        findings.push(finding);
    }
    if let Some(finding) = pending_receiver_drain(events, logs, terminal) {
        findings.push(finding);
    }
    if let Some(finding) = pending_sends_buffered(metrics, logs) {
        findings.push(finding);
    }
    if let Some(probe) = readyz
        && !probe_is_ok(probe)
    {
        findings.push(DiagnosisFinding {
            code: "pipeline_not_ready".to_string(),
            severity: FindingSeverity::Warning,
            summary: probe.message.clone().map_or_else(
                || "pipeline ready probe is failing".to_string(),
                |message| format!("pipeline ready probe is failing: {message}"),
            ),
            evidence: vec![EvidenceExcerpt {
                source: "readyz".to_string(),
                time: None,
                message: probe
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("status={:?}", probe.status)),
            }],
        });
    }

    if findings.is_empty() && !terminal {
        findings.push(DiagnosisFinding {
            code: "no_matching_evidence".to_string(),
            severity: FindingSeverity::Info,
            summary: format!(
                "{scope} is not yet terminal, but current events/logs do not match a known failure signature"
            ),
            evidence: vec![],
        });
    }

    dedupe_findings(findings)
}

fn operation_terminal_failure(
    scope: &str,
    rollout_status: Option<&pipelines::RolloutStatus>,
    shutdown_status: Option<&pipelines::ShutdownStatus>,
) -> Option<DiagnosisFinding> {
    if let Some(status) = rollout_status
        && matches!(
            status.state,
            pipelines::PipelineRolloutState::Failed
                | pipelines::PipelineRolloutState::RollbackFailed
        )
    {
        return Some(DiagnosisFinding {
            code: "operation_terminal_failure".to_string(),
            severity: FindingSeverity::Error,
            summary: status.failure_reason.clone().map_or_else(
                || {
                    format!(
                        "rollout '{}' ended in state {:?}",
                        status.rollout_id, status.state
                    )
                },
                |reason| {
                    format!(
                        "rollout '{}' ended in state {:?}: {reason}",
                        status.rollout_id, status.state
                    )
                },
            ),
            evidence: vec![EvidenceExcerpt {
                source: scope.to_string(),
                time: Some(status.updated_at.clone()),
                message: format!(
                    "rollout_id={} state={:?} failure_reason={}",
                    status.rollout_id,
                    status.state,
                    status.failure_reason.as_deref().unwrap_or("none")
                ),
            }],
        });
    }
    if let Some(status) = shutdown_status
        && shutdown_is_terminal(&status.state)
        && status.state != "succeeded"
    {
        return Some(DiagnosisFinding {
            code: "operation_terminal_failure".to_string(),
            severity: FindingSeverity::Error,
            summary: status.failure_reason.clone().map_or_else(
                || {
                    format!(
                        "shutdown '{}' ended in state {}",
                        status.shutdown_id, status.state
                    )
                },
                |reason| {
                    format!(
                        "shutdown '{}' ended in state {}: {reason}",
                        status.shutdown_id, status.state
                    )
                },
            ),
            evidence: vec![EvidenceExcerpt {
                source: scope.to_string(),
                time: Some(status.updated_at.clone()),
                message: format!(
                    "shutdown_id={} state={} failure_reason={}",
                    status.shutdown_id,
                    status.state,
                    status.failure_reason.as_deref().unwrap_or("none")
                ),
            }],
        });
    }
    None
}

fn drain_deadline_reached(
    events: &[NormalizedEvent],
    logs: &[telemetry::LogEntry],
) -> Option<DiagnosisFinding> {
    let mut evidence = event_evidence(
        events
            .iter()
            .filter(|event| event.name == "DrainDeadlineReached"),
        "recent_event",
    );
    evidence.extend(log_evidence(
        logs.iter()
            .filter(|entry| contains_ci(&entry.rendered, "DrainDeadlineReached")),
        "retained_log",
    ));
    if evidence.is_empty() {
        return None;
    }
    Some(DiagnosisFinding {
        code: "drain_deadline_reached".to_string(),
        severity: FindingSeverity::Error,
        summary: "a drain deadline was reached before the operation could complete".to_string(),
        evidence,
    })
}

fn runtime_error_while_draining(
    events: &[NormalizedEvent],
    logs: &[telemetry::LogEntry],
) -> Option<DiagnosisFinding> {
    let mut evidence = event_evidence(
        events.iter().filter(|event| {
            event.name == "RuntimeError"
                && event
                    .detail
                    .as_ref()
                    .or(event.message.as_ref())
                    .is_some_and(|value| contains_ci(value, "drain"))
        }),
        "recent_event",
    );
    evidence.extend(log_evidence(
        logs.iter().filter(|entry| {
            contains_ci(&entry.rendered, "runtime error")
                && (contains_ci(&entry.rendered, "drain")
                    || contains_ci(&entry.event_name, "pipeline_runtime_failed"))
        }),
        "retained_log",
    ));
    if evidence.is_empty() {
        return None;
    }
    Some(DiagnosisFinding {
        code: "runtime_error_while_draining".to_string(),
        severity: FindingSeverity::Error,
        summary: "a runtime error occurred while the pipeline was draining".to_string(),
        evidence,
    })
}

fn blocked_pending_forward_drop(logs: &[telemetry::LogEntry]) -> Option<DiagnosisFinding> {
    let evidence = log_evidence(
        logs.iter().filter(|entry| {
            contains_ci(&entry.event_name, "drop_pending_forward")
                || contains_ci(&entry.rendered, "blocked pending forward")
        }),
        "retained_log",
    );
    if evidence.is_empty() {
        return None;
    }
    Some(DiagnosisFinding {
        code: "blocked_pending_forward_drop".to_string(),
        severity: FindingSeverity::Error,
        summary: "a topic receiver dropped a blocked pending forward during shutdown".to_string(),
        evidence,
    })
}

fn pending_receiver_drain(
    events: &[NormalizedEvent],
    logs: &[telemetry::LogEntry],
    terminal: bool,
) -> Option<DiagnosisFinding> {
    if terminal {
        return None;
    }

    let mut evidence = event_evidence(
        events.iter().filter(|event| {
            event.name == "IngressDrainStarted" || event.name == "ReceiversDrained"
        }),
        "recent_event",
    );
    evidence.extend(log_evidence(
        logs.iter().filter(|entry| {
            contains_ci(&entry.event_name, "drain_ingress")
                || (contains_ci(&entry.rendered, "receiver")
                    && contains_ci(&entry.rendered, "drain"))
        }),
        "retained_log",
    ));
    if evidence.is_empty() {
        return None;
    }
    Some(DiagnosisFinding {
        code: "pending_receiver_drain".to_string(),
        severity: FindingSeverity::Warning,
        summary: "receivers appear to still be draining ingress".to_string(),
        evidence,
    })
}

fn pending_sends_buffered(
    metrics: Option<&telemetry::CompactMetricsResponse>,
    logs: &[telemetry::LogEntry],
) -> Option<DiagnosisFinding> {
    let mut evidence = Vec::new();
    if let Some(metrics) = metrics {
        for metric_set in &metrics.metric_sets {
            for (name, value) in &metric_set.metrics {
                if (contains_ci(name, "pending") || contains_ci(name, "buffer"))
                    && metric_value_positive(value)
                {
                    evidence.push(EvidenceExcerpt {
                        source: "metrics".to_string(),
                        time: Some(metrics.timestamp.clone()),
                        message: format!(
                            "metric_set={} metric={} value={}",
                            metric_set.name,
                            name,
                            metric_value_string(value)
                        ),
                    });
                    if evidence.len() == 3 {
                        break;
                    }
                }
            }
            if evidence.len() == 3 {
                break;
            }
        }
    }
    evidence.extend(log_evidence(
        logs.iter().filter(|entry| {
            contains_ci(&entry.rendered, "pending send")
                || contains_ci(&entry.rendered, "buffered")
                || contains_ci(
                    &entry.rendered,
                    "channel is closed and the message could not be sent",
                )
        }),
        "retained_log",
    ));
    if evidence.is_empty() {
        return None;
    }
    Some(DiagnosisFinding {
        code: "pending_sends_buffered".to_string(),
        severity: FindingSeverity::Warning,
        summary: "buffered or pending sends may still be delaying shutdown".to_string(),
        evidence,
    })
}

fn finalize_diagnosis(
    scope: &str,
    findings: Vec<DiagnosisFinding>,
    terminal: bool,
    operation_in_progress: bool,
    unhealthy: bool,
) -> DiagnosisReport {
    let highest = findings
        .iter()
        .map(|finding| finding.severity)
        .max()
        .unwrap_or(FindingSeverity::Info);
    let status = if highest == FindingSeverity::Error {
        DiagnosisStatus::Failed
    } else if operation_in_progress {
        DiagnosisStatus::InProgress
    } else if unhealthy && !terminal {
        DiagnosisStatus::Blocked
    } else if terminal {
        DiagnosisStatus::Healthy
    } else if unhealthy {
        DiagnosisStatus::Blocked
    } else {
        DiagnosisStatus::Unknown
    };
    let summary = findings
        .iter()
        .find(|finding| finding.code != "no_matching_evidence")
        .map(|finding| finding.summary.clone())
        .unwrap_or_else(|| match status {
            DiagnosisStatus::Healthy => format!("{scope} looks healthy"),
            DiagnosisStatus::InProgress => format!("{scope} is still progressing"),
            DiagnosisStatus::Blocked => format!("{scope} appears blocked"),
            DiagnosisStatus::Failed => format!("{scope} has failed"),
            DiagnosisStatus::Unknown => format!("{scope} is in an unknown state"),
        });

    DiagnosisReport {
        scope: scope.to_string(),
        status,
        summary,
        recommended_next_steps: recommended_next_steps(&findings),
        findings,
    }
}

fn recommended_next_steps(findings: &[DiagnosisFinding]) -> Vec<String> {
    let mut steps = BTreeSet::new();
    for finding in findings {
        match finding.code.as_str() {
            "drain_deadline_reached" => {
                let _ = steps.insert("Inspect the affected pipeline's recent events and retained logs around the drain deadline.".to_string());
                let _ = steps.insert("Increase the operation drain timeout only after identifying which receiver/exporter path is still busy.".to_string());
            }
            "runtime_error_while_draining" => {
                let _ = steps.insert("Check the runtime error source chain in retained logs to identify the failing node or channel.".to_string());
            }
            "blocked_pending_forward_drop" => {
                let _ = steps.insert("Inspect the topic receiver/exporter path for backpressure or closed-channel handling during shutdown.".to_string());
            }
            "pending_receiver_drain" => {
                let _ = steps.insert("Watch the scoped metrics and recent events to see which receiver has not completed ingress drain.".to_string());
            }
            "pending_sends_buffered" => {
                let _ = steps.insert("Inspect channel and node metrics for queues, pending sends, or buffered work that remains above zero.".to_string());
            }
            "pipeline_not_ready" => {
                let _ = steps.insert("Re-run the pipeline ready probe and inspect rollout/shutdown status before issuing another mutation.".to_string());
            }
            "operation_terminal_failure" => {
                let _ = steps.insert("Fetch the specific rollout/shutdown resource to inspect per-core failure detail.".to_string());
            }
            _ => {}
        }
    }
    if steps.is_empty() {
        let _ = steps.insert(
            "No immediate troubleshooting action is suggested by the current evidence.".to_string(),
        );
    }
    steps.into_iter().collect()
}

fn dedupe_findings(findings: Vec<DiagnosisFinding>) -> Vec<DiagnosisFinding> {
    let mut merged = std::collections::BTreeMap::<String, DiagnosisFinding>::new();
    for finding in findings {
        let _ = merged
            .entry(finding.code.clone())
            .and_modify(|existing| {
                if finding.severity > existing.severity {
                    existing.severity = finding.severity;
                    existing.summary = finding.summary.clone();
                }
                existing.evidence.extend(finding.evidence.clone());
                existing.evidence.truncate(5);
            })
            .or_insert(finding);
    }
    merged.into_values().collect()
}

fn probe_is_ok(probe: &pipelines::ProbeResult) -> bool {
    probe.status == pipelines::ProbeStatus::Ok
}

fn shutdown_is_terminal(state: &str) -> bool {
    matches!(state, "succeeded" | "failed")
}

fn event_evidence<'a>(
    events: impl Iterator<Item = &'a NormalizedEvent>,
    source: &str,
) -> Vec<EvidenceExcerpt> {
    events
        .take(3)
        .map(|event| EvidenceExcerpt {
            source: source.to_string(),
            time: Some(event.time.clone()),
            message: format!(
                "{}/{} core={} name={} {}{}",
                event.pipeline_group_id,
                event.pipeline_id,
                event.core_id,
                event.name,
                event.message.as_deref().unwrap_or(""),
                event
                    .detail
                    .as_ref()
                    .map(|value| format!(" ({value})"))
                    .unwrap_or_default()
            )
            .trim()
            .to_string(),
        })
        .collect()
}

fn log_evidence<'a>(
    entries: impl Iterator<Item = &'a telemetry::LogEntry>,
    source: &str,
) -> Vec<EvidenceExcerpt> {
    entries
        .take(3)
        .map(|entry| EvidenceExcerpt {
            source: source.to_string(),
            time: Some(entry.timestamp.clone()),
            message: format!("{} {} {}", entry.level, entry.target, entry.rendered),
        })
        .collect()
}

fn metric_value_positive(value: &telemetry::MetricValue) -> bool {
    match value {
        telemetry::MetricValue::U64(value) => *value > 0,
        telemetry::MetricValue::F64(value) => *value > 0.0,
        telemetry::MetricValue::Mmsc(value) => value.count > 0 || value.sum > 0.0,
    }
}

fn metric_value_string(value: &telemetry::MetricValue) -> String {
    match value {
        telemetry::MetricValue::U64(value) => value.to_string(),
        telemetry::MetricValue::F64(value) => value.to_string(),
        telemetry::MetricValue::Mmsc(value) => format!(
            "min={} max={} sum={} count={}",
            value.min, value.max, value.sum, value.count
        ),
    }
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}
