// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement local extensions (!Send).
//!
//! An extension is a long-lived component that runs alongside the pipeline and
//! exposes functionality (e.g., authentication, service discovery) to other
//! components through the [`CapabilityRegistry`](crate::extension::registry::CapabilityRegistry).
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

use crate::control::ExtensionControlMsg;
use crate::error::Error;
use crate::extension::registry::CapabilityRegistration;
use crate::local::message::LocalReceiver;
use crate::node::NodeId;
use crate::terminal_state::TerminalState;
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_telemetry::reporter::MetricsReporter;

/// A trait for pipeline extensions (!Send definition).
///
/// Extensions are long-lived components that run alongside the pipeline and
/// expose functionality (e.g., authentication, service discovery) to other
/// components through the [`CapabilityRegistry`](crate::extension::registry::CapabilityRegistry).
///
/// Unlike receivers, processors, and exporters, extensions are NOT generic over
/// PData — they never process pipeline data.
///
/// # Example
///
/// ```ignore
/// use async_trait::async_trait;
/// use otap_df_engine::local::extension::{Extension, EffectHandler, ControlChannel};
/// use otap_df_engine::control::ExtensionControlMsg;
/// use otap_df_engine::terminal_state::TerminalState;
/// use otap_df_engine::error::Error;
///
/// struct MyAuthExtension { /* ... */ }
///
/// #[async_trait(?Send)]
/// impl Extension for MyAuthExtension {
///     async fn start(
///         self: Box<Self>,
///         mut ctrl_chan: ControlChannel,
///         effect_handler: EffectHandler,
///     ) -> Result<TerminalState, Error> {
///         loop {
///             match ctrl_chan.recv().await? {
///                 ExtensionControlMsg::Shutdown { .. } => break,
///                 _ => {}
///             }
///         }
///         Ok(TerminalState::default())
///     }
/// }
/// ```
#[async_trait(?Send)]
pub trait Extension {
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
    /// - `ctrl_chan`: A channel to receive control messages. Extensions only
    ///   receive [`ExtensionControlMsg`] — never PData.
    /// - `effect_handler`: A handler to perform side effects such as
    ///   info logging.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if an unrecoverable error occurs.
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;

    /// Returns extension trait registrations for this extension.
    ///
    /// Override this method to publish traits that other pipeline components can
    /// consume via `registry.get::<dyn Trait>(name)`.  The default
    /// implementation returns an empty vec — suitable for pure background-task
    /// extensions that do not expose any traits.
    ///
    /// Inside the override, use the [`extension_capabilities!`](crate::extension_capabilities!) macro:
    ///
    /// ```ignore
    /// fn extension_capabilities(&self) -> Vec<CapabilityRegistration> {
    ///     extension_capabilities!(self => BearerTokenProvider)
    /// }
    /// ```
    fn extension_capabilities(&self) -> Vec<CapabilityRegistration> {
        Vec::new()
    }
}

/// A channel for receiving control messages for local extensions.
///
/// Extensions only receive control messages (shutdown, timer ticks, config updates).
/// They do not process pipeline data (PData).
///
/// Unlike the shared variant, there is no draining/deadline logic. A `Shutdown`
/// message is returned immediately when received. Extensions shut down last
/// (after data-plane nodes), so there is typically nothing left to drain.
pub struct ControlChannel {
    control_rx: Option<LocalReceiver<ExtensionControlMsg>>,
}

impl ControlChannel {
    /// Creates a new `ControlChannel` with the given control receiver.
    #[must_use]
    pub const fn new(control_rx: LocalReceiver<ExtensionControlMsg>) -> Self {
        ControlChannel {
            control_rx: Some(control_rx),
        }
    }

    /// Asynchronously receives the next control message.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if the channel is closed.
    pub async fn recv(&mut self) -> Result<ExtensionControlMsg, RecvError> {
        let rx = self.control_rx.as_mut().ok_or(RecvError::Closed)?;
        rx.recv().await
    }
}

/// A `!Send` implementation of the EffectHandler for extensions.
///
/// Provides extensions with the ability to:
/// - Print info messages
/// - Access node identity
///
/// Extensions manage their own timers directly via `tokio::time` rather than
/// through the engine's timer infrastructure, keeping the extension system
/// fully PData-free.
#[derive(Clone)]
pub struct EffectHandler {
    node_id: NodeId,
    #[allow(dead_code)]
    metrics_reporter: MetricsReporter,
}

impl EffectHandler {
    /// Creates a new local (!Send) `EffectHandler` for the given extension node.
    #[must_use]
    pub const fn new(node_id: NodeId, metrics_reporter: MetricsReporter) -> Self {
        EffectHandler {
            node_id,
            metrics_reporter,
        }
    }

    /// Returns the id of the extension associated with this handler.
    #[must_use]
    pub fn extension_id(&self) -> NodeId {
        self.node_id.clone()
    }

    /// Print an info message to stdout.
    pub async fn info(&self, message: &str) {
        use tokio::io::{AsyncWriteExt, stdout};
        let mut out = stdout();
        let _ = out.write_all(message.as_bytes()).await;
        let _ = out.write_all(b"\n").await;
        let _ = out.flush().await;
    }
}
