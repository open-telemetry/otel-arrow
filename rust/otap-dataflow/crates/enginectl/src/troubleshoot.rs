// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otap_df_admin_api::{groups, pipelines, telemetry};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedEventKind {
    Request,
    Success,
    Error,
    Log,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedEvent {
    pub time: String,
    pub kind: NormalizedEventKind,
    pub name: String,
    pub pipeline_group_id: String,
    pub pipeline_id: String,
    pub core_id: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record: Option<Value>,
}

impl NormalizedEvent {
    pub fn identity_key(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                "{}|{:?}|{}|{}/{}|{}|{:?}|{:?}|{:?}|{:?}",
                self.time,
                self.kind,
                self.name,
                self.pipeline_group_id,
                self.pipeline_id,
                self.core_id,
                self.deployment_generation,
                self.node_id,
                self.message,
                self.detail
            )
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventFilters {
    pub kinds: Vec<NormalizedEventKind>,
    pub pipeline_group_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub contains: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LogFilters {
    pub level: Option<String>,
    pub target: Option<String>,
    pub event: Option<String>,
    pub pipeline_group_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub contains: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MetricsFilters {
    pub metric_sets: Vec<String>,
    pub metric_names: Vec<String>,
    pub pipeline_group_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub core_id: Option<usize>,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsDescribeReport {
    pub status: groups::Status,
    pub summary: GroupsSummary,
    pub recent_events: Vec<NormalizedEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsSummary {
    pub total_pipelines: usize,
    pub running_pipelines: usize,
    pub ready_pipelines: usize,
    pub terminal_pipelines: usize,
    pub non_ready_pipelines: Vec<String>,
    pub non_terminal_pipelines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDescribeReport {
    pub details: pipelines::PipelineDetails,
    pub status: pipelines::Status,
    pub livez: pipelines::ProbeResult,
    pub readyz: pipelines::ProbeResult,
    pub recent_events: Vec<NormalizedEvent>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisStatus {
    Healthy,
    InProgress,
    Blocked,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceExcerpt {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisFinding {
    pub code: String,
    pub severity: FindingSeverity,
    pub summary: String,
    pub evidence: Vec<EvidenceExcerpt>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisReport {
    pub scope: String,
    pub status: DiagnosisStatus,
    pub summary: String,
    pub findings: Vec<DiagnosisFinding>,
    pub recommended_next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupShutdownWatchPipeline {
    pub pipeline: String,
    pub running_cores: usize,
    pub total_cores: usize,
    pub terminal: bool,
    pub phases: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupShutdownWatchSnapshot {
    pub started_at: String,
    pub generated_at: String,
    pub request_status: String,
    pub elapsed_ms: u64,
    pub total_pipelines: usize,
    pub terminal_pipelines: usize,
    pub all_terminal: bool,
    pub pipelines: Vec<GroupShutdownWatchPipeline>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleMetricsShape {
    Compact,
    Full,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleMetadata {
    pub collected_at: String,
    pub logs_limit: usize,
    pub metrics_shape: BundleMetricsShape,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "shape", content = "data", rename_all = "snake_case")]
pub enum BundleMetrics {
    Compact(telemetry::CompactMetricsResponse),
    Full(telemetry::MetricsResponse),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupsBundle {
    pub metadata: BundleMetadata,
    pub describe: GroupsDescribeReport,
    pub diagnosis: DiagnosisReport,
    pub logs: telemetry::LogsResponse,
    pub metrics: BundleMetrics,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineBundle {
    pub metadata: BundleMetadata,
    pub describe: PipelineDescribeReport,
    pub diagnosis: DiagnosisReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollout_status: Option<pipelines::RolloutStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutdown_status: Option<pipelines::ShutdownStatus>,
    pub logs: telemetry::LogsResponse,
    pub metrics: BundleMetrics,
}

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

pub fn extract_events_from_group_status(
    status: &groups::Status,
    filters: Option<&EventFilters>,
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

pub fn extract_events_from_pipeline_status(
    pipeline_group_id: &str,
    pipeline_id: &str,
    status: &pipelines::Status,
    filters: Option<&EventFilters>,
) -> Vec<NormalizedEvent> {
    let mut events = Vec::new();
    for (core_id, core) in &status.cores {
        let Some(recent_events) = &core.recent_events else {
            continue;
        };
        for event in recent_events {
            let normalized = normalize_event(pipeline_group_id, pipeline_id, *core_id, event);
            if filters.is_none_or(|value| event_matches(&normalized, value)) {
                events.push(normalized);
            }
        }
    }
    sort_events(&mut events);
    events
}

pub fn tail_events(events: Vec<NormalizedEvent>, tail: Option<usize>) -> Vec<NormalizedEvent> {
    let Some(tail) = tail else {
        return events;
    };
    let keep_from = events.len().saturating_sub(tail);
    events.into_iter().skip(keep_from).collect()
}

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

pub fn diagnose_group_shutdown(
    status: &groups::Status,
    logs: &telemetry::LogsResponse,
    metrics: &telemetry::CompactMetricsResponse,
) -> DiagnosisReport {
    let events = extract_events_from_group_status(status, None);
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

pub fn pipeline_is_ready(status: &pipelines::Status) -> bool {
    status.conditions.iter().any(|condition| {
        condition.kind == pipelines::ConditionKind::Ready
            && condition.status == pipelines::ConditionStatus::True
    })
}

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

fn event_matches(event: &NormalizedEvent, filters: &EventFilters) -> bool {
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
    let mut merged = BTreeMap::<String, DiagnosisFinding>::new();
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

fn pipeline_phases(status: &pipelines::Status) -> Vec<String> {
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

fn log_record_summary(record: &Value) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

        assert!(event_matches(&event, &filters));
    }

    #[test]
    fn log_filter_matches_context_attributes_and_contains() {
        let response = telemetry::LogsResponse {
            oldest_seq: Some(1),
            newest_seq: Some(1),
            next_seq: 2,
            truncated_before_seq: None,
            dropped_on_ingest: 0,
            dropped_on_retention: 0,
            retained_bytes: 32,
            logs: vec![telemetry::LogEntry {
                seq: 1,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
                level: "WARN".to_string(),
                target: "controller".to_string(),
                event_name: "controller.pipeline_runtime_failed".to_string(),
                file: None,
                line: None,
                rendered: "channel is closed and the message could not be sent".to_string(),
                contexts: vec![telemetry::ResolvedLogContext {
                    entity_key: "EntityKey(1)".to_string(),
                    schema_name: Some("node.attrs".to_string()),
                    attributes: BTreeMap::from([
                        (
                            "pipeline.group.id".to_string(),
                            telemetry::AttributeValue::String("group-a".to_string()),
                        ),
                        (
                            "pipeline.id".to_string(),
                            telemetry::AttributeValue::String("pipeline-a".to_string()),
                        ),
                        (
                            "node.id".to_string(),
                            telemetry::AttributeValue::String("receiver".to_string()),
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

    #[test]
    fn metrics_filters_prune_sets_and_metric_names() {
        let response = telemetry::CompactMetricsResponse {
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            metric_sets: vec![telemetry::MetricSet {
                name: "engine.pipeline".to_string(),
                attributes: BTreeMap::from([
                    (
                        "pipeline.group.id".to_string(),
                        telemetry::AttributeValue::String("group-a".to_string()),
                    ),
                    (
                        "pipeline.id".to_string(),
                        telemetry::AttributeValue::String("pipeline-a".to_string()),
                    ),
                    (
                        "node.id".to_string(),
                        telemetry::AttributeValue::String("receiver".to_string()),
                    ),
                ]),
                metrics: BTreeMap::from([
                    ("pending.sends".to_string(), telemetry::MetricValue::U64(4)),
                    ("processed".to_string(), telemetry::MetricValue::U64(9)),
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

    #[test]
    fn diagnosis_surfaces_drain_deadline_and_drop_signatures() {
        let report = diagnose_group_shutdown(
            &groups::Status {
                generated_at: "2026-01-01T00:00:00Z".to_string(),
                pipelines: BTreeMap::new(),
            },
            &telemetry::LogsResponse {
                oldest_seq: Some(1),
                newest_seq: Some(2),
                next_seq: 3,
                truncated_before_seq: None,
                dropped_on_ingest: 0,
                dropped_on_retention: 0,
                retained_bytes: 32,
                logs: vec![telemetry::LogEntry {
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
            &telemetry::CompactMetricsResponse {
                timestamp: "2026-01-01T00:00:00Z".to_string(),
                metric_sets: vec![telemetry::MetricSet {
                    name: "engine.pipeline".to_string(),
                    attributes: BTreeMap::new(),
                    metrics: BTreeMap::from([(
                        "pending.sends".to_string(),
                        telemetry::MetricValue::U64(1),
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

    #[test]
    fn normalized_log_event_preserves_record() {
        let event = normalize_event(
            "group-a",
            "pipeline-a",
            0,
            &pipelines::ObservedEvent::Log(pipelines::LoggedObservedEvent {
                time: "2026-01-01T00:00:00Z".to_string(),
                record: json!({"message": "hello"}),
            }),
        );

        assert_eq!(event.kind, NormalizedEventKind::Log);
        assert_eq!(event.record, Some(json!({"message": "hello"})));
    }
}
