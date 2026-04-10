// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::args::{BundleOutput, MutationOutput, ReadOutput, StreamOutput};
use crate::error::CliError;
use crate::style::HumanStyle;
use crate::troubleshoot::{
    DiagnosisFinding, DiagnosisReport, GroupShutdownWatchSnapshot, GroupsDescribeReport,
    NormalizedEvent, PipelineDescribeReport,
};
use otap_df_admin_api::{engine, groups, pipelines, telemetry};
use serde::Serialize;
use serde_json::json;
use std::io::Write;

pub fn write_read_output<T: Serialize>(
    writer: &mut dyn Write,
    output: ReadOutput,
    value: &T,
) -> Result<(), CliError> {
    match output {
        ReadOutput::Human => unreachable!("human rendering is handled separately"),
        ReadOutput::Json => {
            serde_json::to_writer_pretty(&mut *writer, value).map_err(io_serialize_error)?;
            writeln!(writer)?;
        }
        ReadOutput::Yaml => {
            write!(
                writer,
                "{}",
                serde_yaml::to_string(value).map_err(io_serialize_error)?
            )?;
        }
    }
    writer.flush()?;
    Ok(())
}

pub fn write_mutation_output<T: Serialize>(
    writer: &mut dyn Write,
    output: MutationOutput,
    outcome: &str,
    value: &T,
) -> Result<(), CliError> {
    match output {
        MutationOutput::Human => unreachable!("human rendering is handled separately"),
        MutationOutput::Json => {
            serde_json::to_writer_pretty(
                &mut *writer,
                &json!({ "outcome": outcome, "data": value }),
            )
            .map_err(io_serialize_error)?;
            writeln!(writer)?;
        }
        MutationOutput::Yaml => {
            write!(
                writer,
                "{}",
                serde_yaml::to_string(&json!({ "outcome": outcome, "data": value }))
                    .map_err(io_serialize_error)?
            )?;
        }
        MutationOutput::Ndjson => {
            serde_json::to_writer(
                &mut *writer,
                &json!({ "event": "snapshot", "outcome": outcome, "data": value }),
            )
            .map_err(io_serialize_error)?;
            writeln!(writer)?;
        }
    }
    writer.flush()?;
    Ok(())
}

pub fn write_bundle_output<T: Serialize>(
    writer: &mut dyn Write,
    output: BundleOutput,
    value: &T,
) -> Result<(), CliError> {
    match output {
        BundleOutput::Json => {
            serde_json::to_writer_pretty(&mut *writer, value).map_err(io_serialize_error)?;
            writeln!(writer)?;
        }
        BundleOutput::Yaml => {
            write!(
                writer,
                "{}",
                serde_yaml::to_string(value).map_err(io_serialize_error)?
            )?;
        }
    }
    writer.flush()?;
    Ok(())
}

pub fn write_snapshot_event<T: Serialize>(
    writer: &mut dyn Write,
    resource: &str,
    value: &T,
) -> Result<(), CliError> {
    serde_json::to_writer(
        &mut *writer,
        &json!({
            "event": "snapshot",
            "resource": resource,
            "data": value
        }),
    )
    .map_err(io_serialize_error)?;
    writeln!(writer)?;
    writer.flush()?;
    Ok(())
}

pub fn write_log_event(
    writer: &mut dyn Write,
    entry: &telemetry::LogEntry,
) -> Result<(), CliError> {
    serde_json::to_writer(&mut *writer, &json!({ "event": "log", "log": entry }))
        .map_err(io_serialize_error)?;
    writeln!(writer)?;
    writer.flush()?;
    Ok(())
}

pub fn write_event_output<T: Serialize>(
    writer: &mut dyn Write,
    resource: &str,
    value: &T,
) -> Result<(), CliError> {
    serde_json::to_writer(
        &mut *writer,
        &json!({
            "event": resource,
            "data": value
        }),
    )
    .map_err(io_serialize_error)?;
    writeln!(writer)?;
    writer.flush()?;
    Ok(())
}

pub fn render_engine_status(style: &HumanStyle, status: &engine::Status) -> String {
    let mut lines = vec![
        field(style, "generated_at", status.generated_at.to_string()),
        field(style, "pipelines", status.pipelines.len().to_string()),
    ];
    for (name, pipeline) in &status.pipelines {
        lines.push(pipeline_summary_line(style, name, pipeline));
    }
    lines.join("\n")
}

