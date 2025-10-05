// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observed pipeline status and aggregation logic per core.

use crate::CoreId;
use crate::phase::{DeletionMode, FailReason, PipelineAggPhase, PipelinePhase, RejectReason};
use crate::pipeline_rt_status::PipelineRuntimeStatus;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::time::SystemTime;

/// Aggregated, controller-synthesized view for a pipeline across all targeted
/// cores. This is what external APIs will return for `status`.
#[derive(Debug, Serialize, Clone)]
pub struct PipelineStatus {
    /// Coarse phase synthesized from all per-core phases.
    phase: PipelineAggPhase,

    /// Per-core details to aid debugging and aggregation.
    pub(crate) per_core: HashMap<CoreId, PipelineRuntimeStatus>,
}

impl PipelineStatus {
    pub(crate) fn new(now: SystemTime) -> Self {
        Self {
            phase: PipelineAggPhase::Unknown,
            per_core: HashMap::new(),
        }
    }

    /// Returns the current aggregated phase of the pipeline.
    pub fn phase(&self) -> &PipelineAggPhase {
        &self.phase
    }

    /// Returns the current per-core status map.
    pub fn per_core(&self) -> &HashMap<CoreId, PipelineRuntimeStatus> {
        &self.per_core
    }

    fn counts(&self) -> AggregateCounts {
        let mut agg = AggregateCounts::default();
        for c in self.per_core.values() {
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
                PipelinePhase::Rejected(r) => { agg.rejected += 1; *agg.rejected_reasons.entry(*r).or_insert(0) += 1; },
                PipelinePhase::Unknown => agg.unknown += 1,
            }
        }
        agg
    }

    /// Infer the aggregated phase.
    /// Choose a single, meaningful headline phase from the per-core counts.
    /// Precedence (highest first): Deleting, Failed, Rejected, RollingBack, Updating, Draining,
    /// RunningAll/RunningDegraded, Starting, StoppedAll/StoppedPartial, Deleted.
    pub fn infer_aggregated_phase(&mut self) {
        let count = self.counts();
        let total = count.total();
        let active = count.active();
        let forced = count.forced_deletes > 0;
        let top_fail = count.failed_reasons.iter().max_by_key(|(_, n)| *n).map(|(r, _)| *r);
        let top_reject = count.rejected_reasons.iter().max_by_key(|(_, n)| *n).map(|(r, _)| *r);

        if count.deleted == total { self.phase = PipelineAggPhase::Deleted; return; }
        if count.deleting > 0 { self.phase =  PipelineAggPhase::Deleting { forced, remaining: active }; return; }
        if count.failed   > 0 { self.phase =  PipelineAggPhase::Failed { failed: count.failed, running: count.running, top_reason: top_fail }; return; }
        if count.rejected > 0 { self.phase =  PipelineAggPhase::Rejected { rejected: count.rejected, running: count.running, top_reason: top_reject }; return; }
        if count.rolling_back > 0 { self.phase =  PipelineAggPhase::RollingBack { rolling_back: count.rolling_back, running: count.running }; return; }
        if count.updating > 0 { self.phase =  PipelineAggPhase::Updating { updating: count.updating, running: count.running }; return; }
        if count.draining > 0 { self.phase =  PipelineAggPhase::Draining { draining: count.draining, running: count.running }; return; }

        if active > 0 && count.running == active { self.phase =  PipelineAggPhase::RunningAll; return; }
        if count.running > 0 && count.running < active { self.phase =  PipelineAggPhase::RunningDegraded { running: count.running, total_active: active }; return; }

        if count.pending > 0 || count.starting > 0 { self.phase =  PipelineAggPhase::Starting { pending: count.pending, starting: count.starting }; return; }

        if active == 0 { self.phase =  PipelineAggPhase::Deleted; return; }
        if count.stopped == active { self.phase =  PipelineAggPhase::StoppedAll { stopped: count.stopped }; return; }
        self.phase = PipelineAggPhase::StoppedPartial { stopped: count.stopped, total_active: active }
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

/// Choose a single, meaningful headline phase from the per-core counts.
/// Precedence (highest first): Deleting, Failed, RollingBack, Updating, Draining,
/// RunningAll/RunningDegraded, Starting, StoppedAll/StoppedPartial, Deleted.
fn derive_headline(c: &AggregateCounts) -> PipelineAggPhase {
    let total = c.total();
    let active = c.active();
    let forced = c.forced_deletes > 0;
    let top_reason = c
        .failed_reasons
        .iter()
        .max_by_key(|(_, n)| *n)
        .map(|(r, _)| *r);

    if c.deleted == total {
        return PipelineAggPhase::Deleted;
    }
    if c.deleting > 0 {
        return PipelineAggPhase::Deleting {
            forced,
            remaining: active,
        };
    }
    if c.failed > 0 {
        return PipelineAggPhase::Failed {
            failed: c.failed,
            running: c.running,
            top_reason,
        };
    }
    if c.rolling_back > 0 {
        return PipelineAggPhase::RollingBack {
            rolling_back: c.rolling_back,
            running: c.running,
        };
    }
    if c.updating > 0 {
        return PipelineAggPhase::Updating {
            updating: c.updating,
            running: c.running,
        };
    }
    if c.draining > 0 {
        return PipelineAggPhase::Draining {
            draining: c.draining,
            running: c.running,
        };
    }

    if active > 0 && c.running == active {
        return PipelineAggPhase::RunningAll;
    }
    if c.running > 0 && c.running < active {
        return PipelineAggPhase::RunningDegraded {
            running: c.running,
            total_active: active,
        };
    }

    // Early lifecycle gets priority over "stopped" summaries when activity is present.
    if c.pending > 0 || c.starting > 0 {
        return PipelineAggPhase::Starting {
            pending: c.pending,
            starting: c.starting,
        };
    }

    if active == 0 {
        return PipelineAggPhase::Deleted;
    } // only deleted cores remain
    if c.stopped == active {
        return PipelineAggPhase::StoppedAll { stopped: c.stopped };
    }
    PipelineAggPhase::StoppedPartial {
        stopped: c.stopped,
        total_active: active,
    }
}
