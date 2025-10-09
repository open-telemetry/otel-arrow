// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observed pipeline status and aggregation logic per core.

use crate::CoreId;
use crate::phase::{DeletionMode, FailReason, PipelineAggPhase, PipelinePhase, RejectReason};
use crate::pipeline_rt_status::PipelineRuntimeStatus;
use otap_df_config::health::{HealthPolicy, PhaseKind, Quorum};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

/// Aggregated, controller-synthesized view for a pipeline across all targeted
/// cores. This is what external APIs will return for `status`.
#[derive(Debug, Serialize, Clone)]
pub struct PipelineStatus {
    /// Coarse phase synthesized from all per-core phases.
    phase: PipelineAggPhase,

    /// Per-core details to aid debugging and aggregation.
    pub(crate) cores: HashMap<CoreId, PipelineRuntimeStatus>,

    #[serde(skip)]
    health_policy: HealthPolicy,
}

impl PipelineStatus {
    pub(crate) fn new(health_policy: HealthPolicy) -> Self {
        Self {
            phase: PipelineAggPhase::Unknown,
            cores: HashMap::new(),
            health_policy,
        }
    }

    /// Returns the current aggregated phase of the pipeline.
    #[must_use]
    pub fn phase(&self) -> &PipelineAggPhase {
        &self.phase
    }

    /// Returns the current per-core status map.
    #[must_use]
    pub fn per_core(&self) -> &HashMap<CoreId, PipelineRuntimeStatus> {
        &self.cores
    }

    fn counts(&self) -> AggregateCounts {
        let mut agg = AggregateCounts::default();
        for c in self.cores.values() {
            match &c.phase {
                PipelinePhase::Pending => agg.pending += 1,
                PipelinePhase::Starting => agg.starting += 1,
                PipelinePhase::Running => agg.running += 1,
                PipelinePhase::Updating => agg.updating += 1,
                PipelinePhase::RollingBack => agg.rolling_back += 1,
                PipelinePhase::Draining => agg.draining += 1,
                PipelinePhase::Stopped => agg.stopped += 1,
                PipelinePhase::Failed(r) => {
                    agg.failed += 1;
                    *agg.failed_reasons.entry(*r).or_insert(0) += 1;
                }
                PipelinePhase::Deleting(m) => {
                    agg.deleting += 1;
                    if matches!(m, DeletionMode::Forced) {
                        agg.forced_deletes += 1;
                    }
                }
                PipelinePhase::Deleted => agg.deleted += 1,
                PipelinePhase::Rejected(r) => {
                    agg.rejected += 1;
                    *agg.rejected_reasons.entry(*r).or_insert(0) += 1;
                } // PipelinePhase::Unknown => agg.unknown += 1,
            }
        }
        agg
    }

    /// Infer the aggregated phase.
    ///
    /// Choose a single, meaningful headline phase from the per-core counts.
    /// Precedence (highest first): Deleted, Deleting, Failed, Rejected, RollingBack, Updating,
    /// Draining, RunningAll/RunningDegraded, Starting, StoppedAll/StoppedPartial.
    pub fn infer_agg_phase(&mut self) {
        let count = self.counts();
        let total = count.total();
        let active = count.active();
        let forced = count.forced_deletes > 0;
        let top_fail = count
            .failed_reasons
            .iter()
            .max_by_key(|(_, n)| *n)
            .map(|(r, _)| *r);
        let top_reject = count
            .rejected_reasons
            .iter()
            .max_by_key(|(_, n)| *n)
            .map(|(r, _)| *r);

        if count.deleted == total {
            self.phase = PipelineAggPhase::Deleted;
            return;
        }
        if count.deleting > 0 {
            self.phase = PipelineAggPhase::Deleting {
                forced,
                remaining: active,
            };
            return;
        }
        if count.failed > 0 {
            self.phase = PipelineAggPhase::Failed {
                failed: count.failed,
                running: count.running,
                top_reason: top_fail,
            };
            return;
        }
        if count.rejected > 0 {
            self.phase = PipelineAggPhase::Rejected {
                rejected: count.rejected,
                running: count.running,
                top_reason: top_reject,
            };
            return;
        }
        if count.rolling_back > 0 {
            self.phase = PipelineAggPhase::RollingBack {
                rolling_back: count.rolling_back,
                running: count.running,
            };
            return;
        }
        if count.updating > 0 {
            self.phase = PipelineAggPhase::Updating {
                updating: count.updating,
                running: count.running,
            };
            return;
        }
        if count.draining > 0 {
            self.phase = PipelineAggPhase::Draining {
                draining: count.draining,
                running: count.running,
            };
            return;
        }

        if active > 0 && count.running == active {
            self.phase = PipelineAggPhase::RunningAll;
            return;
        }
        if count.running > 0 && count.running < active {
            self.phase = PipelineAggPhase::RunningDegraded {
                running: count.running,
                total_active: active,
            };
            return;
        }

        if count.pending > 0 || count.starting > 0 {
            self.phase = PipelineAggPhase::Starting {
                pending: count.pending,
                starting: count.starting,
            };
            return;
        }

        if active == 0 {
            self.phase = PipelineAggPhase::Deleted;
            return;
        }
        if count.stopped == active {
            self.phase = PipelineAggPhase::StoppedAll {
                stopped: count.stopped,
            };
            return;
        }
        self.phase = PipelineAggPhase::StoppedPartial {
            stopped: count.stopped,
            total_active: active,
        }
    }

