// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dead-letter-queue (DLQ) egress for the Kafka receiver.
//!
//! When a consumed message cannot be processed -- it fails to decode, targets
//! an unknown topic, or is permanently rejected downstream -- the receiver can
//! forward the original message bytes (plus `dlq.*` context headers) to a
//! user-configured Kafka topic instead of silently dropping it. The source
//! offset advances only after the DLQ delivery is confirmed.
//!
//! # Non-stall contract
//!
//! All DLQ broker I/O runs off the receive loop and is timeout-bounded: the
//! producer polls on its own background thread and each produce awaits a bounded
//! delivery future; the permanent-nack re-read runs on `spawn_blocking` with a
//! fetch timeout. Outstanding deliveries are bounded by [`DLQ_MAX_IN_FLIGHT`];
//! when that bound is reached an incoming failure is immediately reported as a
//! loss (no queue). On any failure -- produce error, timeout, unrecoverable
//! bytes, or in-flight full -- the message is counted as `dlq.loss` and the
//! source offset is advanced, so ingestion never stalls.
//!
//! # Swap seam
//!
//! [`DlqManager`] is the single boundary a future out-port implementation would
//! replace. The completion it yields ([`DlqCompletion`]) carries exactly the
//! offset identity needed to advance the source offset -- the same contract an
//! engine ack/nack would satisfy when the producer is replaced by a named
//! output port.

mod headers;
mod producer;
mod reread;

use self::headers::{DlqHeaderContext, build_dlq_headers};
use self::producer::{DlqProducer, DlqRecord, DlqSendOutcome};
use self::reread::{RereadConsumer, RereadOutcome};
use super::super::config::{
    DLQ_MAX_IN_FLIGHT, DLQ_OP_TIMEOUT_MS, KafkaReceiverConfig, ResolvedDlqConfig,
};
use futures::stream::{FuturesUnordered, StreamExt};
use otel_arrow_dfe_config::SignalType;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedHeaders;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// The failure category that triggered a dead-letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DlqReason {
    /// The payload could not be decoded.
    Decode,
    /// The topic did not map to a configured signal.
    UnknownTopic,
    /// The message was permanently rejected downstream.
    PermanentNack,
}

impl DlqReason {
    /// The wire string used in the `dlq.reason` header.
    fn as_str(self) -> &'static str {
        match self {
            DlqReason::Decode => "decode",
            DlqReason::UnknownTopic => "unknown_topic",
            DlqReason::PermanentNack => "permanent_nack",
        }
    }
}

/// Original message identity needed to advance the source offset after a DLQ
/// outcome is known.
#[derive(Debug, Clone)]
pub(crate) struct DlqSource {
    /// Original source topic.
    pub(crate) topic: Arc<str>,
    /// Original source partition.
    pub(crate) partition: i32,
    /// Original source offset.
    pub(crate) offset: i64,
}

/// Common context shared by every dead-letter job, independent of how the bytes
/// are obtained (in hand vs. recovered by re-read).
struct JobMeta {
    reason: DlqReason,
    source: DlqSource,
    signal: Option<SignalType>,
    /// The error/reason string rendered into the `dlq.error` header.
    error: String,
    /// The resolved destination DLQ topic.
    topic: String,
}

impl JobMeta {
    /// Build the header context for this job.
    fn header_context(&self) -> DlqHeaderContext {
        DlqHeaderContext {
            error: self.error.clone(),
            reason: self.reason.as_str(),
            source_topic: self.source.topic.to_string(),
            source_partition: self.source.partition,
            source_offset: self.source.offset,
            signal: signal_str(self.signal),
            timestamp_millis: now_millis(),
        }
    }
}

/// Signal string for the `dlq.signal` header.
fn signal_str(signal: Option<SignalType>) -> &'static str {
    match signal {
        Some(SignalType::Traces) => "traces",
        Some(SignalType::Metrics) => "metrics",
        Some(SignalType::Logs) => "logs",
        None => "unknown",
    }
}

/// The terminal result of a dead-letter workflow for one message. The receiver
/// uses this to advance the source offset (on any outcome) and to record
/// telemetry.
#[derive(Debug)]
pub(crate) struct DlqCompletion {
    /// Original source identity.
    pub(crate) source: DlqSource,
    /// Signal of the dead-lettered message.
    pub(crate) signal: Option<SignalType>,
    /// The failure category.
    pub(crate) reason: DlqReason,
    /// Whether the message was produced to the DLQ or dropped (loss).
    pub(crate) produced: bool,
    /// Whether a produce failure (when it occurred) was record-level permanent.
    pub(crate) permanent_failure: bool,
}

impl DlqCompletion {
    /// A message that could not be dead-lettered and must be dropped (loss).
    fn loss(meta: JobMeta) -> Self {
        Self {
            source: meta.source,
            signal: meta.signal,
            reason: meta.reason,
            produced: false,
            permanent_failure: true,
        }
    }

