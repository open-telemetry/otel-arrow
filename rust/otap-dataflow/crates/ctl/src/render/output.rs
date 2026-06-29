// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Writer helpers for machine and human CLI output modes.

use crate::args::{BundleOutput, MutationOutput, ReadOutput, StreamOutput};
use crate::branding;
use crate::error::CliError;
use crate::style::HumanStyle;
use otap_df_admin_api::telemetry;
use serde::Serialize;
use serde_json::json;
use std::io::Write;

/// Serializes a finite read-only response in JSON, YAML, or agent JSON form.
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
        ReadOutput::AgentJson => {
            serde_json::to_writer_pretty(&mut *writer, &agent_envelope("snapshot", None, value))
                .map_err(io_serialize_error)?;
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

/// Serializes a finite mutation response with its operation outcome.
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
            let schema_version = branding::active().schema_version;
            serde_json::to_writer(
                &mut *writer,
                &json!({
                    "schemaVersion": schema_version,
                    "type": "snapshot",
                    "event": "snapshot",
                    "resource": "mutation",
                    "outcome": outcome,
                    "data": value
                }),
            )
            .map_err(io_serialize_error)?;
            writeln!(writer)?;
        }
        MutationOutput::AgentJson => {
            serde_json::to_writer_pretty(
                &mut *writer,
                &agent_mutation_envelope(outcome, "mutation", value),
            )
            .map_err(io_serialize_error)?;
            writeln!(writer)?;
        }
    }
    writer.flush()?;
    Ok(())
}

/// Writes an arbitrary value in the stable agent JSON envelope.
pub fn write_agent_output<T: Serialize>(
    writer: &mut dyn Write,
    kind: &str,
    resource: Option<&str>,
    value: &T,
) -> Result<(), CliError> {
    serde_json::to_writer_pretty(&mut *writer, &agent_envelope(kind, resource, value))
        .map_err(io_serialize_error)?;
    writeln!(writer)?;
    writer.flush()?;
    Ok(())
}

