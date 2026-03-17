// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Message definitions for the pipeline engine.

use crate::control::{AckMsg, NackMsg, NodeControlMsg};
use crate::local::message::{LocalReceiver, LocalSender};
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::{Interests, ReceivedAtNode};
use otap_df_channel::error::{RecvError, SendError};
use otap_df_channel::mpsc;
use std::ops::Add;
use std::pin::Pin;
use std::time::{Duration, Instant};
use tokio::time::{Sleep, sleep_until};

/// Maximum number of consecutive control messages delivered before the channel
/// forces one pdata attempt when pdata delivery is allowed.
const CONTROL_BURST_LIMIT: usize = 32;

/// Represents messages sent to nodes (receivers, processors, exporters, or connectors) within the
/// pipeline.
///
/// Messages are categorized as either pipeline data (`PData`) or control messages (`Control`).
#[derive(Debug, Clone)]
pub enum Message<PData> {
    /// A pipeline data message traversing the pipeline.
    PData(PData),

    /// A control message.
    Control(NodeControlMsg<PData>),
}

impl<Data> Message<Data> {
    /// Create a data message with the given payload.
    #[must_use]
    pub const fn data_msg(data: Data) -> Self {
        Message::PData(data)
    }

    /// Create a ACK control message with the given ID.
    #[must_use]
    pub const fn ack_ctrl_msg(ack: AckMsg<Data>) -> Self {
        Message::Control(NodeControlMsg::Ack(ack))
    }

    /// Create a NACK control message with the given ID and reason.
    #[must_use]
    pub const fn nack_ctrl_msg(nack: NackMsg<Data>) -> Self {
        Message::Control(NodeControlMsg::Nack(nack))
    }

    /// Creates a config control message with the given configuration.
    #[must_use]
    pub const fn config_ctrl_msg(config: serde_json::Value) -> Self {
        Message::Control(NodeControlMsg::Config { config })
    }

    /// Creates a timer tick control message.
    #[must_use]
    pub const fn timer_tick_ctrl_msg() -> Self {
        Message::Control(NodeControlMsg::TimerTick {})
    }

    /// Creates a shutdown control message with the given reason.
    #[must_use]
    pub fn shutdown_ctrl_msg(deadline: Instant, reason: &str) -> Self {
        Message::Control(NodeControlMsg::Shutdown {
            deadline,
            reason: reason.to_owned(),
        })
    }

    /// Checks if this message is a data message.
    #[must_use]
    pub const fn is_data(&self) -> bool {
        matches!(self, Message::PData(..))
    }

    /// Checks if this message is a control message.
    #[must_use]
    pub const fn is_control(&self) -> bool {
        matches!(self, Message::Control(..))
    }

    /// Checks if this message is a shutdown control message.
    #[must_use]
    pub const fn is_shutdown(&self) -> bool {
        matches!(self, Message::Control(NodeControlMsg::Shutdown { .. }))
    }
}

/// A generic channel Sender supporting both local and shared semantic (i.e. !Send and Send).
///
/// Rationale:
/// - Local nodes run on a single-threaded `LocalSet`, so it is safe for them to hold either a
///   local sender or a shared sender. This lets the engine select shared channels when any edge
///   requires `Send` (e.g. mixed local/shared fan-in) without extra wiring paths.
/// - Shared nodes keep `SharedSender` directly because their effect handlers must be `Send` to run
///   on multi-threaded executors (`tokio::spawn`). Wrapping in this enum would make them `!Send`
///   and introduce unnecessary branching on hot paths.
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
    pub const fn new_local_mpsc_sender(mpsc_sender: mpsc::Sender<T>) -> Self {
        Sender::Local(LocalSender::mpsc(mpsc_sender))
    }

    /// Sends a message to the channel.
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        match self {
            Sender::Local(sender) => sender.send(msg).await,
            Sender::Shared(sender) => sender.send(msg).await,
        }
    }

    /// Attempts to send a message without awaiting.
    pub fn try_send(&self, msg: T) -> Result<(), SendError<T>> {
        match self {
            Sender::Local(sender) => sender.try_send(msg),
            Sender::Shared(sender) => sender.try_send(msg),
        }
    }
}

