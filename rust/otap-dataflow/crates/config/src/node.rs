// SPDX-License-Identifier: Apache-2.0

//! Node configuration model
//! IMPORTANT NOTE: This is a work in progress and not yet fully implemented.

use crate::{NodeKind, SignalType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

type NodeName = Cow<'static, str>;
type PortName = Cow<'static, str>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperDAG {
    nodes: HashMap<NodeName, Node>,
}

/// A node in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    name: NodeName,
    kind: NodeKind,

    out_ports: HashMap<PortName, OutPort>,

    #[serde(default)]
    chain_members: Vec<NodeName>,

    #[serde(default)]
    config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutPort {
    signal_type: HashSet<SignalType>,
    targets: HashSet<NodeName>,
}
