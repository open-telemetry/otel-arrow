// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Definition of all signals/conditions that drive state transitions.

use crate::DeployedPipelineKey;
use crate::store::ts_to_rfc3339;
use otap_df_config::NodeId;
use otap_df_config::node::NodeKind;
use serde::Serialize;
use serde::ser::{SerializeSeq, Serializer};
use std::collections::VecDeque;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub(crate) struct ObservedEventRingBuffer {
    buf: VecDeque<ObservedEvent>,
    cap: usize,
}

impl Serialize for ObservedEventRingBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.buf.len()))?;
        for ev in self.buf.iter().rev() {
            // <— reverse iteration
            seq.serialize_element(ev)?;
        }
        seq.end()
    }
}

impl ObservedEventRingBuffer {
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(cap),
            cap,
        }
    }

    pub(crate) fn push(&mut self, event: ObservedEvent) {
        if self.buf.len() == self.cap {
            _ = self.buf.pop_front(); // drop oldest
        }
        self.buf.push_back(event);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

/// An observed event emitted by the engine.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedEvent {
    // ---- Source identification ----
    /// Unique key identifying the pipeline instance.
    #[serde(skip_serializing)]
    pub(crate) key: DeployedPipelineKey,
    /// When reporting a node-level event, the node it applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    node_id: Option<NodeId>,
    /// When reporting a node-level event, the kind of node it applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    node_kind: Option<NodeKind>,

    // ---- Event context ----
    /// Timestamp of when the event was observed.
    #[serde(serialize_with = "ts_to_rfc3339")]
    pub(crate) time: SystemTime,
    /// One of the defined event types (e.g. `StartRequested`, `Ready`, `RuntimeError`, ...).
    pub(crate) r#type: EventType,
    /// Human-readable context.
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Grouping of event types by category.
#[derive(Debug, Clone, Serialize)]
pub enum EventType {
    /// Request-level events.
    Request(RequestEvent),
    /// Success/normal-progress events.
    Success(SuccessEvent),
    /// Error/failure events.
    Error(ErrorEvent),
}

/// Request-level events
#[derive(Debug, Clone, Serialize)]
pub enum RequestEvent {
    /// A request to (re)start the pipeline lifecycle from a stopped state.
    StartRequested,
    /// Begin graceful shutdown and quiesce existing work.
    ShutdownRequested,
    /// Request graceful deletion (drain if needed, then delete).
    DeleteRequested,
    /// Request immediate deletion (skip draining; preempt everything).
    ForceDeleteRequested,
}

/// Success/normal events
#[derive(Debug, Clone, Serialize)]
pub enum SuccessEvent {
    /// Controller accepted the pipeline and allows startup to proceed.
    Admitted,
    /// All the nodes are initialized and the pipeline tasks are ready to start.
    Ready,
    /// An update plan has been accepted and should begin applying.
    UpdateAdmitted,
    /// The update completed successfully and the new spec is active.
    UpdateApplied,
    /// Rollback finished successfully; last known good is restored.
    RollbackComplete,
    /// All ongoing work has drained to zero; safe to stop or delete.
    Drained,
    /// Resource teardown has finished; nothing remains.
    Deleted,
}

/// Error/failure events
#[derive(Debug, Clone, Serialize)]
pub enum ErrorEvent {
    /// Admission refused the pipeline.
    /// Reasons might include invalid config, resource limits, ...
    AdmissionError(ErrorSummary),
    /// Startup aborted because configuration was invalid/incompatible.
    ConfigRejected(ErrorSummary),
    /// The update could not complete; transition to rollback.
    UpdateFailed(ErrorSummary),
    /// Rollback could not complete successfully.
    RollbackFailed(ErrorSummary),
    /// Draining failed or timed out according to policy.
    DrainError(ErrorSummary),
    /// An unrecoverable runtime fault/crash occurred.
    RuntimeError(ErrorSummary),
    /// An error occurred during teardown.
    DeleteError(ErrorSummary),
}

/// A structured summary of an error, suitable for serialization and user display.
#[derive(Debug, Clone, Serialize)]
pub enum ErrorSummary {
    /// Summary of a pipeline-level error.
    Pipeline {
        /// High-level error category (connect/configuration/transport/etc.).
        error_kind: String,
        /// User-facing error message.
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Flattened source chain for deeper debugging, if available.
        source: Option<String>,
    },

    /// Summary of a node-level error.
    Node {
        /// Identifier of the node emitting the error.
        node: String,
        /// Node classification (e.g. `exporter`, `processor`).
        node_kind: NodeKind,
        /// High-level error category (connect/configuration/transport/etc.).
        error_kind: String,
        /// User-facing error message.
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Flattened source chain for deeper debugging, if available.
        source: Option<String>,
    },
}

impl ObservedEvent {
    /// Create an `Admitted` engine-level event.
    #[must_use]
    pub fn admitted(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::Admitted),
            message,
        }
    }

    /// Create a `Ready` pipeline-level event.
    #[must_use]
    pub fn ready(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::Ready),
            message,
        }
    }

    /// Create an `UpdateAdmitted` pipeline-level event.
    #[must_use]
    pub fn update_admitted(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::UpdateAdmitted),
            message,
        }
    }

    /// Create an `UpdateApplied` pipeline-level event.
    #[must_use]
    pub fn update_applied(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::UpdateApplied),
            message,
        }
    }

    /// Create a `RollbackComplete` pipeline-level event.
    #[must_use]
    pub fn rollback_complete(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::RollbackComplete),
            message,
        }
    }

    /// Create a `Drained` pipeline-level event.
    #[must_use]
    pub fn drained(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::Drained),
            message,
        }
    }

    /// Create a `Deleted` pipeline-level event.
    #[must_use]
    pub fn deleted(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::Deleted),
            message,
        }
    }

    /// Create an `AdmissionError` engine-level event.
    #[must_use]
    pub fn admission_error(
        key: DeployedPipelineKey,
        message: Option<String>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::AdmissionError(error)),
            message,
        }
    }

    /// Create a `ConfigRejected` pipeline-level event.
    #[must_use]
    pub fn config_rejected(
        key: DeployedPipelineKey,
        message: Option<String>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::ConfigRejected(error)),
            message,
        }
    }

    /// Create an `UpdateFailed` pipeline-level event.
    #[must_use]
    pub fn update_failed(
        key: DeployedPipelineKey,
        message: Option<String>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::UpdateFailed(error)),
            message,
        }
    }

    /// Create a `RollbackFailed` pipeline-level event.
    #[must_use]
    pub fn rollback_failed(
        key: DeployedPipelineKey,
        message: Option<String>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::RollbackFailed(error)),
            message,
        }
    }

    /// Create a `DrainError` pipeline-level event.
    #[must_use]
    pub fn drain_error(
        key: DeployedPipelineKey,
        message: Option<String>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::DrainError(error)),
            message,
        }
    }

    /// Create a `DeleteError` pipeline-level event.
    #[must_use]
    pub fn delete_error(
        key: DeployedPipelineKey,
        message: Option<String>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::DeleteError(error)),
            message,
        }
    }

    /// Create a `RuntimeError` pipeline-level event.
    #[must_use]
    pub fn pipeline_runtime_error(
        key: DeployedPipelineKey,
        message: impl Into<String>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::RuntimeError(error)),
            message: Some(message.into()),
        }
    }

    /// Create a `RuntimeError` node-level event.
    #[must_use]
    pub fn node_runtime_error(
        key: DeployedPipelineKey,
        node_id: NodeId,
        node_kind: NodeKind,
        message: Option<String>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key,
            node_id: Some(node_id),
            node_kind: Some(node_kind),
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::RuntimeError(error)),
            message,
        }
    }

    /// Create a `StartRequested` request-level event.
    #[must_use]
    pub fn start_requested(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Request(RequestEvent::StartRequested),
            message,
        }
    }

    /// Create a `ShutdownRequested` request-level event.
    #[must_use]
    pub fn shutdown_requested(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Request(RequestEvent::ShutdownRequested),
            message,
        }
    }

    /// Create a `DeleteRequested` request-level event.
    #[must_use]
    pub fn delete_requested(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Request(RequestEvent::DeleteRequested),
            message,
        }
    }

    /// Create a `ForceDeleteRequested` request-level event.
    #[must_use]
    pub fn force_delete_requested(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Request(RequestEvent::ForceDeleteRequested),
            message,
        }
    }
}
