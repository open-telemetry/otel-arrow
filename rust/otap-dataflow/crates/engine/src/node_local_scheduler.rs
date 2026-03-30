// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node-local wakeup scheduling for processor inboxes.

use crate::control::WakeupSlot;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::Notify;

/// Error returned when a wakeup request cannot be accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeupError {
    /// The processor has already latched shutdown.
    ShuttingDown,
    /// The bounded live wakeup slot set is full.
    Capacity,
}

#[derive(Clone, Copy, Debug)]
struct WakeupState {
    when: Instant,
    generation: u64,
    sequence: u64,
}

#[derive(Debug)]
struct ScheduledWakeup {
    slot: WakeupSlot,
    when: Instant,
    generation: u64,
    sequence: u64,
}

impl Ord for ScheduledWakeup {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .when
            .cmp(&self.when)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl PartialOrd for ScheduledWakeup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledWakeup {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot
            && self.when == other.when
            && self.generation == other.generation
            && self.sequence == other.sequence
    }
}

impl Eq for ScheduledWakeup {}

struct NodeLocalScheduler {
    wakeup_capacity: usize,
    next_sequence: u64,
    wakeups: BinaryHeap<ScheduledWakeup>,
    wakeup_state: HashMap<WakeupSlot, WakeupState>,
    shutting_down: bool,
}

impl NodeLocalScheduler {
    fn new(wakeup_capacity: usize) -> Self {
        Self {
            wakeup_capacity,
            next_sequence: 0,
            wakeups: BinaryHeap::new(),
            wakeup_state: HashMap::new(),
            shutting_down: false,
        }
    }

    fn next_sequence(&mut self) -> u64 {
        let next = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        next
    }

    fn set_wakeup(&mut self, slot: WakeupSlot, when: Instant) -> Result<(), WakeupError> {
        if self.shutting_down {
            return Err(WakeupError::ShuttingDown);
        }

        let sequence = self.next_sequence();
        let generation = if let Some(state) = self.wakeup_state.get_mut(&slot) {
            state.when = when;
            state.generation = state.generation.saturating_add(1);
            state.sequence = sequence;
            state.generation
        } else {
            if self.wakeup_state.len() >= self.wakeup_capacity {
                return Err(WakeupError::Capacity);
            }
            let _ = self.wakeup_state.insert(
                slot,
                WakeupState {
                    when,
                    generation: 0,
                    sequence,
                },
            );
            0
        };

        self.wakeups.push(ScheduledWakeup {
            slot,
            when,
            generation,
            sequence,
        });
        Ok(())
    }

    fn cancel_wakeup(&mut self, slot: WakeupSlot) -> bool {
        if self.shutting_down {
            return false;
        }
        self.wakeup_state.remove(&slot).is_some()
    }

    fn discard_stale_wakeup_head(&mut self) {
        while let Some(head) = self.wakeups.peek() {
            let Some(state) = self.wakeup_state.get(&head.slot) else {
                let _ = self.wakeups.pop();
                continue;
            };
            if state.generation != head.generation || state.when != head.when {
                let _ = self.wakeups.pop();
                continue;
            }
            break;
        }
    }

    fn next_expiry(&mut self) -> Option<Instant> {
        self.discard_stale_wakeup_head();
        self.wakeups.peek().map(|wakeup| wakeup.when)
    }

    fn pop_due(&mut self, now: Instant) -> Option<(WakeupSlot, Instant)> {
        self.discard_stale_wakeup_head();

        let next_due = self.wakeups.peek().map(|wakeup| wakeup.when)?;
        if next_due > now {
            return None;
        }

        let wakeup = self.wakeups.pop().expect("wakeup must exist");
        let _ = self.wakeup_state.remove(&wakeup.slot);
        Some((wakeup.slot, wakeup.when))
    }

    fn begin_shutdown(&mut self) {
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;
        self.wakeup_state.clear();
        self.wakeups.clear();
    }

    fn is_drained(&self) -> bool {
        self.wakeup_state.is_empty()
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

    pub(crate) fn set_wakeup(&self, slot: WakeupSlot, when: Instant) -> Result<(), WakeupError> {
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

    pub(crate) fn pop_due(&self, now: Instant) -> Option<(WakeupSlot, Instant)> {
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

    #[test]
    fn set_wakeup_schedules_a_wakeup() {
        let mut scheduler = NodeLocalScheduler::new(2);
        let now = Instant::now();
        let when = now + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(7), when), Ok(()));
        assert_eq!(scheduler.next_expiry(), Some(when));
        assert_eq!(scheduler.pop_due(now), None);
        assert_eq!(scheduler.pop_due(when), Some((WakeupSlot(7), when)));
        assert_eq!(scheduler.next_expiry(), None);
    }

    #[test]
    fn setting_same_slot_replaces_previous_due_time() {
        let mut scheduler = NodeLocalScheduler::new(2);
        let now = Instant::now();
        let later = now + Duration::from_secs(10);
        let sooner = now + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(3), later), Ok(()));
        assert_eq!(scheduler.set_wakeup(WakeupSlot(3), sooner), Ok(()));
        assert_eq!(scheduler.next_expiry(), Some(sooner));
        assert_eq!(scheduler.pop_due(sooner), Some((WakeupSlot(3), sooner)));
        assert_eq!(scheduler.pop_due(later), None);
    }

    #[test]
    fn cancel_wakeup_removes_pending_wakeup() {
        let mut scheduler = NodeLocalScheduler::new(2);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(5), when), Ok(()));
        assert!(scheduler.cancel_wakeup(WakeupSlot(5)));
        assert!(!scheduler.cancel_wakeup(WakeupSlot(5)));
        assert_eq!(scheduler.next_expiry(), None);
        assert_eq!(scheduler.pop_due(when), None);
    }

    #[test]
    fn capacity_is_enforced_on_distinct_live_slots() {
        let mut scheduler = NodeLocalScheduler::new(1);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(0), when), Ok(()));
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(1), when),
            Err(WakeupError::Capacity)
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(0), when + Duration::from_secs(1)),
            Ok(())
        );
    }

    #[test]
    fn stale_heap_entries_are_ignored() {
        let mut scheduler = NodeLocalScheduler::new(2);
        let now = Instant::now();
        let first = now + Duration::from_secs(5);
        let replacement = now + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(9), first), Ok(()));
        assert_eq!(scheduler.set_wakeup(WakeupSlot(9), replacement), Ok(()));
        assert_eq!(
            scheduler.pop_due(replacement),
            Some((WakeupSlot(9), replacement))
        );
        assert_eq!(scheduler.pop_due(first), None);
        assert_eq!(scheduler.next_expiry(), None);
    }
}
