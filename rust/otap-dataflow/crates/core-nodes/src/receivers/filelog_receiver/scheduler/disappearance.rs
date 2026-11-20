// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::{
    config::Rotation,
    discovery::{
        ScanReason,
        reconcile::{Evidence, TrackedId},
    },
    source::FileLocator,
};
use std::time::{Duration, Instant};

struct EofInterval {
    offset: u64,
    first_observed: Instant,
    latest_turn_started: Instant,
}

/// Volatile evidence for one continuously resident reader.
#[derive(Default)]
pub(crate) struct ResidentDisappearance {
    pub(super) validated_locator: Option<FileLocator>,
    // Confirmation must start strictly after the first miss or latest failure.
    confirm_after: Option<Duration>,
    missing_this_pass: bool,
    confirmed_missing: bool,
    excluded: bool,
    eof: Option<EofInterval>,
}

impl ResidentDisappearance {
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
        if self.excluded {
            return;
        }
        match evidence {
            Evidence::Completed(pass) => {
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
                self.confirmed_missing = false;
            }
            _ => {}
        }
    }

    fn reset_missing(&mut self) {
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
