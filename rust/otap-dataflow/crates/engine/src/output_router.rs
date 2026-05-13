// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic output port router shared by all processor and receiver
//! `EffectHandler` variants (local/shared × processor/receiver).

use crate::StampOutputPort;
use crate::error::{Error, TypedError};
use crate::node::NodeId;
use otap_df_channel::error::SendError;
use otap_df_config::PortName;
use std::collections::HashMap;
use std::future::Future;

/// A generic crate::message::Sender or crate::shared::message::SharedSender.
pub trait OutputSend: Clone {
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

/// Port-routing state shared by all processor and receiver effect handlers.
#[derive(Clone)]
pub struct OutputRouter<S> {
    node_id: NodeId,
    /// Map of port name to sender and port_index.
    ports: HashMap<PortName, (S, u16)>,
    /// The default port name, sender, and port index.
    default: Option<(PortName, S, u16)>,
}

impl<S: Clone> OutputRouter<S> {
    /// Create a new router, indexing the ports and resolving the default sender.
    #[must_use]
    pub fn new(
        node_id: NodeId,
        msg_senders: HashMap<PortName, S>,
        default_port: Option<PortName>,
    ) -> Self {
        let mut entries: Vec<(PortName, S)> = msg_senders.into_iter().collect();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        let ports: HashMap<PortName, (S, u16)> = entries
            .into_iter()
            .enumerate()
            .map(|(i, (name, sender))| (name, (sender, i as u16)))
            .collect();

        let default = if let Some(ref port) = default_port {
            ports
                .get(port)
                .cloned()
                .map(|(sender, idx)| (port.clone(), sender, idx))
        } else if ports.len() == 1 {
            ports
                .iter()
                .next()
                .map(|(name, (sender, idx))| (name.clone(), sender.clone(), *idx))
        } else {
            None
        };

        Self {
            node_id,
            ports,
            default,
        }
    }

    /// Returns the list of connected output port names.
    #[must_use]
    pub fn connected_ports(&self) -> Vec<PortName> {
        let mut ports: Vec<_> = self.ports.keys().cloned().collect();
        ports.sort();
        ports
    }

    /// Returns the selected default output port name, if one exists.
    #[must_use]
    pub fn default_port(&self) -> Option<PortName> {
        self.default.as_ref().map(|(name, _, _)| name.clone())
    }
}

impl<S: OutputSend> OutputRouter<S> {
    /// Sends a message via the default output port.
    #[inline]
    pub async fn send_default(&self, data: S::Data) -> Result<(), TypedError<S::Data>> {
        match &self.default {
            Some((_, sender, _)) => sender
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
        match &self.default {
            Some((_, sender, _)) => sender
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
        match self.ports.get(&port_name) {
            Some((sender, _)) => sender
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
        match self.ports.get(&port_name) {
            Some((sender, _)) => sender
                .try_output_send(data)
                .map_err(TypedError::ChannelSendError),
            None => Err(TypedError::Error(Error::UnknownOutputPort {
                node: self.node_id.clone(),
                port: port_name,
            })),
        }
    }
}

impl<S: OutputSend> OutputRouter<S>
where
    S::Data: StampOutputPort,
{
    /// Stamps the output port index on data and sends via the default output port.
    #[inline]
    pub async fn send_default_stamped(&self, mut data: S::Data) -> Result<(), TypedError<S::Data>> {
        match &self.default {
            Some((_, sender, idx)) => {
                data.stamp_output_port_index(*idx);
                sender
                    .output_send(data)
                    .await
                    .map_err(TypedError::ChannelSendError)
            }
            None => Err(TypedError::Error(Error::NoDefaultOutputPort {
                node: self.node_id.clone(),
            })),
        }
    }

    /// Stamps the output port index and attempts to send via the default port without awaiting.
    #[inline]
    pub fn try_send_default_stamped(&self, mut data: S::Data) -> Result<(), TypedError<S::Data>> {
        match &self.default {
            Some((_, sender, idx)) => {
                data.stamp_output_port_index(*idx);
                sender
                    .try_output_send(data)
                    .map_err(TypedError::ChannelSendError)
            }
            None => Err(TypedError::Error(Error::NoDefaultOutputPort {
                node: self.node_id.clone(),
            })),
        }
    }

    /// Stamps the output port index and sends to a specific named output port.
    /// Performs a single hash-map lookup for both the port index and the sender.
    #[inline]
    pub async fn send_to_stamped<P: Into<PortName>>(
        &self,
        port: P,
        mut data: S::Data,
    ) -> Result<(), TypedError<S::Data>> {
        let port_name: PortName = port.into();
        match self.ports.get(&port_name) {
            Some((sender, idx)) => {
                data.stamp_output_port_index(*idx);
                sender
                    .output_send(data)
                    .await
                    .map_err(TypedError::ChannelSendError)
            }
            None => Err(TypedError::Error(Error::UnknownOutputPort {
                node: self.node_id.clone(),
                port: port_name,
            })),
        }
    }

    /// Stamps the output port index and attempts to send to a named port without awaiting.
    /// Performs a single hash-map lookup for both the port index and the sender.
    #[inline]
    pub fn try_send_to_stamped<P: Into<PortName>>(
        &self,
        port: P,
        mut data: S::Data,
    ) -> Result<(), TypedError<S::Data>> {
        let port_name: PortName = port.into();
        match self.ports.get(&port_name) {
            Some((sender, idx)) => {
                data.stamp_output_port_index(*idx);
                sender
                    .try_output_send(data)
                    .map_err(TypedError::ChannelSendError)
            }
            None => Err(TypedError::Error(Error::UnknownOutputPort {
                node: self.node_id.clone(),
                port: port_name,
            })),
        }
    }
}
