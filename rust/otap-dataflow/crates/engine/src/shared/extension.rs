// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and types for shared (Send) extensions.

use crate::control::ExtensionControlMsg;
use crate::error::Error;
use crate::extension::EffectHandler;
use crate::shared::message::SharedReceiver;
use crate::terminal_state::TerminalState;
use async_trait::async_trait;

/// Shared (Send) control channel for extensions.
///
/// Wraps a [`SharedReceiver`] with shutdown-grace-period logic.
/// See [`crate::extension::ControlChannel`] for the implementation.
pub type ControlChannel = crate::extension::ControlChannel<SharedReceiver<ExtensionControlMsg>>;

/// A trait for pipeline extensions (Send variant).
///
/// Extensions are long-lived components that run alongside the pipeline and
/// expose functionality (e.g., authentication, service discovery) to other
/// components.
///
/// Unlike receivers, processors, and exporters, extensions are NOT generic over
/// PData — they never process pipeline data.
///
/// # Thread Safety
///
/// The shared `Extension` trait requires the `Send` bound, enabling use in both
/// single-threaded and multi-threaded runtime contexts.
#[async_trait]
pub trait Extension: Send {
    /// Starts the extension.
    ///
    /// Extensions are started BEFORE receivers, processors, and exporters.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if an unrecoverable error occurs.
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;
}
