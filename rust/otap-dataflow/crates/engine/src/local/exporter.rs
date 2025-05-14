// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement local exporters (!Send).
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
//! This implementation is designed to be used in a single-threaded environment.
//! The `Exporter` trait does not require the `Send` bound, allowing for the use of non-thread-safe
//! types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own exporter instance.

use crate::effect_handler::EffectHandlerCore;
use crate::error::Error;
use crate::message::MessageChannel;
use async_trait::async_trait;
use std::borrow::Cow;
use std::marker::PhantomData;
/// A trait for egress exporters (!Send definition).
#[async_trait( ? Send)]
pub trait Exporter<PData> {
    /// Starts the exporter and begins exporting incoming data.
    ///
    /// The pipeline engine will call this function to start the exporter in a separate task.
    /// Exporters are assigned their own dedicated task at pipeline initialization because their
    /// primary function involves interacting with the external world, and the pipeline has no
    /// prior knowledge of when these interactions will occur.
    ///
    /// The exporter is taken as `Box<Self>` so the method takes ownership of the exporter once `start` is called.
    /// This lets it move into an independent task, after which the pipeline can only
    /// reach it through the control-message channel.
    ///
    /// Because ownership is now exclusive, the code inside `start` can freely use
    /// `&mut self` to update internal state without worrying about aliasing or
    /// borrowing rules at the call-site. That keeps the public API simple (no
    /// exterior `&mut` references to juggle) while still allowing the exporter to
    /// mutate itself as much as it needs during its run loop.
    ///
    /// Exporters are expected to process both internal control messages and pipeline data messages,
    /// prioritizing control messages over data messages. This prioritization guarantee is ensured
    /// by the `MessageChannel` implementation.
    ///
    /// # Parameters
    ///
    /// - `msg_chan`: A channel to receive pdata or control messages. Control messages are
    ///   prioritized over pdata messages.
    /// - `effect_handler`: A handler to perform side effects such as network operations.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if an unrecoverable error occurs.
    ///
    /// # Cancellation Safety
    ///
    /// This method should be cancellation safe and clean up any resources when dropped.
    async fn start(
        self: Box<Self>,
        msg_chan: MessageChannel<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<(), Error<PData>>;
}

/// A `!Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: EffectHandlerCore,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    _pd: PhantomData<PData>,
}

/// Implementation for the `!Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given exporter name.
    #[must_use]
    pub fn new(name: Cow<'static, str>) -> Self {
        EffectHandler {
            core: EffectHandlerCore { node_name: name },
            _pd: PhantomData,
        }
    }

    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    pub fn exporter_name(&self) -> Cow<'static, str> {
        self.core.node_name()
    }

    // More methods will be added in the future as needed.
}
