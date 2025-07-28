// SPDX-License-Identifier: Apache-2.0

//! Errors for the pipeline engine.
//!
//! Important note: It is important not to use `!Send` data types in errors (e.g. avoid using Rc) to
//! ensure these errors can be emitted in both `Send` and `!Send` contexts.

use crate::control::ControlMsg;
use otap_df_channel::error::SendError;
use otap_df_config::{NodeId, PortName, Urn};
use std::borrow::Cow;

/// All errors that can occur in the pipeline engine infrastructure.
#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    /// A wrapper for the config errors.
    #[error("A config error occurred: {0}")]
    ConfigError(#[from] Box<otap_df_config::error::Error>),

    /// A wrapper for the channel errors.
    #[error("A channel error occurred: {0}")]
    ChannelRecvError(#[from] otap_df_channel::error::RecvError),

    /// A wrapper for the channel errors.
    #[error("A channel error occurred: {0}")]
    ChannelSendError(#[from] SendError<T>),

    /// A wrapper for the control message send errors.
    #[error("A control message send error occurred in node {node}: {error}")]
    ControlMsgSendError {
        /// The name of the node that encountered the error.
        node: NodeId,

        /// The error that occurred.
        error: SendError<ControlMsg>,
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
    #[error("A receiver error occurred in node {receiver}: {error}")]
    ReceiverError {
        /// The name of the receiver that encountered the error.
        receiver: NodeId,

        /// The error that occurred.
        /// ToDo We probably need to use a more specific error type here (JSON Node?).
        error: String,
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
    #[error("A processor error occurred in node {processor}: {error}")]
    ProcessorError {
        /// The name of the processor that encountered the error.
        processor: NodeId,

        /// The error that occurred.
        /// ToDo We probably need to use a more specific error type here (JSON Node?).
        error: String,
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
    #[error("An exporter error occurred in node {exporter}: {error}")]
    ExporterError {
        /// The name of the exporter that encountered the error.
        exporter: NodeId,

        /// The error that occurred.
        /// ToDo We probably need to use a more specific error type here (JSON Node?).
        error: String,
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
    #[error("Unknown node `{node_id}`")]
    UnknownNode {
        /// The id of the unknown node.
        node_id: NodeId,
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

    /// A task error that occurred during the execution of the pipeline.
    #[error("Task error: {error}, cancelled: {is_cancelled}, panic: {is_panic}")]
    TaskError {
        /// Flag indicating whether the task was cancelled.
        is_cancelled: bool,
        /// Flag indicating whether the task panicked.
        is_panic: bool,
        /// The error that occurred.
        error: String,
    },

    /// A list of errors that occurred during the execution of the pipeline.
    #[error("Errors detected during the execution of the engine: {errors:?}")]
    EngineErrors {
        /// A list of errors that occurred during the execution of the pipeline.
        errors: Vec<Error<T>>,
    },
}
