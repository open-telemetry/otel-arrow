// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared pipeline-scoped admin models.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;

/// Pipeline status across all cores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Aggregated pipeline conditions.
    pub conditions: Vec<Condition>,
    /// Total observed cores for the pipeline.
    pub total_cores: usize,
    /// Number of currently running cores.
    pub running_cores: usize,
    /// Per-core details.
    pub cores: BTreeMap<usize, CoreStatus>,
}

/// Per-core pipeline status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStatus {
    /// Current core phase.
    pub phase: Phase,
    /// Last observed heartbeat time as RFC 3339.
    pub last_heartbeat_time: String,
    /// Per-core conditions.
    pub conditions: Vec<Condition>,
    /// Whether graceful delete is pending.
    pub delete_pending: bool,
    /// Recent observed events, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_events: Option<Vec<ObservedEvent>>,
}

/// Pipeline phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Pending admission.
    Pending,
    /// Starting.
    Starting,
    /// Running.
    Running,
    /// Draining.
    Draining,
    /// Stopped.
    Stopped,
    /// Failed with a categorized reason.
    Failed(FailReason),
    /// Rejected with a categorized reason.
    Rejected(RejectReason),
    /// Updating.
    Updating,
    /// Rolling back.
    RollingBack,
    /// Deleting with a deletion mode.
    Deleting(DeletionMode),
    /// Deleted.
    Deleted,
}

/// Failure reason for `Phase::Failed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailReason {
    /// Admission error.
    AdmissionError,
    /// Config rejected.
    ConfigRejected,
    /// Runtime error.
    RuntimeError,
    /// Drain error.
    DrainError,
    /// Rollback failed.
    RollbackFailed,
    /// Delete error.
    DeleteError,
    /// Update failed.
    UpdateFailed,
}

/// Rejection reason for `Phase::Rejected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectReason {
    /// Admission error.
    AdmissionError,
    /// Config rejected.
    ConfigRejected,
}

/// Deletion mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletionMode {
    /// Graceful deletion.
    Graceful,
    /// Forced deletion.
    Forced,
}

/// Status condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    /// Condition type.
    #[serde(rename = "type")]
    pub kind: ConditionKind,
    /// Condition status.
    pub status: ConditionStatus,
    /// Last transition timestamp in RFC 3339 format, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,
    /// Machine-readable reason, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ConditionReason>,
    /// Human-readable message, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Condition kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionKind {
    /// Accepted.
    Accepted,
    /// Ready.
    Ready,
    /// Cores accepted.
    CoresAccepted,
}

/// Condition status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionStatus {
    /// True.
    True,
    /// False.
    False,
    /// Unknown.
    Unknown,
}

/// Condition reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionReason {
    /// Pending.
    Pending,
    /// Start requested.
    StartRequested,
    /// Shutdown requested.
    ShutdownRequested,
    /// Delete requested.
    DeleteRequested,
    /// Force delete requested.
    ForceDeleteRequested,
    /// Config valid.
    ConfigValid,
    /// Running.
    Running,
    /// Update applied.
    UpdateApplied,
    /// Updating.
    Updating,
    /// Rolling back.
    RollingBack,
    /// Draining.
    Draining,
    /// Drained.
    Drained,
    /// Stopped.
    Stopped,
    /// Deleting.
    Deleting,
    /// Force deleting.
    ForceDeleting,
    /// Deleted.
    Deleted,
    /// Initializing.
    Initializing,
    /// Admission error.
    AdmissionError,
    /// Config rejected.
    ConfigRejected,
    /// Runtime error.
    RuntimeError,
    /// Drain error.
    DrainError,
    /// Rollback failed.
    RollbackFailed,
    /// Delete error.
    DeleteError,
    /// Update failed.
    UpdateFailed,
    /// Quorum met.
    QuorumMet,
    /// Quorum not met.
    QuorumNotMet,
    /// No active cores.
    NoActiveCores,
    /// No pipeline runtime.
    NoPipelineRuntime,
    /// Not accepted.
    NotAccepted,
    /// Unknown or custom reason.
    Unknown(String),
}

