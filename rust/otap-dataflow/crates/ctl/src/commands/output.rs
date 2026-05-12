// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Command-facing output policy for dfctl command runners.
//!
//! `commands/*` modules decide what data to fetch or mutate. `render/output.rs`
//! knows how to serialize concrete JSON/YAML/NDJSON/human envelopes. This
//! module sits between those layers and gives command runners a small, named API
//! for the output contracts they need.
//!
//! Output taxonomy:
//!
//! - read snapshots: one-time status/config/diagnostic data, emitted directly
//!   for JSON/YAML or wrapped as an agent `snapshot`;
//! - mutations: one-time operation responses that carry an explicit outcome
//!   such as `accepted`, `completed`, `failed`, or `preflight_only`;
//! - watch streams: ongoing human lines or NDJSON events, never pretty JSON/YAML;
//! - group shutdown: a legacy group-wide mutation surface with its own CLI enum;
//! - support bundles: structured troubleshooting payloads that may be written to
//!   stdout or a private file.
//!
//! Human output is deliberately supplied as a closure. This keeps the common
//! path cheap for JSON/YAML/NDJSON/agent output, where a command should not
//! spend time building a human table that will never be printed.

use crate::args::{
    BundleOutput, GroupShutdownOutput, MetricsShape, MutationOutput, ReadOutput, StreamOutput,
};
use crate::error::CliError;
use crate::render::{
    write_agent_output as render_agent_output, write_bundle_output as render_bundle_output,
    write_event_output as render_event_output, write_human as render_human_output,
    write_mutation_output as render_mutation_output, write_read_output as render_read_output,
};
use crate::troubleshoot::{BundleMetadata, BundleMetricsShape};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

// Command response writers.

/// Writes the response for a read-only snapshot command.
///
/// Read commands return the admin SDK value directly for `json` and `yaml`.
/// `agent-json` wraps the same value in the stable dfctl agent envelope with a
/// `snapshot` type. `human` delegates formatting to the command-specific
/// closure because each resource needs a different table or summary layout.
pub(crate) fn write_read_command_output<T: Serialize>(
    stdout: &mut dyn Write,
    output: ReadOutput,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        ReadOutput::Human => render_human_output(stdout, &human()?),
        ReadOutput::Json | ReadOutput::Yaml | ReadOutput::AgentJson => {
            render_read_output(stdout, output, value)
        }
    }
}

/// Writes the response for a mutation command.
///
/// Mutations differ from reads because callers need to know whether the
/// accepted operation ultimately succeeded, timed out, or failed. Machine
/// output therefore includes the supplied `outcome` next to the command data.
/// For `json` and `yaml` this is a `{ outcome, data }` object; for `agent-json`
/// it is the dfctl mutation envelope; for `ndjson` it is a single stream-style
/// snapshot event. Human output is still command-specific and generated lazily.
pub(crate) fn write_mutation_command_output<T: Serialize>(
    stdout: &mut dyn Write,
    output: MutationOutput,
    outcome: &str,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        MutationOutput::Human => render_human_output(stdout, &human()?),
        MutationOutput::Json
        | MutationOutput::Yaml
        | MutationOutput::Ndjson
        | MutationOutput::AgentJson => render_mutation_output(stdout, output, outcome, value),
    }
}

/// Writes the response for the group-wide shutdown command.
///
/// Group shutdown currently has its own CLI output enum because it predates the
/// shared mutation output path and still has group-specific compatibility
/// rules. The emitted shape intentionally mirrors mutation output:
/// human output is delegated, JSON/YAML serialize the raw response or preflight
/// report, NDJSON emits one snapshot-like event, and agent JSON uses the dfctl
/// envelope with `group_shutdown` as the resource.
pub(crate) fn write_group_shutdown_command_output<T: Serialize>(
    stdout: &mut dyn Write,
    output: GroupShutdownOutput,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        GroupShutdownOutput::Human => render_human_output(stdout, &human()?),
        GroupShutdownOutput::Json => render_read_output(stdout, ReadOutput::Json, value),
        GroupShutdownOutput::Yaml => render_read_output(stdout, ReadOutput::Yaml, value),
        GroupShutdownOutput::Ndjson => render_event_output(stdout, "snapshot", value),
        GroupShutdownOutput::AgentJson => {
            render_agent_output(stdout, "mutation", Some("group_shutdown"), value)
        }
    }
}

