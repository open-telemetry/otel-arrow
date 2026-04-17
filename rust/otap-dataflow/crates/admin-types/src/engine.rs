// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared engine-scoped admin models.

use crate::pipelines::{Condition, Status as PipelineStatus};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Global status response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// RFC 3339 timestamp when the status snapshot was generated.
    pub generated_at: String,
    /// Pipeline statuses keyed by `pipeline_group_id:pipeline_id`.
    pub pipelines: BTreeMap<String, PipelineStatus>,
}

/// Health probe response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResponse {
    /// Probe name.
    pub probe: ProbeKind,
    /// Probe status.
    pub status: ProbeStatus,
    /// RFC 3339 timestamp when the probe result was generated.
    pub generated_at: String,
    /// Informational probe message, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Failing pipeline conditions, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failing: Vec<PipelineConditionFailure>,
}

/// Global probe kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeKind {
    /// Liveness probe.
    Livez,
    /// Readiness probe.
    Readyz,
}

/// Global probe status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeStatus {
    /// Probe succeeded.
    Ok,
    /// Probe failed.
    Failed,
}

/// Failing pipeline condition in a probe response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineConditionFailure {
    /// Pipeline identifier formatted as `pipeline_group_id:pipeline_id`.
    pub pipeline: String,
    /// Failing condition.
    pub condition: Condition,
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
    fn probe_response_roundtrips_current_wire_shape() {
        assert_roundtrip::<ProbeResponse>(json!({
            "probe": "readyz",
            "status": "failed",
            "generatedAt": "2026-01-01T00:00:00Z",
            "message": "process memory pressure at hard limit"
        }));
    }

    #[test]
    fn probe_response_roundtrips_failure_details_wire_shape() {
        assert_roundtrip::<ProbeResponse>(json!({
            "probe": "readyz",
            "status": "failed",
            "generatedAt": "2026-01-01T00:00:00Z",
            "failing": [
                {
                    "pipeline": "default:main",
                    "condition": {
                        "type": "Ready",
                        "status": "False",
                        "reason": "QuorumNotMet",
                        "message": "not ready"
                    }
                }
            ]
        }));
    }
}
