// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Azure Identity Auth Extension.

use super::config::AuthMethod;

/// Error definitions for Azure Identity Auth Extension.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error during configuration of a component.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication/authorization error.
    #[error("Auth error ({kind})")]
    Auth {
        /// The kind of authentication error.
        kind: AuthErrorKind,
        /// The underlying Azure error, if any.
        #[source]
        source: Option<azure_core::error::Error>,
    },
}

/// Specific authentication error variants.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthErrorKind {
    /// Failed to create the credential provider.
    CreateCredential {
        /// The authentication method that failed.
        method: AuthMethod,
    },

    /// Failed to acquire a token.
    TokenAcquisition,
}

impl std::fmt::Display for AuthErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthErrorKind::CreateCredential { method } => {
                write!(f, "failed to create credential for method: {}", method)
            }
            AuthErrorKind::TokenAcquisition => write!(f, "failed to acquire token"),
        }
    }
}

impl Error {
    /// Creates a new credential creation error.
    #[must_use]
    pub fn create_credential(method: AuthMethod, source: azure_core::error::Error) -> Self {
        Error::Auth {
            kind: AuthErrorKind::CreateCredential { method },
            source: Some(source),
        }
    }

    /// Creates a new token acquisition error.
    #[must_use]
    pub fn token_acquisition(source: azure_core::error::Error) -> Self {
        Error::Auth {
            kind: AuthErrorKind::TokenAcquisition,
            source: Some(source),
        }
    }
}
