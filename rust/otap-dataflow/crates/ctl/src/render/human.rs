// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Human-oriented renderers for CLI data models.

use super::output::io_serialize_error;
use super::table::{
    TableColumn, condition_line, field, format_metric_attributes, metric_value_string,
    render_table, state_field,
};
use crate::error::CliError;
use crate::style::{HumanStyle, terminal_safe};
use crate::troubleshoot::{
    DiagnosisFinding, DiagnosisReport, GroupShutdownWatchSnapshot, GroupsDescribeReport,
    NormalizedEvent, PipelineDescribeReport,
};
use otap_df_admin_api::{engine, groups, pipelines, telemetry};
use std::collections::BTreeMap;

/// Renders engine status as human-readable fields and a pipeline table.
pub fn render_engine_status(style: &HumanStyle, status: &engine::Status) -> String {
    let mut lines = vec![
        field(style, "generated_at", status.generated_at.to_string()),
        field(style, "pipelines", status.pipelines.len().to_string()),
    ];
    if !status.pipelines.is_empty() {
        lines.push(String::new());
        lines.push(render_pipeline_summary_table(style, &status.pipelines));
    }
    lines.join("\n")
}

/// Renders group status as human-readable fields and a pipeline table.
pub fn render_groups_status(style: &HumanStyle, status: &groups::Status) -> String {
    let mut lines = vec![
        field(style, "generated_at", status.generated_at.to_string()),
        field(style, "pipelines", status.pipelines.len().to_string()),
    ];
    if !status.pipelines.is_empty() {
        lines.push(String::new());
        lines.push(render_pipeline_summary_table(style, &status.pipelines));
    }
    lines.join("\n")
}

/// Renders the derived group describe report for terminal output.
pub fn render_groups_describe(style: &HumanStyle, report: &GroupsDescribeReport) -> String {
    let mut lines = vec![
        field(style, "generated_at", &report.status.generated_at),
        field(style, "total_pipelines", report.summary.total_pipelines),
        field(style, "running_pipelines", report.summary.running_pipelines),
        field(style, "ready_pipelines", report.summary.ready_pipelines),
        field(
            style,
            "terminal_pipelines",
            report.summary.terminal_pipelines,
        ),
    ];
    if !report.summary.non_ready_pipelines.is_empty() {
        lines.push(field(
            style,
            "non_ready",
            report.summary.non_ready_pipelines.join(", "),
        ));
    }
    if !report.summary.non_terminal_pipelines.is_empty() {
        lines.push(field(
            style,
            "non_terminal",
            report.summary.non_terminal_pipelines.join(", "),
        ));
    }
    if !report.status.pipelines.is_empty() {
        lines.push(String::new());
        lines.push(render_pipeline_summary_table(
            style,
            &report.status.pipelines,
        ));
    }
    if !report.recent_events.is_empty() {
        lines.push(String::new());
        lines.push(field(style, "recent_events", report.recent_events.len()));
        for event in report.recent_events.iter().take(8) {
            lines.push(render_event_line(style, event));
        }
    }
    lines.join("\n")
}

/// Renders an engine liveness or readiness probe response.
pub fn render_engine_probe(style: &HumanStyle, probe: &engine::ProbeResponse) -> String {
    let mut lines = vec![
        field(style, "probe", format!("{:?}", probe.probe)),
        state_field(style, "status", format!("{:?}", probe.status)),
        field(style, "generated_at", probe.generated_at.to_string()),
    ];
    if let Some(message) = &probe.message {
        lines.push(field(style, "message", message));
    }
    for failure in &probe.failing {
        lines.push(format!(
            "{} {} kind={} status={}",
            style.label("failing_pipeline:"),
            terminal_safe(&failure.pipeline),
            style.state(format!("{:?}", failure.condition.kind)),
            style.state(format!("{:?}", failure.condition.status))
        ));
    }
    lines.join("\n")
}

/// Renders a pipeline liveness or readiness probe response.
pub fn render_pipeline_probe(style: &HumanStyle, probe: &pipelines::ProbeResult) -> String {
    let mut lines = vec![state_field(style, "status", format!("{:?}", probe.status))];
    if let Some(message) = &probe.message {
        lines.push(field(style, "message", message));
    }
    lines.join("\n")
}

