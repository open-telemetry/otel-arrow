// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otap_df_engine::control::{WakeupRevision, WakeupSlot};
use otap_df_engine::local::processor::EffectHandler;
use otap_df_engine::{WakeupError, WakeupSetOutcome};
use otap_df_otap::pdata::OtapPdata;
use quiver::subscriber::BundleRef;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::time::Instant;

/// Convert a Quiver bundle identity into the stable key used by retry state.
pub(super) fn retry_key(bundle_ref: BundleRef) -> (u64, u32) {
    (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw())
}

/// Encodes a durable-buffer bundle identity into a processor-local wakeup slot.
///
/// Layout: `[segment_seq:u64 | bundle_index:u64]`
pub(super) const fn retry_wakeup_slot(key: (u64, u32)) -> WakeupSlot {
    WakeupSlot(((key.0 as u128) << 64) | (key.1 as u128))
}

/// Retry state for a bundle that has already acquired an engine wakeup slot.
///
/// This is the armed phase of retry deferral:
/// - Quiver has released the bundle claim via implicit defer
/// - the processor has successfully registered a node-local wakeup for that bundle
/// - the wakeup slot is the bundle key encoded directly via `retry_wakeup_slot(...)`
/// - `revision` is the current scheduler revision for that slot
///
/// The struct intentionally keeps only the minimum information needed to resume
/// the retry when the matching wakeup fires.
#[derive(Clone, Copy)]
pub(super) struct RetryWakeup {
    bundle_ref: BundleRef,
    retry_count: u32,
    revision: WakeupRevision,
}

impl RetryWakeup {
    const fn new(bundle_ref: BundleRef, retry_count: u32, revision: WakeupRevision) -> Self {
        Self {
            bundle_ref,
            retry_count,
            revision,
        }
    }

    pub(super) const fn bundle_ref(self) -> BundleRef {
        self.bundle_ref
    }

    pub(super) const fn retry_count(self) -> u32 {
        self.retry_count
    }
}

/// Retry state for a bundle that could not acquire an engine wakeup slot yet.
///
/// This is the local overflow phase of retry deferral, used when
/// `EffectHandler::set_wakeup(...)` returns `WakeupError::Capacity`.
///
/// Guarantees supported by this representation:
/// - the bundle still remains deferred and is kept out of `poll_next_bundle()`
///   through `retry_scheduled`
/// - the intended retry deadline is preserved in `retry_at`
/// - equal deadlines are ordered deterministically by `sequence`
///
/// `OverflowRetry` is stored in `retry_overflow` and indexed for due-order by
/// a matching `OverflowRetryOrder` entry in `retry_overflow_order`.
#[derive(Clone, Copy)]
struct OverflowRetry {
    bundle_ref: BundleRef,
    retry_count: u32,
    retry_at: Instant,
    sequence: u64,
}

/// Ordering/index key for `retry_overflow_order`.
///
/// This is kept separate from `OverflowRetry` so the processor can maintain:
/// - a keyed lookup map (`retry_overflow`) for exact replacement/removal
/// - an ordered set (`retry_overflow_order`) for "next due" selection
///
/// Ordering is lexicographic by `(retry_at, sequence, key)`, which means:
/// - earlier deadlines are resumed first
/// - equal deadlines use insertion sequence as a deterministic tie-breaker
/// - `key` keeps the ordering total and points back to the authoritative
///   `OverflowRetry` stored in the map
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct OverflowRetryOrder {
    retry_at: Instant,
    sequence: u64,
    key: (u64, u32),
}

/// Local wakeup bookkeeping for durable-buffer retry deferral.
///
/// This state owns the invariants around armed retries, overflow retries, and
/// deferred bundle membership. `DurableBuffer` itself still owns retry policy,
/// Quiver interaction, and downstream resend behavior.
pub(super) struct RetryWakeupState {
    /// Bundles currently held out of the normal poll loop while backoff is active.
    ///
    /// Invariant: every key here is deferred for retry either by an armed wakeup
    /// (`retry_wakeups`) or by the local overflow queue
    /// (`retry_overflow` + `retry_overflow_order`).
    retry_scheduled: HashSet<(u64, u32)>,