pub fn render_groups_status(style: &HumanStyle, status: &groups::Status) -> String {
    let mut lines = vec![
        field(style, "generated_at", status.generated_at.to_string()),
        field(style, "pipelines", status.pipelines.len().to_string()),
    ];
    for (name, pipeline) in &status.pipelines {
        lines.push(pipeline_summary_line(style, name, pipeline));
    }
    lines.join("\n")
}

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
    for (name, pipeline) in &report.status.pipelines {
        lines.push(pipeline_summary_line(style, name, pipeline));
    }
    if !report.recent_events.is_empty() {
        lines.push(field(style, "recent_events", report.recent_events.len()));
        for event in report.recent_events.iter().take(8) {
            lines.push(render_event_line(style, event));
        }
    }
    lines.join("\n")
}

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
            failure.pipeline,
            style.state(format!("{:?}", failure.condition.kind)),
            style.state(format!("{:?}", failure.condition.status))
        ));
    }
    lines.join("\n")
}

pub fn render_pipeline_probe(style: &HumanStyle, probe: &pipelines::ProbeResult) -> String {
    let mut lines = vec![state_field(style, "status", format!("{:?}", probe.status))];
    if let Some(message) = &probe.message {
        lines.push(field(style, "message", message));
    }
    lines.join("\n")
}

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
            rollout.rollout_id,
            style.state(format!("{:?}", rollout.state)),
            rollout.target_generation
        ));
    }
    for condition in &report.status.conditions {
        lines.push(condition_line(style, condition));
    }
    for (core_id, core) in &report.status.cores {
        lines.push(format!(
            "{} {} phase={:?} delete_pending={} last_heartbeat_time={}",
            style.label("core:"),
            core_id,
            core.phase,
            core.delete_pending,
            core.last_heartbeat_time
        ));
    }
    if !report.recent_events.is_empty() {
        lines.push(field(style, "recent_events", report.recent_events.len()));
        for event in report.recent_events.iter().take(8) {
            lines.push(render_event_line(style, event));
        }
    }
    lines.join("\n")
}

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
            rollout.rollout_id,
            style.state(format!("{:?}", rollout.state)),
            rollout.target_generation
        ));
    }
    lines.push(style.header("pipeline:"));
    lines.push(serde_yaml::to_string(&details.pipeline).map_err(io_serialize_error)?);
    Ok(lines.join("\n"))
}

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
            rollout.rollout_id,
            style.state(format!("{:?}", rollout.state)),
            rollout.target_generation
        ));
    }
    for condition in &status.conditions {
        lines.push(condition_line(style, condition));
    }
    for (core_id, core) in &status.cores {
        lines.push(format!(
            "{} {} phase={:?} delete_pending={} last_heartbeat_time={}",
            style.label("core:"),
            core_id,
            core.phase,
            core.delete_pending,
            core.last_heartbeat_time
        ));
    }
    lines.join("\n")
}

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
    for core in &status.cores {
        lines.push(format!(
            "{} {} state={} target_generation={} previous_generation={} updated_at={}",
            style.label("core:"),
            core.core_id,
            style.state(&core.state),
            core.target_generation,
            core.previous_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            core.updated_at
        ));
    }
    lines.join("\n")
}

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
    for core in &status.cores {
        lines.push(format!(
            "{} {} state={} deployment_generation={} updated_at={}",
            style.label("core:"),
            core.core_id,
            style.state(&core.state),
            core.deployment_generation,
            core.updated_at
        ));
    }
    lines.join("\n")
}

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

pub fn render_metrics_compact(
    style: &HumanStyle,
    response: &telemetry::CompactMetricsResponse,
) -> String {
    let mut lines = vec![
        field(style, "timestamp", response.timestamp.to_string()),
        field(style, "metric_sets", response.metric_sets.len().to_string()),
    ];
    for metric_set in &response.metric_sets {
        lines.push(format!(
            "{} {} attributes={} metrics={}",
            style.header("metric_set:"),
            metric_set.name,
            metric_set.attributes.len(),
            metric_set.metrics.len()
        ));
        for (name, value) in &metric_set.metrics {
            lines.push(format!(
                "  {}={}",
                style.label(name),
                metric_value_string(value)
            ));
        }
    }
    lines.join("\n")
}

pub fn render_metrics_full(style: &HumanStyle, response: &telemetry::MetricsResponse) -> String {
    let mut lines = vec![
        field(style, "timestamp", response.timestamp.to_string()),
        field(style, "metric_sets", response.metric_sets.len().to_string()),
    ];
    for metric_set in &response.metric_sets {
        lines.push(format!(
            "{} {} attributes={} metrics={}",
            style.header("metric_set:"),
            metric_set.name,
            metric_set.attributes.len(),
            metric_set.metrics.len()
        ));
        for point in &metric_set.metrics {
            lines.push(format!(
                "  {} [{}] ({:?}) = {}",
                style.label(&point.metadata.name),
                style.dim(&point.metadata.unit),
                point.metadata.instrument,
                metric_value_string(&point.value)
            ));
        }
    }
    lines.join("\n")
}

