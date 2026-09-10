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
//! All DLQ broker I/O runs off the receive loop and is timeout-bounded:
//! - the producer polls on its own background thread and each produce awaits a
//!   bounded delivery future;
//! - the permanent-nack re-read runs on `spawn_blocking` with a fetch timeout.
//!
//! The receive loop only ever polls the manager's completion future and drains
//! the pending queue, so a stalled broker or slow re-read never blocks
//! ingestion. On any failure (produce error, timeout, or unrecoverable bytes)
//! the message is counted as `dlq.loss` and the source offset is advanced.
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
use self::reread::{RereadConsumer, RereadOutcome, reread_blocking};
use super::super::config::{
    DLQ_MAX_IN_FLIGHT, DLQ_OP_TIMEOUT_MS, DLQ_PENDING_QUEUE_CAP, EffectiveDlqConfig,
    KafkaReceiverConfig,
};
use futures::stream::{FuturesUnordered, StreamExt};
use otel_arrow_dfe_config::SignalType;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedHeaders;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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
    /// The failure category as its wire string, for logging.
    pub(crate) fn reason_str(&self) -> &'static str {
        self.reason.as_str()
    }
}

/// A dead-letter job with the source bytes already in hand (decode /
/// unknown_topic path).
struct InlineJob {
    reason: DlqReason,
    source: DlqSource,
    signal: Option<SignalType>,
    error: String,
    topic: String,
    payload: Vec<u8>,
    original_headers: Option<OwnedHeaders>,
}

/// A dead-letter job whose bytes must be recovered from Kafka (permanent-nack
/// path).
struct RereadJob {
    reason: DlqReason,
    source: DlqSource,
    signal: Option<SignalType>,
    error: String,
    topic: String,
}

/// A job awaiting an in-flight slot.
enum PendingJob {
    Inline(InlineJob),
    Reread(RereadJob),
}

impl PendingJob {
    fn source(&self) -> &DlqSource {
        match self {
            PendingJob::Inline(j) => &j.source,
            PendingJob::Reread(j) => &j.source,
        }
    }
    fn signal(&self) -> Option<SignalType> {
        match self {
            PendingJob::Inline(j) => j.signal,
            PendingJob::Reread(j) => j.signal,
        }
    }
    fn reason(&self) -> DlqReason {
        match self {
            PendingJob::Inline(j) => j.reason,
            PendingJob::Reread(j) => j.reason,
        }
    }
}

/// A future producing one [`DlqCompletion`].
type DlqFuture = Pin<Box<dyn Future<Output = DlqCompletion>>>;

/// Owns the DLQ producer and (optionally) the re-read consumer, and bounds the
/// number of outstanding deliveries with an overflow pending queue.
///
// DLQ-PHASE-2 (Change): `DlqCompletion` stays as the outcome type, but it is
// filled from the downstream "dlq"-port ack/nack instead of from the producer.
pub(crate) struct DlqManager {
    config: EffectiveDlqConfig,
    // DLQ-PHASE-2 (Remove): producer, re-read consumer, and the in-flight /
    // pending bounding all go away; the port channel plus the downstream
    // exporter's max_in_flight provide backpressure.
    producer: Arc<DlqProducer>,
    /// Present only when permanent-nack capture is enabled.
    reread: Option<Arc<RereadConsumer>>,
    in_flight: FuturesUnordered<DlqFuture>,
    pending: VecDeque<PendingJob>,
}

