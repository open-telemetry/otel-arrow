// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for the pipeline engine.
//!
//! Important note: It is important not to use `!Send` data types in errors (e.g. avoid using Rc) to
//! ensure these errors can be emitted in both `Send` and `!Send` contexts.

use crate::node::{NodeId, NodeName};
use otap_df_channel::error::SendError;
use otap_df_config::node::NodeKind;
use otap_df_config::{NodeUrn, PortName, TopicName};
use otap_df_telemetry::event::ErrorSummary;
use std::borrow::Cow;
use std::fmt;

/// High-level classification for exporter failures to aid troubleshooting.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExporterErrorKind {
    /// Errors encountered while establishing a connection to a remote endpoint.
    Connect,
    /// Errors caused by invalid or incomplete configuration detected at runtime.
    Configuration,
    /// Errors transporting telemetry payloads after an exporter has started.
    Transport,
    /// Errors raised while shutting down an exporter.
    Shutdown,
    /// Catch-all for exporter failures that do not fit other categories.
    Other,
}

impl fmt::Display for ExporterErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ExporterErrorKind::Connect => "connect",
            ExporterErrorKind::Configuration => "configuration",
            ExporterErrorKind::Transport => "transport",
            ExporterErrorKind::Shutdown => "shutdown",
            ExporterErrorKind::Other => "other",
        };
        write!(f, "{label}")
    }
}

/// High-level classification for receiver failures to aid troubleshooting.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReceiverErrorKind {
    /// Errors encountered while binding or establishing inbound connections.
    Connect,
    /// Errors caused by invalid or missing configuration.
    Configuration,
    /// Errors transporting or decoding telemetry payloads after the receiver has started.
    Transport,
    /// Errors raised while shutting down a receiver.
    Shutdown,
    /// Catch-all for receiver failures that do not fit other categories.
    Other,
}

impl fmt::Display for ReceiverErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ReceiverErrorKind::Connect => "connect",
            ReceiverErrorKind::Configuration => "configuration",
            ReceiverErrorKind::Transport => "transport",
            ReceiverErrorKind::Shutdown => "shutdown",
            ReceiverErrorKind::Other => "other",
        };
        write!(f, "{label}")
    }
}

/// High-level classification for processor failures to aid troubleshooting.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProcessorErrorKind {
    /// Errors encountered while initialising or wiring processor inputs/outputs.
    Configuration,
    /// Errors encountered when receiving or emitting pdata.
    Transport,
    /// Errors raised while shutting down a processor.
    Shutdown,
    /// Catch-all for processor failures that do not fit other categories.
    Other,
}

impl fmt::Display for ProcessorErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ProcessorErrorKind::Configuration => "configuration",
            ProcessorErrorKind::Transport => "transport",
            ProcessorErrorKind::Shutdown => "shutdown",
            ProcessorErrorKind::Other => "other",
        };
        write!(f, "{label}")
    }
}

