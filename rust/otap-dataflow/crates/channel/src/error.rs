// SPDX-License-Identifier: Apache-2.0

//! Errors for the channels.

/// Errors that can occur when sending on a channel.
#[derive(thiserror::Error, Debug)]
pub enum SendError<T> {
    /// The channel is full.
    #[error("Channel is full")]
    Full(T),
    /// The channel is closed.
    #[error("Channel is closed")]
    Closed(T),
}

/// Errors that can occur when receiving from a channel.
#[derive(thiserror::Error, Debug)]
pub enum RecvError {
    /// The channel is closed.
    #[error("Channel is closed")]
    Closed,
}

/// Errors that can occur when trying to receive from a channel.
#[derive(thiserror::Error, Debug)]
pub enum TryRecvError {
    /// The channel is empty.
    #[error("Channel is empty")]
    Empty,
    /// The channel is closed.
    #[error("Channel is closed")]
    Closed,
}