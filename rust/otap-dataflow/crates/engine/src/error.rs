// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for the pipeline engine.
//!
//! Important note: It is important not to use `!Send` data types in errors (e.g. avoid using Rc) to
//! ensure these errors can be emitted in both `Send` and `!Send` contexts.

use crate::node::{NodeId, NodeName};
use otap_df_channel::error::SendError;
use otap_df_config::{PortName, Urn};
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
    #[error("Invalid hyper-edge in node {source} with out port {out_port}: {error}")]
    InvalidHyperEdge {
        /// The name of the node that contains the invalid hyper-edge.
        r#source: NodeId,

        /// The invalid out port.
        out_port: PortName,

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
        plugin_urn: Urn,
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
        plugin_urn: Urn,
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
        plugin_urn: Urn,
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

    /// Too many nodes are configured.
    #[error("Too many nodes defined")]
    TooManyNodes {},
}

impl Error {
    /// Returns the name of the error variant as a string.
    #[must_use]
    pub fn variant_name(&self) -> String {
        match self {
            Error::ConfigError(_) => "ConfigError",
            Error::ChannelRecvError(_) => "ChannelRecvError",
            Error::ChannelSendError { .. } => "ChannelSendError",
            Error::PipelineControlMsgError { .. } => "PipelineControlMsgError",
            Error::NodeControlMsgSendError { .. } => "NodeControlMsgSendError",
            Error::InvalidHyperEdge { .. } => "InvalidHyperEdge",
            Error::IoError { .. } => "IoError",
            Error::ReceiverAlreadyExists { .. } => "ReceiverAlreadyExists",
            Error::ReceiverError { .. } => "ReceiverError",
            Error::UnknownReceiver { .. } => "UnknownReceiver",
            Error::ProcessorAlreadyExists { .. } => "ProcessorAlreadyExists",
            Error::ProcessorError { .. } => "ProcessorError",
            Error::UnknownProcessor { .. } => "UnknownProcessor",
            Error::ExporterAlreadyExists { .. } => "ExporterAlreadyExists",
            Error::ExporterError { .. } => "ExporterError",
            Error::PdataConversionError { .. } => "PdataConversionError",
            Error::UnknownExporter { .. } => "UnknownExporter",
            Error::UnknownNode { .. } => "UnknownNode",
            Error::PdataReceiverNotSupported => "PdataReceiverNotSupported",
            Error::PdataSenderNotSupported => "PdataSenderNotSupported",
            Error::SpmcSharedNotSupported { .. } => "SpmcSharedNotSupported",
            Error::UnsupportedNodeKind { .. } => "UnsupportedNodeKind",
            Error::JoinTaskError { .. } => "JoinTaskError",
            Error::InternalError { .. } => "InternalError",
            Error::TooManyNodes {} => "TooManyNodes",
        }
        .to_owned()
    }
}