/// Renders the derived pipeline describe report for terminal output.
pub fn render_pipeline_describe(style: &HumanStyle, report: &PipelineDescribeReport) -> String {
    let mut lines = vec![
        field(
            style,
            "pipeline_group_id",
            &report.details.pipeline_group_id,
        ),
        field(style, "pipeline_id", &report.details.pipeline_id),
        field(
            style,
            "active_generation",
            report
                .details
                .active_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
        ),
        state_field(style, "livez", format!("{:?}", report.livez.status)),
        state_field(style, "readyz", format!("{:?}", report.readyz.status)),
        field(
            style,
            "running_cores",
            format!(
                "{}/{}",
                report.status.running_cores, report.status.total_cores
            ),
        ),
    ];
    if let Some(message) = &report.livez.message {
        lines.push(field(style, "livez_message", message));
    }
    if let Some(message) = &report.readyz.message {
        lines.push(field(style, "readyz_message", message));
    }
    if let Some(rollout) = &report.status.rollout {
        lines.push(format!(
            "{} id={} state={} target_generation={}",
            style.label("rollout:"),
            terminal_safe(&rollout.rollout_id),
            style.state(format!("{:?}", rollout.state)),
            rollout.target_generation
        ));
    }
    for condition in &report.status.conditions {
        lines.push(condition_line(style, condition));
    }
    if !report.status.cores.is_empty() {
        lines.push(String::new());
        lines.push(render_pipeline_core_table(style, &report.status.cores));
    }
    if !report.recent_events.is_empty() {
        lines.push(String::new());
        lines.push(field(style, "recent_events", report.recent_events.len()));
        for event in report.recent_events.iter().take(8) {
            lines.push(render_event_line(style, event));
        }
    }
    lines.join("\n")
}

/// Renders raw pipeline details, including the pipeline config as YAML.
pub fn render_pipeline_details(
    style: &HumanStyle,
    details: &pipelines::PipelineDetails,
) -> Result<String, CliError> {
    let mut lines = vec![
        field(style, "pipeline_group_id", &details.pipeline_group_id),
        field(style, "pipeline_id", &details.pipeline_id),
        field(
            style,
            "active_generation",
            details
                .active_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
        ),
    ];
    if let Some(rollout) = &details.rollout {
        lines.push(format!(
            "{} id={} state={} target_generation={}",
            style.label("rollout:"),
            terminal_safe(&rollout.rollout_id),
            style.state(format!("{:?}", rollout.state)),
            rollout.target_generation
        ));
    }
    lines.push(style.header("pipeline:"));
    lines.push(terminal_safe(
        serde_yaml::to_string(&details.pipeline).map_err(io_serialize_error)?,
    ));
    Ok(lines.join("\n"))
}

/// Renders one pipeline status snapshot.
pub fn render_pipeline_status(style: &HumanStyle, status: &pipelines::Status) -> String {
    let mut lines = vec![
        field(
            style,
            "running_cores",
            format!("{}/{}", status.running_cores, status.total_cores),
        ),
        field(
            style,
            "active_generation",
            status
                .active_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
        ),
    ];
    if let Some(rollout) = &status.rollout {
        lines.push(format!(
            "{} id={} state={} target_generation={}",
            style.label("rollout:"),
            terminal_safe(&rollout.rollout_id),
            style.state(format!("{:?}", rollout.state)),
            rollout.target_generation
        ));
    }
    for condition in &status.conditions {
        lines.push(condition_line(style, condition));
    }
    if !status.cores.is_empty() {
        lines.push(String::new());
        lines.push(render_pipeline_core_table(style, &status.cores));
    }
    lines.join("\n")
}

