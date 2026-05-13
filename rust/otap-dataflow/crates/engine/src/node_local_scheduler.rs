// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Processor-local scheduling for work that should re-enter a processor inbox.
//!
//! This module implements two related but intentionally distinct mechanisms:
//!
//! - Delayed resume stores a retained `Box<PData>` until a deadline and then
//!   surfaces that same payload as `NodeControlMsg::DelayedData`. Delayed
//!   resumes are append-only: there is no key, deduplication, or replacement.
//!   Equal deadlines are emitted FIFO by scheduler sequence. Rejection is
//!   payload-preserving so callers can turn failed scheduling into explicit
//!   failure handling.
//! - Wakeup scheduling stores only a lightweight keyed deadline. A
//!   `WakeupSlot` identifies the logical timer and later surfaces as
//!   `NodeControlMsg::Wakeup`; no `PData` is retained. Scheduling the same slot
//!   again replaces the previous deadline and assigns a new revision so callers
//!   can ignore stale wakeups.
//!
//! The processor inbox consumes both mechanisms as control traffic. That keeps
//! fairness, shutdown latching, and drain behavior in one place while avoiding
//! the older runtime-global delayed-data path for processor-local retry work.

use crate::clock;
use crate::control::{NodeControlMsg, WakeupRevision, WakeupSlot};
use crate::entity_context::current_node_telemetry_handle;
use crate::indexed_min_heap::IndexedMinHeap;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
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

/// Metrics for processor-local wakeup scheduler activity.
#[metric_set(name = "node.local_wakeup.scheduler")]
#[derive(Debug, Default, Clone)]
pub(crate) struct NodeLocalWakeupSchedulerMetrics {
    /// Count of newly inserted wakeups.
    #[metric(name = "set.inserted", unit = "{wakeup}")]
    set_inserted: Counter<u64>,
    /// Count of wakeups that replaced an existing live slot.
    #[metric(name = "set.replaced", unit = "{wakeup}")]
    set_replaced: Counter<u64>,
    /// Count of set attempts rejected because the live-slot capacity was full.
    #[metric(name = "set.error_capacity", unit = "{attempt}")]
    set_error_capacity: Counter<u64>,
    /// Count of set attempts rejected after shutdown was latched.
    #[metric(name = "set.error_shutdown", unit = "{attempt}")]
    set_error_shutdown: Counter<u64>,
    /// Count of cancel attempts that removed a live wakeup.
    #[metric(name = "cancel.removed", unit = "{wakeup}")]
    cancel_removed: Counter<u64>,
    /// Count of cancel attempts that found no live wakeup in the slot.
    #[metric(name = "cancel.missed", unit = "{attempt}")]
    cancel_missed: Counter<u64>,
    /// Count of cancel attempts ignored after shutdown was latched.
    #[metric(name = "cancel.ignored_shutdown", unit = "{attempt}")]
    cancel_ignored_shutdown: Counter<u64>,
    /// Count of wakeups popped for delivery to the processor.
    #[metric(name = "pop.due", unit = "{wakeup}")]
    pop_due: Counter<u64>,
    /// Count of live wakeups dropped when shutdown was latched.
    #[metric(name = "shutdown.cleared", unit = "{wakeup}")]
    shutdown_cleared: Counter<u64>,
    /// Current number of live wakeups.
    #[metric(name = "live", unit = "{wakeup}")]
    live: Gauge<u64>,
    /// Configured live wakeup slot capacity.
    #[metric(name = "capacity", unit = "{wakeup}")]
    capacity: Gauge<u64>,
}

/// Retained payload scheduled for future delivery back to the same processor.
///
/// `BinaryHeap` is a max-heap, so the ordering implementation below reverses
/// comparisons to make the earliest deadline appear at the heap head. The
/// scheduler-owned `sequence` field breaks equal-deadline ties FIFO.
#[derive(Debug)]
struct ScheduledResume<PData> {
    /// Deadline at which the retained payload becomes due.
    when: Instant,
    /// Monotonic scheduler sequence used as a FIFO tie-breaker.
    sequence: u64,
    /// Original retained pdata returned to the processor when due.
    data: Box<PData>,
}