/// A generic channel Receiver supporting both local and shared semantic (i.e. !Send and Send).
///
/// See [`Sender`] for the rationale behind using the enum in local contexts while keeping shared
/// nodes on `SharedReceiver` directly.
pub enum Receiver<T> {
    /// Receiver of a local channel.
    Local(LocalReceiver<T>),
    /// Receiver of a shared channel.
    Shared(SharedReceiver<T>),
}

impl<T> Receiver<T> {
    /// Creates a new local MPMC receiver.
    #[must_use]
    pub const fn new_local_mpsc_receiver(mpsc_receiver: mpsc::Receiver<T>) -> Self {
        Receiver::Local(LocalReceiver::mpsc(mpsc_receiver))
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

    /// Checks if the channel is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Receiver::Local(receiver) => receiver.is_empty(),
            Receiver::Shared(receiver) => receiver.is_empty(),
        }
    }
}

/// A channel for receiving control and pdata messages.
///
/// Control messages are prioritized until the first `Shutdown` is received.
/// After that, both control messages and pdata are considered up to the deadline,
/// with pdata gated by the `accept_pdata` flag passed to `recv_when`.
///
/// Note: This approach is used to implement a graceful shutdown. The engine will first close all
/// data sources in the pipeline, and then send a shutdown message with a deadline to all nodes in
/// the pipeline.
pub struct MessageChannel<PData> {
    control_rx: Option<Receiver<NodeControlMsg<PData>>>,
    pdata_rx: Option<Receiver<PData>>,
    /// Once a Shutdown is seen, this is set to `Some(instant)` representing the drain deadline.
    shutting_down_deadline: Option<Instant>,
    /// Holds the ControlMsg::Shutdown until after we’ve drained pdata.
    pending_shutdown: Option<NodeControlMsg<PData>>,
    /// Node ID for entry-frame stamping via `ReceivedAtNode`.
    node_id: usize,
    /// Node interests for entry-frame stamping via `ReceivedAtNode`.
    interests: Interests,
    /// Number of consecutive control messages delivered without a pdata message.
    consecutive_control: usize,
}

impl<PData> MessageChannel<PData> {
    /// Creates a new `MessageChannel` with the given control and data receivers.
    #[must_use]
    pub fn new(
        control_rx: Receiver<NodeControlMsg<PData>>,
        pdata_rx: Receiver<PData>,
        node_id: usize,
        interests: Interests,
    ) -> Self {
        MessageChannel {
            control_rx: Some(control_rx),
            pdata_rx: Some(pdata_rx),
            shutting_down_deadline: None,
            pending_shutdown: None,
            node_id,
            interests,
            consecutive_control: 0,
        }
    }
}

impl<PData: ReceivedAtNode> MessageChannel<PData> {
    fn control_message(&mut self, msg: NodeControlMsg<PData>) -> Message<PData> {
        self.consecutive_control = self.consecutive_control.saturating_add(1);
        Message::Control(msg)
    }

    fn pdata_message(&mut self, mut pdata: PData) -> Message<PData> {
        self.consecutive_control = 0;
        pdata.received_at_node(self.node_id, self.interests);
        Message::PData(pdata)
    }

    fn closed_pdata_shutdown(&mut self) -> Message<PData> {
        self.shutdown();
        Message::Control(NodeControlMsg::Shutdown {
            deadline: Instant::now().add(Duration::from_secs(1)),
            reason: "pdata channel closed".to_owned(),
        })
    }

