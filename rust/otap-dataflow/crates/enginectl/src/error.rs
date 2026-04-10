// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otap_df_admin_api::operations::OperationErrorKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{message}")]
    Message {
        exit_code: u8,
        message: String,
        print: bool,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Admin(#[from] otap_df_admin_api::Error),
}

impl CliError {
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Message { exit_code, .. } => *exit_code,
            CliError::Io(_) => 6,
            CliError::Admin(error) => map_admin_error(error),
        }
    }

    pub fn should_print(&self) -> bool {
        match self {
            CliError::Message { print, .. } => *print,
            CliError::Io(_) | CliError::Admin(_) => true,
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Message {
            exit_code: 6,
            message: message.into(),
            print: true,
        }
    }

    pub fn invalid_usage(message: impl Into<String>) -> Self {
        Self::Message {
            exit_code: 2,
            message: message.into(),
            print: true,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::Message {
            exit_code: 3,
            message: message.into(),
            print: true,
        }
    }

    pub fn outcome_failure(message: impl Into<String>) -> Self {
        Self::Message {
            exit_code: 5,
            message: message.into(),
            print: false,
        }
    }
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
