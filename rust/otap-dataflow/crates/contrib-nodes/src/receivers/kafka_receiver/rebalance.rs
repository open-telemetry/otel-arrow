// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-group rebalance handling for the Kafka receiver.
//!
//! The receiver tracks pending offsets in memory via the
//! [`OffsetTracker`](super::offset_tracker::OffsetTracker), which lives on the
//! single-threaded `LocalSet` runtime and is owned by the receive loop. Kafka
//! consumer-group rebalances, however, are delivered by librdkafka on its own
//! poll thread via the [`ConsumerContext`] callbacks. That thread cannot touch
//! the `LocalSet`-owned tracker directly.
//!
//! This module bridges the two worlds with a small amount of shared,
//! synchronized state ([`RebalanceState`]):
//!
//! - **`assigned`** -- the set of topic-partitions currently owned by this
//!   consumer. Updated by the rebalance callbacks and read by the receive loop
//!   to scope commits to owned partitions only.
//! - **`committable`** -- a snapshot of the offset that would be committed for
//!   each tracked partition, refreshed by the receive loop after each
//!   ack/commit cycle. Read by [`pre_rebalance`](ConsumerContext::pre_rebalance)
//!   to commit owned partitions *before* they are revoked (commit-before-revoke).
//! - **`revoked`** -- a queue of partitions revoked since the loop last
//!   reconciled. Drained by the receive loop, which then purges the tracker.
//!
//! Rebalance handling is a no-op when auto-commit is enabled, since librdkafka
//! manages offsets itself in that mode.
//!
//! # In-flight messages
//!
//! Unlike the Go (franz-go) collector receiver, this implementation does **not**
//! interrupt or drain in-flight messages for a revoked partition. Any such
//! message simply will not have its offset committed by this consumer; the new
//! owner re-delivers it. This is safe under at-least-once semantics and mirrors
//! the rotel Kafka receiver's behavior.
//!
//! When such an in-flight message is finally acknowledged, its Ack/Nack carries
//! the ownership **generation** it was tracked under, and
//! `classify_offset_feedback` drops it as stale if that generation is older than
//! the partition's current tracked *or* currently-assigned generation. Both are
//! consulted: after a revoke/reassign the tracker may still report the old
//! generation until a record of the new period is tracked, so comparing against
//! the assigned generation is what rejects a stale ack during that window --
//! before it can advance the tracker or roll back the committed offset.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "aws")]
use crate::common::kafka::aws::AwsMskAuthClientContext;
use rdkafka::ClientContext;
use rdkafka::client::OAuthToken;
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};

/// The set of topic-partitions currently assigned to this consumer, each tagged
/// with a per-partition **ownership generation**.
///
/// Keyed by topic name, then by partition -> generation. Used to scope offset commits
/// to partitions this consumer actually owns, so that a late ack/nack or a
/// periodic timer tick never commits an offset for a partition that has been
/// reassigned to another consumer.
///
/// The generation changes only when *this* partition's ownership is (re)acquired -- it
/// is stable while the partition stays continuously owned across unrelated
/// rebalances. This is intentionally decoupled from any global rebalance counter:
/// two records tracked during one continuous ownership must carry the same generation,
/// otherwise a legitimate late ACK could be mistaken for feedback from a previous
/// ownership period.
#[derive(Debug, Default)]
pub(crate) struct AssignedPartitions {
    topics: HashMap<String, HashMap<i32, u64>>,
}

/// A partition revoked during a rebalance, tagged with the assignment
/// generation that was current when the revocation occurred.
///
/// The generation lets the receive loop tell whether tracker state for the
/// partition belongs to the revoked ownership period (and should be purged) or
/// to a newer one created after the partition was reassigned to this consumer
/// (which must be preserved).
#[derive(Debug, Clone)]
pub(crate) struct RevokedPartition {
    /// Topic of the revoked partition.
    pub(crate) topic: String,
    /// Partition number.
    pub(crate) partition: i32,
    /// Assignment generation at the time of revocation.
    pub(crate) generation: u64,
}

impl AssignedPartitions {
    /// Create an empty assignment set.
    pub(crate) fn new() -> Self {
        Self {
            topics: HashMap::new(),
        }
    }

    /// Record `(topic, partition)` as assigned under `generation`.
    ///
    /// If the partition is already owned, its generation is left unchanged (the
    /// partition was retained across the rebalance). Only a fresh acquisition
    /// assigns a new generation.
    pub(crate) fn add_partition(&mut self, topic: &str, partition: i32, generation: u64) {
        let _ = self
            .topics
            .entry(topic.to_string())
            .or_default()
            .entry(partition)
            .or_insert(generation);
    }

    /// Remove `(topic, partition)` from the assignment, dropping the topic
    /// entry once it has no remaining partitions. Returns `true` if the
    /// partition was actually owned (and thus removed).
    pub(crate) fn remove_partition(&mut self, topic: &str, partition: i32) -> bool {
        let mut removed = false;
        if let Some(partitions) = self.topics.get_mut(topic) {
            removed = partitions.remove(&partition).is_some();
            if partitions.is_empty() {
                let _ = self.topics.remove(topic);
            }
        }
        removed
    }

    /// Returns `true` if `(topic, partition)` is currently assigned.
    #[must_use]
    pub(crate) fn contains(&self, topic: &str, partition: i32) -> bool {
        self.topics
            .get(topic)
            .is_some_and(|partitions| partitions.contains_key(&partition))
    }

    /// The ownership generation for `(topic, partition)`, or `None` if not owned.
    #[must_use]
    pub(crate) fn generation(&self, topic: &str, partition: i32) -> Option<u64> {
        self.topics
            .get(topic)
            .and_then(|partitions| partitions.get(&partition).copied())
    }

    /// Total number of partitions currently owned across all topics.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.topics.values().map(HashMap::len).sum()
    }
}

/// Shared state bridging the librdkafka rebalance callbacks and the
/// `LocalSet`-owned receive loop.
///
/// All fields are behind their own lock so the (rare) callback path and the
/// (hot) receive-loop reconcile path contend as little as possible. The
/// receive loop only ever takes these locks briefly to read/refresh snapshots;
/// the hot per-message offset tracking stays lock-free in the tracker itself.
#[derive(Debug)]
pub(crate) struct RebalanceState {
    /// Partitions currently owned by this consumer.
    assigned: Mutex<AssignedPartitions>,
    /// Partitions revoked since the receive loop last reconciled, each tagged
    /// with the generation at revoke time. Drained by the loop, which then
    /// purges the tracker (only for state not newer than the revoke generation).
    revoked: Mutex<Vec<RevokedPartition>>,
    /// Monotonic allocator for per-partition ownership generations. Advanced in
    /// [`set_assignment`](Self::set_assignment) for each partition that is
    /// *newly* acquired (not-owned -> owned), so a partition reacquired after a
    /// revocation gets a strictly greater generation than the revocation queued for
    /// its prior ownership period. Retained partitions keep their generation.
    generation_allocator: AtomicU64,
    /// Latest committable offset per `(topic, partition)`, refreshed by the
    /// receive loop. Read by `pre_rebalance` to commit before revocation.
    committable: Mutex<HashMap<(String, i32), i64>>,
    /// When `true`, rebalance handling is skipped (librdkafka owns offsets).
    auto_commit: bool,
    /// Count of consumer-group rebalance (assign) events observed
    /// (callback-incremented once per `post_rebalance(Assign)`).
    rebalances_total: AtomicU64,
    /// Count of partitions **newly acquired** by this consumer across rebalances
    /// (callback-incremented; retained partitions are not re-counted).
    partition_assignments: AtomicU64,
    /// Count of **genuinely-owned** partitions revoked from this consumer across
    /// rebalances (callback-incremented; a revoke reported for a partition this
    /// consumer did not own is not counted).
    partition_revocations: AtomicU64,
    /// Count of commit failures during pre-rebalance revoke (callback-incremented).
    rebalance_commit_errors: AtomicU64,
    /// Count of offset commits acknowledged by the broker, observed on the
    /// commit callback (callback-incremented). Covers the receiver's async
    /// steady-state commits and the sync pre-rebalance commit.
    offset_commits: AtomicU64,
    /// Count of offset commits rejected by the broker, observed on the commit
    /// callback (callback-incremented).
    offset_commit_errors: AtomicU64,
}

