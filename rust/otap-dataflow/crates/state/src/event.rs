// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Definition of all signals/conditions that drive state transitions.

use std::collections::VecDeque;
use crate::DeployedPipelineKey;
use otap_df_config::NodeId;
use otap_df_config::node::NodeKind;
use std::time::SystemTime;
use serde::ser::{Serializer, SerializeSeq};
use serde::Serialize;
use crate::store::ts_to_rfc3339;

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
        for ev in self.buf.iter().rev() {    // <— reverse iteration
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ObservedEvent> {
        self.buf.iter()
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

/// Event types that can be reported by the controller runtime, the pipelines, and the nodes.
#[derive(Debug, Clone, Serialize)]
pub enum EventType {
    // ---- Request events ----
    /// A request to (re)start the pipeline lifecycle from a stopped state.
    StartRequested,
    /// Begin graceful shutdown and quiesce existing work.
    ShutdownRequested,
    /// Request graceful deletion (drain if needed, then delete).
    DeleteRequested,
    /// Request immediate deletion (skip draining; preempt everything).
    ForceDeleteRequested,

    // ---- Success events ----
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

    // ---- Error events ----
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
    }
}

impl ObservedEvent {
    /// Create an `Admitted` engine-level event.
    pub fn admitted(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Admitted,
            message,
        }
    }

    /// Create a `Ready` pipeline-level event.
    pub fn ready(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Ready,
            message,
        }
    }

    /// Create an `UpdateAdmitted` pipeline-level event.
    pub fn update_admitted(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::UpdateAdmitted,
            message,
        }
    }
    
    /// Create an `UpdateApplied` pipeline-level event.
    pub fn update_applied(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::UpdateApplied,
            message,
        }
    }
    
    /// Create a `RollbackComplete` pipeline-level event.
    pub fn rollback_complete(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::RollbackComplete,
            message,
        }
    }
    
    /// Create a `Drained` pipeline-level event.
    pub fn drained(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Drained,
            message,
        }
    }
    
    /// Create a `Deleted` pipeline-level event.
    pub fn deleted(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Deleted,
            message,
        }
    }
    
    /// Create an `AdmissionError` engine-level event.
    pub fn admission_error(key: DeployedPipelineKey, message: Option<String>, error: ErrorSummary) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::AdmissionError(error),
            message,
        }
    }
    
    /// Create a `ConfigRejected` pipeline-level event.
    pub fn config_rejected(key: DeployedPipelineKey, message: Option<String>, error: ErrorSummary) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::ConfigRejected(error),
            message,
        }
    }
    
    /// Create an `UpdateFailed` pipeline-level event.
    pub fn update_failed(key: DeployedPipelineKey, message: Option<String>, error: ErrorSummary) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::UpdateFailed(error),
            message,
        }
    }
    
    /// Create a `RollbackFailed` pipeline-level event.
    pub fn rollback_failed(key: DeployedPipelineKey, message: Option<String>, error: ErrorSummary) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::RollbackFailed(error),
            message,
        }
    }
    
    /// Create a `DrainError` pipeline-level event.
    pub fn drain_error(key: DeployedPipelineKey, message: Option<String>, error: ErrorSummary) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::DrainError(error),
            message,
        }
    }
    
    /// Create a `DeleteError` pipeline-level event.
    pub fn delete_error(key: DeployedPipelineKey, message: Option<String>, error: ErrorSummary) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::DeleteError(error),
            message,
        }
    }
    
    /// Create a `RuntimeError` pipeline-level event.
    pub fn pipeline_runtime_error(key: DeployedPipelineKey, message: impl Into<String>, error: ErrorSummary) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::RuntimeError(error),
            message: Some(message.into()),
        }
    }

    /// Create a `RuntimeError` node-level event.
    pub fn node_runtime_error(
        key: DeployedPipelineKey,
        node_id: NodeId,
        node_kind: NodeKind,
        message: Option<String>,
        error: ErrorSummary
    ) -> Self {
        Self {
            key,
            node_id: Some(node_id),
            node_kind: Some(node_kind),
            time: SystemTime::now(),
            r#type: EventType::RuntimeError(error),
            message,
        }
    }

    /// Create a `StartRequested` request-level event.
    pub fn start_requested(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::StartRequested,
            message,
        }
    }

    /// Create a `ShutdownRequested` request-level event.
    pub fn shutdown_requested(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::ShutdownRequested,
            message,
        }
    }

    /// Create a `DeleteRequested` request-level event.
    pub fn delete_requested(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::DeleteRequested,
            message,
        }
    }

    /// Create a `ForceDeleteRequested` request-level event.
    pub fn force_delete_requested(key: DeployedPipelineKey, message: Option<String>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::ForceDeleteRequested,
            message,
        }
    }
}
