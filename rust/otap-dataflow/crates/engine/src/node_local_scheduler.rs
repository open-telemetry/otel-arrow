// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node-local wakeup scheduling for processor inboxes.

use crate::control::{WakeupRevision, WakeupSlot};
use crate::entity_context::current_node_telemetry_handle;
use crate::indexed_min_heap::IndexedMinHeap;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
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

/// Priority key for the wakeup heap.  Ordered by wall-clock time first,
/// then by revision to break ties deterministically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WakeupPriority {
    when: Instant,
    revision: WakeupRevision,
}

impl Ord for WakeupPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.when
            .cmp(&other.when)
            .then_with(|| self.revision.cmp(&other.revision))
    }
}

impl PartialOrd for WakeupPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct NodeLocalScheduler {
    wakeup_capacity: usize,
    next_revision: WakeupRevision,
    heap: IndexedMinHeap<WakeupSlot, WakeupPriority>,
    shutting_down: bool,
    metrics: Option<MetricSet<NodeLocalWakeupSchedulerMetrics>>,
}

impl NodeLocalScheduler {
    #[cfg(test)]
    fn new(wakeup_capacity: usize) -> Self {
        Self::new_with_metrics(wakeup_capacity, None)
    }

    fn new_with_metrics(
        wakeup_capacity: usize,
        metrics: Option<MetricSet<NodeLocalWakeupSchedulerMetrics>>,
    ) -> Self {
        Self {
            wakeup_capacity,
            next_revision: 0,
            heap: IndexedMinHeap::new(),
            shutting_down: false,
            metrics,
        }
    }

    fn next_revision(&mut self) -> WakeupRevision {
        let next = self.next_revision;
        self.next_revision = self.next_revision.saturating_add(1);
        next
    }

    fn refresh_gauges(&mut self) {
        if let Some(metrics) = &mut self.metrics {
            metrics.live.set(self.heap.len() as u64);
            metrics.capacity.set(self.wakeup_capacity as u64);
        }
    }

    fn set_wakeup(
        &mut self,
        slot: WakeupSlot,
        when: Instant,
    ) -> Result<WakeupSetOutcome, WakeupError> {
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

    fn next_expiry(&mut self) -> Option<Instant> {
        #[cfg(debug_assertions)]
        self.heap.assert_consistent();
        self.heap.peek().map(|(_, priority)| priority.when)
    }

    fn pop_due(&mut self, now: Instant) -> Option<(WakeupSlot, Instant, WakeupRevision)> {
        #[cfg(debug_assertions)]
        self.heap.assert_consistent();

        let next_due = self.heap.peek().map(|(_, p)| p.when)?;
        if next_due > now {
            return None;
        }

        let (slot, priority) = self.heap.pop().expect("due wakeup should exist");
        if let Some(metrics) = &mut self.metrics {
            metrics.pop_due.inc();
        }
        self.refresh_gauges();
        Some((slot, priority.when, priority.revision))
    }

    /// Pop the next scheduled wakeup regardless of whether it is due.
    ///
    /// This is the unconditional counterpart of [`pop_due`](Self::pop_due) and
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

    fn begin_shutdown(&mut self) {
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;
        let cleared = self.heap.len();
        if let Some(metrics) = &mut self.metrics {
            metrics.shutdown_cleared.add(cleared as u64);
        }
        self.heap.clear();
        self.refresh_gauges();
    }

    fn is_drained(&self) -> bool {
        self.heap.is_empty()
    }

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
        let metrics = current_node_telemetry_handle()
            .map(|telemetry| telemetry.register_metric_set::<NodeLocalWakeupSchedulerMetrics>());

        Self {
            inner: Arc::new(Mutex::new(NodeLocalScheduler::new_with_metrics(
                wakeup_capacity,
                metrics,
            ))),
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

    /// Pop the next scheduled wakeup regardless of whether it is due.
    ///
    /// This is the unconditional counterpart of [`pop_due`](Self::pop_due) and
    /// exists for test/benchmark harnesses where the inbox loop is not running.
    #[cfg(any(test, feature = "test-utils"))]
    pub(crate) fn pop_next(&self) -> Option<(WakeupSlot, Instant, WakeupRevision)> {
        self.with_scheduler(|scheduler| scheduler.pop_next())
    }

    pub(crate) fn begin_shutdown(&self) {
        self.with_scheduler(NodeLocalScheduler::begin_shutdown);
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

    fn assert_heap_bound(scheduler: &NodeLocalScheduler) {
        #[cfg(debug_assertions)]
        scheduler.heap.assert_consistent();
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
        assert_eq!(scheduler.heap.len(), 1);
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

    #[test]
    fn scheduler_metrics_count_wakeup_lifecycle() {
        let registry = otap_df_telemetry::registry::TelemetryRegistryHandle::default();
        let metrics: MetricSet<NodeLocalWakeupSchedulerMetrics> =
            registry.register_metric_set(otap_df_telemetry::testing::EmptyAttributes());
        let mut scheduler = NodeLocalScheduler::new_with_metrics(1, Some(metrics));
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
        assert_eq!(scheduler.pop_due(now), None);
        assert_eq!(scheduler.pop_due(later), Some((WakeupSlot(0), later, 2)));

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

        assert!(scheduler.heap.contains_key(&WakeupSlot(3)));
        // Verify that slot 1 (earliest deadline) should still be at the root.
        assert_eq!(scheduler.heap.peek().map(|(k, _)| *k), Some(WakeupSlot(1)));

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
            assert_eq!(scheduler.heap.len(), 1);
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
