// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Definition of all signals/conditions that drive state transitions and log events.

use crate::self_tracing::LogRecord;
use otap_df_config::{
    DeployedPipelineKey, NodeId, node::NodeKind, pipeline::service::telemetry::logs::ProviderMode,
};
use serde::Serialize;
use serde::ser::Serializer;
use std::time::{Duration, SystemTime};

/// A sharable/clonable observed event reporter sending events to an `ObservedStore`.
#[derive(Clone)]
pub struct ObservedEventReporter {
    timeout: Duration,
    fallback_mode: ProviderMode,
    sender: flume::Sender<ObservedEvent>,
}

impl ObservedEventReporter {
    /// Creates a new `ObservedEventReporter` with the given sender channel.
    #[must_use]
    pub fn new(
        timeout: Duration,
        fallback_mode: ProviderMode,
        sender: flume::Sender<ObservedEvent>,
    ) -> Self {
        Self {
            timeout,
            fallback_mode,
            sender,
        }
    }

    /// Report an observed event.
    pub fn report(&self, event: ObservedEvent) {
        let sent = self.sender.send_timeout(event, self.timeout);
        if self.fallback_mode != ProviderMode::ConsoleDirect {
            // Valid: Noop or ConsoleDirect.
            return;
        }
        match sent {
            Err(flume::SendTimeoutError::Timeout(event)) => {
                crate::raw_error!("Timeout sending observed event", event = ?event);
            }
            Err(flume::SendTimeoutError::Disconnected(event)) => {
                crate::raw_error!("Disconnect sending observed event", event = ?event);
            }
            Ok(_) => {}
        };
    }
}

/// An observed event emitted by the engine.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedEvent {
    // ---- Source identification ----
    /// Unique key identifying the pipeline instance (None for global/controller-level events).
    #[serde(skip_serializing_if = "Option::is_none")]
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

/// Message content for an observed event.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EventMessage {
    /// No message content.
    None,
    /// A simple string message.
    Message(String),
    /// A full log record (except timestamp).
    Log(LogRecord),
}

impl EventMessage {
    /// Returns the message as a string.
    #[must_use]
    pub fn formatted(&self) -> Option<String> {
        match self {
            EventMessage::None => None,
            EventMessage::Message(s) => Some(s.as_str().into()),
            EventMessage::Log(record) => Some(record.format_without_timestamp()),
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

/// Grouping of event types by category.
#[derive(Debug, Clone, Serialize)]
pub enum EventType {
    /// Request-level events.
    Request(RequestEvent),
    /// Success/normal-progress events.
    Success(SuccessEvent),
    /// Error/failure events.
    Error(ErrorEvent),
    /// Log record events (from tracing layer).
    Log,
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

    /// Create a log record event.
    ///
    /// The `key` is optional - use `None` for global/controller-level logs that
    /// don't have a specific pipeline context.
    #[must_use]
    pub fn log_record(key: Option<DeployedPipelineKey>, message: impl Into<EventMessage>) -> Self {
        Self {
            key,
            node_id: None,
            node_kind: None,
            time: SystemTime::now(),
            r#type: EventType::Log,
            message: message.into(),
        }
    }
}

/// Serialize a `SystemTime` as an RFC 3339 string.
fn ts_to_rfc3339<S>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let dt: chrono::DateTime<chrono::Utc> = (*t).into();
    s.serialize_str(&dt.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::info_event;
    use otap_df_config::DeployedPipelineKey;

    fn test_key() -> DeployedPipelineKey {
        DeployedPipelineKey {
            pipeline_group_id: "test-ns".into(),
            pipeline_id: "test-pipeline".into(),
            core_id: 0,
        }
    }

    #[test]
    fn test_observed_event_with_log_record() {
        // Create an ObservedEvent with an INFO message
        let event = ObservedEvent::ready(
            test_key(),
            info_event!("pipeline.started", version = "1.0.0"),
        );

        let formatted = event
            .message
            .formatted()
            .expect("should have formatted output");
        assert!(formatted.contains("version=1.0.0"), "got: {}", formatted);
        assert!(formatted.contains("pipeline.started"), "got: {}", formatted);
    }
}
