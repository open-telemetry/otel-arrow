// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ack/Nack resolution, stale/late-ack classification, and offset commit.
//!
//! Resolves downstream Ack/Nack feedback (carrying Kafka offset identity in its
//! [`CallData`]) against the partition's ownership and delivery generations,
//! then advances the offset tracker and commits owned partitions. The
//! [`classify_offset_feedback`] policy is pure and exhaustively unit-testable
//! without a live consumer.

use super::super::identity::{DeliveryGeneration, OwnershipGeneration};
use super::KafkaReceiver;
use super::decode::decode_calldata;
use otel_arrow_dfe_engine::control::CallData;
use otel_arrow_dfe_engine::error::{Error as EngineError, ReceiverErrorKind, format_error_sources};
use otel_arrow_dfe_engine::node::NodeId;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext};
use std::sync::Arc;

/// Feedback identity resolved from receiver calldata and retry state.
pub(super) struct ResolvedOffsetFeedback {
    pub(super) topic: Arc<str>,
    pub(super) partition: i32,
    pub(super) offset: i64,
    pub(super) ownership_generation: OwnershipGeneration,
    pub(super) delivery_generation: DeliveryGeneration,
}

/// Decision for incoming feedback carrying Kafka offset identity, derived
/// purely from generation/ownership state.
///
/// Extracted from [`KafkaReceiver::resolve_offset_feedback`] so the stale/late-ack
/// policy is self-contained and exhaustively unit-testable without a live
/// consumer.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum OffsetFeedbackAction {
    /// Advance the offset tracker and commit: the ack belongs to the current
    /// ownership period of a currently-owned partition.
    Commit,
    /// Drop as stale: the ack is from an ownership period strictly older than
    /// the partition's current tracked *or* currently-assigned generation. The
    /// partition was revoked and reassigned since the record was delivered.
    DropStale,
    /// Drop as a late ack: the partition is no longer assigned to this consumer.
    /// `purge` indicates whether lingering tracker state should also be removed
    /// (only when that state is not newer than the ack's ownership period).
    DropLateAck { purge: bool },
}

/// Classify feedback given its ownership `generation` and the
/// partition's current tracker/assignment state.
///
/// The stale-generation check compares the ack against the **maximum** of the
/// tracker generation and the currently-assigned generation. Consulting the
/// assigned generation (not just the tracker's) closes the window where a
/// partition was revoked and reassigned to this consumer under a newer
/// generation but no record of the new period has been tracked yet: in that
/// window the tracker still reports the old generation, so an ack that equals
/// the tracker generation would otherwise pass the guard, find the partition
/// assigned, and mutate/commit stale state. Because real generations start at
/// `1`, a `0` assigned/tracked generation means "not owned / untracked" and is
/// treated as no lower bound.
pub(super) fn classify_offset_feedback(
    feedback_generation: u64,
    tracked_generation: Option<u64>,
    assigned_generation: u64,
    is_assigned: bool,
) -> OffsetFeedbackAction {
    let current = tracked_generation.unwrap_or(0).max(assigned_generation);
    if current > 0 && feedback_generation < current {
        return OffsetFeedbackAction::DropStale;
    }
    if !is_assigned {
        let purge = tracked_generation.is_some_and(|tracked| tracked <= feedback_generation);
        return OffsetFeedbackAction::DropLateAck { purge };
    }
    OffsetFeedbackAction::Commit
}

impl KafkaReceiver {
    /// Commit the current committable offsets to Kafka.
    ///
    /// Updates the offset tracker's internal [`TopicPartitionList`] in-place and
    /// commits **asynchronously**: [`CommitMode::Async`] enqueues the request in
    /// librdkafka's local work queue and returns immediately, so the pipeline's
    /// single-thread runtime never blocks on a broker round-trip. This method is
    /// on the hot ACK/NACK path, so it must not stall data processing, control
    /// messages, telemetry, or shutdown.
    ///
    /// Because the commit is async, the returned `Ok(())` only means the request
    /// was enqueued -- not that the broker accepted it. The eventual broker
    /// outcome is observed via
    /// [`RebalancingConsumerContext::commit_callback`](super::rebalance::RebalancingConsumerContext),
    /// which folds success/failure counts into
    /// `receiver.kafka.offset_commits` with `outcome=success` or `outcome=failure`
    /// via the shared rebalance state. A rare *enqueue* failure is returned here so
    /// callers can log it; the offsets stay tracked and are retried on the next
    /// ack/nack/timer-tick.
    ///
    /// Only commits when auto-commit is disabled.
    pub(super) fn commit_offsets<C: ConsumerContext>(
        &mut self,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
    ) -> Result<(), EngineError> {
        if self.config.is_auto_commit() {
            return Ok(());
        }
        // Drop any partitions revoked by the rebalance callback since the last
        // reconcile *before* building the commit list, so we never commit an
        // offset for a partition this consumer no longer owns.
        self.purge_revoked_partitions();
        let tpl = self.offset_tracker.committable_tpl();
        if tpl.count() == 0 {
            return Ok(());
        }
        // Enqueue asynchronously; the broker result arrives later on
        // `commit_callback`, which is the single source of truth for commit
        // success/failure metrics (avoids double counting).
        match consumer.commit(tpl, CommitMode::Async) {
            Ok(()) => Ok(()),
            Err(e) => {
                let source_detail = format_error_sources(&e);
                Err(EngineError::ReceiverError {
                    receiver: receiver_id.clone(),
                    kind: ReceiverErrorKind::Transport,
                    error: e.to_string(),
                    source_detail,
                })
            }
        }
    }