    /// Retry state keyed by wakeup slot.
    ///
    /// Invariant: each slot is the encoded bundle key for the matching
    /// `RetryWakeup.bundle_ref`, and `RetryWakeup.revision` is the only wakeup
    /// revision for that slot that is allowed to resume the retry.
    retry_wakeups: HashMap<WakeupSlot, RetryWakeup>,

    /// Retry state held locally while wakeup scheduling is at capacity.
    ///
    /// Guarantee: overflowed retries remain deferred and keep their target due
    /// time even when the engine wakeup scheduler is full.
    retry_overflow: HashMap<(u64, u32), OverflowRetry>,

    /// Due-order index for locally deferred retries.
    ///
    /// Invariant: this contains exactly one ordering key for each entry in
    /// `retry_overflow`, using `sequence` as a deterministic tie-breaker.
    retry_overflow_order: BTreeSet<OverflowRetryOrder>,

    /// Monotonic tie-breaker for locally deferred retry ordering.
    next_retry_overflow_sequence: u64,
}

impl RetryWakeupState {
    pub(super) fn new() -> Self {
        Self {
            retry_scheduled: HashSet::new(),
            retry_wakeups: HashMap::new(),
            retry_overflow: HashMap::new(),
            retry_overflow_order: BTreeSet::new(),
            next_retry_overflow_sequence: 0,
        }
    }

    pub(super) fn scheduled_len(&self) -> usize {
        self.retry_scheduled.len()
    }

    pub(super) fn is_deferred_key(&self, key: (u64, u32)) -> bool {
        self.retry_scheduled.contains(&key)
    }

    fn overflow_retry_order(key: (u64, u32), retry: OverflowRetry) -> OverflowRetryOrder {
        OverflowRetryOrder {
            retry_at: retry.retry_at,
            sequence: retry.sequence,
            key,
        }
    }

    /// Removes one overflowed retry from both local indexes.
    ///
    /// Invariant preserved: `retry_overflow` and `retry_overflow_order` stay in
    /// lockstep after every insertion/removal.
    fn remove_retry_overflow(&mut self, key: (u64, u32)) -> Option<OverflowRetry> {
        let retry = self.retry_overflow.remove(&key)?;
        let _ = self
            .retry_overflow_order
            .remove(&Self::overflow_retry_order(key, retry));
        Some(retry)
    }

    /// Defers a retry in local processor state when the engine wakeup scheduler
    /// has no free slot.
    ///
    /// Guarantees:
    /// - the bundle remains in `retry_scheduled`, so `poll_next_bundle()` keeps
    ///   skipping it
    /// - the most recent `(retry_count, retry_at)` replaces any older local
    ///   overflow record for the same bundle
    /// - equal due times are processed deterministically by `sequence`
    fn insert_retry_overflow(&mut self, bundle_ref: BundleRef, retry_count: u32, retry_at: Instant) {
        let key = retry_key(bundle_ref);
        let _ = self.remove_retry_overflow(key);
        let retry = OverflowRetry {
            bundle_ref,
            retry_count,
            retry_at,
            sequence: self.next_retry_overflow_sequence,
        };
        self.next_retry_overflow_sequence = self.next_retry_overflow_sequence.saturating_add(1);
        let _ = self.retry_scheduled.insert(key);
        let _ = self.retry_overflow.insert(key, retry);
        let _ = self
            .retry_overflow_order
            .insert(Self::overflow_retry_order(key, retry));
    }

    /// Pops the next locally deferred retry only when its due time has arrived.
    ///
    /// Guarantee: returning a retry clears all local overflow bookkeeping for
    /// that bundle so it can be resumed exactly once.
    pub(super) fn take_due_retry_overflow(&mut self, now: Instant) -> Option<RetryWakeup> {
        let order = *self.retry_overflow_order.first()?;
        if order.retry_at > now {
            return None;
        }

        let _ = self.retry_overflow_order.remove(&order);
        let retry = self.retry_overflow.remove(&order.key)?;
        let _ = self.retry_scheduled.remove(&order.key);

        Some(RetryWakeup::new(retry.bundle_ref, retry.retry_count, 0))
    }