// Support bundle writing.

/// Writes support-bundle output either to stdout or to an explicit file path.
///
/// Passing no path, or `-`, writes to stdout. Passing a real path creates or
/// truncates that file before serialization. On Unix, files are created with
/// `0600` permissions because support bundles may contain logs, endpoint names,
/// or other operational details that should not become world-readable.
pub(crate) fn write_support_bundle_output<T: Serialize>(
    stdout: &mut dyn Write,
    output: BundleOutput,
    path: Option<&Path>,
    value: &T,
) -> Result<(), CliError> {
    match path {
        Some(path) if path != Path::new("-") => {
            let mut file = create_private_bundle_file(path).map_err(|err| {
                CliError::config(format!(
                    "failed to create bundle output file '{}': {err}",
                    path.display()
                ))
            })?;
            render_bundle_output(&mut file, output, value)
        }
        _ => render_bundle_output(stdout, output, value),
    }
}

/// Creates a private file for support-bundle output on Unix platforms.
#[cfg(unix)]
fn create_private_bundle_file(path: &Path) -> Result<fs::File, std::io::Error> {
    fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)
}

/// Creates a bundle file on non-Unix platforms.
///
/// The standard library does not expose a portable mode API, so this falls back
/// to the platform default permissions.
#[cfg(not(unix))]
fn create_private_bundle_file(path: &Path) -> Result<fs::File, std::io::Error> {
    fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
}

// Watch output validation and conversion.

