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
//! The `Receiver` trait requires the `Send` bound, enabling the use of thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline in
//! parallel on different cores, each with its own receiver instance.

use crate::context::NodeUniq;
use crate::control::{NodeControlMsg, PipelineCtrlMsgSender};
use crate::effect_handler::{EffectHandlerCore, TimerCancelHandle};
use crate::error::Error;
use crate::local::message::LocalSender;
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_config::{NodeId, PortName};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};

/// A trait for ingress receivers (!Send definition).
///
/// Receivers are responsible for accepting data from external sources and converting
/// it into messages that can be processed by the pipeline.
#[async_trait( ? Send)]
pub trait Receiver<PData> {
    /// Starts the receiver and begins processing incoming external data and control messages.
    ///
    /// The pipeline engine will call this function to start the receiver in a separate task.
    /// Receivers are assigned their own dedicated task at pipeline initialization because their
    /// primary function involves interacting with the external world, and the pipeline has no
    /// prior knowledge of when these interactions will occur.
    ///
    /// The `Box<Self>` signature indicates that when this method is called, the receiver takes
    /// exclusive ownership of its instance. This approach is necessary because a receiver cannot
    /// yield control back to the pipeline engine - it must independently manage its inputs and
    /// processing timing. The only way the pipeline engine can interact with the receiver after
    /// starting it is through the control message channel.
    ///
    /// Receivers are expected to process both internal control messages and external sources and
    /// use the EffectHandler to send messages to the next node(s) in the pipeline.
    ///
    /// Important note: Receivers are expected to process internal control messages in priority over
    /// external data.
    ///
    /// # Parameters
    ///
    /// - `ctrl_chan`: A channel to receive control messages.
    /// - `effect_handler`: A handler to perform side effects such as opening a listener.
    ///
    /// Each of these parameters is **NOT** [`Send`].
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
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler<PData>,
    ) -> Result<(), Error<PData>>;
}

/// A channel for receiving control messages (in a !Send environment).
///
/// This structure wraps a receiver end of a channel that carries [`NodeControlMsg`]
/// values used to control the behavior of a receiver at runtime.
pub struct ControlChannel {
    rx: crate::message::Receiver<NodeControlMsg>,
}

impl ControlChannel {
    /// Creates a new `ControlChannelLocal` with the given receiver.
    #[must_use]
    pub fn new(rx: crate::message::Receiver<NodeControlMsg>) -> Self {
        Self { rx }
    }

    /// Asynchronously receives the next control message.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if the channel is closed.
    pub async fn recv(&mut self) -> Result<NodeControlMsg, RecvError> {
        self.rx.recv().await
    }
}

/// A `!Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: EffectHandlerCore,

    /// A sender used to forward messages from the receiver.
    /// Supports multiple named output ports.
    msg_senders: HashMap<PortName, LocalSender<PData>>,
    /// Cached default sender for fast access in the hot path
    default_sender: Option<LocalSender<PData>>,
}

/// Implementation for the `!Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given receiver name and timer request sender.
    #[must_use]
    pub fn new(
        node: NodeUniq,
        msg_senders: HashMap<PortName, LocalSender<PData>>,
        default_port: Option<PortName>,
        node_request_sender: PipelineCtrlMsgSender,
    ) -> Self {
        let mut core = EffectHandlerCore::new(node);
        core.set_pipeline_ctrl_msg_sender(node_request_sender);

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
        }
    }

    /// Returns the id of the receiver associated with this handler.
    #[must_use]
    pub fn receiver_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Returns the list of connected out ports for this receiver.
    #[must_use]
    pub fn connected_ports(&self) -> Vec<PortName> {
        self.msg_senders.keys().cloned().collect()
    }

    /// Sends a message to the next node(s) in the pipeline using the default port.
    ///
    /// If a default port is configured (either explicitly or deduced when a single port is
    /// connected), it will be used. Otherwise, an error is returned.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent or [`Error::ReceiverError`]
    /// if the default port is not configured.
    #[inline]
    pub async fn send_message(&self, data: PData) -> Result<(), Error<PData>> {
        match &self.default_sender {
            Some(sender) => sender.send(data).await.map_err(Error::ChannelSendError),
            None => Err(Error::ReceiverError {
                receiver: self.receiver_id(),
                error:
                    "Ambiguous default out port: multiple ports connected and no default configured"
                        .to_string(),
            }),
        }
    }

    /// Sends a message to a specific named out port.
    ///
    /// # Errors
    ///
    /// Returns a [`Error::ChannelSendError`] if the message could not be sent, or
    /// [`Error::ReceiverError`] if the port does not exist.
    #[inline]
    pub async fn send_message_to<P>(&self, port: P, data: PData) -> Result<(), Error<PData>>
    where
        P: Into<PortName>,
    {
        let port_name: PortName = port.into();
        match self.msg_senders.get(&port_name) {
            Some(sender) => sender.send(data).await.map_err(Error::ChannelSendError),
            None => Err(Error::ReceiverError {
                receiver: self.receiver_id(),
                error: format!(
                    "Unknown out port '{port_name}' for node {}",
                    self.receiver_id()
                ),
            }),
        }
    }

    /// Creates a non-blocking TCP listener on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create TCP
    /// listeners via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    pub fn tcp_listener(&self, addr: SocketAddr) -> Result<TcpListener, Error<PData>> {
        self.core.tcp_listener(addr, self.receiver_id())
    }

    /// Creates a non-blocking UDP socket on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create UDP
    /// sockets via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    pub fn udp_socket(&self, addr: SocketAddr) -> Result<UdpSocket, Error<PData>> {
        self.core.udp_socket(addr, self.receiver_id())
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
    /// Current limitation: Only one timer can be started by a receiver at a time.
    pub async fn start_periodic_timer(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle, Error<PData>> {
        self.core.start_periodic_timer(duration).await
    }

    // More methods will be added in the future as needed.
}

