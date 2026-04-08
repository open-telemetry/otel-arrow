// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deferred retry scheduling for `durable_buffer_processor`.
//!
//! Durable buffer needs local per-bundle retry state even though it only uses
//! one engine wakeup slot. This module owns that state and keeps the two layers
//! aligned:
//!
//! - the processor tracks every deferred bundle locally by bundle identity
//! - a local ordered index gives the next retry to resume
//! - the single engine wakeup slot is always armed to the earliest deferred
//!   retry deadline
//! - wakeup revisions are used to ignore stale wakeups after re-arming
//!
//! This keeps retry scheduling on one mechanism under heavy NACK pressure
//! instead of splitting between per-bundle wakeups and a separate overflow
//! path.
//!
//! Guarantees:
//!
//! - a deferred bundle is held out of the normal poll loop until it is resumed
//!   or explicitly re-deferred
//! - due retries are resumed in deadline order, with deterministic ordering for
//!   equal deadlines
//! - this module does not introduce any growth path beyond the number of
//!   deferred bundles: it keeps one authoritative map entry and one ordered
//!   index entry per deferred bundle, plus at most one armed wakeup record
//! - durable buffer retry scheduling does not depend on having one engine
//!   wakeup slot per deferred bundle

use otap_df_engine::WakeupError;
use otap_df_engine::control::{WakeupRevision, WakeupSlot};
use otap_df_engine::local::processor::EffectHandler;
use otap_df_otap::pdata::OtapPdata;
use quiver::subscriber::BundleRef;
use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

/// Durable buffer uses one processor-local wakeup slot for "the earliest retry
/// currently pending in local state".
pub(super) const RETRY_WAKEUP_SLOT: WakeupSlot = WakeupSlot(0);

/// Convert a Quiver bundle identity into the stable key used by retry state.
pub(super) fn retry_key(bundle_ref: BundleRef) -> (u64, u32) {
    (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw())
}

/// Local deferred retry state for one bundle.
///
/// Durable buffer keeps retry scheduling state locally and only uses the engine
/// wakeup API to re-arm the earliest pending retry deadline.
#[derive(Clone, Copy)]
pub(super) struct DeferredRetry {
    bundle_ref: BundleRef,
    retry_count: u32,
    retry_at: Instant,
    sequence: u64,
}

impl DeferredRetry {
    const fn new(
        bundle_ref: BundleRef,
        retry_count: u32,
        retry_at: Instant,
        sequence: u64,
    ) -> Self {
        Self {
            bundle_ref,
            retry_count,
            retry_at,
            sequence,
        }
    }

    pub(super) const fn bundle_ref(self) -> BundleRef {
        self.bundle_ref
    }

    pub(super) const fn retry_count(self) -> u32 {
        self.retry_count
    }
}

/// Tracks the engine wakeup currently armed for the earliest deferred retry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArmedRetryWakeup {
    when: Instant,
    revision: WakeupRevision,
}

/// Ordering/index key for deferred retries.
///
/// Ordering is lexicographic by `(retry_at, sequence, key)`, which means:
/// - earlier retry deadlines are resumed first
/// - equal deadlines use insertion sequence as a deterministic tie-breaker
/// - `key` keeps the ordering total and points back to the authoritative
///   `DeferredRetry` stored in the map
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DeferredRetryOrder {
    retry_at: Instant,
    sequence: u64,
    key: (u64, u32),
}

/// Local deferred-retry scheduling state for durable buffer.
///
/// The processor keeps all retry deadlines locally and uses one engine wakeup
/// slot for "the earliest retry currently pending". This keeps the heavy-NACK
/// path on a single scheduling mechanism instead of splitting between many
/// armed wakeups and a separate overflow queue.
pub(super) struct DeferredRetryState {
    /// Authoritative retry state keyed by bundle identity.
    ///
    /// Invariant: every deferred bundle appears exactly once here and exactly
    /// once in `deferred_order`.
    deferred: HashMap<(u64, u32), DeferredRetry>,

    /// Due-order index for deferred retries.
    deferred_order: BTreeSet<DeferredRetryOrder>,

    /// Engine wakeup currently armed for the earliest deferred retry, if any.
    ///
    /// Invariant: when present, `when` equals the earliest currently armed
    /// retry deadline as seen by the engine wakeup API, and `revision` is the
    /// only wakeup revision allowed to trigger that arm.
    armed_wakeup: Option<ArmedRetryWakeup>,

