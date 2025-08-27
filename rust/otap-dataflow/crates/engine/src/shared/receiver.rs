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
use crate::effect_handler::{EffectHandlerCore, TimerCancelHandle};
use crate::error::{Error, ErrorT};
use crate::node::NodeId;
use crate::shared::message::{SharedReceiver, SharedSender};
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_config::PortName;
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
    ) -> Result<(), Error>;
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
    core: EffectHandlerCore,

    /// A sender used to forward messages from the receiver.
    /// Supports multiple named output ports.
    msg_senders: HashMap<PortName, SharedSender<PData>>,
    /// Cached default sender for fast access in the hot path
    default_sender: Option<SharedSender<PData>>,
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
        pipeline_ctrl_msg_sender: PipelineCtrlMsgSender,
    ) -> Self {
        let mut core = EffectHandlerCore::new(node_id);
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
    pub async fn send_message(&self, data: PData) -> Result<(), ErrorT<PData>> {
        match &self.default_sender {
            Some(sender) => sender.send(data).await.map_err(ErrorT::ChannelSendError),
            None => Err(ErrorT::Error(Error::ReceiverError {
                receiver: self.receiver_id(),
                error:
                    "Ambiguous default out port: multiple ports connected and no default configured"
                        .to_string(),
            })),
        }
    }

    /// Sends a message to a specific named out port.
    #[inline]
    pub async fn send_message_to<P>(&self, port: P, data: PData) -> Result<(), ErrorT<PData>>
    where
        P: Into<PortName>,
    {
        let port_name: PortName = port.into();
        match self.msg_senders.get(&port_name) {
            Some(sender) => sender.send(data).await.map_err(ErrorT::ChannelSendError),
            None => Err(ErrorT::Error(Error::ReceiverError {
                receiver: self.receiver_id(),
                error: format!(
                    "Unknown out port '{port_name}' for node {}",
                    self.receiver_id()
                ),
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
    ) -> Result<TimerCancelHandle, Error> {
        self.core.start_periodic_timer(duration).await
    }

    // More methods will be added in the future as needed.
}
