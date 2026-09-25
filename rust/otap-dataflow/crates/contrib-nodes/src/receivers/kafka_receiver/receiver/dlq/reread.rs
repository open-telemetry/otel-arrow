// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// DLQ-PHASE-2 (Change): the re-read consumer is retained in port mode; it still
// recovers the original Kafka bytes for a terminal nack, but the recovered bytes
// are sent out the "dlq" port (as pdata) instead of to the in-receiver producer.

//! Dedicated idle Kafka consumer that recovers the original bytes of a
//! permanently-nacked message.
//!
//! At permanent-nack time the receiver holds only offset identity, not the
//! original payload. This consumer assigns the exact `(topic, partition)` at
//! the failed offset, polls once (bounded by a timeout), verifies the returned
//! record is the requested offset (guarding against compaction/retention), and
//! then unassigns to return to a fully idle state that issues no fetch traffic.
//!
//! It is a manual-assignment consumer with no group subscription, independent
//! of the main consumer and the transient-replay machinery, so it is safe
//! whether replay is enabled or disabled and never perturbs the main consumer's
//! position or pause state.

use super::DlqSource;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::{Message, OwnedHeaders, OwnedMessage};
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use rdkafka::util::Timeout;
use rdkafka::{ClientConfig, error::KafkaError};
use std::sync::Arc;
use std::time::Duration;

/// The recovered original bytes and headers for a dead-letter.
pub(crate) struct RereadResult {
    /// Original raw payload bytes (byte-identical to the source message).
    pub(crate) payload: Vec<u8>,
    /// Original source headers, if any.
    pub(crate) headers: Option<OwnedHeaders>,
}

/// Outcome of a re-read attempt.
pub(crate) enum RereadOutcome {
    /// The exact offset was recovered.
    Recovered(RereadResult),
    /// The offset could not be recovered (compacted, retention-deleted, out of
    /// range, timed out, or an assignment/poll error). The message must be
    /// treated as a DLQ loss and the source offset advanced.
    NotFound,
}

/// A wrapper around the dedicated re-read consumer that keeps it idle between
/// jobs and services a keep-warm poll so the broker connection does not go
/// cold.
pub(crate) struct RereadConsumer {
    consumer: Arc<BaseConsumer>,
    /// Per-job fetch deadline.
    fetch_timeout: Duration,
}

impl RereadConsumer {
    /// Build the dedicated re-read consumer. Fails when the consumer cannot be
    /// constructed, so the receiver can fail fast at startup.
    pub(crate) fn new(
        client_config: &ClientConfig,
        fetch_timeout: Duration,
    ) -> Result<Self, KafkaError> {
        let consumer: BaseConsumer = client_config.create()?;
        Ok(Self {
            consumer: Arc::new(consumer),
            fetch_timeout,
        })
    }

    /// Recover the original bytes for `source` off the receive loop.
    ///
    /// Runs the blocking assign/poll/unassign on `spawn_blocking` so the
    /// single-threaded receive loop is never blocked; a join failure is folded
    /// into [`RereadOutcome::NotFound`].
    pub(crate) async fn recover(&self, source: &DlqSource) -> RereadOutcome {
        let consumer = Arc::clone(&self.consumer);
        let fetch_timeout = self.fetch_timeout;
        let topic = Arc::clone(&source.topic);
        let partition = source.partition;
        let offset = source.offset;
        tokio::task::spawn_blocking(move || {
            reread_blocking(&consumer, &topic, partition, offset, fetch_timeout)
        })
        .await
        .unwrap_or(RereadOutcome::NotFound)
    }

    /// Service a keep-warm poll so the idle consumer stays connected and its
    /// event queue is drained. Must not have any assignment when called.
    pub(crate) fn keep_warm(&self) {
        // Zero-timeout poll: serves queued events without blocking.
        let _ = self.consumer.poll(Timeout::After(Duration::ZERO));
    }
}

/// Recover the original bytes for `(topic, partition, offset)` using a dedicated
/// consumer handle. Intended to run inside `spawn_blocking`.
///
/// Assigns the partition at the exact offset (via assign, so no separate seek is
/// needed and the just-assigned seek race is avoided), polls until the target
/// offset is observed or the deadline elapses, verifies the offset matches, and
/// always unassigns before returning so the consumer is idle again.
fn reread_blocking(
    consumer: &BaseConsumer,
    topic: &str,
    partition: i32,
    offset: i64,
    fetch_timeout: Duration,
) -> RereadOutcome {
    // Assign at the exact offset. `assign` sets the fetch start position, so the
    // next poll returns the message at `offset` without a separate seek.
    let mut tpl = TopicPartitionList::new();
    if tpl
        .add_partition_offset(topic, partition, Offset::Offset(offset))
        .is_err()
    {
        return RereadOutcome::NotFound;
    }
    if consumer.assign(&tpl).is_err() {
        let _ = consumer.unassign();
        return RereadOutcome::NotFound;
    }

    let outcome = poll_for_offset(consumer, partition, offset, fetch_timeout);

    // Always return to idle so no fetch traffic continues.
    let _ = consumer.unassign();
    outcome
}

/// Poll until the record at `offset` is observed, the deadline elapses, or the
/// broker returns a record beyond the target offset (meaning the target was
/// compacted or aged out).
fn poll_for_offset(
    consumer: &BaseConsumer,
    partition: i32,
    offset: i64,
    fetch_timeout: Duration,
) -> RereadOutcome {
    let deadline = std::time::Instant::now() + fetch_timeout;
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return RereadOutcome::NotFound;
        }
        // Poll in small slices so the loop can honor the overall deadline even
        // when the broker is slow, and so a keep-warm-style responsiveness is
        // retained.
        let slice = remaining.min(Duration::from_millis(200));
        match consumer.poll(Timeout::After(slice)) {
            Some(Ok(message)) => {
                if message.partition() != partition {
                    continue;
                }
                let msg_offset = message.offset();
                if msg_offset < offset {
                    // Older record: keep polling toward the target.
                    continue;
                }
                if msg_offset > offset {
                    // The target offset was skipped (compaction/retention): the
                    // exact original bytes are gone.
                    return RereadOutcome::NotFound;
                }
                let mut owned: OwnedMessage = message.detach();
                let payload = owned.payload().map(<[u8]>::to_vec).unwrap_or_default();
                let headers = owned.detach_headers();
                return RereadOutcome::Recovered(RereadResult { payload, headers });
            }
            // Transient consumer error (e.g. offset out of range): give up on
            // this offset and treat it as unrecoverable.
            Some(Err(_)) => return RereadOutcome::NotFound,
            // No message yet: keep polling until the deadline.
            None => continue,
        }
    }
}