impl<PData> Ord for ScheduledResume<PData> {
    // Reverse ordering so BinaryHeap behaves like a min-heap by due time.
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .when
            .cmp(&self.when)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl<PData> PartialOrd for ScheduledResume<PData> {
    // Delegate partial ordering to the total ordering used by BinaryHeap.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<PData> PartialEq for ScheduledResume<PData> {
    // Payload identity is not part of scheduling identity.
    fn eq(&self, other: &Self) -> bool {
        self.when == other.when && self.sequence == other.sequence
    }
}

impl<PData> Eq for ScheduledResume<PData> {}

/// Priority key for the wakeup heap. Ordered by wall-clock time first,
/// then by revision to break ties deterministically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WakeupPriority {
    when: Instant,
    revision: WakeupRevision,
}

impl Ord for WakeupPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.when
            .cmp(&other.when)
            .then_with(|| self.revision.cmp(&other.revision))
    }
}

impl PartialOrd for WakeupPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Mutable state behind a processor's local delayed-resume and wakeup handle.
///
/// The scheduler is shared between the processor `EffectHandler` and inbox via
/// [`NodeLocalSchedulerHandle`]. All mutation happens under the handle mutex,
/// and the inbox polls due work through `next_expiry` and `pop_due`.
struct NodeLocalScheduler<PData> {
    /// Maximum number of retained delayed resumes accepted at once.
    delayed_resume_capacity: usize,
    /// Maximum number of live wakeup slots accepted at once.
    wakeup_capacity: usize,
    /// Next FIFO sequence number for delayed resumes.
    next_sequence: u64,
    /// Next revision assigned to accepted wakeup schedules.
    next_revision: WakeupRevision,
    /// Retained payloads ordered by earliest due time and FIFO sequence.
    delayed_resumes: BinaryHeap<ScheduledResume<PData>>,
    /// Immediate control messages produced by shutdown drain conversion.
    due_now: VecDeque<NodeControlMsg<PData>>,
    /// Keyed wakeup deadlines with replace/remove support by slot.
    heap: IndexedMinHeap<WakeupSlot, WakeupPriority>,
    /// Whether shutdown has latched and new local scheduling must be rejected.
    shutting_down: bool,
    /// Optional per-node telemetry for wakeup activity.
    metrics: Option<MetricSet<NodeLocalWakeupSchedulerMetrics>>,
}

impl<PData> NodeLocalScheduler<PData> {
    /// Creates a scheduler without metrics for tests.
    #[cfg(test)]
    fn new(delayed_resume_capacity: usize, wakeup_capacity: usize) -> Self {
        Self::new_with_metrics(delayed_resume_capacity, wakeup_capacity, None)
    }

    /// Creates a scheduler with explicit delayed-resume/wakeup capacities and
    /// optional wakeup metrics.
    fn new_with_metrics(
        delayed_resume_capacity: usize,
        wakeup_capacity: usize,
        metrics: Option<MetricSet<NodeLocalWakeupSchedulerMetrics>>,
    ) -> Self {
        Self {
            delayed_resume_capacity,
            wakeup_capacity,
            next_sequence: 0,
            next_revision: 0,
            delayed_resumes: BinaryHeap::new(),
            due_now: VecDeque::new(),
            heap: IndexedMinHeap::new(),
            shutting_down: false,
            metrics,
        }
    }

    /// Allocates the next delayed-resume FIFO sequence number.
    fn next_sequence(&mut self) -> u64 {
        let next = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        next
    }

    /// Allocates the next wakeup revision number.
    fn next_revision(&mut self) -> WakeupRevision {
        let next = self.next_revision;
        self.next_revision = self.next_revision.saturating_add(1);
        next
    }

    /// Refreshes gauges derived from current wakeup state.
    fn refresh_gauges(&mut self) {
        if let Some(metrics) = &mut self.metrics {
            metrics.live.set(self.heap.len() as u64);
            metrics.capacity.set(self.wakeup_capacity as u64);
        }
    }

    /// Stores a retained payload for later delivery to this processor.
    ///
    /// Returns the original payload on capacity pressure or after shutdown is
    /// latched so callers never lose ownership on rejection.
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

