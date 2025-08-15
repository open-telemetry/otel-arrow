//! Test helpers to create NodeId values.

use crate::node::{NodeDefs, NodeId, NodeName, NodeType};

/// test_node returns a NodeId-named NodeId with Index=0, i.e.,
/// the first node.  the NodeDefs are thrown away, which usually
/// signals that the pipeline data type is meaningless in a certain
/// test.
#[must_use]
pub fn test_node<T: Into<NodeName>>(name: T) -> NodeId {
    NodeDefs::<()>::default()
        // Note that the node must be irrelevant since the user is not
        // saving the NodeDefs object.
        .next(name.into(), NodeType::Processor)
        .expect("ok")
}

/// test_node returns multiple NodeId values matching the provided
/// names.
#[must_use]
pub fn test_nodes<T: Into<NodeName>>(names: Vec<T>) -> Vec<NodeId> {
    let mut defs = NodeDefs::<()>::default();
    for name in names {
        _ = defs.next(name.into(), NodeType::Processor).expect("ok");
    }
    defs.iter().collect()
}
