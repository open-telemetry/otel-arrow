// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::config::AuthMethod;
use http::StatusCode;
use http::header::InvalidHeaderValue;

/// Error definitions for azure monitor exporter.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // ==================== Configuration Errors ====================
    /// Error during configuration of a component.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Error due to duplicate columns in schema mapping.
    #[error("Configuration error: duplicate columns found: {columns:?}")]
    ConfigDuplicateColumns {
        /// The duplicate column names.
        columns: Vec<String>,
    },

    // ==================== Authentication Errors ====================
    /// Authentication/authorization error.
    #[error("Auth error ({kind})")]
    Auth {
        /// The kind of authentication error.
        kind: AuthErrorKind,
        /// The underlying Azure error, if any.
        #[source]
        source: Option<azure_core::error::Error>,
        /// Response body for HTTP auth errors (401/403).
        body: Option<String>,
    },

    // ==================== HTTP/Network Errors ====================
    /// Failed to create HTTP client.
    #[error("Failed to create HTTP client")]
    CreateClient(#[source] reqwest::Error),

    /// Invalid HTTP header.
    #[error("Invalid HTTP header")]
    InvalidHeader(#[source] InvalidHeaderValue),

    /// Network error during export.
    #[error("Network error ({kind})")]
    Network {
        /// The kind of network error.
        kind: NetworkErrorKind,
        /// The underlying reqwest error.
        #[source]
        source: reqwest::Error,
    },

    // ==================== Server Response Errors ====================
    /// Payload too large (413).
    #[error("Payload too large")]
    PayloadTooLarge,

    /// Rate limited (429).
    #[error("Rate limited")]
    RateLimited {
        /// The response body.
        body: String,
        /// Server-specified retry delay.
        retry_after: Option<std::time::Duration>,
    },

    /// Server error (5xx).
    #[error("Server error ({status})")]
    ServerError {
        /// The HTTP status code.
        status: StatusCode,
        /// The response body.
        body: String,
        /// Server-specified retry delay.
        retry_after: Option<std::time::Duration>,
    },

    /// Unexpected HTTP status.
    #[error("Unexpected status ({status})")]
    UnexpectedStatus {
        /// The HTTP status code.
        status: StatusCode,
        /// The response body.
        body: String,
    },

    // ==================== Export Errors ====================
    /// Export failed after retries.
    #[error("Export failed after {attempts} attempts")]
    ExportFailed {
        /// Number of attempts made.
        attempts: u32,
        /// The last error encountered.
        #[source]
        last_error: Box<Error>,
    },

    // ==================== Internal Errors ====================
    /// Log entry too large to export.
    #[error("Log entry too large to export")]
    LogEntryTooLarge,

    /// Failed to add log entry to batch.
    #[error("Failed to add log entry to batch")]
    BatchPushFailed(#[source] std::io::Error),

    /// Failed to finalize batch.
    #[error("Failed to finalize batch")]
    BatchFinalizeFailed(#[source] std::io::Error),

    /// Failed to create logs view.
    #[error("Failed to create logs view")]
    LogsViewCreationFailed {
        /// The underlying error.
        #[source]
        source: otap_df_pdata::error::Error,
    },

    /// Channel receive error.
    #[error("Channel receive error")]
    ChannelRecv(#[source] otap_df_channel::error::RecvError),

    /// Failed to create auth handler.
    #[error("Failed to create auth handler")]
    AuthHandlerCreation(#[source] Box<Error>),

    /// Client pool initialization failed.
    #[error("Client pool initialization failed")]
    ClientPoolInit(#[source] Box<Error>),

    // ==================== Transformer Errors ====================
    /// Unknown log record field encountered during transformation.
    #[error("Unknown log record field: {field}")]
    UnknownLogRecordField {
        /// The name of the unrecognized field.
        field: String,
    },

    /// Field mapping value is not a string.
    #[error("Field mapping for '{field}' must be a string")]
    InvalidFieldMapping {
        /// The name of the field with invalid mapping.
        field: String,
    },
}

/// Authentication error classification.
#[derive(Debug, Clone)]
pub enum AuthErrorKind {
    /// Failed to create credential (during setup).
    CreateCredential { method: AuthMethod },
    /// Failed to acquire token.
    TokenAcquisition,
    /// Token refresh failed during retry.
    TokenRefresh,
    /// Server returned 401.
    Unauthorized,
    /// Server returned 403.
    Forbidden,
}

impl std::fmt::Display for AuthErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateCredential { method } => write!(f, "create credential: {method:?}"),
            Self::TokenAcquisition => write!(f, "token acquisition"),
            Self::TokenRefresh => write!(f, "token refresh"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Forbidden => write!(f, "forbidden"),
        }
    }
}

/// Network error classification.
#[derive(Debug, Clone, Copy)]
pub enum NetworkErrorKind {
    Timeout,
    Connect,
    Request,
    Body,
    Unknown,
}

impl std::fmt::Display for NetworkErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "timeout"),
            Self::Connect => write!(f, "connect"),
            Self::Request => write!(f, "request"),
            Self::Body => write!(f, "body"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl Error {
    /// Returns true if this error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Network { .. }
                | Error::Auth {
                    kind: AuthErrorKind::Unauthorized,
                    ..
                }
                | Error::RateLimited { .. }
                | Error::ServerError { .. }
        )
    }

    /// Returns the retry-after duration if specified by the server.
    #[must_use]
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Error::RateLimited { retry_after, .. } => *retry_after,
            Error::ServerError { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

// Convenience constructors
impl Error {
    /// Creates a network error from a reqwest error, classifying the error kind.
    #[must_use]
    pub fn network(source: reqwest::Error) -> Self {
        let kind = if source.is_timeout() {
            NetworkErrorKind::Timeout
        } else if source.is_connect() {
            NetworkErrorKind::Connect
        } else if source.is_request() {
            NetworkErrorKind::Request
        } else if source.is_body() {
            NetworkErrorKind::Body
        } else {
            NetworkErrorKind::Unknown
        };

        Self::Network { kind, source }
    }

    /// Creates a credential creation error.
    #[must_use]
    pub fn create_credential(method: AuthMethod, source: azure_core::error::Error) -> Self {
        Self::Auth {
            kind: AuthErrorKind::CreateCredential { method },
            source: Some(source),
            body: None,
        }
    }

    /// Creates a token acquisition error.
    #[must_use]
    pub fn token_acquisition(source: azure_core::error::Error) -> Self {
        Self::Auth {
            kind: AuthErrorKind::TokenAcquisition,
            source: Some(source),
            body: None,
        }
    }

    /// Creates a token refresh error.
    #[must_use]
    pub fn token_refresh(source: Error) -> Self {
        // Unwrap the inner auth error source if possible
        let inner_source = match source {
            Error::Auth { source, .. } => source,
            _ => None,
        };

        Self::Auth {
            kind: AuthErrorKind::TokenRefresh,
            source: inner_source,
            body: None,
        }
    }

    /// Creates an unauthorized (401) error.
    #[must_use]
    pub fn unauthorized(body: String) -> Self {
        Self::Auth {
            kind: AuthErrorKind::Unauthorized,
            source: None,
            body: Some(body),
        }
    }

    /// Creates a forbidden (403) error.
    #[must_use]
    pub fn forbidden(body: String) -> Self {
        Self::Auth {
            kind: AuthErrorKind::Forbidden,
            source: None,
            body: Some(body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    // ==================== Configuration Error Tests ====================

    #[test]
    fn test_config_error_message() {
        let error = Error::Config("invalid endpoint URL".to_string());
        assert_eq!(
            error.to_string(),
            "Configuration error: invalid endpoint URL"
        );
        assert!(error.source().is_none());
    }

    #[test]
    fn test_config_duplicate_columns_single() {
        let error = Error::ConfigDuplicateColumns {
            columns: vec!["timestamp".to_string()],
        };
        assert_eq!(
            error.to_string(),
            r#"Configuration error: duplicate columns found: ["timestamp"]"#
        );
    }

    #[test]
    fn test_config_duplicate_columns_multiple() {
        let error = Error::ConfigDuplicateColumns {
            columns: vec![
                "timestamp".to_string(),
                "severity".to_string(),
                "message".to_string(),
            ],
        };
        assert_eq!(
            error.to_string(),
            r#"Configuration error: duplicate columns found: ["timestamp", "severity", "message"]"#
        );
    }

    // ==================== Auth Error Tests ====================

    #[test]
    fn test_auth_create_credential_message() {
        let azure_error = azure_core::error::Error::with_message(
            azure_core::error::ErrorKind::Credential,
            "managed identity not available",
        );
        let error = Error::create_credential(AuthMethod::ManagedIdentity, azure_error);
        assert_eq!(
            error.to_string(),
            "Auth error (create credential: ManagedIdentity)"
        );
        assert!(error.source().is_some());
    }

    #[test]
    fn test_auth_token_acquisition_message() {
        let azure_error = azure_core::error::Error::with_message(
            azure_core::error::ErrorKind::Credential,
            "token expired",
        );
        let error = Error::token_acquisition(azure_error);
        assert_eq!(error.to_string(), "Auth error (token acquisition)");
        assert!(error.source().is_some());
    }

    #[test]
    fn test_auth_unauthorized_message() {
        let error = Error::unauthorized("invalid token".to_string());
        assert_eq!(error.to_string(), "Auth error (unauthorized)");
        assert!(error.source().is_none());
    }

    #[test]
    fn test_auth_forbidden_message() {
        let error = Error::forbidden("insufficient permissions".to_string());
        assert_eq!(error.to_string(), "Auth error (forbidden)");
        assert!(error.source().is_none());
    }

    #[test]
    fn test_auth_token_refresh_message() {
        let inner = Error::token_acquisition(azure_core::error::Error::with_message(
            azure_core::error::ErrorKind::Credential,
            "refresh failed",
        ));
        let error = Error::token_refresh(inner);
        assert_eq!(error.to_string(), "Auth error (token refresh)");
    }

    // ==================== Server Response Error Tests ====================

    #[test]
    fn test_payload_too_large_message() {
        let error = Error::PayloadTooLarge;
        assert_eq!(error.to_string(), "Payload too large");
    }

    #[test]
    fn test_rate_limited_message() {
        let error = Error::RateLimited {
            body: "too many requests".to_string(),
            retry_after: Some(std::time::Duration::from_secs(30)),
        };
        assert_eq!(error.to_string(), "Rate limited");
        assert_eq!(
            error.retry_after(),
            Some(std::time::Duration::from_secs(30))
        );
    }

    #[test]
    fn test_server_error_message() {
        let error = Error::ServerError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: "internal error".to_string(),
            retry_after: None,
        };
        assert_eq!(
            error.to_string(),
            "Server error (500 Internal Server Error)"
        );
    }

    #[test]
    fn test_unexpected_status_message() {
        let error = Error::UnexpectedStatus {
            status: StatusCode::IM_A_TEAPOT,
            body: "I'm a teapot".to_string(),
        };
        // Note: http crate canonical_reason() returns "I'm a teapot" (lowercase)
        assert_eq!(error.to_string(), "Unexpected status (418 I'm a teapot)");
    }

    // ==================== Export Error Tests ====================

    #[test]
    fn test_export_failed_message() {
        let inner = Error::PayloadTooLarge;
        let error = Error::ExportFailed {
            attempts: 5,
            last_error: Box::new(inner),
        };
        assert_eq!(error.to_string(), "Export failed after 5 attempts");
        assert!(error.source().is_some());
    }

    // ==================== Retryable Tests ====================

    #[test]
    fn test_is_retryable() {
        // Retryable
        assert!(Error::unauthorized(String::new()).is_retryable());
        assert!(
            Error::RateLimited {
                body: String::new(),
                retry_after: None
            }
            .is_retryable()
        );
        assert!(
            Error::ServerError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: String::new(),
                retry_after: None
            }
            .is_retryable()
        );

        // Not retryable
        assert!(!Error::forbidden(String::new()).is_retryable());
        assert!(!Error::PayloadTooLarge.is_retryable());
        assert!(
            !Error::UnexpectedStatus {
                status: StatusCode::BAD_REQUEST,
                body: String::new()
            }
            .is_retryable()
        );
        assert!(
            !Error::token_acquisition(azure_core::error::Error::with_message(
                azure_core::error::ErrorKind::Credential,
                "test"
            ))
            .is_retryable()
        );
    }

    // ==================== Display Tests ====================

    #[test]
    fn test_network_error_kind_display() {
        assert_eq!(NetworkErrorKind::Timeout.to_string(), "timeout");
        assert_eq!(NetworkErrorKind::Connect.to_string(), "connect");
        assert_eq!(NetworkErrorKind::Request.to_string(), "request");
        assert_eq!(NetworkErrorKind::Body.to_string(), "body");
        assert_eq!(NetworkErrorKind::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_auth_error_kind_display() {
        assert_eq!(
            AuthErrorKind::CreateCredential {
                method: AuthMethod::ManagedIdentity
            }
            .to_string(),
            "create credential: ManagedIdentity"
        );
        assert_eq!(
            AuthErrorKind::TokenAcquisition.to_string(),
            "token acquisition"
        );
        assert_eq!(AuthErrorKind::TokenRefresh.to_string(), "token refresh");
        assert_eq!(AuthErrorKind::Unauthorized.to_string(), "unauthorized");
        assert_eq!(AuthErrorKind::Forbidden.to_string(), "forbidden");
    }

    // ==================== Trait Tests ====================

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn test_error_implements_std_error() {
        fn assert_std_error<T: StdError>() {}
        assert_std_error::<Error>();
    }

    // ==================== Internal Error Tests ====================

    #[test]
    fn test_log_entry_too_large_message() {
        let error = Error::LogEntryTooLarge;
        assert_eq!(error.to_string(), "Log entry too large to export");
    }

    #[test]
    fn test_batch_push_failed_message() {
        let io_error = std::io::Error::other("write failed");
        let error = Error::BatchPushFailed(io_error);
        assert_eq!(error.to_string(), "Failed to add log entry to batch");
        assert!(error.source().is_some());
    }

    #[test]
    fn test_batch_finalize_failed_message() {
        let io_error = std::io::Error::other("flush failed");
        let error = Error::BatchFinalizeFailed(io_error);
        assert_eq!(error.to_string(), "Failed to finalize batch");
        assert!(error.source().is_some());
    }

    #[test]
    fn test_logs_view_creation_failed_message() {
        let error = Error::LogsViewCreationFailed {
            source: otap_df_pdata::error::Error::ColumnNotFound {
                name: "test_column".to_string(),
            },
        };
        assert_eq!(error.to_string(), "Failed to create logs view");
        assert!(error.source().is_some());
    }

    #[test]
    fn test_channel_recv_message() {
        let recv_error = otap_df_channel::error::RecvError::Closed;
        let error = Error::ChannelRecv(recv_error);
        assert_eq!(error.to_string(), "Channel receive error");
        assert!(error.source().is_some());
    }

    #[test]
    fn test_auth_handler_creation_message() {
        let inner = Error::Config("test".to_string());
        let error = Error::AuthHandlerCreation(Box::new(inner));
        assert_eq!(error.to_string(), "Failed to create auth handler");
        assert!(error.source().is_some());
    }

    #[test]
    fn test_client_pool_init_message() {
        let inner = Error::Config("test".to_string());
        let error = Error::ClientPoolInit(Box::new(inner));
        assert_eq!(error.to_string(), "Client pool initialization failed");
        assert!(error.source().is_some());
    }

    // ==================== HTTP/Network Error Tests ====================

    #[test]
    fn test_invalid_header_message() {
        // Create an invalid header value error
        let invalid_value = http::header::HeaderValue::from_bytes(b"\x00invalid").unwrap_err();
        let error = Error::InvalidHeader(invalid_value);
        assert_eq!(error.to_string(), "Invalid HTTP header");
        assert!(error.source().is_some());
    }

    // ==================== Transformer Error Tests ====================

    #[test]
    fn test_unknown_log_record_field_message() {
        let error = Error::UnknownLogRecordField {
            field: "invalid_field".to_string(),
        };
        assert_eq!(error.to_string(), "Unknown log record field: invalid_field");
    }

    #[test]
    fn test_invalid_field_mapping_message() {
        let error = Error::InvalidFieldMapping {
            field: "body".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Field mapping for 'body' must be a string"
        );
    }
}