pub fn render_events(style: &HumanStyle, events: &[NormalizedEvent]) -> String {
    let mut lines = vec![field(style, "event_count", events.len())];
    for event in events {
        lines.push(render_event_line(style, event));
    }
    lines.join("\n")
}

pub fn render_event_line(style: &HumanStyle, event: &NormalizedEvent) -> String {
    let target = format!("{}:{}", event.pipeline_group_id, event.pipeline_id);
    let extra = match (&event.message, &event.detail) {
        (Some(message), Some(detail)) => format!(" message={message} detail={detail}"),
        (Some(message), None) => format!(" message={message}"),
        (None, Some(detail)) => format!(" detail={detail}"),
        (None, None) => String::new(),
    };
    let node = event
        .node_id
        .as_ref()
        .map(|value| format!(" node={value}"))
        .unwrap_or_default();
    let generation = event
        .deployment_generation
        .map(|value| format!(" generation={value}"))
        .unwrap_or_default();
    format!(
        "{} {} core={} kind={} name={}{}{}{}",
        style.dim(&event.time),
        style.target(target),
        event.core_id,
        style.state(format!("{:?}", event.kind)),
        style.label(&event.name),
        generation,
        node,
        extra
    )
}

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
                evidence.message
            ));
        }
    }
    for step in &report.recommended_next_steps {
        lines.push(format!("{} {}", style.label("next_step:"), step));
    }
    lines.join("\n")
}

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
    for pipeline in &snapshot.pipelines {
        lines.push(format!(
            "{} {} running={}/{} terminal={} phases={}",
            style.header("pipeline:"),
            pipeline.pipeline,
            pipeline.running_cores,
            pipeline.total_cores,
            style.state(pipeline.terminal.to_string()),
            pipeline.phases.join(",")
        ));
    }
    lines.join("\n")
}

pub fn write_human(writer: &mut dyn Write, content: &str) -> Result<(), CliError> {
    writeln!(writer, "{content}")?;
    writer.flush()?;
    Ok(())
}

pub fn write_stream_human(
    writer: &mut dyn Write,
    resource: &str,
    content: &str,
    style: HumanStyle,
) -> Result<(), CliError> {
    writeln!(writer, "{}", style.header(format!("[{resource}]")))?;
    writeln!(writer, "{content}")?;
    writeln!(writer)?;
    writer.flush()?;
    Ok(())
}

pub fn write_stream_snapshot<T: Serialize>(
    writer: &mut dyn Write,
    output: StreamOutput,
    resource: &str,
    human: impl FnOnce() -> Result<String, CliError>,
    value: &T,
    style: HumanStyle,
) -> Result<(), CliError> {
    match output {
        StreamOutput::Human => write_stream_human(writer, resource, &human()?, style),
        StreamOutput::Ndjson => write_snapshot_event(writer, resource, value),
    }
}

fn pipeline_summary_line(style: &HumanStyle, name: &str, status: &pipelines::Status) -> String {
    let rollout = status
        .rollout
        .as_ref()
        .map(|value| style.state(format!("{:?}", value.state)))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "{} {} running={}/{} active_generation={} rollout={}",
        style.header("pipeline:"),
        name,
        style.state(status.running_cores.to_string()),
        status.total_cores,
        status
            .active_generation
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        rollout
    )
}

fn render_finding_line(style: &HumanStyle, finding: &DiagnosisFinding) -> String {
    format!(
        "{} code={} severity={} summary={}",
        style.label("finding:"),
        finding.code,
        style.state(format!("{:?}", finding.severity)),
        finding.summary
    )
}

fn condition_line(style: &HumanStyle, condition: &pipelines::Condition) -> String {
    format!(
        "{} kind={} status={} reason={} message={}",
        style.label("condition:"),
        style.state(format!("{:?}", condition.kind)),
        style.state(format!("{:?}", condition.status)),
        condition
            .reason
            .as_ref()
            .map(|value| value.as_str().to_string())
            .unwrap_or_else(|| "none".to_string()),
        condition
            .message
            .clone()
            .unwrap_or_else(|| "none".to_string())
    )
}

pub fn render_log_line(style: &HumanStyle, entry: &telemetry::LogEntry) -> String {
    format!(
        "{} [{}] {} {}",
        style.dim(&entry.timestamp),
        style.log_level(&entry.level),
        style.target(&entry.target),
        entry.rendered
    )
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

fn io_serialize_error(error: impl std::fmt::Display) -> CliError {
    CliError::config(format!("failed to serialize output: {error}"))
}

fn field(style: &HumanStyle, label: &str, value: impl std::fmt::Display) -> String {
    format!("{} {}", style.label(format!("{label}:")), value)
}

fn state_field(style: &HumanStyle, label: &str, value: impl AsRef<str>) -> String {
    format!(
        "{} {}",
        style.label(format!("{label}:")),
        style.state(value.as_ref())
    )
}
