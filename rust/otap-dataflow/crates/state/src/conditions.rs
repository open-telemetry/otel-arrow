// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared types for expressing status conditions.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::ser::SerializeStruct;
use std::time::SystemTime;

/// Standardized condition status values in line with Kubernetes conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionStatus {
    /// Condition is true.
    True,
    /// Condition is false.
    False,
    /// Condition value is unknown.
    Unknown,
}

impl ConditionStatus {
    #[must_use]
    /// Returns the canonical Kubernetes-style string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            ConditionStatus::True => "True",
            ConditionStatus::False => "False",
            ConditionStatus::Unknown => "Unknown",
        }
    }
}

impl Serialize for ConditionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Named condition types surfaced by the admin API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionKind {
    /// Pipeline (or core) configuration has been accepted and admitted.
    Accepted,
    /// Pipeline (or core) is ready to process telemetry.
    Ready,
    /// All targeted cores have accepted the configuration.
    CoresAccepted,
}

impl ConditionKind {
    #[must_use]
    /// Returns the canonical type identifier used in serialized payloads.
    pub fn as_str(self) -> &'static str {
        match self {
            ConditionKind::Accepted => "Accepted",
            ConditionKind::Ready => "Ready",
            ConditionKind::CoresAccepted => "CoresAccepted",
        }
    }
}

impl Serialize for ConditionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Machine-readable reason associated with a condition transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionReason {
    /// Pipeline is awaiting admission/config validation.
    Pending,
    /// A start request has been issued for the pipeline.
    StartRequested,
    /// A shutdown request has been issued and draining will begin.
    ShutdownRequested,
    /// A graceful delete was requested.
    DeleteRequested,
    /// A force delete was requested.
    ForceDeleteRequested,
    /// Configuration has been validated and accepted.
    ConfigValid,
    /// Pipeline is actively running and serving work.
    Running,
    /// An update has been applied successfully.
    UpdateApplied,
    /// An update is currently in progress.
    Updating,
    /// A rollback is in progress.
    RollingBack,
    /// Pipeline is draining in-flight work.
    Draining,
    /// Pipeline has drained successfully.
    Drained,
    /// Pipeline is stopped but not deleted.
    Stopped,
    /// Deletion is underway.
    Deleting,
    /// Forceful deletion is underway.
    ForceDeleting,
    /// Pipeline resources have been deleted.
    Deleted,
    /// Pipeline is still initializing before readiness.
    Initializing,
    /// Admission failed due to controller policy.
    AdmissionError,
    /// Configuration was rejected during startup.
    ConfigRejected,
    /// Runtime error occurred while processing.
    RuntimeError,
    /// Draining failed or timed out.
    DrainError,
    /// Rollback failed.
    RollbackFailed,
    /// Delete operation failed.
    DeleteError,
    /// Update application failed.
    UpdateFailed,
    /// Ready quorum satisfied.
    QuorumMet,
    /// Ready quorum not satisfied.
    QuorumNotMet,
    /// No active cores counted for readiness.
    NoActiveCores,
    /// Pipeline has no observed runtime cores yet.
    NoPipelineRuntime,
    /// Catch-all for acceptance failures without a more specific reason.
    NotAccepted,
    /// Unknown or custom reason string.
    Unknown(String),
}

impl ConditionReason {
    #[must_use]
    /// Returns the canonical string representation used in serialized payloads.
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

    #[must_use]
    /// Construct an unknown/custom reason variant.
    pub fn unknown(reason: impl Into<String>) -> Self {
        ConditionReason::Unknown(reason.into())
    }
}

impl Serialize for ConditionReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Mutable condition state used while processing events.
#[derive(Debug, Clone)]
pub struct ConditionState {
    /// Current status value for the condition.
    pub status: ConditionStatus,
    /// Machine-readable reason for the latest transition.
    pub reason: Option<ConditionReason>,
    /// Human-readable message providing context for the condition.
    pub message: Option<String>,
    /// Timestamp of the most recent transition.
    pub last_transition_time: Option<SystemTime>,
}

impl ConditionState {
    #[must_use]
    /// Construct a new condition state snapshot.
    pub fn new(
        status: ConditionStatus,
        reason: impl Into<Option<ConditionReason>>,
        message: impl Into<Option<String>>,
        last_transition_time: Option<SystemTime>,
    ) -> Self {
        Self {
            status,
            reason: reason.into(),
            message: message.into(),
            last_transition_time,
        }
    }

    /// Update the state if the new values differ, returning `true` when a transition occurred.
    pub fn update(
        &mut self,
        status: ConditionStatus,
        reason: impl Into<Option<ConditionReason>>,
        message: impl Into<Option<String>>,
        transition_time: SystemTime,
    ) -> bool {
        let reason = reason.into();
        let message = message.into();
        if self.status == status && self.reason == reason && self.message == message {
            return false;
        }
        self.status = status;
        self.reason = reason;
        self.message = message;
        self.last_transition_time = Some(transition_time);
        true
    }
}

impl Default for ConditionState {
    fn default() -> Self {
        Self {
            status: ConditionStatus::Unknown,
            reason: None,
            message: None,
            last_transition_time: None,
        }
    }
}

/// Serializable snapshot of a condition.
#[derive(Debug, Clone)]
pub struct Condition {
    /// Name of the condition.
    pub kind: ConditionKind,
    /// Current status of the condition.
    pub status: ConditionStatus,
    /// Machine-readable reason for the condition status.
    pub reason: Option<ConditionReason>,
    /// Human-readable context message.
    pub message: Option<String>,
    /// Timestamp of the last observed transition.
    pub last_transition_time: Option<SystemTime>,
    // ToDo last_heartbeat_time: Option<SystemTime>, // For future use when we want to track the last time the condition was observed, even if it hasn't changed.
}

impl Condition {
    #[must_use]
    /// Build a serializable snapshot from a mutable `ConditionState`.
    pub fn from_state(kind: ConditionKind, state: &ConditionState) -> Self {
        Self {
            kind,
            status: state.status,
            reason: state.reason.clone(),
            message: state.message.clone(),
            last_transition_time: state.last_transition_time,
        }
    }
}

impl Serialize for Condition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Condition", 5)?;
        state.serialize_field("type", &self.kind)?;
        state.serialize_field("status", &self.status)?;
        if let Some(ts) = self.last_transition_time {
            let ts: DateTime<Utc> = ts.into();
            state.serialize_field("lastTransitionTime", &ts.to_rfc3339())?;
        }
        if let Some(reason) = &self.reason {
            state.serialize_field("reason", reason)?;
        }
        if let Some(message) = &self.message {
            state.serialize_field("message", message)?;
        }
        state.end()
    }
}
