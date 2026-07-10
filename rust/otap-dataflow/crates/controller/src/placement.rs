// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Controller-owned runtime placement metadata.

use core_affinity::CoreId;
use otap_df_config::{PipelineGroupId, PipelineId};
use otap_df_engine::topology::{NumaTopology, TopologyCompleteness};

/// Stable placement snapshot for a controller deployment generation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlacementSnapshot {
    /// Monotonic placement generation. Startup uses generation `0`.
    pub generation: u64,
    /// Per-pipeline placements in controller launch order.
    pub pipelines: Vec<PipelinePlacement>,
}

impl PlacementSnapshot {
    /// Creates a placement snapshot from resolved per-pipeline assignments.
    #[must_use]
    pub fn from_assignments(generation: u64, pipelines: Vec<PipelinePlacement>) -> Self {
        Self {
            generation,
            pipelines,
        }
    }
}

/// Resolved placement for one logical pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelinePlacement {
    /// Pipeline group id.
    pub pipeline_group_id: PipelineGroupId,
    /// Pipeline id.
    pub pipeline_id: PipelineId,
    /// Assigned cores with NUMA metadata.
    pub cores: Vec<CorePlacement>,
}

impl PipelinePlacement {
    /// Returns the number of worker cores in this pipeline placement.
    #[must_use]
    pub fn core_count(&self) -> usize {
        self.cores.len()
    }
}

/// Resolved placement for one pipeline worker core.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorePlacement {
    /// CPU core selected by the controller.
    pub core_id: CoreId,
    /// NUMA node for `core_id`; unknown topology falls back to `0`.
    pub numa_node_id: usize,
    /// Completeness of the topology used for this placement.
    pub topology_completeness: TopologyCompleteness,
}

impl CorePlacement {
    /// Creates a core placement using the controller-owned topology snapshot.
    #[must_use]
    pub fn from_core_id(core_id: CoreId, topology: &NumaTopology) -> Self {
        Self {
            core_id,
            numa_node_id: topology.numa_node_or_zero(core_id.id as u32) as usize,
            topology_completeness: topology.completeness(),
        }
    }
}

/// NUMA-aware selection helper for `core_count` allocations.
#[derive(Debug)]
pub struct PlacementPlanner<'a> {
    topology: &'a NumaTopology,
}

impl<'a> PlacementPlanner<'a> {
    /// Creates a planner over a single topology snapshot.
    #[must_use]
    pub fn new(topology: &'a NumaTopology) -> Self {
        Self { topology }
    }

    /// Selects `count` cores from `available`, preferring one NUMA node when possible.
    #[must_use]
    pub fn select_core_count(
        &self,
        available: &[CoreId],
        reserved: &std::collections::BTreeSet<usize>,
        count: usize,
    ) -> Vec<CoreId> {
        let mut free: Vec<_> = available
            .iter()
            .copied()
            .filter(|core| !reserved.contains(&core.id))
            .collect();
        free.sort_by_key(|core| core.id);

        if count == 0 || self.topology.is_unknown() {
            return free.into_iter().take(count).collect();
        }

        let mut by_node: std::collections::BTreeMap<u32, Vec<CoreId>> =
            std::collections::BTreeMap::new();
        let mut unknown = Vec::new();
        for core in free {
            if let Some(node) = self.topology.numa_node(core.id as u32) {
                by_node.entry(node).or_default().push(core);
            } else {
                unknown.push(core);
            }
        }

        for cores in by_node.values_mut() {
            cores.sort_by_key(|core| core.id);
        }

        for cores in by_node.values() {
            if cores.len() >= count {
                return cores.iter().take(count).copied().collect();
            }
        }

        let mut selected = Vec::with_capacity(count);
        for cores in by_node.values() {
            for core in cores {
                selected.push(*core);
                if selected.len() == count {
                    return selected;
                }
            }
        }
        unknown.sort_by_key(|core| core.id);
        for core in unknown {
            selected.push(core);
            if selected.len() == count {
                return selected;
            }
        }
        selected
    }
}