/// Formats the source chain of an error into a single display string.
#[must_use]
pub fn format_error_sources(error: &(dyn std::error::Error + 'static)) -> String {
    let mut segments = Vec::new();
    let mut current = error.source();
    while let Some(err) = current {
        let msg = err.to_string();
        if !msg.is_empty() {
            segments.push(msg);
        }
        current = err.source();
    }

    if segments.is_empty() {
        String::new()
    } else {
        format!("; source: {}", segments.join(" -> "))
    }
}

/// All errors that can occur in the pipeline engine infrastructure
/// that contain a variant type <T>. Generally these errors are
/// returned in situations where you may want to take ownership of the
/// request after it has failed to send. Most callers that encounter
/// TypedError<T> and wish to drop the <T> can use .map_err(|e| e.into())
/// to return an Error.
#[derive(thiserror::Error, Debug)]
pub enum TypedError<T> {
    /// A wrapper for the channel errors.
    #[error("A channel error occurred: {0}")]
    ChannelSendError(SendError<T>),

    /// A wrapper for the pipeline control message send errors.
    #[error("A pipeline control channel error occurred: {0}")]
    PipelineControlMsgError(SendError<T>),

    /// A wrapper for the node control message send errors.
    #[error("A node control message send error occurred in node {node_id}: {error}")]
    NodeControlMsgSendError {
        /// The name of the node that encountered the error.
        node_id: usize,

        /// The error that occurred.
        error: SendError<T>,
    },

    /// A type-less error in TypedError<T> context.
    #[error("{0}")]
    Error(Error),
}

impl<T: Sized> From<TypedError<T>> for Error {
    /// This drops the SendError<T> field yielding an untyped error.
    fn from(value: TypedError<T>) -> Self {
        match value {
            TypedError::ChannelSendError(e) => Error::ChannelSendError {
                error: e.to_string(),
            },
            TypedError::PipelineControlMsgError(e) => Error::PipelineControlMsgError {
                error: e.to_string(),
            },
            TypedError::NodeControlMsgSendError { node_id, error } => {
                Error::NodeControlMsgSendError {
                    node: NodeId::build(node_id, "name is unknown".into()),
                    error: error.to_string(),
                }
            }
            TypedError::Error(e) => e,
        }
    }
}

/// All errors that can occur in the pipeline engine infrastructure.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A wrapper for the config errors.
    #[error("A config error occurred: {0}")]
    ConfigError(#[from] Box<otap_df_config::error::Error>),

    /// A wrapper for the channel errors.
    #[error("A channel error occurred: {0}")]
    ChannelRecvError(#[from] otap_df_channel::error::RecvError),

    /// A wrapper for the channel errors.
    #[error("A data channel error occurred: {error}")]
    ChannelSendError {
        /// The reason (e.g., channel full)
        error: String,
    },

    /// A wrapper for the pipeline control message send errors.
    #[error("A control channel error occurred: {error}")]
    PipelineControlMsgError {
        /// The reason (e.g., channel closed)
        error: String,
    },

    /// A wrapper for the node control message send errors.
    #[error("A node control message send error occurred in node {node}: {error}")]
    NodeControlMsgSendError {
        /// The node to which a message could not be sent.
        node: NodeId,
        /// The reason (e.g., channel closed)
        error: String,
    },

    /// The specified hyper-edge is invalid.
    #[error("Invalid hyper-edge in node {source} with output port {output_port}: {error}")]
    InvalidHyperEdge {
        /// The name of the node that contains the invalid hyper-edge.
        r#source: NodeId,

        /// The invalid output port.
        output_port: PortName,

        /// The reason why the hyper-edge is invalid.
        error: String,
    },

    /// A wrapper for the IO errors.
    #[error("An IO error occurred in node {node}: {error}")]
    IoError {
        /// The name of the node that encountered the error.
        node: NodeId,

        /// The error that occurred.
        error: std::io::Error,
    },

    /// The specified already exists in the pipeline.
    #[error("The receiver `{receiver}` already exists")]
    ReceiverAlreadyExists {
        /// The name of the receiver that already exists.
        receiver: NodeId,
    },

    /// A wrapper for the receiver errors.
    #[error("A receiver error occurred in node {receiver} ({kind}): {error}{source_detail}")]
    ReceiverError {
        /// The name of the receiver that encountered the error.
        receiver: NodeId,

        /// High-level classification for the receiver failure.
        kind: ReceiverErrorKind,

        /// The error that occurred.
        /// ToDo We probably need to use a more specific error type here (JSON Node?).
        error: String,

        /// Pre-formatted representation of the source chain used when rendering the error.
        source_detail: String,
    },

    /// Unknown receiver plugin.
    #[error("Unknown receiver plugin `{plugin_urn}`")]
    UnknownReceiver {
        /// The name of the unknown receiver plugin.
        plugin_urn: NodeUrn,
    },

    /// The specified processor already exists in the pipeline.
    #[error("The processor `{processor}` already exists")]
    ProcessorAlreadyExists {
        /// The name of the processor that already exists.
        processor: NodeId,
    },

    /// A wrapper for the processor errors.
    #[error("A processor error occurred in node {processor} ({kind}): {error}{source_detail}")]
    ProcessorError {
        /// The name of the processor that encountered the error.
        processor: NodeId,

        /// High-level classification for the processor failure.
        kind: ProcessorErrorKind,

        /// The error that occurred.
        /// ToDo We probably need to use a more specific error type here (JSON Node?).
        error: String,

        /// Pre-formatted representation of the source chain used when rendering the error.
        source_detail: String,
    },

    /// Unknown processor plugin.
    #[error("Unknown processor plugin `{plugin_urn}`")]
    UnknownProcessor {
        /// The name of the unknown processor plugin.
        plugin_urn: NodeUrn,
    },

    /// The specified exporter already exists in the pipeline.
    #[error("The exporter `{exporter}` already exists")]
    ExporterAlreadyExists {
        /// The name of the exporter that already exists.
        exporter: NodeId,
    },

    /// A wrapper for the exporter errors.
    #[error("An exporter error occurred in node {exporter} ({kind}): {error}{source_detail}")]
    ExporterError {
        /// The name of the exporter that encountered the error.
        exporter: NodeId,

        /// High-level classification for the exporter failure.
        kind: ExporterErrorKind,

        /// The error that occurred.
        error: String,

        /// Pre-formatted representation of the source chain used when rendering the error.
        source_detail: String,
    },

    /// A Wrapper for the pdata conversion errors
    #[error("Internal error occurred transforming pdata: {error}")]
    PdataConversionError {
        /// The error that occurred
        error: String,
    },

    /// Unknown exporter plugin.
    #[error("Unknown exporter plugin `{plugin_urn}`")]
    UnknownExporter {
        /// The name of the unknown exporter plugin.
        plugin_urn: NodeUrn,
    },

    /// Unknown node.
    #[error("Unknown node `{node}`")]
    UnknownNode {
        /// The name of the unknown node.
        node: NodeName,
    },

    /// Pdata receiver is not supported for receiver nodes. Receiver nodes only support
    /// control sender and pdata sender.
    #[error("Pdata receiver is not supported for a receiver node.")]
    PdataReceiverNotSupported,

    /// Pdata sender is not supported for exporter nodes. Exporter nodes only support
    /// control sender and pdata receiver.
    #[error("Pdata sender is not supported for exporter nodes.")]
    PdataSenderNotSupported,

    /// SPMC shared channels are not yet supported.
    #[error("SPMC shared channels are not yet supported. Source: {source_id}, Port: {port_name}")]
    SpmcSharedNotSupported {
        /// The id of the source node.
        source_id: NodeId,
        /// The name of the port.
        port_name: PortName,
    },

    /// Unsupported node kind.
    #[error("Unsupported node kind `{kind}`")]
    UnsupportedNodeKind {
        /// The kind of the node that is not supported.
        kind: Cow<'static, str>,
    },

    /// Node wiring violates the node type contract.
    #[error(
        "Invalid wiring for node `{node}` output `{output}`: allowed at most {max_destinations} destination(s), found {actual_destinations:?}"
    )]
    InvalidNodeWiring {
        /// The source node.
        node: NodeName,
        /// The source output port.
        output: PortName,
        /// Maximum allowed destination count for this node output.
        max_destinations: usize,
        /// Actual resolved destinations connected to this output.
        actual_destinations: Vec<NodeName>,
    },

    /// A task error that occurred during the execution of a join task.
    #[error("Join task error: {error}, cancelled: {is_canceled}, panic: {is_panic}")]
    JoinTaskError {
        /// Flag indicating whether the task was canceled.
        is_canceled: bool,
        /// Flag indicating whether the task panicked.
        is_panic: bool,
        /// The error that occurred.
        error: String,
    },

    /// An internal error that occurred in the pipeline engine.
    #[error("Internal error: {message}")]
    InternalError {
        /// An internal error message.
        message: String,
    },

    /// All nodes were removed from the pipeline (none are connected).
    #[error(
        "Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration"
    )]
    EmptyPipeline,

    /// Too many nodes are configured.
    #[error("Too many nodes defined")]
    TooManyNodes {},

    /// Error in pipeline data (e.g., translation)
    /// Note: this is not a specific type such as otap_df_pdata::error::Error
    /// because this crate does not specifically depend on that crate.  We
    /// could use a dyn Error, maybe.
    #[error("Pipeline data error: {}", reason)]
    PDataError {
        /// otap_df_pdata error string
        reason: String,
    },

    /// Error from the prost encoder.
    #[error("Prost encode error: {}", reason)]
    ProtoEncodeError {
        /// Prost error string
        reason: String,
    },

    /// No default output port is configured when multiple ports are connected.
    #[error(
        "Ambiguous default output port for node {node}: multiple ports connected and no default configured"
    )]
    NoDefaultOutputPort {
        /// The node that has no default port configured.
        node: NodeId,
    },

    /// An unknown output port was specified.
    #[error("Unknown output port '{port}' for node {node}")]
    UnknownOutputPort {
        /// The node where the unknown port was referenced.
        node: NodeId,
        /// The name of the unknown port.
        port: PortName,
    },
    /// A topic with the same name already exists in the broker.
    #[error("topic `{topic}` already exists")]
    TopicAlreadyExists {
        /// The name of the topic that already exists.
        topic: TopicName,
    },

    /// Acknowledgement is not enabled for this subscription.
    #[error("ack not enabled for this subscription")]
    AckNotEnabled,

    /// An ack event could not be sent because the ack channel is full.
    #[error("ack channel full, event dropped")]
    AckChannelFull,

    /// An ack event could not be sent because the ack channel is closed.
    #[error("ack channel closed")]
    AckChannelClosed,

    /// An operation could not be completed because the topic is closed.
    #[error("topic closed")]
    TopicClosed,

    /// Balanced (consumer-group) subscriptions are not supported on this topic.
    #[error("balanced subscriptions are not supported on this topic")]
    SubscribeBalancedNotSupported,

    /// Broadcast subscriptions are not supported on this topic.
    #[error("broadcast subscriptions are not supported on this topic")]
    SubscribeBroadcastNotSupported,

    /// This topic only supports a single consumer group, but a subscription request would violate that constraint.
    #[error("this topic only supports a single consumer group")]
    SubscribeSingleGroupViolation,

    /// A subscription operation failed because the subscription was closed.
    #[error("subscription closed")]
    SubscriptionClosed,
}

