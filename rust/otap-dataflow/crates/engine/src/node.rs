// Copyright The OpenTelemetry Authors
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
pub trait Node<PData> {
    /// Flag indicating whether the node is shared (true) or local (false).
    #[must_use]
    fn is_shared(&self) -> bool;

    /// Node identifier.
    fn node_id(&self) -> NodeId;

    /// Returns a reference to the node's user configuration.
    #[must_use]
    fn user_config(&self) -> Arc<NodeUserConfig>;

    /// Sends a control message to the node.
    async fn send_control_msg(
        &self,
        msg: NodeControlMsg<PData>,
    ) -> Result<(), SendError<NodeControlMsg<PData>>>;
}

/// NodeId consists of a unique integer index and a name.
#[derive(Clone, Debug)]
pub struct NodeId {
    /// A unique integer.
    pub(crate) index: usize,

    /// A unique name as defined by otap_df_config.
    pub name: NodeName,
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

/// Trait for nodes that can send pdata to a specific port.
pub trait NodeWithPDataSender<PData>: Node<PData> {
    /// Sets the sender for pdata messages on the node.
    fn set_pdata_sender(
        &mut self,
        node: NodeId,
        port: PortName,
        sender: Sender<PData>,
    ) -> Result<(), Error>;
}

/// Trait for nodes that can receive pdata.
pub trait NodeWithPDataReceiver<PData>: Node<PData> {
    /// Sets the receiver for pdata messages on the node.
    fn set_pdata_receiver(&mut self, node: NodeId, receiver: Receiver<PData>) -> Result<(), Error>;
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
    pub(crate) fn get(&self, index: usize) -> Option<&NodeDefinition<Inner>> {
        self.entries.get(index)
    }

    /// Gets the next unique node identifier. Returns an error when
    /// the node limit (65,535) is exceeded.
    pub fn next(&mut self, name: NodeName, ntype: NodeType, inner: Inner) -> Result<NodeId, Error> {
        let index = self.entries.len();
        if index > u16::MAX as usize {
            return Err(Error::TooManyNodes {});
        }

        let uniq = NodeId::build(index, name.clone());
        self.entries.push(NodeDefinition { ntype, name, inner });
        Ok(uniq)
    }

    /// Returns an iterator over NodeId values for this set.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (NodeId, &NodeDefinition<Inner>)> {
        self.entries.iter().enumerate().map(|(idx, val)| {
            (
                NodeId {
                    name: val.name.clone(),
                    index: idx,
                },
                val,
            )
        })
    }
}

impl NodeId {
    pub(crate) fn build(index: usize, name: NodeName) -> NodeId {
        NodeId { index, name }
    }

    /// The global index (TODO discuss)
    pub fn index(&self) -> usize {
        self.index
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_too_many_nodes_error() {
        let mut node_defs: NodeDefs<(), ()> = NodeDefs::default();
        let name: NodeName = "test_node".into();
        const LIMIT: usize = u16::MAX as usize + 1;
        for i in 0..=LIMIT {
            let result = node_defs.next(name.clone(), NodeType::Processor, ());

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
