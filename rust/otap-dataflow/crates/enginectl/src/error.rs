// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! CLI error model, exit-code mapping, and machine-readable error rendering.
//!
//! `dfctl` needs predictable failure behavior for humans, shell scripts, and AI
//! agents. This module normalizes IO errors, admin SDK failures, invalid usage,
//! missing resources, and operation outcomes into stable exit codes plus either
//! human text or JSON error envelopes.

use crate::args::ErrorFormat;
use otap_df_admin_api::operations::OperationErrorKind;
use serde::Serialize;
use std::io::{self, Write};
use thiserror::Error;

/// Runtime error type used by CLI command execution.
///
/// The variants deliberately carry enough information to map failures to
/// stable process exit codes and machine-readable error kinds.
#[derive(Debug, Error)]
pub enum CliError {
    #[error("{message}")]
    Message {
        exit_code: u8,
        message: String,
        print: bool,
    },

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Admin(#[from] otap_df_admin_api::Error),
}

impl CliError {
    /// Returns the POSIX-style process exit code associated with this failure.
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Message { exit_code, .. } => *exit_code,
            CliError::Io(_) => 6,
            CliError::Admin(error) => map_admin_error(error),
        }
    }

    /// Returns true when the CLI should render the error on stderr.
    pub fn should_print(&self) -> bool {
        match self {
            CliError::Message { print, .. } => *print,
            CliError::Io(_) | CliError::Admin(_) => true,
        }
    }

    /// Builds a configuration or local environment failure.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Message {
            exit_code: 6,
            message: message.into(),
            print: true,
        }
    }

    /// Builds a command usage failure that should exit with code 2.
    pub fn invalid_usage(message: impl Into<String>) -> Self {
        Self::Message {
            exit_code: 2,
            message: message.into(),
            print: true,
        }
    }

    /// Builds a missing-resource failure that should exit with code 3.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::Message {
            exit_code: 3,
            message: message.into(),
            print: true,
        }
    }

    /// Builds a terminal operation failure that should exit with code 5.
    pub fn outcome_failure(message: impl Into<String>) -> Self {
        Self::Message {
            exit_code: 5,
            message: message.into(),
            print: true,
        }
    }

    /// Writes this error using the selected human, JSON, or agent JSON format.
    pub fn write_to(&self, writer: &mut dyn Write, format: ErrorFormat) -> io::Result<()> {
        match format {
            ErrorFormat::Text => writeln!(writer, "error: {self}"),
            ErrorFormat::Json => write_json_line(writer, &self.report()),
            ErrorFormat::AgentJson => write_json_line(writer, &self.agent_report()),
        }
    }

    fn report(&self) -> ErrorReport {
        ErrorReport {
            kind: self.kind(),
            exit_code: self.exit_code(),
            message: self.to_string(),
        }
    }

    fn agent_report(&self) -> AgentErrorReport {
        AgentErrorReport {
            schema_version: "dfctl/v1",
            kind: "error",
            generated_at: humantime::format_rfc3339_seconds(std::time::SystemTime::now())
                .to_string(),
            error: self.report(),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            CliError::Message { exit_code, .. } => match *exit_code {
                2 => "invalid_usage",
                3 => "not_found",
                4 => "invalid_request",
                5 => "operation_failed",
                _ => "configuration",
            },
            CliError::Io(_) => "io",
            CliError::Admin(error) => admin_error_kind(error),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorReport {
    kind: &'static str,
    exit_code: u8,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentErrorReport {
    schema_version: &'static str,
    #[serde(rename = "type")]
    kind: &'static str,
    generated_at: String,
    error: ErrorReport,
}

fn write_json_line<T: Serialize>(writer: &mut dyn Write, value: &T) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value).map_err(io::Error::other)?;
    writeln!(writer)
}

fn map_admin_error(error: &otap_df_admin_api::Error) -> u8 {
    match error {
        otap_df_admin_api::Error::AdminOperation { error, .. } => match error.kind {
            OperationErrorKind::GroupNotFound
            | OperationErrorKind::PipelineNotFound
            | OperationErrorKind::RolloutNotFound
            | OperationErrorKind::ShutdownNotFound => 3,
            OperationErrorKind::Conflict | OperationErrorKind::InvalidRequest => 4,
            OperationErrorKind::Internal => 6,
        },
        _ => 6,
    }
}

fn admin_error_kind(error: &otap_df_admin_api::Error) -> &'static str {
    match error {
        otap_df_admin_api::Error::AdminOperation { error, .. } => match error.kind {
            OperationErrorKind::GroupNotFound
            | OperationErrorKind::PipelineNotFound
            | OperationErrorKind::RolloutNotFound
            | OperationErrorKind::ShutdownNotFound => "not_found",
            OperationErrorKind::Conflict => "conflict",
            OperationErrorKind::InvalidRequest => "invalid_request",
            OperationErrorKind::Internal => "internal",
        },
        otap_df_admin_api::Error::ClientConfig { .. } => "client_config",
        otap_df_admin_api::Error::Transport { .. } => "transport",
        otap_df_admin_api::Error::Decode { .. } => "decode",
        otap_df_admin_api::Error::RemoteStatus { .. } => "remote_status",
        otap_df_admin_api::Error::Endpoint(_) => "endpoint",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a runtime failure is rendered with `--error-format json`.
    /// Guarantees: scripts can parse a compact structured error object with a
    /// stable kind and exit code.
    #[test]
    fn json_error_format_contains_kind_and_exit_code() {
        let error = CliError::not_found("pipeline was not found");
        let mut output = Vec::new();

        error
            .write_to(&mut output, ErrorFormat::Json)
            .expect("write error");

        let value: serde_json::Value = serde_json::from_slice(&output).expect("json");
        assert_eq!(value["kind"], "not_found");
        assert_eq!(value["exitCode"], 3);
        assert_eq!(value["message"], "pipeline was not found");
    }

    /// Scenario: a long-running operation reaches a terminal failure state.
    /// Guarantees: the CLI now emits stderr context instead of only returning a
    /// non-zero exit code.
    #[test]
    fn operation_failures_are_printable() {
        let error = CliError::outcome_failure("rollout failed");

        assert!(error.should_print());
        assert_eq!(error.exit_code(), 5);
    }
}
