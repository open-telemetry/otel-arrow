// SPDX-License-Identifier: Apache-2.0

//! Node configuration model

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{NodeKind, SignalType};

type NodeName = Rc<str>;
type PortName = String; 

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperDAG {
    nodes:  HashMap<NodeName, Node>,
}

/// A node in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    name: NodeName,
    kind: NodeKind,
    
    in_ports: HashMap<PortName, HashSet<SignalType>>,
    out_ports: HashMap<PortName, OutPort>,
    
    #[serde(default)]
    chain_members: Vec<Rc<str>>,
    
    #[serde(default)]
    config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutPort {
    signal_type: HashSet<SignalType>,
    targets: HashSet<NodeName>,
}