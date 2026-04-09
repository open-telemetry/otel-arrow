// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait for shared (Send) extensions.

use crate::error::Error;
use crate::extension::{ControlChannel, EffectHandler};
use crate::terminal_state::TerminalState;
use async_trait::async_trait;

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
