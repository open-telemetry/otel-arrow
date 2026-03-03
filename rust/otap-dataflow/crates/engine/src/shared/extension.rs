// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures for shared (Send) extensions.
//!
//! This is the `Send`-bound counterpart of [`super::super::local::extension`].
//! Extensions using this module can be moved across thread boundaries, which
//! may be useful for integrations that require `tokio::spawn` (as opposed to
//! `spawn_local`).
//!
//! See the local extension module documentation for full lifecycle details.
//!
//! # PData Independence
//!
//! Like the local variant, shared extension types are **not** generic over
//! `PData`.  They use the PData-free [`ExtensionControlMsg`].

use crate::control::ExtensionControlMsg;
use crate::error::Error;
use crate::node::NodeId;
use crate::shared::message::SharedReceiver;
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_telemetry::reporter::MetricsReporter;

/// A trait for shared (Send) extensions.
#[async_trait]
pub trait Extension: Send {
    /// Starts the extension's background work (Send-compatible).
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<(), Error>;
}

/// A channel for receiving control messages in a `Send` extension.
pub struct ControlChannel {
    rx: SharedReceiver<ExtensionControlMsg>,
}

impl ControlChannel {
    /// Creates a new control channel.
    #[must_use]
    pub const fn new(rx: SharedReceiver<ExtensionControlMsg>) -> Self {
        Self { rx }
    }

    /// Receives the next control message, waiting if none is available.
    ///
    /// # Errors
    ///
    /// Returns [`RecvError`] if the channel is closed.
    pub async fn recv(&mut self) -> Result<ExtensionControlMsg, RecvError> {
        self.rx.recv().await
    }
}

/// A `Send` effect handler for extensions.
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
    /// Creates a new shared extension effect handler.
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
