// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::args::{MutationOutput, ReadOutput, StreamOutput};
use crate::error::CliError;
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

pub fn render_engine_status(status: &engine::Status) -> String {
    let mut lines = vec![
        format!("generated_at: {}", status.generated_at),
        format!("pipelines: {}", status.pipelines.len()),
    ];
    for (name, pipeline) in &status.pipelines {
        lines.push(pipeline_summary_line(name, pipeline));
    }
    lines.join("\n")
}

pub fn render_groups_status(status: &groups::Status) -> String {
    let mut lines = vec![
        format!("generated_at: {}", status.generated_at),
        format!("pipelines: {}", status.pipelines.len()),
    ];
    for (name, pipeline) in &status.pipelines {
        lines.push(pipeline_summary_line(name, pipeline));
    }
    lines.join("\n")
}

pub fn render_engine_probe(probe: &engine::ProbeResponse) -> String {
    let mut lines = vec![
        format!("probe: {:?}", probe.probe),
        format!("status: {:?}", probe.status),
        format!("generated_at: {}", probe.generated_at),
    ];
    if let Some(message) = &probe.message {
        lines.push(format!("message: {message}"));
    }
    for failure in &probe.failing {
        lines.push(format!(
            "failing_pipeline: {} kind={:?} status={:?}",
            failure.pipeline, failure.condition.kind, failure.condition.status
        ));
    }
    lines.join("\n")
}

pub fn render_pipeline_probe(probe: &pipelines::ProbeResult) -> String {
    let mut lines = vec![format!("status: {:?}", probe.status)];
    if let Some(message) = &probe.message {
        lines.push(format!("message: {message}"));
    }
    lines.join("\n")
}

