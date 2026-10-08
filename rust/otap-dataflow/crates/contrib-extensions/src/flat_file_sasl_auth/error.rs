// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for file-backed SASL acquisition.

use std::path::PathBuf;

use crate::common::secret_file::ReadSecretFileError;
use crate::common::user_pass_file::ReadUserPassError;

/// Errors that never include credential contents.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reading a configured credential file failed.
    #[error("failed to read `{field}` at {}: {source}", .path.display())]
    ReadCredentialFile {
        /// Configuration field identifying the failed read.
        field: &'static str,
        /// Configured path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A configured file contains invalid UTF-8.
    #[error("`{field}` at {} does not contain valid UTF-8", .path.display())]
    InvalidUtf8 {
        /// Configuration field identifying the invalid file.
        field: &'static str,
        /// Configured path.
        path: PathBuf,
    },
    /// Credential values failed SASL validation.
    #[error("credential acquisition failed: {message}")]
    CredentialAcquisition {
        /// Validation failure, without credential values.
        message: String,
    },
}

impl From<ReadUserPassError> for Error {
    fn from(error: ReadUserPassError) -> Self {
        match error {
            ReadUserPassError::ReadFile {
                field,
                path,
                source,
            } => match source {
                ReadSecretFileError::Read(source) => Self::ReadCredentialFile {
                    field,
                    path,
                    source,
                },
                ReadSecretFileError::InvalidUtf8 => Self::InvalidUtf8 { field, path },
            },
            ReadUserPassError::MissingValue { .. } => Self::CredentialAcquisition {
                message: error.to_string(),
            },
        }
    }
}
