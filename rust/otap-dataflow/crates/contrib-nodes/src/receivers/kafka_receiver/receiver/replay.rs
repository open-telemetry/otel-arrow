// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Kafka client operations and receiver orchestration for transient-NACK replay.

use super::KafkaReceiver;
use crate::receivers::kafka_receiver::retry::{BeginRetry, DueReplay};
use otel_arrow_dfe_engine::control::CallData;
use rdkafka::consumer::{Consumer, ConsumerContext, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::Timeout;
use std::time::{Duration, Instant};

/// Maximum partition replay attempts performed before returning to `select!`.
const MAX_DUE_REPLAYS_PER_TURN: usize = 8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum ReplayOperation {
    Pause,
    Seek,
    Resume,
}

impl ReplayOperation {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Seek => "seek",
            Self::Resume => "resume",
        }
    }
}

/// Narrow consumer-operation seam used by replay logic and failure-path tests.
pub(super) trait ReplayConsumerOperations {
    fn pause_partition(&self, topic: &str, partition: i32) -> Result<(), KafkaError>;
    fn seek_partition(&self, topic: &str, partition: i32, offset: i64) -> Result<(), KafkaError>;
    fn resume_partition(&self, topic: &str, partition: i32) -> Result<(), KafkaError>;
}

impl<C: ConsumerContext> ReplayConsumerOperations for StreamConsumer<C> {
    fn pause_partition(&self, topic: &str, partition: i32) -> Result<(), KafkaError> {
        self.pause(&single_partition_list(topic, partition))
    }

    fn seek_partition(&self, topic: &str, partition: i32, offset: i64) -> Result<(), KafkaError> {
        self.seek(
            topic,
            partition,
            rdkafka::Offset::Offset(offset),
            Timeout::After(Duration::ZERO),
        )
    }

    fn resume_partition(&self, topic: &str, partition: i32) -> Result<(), KafkaError> {
        self.resume(&single_partition_list(topic, partition))
    }
}

impl KafkaReceiver {
    /// Discard replay state whose partition ownership changed during an operation.
    fn discard_stale_replay(&mut self, replay: &DueReplay) -> bool {
        if self
            .rebalance_state
            .is_assigned(&replay.topic, replay.partition)
            && self
                .rebalance_state
                .current_generation(&replay.topic, replay.partition)
                == replay.ownership_generation.raw()
        {
            return false;
        }

        let _ = self.offset_tracker.revoke_if_older(
            &replay.topic,
            replay.partition,
            replay.ownership_generation.raw(),
        );
        self.retry_manager.revoke_if_older(
            &replay.topic,
            replay.partition,
            replay.ownership_generation,
        );
        self.refresh_committable_snapshot();
        true
    }

    /// Record a failed Kafka replay operation with one stable event schema.
    fn record_retry_operation_failure(
        &mut self,
        operation: ReplayOperation,
        topic: &str,
        partition: i32,
        offset: i64,
        error: &KafkaError,
    ) {
        match operation {
            ReplayOperation::Pause => self.metrics.consumer.retry_pause_failures.inc(),
            ReplayOperation::Seek => self.metrics.consumer.retry_seek_failures.inc(),
            ReplayOperation::Resume => self.metrics.consumer.retry_resume_failures.inc(),
        }
        otel_error!(
            "kafka.retry.fail",
            operation = operation.as_str(),
            topic = %topic,
            partition = partition,
            offset = offset,
            "exception.type" = "rdkafka",
            "exception.message" = %error,
        );
    }

