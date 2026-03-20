// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Message definitions for the pipeline engine.

use crate::clock;
use crate::control::{AckMsg, NackMsg, NodeControlMsg};
use crate::local::message::{LocalReceiver, LocalSender};
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::{Interests, ReceivedAtNode};
use otap_df_channel::error::{RecvError, SendError};
use otap_df_channel::mpsc;
use std::future::Future;
use std::ops::Add;
use std::time::{Duration, Instant};

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

/// Small private adapter trait used by [`MessageChannelCore`].
///
/// The core receive state machine is shared by:
///
/// - local processor/exporter channels, which use [`Receiver`]
/// - shared exporter channels, which use [`SharedReceiver`]
///
/// Rather than duplicating the shutdown/fairness logic for each concrete
/// receiver flavor, the core is generic over this minimal interface. The trait
/// stays private because it is an implementation detail of the channel split,
/// not part of the engine's public channel API.
trait ChannelReceiver<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, RecvError>> + '_;

    fn try_recv(&mut self) -> Result<T, RecvError>;

    fn is_empty(&self) -> bool;
}

impl<T> ChannelReceiver<T> for Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, RecvError>> + '_ {
        Receiver::recv(self)
    }

    fn try_recv(&mut self) -> Result<T, RecvError> {
        Receiver::try_recv(self)
    }

    fn is_empty(&self) -> bool {
        Receiver::is_empty(self)
    }
}

impl<T> ChannelReceiver<T> for SharedReceiver<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, RecvError>> + '_ {
        SharedReceiver::recv(self)
    }

    fn try_recv(&mut self) -> Result<T, RecvError> {
        SharedReceiver::try_recv(self)
    }

    fn is_empty(&self) -> bool {
        SharedReceiver::is_empty(self)
    }
}

/// Shutdown-drain policy for [`MessageChannelCore::recv_with_policy`].
///
/// Both processor and exporter channels share the same multiplexing and
/// shutdown machinery, but they intentionally diverge once shutdown has been
/// latched:
///
/// - processors keep honoring admission closure during drain, because
///   `accept_pdata()` is part of their existing engine-managed contract
/// - exporters force-drain already buffered channel data during drain, because
///   exporter-side admission is a self-managed operational choice rather than a
///   processor-style engine contract
///
/// This enum lets the shared core express that difference explicitly without
/// forking the whole receive loop.
enum DrainPolicy {
    /// Respect the caller's admission flag even after shutdown has been
    /// latched.
    HonorAdmission,
    /// Continue to respect normal admission before shutdown, but once shutdown
    /// is latched, allow buffered `pdata` to drain even if admission is
    /// currently closed.
    ForceDrainDuringShutdown,
}

struct MessageChannelCore<PData, ControlRx, PDataRx> {
    control_rx: Option<ControlRx>,
    pdata_rx: Option<PDataRx>,
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

impl<PData, ControlRx, PDataRx> MessageChannelCore<PData, ControlRx, PDataRx> {
    fn new(control_rx: ControlRx, pdata_rx: PDataRx, node_id: usize, interests: Interests) -> Self {
        Self {
            control_rx: Some(control_rx),
            pdata_rx: Some(pdata_rx),
            shutting_down_deadline: None,
            pending_shutdown: None,
            node_id,
            interests,
            consecutive_control: 0,
        }
    }