impl ConditionReason {
    /// Returns the canonical wire-format string for this reason.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            ConditionReason::Pending => "Pending",
            ConditionReason::StartRequested => "StartRequested",
            ConditionReason::ShutdownRequested => "ShutdownRequested",
            ConditionReason::DeleteRequested => "DeleteRequested",
            ConditionReason::ForceDeleteRequested => "ForceDeleteRequested",
            ConditionReason::ConfigValid => "ConfigValid",
            ConditionReason::Running => "Running",
            ConditionReason::UpdateApplied => "UpdateApplied",
            ConditionReason::Updating => "Updating",
            ConditionReason::RollingBack => "RollingBack",
            ConditionReason::Draining => "Draining",
            ConditionReason::Drained => "Drained",
            ConditionReason::Stopped => "Stopped",
            ConditionReason::Deleting => "Deleting",
            ConditionReason::ForceDeleting => "ForceDeleting",
            ConditionReason::Deleted => "Deleted",
            ConditionReason::Initializing => "Initializing",
            ConditionReason::AdmissionError => "AdmissionError",
            ConditionReason::ConfigRejected => "ConfigRejected",
            ConditionReason::RuntimeError => "RuntimeError",
            ConditionReason::DrainError => "DrainError",
            ConditionReason::RollbackFailed => "RollbackFailed",
            ConditionReason::DeleteError => "DeleteError",
            ConditionReason::UpdateFailed => "UpdateFailed",
            ConditionReason::QuorumMet => "QuorumMet",
            ConditionReason::QuorumNotMet => "QuorumNotMet",
            ConditionReason::NoActiveCores => "NoActiveCores",
            ConditionReason::NoPipelineRuntime => "NoPipelineRuntime",
            ConditionReason::NotAccepted => "NotAccepted",
            ConditionReason::Unknown(value) => value.as_str(),
        }
    }

    /// Parses a reason from its wire-format string, preserving unknown values.
    #[must_use]
    pub fn from_wire(value: String) -> Self {
        match value.as_str() {
            "Pending" => ConditionReason::Pending,
            "StartRequested" => ConditionReason::StartRequested,
            "ShutdownRequested" => ConditionReason::ShutdownRequested,
            "DeleteRequested" => ConditionReason::DeleteRequested,
            "ForceDeleteRequested" => ConditionReason::ForceDeleteRequested,
            "ConfigValid" => ConditionReason::ConfigValid,
            "Running" => ConditionReason::Running,
            "UpdateApplied" => ConditionReason::UpdateApplied,
            "Updating" => ConditionReason::Updating,
            "RollingBack" => ConditionReason::RollingBack,
            "Draining" => ConditionReason::Draining,
            "Drained" => ConditionReason::Drained,
            "Stopped" => ConditionReason::Stopped,
            "Deleting" => ConditionReason::Deleting,
            "ForceDeleting" => ConditionReason::ForceDeleting,
            "Deleted" => ConditionReason::Deleted,
            "Initializing" => ConditionReason::Initializing,
            "AdmissionError" => ConditionReason::AdmissionError,
            "ConfigRejected" => ConditionReason::ConfigRejected,
            "RuntimeError" => ConditionReason::RuntimeError,
            "DrainError" => ConditionReason::DrainError,
            "RollbackFailed" => ConditionReason::RollbackFailed,
            "DeleteError" => ConditionReason::DeleteError,
            "UpdateFailed" => ConditionReason::UpdateFailed,
            "QuorumMet" => ConditionReason::QuorumMet,
            "QuorumNotMet" => ConditionReason::QuorumNotMet,
            "NoActiveCores" => ConditionReason::NoActiveCores,
            "NoPipelineRuntime" => ConditionReason::NoPipelineRuntime,
            "NotAccepted" => ConditionReason::NotAccepted,
            other => ConditionReason::Unknown(other.to_string()),
        }
    }
}

impl Serialize for ConditionReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ConditionReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_wire(value))
    }
}

/// Probe status for a single pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeStatus {
    /// Probe succeeded.
    Ok,
    /// Probe failed.
    Failed,
}

/// Semantic probe result for a single pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    /// Probe status.
    pub status: ProbeStatus,
    /// Optional human-readable probe message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ProbeResult {
    /// Creates a probe result and normalizes empty messages to `None`.
    #[must_use]
    pub fn new(status: ProbeStatus, message: Option<String>) -> Self {
        Self {
            status,
            message: message.filter(|value| !value.is_empty()),
        }
    }
}

/// Recent observed event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObservedEvent {
    /// Engine event.
    Engine(Box<EngineEvent>),
    /// Log event.
    Log(LoggedObservedEvent),
}

/// Opaque observed log event used in recent-events payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggedObservedEvent {
    /// RFC 3339 timestamp.
    pub time: String,
    /// Opaque record payload.
    pub record: Value,
}

/// Engine event in recent-events payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineEvent {
    /// Pipeline instance key.
    pub key: DeployedPipelineKey,
    /// Node ID, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Node kind, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<NodeKind>,
    /// RFC 3339 timestamp.
    pub time: String,
    /// Event type.
    pub r#type: EventType,
    /// Message, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Deployed pipeline key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployedPipelineKey {
    /// Pipeline group identifier.
    pub pipeline_group_id: String,
    /// Pipeline identifier.
    pub pipeline_id: String,
    /// Core identifier.
    pub core_id: usize,
    /// Deployment generation when multiple generations overlap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_generation: Option<u64>,
}

