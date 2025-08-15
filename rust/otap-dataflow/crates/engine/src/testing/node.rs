//! Test helpers to create NodeUnique values.

use crate::node::{NodeDefs, NodeType, NodeUnique};
use otap_df_config::NodeId;

/// test_node returns a NodeId-named NodeUnique with Unique=0, i.e.,
/// the first node.  the NodeDefs are thrown away, which usually
/// signals that the pipeline data type is meaningless in a certain
/// test.
#[must_use]
pub fn test_node<T: Into<NodeId>>(name: T) -> NodeUnique {
    NodeDefs::<()>::default()
        // Note that the node must be irrelevant since the user is not
        // saving the NodeDefs object.
        .next(name.into(), NodeType::Processor)
        .expect("ok")
}

/// test_node returns multiple NodeUnique values matching the provided
/// names.
#[must_use]
pub fn test_nodes<T: Into<NodeId>>(names: Vec<T>) -> Vec<NodeUnique> {
    let mut defs = NodeDefs::<()>::default();
    for name in names {
        _ = defs.next(name.into(), NodeType::Processor).expect("ok");
    }
    defs.iter().collect()
}