/// A batch of rebalance counter deltas drained by the receive loop and folded
/// into the receiver's [`MetricSet`](otap_df_telemetry::metrics::MetricSet).
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct RebalanceMetricsDelta {
    /// Rebalance (assign) events since the last drain.
    pub(crate) rebalances_total: u64,
    /// Partitions newly acquired since the last drain.
    pub(crate) partition_assignments: u64,
    /// Genuinely-owned partitions revoked since the last drain.
    pub(crate) partition_revocations: u64,
    /// Current number of partitions owned by this consumer at drain time.
    ///
    /// Unlike the other fields (which are counter deltas), this is an absolute
    /// snapshot used to drive `receiver.kafka.consumer.group.partitions`.
    pub(crate) partitions_owned: u64,
    /// Commit failures during revoke since the last drain.
    pub(crate) rebalance_commit_errors: u64,
    /// Broker-acknowledged offset commits since the last drain.
    pub(crate) offset_commits: u64,
    /// Broker-rejected offset commits since the last drain.
    pub(crate) offset_commit_errors: u64,
}

impl RebalanceMetricsDelta {
    /// Returns `true` if there is nothing to report.
    ///
    /// `partitions_owned` is a gauge snapshot, not a counter delta, so it is
    /// deliberately excluded here: the receive loop folds the gauge only when a
    /// rebalance actually changed the assignment (i.e. when one of the counter
    /// deltas is non-zero), avoiding a redundant gauge write on every idle tick.
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.rebalances_total == 0
            && self.partition_assignments == 0
            && self.partition_revocations == 0
            && self.rebalance_commit_errors == 0
            && self.offset_commits == 0
            && self.offset_commit_errors == 0
    }
}

impl RebalanceState {
    /// Create new rebalance state.
    ///
    /// When `auto_commit` is `true`, the rebalance callbacks short-circuit and
    /// the receive loop's reconcile steps become no-ops.
    #[must_use]
    pub(crate) fn new(auto_commit: bool) -> Self {
        Self {
            assigned: Mutex::new(AssignedPartitions::new()),
            revoked: Mutex::new(Vec::new()),
            committable: Mutex::new(HashMap::new()),
            auto_commit,
            rebalances_total: AtomicU64::new(0),
            partition_assignments: AtomicU64::new(0),
            partition_revocations: AtomicU64::new(0),
            rebalance_commit_errors: AtomicU64::new(0),
            offset_commits: AtomicU64::new(0),
            offset_commit_errors: AtomicU64::new(0),
            generation_allocator: AtomicU64::new(0),
        }
    }

    /// The current ownership generation to stamp onto a record being tracked
    /// for `(topic, partition)`.
    ///
    /// Read by the receive loop when tracking a record so the tracked state and
    /// its Ack/Nack calldata carry the ownership period they belong to. Stable
    /// while the partition stays continuously owned; changes only when the
    /// partition is reacquired after a revocation.
    ///
    /// Returns `0` when the partition is not currently owned. Real generations
    /// start at `1` (see [`next_generation`](Self::next_generation)), so `0` is a
    /// safe sentinel; in practice the caller only tracks owned partitions, so
    /// this fallback is not reached.
    #[must_use]
    pub(crate) fn current_generation(&self, topic: &str, partition: i32) -> u64 {
        self.lock_assigned()
            .generation(topic, partition)
            .unwrap_or(0)
    }

    /// Returns `true` if auto-commit is enabled (rebalance handling disabled).
    #[must_use]
    pub(crate) fn is_auto_commit(&self) -> bool {
        self.auto_commit
    }

    /// Returns `true` if `(topic, partition)` is currently assigned.
    ///
    /// Used by the receive loop's late-ack guard: an ack/nack for a partition
    /// that is no longer assigned must not trigger a commit.
    #[must_use]
    pub(crate) fn is_assigned(&self, topic: &str, partition: i32) -> bool {
        self.lock_assigned().contains(topic, partition)
    }

    /// Replace the committable snapshot with `snapshot`.
    ///
    /// Called by the receive loop after each ack/commit cycle so that
    /// `pre_rebalance` always commits reasonably fresh offsets.
    pub(crate) fn set_committable_snapshot(&self, snapshot: HashMap<(String, i32), i64>) {
        match self.committable.lock() {
            Ok(mut guard) => *guard = snapshot,
            Err(poisoned) => *poisoned.into_inner() = snapshot,
        }
    }

    /// Drain the revoked-partition queue, returning the partitions the receive
    /// loop should purge from the tracker. Returns an empty `Vec` if nothing
    /// has been revoked since the last call.
    pub(crate) fn drain_revoked(&self) -> Vec<RevokedPartition> {
        let mut guard = lock_ignore_poison(&self.revoked);
        if guard.is_empty() {
            return Vec::new();
        }
        std::mem::take(&mut *guard)
    }

    /// Drain accumulated rebalance metric counters into a delta.
    ///
    /// The counter fields are swapped to zero; `partitions_owned` is a
    /// point-in-time read of the current assignment size (the gauge source).
    pub(crate) fn drain_metrics(&self) -> RebalanceMetricsDelta {
        RebalanceMetricsDelta {
            rebalances_total: self.rebalances_total.swap(0, Ordering::Relaxed),
            partition_assignments: self.partition_assignments.swap(0, Ordering::Relaxed),
            partition_revocations: self.partition_revocations.swap(0, Ordering::Relaxed),
            partitions_owned: self.lock_assigned().len() as u64,
            rebalance_commit_errors: self.rebalance_commit_errors.swap(0, Ordering::Relaxed),
            offset_commits: self.offset_commits.swap(0, Ordering::Relaxed),
            offset_commit_errors: self.offset_commit_errors.swap(0, Ordering::Relaxed),
        }
    }

    /// Record the outcome of an offset commit reported by librdkafka on the
    /// commit callback. Called on the poll thread for both the receiver's async
    /// commits and the synchronous pre-rebalance commit.
    fn record_commit_result(&self, result: &rdkafka::error::KafkaResult<()>) {
        match result {
            Ok(()) => {
                let _ = self.offset_commits.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                let _ = self.offset_commit_errors.fetch_add(1, Ordering::Relaxed);
                otel_error!(
                    "kafka.commit.async_failed",
                    error = %e,
                );
            }
        }
    }

