// SPDX-License-Identifier: Apache-2.0

//! Errors for the dataflow engine.

use crate::NodeName;

/// All errors that can occur in the dataflow engine infrastructure.
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
        error: std::io::Error 
    },
}
