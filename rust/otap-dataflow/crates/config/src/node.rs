// SPDX-License-Identifier: Apache-2.0

//! Node configuration model
//! IMPORTANT NOTE: This is a work in progress and not yet fully implemented.

use crate::{NodeKind, SignalType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// The name of a node in the pipeline.
pub type NodeName = Cow<'static, str>;
/// The name of a node out port in the pipeline.
pub type PortName = Cow<'static, str>;

/// A hyper-DAG representing a pipeline of nodes.
///
/// Note: We use a hyper-DAG instead of a simple DAG to allow for more complex interconnections
/// between nodes. For example, a single node can have multiple named output ports, each of which
/// can be connected to multiple nodes. This allows for more flexible and complex data flows within
/// the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperDAG {
    nodes: HashMap<NodeName, Node>,
}

/// A node in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    name: NodeName,
    kind: NodeKind,

    out_ports: HashMap<PortName, HyperEdge>,

    #[serde(default)]
    chain_members: Vec<NodeName>,

    /// The custom configuration for this node.
    ///
    /// This configuration is interpreted by the node itself and is not interpreted and validated by
    /// the pipeline engine.
    #[serde(default)]
    config: Value,
}

/// A directed hyper-edge in the pipeline establishing a connection between one source node and
/// one or more target nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    /// The type of signal carried by this hyper-edge.
    signal_type: HashSet<SignalType>,

    /// The target nodes of this hyper-edge.
    ///
    /// When there is only one target node, the hyper-edge is a simple edge and the dispatch
    /// strategy is ignored.
    targets: HashSet<NodeName>,

    /// The strategy used to dispatch data to the targets of this hyper-edge.
    dispatch_strategy: DispatchStrategy,
}

/// The strategy used to dispatch data to the targets of a hyper-edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DispatchStrategy {
    /// Broadcast the data to all targeted nodes.
    Broadcast,
    /// Round-robin dispatching to the targets.
    RoundRobin,
    /// Randomly select a target node to dispatch the data to.
    Random,
    /// Dispatch the data to the least loaded target node.
    LeastLoaded,
}
