// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Definition of all signals/conditions that drive state transitions and log events.

use otap_df_config::NodeId;
use otap_df_config::node::NodeKind;
use otap_df_config::{PipelineGroupId, PipelineId};
use serde::Serialize;
use serde::ser::{SerializeSeq, Serializer};
use std::collections::VecDeque;
use std::hash::Hash;
use std::time::SystemTime;

use crate::self_tracing::LogRecord;

/// Message content for an observed event.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EventMessage {
    /// No message content.
    None,
    /// A simple string message.
    Message(String),
    /// A full log record.
    Log(LogRecord),
}

impl EventMessage {
    /// Returns the message as a string slice if it's a simple message.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            EventMessage::Message(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns true if this is the None variant.
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, EventMessage::None)
    }
}

impl From<String> for EventMessage {
    fn from(s: String) -> Self {
        EventMessage::Message(s)
    }
}

impl<'a> From<&'a str> for EventMessage {
    fn from(s: &'a str) -> Self {
        EventMessage::Message(s.into())
    }
}

impl From<Option<String>> for EventMessage {
    fn from(s: Option<String>) -> Self {
        match s {
            Some(s) => EventMessage::Message(s),
            None => EventMessage::None,
        }
    }
}

impl From<LogRecord> for EventMessage {
    fn from(record: LogRecord) -> Self {
        EventMessage::Log(record)
    }
}

/// Serialize a `SystemTime` as an RFC 3339 string.
pub fn ts_to_rfc3339<S>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let dt: chrono::DateTime<chrono::Utc> = (*t).into();
    s.serialize_str(&dt.to_rfc3339())
}

/// Unique key for identifying a pipeline within a pipeline group.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PipelineKey {
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
}

impl PipelineKey {
    /// Construct a new PipelineKey from group and pipeline ids.
    #[must_use]
    pub fn new(pipeline_group_id: PipelineGroupId, pipeline_id: PipelineId) -> Self {
        Self {
            pipeline_group_id,
            pipeline_id,
        }
    }

    /// Returns the pipeline group identifier.
    #[must_use]
    pub fn pipeline_group_id(&self) -> &PipelineGroupId {
        &self.pipeline_group_id
    }

    /// Returns the pipeline identifier.
    #[must_use]
    pub fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    /// Returns a `group_id:pipeline_id` string representation.
    #[must_use]
    pub fn as_string(&self) -> String {
        format!("{}:{}", self.pipeline_group_id, self.pipeline_id)
    }
}

impl Serialize for PipelineKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}:{}", self.pipeline_group_id, self.pipeline_id);
        serializer.serialize_str(&s)
    }
}

/// Unique key for identifying a pipeline running on a specific core.
#[derive(Debug, Clone, Serialize)]
pub struct DeployedPipelineKey {
    /// The unique ID of the pipeline group the pipeline belongs to.
    pub pipeline_group_id: PipelineGroupId,

    /// The unique ID of the pipeline within its group.
    pub pipeline_id: PipelineId,

    /// The CPU core ID the pipeline is pinned to.
    /// Note: Not using core_affinity::CoreId directly to avoid dependency leakage in this public API
    pub core_id: usize,
}