    /// Inserts or replaces a lightweight keyed wakeup.
    ///
    /// Unlike delayed resumes, wakeups retain no payload and replacement of an
    /// existing slot does not consume additional capacity.
    fn set_wakeup(
        &mut self,
        slot: WakeupSlot,
        when: Instant,
    ) -> Result<WakeupSetOutcome, WakeupError> {
        if self.wakeup_capacity == 0 {
            return Err(WakeupError::Unsupported);
        }

        if self.shutting_down {
            if let Some(metrics) = &mut self.metrics {
                metrics.set_error_shutdown.inc();
            }
            return Err(WakeupError::ShuttingDown);
        }

        if self.heap.contains_key(&slot) {
            let revision = self.next_revision();
            let priority = WakeupPriority { when, revision };
            let _ = self.heap.insert(slot, priority);
            if let Some(metrics) = &mut self.metrics {
                metrics.set_replaced.inc();
            }
            self.refresh_gauges();
            Ok(WakeupSetOutcome::Replaced { revision })
        } else {
            if self.heap.len() >= self.wakeup_capacity {
                if let Some(metrics) = &mut self.metrics {
                    metrics.set_error_capacity.inc();
                }
                return Err(WakeupError::Capacity);
            }
            let revision = self.next_revision();
            let priority = WakeupPriority { when, revision };
            let _ = self.heap.insert(slot, priority);
            if let Some(metrics) = &mut self.metrics {
                metrics.set_inserted.inc();
            }
            self.refresh_gauges();
            Ok(WakeupSetOutcome::Inserted { revision })
        }
    }

    /// Cancels one live wakeup slot if shutdown has not already latched.
    fn cancel_wakeup(&mut self, slot: WakeupSlot) -> bool {
        if self.shutting_down {
            if let Some(metrics) = &mut self.metrics {
                metrics.cancel_ignored_shutdown.inc();
            }
            return false;
        }
        let removed = self.heap.remove(&slot).is_some();
        if let Some(metrics) = &mut self.metrics {
            if removed {
                metrics.cancel_removed.inc();
            } else {
                metrics.cancel_missed.inc();
            }
        }
        self.refresh_gauges();
        removed
    }

    /// Returns the earliest instant at which local work may become due.
    ///
    /// `due_now` is represented as ready immediately so the inbox can drain
    /// shutdown-converted delayed resumes without sleeping.
    fn next_expiry(&mut self) -> Option<Instant> {
        if !self.due_now.is_empty() {
            return Some(clock::now());
        }

        #[cfg(debug_assertions)]
        self.heap.assert_consistent();

        match (
            self.delayed_resumes.peek().map(|resume| resume.when),
            self.heap.peek().map(|(_, priority)| priority.when),
        ) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    /// Pops one local control message due at or before `now`.
    ///
    /// Delayed resumes and wakeups share one due stream. When both mechanisms
    /// are due, earlier deadlines win; equal deadlines prefer delayed resume
    /// because it represents retained payload drain.
    fn pop_due(&mut self, now: Instant) -> Option<NodeControlMsg<PData>> {
        if let Some(msg) = self.due_now.pop_front() {
            return Some(msg);
        }

        #[cfg(debug_assertions)]
        self.heap.assert_consistent();

        let next_resume = self.delayed_resumes.peek().map(|resume| resume.when);
        let next_wakeup = self.heap.peek().map(|(_, priority)| priority.when);
        let take_resume = match (next_resume, next_wakeup) {
            (Some(resume_when), Some(wakeup_when)) => {
                resume_when <= now && (wakeup_when > now || resume_when <= wakeup_when)
            }
            (Some(resume_when), None) => resume_when <= now,
            (None, Some(_)) => false,
            (None, None) => return None,
        };

        if take_resume {
            let resume = self
                .delayed_resumes
                .pop()
                // Safety: take_resume is true only after peeking a due resume,
                // and no delayed-resume mutation happens between peek and pop.
                .expect("resume must exist");
            return Some(NodeControlMsg::DelayedData {
                when: resume.when,
                data: resume.data,
            });
        }

        let next_due = self.heap.peek().map(|(_, priority)| priority.when)?;
        if next_due > now {
            return None;
        }

        let (slot, priority) = self
            .heap
            .pop()
            // Safety: next_due was read from the heap head above, and no wakeup
            // heap mutation happens between peek and pop.
            .expect("due wakeup should exist");
        if let Some(metrics) = &mut self.metrics {
            metrics.pop_due.inc();
        }
        self.refresh_gauges();
        Some(NodeControlMsg::Wakeup {
            slot,
            when: priority.when,
            revision: priority.revision,
        })
    }

    /// Pop the next scheduled wakeup regardless of whether it is due.
    ///
    /// This is the unconditional wakeup-only counterpart of [`pop_due`](Self::pop_due) and
    /// exists for test/benchmark harnesses where the inbox loop is not running.
    #[cfg(any(test, feature = "test-utils"))]
    fn pop_next(&mut self) -> Option<(WakeupSlot, Instant, WakeupRevision)> {
        #[cfg(debug_assertions)]
        self.heap.assert_consistent();

        let (slot, priority) = self.heap.pop()?;
        if let Some(metrics) = &mut self.metrics {
            metrics.pop_due.inc();
        }
        self.refresh_gauges();
        Some((slot, priority.when, priority.revision))
    }

    /// Latches shutdown and converts pending delayed resumes into immediate
    /// `DelayedData` controls while dropping pending wakeups.
    ///
    /// This preserves payload drain expectations for retry-like flows. Wakeups
    /// are intentionally not retained because they carry no payload.
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

        let cleared = self.heap.len();
        if let Some(metrics) = &mut self.metrics {
            metrics.shutdown_cleared.add(cleared as u64);
        }
        self.heap.clear();
        self.refresh_gauges();
    }

