// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared operation options for long-running admin actions.

use serde::{Deserialize, Serialize};

/// Wait behavior for long-running admin operations such as reconfigure and shutdown.
///
/// By default operations are asynchronous: the SDK returns as soon as the
/// request has been accepted for background execution or has already completed.
/// Set `wait = true` to wait up to `timeout_secs` for a terminal result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationOptions {
    /// Whether the SDK should wait for the operation to reach a terminal result.
    #[serde(default)]
    pub wait: bool,
    /// Maximum number of seconds to wait when `wait` is `true`.
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for OperationOptions {
    fn default() -> Self {
        Self {
            wait: false,
            timeout_secs: default_timeout_secs(),
        }
    }
}

const fn default_timeout_secs() -> u64 {
    60
}

impl OperationOptions {
    /// Converts these options into URL query pairs for SDK transports.
    ///
    /// Most callers do not need this directly because the built-in HTTP
    /// transport uses it automatically.
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("wait", self.wait.to_string()),
            ("timeout_secs", self.timeout_secs.to_string()),
        ]
    }
}

/// Typed request rejection for live admin operations.
///
/// This is returned when the server refuses to start the requested operation at
/// all. It is different from an accepted operation that later reports
/// `Failed(...)` or `TimedOut(...)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationError {
    /// Machine-readable rejection kind.
    pub kind: OperationErrorKind,
    /// Optional human-readable detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl OperationError {
    /// Creates a typed operation rejection without a human-readable message.
    #[must_use]
    pub const fn new(kind: OperationErrorKind) -> Self {
        Self {
            kind,
            message: None,
        }
    }

    /// Attaches a human-readable detail message to the rejection.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// Machine-readable rejection kinds for live admin operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationErrorKind {
    /// The requested pipeline group does not exist.
    GroupNotFound,
    /// The requested pipeline does not exist.
    PipelineNotFound,
    /// The requested rollout does not exist.
    RolloutNotFound,
    /// The requested shutdown does not exist.
    ShutdownNotFound,
    /// Another incompatible live operation is active in the server's consistency scope.
    Conflict,
    /// The request was rejected as invalid.
    InvalidRequest,
    /// The server failed while processing the request.
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: the server returns a structured admin operation rejection in
    /// the shared public wire format.
    /// Guarantees: the SDK-owned `OperationError` model round-trips without
    /// renaming fields or changing enum values.
    #[test]
    fn operation_error_roundtrips() {
        let value = json!({
            "kind": "invalid_request",
            "message": "core allocation change is not supported"
        });
        let parsed: OperationError =
            serde_json::from_value(value.clone()).expect("fixture should deserialize");
        assert_eq!(
            serde_json::to_value(parsed).expect("model should serialize"),
            value
        );
    }
}
