// SPDX-License-Identifier: Apache-2.0

//! Abstraction to represent generic shared senders and receivers.

use otap_df_channel::error::{RecvError, SendError};

/// A generic shared channel Sender.
#[must_use = "A `Sender` is requested but not used."]
pub enum SharedSender<T> {
    /// Sender of a MPSC shared channel.
    MpscSender(tokio::sync::mpsc::Sender<T>),
    /// Sender of a MPMC shared channel.
    MpmcSender(flume::Sender<T>),
}

impl<T> Clone for SharedSender<T> {
    fn clone(&self) -> Self {
        match self {
            SharedSender::MpscSender(sender) => SharedSender::MpscSender(sender.clone()),
            SharedSender::MpmcSender(sender) => SharedSender::MpmcSender(sender.clone()),
        }
    }
}

impl<T> SharedSender<T> {
    /// Sends a message to the channel.
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        match self {
            SharedSender::MpscSender(sender) => {
                sender.send(msg).await.map_err(|e| SendError::Closed(e.0))
            }
            SharedSender::MpmcSender(sender) => {
                sender.send(msg).map_err(|e| SendError::Closed(e.0))
            }
        }
    }
}

/// A generic shared channel Receiver.
pub enum SharedReceiver<T> {
    /// Receiver of a MPSC shared channel.
    MpscReceiver(tokio::sync::mpsc::Receiver<T>),
    /// Receiver of a MPMC shared channel.
    MpmcReceiver(flume::Receiver<T>),
}

impl<T> SharedReceiver<T> {
    /// Receives a message from the channel.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        match self {
            SharedReceiver::MpscReceiver(receiver) => {
                receiver.recv().await.ok_or(RecvError::Closed)
            }
            SharedReceiver::MpmcReceiver(receiver) => {
                receiver.recv().map_err(|_| RecvError::Closed)
            }
        }
    }

    /// Tries to receive a message from the channel.
    pub fn try_recv(&mut self) -> Result<T, RecvError> {
        match self {
            SharedReceiver::MpscReceiver(receiver) => {
                receiver.try_recv().map_err(|_| RecvError::Closed)
            }
            SharedReceiver::MpmcReceiver(receiver) => {
                receiver.try_recv().map_err(|_| RecvError::Closed)
            }
        }
    }
}
