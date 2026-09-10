// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Assignment-resume retry subsystem.
//!
//! During a rebalance, librdkafka pause state is client-local and survives
//! reassignment. If the immediate `resume` at assign time fails, a partition
//! could stay paused indefinitely. This module records failed resumes and lets
//! the receive loop retry them with capped exponential backoff until the resume
//! succeeds or the partition's ownership generation changes.

use super::super::scheduling::{DeadlineIndex, capped_exponential_backoff, checked_deadline};
use rdkafka::consumer::{BaseConsumer, Consumer, ConsumerContext};
use rdkafka::topic_partition_list::TopicPartitionList;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Delay before retrying a partition resume that failed during assignment.
const ASSIGNMENT_RESUME_INITIAL_BACKOFF: Duration = Duration::from_millis(100);
/// Maximum delay between assignment-resume attempts.
const ASSIGNMENT_RESUME_MAX_BACKOFF: Duration = Duration::from_secs(5);
/// Maximum assignment-resume attempts performed by one receive-loop turn.
pub(super) const MAX_DUE_ASSIGNMENT_RESUMES_PER_TURN: usize = 8;

type PartitionKey = (String, i32);

/// Consumer operation used by rebalance-time resume handling and its tests.
pub(crate) trait PartitionResumeOperations {
    /// Resume every partition in `tpl`.
     fn resume_partitions(
         &self,
         tpl: &TopicPartitionList,
     ) -> Result<(), rdkafka::error::KafkaError>;
}

impl<C: ConsumerContext> PartitionResumeOperations for BaseConsumer<C> {
    fn resume_partitions(
        &self,
        tpl: &TopicPartitionList,
    ) -> Result<(), rdkafka::error::KafkaError> {
        self.resume(tpl)
    }
}

impl<C: ConsumerContext> PartitionResumeOperations for rdkafka::consumer::StreamConsumer<C> {
    fn resume_partitions(
        &self,
        tpl: &TopicPartitionList,
    ) -> Result<(), rdkafka::error::KafkaError> {
        self.resume(tpl)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResumePhase {
    Assign,
    Revoke,
}

impl ResumePhase {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Assign => "assign",
            Self::Revoke => "revoke",
        }
    }
}

#[derive(Debug)]
struct AssignmentResumeRetryState {
    generation: u64,
    failures: u32,
    deadline: Option<Instant>,
    version: u64,
}

#[derive(Debug)]
pub(super) struct DueAssignmentResume {
    pub(super) topic: String,
    pub(super) partition: i32,
    pub(super) generation: u64,
    pub(super) failures: u32,
    version: u64,
}

/// Deduplicated, deadline-ordered retry state for assignment resume failures.
///
/// Entries are bounded by the consumer's assigned partitions. `version`
/// prevents an in-flight receive-loop attempt from overwriting a newer callback
/// update for the same partition. The ordered deadline index is provided by the
/// shared [`DeadlineIndex`]; this type keeps only the per-partition payload
/// (generation, failure count, live version) and its own state machine.
#[derive(Debug, Default)]
pub(super) struct AssignmentResumeRetries {
    entries: HashMap<PartitionKey, AssignmentResumeRetryState>,
    deadlines: DeadlineIndex<PartitionKey, u64>,
    next_version: u64,
}

impl AssignmentResumeRetries {
    pub(super) fn schedule(
        &mut self,
        topic: String,
        partition: i32,
        generation: u64,
        failures: u32,
        now: Instant,
    ) {
        let key = (topic, partition);
        self.remove(&key);
        self.next_version = self.next_version.wrapping_add(1);
        let version = self.next_version;
        let deadline = checked_deadline(now, assignment_resume_backoff(failures));
        self.deadlines.insert(key.clone(), version, deadline);
        let _ = self.entries.insert(
            key,
            AssignmentResumeRetryState {
                generation,
                failures,
                deadline: Some(deadline),
                version,
            },
        );
    }

    pub(super) fn remove(&mut self, key: &PartitionKey) {
        if let Some(state) = self.entries.remove(key)
            && let Some(deadline) = state.deadline
        {
            let _ = self.deadlines.remove(key, state.version, deadline);
        }
    }

    pub(super) fn next_deadline(&self) -> Option<Instant> {
        self.deadlines.next_deadline()
    }

