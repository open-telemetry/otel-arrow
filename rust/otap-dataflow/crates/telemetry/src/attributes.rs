// SPDX-License-Identifier: Apache-2.0

//! Attributes for telemetry metrics.
//!
//! Note: Dynamic attributes are not yet supported.

use std::borrow::Cow;

/// Immutable attributes for a specific instance of a node.
#[derive(Debug, Clone, Default)]
pub struct NodeStaticAttrs {
    /// Node unique identifier (in scope of the pipeline).
    pub node_id: Cow<'static, str>,
    /// Node type (e.g., "receiver", "processor", "exporter").
    pub node_type: Cow<'static, str>,
    /// Pipeline unique identifier (in scope of the process).
    pub pipeline_id: Cow<'static, str>,

    /// Core identifier.
    pub core_id: u16,
    /// NUMA node identifier.
    pub numa_node_id: u16,
    /// Unique process identifier.
    pub process_id: u32,
}