    /// Opportunistically moves overflowed retries back into engine wakeup slots.
    ///
    /// Guarantees:
    /// - never drops a deferred retry when slot acquisition fails
    /// - preserves retry due time when promotion succeeds
    /// - stops as soon as the scheduler reports `Capacity` or shutdown
    pub(super) fn promote_overflow_to_wakeups(
        &mut self,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) {
        while let Some(order) = self.retry_overflow_order.first().copied() {
            let Some(retry) = self.retry_overflow.get(&order.key).copied() else {
                let _ = self.retry_overflow_order.remove(&order);
                continue;
            };

            let slot = retry_wakeup_slot(order.key);
            match effect_handler.set_wakeup(slot, retry.retry_at) {
                Ok(outcome) => {
                    let _ = self.retry_overflow_order.remove(&order);
                    let _ = self.retry_overflow.remove(&order.key);
                    let _ = self.retry_wakeups.insert(
                        slot,
                        RetryWakeup::new(
                            retry.bundle_ref,
                            retry.retry_count,
                            outcome.revision(),
                        ),
                    );
                }
                Err(WakeupError::Capacity | WakeupError::ShuttingDown) => break,
            }
        }
    }

    /// Schedule or re-schedule retry deferral for a bundle.
    ///
    /// Guarantees:
    /// - on success, the bundle remains deferred until either an armed wakeup
    ///   or local overflow retry resumes it
    /// - wakeup-capacity exhaustion falls back to local overflow state instead
    ///   of immediate re-polling
    /// - returns `false` only when the processor is already shutting down
    pub(super) fn schedule_at(
        &mut self,
        bundle_ref: BundleRef,
        retry_count: u32,
        retry_at: Instant,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> bool {
        let key = retry_key(bundle_ref);
        let _ = self.remove_retry_overflow(key);
        let slot = retry_wakeup_slot(key);
        match effect_handler.set_wakeup(slot, retry_at) {
            Ok(WakeupSetOutcome::Inserted { revision } | WakeupSetOutcome::Replaced { revision }) => {
                let _ = self.retry_scheduled.insert(key);
                let _ = self
                    .retry_wakeups
                    .insert(slot, RetryWakeup::new(bundle_ref, retry_count, revision));
                true
            }
            Err(WakeupError::Capacity) => {
                self.insert_retry_overflow(bundle_ref, retry_count, retry_at);
                true
            }
            Err(WakeupError::ShuttingDown) => false,
        }
    }

    /// Remove retry-wakeup tracking for a bundle now being resumed.
    ///
    /// Guarantee: taking a wakeup clears the armed-wakeup bookkeeping for that
    /// bundle before retry resumption starts.
    pub(super) fn take_retry_wakeup(
        &mut self,
        slot: WakeupSlot,
        revision: WakeupRevision,
    ) -> Option<RetryWakeup> {
        let wakeup = self.retry_wakeups.get(&slot).copied()?;
        if wakeup.revision != revision {
            return None;
        }

        let wakeup = self
            .retry_wakeups
            .remove(&slot)
            .expect("matching wakeup should still exist");
        let key = retry_key(wakeup.bundle_ref);
        let _ = self.retry_scheduled.remove(&key);
        Some(wakeup)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quiver::segment::SegmentSeq;
    use quiver::subscriber::BundleIndex;
    use std::time::Duration;

    /// Scenario: a retry is armed in the engine wakeup scheduler for one
    /// bundle, and the matching wakeup later arrives with the same revision.
    /// Guarantees: taking that wakeup clears the armed retry bookkeeping and
    /// returns the original `(bundle_ref, retry_count)` exactly once.
    #[test]
    fn take_retry_wakeup_clears_tracking() {
        let mut state = RetryWakeupState::new();
        let bundle_ref = BundleRef {
            segment_seq: SegmentSeq::new(98765),
            bundle_index: BundleIndex::new(123),
        };
        let key = retry_key(bundle_ref);
        let slot = retry_wakeup_slot(key);
        let _ = state.retry_scheduled.insert(key);
        let _ = state
            .retry_wakeups
            .insert(slot, RetryWakeup::new(bundle_ref, 3, 17));

        let taken = state
            .take_retry_wakeup(slot, 17)
            .expect("retry wakeup should exist");
        assert_eq!(taken.bundle_ref().segment_seq.raw(), 98765);
        assert_eq!(taken.bundle_ref().bundle_index.raw(), 123);
        assert_eq!(taken.retry_count(), 3);
        assert!(!state.retry_scheduled.contains(&key));
        assert!(!state.retry_wakeups.contains_key(&slot));
    }

    /// Scenario: the processor receives a wakeup for a slot that has no armed
    /// retry state.
    /// Guarantees: the unknown wakeup is ignored and does not mutate retry
    /// bookkeeping.
    #[test]
    fn take_retry_wakeup_unknown_slot_is_ignored() {
        let mut state = RetryWakeupState::new();
        assert!(state.take_retry_wakeup(WakeupSlot(999), 0).is_none());
    }

    /// Scenario: a slot has been rescheduled, so the processor still has armed
    /// retry state for that slot but the arriving wakeup carries an older
    /// revision.
    /// Guarantees: the stale wakeup is ignored, and the current armed retry
    /// state remains available for the matching revision.
    #[test]
    fn take_retry_wakeup_stale_revision_is_ignored() {
        let mut state = RetryWakeupState::new();
        let bundle_ref = BundleRef {
            segment_seq: SegmentSeq::new(123),
            bundle_index: BundleIndex::new(9),
        };
        let key = retry_key(bundle_ref);
        let slot = retry_wakeup_slot(key);

        let _ = state.retry_scheduled.insert(key);
        let _ = state
            .retry_wakeups
            .insert(slot, RetryWakeup::new(bundle_ref, 2, 5));

        assert!(state.take_retry_wakeup(slot, 4).is_none());
        assert!(state.retry_scheduled.contains(&key));
        assert!(state.retry_wakeups.contains_key(&slot));
    }

    /// Scenario: a retry was deferred in local overflow state because wakeup
    /// capacity was exhausted, and its due time has now arrived.
    /// Guarantees: taking that retry clears all overflow bookkeeping, removes it
    /// from `retry_scheduled`, and returns the original `(bundle_ref, retry_count)`.
    #[test]
    fn take_due_retry_overflow_clears_tracking() {
        let mut state = RetryWakeupState::new();
        let bundle_ref = BundleRef {
            segment_seq: SegmentSeq::new(321),
            bundle_index: BundleIndex::new(7),
        };
        let key = retry_key(bundle_ref);
        let retry_at = Instant::now();

        state.insert_retry_overflow(bundle_ref, 4, retry_at);

        assert!(state.retry_scheduled.contains(&key));
        assert!(state.retry_overflow.contains_key(&key));
        assert_eq!(state.retry_overflow_order.len(), 1);

        let retry = state
            .take_due_retry_overflow(retry_at + Duration::from_millis(1))
            .expect("retry should be due");

        assert_eq!(retry.bundle_ref().segment_seq.raw(), 321);
        assert_eq!(retry.bundle_ref().bundle_index.raw(), 7);
        assert_eq!(retry.retry_count(), 4);
        assert!(!state.retry_scheduled.contains(&key));
        assert!(!state.retry_overflow.contains_key(&key));
        assert!(state.retry_overflow_order.is_empty());
    }

    /// Scenario: multiple retries overflow the wakeup scheduler and are stored
    /// locally with the same due timestamp.
    /// Guarantees: equal-deadline overflow retries are resumed in insertion
    /// order using the local sequence tie-breaker.
    #[test]
    fn equal_deadline_overflow_retries_follow_sequence_order() {
        let mut state = RetryWakeupState::new();
        let retry_at = Instant::now();
        let first = BundleRef {
            segment_seq: SegmentSeq::new(111),
            bundle_index: BundleIndex::new(1),
        };
        let second = BundleRef {
            segment_seq: SegmentSeq::new(222),
            bundle_index: BundleIndex::new(2),
        };
        let third = BundleRef {
            segment_seq: SegmentSeq::new(333),
            bundle_index: BundleIndex::new(3),
        };

        state.insert_retry_overflow(first, 1, retry_at);
        state.insert_retry_overflow(second, 2, retry_at);
        state.insert_retry_overflow(third, 3, retry_at);

        assert_eq!(
            state
                .take_due_retry_overflow(retry_at + Duration::from_millis(1))
                .expect("first retry")
                .bundle_ref(),
            first
        );
        assert_eq!(
            state
                .take_due_retry_overflow(retry_at + Duration::from_millis(1))
                .expect("second retry")
                .bundle_ref(),
            second
        );
        assert_eq!(
            state
                .take_due_retry_overflow(retry_at + Duration::from_millis(1))
                .expect("third retry")
                .bundle_ref(),
            third
        );
        assert!(state.retry_overflow.is_empty());
        assert!(state.retry_overflow_order.is_empty());
    }
}
