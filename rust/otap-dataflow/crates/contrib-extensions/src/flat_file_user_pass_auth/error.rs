// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the flat file user pass extension.

use std::path::PathBuf;

use crate::common::secret_file::ReadSecretFileError;
use crate::common::user_pass_file::ReadUserPassError;

/// Errors raised while building the token client or acquiring tokens.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reading a credential file (`*_file` config field) failed.
    #[error("failed to read credential file {}: {source}", .path.display())]
    ReadCredentialFile {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// Acquiring a credential failed.
    #[error("credential acquisition failed: {message}")]
    CredentialAcquisition {
        /// Human-readable cause reported.
        message: String,
    },
}

impl From<ReadUserPassError> for Error {
    fn from(error: ReadUserPassError) -> Self {
        match error {
            ReadUserPassError::ReadFile {
                path,
                source: ReadSecretFileError::Read(source),
                ..
            } => Self::ReadCredentialFile { path, source },
            ReadUserPassError::ReadFile {
                field,
                source: ReadSecretFileError::InvalidUtf8,
                ..
            } => Self::CredentialAcquisition {
                message: format!("`{field}` does not contain valid UTF-8"),
            },
            ReadUserPassError::MissingValue { .. } => Self::CredentialAcquisition {
                message: error.to_string(),
            },
        }
    }
}
