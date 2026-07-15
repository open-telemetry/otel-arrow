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
//! - **`assigned`** — the set of topic-partitions currently owned by this
//!   consumer. Updated by the rebalance callbacks and read by the receive loop
//!   to scope commits to owned partitions only.
//! - **`committable`** — a snapshot of the offset that would be committed for
//!   each tracked partition, refreshed by the receive loop after each
//!   ack/commit cycle. Read by [`pre_rebalance`](ConsumerContext::pre_rebalance)
//!   to commit owned partitions *before* they are revoked (commit-before-revoke).
//! - **`revoked`** — a queue of partitions revoked since the loop last
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

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "aws")]
use crate::common::kafka::aws::AwsMskAuthClientContext;
use otap_df_telemetry::{otel_error, otel_warn};
use rdkafka::ClientContext;
use rdkafka::client::OAuthToken;
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};

/// The set of topic-partitions currently assigned to this consumer.
///
/// Keyed by topic name, then by partition. Used to scope offset commits to
/// partitions this consumer actually owns, so that a late ack/nack or a
/// periodic timer tick never commits an offset for a partition that has been
/// reassigned to another consumer.
#[derive(Debug, Default)]
pub(crate) struct AssignedPartitions {
    topics: HashMap<String, HashSet<i32>>,
}

impl AssignedPartitions {
    /// Create an empty assignment set.
    pub(crate) fn new() -> Self {
        Self {
            topics: HashMap::new(),
        }
    }

    /// Remove all assigned partitions.
    pub(crate) fn clear(&mut self) {
        self.topics.clear();
    }

    /// Record `(topic, partition)` as assigned.
    pub(crate) fn add_partition(&mut self, topic: &str, partition: i32) {
        let _ = self
            .topics
            .entry(topic.to_string())
            .or_default()
            .insert(partition);
    }

    /// Remove `(topic, partition)` from the assignment, dropping the topic
    /// entry once it has no remaining partitions.
    pub(crate) fn remove_partition(&mut self, topic: &str, partition: i32) {
        if let Some(partitions) = self.topics.get_mut(topic) {
            let _ = partitions.remove(&partition);
            if partitions.is_empty() {
                let _ = self.topics.remove(topic);
            }
        }
    }

    /// Returns `true` if `(topic, partition)` is currently assigned.
    #[must_use]
    pub(crate) fn contains(&self, topic: &str, partition: i32) -> bool {
        self.topics
            .get(topic)
            .is_some_and(|partitions| partitions.contains(&partition))
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
    /// Partitions revoked since the receive loop last reconciled. Drained by
    /// the loop, which then purges the tracker.
    revoked: Mutex<Vec<(String, i32)>>,
    /// Latest committable offset per `(topic, partition)`, refreshed by the
    /// receive loop. Read by `pre_rebalance` to commit before revocation.
    committable: Mutex<HashMap<(String, i32), i64>>,
    /// When `true`, rebalance handling is skipped (librdkafka owns offsets).
    auto_commit: bool,
    /// Count of partitions assigned across rebalances (callback-incremented).
    partitions_assigned: AtomicU64,
    /// Count of partitions revoked across rebalances (callback-incremented).
    partitions_revoked: AtomicU64,
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
    /// Partitions assigned since the last drain.
    pub(crate) partitions_assigned: u64,
    /// Partitions revoked since the last drain.
    pub(crate) partitions_revoked: u64,
    /// Commit failures during revoke since the last drain.
    pub(crate) rebalance_commit_errors: u64,
    /// Broker-acknowledged offset commits since the last drain.
    pub(crate) offset_commits: u64,
    /// Broker-rejected offset commits since the last drain.
    pub(crate) offset_commit_errors: u64,
}

impl RebalanceMetricsDelta {
    /// Returns `true` if every counter is zero (nothing to report).
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.partitions_assigned == 0
            && self.partitions_revoked == 0
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
            partitions_assigned: AtomicU64::new(0),
            partitions_revoked: AtomicU64::new(0),
            rebalance_commit_errors: AtomicU64::new(0),
            offset_commits: AtomicU64::new(0),
            offset_commit_errors: AtomicU64::new(0),
        }
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
    pub(crate) fn drain_revoked(&self) -> Vec<(String, i32)> {
        let mut guard = lock_ignore_poison(&self.revoked);
        if guard.is_empty() {
            return Vec::new();
        }
        std::mem::take(&mut *guard)
    }

