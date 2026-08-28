// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Partition-local delivery generations and transient-NACK replay state.

use super::config::TransientNackConfig;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Maximum revoked delivery identities retained for late-feedback classification.
const MAX_FEEDBACK_TOMBSTONES: usize = 1_024;

/// A topic-partition whose replay backoff has elapsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DueReplay {
    pub(crate) topic: String,
    pub(crate) partition: i32,
    pub(crate) ownership_generation: u64,
    pub(crate) delivery_generation: u64,
    pub(crate) rewind_offset: i64,
    pub(crate) failed_offset: i64,
    pub(crate) paused: bool,
}

#[derive(Debug)]
struct RetryState {
    rewind_offset: i64,
    failed_offset: i64,
    attempts: u32,
    next_attempt: Option<Instant>,
    paused: bool,
}

#[derive(Debug)]
struct PartitionState {
    ownership_generation: u64,
    delivery_generation: u64,
    retry: Option<RetryState>,
}

#[derive(Debug)]
struct FeedbackTombstone {
    topic: String,
    partition: i32,
    ownership_generation: u64,
    delivery_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ScheduledReplay {
    deadline: Instant,
    topic: String,
    partition: i32,
    delivery_generation: u64,
}

/// Tracks the current delivery generation and optional replay for each partition.
#[derive(Debug)]
pub(crate) struct RetryManager {
    partitions: HashMap<String, HashMap<i32, PartitionState>>,
    /// Ordered, one-entry-per-partition replay schedule.
    scheduled_replays: BTreeSet<ScheduledReplay>,
    /// Bounded identities for classifying feedback received after revocation.
    feedback_tombstones: VecDeque<FeedbackTombstone>,
    next_delivery_generation: u64,
    paused_partitions: usize,
}

impl RetryManager {
    /// Creates an empty retry manager.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            partitions: HashMap::new(),
            scheduled_replays: BTreeSet::new(),
            feedback_tombstones: VecDeque::with_capacity(MAX_FEEDBACK_TOMBSTONES),
            next_delivery_generation: 1,
            paused_partitions: 0,
        }
    }

    fn allocate_delivery_generation(&mut self) -> u64 {
        let generation = self.next_delivery_generation;
        self.next_delivery_generation = self.next_delivery_generation.saturating_add(1);
        generation
    }

    fn state(&self, topic: &str, partition: i32) -> Option<&PartitionState> {
        self.partitions
            .get(topic)
            .and_then(|partitions| partitions.get(&partition))
    }

    fn state_mut(&mut self, topic: &str, partition: i32) -> Option<&mut PartitionState> {
        self.partitions
            .get_mut(topic)
            .and_then(|partitions| partitions.get_mut(&partition))
    }

    fn insert_state(&mut self, topic: &str, partition: i32, state: PartitionState) {
        if let Some(partitions) = self.partitions.get_mut(topic) {
            let _ = partitions.insert(partition, state);
        } else {
            let mut partitions = HashMap::new();
            let _ = partitions.insert(partition, state);
            let _ = self.partitions.insert(topic.to_string(), partitions);
        }
    }

    fn remove_state(&mut self, topic: &str, partition: i32) -> Option<PartitionState> {
        let partitions = self.partitions.get_mut(topic)?;
        let removed = partitions.remove(&partition);
        if partitions.is_empty() {
            let _ = self.partitions.remove(topic);
        }
        removed
    }

    fn remember_feedback_tombstone(&mut self, topic: &str, partition: i32, state: &PartitionState) {
        if self.feedback_tombstones.len() == MAX_FEEDBACK_TOMBSTONES {
            let _ = self.feedback_tombstones.pop_front();
        }
        self.feedback_tombstones.push_back(FeedbackTombstone {
            topic: topic.to_string(),
            partition,
            ownership_generation: state.ownership_generation,
            delivery_generation: state.delivery_generation,
        });
    }

    fn remove_retry_indexes(&mut self, topic: &str, partition: i32, state: &PartitionState) {
        let Some(retry) = state.retry.as_ref() else {
            return;
        };
        if retry.paused {
            self.paused_partitions = self.paused_partitions.saturating_sub(1);
        }
        if let Some(deadline) = retry.next_attempt {
            let _ = self.scheduled_replays.remove(&ScheduledReplay {
                deadline,
                topic: topic.to_string(),
                partition,
                delivery_generation: state.delivery_generation,
            });
        }
    }

    fn insert_retry_indexes(&mut self, topic: &str, partition: i32, state: &PartitionState) {
        let Some(retry) = state.retry.as_ref() else {
            return;
        };
        if retry.paused {
            self.paused_partitions = self.paused_partitions.saturating_add(1);
        }
        if let Some(deadline) = retry.next_attempt {
            let _ = self.scheduled_replays.insert(ScheduledReplay {
                deadline,
                topic: topic.to_string(),
                partition,
                delivery_generation: state.delivery_generation,
            });
        }
    }

    /// Returns the current delivery generation for an owned partition.
    ///
    /// A newly observed ownership period gets a fresh delivery generation. A
    /// transient NACK also advances this generation, independently of Kafka
    /// assignment generations, so feedback from the pre-replay delivery is stale.
    pub(crate) fn delivery_generation(
        &mut self,
        topic: &str,
        partition: i32,
        ownership_generation: u64,
    ) -> u64 {
        if let Some(state) = self.state(topic, partition) {
            if state.ownership_generation == ownership_generation {
                return state.delivery_generation;
            }
        }

        if let Some(previous) = self.remove_state(topic, partition) {
            self.remove_retry_indexes(topic, partition, &previous);
            self.remember_feedback_tombstone(topic, partition, &previous);
        }

        let delivery_generation = self.allocate_delivery_generation();
        self.insert_state(
            topic,
            partition,
            PartitionState {
                ownership_generation,
                delivery_generation,
                retry: None,
            },
        );
        delivery_generation
    }

    /// Returns the ownership generation for current feedback.
    ///
    /// `None` means the feedback belongs to an obsolete delivery generation.
    #[must_use]
    pub(crate) fn feedback_ownership_generation(
        &self,
        topic: &str,
        partition: i32,
        delivery_generation: u64,
    ) -> Option<u64> {
        self.state(topic, partition)
            .filter(|state| state.delivery_generation == delivery_generation)
            .map(|state| state.ownership_generation)
            .or_else(|| {
                self.feedback_tombstones
                    .iter()
                    .rev()
                    .find(|tombstone| {
                        tombstone.topic == topic
                            && tombstone.partition == partition
                            && tombstone.delivery_generation == delivery_generation
                    })
                    .map(|tombstone| tombstone.ownership_generation)
            })
    }

    /// Starts or restarts replay backoff and advances the delivery generation.
    pub(crate) fn begin_retry(
        &mut self,
        topic: &str,
        partition: i32,
        ownership_generation: u64,
        failed_offset: i64,
        rewind_offset: i64,
        paused: bool,
        now: Instant,
        config: &TransientNackConfig,
    ) -> u64 {
        let attempts = self
            .state(topic, partition)
            .and_then(|state| state.retry.as_ref())
            .map_or(0, |retry| retry.attempts);
        let delivery_generation = self.allocate_delivery_generation();
        let next_attempt = retry_deadline(now, config, attempts);
        if let Some(previous) = self.remove_state(topic, partition) {
            self.remove_retry_indexes(topic, partition, &previous);
        }
        let state = PartitionState {
            ownership_generation,
            delivery_generation,
            retry: Some(RetryState {
                rewind_offset,
                failed_offset,
                attempts,
                next_attempt: Some(next_attempt),
                paused,
            }),
        };
        self.insert_retry_indexes(topic, partition, &state);
        self.insert_state(topic, partition, state);
        delivery_generation
    }

    /// Returns the earliest scheduled replay deadline.
    #[must_use]
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.scheduled_replays.first().map(|replay| replay.deadline)
    }

    /// Returns at most `limit` replay attempts whose deadlines have elapsed.
    #[must_use]
    pub(crate) fn due_replays(&mut self, now: Instant, limit: usize) -> Vec<DueReplay> {
        let mut due = Vec::with_capacity(limit);
        let mut examined = 0;
        while examined < limit {
            let Some(scheduled) = self.scheduled_replays.first() else {
                break;
            };
            if scheduled.deadline > now {
                break;
            }
            let scheduled = scheduled.clone();
            let _ = self.scheduled_replays.remove(&scheduled);
            examined += 1;
            let Some(state) = self.state(&scheduled.topic, scheduled.partition) else {
                continue;
            };
            if state.delivery_generation != scheduled.delivery_generation {
                continue;
            }
            let Some(retry) = state.retry.as_ref() else {
                continue;
            };
            if retry.next_attempt != Some(scheduled.deadline) {
                continue;
            }
            due.push(DueReplay {
                topic: scheduled.topic,
                partition: scheduled.partition,
                ownership_generation: state.ownership_generation,
                delivery_generation: state.delivery_generation,
                rewind_offset: retry.rewind_offset,
                failed_offset: retry.failed_offset,
                paused: retry.paused,
            });
        }
        due
    }

    /// Reschedules a failed pause, seek, or resume operation.
    pub(crate) fn reschedule_operation_failure(
        &mut self,
        replay: &DueReplay,
        paused: bool,
        now: Instant,
        config: &TransientNackConfig,
    ) {
        let Some(mut state) = self.remove_state(&replay.topic, replay.partition) else {
            return;
        };
        if state.delivery_generation != replay.delivery_generation {
            self.insert_state(&replay.topic, replay.partition, state);
            return;
        }
        self.remove_retry_indexes(&replay.topic, replay.partition, &state);
        let Some(retry) = state.retry.as_mut() else {
            self.insert_state(&replay.topic, replay.partition, state);
            return;
        };
        retry.attempts = retry.attempts.saturating_add(1);
        retry.paused = paused;
        retry.next_attempt = Some(retry_deadline(now, config, retry.attempts));
        self.insert_retry_indexes(&replay.topic, replay.partition, &state);
        self.insert_state(&replay.topic, replay.partition, state);
    }

    /// Marks a due replay as resumed and awaiting feedback.
    pub(crate) fn mark_replaying(&mut self, replay: &DueReplay) {
        let Some(mut state) = self.remove_state(&replay.topic, replay.partition) else {
            return;
        };
        if state.delivery_generation != replay.delivery_generation {
            self.insert_state(&replay.topic, replay.partition, state);
            return;
        }
        self.remove_retry_indexes(&replay.topic, replay.partition, &state);
        let Some(retry) = state.retry.as_mut() else {
            self.insert_state(&replay.topic, replay.partition, state);
            return;
        };
        retry.attempts = retry.attempts.saturating_add(1);
        retry.next_attempt = None;
        retry.paused = false;
        self.insert_state(&replay.topic, replay.partition, state);
    }

    /// Returns `true` while buffered deliveries must be discarded before seek.
    #[must_use]
    pub(crate) fn blocks_delivery(&self, topic: &str, partition: i32) -> bool {
        self.state(topic, partition)
            .and_then(|state| state.retry.as_ref())
            .is_some_and(|retry| retry.next_attempt.is_some())
    }

    /// Returns `true` for a delivery intentionally produced by a replay seek.
    #[must_use]
    pub(crate) fn is_replay_delivery(
        &self,
        topic: &str,
        partition: i32,
        delivery_generation: u64,
    ) -> bool {
        self.state(topic, partition)
            .filter(|state| state.delivery_generation == delivery_generation)
            .and_then(|state| state.retry.as_ref())
            .is_some_and(|retry| retry.next_attempt.is_none())
    }

    /// Completes replay after terminal feedback for the rewind record.
    pub(crate) fn complete_if_rewind(
        &mut self,
        topic: &str,
        partition: i32,
        delivery_generation: u64,
        offset: i64,
    ) {
        let Some(state) = self.state_mut(topic, partition) else {
            return;
        };
        if state.delivery_generation != delivery_generation {
            return;
        }
        if state
            .retry
            .as_ref()
            .is_some_and(|retry| retry.rewind_offset == offset && retry.next_attempt.is_none())
        {
            state.retry = None;
        }
    }

    /// Removes state for a revoked ownership period and retains its delivery
    /// identity in the bounded late-feedback tombstone queue.
    pub(crate) fn revoke_if_older(
        &mut self,
        topic: &str,
        partition: i32,
        ownership_generation: u64,
    ) {
        let Some(state) = self.state(topic, partition) else {
            return;
        };
        if state.ownership_generation > ownership_generation {
            return;
        }
        let Some(state) = self.remove_state(topic, partition) else {
            return;
        };
        self.remove_retry_indexes(topic, partition, &state);
        self.remember_feedback_tombstone(topic, partition, &state);
    }

    /// Number of partitions currently believed to be paused for retry.
    #[must_use]
    pub(crate) fn paused_count(&self) -> usize {
        self.paused_partitions
    }
}

