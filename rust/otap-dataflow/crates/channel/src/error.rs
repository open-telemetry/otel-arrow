// SPDX-License-Identifier: Apache-2.0

//! Errors for the channels.

/// Errors that can occur when using the channels.
#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    /// The channel is full and the message could not be sent.
    #[error("Channel is full and the message could not be sent")]
    SendFull(T),

    /// The channel is closed and the message could not be sent.
    #[error("Channel is closed and the message could not be sent")]
    SendClosed(T),

    /// The channel associated with the receiver is closed.
    #[error("The channel associated with the receiver is closed")]
    RecvClosed,

    /// The channel associated with the receiver is empty.
    #[error("The channel associated with the receiver is empty")]
    RecvEmpty,
}