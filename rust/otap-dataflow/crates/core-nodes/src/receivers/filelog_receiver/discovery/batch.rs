// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reusable observation storage and ordered completion accounting; no filesystem I/O.

use super::{Observation, ScanCompletion, ScanLimits, ScanReason};
use std::path::{Path, PathBuf};

// Retain allocation capacity across delivery and subsequent passes. The consumer
// borrows only the initialized prefix; no descriptor or unbounded queue lives here.
pub(super) struct Batch {
    slots: Vec<Observation>,
    used: usize,
}

impl Batch {
    pub(super) fn new(limits: ScanLimits) -> Self {
        Self {
            slots: (0..limits.batch)
                .map(|_| Observation {
                    path: PathBuf::with_capacity(limits.path_bytes),
                    locator: None,
                    size: None,
                    modified: None,
                    fingerprint: Vec::with_capacity(limits.fingerprint_bytes),
                    reason: None,
                    errno: None,
                })
                .collect(),
            used: 0,
        }
    }
    pub(super) fn heap_bytes(&self) -> usize {
        self.slots.capacity() * size_of::<Observation>()
            + self
                .slots
                .iter()
                .map(|o| o.path.capacity() + o.fingerprint.capacity())
                .sum::<usize>()
    }
    pub(super) fn clear(&mut self) {
        self.used = 0;
    }
    pub(super) fn is_full(&self) -> bool {
        self.used == self.slots.len()
    }
    pub(super) fn emit(&mut self, path: &Path, reason: ScanReason, errno: Option<i32>) {
        // Overflow is an internal invariant violation: stop rather than lose
        // evidence and risk reporting incomplete coverage as complete.
        assert!(!self.is_full(), "deliver a full batch before emitting");
        let out = &mut self.slots[self.used];
        self.used += 1;
        out.path.clear();
        out.path.push(path);
        out.locator = None;
        out.size = None;
        out.modified = None;
        out.fingerprint.clear();
        out.reason = Some(reason);
        out.errno = errno;
    }
    pub(super) fn last_mut(&mut self) -> &mut Observation {
        &mut self.slots[self.used - 1]
    }
}
impl std::ops::Deref for Batch {
    type Target = [Observation];
    fn deref(&self) -> &Self::Target {
        &self.slots[..self.used]
    }
}

pub(super) fn deliver(
    batch: &mut Batch,
    completion: &mut ScanCompletion,
    consume: &mut impl FnMut(&[Observation]) -> bool,
) -> bool {
    if batch.is_empty() {
        return !completion.cancelled;
    }
    for observation in batch.iter() {
        if observation
            .reason
            .is_some_and(ScanReason::invalidates_coverage)
        {
            completion.complete = false;
        }
        if observation.reason == Some(ScanReason::Cancelled) {
            completion.cancelled = true;
        }
    }
    completion.observations = completion.observations.saturating_add(batch.len() as u64);
    if !consume(batch) {
        completion.complete = false;
        completion.cancelled = true;
    }
    batch.clear();
    !completion.cancelled
}

#[cfg(test)]
mod tests {
    use super::super::FileLocator;
    use super::*;

    fn completion() -> ScanCompletion {
        ScanCompletion {
            generation: 17,
            complete: true,
            cancelled: false,
            observations: 0,
        }
    }

