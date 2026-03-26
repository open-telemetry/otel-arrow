// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement processors.
//!
//! A processor is a node in the pipeline that transforms, filters, or otherwise processes messages
//! as they flow through the pipeline. Processors can perform operations such as:
//!
//! 1. Filtering messages based on certain criteria
//! 2. Transforming message content or format
//! 3. Aggregating multiple messages into a single message
//! 4. Splitting a single message into multiple messages
//! 5. Adding or removing attributes from messages
//!
//! # Lifecycle
//!
//! 1. The processor is instantiated and configured
//! 2. The processor receives and processes both data messages and control messages
//! 3. For each message, the processor can transform it, filter it, or split it into multiple messages
//! 4. The processor can maintain state between processing calls if needed
//! 5. The processor responds to control messages such as Config, TimerTick, or Shutdown
//! 6. The processor shuts down when it receives a `Shutdown` control message or encounters a fatal error
//!
//! # Thread Safety
//!
//! This implementation is designed for use in both single-threaded and multi-threaded environments.
//! The `Processor` trait requires the `Send` bound, enabling the use of thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own processor instance.

use crate::Interests;
use crate::control::{AckMsg, NackMsg, RuntimeCtrlMsgSender};
use crate::effect_handler::{
    EffectHandlerCore, SourceTagging, TelemetryTimerCancelHandle, TimerCancelHandle,
};
use crate::error::{Error, TypedError};
use crate::message::Message;
use crate::node::NodeId;
use crate::output_router::OutputRouter;
use crate::shared::message::SharedSender;
use async_trait::async_trait;
use otap_df_config::PortName;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A trait for processors in the pipeline (Send definition).
#[async_trait]
pub trait Processor<PData> {
    /// Processes a message and optionally produces effects, such as generating new pdata messages.
    ///
    /// This method is called by the pipeline engine for each message that arrives at the processor.
    /// Unlike receivers, processors have known inputs (messages from previous stages), so the pipeline
    /// engine can control when to call this method and when the processor executes.
    ///
    /// This approach allows for greater flexibility and optimization, giving the pipeline engine
    /// the ability to decide whether to spawn one task per processor or one task for a group of processors.
    /// The method signature uses `&mut self` rather than `Box<Self>` because the engine only wants to
    /// temporarily allow mutation of the processor instance, not transfer ownership.
    ///
    /// The processor can:
    /// - Transform the message and return a new message
    /// - Filter the message by returning None
    /// - Split the message into multiple messages by returning a vector
    /// - Handle control messages (e.g., Config, TimerTick, Shutdown)
    ///
    /// # Parameters
    ///
    /// - `msg`: The message to process, which can be either a data message or a control message
    /// - `effect_handler`: A handler to perform side effects such as sending messages to the next node.
    ///    This can be either Send or !Send depending on the processor's Mode type.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: The processor successfully processed the message
    /// - `Err(Error)`: The processor encountered an error and could not process the message
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the processor encounters an unrecoverable error.
    async fn process(
        &mut self,
        msg: Message<PData>,
        effect_handler: &mut EffectHandler<PData>,
    ) -> Result<(), Error>;

    /// Returns whether the engine should deliver pdata messages to this processor right now.
    ///
    /// When this returns `false` the engine pauses pdata delivery and only forwards control
    /// messages (acks/nacks) until the processor signals it is ready again. Defaults to `true`.
    fn accept_pdata(&self) -> bool {
        true
    }
}

/// A `Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    pub(crate) core: EffectHandlerCore<PData>,
    /// Output-port router.
    pub router: OutputRouter<SharedSender<PData>>,
}

