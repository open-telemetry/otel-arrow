// SPDX-License-Identifier: Apache-2.0

//! Message definitions for the pipeline engine.

use crate::error::Error;
use otap_df_channel::error::{RecvError, SendError};
use otap_df_channel::mpsc;
use std::time::Duration;

/// Represents messages sent to nodes (receivers, processors, exporters, or connectors) within the
/// pipeline.
///
/// Messages are categorized as either pipeline data (`PData`) or control messages (`Control`).
#[derive(Debug, Clone)]
pub enum Message<PData> {
    /// A pipeline data message traversing the pipeline.
    PData(PData),

    /// A control message.
    Control(ControlMsg),
}

/// Control messages used for managing pipeline operations and node behaviors.
#[derive(Debug, Clone)]
pub enum ControlMsg {
    /// Indicates that a downstream component (either internal or external) has reliably received
    /// and processed telemetry data.
    Ack {
        /// The ID of the message being acknowledged.
        id: u64,
    },

    /// Indicates that a downstream component (either internal or external) failed to process or
    /// deliver telemetry data. The NACK signal includes a reason, such as exceeding a deadline,
    /// downstream system unavailability, or other conditions preventing successful processing.
    Nack {
        /// The ID of the message not being acknowledged.
        id: u64,
        /// The reason for the NACK.
        reason: String,
    },

    /// Indicates a change in the configuration of a node. For example, a config message can
    /// instruct a Filter Processor to include or exclude certain attributes, or notify a Retry
    /// Processor to adjust backoff settings.
    Config {
        /// The new configuration.
        config: serde_json::Value,
    },

    /// Emitted upon timer expiration, used to trigger scheduled tasks (e.g., batch emissions).
    TimerTick {
        // TBD
    },

    /// A graceful shutdown message requiring the node to finish processing messages and release
    /// resources by a specified deadline. A deadline of 0 indicates an immediate shutdown.
    Shutdown {
        /// The deadline for the shutdown.
        deadline: Duration,
        /// The reason for the shutdown.
        reason: String,
    },
}

impl ControlMsg {
    /// Checks if this control message is a shutdown message.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        matches!(self, ControlMsg::Shutdown { .. })
    }
}

/// A generic sender for control messages.
pub enum ControlSender {
    /// A MPSC sender that does NOT implement [`Send`].
    Local(mpsc::Sender<ControlMsg>),
    /// A MPSC sender that implements [`Send`].
    Shared(tokio::sync::mpsc::Sender<ControlMsg>),
}

impl ControlSender {
    /// Sends a control message to the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send(&self, msg: ControlMsg) -> Result<(), Error<ControlMsg>> {
        match self {
            ControlSender::Local(sender) => sender
                .send_async(msg)
                .await
                .map_err(Error::ChannelSendError),
            ControlSender::Shared(sender) => sender
                .send(msg)
                .await
                .map_err(|e| Error::ChannelSendError(SendError::Closed(e.0))),
        }
    }
}

impl<Data> Message<Data> {
    /// Create a data message with the given payload.
    #[must_use]
    pub fn data_msg(data: Data) -> Self {
        Message::PData(data)
    }

    /// Create a ACK control message with the given ID.
    #[must_use]
    pub fn ack_ctrl_msg(id: u64) -> Self {
        Message::Control(ControlMsg::Ack { id })
    }

    /// Create a NACK control message with the given ID and reason.
    #[must_use]
    pub fn nack_ctrl_msg(id: u64, reason: &str) -> Self {
        Message::Control(ControlMsg::Nack {
            id,
            reason: reason.to_owned(),
        })
    }

    /// Creates a config control message with the given configuration.
    #[must_use]
    pub fn config_ctrl_msg(config: serde_json::Value) -> Self {
        Message::Control(ControlMsg::Config { config })
    }

    /// Creates a timer tick control message.
    #[must_use]
    pub fn timer_tick_ctrl_msg() -> Self {
        Message::Control(ControlMsg::TimerTick {})
    }

    /// Creates a shutdown control message with the given reason.
    #[must_use]
    pub fn shutdown_ctrl_msg(deadline: Duration, reason: &str) -> Self {
        Message::Control(ControlMsg::Shutdown {
            deadline,
            reason: reason.to_owned(),
        })
    }

    /// Checks if this message is a data message.
    #[must_use]
    pub fn is_data(&self) -> bool {
        matches!(self, Message::PData(..))
    }

    /// Checks if this message is a control message.
    #[must_use]
    pub fn is_control(&self) -> bool {
        matches!(self, Message::Control(..))
    }

    /// Checks if this message is a shutdown control message.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        matches!(self, Message::Control(ControlMsg::Shutdown { .. }))
    }
}

/// A MPSC receiver for pdata messages.
/// It can be either a not send or a send receiver implementation.
pub enum PDataReceiver<PData> {
    /// A MPSC receiver that does NOT implement [`Send`].
    NotSend(mpsc::Receiver<PData>),
    /// A MPSC receiver that implements [`Send`].
    Send(tokio::sync::mpsc::Receiver<PData>),
}

impl<PData> PDataReceiver<PData> {
    /// Returns the next message from the receiver.
    pub async fn recv(&mut self) -> Result<PData, Error<PData>> {
        match self {
            PDataReceiver::NotSend(receiver) => receiver
                .recv()
                .await
                .map_err(|e| Error::ChannelRecvError(e)),
            PDataReceiver::Send(receiver) => receiver
                .recv()
                .await
                .ok_or(Error::ChannelRecvError(RecvError::Closed)),
        }
    }

    /// Drains and returns all messages from the pdata receiver.
    pub async fn drain_pdata(&mut self) -> Vec<PData> {
        let mut emitted = Vec::new();
        match self {
            PDataReceiver::NotSend(receiver) => {
                while let Ok(msg) = receiver.try_recv() {
                    emitted.push(msg);
                }
            }
            PDataReceiver::Send(receiver) => {
                while let Ok(msg) = receiver.try_recv() {
                    emitted.push(msg);
                }
            }
        }
        emitted
    }
}

/// A generic channel Sender supporting both local and shared semantic (i.e. !Send and Send).
pub enum Sender<T> {
    /// Local channel sender.
    Local(mpsc::Sender<T>),
    /// Shared channel sender.
    Shared(tokio::sync::mpsc::Sender<T>),
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        match self {
            Sender::Local(sender) => Sender::Local(sender.clone()),
            Sender::Shared(sender) => Sender::Shared(sender.clone()),
        }
    }
}

impl<T> Sender<T> {
    /// Sends a message to the channel.
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        match self {
            Sender::Local(sender) => sender.send_async(msg).await,
            Sender::Shared(sender) => sender.send(msg).await.map_err(|e| SendError::Closed(e.0)),
        }
    }
}

/// A generic channel Receiver supporting both local and shared semantic (i.e. !Send and Send).
pub enum Receiver<T> {
    /// Local channel receiver.
    Local(mpsc::Receiver<T>),
    /// Shared channel receiver.
    Shared(tokio::sync::mpsc::Receiver<T>),
}

impl<T> Receiver<T> {
    /// Receives a message from the channel.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        match self {
            Receiver::Local(receiver) => receiver.recv().await,
            Receiver::Shared(receiver) => receiver.recv().await.ok_or(RecvError::Closed),
        }
    }
}
