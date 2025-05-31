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

/// A node specification within the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeSpec {
    /// The kind of this node, which determines its role in the pipeline.
    /// 4 kinds are currently specified:
    /// - `Receiver`: A node that receives data from an external source.
    /// - `Processor`: A node that processes data, transforming it in some way.
    /// - `Exporter`: A node that exports data to an external destination.
    /// - `Connector`: A node that connects 2 pipelines together, allowing data to flow between them.
    pub kind: NodeKind,

    /// An optional description of this node.
    pub description: Option<Description>,

    /// The outgoing ports of this node, each connected to a hyper-edge.
    pub out_ports: HashMap<PortName, HyperEdgeSpec>,

    /// The custom configuration for this node.
    ///
    /// This configuration is interpreted by the node itself and is not interpreted and validated by
    /// the pipeline engine.
    ///
    /// Note: A pre-validation step using a JSON schema or protobuf could be added to the
    /// management plane to ensure that the configuration is valid.
    #[serde(default)]
    pub config: Value,
}

/// A directed hyper-edge in the pipeline establishing a connection between one source node and
/// one or more target nodes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HyperEdgeSpec {
    /// The target nodes of this hyper-edge.
    ///
    /// When there is only one target node, the hyper-edge is a simple edge and the dispatch
    /// strategy is ignored.
    pub targets: HashSet<NodeId>,

    /// The strategy used to dispatch data to the targets of this hyper-edge.
    pub dispatch_strategy: DispatchStrategy,
}

/// The strategy used to dispatch data to the targets of a hyper-edge.
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