    fn lock_assigned(&self) -> std::sync::MutexGuard<'_, AssignedPartitions> {
        lock_ignore_poison(&self.assigned)
    }

    /// Test-only: enqueue a revoked partition (tagged with `generation`) as if a
    /// rebalance callback had fired, so the receive-loop reconcile path can be
    /// exercised without a live broker.
    #[cfg(test)]
    pub(crate) fn push_revoked_for_test(&self, topic: &str, partition: i32, generation: u64) {
        lock_ignore_poison(&self.revoked).push(RevokedPartition {
            topic: topic.to_string(),
            partition,
            generation,
        });
    }

    /// Test-only: mark a partition as assigned (at `generation`) without going
    /// through a rebalance callback.
    #[cfg(test)]
    pub(crate) fn assign_for_test(&self, topic: &str, partition: i32, generation: u64) {
        self.lock_assigned()
            .add_partition(topic, partition, generation);
    }

    /// Test-only: apply a full assignment (allocating/retaining generations) as if a
    /// `post_rebalance(Assign)` had delivered `full`.
    #[cfg(test)]
    pub(crate) fn set_assignment_for_test(&self, full: &TopicPartitionList) {
        self.set_assignment(full);
    }

    /// Test-only: record a commit outcome as if the commit callback had fired,
    /// so the receive-loop metric-folding path can be exercised without a live
    /// broker.
    #[cfg(test)]
    pub(crate) fn record_commit_result_for_test(&self, ok: bool) {
        let result = if ok {
            Ok(())
        } else {
            Err(rdkafka::error::KafkaError::ClientCreation(
                "test".to_string(),
            ))
        };
        self.record_commit_result(&result);
    }

    /// Test-only: read the committable offset snapshot for a partition.
    #[cfg(test)]
    pub(crate) fn committable_for_test(&self, topic: &str, partition: i32) -> Option<i64> {
        lock_ignore_poison(&self.committable)
            .get(&(topic.to_string(), partition))
            .copied()
    }

    /// Handle a `pre_rebalance` revoke: commit the committable offsets for the
    /// revoked partitions, queue them for tracker purge, and drop them from the
    /// assigned set.
    fn handle_revoke<C: ConsumerContext>(
        &self,
        consumer: &BaseConsumer<C>,
        tpl: &TopicPartitionList,
    ) {
        let revoked = topic_partitions(tpl);
        if revoked.is_empty() {
            return;
        }

        // Build a commit list from the latest committable snapshot, scoped to
        // the partitions being revoked.
        let commit_tpl = {
            let committable = lock_ignore_poison(&self.committable);
            build_commit_tpl(&committable, &revoked)
        };

        if commit_tpl.count() > 0 {
            if let Err(e) = consumer.commit(&commit_tpl, CommitMode::Sync) {
                let _ = self.rebalance_commit_errors.fetch_add(1, Ordering::Relaxed);
                otel_error!(
                    "kafka.rebalance.commit_failed",
                    error = %e,
                );
            }
        }

        // Look up each partition's ownership generation and drop it from the assigned
        // set. Queue every revoked partition (tagged with its own generation) for the
        // receive loop to purge from the tracker; the generation lets the purge skip
        // state that was re-tracked under a newer ownership period. Count only
        // partitions that were genuinely owned for the revoked metric, mirroring
        // the newly-added count in `set_assignment`.
        let mut revoked_tagged = Vec::with_capacity(revoked.len());
        let mut revoked_list = PartitionListFmt::default();
        let owned_after;
        {
            let mut assigned = self.lock_assigned();
            for (topic, partition) in &revoked {
                // Generation of the ownership period being revoked (default 0 if the
                // partition wasn't actually owned).
                let generation = assigned.generation(topic, *partition).unwrap_or(0);
                if assigned.remove_partition(topic, *partition) {
                    revoked_list.append(topic, *partition);
                }
                revoked_tagged.push(RevokedPartition {
                    topic: topic.clone(),
                    partition: *partition,
                    generation,
                });
            }
            owned_after = assigned.len();
        }
        let owned_revoked = revoked_list.count();
        {
            let mut revoked_queue = lock_ignore_poison(&self.revoked);
            revoked_queue.extend(revoked_tagged);
        }

        let _ = self
            .partition_revocations
            .fetch_add(owned_revoked, Ordering::Relaxed);

        // Structured observability: list the genuinely-owned revoked partition
        // IDs.
        if owned_revoked > 0 {
            otel_info!(
                "kafka.rebalance.partitions_revoked",
                partitions = %revoked_list.as_str(),
                count = owned_revoked,
                listed_count = revoked_list.listed_count(),
                truncated = revoked_list.truncated(),
            );
            if owned_after == 0 {
                otel_info!("kafka.assignment.became_empty");
            }
        }
    }

    /// Replace the assigned set with the *complete* assignment `full`.
    ///
    /// `full` must be the consumer's entire current assignment, not a rebalance
    /// delta. `receiver.kafka.consumer.group.partition.assignments` is incremented
    /// only by the number of partitions that are newly present (not previously
    /// owned), so cooperative-sticky rebalances that retain partitions don't
    /// re-count them.
    ///
    /// All partitions **newly acquired** in this rebalance share a single fresh
    /// ownership generation (the allocator is bumped at most once per call, and
    /// only when at least one partition is newly acquired); **retained**
    /// partitions keep their existing generation so records tracked during one
    /// continuous ownership all share one generation. A partition reacquired
    /// after a revocation therefore gets a strictly greater generation than the
    /// revocation queued for its prior ownership period.
    ///
    /// Emits the `kafka.rebalance.partitions_assigned` /
    /// `kafka.assignment.became_non_empty` observability events for the
    /// newly-acquired partitions before returning.
    fn set_assignment(&self, full: &TopicPartitionList) {
        let elements = full.elements();
        let mut assigned = self.lock_assigned();
        let owned_before = assigned.len();

        // A single generation shared by every partition acquired in this
        // rebalance, allocated lazily on the first newly-acquired partition.
        let mut rebalance_generation: Option<u64> = None;
        // Rebuild the owned set: carry over the generation for retained partitions,
        // allocate a fresh generation for newly acquired ones, and build the
        // observability log line inline from the still-borrowed topic names.
        //
        // Ordering dependency: librdkafka runs `pre_rebalance(Revoke)` (which
        // removes revoked partitions from the assigned set) *before*
        // `post_rebalance(Assign)` reaches here, so a partition that was revoked
        // and reassigned to this consumer is absent from the set at this point
        // and correctly receives a fresh, strictly-greater generation.
        let mut next = AssignedPartitions::new();
        let mut acquired_list = PartitionListFmt::default();
        for elem in &elements {
            let topic = elem.topic();
            let partition = elem.partition();
            match assigned.generation(topic, partition) {
                Some(existing) => next.add_partition(topic, partition, existing),
                None => {
                    acquired_list.append(topic, partition);
                    let generation =
                        *rebalance_generation.get_or_insert_with(|| self.next_generation());
                    next.add_partition(topic, partition, generation);
                }
            }
        }
        *assigned = next;
        // Drop the assignment lock before logging.
        drop(assigned);

        let newly_acquired = acquired_list.count();
        let _ = self
            .partition_assignments
            .fetch_add(newly_acquired, Ordering::Relaxed);

        if newly_acquired > 0 {
            otel_info!(
                "kafka.rebalance.partitions_assigned",
                partitions = %acquired_list.as_str(),
                count = newly_acquired,
                listed_count = acquired_list.listed_count(),
                truncated = acquired_list.truncated(),
            );
            if owned_before == 0 {
                otel_info!("kafka.assignment.became_non_empty");
            }
        }
    }

    /// Merge a rebalance **delta** into the current assignment without removing
    /// anything.
    ///
    /// Used as the fallback when [`Consumer::assignment`] cannot be queried in
    /// [`handle_assign`](Self::handle_assign). Unlike
    /// [`set_assignment`](Self::set_assignment) (which treats its argument as the
    /// complete owned set and replaces), this only *adds* partitions reported in
    /// `delta` that are not already owned -- so it can never drop a partition this
    /// consumer still owns, even if `delta` is an incremental cooperative-sticky
    /// delta rather than the full set. All partitions newly added in this call
    /// share a single fresh ownership generation (allocated lazily, at most once
    /// per call); already-owned partitions keep theirs.
    ///
    /// Emits the same observability events as
    /// [`set_assignment`](Self::set_assignment) on the fallback path
    fn merge_assignment(&self, delta: &TopicPartitionList) {
        let elements = delta.elements();
        let mut assigned = self.lock_assigned();
        let owned_before = assigned.len();
        let mut rebalance_generation: Option<u64> = None;
        // Bounded log buffer for the `topic-partition` list, built inline (see
        // `set_assignment`).
        let mut acquired_list = PartitionListFmt::default();
        for elem in &elements {
            let topic = elem.topic();
            let partition = elem.partition();
            if !assigned.contains(topic, partition) {
                acquired_list.append(topic, partition);
                let generation =
                    *rebalance_generation.get_or_insert_with(|| self.next_generation());
                assigned.add_partition(topic, partition, generation);
            }
        }
        // Drop the assignment lock before logging.
        drop(assigned);

        let newly_acquired = acquired_list.count();
        let _ = self
            .partition_assignments
            .fetch_add(newly_acquired, Ordering::Relaxed);

        if newly_acquired > 0 {
            otel_info!(
                "kafka.rebalance.partitions_assigned",
                partitions = %acquired_list.as_str(),
                count = newly_acquired,
                listed_count = acquired_list.listed_count(),
                truncated = acquired_list.truncated(),
            );
            // See `set_assignment`: an assignment-size transition to non-empty,
            // not a consumer-group join.
            if owned_before == 0 {
                otel_info!("kafka.assignment.became_non_empty");
            }
        }
    }

    /// Allocate the next ownership generation.
    ///
    /// Real generations start at `1` (the allocator starts at `0` and this
    /// returns `previous + 1`), so `0` is reserved as a safe "unowned/absent"
    /// sentinel that can never collide with a real generation -- which keeps the
    /// generation comparisons in the offset tracker unambiguous.
    fn next_generation(&self) -> u64 {
        self.generation_allocator.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Handle a `post_rebalance` assign by storing the consumer's *complete*
    /// current assignment.
    ///
    /// The `tpl` reported to `post_rebalance` is only the rebalance **delta**
    /// under the cooperative-sticky protocol (librdkafka calls
    /// `rd_kafka_incremental_assign` with just the added partitions). Clearing
    /// and storing only that delta would drop partitions the consumer still
    /// owns, causing later ACK/NACK feedback for those partitions to be rejected
    /// as revoked. Since librdkafka applies the assignment before invoking
    /// `post_rebalance`, querying [`Consumer::assignment`] returns the full,
    /// current set for both the cooperative and eager protocols.
    ///
    /// If the query fails (rare), fall back to **merging** the reported delta
    /// into the existing assignment ([`merge_assignment`](Self::merge_assignment))
    /// rather than replacing. Replacing with a cooperative-sticky delta would
    /// drop retained partitions (whose ACKs would then be rejected as revoked);
    /// merging only adds and never drops, so retained partitions are preserved
    /// until the next successful rebalance reconciles the full set.
    fn handle_assign<C: ConsumerContext>(
        &self,
        base_consumer: &BaseConsumer<C>,
        tpl: &TopicPartitionList,
    ) {
        // Count the rebalance (assign) event regardless of how the assignment is
        // resolved below. The assignment handlers emit the per-partition
        // observability events themselves (while their borrowed topic names are
        // still live), so no partition data is threaded back here.
        let _ = self.rebalances_total.fetch_add(1, Ordering::Relaxed);

        match base_consumer.assignment() {
            Ok(full) => self.set_assignment(&full),
            Err(e) => {
                otel_warn!(
                    "kafka.rebalance.assignment_query_failed",
                    error = %e,
                );
                self.merge_assignment(tpl);
            }
        }
    }
}