#[cfg(test)]
mod tests {
    #![allow(missing_docs)]
    use super::*;
    use crate::context::NodeDefinition;
    use crate::control::pipeline_ctrl_msg_channel;
    use crate::local::message::LocalSender;
    use crate::runtime_pipeline::NodeType;
    use otap_df_channel::mpsc;
    use std::borrow::Cow;
    use std::collections::{HashMap, HashSet};
    use tokio::time::{Duration, timeout};

    fn channel<T>(capacity: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::Channel::new(capacity)
    }

    fn node_defs() -> (NodeUniq, Vec<NodeDefinition>) {
        let mut node_defs = Vec::new();
        let node =
            NodeUniq::next("recv".into(), NodeType::Receiver, &mut node_defs).expect("first");
        (node, node_defs)
    }

    #[tokio::test]
    async fn effect_handler_send_message_to_named_port() {
        let (a_tx, a_rx) = channel::<u64>(10);
        let (b_tx, b_rx) = channel::<u64>(10);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::MpscSender(a_tx));
        let _ = senders.insert("b".into(), LocalSender::MpscSender(b_tx));

        let (node, _) = node_defs();
        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let eh = EffectHandler::new(node, senders, None, ctrl_tx);

        eh.send_message_to("b", 42).await.unwrap();

        // Ensure only 'b' received
        assert!(
            timeout(Duration::from_millis(50), a_rx.recv())
                .await
                .is_err()
        );
        assert_eq!(b_rx.recv().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn effect_handler_send_message_single_port_fallback() {
        let (tx, rx) = channel::<u64>(10);
        let mut senders = HashMap::new();
        let _ = senders.insert("only".into(), LocalSender::MpscSender(tx));

        let (node, _) = node_defs();
        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let eh = EffectHandler::new(node, senders, None, ctrl_tx);

        eh.send_message(7).await.unwrap();
        assert_eq!(rx.recv().await.unwrap(), 7);
    }

    #[tokio::test]
    async fn effect_handler_send_message_uses_default_port() {
        let (a_tx, a_rx) = channel::<u64>(10);
        let (b_tx, b_rx) = channel::<u64>(10);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::MpscSender(a_tx));
        let _ = senders.insert("b".into(), LocalSender::MpscSender(b_tx));

        let (node, _) = node_defs();
        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let eh = EffectHandler::new(node, senders, Some("a".into()), ctrl_tx);

        eh.send_message(11).await.unwrap();

        assert_eq!(a_rx.recv().await.unwrap(), 11);
        assert!(
            timeout(Duration::from_millis(50), b_rx.recv())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn effect_handler_send_message_ambiguous_without_default() {
        let (a_tx, a_rx) = channel::<u64>(10);
        let (b_tx, b_rx) = channel::<u64>(10);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::MpscSender(a_tx));
        let _ = senders.insert("b".into(), LocalSender::MpscSender(b_tx));

        let (node, _) = node_defs();
        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let eh = EffectHandler::new(node, senders, None, ctrl_tx);

        let res = eh.send_message(5).await;
        assert!(res.is_err());

        // Nothing should be received on either port
        assert!(
            timeout(Duration::from_millis(50), a_rx.recv())
                .await
                .is_err()
        );
        assert!(
            timeout(Duration::from_millis(50), b_rx.recv())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn effect_handler_connected_ports_lists_all() {
        let (a_tx, _a_rx) = channel::<u64>(1);
        let (b_tx, _b_rx) = channel::<u64>(1);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::MpscSender(a_tx));
        let _ = senders.insert("b".into(), LocalSender::MpscSender(b_tx));

        let (node, _) = node_defs();
        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let eh = EffectHandler::new(node, senders, None, ctrl_tx);

        let ports: HashSet<_> = eh.connected_ports().into_iter().collect();
        let expected: HashSet<_> = [Cow::from("a"), Cow::from("b")].into_iter().collect();
        assert_eq!(ports, expected);
    }
}
