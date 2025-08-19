// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// SPDX-License-Identifier: Apache-2.0

//! Abstraction to represent generic local senders and receivers.

use otap_df_channel::error::{RecvError, SendError};
use otap_df_channel::{mpmc, mpsc};

/// A generic local channel Sender.
#[must_use = "A `Sender` is requested but not used."]
pub enum LocalSender<T> {
    /// Sender of a MPSC local channel.
    MpscSender(mpsc::Sender<T>),
    /// Sender of a MPMC local channel.
    MpmcSender(mpmc::Sender<T>),
}

impl<T> Clone for LocalSender<T> {
    fn clone(&self) -> Self {
        match self {
            LocalSender::MpscSender(sender) => LocalSender::MpscSender(sender.clone()),
            LocalSender::MpmcSender(sender) => LocalSender::MpmcSender(sender.clone()),
        }
    }
}

impl<T> LocalSender<T> {
    /// Sends a message to the channel.
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        match self {
            LocalSender::MpscSender(sender) => sender.send_async(msg).await,
            LocalSender::MpmcSender(sender) => sender.send_async(msg).await,
        }
    }
}

/// A generic local channel Receiver.
pub enum LocalReceiver<T> {
    /// Receiver of a MPSC local channel.
    MpscReceiver(mpsc::Receiver<T>),
    /// Receiver of a MPMC local channel.
    MpmcReceiver(mpmc::Receiver<T>),
}

impl<T> LocalReceiver<T> {
    /// Receives a message from the channel.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        match self {
            LocalReceiver::MpscReceiver(receiver) => receiver.recv().await,
            LocalReceiver::MpmcReceiver(receiver) => receiver.recv().await,
        }
    }

    /// Tries to receive a message from the channel.
    pub fn try_recv(&mut self) -> Result<T, RecvError> {
        match self {
            LocalReceiver::MpscReceiver(receiver) => receiver.try_recv(),
            LocalReceiver::MpmcReceiver(receiver) => receiver.try_recv(),
        }
    }
}