/// Lock a mutex, recovering the inner guard even if it was poisoned.
///
/// The data protected by these locks is plain bookkeeping state; a poisoned
/// lock (from a panic elsewhere) should not bring down the receiver, so we
/// recover rather than propagate the panic.
fn lock_ignore_poison<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Maximum number of `topic-partition` listed in a single rebalance
/// observability event.
const MAX_LISTED_PARTITIONS: u64 = 64;

#[derive(Debug, Default)]
struct PartitionListFmt {
    buf: String,
    count: u64,
    listed: u64,
    truncated: bool,
}

impl PartitionListFmt {
    fn append(&mut self, topic: &str, partition: i32) {
        self.count += 1;

        // Once truncated the trailing "..." is already present; keep counting only.
        if self.truncated {
            return;
        }

        // Entry cap: append a single trailing "..." and stop updating buf.
        if self.listed >= MAX_LISTED_PARTITIONS {
            self.truncated = true;
            if !self.buf.is_empty() {
                self.buf.push(',');
            }
            self.buf.push_str("...");
            return;
        }

        use std::fmt::Write as _;
        if !self.buf.is_empty() {
            self.buf.push(',');
        }
        let _ = write!(self.buf, "{topic}:{partition}");
        self.listed += 1;
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn listed_count(&self) -> u64 {
        self.listed
    }

    fn truncated(&self) -> bool {
        self.truncated
    }

    fn as_str(&self) -> &str {
        &self.buf
    }
}

/// Collect `(topic, partition)` pairs from a [`TopicPartitionList`].
fn topic_partitions(tpl: &TopicPartitionList) -> Vec<(String, i32)> {
    tpl.elements()
        .iter()
        .map(|e| (e.topic().to_string(), e.partition()))
        .collect()
}

/// Build a [`TopicPartitionList`] containing the committable offset for each
/// partition in `revoked` that has a known committable offset.
fn build_commit_tpl(
    committable: &HashMap<(String, i32), i64>,
    revoked: &[(String, i32)],
) -> TopicPartitionList {
    let mut tpl = TopicPartitionList::new();
    for (topic, partition) in revoked {
        if let Some(&offset) = committable.get(&(topic.clone(), *partition)) {
            let _ = tpl.add_partition_offset(topic, *partition, Offset::Offset(offset));
        }
    }
    tpl
}

/// A [`ConsumerContext`] that records partition assignments and commits offsets
/// before partitions are revoked.
///
/// Wraps either the default (no-auth) context or the AWS MSK IAM auth context
/// so that both authentication modes get rebalance handling. The variant is
/// chosen at consumer-creation time based on the receiver's auth config.
pub(crate) enum RebalancingConsumerContext {
    /// No special authentication.
    Default(std::sync::Arc<RebalanceState>),
    /// AWS MSK IAM OAUTHBEARER authentication, delegating token refresh to the
    /// wrapped context.
    #[cfg(feature = "aws")]
    AwsMsk {
        /// Inner context providing the OAUTHBEARER token.
        inner: AwsMskAuthClientContext,
        /// Shared rebalance state.
        state: std::sync::Arc<RebalanceState>,
    },
}

impl RebalancingConsumerContext {
    fn state(&self) -> &RebalanceState {
        match self {
            RebalancingConsumerContext::Default(state) => state,
            #[cfg(feature = "aws")]
            RebalancingConsumerContext::AwsMsk { state, .. } => state,
        }
    }
}

impl ClientContext for RebalancingConsumerContext {
    // Mirror `AwsMskAuthClientContext`: the AWS variant needs periodic
    // OAUTHBEARER token refresh.
    //
    // This constant is inert unless SASL OAUTHBEARER is configured: librdkafka
    // only emits the token-refresh event (and thus only calls
    // `generate_oauth_token`) for the OAUTHBEARER mechanism. For the default,
    // non-AWS variant (plaintext/SSL/SCRAM) the event never fires, so leaving
    // it `true` is harmless; the `Err(..)` returned below is a defensive
    // fallback that should be unreachable in practice.
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = true;

    fn generate_oauth_token(
        &self,
        oauthbearer_config: Option<&str>,
    ) -> Result<OAuthToken, Box<dyn std::error::Error>> {
        // `oauthbearer_config` is only consumed by the AWS MSK variant, which
        // is compiled out when the `aws` feature is disabled.
        let _ = oauthbearer_config;
        match self {
            RebalancingConsumerContext::Default(_) => {
                Err("OAUTH token refresh is not configured for this consumer".into())
            }
            #[cfg(feature = "aws")]
            RebalancingConsumerContext::AwsMsk { inner, .. } => {
                inner.generate_oauth_token(oauthbearer_config)
            }
        }
    }
}

impl ConsumerContext for RebalancingConsumerContext {
    fn pre_rebalance(&self, base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        let state = self.state();
        if state.is_auto_commit() {
            return;
        }
        match rebalance {
            Rebalance::Revoke(tpl) => {
                state.handle_revoke(base_consumer, tpl);
            }
            Rebalance::Assign(_) => {
                // Assignment is recorded in post_rebalance once it is in effect.
            }
            Rebalance::Error(err) => {
                otel_warn!(
                    "kafka.rebalance.error",
                    error = %err,
                );
            }
        }
    }

    fn post_rebalance(&self, base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        let state = self.state();
        if state.is_auto_commit() {
            return;
        }
        match rebalance {
            Rebalance::Assign(tpl) => {
                state.handle_assign(base_consumer, tpl);
            }
            Rebalance::Revoke(_) => {
                // Revocation bookkeeping already happened in pre_rebalance.
            }
            Rebalance::Error(err) => {
                otel_warn!(
                    "kafka.rebalance.error",
                    error = %err,
                );
            }
        }
    }

