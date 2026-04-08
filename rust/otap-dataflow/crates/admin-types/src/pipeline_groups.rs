// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared pipeline-group-scoped admin models.

use crate::pipelines::Status as PipelineStatus;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Pipeline-group status response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// RFC 3339 timestamp when the status snapshot was generated.
    pub generated_at: String,
    /// Pipeline statuses keyed by `pipeline_group_id:pipeline_id`.
    pub pipelines: BTreeMap<String, PipelineStatus>,
}

/// Shutdown endpoint response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownResponse {
    /// Result status.
    pub status: ShutdownStatus,
    /// Error messages, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    /// Duration in milliseconds, if reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Shutdown result status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownStatus {
    /// Request accepted asynchronously.
    Accepted,
    /// Shutdown completed successfully.
    Completed,
    /// Shutdown failed.
    Failed,
    /// Shutdown timed out.
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::{Value, json};

    fn assert_roundtrip<T>(value: Value)
    where
        T: DeserializeOwned + Serialize,
    {
        let parsed: T = serde_json::from_value(value.clone()).expect("fixture should deserialize");
        let serialized = serde_json::to_value(parsed).expect("model should serialize");
        assert_eq!(serialized, value);
    }

    #[test]
    fn status_roundtrips_current_wire_shape() {
        assert_roundtrip::<Status>(json!({
            "generatedAt": "2026-01-01T00:00:00Z",
            "pipelines": {
                "default:main": {
                    "conditions": [],
                    "totalCores": 1,
                    "runningCores": 1,
                    "cores": {}
                }
            }
        }));
    }

    #[test]
    fn shutdown_response_roundtrips_current_wire_shape() {
        assert_roundtrip::<ShutdownResponse>(json!({
            "status": "timeout",
            "errors": ["timed out"],
            "durationMs": 1200
        }));
    }
}