/// A ring buffer for storing recent observed events.
///
/// When the buffer reaches capacity, the oldest event is dropped to make room
/// for new events. Events are serialized in reverse order (newest first).
#[derive(Debug, Clone)]
pub struct ObservedEventRingBuffer {
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
    /// Create a new ring buffer with the given capacity.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(cap),
            cap,
        }
    }

    /// Push an event into the ring buffer, dropping the oldest if full.
    pub fn push(&mut self, event: ObservedEvent) {
        if self.buf.len() == self.cap {
            _ = self.buf.pop_front(); // drop oldest
        }
        self.buf.push_back(event);
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

/// An observed event emitted by the engine.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedEvent {
    // ---- Source identification ----
    /// Unique key identifying the pipeline instance (None for global/log events).
    #[serde(skip_serializing)]
    pub key: Option<DeployedPipelineKey>,
    /// When reporting a node-level event, the node it applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    /// When reporting a node-level event, the kind of node it applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<NodeKind>,

    // ---- Event context ----
    /// Timestamp of when the event was observed.
    #[serde(serialize_with = "ts_to_rfc3339")]
    pub time: SystemTime,
    /// One of the defined event types (e.g. `StartRequested`, `Ready`, `RuntimeError`, ...).
    pub r#type: EventType,
    /// Message content for this event.
    #[serde(skip_serializing_if = "EventMessage::is_none")]
    pub message: EventMessage,
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
    /// Returns the human-readable message if present (only for Message variant).
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_str()
    }

    /// Create an `Admitted` engine-level event.
    #[must_use]
    pub fn admitted(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::Admitted),
            message: message.into(),
        }
    }

    /// Create a `Ready` pipeline-level event.
    #[must_use]
    pub fn ready(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::Ready),
            message: message.into(),
        }
    }

    /// Create an `UpdateAdmitted` pipeline-level event.
    #[must_use]
    pub fn update_admitted(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::UpdateAdmitted),
            message: message.into(),
        }
    }

    /// Create an `UpdateApplied` pipeline-level event.
    #[must_use]
    pub fn update_applied(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::UpdateApplied),
            message: message.into(),
        }
    }

    /// Create a `RollbackComplete` pipeline-level event.
    #[must_use]
    pub fn rollback_complete(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::RollbackComplete),
            message: message.into(),
        }
    }

    /// Create a `Drained` pipeline-level event.
    #[must_use]
    pub fn drained(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::Drained),
            message: message.into(),
        }
    }

    /// Create a `Deleted` pipeline-level event.
    #[must_use]
    pub fn deleted(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Success(SuccessEvent::Deleted),
            message: message.into(),
        }
    }

    /// Create an `AdmissionError` engine-level event.
    #[must_use]
    pub fn admission_error(
        key: DeployedPipelineKey,
        message: impl Into<EventMessage>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::AdmissionError(error)),
            message: message.into(),
        }
    }

    /// Create a `ConfigRejected` pipeline-level event.
    #[must_use]
    pub fn config_rejected(
        key: DeployedPipelineKey,
        message: impl Into<EventMessage>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::ConfigRejected(error)),
            message: message.into(),
        }
    }

    /// Create an `UpdateFailed` pipeline-level event.
    #[must_use]
    pub fn update_failed(
        key: DeployedPipelineKey,
        message: impl Into<EventMessage>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::UpdateFailed(error)),
            message: message.into(),
        }
    }

    /// Create a `RollbackFailed` pipeline-level event.
    #[must_use]
    pub fn rollback_failed(
        key: DeployedPipelineKey,
        message: impl Into<EventMessage>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::RollbackFailed(error)),
            message: message.into(),
        }
    }

    /// Create a `DrainError` pipeline-level event.
    #[must_use]
    pub fn drain_error(
        key: DeployedPipelineKey,
        message: impl Into<EventMessage>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::DrainError(error)),
            message: message.into(),
        }
    }

    /// Create a `DeleteError` pipeline-level event.
    #[must_use]
    pub fn delete_error(
        key: DeployedPipelineKey,
        message: impl Into<EventMessage>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::DeleteError(error)),
            message: message.into(),
        }
    }

    /// Create a `RuntimeError` pipeline-level event.
    #[must_use]
    pub fn pipeline_runtime_error(
        key: DeployedPipelineKey,
        message: impl Into<EventMessage>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::RuntimeError(error)),
            message: message.into(),
        }
    }

    /// Create a `RuntimeError` node-level event.
    #[must_use]
    pub fn node_runtime_error(
        key: DeployedPipelineKey,
        node_id: NodeId,
        node_kind: NodeKind,
        message: impl Into<EventMessage>,
        error: ErrorSummary,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: Some(node_id),
            node_kind: Some(node_kind),
            time: SystemTime::now(),
            r#type: EventType::Error(ErrorEvent::RuntimeError(error)),
            message: message.into(),
        }
    }

    /// Create a `StartRequested` request-level event.
    #[must_use]
    pub fn start_requested(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Request(RequestEvent::StartRequested),
            message: message.into(),
        }
    }

    /// Create a `ShutdownRequested` request-level event.
    #[must_use]
    pub fn shutdown_requested(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Request(RequestEvent::ShutdownRequested),
            message: message.into(),
        }
    }

    /// Create a `DeleteRequested` request-level event.
    #[must_use]
    pub fn delete_requested(key: DeployedPipelineKey, message: impl Into<EventMessage>) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Request(RequestEvent::DeleteRequested),
            message: message.into(),
        }
    }

    /// Create a `ForceDeleteRequested` request-level event.
    #[must_use]
    pub fn force_delete_requested(
        key: DeployedPipelineKey,
        message: impl Into<EventMessage>,
    ) -> Self {
        Self {
            key: Some(key),
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Request(RequestEvent::ForceDeleteRequested),
            message: message.into(),
        }
    }
}