    fn commit_callback(
        &self,
        result: rdkafka::error::KafkaResult<()>,
        _offsets: &TopicPartitionList,
    ) {
        let state = self.state();
        // In auto-commit mode librdkafka manages offsets itself; keep manual-mode
        // commit metrics clean by ignoring those callbacks.
        if state.is_auto_commit() {
            return;
        }
        // Single source of truth for commit success/failure metrics: the
        // receiver's steady-state commits are asynchronous, so the broker
        // outcome is only known here (not at commit-issue time).
        state.record_commit_result(&result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::kafka::test::cluster::KafkaTestCluster;
    use crate::common::kafka::test::with_cluster;
    use rdkafka::ClientConfig;
    use std::sync::Arc;

    // ---- Shared test helpers ----

    /// Builds a real `BaseConsumer` wired to the receiver's rebalance context
    /// against the mock `cluster`, so the private `handle_assign` / `handle_revoke`
    /// methods can be driven for real while assertions stay on the deterministic
    /// in-memory rebalance state (not the async broker outcome).
    fn mock_base_consumer(
        cluster: &KafkaTestCluster,
        group: &str,
        state: Arc<RebalanceState>,
    ) -> BaseConsumer<RebalancingConsumerContext> {
        let ctx = RebalancingConsumerContext::Default(state);
        ClientConfig::new()
            .set("bootstrap.servers", cluster.bootstrap_servers())
            .set("group.id", group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create_with_context(ctx)
            .expect("failed to create mock base consumer")
    }

    // ---- Construction and configuration ----

    /// Scenario (construction and configuration): a `RebalanceState` is built with
    /// auto-commit on and off.
    /// Guarantees: `is_auto_commit()` reflects the constructor argument, so the rebalance
    /// callbacks know whether to short-circuit.
    #[test]
    fn state_auto_commit_flag() {
        assert!(RebalanceState::new(true).is_auto_commit());
        assert!(!RebalanceState::new(false).is_auto_commit());
    }

    /// Scenario (construction and configuration): a `RebalanceState` is shared via `Arc`
    /// across the callback thread and the receive loop.
    /// Guarantees: the auto-commit flag is visible through every clone, so all holders
    /// agree on whether manual commit handling is active.
    #[test]
    fn auto_commit_state_shared_across_arc() {
        let state = Arc::new(RebalanceState::new(true));
        let clone = Arc::clone(&state);
        assert!(clone.is_auto_commit());
    }

    // ---- Consumer-group rebalancing ----

    /// Scenario (consumer-group rebalancing): partitions are added to, removed from, and
    /// queried on an `AssignedPartitions` set.
    /// Guarantees: add/remove/contains behave consistently and removing a topic's last
    /// partition drops the topic entry, so the owned-set bookkeeping stays accurate.
    #[test]
    fn assigned_partitions_add_remove_contains() {
        let mut ap = AssignedPartitions::new();
        assert!(!ap.contains("traces", 0));

        ap.add_partition("traces", 0, 1);
        ap.add_partition("traces", 1, 1);
        ap.add_partition("metrics", 0, 1);

        assert!(ap.contains("traces", 0));
        assert!(ap.contains("traces", 1));
        assert!(ap.contains("metrics", 0));
        assert!(!ap.contains("traces", 2));
        assert!(!ap.contains("logs", 0));

        // remove_partition reports whether an owned partition was removed.
        assert!(ap.remove_partition("traces", 0));
        assert!(!ap.contains("traces", 0));
        assert!(ap.contains("traces", 1));

        // Removing the last partition drops the topic entry.
        assert!(ap.remove_partition("metrics", 0));
        assert!(!ap.contains("metrics", 0));
        assert!(!ap.topics.contains_key("metrics"));
    }

    /// Scenario (consumer-group rebalancing): an already-owned partition is re-added with a
    /// different generation.
    /// Guarantees: the existing generation is retained (not overwritten), so a partition
    /// kept across a rebalance keeps its ownership generation.
    #[test]
    fn add_partition_keeps_existing_generation() {
        let mut ap = AssignedPartitions::new();
        ap.add_partition("traces", 0, 5);
        // Re-adding an already-owned partition must not change its generation
        // (retained across a rebalance).
        ap.add_partition("traces", 0, 9);
        assert_eq!(ap.generation("traces", 0), Some(5));
        assert_eq!(ap.generation("traces", 9), None);
    }

    /// Scenario (consumer-group rebalancing): an unknown partition or topic is removed from
    /// the assigned set.
    /// Guarantees: the removal reports `false` and changes nothing, so spurious revocations
    /// cannot corrupt the owned set.
    #[test]
    fn remove_unknown_partition_is_noop() {
        let mut ap = AssignedPartitions::new();
        ap.add_partition("traces", 0, 1);
        // Unknown partition/topic removals report false and change nothing.
        assert!(!ap.remove_partition("traces", 99));
        assert!(!ap.remove_partition("unknown", 0));
        assert!(ap.contains("traces", 0));
    }

    /// Scenario (consumer-group rebalancing): a full assignment is applied that retains
    /// some partitions and acquires others.
    /// Guarantees: only newly-acquired partitions get a fresh generation while retained
    /// ones keep theirs, so generation churn is scoped to genuine acquisitions.
    #[test]
    fn set_assignment_allocates_generation_for_new_partitions_only() {
        let state = RebalanceState::new(false);

        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        state.set_assignment(&tpl);
        let e0 = state.current_generation("traces", 0);
        assert!(e0 > 0);

        // Re-assigning the same set retains the partition -> generation unchanged.
        state.set_assignment(&tpl);
        assert_eq!(state.current_generation("traces", 0), e0);

        // Adding a new partition allocates a fresh, strictly-greater generation for
        // it while the retained partition keeps its generation.
        let mut tpl2 = TopicPartitionList::new();
        let _ = tpl2.add_partition("traces", 0);
        let _ = tpl2.add_partition("traces", 1);
        state.set_assignment(&tpl2);
        assert_eq!(state.current_generation("traces", 0), e0);
        assert!(state.current_generation("traces", 1) > e0);
    }

    /// Scenario (consumer-group rebalancing): a single rebalance acquires several
    /// partitions at once.
    /// Guarantees: all partitions acquired in that rebalance share one freshly-allocated
    /// generation (the allocator bumps at most once per call).
    #[test]
    fn set_assignment_shares_one_generation_across_partitions() {
        // All partitions acquired in a single rebalance share one generation,
        // and the allocator advances by exactly one per rebalance.
        let state = RebalanceState::new(false);

        // First rebalance acquires two partitions at once -> same generation.
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        let _ = tpl.add_partition("traces", 1);
        state.set_assignment(&tpl);
        let g0 = state.current_generation("traces", 0);
        let g1 = state.current_generation("traces", 1);
        assert_eq!(g0, g1, "partitions acquired together share one generation");
        assert_eq!(g0, 1, "first generation is 1");

        // Second rebalance retains 0 and 1, acquires 2 -> single bump to 2.
        let mut tpl2 = TopicPartitionList::new();
        let _ = tpl2.add_partition("traces", 0);
        let _ = tpl2.add_partition("traces", 1);
        let _ = tpl2.add_partition("traces", 2);
        state.set_assignment(&tpl2);
        assert_eq!(state.current_generation("traces", 0), g0);
        assert_eq!(state.current_generation("traces", 1), g1);
        assert_eq!(
            state.current_generation("traces", 2),
            2,
            "one bump for the rebalance that acquired partition 2"
        );

        // A pure-retain rebalance acquires nothing -> allocator not bumped.
        state.set_assignment(&tpl2);
        assert_eq!(state.current_generation("traces", 0), g0);
        assert_eq!(state.current_generation("traces", 2), 2);
    }

    /// Scenario (consumer-group rebalancing): the additive-merge fallback adds several
    /// partitions in one call.
    /// Guarantees: all partitions merged in that call share one freshly-allocated
    /// generation, matching the full-assignment path.
    #[test]
    fn merge_assignment_shares_one_generation_across_partitions() {
        let state = RebalanceState::new(false);

        // Seed one owned partition (generation 1).
        let mut initial = TopicPartitionList::new();
        let _ = initial.add_partition("traces", 0);
        state.set_assignment(&initial);

        // Merge a delta adding two partitions at once -> both share one new
        // generation (a single bump to 2).
        let mut delta = TopicPartitionList::new();
        let _ = delta.add_partition("traces", 1);
        let _ = delta.add_partition("traces", 2);
        state.merge_assignment(&delta);
        assert_eq!(state.current_generation("traces", 0), 1);
        assert_eq!(state.current_generation("traces", 1), 2);
        assert_eq!(state.current_generation("traces", 2), 2);
    }

    /// Scenario (consumer-group rebalancing): a partition is revoked and later reassigned
    /// to this consumer via `set_assignment`.
    /// Guarantees: the reacquired partition receives a strictly greater generation than its
    /// prior ownership period, so stale acks cannot be mistaken for the new period.
    #[test]
    fn reacquired_partition_gets_greater_generation() {
        // A partition revoked and later reassigned to this consumer must get a
        // strictly greater generation than its prior ownership period.
        let state = RebalanceState::new(false);

        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        state.set_assignment(&tpl);
        let first = state.current_generation("traces", 0);

        // Revoke it (drops from the assigned set).
        {
            let mut assigned = state.lock_assigned();
            assert!(assigned.remove_partition("traces", 0));
        }

        // Reassign -> new, greater generation.
        state.set_assignment(&tpl);
        assert!(state.current_generation("traces", 0) > first);
    }

    /// Scenario (consumer-group rebalancing): the assignment-query-failure fallback merges
    /// a cooperative delta that reports only a newly-gained partition.
    /// Guarantees: retained partitions survive with their original generation and only the
    /// new partition is added and counted, so the fallback never drops an owned partition.
    #[test]
    fn merge_assignment_adds_new_without_dropping_retained() {
        // The assignment-query-failure fallback must never drop a retained
        // partition: merging a cooperative delta only adds new partitions.
        let state = RebalanceState::new(false);

        let mut initial = TopicPartitionList::new();
        let _ = initial.add_partition("traces", 0);
        state.set_assignment(&initial);
        let g0 = state.current_generation("traces", 0);
        let _ = state.drain_metrics(); // reset counters

        // A delta reporting only the newly-gained partition 1.
        let mut delta = TopicPartitionList::new();
        let _ = delta.add_partition("traces", 1);
        state.merge_assignment(&delta);

        // Retained partition 0 survives with its original generation; partition
        // 1 is added with a fresh, strictly-greater generation.
        assert!(state.is_assigned("traces", 0));
        assert_eq!(state.current_generation("traces", 0), g0);
        assert!(state.is_assigned("traces", 1));
        assert!(state.current_generation("traces", 1) > g0);

        // Only the newly-added partition is counted.
        assert_eq!(state.drain_metrics().partition_assignments, 1);
    }

    /// Scenario (consumer-group rebalancing): the merge fallback is given a partition the
    /// consumer already owns.
    /// Guarantees: nothing changes -- no generation churn and no assignment count -- so
    /// re-reporting an owned partition is inert.
    #[test]
    fn merge_assignment_is_noop_for_already_owned() {
        let state = RebalanceState::new(false);

        let mut initial = TopicPartitionList::new();
        let _ = initial.add_partition("traces", 0);
        state.set_assignment(&initial);
        let g0 = state.current_generation("traces", 0);
        let _ = state.drain_metrics();

        // Merging a partition we already own changes nothing.
        state.merge_assignment(&initial);
        assert_eq!(state.current_generation("traces", 0), g0);
        assert_eq!(state.drain_metrics().partition_assignments, 0);
    }

    /// Scenario (consumer-group rebalancing): revoked partitions are queued with their
    /// ownership generation and then drained.
    /// Guarantees: each drained revocation carries its generation tag, so the receive loop
    /// can purge only same-or-older tracker state.
    #[test]
    fn drain_revoked_preserves_generation_tag() {
        // Revocations carry the generation of the ownership period being
        // revoked, so the receive loop can purge only same-or-older tracker
        // state. (handle_revoke stamps this; exercised end-to-end in the
        // receiver integration tests.)
        let state = RebalanceState::new(false);
        state.push_revoked_for_test("traces", 0, 1);
        state.push_revoked_for_test("traces", 1, 2);

        let mut drained = state.drain_revoked();
        drained.sort_by_key(|r| r.partition);
        assert_eq!(drained[0].partition, 0);
        assert_eq!(drained[0].generation, 1);
        assert_eq!(drained[1].partition, 1);
        assert_eq!(drained[1].generation, 2);
    }

    /// Scenario (consumer-group rebalancing): a new full assignment replaces a prior one
    /// that owned different partitions.
    /// Guarantees: the old partitions are dropped, the new ones owned, and the assignment
    /// counter reflects only newly-acquired partitions across both assignments.
    #[test]
    fn set_assignment_replaces_and_counts_new_partitions() {
        let state = RebalanceState::new(false);

        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        let _ = tpl.add_partition("traces", 1);
        state.set_assignment(&tpl);

        assert!(state.is_assigned("traces", 0));
        assert!(state.is_assigned("traces", 1));

        // A new full assignment replaces the old one.
        let mut tpl2 = TopicPartitionList::new();
        let _ = tpl2.add_partition("metrics", 0);
        state.set_assignment(&tpl2);

        assert!(!state.is_assigned("traces", 0));
        assert!(!state.is_assigned("traces", 1));
        assert!(state.is_assigned("metrics", 0));

        // 2 (initial) + 1 (metrics-0 is newly added).
        let delta = state.drain_metrics();
        assert_eq!(delta.partition_assignments, 3);
    }

    /// Scenario (consumer-group rebalancing): a cooperative-sticky rebalance stores the
    /// full queried assignment that keeps one partition, drops another, and gains a third.
    /// Guarantees: the retained partition stays assigned (its acks are not rejected as
    /// revoked), so storing the full set -- not the delta -- preserves ownership.
    #[test]
    fn set_assignment_retains_partitions_across_cooperative_rebalance() {
        // Regression: under the cooperative-sticky protocol, post_rebalance
        // reports only the delta, but we store the full assignment queried from
        // the consumer. Simulate the full set the query would return.
        let state = RebalanceState::new(false);

        // Initially own partitions 0 and 1.
        let mut initial = TopicPartitionList::new();
        let _ = initial.add_partition("traces", 0);
        let _ = initial.add_partition("traces", 1);
        state.set_assignment(&initial);

        // After the rebalance: kept 0, dropped 1, gained 2. The full assignment
        // is {0, 2}.
        let mut full = TopicPartitionList::new();
        let _ = full.add_partition("traces", 0);
        let _ = full.add_partition("traces", 2);
        state.set_assignment(&full);

        // Retained partition 0 must still be assigned (previously this was
        // cleared, causing ACK/NACK for it to be rejected as revoked).
        assert!(state.is_assigned("traces", 0));
        assert!(state.is_assigned("traces", 2));
        assert!(!state.is_assigned("traces", 1));

        // Only partition 2 is newly added on the second assignment: 2 + 1.
        let delta = state.drain_metrics();
        assert_eq!(delta.partition_assignments, 3);
    }

    /// Scenario (consumer-group rebalancing): `set_assignment` counts only genuinely-new partitions while
    /// retaining previously-owned ones.
    /// Guarantees: the `partition_assignments` counter is bumped solely for
    /// newly-acquired partitions (retained ones excluded) and current ownership
    /// reflects the full set, so the metric never double-counts a retained
    /// partition across cooperative rebalances.
    #[test]
    fn set_assignment_counts_only_newly_acquired_partitions() {
        let state = RebalanceState::new(false);

        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        state.set_assignment(&tpl);
        // First assignment acquires one partition.
        assert_eq!(state.drain_metrics().partition_assignments, 1);
        assert!(state.is_assigned("traces", 0));

        // Retaining 0 and gaining 1: only 1 is newly acquired.
        let mut tpl2 = TopicPartitionList::new();
        let _ = tpl2.add_partition("traces", 0);
        let _ = tpl2.add_partition("traces", 1);
        state.set_assignment(&tpl2);
        assert_eq!(state.drain_metrics().partition_assignments, 1);
        assert!(state.is_assigned("traces", 0));
        assert!(state.is_assigned("traces", 1));
    }

    /// Scenario (consumer-group rebalancing): partitions are revoked down to an empty assignment.
    /// Guarantees: `AssignedPartitions::len` and the drained `partitions_owned`
    /// snapshot both reach zero, which is the signal the receiver uses to emit
    /// the `kafka.assignment.became_empty` event.
    #[test]
    fn owned_count_reaches_zero_after_full_revocation() {
        let state = RebalanceState::new(false);
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        state.set_assignment(&tpl);
        assert_eq!(state.lock_assigned().len(), 1);

        // Simulate the revoke bookkeeping dropping the only owned partition.
        {
            let mut assigned = state.lock_assigned();
            assert!(assigned.remove_partition("traces", 0));
            assert_eq!(assigned.len(), 0);
        }
        assert_eq!(state.drain_metrics().partitions_owned, 0);
    }

    /// Scenario (consumer-group rebalancing): the revoked queue is drained while empty and
    /// again after revocations are queued.
    /// Guarantees: an empty drain returns nothing and a populated drain returns then clears
    /// the queue, so revocations are delivered exactly once to the receive loop.
    #[test]
    fn drain_revoked_empty_then_populated() {
        let state = RebalanceState::new(false);
        assert!(state.drain_revoked().is_empty());

        state.push_revoked_for_test("traces", 0, 0);
        state.push_revoked_for_test("traces", 1, 0);
        let drained = state.drain_revoked();
        assert_eq!(drained.len(), 2);
        // Second drain is empty again.
        assert!(state.drain_revoked().is_empty());
    }

    /// Scenario (consumer-group rebalancing): `handle_assign` runs against a live
    /// consumer whose `assignment()` query succeeds, so it takes the full-query
    /// (`Ok`) path rather than the merge fallback.
    /// Guarantees: the queried full assignment is stored (the assigned partition
    /// becomes owned with a real generation) and the rebalance/assignment
    /// counters advance, so the cooperative full-`assignment()` dispatch records
    /// the complete owned set.
    #[tokio::test]
    async fn handle_assign_query_path_stores_full_assignment() {
        const TOPIC: &str = "rebalance-handle-assign-traces";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
            |cluster| async move {
                let state = Arc::new(RebalanceState::new(false));
                let consumer =
                    mock_base_consumer(&cluster, "handle-assign-group", Arc::clone(&state));

                // Assign the partition so the consumer's own `assignment()` query
                // returns a non-empty full set (the Ok dispatch path).
                let mut tpl = TopicPartitionList::new();
                let _ = tpl.add_partition(TOPIC, 0);
                consumer.assign(&tpl).expect("assign");

                state.handle_assign(&consumer, &tpl);

                assert!(
                    state.is_assigned(TOPIC, 0),
                    "the queried full assignment should be stored as owned",
                );
                assert!(
                    state.current_generation(TOPIC, 0) >= 1,
                    "a newly-acquired partition should get a real (>=1) generation",
                );
                let delta = state.drain_metrics();
                assert_eq!(delta.rebalances_total, 1, "one assign event counted");
                assert_eq!(
                    delta.partition_assignments, 1,
                    "one newly-acquired partition counted",
                );
            },
        )
        .await;
    }

    /// Scenario (consumer-group rebalancing): `handle_revoke` runs against a live
    /// consumer for a subset of currently-owned partitions, with a committable
    /// snapshot in place.
    /// Guarantees: the revoke is scoped to only the requested partitions -- they
    /// are dropped from the assigned set, queued for tracker purge tagged with
    /// their ownership generation, and counted in `partition_revocations`, while
    /// a non-revoked owned partition stays assigned -- so a rebalance commits and
    /// releases only the partitions being revoked.
    #[tokio::test]
    async fn handle_revoke_is_scoped_and_tags_generation() {
        const TOPIC: &str = "rebalance-handle-revoke-traces";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
            |cluster| async move {
                let state = Arc::new(RebalanceState::new(false));
                let consumer =
                    mock_base_consumer(&cluster, "handle-revoke-group", Arc::clone(&state));

                // Own both partitions with a known generation.
                let mut full = TopicPartitionList::new();
                let _ = full.add_partition(TOPIC, 0);
                let _ = full.add_partition(TOPIC, 1);
                state.set_assignment(&full);
                let gen0 = state.current_generation(TOPIC, 0);
                assert!(gen0 >= 1);
                let _ = state.drain_metrics(); // reset counters after setup

                // A committable snapshot so the scoped commit has offsets to use.
                let mut snapshot = HashMap::new();
                let _ = snapshot.insert((TOPIC.to_string(), 0), 42_i64);
                let _ = snapshot.insert((TOPIC.to_string(), 1), 99_i64);
                state.set_committable_snapshot(snapshot);

                // Revoke only partition 0.
                let mut revoke = TopicPartitionList::new();
                let _ = revoke.add_partition(TOPIC, 0);
                state.handle_revoke(&consumer, &revoke);

                // Scoping: partition 0 released, partition 1 retained.
                assert!(
                    !state.is_assigned(TOPIC, 0),
                    "revoked partition 0 must be dropped from the assigned set",
                );
                assert!(
                    state.is_assigned(TOPIC, 1),
                    "non-revoked partition 1 must remain assigned",
                );

                // Revoked partition queued for purge, tagged with its generation.
                let drained = state.drain_revoked();
                assert_eq!(drained.len(), 1, "only the revoked partition is queued");
                assert_eq!(drained[0].partition, 0);
                assert_eq!(
                    drained[0].generation, gen0,
                    "the revocation carries the ownership generation being revoked",
                );

                // Only the genuinely-owned revoked partition is counted.
                assert_eq!(
                    state.drain_metrics().partition_revocations,
                    1,
                    "exactly one owned partition was revoked",
                );
            },
        )
        .await;
    }

    /// Scenario (consumer-group rebalancing): a partition is assigned, revoked,
    /// then reassigned to the same consumer through the real
    /// `handle_assign`/`handle_revoke` methods.
    /// Guarantees: the reacquired partition receives a strictly greater ownership
    /// generation than its prior period, so acks stamped with the old generation
    /// cannot be mistaken for the new ownership after a return.
    #[tokio::test]
    async fn reacquired_partition_via_handle_gets_greater_generation() {
        const TOPIC: &str = "rebalance-handle-reacquire-traces";
        with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
            |cluster| async move {
                let state = Arc::new(RebalanceState::new(false));
                let consumer =
                    mock_base_consumer(&cluster, "handle-reacquire-group", Arc::clone(&state));

                let mut tpl = TopicPartitionList::new();
                let _ = tpl.add_partition(TOPIC, 0);

                // Assign -> generation g1.
                consumer.assign(&tpl).expect("assign");
                state.handle_assign(&consumer, &tpl);
                let g1 = state.current_generation(TOPIC, 0);
                assert!(g1 >= 1);

                // Revoke (drops from the assigned set; also unassign on the
                // consumer so the next query reflects the reacquisition).
                state.handle_revoke(&consumer, &tpl);
                consumer.unassign().expect("unassign");
                assert!(!state.is_assigned(TOPIC, 0));

                // Reassign -> strictly greater generation g2.
                consumer.assign(&tpl).expect("reassign");
                state.handle_assign(&consumer, &tpl);
                let g2 = state.current_generation(TOPIC, 0);
                assert!(
                    g2 > g1,
                    "a reacquired partition must get a strictly greater generation ({g2} > {g1})",
                );
            },
        )
        .await;
    }

