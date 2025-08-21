// SPDX-License-Identifier: Apache-2.0

//! Attributes describing the engine, pipeline, and node state.

use std::borrow::Cow;
use otap_df_telemetry_macros::attribute_set;

/// Engine-specific attributes (core, NUMA, process info).
#[attribute_set(name = "engine.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct EngineAttributeSet {
    /// Core identifier.
    #[attribute]
    pub core_id: usize,
    /// NUMA node identifier.
    #[attribute]
    pub numa_node_id: usize,
    /// Unique process identifier.
    #[attribute]
    pub process_id: u32,
}

/// Pipeline and node attributes (with engine attributes).
#[attribute_set(name = "pipeline.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct PipelineAttributeSet {
    /// Engine attributes.
    #[compose]
    pub engine: EngineAttributeSet,

    /// Node unique identifier (in scope of the pipeline).
    #[attribute]
    pub node_id: Cow<'static, str>,
    /// Node type (e.g., "receiver", "processor", "exporter").
    #[attribute]
    pub node_type: Cow<'static, str>,
    /// Pipeline unique identifier (in scope of the process).
    #[attribute]
    pub pipeline_id: Cow<'static, str>,
}
