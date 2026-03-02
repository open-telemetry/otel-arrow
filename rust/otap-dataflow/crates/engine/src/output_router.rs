// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic output port router shared by all processor and receiver
//! `EffectHandler` variants (local/shared × processor/receiver).
//!
//! Factored from four nearly-identical implementations to eliminate
//! duplication of port-index bookkeeping, default-sender resolution,
//! and message-sending logic.

use crate::error::{Error, TypedError};
use crate::node::NodeId;
use otap_df_channel::error::SendError;
use otap_df_config::PortName;
use std::collections::HashMap;
use std::future::Future;

// ---------------------------------------------------------------------------
// Sender abstraction
// ---------------------------------------------------------------------------

/// Trait abstracting over local [`crate::message::Sender`] and
/// [`crate::shared::message::SharedSender`] so that `OutputRouter`
/// can be generic over send mode.
pub(crate) trait OutputSend: Clone {
    /// The data type carried by this sender.
    type Data;
    /// Asynchronously send a message.
    fn output_send(
        &self,
        msg: Self::Data,
    ) -> impl Future<Output = Result<(), SendError<Self::Data>>>;
    /// Try to send without blocking.
    fn try_output_send(&self, msg: Self::Data) -> Result<(), SendError<Self::Data>>;
}

impl<T> OutputSend for crate::message::Sender<T> {
    type Data = T;
    async fn output_send(&self, msg: T) -> Result<(), SendError<T>> {
        self.send(msg).await
    }
    fn try_output_send(&self, msg: T) -> Result<(), SendError<T>> {
        self.try_send(msg)
    }
}

impl<T> OutputSend for crate::shared::message::SharedSender<T> {
    type Data = T;
    async fn output_send(&self, msg: T) -> Result<(), SendError<T>> {
        self.send(msg).await
    }
    fn try_output_send(&self, msg: T) -> Result<(), SendError<T>> {
        self.try_send(msg)
    }
}

// ---------------------------------------------------------------------------
// OutputRouter
// ---------------------------------------------------------------------------

/// Port-routing state and message-sending logic shared by all
/// processor and receiver `EffectHandler` variants.
#[derive(Clone)]
pub(crate) struct OutputRouter<S> {
    node_id: NodeId,
    msg_senders: HashMap<PortName, S>,
    default_sender: Option<S>,
    default_port_index: u16,
    port_indices: HashMap<PortName, u16>,
}

impl<S: Clone> OutputRouter<S> {
    /// Create a new router, resolving the default sender and building
    /// the alphabetical port-index mapping.
    pub fn new(
        node_id: NodeId,
        msg_senders: HashMap<PortName, S>,
        default_port: Option<PortName>,
    ) -> Self {
        let port_indices = Self::build_port_indices(&msg_senders);

        let (default_sender, default_port_index) = if let Some(ref port) = default_port {
            (
                msg_senders.get(port).cloned(),
                port_indices.get(port).copied().unwrap_or(0),
            )
        } else if msg_senders.len() == 1 {
            (msg_senders.values().next().cloned(), 0)
        } else {
            (None, 0)
        };

        Self {
            node_id,
            msg_senders,
            default_sender,
            default_port_index,
            port_indices,
        }
    }

    /// Returns the list of connected output port names.
    #[must_use]
    pub fn connected_ports(&self) -> Vec<PortName> {
        self.msg_senders.keys().cloned().collect()
    }

    /// Returns the stable output port index for the default port.
    #[must_use]
    pub fn default_output_port_index(&self) -> u16 {
        self.default_port_index
    }

    /// Returns the stable output port index for a named port.
    #[must_use]
    pub fn output_port_index(&self, port: &PortName) -> u16 {
        self.port_indices.get(port).copied().unwrap_or(0)
    }

    /// Build a stable port-name → u16 mapping by sorting names alphabetically.
    fn build_port_indices(senders: &HashMap<PortName, S>) -> HashMap<PortName, u16> {
        let mut names: Vec<&PortName> = senders.keys().collect();
        names.sort();
        names
            .into_iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i as u16))
            .collect()
    }
}

impl<S: OutputSend> OutputRouter<S> {
    /// Sends a message via the default output port.
    #[inline]
    pub async fn send_default(&self, data: S::Data) -> Result<(), TypedError<S::Data>> {
        match &self.default_sender {
            Some(sender) => sender
                .output_send(data)
                .await
                .map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::NoDefaultOutputPort {
                node: self.node_id.clone(),
            })),
        }
    }

    /// Attempts to send a message via the default output port without awaiting.
    #[inline]
    pub fn try_send_default(&self, data: S::Data) -> Result<(), TypedError<S::Data>> {
        match &self.default_sender {
            Some(sender) => sender
                .try_output_send(data)
                .map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::NoDefaultOutputPort {
                node: self.node_id.clone(),
            })),
        }
    }

    /// Sends a message to a specific named output port.
    #[inline]
    pub async fn send_to<P: Into<PortName>>(
        &self,
        port: P,
        data: S::Data,
    ) -> Result<(), TypedError<S::Data>> {
        let port_name: PortName = port.into();
        match self.msg_senders.get(&port_name) {
            Some(sender) => sender
                .output_send(data)
                .await
                .map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::UnknownOutputPort {
                node: self.node_id.clone(),
                port: port_name,
            })),
        }
    }

    /// Attempts to send a message to a specific named output port without awaiting.
    #[inline]
    pub fn try_send_to<P: Into<PortName>>(
        &self,
        port: P,
        data: S::Data,
    ) -> Result<(), TypedError<S::Data>> {
        let port_name: PortName = port.into();
        match self.msg_senders.get(&port_name) {
            Some(sender) => sender
                .try_output_send(data)
                .map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::UnknownOutputPort {
                node: self.node_id.clone(),
                port: port_name,
            })),
        }
    }
}