    /// Returns a boolean representing the liveness across cores, governed by the aggregation
    /// policy.
    #[must_use]
    pub fn liveness(&self) -> bool {
        let (numer, denom) = self.count_quorum(|c| self.health_policy.is_live(c.phase.kind()));
        quorum_satisfied(numer, denom, self.health_policy.live_quorum)
    }

    /// Returns a boolean representing the readiness across cores, governed by the aggregation
    /// policy.
    #[must_use]
    pub fn readiness(&self) -> bool {
        let (numer, denom) = self.count_quorum(|c| {
            c.phase.kind() != PhaseKind::Deleted && self.health_policy.is_ready(c.phase.kind())
        });
        denom > 0 && quorum_satisfied(numer, denom, self.health_policy.ready_quorum)
    }

    /// Counts how many cores satisfy the given predicate, returning (numerator, denominator).
    ///
    /// The denominator excludes cores in `Deleted` phase.
    /// The numerator excludes cores in `Deleted` phase and counts only cores satisfying the
    /// predicate. The predicate is usually checking for liveness or readiness.
    fn count_quorum<F>(&self, pred: F) -> (usize, usize)
    where
        F: Fn(&PipelineRuntimeStatus) -> bool,
    {
        let denom = self
            .cores
            .values()
            .filter(|c| c.phase.kind() != PhaseKind::Deleted)
            .count();
        let numer = self
            .cores
            .values()
            .filter(|c| c.phase.kind() != PhaseKind::Deleted)
            .filter(|c| pred(c))
            .count();
        (numer, denom)
    }
}

/// Decide if (numerator/denominator) satisfies a quorum.
fn quorum_satisfied(numer: usize, denom: usize, q: Quorum) -> bool {
    match q {
        Quorum::All => numer == denom && denom > 0,
        Quorum::AtLeast(n) => numer >= n,
        Quorum::Percent(p) => {
            if denom == 0 {
                return false;
            }
            let needed = (denom * usize::from(p)).div_ceil(100);
            numer >= needed
        }
    }
}

/// Counts by coarse phase kind plus failure/deletion details.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AggregateCounts {
    pending: usize,
    starting: usize,
    running: usize,
    updating: usize,
    rolling_back: usize,
    draining: usize,
    stopped: usize,
    failed: usize,
    rejected: usize,
    deleting: usize,
    deleted: usize,
    forced_deletes: usize,
    failed_reasons: BTreeMap<FailReason, usize>,
    rejected_reasons: BTreeMap<RejectReason, usize>,
    unknown: usize,
}

impl AggregateCounts {
    fn total(&self) -> usize {
        self.pending
            + self.starting
            + self.running
            + self.updating
            + self.rolling_back
            + self.draining
            + self.stopped
            + self.failed
            + self.deleting
            + self.deleted
    }
    fn active(&self) -> usize {
        self.total() - self.deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn runtime(phase: PipelinePhase) -> PipelineRuntimeStatus {
        PipelineRuntimeStatus {
            phase,
            ..PipelineRuntimeStatus::default()
        }
    }

    fn new_status(policy: HealthPolicy) -> PipelineStatus {
        PipelineStatus {
            phase: PipelineAggPhase::Unknown,
            cores: HashMap::new(),
            health_policy: policy,
        }
    }

    #[test]
    fn infer_agg_phase_prioritizes_deleting_with_forced_flag() {
        let policy = HealthPolicy::default();
        let mut status = new_status(policy);
        _ = status
            .cores
            .insert(0, runtime(PipelinePhase::Deleting(DeletionMode::Forced)));
        _ = status
            .cores
            .insert(1, runtime(PipelinePhase::Failed(FailReason::DrainError)));

        status.infer_agg_phase();

        assert_eq!(
            status.phase(),
            &PipelineAggPhase::Deleting {
                forced: true,
                remaining: 2,
            }
        );
    }

    #[test]
    fn liveness_respects_percent_quorum_and_excludes_deleted() {
        let policy = HealthPolicy {
            live_if: vec![PhaseKind::Running],
            ready_if: vec![PhaseKind::Running],
            live_quorum: Quorum::Percent(60),
            ready_quorum: Quorum::All,
        };
        let mut status = new_status(policy);
        _ = status.cores.insert(0, runtime(PipelinePhase::Running));
        _ = status.cores.insert(1, runtime(PipelinePhase::Running));
        _ = status
            .cores
            .insert(2, runtime(PipelinePhase::Failed(FailReason::RuntimeError)));
        _ = status.cores.insert(3, runtime(PipelinePhase::Deleted));

        assert!(status.liveness());

        _ = status
            .cores
            .insert(1, runtime(PipelinePhase::Failed(FailReason::RuntimeError)));

        assert!(!status.liveness());
    }

    #[test]
    fn readiness_requires_all_non_deleted_cores_to_be_ready() {
        let policy = HealthPolicy {
            live_if: vec![PhaseKind::Running],
            ready_if: vec![PhaseKind::Running],
            live_quorum: Quorum::AtLeast(1),
            ready_quorum: Quorum::All,
        };
        let mut status = new_status(policy);
        _ = status.cores.insert(0, runtime(PipelinePhase::Running));
        _ = status.cores.insert(1, runtime(PipelinePhase::Running));

        assert!(status.readiness());

        _ = status.cores.insert(1, runtime(PipelinePhase::Updating));

        assert!(!status.readiness());
    }
}