    /// Monotonic tie-breaker for equal-deadline ordering.
    next_sequence: u64,
}

impl DeferredRetryState {
    pub(super) fn new() -> Self {
        Self {
            deferred: HashMap::new(),
            deferred_order: BTreeSet::new(),
            armed_wakeup: None,
            next_sequence: 0,
        }
    }

    pub(super) fn scheduled_len(&self) -> usize {
        self.deferred.len()
    }

    pub(super) fn is_deferred_key(&self, key: (u64, u32)) -> bool {
        self.deferred.contains_key(&key)
    }

    fn deferred_order(key: (u64, u32), retry: DeferredRetry) -> DeferredRetryOrder {
        DeferredRetryOrder {
            retry_at: retry.retry_at,
            sequence: retry.sequence,
            key,
        }
    }

    fn remove_deferred(&mut self, key: (u64, u32)) -> Option<DeferredRetry> {
        let retry = self.deferred.remove(&key)?;
        let _ = self
            .deferred_order
            .remove(&Self::deferred_order(key, retry));
        Some(retry)
    }

    fn insert_deferred(&mut self, bundle_ref: BundleRef, retry_count: u32, retry_at: Instant) {
        let key = retry_key(bundle_ref);
        let _ = self.remove_deferred(key);
        let retry = DeferredRetry::new(bundle_ref, retry_count, retry_at, self.next_sequence);
        self.next_sequence = self.next_sequence.saturating_add(1);
        let _ = self.deferred.insert(key, retry);
        let _ = self.deferred_order.insert(Self::deferred_order(key, retry));
    }

    fn desired_wakeup_at(&self, no_earlier_than: Option<Instant>) -> Option<Instant> {
        let earliest = self.deferred_order.first().map(|order| order.retry_at)?;
        Some(match no_earlier_than {
            Some(not_before) if earliest < not_before => not_before,
            _ => earliest,
        })
    }

    fn sync_armed_wakeup(
        &mut self,
        effect_handler: &mut EffectHandler<OtapPdata>,
        no_earlier_than: Option<Instant>,
    ) -> Result<(), WakeupError> {
        let Some(when) = self.desired_wakeup_at(no_earlier_than) else {
            if self.armed_wakeup.is_some() {
                let _ = effect_handler.cancel_wakeup(RETRY_WAKEUP_SLOT);
                self.armed_wakeup = None;
            }
            return Ok(());
        };

        if self
            .armed_wakeup
            .is_some_and(|armed_wakeup| armed_wakeup.when == when)
        {
            return Ok(());
        }

        let revision = effect_handler
            .set_wakeup(RETRY_WAKEUP_SLOT, when)?
            .revision();
        self.armed_wakeup = Some(ArmedRetryWakeup { when, revision });
        Ok(())
    }

