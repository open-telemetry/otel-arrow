// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement shared extensions (Send bound).
//!
//! An extension is a long-lived component that runs alongside the pipeline and
//! exposes functionality (e.g., authentication, service discovery) to other
//! components through the [`ExtensionRegistry`](crate::extension::registry::ExtensionRegistry).
//!
//! Unlike receivers, processors, and exporters, extensions:
//! - Do NOT process pipeline data (PData)
//! - Do NOT have input/output pdata channels
//! - Only receive control messages (shutdown, timer ticks, config updates)
//!
//! # Thread Safety
//!
//! This implementation is designed for use in both single-threaded and multi-threaded environments.
//! The `Extension` trait requires the `Send` bound, enabling the use of thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own extension instance.

use crate::control::NodeControlMsg;
use crate::effect_handler::{EffectHandlerCore, TelemetryTimerCancelHandle, TimerCancelHandle};
use crate::error::Error;
use crate::extension::registry::TraitRegistration;
use crate::message::Message;
use crate::node::NodeId;
use crate::shared::message::SharedReceiver;
use crate::terminal_state::TerminalState;
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_telemetry::reporter::MetricsReporter;
use std::pin::Pin;
use std::time::{Duration, Instant};
use tokio::time::{Sleep, sleep_until};

/// A trait for pipeline extensions (Send definition).
///
/// Extensions are long-lived components that run alongside the pipeline and
/// expose functionality (e.g., authentication, service discovery) to other
/// components through the [`ExtensionRegistry`](crate::extension::registry::ExtensionRegistry).
///
/// # Parameters
///
/// The `PData` type parameter is required for compatibility with the pipeline's
/// control message infrastructure, but extensions never process PData directly.
#[async_trait]
pub trait Extension<PData> {
    /// Starts the extension.
    ///
    /// The pipeline engine calls this to start the extension in a dedicated task.
    /// Extensions are started BEFORE receivers, processors, and exporters so that
    /// their capabilities are available when data-path components initialize.
    ///
    /// The extension is taken as `Box<Self>` so the method takes ownership once
    /// `start` is called. This lets it move into an independent task, after which
    /// the pipeline can only reach it through the control-message channel.
    ///
    /// Unlike the local variant, the returned future is `Send` (via `#[async_trait]`),
    /// enabling use in multi-threaded runtime contexts.
    ///
    /// # Parameters
    ///
    /// - `msg_chan`: A channel to receive control messages. Extensions do not
    ///   receive PData messages — only control messages (shutdown, timer, config).
    /// - `effect_handler`: A handler to perform side effects such as
    ///   timers and info logging.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if an unrecoverable error occurs.
    async fn start(
        self: Box<Self>,
        msg_chan: MessageChannel<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<TerminalState, Error>;

    /// Returns extension trait registrations for this extension.
    ///
    /// Override this method to publish traits that other pipeline components can
    /// consume via `registry.get::<dyn Trait>(name)`.  The default
    /// implementation returns an empty vec — suitable for pure background-task
    /// extensions that do not expose any traits.
    fn extension_traits(&self) -> Vec<TraitRegistration> {
        Vec::new()
    }
}

/// A channel for receiving control messages for shared extensions.
///
/// Extensions only receive control messages (shutdown, timer ticks, config updates).
/// They do not process pipeline data (PData).
///
/// Control messages are received until a `Shutdown` is seen. After that, the
/// channel transitions to a draining state and eventually returns the shutdown.
pub struct MessageChannel<PData> {
    control_rx: Option<SharedReceiver<NodeControlMsg<PData>>>,
    /// Once a Shutdown is seen, this is set to `Some(instant)` at which point
    /// no more messages will be accepted.
    shutting_down_deadline: Option<Instant>,
    /// Holds the ControlMsg::Shutdown until after we've finished draining.
    pending_shutdown: Option<NodeControlMsg<PData>>,
}

impl<PData> MessageChannel<PData> {
    /// Creates a new `MessageChannel` with the given control receiver.
    #[must_use]
    pub const fn new(control_rx: SharedReceiver<NodeControlMsg<PData>>) -> Self {
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
    /// Returns a [`RecvError`] if the channel is closed.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        let mut sleep_until_deadline: Option<Pin<Box<Sleep>>> = None;

        loop {
            if self.control_rx.is_none() {
                return Err(RecvError::Closed);
            }

            // Draining mode: Shutdown pending
            if let Some(dl) = self.shutting_down_deadline {
                if Instant::now() >= dl {
                    let shutdown = self
                        .pending_shutdown
                        .take()
                        .expect("pending_shutdown must exist");
                    self.shutdown();
                    return Ok(Message::Control(shutdown));
                }

                if sleep_until_deadline.is_none() {
                    sleep_until_deadline = Some(Box::pin(sleep_until(dl.into())));
                }

                tokio::select! {
                    biased;
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
                    Ok(msg) => return Ok(Message::Control(msg)),
                    Err(e)  => return Err(e),
                },
            }
        }
    }

    fn shutdown(&mut self) {
        self.shutting_down_deadline = None;
        drop(self.control_rx.take().expect("control_rx must exist"));
    }
}

/// A `Send` implementation of the EffectHandler for extensions.
///
/// Provides extensions with the ability to:
/// - Start periodic timers
/// - Print info messages
/// - Access node identity
#[derive(Clone)]
pub struct EffectHandler<PData> {
    pub(crate) core: EffectHandlerCore<PData>,
}

impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` for the given extension node.
    #[must_use]
    pub const fn new(node_id: NodeId, metrics_reporter: MetricsReporter) -> Self {
        EffectHandler {
            core: EffectHandlerCore::new(node_id, metrics_reporter),
        }
    }

    /// Returns the id of the extension associated with this handler.
    #[must_use]
    pub fn extension_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Print an info message to stdout.
    pub async fn info(&self, message: &str) {
        self.core.info(message).await;
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    pub async fn start_periodic_timer(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle<PData>, Error> {
        self.core.start_periodic_timer(duration).await
    }

    /// Starts a cancellable periodic telemetry timer.
    pub async fn start_periodic_telemetry(
        &self,
        duration: Duration,
    ) -> Result<TelemetryTimerCancelHandle<PData>, Error> {
        self.core.start_periodic_telemetry(duration).await
    }
}