impl Error {
    /// Returns the name of the error variant as a string.
    #[must_use]
    pub fn variant_name(&self) -> String {
        match self {
            Error::ChannelRecvError(_) => "ChannelRecvError",
            Error::ChannelSendError { .. } => "ChannelSendError",
            Error::ConfigError(_) => "ConfigError",
            Error::EmptyPipeline => "EmptyPipeline",
            Error::ExporterAlreadyExists { .. } => "ExporterAlreadyExists",
            Error::ExporterError { .. } => "ExporterError",
            Error::InternalError { .. } => "InternalError",
            Error::InvalidHyperEdge { .. } => "InvalidHyperEdge",
            Error::IoError { .. } => "IoError",
            Error::JoinTaskError { .. } => "JoinTaskError",
            Error::NoDefaultOutputPort { .. } => "NoDefaultOutputPort",
            Error::NodeControlMsgSendError { .. } => "NodeControlMsgSendError",
            Error::PDataError { .. } => "PDataError",
            Error::PdataConversionError { .. } => "PdataConversionError",
            Error::PdataReceiverNotSupported => "PdataReceiverNotSupported",
            Error::PdataSenderNotSupported => "PdataSenderNotSupported",
            Error::PipelineControlMsgError { .. } => "PipelineControlMsgError",
            Error::ProcessorAlreadyExists { .. } => "ProcessorAlreadyExists",
            Error::ProcessorError { .. } => "ProcessorError",
            Error::ProtoEncodeError { .. } => "ProtoEncodeError",
            Error::ReceiverAlreadyExists { .. } => "ReceiverAlreadyExists",
            Error::ReceiverError { .. } => "ReceiverError",
            Error::SpmcSharedNotSupported { .. } => "SpmcSharedNotSupported",
            Error::TooManyNodes {} => "TooManyNodes",
            Error::UnknownExporter { .. } => "UnknownExporter",
            Error::UnknownNode { .. } => "UnknownNode",
            Error::UnknownOutputPort { .. } => "UnknownOutputPort",
            Error::UnknownProcessor { .. } => "UnknownProcessor",
            Error::UnknownReceiver { .. } => "UnknownReceiver",
            Error::UnsupportedNodeKind { .. } => "UnsupportedNodeKind",
            Error::InvalidNodeWiring { .. } => "InvalidNodeWiring",
            Error::TopicAlreadyExists { .. } => "TopicAlreadyExists",
            Error::AckNotEnabled => "AckNotEnabled",
            Error::AckChannelFull => "AckChannelFull",
            Error::AckChannelClosed => "AckChannelClosed",
            Error::TopicClosed => "TopicClosed",
            Error::SubscribeBalancedNotSupported => "SubscribeBalancedNotSupported",
            Error::SubscribeBroadcastNotSupported => "SubscribeBroadcastNotSupported",
            Error::SubscribeSingleGroupViolation => "SubscribeSingleGroupViolation",
            Error::SubscriptionClosed => "SubscriptionClosed",
        }
        .to_owned()
    }
}