    // ---- Offset guarantees ----

    /// Scenario (offset guarantees): a committable snapshot is set and a commit TPL is
    /// built scoped to a subset of revoked partitions.
    /// Guarantees: only the requested revoked partitions with a known committable offset
    /// appear in the commit TPL, so a pre-revoke commit is scoped to exactly those
    /// partitions.
    #[test]
    fn refresh_and_drain_committable_via_build() {
        let state = RebalanceState::new(false);
        let mut snapshot = HashMap::new();
        let _ = snapshot.insert(("traces".to_string(), 0), 100);
        let _ = snapshot.insert(("traces".to_string(), 1), 200);
        let _ = snapshot.insert(("metrics".to_string(), 0), 300);
        state.set_committable_snapshot(snapshot);

        // Only the requested revoked partitions appear in the commit TPL.
        let revoked = vec![("traces".to_string(), 0), ("metrics".to_string(), 0)];
        let committable = lock_ignore_poison(&state.committable);
        let tpl = build_commit_tpl(&committable, &revoked);
        assert_eq!(tpl.count(), 2);
        let map = tpl.to_topic_map();
        assert_eq!(
            map.get(&("traces".to_string(), 0)),
            Some(&Offset::Offset(100))
        );
        assert_eq!(
            map.get(&("metrics".to_string(), 0)),
            Some(&Offset::Offset(300))
        );
        // Partition 1 of traces was not in the revoked list.
        assert!(!map.contains_key(&("traces".to_string(), 1)));
    }

