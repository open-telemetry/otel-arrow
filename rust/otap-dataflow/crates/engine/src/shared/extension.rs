// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement shared extensions (Send bound).
//!
//! An extension is a special component that doesn't process pipeline data (pdata).
//! Extensions provide auxiliary services to the pipeline, such as health checks,
//! service discovery, or configuration management.
//!
//! Unlike receivers, processors, and exporters, extensions do not participate in
//! the data flow - they only handle control messages and provide services.
//!
//! # Lifecycle
//!
//! 1. The extension is instantiated and configured
//! 2. The `start` method is called, which begins the extension's operation
//! 3. The extension processes internal control messages
//! 4. The extension shuts down when it receives a `Shutdown` control message or encounters a fatal
//!    error
//!
//! # Thread Safety
//!
//! This implementation is designed for use in both single-threaded and multi-threaded environments.
//! The `Extension` trait requires the `Send` bound, enabling the use of thread-safe types.

use crate::control::{AckMsg, NackMsg, NodeControlMsg};
use crate::effect_handler::{EffectHandlerCore, TelemetryTimerCancelHandle, TimerCancelHandle};
use crate::error::Error;
use crate::extensions::{ExtensionError, ExtensionTrait};
use crate::message::Message;
use crate::node::NodeId;
use crate::shared::message::SharedReceiver;
use crate::terminal_state::TerminalState;
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::{Sleep, sleep_until};

/// A trait for extensions (Send definition).
///
/// Extensions are special components that don't process pipeline data.
/// They provide auxiliary services to the pipeline.
#[async_trait]
pub trait Extension<PData> {
    /// Similar to local::extension::Extension::start, but operates in a Send context.
    async fn start(
        self: Box<Self>,
        msg_chan: MessageChannel<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<TerminalState, Error>;
}

/// A channel for receiving control messages only (no pdata).
///
/// Extensions only process control messages, not pipeline data.
pub struct MessageChannel<PData> {
    control_rx: Option<SharedReceiver<NodeControlMsg<PData>>>,
    /// Once a Shutdown is seen, this is set to `Some(instant)` at which point
    /// the extension should finish up.
    shutting_down_deadline: Option<Instant>,
    /// Holds the ControlMsg::Shutdown until we're ready to return it.
    pending_shutdown: Option<NodeControlMsg<PData>>,
}

impl<PData> MessageChannel<PData> {
    /// Creates a new `MessageChannel` with the given control receiver.
    #[must_use]
    pub fn new(control_rx: SharedReceiver<NodeControlMsg<PData>>) -> Self {
        MessageChannel {
            control_rx: Some(control_rx),
            shutting_down_deadline: None,
            pending_shutdown: None,
        }
    }

    /// Asynchronously receives the next control message to process.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if the channel is closed, or if the
    /// shutdown deadline has passed.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        let mut sleep_until_deadline: Option<Pin<Box<Sleep>>> = None;

        loop {
            if self.control_rx.is_none() {
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

                // Wait for deadline or control messages
                tokio::select! {
                    biased;

                    // Timer expired
                    _ = async { sleep_until_deadline.as_mut().expect("sleep_until_deadline").as_mut().await }, if sleep_until_deadline.is_some() => {
                        let shutdown = self.pending_shutdown
                            .take()
                            .expect("pending_shutdown must exist");
                        self.shutdown();
                        return Ok(Message::Control(shutdown));
                    }

                    // Control messages (discard after shutdown)
                    ctrl = self.control_rx.as_mut().expect("control_rx must exist").recv() => match ctrl {
                        Ok(_) => {
                            // Discard control messages after shutdown is pending
                            continue;
                        }
                        Err(_) => {
                            let shutdown = self.pending_shutdown
                                .take()
                                .expect("pending_shutdown must exist");
                            self.shutdown();
                            return Ok(Message::Control(shutdown));
                        }
                    }
                }
            }

            // Normal mode: wait for control messages
            let ctrl_rx = self
                .control_rx
                .as_mut()
                .expect("control_rx must exist in normal mode");

            match ctrl_rx.recv().await {
                Ok(ctrl) => {
                    if let NodeControlMsg::Shutdown { deadline, .. } = &ctrl {
                        self.shutting_down_deadline = Some(*deadline);
                        self.pending_shutdown = Some(ctrl);
                        // Continue to handle the shutdown in draining mode
                        continue;
                    }
                    return Ok(Message::Control(ctrl));
                }
                Err(e) => {
                    self.shutdown();
                    return Err(e);
                }
            }
        }
    }

    /// Shuts down the message channel by dropping the control receiver.
    fn shutdown(&mut self) {
        self.control_rx = None;
    }
}

/// A `Send` implementation of the EffectHandler for extensions.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    pub(crate) core: EffectHandlerCore<PData>,
    _pd: PhantomData<PData>,
}

impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` with the given extension node id and metrics
    /// reporter.
    #[must_use]
    pub fn new(node_id: NodeId, metrics_reporter: MetricsReporter) -> Self {
        EffectHandler {
            core: EffectHandlerCore::new(node_id, metrics_reporter),
            _pd: PhantomData,
        }
    }

    /// Returns the id of the extension associated with this handler.
    #[must_use]
    pub fn extension_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Sets the extension registry for this effect handler.
    pub fn set_extension_registry(&mut self, registry: crate::extensions::ExtensionRegistry) {
        self.core.set_extension_registry(registry);
    }

    /// Returns an extension trait implementation by name.
    ///
    /// # Errors
    ///
    /// Returns an [`ExtensionError`] if the extension is not found or doesn't implement the trait.
    pub fn get_extension<T: ExtensionTrait + ?Sized + 'static>(
        &self,
        name: &str,
    ) -> Result<Arc<T>, ExtensionError>
    where
        Arc<T>: Send + Sync + Clone,
    {
        self.core.get_extension::<T>(name)
    }

    /// Print an info message to stdout.
    pub async fn info(&self, message: &str) {
        self.core.info(message).await;
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
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

    /// Send an Ack to a node of known-interest.
    pub async fn route_ack<F>(&self, ack: AckMsg<PData>, cxf: F) -> Result<(), Error>
    where
        F: FnOnce(AckMsg<PData>) -> Option<(usize, AckMsg<PData>)>,
    {
        self.core.route_ack(ack, cxf).await
    }

    /// Send a Nack to a node of known-interest.
    pub async fn route_nack<F>(&self, nack: NackMsg<PData>, cxf: F) -> Result<(), Error>
    where
        F: FnOnce(NackMsg<PData>) -> Option<(usize, NackMsg<PData>)>,
    {
        self.core.route_nack(nack, cxf).await
    }

    /// Reports metrics collected by the extension.
    #[allow(dead_code)] // Will be used in the future. ToDo report metrics from channel and messages.
    pub(crate) fn report_metrics<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<(), TelemetryError> {
        self.core.report_metrics(metrics)
    }
}
