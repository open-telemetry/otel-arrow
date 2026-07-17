// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Azure Identity Auth extension.

use super::config::AuthMethod;

/// Errors raised while constructing credentials or acquiring tokens.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Constructing the Azure credential failed.
    #[error("failed to create {method} credential: {source}")]
    CreateCredential {
        /// Authentication method that failed to construct.
        method: AuthMethod,
        /// Underlying Azure SDK error.
        source: azure_core::Error,
    },

    /// Acquiring a token from the credential failed.
    #[error("token acquisition failed: {source}")]
    TokenAcquisition {
        /// Underlying Azure SDK error.
        source: azure_core::Error,
    },
}