/// Serializes a support bundle in one of the bundle-supported machine formats.
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
        BundleOutput::AgentJson => {
            serde_json::to_writer_pretty(&mut *writer, &agent_envelope("bundle", None, value))
                .map_err(io_serialize_error)?;
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

/// Writes one NDJSON snapshot event for a streamed resource.
pub fn write_snapshot_event<T: Serialize>(
    writer: &mut dyn Write,
    resource: &str,
    value: &T,
) -> Result<(), CliError> {
    serde_json::to_writer(
        &mut *writer,
        &stream_envelope("snapshot", "snapshot", resource, value),
    )
    .map_err(io_serialize_error)?;
    writeln!(writer)?;
    writer.flush()?;
    Ok(())
}

/// Writes one retained-log entry as an NDJSON log event.
pub fn write_log_event(
    writer: &mut dyn Write,
    entry: &telemetry::LogEntry,
) -> Result<(), CliError> {
    let schema_version = branding::active().schema_version;
    serde_json::to_writer(
        &mut *writer,
        &json!({
            "schemaVersion": schema_version,
            "type": "log",
            "event": "log",
            "resource": "telemetry_logs",
            "data": entry,
            "log": entry
        }),
    )
    .map_err(io_serialize_error)?;
    writeln!(writer)?;
    writer.flush()?;
    Ok(())
}

/// Writes one generic NDJSON event envelope for watch output.
pub fn write_event_output<T: Serialize>(
    writer: &mut dyn Write,
    resource: &str,
    value: &T,
) -> Result<(), CliError> {
    serde_json::to_writer(
        &mut *writer,
        &stream_envelope("event", resource, resource, value),
    )
    .map_err(io_serialize_error)?;
    writeln!(writer)?;
    writer.flush()?;
    Ok(())
}

/// Writes finite human-rendered command output and flushes the writer.
pub fn write_human(writer: &mut dyn Write, content: &str) -> Result<(), CliError> {
    writeln!(writer, "{content}")?;
    writer.flush()?;
    Ok(())
}

/// Writes one human-readable watch update with a resource header.
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

/// Writes a watch snapshot in the selected stream output mode.
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

/// Converts serializer failures into the CLI's configuration-style error.
pub(super) fn io_serialize_error(error: impl std::fmt::Display) -> CliError {
    CliError::config(format!("failed to serialize output: {error}"))
}

fn agent_envelope<T: Serialize>(
    kind: &str,
    resource: Option<&str>,
    value: &T,
) -> serde_json::Value {
    json!({
        "schemaVersion": branding::active().schema_version,
        "type": kind,
        "resource": resource,
        "generatedAt": humantime::format_rfc3339_seconds(std::time::SystemTime::now()).to_string(),
        "data": value
    })
}

fn agent_mutation_envelope<T: Serialize>(
    outcome: &str,
    resource: &str,
    value: &T,
) -> serde_json::Value {
    json!({
        "schemaVersion": branding::active().schema_version,
        "type": "mutation",
        "resource": resource,
        "generatedAt": humantime::format_rfc3339_seconds(std::time::SystemTime::now()).to_string(),
        "outcome": outcome,
        "data": value
    })
}

fn stream_envelope<T: Serialize>(
    kind: &str,
    event: &str,
    resource: &str,
    value: &T,
) -> serde_json::Value {
    json!({
        "schemaVersion": branding::active().schema_version,
        "type": kind,
        "event": event,
        "resource": resource,
        "data": value
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::MutationOutput;
    use serde_json::Value;

    /// Parses the single-line NDJSON written to `buf` into a JSON value.
    fn parse_line(buf: Vec<u8>) -> Value {
        let text = String::from_utf8(buf).expect("utf8 output");
        serde_json::from_str(text.trim()).expect("valid JSON line")
    }

    /// Scenario: NDJSON mutation output carries the default schema version.
    /// Guarantees: the mutation envelope stamps the active branding's
    /// `schemaVersion`, which is `dfctl/v1` when no branding is installed.
    #[test]
    fn mutation_ndjson_uses_default_schema_version() {
        let mut buf = Vec::new();
        write_mutation_output(
            &mut buf,
            MutationOutput::Ndjson,
            "succeeded",
            &json!({"k": "v"}),
        )
        .expect("write mutation ndjson");
        let value = parse_line(buf);
        assert_eq!(value["schemaVersion"], "dfctl/v1");
        assert_eq!(value["resource"], "mutation");
    }

    /// Scenario: agent-json mutation output carries the default schema version.
    /// Guarantees: `agent_mutation_envelope` stamps the active branding's
    /// `schemaVersion`.
    #[test]
    fn mutation_agent_json_uses_default_schema_version() {
        let mut buf = Vec::new();
        write_mutation_output(
            &mut buf,
            MutationOutput::AgentJson,
            "succeeded",
            &json!({"k": "v"}),
        )
        .expect("write mutation agent-json");
        let value = parse_line(buf);
        assert_eq!(value["schemaVersion"], "dfctl/v1");
        assert_eq!(value["type"], "mutation");
    }

    /// Scenario: NDJSON stream event output carries the default schema version.
    /// Guarantees: `stream_envelope` stamps the active branding's
    /// `schemaVersion`.
    #[test]
    fn event_output_uses_default_schema_version() {
        let mut buf = Vec::new();
        write_event_output(&mut buf, "snapshot", &json!({"k": "v"})).expect("write event");
        let value = parse_line(buf);
        assert_eq!(value["schemaVersion"], "dfctl/v1");
        assert_eq!(value["resource"], "snapshot");
    }

    /// Scenario: NDJSON log event output carries the default schema version.
    /// Guarantees: `write_log_event` stamps the active branding's
    /// `schemaVersion`.
    #[test]
    fn log_event_uses_default_schema_version() {
        let entry = telemetry::LogEntry {
            seq: 1,
            timestamp: "1970-01-01T00:00:00Z".to_string(),
            level: "INFO".to_string(),
            target: "test".to_string(),
            event_name: "event".to_string(),
            file: None,
            line: None,
            rendered: "hello".to_string(),
            contexts: Vec::new(),
        };
        let mut buf = Vec::new();
        write_log_event(&mut buf, &entry).expect("write log event");
        let value = parse_line(buf);
        assert_eq!(value["schemaVersion"], "dfctl/v1");
        assert_eq!(value["type"], "log");
    }
}
