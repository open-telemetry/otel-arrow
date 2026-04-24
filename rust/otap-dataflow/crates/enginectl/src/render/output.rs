// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Writer helpers for machine and human CLI output modes.

use crate::args::{BundleOutput, MutationOutput, ReadOutput, StreamOutput};
use crate::error::CliError;
use crate::style::HumanStyle;
use otap_df_admin_api::telemetry;
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

pub(super) fn io_serialize_error(error: impl std::fmt::Display) -> CliError {
    CliError::config(format!("failed to serialize output: {error}"))
}
