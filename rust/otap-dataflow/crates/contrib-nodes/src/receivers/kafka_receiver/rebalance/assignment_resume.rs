// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Assignment-resume retry subsystem.
//!
//! During a rebalance, librdkafka pause state is client-local and survives
//! reassignment. If the immediate `resume` at assign time fails, a partition
//! could stay paused indefinitely. This module records failed resumes and lets
//! the receive loop retry them with capped exponential backoff until the resume
//! succeeds or the partition's ownership generation changes.

use rdkafka::consumer::{BaseConsumer, Consumer, ConsumerContext};
use rdkafka::topic_partition_list::TopicPartitionList;
use std::collections::{BTreeSet, HashMap};
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
    fn resume_partitions(&self, tpl: &TopicPartitionList)
    -> Result<(), rdkafka::error::KafkaError>;
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ScheduledAssignmentResume {
    deadline: Instant,
    version: u64,
    topic: String,
    partition: i32,
}

/// Deduplicated, deadline-ordered retry state for assignment resume failures.
///
/// Entries are bounded by the consumer's assigned partitions. `version`
/// prevents an in-flight receive-loop attempt from overwriting a newer callback
/// update for the same partition.
#[derive(Debug, Default)]
pub(super) struct AssignmentResumeRetries {
    entries: HashMap<PartitionKey, AssignmentResumeRetryState>,
    deadlines: BTreeSet<ScheduledAssignmentResume>,
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
        let key = (topic.clone(), partition);
        self.remove(&key);
        self.next_version = self.next_version.wrapping_add(1);
        let version = self.next_version;
        let deadline = now + assignment_resume_backoff(failures);
        let _ = self.deadlines.insert(ScheduledAssignmentResume {
            deadline,
            version,
            topic,
            partition,
        });
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
            let _ = self.deadlines.remove(&ScheduledAssignmentResume {
                deadline,
                version: state.version,
                topic: key.0.clone(),
                partition: key.1,
            });
        }
    }

    pub(super) fn next_deadline(&self) -> Option<Instant> {
        self.deadlines.first().map(|entry| entry.deadline)
    }

    pub(super) fn take_due(&mut self, now: Instant, limit: usize) -> Vec<DueAssignmentResume> {
        let mut due = Vec::with_capacity(limit);
        while due.len() < limit {
            let Some(scheduled) = self.deadlines.first() else {
                break;
            };
            if scheduled.deadline > now {
                break;
            }
            let Some(scheduled) = self.deadlines.pop_first() else {
                break;
            };
            let key = (scheduled.topic.clone(), scheduled.partition);
            let Some(state) = self.entries.get_mut(&key) else {
                continue;
            };
            if state.version != scheduled.version || state.deadline != Some(scheduled.deadline) {
                continue;
            }
            state.deadline = None;
            due.push(DueAssignmentResume {
                topic: scheduled.topic,
                partition: scheduled.partition,
                generation: state.generation,
                failures: state.failures,
                version: scheduled.version,
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
        let deadline = now + assignment_resume_backoff(state.failures);
        state.deadline = Some(deadline);
        let _ = self.deadlines.insert(ScheduledAssignmentResume {
            deadline,
            version: state.version,
            topic: retry.topic.clone(),
            partition: retry.partition,
        });
    }
}

fn assignment_resume_backoff(failures: u32) -> Duration {
    let exponent = failures.saturating_sub(1).min(31);
    ASSIGNMENT_RESUME_INITIAL_BACKOFF
        .saturating_mul(1_u32 << exponent)
        .min(ASSIGNMENT_RESUME_MAX_BACKOFF)
}