impl DlqManager {
    /// Build a DLQ manager from a validated receiver config.
    ///
    /// Constructs the producer eagerly and, when permanent-nack capture is
    /// enabled, the dedicated re-read consumer, so an unreachable or
    /// misconfigured DLQ connection fails fast at startup.
    // DLQ-PHASE-2 (Change): take the effect handler for port send/subscribe and
    // drop the eager producer/re-read build; validity becomes "the dlq port is
    // connected" checked at wiring time, not a broker connect here.
    pub(crate) fn new(config: &KafkaReceiverConfig) -> Result<Option<Self>, KafkaError> {
        let Some(dlq) = config.dlq() else {
            return Ok(None);
        };

        let producer_config = config
            .build_dlq_producer_config()
            .expect("dlq present implies producer config");
        let producer = DlqProducer::new(&producer_config)?;

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
            producer: Arc::new(producer),
            reread,
            in_flight: FuturesUnordered::new(),
            pending: VecDeque::new(),
        }))
    }

    /// Whether `decode` failures should be dead-lettered.
    pub(crate) fn captures_decode(&self) -> bool {
        self.config.capture_decode
    }

    /// Whether `unknown_topic` failures should be dead-lettered.
    pub(crate) fn captures_unknown_topic(&self) -> bool {
        self.config.capture_unknown_topic
    }

    /// Whether `permanent_nack` failures should be dead-lettered.
    pub(crate) fn captures_permanent_nack(&self) -> bool {
        self.config.capture_permanent_nack
    }

    /// Current number of outstanding (in-flight) deliveries.
    pub(crate) fn in_flight_len(&self) -> usize {
        self.in_flight.len()
    }

    /// Current pending-queue depth.
    pub(crate) fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Resolve the DLQ topic for a signal. `None` when the message's signal has
    /// no configured DLQ topic (should not happen for captured signals given
    /// validation, but callers treat `None` as an immediate loss).
    fn topic_for(&self, signal: Option<SignalType>) -> Option<String> {
        // Prefer the signal-specific topic; fall back to any configured topic
        // for signal-less categories (e.g. unknown_topic).
        if let Some(sig) = signal
            && let Some(topic) = self.config.topic_for(sig)
        {
            return Some(topic.to_string());
        }
        self.config.all_topics().first().map(|t| (*t).to_string())
    }

    /// Submit a decode / unknown_topic failure whose bytes are already in hand.
    ///
    /// Returns `Some(DlqCompletion)` immediately when the message cannot even be
    /// enqueued (no topic resolved, or the pending queue is at capacity); in
    /// that case the caller records the loss and advances the source offset.
    pub(crate) fn submit_inline(
        &mut self,
        reason: DlqReason,
        source: DlqSource,
        signal: Option<SignalType>,
        error: String,
        payload: Vec<u8>,
        original_headers: Option<OwnedHeaders>,
    ) -> Option<DlqCompletion> {
        let Some(topic) = self.topic_for(signal) else {
            return Some(DlqCompletion {
                source,
                signal,
                reason,
                produced: false,
                permanent_failure: true,
            });
        };
        let job = PendingJob::Inline(InlineJob {
            reason,
            source,
            signal,
            error,
            topic,
            payload,
            original_headers,
        });
        self.admit(job)
    }

    /// Submit a permanent-nack failure whose bytes must be recovered by the
    /// dedicated re-read consumer.
    // DLQ-PHASE-2 (Remove): a permanent nack reuses the refused pdata in hand;
    // it is sent out the "dlq" port directly with no byte re-read.
    pub(crate) fn submit_reread(
        &mut self,
        source: DlqSource,
        signal: Option<SignalType>,
        error: String,
    ) -> Option<DlqCompletion> {
        let reason = DlqReason::PermanentNack;
        let Some(topic) = self.topic_for(signal) else {
            return Some(DlqCompletion {
                source,
                signal,
                reason,
                produced: false,
                permanent_failure: true,
            });
        };
        let job = PendingJob::Reread(RereadJob {
            reason,
            source,
            signal,
            error,
            topic,
        });
        self.admit(job)
    }

    /// Admit a job: start it if a slot is free, else enqueue it. Returns a loss
    /// completion when the pending queue is full (overflow -> drop incoming).
    fn admit(&mut self, job: PendingJob) -> Option<DlqCompletion> {
        if self.in_flight.len() < DLQ_MAX_IN_FLIGHT {
            self.start(job);
            None
        } else if self.pending.len() < DLQ_PENDING_QUEUE_CAP {
            self.pending.push_back(job);
            None
        } else {
            // Overflow: drop the incoming message so ingestion never stalls.
            Some(DlqCompletion {
                source: job.source().clone(),
                signal: job.signal(),
                reason: job.reason(),
                produced: false,
                permanent_failure: true,
            })
        }
    }

    /// Spawn the work future for a job into the in-flight set.
    // DLQ-PHASE-2 (Change): instead of handing the job to the producer, build an
    // OtapPdata (raw bytes -> OtlpProtoBytes::Export*Request, dlq.* as transport
    // headers) and send it out the "dlq" port with an ACKS_OR_NACKS
    // subscription; the ack/nack later completes the job.
    fn start(&mut self, job: PendingJob) {
        let producer = Arc::clone(&self.producer);
        match job {
            PendingJob::Inline(inline) => {
                self.in_flight.push(Box::pin(run_inline(producer, inline)));
            }
            PendingJob::Reread(reread_job) => {
                let reread = self.reread.clone();
                self.in_flight
                    .push(Box::pin(run_reread(producer, reread, reread_job)));
            }
        }
    }

    /// Await the next completed DLQ delivery, refilling an in-flight slot from
    /// the pending queue. Resolves to `None` only when there is no outstanding
    /// work (so callers must guard with [`has_work`](Self::has_work)).
    // DLQ-PHASE-2 (Remove): there is no completion future in port mode;
    // completions arrive on the receiver's Ack/Nack control handlers.
    pub(crate) async fn next_completion(&mut self) -> Option<DlqCompletion> {
        let completion = self.in_flight.next().await?;
        // A slot just freed: promote the oldest pending job.
        if let Some(job) = self.pending.pop_front() {
            self.start(job);
        }
        Some(completion)
    }

    /// Whether the manager has any outstanding or queued work.
    // DLQ-PHASE-2 (Remove): the completion-drain branch it guards is removed.
    pub(crate) fn has_work(&self) -> bool {
        !self.in_flight.is_empty()
    }

    /// Service a keep-warm poll on the idle re-read consumer so its broker
    /// connection stays serviced between jobs.
    // DLQ-PHASE-2 (Remove): no re-read consumer to keep warm in port mode.
    pub(crate) fn keep_warm(&self) {
        if let Some(reread) = &self.reread {
            reread.keep_warm();
        }
    }
}

