// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the OAuth 2.0 Client Auth extension.

use std::path::PathBuf;

/// Errors raised while building the token client or acquiring tokens.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Building the TLS-configured HTTP client for the token endpoint failed.
    #[error("failed to build the token HTTP client: {reason}")]
    BuildHttpClient {
        /// Human-readable cause.
        reason: String,
    },

    /// Reading a credential file (`*_file` config field) failed.
    #[error("failed to read credential file {}: {source}", .path.display())]
    ReadCredentialFile {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// Signing the JWT-bearer assertion failed.
    #[error("failed to sign JWT assertion: {message}")]
    JwtSigning {
        /// Human-readable cause.
        message: String,
    },

    /// Acquiring a token from the token endpoint failed.
    #[error("token acquisition failed: {message}")]
    TokenAcquisition {
        /// Human-readable cause reported by the token endpoint or client.
        message: String,
    },
}