    /// Map a producer send outcome for `meta` to a completion.
    fn from_outcome(meta: JobMeta, outcome: DlqSendOutcome) -> Self {
        let (produced, permanent_failure) = match outcome {
            DlqSendOutcome::Produced => (true, false),
            DlqSendOutcome::Failed { permanent } => (false, permanent),
        };
        Self {
            source: meta.source,
            signal: meta.signal,
            reason: meta.reason,
            produced,
            permanent_failure,
        }
    }

    /// The failure category as its wire string, for logging.
    pub(crate) fn reason_str(&self) -> &'static str {
        self.reason.as_str()
    }
}

/// How a job's payload bytes are obtained.
enum JobKind {
    /// Bytes already in hand (decode / unknown_topic path).
    Inline {
        payload: Vec<u8>,
        original_headers: Option<OwnedHeaders>,
    },
    /// Bytes must be recovered from Kafka by the re-read consumer.
    Reread,
}

/// A dead-letter job awaiting an in-flight slot.
struct Job {
    meta: JobMeta,
    kind: JobKind,
}

/// A future producing one [`DlqCompletion`].
type DlqFuture = Pin<Box<dyn Future<Output = DlqCompletion>>>;

/// Owns the DLQ producer and (optionally) the re-read consumer, and bounds the
/// number of outstanding deliveries with [`DLQ_MAX_IN_FLIGHT`].
///
// DLQ-PHASE-2 (Change): `DlqCompletion` stays as the outcome type, but it is
// filled from the downstream "dlq"-port ack/nack instead of from the producer.
pub(crate) struct DlqManager {
    config: ResolvedDlqConfig,
    // DLQ-PHASE-2 (Remove): the producer and the in-flight bounding go away; the
    // port channel plus the downstream exporter's max_in_flight provide
    // backpressure. The re-read consumer below is retained.
    producer: Arc<DlqProducer>,
    /// Retained to recover original Kafka bytes for permanent nacks. Present
    /// only when permanent-nack capture is enabled.
    reread: Option<Arc<RereadConsumer>>,
    in_flight: FuturesUnordered<DlqFuture>,
}

impl DlqManager {
    /// Build a DLQ manager from a validated receiver config.
    ///
    /// Constructs the producer eagerly and, when permanent-nack capture is
    /// enabled, the dedicated re-read consumer, so an unreachable or
    /// misconfigured DLQ connection fails fast at startup.
    // DLQ-PHASE-2 (Change): take the effect handler for port send/subscribe and
    // drop the eager producer build; the re-read consumer build is retained.
    // Validity becomes "the dlq port is connected" checked at wiring time.
    pub(crate) fn new(config: &KafkaReceiverConfig) -> Result<Option<Self>, KafkaError> {
        let Some(dlq) = config.dlq() else {
            return Ok(None);
        };

        let producer_config = config
            .build_dlq_producer_config()
            .expect("dlq present implies producer config");
        let producer = Arc::new(DlqProducer::new(&producer_config)?);

        let reread = if dlq.capture_permanent_nack {
            let consumer_config = config
                .build_dlq_reread_consumer_config()
                .expect("dlq present implies reread config");
            // Fixed re-read fetch bound, independent of librdkafka defaults.
            let fetch_timeout = std::time::Duration::from_millis(DLQ_OP_TIMEOUT_MS);
            Some(Arc::new(RereadConsumer::new(
                &consumer_config,
                fetch_timeout,
            )?))
        } else {
            None
        };

        Ok(Some(Self {
            config: dlq.clone(),
            producer,
            reread,
            in_flight: FuturesUnordered::new(),
        }))
    }

    /// Whether the given failure category should be dead-lettered.
    pub(crate) fn captures(&self, reason: DlqReason) -> bool {
        match reason {
            DlqReason::Decode => self.config.capture_decode,
            DlqReason::UnknownTopic => self.config.capture_unknown_topic,
            DlqReason::PermanentNack => self.config.capture_permanent_nack,
        }
    }

    /// Resolve the DLQ topic for a signal. `None` when the message's signal has
    /// no configured DLQ topic (should not happen for captured signals given
    /// validation, but callers treat `None` as an immediate loss).
    fn topic_for(&self, signal: Option<SignalType>) -> Option<String> {
        // Prefer the signal-specific topic; fall back to any configured topic
        // for signal-less categories (e.g. unknown_topic).
        signal
            .and_then(|sig| self.config.topic_for(sig))
            .or_else(|| self.config.all_topics().first().copied())
            .map(str::to_string)
    }

    /// Submit a decode / unknown_topic failure whose bytes are already in hand.
    ///
    /// Returns `Some(DlqCompletion)` immediately when the message cannot be
    /// admitted (no topic resolved, or the in-flight bound is reached); in that
    /// case the caller records the loss and advances the source offset.
    pub(crate) fn submit_inline(
        &mut self,
        reason: DlqReason,
        source: DlqSource,
        signal: Option<SignalType>,
        error: String,
        payload: Vec<u8>,
        original_headers: Option<OwnedHeaders>,
    ) -> Option<DlqCompletion> {
        self.submit(
            reason,
            source,
            signal,
            error,
            JobKind::Inline {
                payload,
                original_headers,
            },
        )
    }

