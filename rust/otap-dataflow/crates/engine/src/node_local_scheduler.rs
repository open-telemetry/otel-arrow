// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node-local wakeup scheduling for processor inboxes.

use crate::control::{WakeupRevision, WakeupSlot};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::Notify;

/// Error returned when a wakeup request cannot be accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeupError {
    /// Processor-local wakeups were not enabled for this processor runtime.
    Unsupported,
    /// The processor has already latched shutdown.
    ShuttingDown,
    /// The bounded live wakeup slot set is full.
    Capacity,
}

/// Outcome of setting a processor-local wakeup slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeupSetOutcome {
    /// A new live slot was inserted into the scheduler.
    Inserted {
        /// Scheduler-assigned revision for the live wakeup now stored in the slot.
        revision: WakeupRevision,
    },
    /// An existing live slot was updated in place.
    Replaced {
        /// Scheduler-assigned revision for the replacement wakeup now stored in the slot.
        revision: WakeupRevision,
    },
}

impl WakeupSetOutcome {
    /// Returns the scheduler-assigned revision for the accepted wakeup.
    #[must_use]
    pub const fn revision(self) -> WakeupRevision {
        match self {
            Self::Inserted { revision } | Self::Replaced { revision } => revision,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ScheduledWakeup {
    slot: WakeupSlot,
    when: Instant,
    revision: WakeupRevision,
}

struct NodeLocalScheduler {
    wakeup_capacity: usize,
    next_revision: WakeupRevision,
    wakeups: Vec<ScheduledWakeup>,
    wakeup_indices: HashMap<WakeupSlot, usize>,
    shutting_down: bool,
}

impl NodeLocalScheduler {
    fn new(wakeup_capacity: usize) -> Self {
        Self {
            wakeup_capacity,
            next_revision: 0,
            wakeups: Vec::new(),
            wakeup_indices: HashMap::new(),
            shutting_down: false,
        }
    }

    fn next_revision(&mut self) -> WakeupRevision {
        let next = self.next_revision;
        self.next_revision = self.next_revision.saturating_add(1);
        next
    }

    fn wakeup_precedes(left: &ScheduledWakeup, right: &ScheduledWakeup) -> bool {
        left.when < right.when || (left.when == right.when && left.revision < right.revision)
    }

    fn swap_entries(&mut self, left: usize, right: usize) {
        if left == right {
            return;
        }

        self.wakeups.swap(left, right);

        let left_slot = self.wakeups[left].slot;
        let right_slot = self.wakeups[right].slot;
        let _ = self
            .wakeup_indices
            .insert(left_slot, left)
            .expect("left slot index should exist");
        let _ = self
            .wakeup_indices
            .insert(right_slot, right)
            .expect("right slot index should exist");
    }

    fn sift_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if !Self::wakeup_precedes(&self.wakeups[index], &self.wakeups[parent]) {
                break;
            }
            self.swap_entries(index, parent);
            index = parent;
        }
    }

    fn sift_down(&mut self, mut index: usize) {
        let len = self.wakeups.len();
        loop {
            let left = index * 2 + 1;
            if left >= len {
                break;
            }

            let right = left + 1;
            let mut smallest = left;
            if right < len && Self::wakeup_precedes(&self.wakeups[right], &self.wakeups[left]) {
                smallest = right;
            }

            if !Self::wakeup_precedes(&self.wakeups[smallest], &self.wakeups[index]) {
                break;
            }

            self.swap_entries(index, smallest);
            index = smallest;
        }
    }

    fn repair_heap_at(&mut self, index: usize) {
        if index > 0 {
            let parent = (index - 1) / 2;
            if Self::wakeup_precedes(&self.wakeups[index], &self.wakeups[parent]) {
                self.sift_up(index);
                return;
            }
        }
        self.sift_down(index);
    }

    fn remove_heap_entry(&mut self, index: usize) -> ScheduledWakeup {
        let last = self
            .wakeups
            .len()
            .checked_sub(1)
            .expect("heap entry removal requires a non-empty heap");

        if index == last {
            return self.wakeups.pop().expect("last wakeup should exist");
        }

        self.wakeups.swap(index, last);
        let removed = self.wakeups.pop().expect("removed wakeup should exist");

        let moved_slot = self.wakeups[index].slot;
        let _ = self
            .wakeup_indices
            .insert(moved_slot, index)
            .expect("moved slot index should exist");
        self.repair_heap_at(index);
        removed
    }

    fn set_wakeup(
        &mut self,
        slot: WakeupSlot,
        when: Instant,
    ) -> Result<WakeupSetOutcome, WakeupError> {
        if self.shutting_down {
            return Err(WakeupError::ShuttingDown);
        }

        if let Some(&index) = self.wakeup_indices.get(&slot) {
            let revision = self.next_revision();
            self.wakeups[index].when = when;
            self.wakeups[index].revision = revision;
            self.repair_heap_at(index);
            Ok(WakeupSetOutcome::Replaced { revision })
        } else {
            if self.wakeup_indices.len() >= self.wakeup_capacity {
                return Err(WakeupError::Capacity);
            }
            let revision = self.next_revision();
            let index = self.wakeups.len();
            self.wakeups.push(ScheduledWakeup {
                slot,
                when,
                revision,
            });
            assert!(
                self.wakeup_indices.insert(slot, index).is_none(),
                "new wakeup slot should not already exist"
            );
            self.sift_up(index);
            Ok(WakeupSetOutcome::Inserted { revision })
        }
    }

    fn cancel_wakeup(&mut self, slot: WakeupSlot) -> bool {
        if self.shutting_down {
            return false;
        }

        let Some(index) = self.wakeup_indices.remove(&slot) else {
            return false;
        };

        let removed = self.remove_heap_entry(index);
        debug_assert_eq!(removed.slot, slot);
        true
    }

    #[cfg(debug_assertions)]
    fn assert_consistent(&self) {
        assert_eq!(self.wakeups.len(), self.wakeup_indices.len());

        for (index, wakeup) in self.wakeups.iter().enumerate() {
            assert_eq!(
                self.wakeup_indices.get(&wakeup.slot).copied(),
                Some(index),
                "heap index must match map entry"
            );

            if index > 0 {
                let parent = (index - 1) / 2;
                assert!(
                    !Self::wakeup_precedes(&self.wakeups[index], &self.wakeups[parent]),
                    "heap child must not precede parent"
                );
            }
        }
    }

    fn next_expiry(&mut self) -> Option<Instant> {
        #[cfg(debug_assertions)]
        self.assert_consistent();
        self.wakeups.first().map(|wakeup| wakeup.when)
    }

    fn pop_due(&mut self, now: Instant) -> Option<(WakeupSlot, Instant, WakeupRevision)> {
        #[cfg(debug_assertions)]
        self.assert_consistent();

        let next_due = self.wakeups.first().map(|wakeup| wakeup.when)?;
        if next_due > now {
            return None;
        }

        let slot = self.wakeups.first().expect("due wakeup should exist").slot;
        let removed_index = self
            .wakeup_indices
            .remove(&slot)
            .expect("due wakeup slot index should exist");
        debug_assert_eq!(removed_index, 0);
        let wakeup = self.remove_heap_entry(0);
        Some((wakeup.slot, wakeup.when, wakeup.revision))
    }

    fn begin_shutdown(&mut self) {
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;
        self.wakeup_indices.clear();
        self.wakeups.clear();
    }

    fn is_drained(&self) -> bool {
        self.wakeup_indices.is_empty()
    }
}

/// Shared handle used by the processor inbox and the processor effect handler.
pub(crate) struct NodeLocalSchedulerHandle {
    inner: Arc<Mutex<NodeLocalScheduler>>,
    notify: Arc<Notify>,
}

impl Clone for NodeLocalSchedulerHandle {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            notify: Arc::clone(&self.notify),
        }
    }
}

