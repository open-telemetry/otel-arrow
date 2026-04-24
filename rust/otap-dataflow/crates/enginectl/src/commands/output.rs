// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared output validation and serialization helpers for command runners.

use crate::args::{
    BundleOutput, GroupShutdownOutput, MetricsShape, MutationOutput, ReadOutput, StreamOutput,
};
use crate::error::CliError;
use crate::render::{
    write_agent_output, write_bundle_output, write_event_output, write_human,
    write_mutation_output, write_read_output,
};
use crate::troubleshoot::{BundleMetadata, BundleMetricsShape};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

/// Emit read-style command output, delegating human rendering to the caller.
pub(crate) fn emit_read<T: Serialize>(
    stdout: &mut dyn Write,
    output: ReadOutput,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        ReadOutput::Human => write_human(stdout, &human()?),
        ReadOutput::Json | ReadOutput::Yaml | ReadOutput::AgentJson => {
            write_read_output(stdout, output, value)
        }
    }
}

/// Emit mutation output while preserving the existing JSON/YAML/NDJSON wrappers.
pub(crate) fn emit_mutation<T: Serialize>(
    stdout: &mut dyn Write,
    output: MutationOutput,
    outcome: &str,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        MutationOutput::Human => write_human(stdout, &human()?),
        MutationOutput::Json
        | MutationOutput::Yaml
        | MutationOutput::Ndjson
        | MutationOutput::AgentJson => write_mutation_output(stdout, output, outcome, value),
    }
}

/// Groups shutdown is the only mutation flow with a separate output enum today.
pub(crate) fn emit_group_shutdown<T: Serialize>(
    stdout: &mut dyn Write,
    output: GroupShutdownOutput,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        GroupShutdownOutput::Human => write_human(stdout, &human()?),
        GroupShutdownOutput::Json => write_read_output(stdout, ReadOutput::Json, value),
        GroupShutdownOutput::Yaml => write_read_output(stdout, ReadOutput::Yaml, value),
        GroupShutdownOutput::Ndjson => write_event_output(stdout, "snapshot", value),
        GroupShutdownOutput::AgentJson => {
            write_agent_output(stdout, "mutation", Some("group_shutdown"), value)
        }
    }
}

/// Write bundle output either to stdout or to an explicit file path.
pub(crate) fn write_bundle<T: Serialize>(
    stdout: &mut dyn Write,
    output: BundleOutput,
    path: Option<&Path>,
    value: &T,
) -> Result<(), CliError> {
    match path {
        Some(path) if path != Path::new("-") => {
            let mut file = create_private_file(path).map_err(|err| {
                CliError::config(format!(
                    "failed to create bundle output file '{}': {err}",
                    path.display()
                ))
            })?;
            write_bundle_output(&mut file, output, value)
        }
        _ => write_bundle_output(stdout, output, value),
    }
}

#[cfg(unix)]
fn create_private_file(path: &Path) -> Result<fs::File, std::io::Error> {
    fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn create_private_file(path: &Path) -> Result<fs::File, std::io::Error> {
    fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
}

/// Reject invalid output combinations before command execution starts.
pub(crate) fn validate_mutation_output(
    output: MutationOutput,
    watch: bool,
) -> Result<(), CliError> {
    if watch
        && matches!(
            output,
            MutationOutput::Json | MutationOutput::Yaml | MutationOutput::AgentJson
        )
    {
        return Err(CliError::invalid_usage(
            "--watch requires --output human or --output ndjson",
        ));
    }
    if !watch && matches!(output, MutationOutput::Ndjson) {
        return Err(CliError::invalid_usage("--output ndjson requires --watch"));
    }
    Ok(())
}

/// Reject invalid output combinations for the client-side groups shutdown watch.
pub(crate) fn validate_group_shutdown_output(
    output: GroupShutdownOutput,
    watch: bool,
) -> Result<(), CliError> {
    if watch
        && matches!(
            output,
            GroupShutdownOutput::Json | GroupShutdownOutput::Yaml | GroupShutdownOutput::AgentJson
        )
    {
        return Err(CliError::invalid_usage(
            "--watch requires --output human or --output ndjson",
        ));
    }
    if !watch && matches!(output, GroupShutdownOutput::Ndjson) {
        return Err(CliError::invalid_usage("--output ndjson requires --watch"));
    }
    Ok(())
}

/// Convert mutation output to stream output once watch compatibility is known.
pub(crate) fn stream_output_from_mutation(
    output: MutationOutput,
) -> Result<StreamOutput, CliError> {
    match output {
        MutationOutput::Human => Ok(StreamOutput::Human),
        MutationOutput::Ndjson => Ok(StreamOutput::Ndjson),
        MutationOutput::Json | MutationOutput::Yaml | MutationOutput::AgentJson => Err(
            CliError::invalid_usage("--watch requires --output human or --output ndjson"),
        ),
    }
}

/// Convert group shutdown output to stream output once watch compatibility is known.
pub(crate) fn group_shutdown_stream_output(
    output: GroupShutdownOutput,
) -> Result<StreamOutput, CliError> {
    match output {
        GroupShutdownOutput::Human => Ok(StreamOutput::Human),
        GroupShutdownOutput::Ndjson => Ok(StreamOutput::Ndjson),
        GroupShutdownOutput::Json | GroupShutdownOutput::Yaml | GroupShutdownOutput::AgentJson => {
            Err(CliError::invalid_usage(
                "--watch requires --output human or --output ndjson",
            ))
        }
    }
}

/// Normalize admin operation wait options to the current wire contract.
pub(crate) fn operation_options(wait: bool, wait_timeout: Duration) -> OperationOptions {
    OperationOptions {
        wait,
        timeout_secs: duration_to_secs_ceil(wait_timeout),
    }
}

use otap_df_admin_api::operations::OperationOptions;

/// The admin API accepts whole-second timeout values, so callers round up.
pub(crate) fn duration_to_secs_ceil(duration: Duration) -> u64 {
    let secs = duration.as_secs();
    if duration.subsec_nanos() == 0 {
        secs
    } else {
        secs.saturating_add(1)
    }
    .max(1)
}

/// Bundle metadata is computed client side at collection time.
pub(crate) fn bundle_metadata(logs_limit: usize, shape: MetricsShape) -> BundleMetadata {
    BundleMetadata {
        collected_at: humantime::format_rfc3339_seconds(SystemTime::now()).to_string(),
        logs_limit,
        metrics_shape: match shape {
            MetricsShape::Compact => BundleMetricsShape::Compact,
            MetricsShape::Full => BundleMetricsShape::Full,
        },
    }
}

/// Helper for human stream output used by event/log streaming paths.
pub(crate) fn write_stream_line(stdout: &mut dyn Write, content: &str) -> Result<(), CliError> {
    write_human(stdout, content)
}
