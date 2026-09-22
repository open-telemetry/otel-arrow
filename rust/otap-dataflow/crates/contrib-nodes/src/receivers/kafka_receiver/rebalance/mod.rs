// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-group rebalance handling for the Kafka receiver.
//!
//! The receiver tracks pending offsets in memory via the
//! [`OffsetTracker`](super::offset_tracker::OffsetTracker), which lives on the
//! single-threaded `LocalSet` runtime and is owned by the receive loop. Kafka
//! consumer-group rebalances are delivered through the [`ConsumerContext`]
//! callbacks. In rdkafka 0.38.0 these callbacks are served inline by
//! `consumer.recv()` (`MessageStream::poll_next` -> `BaseConsumer::poll_queue`
//! runs any queued rebalance/commit event on the calling thread), and the
//! receive loop is the only caller of `recv()`, so they run on the same
//! single-threaded pipeline thread as the loop rather than on a separate
//! librdkafka poll thread. They still may not mutate the `LocalSet`-owned
//! tracker directly: they only record facts into the shared state below, which
//! the loop reconciles on its next turn.
//!
//! NOTE: because the callbacks run on the pipeline thread, the synchronous
//! commit-before-revoke in `RebalanceState::handle_revoke` (a
//! `CommitMode::Sync` broker round-trip) executes on the single-threaded runtime
//! and can block it while a rebalance is processed inside `recv()`. It is
//! bounded by librdkafka's internal commit timeout; moving this commit off the
//! pipeline thread is future work.
//!
//! This module bridges the two concerns with a small amount of shared,
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

mod assignment_resume;
mod context;

pub(crate) use assignment_resume::PartitionResumeOperations;
pub(crate) use context::RebalancingConsumerContext;

use assignment_resume::{
    AssignmentResumeRetries, MAX_DUE_ASSIGNMENT_RESUMES_PER_TURN, ResumePhase,
};
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer, ConsumerContext};
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

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
    /// Newly assigned partitions whose immediate resume failed. The receive
    /// loop retries these with capped backoff until resume succeeds or the
    /// ownership generation changes.
    assignment_resume_retries: Mutex<AssignmentResumeRetries>,
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
    /// Count of failed resume operations while clearing rebalance pause state.
    rebalance_resume_errors: AtomicU64,
    /// Count of offset commits acknowledged by the broker, observed on the
    /// commit callback (callback-incremented). Covers the receiver's async
    /// steady-state commits and the sync pre-rebalance commit.
    offset_commits: AtomicU64,
    /// Count of offset commits rejected by the broker, observed on the commit
    /// callback (callback-incremented).
    offset_commit_errors: AtomicU64,
}

