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

use crate::effect_handler::SharedEffectHandlerCore;
use crate::error::Error;
use crate::message::ControlMsg;
use async_trait::async_trait;
use otap_df_channel::error::{RecvError, SendError};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::net::SocketAddr;
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
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler<PData>,
    ) -> Result<(), Error<PData>>;
}

/// A channel for receiving control messages (in a Send environment).
///
/// This structure wraps a receiver end of a channel that carries [`ControlMsg`]
/// values used to control the behavior of a receiver at runtime.
pub struct ControlChannel {
    rx: tokio::sync::mpsc::Receiver<ControlMsg>,
}

impl ControlChannel {
    /// Creates a new `ControlChannelShared` with the given receiver.
    #[must_use]
    pub fn new(rx: tokio::sync::mpsc::Receiver<ControlMsg>) -> Self {
        Self { rx }
    }

    /// Asynchronously receives the next control message.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if the channel is closed.
    pub async fn recv(&mut self) -> Result<ControlMsg, RecvError> {
        self.rx.recv().await.ok_or(RecvError::Closed)
    }
}

/// A `Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: SharedEffectHandlerCore,

    /// A sender used to forward messages from the receiver.
    msg_sender: tokio::sync::mpsc::Sender<PData>,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the receiver
    /// will produce.
    _pd: PhantomData<PData>,
}

/// Implementation for the `Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` with the given receiver name.
    #[must_use]
    pub fn new(
        receiver_name: Cow<'static, str>,
        msg_sender: tokio::sync::mpsc::Sender<PData>,
    ) -> Self {
        EffectHandler {
            core: SharedEffectHandlerCore {
                node_name: receiver_name,
                control_sender: None,
            },
            msg_sender,
            _pd: PhantomData,
        }
    }

    /// Creates a new shared (Send) `EffectHandler` with the given receiver name and control sender.
    #[must_use]
    pub fn with_control_sender(
        receiver_name: Cow<'static, str>,
        msg_sender: tokio::sync::mpsc::Sender<PData>,
        control_sender: tokio::sync::mpsc::Sender<ControlMsg>,
    ) -> Self {
        EffectHandler {
            core: SharedEffectHandlerCore {
                node_name: receiver_name,
                control_sender: Some(control_sender),
            },
            msg_sender,
            _pd: PhantomData,
        }
    }

    /// Returns the name of the receiver associated with this handler.
    #[must_use]
    pub fn receiver_name(&self) -> Cow<'static, str> {
        self.core.node_name()
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    pub async fn send_message(&self, data: PData) -> Result<(), Error<PData>> {
        self.msg_sender
            .send(data)
            .await
            .map_err(|tokio::sync::mpsc::error::SendError(pdata)| {
                Error::ChannelSendError(SendError::Full(pdata))
            })
    }

    /// Creates a non-blocking TCP listener on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create TCP
    /// listeners via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    pub fn tcp_listener(&self, addr: SocketAddr) -> Result<TcpListener, Error<PData>> {
        self.core.tcp_listener(addr, self.receiver_name())
    }

    // More methods will be added in the future as needed.
}