/// Converts an `Error` into an `ErrorSummary` for easier reporting and troubleshooting.
#[must_use]
pub fn error_summary_from(err: &Error) -> ErrorSummary {
    match err {
        Error::ReceiverError {
            receiver,
            kind,
            error,
            source_detail,
        } => ErrorSummary::Node {
            node: receiver.name.to_string(),
            node_kind: NodeKind::Receiver,
            error_kind: kind.to_string(),
            message: error.clone(),
            source: (!source_detail.is_empty()).then(|| source_detail.clone()),
        },
        Error::ProcessorError {
            processor,
            kind,
            error,
            source_detail,
        } => ErrorSummary::Node {
            node: processor.name.to_string(),
            node_kind: NodeKind::Processor,
            error_kind: kind.to_string(),
            message: error.clone(),
            source: (!source_detail.is_empty()).then(|| source_detail.clone()),
        },
        Error::ExporterError {
            exporter,
            kind,
            error,
            source_detail,
        } => ErrorSummary::Node {
            node: exporter.name.to_string(),
            node_kind: NodeKind::Exporter,
            error_kind: kind.to_string(),
            message: error.clone(),
            source: (!source_detail.is_empty()).then(|| source_detail.clone()),
        },
        _ => ErrorSummary::Pipeline {
            error_kind: err.variant_name(),
            message: err.to_string(),
            source: None,
        },
    }
}

impl From<prost::EncodeError> for Error {
    fn from(e: prost::EncodeError) -> Self {
        Self::ProtoEncodeError {
            reason: e.to_string(),
        }
    }
}

impl From<otap_df_pdata::error::Error> for Error {
    fn from(e: otap_df_pdata::error::Error) -> Self {
        Self::PDataError {
            reason: e.to_string(),
        }
    }
}
