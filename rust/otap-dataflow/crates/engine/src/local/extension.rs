// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement local extensions (!Send).
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
//! This implementation is designed to be used in a single-threaded environment.
//! The `Extension` trait does not require the `Send` bound on the returned future,
//! allowing for the use of non-thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own extension instance.

use crate::effect_handler::{EffectHandlerCore, TelemetryTimerCancelHandle, TimerCancelHandle};
use crate::error::Error;
use crate::extension::registry::TraitRegistration;
use crate::message::MessageChannel;
use crate::node::NodeId;
use crate::terminal_state::TerminalState;
use async_trait::async_trait;
use otap_df_telemetry::reporter::MetricsReporter;
use std::time::Duration;

/// A trait for pipeline extensions (!Send definition).
///
/// Extensions are long-lived components that run alongside the pipeline and
/// expose functionality (e.g., authentication, service discovery) to other
/// components through the [`ExtensionRegistry`](crate::extension::registry::ExtensionRegistry).
///
/// # Parameters
///
/// The `PData` type parameter is required for compatibility with the pipeline's
/// control message infrastructure, but extensions never process PData directly.
///
/// # Example
///
/// ```ignore
/// use async_trait::async_trait;
/// use otap_df_engine::local::extension::{Extension, EffectHandler};
/// use otap_df_engine::message::{Message, MessageChannel};
/// use otap_df_engine::control::NodeControlMsg;
/// use otap_df_engine::terminal_state::TerminalState;
/// use otap_df_engine::error::Error;
///
/// struct MyAuthExtension { /* ... */ }
///
/// #[async_trait(?Send)]
/// impl<PData> Extension<PData> for MyAuthExtension {
///     async fn start(
///         self: Box<Self>,
///         mut msg_chan: MessageChannel<PData>,
///         effect_handler: EffectHandler<PData>,
///     ) -> Result<TerminalState, Error> {
///         loop {
///             match msg_chan.recv().await? {
///                 Message::Control(NodeControlMsg::Shutdown { .. }) => break,
///                 _ => {}
///             }
///         }
///         Ok(TerminalState::default())
///     }
/// }
/// ```
#[async_trait(?Send)]
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
    /// # Parameters
    ///
    /// - `msg_chan`: A channel to receive control messages. Extensions do not
    ///   receive PData messages -- only control messages (shutdown, timer, config).
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
    ///
    /// Inside the override, use the [`extension_traits!`](crate::extension_traits!) macro:
    ///
    /// ```ignore
    /// fn extension_traits(&self) -> Vec<TraitRegistration> {
    ///     extension_traits!(self => BearerTokenProvider)
    /// }
    /// ```
    fn extension_traits(&self) -> Vec<TraitRegistration> {
        Vec::new()
    }
}

/// A `!Send` implementation of the EffectHandler for extensions.
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
    /// Creates a new local (!Send) `EffectHandler` for the given extension node.
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
