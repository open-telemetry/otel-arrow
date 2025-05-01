// SPDX-License-Identifier: Apache-2.0

//! Errors for the channels.
//!
//! Important note: It is important not to use `!Send` data types in errors (e.g. avoid using Rc) to
//! ensure these errors can be emitted in both `Send` and `!Send` contexts.

/// Errors that can occur sending messages to a channel.
#[derive(thiserror::Error, Debug)]
pub enum SendError<T> {
    /// The channel is full and the message could not be sent.
    #[error("Channel is full and the message could not be sent")]
    Full(T),

    /// The channel is closed and the message could not be sent.
    #[error("Channel is closed and the message could not be sent")]
    Closed(T),
}

/// Errors that can occur when consuming messages from a channel.
#[derive(thiserror::Error, Debug)]
pub enum RecvError {
    /// The channel is closed.
    #[error("The channel is closed")]
    Closed,

    /// The channel is empty.
    #[error("The channel is empty")]
    Empty,
}