    /// Submit a permanent-nack failure whose bytes must be recovered by the
    /// dedicated re-read consumer.
    // DLQ-PHASE-2 (Change): still recovers the original bytes via the re-read
    // consumer, then sends them out the "dlq" port instead of producing.
    pub(crate) fn submit_reread(
        &mut self,
        source: DlqSource,
        signal: Option<SignalType>,
        error: String,
    ) -> Option<DlqCompletion> {
        self.submit(
            DlqReason::PermanentNack,
            source,
            signal,
            error,
            JobKind::Reread,
        )
    }

    /// Resolve the destination topic, build the job, and admit it. Returns an
    /// immediate loss completion when no topic resolves or the in-flight bound
    /// is reached.
    fn submit(
        &mut self,
        reason: DlqReason,
        source: DlqSource,
        signal: Option<SignalType>,
        error: String,
        kind: JobKind,
    ) -> Option<DlqCompletion> {
        let Some(topic) = self.topic_for(signal) else {
            return Some(DlqCompletion::loss(JobMeta {
                reason,
                source,
                signal,
                error,
                topic: String::new(),
            }));
        };
        let job = Job {
            meta: JobMeta {
                reason,
                source,
                signal,
                error,
                topic,
            },
            kind,
        };
        self.admit(job)
    }

    /// Admit a job: start it when an in-flight slot is free, otherwise return a
    /// loss completion so the caller records `dlq.loss` and advances the source
    /// offset.
    fn admit(&mut self, job: Job) -> Option<DlqCompletion> {
        if self.in_flight.len() >= DLQ_MAX_IN_FLIGHT {
            // In-flight full: drop the incoming message so ingestion never stalls.
            return Some(DlqCompletion::loss(job.meta));
        }
        // DLQ-PHASE-2 (Change): instead of handing the job to the producer, build
        // an OtapPdata (raw bytes -> OtlpProtoBytes::Export*Request, dlq.* as
        // transport headers) and send it out the "dlq" port with an
        // ACKS_OR_NACKS subscription; the ack/nack later completes the job.
        let producer = Arc::clone(&self.producer);
        let reread = self.reread.clone();
        self.in_flight
            .push(Box::pin(run_job(producer, reread, job)));
        None
    }

    /// Await the next completed DLQ delivery. Resolves to `None` only when there
    /// is no outstanding work (so callers must guard with
    /// [`has_work`](Self::has_work)).
    // DLQ-PHASE-2 (Remove): there is no completion future in port mode;
    // completions arrive on the receiver's Ack/Nack control handlers.
    pub(crate) async fn next_completion(&mut self) -> Option<DlqCompletion> {
        self.in_flight.next().await
    }

    /// Whether the manager has any outstanding work.
    // DLQ-PHASE-2 (Remove): the completion-drain branch it guards is removed.
    pub(crate) fn has_work(&self) -> bool {
        !self.in_flight.is_empty()
    }

    /// Service a keep-warm poll on the idle re-read consumer so its broker
    /// connection stays serviced between jobs.
    // DLQ-PHASE-2 (Keep): the re-read consumer is retained in port mode and
    // still needs keep-warm polling between terminal-nack recoveries.
    pub(crate) fn keep_warm(&self) {
        if let Some(reread) = &self.reread {
            reread.keep_warm();
        }
    }
}

/// Run one dead-letter job to completion: obtain the payload bytes (in hand for
/// inline jobs, recovered by the re-read consumer for permanent nacks), build
/// the DLQ record, and produce it. Any recovery failure is a loss with no
/// produce attempt.
async fn run_job(
    producer: Arc<DlqProducer>,
    reread: Option<Arc<RereadConsumer>>,
    job: Job,
) -> DlqCompletion {
    let Job { meta, kind } = job;
    let (payload, source_headers) = match kind {
        JobKind::Inline {
            payload,
            original_headers,
        } => (payload, original_headers),
        JobKind::Reread => {
            // Permanent-nack capture guarantees the re-read consumer exists.
            let Some(reread) = reread else {
                return DlqCompletion::loss(meta);
            };
            match reread.recover(&meta.source).await {
                RereadOutcome::Recovered(result) => (result.payload, result.headers),
                // Not found, timed out, or the blocking task failed: loss.
                RereadOutcome::NotFound => return DlqCompletion::loss(meta),
            }
        }
    };

    let headers = build_dlq_headers(&meta.header_context(), source_headers.as_ref());
    let outcome = producer
        .produce(DlqRecord {
            topic: meta.topic.clone(),
            payload,
            headers,
        })
        .await;
    DlqCompletion::from_outcome(meta, outcome)
}

/// Current wall-clock time in unix milliseconds (best-effort; 0 before epoch).
fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
