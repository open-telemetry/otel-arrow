// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// DLQ-PHASE-2 (Remove): entire file; the "dlq" output port plus a downstream
// exporter replaces the in-receiver producer.

//! Low-level DLQ producer wrapper.
//!
//! Wraps the shared, low-CPU [`ExporterFutureProducer`] port so the receiver
//! can produce dead-letter records with per-delivery futures and a bounded set
//! of outstanding sends. This module is intentionally the single seam that a
//! future out-port implementation would replace: the receiver interacts with
//! DLQ egress only through [`DlqProducer`] and the delivery outcome it yields,
//! which mirrors the ack/nack an out-port would eventually deliver.

use super::super::super::config::DLQ_OP_TIMEOUT_MS;
use crate::exporters::kafka_exporter::producer::{ExporterFutureProducer, ExporterFutureRecord};
use rdkafka::ClientConfig;
use rdkafka::client::DefaultClientContext;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use std::time::Duration;
use tokio::time::timeout;

/// A record ready to be produced to the DLQ.
pub(crate) struct DlqRecord {
    /// Destination DLQ topic.
    pub(crate) topic: String,
    /// Original raw payload bytes (byte-identical to the source message).
    pub(crate) payload: Vec<u8>,
    /// Fully-built DLQ headers (context + source passthrough).
    pub(crate) headers: OwnedHeaders,
}

/// Outcome of a single DLQ produce attempt.
#[derive(Debug)]
pub(crate) enum DlqSendOutcome {
    /// The record was accepted by the broker (delivered).
    Produced,
    /// The produce failed or timed out. `permanent` marks record-level errors
    /// that would never succeed on retry (informational; the receiver treats
    /// all failures as loss + advance, since retries are owned by replay).
    Failed { permanent: bool },
}

/// Thin wrapper over the shared future producer used exclusively for the DLQ.
pub(crate) struct DlqProducer {
    producer: ExporterFutureProducer<DefaultClientContext>,
    /// Fixed per-send delivery deadline ([`DLQ_OP_TIMEOUT_MS`]), independent of
    /// librdkafka's own `message.timeout.ms`.
    send_timeout: Duration,
}

impl DlqProducer {
    /// Build a DLQ producer from a resolved client config.
    ///
    /// The producer sets no tuning, so librdkafka's own defaults apply. The
    /// per-send wait is bounded by [`DLQ_OP_TIMEOUT_MS`] so a stalled broker
    /// never holds a DLQ delivery (and its source offset) indefinitely.
    ///
    /// Fails when the producer cannot be constructed (e.g. an unreachable or
    /// misconfigured DLQ connection), so the receiver can fail fast at startup.
    pub(crate) fn new(client_config: &ClientConfig) -> Result<Self, KafkaError> {
        let producer: ExporterFutureProducer<DefaultClientContext> = client_config.create()?;
        Ok(Self {
            producer,
            send_timeout: Duration::from_millis(DLQ_OP_TIMEOUT_MS),
        })
    }

    /// Produce one DLQ record, awaiting its delivery outcome (bounded by the
    /// send timeout). This is `async` and runs on the receiver's single-thread
    /// runtime; the underlying producer polls on its own background thread, so
    /// awaiting here does not block librdkafka.
    pub(crate) async fn produce(&self, record: DlqRecord) -> DlqSendOutcome {
        let future_record = ExporterFutureRecord::<[u8], [u8]>::to(&record.topic)
            .payload(&record.payload)
            .headers(record.headers.clone());

        let delivery = match self.producer.send_result(future_record) {
            Ok(delivery) => delivery,
            Err((kafka_err, _record)) => {
                return DlqSendOutcome::Failed {
                    permanent: is_permanent_send_error(&kafka_err),
                };
            }
        };

        match timeout(self.send_timeout, delivery).await {
            // Delivered successfully.
            Ok(Ok(Ok(_delivery))) => DlqSendOutcome::Produced,
            // Broker rejected the delivery.
            Ok(Ok(Err((kafka_err, _msg)))) => DlqSendOutcome::Failed {
                permanent: is_permanent_send_error(&kafka_err),
            },
            // Delivery future canceled (producer purged/dropped): transient.
            Ok(Err(_canceled)) => DlqSendOutcome::Failed { permanent: false },
            // Timed out waiting for the broker: transient.
            Err(_elapsed) => DlqSendOutcome::Failed { permanent: false },
        }
    }
}

/// Classify a produce error as a record-level (permanent) failure that would
/// never succeed on retry, versus a transient/environmental failure. Used only
/// to annotate telemetry; the DLQ never retries.
fn is_permanent_send_error(err: &KafkaError) -> bool {
    use rdkafka::types::RDKafkaErrorCode;
    let Some(code) = err.rdkafka_error_code() else {
        return false;
    };
    matches!(
        code,
        RDKafkaErrorCode::MessageSizeTooLarge
            | RDKafkaErrorCode::InvalidMessageSize
            | RDKafkaErrorCode::MessageBatchTooLarge
            | RDKafkaErrorCode::InvalidMessage
            | RDKafkaErrorCode::InvalidRecord
            | RDKafkaErrorCode::InvalidRequiredAcks
            | RDKafkaErrorCode::UnsupportedVersion
            | RDKafkaErrorCode::UnsupportedForMessageFormat
            | RDKafkaErrorCode::InvalidArgument
            | RDKafkaErrorCode::UnknownTopicOrPartition
            | RDKafkaErrorCode::UnknownTopic
    )
}

/// Unused re-export sink to keep `OwnedDeliveryResult` documented in scope for
/// readers of this module.
#[allow(dead_code)]
type _DeliveryResult = OwnedDeliveryResult;
