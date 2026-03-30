// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node-local delayed resume and wakeup scheduling for processor inboxes.

use crate::clock;
use crate::control::{NodeControlMsg, WakeupSlot};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
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

#[derive(Debug)]
struct ScheduledResume<PData> {
    when: Instant,
    sequence: u64,
    data: Box<PData>,
}

impl<PData> Ord for ScheduledResume<PData> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .when
            .cmp(&self.when)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl<PData> PartialOrd for ScheduledResume<PData> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<PData> PartialEq for ScheduledResume<PData> {
    fn eq(&self, other: &Self) -> bool {
        self.when == other.when && self.sequence == other.sequence
    }
}

impl<PData> Eq for ScheduledResume<PData> {}

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

struct NodeLocalScheduler<PData> {
    delayed_resume_capacity: usize,
    wakeup_capacity: usize,
    next_sequence: u64,
    delayed_resumes: BinaryHeap<ScheduledResume<PData>>,
    wakeups: BinaryHeap<ScheduledWakeup>,
    wakeup_state: HashMap<WakeupSlot, WakeupState>,
    due_now: VecDeque<NodeControlMsg<PData>>,
    shutting_down: bool,
}

impl<PData> NodeLocalScheduler<PData> {
    fn new(delayed_resume_capacity: usize, wakeup_capacity: usize) -> Self {
        Self {
            delayed_resume_capacity,
            wakeup_capacity,
            next_sequence: 0,
            delayed_resumes: BinaryHeap::new(),
            wakeups: BinaryHeap::new(),
            wakeup_state: HashMap::new(),
            due_now: VecDeque::new(),
            shutting_down: false,
        }
    }

    fn next_sequence(&mut self) -> u64 {
        let next = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        next
    }

    fn requeue_later(&mut self, when: Instant, data: Box<PData>) -> Result<(), Box<PData>> {
        if self.shutting_down || self.delayed_resumes.len() >= self.delayed_resume_capacity {
            return Err(data);
        }

        let sequence = self.next_sequence();
        self.delayed_resumes.push(ScheduledResume {
            when,
            sequence,
            data,
        });
        Ok(())
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
        if !self.due_now.is_empty() {
            return Some(clock::now());
        }

        self.discard_stale_wakeup_head();
        match (
            self.delayed_resumes.peek().map(|resume| resume.when),
            self.wakeups.peek().map(|wakeup| wakeup.when),
        ) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    fn pop_due(&mut self, now: Instant) -> Option<NodeControlMsg<PData>> {
        if let Some(msg) = self.due_now.pop_front() {
            return Some(msg);
        }

        self.discard_stale_wakeup_head();

        let next_resume = self.delayed_resumes.peek().map(|resume| resume.when);
        let next_wakeup = self.wakeups.peek().map(|wakeup| wakeup.when);
        let take_resume = match (next_resume, next_wakeup) {
            (Some(resume_when), Some(wakeup_when)) => {
                resume_when <= now && (wakeup_when > now || resume_when <= wakeup_when)
            }
            (Some(resume_when), None) => resume_when <= now,
            (None, Some(_)) => false,
            (None, None) => return None,
        };

        if take_resume {
            let resume = self.delayed_resumes.pop().expect("resume must exist");
            return Some(NodeControlMsg::DelayedData {
                when: resume.when,
                data: resume.data,
            });
        }

        if next_wakeup
            .map(|wakeup_when| wakeup_when <= now)
            .unwrap_or(false)
        {
            let wakeup = self.wakeups.pop().expect("wakeup must exist");
            let _ = self.wakeup_state.remove(&wakeup.slot);
            return Some(NodeControlMsg::Wakeup {
                slot: wakeup.slot,
                when: wakeup.when,
            });
        }

        None
    }

    fn begin_shutdown(&mut self, now: Instant) {
        if self.shutting_down {
            return;
        }

        self.shutting_down = true;

        while let Some(resume) = self.delayed_resumes.pop() {
            self.due_now.push_back(NodeControlMsg::DelayedData {
                when: now,
                data: resume.data,
            });
        }

        self.wakeup_state.clear();
        self.wakeups.clear();
    }

    fn is_drained(&self) -> bool {
        self.due_now.is_empty() && self.delayed_resumes.is_empty() && self.wakeup_state.is_empty()
    }
}

/// Shared handle used by the processor inbox and the processor effect handler.
pub(crate) struct NodeLocalSchedulerHandle<PData> {
    inner: Arc<Mutex<NodeLocalScheduler<PData>>>,
    notify: Arc<Notify>,
}

impl<PData> Clone for NodeLocalSchedulerHandle<PData> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            notify: Arc::clone(&self.notify),
        }
    }
}

