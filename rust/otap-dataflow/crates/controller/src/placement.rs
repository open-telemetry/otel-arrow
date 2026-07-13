// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Controller-owned runtime placement metadata.

use core_affinity::CoreId;
use otap_df_config::{PipelineGroupId, PipelineId};
use otap_df_engine::topology::{NumaTopology, TopologyCompleteness};
use std::collections::{BTreeMap, BTreeSet};

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
    /// NUMA node for `core_id`, when topology discovery mapped it.
    pub known_numa_node_id: Option<usize>,
    /// Completeness of the topology used for this placement.
    pub topology_completeness: TopologyCompleteness,
}

impl CorePlacement {
    /// Creates a core placement using the controller-owned topology snapshot.
    #[must_use]
    pub fn from_core_id(core_id: CoreId, topology: &NumaTopology) -> Self {
        let known_numa_node_id = topology
            .numa_node(core_id.id as u32)
            .map(|node| node as usize);
        Self {
            core_id,
            numa_node_id: known_numa_node_id.unwrap_or(0),
            known_numa_node_id,
            topology_completeness: topology.completeness(),
        }
    }
}

/// Strategy used to select concrete cores for `core_count` allocations.
///
/// Implementations are called during startup preflight and live-control
/// rollout/reconcile planning, so they must be deterministic for identical
/// inputs. Strategies must honor `reserved`, return at most `count` cores, and
/// only select cores from `available`.
pub trait PlacementStrategy: std::fmt::Debug {
    /// Selects `count` cores from `available` while excluding `reserved`.
    #[must_use]
    fn select_core_count(
        &self,
        topology: &NumaTopology,
        available: &[CoreId],
        reserved: &BTreeSet<usize>,
        count: usize,
    ) -> Vec<CoreId>;
}

/// Default `core_count` strategy that prefers one NUMA node when possible.
///
/// This policy is chosen for intra-pipeline cache and NUMA locality. Future
/// balancing or hardware-aware policies can implement [`PlacementStrategy`]
/// without changing controller placement call sites.
#[derive(Clone, Copy, Debug, Default)]
pub struct NumaPackingPlacementStrategy;

impl PlacementStrategy for NumaPackingPlacementStrategy {
    fn select_core_count(
        &self,
        topology: &NumaTopology,
        available: &[CoreId],
        reserved: &BTreeSet<usize>,
        count: usize,
    ) -> Vec<CoreId> {
        let mut free: Vec<_> = available
            .iter()
            .copied()
            .filter(|core| !reserved.contains(&core.id))
            .collect();
        free.sort_by_key(|core| core.id);

        if count == 0 || topology.is_unknown() {
            return free.into_iter().take(count).collect();
        }

        let mut by_node: BTreeMap<u32, Vec<CoreId>> = BTreeMap::new();
        let mut unknown = Vec::new();
        for core in free {
            if let Some(node) = topology.numa_node(core.id as u32) {
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

/// Selection helper for controller-owned pipeline placement.
#[derive(Debug)]
pub struct PlacementPlanner<'a, S = NumaPackingPlacementStrategy> {
    topology: &'a NumaTopology,
    strategy: S,
}

impl<'a> PlacementPlanner<'a, NumaPackingPlacementStrategy> {
    /// Creates a planner over a single topology snapshot using the default
    /// NUMA-packing strategy.
    #[must_use]
    pub fn new(topology: &'a NumaTopology) -> Self {
        Self::with_strategy(topology, NumaPackingPlacementStrategy)
    }
}

impl<'a, S: PlacementStrategy> PlacementPlanner<'a, S> {
    /// Creates a planner over a single topology snapshot and placement strategy.
    #[must_use]
    pub fn with_strategy(topology: &'a NumaTopology, strategy: S) -> Self {
        Self { topology, strategy }
    }

    /// Selects `count` cores from `available` using this planner's strategy.
    #[must_use]
    pub fn select_core_count(
        &self,
        available: &[CoreId],
        reserved: &BTreeSet<usize>,
        count: usize,
    ) -> Vec<CoreId> {
        self.strategy
            .select_core_count(self.topology, available, reserved, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct ReversePlacementStrategy;

    impl PlacementStrategy for ReversePlacementStrategy {
        fn select_core_count(
            &self,
            _topology: &NumaTopology,
            available: &[CoreId],
            reserved: &BTreeSet<usize>,
            count: usize,
        ) -> Vec<CoreId> {
            let mut selected = available
                .iter()
                .copied()
                .filter(|core| !reserved.contains(&core.id))
                .collect::<Vec<_>>();
            selected.sort_by_key(|core| std::cmp::Reverse(core.id));
            selected.truncate(count);
            selected
        }
    }

    #[test]
    fn planner_can_use_injected_strategy() {
        let topology = NumaTopology::unknown();
        let planner = PlacementPlanner::with_strategy(&topology, ReversePlacementStrategy);
        let selected = planner.select_core_count(
            &[
                CoreId { id: 0 },
                CoreId { id: 1 },
                CoreId { id: 2 },
                CoreId { id: 3 },
            ],
            &BTreeSet::from([2]),
            2,
        );

        assert_eq!(
            selected.iter().map(|core| core.id).collect::<Vec<_>>(),
            vec![3, 1]
        );
    }
}
