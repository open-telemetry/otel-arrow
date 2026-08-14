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

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum PlacementError {
    #[error("strategy returned duplicate core {0}")]
    DuplicateCore(usize),
    #[error("strategy returned unavailable core {0}")]
    UnavailableCore(usize),
    #[error("strategy returned reserved core {0}")]
    ReservedCore(usize),
    #[error("strategy returned {actual} cores, expected {expected}")]
    WrongCount { expected: usize, actual: usize },
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
        for core in &free {
            if let Some(node) = topology.numa_node(core.id as u32) {
                by_node.entry(node).or_default().push(*core);
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

        free.into_iter().take(count).collect()
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
    pub fn select_core_count(
        &self,
        available: &[CoreId],
        reserved: &BTreeSet<usize>,
        count: usize,
    ) -> Result<Vec<CoreId>, PlacementError> {
        let selected = self
            .strategy
            .select_core_count(self.topology, available, reserved, count);
        self.validate_selected_cores(available, reserved, count, selected)
    }

    fn validate_selected_cores(
        &self,
        available: &[CoreId],
        reserved: &BTreeSet<usize>,
        count: usize,
        selected: Vec<CoreId>,
    ) -> Result<Vec<CoreId>, PlacementError> {
        if selected.len() != count {
            return Err(PlacementError::WrongCount {
                expected: count,
                actual: selected.len(),
            });
        }

        let available_ids: BTreeSet<_> = available.iter().map(|core| core.id).collect();
        let mut seen = BTreeSet::new();
        for core in &selected {
            if !seen.insert(core.id) {
                return Err(PlacementError::DuplicateCore(core.id));
            }
            if !available_ids.contains(&core.id) {
                return Err(PlacementError::UnavailableCore(core.id));
            }
            if reserved.contains(&core.id) {
                return Err(PlacementError::ReservedCore(core.id));
            }
        }
        Ok(selected)
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

    #[derive(Debug)]
    struct FixedPlacementStrategy(Vec<CoreId>);

    impl PlacementStrategy for FixedPlacementStrategy {
        fn select_core_count(
            &self,
            _topology: &NumaTopology,
            _available: &[CoreId],
            _reserved: &BTreeSet<usize>,
            _count: usize,
        ) -> Vec<CoreId> {
            self.0.clone()
        }
    }

    /// Scenario: a placement planner is constructed with a non-default
    /// strategy implementation.
    /// Guarantees: controller placement honors the injected strategy while
    /// still applying reservation validation.
    #[test]
    fn planner_can_use_injected_strategy() {
        let topology = NumaTopology::unknown();
        let planner = PlacementPlanner::with_strategy(&topology, ReversePlacementStrategy);
        let selected = planner
            .select_core_count(
                &[
                    CoreId { id: 0 },
                    CoreId { id: 1 },
                    CoreId { id: 2 },
                    CoreId { id: 3 },
                ],
                &BTreeSet::from([2]),
                2,
            )
            .expect("reverse strategy should produce a valid selection");

        assert_eq!(
            selected.iter().map(|core| core.id).collect::<Vec<_>>(),
            vec![3, 1]
        );
    }

    /// Scenario: a placement strategy returns duplicate, unavailable,
    /// reserved, or under-sized core selections.
    /// Guarantees: planner validation rejects invalid strategy output before
    /// it can become a committed pipeline placement.
    #[test]
    fn planner_rejects_invalid_strategy_output() {
        let topology = NumaTopology::unknown();
        let available = [CoreId { id: 0 }, CoreId { id: 1 }];

        let duplicate = PlacementPlanner::with_strategy(
            &topology,
            FixedPlacementStrategy(vec![CoreId { id: 0 }, CoreId { id: 0 }]),
        );
        assert_eq!(
            duplicate.select_core_count(&available, &BTreeSet::new(), 2),
            Err(PlacementError::DuplicateCore(0))
        );

        let unavailable = PlacementPlanner::with_strategy(
            &topology,
            FixedPlacementStrategy(vec![CoreId { id: 0 }, CoreId { id: 7 }]),
        );
        assert_eq!(
            unavailable.select_core_count(&available, &BTreeSet::new(), 2),
            Err(PlacementError::UnavailableCore(7))
        );

        let reserved = PlacementPlanner::with_strategy(
            &topology,
            FixedPlacementStrategy(vec![CoreId { id: 0 }, CoreId { id: 1 }]),
        );
        assert_eq!(
            reserved.select_core_count(&available, &BTreeSet::from([1]), 2),
            Err(PlacementError::ReservedCore(1))
        );

        let wrong_count = PlacementPlanner::with_strategy(
            &topology,
            FixedPlacementStrategy(vec![CoreId { id: 0 }]),
        );
        assert_eq!(
            wrong_count.select_core_count(&available, &BTreeSet::new(), 2),
            Err(PlacementError::WrongCount {
                expected: 2,
                actual: 1
            })
        );
    }

    /// Scenario: no single NUMA node can satisfy the requested core count.
    /// Guarantees: the NUMA-packing strategy falls back to deterministic
    /// global visible-core order.
    #[test]
    fn numa_packing_fallback_uses_global_core_order() {
        let topology = NumaTopology::with_visible_cpus(
            BTreeMap::from([(0, 0), (1, 1), (2, 0)]),
            BTreeSet::from([0, 1, 2]),
            TopologyCompleteness::Complete,
        );
        let planner = PlacementPlanner::new(&topology);

        let selected = planner
            .select_core_count(
                &[CoreId { id: 0 }, CoreId { id: 1 }, CoreId { id: 2 }],
                &BTreeSet::new(),
                3,
            )
            .expect("fallback should select all requested cores");

        assert_eq!(
            selected.iter().map(|core| core.id).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }
}