/// SDK-owned node kind for observed events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Receiver.
    Receiver,
    /// Processor.
    Processor,
    /// Exporter.
    Exporter,
    /// Processor chain.
    ProcessorChain,
}

/// Engine event type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    /// Request event.
    Request(RequestEvent),
    /// Success event.
    Success(SuccessEvent),
    /// Error event.
    Error(ErrorEvent),
}

/// Request event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestEvent {
    /// Start requested.
    StartRequested,
    /// Shutdown requested.
    ShutdownRequested,
    /// Delete requested.
    DeleteRequested,
    /// Force delete requested.
    ForceDeleteRequested,
}

/// Success event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SuccessEvent {
    /// Admitted.
    Admitted,
    /// Ready.
    Ready,
    /// Update admitted.
    UpdateAdmitted,
    /// Update applied.
    UpdateApplied,
    /// Rollback complete.
    RollbackComplete,
    /// Ingress drain started.
    IngressDrainStarted,
    /// Receivers drained.
    ReceiversDrained,
    /// Downstream shutdown started.
    DownstreamShutdownStarted,
    /// Drained.
    Drained,
    /// Deleted.
    Deleted,
}

/// Error event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorEvent {
    /// Admission error.
    AdmissionError(ErrorSummary),
    /// Config rejected.
    ConfigRejected(ErrorSummary),
    /// Update failed.
    UpdateFailed(ErrorSummary),
    /// Rollback failed.
    RollbackFailed(ErrorSummary),
    /// Drain error.
    DrainError(ErrorSummary),
    /// Drain deadline reached.
    DrainDeadlineReached,
    /// Runtime error.
    RuntimeError(ErrorSummary),
    /// Delete error.
    DeleteError(ErrorSummary),
}

/// Structured error summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorSummary {
    /// Pipeline-level error.
    Pipeline {
        /// Error kind.
        error_kind: String,
        /// User-facing message.
        message: String,
        /// Flattened source chain, if available.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
    /// Node-level error.
    Node {
        /// Node identifier.
        node: String,
        /// Node kind.
        node_kind: NodeKind,
        /// Error kind.
        error_kind: String,
        /// User-facing message.
        message: String,
        /// Flattened source chain, if available.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
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
            "conditions": [
                {
                    "type": "Accepted",
                    "status": "True",
                    "reason": "ConfigValid",
                    "message": "accepted"
                },
                {
                    "type": "Ready",
                    "status": "True",
                    "lastTransitionTime": "2026-01-01T00:00:00Z",
                    "reason": "QuorumMet",
                    "message": "ready"
                }
            ],
            "totalCores": 1,
            "runningCores": 1,
            "cores": {
                "0": {
                    "phase": "running",
                    "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                    "conditions": [
                        {
                            "type": "Accepted",
                            "status": "True",
                            "reason": "ConfigValid"
                        },
                        {
                            "type": "Ready",
                            "status": "True",
                            "reason": "Running"
                        }
                    ],
                    "deletePending": false,
                    "recentEvents": [
                        {
                            "Engine": {
                                "key": {
                                    "pipeline_group_id": "default",
                                    "pipeline_id": "main",
                                    "core_id": 0
                                },
                                "time": "2026-01-01T00:00:00Z",
                                "type": {
                                    "Success": "Ready"
                                }
                            }
                        }
                    ]
                }
            }
        }));
    }

    #[test]
    fn deployed_pipeline_key_accepts_current_wire_shape_without_generation() {
        assert_roundtrip::<DeployedPipelineKey>(json!({
            "pipeline_group_id": "default",
            "pipeline_id": "main",
            "core_id": 0
        }));
    }

    #[test]
    fn deployed_pipeline_key_roundtrips_with_generation() {
        assert_roundtrip::<DeployedPipelineKey>(json!({
            "pipeline_group_id": "default",
            "pipeline_id": "main",
            "core_id": 0,
            "deployment_generation": 7
        }));
    }

    #[test]
    fn probe_result_normalizes_empty_message() {
        assert_eq!(
            ProbeResult::new(ProbeStatus::Ok, Some(String::new())),
            ProbeResult {
                status: ProbeStatus::Ok,
                message: None,
            }
        );
        assert_eq!(
            ProbeResult::new(ProbeStatus::Failed, Some("NOT OK".to_string())),
            ProbeResult {
                status: ProbeStatus::Failed,
                message: Some("NOT OK".to_string()),
            }
        );
    }
}
