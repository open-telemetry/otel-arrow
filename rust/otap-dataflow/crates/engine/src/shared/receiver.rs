// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement receivers.
//!
//! A receiver is an ingress node that feeds a pipeline with data from external sources while
//! performing the necessary conversions to produce messages in a format recognized by the rest of
//! downstream pipeline nodes (e.g. OTLP or OTAP message format).
//!
//! A receiver can operate in various ways, including:
//!
//! 1. Listening on a socket to receive push-based telemetry data,
//! 2. Being notified of changes in a local directory (e.g. log file monitoring),
//! 3. Actively scraping an endpoint to retrieve the latest metrics from a system,
//! 4. Or using any other method to receive or extract telemetry data from external sources.
//!
//! # Lifecycle
//!
//! 1. The receiver is instantiated and configured.
//! 2. The `start` method is called, which begins the receiver's operation.
//! 3. The receiver processes both internal control messages and external data.
//! 4. The receiver shuts down when it receives a `Shutdown` control message or encounters a fatal
//!    error.
//!
//! # Thread Safety
//!
//! This implementation is designed for use in both single-threaded and multi-threaded environments.
//! The `Exporter` trait requires the `Send` bound, enabling the use of thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline in
//! parallel on different cores, each with its own receiver instance.

use crate::control::{NodeControlMsg, PipelineCtrlMsgSender};
use crate::effect_handler::{EffectHandlerCore, TelemetryTimerCancelHandle, TimerCancelHandle};
use crate::error::{Error, TypedError};
use crate::node::NodeId;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::terminal_state::TerminalState;
use async_trait::async_trait;
use bytes::Bytes;
use otap_df_channel::error::RecvError;
use otap_df_config::PortName;
use otap_df_telemetry::InternalTelemetrySettings;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::event::ObservedEvent;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;

/// A trait for ingress receivers (Send definition).
///
/// Receivers are responsible for accepting data from external sources and converting
/// it into messages that can be processed by the pipeline.
#[async_trait]
pub trait Receiver<PData> {
    /// Similar to local::receiver::Receiver::start, but operates in a Send context.
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<TerminalState, Error>;
}

/// A channel for receiving control messages (in a Send environment).
///
/// This structure wraps a receiver end of a channel that carries [`NodeControlMsg`]
/// values used to control the behavior of a receiver at runtime.
pub struct ControlChannel<PData> {
    rx: SharedReceiver<NodeControlMsg<PData>>,
}

impl<PData> ControlChannel<PData> {
    /// Creates a new `ControlChannelShared` with the given receiver.
    #[must_use]
    pub fn new(rx: SharedReceiver<NodeControlMsg<PData>>) -> Self {
        Self { rx }
    }

    /// Asynchronously receives the next control message.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if the channel is closed.
    pub async fn recv(&mut self) -> Result<NodeControlMsg<PData>, RecvError> {
        self.rx.recv().await
    }
}

/// A `Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: EffectHandlerCore<PData>,

    /// A sender used to forward messages from the receiver.
    /// Supports multiple named output ports.
    msg_senders: HashMap<PortName, SharedSender<PData>>,
    /// Cached default sender for fast access in the hot path
    default_sender: Option<SharedSender<PData>>,
    /// Internal telemetry settings for the internal telemetry receiver.
    internal_telemetry: Option<InternalTelemetrySettings>,
}