/// Renders one rollout operation status snapshot.
pub fn render_rollout_status(style: &HumanStyle, status: &pipelines::RolloutStatus) -> String {
    let mut lines = vec![
        field(style, "rollout_id", &status.rollout_id),
        field(style, "pipeline_group_id", &status.pipeline_group_id),
        field(style, "pipeline_id", &status.pipeline_id),
        field(style, "action", &status.action),
        state_field(style, "state", format!("{:?}", status.state)),
        field(
            style,
            "target_generation",
            status.target_generation.to_string(),
        ),
        field(
            style,
            "previous_generation",
            status
                .previous_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
        ),
        field(style, "started_at", status.started_at.to_string()),
        field(style, "updated_at", status.updated_at.to_string()),
    ];
    if let Some(reason) = &status.failure_reason {
        lines.push(field(style, "failure_reason", reason));
    }
    if !status.cores.is_empty() {
        lines.push(String::new());
        lines.push(render_rollout_core_table(style, &status.cores));
    }
    lines.join("\n")
}

/// Renders one shutdown operation status snapshot.
pub fn render_shutdown_status(style: &HumanStyle, status: &pipelines::ShutdownStatus) -> String {
    let mut lines = vec![
        field(style, "shutdown_id", &status.shutdown_id),
        field(style, "pipeline_group_id", &status.pipeline_group_id),
        field(style, "pipeline_id", &status.pipeline_id),
        state_field(style, "state", &status.state),
        field(style, "started_at", status.started_at.to_string()),
        field(style, "updated_at", status.updated_at.to_string()),
    ];
    if let Some(reason) = &status.failure_reason {
        lines.push(field(style, "failure_reason", reason));
    }
    if !status.cores.is_empty() {
        lines.push(String::new());
        lines.push(render_shutdown_core_table(style, &status.cores));
    }
    lines.join("\n")
}

/// Renders the response returned by a group shutdown request.
pub fn render_groups_shutdown(style: &HumanStyle, response: &groups::ShutdownResponse) -> String {
    let mut lines = vec![state_field(
        style,
        "status",
        format!("{:?}", response.status),
    )];
    if let Some(duration_ms) = response.duration_ms {
        lines.push(field(style, "duration_ms", duration_ms.to_string()));
    }
    if let Some(errors) = &response.errors {
        for error in errors {
            lines.push(field(style, "error", error));
        }
    }
    lines.join("\n")
}

/// Renders retained logs with response cursor metadata.
pub fn render_logs(style: &HumanStyle, response: &telemetry::LogsResponse) -> String {
    let mut lines = vec![
        field(style, "oldest_seq", format!("{:?}", response.oldest_seq)),
        field(style, "newest_seq", format!("{:?}", response.newest_seq)),
        field(style, "next_seq", response.next_seq.to_string()),
        field(style, "retained_bytes", response.retained_bytes.to_string()),
        field(style, "log_count", response.logs.len().to_string()),
    ];
    for entry in &response.logs {
        lines.push(render_log_line(style, entry));
    }
    lines.join("\n")
}

/// Renders compact metrics as grouped human-readable tables.
pub fn render_metrics_compact(
    style: &HumanStyle,
    response: &telemetry::CompactMetricsResponse,
) -> String {
    let mut lines = vec![
        field(style, "timestamp", response.timestamp.to_string()),
        field(style, "metric_sets", response.metric_sets.len().to_string()),
    ];
    for metric_set in &response.metric_sets {
        lines.push(String::new());
        lines.push(format!(
            "{} {}",
            style.header("metric_set:"),
            terminal_safe(&metric_set.name)
        ));
        lines.push(field(
            style,
            "attributes",
            format_metric_attributes(&metric_set.attributes),
        ));
        lines.push(render_table(
            style,
            &[TableColumn::left("metric"), TableColumn::right("value")],
            metric_set
                .metrics
                .iter()
                .map(|(name, value)| vec![terminal_safe(name), metric_value_string(value)])
                .collect(),
        ));
    }
    lines.join("\n")
}