impl<PData> NodeLocalSchedulerHandle<PData> {
    pub(crate) fn new(delayed_resume_capacity: usize, wakeup_capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(NodeLocalScheduler::new(
                delayed_resume_capacity,
                wakeup_capacity,
            ))),
            notify: Arc::new(Notify::new()),
        }
    }

    fn with_scheduler<R>(&self, f: impl FnOnce(&mut NodeLocalScheduler<PData>) -> R) -> R {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut guard)
    }

    pub(crate) fn requeue_later(&self, when: Instant, data: Box<PData>) -> Result<(), Box<PData>> {
        let result = self.with_scheduler(|scheduler| scheduler.requeue_later(when, data));
        if result.is_ok() {
            self.notify.notify_one();
        }
        result
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

    pub(crate) fn pop_due(&self, now: Instant) -> Option<NodeControlMsg<PData>> {
        self.with_scheduler(|scheduler| scheduler.pop_due(now))
    }

    pub(crate) fn begin_shutdown(&self, now: Instant) {
        self.with_scheduler(|scheduler| scheduler.begin_shutdown(now));
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
    fn requeue_later_emits_the_stored_payload() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(scheduler.requeue_later(when, Box::new(17)), Ok(()));
        assert!(matches!(
            scheduler.pop_due(when),
            Some(NodeControlMsg::DelayedData { when: observed, data })
                if observed == when && *data == 17
        ));
        assert_eq!(scheduler.next_expiry(), None);
    }

    #[test]
    fn delayed_resumes_preserve_due_time_ordering() {
        let mut scheduler = NodeLocalScheduler::new(4, 2);
        let now = Instant::now();
        let later = now + Duration::from_secs(3);
        let sooner = now + Duration::from_secs(1);
        let same_time_a = now + Duration::from_secs(2);
        let same_time_b = same_time_a;

        assert_eq!(scheduler.requeue_later(later, Box::new(3)), Ok(()));
        assert_eq!(scheduler.requeue_later(same_time_a, Box::new(1)), Ok(()));
        assert_eq!(scheduler.requeue_later(same_time_b, Box::new(2)), Ok(()));
        assert_eq!(scheduler.requeue_later(sooner, Box::new(0)), Ok(()));

        assert!(matches!(
            scheduler.pop_due(sooner),
            Some(NodeControlMsg::DelayedData { data, .. }) if *data == 0
        ));
        assert!(matches!(
            scheduler.pop_due(same_time_a),
            Some(NodeControlMsg::DelayedData { data, .. }) if *data == 1
        ));
        assert!(matches!(
            scheduler.pop_due(same_time_b),
            Some(NodeControlMsg::DelayedData { data, .. }) if *data == 2
        ));
        assert!(matches!(
            scheduler.pop_due(later),
            Some(NodeControlMsg::DelayedData { data, .. }) if *data == 3
        ));
    }

    #[test]
    fn delayed_resume_capacity_is_enforced() {
        let mut scheduler = NodeLocalScheduler::new(1, 1);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(scheduler.requeue_later(when, Box::new(1)), Ok(()));
        let rejected = scheduler
            .requeue_later(when, Box::new(2))
            .expect_err("capacity should reject");
        assert_eq!(*rejected, 2);
    }

    #[test]
    fn rejected_requeue_returns_the_original_payload() {
        let mut scheduler = NodeLocalScheduler::new(2, 1);
        let now = Instant::now();

        scheduler.begin_shutdown(now);
        let rejected = scheduler
            .requeue_later(now + Duration::from_secs(1), Box::new(99))
            .expect_err("shutdown should reject");
        assert_eq!(*rejected, 99);
    }

    #[test]
    fn shutdown_makes_pending_delayed_resumes_due_immediately() {
        let mut scheduler = NodeLocalScheduler::new(4, 2);
        let now = Instant::now();
        let later = now + Duration::from_secs(30);

        assert_eq!(scheduler.requeue_later(later, Box::new(11)), Ok(()));
        assert_eq!(
            scheduler.requeue_later(later + Duration::from_secs(1), Box::new(12)),
            Ok(())
        );

        scheduler.begin_shutdown(now);

        assert!(matches!(
            scheduler.pop_due(now),
            Some(NodeControlMsg::DelayedData { when: observed, data })
                if observed == now && *data == 11
        ));
        assert!(matches!(
            scheduler.pop_due(now),
            Some(NodeControlMsg::DelayedData { when: observed, data })
                if observed == now && *data == 12
        ));
        assert!(scheduler.pop_due(now).is_none());
    }

    #[test]
    fn set_wakeup_schedules_a_wakeup() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
        let now = Instant::now();
        let when = now + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(7), when), Ok(()));
        assert_eq!(scheduler.next_expiry(), Some(when));
        assert!(scheduler.pop_due(now).is_none());
        assert!(matches!(
            scheduler.pop_due(when),
            Some(NodeControlMsg::Wakeup {
                slot: WakeupSlot(7),
                when: observed,
            }) if observed == when
        ));
        assert_eq!(scheduler.next_expiry(), None);
    }

    #[test]
    fn setting_same_slot_replaces_previous_due_time() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
        let now = Instant::now();
        let later = now + Duration::from_secs(10);
        let sooner = now + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(3), later), Ok(()));
        assert_eq!(scheduler.set_wakeup(WakeupSlot(3), sooner), Ok(()));
        assert_eq!(scheduler.next_expiry(), Some(sooner));
        assert!(matches!(
            scheduler.pop_due(sooner),
            Some(NodeControlMsg::Wakeup {
                slot: WakeupSlot(3),
                when: observed,
            }) if observed == sooner
        ));
        assert!(scheduler.pop_due(later).is_none());
    }

    #[test]
    fn cancel_wakeup_removes_pending_wakeup() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(5), when), Ok(()));
        assert!(scheduler.cancel_wakeup(WakeupSlot(5)));
        assert!(!scheduler.cancel_wakeup(WakeupSlot(5)));
        assert_eq!(scheduler.next_expiry(), None);
        assert!(scheduler.pop_due(when).is_none());
    }

    #[test]
    fn wakeup_capacity_is_enforced_on_distinct_live_slots() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(1, 1);
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
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
        let now = Instant::now();
        let first = now + Duration::from_secs(5);
        let replacement = now + Duration::from_secs(1);

        assert_eq!(scheduler.set_wakeup(WakeupSlot(9), first), Ok(()));
        assert_eq!(scheduler.set_wakeup(WakeupSlot(9), replacement), Ok(()));
        assert!(matches!(
            scheduler.pop_due(replacement),
            Some(NodeControlMsg::Wakeup {
                slot: WakeupSlot(9),
                when: observed,
            }) if observed == replacement
        ));
        assert!(scheduler.pop_due(first).is_none());
        assert_eq!(scheduler.next_expiry(), None);
    }
}