    /// Scenario (offset guarantees): a commit TPL is built for a revoked partition that has
    /// no committable offset.
    /// Guarantees: partitions without a committable offset are omitted from the TPL, so the
    /// receiver never commits an offset it does not have.
    #[test]
    fn build_commit_tpl_skips_unknown_offsets() {
        let mut committable = HashMap::new();
        let _ = committable.insert(("traces".to_string(), 0), 100);
        // partition 5 has no committable offset
        let revoked = vec![("traces".to_string(), 0), ("traces".to_string(), 5)];
        let tpl = build_commit_tpl(&committable, &revoked);
        assert_eq!(tpl.count(), 1);
    }

    // ---- Operational visibility ----

    /// Scenario (operational visibility): a full assignment is applied via `handle_assign`'s
    /// `set_assignment` and then drained.
    /// Guarantees: `partitions_owned` reports the current assignment size (an
    /// absolute snapshot) even though it is delivered alongside counter deltas,
    /// so `receiver.kafka.consumer.group.partitions` tracks live ownership.
    #[test]
    fn drain_metrics_reports_current_owned_count() {
        let state = RebalanceState::new(false);

        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        let _ = tpl.add_partition("traces", 1);
        state.set_assignment(&tpl);

        assert_eq!(state.drain_metrics().partitions_owned, 2);

        // A subsequent full assignment that shrinks ownership is reflected.
        let mut smaller = TopicPartitionList::new();
        let _ = smaller.add_partition("traces", 0);
        state.set_assignment(&smaller);
        assert_eq!(state.drain_metrics().partitions_owned, 1);
    }

