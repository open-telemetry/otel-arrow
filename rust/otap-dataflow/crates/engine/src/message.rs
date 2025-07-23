// SPDX-License-Identifier: Apache-2.0

//! Message definitions for the pipeline engine.

use crate::control::ControlMsg;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::shared::message::{SharedReceiver, SharedSender};
use otap_df_channel::error::{RecvError, SendError};
use otap_df_channel::mpsc;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::{Instant, Sleep, sleep_until};

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

/// A generic channel Sender supporting both local and shared semantic (i.e. !Send and Send).
#[must_use = "A `Sender` is requested but not used."]
pub enum Sender<T> {
    /// Sender of a local channel.
    Local(LocalSender<T>),
    /// Sender of a shared channel.
    Shared(SharedSender<T>),
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
    /// Creates a new local MPSC sender.
    pub fn new_local_mpsc_sender(mpsc_sender: mpsc::Sender<T>) -> Self {
        Sender::Local(LocalSender::MpscSender(mpsc_sender))
    }

    /// Sends a message to the channel.
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        match self {
            Sender::Local(sender) => sender.send(msg).await,
            Sender::Shared(sender) => sender.send(msg).await,
        }
    }
}

/// A generic channel Receiver supporting both local and shared semantic (i.e. !Send and Send).
pub enum Receiver<T> {
    /// Receiver of a local channel.
    Local(LocalReceiver<T>),
    /// Receiver of a shared channel.
    Shared(SharedReceiver<T>),
}

impl<T> Receiver<T> {
    /// Creates a new local MPMC receiver.
    pub fn new_local_mpsc_receiver(mpsc_receiver: mpsc::Receiver<T>) -> Self {
        Receiver::Local(LocalReceiver::MpscReceiver(mpsc_receiver))
    }

    /// Receives a message from the channel.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        match self {
            Receiver::Local(receiver) => receiver.recv().await,
            Receiver::Shared(receiver) => receiver.recv().await,
        }
    }

    /// Tries to receive a message from the channel.
    pub fn try_recv(&mut self) -> Result<T, RecvError> {
        match self {
            Receiver::Local(receiver) => receiver.try_recv(),
            Receiver::Shared(receiver) => receiver.try_recv(),
        }
    }
}

/// A channel for receiving control and pdata messages.
///
/// Control messages are prioritized until the first `Shutdown` is received.
/// After that, only pdata messages are considered, up to the deadline.
///
/// Note: This approach is used to implement a graceful shutdown. The engine will first close all
/// data sources in the pipeline, and then send a shutdown message with a deadline to all nodes in
/// the pipeline.
pub struct MessageChannel<PData> {
    control_rx: Option<Receiver<ControlMsg>>,
    pdata_rx: Option<Receiver<PData>>,
    /// Once a Shutdown is seen, this is set to `Some(instant)` at which point
    /// no more pdata will be accepted.
    shutting_down_deadline: Option<Instant>,
    /// Holds the ControlMsg::Shutdown until after we’ve drained pdata.
    pending_shutdown: Option<ControlMsg>,
}

impl<PData> MessageChannel<PData> {
    /// Creates a new `MessageChannel` with the given control and data receivers.
    #[must_use]
    pub fn new(control_rx: Receiver<ControlMsg>, pdata_rx: Receiver<PData>) -> Self {
        MessageChannel {
            control_rx: Some(control_rx),
            pdata_rx: Some(pdata_rx),
            shutting_down_deadline: None,
            pending_shutdown: None,
        }
    }

    /// Asynchronously receives the next message to process.
    ///
    /// Order of precedence:
    ///
    /// 1. Before a `Shutdown` is seen: control messages are always
    ///    returned ahead of pdata.
    /// 2. After the first `Shutdown` is received:
    ///    - All further control messages are silently discarded.
    ///    - Pending pdata are drained until the shutdown deadline.
    /// 3. When the deadline expires (or was `0`): the stored `Shutdown` is returned.
    ///    Subsequent calls return `RecvError::Closed`.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if both channels are closed, or if the
    /// shutdown deadline has passed.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        let mut sleep_until_deadline: Option<Pin<Box<Sleep>>> = None;

        loop {
            if self.control_rx.is_none() || self.pdata_rx.is_none() {
                // MessageChannel has been shutdown
                return Err(RecvError::Closed);
            }

            // Draining mode: Shutdown pending
            if let Some(dl) = self.shutting_down_deadline {
                // If the deadline has passed, emit the pending Shutdown now.
                if Instant::now() >= dl {
                    let shutdown = self
                        .pending_shutdown
                        .take()
                        .expect("pending_shutdown must exist");
                    self.shutdown();
                    return Ok(Message::Control(shutdown));
                }

                if sleep_until_deadline.is_none() {
                    // Create a sleep timer for the deadline
                    sleep_until_deadline = Some(Box::pin(sleep_until(dl)));
                }

                // Drain pdata first, then timer, then other control msgs
                tokio::select! {
                    biased;

                    // 1) Any pdata?
                    pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv() => match pdata {
                        Ok(pdata) => return Ok(Message::PData(pdata)),
                        Err(_) => {
                            // pdata channel closed → emit Shutdown
                            let shutdown = self.pending_shutdown
                                .take()
                                .expect("pending_shutdown must exist");
                            self.shutdown();
                            return Ok(Message::Control(shutdown));
                        }
                    },

                    // 2) Deadline hit?
                    _ = sleep_until_deadline.as_mut().expect("sleep_until_deadline must exist") => {
                        let shutdown = self.pending_shutdown
                            .take()
                            .expect("pending_shutdown must exist");
                        self.shutdown();
                        return Ok(Message::Control(shutdown));
                    }
                }
            }

            // Normal mode: no shutdown yet
            tokio::select! {
                biased;

                // A) Control first
                ctrl = self.control_rx.as_mut().expect("control_rx must exist").recv() => match ctrl {
                    Ok(ControlMsg::Shutdown { deadline, reason }) => {
                        if deadline.is_zero() {
                            // Immediate shutdown, no draining
                            self.shutdown();
                            return Ok(Message::Control(ControlMsg::Shutdown { deadline: Duration::ZERO, reason }));
                        }
                        // Begin draining mode, but don’t return Shutdown yet
                        let when = Instant::now() + deadline;
                        self.shutting_down_deadline = Some(when);
                        self.pending_shutdown = Some(ControlMsg::Shutdown { deadline, reason });
                        continue; // re-enter the loop into draining mode
                    }
                    Ok(msg) => return Ok(Message::Control(msg)),
                    Err(e)  => return Err(e),
                },

                // B) Then pdata
                pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv() => {
                    match pdata {
                        Ok(pdata) => {
                            return Ok(Message::PData(pdata));
                        }
                        Err(RecvError::Closed) => {
                            // pdata channel closed -> emit Shutdown
                            self.shutdown();
                            return Ok(Message::Control(ControlMsg::Shutdown {
                                deadline: Duration::ZERO,
                                reason: "pdata channel closed".to_owned(),
                            }));
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }

    fn shutdown(&mut self) {
        self.shutting_down_deadline = None;
        drop(self.control_rx.take().expect("control_rx must exist"));
        drop(self.pdata_rx.take().expect("pdata_rx must exist"));
    }
}
