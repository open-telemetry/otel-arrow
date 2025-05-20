// SPDX-License-Identifier: Apache-2.0

//! Errors for the config crate.

use crate::{NodeName, PortName, SignalType};
use crate::node::DispatchStrategy;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cycle detected involving nodes: {0:?}")]
    CycleDetected(Vec<NodeName>),

    #[error("Type mismatch on edge from `{from_id}` ({from_out:?}) to `{to_id}` ({to_in:?})")]
    TypeMismatch {
        from_id: String,
        to_id: String,
        from_out: SignalType,
        to_in: SignalType,
    },

    #[error("Duplicated node id `{node_name}`")]
    DuplicateNode { node_name: NodeName },

    /// The same out‐port was connected more than once on a single node.
    #[error("The same out-port `{port}` was connected more than once on the node `{source_node}`")]
    DuplicateOutPort {
        /// The node on which the port was duplicated.
        source_node: NodeName,
        /// The port name that was used twice.
        port: PortName,
    },

    #[error("Edge references unknown node `{0}`")]
    UnknownNode(NodeName),

    #[error("Invalid hyper-dag configuration: {errors:?}")]
    InvalidHyperDag {
        errors: Vec<Error>,
    },

    #[error("Invalid hyper-edge configuration: {source_node} -> {target_nodes:?}")]
    InvalidHyperEdge {
        source_node: NodeName,
        target_nodes: Vec<NodeName>,
        dispatch_strategy: DispatchStrategy,
        missing_source: bool,
        missing_targets: Vec<NodeName>,
    },
}
