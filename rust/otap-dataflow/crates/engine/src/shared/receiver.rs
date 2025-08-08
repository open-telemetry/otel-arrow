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
use crate::error::Error;
use crate::shared::message::{SharedReceiver, SharedSender};
use async_trait::async_trait;
use otap_df_channel::error::{RecvError, SendError};
use otap_df_config::{NodeId, PortName};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use std::collections::HashMap;

/// A trait for ingress receivers (Send definition).
///
/// Receivers are responsible for accepting data from external sources and converting
/// it into messages that can be processed by the pipeline.
#[async_trait]
pub trait Receiver<PData> {
    /// Similar to local::receiver::Receiver::start, but operates in a Send context.
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler<PData>,
    ) -> Result<(), Error<PData>>;
}

/// A channel for receiving control messages (in a Send environment).
///
/// This structure wraps a receiver end of a channel that carries [`NodeControlMsg`]
/// values used to control the behavior of a receiver at runtime.
pub struct ControlChannel {
    rx: SharedReceiver<NodeControlMsg>,
}

impl ControlChannel {
    /// Creates a new `ControlChannelShared` with the given receiver.
    #[must_use]
    pub fn new(rx: SharedReceiver<NodeControlMsg>) -> Self {
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

/// A `Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: EffectHandlerCore,

    /// A sender used to forward messages from the receiver.
    /// Supports multiple named output ports.
    msg_senders: HashMap<PortName, SharedSender<PData>>,
    /// Optional default port to use when calling send_message.
    default_port: Option<PortName>,
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
        EffectHandler { core, msg_senders, default_port }
    }

    /// Returns the name of the receiver associated with this handler.
    #[must_use]
    pub fn receiver_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Returns the list of connected out ports for this receiver.
    pub fn connected_ports(&self) -> Vec<PortName> {
        self.msg_senders.keys().cloned().collect()
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    pub async fn send_message(&self, data: PData) -> Result<(), SendError<PData>> {
        let port = if let Some(p) = &self.default_port {
            p.clone()
        } else if self.msg_senders.len() == 1 {
            self.msg_senders.keys().next().cloned().expect("non-empty")
        } else {
            return Err(SendError::Closed(data));
        };
        self.send_message_to(port, data).await
    }

    /// Sends a message to a specific named out port.
    pub async fn send_message_to<P>(&self, port: P, data: PData) -> Result<(), SendError<PData>>
    where
        P: Into<PortName>,
    {
        let port_name: PortName = port.into();
        let sender = match self.msg_senders.get(&port_name).cloned() {
            Some(s) => s,
            None => return Err(SendError::Closed(data)),
        };
        sender.send(data).await
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
    ) -> Result<TimerCancelHandle, Error<PData>> {
        self.core.start_periodic_timer(duration).await
    }

    // More methods will be added in the future as needed.
}
