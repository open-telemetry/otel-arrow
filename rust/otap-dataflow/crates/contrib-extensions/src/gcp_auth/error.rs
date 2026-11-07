// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Google Cloud (GCP) Auth extension.

/// Errors raised while building the metadata credential or acquiring tokens.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Building the GCP metadata credential failed.
    #[error("failed to build the GCP metadata credential: {reason}")]
    BuildCredentials {
        /// Human-readable cause.
        reason: String,
    },

    /// Acquiring a token from the GCE/GKE metadata server failed.
    #[error("GCP metadata token acquisition failed: {message}")]
    TokenAcquisition {
        /// Human-readable cause reported by `google-cloud-auth`.
        message: String,
    },
}
