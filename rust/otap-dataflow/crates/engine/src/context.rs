//! Context carries metadata about messages in the pipeline.

use super::runtime_pipeline::NodeType;
use otap_df_config::NodeId;

/// Node is defined ...
pub struct NodeDefinition {
    pub(crate) ntype: NodeType,
    pub(crate) name: NodeId,
}

/// Uniqueness value
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Unique(u16);

impl Unique {
    /// Index of this node in the runtime nodes vector.
    pub(crate) fn index(&self) -> usize {
        self.0 as usize
    }
}

impl TryFrom<usize> for Unique {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(u16::try_from(value)?))
    }
}

/// NodeUniq is a u16 consisting of NodeID plus uniqueness bits.
#[derive(Clone, Debug)]
pub struct NodeUniq {
    pub(crate) id: Unique,
    pub(crate) name: NodeId,
}

impl NodeUniq {
    /// Gets the next unique node identifier. Returns an error when the underlying
    /// u16 overflows.
    pub(crate) fn next(
        name: NodeId,
        ntype: NodeType,
        defs: &mut Vec<NodeDefinition>,
    ) -> Result<NodeUniq, std::num::TryFromIntError> {
        let uniq = Self {
            name: name.clone(),
            id: Unique::try_from(defs.len())?,
        };
        defs.push(NodeDefinition { ntype, name: name });
        Ok(uniq)
    }
}