    /// Returns whether all retained local work has drained.
    fn is_drained(&self) -> bool {
        self.due_now.is_empty() && self.delayed_resumes.is_empty() && self.heap.is_empty()
    }

    /// Reports current wakeup scheduler metrics through the provided reporter.
    fn report_metrics(
        &mut self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        self.refresh_gauges();
        if let Some(metrics) = &mut self.metrics {
            metrics_reporter.report(metrics)?;
        }
        Ok(())
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
        let metrics = current_node_telemetry_handle()
            .map(|telemetry| telemetry.register_metric_set::<NodeLocalWakeupSchedulerMetrics>());

        Self {
            inner: Arc::new(Mutex::new(NodeLocalScheduler::new_with_metrics(
                delayed_resume_capacity,
                wakeup_capacity,
                metrics,
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

    pub(crate) fn pop_due(&self, now: Instant) -> Option<NodeControlMsg<PData>> {
        self.with_scheduler(|scheduler| scheduler.pop_due(now))
    }

    /// Pop the next scheduled wakeup regardless of whether it is due.
    ///
    /// This is the unconditional wakeup-only counterpart of [`pop_due`](Self::pop_due) and
    /// exists for test/benchmark harnesses where the inbox loop is not running.
    #[cfg(any(test, feature = "test-utils"))]
    pub(crate) fn pop_next(&self) -> Option<(WakeupSlot, Instant, WakeupRevision)> {
        self.with_scheduler(|scheduler| scheduler.pop_next())
    }

    pub(crate) fn begin_shutdown(&self, now: Instant) {
        self.with_scheduler(|scheduler| scheduler.begin_shutdown(now));
        self.notify.notify_waiters();
    }

    pub(crate) fn is_drained(&self) -> bool {
        self.with_scheduler(|scheduler| scheduler.is_drained())
    }

    pub(crate) fn report_metrics(
        &self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        self.with_scheduler(|scheduler| scheduler.report_metrics(metrics_reporter))
    }

    pub(crate) async fn wait_for_change(&self) {
        self.notify.notified().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn assert_heap_bound<PData>(scheduler: &NodeLocalScheduler<PData>) {
        #[cfg(debug_assertions)]
        scheduler.heap.assert_consistent();
    }

    fn expect_delayed(
        msg: Option<NodeControlMsg<i32>>,
        expected_when: Instant,
        expected_data: i32,
    ) {
        match msg {
            Some(NodeControlMsg::DelayedData { when, data }) => {
                assert_eq!(when, expected_when);
                assert_eq!(*data, expected_data);
            }
            _ => panic!("expected delayed data"),
        }
    }

    fn expect_wakeup(
        msg: Option<NodeControlMsg<i32>>,
        expected_slot: WakeupSlot,
        expected_when: Instant,
        expected_revision: WakeupRevision,
    ) {
        match msg {
            Some(NodeControlMsg::Wakeup {
                slot,
                when,
                revision,
            }) => {
                assert_eq!(slot, expected_slot);
                assert_eq!(when, expected_when);
                assert_eq!(revision, expected_revision);
            }
            _ => panic!("expected wakeup"),
        }
    }

    /// Scenario: a retained payload is scheduled through the processor-local
    /// delayed-resume queue and then becomes due.
    /// Guarantees: the scheduler emits the original payload as
    /// `NodeControlMsg::DelayedData` and clears the delayed-resume deadline.
    #[test]
    fn requeue_later_emits_the_stored_payload() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(scheduler.requeue_later(when, Box::new(17)), Ok(()));
        expect_delayed(scheduler.pop_due(when), when, 17);
        assert_eq!(scheduler.next_expiry(), None);
    }

    /// Scenario: multiple delayed resumes are scheduled at different times,
    /// including two with the same due time.
    /// Guarantees: due resumes are emitted by due time, with FIFO ordering for
    /// equal deadlines.
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

        expect_delayed(scheduler.pop_due(sooner), sooner, 0);
        expect_delayed(scheduler.pop_due(same_time_a), same_time_a, 1);
        expect_delayed(scheduler.pop_due(same_time_b), same_time_b, 2);
        expect_delayed(scheduler.pop_due(later), later, 3);
    }

    /// Scenario: the delayed-resume heap has reached its configured capacity.
    /// Guarantees: additional requeue attempts are rejected instead of
    /// exceeding the bound.
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

    /// Scenario: the scheduler has already latched shutdown when a caller tries
    /// to schedule another delayed resume.
    /// Guarantees: rejection preserves ownership by returning the original
    /// retained payload to the caller.
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

    /// Scenario: shutdown begins while future delayed resumes are still pending
    /// in the processor-local scheduler.
    /// Guarantees: pending delayed resumes are converted into immediate
    /// `DelayedData` delivery using the shutdown-start timestamp.
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

        expect_delayed(scheduler.pop_due(now), now, 11);
        expect_delayed(scheduler.pop_due(now), now, 12);
        assert!(scheduler.pop_due(now).is_none());
    }

    /// Scenario: a wakeup slot is scheduled once and then reaches its deadline.
    /// Guarantees: the scheduler reports the deadline, emits one `Wakeup`
    /// control with the accepted revision, and clears the slot afterward.
    #[test]
    fn set_wakeup_schedules_a_wakeup() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
        let now = Instant::now();
        let when = now + Duration::from_secs(1);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(7), when),
            Ok(WakeupSetOutcome::Inserted { revision: 0 })
        );
        assert_heap_bound(&scheduler);
        assert_eq!(scheduler.next_expiry(), Some(when));
        assert!(scheduler.pop_due(now).is_none());
        expect_wakeup(scheduler.pop_due(when), WakeupSlot(7), when, 0);
        assert_heap_bound(&scheduler);
        assert_eq!(scheduler.next_expiry(), None);
    }

