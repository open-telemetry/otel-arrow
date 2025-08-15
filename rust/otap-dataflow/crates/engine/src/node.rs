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
use otap_df_config::PortName;
use otap_df_config::node::NodeUserConfig;
use std::marker::PhantomData;
use std::sync::Arc;

pub use otap_df_config::NodeId as NodeName;

/// Common trait for nodes in the pipeline.
#[async_trait::async_trait(?Send)]
pub trait Node {
    /// Flag indicating whether the node is shared (true) or local (false).
    #[must_use]
    fn is_shared(&self) -> bool;

    /// Node identifier.
    fn node_id(&self) -> NodeId;

    /// Returns a reference to the node's user configuration.
    #[must_use]
    fn user_config(&self) -> Arc<NodeUserConfig>;

    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: NodeControlMsg) -> Result<(), SendError<NodeControlMsg>>;
}

/// NodeId consists of NodeId and NodeIndex integer.
#[derive(Clone, Debug)]
pub struct NodeId {
    /// A unique integer.
    pub(crate) index: NodeIndex,

    /// A unique name as defined by otap_df_config.
    pub name: NodeName,
}

/// Index in the NodeDefs vector
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct NodeIndex(u16);

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

/// NodeDefinition is an entry in NodeDefs, indexed by the corresponding NodeIndex assignment.
pub(crate) struct NodeDefinition<Inner> {
    /// Type of node.
    pub(crate) ntype: NodeType,
    // Node name.
    pub(crate) name: NodeName,
    /// Inner data.
    pub(crate) inner: Inner,
}

/// NodeDefs is a NodeIndex-indexed set of node definitions.
pub(crate) struct NodeDefs<PData, Inner> {
    /// Entries have an implicit index equal to their NodeIndex value.
    entries: Vec<NodeDefinition<Inner>>,

    _data: PhantomData<PData>,
}

impl<PData, Inner> Default for NodeDefs<PData, Inner> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            _data: PhantomData,
        }
    }
}

impl<PData, Inner> NodeDefs<PData, Inner> {
    /// Gets a the node definition
    #[must_use]
    pub(crate) fn get(&self, index: NodeIndex) -> Option<&NodeDefinition<Inner>> {
        self.entries.get(index.0 as usize)
    }

    /// Gets the next unique node identifier. Returns an error when
    /// the underlying u16 overflows.
    pub fn next(
        &mut self,
        name: NodeName,
        ntype: NodeType,
        inner: Inner,
    ) -> Result<NodeId, Error<PData>> {
        let uniq = NodeId::build(
            NodeIndex::try_from(self.entries.len()).map_err(|_| Error::TooManyNodes {})?,
            name.clone(),
        );
        self.entries.push(NodeDefinition { ntype, name, inner });
        Ok(uniq)
    }

    /// Returns an iterator over NodeId values for this set.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (NodeId, &NodeDefinition<Inner>)> {
        self.entries.iter().enumerate().map(|(idx, val)| {
            (
                NodeId {
                    name: val.name.clone(),
                    index: NodeIndex::try_from(idx).expect("enumerated"),
                },
                val,
            )
        })
    }
}

impl TryFrom<usize> for NodeIndex {
    type Error = std::num::TryFromIntError;

    /// TryFrom signals an error when the u16 overflows.
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(u16::try_from(value)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::NodeId;

    #[test]
    fn test_too_many_nodes_error() {
        let mut node_defs: NodeDefs<(), ()> = NodeDefs::default();
        let node_id = NodeId::try_from("test_node").expect("valid node id");
        const LIMIT: usize = u16::MAX as usize + 1;
        for i in 0..=LIMIT {
            let result = node_defs.next(node_id.clone(), NodeType::Processor, ());

            if i == LIMIT {
                // This should fail with TooManyNodes error
                assert!(matches!(result, Err(Error::TooManyNodes {})));
                break;
            } else {
                assert!(result.is_ok());
            }
        }
    }
}

impl NodeId {
    pub(crate) fn build(index: NodeIndex, name: NodeName) -> NodeId {
        NodeId { index, name }
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