/// Implementation for the `Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` with the given processor name and pdata sender.
    #[must_use]
    pub fn new(
        node_id: NodeId,
        msg_senders: HashMap<PortName, SharedSender<PData>>,
        default_port: Option<PortName>,
        metrics_reporter: MetricsReporter,
    ) -> Self {
        let core = EffectHandlerCore::new(node_id.clone(), metrics_reporter);
        let router = OutputRouter::new(node_id, msg_senders, default_port);
        EffectHandler { core, router }
    }

    /// Returns the id of the processor associated with this handler.
    #[must_use]
    pub fn processor_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Sets outgoing messages source tagging mode.
    pub fn set_source_tagging(&mut self, value: SourceTagging) {
        self.core.set_source_tagging(value);
    }

    /// Returns outgoing messages source tagging mode. Enabled when
    /// the destination node has multiple input sources.
    #[must_use]
    pub const fn source_tagging(&self) -> SourceTagging {
        self.core.source_tagging()
    }

    /// Returns the list of connected output ports for this processor.
    #[must_use]
    pub fn connected_ports(&self) -> Vec<PortName> {
        self.router.connected_ports()
    }

    /// Returns the precomputed node interests.
    #[must_use]
    pub fn node_interests(&self) -> Interests {
        self.core.node_interests()
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ProcessorError`] if the message could not be routed to a port.
    #[inline]
    pub async fn send_message(&self, data: PData) -> Result<(), TypedError<PData>> {
        self.router.send_default(data).await
    }

    /// Attempts to send a message without awaiting.
    ///
    /// Unlike `send_message`, this method returns immediately if the downstream
    /// channel is full, allowing the caller to handle backpressure without awaiting.
    ///
    /// # Errors
    ///
    /// Returns a [`TypedError::ChannelSendError`] containing [`SendError::Full`] if the
    /// channel is full, or [`SendError::Closed`] if the channel is closed.
    /// Returns a [`TypedError::Error`] if no default port is configured.
    #[inline]
    pub fn try_send_message(&self, data: PData) -> Result<(), TypedError<PData>> {
        self.router.try_send_default(data)
    }

    /// Sends a message to a specific named output port.
    #[inline]
    pub async fn send_message_to<P>(&self, port: P, data: PData) -> Result<(), TypedError<PData>>
    where
        P: Into<PortName>,
    {
        self.router.send_to(port, data).await
    }

    /// Attempts to send a message to a specific named output port without awaiting.
    ///
    /// Unlike `send_message_to`, this method returns immediately if the downstream
    /// channel is full, allowing the caller to handle backpressure without awaiting.
    ///
    /// # Errors
    ///
    /// Returns a [`TypedError::ChannelSendError`] containing [`SendError::Full`] if the
    /// channel is full, or [`SendError::Closed`] if the channel is closed.
    /// Returns a [`TypedError::Error`] if the port does not exist.
    #[inline]
    pub fn try_send_message_to<P>(&self, port: P, data: PData) -> Result<(), TypedError<PData>>
    where
        P: Into<PortName>,
    {
        self.router.try_send_to(port, data)
    }

    /// Print an info message to stdout.
    ///
    /// This method provides a standardized way for processors to output
    /// informational messages without blocking the async runtime.
    pub async fn info(&self, message: &str) {
        self.core.info(message).await;
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Current limitation: Only one timer can be started by a processor at a time.
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

    /// Send an Ack to the runtime control manager for context unwinding.
    pub async fn route_ack(&self, ack: AckMsg<PData>) -> Result<(), Error>
    where
        PData: crate::Unwindable,
    {
        self.core.route_ack(ack).await
    }

    /// Send a Nack to the runtime control manager for context unwinding.
    pub async fn route_nack(&self, nack: NackMsg<PData>) -> Result<(), Error>
    where
        PData: crate::Unwindable,
    {
        self.core.route_nack(nack).await
    }

    /// Delay data.
    pub async fn delay_data(&self, when: Instant, data: Box<PData>) -> Result<(), PData> {
        self.core.delay_data(when, data).await
    }

    /// Reports metrics collected by the processor.
    #[allow(dead_code)] // Will be used in the future. ToDo report metrics from channel and messages.
    pub(crate) fn report_metrics<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<(), TelemetryError> {
        self.core.report_metrics(metrics)
    }

    /// Sets the runtime control message sender for this effect handler.
    ///
    /// Primarily used by tests and manual harnesses that construct an EffectHandler directly;
    /// the engine wiring sets this automatically in `prepare_runtime`.
    pub fn set_runtime_ctrl_msg_sender(
        &mut self,
        runtime_ctrl_msg_sender: RuntimeCtrlMsgSender<PData>,
    ) {
        self.core
            .set_runtime_ctrl_msg_sender(runtime_ctrl_msg_sender);
    }

    // More methods will be added in the future as needed.
}

#[cfg(test)]
mod tests {
    #![allow(missing_docs)]
    use super::*;
    use crate::shared::message::SharedSender;
    use crate::testing::test_node;
    use otap_df_channel::error::SendError;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::collections::HashMap;

    #[test]
    fn effect_handler_try_send_message_success() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<u64>(10);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(
            test_node("proc"),
            senders,
            Some("out".into()),
            metrics_reporter,
        );

        // Should succeed when channel has capacity
        assert!(eh.try_send_message(42).is_ok());
        assert_eq!(rx.try_recv().unwrap(), 42);
    }

    #[test]
    fn effect_handler_try_send_message_channel_full() {
        // Create a channel with capacity 1
        let (tx, _rx) = tokio::sync::mpsc::channel::<u64>(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(
            test_node("proc"),
            senders,
            Some("out".into()),
            metrics_reporter,
        );

        // First send should succeed
        assert!(eh.try_send_message(1).is_ok());

        // Second send should fail with Full since channel capacity is 1
        let result = eh.try_send_message(2);
        assert!(matches!(
            result,
            Err(TypedError::ChannelSendError(SendError::Full(2)))
        ));
    }

    #[test]
    fn effect_handler_try_send_message_no_default_sender() {
        let (a_tx, _a_rx) = tokio::sync::mpsc::channel::<u64>(10);
        let (b_tx, _b_rx) = tokio::sync::mpsc::channel::<u64>(10);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), SharedSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), SharedSender::mpsc(b_tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        // No default port specified with multiple ports = ambiguous
        let eh = EffectHandler::new(test_node("proc"), senders, None, metrics_reporter);

        // Should return configuration error when no default sender
        let result = eh.try_send_message(99);
        assert!(matches!(result, Err(TypedError::Error(_))));
    }

    #[test]
    fn effect_handler_try_send_message_to_success() {
        let (a_tx, mut a_rx) = tokio::sync::mpsc::channel::<u64>(10);
        let (b_tx, mut b_rx) = tokio::sync::mpsc::channel::<u64>(10);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), SharedSender::mpsc(a_tx));
        let _ = senders.insert("b".into(), SharedSender::mpsc(b_tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(test_node("proc"), senders, None, metrics_reporter);

        // Should succeed when sending to a specific port
        assert!(eh.try_send_message_to("b", 42).is_ok());
        assert_eq!(b_rx.try_recv().unwrap(), 42);
        // Port 'a' should not have received anything
        assert!(a_rx.try_recv().is_err());
    }

    #[test]
    fn effect_handler_try_send_message_to_channel_full() {
        let (tx, _rx) = tokio::sync::mpsc::channel::<u64>(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(test_node("proc"), senders, None, metrics_reporter);

        // First send should succeed
        assert!(eh.try_send_message_to("out", 1).is_ok());
        // Second send should fail with Full
        let result = eh.try_send_message_to("out", 2);
        assert!(matches!(
            result,
            Err(TypedError::ChannelSendError(SendError::Full(2)))
        ));
    }

    #[test]
    fn effect_handler_try_send_message_to_unknown_port() {
        let (tx, _rx) = tokio::sync::mpsc::channel::<u64>(10);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(test_node("proc"), senders, None, metrics_reporter);

        // Should return error for unknown port
        let result = eh.try_send_message_to("unknown", 99);
        assert!(matches!(result, Err(TypedError::Error(_))));
    }
}