    /// Drain accumulated rebalance metric counters into a delta.
    pub(crate) fn drain_metrics(&self) -> RebalanceMetricsDelta {
        RebalanceMetricsDelta {
            partitions_assigned: self.partitions_assigned.swap(0, Ordering::Relaxed),
            partitions_revoked: self.partitions_revoked.swap(0, Ordering::Relaxed),
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

    /// Test-only: enqueue a revoked partition as if a rebalance callback had
    /// fired, so the receive-loop reconcile path can be exercised without a
    /// live broker.
    #[cfg(test)]
    pub(crate) fn push_revoked_for_test(&self, topic: &str, partition: i32) {
        lock_ignore_poison(&self.revoked).push((topic.to_string(), partition));
    }

    /// Test-only: mark a partition as assigned without going through a
    /// rebalance callback.
    #[cfg(test)]
    pub(crate) fn assign_for_test(&self, topic: &str, partition: i32) {
        self.lock_assigned().add_partition(topic, partition);
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

        {
            let mut assigned = self.lock_assigned();
            for (topic, partition) in &revoked {
                assigned.remove_partition(topic, *partition);
            }
        }
        // Queue the revoked partitions for the receive loop to purge from the
        // tracker, and drop them from the assigned set.
        {
            let mut revoked_queue = lock_ignore_poison(&self.revoked);
            revoked_queue.extend(revoked.iter().cloned());
        }

        let _ = self
            .partitions_revoked
            .fetch_add(revoked.len() as u64, Ordering::Relaxed);
    }

    /// Replace the assigned set with the *complete* assignment `full`.
    ///
    /// `full` must be the consumer's entire current assignment, not a rebalance
    /// delta. The `partitions_assigned` metric is incremented only by the number
    /// of partitions that are newly present (not previously owned), so
    /// cooperative-sticky rebalances that retain partitions don't re-count them.
    fn set_assignment(&self, full: &TopicPartitionList) {
        let full_partitions = topic_partitions(full);
        let mut assigned = self.lock_assigned();
        // Count partitions that are newly assigned relative to the previous set.
        let newly_added = full_partitions
            .iter()
            .filter(|(topic, partition)| !assigned.contains(topic, *partition))
            .count();
        assigned.clear();
        for (topic, partition) in &full_partitions {
            assigned.add_partition(topic, *partition);
        }
        let _ = self
            .partitions_assigned
            .fetch_add(newly_added as u64, Ordering::Relaxed);
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
    /// If the query fails (rare), fall back to the reported delta so behavior is
    /// no worse than storing the delta directly.
    fn handle_assign<C: ConsumerContext>(
        &self,
        base_consumer: &BaseConsumer<C>,
        tpl: &TopicPartitionList,
    ) {
        match base_consumer.assignment() {
            Ok(full) => self.set_assignment(&full),
            Err(e) => {
                otel_warn!(
                    "kafka.rebalance.assignment_query_failed",
                    error = %e,
                );
                self.set_assignment(tpl);
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
    use std::sync::Arc;

    #[test]
    fn assigned_partitions_add_remove_contains() {
        let mut ap = AssignedPartitions::new();
        assert!(!ap.contains("traces", 0));

        ap.add_partition("traces", 0);
        ap.add_partition("traces", 1);
        ap.add_partition("metrics", 0);

        assert!(ap.contains("traces", 0));
        assert!(ap.contains("traces", 1));
        assert!(ap.contains("metrics", 0));
        assert!(!ap.contains("traces", 2));
        assert!(!ap.contains("logs", 0));

        ap.remove_partition("traces", 0);
        assert!(!ap.contains("traces", 0));
        assert!(ap.contains("traces", 1));

        // Removing the last partition drops the topic entry.
        ap.remove_partition("metrics", 0);
        assert!(!ap.contains("metrics", 0));
        assert!(!ap.topics.contains_key("metrics"));
    }

    #[test]
    fn assigned_partitions_clear() {
        let mut ap = AssignedPartitions::new();
        ap.add_partition("traces", 0);
        ap.add_partition("metrics", 1);
        ap.clear();
        assert!(!ap.contains("traces", 0));
        assert!(!ap.contains("metrics", 1));
    }

    #[test]
    fn remove_unknown_partition_is_noop() {
        let mut ap = AssignedPartitions::new();
        ap.add_partition("traces", 0);
        ap.remove_partition("traces", 99);
        ap.remove_partition("unknown", 0);
        assert!(ap.contains("traces", 0));
    }

    #[test]
    fn state_auto_commit_flag() {
        assert!(RebalanceState::new(true).is_auto_commit());
        assert!(!RebalanceState::new(false).is_auto_commit());
    }

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

    #[test]
    fn build_commit_tpl_skips_unknown_offsets() {
        let mut committable = HashMap::new();
        let _ = committable.insert(("traces".to_string(), 0), 100);
        // partition 5 has no committable offset
        let revoked = vec![("traces".to_string(), 0), ("traces".to_string(), 5)];
        let tpl = build_commit_tpl(&committable, &revoked);
        assert_eq!(tpl.count(), 1);
    }

    #[test]
    fn drain_revoked_empty_then_populated() {
        let state = RebalanceState::new(false);
        assert!(state.drain_revoked().is_empty());

        {
            let mut q = lock_ignore_poison(&state.revoked);
            q.push(("traces".to_string(), 0));
            q.push(("traces".to_string(), 1));
        }
        let drained = state.drain_revoked();
        assert_eq!(drained.len(), 2);
        // Second drain is empty again.
        assert!(state.drain_revoked().is_empty());
    }

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
        assert_eq!(delta.partitions_assigned, 3);
    }

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
        assert_eq!(delta.partitions_assigned, 3);
    }

    #[test]
    fn metrics_delta_is_empty() {
        let state = RebalanceState::new(false);
        assert!(state.drain_metrics().is_empty());
    }

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

    #[test]
    fn commit_result_makes_delta_non_empty() {
        let state = RebalanceState::new(false);
        state.record_commit_result(&Ok(()));
        assert!(!state.drain_metrics().is_empty());
    }

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

    #[test]
    fn auto_commit_state_shared_across_arc() {
        let state = Arc::new(RebalanceState::new(true));
        let clone = Arc::clone(&state);
        assert!(clone.is_auto_commit());
    }
}
