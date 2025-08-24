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

use crate::control::NodeControlMsg;
use crate::effect_handler::{EffectHandlerCore, TelemetryTimerCancelHandle, TimerCancelHandle};
use crate::error::Error;
use crate::message::Message;
use crate::node::NodeId;
use crate::shared::message::SharedReceiver;
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use std::marker::PhantomData;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::{Instant, Sleep, sleep_until};

/// A trait for egress exporters (Send definition).
#[async_trait]
pub trait Exporter<PData> {
    /// Similar to local::exporter::Exporter::start, but operates in a Send context.
    async fn start(
        self: Box<Self>,
        msg_chan: MessageChannel<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<(), Error<PData>>;
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
    control_rx: Option<SharedReceiver<NodeControlMsg>>,
    pdata_rx: Option<SharedReceiver<PData>>,
    /// Once a Shutdown is seen, this is set to `Some(instant)` at which point
    /// no more pdata will be accepted.
    shutting_down_deadline: Option<Instant>,
    /// Holds the ControlMsg::Shutdown until after we’ve drained pdata.
    pending_shutdown: Option<NodeControlMsg>,
}

impl<PData> MessageChannel<PData> {
    /// Creates a new `MessageChannel` with the given control and data receivers.
    #[must_use]
    pub fn new(
        control_rx: SharedReceiver<NodeControlMsg>,
        pdata_rx: SharedReceiver<PData>,
    ) -> Self {
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
                    Ok(NodeControlMsg::Shutdown { deadline, reason }) => {
                        if deadline.is_zero() {
                            // Immediate shutdown, no draining
                            self.shutdown();
                            return Ok(Message::Control(NodeControlMsg::Shutdown { deadline: Duration::ZERO, reason }));
                        }
                        // Begin draining mode, but don’t return Shutdown yet
                        let when = Instant::now() + deadline;
                        self.shutting_down_deadline = Some(when);
                        self.pending_shutdown = Some(NodeControlMsg::Shutdown { deadline: Duration::ZERO, reason });
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

/// A `Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    pub(crate) core: EffectHandlerCore,
    _pd: PhantomData<PData>,
}

impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` with the given exporter name.
    #[must_use]
    pub fn new(node_id: NodeId) -> Self {
        EffectHandler {
            core: EffectHandlerCore::new(node_id),
            _pd: PhantomData,
        }
    }

    /// Returns the id of the exporter associated with this handler.
    #[must_use]
    pub fn exporter_id(&self) -> NodeId {
        self.core.node_id()
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
    ) -> Result<TimerCancelHandle, Error<PData>> {
        self.core.start_periodic_timer(duration).await
    }

    /// Starts a cancellable periodic telemetry timer that emits CollectTelemetry.
    pub async fn start_periodic_telemetry(
        &self,
        duration: Duration,
    ) -> Result<TelemetryTimerCancelHandle, Error<PData>> {
        self.core.start_periodic_telemetry(duration).await
    }

    // More methods will be added in the future as needed.
}
