// SPDX-License-Identifier: Apache-2.0

//! Set of traits defining the common properties between all types of nodes in the pipeline engine.
//!
//! Receivers, processors, and exporters implement the [`Node`] trait.
//! Receivers and processors implement the [`NodeWithPDataSender`] trait.
//! Processors and exporters implement the [`NodeWithPDataReceiver`] trait.

use crate::control::NodeControlMsg;
use crate::error::Error;
use crate::message::{Receiver, Sender};
use otap_df_channel::error::SendError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::{NodeId, PortName};
use std::sync::Arc;

/// Common trait for nodes in the pipeline.
#[async_trait::async_trait(?Send)]
pub trait Node {
    /// Flag indicating whether the node is shared (true) or local (false).
    #[must_use]
    fn is_shared(&self) -> bool;

    /// Unique identifier.
    fn node_uniq(&self) -> NodeUnique;

    /// Returns a reference to the node's user configuration.
    #[must_use]
    fn user_config(&self) -> Arc<NodeUserConfig>;

    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: NodeControlMsg) -> Result<(), SendError<NodeControlMsg>>;
}

/// Trait for nodes that can send pdata to a specific port.
pub trait NodeWithPDataSender<PData>: Node {
    /// Sets the sender for pdata messages on the node.
    fn set_pdata_sender(
        &mut self,
        node: NodeId,
        port: PortName,
        sender: Sender<PData>,
    ) -> Result<(), Error<PData>>;
}

/// Trait for nodes that can receive pdata.
pub trait NodeWithPDataReceiver<PData>: Node {
    /// Sets the receiver for pdata messages on the node.
    fn set_pdata_receiver(
        &mut self,
        node: NodeId,
        receiver: Receiver<PData>,
    ) -> Result<(), Error<PData>>;
}

/// Node is defined ...
pub struct NodeDefinition {
    pub(crate) ntype: NodeType,
    pub(crate) name: NodeId,
}

/// Uniqueness value
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Unique(u16);

impl Unique {
    /// Index of this node in the runtime nodes vector.
    pub(crate) fn index(&self) -> usize {
        self.0 as usize
    }
}

impl TryFrom<usize> for Unique {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(u16::try_from(value)?))
    }
}

/// NodeUnique is a u16 consisting of NodeID plus uniqueness bits.
#[derive(Clone, Debug)]
pub struct NodeUnique {
    pub(crate) id: Unique,
    pub(crate) name: NodeId,
}

impl NodeUnique {
    /// Gets the next unique node identifier. Returns an error when the underlying
    /// u16 overflows.
    pub(crate) fn next(
        name: NodeId,
        ntype: NodeType,
        defs: &mut Vec<NodeDefinition>,
    ) -> Result<NodeUnique, std::num::TryFromIntError> {
        let uniq = Self {
            name: name.clone(),
            id: Unique::try_from(defs.len())?,
        };
        defs.push(NodeDefinition { ntype, name: name });
        Ok(uniq)
    }
}

/// Enum to identify the type of a node for registry lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Represents a node that acts as a receiver, receiving data from an external source.
    Receiver,
    /// Represents a node that processes data, transforming or analyzing it.
    Processor,
    /// Represents a node that exports data to an external destination.
    Exporter,
}
