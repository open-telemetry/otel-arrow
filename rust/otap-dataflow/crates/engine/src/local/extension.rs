// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait for local (!Send) extensions.

use crate::error::Error;
use crate::extension::{ControlChannel, EffectHandler};
use crate::terminal_state::TerminalState;
use async_trait::async_trait;
use std::rc::Rc;

/// A trait for pipeline extensions (!Send variant).
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
/// The local `Extension` trait does NOT require the `Send` bound, allowing
/// use of `Rc`, `RefCell`, and other !Send types within a single-threaded
/// `LocalSet`.
///
/// # Ownership
///
/// `start` takes `Rc<Self>` so the same instance can serve both the background
/// task and capability consumers without cloning internal state.
#[async_trait(?Send)]
pub trait Extension {
    /// Starts the extension.
    ///
    /// Extensions are started BEFORE receivers, processors, and exporters.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if an unrecoverable error occurs.
    async fn start(
        self: Rc<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;
}