    pub(super) fn take_due(&mut self, now: Instant, limit: usize) -> Vec<DueAssignmentResume> {
        let mut due = Vec::with_capacity(limit);
        for entry in self.deadlines.take_due(now, limit) {
            let Some(state) = self.entries.get_mut(&entry.key) else {
                continue;
            };
            // Re-validate against the caller-owned payload: skip an entry whose
            // version or deadline no longer matches (it was rescheduled or the
            // partition dropped after this deadline was inserted).
            if state.version != entry.token || state.deadline != Some(entry.deadline) {
                continue;
            }
            state.deadline = None;
            due.push(DueAssignmentResume {
                topic: entry.key.0,
                partition: entry.key.1,
                generation: state.generation,
                failures: state.failures,
                version: entry.token,
            });
        }
        due
    }

    pub(super) fn complete(&mut self, retry: &DueAssignmentResume) {
        let key = (retry.topic.clone(), retry.partition);
        if self
            .entries
            .get(&key)
            .is_some_and(|state| state.version == retry.version)
        {
            self.remove(&key);
        }
    }

    pub(super) fn reschedule(&mut self, retry: &DueAssignmentResume, now: Instant) {
        let key = (retry.topic.clone(), retry.partition);
        let Some(state) = self.entries.get_mut(&key) else {
            return;
        };
        if state.version != retry.version || state.deadline.is_some() {
            return;
        }
        state.failures = retry.failures.saturating_add(1);
        let deadline = checked_deadline(now, assignment_resume_backoff(state.failures));
        state.deadline = Some(deadline);
        self.deadlines.insert(key, state.version, deadline);
    }
}

fn assignment_resume_backoff(failures: u32) -> Duration {
    // `failures` counts attempts made; the first backoff (failures == 1) is the
    // initial delay, so the exponent is `failures - 1`.
    let attempts = failures.saturating_sub(1);
    capped_exponential_backoff(
        ASSIGNMENT_RESUME_INITIAL_BACKOFF,
        ASSIGNMENT_RESUME_MAX_BACKOFF,
        attempts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a partition resume failure is scheduled, becomes due, and is
    /// drained.
    /// Guarantees: schedule records a deadline that next_deadline reports and
    /// take_due returns once it elapses, carrying the generation and failure
    /// count the caller needs to retry the resume.
    #[test]
    fn schedule_then_take_due_returns_the_partition() {
        let now = Instant::now();
        let mut retries = AssignmentResumeRetries::default();
        retries.schedule("t".to_string(), 0, 7, 1, now);

        let deadline = retries.next_deadline().expect("a deadline was scheduled");
        assert_eq!(deadline, now + ASSIGNMENT_RESUME_INITIAL_BACKOFF);

        // Not yet due.
        assert!(retries.take_due(now, 8).is_empty());

        let due = retries.take_due(deadline, 8);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].topic, "t");
        assert_eq!(due[0].partition, 0);
        assert_eq!(due[0].generation, 7);
        assert_eq!(due[0].failures, 1);
    }

    /// Scenario: a due resume attempt fails again and is rescheduled.
    /// Guarantees: reschedule grows the failure count and pushes the next
    /// deadline out by the capped exponential backoff, so repeated resume
    /// failures back off instead of hot-looping.
    #[test]
    fn reschedule_grows_backoff_after_repeated_failure() {
        let now = Instant::now();
        let mut retries = AssignmentResumeRetries::default();
        retries.schedule("t".to_string(), 0, 1, 1, now);
        let due = retries.take_due(retries.next_deadline().unwrap(), 8);
        assert_eq!(due.len(), 1);

        retries.reschedule(&due[0], now);
        // failures went 1 -> 2, so the delay is initial * 2^(2-1).
        let expected = now + ASSIGNMENT_RESUME_INITIAL_BACKOFF * 2;
        assert_eq!(retries.next_deadline(), Some(expected));
    }

    /// Scenario: a resume is scheduled with an extreme failure count.
    /// Guarantees: the backoff is clamped to the max (via the shared
    /// capped_exponential_backoff) and scheduling routes `now + backoff` through
    /// the overflow-safe checked_deadline, so it records a valid deadline
    /// without panicking -- closing the divergence with the replay scheduler,
    /// which was already overflow-safe while assignment-resume previously used a
    /// plain `now + backoff`. (The Instant-max overflow path itself is exercised
    /// directly in `scheduling::tests`.)
    #[test]
    fn schedule_with_extreme_failures_clamps_and_does_not_panic() {
        let now = Instant::now();
        let mut retries = AssignmentResumeRetries::default();
        retries.schedule("t".to_string(), 0, 1, u32::MAX, now);
        let deadline = retries.next_deadline().expect("a deadline was scheduled");
        // At max failures the delay is clamped to the max backoff.
        assert_eq!(deadline, now + ASSIGNMENT_RESUME_MAX_BACKOFF);
    }
}
