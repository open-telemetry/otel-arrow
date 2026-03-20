// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement shared exporters (Send bound).
//!
//! An exporter is an egress node that sends data from a pipeline to external systems, performing
//! the necessary conversions from the internal pdata format to the format required by the external
//! system.
//!
//! Exporters can operate in various ways, including:
//!
//! 1. Sending telemetry data to remote endpoints via network protocols,
//! 2. Writing data to files or databases,
//! 3. Pushing data to message queues or event buses,
//! 4. Or any other method of exporting telemetry data to external systems.
//!
//! # Lifecycle
//!
//! 1. The exporter is instantiated and configured
//! 2. The `start` method is called, which begins the exporter's operation
//! 3. The exporter processes both internal control messages and pipeline data (pdata)
//! 4. The exporter shuts down when it receives a `Shutdown` control message or encounters a fatal
//!    error
//!
//! # Thread Safety
//!
//! This implementation is designed for use in both single-threaded and multi-threaded environments.  
//! The `Exporter` trait requires the `Send` bound, enabling the use of thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own exporter instance.

use crate::control::{AckMsg, NackMsg, NodeControlMsg};
use crate::effect_handler::{EffectHandlerCore, TelemetryTimerCancelHandle, TimerCancelHandle};
use crate::error::Error;
use crate::message::Message;
use crate::node::NodeId;
use crate::shared::message::SharedReceiver;
use crate::terminal_state::TerminalState;
use crate::{Interests, ReceivedAtNode};
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::marker::PhantomData;
use std::pin::Pin;
use std::time::{Duration, Instant};
use tokio::time::{Sleep, sleep_until};

/// Maximum number of consecutive control messages delivered before the channel
/// forces one pdata attempt.
const CONTROL_BURST_LIMIT: usize = 32;

/// A trait for egress exporters (Send definition).
#[async_trait]
pub trait Exporter<PData> {
    /// Similar to local::exporter::Exporter::start, but operates in a Send context.
    async fn start(
        self: Box<Self>,
        msg_chan: MessageChannel<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<TerminalState, Error>;
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
    control_rx: Option<SharedReceiver<NodeControlMsg<PData>>>,
    pdata_rx: Option<SharedReceiver<PData>>,
    /// Once a Shutdown is seen, this is set to `Some(instant)` at which point
    /// no more pdata will be accepted.
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
        control_rx: SharedReceiver<NodeControlMsg<PData>>,
        pdata_rx: SharedReceiver<PData>,
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

    /// Asynchronously receives the next message to process.
    ///
    /// Order of precedence:
    ///
    /// 1. Before a `Shutdown` is seen: control messages are always
    ///    returned ahead of pdata.
    /// 2. After the first `Shutdown` is received:
    ///    - Control messages continue to be delivered while pdata drains.
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
                    sleep_until_deadline = Some(Box::pin(sleep_until(dl.into())));
                }

                if self.consecutive_control >= CONTROL_BURST_LIMIT {
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

                if self.consecutive_control >= CONTROL_BURST_LIMIT {
                    tokio::select! {
                        biased;

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

                        _ = sleep_until_deadline.as_mut().expect("sleep_until_deadline must exist") => {
                            let shutdown = self.pending_shutdown
                                .take()
                                .expect("pending_shutdown must exist");
                            self.shutdown();
                            return Ok(Message::Control(shutdown));
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
                        },

                        ctrl = self.control_rx.as_mut().expect("control_rx must exist").recv() => match ctrl {
                            Ok(msg) => return Ok(self.control_message(msg)),
                            Err(e) => return Err(e),
                        },

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
                    }
                }
            }

            // Normal mode: no shutdown yet
            if self.consecutive_control >= CONTROL_BURST_LIMIT {
                match self
                    .pdata_rx
                    .as_mut()
                    .expect("pdata_rx must exist")
                    .try_recv()
                {
                    Ok(pdata) => return Ok(self.pdata_message(pdata)),
                    Err(RecvError::Closed) => return Err(RecvError::Closed),
                    Err(RecvError::Empty) => {}
                }
            }

            if self.consecutive_control >= CONTROL_BURST_LIMIT {
                tokio::select! {
                    biased;

                    pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv() => {
                        match pdata {
                            Ok(pdata) => return Ok(self.pdata_message(pdata)),
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

                    pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv() => {
                        match pdata {
                            Ok(pdata) => return Ok(self.pdata_message(pdata)),
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

/// A `Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    pub(crate) core: EffectHandlerCore<PData>,
    _pd: PhantomData<PData>,
}

impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` with the given exporter node id and the metrics
    /// exporter.
    #[must_use]
    pub fn new(node_id: NodeId, metrics_reporter: MetricsReporter) -> Self {
        EffectHandler {
            core: EffectHandlerCore::new(node_id, metrics_reporter),
            _pd: PhantomData,
        }
    }

    /// Returns the id of the exporter associated with this handler.
    #[must_use]
    pub fn exporter_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Returns the precomputed node interests.
    #[must_use]
    pub fn node_interests(&self) -> Interests {
        self.core.node_interests()
    }

    /// Print an info message to stdout.
    ///
    /// This method provides a standardized way for exporters to output
    /// informational messages without blocking the async runtime.
    pub async fn info(&self, message: &str) {
        self.core.info(message).await;
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Current limitation: Only one timer can be started by an exporter at a time.
    pub async fn start_periodic_timer(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle<PData>, Error> {
        self.core.start_periodic_timer(duration).await
    }

    /// Starts a cancellable periodic telemetry timer that emits CollectTelemetry.
    pub async fn start_periodic_telemetry(
        &self,
        duration: Duration,
    ) -> Result<TelemetryTimerCancelHandle<PData>, Error> {
        self.core.start_periodic_telemetry(duration).await
    }

    /// Send an Ack to the pipeline controller for context unwinding.
    pub async fn route_ack(&self, ack: AckMsg<PData>) -> Result<(), Error>
    where
        PData: crate::Unwindable,
    {
        self.core.route_ack(ack).await
    }

    /// Send a Nack to the pipeline controller for context unwinding.
    pub async fn route_nack(&self, nack: NackMsg<PData>) -> Result<(), Error>
    where
        PData: crate::Unwindable,
    {
        self.core.route_nack(nack).await
    }

    /// Reports metrics collected by the exporter.
    #[allow(dead_code)] // Will be used in the future. ToDo report metrics from channel and messages.
    pub(crate) fn report_metrics<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<(), TelemetryError> {
        self.core.report_metrics(metrics)
    }

    // More methods will be added in the future as needed.
}
