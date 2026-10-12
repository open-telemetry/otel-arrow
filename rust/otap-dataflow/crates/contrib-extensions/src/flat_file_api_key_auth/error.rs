// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the flat file API key authentication extension.

use std::path::PathBuf;

/// Errors raised while acquiring API keys.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reading a credential file failed.
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
        /// Human-readable cause.
        message: String,
    },
}
