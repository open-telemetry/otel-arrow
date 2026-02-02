// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement local extensions (!Send).
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
//! This implementation is designed to be used in a single-threaded environment.
//! The `Extension` trait does not require the `Send` bound, allowing for the use of non-thread-safe
//! types.

use crate::control::{AckMsg, NackMsg};
use crate::effect_handler::{EffectHandlerCore, TelemetryTimerCancelHandle, TimerCancelHandle};
use crate::error::Error;
use crate::message::MessageChannel;
use crate::node::NodeId;
use crate::terminal_state::TerminalState;
use async_trait::async_trait;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::marker::PhantomData;
use std::time::Duration;

/// A trait for extensions (!Send definition).
///
/// Extensions are special components that don't process pipeline data.
/// They provide auxiliary services to the pipeline.
#[async_trait(?Send)]
pub trait Extension<PData> {
    /// Starts the extension and begins its operation.
    ///
    /// The pipeline engine will call this function to start the extension in a separate task.
    /// Extensions are assigned their own dedicated task at pipeline initialization.
    ///
    /// The extension is taken as `Box<Self>` so the method takes ownership of the extension once `start` is called.
    /// This lets it move into an independent task, after which the pipeline can only
    /// reach it through the control-message channel.
    ///
    /// Extensions process control messages only - they do not receive or send pipeline data.
    ///
    /// # Parameters
    ///
    /// - `msg_chan`: A channel to receive control messages only (no pdata).
    /// - `effect_handler`: A handler to perform side effects.
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
    ) -> Result<TerminalState, Error>;
}

/// A `!Send` implementation of the EffectHandler for extensions.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    pub(crate) core: EffectHandlerCore<PData>,
    _pd: PhantomData<PData>,
}

impl<PData> EffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given extension node id and metrics
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

    /// Print an info message to stdout.
    ///
    /// This method provides a standardized way for extensions to output
    /// informational messages without blocking the async runtime.
    pub async fn info(&self, message: &str) {
        self.core.info(message).await;
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Current limitation: Only one timer can be started by an extension at a time.
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
