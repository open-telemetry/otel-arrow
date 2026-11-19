// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::{
    config::Rotation,
    delivery::Delivery,
    discovery::{
        ScanReason,
        reconcile::{Evidence, TrackedId},
    },
    identity::IdentityOwner,
    source::FileLocator,
};
use super::{LeaseGuard, LifecycleRefusal, ReaderIo, ReaderKey, Scheduler, batch::Batcher, cp};
use std::time::{Duration, Instant};

struct EofInterval {
    offset: u64,
    first_observed: Instant,
    latest_turn_started: Instant,
}

/// Ordered discovery evidence for one reader generation.
#[derive(Default)]
pub(crate) struct Disappearance {
    pub(super) validated_locator: Option<FileLocator>,
    // Confirmation must start strictly after the first miss or latest failure.
    confirm_after: Option<Duration>,
    missing_this_pass: bool,
    confirmed_missing: bool,
    excluded: bool,
    pub(super) evicted: bool,
    pub(super) absent_since: Option<Duration>,
    eof: Option<EofInterval>,
}

impl Disappearance {
    pub(super) fn set_evicted(&mut self, evicted: bool) {
        // A provisional handle can itself be evicted before its locator is
        // checked. Keep the original absence interval across that attempt.
        if self.evicted == evicted {
            return;
        }
        *self = Self {
            excluded: self.excluded,
            evicted,
            ..Self::default()
        };
    }

    pub(super) fn observe_eof(
        &mut self,
        offset: u64,
        turn_started: Instant,
        clock: impl FnOnce() -> Instant,
    ) {
        // A blocking probe can straddle the deadline. Measure from completion
        // of the first EOF to the start of a turn that successfully saw EOF.
        let interval = self.eof.get_or_insert_with(|| EofInterval {
            offset,
            first_observed: clock(),
            latest_turn_started: turn_started,
        });
        interval.latest_turn_started = turn_started;
    }

    pub(super) fn observe_growth(&mut self) {
        self.eof = None;
    }

    /// Feed synchronous reconciler callbacks in order. The lifecycle owner maps
    /// this reader to its lifetime-unique discovery ID and forwards global events
    /// as well as this ID's events, including those from an unfinished pass.
    pub(crate) fn observe(&mut self, id: TrackedId, evidence: &Evidence<'_>) {
        if self.excluded
            || (!self.evicted
                && self.validated_locator.is_none()
                && !matches!(evidence, Evidence::Excluded(file) if *file == id))
        {
            return;
        }
        match evidence {
            Evidence::Completed(pass) => {
                if self.evicted && pass.complete && self.missing_this_pass {
                    let _ = self.absent_since.get_or_insert(pass.finished);
                } else {
                    self.absent_since = None;
                }
                self.confirmed_missing = pass.complete
                    && self.missing_this_pass
                    && self.confirm_after.is_some_and(|end| pass.started > end);
                if pass.complete && self.missing_this_pass {
                    let _ = self.confirm_after.get_or_insert(pass.finished);
                } else if !pass.complete && self.confirm_after.is_some() {
                    self.confirm_after = Some(pass.finished);
                }
                self.missing_this_pass = false;
            }
            Evidence::Disappeared(file) if *file == id => self.missing_this_pass = true,
            Evidence::Validated(file, observation) if *file == id => {
                self.absent_since = None;
                self.confirmed_missing = false;
                if self.validated_locator.is_some() && observation.locator == self.validated_locator
                {
                    self.reset_missing();
                    if let (Some(size), Some(eof)) = (observation.size, &self.eof)
                        && size > eof.offset
                    {
                        self.observe_growth();
                    }
                }
            }
            Evidence::Present(file) if *file == id => {
                self.absent_since = None;
                self.confirmed_missing = false;
            }
            Evidence::Excluded(file) if *file == id => {
                self.reset_missing();
                self.excluded = true;
            }
            Evidence::RootChanged(_)
            | Evidence::ParentChanged(_)
            | Evidence::CoverageLost(ScanReason::RootChanged) => self.reset_missing(),
            Evidence::CoverageLost(_) => {
                self.absent_since = None;
                self.confirmed_missing = false;
            }
            _ => {}
        }
    }

    fn reset_missing(&mut self) {
        self.absent_since = None;
        self.confirmed_missing = false;
        self.confirm_after = None;
        self.missing_this_pass = false;
    }

    /// Sample at a point serialized with discovery delivery and read activity.
    /// Framing, delivery and durable progress remain separate preconditions.
    pub(crate) fn discovery_and_eof_inactivity_met(&self, rotation: &Rotation) -> bool {
        self.confirmed_missing
            && self.eof.as_ref().is_some_and(|interval| {
                interval
                    .latest_turn_started
                    .checked_duration_since(interval.first_observed)
                    .is_some_and(|idle| idle >= rotation.rotate_wait)
            })
    }
}

/// High-severity lifecycle report. The absence clock starts at reconciliation
/// completion, even when delivery delays the handoff to the lifecycle owner.
#[must_use]
#[derive(Debug)]
pub(crate) struct LateWriteCaptureLimitation {
    pub file_id: cp::FileId,
    pub absent_since: Duration,
    pub lease: LeaseGuard,
}

impl<I: ReaderIo> Scheduler<I> {
    /// Sample after forwarding discovery callbacks, with source turns paused.
    /// The lifecycle owner reports this limitation and explicitly releases the lease.
    pub(crate) fn contain_disappearance<C: Fn() -> Instant>(
        &mut self,
        key: ReaderKey,
        delivery: &mut Delivery<Batcher<C>>,
        owner: &IdentityOwner,
        now: Instant,
    ) -> Result<LateWriteCaptureLimitation, LifecycleRefusal> {
        let reader = self.removable_reader(key)?;
        let absent_since = reader
            .disappearance
            .absent_since
            .filter(|_| reader.handle.is_none() && reader.read_state != super::ReadState::Halted)
            .ok_or(LifecycleRefusal::NoClosedHandleAbsence)?;
        if owner.is_stopped() || delivery.is_stopped() {
            return Err(LifecycleRefusal::Stopped);
        }
        if owner.has_pending_write() {
            return Err(LifecycleRefusal::PendingDelivery);
        }
        let sink = delivery
            .builder_mut(now)
            .ok_or(LifecycleRefusal::PendingDelivery)?;
        let file_id = reader.file_id;
        let record = owner
            .state()
            .map_err(|_| LifecycleRefusal::Stopped)?
            .get(&file_id)
            .ok_or(LifecycleRefusal::UnappliedProgress)?;
        if !reader.owns_active_record(record)
            || record.committed_offset != reader.evidence.applied_offset
            || record.committed_frontier_guard != reader.evidence.applied_guard
        {
            return Err(LifecycleRefusal::UnappliedProgress);
        }
        let lease = self.remove_reader(key, sink, record.framing_resume)?;
        Ok(LateWriteCaptureLimitation {
            file_id,
            absent_since,
            lease,
        })
    }
}