    /// Asynchronously receives the next message to process.
    ///
    /// Order of precedence:
    ///
    /// 1. Before a `Shutdown` is seen: control messages are always
    ///    returned ahead of pdata.
    /// 2. After the first `Shutdown` is received (draining mode):
    ///    - Control messages (e.g. Ack/Nack) continue to be delivered so stateful
    ///      processors can reduce in-flight state and reopen capacity.
    ///    - Pending pdata are drained until the shutdown deadline, gated by `accept_pdata`.
    /// 3. When the deadline expires (or was `0`): the stored `Shutdown` is returned.
    ///    Subsequent calls return `RecvError::Closed`.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if both channels are closed, or if the
    /// shutdown deadline has passed.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        self.recv_when(true).await
    }

    /// Like [`recv()`](Self::recv), but with an `accept_pdata` guard.
    ///
    /// When `accept_pdata` is `false`, only control messages are
    /// returned. Pipeline data stays in the channel, providing
    /// natural backpressure to upstream nodes.
    ///
    /// During shutdown draining, pdata is drained until the deadline.
    /// The `accept_pdata` guard is still honored — when it is `false`,
    /// only control messages (e.g. Ack/Nack) are delivered so that
    /// stateful processors can reduce their in-flight state and
    /// reopen capacity for further draining.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if both channels are closed, or if the
    /// shutdown deadline has passed.
    pub async fn recv_when(&mut self, accept_pdata: bool) -> Result<Message<PData>, RecvError> {
        let mut sleep_until_deadline: Option<Pin<Box<Sleep>>> = None;

        loop {
            if self.control_rx.is_none() || self.pdata_rx.is_none() {
                // MessageChannel has been shutdown
                return Err(RecvError::Closed);
            }

            // When pdata is guarded (!accept_pdata), detect a closed pdata
            // channel eagerly so we don't block forever on control-only select.
            // We only probe when the buffer is empty — try_recv on an empty
            // channel distinguishes Closed from Empty without consuming data.
            if !accept_pdata
                && self
                    .pdata_rx
                    .as_ref()
                    .expect("pdata_rx must exist")
                    .is_empty()
            {
                if let Err(RecvError::Closed) = self
                    .pdata_rx
                    .as_mut()
                    .expect("pdata_rx must exist")
                    .try_recv()
                {
                    return Ok(self.closed_pdata_shutdown());
                }
            }

            // Draining mode: Shutdown pending
            if let Some(dl) = self.shutting_down_deadline {
                // If shutdown pending and no pdata left, return Shutdown immediately
                if self
                    .pdata_rx
                    .as_ref()
                    .expect("pdata_rx must exist")
                    .is_empty()
                {
                    let shutdown = self
                        .pending_shutdown
                        .take()
                        .expect("pending_shutdown must exist");
                    self.shutdown();
                    return Ok(Message::Control(shutdown));
                }

                if sleep_until_deadline.is_none() {
                    // Create a sleep timer for the deadline
                    sleep_until_deadline = Some(Box::pin(sleep_until(dl.into())));
                }

                if accept_pdata && self.consecutive_control >= CONTROL_BURST_LIMIT {
                    match self
                        .pdata_rx
                        .as_mut()
                        .expect("pdata_rx must exist")
                        .try_recv()
                    {
                        Ok(pdata) => return Ok(self.pdata_message(pdata)),
                        Err(RecvError::Closed) => {
                            let shutdown = self
                                .pending_shutdown
                                .take()
                                .expect("pending_shutdown must exist");
                            self.shutdown();
                            return Ok(Message::Control(shutdown));
                        }
                        Err(RecvError::Empty) => {}
                    }
                }

                // Drain pdata (gated by accept_pdata) and deliver control messages.
                // Honoring accept_pdata during draining lets stateful processors
                // receive Ack/Nack to reduce in-flight state and reopen capacity.
                if accept_pdata && self.consecutive_control >= CONTROL_BURST_LIMIT {
                    tokio::select! {
                        biased;

                        _ = sleep_until_deadline.as_mut().expect("sleep_until_deadline must exist") => {
                            let shutdown = self.pending_shutdown
                                .take()
                                .expect("pending_shutdown must exist");
                            self.shutdown();
                            return Ok(Message::Control(shutdown));
                        }

                        pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv() => match pdata {
                            Ok(pdata) => return Ok(self.pdata_message(pdata)),
                            Err(_) => {
                                let shutdown = self.pending_shutdown
                                    .take()
                                    .expect("pending_shutdown must exist");
                                self.shutdown();
                                return Ok(Message::Control(shutdown));
                            }
                        },

                        ctrl = self.control_rx.as_mut().expect("control_rx must exist").recv() => match ctrl {
                            Ok(msg) => return Ok(self.control_message(msg)),
                            Err(e) => return Err(e),
                        },
                    }
                } else {
                    tokio::select! {
                        biased;

                        _ = sleep_until_deadline.as_mut().expect("sleep_until_deadline must exist") => {
                            let shutdown = self.pending_shutdown
                                .take()
                                .expect("pending_shutdown must exist");
                            self.shutdown();
                            return Ok(Message::Control(shutdown));
                        }

                        ctrl = self.control_rx.as_mut().expect("control_rx must exist").recv() => match ctrl {
                            Ok(msg) => return Ok(self.control_message(msg)),
                            Err(e) => return Err(e),
                        },

                        pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv(), if accept_pdata => match pdata {
                            Ok(pdata) => return Ok(self.pdata_message(pdata)),
                            Err(_) => {
                                let shutdown = self.pending_shutdown
                                    .take()
                                    .expect("pending_shutdown must exist");
                                self.shutdown();
                                return Ok(Message::Control(shutdown));
                            }
                        },
                    }
                }
            }

            // Normal mode: no shutdown yet
            if accept_pdata && self.consecutive_control >= CONTROL_BURST_LIMIT {
                match self
                    .pdata_rx
                    .as_mut()
                    .expect("pdata_rx must exist")
                    .try_recv()
                {
                    Ok(pdata) => return Ok(self.pdata_message(pdata)),
                    Err(RecvError::Closed) => return Ok(self.closed_pdata_shutdown()),
                    Err(RecvError::Empty) => {}
                }
            }

            if accept_pdata && self.consecutive_control >= CONTROL_BURST_LIMIT {
                tokio::select! {
                    biased;

                    pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv() => {
                        match pdata {
                            Ok(pdata) => return Ok(self.pdata_message(pdata)),
                            Err(RecvError::Closed) => return Ok(self.closed_pdata_shutdown()),
                            Err(e) => return Err(e),
                        }
                    }

                    ctrl = self.control_rx.as_mut().expect("control_rx must exist").recv() => match ctrl {
                        Ok(NodeControlMsg::Shutdown { deadline, reason }) => {
                            if deadline.duration_since(Instant::now()).is_zero() {
                                self.shutdown();
                                return Ok(Message::Control(NodeControlMsg::Shutdown { deadline, reason }));
                            }
                            self.shutting_down_deadline = Some(deadline);
                            self.pending_shutdown = Some(NodeControlMsg::Shutdown { deadline, reason });
                            continue;
                        }
                        Ok(msg) => return Ok(self.control_message(msg)),
                        Err(e)  => return Err(e),
                    },
                }
            } else {
                tokio::select! {
                    biased;

                    ctrl = self.control_rx.as_mut().expect("control_rx must exist").recv() => match ctrl {
                        Ok(NodeControlMsg::Shutdown { deadline, reason }) => {
                            if deadline.duration_since(Instant::now()).is_zero() {
                                self.shutdown();
                                return Ok(Message::Control(NodeControlMsg::Shutdown { deadline, reason }));
                            }
                            self.shutting_down_deadline = Some(deadline);
                            self.pending_shutdown = Some(NodeControlMsg::Shutdown { deadline, reason });
                            continue;
                        }
                        Ok(msg) => return Ok(self.control_message(msg)),
                        Err(e)  => return Err(e),
                    },

                    pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv(), if accept_pdata => {
                        match pdata {
                            Ok(pdata) => return Ok(self.pdata_message(pdata)),
                            Err(RecvError::Closed) => return Ok(self.closed_pdata_shutdown()),
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        }
    }

    fn shutdown(&mut self) {
        self.shutting_down_deadline = None;
        self.consecutive_control = 0;
        drop(self.control_rx.take().expect("control_rx must exist"));
        drop(self.pdata_rx.take().expect("pdata_rx must exist"));
    }
}
