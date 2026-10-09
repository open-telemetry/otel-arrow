// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for flat file SASL credential acquisition.

use std::path::PathBuf;

/// Errors raised while acquiring credentials, without credential contents.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reading the password file failed.
    #[error("failed to read `password_secret_file` {}: {source}", .path.display())]
    ReadCredentialFile {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The acquired credential could not be decoded or validated.
    #[error("invalid credential from `password_secret_file` {}: {message}", .path.display())]
    CredentialAcquisition {
        /// Path containing the invalid credential.
        path: PathBuf,
        /// Encoding or validation failure, without the secret value.
        message: String,
    },
}