impl NodeLocalSchedulerHandle {
    pub(crate) fn new(wakeup_capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(NodeLocalScheduler::new(wakeup_capacity))),
            notify: Arc::new(Notify::new()),
        }
    }

    fn with_scheduler<R>(&self, f: impl FnOnce(&mut NodeLocalScheduler) -> R) -> R {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut guard)
    }

    pub(crate) fn set_wakeup(
        &self,
        slot: WakeupSlot,
        when: Instant,
    ) -> Result<WakeupSetOutcome, WakeupError> {
        let result = self.with_scheduler(|scheduler| scheduler.set_wakeup(slot, when));
        if result.is_ok() {
            self.notify.notify_one();
        }
        result
    }

    #[must_use]
    pub(crate) fn cancel_wakeup(&self, slot: WakeupSlot) -> bool {
        let changed = self.with_scheduler(|scheduler| scheduler.cancel_wakeup(slot));
        if changed {
            self.notify.notify_one();
        }
        changed
    }

    pub(crate) fn next_expiry(&self) -> Option<Instant> {
        self.with_scheduler(NodeLocalScheduler::next_expiry)
    }

    pub(crate) fn pop_due(&self, now: Instant) -> Option<(WakeupSlot, Instant, WakeupRevision)> {
        self.with_scheduler(|scheduler| scheduler.pop_due(now))
    }

    pub(crate) fn begin_shutdown(&self) {
        self.with_scheduler(NodeLocalScheduler::begin_shutdown);
        self.notify.notify_waiters();
    }

    pub(crate) fn is_drained(&self) -> bool {
        self.with_scheduler(|scheduler| scheduler.is_drained())
    }

    pub(crate) async fn wait_for_change(&self) {
        self.notify.notified().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn assert_heap_bound(scheduler: &NodeLocalScheduler) {
        assert_eq!(
            scheduler.wakeups.len(),
            scheduler.wakeup_indices.len(),
            "scheduler should keep exactly one heap entry per live slot"
        );
        #[cfg(debug_assertions)]
        scheduler.assert_consistent();
    }

    #[test]
    fn set_wakeup_schedules_a_wakeup() {
        let mut scheduler = NodeLocalScheduler::new(2);
        let now = Instant::now();
        let when = now + Duration::from_secs(1);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(7), when),
            Ok(WakeupSetOutcome::Inserted { revision: 0 })
        );
        assert_heap_bound(&scheduler);
        assert_eq!(scheduler.next_expiry(), Some(when));
        assert_eq!(scheduler.pop_due(now), None);
        assert_eq!(scheduler.pop_due(when), Some((WakeupSlot(7), when, 0)));
        assert_heap_bound(&scheduler);
        assert_eq!(scheduler.next_expiry(), None);
    }

    #[test]
    fn setting_same_slot_replaces_previous_due_time() {
        let mut scheduler = NodeLocalScheduler::new(2);
        let now = Instant::now();
        let later = now + Duration::from_secs(10);
        let sooner = now + Duration::from_secs(1);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(3), later),
            Ok(WakeupSetOutcome::Inserted { revision: 0 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(3), sooner),
            Ok(WakeupSetOutcome::Replaced { revision: 1 })
        );
        assert_heap_bound(&scheduler);
        assert_eq!(scheduler.wakeups.len(), 1);
        assert_eq!(scheduler.next_expiry(), Some(sooner));
        assert_eq!(scheduler.pop_due(sooner), Some((WakeupSlot(3), sooner, 1)));
        assert_heap_bound(&scheduler);
        assert_eq!(scheduler.pop_due(later), None);
    }

    #[test]
    fn cancel_wakeup_removes_pending_wakeup() {
        let mut scheduler = NodeLocalScheduler::new(2);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(5), when),
            Ok(WakeupSetOutcome::Inserted { revision: 0 })
        );
        assert_heap_bound(&scheduler);
        assert!(scheduler.cancel_wakeup(WakeupSlot(5)));
        assert_heap_bound(&scheduler);
        assert!(!scheduler.cancel_wakeup(WakeupSlot(5)));
        assert_eq!(scheduler.next_expiry(), None);
        assert_eq!(scheduler.pop_due(when), None);
    }

    /// Scenario: a wakeup is rescheduled after heap reordering and then
    /// canceled while it is tracked at a moved, non-root heap index.
    /// Guarantees: cancellation removes the correct slot, preserves heap/index
    /// consistency, and leaves the remaining wakeups due in the expected order.
    #[test]
    fn cancel_after_reschedule_removes_the_moved_entry() {
        let mut scheduler = NodeLocalScheduler::new(4);
        let now = Instant::now();
        let first = now + Duration::from_secs(1);
        let second = now + Duration::from_secs(10);
        let third = now + Duration::from_secs(20);
        let fourth = now + Duration::from_secs(30);
        let moved = now + Duration::from_secs(2);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(1), first),
            Ok(WakeupSetOutcome::Inserted { revision: 0 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(2), second),
            Ok(WakeupSetOutcome::Inserted { revision: 1 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(3), third),
            Ok(WakeupSetOutcome::Inserted { revision: 2 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(4), fourth),
            Ok(WakeupSetOutcome::Inserted { revision: 3 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(3), moved),
            Ok(WakeupSetOutcome::Replaced { revision: 4 })
        );

        let moved_index = scheduler
            .wakeup_indices
            .get(&WakeupSlot(3))
            .copied()
            .expect("rescheduled slot should still be tracked");
        assert!(
            moved_index > 0,
            "rescheduled slot should be a non-root entry"
        );

        assert!(scheduler.cancel_wakeup(WakeupSlot(3)));
        assert_heap_bound(&scheduler);
        assert_eq!(scheduler.pop_due(first), Some((WakeupSlot(1), first, 0)));
        assert_eq!(scheduler.pop_due(moved), None);
        assert_eq!(scheduler.pop_due(second), Some((WakeupSlot(2), second, 1)));
        assert_eq!(scheduler.pop_due(fourth), Some((WakeupSlot(4), fourth, 3)));
        assert_eq!(scheduler.next_expiry(), None);
    }

    #[test]
    fn capacity_is_enforced_on_distinct_live_slots() {
        let mut scheduler = NodeLocalScheduler::new(1);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(0), when),
            Ok(WakeupSetOutcome::Inserted { revision: 0 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(1), when),
            Err(WakeupError::Capacity)
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(0), when + Duration::from_secs(1)),
            Ok(WakeupSetOutcome::Replaced { revision: 1 })
        );
        assert_heap_bound(&scheduler);
    }

    #[test]
    fn repeated_reschedules_keep_single_heap_entry() {
        let mut scheduler = NodeLocalScheduler::new(2);
        let now = Instant::now();
        for offset in (1..=32).rev() {
            let when = now + Duration::from_secs(offset);
            let outcome = scheduler
                .set_wakeup(WakeupSlot(9), when)
                .expect("wakeup should schedule");
            let expected_revision: WakeupRevision = 32 - offset;
            assert_eq!(outcome.revision(), expected_revision);
            assert_heap_bound(&scheduler);
            assert_eq!(scheduler.wakeups.len(), 1);
            assert_eq!(scheduler.next_expiry(), Some(when));
        }

        let expected = now + Duration::from_secs(1);
        assert_eq!(
            scheduler.pop_due(expected),
            Some((WakeupSlot(9), expected, 31))
        );
        assert_eq!(scheduler.next_expiry(), None);
    }

    #[test]
    fn equal_deadlines_follow_schedule_sequence() {
        let mut scheduler = NodeLocalScheduler::new(4);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(1), when),
            Ok(WakeupSetOutcome::Inserted { revision: 0 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(2), when),
            Ok(WakeupSetOutcome::Inserted { revision: 1 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(3), when),
            Ok(WakeupSetOutcome::Inserted { revision: 2 })
        );
        assert_heap_bound(&scheduler);

        assert_eq!(scheduler.pop_due(when), Some((WakeupSlot(1), when, 0)));
        assert_eq!(scheduler.pop_due(when), Some((WakeupSlot(2), when, 1)));
        assert_eq!(scheduler.pop_due(when), Some((WakeupSlot(3), when, 2)));
        assert_heap_bound(&scheduler);
    }
}