    /// Advance the offset tracker for a terminally processed message and, if the
    /// committable watermark moved, commit and refresh the rebalance snapshot.
    ///
    /// This is the single place that persists forward progress past a message
    /// (whether it was acked, permanently nacked, or a poison pill). Commit
    /// failures are recoverable: the offset stays tracked and is retried on the
    /// next terminal feedback or timer tick.
    ///
    /// Caller must ensure manual-commit mode.
    pub(super) fn advance_offset_and_commit<C: ConsumerContext>(
        &mut self,
        topic: &str,
        partition: i32,
        offset: i64,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
    ) {
        if self.offset_tracker.acknowledge(topic, partition, offset) {
            if let Err(e) = self.commit_offsets(consumer, receiver_id) {
                otel_error!(
                    "kafka.commit.failed",
                    error = %e,
                );
            }
            // The committable watermark moved; keep the rebalance snapshot
            // fresh for a potential pre-rebalance commit.
            self.refresh_committable_snapshot();
        }
    }

    /// Resolve feedback carrying Kafka offset identity in its `CallData`.
    ///
    /// Feedback is accepted only when both its delivery generation and Kafka
    /// ownership generation are current. This protects replay from feedback
    /// already in flight when the partition was paused and protects rebalances
    /// from feedback produced by a previous owner.
    ///
    /// Caller must ensure manual-commit mode and a non-empty `calldata`.
    pub(super) fn resolve_offset_feedback(
        &mut self,
        calldata: &CallData,
    ) -> Option<ResolvedOffsetFeedback> {
        let (topic_id, partition, offset, delivery_generation) = decode_calldata(calldata);
        // Resolve the dynamic topic ID back to the actual topic name. The
        // `Arc<str>` is an owned handle, so it does not borrow `self` and can
        // coexist with the `&mut self` calls below.
        let name = self.topic_registry.name_for(topic_id)?;

        let Some(ownership_generation) =
            self.retry_manager
                .feedback_ownership_generation(&name, partition, delivery_generation)
        else {
            self.metrics.consumer.stale_retry_feedback.inc();
            return None;
        };

        // Read the partition's tracked generation, its currently-assigned
        // generation, and whether it is still owned. The assigned generation is
        // consulted (not just the tracker's) so a stale ack is rejected even in
        // the window after a revoke/reassign where the tracker still reports the
        // old generation because no record of the new period has been tracked
        // yet. The `is_assigned` membership check remains explicit for clarity.
        //
        // The late-ack path is safe because `post_rebalance(Assign)` is served
        // inline by `consumer.recv()` (so on the pipeline thread) and completes
        // *before* that same `recv()` call yields messages for the newly
        // assigned partitions, so `assigned` is always populated before any ack
        // for those partitions can return.
        let tracked_generation = self.offset_tracker.partition_generation(&name, partition);
        let assigned_generation = self.rebalance_state.current_generation(&name, partition);
        let is_assigned = self.rebalance_state.is_assigned(&name, partition);

        match classify_offset_feedback(
            ownership_generation.raw(),
            tracked_generation,
            assigned_generation,
            is_assigned,
        ) {
            OffsetFeedbackAction::Commit => Some(ResolvedOffsetFeedback {
                topic: name,
                partition,
                offset,
                ownership_generation,
                delivery_generation,
            }),
            OffsetFeedbackAction::DropStale => {
                self.metrics.consumer.feedback_after_revocation.inc();
                None
            }
            OffsetFeedbackAction::DropLateAck { purge } => {
                self.metrics.consumer.feedback_after_revocation.inc();
                if purge {
                    self.offset_tracker.revoke(&name, partition);
                    self.retry_manager
                        .revoke_if_older(&name, partition, ownership_generation);
                    self.refresh_committable_snapshot();
                }
                None
            }
        }
    }

    /// Apply terminal feedback (ACK, permanent NACK, or commit-and-skip NACK).
    pub(super) fn handle_terminal_offset_feedback<C: ConsumerContext>(
        &mut self,
        calldata: &CallData,
        consumer: &StreamConsumer<C>,
        receiver_id: &NodeId,
    ) {
        let Some(feedback) = self.resolve_offset_feedback(calldata) else {
            return;
        };
        self.advance_offset_and_commit(
            &feedback.topic,
            feedback.partition,
            feedback.offset,
            consumer,
            receiver_id,
        );
        self.retry_manager.complete_if_rewind(
            &feedback.topic,
            feedback.partition,
            feedback.delivery_generation,
            feedback.offset,
        );
    }
}