    /// Scenario: Each outcome-table reason is delivered, including failures that retain locator presence.
    /// Guarantees: Completion follows the coverage contract without discarding presence or changing pass ordering.
    #[test]
    fn delivery_follows_outcome_table_and_preserves_presence() {
        use ScanReason::*;
        let cases = [
            (None, false),
            (Some(Directory), false),
            (Some(PresenceOnly), false),
            (Some(Missing), false),
            (Some(LeafLoop), false),
            (Some(Unsupported), false),
            (Some(Excluded), false),
            (Some(Content), false),
            (Some(VerificationUnavailable), true),
            (Some(RootUnavailable), true),
            (Some(RootChanged), true),
            (Some(Coverage), true),
            (Some(Resumption), true),
            (Some(UnqualifiedResumption), true),
            (Some(Cycle), true),
            (Some(Limit), true),
            (Some(Policy), true),
            (Some(Unstable), true),
            (Some(ProbeBusy), true),
            (Some(DescriptorExhausted), true),
            (Some(Cancelled), true),
        ];
        for (reason, invalidates) in cases {
            let mut batch = Batch::new(ScanLimits {
                batch: 1,
                ..ScanLimits::default()
            });
            batch.emit(Path::new("entry"), reason.unwrap_or(PresenceOnly), None);
            let presence = matches!(
                reason,
                Some(PresenceOnly | Excluded | Content | Policy | Unstable)
            )
            .then_some(FileLocator {
                device: 1,
                inode: 2,
            });
            batch.last_mut().reason = reason;
            batch.last_mut().locator = presence;
            let mut end = completion();
            let mut calls = 0;
            let continued = deliver(&mut batch, &mut end, &mut |observations| {
                calls += 1;
                assert_eq!(observations.len(), 1);
                assert_eq!(observations[0].reason, reason);
                assert_eq!(observations[0].locator, presence);
                true
            });
            assert_eq!(calls, 1);
            assert_eq!(end.generation, 17);
            assert_eq!(end.observations, 1);
            assert_eq!(end.complete, !invalidates, "{reason:?}");
            assert_eq!(end.cancelled, reason == Some(Cancelled));
            assert_eq!(continued, !end.cancelled);
            assert!(batch.is_empty());
        }
    }

    /// Scenario: Positive observations arrive after coverage has failed and the observation counter is near its limit.
    /// Guarantees: Delivery cannot restore completeness or wrap the counter, and preserves observation order.
    #[test]
    fn delivery_keeps_failure_and_saturates_count() {
        let mut batch = Batch::new(ScanLimits {
            batch: 2,
            ..ScanLimits::default()
        });
        batch.emit(Path::new("first"), ScanReason::PresenceOnly, None);
        batch.emit(Path::new("second"), ScanReason::Content, None);
        let mut end = completion();
        end.complete = false;
        end.observations = u64::MAX - 1;
        assert!(deliver(&mut batch, &mut end, &mut |observations| {
            assert_eq!(observations[0].path, Path::new("first"));
            assert_eq!(observations[1].path, Path::new("second"));
            true
        }));
        assert!(!end.complete && !end.cancelled);
        assert_eq!(end.observations, u64::MAX);
        assert!(batch.is_empty());
    }

    /// Scenario: A consumer cancels a batch and delivery is called again without new observations.
    /// Guarantees: Cancellation is sticky, marks coverage incomplete, and does not redeliver cleared observations.
    #[test]
    fn consumer_cancellation_is_sticky() {
        let mut batch = Batch::new(ScanLimits::default());
        batch.emit(Path::new("entry"), ScanReason::PresenceOnly, None);
        let mut end = completion();
        assert!(!deliver(&mut batch, &mut end, &mut |_| false));
        assert!(end.cancelled && !end.complete);
        assert_eq!(end.observations, 1);
        assert!(batch.is_empty());
        assert!(!deliver(&mut batch, &mut end, &mut |_| panic!(
            "empty batch must not be delivered"
        )));
        assert_eq!(end.observations, 1);
    }

    /// Scenario: A reusable slot has all optional fields populated before recording a missing name.
    /// Guarantees: No prior locator, metadata, fingerprint, or errno leaks into the next observation; capacity is retained.
    #[test]
    fn reused_slot_clears_evidence_without_reallocating() {
        let mut batch = Batch::new(ScanLimits {
            batch: 1,
            ..ScanLimits::default()
        });
        let heap_bytes = batch.heap_bytes();
        batch.emit(Path::new("old"), ScanReason::Content, Some(1));
        let old = batch.last_mut();
        old.locator = Some(FileLocator {
            device: 1,
            inode: 2,
        });
        old.size = Some(3);
        old.modified = Some(std::time::UNIX_EPOCH);
        old.fingerprint.extend_from_slice(b"abc");
        batch.clear();
        batch.emit(Path::new("new"), ScanReason::Missing, None);
        let new = batch.last_mut();
        assert_eq!(new.path, Path::new("new"));
        assert_eq!(new.reason, Some(ScanReason::Missing));
        assert!(new.locator.is_none() && new.size.is_none() && new.modified.is_none());
        assert!(new.fingerprint.is_empty() && new.errno.is_none());
        assert_eq!(batch.heap_bytes(), heap_bytes);
    }
}
