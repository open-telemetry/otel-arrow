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
use std::marker::PhantomData;
use std::sync::Arc;

/// Common trait for nodes in the pipeline.
#[async_trait::async_trait(?Send)]
pub trait Node {
    /// Flag indicating whether the node is shared (true) or local (false).
    #[must_use]
    fn is_shared(&self) -> bool;

    /// Unique identifier.
    fn unique(&self) -> NodeUnique;

    /// Returns a reference to the node's user configuration.
    #[must_use]
    fn user_config(&self) -> Arc<NodeUserConfig>;

    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: NodeControlMsg) -> Result<(), SendError<NodeControlMsg>>;
}

/// NodeUnique consists of NodeId and Unique integer.
#[derive(Clone, Debug)]
pub struct NodeUnique {
    /// A unique integer.
    pub(crate) id: Unique,

    /// A unique name as defined by otap_df_config.
    pub name: NodeId,
}

/// Uniqueness value, presently a u16.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Unique(u16);

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

/// Trait for nodes that can send pdata to a specific port.
pub trait NodeWithPDataSender<PData>: Node {
    /// Sets the sender for pdata messages on the node.
    fn set_pdata_sender(
        &mut self,
        node: NodeUnique,
        port: PortName,
        sender: Sender<PData>,
    ) -> Result<(), Error<PData>>;
}

/// Trait for nodes that can receive pdata.
pub trait NodeWithPDataReceiver<PData>: Node {
    /// Sets the receiver for pdata messages on the node.
    fn set_pdata_receiver(
        &mut self,
        node: NodeUnique,
        receiver: Receiver<PData>,
    ) -> Result<(), Error<PData>>;
}

/// NodeDefinition is an entry in NodeDefs, indexed by the corresponding Unique assignment.
pub struct NodeDefinition {
    /// Type of node.
    pub(crate) ntype: NodeType,
    // Node name.
    pub(crate) name: NodeId,
}

/// NodeDefs is a Unique-indexed set of node definitions.
pub struct NodeDefs<PData> {
    /// Entries have an implicit index equal to their Unique value.
    entries: Vec<NodeDefinition>,

    _data: PhantomData<PData>,
}

impl<PData> NodeDefs<PData> {
    /// Create an empty set of node definitions.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            _data: PhantomData,
        }
    }

    /// Gets a NodeUnique by the assigned Unique index value,
    /// consisting of name and type information.
    pub fn get(&self, u: Unique) -> Option<(NodeUnique, NodeType)> {
        self.entries.get(u.index()).map(|d| {
            (
                NodeUnique {
                    id: u,
                    name: d.name.clone(),
                },
                d.ntype,
            )
        })
    }

    /// Gets the next unique node identifier. Returns an error when
    /// the underlying u16 overflows.
    pub(crate) fn next(
        &mut self,
        name: NodeId,
        ntype: NodeType,
    ) -> Result<NodeUnique, Error<PData>> {
        let uniq = NodeUnique {
            name: name.clone(),
            id: Unique::try_from(self.entries.len()).map_err(|_| Error::TooManyNodes {})?,
        };
        self.entries.push(NodeDefinition { ntype, name: name });
        Ok(uniq)
    }

    /// Returns an iterator over NodeUnique values for this set.
    pub(crate) fn iter(&self) -> impl Iterator<Item = NodeUnique> {
        self.entries
            .iter()
            .enumerate()
            .map(|(idx, val)| NodeUnique {
                name: val.name.clone(),
                id: Unique::try_from(idx).expect("valid defs"),
            })
    }
}

impl Unique {
    /// Index of this node in the runtime nodes vector.
    pub(crate) fn index(&self) -> usize {
        self.0 as usize
    }
}

impl TryFrom<usize> for Unique {
    type Error = std::num::TryFromIntError;

    /// TryFrom signals an error when the u16 overflows.
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(u16::try_from(value)?))
    }
}