/// Rejects invalid mutation output combinations before command execution.
///
/// One-shot mutations may emit human, JSON, YAML, or agent JSON. Watch mode is
/// an ongoing stream, so it may only emit human updates or NDJSON events.
/// Conversely, NDJSON without `--watch` would imply a stream where no stream
/// exists, so it is rejected as invalid usage.
pub(crate) fn validate_mutation_output_mode(
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

/// Rejects invalid output combinations for group shutdown.
///
/// This mirrors `validate_mutation_output_mode` but accepts the group-shutdown
/// enum used by that command's CLI surface. Keeping this separate avoids
/// accidental enum conversions in command code while preserving the same
/// user-facing rules.
pub(crate) fn validate_group_shutdown_output_mode(
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

/// Converts mutation output to stream output after watch compatibility checks.
///
/// Watch implementations only know about `StreamOutput`, so mutation commands
/// call this after validation to bridge from the command-specific output enum to
/// the stream renderer enum.
pub(crate) fn mutation_output_to_stream_output(
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

/// Converts group shutdown output to stream output after compatibility checks.
///
/// This is the group-shutdown equivalent of `mutation_output_to_stream_output`.
pub(crate) fn group_shutdown_output_to_stream_output(
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

// Admin operation option helpers.

/// Normalizes CLI wait options to the admin SDK operation contract.
///
/// The CLI accepts a `Duration`, while the admin API currently expects a
/// whole-second timeout. This helper keeps that conversion consistent across
/// rollout, reconfiguration, shutdown, and group shutdown commands.
pub(crate) fn build_operation_options(wait: bool, wait_timeout: Duration) -> OperationOptions {
    OperationOptions {
        wait,
        timeout_secs: duration_to_admin_timeout_secs(wait_timeout),
    }
}

use otap_df_admin_api::operations::OperationOptions;

/// Converts a duration to the whole-second timeout accepted by the admin API.
///
/// Partial seconds are rounded up so the effective timeout is never shorter
/// than the user's requested duration. The result is clamped to at least one
/// second because a zero-second wait timeout would make long-running operations
/// fail immediately in surprising ways.
pub(crate) fn duration_to_admin_timeout_secs(duration: Duration) -> u64 {
    let secs = duration.as_secs();
    if duration.subsec_nanos() == 0 {
        secs
    } else {
        secs.saturating_add(1)
    }
    .max(1)
}

/// Builds the metadata block embedded in support bundles.
///
/// Support bundles combine status, logs, metrics, and diagnoses collected by
/// the CLI. The metadata records when collection happened and which collection
/// limits or shapes were used so a later reader can interpret the evidence.
pub(crate) fn build_bundle_metadata(logs_limit: usize, shape: MetricsShape) -> BundleMetadata {
    BundleMetadata {
        collected_at: humantime::format_rfc3339_seconds(SystemTime::now()).to_string(),
        logs_limit,
        metrics_shape: match shape {
            MetricsShape::Compact => BundleMetricsShape::Compact,
            MetricsShape::Full => BundleMetricsShape::Full,
        },
    }
}

// Stream helper.

/// Writes one human-formatted stream line.
///
/// This is a small adapter used by watch paths that already assembled their
/// human text and only need the shared flushing/error behavior from
/// `write_human`.
pub(crate) fn write_human_stream_line(
    stdout: &mut dyn Write,
    content: &str,
) -> Result<(), CliError> {
    render_human_output(stdout, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::cell::Cell;

    /// Scenario: a read command emits machine-readable JSON.
    /// Guarantees: machine output serializes the data directly and does not pay
    /// the cost of building human output.
    #[test]
    fn read_json_output_does_not_render_human_text() {
        let value = json!({ "status": "ok" });
        let mut stdout = Vec::new();

        write_read_command_output(&mut stdout, ReadOutput::Json, &value, || {
            panic!("human renderer should not run for JSON output")
        })
        .expect("write read JSON output");

        let output: Value = serde_json::from_slice(&stdout).expect("valid JSON");
        assert_eq!(output, value);
    }

    /// Scenario: a read command emits human-readable output.
    /// Guarantees: human mode delegates rendering to the command-specific
    /// closure and writes the returned text.
    #[test]
    fn read_human_output_renders_human_text() {
        let rendered = Cell::new(false);
        let mut stdout = Vec::new();

        write_read_command_output(
            &mut stdout,
            ReadOutput::Human,
            &json!({ "ignored": true }),
            || {
                rendered.set(true);
                Ok("human table".to_string())
            },
        )
        .expect("write read human output");

        assert!(rendered.get());
        assert_eq!(
            String::from_utf8(stdout).expect("utf8 output"),
            "human table\n"
        );
    }

    /// Scenario: a mutation command emits one-shot JSON output.
    /// Guarantees: machine output includes both the operation outcome and the
    /// command data payload.
    #[test]
    fn mutation_json_output_includes_outcome_and_data() {
        let value = json!({ "rolloutId": "rollout-1" });
        let mut stdout = Vec::new();

        write_mutation_command_output(
            &mut stdout,
            MutationOutput::Json,
            "accepted",
            &value,
            || panic!("human renderer should not run for JSON output"),
        )
        .expect("write mutation JSON output");

        let output: Value = serde_json::from_slice(&stdout).expect("valid JSON");
        assert_eq!(output["outcome"], "accepted");
        assert_eq!(output["data"], value);
    }

    /// Scenario: CLI durations are converted for admin operation requests.
    /// Guarantees: partial seconds round up and zero durations become a minimum
    /// one-second timeout.
    #[test]
    fn admin_timeout_seconds_round_up_and_clamp_to_one() {
        assert_eq!(duration_to_admin_timeout_secs(Duration::ZERO), 1);
        assert_eq!(duration_to_admin_timeout_secs(Duration::from_millis(1)), 1);
        assert_eq!(duration_to_admin_timeout_secs(Duration::from_secs(2)), 2);
        assert_eq!(
            duration_to_admin_timeout_secs(Duration::from_millis(2500)),
            3
        );
    }
}
