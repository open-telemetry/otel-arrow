// SPDX-License-Identifier: Apache-2.0

//! Errors for the pipeline engine.

use crate::NodeName;

/// All errors that can occur in the pipeline engine infrastructure.
#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    /// A wrapper for the channel errors.
    #[error("A channel error occurred: {0}")]
    ChannelRecvError(#[from] otap_df_channel::error::RecvError),

    /// A wrapper for the channel errors.
    #[error("A channel error occurred: {0}")]
    ChannelSendError(#[from] otap_df_channel::error::SendError<T>),

    /// A wrapper for the IO errors.
    #[error("An IO error occurred in node {node}: {error}")]
    IoError {
        /// The name of the node that encountered the error.
        node: NodeName,

        /// The error that occurred.
        error: std::io::Error,
    },

    /// A wrapper for the receiver errors.
    #[error("A receiver error occurred in node {receiver}: {error}")]
    ReceiverError {
        /// The name of the receiver that encountered the error.
        receiver: NodeName,

        /// The error that occurred.
        /// ToDo We probably need to use a more specific error type here (JSON Node?).
        error: String,
    },

    /// A wrapper for the processor errors.
    #[error("A processor error occurred in node {processor}: {error}")]
    ProcessorError {
        /// The name of the processor that encountered the error.
        processor: NodeName,

        /// The error that occurred.
        /// ToDo We probably need to use a more specific error type here (JSON Node?).
        error: String,
    },

    /// A wrapper for the exporter errors.
    #[error("An exporter error occurred in node {exporter}: {error}")]
    ExporterError {
        /// The name of the exporter that encountered the error.
        exporter: NodeName,

        /// The error that occurred.
        /// ToDo We probably need to use a more specific error type here (JSON Node?).
        error: String,
    },
}