/// Renders full metrics as grouped human-readable tables.
pub fn render_metrics_full(style: &HumanStyle, response: &telemetry::MetricsResponse) -> String {
    let mut lines = vec![
        field(style, "timestamp", response.timestamp.to_string()),
        field(style, "metric_sets", response.metric_sets.len().to_string()),
    ];
    for metric_set in &response.metric_sets {
        lines.push(String::new());
        lines.push(format!(
            "{} {}",
            style.header("metric_set:"),
            terminal_safe(&metric_set.name)
        ));
        lines.push(field(
            style,
            "attributes",
            format_metric_attributes(&metric_set.attributes),
        ));
        lines.push(render_table(
            style,
            &[
                TableColumn::left("metric"),
                TableColumn::left("unit"),
                TableColumn::left("instrument"),
                TableColumn::right("value"),
            ],
            metric_set
                .metrics
                .iter()
                .map(|point| {
                    vec![
                        terminal_safe(&point.metadata.name),
                        terminal_safe(&point.metadata.unit),
                        format!("{:?}", point.metadata.instrument),
                        metric_value_string(&point.value),
                    ]
                })
                .collect(),
        ));
    }
    lines.join("\n")
}

/// Renders normalized events as a finite human-readable list.
pub fn render_events(style: &HumanStyle, events: &[NormalizedEvent]) -> String {
    let mut lines = vec![field(style, "event_count", events.len())];
    for event in events {
        lines.push(render_event_line(style, event));
    }
    lines.join("\n")
}

/// Renders one normalized event as a compact single-line entry.
pub fn render_event_line(style: &HumanStyle, event: &NormalizedEvent) -> String {
    let target = format!(
        "{}:{}",
        terminal_safe(&event.pipeline_group_id),
        terminal_safe(&event.pipeline_id)
    );
    let extra = match (&event.message, &event.detail) {
        (Some(message), Some(detail)) => {
            format!(
                " message={} detail={}",
                terminal_safe(message),
                terminal_safe(detail)
            )
        }
        (Some(message), None) => format!(" message={}", terminal_safe(message)),
        (None, Some(detail)) => format!(" detail={}", terminal_safe(detail)),
        (None, None) => String::new(),
    };
    let node = event
        .node_id
        .as_ref()
        .map(|value| format!(" node={}", terminal_safe(value)))
        .unwrap_or_default();
    let generation = event
        .deployment_generation
        .map(|value| format!(" generation={value}"))
        .unwrap_or_default();
    format!(
        "{} {} core={} kind={} name={}{}{}{}",
        style.dim(terminal_safe(&event.time)),
        style.target(target),
        event.core_id,
        style.state(format!("{:?}", event.kind)),
        style.label(terminal_safe(&event.name)),
        generation,
        node,
        extra
    )
}

/// Renders a diagnosis report with findings, evidence, and next steps.
pub fn render_diagnosis(style: &HumanStyle, report: &DiagnosisReport) -> String {
    let mut lines = vec![
        field(style, "scope", &report.scope),
        state_field(style, "status", format!("{:?}", report.status)),
        field(style, "summary", &report.summary),
        field(style, "findings", report.findings.len()),
    ];
    for finding in &report.findings {
        lines.push(render_finding_line(style, finding));
        for evidence in &finding.evidence {
            lines.push(format!(
                "  {} {} {}",
                style.label("evidence:"),
                evidence
                    .time
                    .as_ref()
                    .map(|value| style.dim(value))
                    .unwrap_or_default(),
                terminal_safe(&evidence.message)
            ));
        }
    }
    for step in &report.recommended_next_steps {
        lines.push(format!(
            "{} {}",
            style.label("next_step:"),
            terminal_safe(step)
        ));
    }
    lines.join("\n")
}

/// Renders one group shutdown watch snapshot.
pub fn render_group_shutdown_watch(
    style: &HumanStyle,
    snapshot: &GroupShutdownWatchSnapshot,
) -> String {
    let mut lines = vec![
        field(style, "started_at", &snapshot.started_at),
        field(style, "generated_at", &snapshot.generated_at),
        state_field(style, "request_status", &snapshot.request_status),
        field(style, "elapsed_ms", snapshot.elapsed_ms),
        field(
            style,
            "terminal_pipelines",
            format!(
                "{}/{}",
                snapshot.terminal_pipelines, snapshot.total_pipelines
            ),
        ),
        state_field(style, "all_terminal", snapshot.all_terminal.to_string()),
    ];
    if !snapshot.pipelines.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            style,
            &[
                TableColumn::left("pipeline"),
                TableColumn::right("running"),
                TableColumn::left("terminal"),
                TableColumn::left("phases"),
            ],
            snapshot
                .pipelines
                .iter()
                .map(|pipeline| {
                    vec![
                        terminal_safe(&pipeline.pipeline),
                        format!("{}/{}", pipeline.running_cores, pipeline.total_cores),
                        style.state(pipeline.terminal.to_string()),
                        style.state(terminal_safe(pipeline.phases.join(", "))),
                    ]
                })
                .collect(),
        ));
    }
    lines.join("\n")
}

