// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for flat file SASL credential acquisition.

use std::path::PathBuf;

/// Errors raised while acquiring credentials, without credential contents.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reading a credential file failed.
    #[error("failed to read `{field}` {}: {source}", .path.display())]
    ReadCredentialFile {
        /// Configuration field selecting the file.
        field: &'static str,
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The acquired credential could not be decoded or validated.
    #[error(
        "invalid credential from `{field}`{path_context}: {message}",
        path_context = .path.as_ref().map(|path| format!(" {}", path.display())).unwrap_or_default()
    )]
    CredentialAcquisition {
        /// Configuration field selecting the credential.
        field: &'static str,
        /// Path containing the invalid credential, absent for an inline username.
        path: Option<PathBuf>,
        /// Encoding or validation failure, without the secret value.
        message: String,
    },
}