    /// Pause a partition and schedule Kafka replay for a non-permanent NACK.
    pub(super) fn handle_transient_nack<O: ReplayConsumerOperations + ?Sized>(
        &mut self,
        calldata: &CallData,
        consumer: &O,
    ) {
        let Some(retry_config) = self.config.replay_backoff().cloned() else {
            return;
        };
        let Some(feedback) = self.resolve_offset_feedback(calldata) else {
            return;
        };
        let Some(rewind_offset) = self.offset_tracker.prepare_replay(
            &feedback.topic,
            feedback.partition,
            feedback.offset,
            feedback.ownership_generation.raw(),
        ) else {
            return;
        };

        let paused = match consumer.pause_partition(&feedback.topic, feedback.partition) {
            Ok(()) => true,
            Err(error) => {
                self.record_retry_operation_failure(
                    ReplayOperation::Pause,
                    &feedback.topic,
                    feedback.partition,
                    feedback.offset,
                    &error,
                );
                false
            }
        };

        let delivery_generation = self.retry_manager.begin_retry(
            BeginRetry {
                topic: &feedback.topic,
                partition: feedback.partition,
                ownership_generation: feedback.ownership_generation,
                failed_offset: feedback.offset,
                rewind_offset,
                paused,
                now: Instant::now(),
            },
            &retry_config,
        );
        self.refresh_committable_snapshot();
        otel_warn!(
            "kafka.retry.schedule",
            topic = %feedback.topic,
            partition = feedback.partition,
            failed_offset = feedback.offset,
            rewind_offset = rewind_offset,
            delivery_generation = delivery_generation.raw(),
        );
    }

    /// Run every pause/seek/resume replay attempt whose backoff has elapsed.
    pub(super) fn process_due_replays<O: ReplayConsumerOperations + ?Sized>(
        &mut self,
        consumer: &O,
    ) {
        let now = Instant::now();
        let Some(retry_config) = self.config.replay_backoff().cloned() else {
            return;
        };
        for replay in self
            .retry_manager
            .due_replays(now, MAX_DUE_REPLAYS_PER_TURN)
        {
            if self.discard_stale_replay(&replay) {
                continue;
            }

            let paused = if replay.paused {
                true
            } else {
                match consumer.pause_partition(&replay.topic, replay.partition) {
                    Ok(()) => true,
                    Err(error) => {
                        self.record_retry_operation_failure(
                            ReplayOperation::Pause,
                            &replay.topic,
                            replay.partition,
                            replay.rewind_offset,
                            &error,
                        );
                        self.retry_manager.reschedule_operation_failure(
                            &replay,
                            false,
                            now,
                            &retry_config,
                        );
                        continue;
                    }
                }
            };
            if self.discard_stale_replay(&replay) {
                continue;
            }

            self.metrics.consumer.replay_attempts.inc();
            if let Err(error) =
                consumer.seek_partition(&replay.topic, replay.partition, replay.rewind_offset)
            {
                self.record_retry_operation_failure(
                    ReplayOperation::Seek,
                    &replay.topic,
                    replay.partition,
                    replay.rewind_offset,
                    &error,
                );
                self.retry_manager.reschedule_operation_failure(
                    &replay,
                    paused,
                    now,
                    &retry_config,
                );
                continue;
            }
            if self.discard_stale_replay(&replay) {
                continue;
            }

            if let Err(error) = consumer.resume_partition(&replay.topic, replay.partition) {
                self.record_retry_operation_failure(
                    ReplayOperation::Resume,
                    &replay.topic,
                    replay.partition,
                    replay.rewind_offset,
                    &error,
                );
                self.retry_manager
                    .reschedule_operation_failure(&replay, true, now, &retry_config);
                continue;
            }
            if self.discard_stale_replay(&replay) {
                continue;
            }

            self.retry_manager.mark_replaying(&replay);
            otel_info!(
                "kafka.retry.start",
                topic = %replay.topic,
                partition = replay.partition,
                offset = replay.rewind_offset,
                failed_offset = replay.failed_offset,
                delivery_generation = replay.delivery_generation.raw(),
            );
        }
    }
}

/// Build a one-entry partition list for pause and resume operations.
fn single_partition_list(topic: &str, partition: i32) -> TopicPartitionList {
    let mut partitions = TopicPartitionList::new();
    let _ = partitions.add_partition(topic, partition);
    partitions
}