/// Implementation for the `Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new sendable effect handler with the given receiver name.
    ///
    /// Use this constructor when your receiver do need to be sent across threads or
    /// when it uses components that are `Send`.
    #[must_use]
    pub fn new(
        node_id: NodeId,
        msg_senders: HashMap<PortName, SharedSender<PData>>,
        default_port: Option<PortName>,
        pipeline_ctrl_msg_sender: PipelineCtrlMsgSender<PData>,
        metrics_reporter: MetricsReporter,
    ) -> Self {
        let mut core = EffectHandlerCore::new(node_id, metrics_reporter);
        core.set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_sender);

        // Determine and cache the default sender
        let default_sender = if let Some(ref port) = default_port {
            msg_senders.get(port).cloned()
        } else if msg_senders.len() == 1 {
            msg_senders.values().next().cloned()
        } else {
            None
        };

        EffectHandler {
            core,
            msg_senders,
            default_sender,
            internal_telemetry: None,
        }
    }

    /// Returns the name of the receiver associated with this handler.
    #[must_use]
    pub fn receiver_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Returns the list of connected out ports for this receiver.
    #[must_use]
    pub fn connected_ports(&self) -> Vec<PortName> {
        self.msg_senders.keys().cloned().collect()
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ReceiverError`] if the message could not be routed to a port.
    #[inline]
    pub async fn send_message(&self, data: PData) -> Result<(), TypedError<PData>> {
        match &self.default_sender {
            Some(sender) => sender
                .send(data)
                .await
                .map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::NoDefaultOutPort {
                node: self.receiver_id(),
            })),
        }
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
        match &self.default_sender {
            Some(sender) => sender.try_send(data).map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::NoDefaultOutPort {
                node: self.receiver_id(),
            })),
        }
    }

    /// Sends a message to a specific named out port.
    #[inline]
    pub async fn send_message_to<P>(&self, port: P, data: PData) -> Result<(), TypedError<PData>>
    where
        P: Into<PortName>,
    {
        let port_name: PortName = port.into();
        match self.msg_senders.get(&port_name) {
            Some(sender) => sender
                .send(data)
                .await
                .map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::UnknownOutPort {
                node: self.receiver_id(),
                port: port_name,
            })),
        }
    }

    /// Attempts to send a message to a specific named out port without awaiting.
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
        let port_name: PortName = port.into();
        match self.msg_senders.get(&port_name) {
            Some(sender) => sender.try_send(data).map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::UnknownOutPort {
                node: self.receiver_id(),
                port: port_name,
            })),
        }
    }

    /// Creates a non-blocking TCP listener on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create TCP
    /// listeners via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    pub fn tcp_listener(&self, addr: SocketAddr) -> Result<TcpListener, Error> {
        self.core.tcp_listener(addr, self.receiver_id())
    }

    /// Print an info message to stdout.
    ///
    /// This method provides a standardized way for receivers to output
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

    /// Reports metrics collected by the receiver.
    #[allow(dead_code)] // Will be used in the future. ToDo report metrics from channel and messages.
    pub(crate) fn report_metrics<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<(), TelemetryError> {
        self.core.report_metrics(metrics)
    }

    /// Sets internal telemetry settings for the internal telemetry receiver.
    pub fn set_internal_telemetry(&mut self, settings: InternalTelemetrySettings) {
        self.internal_telemetry = Some(settings);
    }

    /// Returns the logs receiver for internal telemetry, if configured.
    #[must_use]
    pub fn logs_receiver(&self) -> Option<&flume::Receiver<ObservedEvent>> {
        self.internal_telemetry
            .as_ref()
            .map(|its| &its.logs_receiver)
    }

    /// Returns the resource bytes for internal telemetry, if configured.
    #[must_use]
    pub fn resource_bytes(&self) -> Option<&Bytes> {
        self.internal_telemetry
            .as_ref()
            .map(|its| &its.resource_bytes)
    }

    // More methods will be added in the future as needed.
}

#[cfg(test)]
mod tests {
    #![allow(missing_docs)]
    use super::*;
    use crate::control::pipeline_ctrl_msg_channel;
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

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(
            test_node("recv"),
            senders,
            Some("out".into()),
            ctrl_tx,
            metrics_reporter,
        );

        // Should succeed when channel has capacity
        assert!(eh.try_send_message(42).is_ok());
        assert_eq!(rx.try_recv().unwrap(), 42);
    }

    #[test]
    fn effect_handler_try_send_message_channel_full() {
        let (tx, _rx) = tokio::sync::mpsc::channel::<u64>(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("out".into(), SharedSender::mpsc(tx));

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(
            test_node("recv"),
            senders,
            Some("out".into()),
            ctrl_tx,
            metrics_reporter,
        );

        // First send should succeed
        assert!(eh.try_send_message(1).is_ok());
        // Second send should fail with Full
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

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(test_node("recv"), senders, None, ctrl_tx, metrics_reporter);

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

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(test_node("recv"), senders, None, ctrl_tx, metrics_reporter);

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

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(test_node("recv"), senders, None, ctrl_tx, metrics_reporter);

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

        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let eh = EffectHandler::new(test_node("recv"), senders, None, ctrl_tx, metrics_reporter);

        // Should return error for unknown port
        let result = eh.try_send_message_to("unknown", 99);
        assert!(matches!(result, Err(TypedError::Error(_))));
    }
}