pub fn render_pipeline_details(details: &pipelines::PipelineDetails) -> Result<String, CliError> {
    let mut lines = vec![
        format!("pipeline_group_id: {}", details.pipeline_group_id),
        format!("pipeline_id: {}", details.pipeline_id),
        format!(
            "active_generation: {}",
            details
                .active_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
    ];
    if let Some(rollout) = &details.rollout {
        lines.push(format!(
            "rollout: id={} state={:?} target_generation={}",
            rollout.rollout_id, rollout.state, rollout.target_generation
        ));
    }
    lines.push("pipeline:".to_string());
    lines.push(serde_yaml::to_string(&details.pipeline).map_err(io_serialize_error)?);
    Ok(lines.join("\n"))
}

pub fn render_pipeline_status(status: &pipelines::Status) -> String {
    let mut lines = vec![
        format!(
            "running_cores: {}/{}",
            status.running_cores, status.total_cores
        ),
        format!(
            "active_generation: {}",
            status
                .active_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
    ];
    if let Some(rollout) = &status.rollout {
        lines.push(format!(
            "rollout: id={} state={:?} target_generation={}",
            rollout.rollout_id, rollout.state, rollout.target_generation
        ));
    }
    for condition in &status.conditions {
        lines.push(condition_line(condition));
    }
    for (core_id, core) in &status.cores {
        lines.push(format!(
            "core={} phase={:?} delete_pending={} last_heartbeat_time={}",
            core_id, core.phase, core.delete_pending, core.last_heartbeat_time
        ));
    }
    lines.join("\n")
}

pub fn render_rollout_status(status: &pipelines::RolloutStatus) -> String {
    let mut lines = vec![
        format!("rollout_id: {}", status.rollout_id),
        format!("pipeline_group_id: {}", status.pipeline_group_id),
        format!("pipeline_id: {}", status.pipeline_id),
        format!("action: {}", status.action),
        format!("state: {:?}", status.state),
        format!("target_generation: {}", status.target_generation),
        format!(
            "previous_generation: {}",
            status
                .previous_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!("started_at: {}", status.started_at),
        format!("updated_at: {}", status.updated_at),
    ];
    if let Some(reason) = &status.failure_reason {
        lines.push(format!("failure_reason: {reason}"));
    }
    for core in &status.cores {
        lines.push(format!(
            "core={} state={} target_generation={} previous_generation={} updated_at={}",
            core.core_id,
            core.state,
            core.target_generation,
            core.previous_generation
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            core.updated_at
        ));
    }
    lines.join("\n")
}

pub fn render_shutdown_status(status: &pipelines::ShutdownStatus) -> String {
    let mut lines = vec![
        format!("shutdown_id: {}", status.shutdown_id),
        format!("pipeline_group_id: {}", status.pipeline_group_id),
        format!("pipeline_id: {}", status.pipeline_id),
        format!("state: {}", status.state),
        format!("started_at: {}", status.started_at),
        format!("updated_at: {}", status.updated_at),
    ];
    if let Some(reason) = &status.failure_reason {
        lines.push(format!("failure_reason: {reason}"));
    }
    for core in &status.cores {
        lines.push(format!(
            "core={} state={} deployment_generation={} updated_at={}",
            core.core_id, core.state, core.deployment_generation, core.updated_at
        ));
    }
    lines.join("\n")
}

pub fn render_groups_shutdown(response: &groups::ShutdownResponse) -> String {
    let mut lines = vec![format!("status: {:?}", response.status)];
    if let Some(duration_ms) = response.duration_ms {
        lines.push(format!("duration_ms: {duration_ms}"));
    }
    if let Some(errors) = &response.errors {
        for error in errors {
            lines.push(format!("error: {error}"));
        }
    }
    lines.join("\n")
}

pub fn render_logs(response: &telemetry::LogsResponse) -> String {
    let mut lines = vec![
        format!("oldest_seq: {:?}", response.oldest_seq),
        format!("newest_seq: {:?}", response.newest_seq),
        format!("next_seq: {}", response.next_seq),
        format!("retained_bytes: {}", response.retained_bytes),
        format!("log_count: {}", response.logs.len()),
    ];
    for entry in &response.logs {
        lines.push(render_log_line(entry));
    }
    lines.join("\n")
}

pub fn render_metrics_compact(response: &telemetry::CompactMetricsResponse) -> String {
    let mut lines = vec![
        format!("timestamp: {}", response.timestamp),
        format!("metric_sets: {}", response.metric_sets.len()),
    ];
    for metric_set in &response.metric_sets {
        lines.push(format!(
            "metric_set: {} attributes={} metrics={}",
            metric_set.name,
            metric_set.attributes.len(),
            metric_set.metrics.len()
        ));
        for (name, value) in &metric_set.metrics {
            lines.push(format!("  {name}={}", metric_value_string(value)));
        }
    }
    lines.join("\n")
}

pub fn render_metrics_full(response: &telemetry::MetricsResponse) -> String {
    let mut lines = vec![
        format!("timestamp: {}", response.timestamp),
        format!("metric_sets: {}", response.metric_sets.len()),
    ];
    for metric_set in &response.metric_sets {
        lines.push(format!(
            "metric_set: {} attributes={} metrics={}",
            metric_set.name,
            metric_set.attributes.len(),
            metric_set.metrics.len()
        ));
        for point in &metric_set.metrics {
            lines.push(format!(
                "  {} [{}] ({:?}) = {}",
                point.metadata.name,
                point.metadata.unit,
                point.metadata.instrument,
                metric_value_string(&point.value)
            ));
        }
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
) -> Result<(), CliError> {
    writeln!(writer, "[{resource}]")?;
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
) -> Result<(), CliError> {
    match output {
        StreamOutput::Human => write_stream_human(writer, resource, &human()?),
        StreamOutput::Ndjson => write_snapshot_event(writer, resource, value),
    }
}

fn pipeline_summary_line(name: &str, status: &pipelines::Status) -> String {
    let rollout = status
        .rollout
        .as_ref()
        .map(|value| format!("{:?}", value.state))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "{name}: running={}/{} active_generation={} rollout={}",
        status.running_cores,
        status.total_cores,
        status
            .active_generation
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        rollout
    )
}

fn condition_line(condition: &pipelines::Condition) -> String {
    format!(
        "condition: kind={:?} status={:?} reason={} message={}",
        condition.kind,
        condition.status,
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

fn render_log_line(entry: &telemetry::LogEntry) -> String {
    format!(
        "{} [{}] {} {}",
        entry.timestamp, entry.level, entry.target, entry.rendered
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
