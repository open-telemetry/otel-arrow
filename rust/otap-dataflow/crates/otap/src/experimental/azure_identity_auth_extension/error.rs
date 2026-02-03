// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Azure Identity Auth Extension.

use super::config::AuthMethod;

/// Error definitions for Azure Identity Auth Extension.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // ==================== Configuration Errors ====================
    /// Error during configuration of a component.
    #[error("Configuration error: {0}")]
    Config(String),

    // ==================== Authentication Errors ====================
    /// Authentication/authorization error.
    #[error("Auth error ({kind})")]
    Auth {
        /// The kind of authentication error.
        kind: AuthErrorKind,
        /// The underlying Azure error, if any.
        #[source]
        source: Option<azure_core::error::Error>,
    },

    // ==================== Internal Errors ====================
    /// Shutdown requested.
    #[error("Shutdown requested: {reason}")]
    Shutdown {
        /// The reason for shutdown.
        reason: String,
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

    /// Token has expired and refresh failed.
    TokenExpired,
}

impl std::fmt::Display for AuthErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthErrorKind::CreateCredential { method } => {
                write!(f, "failed to create credential for method: {}", method)
            }
            AuthErrorKind::TokenAcquisition => write!(f, "failed to acquire token"),
            AuthErrorKind::TokenExpired => write!(f, "token expired and refresh failed"),
        }
    }
}

impl Error {
    /// Creates a new credential creation error.
    pub fn create_credential(method: AuthMethod, source: azure_core::error::Error) -> Self {
        Error::Auth {
            kind: AuthErrorKind::CreateCredential { method },
            source: Some(source),
        }
    }

    /// Creates a new token acquisition error.
    pub fn token_acquisition(source: azure_core::error::Error) -> Self {
        Error::Auth {
            kind: AuthErrorKind::TokenAcquisition,
            source: Some(source),
        }
    }

    /// Creates a new token expired error.
    pub fn token_expired() -> Self {
        Error::Auth {
            kind: AuthErrorKind::TokenExpired,
            source: None,
        }
    }

    /// Creates a new shutdown error.
    pub fn shutdown(reason: impl Into<String>) -> Self {
        Error::Shutdown {
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = Error::Config("test error".to_string());
        assert_eq!(format!("{}", err), "Configuration error: test error");
    }

    #[test]
    fn test_auth_error_kind_display() {
        let kind = AuthErrorKind::CreateCredential {
            method: AuthMethod::ManagedIdentity,
        };
        assert!(format!("{}", kind).contains("managed_identity"));

        let kind = AuthErrorKind::TokenAcquisition;
        assert_eq!(format!("{}", kind), "failed to acquire token");

        let kind = AuthErrorKind::TokenExpired;
        assert_eq!(format!("{}", kind), "token expired and refresh failed");
    }

    #[test]
    fn test_shutdown_error() {
        let err = Error::shutdown("test reason");
        match err {
            Error::Shutdown { reason } => assert_eq!(reason, "test reason"),
            _ => panic!("Expected Shutdown error"),
        }
    }
}