/// A batch of rebalance counter deltas drained by the receive loop and folded
/// into the receiver's [`MetricSet`](otel_arrow_dfe_telemetry::metrics::MetricSet).
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
    /// Resume failures while clearing rebalance pause state since the last drain.
    pub(crate) rebalance_resume_errors: u64,
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
            && self.rebalance_resume_errors == 0
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
            assignment_resume_retries: Mutex::new(AssignmentResumeRetries::default()),
            committable: Mutex::new(HashMap::new()),
            auto_commit,
            rebalances_total: AtomicU64::new(0),
            partition_assignments: AtomicU64::new(0),
            partition_revocations: AtomicU64::new(0),
            rebalance_commit_errors: AtomicU64::new(0),
            rebalance_resume_errors: AtomicU64::new(0),
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

    /// Earliest pending assignment-resume retry deadline.
    #[must_use]
    pub(crate) fn next_assignment_resume_deadline(&self) -> Option<Instant> {
        lock_ignore_poison(&self.assignment_resume_retries).next_deadline()
    }

    /// Retry due assignment resumes without blocking the rebalance callback.
    pub(crate) fn process_due_assignment_resumes<O: PartitionResumeOperations + ?Sized>(
        &self,
        consumer: &O,
    ) {
        self.process_due_assignment_resumes_at(consumer, Instant::now());
    }

    fn process_due_assignment_resumes_at<O: PartitionResumeOperations + ?Sized>(
        &self,
        consumer: &O,
        now: Instant,
    ) {
        let due = lock_ignore_poison(&self.assignment_resume_retries)
            .take_due(now, MAX_DUE_ASSIGNMENT_RESUMES_PER_TURN);
        for retry in due {
            if self.current_generation(&retry.topic, retry.partition) != retry.generation {
                lock_ignore_poison(&self.assignment_resume_retries).complete(&retry);
                continue;
            }

            let mut tpl = TopicPartitionList::new();
            let _ = tpl.add_partition(&retry.topic, retry.partition);
            match consumer.resume_partitions(&tpl) {
                Ok(()) => {
                    lock_ignore_poison(&self.assignment_resume_retries).complete(&retry);
                    otel_info!(
                        "kafka.rebalance.resume.recovered",
                        topic = %retry.topic,
                        partition = retry.partition,
                        attempts = retry.failures,
                    );
                }
                Err(error) => {
                    self.record_resume_failure(ResumePhase::Assign, &error);
                    lock_ignore_poison(&self.assignment_resume_retries).reschedule(&retry, now);
                }
            }
        }
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
            rebalance_resume_errors: self.rebalance_resume_errors.swap(0, Ordering::Relaxed),
            offset_commits: self.offset_commits.swap(0, Ordering::Relaxed),
            offset_commit_errors: self.offset_commit_errors.swap(0, Ordering::Relaxed),
        }
    }

    /// Record the outcome of an offset commit reported by librdkafka on the
    /// commit callback. The commit callback is served inline by
    /// `consumer.recv()`, so this runs on the pipeline thread for both the
    /// receiver's async commits and the synchronous pre-rebalance commit.
    pub(super) fn record_commit_result(&self, result: &rdkafka::error::KafkaResult<()>) {
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
    ///
    /// The commit below is synchronous (`CommitMode::Sync`) so owned partitions
    /// are persisted before they leave the member. Because `pre_rebalance` is
    /// served inline by `consumer.recv()` (see the module docs), this runs on
    /// the single-threaded pipeline thread and can block the receive loop for
    /// the duration of the broker round-trip during a rebalance. It is bounded
    /// by librdkafka's internal commit timeout; moving it off the pipeline
    /// thread is future work.
    pub(super) fn handle_revoke<C: ConsumerContext>(
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

        if commit_tpl.count() > 0
            && let Err(e) = consumer.commit(&commit_tpl, CommitMode::Sync)
        {
            let _ = self.rebalance_commit_errors.fetch_add(1, Ordering::Relaxed);
            otel_error!(
                "kafka.rebalance.commit_failed",
                error = %e,
            );
        }

        // Pause state is client-local and survives revocation/reassignment in
        // librdkafka. Clear it while these partitions are still assigned so a
        // later acquisition by this consumer cannot remain paused forever after
        // the receive loop discards its retry deadline.
        self.resume_rebalance_partitions(consumer, tpl, ResumePhase::Revoke);

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
    pub(super) fn handle_assign<C: ConsumerContext>(
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

        // `tpl` is the newly-assigned delta under cooperative rebalancing and
        // the full newly-owned set under eager rebalancing. Resume exactly this
        // set so retained partitions with an active retry remain paused.
        self.resume_rebalance_partitions(base_consumer, tpl, ResumePhase::Assign);
    }

    fn resume_rebalance_partitions<O: PartitionResumeOperations + ?Sized>(
        &self,
        consumer: &O,
        tpl: &TopicPartitionList,
        phase: ResumePhase,
    ) {
        match consumer.resume_partitions(tpl) {
            Ok(()) => self.clear_assignment_resume_retries(tpl),
            Err(error) => {
                self.record_resume_failure(phase, &error);
                if phase == ResumePhase::Assign {
                    self.schedule_assignment_resume_retries(tpl, Instant::now());
                } else {
                    self.clear_assignment_resume_retries(tpl);
                }
            }
        }
    }

    fn record_resume_failure(&self, phase: ResumePhase, error: &rdkafka::error::KafkaError) {
        let _ = self.rebalance_resume_errors.fetch_add(1, Ordering::Relaxed);
        otel_error!(
            "kafka.rebalance.resume.fail",
            phase = phase.as_str(),
            "exception.type" = "rdkafka",
            "exception.message" = %error,
        );
    }

    fn schedule_assignment_resume_retries(&self, tpl: &TopicPartitionList, now: Instant) {
        let partitions = topic_partitions(tpl);
        let assigned = self.lock_assigned();
        let owned = partitions
            .into_iter()
            .filter_map(|(topic, partition)| {
                assigned
                    .generation(&topic, partition)
                    .map(|generation| (topic, partition, generation))
            })
            .collect::<Vec<_>>();
        drop(assigned);

        let mut retries = lock_ignore_poison(&self.assignment_resume_retries);
        for (topic, partition, generation) in owned {
            retries.schedule(topic, partition, generation, 1, now);
        }
    }

    fn clear_assignment_resume_retries(&self, tpl: &TopicPartitionList) {
        let mut retries = lock_ignore_poison(&self.assignment_resume_retries);
        for key in topic_partitions(tpl) {
            retries.remove(&key);
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

#[cfg(test)]
mod tests;
