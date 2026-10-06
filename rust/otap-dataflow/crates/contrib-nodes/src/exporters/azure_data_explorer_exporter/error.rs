// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Network failure category used in ADX diagnostics.
#[derive(Debug, Clone)]
pub enum NetworkErrorKind {
    /// Connection establishment failed.
    Connect,
    /// Request timed out.
    Timeout,
    /// Another request or response failure occurred.
    Other,
}

impl std::fmt::Display for NetworkErrorKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect => write!(formatter, "connect"),
            Self::Timeout => write!(formatter, "timeout"),
            Self::Other => write!(formatter, "other"),
        }
    }
}

/// Error definitions for the Azure Data Explorer exporter.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid endpoint URL (e.g. cluster URI cannot be used as a URL base, or
    /// contains characters that prevent construction of a valid URL).
    #[error("Invalid endpoint URL: {0}")]
    InvalidEndpoint(String),

    /// Failed to create HTTP client.
    #[error("Failed to create HTTP client")]
    CreateClient(#[source] reqwest::Error),

    /// Invalid HTTP header value.
    #[error("Invalid HTTP header")]
    InvalidHeader(#[source] http::header::InvalidHeaderValue),

    /// Network error during export.
    #[error("Network error ({kind})")]
    Network {
        /// The kind of network error.
        kind: NetworkErrorKind,
        /// The underlying reqwest error.
        #[source]
        source: reqwest::Error,
    },

    /// Rate limited (429).
    #[error("Rate limited: {body}", body = response_body_summary(body))]
    RateLimited {
        /// The response body.
        body: String,
        /// Server-specified minimum retry delay.
        retry_after: Option<std::time::Duration>,
    },

    /// Server error (5xx).
    #[error("Server error ({status}): {body}", body = response_body_summary(body))]
    ServerError {
        /// The HTTP status code.
        status: http::StatusCode,
        /// The response body.
        body: String,
        /// Server-specified minimum retry delay.
        retry_after: Option<std::time::Duration>,
    },

    /// Unexpected HTTP status.
    #[error(
        "Unexpected status ({status}): {body}",
        body = response_body_summary(body)
    )]
    UnexpectedStatus {
        /// The HTTP status code.
        status: http::StatusCode,
        /// The response body.
        body: String,
    },

    /// Export failed after retries.
    #[error("Export failed after {attempts} attempts: {last_error}")]
    ExportFailed {
        /// Number of attempts made.
        attempts: u32,
        /// The last error encountered.
        #[source]
        last_error: Box<Error>,
    },

    /// The configured operation deadline elapsed.
    #[error("ADX operation timed out after {timeout:?}")]
    OperationTimeout {
        /// Configured operation timeout.
        timeout: std::time::Duration,
    },

    /// Failed while reading an ADX HTTP response body.
    #[error("Failed to read ADX response body for status {status}")]
    ResponseBody {
        /// The HTTP status received before the body failed.
        status: http::StatusCode,
        /// The underlying response-body error.
        #[source]
        source: reqwest::Error,
    },

    /// Gzip compression failed.
    #[error("Gzip compression failed")]
    Compression(#[source] std::io::Error),

    /// A serialized row exceeded the configured safety limit.
    #[error("serialized ADX row is {actual} bytes, exceeding the {limit}-byte limit")]
    RowTooLarge {
        /// Serialized row size including the JSON Lines separator.
        actual: usize,
        /// Configured row-size limit.
        limit: usize,
    },

    /// A transformed request exceeded the configured byte limit.
    #[error("serialized ADX request is {actual} bytes, exceeding the {limit}-byte limit")]
    RequestTooLarge {
        /// Serialized JSON Lines request size.
        actual: usize,
        /// Configured request-size limit.
        limit: usize,
    },

    /// A transformed request exceeded the configured row limit.
    #[error("serialized ADX request has {actual} rows, exceeding the {limit}-row limit")]
    TooManyRows {
        /// Number of transformed rows.
        actual: usize,
        /// Configured row-count limit.
        limit: usize,
    },

    /// Metric data cannot be represented without silent data loss.
    #[error("invalid metric data: {reason}")]
    InvalidMetricData {
        /// Stable reason for refusing the source message.
        reason: &'static str,
    },

    /// Failed to create logs view.
    #[error("Failed to create logs view")]
    LogsViewCreationFailed {
        /// The underlying error.
        #[source]
        source: otel_arrow_dfe_pdata::error::Error,
    },

    /// Failed to create metrics view.
    #[error("Failed to create metrics view")]
    MetricsViewCreationFailed {
        /// The underlying error.
        #[source]
        source: otel_arrow_dfe_pdata::error::Error,
    },

    /// Failed to create traces view.
    #[error("Failed to create traces view")]
    TracesViewCreationFailed {
        /// The underlying error.
        #[source]
        source: otel_arrow_dfe_pdata::error::Error,
    },

    /// Channel receive error.
    #[error("Channel receive error")]
    ChannelRecv(#[source] otel_arrow_dfe_channel::error::RecvError),

    /// Client initialization failed.
    #[error("Client initialization failed")]
    ClientInit(#[source] Box<Error>),
}

/// Bound server-provided details when an error is explicitly formatted.
fn response_body_summary(body: &str) -> String {
    const MAX_RESPONSE_BODY_LENGTH: usize = 2048;
    let mut summary = body.replace(['\r', '\n', '\t'], " ");
    if summary.len() > MAX_RESPONSE_BODY_LENGTH {
        let mut end = MAX_RESPONSE_BODY_LENGTH;
        while !summary.is_char_boundary(end) {
            end -= 1;
        }
        summary.truncate(end);
        summary.push_str("...");
    }
    summary
}

impl Error {
    /// Return a log-safe summary without service response content.
    #[must_use]
    pub fn log_safe_summary(&self) -> String {
        match self {
            Self::RateLimited { .. } => "ADX returned HTTP 429 Too Many Requests".to_string(),
            Self::ServerError { status, .. } | Self::UnexpectedStatus { status, .. } => {
                format!("ADX returned HTTP {status}")
            }
            Self::ExportFailed {
                attempts,
                last_error,
            } => format!(
                "Export failed after {attempts} attempts: {}",
                last_error.log_safe_summary()
            ),
            _ => self.to_string(),
        }
    }

    /// Return whether another submission of the same payload may succeed.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network { .. } | Self::RateLimited { .. } | Self::ServerError { .. } => true,
            Self::UnexpectedStatus { status, .. } => *status == http::StatusCode::REQUEST_TIMEOUT,
            Self::ResponseBody { status, .. } => {
                *status == http::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
            }
            _ => false,
        }
    }

    /// Return whether the backend explicitly rejected this attempt.
    #[must_use]
    pub fn is_refusal(&self) -> bool {
        match self {
            Self::RateLimited { .. }
            | Self::RowTooLarge { .. }
            | Self::RequestTooLarge { .. }
            | Self::TooManyRows { .. }
            | Self::InvalidMetricData { .. }
            | Self::LogsViewCreationFailed { .. }
            | Self::MetricsViewCreationFailed { .. }
            | Self::TracesViewCreationFailed { .. } => true,
            Self::UnexpectedStatus { status, .. } => status.is_client_error(),
            Self::ResponseBody { status, .. } => status.is_client_error(),
            Self::ExportFailed { last_error, .. } => last_error.is_refusal(),
            _ => false,
        }
    }

    /// Return whether retrying the same request without changes cannot succeed.
    #[must_use]
    pub fn is_permanent_refusal(&self) -> bool {
        match self {
            Self::RowTooLarge { .. }
            | Self::RequestTooLarge { .. }
            | Self::TooManyRows { .. }
            | Self::InvalidMetricData { .. }
            | Self::LogsViewCreationFailed { .. }
            | Self::MetricsViewCreationFailed { .. }
            | Self::TracesViewCreationFailed { .. } => true,
            Self::UnexpectedStatus { status, .. } | Self::ResponseBody { status, .. } => {
                status.is_client_error()
                    && *status != http::StatusCode::UNAUTHORIZED
                    && *status != http::StatusCode::REQUEST_TIMEOUT
                    && *status != http::StatusCode::TOO_MANY_REQUESTS
            }
            Self::ExportFailed { last_error, .. } => last_error.is_permanent_refusal(),
            _ => false,
        }
    }

    /// Return the server-specified minimum retry delay, when present.
    #[must_use]
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::RateLimited { retry_after, .. } | Self::ServerError { retry_after, .. } => {
                *retry_after
            }
            _ => None,
        }
    }

    /// Whether this error indicates that ADX rejected the bearer token.
    #[must_use]
    pub fn is_unauthorized(&self) -> bool {
        match self {
            Self::UnexpectedStatus { status, .. } => *status == http::StatusCode::UNAUTHORIZED,
            Self::ResponseBody { status, .. } => *status == http::StatusCode::UNAUTHORIZED,
            Self::ExportFailed { last_error, .. } => last_error.is_unauthorized(),
            _ => false,
        }
    }

    /// Classify a reqwest error.
    #[must_use]
    pub fn from_reqwest(e: reqwest::Error) -> Self {
        let kind = if e.is_connect() {
            NetworkErrorKind::Connect
        } else if e.is_timeout() {
            NetworkErrorKind::Timeout
        } else {
            NetworkErrorKind::Other
        };
        Self::Network { kind, source: e }
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, response_body_summary};
    use http::StatusCode;

    /// Scenario: ADX rejects a request with HTTP 401.
    /// Guarantees: the exporter recognizes the rejection and can report that token generation upstream.
    #[test]
    fn unauthorized_status_is_classified_for_token_rejection() {
        let error = Error::UnexpectedStatus {
            status: StatusCode::UNAUTHORIZED,
            body: String::new(),
        };

        assert!(error.is_unauthorized());
    }

    /// Scenario: an HTTP 401 is wrapped after request retry processing.
    /// Guarantees: token rejection classification survives the wrapper.
    #[test]
    fn wrapped_unauthorized_status_is_classified_for_token_rejection() {
        let error = Error::ExportFailed {
            attempts: 1,
            last_error: Box::new(Error::UnexpectedStatus {
                status: StatusCode::UNAUTHORIZED,
                body: String::new(),
            }),
        };

        assert!(error.is_unauthorized());
    }

    /// Scenario: an ADX rejection body contains customer-derived content.
    /// Guarantees: warning-safe diagnostics omit response content.
    #[test]
    fn response_body_is_excluded_from_log_safe_summary() {
        let error = Error::UnexpectedStatus {
            status: StatusCode::FORBIDDEN,
            body: "sensitive-customer-value".to_string(),
        };

        let summary = error.log_safe_summary();
        assert_eq!(summary, "ADX returned HTTP 403 Forbidden");
        assert!(!summary.contains("sensitive-customer-value"));
    }

    /// Scenario: a long ADX response crosses the diagnostic byte limit within a UTF-8 character.
    /// Guarantees: response formatting truncates on a character boundary without panicking.
    #[test]
    fn response_body_summary_truncates_at_utf8_boundary() {
        let body = format!("{}\u{00e9}", "a".repeat(2047));

        let summary = response_body_summary(&body);

        assert_eq!(summary, format!("{}...", "a".repeat(2047)));
    }

    /// Scenario: ADX returns terminal and transient client statuses.
    /// Guarantees: only requests that cannot succeed unchanged are permanent refusals.
    #[test]
    fn permanent_refusal_classification_distinguishes_retryable_statuses() {
        for status in [
            StatusCode::BAD_REQUEST,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::CONFLICT,
            StatusCode::PAYLOAD_TOO_LARGE,
        ] {
            assert!(
                Error::UnexpectedStatus {
                    status,
                    body: String::new()
                }
                .is_permanent_refusal()
            );
        }
        for status in [
            StatusCode::UNAUTHORIZED,
            StatusCode::REQUEST_TIMEOUT,
            StatusCode::TOO_MANY_REQUESTS,
        ] {
            assert!(
                !Error::UnexpectedStatus {
                    status,
                    body: String::new()
                }
                .is_permanent_refusal()
            );
        }
        assert!(
            Error::UnexpectedStatus {
                status: StatusCode::REQUEST_TIMEOUT,
                body: String::new()
            }
            .is_retryable()
        );
    }
}