    /// Schedule or re-schedule retry deferral for a bundle.
    ///
    /// Guarantees:
    /// - the bundle remains deferred in local state until retry resumption
    /// - the single engine wakeup always tracks the earliest deferred retry
    /// - returns `false` only when the wakeup could not be armed
    pub(super) fn schedule_at(
        &mut self,
        bundle_ref: BundleRef,
        retry_count: u32,
        retry_at: Instant,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> bool {
        let key = retry_key(bundle_ref);
        self.insert_deferred(bundle_ref, retry_count, retry_at);
        match self.sync_armed_wakeup(effect_handler, None) {
            Ok(()) => true,
            Err(error) => {
                let _ = self.remove_deferred(key);
                debug_assert_ne!(
                    error,
                    WakeupError::Capacity,
                    "single-slot durable-buffer wakeup should not hit capacity"
                );
                false
            }
        }
    }

    /// Schedule or re-schedule retry deferral after a relative delay.
    ///
    /// Guarantees:
    /// - equivalent to `schedule_at(now + delay)`
    /// - keeps the delay-to-deadline conversion local to deferred retry state
    pub(super) fn schedule_after(
        &mut self,
        bundle_ref: BundleRef,
        retry_count: u32,
        delay: Duration,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> bool {
        self.schedule_at(
            bundle_ref,
            retry_count,
            Instant::now() + delay,
            effect_handler,
        )
    }

    /// Accept one wakeup delivery only when it matches the currently armed
    /// durable-buffer slot and revision.
    ///
    /// Guarantees:
    /// - unrelated slots are ignored
    /// - stale revisions are ignored
    /// - the matching wakeup clears the armed state exactly once
    pub(super) fn accept_wakeup(&mut self, slot: WakeupSlot, revision: WakeupRevision) -> bool {
        if slot != RETRY_WAKEUP_SLOT {
            return false;
        }

        let Some(armed_wakeup) = self.armed_wakeup else {
            return false;
        };

        if armed_wakeup.revision != revision {
            return false;
        }

        self.armed_wakeup = None;
        true
    }

    /// Pop the next deferred retry only when its due time has arrived.
    ///
    /// Guarantee: returning a retry clears all local deferred bookkeeping for
    /// that bundle so it can be resumed exactly once.
    pub(super) fn take_due_retry(&mut self, now: Instant) -> Option<DeferredRetry> {
        let order = *self.deferred_order.first()?;
        if order.retry_at > now {
            return None;
        }

        let _ = self.deferred_order.remove(&order);
        let retry = self.deferred.remove(&order.key)?;
        Some(retry)
    }

    /// Drop all deferred retry gating at shutdown entry.
    ///
    /// Guarantees:
    /// - no bundle remains blocked behind local retry backoff state
    /// - the armed wakeup record is cleared
    /// - previously deferred bundles can be drained through the normal Quiver
    ///   poll path during shutdown
    pub(super) fn clear_for_shutdown(&mut self) {
        self.deferred.clear();
        self.deferred_order.clear();
        self.armed_wakeup = None;
    }

    /// Re-arm the single durable-buffer wakeup after retry processing.
    ///
    /// `no_earlier_than` lets the caller push the next retry attempt out when
    /// retries are already due but resend is currently blocked by flow control.
    pub(super) fn rearm_after_processing(
        &mut self,
        effect_handler: &mut EffectHandler<OtapPdata>,
        no_earlier_than: Option<Instant>,
    ) -> bool {
        match self.sync_armed_wakeup(effect_handler, no_earlier_than) {
            Ok(()) => true,
            Err(error) => {
                debug_assert_ne!(
                    error,
                    WakeupError::Capacity,
                    "single-slot durable-buffer wakeup should not hit capacity"
                );
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quiver::segment::SegmentSeq;
    use quiver::subscriber::BundleIndex;
    use std::time::Duration;

    /// Scenario: one deferred retry is currently the earliest retry, and the
    /// processor computes the wakeup it should arm next.
    /// Guarantees: the single durable-buffer wakeup targets that earliest retry.
    #[test]
    fn desired_wakeup_tracks_earliest_retry() {
        let mut state = DeferredRetryState::new();
        let now = Instant::now();

        state.insert_deferred(
            BundleRef {
                segment_seq: SegmentSeq::new(1),
                bundle_index: BundleIndex::new(1),
            },
            1,
            now + Duration::from_secs(3),
        );
        state.insert_deferred(
            BundleRef {
                segment_seq: SegmentSeq::new(2),
                bundle_index: BundleIndex::new(2),
            },
            2,
            now + Duration::from_secs(1),
        );

        assert_eq!(
            state.desired_wakeup_at(None),
            Some(now + Duration::from_secs(1))
        );
    }

    /// Scenario: retries are already due, but resend is currently blocked and
    /// the processor wants to defer the next retry attempt by one poll interval.
    /// Guarantees: the next wakeup is not armed earlier than the supplied floor.
    #[test]
    fn desired_wakeup_respects_retry_floor() {
        let mut state = DeferredRetryState::new();
        let now = Instant::now();

        state.insert_deferred(
            BundleRef {
                segment_seq: SegmentSeq::new(9),
                bundle_index: BundleIndex::new(1),
            },
            1,
            now - Duration::from_millis(1),
        );

        assert_eq!(
            state.desired_wakeup_at(Some(now + Duration::from_secs(1))),
            Some(now + Duration::from_secs(1))
        );
    }

    /// Scenario: the processor receives the exact wakeup revision currently
    /// armed for the durable-buffer retry slot.
    /// Guarantees: that wakeup is accepted and clears the armed state exactly once.
    #[test]
    fn accept_wakeup_clears_matching_arm() {
        let mut state = DeferredRetryState::new();
        let now = Instant::now();
        state.armed_wakeup = Some(ArmedRetryWakeup {
            when: now,
            revision: 17,
        });

        assert!(state.accept_wakeup(RETRY_WAKEUP_SLOT, 17));
        assert!(state.armed_wakeup.is_none());
        assert!(!state.accept_wakeup(RETRY_WAKEUP_SLOT, 17));
    }

    /// Scenario: the processor receives a wakeup for the retry slot, but the
    /// revision is stale relative to the currently armed wakeup.
    /// Guarantees: the stale wakeup is ignored and the armed wakeup remains.
    #[test]
    fn accept_wakeup_ignores_stale_revision() {
        let mut state = DeferredRetryState::new();
        let now = Instant::now();
        state.armed_wakeup = Some(ArmedRetryWakeup {
            when: now,
            revision: 5,
        });

        assert!(!state.accept_wakeup(RETRY_WAKEUP_SLOT, 4));
        assert_eq!(
            state.armed_wakeup,
            Some(ArmedRetryWakeup {
                when: now,
                revision: 5,
            })
        );
    }

    /// Scenario: the processor receives a wakeup for some unrelated slot.
    /// Guarantees: the unrelated wakeup is ignored and armed retry state remains.
    #[test]
    fn accept_wakeup_ignores_unrelated_slot() {
        let mut state = DeferredRetryState::new();
        let now = Instant::now();
        state.armed_wakeup = Some(ArmedRetryWakeup {
            when: now,
            revision: 3,
        });

        assert!(!state.accept_wakeup(WakeupSlot(999), 3));
        assert!(state.armed_wakeup.is_some());
    }

    /// Scenario: one deferred retry becomes due and is popped for retry resumption.
    /// Guarantees: taking that retry clears all local deferred bookkeeping.
    #[test]
    fn take_due_retry_clears_tracking() {
        let mut state = DeferredRetryState::new();
        let bundle_ref = BundleRef {
            segment_seq: SegmentSeq::new(321),
            bundle_index: BundleIndex::new(7),
        };
        let key = retry_key(bundle_ref);
        let retry_at = Instant::now();

        state.insert_deferred(bundle_ref, 4, retry_at);

        assert!(state.deferred.contains_key(&key));
        assert_eq!(state.deferred_order.len(), 1);

        let retry = state
            .take_due_retry(retry_at + Duration::from_millis(1))
            .expect("retry should be due");

        assert_eq!(retry.bundle_ref().segment_seq.raw(), 321);
        assert_eq!(retry.bundle_ref().bundle_index.raw(), 7);
        assert_eq!(retry.retry_count(), 4);
        assert!(!state.deferred.contains_key(&key));
        assert!(state.deferred_order.is_empty());
    }

    /// Scenario: multiple retries become due at the same timestamp.
    /// Guarantees: equal-deadline retries are resumed in insertion order via sequence.
    #[test]
    fn equal_deadline_retries_follow_sequence_order() {
        let mut state = DeferredRetryState::new();
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

        state.insert_deferred(first, 1, retry_at);
        state.insert_deferred(second, 2, retry_at);
        state.insert_deferred(third, 3, retry_at);

        assert_eq!(
            state
                .take_due_retry(retry_at + Duration::from_millis(1))
                .expect("first retry")
                .bundle_ref(),
            first
        );
        assert_eq!(
            state
                .take_due_retry(retry_at + Duration::from_millis(1))
                .expect("second retry")
                .bundle_ref(),
            second
        );
        assert_eq!(
            state
                .take_due_retry(retry_at + Duration::from_millis(1))
                .expect("third retry")
                .bundle_ref(),
            third
        );
        assert!(state.deferred.is_empty());
        assert!(state.deferred_order.is_empty());
    }

    /// Scenario: durable buffer starts shutdown while it still has deferred
    /// retries tracked locally behind its single retry wakeup.
    /// Guarantees: shutdown clearing removes all local retry gating and the
    /// armed wakeup record so those bundles can be drained through the normal
    /// poll path.
    #[test]
    fn clear_for_shutdown_drops_deferred_tracking() {
        let mut state = DeferredRetryState::new();
        let retry_at = Instant::now() + Duration::from_secs(1);
        state.insert_deferred(
            BundleRef {
                segment_seq: SegmentSeq::new(7),
                bundle_index: BundleIndex::new(1),
            },
            1,
            retry_at,
        );
        state.armed_wakeup = Some(ArmedRetryWakeup {
            when: retry_at,
            revision: 9,
        });

        state.clear_for_shutdown();

        assert!(state.deferred.is_empty());
        assert!(state.deferred_order.is_empty());
        assert!(state.armed_wakeup.is_none());
    }
}
