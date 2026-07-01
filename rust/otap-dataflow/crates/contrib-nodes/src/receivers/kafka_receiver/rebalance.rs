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
}

impl RebalanceMetricsDelta {
    /// Returns `true` if every counter is zero (nothing to report).
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.partitions_assigned == 0
            && self.partitions_revoked == 0
            && self.rebalance_commit_errors == 0
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

    /// Handle a `post_rebalance` assign: replace the assigned set with the new
    /// assignment.
    fn handle_assign(&self, tpl: &TopicPartitionList) {
        let assigned_partitions = topic_partitions(tpl);
        let mut assigned = self.lock_assigned();
        assigned.clear();
        for (topic, partition) in &assigned_partitions {
            assigned.add_partition(topic, *partition);
        }
        let _ = self
            .partitions_assigned
            .fetch_add(assigned_partitions.len() as u64, Ordering::Relaxed);
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

    fn post_rebalance(&self, _base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        let state = self.state();
        if state.is_auto_commit() {
            return;
        }
        match rebalance {
            Rebalance::Assign(tpl) => {
                state.handle_assign(tpl);
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
    fn handle_assign_replaces_assignment() {
        let state = RebalanceState::new(false);

        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_partition("traces", 0);
        let _ = tpl.add_partition("traces", 1);
        state.handle_assign(&tpl);

        assert!(state.is_assigned("traces", 0));
        assert!(state.is_assigned("traces", 1));

        // A new assignment replaces (clears) the old one.
        let mut tpl2 = TopicPartitionList::new();
        let _ = tpl2.add_partition("metrics", 0);
        state.handle_assign(&tpl2);

        assert!(!state.is_assigned("traces", 0));
        assert!(state.is_assigned("metrics", 0));

        let delta = state.drain_metrics();
        assert_eq!(delta.partitions_assigned, 3); // 2 + 1
    }

    #[test]
    fn metrics_delta_is_empty() {
        let state = RebalanceState::new(false);
        assert!(state.drain_metrics().is_empty());
    }

    #[test]
    fn auto_commit_state_shared_across_arc() {
        let state = Arc::new(RebalanceState::new(true));
        let clone = Arc::clone(&state);
        assert!(clone.is_auto_commit());
    }
}