/// Renders one retained-log entry as a compact single-line entry.
pub fn render_log_line(style: &HumanStyle, entry: &telemetry::LogEntry) -> String {
    format!(
        "{} [{}] {} {}",
        style.dim(terminal_safe(&entry.timestamp)),
        style.log_level(terminal_safe(&entry.level)),
        style.target(terminal_safe(&entry.target)),
        terminal_safe(&entry.rendered)
    )
}

fn render_finding_line(style: &HumanStyle, finding: &DiagnosisFinding) -> String {
    format!(
        "{} code={} severity={} summary={}",
        style.label("finding:"),
        terminal_safe(&finding.code),
        style.state(format!("{:?}", finding.severity)),
        terminal_safe(&finding.summary)
    )
}

fn render_pipeline_summary_table(
    style: &HumanStyle,
    pipelines: &BTreeMap<String, pipelines::Status>,
) -> String {
    render_table(
        style,
        &[
            TableColumn::left("pipeline"),
            TableColumn::right("running"),
            TableColumn::right("active_generation"),
            TableColumn::left("rollout"),
        ],
        pipelines
            .iter()
            .map(|(name, status)| {
                vec![
                    terminal_safe(name),
                    format!("{}/{}", status.running_cores, status.total_cores),
                    status
                        .active_generation
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    status
                        .rollout
                        .as_ref()
                        .map(|value| style.state(format!("{:?}", value.state)))
                        .unwrap_or_else(|| "none".to_string()),
                ]
            })
            .collect(),
    )
}

fn render_pipeline_core_table(
    style: &HumanStyle,
    cores: &BTreeMap<usize, pipelines::CoreStatus>,
) -> String {
    render_table(
        style,
        &[
            TableColumn::right("core"),
            TableColumn::left("phase"),
            TableColumn::left("delete_pending"),
            TableColumn::left("last_heartbeat_time"),
        ],
        cores
            .iter()
            .map(|(core_id, core)| {
                vec![
                    core_id.to_string(),
                    style.state(format!("{:?}", core.phase)),
                    core.delete_pending.to_string(),
                    terminal_safe(&core.last_heartbeat_time),
                ]
            })
            .collect(),
    )
}

fn render_rollout_core_table(style: &HumanStyle, cores: &[pipelines::RolloutCoreStatus]) -> String {
    render_table(
        style,
        &[
            TableColumn::right("core"),
            TableColumn::left("state"),
            TableColumn::right("target_generation"),
            TableColumn::right("previous_generation"),
            TableColumn::left("updated_at"),
        ],
        cores
            .iter()
            .map(|core| {
                vec![
                    core.core_id.to_string(),
                    style.state(terminal_safe(&core.state)),
                    core.target_generation.to_string(),
                    core.previous_generation
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    terminal_safe(&core.updated_at),
                ]
            })
            .collect(),
    )
}

fn render_shutdown_core_table(
    style: &HumanStyle,
    cores: &[pipelines::ShutdownCoreStatus],
) -> String {
    render_table(
        style,
        &[
            TableColumn::right("core"),
            TableColumn::left("state"),
            TableColumn::right("deployment_generation"),
            TableColumn::left("updated_at"),
        ],
        cores
            .iter()
            .map(|core| {
                vec![
                    core.core_id.to_string(),
                    style.state(terminal_safe(&core.state)),
                    core.deployment_generation.to_string(),
                    terminal_safe(&core.updated_at),
                ]
            })
            .collect(),
    )
}
