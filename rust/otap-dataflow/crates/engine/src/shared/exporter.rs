// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement shared exporters (Send bound).
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
//! This implementation is designed for use in both single-threaded and multi-threaded environments.  
//! The `Exporter` trait requires the `Send` bound, enabling the use of thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own exporter instance.

use crate::control::{AckMsg, NackMsg, NodeControlMsg};
use crate::effect_handler::{EffectHandlerCore, TelemetryTimerCancelHandle, TimerCancelHandle};
use crate::error::Error;
use crate::message::{Message, SharedExporterInbox};
use crate::node::NodeId;
use crate::shared::message::SharedReceiver;
use crate::terminal_state::TerminalState;
use crate::{Interests, ReceivedAtNode};
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_config::transport_headers_policy::HeaderPropagationPolicy;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::marker::PhantomData;
use std::time::Duration;

/// Send-friendly exporter inbox for shared exporter runtimes.
pub struct ExporterInbox<PData> {
    inner: SharedExporterInbox<PData>,
}

impl<PData> ExporterInbox<PData> {
    /// Creates a new shared exporter inbox.
    #[must_use]
    pub(crate) fn new(
        control_rx: SharedReceiver<NodeControlMsg<PData>>,
        pdata_rx: SharedReceiver<PData>,
        node_id: usize,
        interests: Interests,
    ) -> Self {
        Self {
            inner: SharedExporterInbox::new_internal(control_rx, pdata_rx, node_id, interests),
        }
    }
}

impl<PData: ReceivedAtNode> ExporterInbox<PData> {
    /// Receives the next message with pdata admission enabled.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        self.inner.recv_internal().await
    }

    /// Receives the next message. During shutdown draining, buffered pdata is
    /// drained even if normal exporter admission is currently closed.
    pub async fn recv_when(&mut self, accept_pdata: bool) -> Result<Message<PData>, RecvError> {
        self.inner.recv_when_internal(accept_pdata).await
    }
}

/// A trait for egress exporters (Send definition).
#[async_trait]
pub trait Exporter<PData> {
    /// Similar to local::exporter::Exporter::start, but operates in a Send context.
    async fn start(
        self: Box<Self>,
        inbox: ExporterInbox<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<TerminalState, Error>;
}

/// A `Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    pub(crate) core: EffectHandlerCore<PData>,
    _pd: PhantomData<PData>,
    /// Propagation policy for filtering captured headers on egress.
    /// `None` when no propagation policy is configured (zero overhead).
    propagation_policy: Option<HeaderPropagationPolicy>,
}

impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` with the given exporter node id and the metrics
    /// exporter.
    #[must_use]
    pub fn new(node_id: NodeId, metrics_reporter: MetricsReporter) -> Self {
        EffectHandler {
            core: EffectHandlerCore::new(node_id, metrics_reporter),
            _pd: PhantomData,
            propagation_policy: None,
        }
    }

    /// Returns the id of the exporter associated with this handler.
    #[must_use]
    pub fn exporter_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Returns the precomputed node interests.
    #[must_use]
    pub fn node_interests(&self) -> Interests {
        self.core.node_interests()
    }

    /// Returns the propagation policy if a header propagation policy is configured.
    ///
    /// Returns `None` when no propagation policy is active (zero overhead).
    #[must_use]
    pub fn propagation_policy(&self) -> Option<&HeaderPropagationPolicy> {
        self.propagation_policy.as_ref()
    }

    /// Sets the propagation policy for transport header filtering.
    pub fn set_propagation_policy(&mut self, policy: Option<HeaderPropagationPolicy>) {
        self.propagation_policy = policy;
    }

    /// Print an info message to stdout.
    ///
    /// This method provides a standardized way for exporters to output
    /// informational messages without blocking the async runtime.
    pub async fn info(&self, message: &str) {
        self.core.info(message).await;
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Current limitation: Only one timer can be started by an exporter at a time.
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

    /// Reports metrics collected by the exporter.
    #[allow(dead_code)] // Will be used in the future. ToDo report metrics from channel and messages.
    pub(crate) fn report_metrics<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<(), TelemetryError> {
        self.core.report_metrics(metrics)
    }

    // More methods will be added in the future as needed.

    /// Sets the pipeline result message sender for this effect handler.
    ///
    /// Primarily used by tests and manual harnesses that construct an EffectHandler directly;
    /// the engine wiring sets this automatically in `prepare_runtime`.
    pub fn set_pipeline_completion_msg_sender(
        &mut self,
        pipeline_completion_msg_sender: crate::control::PipelineCompletionMsgSender<PData>,
    ) {
        self.core
            .set_pipeline_completion_msg_sender(pipeline_completion_msg_sender);
    }
}

#[async_trait(?Send)]
impl<PData: crate::Unwindable> crate::_private::AckNackRouting<PData> for EffectHandler<PData> {
    async fn route_ack(&self, ack: AckMsg<PData>) -> Result<(), Error> {
        self.core.route_ack(ack).await
    }

    async fn route_nack(&self, nack: NackMsg<PData>) -> Result<(), Error> {
        self.core.route_nack(nack).await
    }
}