/// Build the DLQ record for an inline job and produce it.
async fn run_inline(producer: Arc<DlqProducer>, job: InlineJob) -> DlqCompletion {
    let ctx = header_context(&job.reason, &job.source, job.signal, &job.error);
    let headers = build_dlq_headers(&ctx, job.original_headers.as_ref());
    let record = DlqRecord {
        topic: job.topic,
        payload: job.payload,
        headers,
    };
    let outcome = producer.produce(record).await;
    finish(job.reason, job.source, job.signal, outcome)
}

/// Recover the bytes for a re-read job, then produce it. A failed recovery is a
/// loss with no produce attempt.
async fn run_reread(
    producer: Arc<DlqProducer>,
    reread: Option<Arc<RereadConsumer>>,
    job: RereadJob,
) -> DlqCompletion {
    let Some(reread) = reread else {
        return DlqCompletion {
            source: job.source,
            signal: job.signal,
            reason: job.reason,
            produced: false,
            permanent_failure: true,
        };
    };

    let consumer = reread.handle();
    let fetch_timeout = reread.fetch_timeout();
    let topic = job.source.topic.clone();
    let partition = job.source.partition;
    let offset = job.source.offset;

    // Recover the original bytes off the receive loop.
    let recovered = tokio::task::spawn_blocking(move || {
        reread_blocking(&consumer, &topic, partition, offset, fetch_timeout)
    })
    .await;

    let result = match recovered {
        Ok(RereadOutcome::Recovered(result)) => result,
        // Not found, timed out, or the blocking task failed: loss.
        Ok(RereadOutcome::NotFound) | Err(_) => {
            return DlqCompletion {
                source: job.source,
                signal: job.signal,
                reason: job.reason,
                produced: false,
                permanent_failure: true,
            };
        }
    };

    let ctx = header_context(&job.reason, &job.source, job.signal, &job.error);
    let headers = build_dlq_headers(&ctx, result.headers.as_ref());
    let record = DlqRecord {
        topic: job.topic,
        payload: result.payload,
        headers,
    };
    let outcome = producer.produce(record).await;
    finish(job.reason, job.source, job.signal, outcome)
}

/// Build the header context for a job.
fn header_context(
    reason: &DlqReason,
    source: &DlqSource,
    signal: Option<SignalType>,
    error: &str,
) -> DlqHeaderContext {
    DlqHeaderContext {
        error: error.to_string(),
        reason: reason.as_str(),
        source_topic: source.topic.to_string(),
        source_partition: source.partition,
        source_offset: source.offset,
        signal: signal_str(signal),
        timestamp_millis: now_millis(),
    }
}

/// Map a producer outcome to a completion.
fn finish(
    reason: DlqReason,
    source: DlqSource,
    signal: Option<SignalType>,
    outcome: DlqSendOutcome,
) -> DlqCompletion {
    match outcome {
        DlqSendOutcome::Produced => DlqCompletion {
            source,
            signal,
            reason,
            produced: true,
            permanent_failure: false,
        },
        DlqSendOutcome::Failed { permanent } => DlqCompletion {
            source,
            signal,
            reason,
            produced: false,
            permanent_failure: permanent,
        },
    }
}

/// Current wall-clock time in unix milliseconds (best-effort; 0 before epoch).
fn now_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