impl Default for RetryManager {
    fn default() -> Self {
        Self::new()
    }
}

fn retry_backoff(config: &TransientNackConfig, attempts: u32) -> Duration {
    let multiplier = 1_u64.checked_shl(attempts.min(63)).unwrap_or(u64::MAX);
    let millis = config
        .initial_backoff_ms
        .saturating_mul(multiplier)
        .min(config.max_backoff_ms);
    Duration::from_millis(millis)
}

fn retry_deadline(now: Instant, config: &TransientNackConfig, attempts: u32) -> Instant {
    let mut delay = retry_backoff(config, attempts);
    loop {
        if let Some(deadline) = now.checked_add(delay) {
            return deadline;
        }
        delay /= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receivers::kafka_receiver::config::TransientNackMode;

    fn replay_config() -> TransientNackConfig {
        TransientNackConfig {
            mode: TransientNackMode::Replay,
            initial_backoff_ms: 10,
            max_backoff_ms: 40,
        }
    }

    /// Scenario: A transient NACK starts replay within a continuously owned partition.
    /// Guarantees: The replay delivery generation differs from the original generation.
    #[test]
    fn retry_advances_delivery_generation_without_changing_ownership() {
        let mut manager = RetryManager::new();
        let original = manager.delivery_generation("traces", 0, 7);
        let replay = manager.begin_retry(
            "traces",
            0,
            7,
            12,
            10,
            true,
            Instant::now(),
            &replay_config(),
        );

        assert_ne!(original, replay);
        assert_eq!(
            manager.feedback_ownership_generation("traces", 0, replay),
            Some(7)
        );
        assert_eq!(
            manager.feedback_ownership_generation("traces", 0, original),
            None
        );
    }

    /// Scenario: Repeated replay operation failures use exponential backoff.
    /// Guarantees: Retry deadlines grow from the initial delay and cap at the configured maximum.
    #[test]
    fn replay_operation_backoff_grows_and_caps() {
        let config = replay_config();
        let start = Instant::now();
        let mut manager = RetryManager::new();
        let _ = manager.delivery_generation("traces", 0, 1);
        let delivery = manager.begin_retry("traces", 0, 1, 5, 5, true, start, &config);

        let mut expected_delay = 10_u128;
        for _ in 0..4 {
            let due_at = manager.next_deadline().expect("deadline scheduled");
            let due = manager.due_replays(due_at, 1).pop().expect("replay is due");
            assert_eq!(due.delivery_generation, delivery);
            let now = due_at;
            manager.reschedule_operation_failure(&due, true, now, &config);
            let deadline = manager.next_deadline().expect("deadline scheduled");
            expected_delay = (expected_delay * 2).min(40);
            assert_eq!(deadline.duration_since(now).as_millis(), expected_delay);
        }
    }

    /// Scenario: A partition is revoked while it is paused in replay backoff.
    /// Guarantees: Local retry state is dropped but late feedback retains ownership classification.
    #[test]
    fn revocation_drops_retry_but_retains_feedback_tombstone() {
        let mut manager = RetryManager::new();
        let _ = manager.delivery_generation("traces", 0, 3);
        let replay =
            manager.begin_retry("traces", 0, 3, 8, 8, true, Instant::now(), &replay_config());

        manager.revoke_if_older("traces", 0, 3);

        assert_eq!(manager.paused_count(), 0);
        assert!(manager.next_deadline().is_none());
        assert_eq!(
            manager.feedback_ownership_generation("traces", 0, replay),
            Some(3)
        );
    }

    /// Scenario: The replayed rewind record receives terminal feedback.
    /// Guarantees: Replay state clears only for that record in the current delivery generation.
    #[test]
    fn only_current_rewind_feedback_completes_replay() {
        let mut manager = RetryManager::new();
        let original = manager.delivery_generation("traces", 0, 1);
        let replay = manager.begin_retry(
            "traces",
            0,
            1,
            11,
            10,
            true,
            Instant::now(),
            &replay_config(),
        );
        let due = manager
            .due_replays(Instant::now() + Duration::from_secs(1), 1)
            .pop()
            .expect("replay due");
        manager.mark_replaying(&due);

        manager.complete_if_rewind("traces", 0, original, 10);
        manager.complete_if_rewind("traces", 0, replay, 11);
        assert!(manager.is_replay_delivery("traces", 0, replay));

        manager.complete_if_rewind("traces", 0, replay, 10);
        assert!(!manager.is_replay_delivery("traces", 0, replay));
    }

    /// Scenario: One partition enters replay backoff while a sibling partition remains healthy.
    /// Guarantees: Delivery blocking and pause accounting are isolated to the failed partition.
    #[test]
    fn replay_state_is_partition_local() {
        let mut manager = RetryManager::new();
        let _ = manager.delivery_generation("traces", 0, 1);
        let healthy_generation = manager.delivery_generation("traces", 1, 1);
        let _ = manager.begin_retry("traces", 0, 1, 4, 4, true, Instant::now(), &replay_config());

        assert!(manager.blocks_delivery("traces", 0));
        assert!(!manager.blocks_delivery("traces", 1));
        assert_eq!(manager.paused_count(), 1);
        assert_eq!(
            manager.feedback_ownership_generation("traces", 1, healthy_generation),
            Some(1)
        );
    }

    /// Scenario: A broker operation fails while a transiently NACKed partition is recovering.
    /// Guarantees: The partition remains blocked and unresumed with another bounded retry scheduled.
    #[test]
    fn replay_operation_failure_keeps_partition_blocked() {
        let config = replay_config();
        let start = Instant::now();
        let mut manager = RetryManager::new();
        let _ = manager.delivery_generation("traces", 0, 1);
        let _ = manager.begin_retry("traces", 0, 1, 5, 5, true, start, &config);
        let due_at = manager.next_deadline().expect("deadline scheduled");
        let replay = manager.due_replays(due_at, 1).pop().expect("replay is due");

        manager.reschedule_operation_failure(&replay, true, due_at, &config);

        assert!(manager.blocks_delivery("traces", 0));
        assert_eq!(manager.paused_count(), 1);
        assert!(manager.next_deadline().is_some_and(|next| next > due_at));
    }

    /// Scenario: An operator configures the largest representable millisecond backoff.
    /// Guarantees: Scheduling remains panic-free and chooses a representable future deadline.
    #[test]
    fn extreme_backoff_schedules_without_instant_overflow() {
        let config = TransientNackConfig {
            mode: TransientNackMode::Replay,
            initial_backoff_ms: u64::MAX,
            max_backoff_ms: u64::MAX,
        };
        let start = Instant::now();
        let mut manager = RetryManager::new();
        let _ = manager.delivery_generation("traces", 0, 1);

        let _ = manager.begin_retry("traces", 0, 1, 5, 5, true, start, &config);

        assert!(
            manager
                .next_deadline()
                .is_some_and(|deadline| deadline > start)
        );
    }

    /// Scenario: More replay deadlines are due than one receive-loop turn may process.
    /// Guarantees: Due replay extraction respects its work limit and leaves another due deadline cached.
    #[test]
    fn due_replays_are_bounded_per_turn() {
        let config = replay_config();
        let start = Instant::now();
        let mut manager = RetryManager::new();
        for partition in 0..5 {
            let _ = manager.delivery_generation("traces", partition, 1);
            let _ = manager.begin_retry("traces", partition, 1, 5, 5, true, start, &config);
        }
        let due_at = manager.next_deadline().expect("deadline scheduled");

        assert_eq!(manager.due_replays(due_at, 2).len(), 2);
        assert!(
            manager
                .next_deadline()
                .is_some_and(|deadline| deadline <= due_at)
        );
    }

    /// Scenario: Revoked partitions exceed the retained late-feedback history.
    /// Guarantees: Active state is reclaimed and the tombstone queue remains at its explicit bound.
    #[test]
    fn revoked_feedback_tombstones_are_bounded() {
        let mut manager = RetryManager::new();
        let mut oldest_delivery = 0;
        let mut newest_delivery = 0;
        for partition in 0..=(MAX_FEEDBACK_TOMBSTONES as i32) {
            let delivery = manager.delivery_generation("traces", partition, 1);
            if partition == 0 {
                oldest_delivery = delivery;
            }
            newest_delivery = delivery;
            manager.revoke_if_older("traces", partition, 1);
        }

        assert!(manager.partitions.is_empty());
        assert!(manager.scheduled_replays.is_empty());
        assert_eq!(manager.feedback_tombstones.len(), MAX_FEEDBACK_TOMBSTONES);
        assert_eq!(
            manager.feedback_ownership_generation("traces", 0, oldest_delivery),
            None,
        );
        assert_eq!(
            manager.feedback_ownership_generation(
                "traces",
                MAX_FEEDBACK_TOMBSTONES as i32,
                newest_delivery,
            ),
            Some(1),
        );
    }
}
