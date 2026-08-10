// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared engine-scoped admin models.

use crate::pipelines::{Condition, RolloutStatus, ShutdownStatus, Status as PipelineStatus};
use otap_df_config::engine::OtelDataflowSpec;
use otap_df_config::{PipelineGroupId, PipelineId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const fn default_reconcile_timeout_secs() -> u64 {
    60
}

/// Global status response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// RFC 3339 timestamp when the status snapshot was generated.
    pub generated_at: String,
    /// Pipeline statuses keyed by `pipeline_group_id:pipeline_id`.
    pub pipelines: BTreeMap<String, PipelineStatus>,
}

/// Desired full engine configuration and timing options for reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfigReconcileRequest {
    /// Desired engine configuration.
    pub config: OtelDataflowSpec,
    /// Per-core admission/ready timeout in seconds for created or replaced pipelines.
    #[serde(default = "default_reconcile_timeout_secs")]
    pub step_timeout_secs: u64,
    /// Graceful drain timeout in seconds for replaced or resized pipelines.
    #[serde(default = "default_reconcile_timeout_secs")]
    pub drain_timeout_secs: u64,
    /// Graceful drain timeout in seconds for pipelines deleted by reconciliation.
    #[serde(default = "default_reconcile_timeout_secs")]
    pub delete_timeout_secs: u64,
    /// Deletes live pipelines and groups that are omitted from the desired config.
    #[serde(default = "default_delete_missing")]
    pub delete_missing: bool,
}

const fn default_delete_missing() -> bool {
    true
}

/// Full-config reconciliation status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfigReconcileStatus {
    /// Controller-assigned reconciliation identifier.
    pub reconcile_id: String,
    /// Overall reconciliation lifecycle state.
    pub state: EngineConfigReconcileState,
    /// Stable hash of the desired config body, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_hash: Option<String>,
    /// RFC3339 timestamp for reconciliation start.
    pub started_at: String,
    /// RFC3339 timestamp for the latest state transition.
    pub updated_at: String,
    /// Per-resource changes applied or considered by reconciliation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<ConfigChangeStatus>,
    /// Optional failure reason when reconciliation does not complete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

impl EngineConfigReconcileStatus {
    /// Creates a new reconciliation status.
    #[must_use]
    pub fn new(
        reconcile_id: String,
        state: EngineConfigReconcileState,
        config_hash: Option<String>,
        timestamp: String,
    ) -> Self {
        Self {
            reconcile_id,
            state,
            config_hash,
            started_at: timestamp.clone(),
            updated_at: timestamp,
            changes: Vec::new(),
            failure_reason: None,
        }
    }
}

/// Overall full-config reconciliation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineConfigReconcileState {
    /// Reconciliation was accepted but has not started.
    Pending,
    /// Reconciliation is running.
    Running,
    /// Reconciliation completed successfully.
    Succeeded,
    /// Reconciliation failed.
    Failed,
}

/// Action selected for one desired-state resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigChangeAction {
    /// Create a missing pipeline.
    Create,
    /// Replace an existing pipeline runtime shape or configuration.
    Replace,
    /// Resize an existing pipeline.
    Resize,
    /// Delete a live pipeline or group omitted from the desired config.
    Delete,
    /// No runtime mutation was required.
    Noop,
    /// The requested change is outside the current live-reconfiguration boundary.
    Unsupported,
}

/// Status for one resource touched by full-config reconciliation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigChangeStatus {
    /// Logical group id when the change is group- or pipeline-scoped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_group_id: Option<PipelineGroupId>,
    /// Logical pipeline id when the change is pipeline-scoped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_id: Option<PipelineId>,
    /// Selected action.
    pub action: ConfigChangeAction,
    /// Current or terminal state for this resource change.
    pub state: String,
    /// Rollout status when this change was applied through a rollout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollout: Option<RolloutStatus>,
    /// Shutdown status when this change required draining a pipeline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutdown: Option<ShutdownStatus>,
    /// Optional human-readable detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Result of deleting one logical pipeline from the controller state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDeleteStatus {
    /// Logical target pipeline group id.
    pub pipeline_group_id: PipelineGroupId,
    /// Logical target pipeline id.
    pub pipeline_id: PipelineId,
    /// Terminal delete state.
    pub state: String,
    /// RFC3339 timestamp when deletion started.
    pub started_at: String,
    /// RFC3339 timestamp when deletion finished or last changed.
    pub updated_at: String,
    /// Shutdown status when deletion drained a live runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutdown: Option<ShutdownStatus>,
    /// Optional failure reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

/// Result of deleting one logical pipeline group from the controller state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupDeleteStatus {
    /// Logical target pipeline group id.
    pub pipeline_group_id: PipelineGroupId,
    /// Terminal delete state.
    pub state: String,
    /// RFC3339 timestamp when deletion started.
    pub started_at: String,
    /// RFC3339 timestamp when deletion finished or last changed.
    pub updated_at: String,
    /// Per-pipeline delete results.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pipelines: Vec<PipelineDeleteStatus>,
    /// Optional failure reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
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
