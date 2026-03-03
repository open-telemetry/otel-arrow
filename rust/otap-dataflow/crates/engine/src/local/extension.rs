// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures for local (!Send) extensions.
//!
//! Extensions are non-pipeline components that provide cross-cutting capabilities
//! such as authentication, health checking, or service discovery. Unlike receivers,
//! processors, and exporters, extensions do not participate in the pdata flow.
//!
//! # Lifecycle
//!
//! 1. The extension is instantiated and configured by its factory.
//! 2. The factory creates **service handles** — lightweight, cloneable values —
//!    that are placed in the [`ExtensionRegistry`](crate::extensions::ExtensionRegistry).
//! 3. The `start` method is called, beginning the extension's background operation.
//! 4. Pipeline components retrieve handles from the registry (via their effect
//!    handler) and use them to interact with the extension.
//! 5. The extension shuts down when it receives a `Shutdown` control message or
//!    encounters a fatal error.
//!
//! # Thread Safety
//!
//! This implementation is designed for a single-threaded environment.
//! The `Extension` trait does not require the `Send` bound.
//!
//! # PData Independence
//!
//! Extension types are deliberately **not** generic over `PData`.  Extensions sit
//! outside the data-flow graph and never touch pipeline data, so they use the
//! PData-free [`ExtensionControlMsg`] instead of `NodeControlMsg<PData>`.

use crate::control::ExtensionControlMsg;
use crate::error::Error;
use crate::local::message::LocalReceiver;
use crate::node::NodeId;
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_telemetry::reporter::MetricsReporter;

/// A trait for local (!Send) extensions.
///
/// Extensions run as independent tasks alongside pipeline components.  They
/// receive only control messages (no pdata) and are started **before** the
/// rest of the pipeline, ensuring their service handles are ready for
/// consumers.
#[async_trait(?Send)]
pub trait Extension {
    /// Starts the extension's background work.
    ///
    /// The pipeline engine calls this once at startup. The extension should
    /// process control messages from `ctrl_chan` and perform its background
    /// duties until a `Shutdown` message is received.
    ///
    /// # Parameters
    ///
    /// - `ctrl_chan`: Channel for receiving control messages (Shutdown, TimerTick, etc.).
    /// - `effect_handler`: Provides node identity and logging.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if an unrecoverable failure occurs.
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<(), Error>;
}

/// A channel for receiving control messages in a `!Send` extension.
pub struct ControlChannel {
    ctrl_rx: LocalReceiver<ExtensionControlMsg>,
}

impl ControlChannel {
    /// Creates a new control channel.
    #[must_use]
    pub const fn new(ctrl_rx: LocalReceiver<ExtensionControlMsg>) -> Self {
        Self { ctrl_rx }
    }

    /// Receives the next control message, waiting if none is available.
    ///
    /// # Errors
    ///
    /// Returns [`RecvError`] if the channel is closed.
    pub async fn recv(&mut self) -> Result<ExtensionControlMsg, RecvError> {
        self.ctrl_rx.recv().await
    }
}

/// A `!Send` effect handler for extensions.
///
/// Provides a minimal set of capabilities — primarily node identity and logging.
/// Extensions that need periodic timers should use `tokio::time::interval` directly.
#[derive(Clone)]
pub struct EffectHandler {
    node_id: NodeId,
    #[allow(dead_code)]
    metrics_reporter: MetricsReporter,
}

impl EffectHandler {
    /// Creates a new local extension effect handler.
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
