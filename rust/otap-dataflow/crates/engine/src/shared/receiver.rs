// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use otap_df_channel::error::{RecvError, SendError};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use async_trait::async_trait;
use crate::error::Error;
use crate::message::ControlMsg;
use crate::effect_handler::EffectHandlerCore;
use crate::local;

/// A trait for ingress receivers (Send definition).
///
/// Receivers are responsible for accepting data from external sources and converting
/// it into messages that can be processed by the pipeline.
#[async_trait]
pub trait Receiver<PData> {
    /// Similar to [`local::receiver::Receiver::start`], but operates in a Send context.
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

/// A `Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: EffectHandlerCore,

    /// A sender used to forward messages from the receiver.
    msg_sender: tokio::sync::mpsc::Sender<PData>,
}

/// Implementation for the `Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new sendable effect handler with the given receiver name.
    ///
    /// Use this constructor when your receiver do need to be sent across threads or
    /// when it uses components that are `Send`.
    #[must_use]
    pub fn new(
        receiver_name: Cow<'static, str>,
        msg_sender: tokio::sync::mpsc::Sender<PData>,
    ) -> Self {
        EffectHandler {
            core: EffectHandlerCore { node_name: receiver_name },
            msg_sender,
        }
    }

    /// Returns the name of the receiver associated with this handler.
    #[must_use]
    pub fn receiver_name(&self) -> &str {
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