    /// Scenario (operational visibility): each `handle_assign`-driven assignment increments the rebalance
    /// event counter once.
    /// Guarantees: `rebalances_total` counts assign events (not partitions), so
    /// operators can distinguish rebalance frequency from partition churn.
    #[test]
    fn rebalances_total_counts_assign_events() {
        let state = RebalanceState::new(false);
        assert_eq!(state.drain_metrics().rebalances_total, 0);

        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        // set_assignment alone does not bump the event counter; handle_assign
        // does. Exercise the counter directly via the shared atomic path by
        // simulating two assign events.
        let _ = state.rebalances_total.fetch_add(1, Ordering::Relaxed);
        state.set_assignment(&tpl);
        let _ = state.rebalances_total.fetch_add(1, Ordering::Relaxed);

        assert_eq!(state.drain_metrics().rebalances_total, 2);
        assert_eq!(state.drain_metrics().rebalances_total, 0, "drain resets");
    }

    /// Scenario (operational visibility): a freshly-drained rebalance metrics delta with no
    /// events is inspected.
    /// Guarantees: `is_empty()` reports true only when every counter delta is zero, so the
    /// receive loop skips redundant metric folds on idle ticks.
    #[test]
    fn metrics_delta_is_empty() {
        let state = RebalanceState::new(false);
        assert!(state.drain_metrics().is_empty());
    }

    /// Scenario (operational visibility): successful and failed commit outcomes are
    /// recorded on the shared rebalance state.
    /// Guarantees: successes increment `offset_commits` and failures increment
    /// `offset_commit_errors`, so a commit failure is counted distinctly from a success.
    #[test]
    fn record_commit_result_counts_success_and_failure() {
        let state = RebalanceState::new(false);

        state.record_commit_result(&Ok(()));
        state.record_commit_result(&Ok(()));
        state.record_commit_result(&Err(rdkafka::error::KafkaError::ClientCreation(
            "boom".to_string(),
        )));

        let delta = state.drain_metrics();
        assert_eq!(delta.offset_commits, 2);
        assert_eq!(delta.offset_commit_errors, 1);

        // Draining resets the counters.
        let empty = state.drain_metrics();
        assert_eq!(empty.offset_commits, 0);
        assert_eq!(empty.offset_commit_errors, 0);
    }

    /// Scenario (operational visibility): a commit outcome is recorded and the metrics
    /// delta is inspected.
    /// Guarantees: the recorded outcome makes the delta non-empty, so the receive loop
    /// folds commit metrics on the next reconcile rather than skipping them.
    #[test]
    fn commit_result_makes_delta_non_empty() {
        let state = RebalanceState::new(false);
        state.record_commit_result(&Ok(()));
        assert!(!state.drain_metrics().is_empty());
    }

    /// Scenario (operational visibility): commit outcomes are delivered through the real
    /// `ConsumerContext::commit_callback`.
    /// Guarantees: the callback folds one success and one failure into
    /// `offset_commits`/`offset_commit_errors`, so the librdkafka wiring records async
    /// commit outcomes correctly.
    #[test]
    fn commit_callback_folds_results_via_context() {
        // Exercise the callback through the real ConsumerContext impl to ensure
        // the wiring (auto-commit gate + state folding) is correct.
        let state = Arc::new(RebalanceState::new(false));
        let ctx = RebalancingConsumerContext::Default(Arc::clone(&state));
        let empty_tpl = TopicPartitionList::new();

        ctx.commit_callback(Ok(()), &empty_tpl);
        ctx.commit_callback(
            Err(rdkafka::error::KafkaError::ClientCreation(
                "boom".to_string(),
            )),
            &empty_tpl,
        );

        let delta = state.drain_metrics();
        assert_eq!(delta.offset_commits, 1);
        assert_eq!(delta.offset_commit_errors, 1);
    }

    /// Scenario (operational visibility): commit callbacks arrive while the receiver is in
    /// auto-commit mode.
    /// Guarantees: the callback records nothing (librdkafka owns offsets), so auto-commit
    /// mode keeps the manual commit metrics clean.
    #[test]
    fn commit_callback_is_noop_in_auto_commit() {
        let state = Arc::new(RebalanceState::new(true));
        let ctx = RebalancingConsumerContext::Default(Arc::clone(&state));
        let empty_tpl = TopicPartitionList::new();

        ctx.commit_callback(Ok(()), &empty_tpl);
        ctx.commit_callback(
            Err(rdkafka::error::KafkaError::ClientCreation(
                "boom".to_string(),
            )),
            &empty_tpl,
        );

        let delta = state.drain_metrics();
        assert_eq!(delta.offset_commits, 0);
        assert_eq!(delta.offset_commit_errors, 0);
    }

    /// Scenario (operational visibility): partition tokens are appended to a structured-log buffer.
    /// Guarantees: `PartitionListFmt` eagerly builds a stable, compact,
    /// comma-separated `topic-partition` string (no leading/trailing comma, no
    /// truncation marker when nothing was dropped) so assignment/revocation logs
    /// are human- and machine-readable, and reports an accurate un-truncated
    /// count/listed_count.
    #[test]
    fn append_partition_builds_comma_separated_list() {
        let mut fmt = PartitionListFmt::default();
        // An empty buffer is the empty string.
        assert_eq!(fmt.as_str(), "");
        assert_eq!(fmt.count(), 0);
        fmt.append("traces", 0);
        fmt.append("traces", 1);
        fmt.append("metrics", 3);
        assert_eq!(fmt.as_str(), "traces:0,traces:1,metrics:3");
        assert_eq!(fmt.count(), 3);
        assert_eq!(fmt.listed_count(), 3);
        // Nothing was dropped, so no truncation marker is present.
        assert!(!fmt.truncated());
        assert!(!fmt.as_str().ends_with("..."));
    }

    /// Scenario (operational visibility): more `topic-partition` tokens are appended than the entry cap
    /// `MAX_LISTED_PARTITIONS` permits.
    /// Guarantees: the rendered list is capped at `MAX_LISTED_PARTITIONS`
    /// tokens, a single trailing `...` marker signals the drop, `truncated()`
    /// flips to `true`, `listed_count()` equals the cap, and `count()` still
    /// reports the full total -- so a huge rebalance never emits an unbounded log
    /// line yet the true magnitude is preserved.
    #[test]
    fn partition_list_fmt_truncates_beyond_cap() {
        let mut fmt = PartitionListFmt::default();
        let total = MAX_LISTED_PARTITIONS + 10;
        for partition in 0..total {
            fmt.append("traces", partition as i32);
        }
        assert_eq!(fmt.count(), total);
        assert_eq!(fmt.listed_count(), MAX_LISTED_PARTITIONS);
        assert!(fmt.truncated());
        let rendered = fmt.as_str();
        // Truncation appends exactly one `...` marker.
        assert!(rendered.ends_with("..."), "rendered = {rendered}");
        assert_eq!(rendered.matches("...").count(), 1, "exactly one marker");
        // The rendered list contains exactly `MAX_LISTED_PARTITIONS` tokens plus
        // the marker: one comma between each of the cap tokens and one before
        // the marker == `MAX_LISTED_PARTITIONS` commas.
        let commas = rendered.matches(',').count() as u64;
        assert_eq!(commas, MAX_LISTED_PARTITIONS);
        // The last listed token is the one at index `MAX_LISTED_PARTITIONS - 1`;
        // nothing past the cap leaks in.
        assert!(rendered.contains(&format!("traces:{}", MAX_LISTED_PARTITIONS - 1)));
        assert!(!rendered.contains(&format!("traces:{MAX_LISTED_PARTITIONS}")));
    }

    /// Scenario (operational visibility): exactly the cap number of tokens are appended.
    /// Guarantees: a list at the cap boundary is fully rendered with no trailing
    /// `...` marker and `truncated()` stays `false`, so the marker has no
    /// off-by-one at the boundary.
    #[test]
    fn partition_list_fmt_at_cap_is_not_truncated() {
        let mut fmt = PartitionListFmt::default();
        for partition in 0..MAX_LISTED_PARTITIONS {
            fmt.append("traces", partition as i32);
        }
        assert_eq!(fmt.count(), MAX_LISTED_PARTITIONS);
        assert_eq!(fmt.listed_count(), MAX_LISTED_PARTITIONS);
        assert!(!fmt.truncated());
        let rendered = fmt.as_str();
        assert!(!rendered.ends_with("..."), "rendered = {rendered}");
        // Exactly `MAX_LISTED_PARTITIONS` tokens => one fewer comma.
        let commas = rendered.matches(',').count() as u64;
        assert_eq!(commas, MAX_LISTED_PARTITIONS - 1);
    }
}
