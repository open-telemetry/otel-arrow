// SPDX-License-Identifier: Apache-2.0

//! Node configuration specification.
//!
//! A node is a fundamental unit in our data processing pipeline, representing either a receiver
//! (source), processor, exporter (sink), or connector (linking pipelines).
//!
//! A node can have multiple outgoing named ports, each connected to a hyper-edge that defines how
//! data flows from this node to one or more target nodes.

use crate::{Description, NodeId, PortName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Configuration for a node in the pipeline.
/// Each node contains its own settings and defines how it connects to downstream nodes via out_ports.
/// Each out_port is a named output (e.g. "success", "error") that defines a hyper-edge:
/// - The hyper-edge configuration determines which downstream nodes are connected,
///   and how messages are routed (broadcast, round-robin, ...).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeConfig {
    /// The kind of this node, which determines its role in the pipeline.
    /// 4 kinds are currently specified:
    /// - `Receiver`: A node that receives data from an external source.
    /// - `Processor`: A node that processes data, transforming it in some way.
    /// - `Exporter`: A node that exports data to an external destination.
    /// - `Connector`: A node that connects 2 pipelines together, allowing data to flow between them.
    pub kind: NodeKind,

    /// An optional description of this node.
    pub description: Option<Description>,

    /// Outgoing hyper-edges, keyed by port name.
    /// Each port connects this node to one or more downstream nodes, with a specific dispatch strategy.
    pub out_ports: HashMap<PortName, HyperEdgeConfig>,

    /// Node-specific configuration.
    ///
    /// This configuration is interpreted by the node itself and is not interpreted and validated by
    /// the pipeline engine.
    ///
    /// Note: A pre-validation step using a JSON schema or protobuf could be added to the
    /// management plane to ensure that the configuration is valid.
    #[serde(default)]
    pub config: Value,
}

/// Describes a hyper-edge from a node output port to one or more destination nodes,
/// and defines the dispatching strategy for this port.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperEdgeConfig {
    /// List of downstream node IDs this port connects to.
    ///
    /// When there is only one target node, the hyper-edge is a simple edge and the dispatch
    /// strategy is ignored.
    pub destinations: HashSet<NodeId>,

    /// Dispatch strategy for sending messages (broadcast, round-robin, ...).
    pub dispatch_strategy: DispatchStrategy,
}

/// Dispatching strategies for hyper-edges.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

/// Node kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum NodeKind {
    /// A source of signals
    Receiver,
    /// A processor of signals
    Processor,
    /// A sink of signals
    Exporter,
    /// A connector between two pipelines
    Connector,
    /// A merged chain of consecutive processors
    ProcessorChain,
}
