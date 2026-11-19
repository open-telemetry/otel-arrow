// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::{
    config::Rotation,
    discovery::reconcile::{Evidence, TrackedId},
    source::FileLocator,
};
use std::time::{Duration, Instant};

/// Volatile evidence for one continuously resident reader.
#[derive(Default)]
pub(crate) struct ResidentDisappearance {
    pub(super) validated_locator: Option<FileLocator>,
    // Confirmation must start strictly after the first miss or latest failure.
    confirm_after: Option<Duration>,
    missing_this_pass: bool,
    confirmed_missing: bool,
    pub(super) eof_since: Option<Instant>,
}

impl ResidentDisappearance {
    /// Feed synchronous reconciler callbacks in order. The lifecycle owner maps
    /// this reader to its lifetime-unique discovery ID and forwards global events
    /// as well as this ID's events, including those from an unfinished pass.
    pub(crate) fn observe(&mut self, id: TrackedId, evidence: &Evidence<'_>) {
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
                    self.confirm_after = None;
                    self.missing_this_pass = false;
                }
            }
            Evidence::Present(file) | Evidence::Excluded(file) if *file == id => {
                self.confirmed_missing = false;
            }
            Evidence::CoverageLost(_) | Evidence::RootChanged(_) | Evidence::ParentChanged(_) => {
                self.confirmed_missing = false;
            }
            _ => {}
        }
    }

    /// Sample at a point serialized with discovery delivery and read activity.
    /// Framing, delivery and durable progress remain separate preconditions.
    pub(crate) fn discovery_and_eof_inactivity_met(
        &self,
        now: Instant,
        rotation: &Rotation,
    ) -> bool {
        self.confirmed_missing
            && self.eof_since.is_some_and(|eof| {
                now.checked_duration_since(eof)
                    .is_some_and(|idle| idle >= rotation.rotate_wait)
            })
    }
}