    fn shutdown(&mut self) {
        self.shutting_down_deadline = None;
        self.consecutive_control = 0;
        drop(self.control_rx.take().expect("control_rx must exist"));
        drop(self.pdata_rx.take().expect("pdata_rx must exist"));
    }
}

impl<PData, ControlRx, PDataRx> MessageChannelCore<PData, ControlRx, PDataRx>
where
    PData: ReceivedAtNode,
    ControlRx: ChannelReceiver<NodeControlMsg<PData>>,
    PDataRx: ChannelReceiver<PData>,
{
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
            deadline: clock::now().add(Duration::from_secs(1)),
            reason: "pdata channel closed".to_owned(),
        })
    }

    /// Returns whether shutdown draining is allowed to pull `pdata` from the
    /// bounded input channel.
    ///
    /// In normal operation, `accept_pdata` always controls admission. During
    /// shutdown, the answer becomes role-specific:
    ///
    /// - processors still honor `accept_pdata`
    /// - exporters switch to forced draining of already buffered channel data
    ///
    /// This method is the single point where that shutdown-time distinction is
    /// decided for the shared receive loop.
    fn shutdown_drain_accepts_pdata(accept_pdata: bool, policy: DrainPolicy) -> bool {
        accept_pdata || matches!(policy, DrainPolicy::ForceDrainDuringShutdown)
    }

    async fn recv_with_policy(
        &mut self,
        accept_pdata: bool,
        drain_policy: DrainPolicy,
    ) -> Result<Message<PData>, RecvError> {
        let mut sleep_until_deadline: Option<clock::Sleep> = None;

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
                let drain_accepts_pdata =
                    Self::shutdown_drain_accepts_pdata(accept_pdata, drain_policy);

                // Once shutdown has been latched, the stored Shutdown is released
                // only after the bounded pdata backlog is empty. This keeps the
                // channel-level drain contract explicit: upstream work that was
                // already accepted into the channel gets a chance to run first.
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
                    sleep_until_deadline = Some(clock::sleep_until(dl));
                }

                // Even while draining we cap control preference. This prevents a
                // sustained Ack/Nack or shutdown-control burst from starving the
                // already buffered pdata that shutdown is trying to drain.
                if drain_accepts_pdata && self.consecutive_control >= CONTROL_BURST_LIMIT {
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
                if drain_accepts_pdata && self.consecutive_control >= CONTROL_BURST_LIMIT {
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

                        pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv(), if drain_accepts_pdata => match pdata {
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
                            // The first Shutdown is latched instead of returned
                            // immediately. That switches the channel into
                            // shutdown-drain mode, where it keeps delivering
                            // cleanup control and buffered pdata until either the
                            // backlog empties or the deadline expires.
                            if deadline <= clock::now() {
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
                            // Same shutdown latching as above, but in the
                            // control-preferred branch used when pdata admission
                            // is currently closed or control has not yet hit the
                            // fairness limit.
                            if deadline <= clock::now() {
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
}

/// Processor-facing receive channel.
///
/// This preserves the existing processor contract: pdata admission is
/// controlled by the engine via `accept_pdata()`, and the admission guard
/// remains authoritative during shutdown draining.
pub struct ProcessorMessageChannel<PData> {
    core: MessageChannelCore<PData, Receiver<NodeControlMsg<PData>>, Receiver<PData>>,
}

impl<PData> ProcessorMessageChannel<PData> {
    /// Creates a new processor message channel.
    #[must_use]
    pub fn new(
        control_rx: Receiver<NodeControlMsg<PData>>,
        pdata_rx: Receiver<PData>,
        node_id: usize,
        interests: Interests,
    ) -> Self {
        Self {
            core: MessageChannelCore::new(control_rx, pdata_rx, node_id, interests),
        }
    }
}

impl<PData: ReceivedAtNode> ProcessorMessageChannel<PData> {
    /// Receives the next message while honoring the current processor
    /// admission state, including during shutdown draining.
    pub async fn recv_when(&mut self, accept_pdata: bool) -> Result<Message<PData>, RecvError> {
        self.core
            .recv_with_policy(accept_pdata, DrainPolicy::HonorAdmission)
            .await
    }
}

/// Exporter-facing receive channel.
///
/// Exporters own their receive loop directly. During shutdown draining,
/// buffered pdata is force-drained even when the exporter has temporarily
/// closed normal pdata admission.
pub struct ExporterMessageChannel<
    PData,
    ControlRx = Receiver<NodeControlMsg<PData>>,
    PDataRx = Receiver<PData>,
> {
    core: MessageChannelCore<PData, ControlRx, PDataRx>,
}

impl<PData, ControlRx, PDataRx> ExporterMessageChannel<PData, ControlRx, PDataRx> {
    #[must_use]
    pub(crate) fn new_internal(
        control_rx: ControlRx,
        pdata_rx: PDataRx,
        node_id: usize,
        interests: Interests,
    ) -> Self {
        Self {
            core: MessageChannelCore::new(control_rx, pdata_rx, node_id, interests),
        }
    }
}

#[allow(private_bounds)]
impl<PData, ControlRx, PDataRx> ExporterMessageChannel<PData, ControlRx, PDataRx>
where
    PData: ReceivedAtNode,
    ControlRx: ChannelReceiver<NodeControlMsg<PData>>,
    PDataRx: ChannelReceiver<PData>,
{
    pub(crate) async fn recv_internal(&mut self) -> Result<Message<PData>, RecvError> {
        self.recv_when_internal(true).await
    }

    pub(crate) async fn recv_when_internal(
        &mut self,
        accept_pdata: bool,
    ) -> Result<Message<PData>, RecvError> {
        self.core
            .recv_with_policy(accept_pdata, DrainPolicy::ForceDrainDuringShutdown)
            .await
    }
}

impl<PData> ExporterMessageChannel<PData> {
    /// Creates a new exporter message channel.
    #[must_use]
    pub fn new(
        control_rx: Receiver<NodeControlMsg<PData>>,
        pdata_rx: Receiver<PData>,
        node_id: usize,
        interests: Interests,
    ) -> Self {
        Self::new_internal(control_rx, pdata_rx, node_id, interests)
    }
}

impl<PData: ReceivedAtNode> ExporterMessageChannel<PData> {
    /// Receives the next message with pdata admission enabled.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        self.recv_internal().await
    }

    /// Receives the next message. During shutdown draining, buffered pdata is
    /// drained even if normal exporter admission is currently closed.
    pub async fn recv_when(&mut self, accept_pdata: bool) -> Result<Message<PData>, RecvError> {
        self.recv_when_internal(accept_pdata).await
    }
}

/// Send-friendly exporter channel type for shared exporter runtimes.
pub(crate) type SharedExporterMessageChannel<PData> =
    ExporterMessageChannel<PData, SharedReceiver<NodeControlMsg<PData>>, SharedReceiver<PData>>;