    /// Scenario: the same wakeup slot is scheduled twice with different due
    /// times before either deadline is popped.
    /// Guarantees: the second schedule replaces the first in place and only
    /// the latest deadline/revision is emitted.
    #[test]
    fn setting_same_slot_replaces_previous_due_time() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
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
        assert_eq!(scheduler.heap.len(), 1);
        assert_eq!(scheduler.next_expiry(), Some(sooner));
        expect_wakeup(scheduler.pop_due(sooner), WakeupSlot(3), sooner, 1);
        assert_heap_bound(&scheduler);
        assert!(scheduler.pop_due(later).is_none());
    }

    /// Scenario: a live wakeup slot is canceled before its deadline.
    /// Guarantees: cancellation removes the wakeup, a second cancellation
    /// reports a miss, and no wakeup is emitted at the old deadline.
    #[test]
    fn cancel_wakeup_removes_pending_wakeup() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
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
        assert!(scheduler.pop_due(when).is_none());
    }

    /// Scenario: wakeup insertion, replacement, capacity rejection,
    /// cancellation, and due delivery all occur on a metrics-enabled scheduler.
    /// Guarantees: lifecycle counters and gauges reflect each accepted,
    /// rejected, canceled, missed, and popped wakeup operation.
    #[test]
    fn scheduler_metrics_count_wakeup_lifecycle() {
        let registry = otap_df_telemetry::registry::TelemetryRegistryHandle::default();
        let metrics: MetricSet<NodeLocalWakeupSchedulerMetrics> =
            registry.register_metric_set(otap_df_telemetry::testing::EmptyAttributes());
        let mut scheduler = NodeLocalScheduler::<i32>::new_with_metrics(1, 1, Some(metrics));
        let now = Instant::now();
        let later = now + Duration::from_secs(1);
        let sooner = now + Duration::from_millis(10);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(0), later),
            Ok(WakeupSetOutcome::Inserted { revision: 0 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(0), sooner),
            Ok(WakeupSetOutcome::Replaced { revision: 1 })
        );
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(1), later),
            Err(WakeupError::Capacity)
        );
        assert!(!scheduler.cancel_wakeup(WakeupSlot(9)));
        assert!(scheduler.cancel_wakeup(WakeupSlot(0)));
        assert!(!scheduler.cancel_wakeup(WakeupSlot(0)));
        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(0), later),
            Ok(WakeupSetOutcome::Inserted { revision: 2 })
        );
        assert!(scheduler.pop_due(now).is_none());
        expect_wakeup(scheduler.pop_due(later), WakeupSlot(0), later, 2);

        let metrics = scheduler.metrics.as_ref().expect("metrics enabled");
        assert_eq!(metrics.set_inserted.get(), 2);
        assert_eq!(metrics.set_replaced.get(), 1);
        assert_eq!(metrics.set_error_capacity.get(), 1);
        assert_eq!(metrics.cancel_removed.get(), 1);
        assert_eq!(metrics.cancel_missed.get(), 2);
        assert_eq!(metrics.pop_due.get(), 1);
        assert_eq!(metrics.live.get(), 0);
        assert_eq!(metrics.capacity.get(), 1);
    }

    /// Scenario: a wakeup is rescheduled after heap reordering and then
    /// canceled while it is tracked at a moved, non-root heap index.
    /// Guarantees: cancellation removes the correct slot, preserves heap/index
    /// consistency, and leaves the remaining wakeups due in the expected order.
    #[test]
    fn cancel_after_reschedule_removes_the_moved_entry() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(4, 4);
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

        assert!(scheduler.heap.contains_key(&WakeupSlot(3)));
        assert_eq!(scheduler.heap.peek().map(|(k, _)| *k), Some(WakeupSlot(1)));

        assert!(scheduler.cancel_wakeup(WakeupSlot(3)));
        assert_heap_bound(&scheduler);
        expect_wakeup(scheduler.pop_due(first), WakeupSlot(1), first, 0);
        assert!(scheduler.pop_due(moved).is_none());
        expect_wakeup(scheduler.pop_due(second), WakeupSlot(2), second, 1);
        expect_wakeup(scheduler.pop_due(fourth), WakeupSlot(4), fourth, 3);
        assert_eq!(scheduler.next_expiry(), None);
    }

    /// Scenario: distinct live wakeup slots fill the configured wakeup
    /// capacity while an existing slot is later rescheduled.
    /// Guarantees: new distinct slots are rejected at capacity, but replacing
    /// an existing slot remains accepted.
    #[test]
    fn wakeup_capacity_is_enforced_on_distinct_live_slots() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(1, 1);
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

    /// Scenario: a scheduler is configured with zero wakeup capacity.
    /// Guarantees: wakeup scheduling reports `WakeupError::Unsupported` rather
    /// than treating the zero bound as a full queue.
    #[test]
    fn wakeup_is_unsupported_when_capacity_is_zero() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(1, 0);
        let when = Instant::now() + Duration::from_secs(1);

        assert_eq!(
            scheduler.set_wakeup(WakeupSlot(0), when),
            Err(WakeupError::Unsupported)
        );
    }

    /// Scenario: one wakeup slot is repeatedly rescheduled with earlier
    /// deadlines.
    /// Guarantees: replacement keeps a single live heap entry, advances the
    /// revision each time, and emits only the final schedule.
    #[test]
    fn repeated_reschedules_keep_single_heap_entry() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(2, 2);
        let now = Instant::now();
        for offset in (1..=32).rev() {
            let when = now + Duration::from_secs(offset);
            let outcome = scheduler
                .set_wakeup(WakeupSlot(9), when)
                .expect("wakeup should schedule");
            let expected_revision: WakeupRevision = 32 - offset;
            assert_eq!(outcome.revision(), expected_revision);
            assert_heap_bound(&scheduler);
            assert_eq!(scheduler.heap.len(), 1);
            assert_eq!(scheduler.next_expiry(), Some(when));
        }

        let expected = now + Duration::from_secs(1);
        expect_wakeup(scheduler.pop_due(expected), WakeupSlot(9), expected, 31);
        assert_eq!(scheduler.next_expiry(), None);
    }

    /// Scenario: multiple wakeup slots share the same deadline.
    /// Guarantees: equal-deadline wakeups are emitted in scheduler revision
    /// order and heap/index consistency is preserved.
    #[test]
    fn equal_deadlines_follow_schedule_sequence() {
        let mut scheduler = NodeLocalScheduler::<i32>::new(4, 4);
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

        expect_wakeup(scheduler.pop_due(when), WakeupSlot(1), when, 0);
        expect_wakeup(scheduler.pop_due(when), WakeupSlot(2), when, 1);
        expect_wakeup(scheduler.pop_due(when), WakeupSlot(3), when, 2);
        assert_heap_bound(&scheduler);
    }
}
