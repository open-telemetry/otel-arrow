// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Durable buffer for crash-resilient buffering of OTAP data.
//!
//! This processor provides durable buffering by writing incoming
//! telemetry data to a write-ahead log and segment storage before forwarding
//! downstream. On acknowledgement from downstream, the data is marked as
//! consumed; on rejection, data can be replayed.
//!
//! # Architecture
//!
//! ```text
//! Upstream → DurableBuffer → Downstream
//!                    ↓
//!              StorageEngine
//!                    ↓
//!            WAL + Segments
//! ```
//!
//! # Per-Core Isolation
//!
//! Each processor instance (one per CPU core) has its own isolated storage engine
//! with a separate WAL and segment store. Data is partitioned by core at runtime,
//! with each core's data stored in `{path}/core_{core_id}/`.
//!
//! # Dispatch Strategy Considerations
//!
//! **Important**: The dispatch strategy on the incoming edge affects behavior:
//!
//! | Strategy | Behavior | Recommendation |
//! |----------|----------|----------------|
//! | `RoundRobin` | Data distributed across cores, each persists its share | ✅ **Recommended** |
//! | `Random` | Similar to round-robin | ✅ OK |
//! | `LeastLoaded` | Similar to round-robin | ✅ OK |
//! | `Broadcast` | Same data persisted N times (once per core) | ⚠️ **Avoid** - causes N× storage and duplicates |
//!
//! For the outgoing edge (to exporters), any dispatch strategy is valid.
//!
//! # Message Flow
//!
//! - `Message::Data`: Ingested to storage, ACK sent upstream after durable write
//! - `TimerTick`: Poll storage for bundles, send downstream
//! - `Ack`: Extract BundleRef from calldata, call handle.ack()
//! - `Nack (permanent)`: Call handle.reject() — no retry
//! - `Nack (transient)`: Call handle.defer() and schedule retry via a wakeup
//! - `Shutdown`: Flush storage engine
//!
//! # Retry Behavior and Error Handling
//!
//! On NACK from downstream, bundles are handled based on the NACK's `permanent` status:
//!
//! - **Permanent NACKs** (e.g., malformed data, schema validation failures): The bundle
//!   is immediately rejected via `handle.reject()` and will not be retried. Monitor the
//!   `bundles_nacked_permanent` metric to detect data being dropped due to permanent failures.
//!
//! - **Transient NACKs** (e.g., network issues, temporary downstream unavailability): Bundles
//!   are retried with exponential backoff until either delivery succeeds or the data is evicted
//!   by the configured retention policy (`retention_size_cap` + `drop_oldest`).
//!
//! There is no `max_retries` limit for transient failures: a retry limit would cause
//! **data loss** during legitimate extended outages.
//!
//! **Operational guidance:**
//!
//! - Monitor `bundles_nacked_permanent` metric to detect permanent failures (data loss)
//! - Monitor `retries_scheduled` metric to detect persistently failing data
//! - Use `retention_size_cap` to bound storage; `drop_oldest` policy evicts
//!   stuck data when space is needed for new data
//! - `max_in_flight` limit prevents thundering herd after recovery

mod bundle_adapter;
mod config;
mod deferred_retry_state;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use linkme::distributed_slice;
use quiver::budget::DiskBudget;
use quiver::segment::SegmentSeq;
use quiver::segment_store::SegmentStore;
use quiver::subscriber::{
    BundleHandle, BundleIndex, BundleRef, RegistryCallback, SegmentProvider, SubscriberId,
};
use quiver::{QuiverConfig, QuiverEngine};
use smallvec::smallvec;

use otap_df_telemetry::{otel_debug, otel_error, otel_info, otel_warn};

use otap_df_otap::OTAP_PROCESSOR_FACTORIES;
use otap_df_otap::pdata::OtapPdata;

use bundle_adapter::{
    OtapRecordBundleAdapter, OtlpBytesAdapter, convert_bundle_to_pdata, signal_type_from_slot_id,
};
pub use config::{DurableBufferConfig, OtlpHandling, SizeCapPolicy};
use deferred_retry_state::DeferredRetryState;
#[cfg(test)]
use deferred_retry_state::RETRY_WAKEUP_SLOT;

use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::Context8u8;
use otap_df_engine::control::{
    AckMsg, CallData, NackMsg, NodeControlMsg, WakeupRevision, WakeupSlot,
};
use otap_df_engine::error::Error;
use otap_df_engine::local::processor::EffectHandler;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, Interests, LocalWakeupRequirements, ProcessorFactory,
    ProcessorRuntimeRequirements, ProducerEffectHandlerExtension,
};
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use otap_df_telemetry::instrument::{Counter, Gauge, ObserveCounter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;

/// URN for the durable buffer.
pub const DURABLE_BUFFER_URN: &str = "urn:otel:processor:durable_buffer";

/// Minimum interval between repeated warning logs for the same condition
/// (backpressure, flush failures). Prevents log flooding when the timer
/// tick fires every poll_interval (~100 ms).
const WARN_RATE_LIMIT: Duration = Duration::from_secs(10);

/// Subscriber ID used by this processor.
const SUBSCRIBER_ID: &str = "durable-buffer";

// ─────────────────────────────────────────────────────────────────────────────
// Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Metrics for the durable buffer processor.
///
/// Follows RFC-aligned telemetry conventions:
/// - Metric set name follows `otap.<role>.<component>` pattern
/// - Channel metrics already track bundle send/receive counts
/// - This tracks ACK/NACK status, item counts, storage, and retries
///
/// Please update the documentation at Telemetry.md when adding new
/// metrics.
#[metric_set(name = "otap.processor.durable_buffer")]
#[derive(Debug, Default, Clone)]
pub struct DurableBufferMetrics {
    // ─── ACK/NACK tracking ──────────────────────────────────────────────────
    // Note: Bundle send/receive counts are tracked by channel metrics.
    // These metrics track downstream acknowledgement status.
    /// Number of bundles acknowledged by downstream.
    #[metric(unit = "{bundle}")]
    pub bundles_acked: Counter<u64>,

    /// Number of bundles deferred for retry after transient downstream failures.
    #[metric(unit = "{bundle}")]
    pub bundles_nacked_deferred: Counter<u64>,

    /// Number of bundles permanently rejected by downstream (not retried).
    /// These indicate data loss due to permanent failures (e.g., malformed data).
    #[metric(unit = "{bundle}")]
    pub bundles_nacked_permanent: Counter<u64>,

    // ─── Rejected item metrics (per signal type) ────────────────────────
    /// Number of log records rejected.
    #[metric(unit = "{log_record}")]
    pub rejected_log_records: Counter<u64>,

    /// Number of metric data points rejected.
    #[metric(unit = "{data_point}")]
    pub rejected_metric_points: Counter<u64>,

    /// Number of spans rejected.
    #[metric(unit = "{span}")]
    pub rejected_spans: Counter<u64>,

    // ─── Consumed item metrics (per signal type) ────────────────────────
    /// Number of log records consumed (ingested to durable storage).
    /// For OTLP bytes, counted by scanning the protobuf wire format without full deserialization.
    #[metric(unit = "{log_record}")]
    pub consumed_log_records: Counter<u64>,

    /// Number of metric data points consumed (ingested to durable storage).
    /// For OTLP bytes, counted by scanning the protobuf wire format without full deserialization.
    #[metric(unit = "{data_point}")]
    pub consumed_metric_points: Counter<u64>,

    /// Number of spans consumed (ingested to durable storage).
    /// For OTLP bytes, counted by scanning the protobuf wire format without full deserialization.
    #[metric(unit = "{span}")]
    pub consumed_spans: Counter<u64>,

    // ─── Produced item metrics (per signal type) ────────────────────────
    /// Number of log records produced (sent downstream).
    /// For OTLP bytes, counted by scanning the protobuf wire format without full deserialization.
    #[metric(unit = "{log_record}")]
    pub produced_log_records: Counter<u64>,

    /// Number of metric data points produced (sent downstream).
    /// For OTLP bytes, counted by scanning the protobuf wire format without full deserialization.
    #[metric(unit = "{data_point}")]
    pub produced_metric_points: Counter<u64>,

    /// Number of spans produced (sent downstream).
    /// For OTLP bytes, counted by scanning the protobuf wire format without full deserialization.
    #[metric(unit = "{span}")]
    pub produced_spans: Counter<u64>,

    // ─── Error and backpressure metrics ─────────────────────────────────────
    /// Number of ingest errors (excludes backpressure/capacity rejections).
    #[metric(unit = "{error}")]
    pub ingest_errors: Counter<u64>,

    /// Number of ingest rejections due to storage backpressure (soft cap exceeded).
    #[metric(unit = "{rejection}")]
    pub ingest_backpressure: Counter<u64>,

    /// Number of read errors.
    #[metric(unit = "{error}")]
    pub read_errors: Counter<u64>,

    // ─── Storage metrics (updated on telemetry collection) ──────────────────
    /// Current bytes used by persistent storage (WAL + segments).
    #[metric(unit = "By")]
    pub storage_bytes_used: Gauge<u64>,

    /// Configured storage capacity cap.
    #[metric(unit = "By")]
    pub storage_bytes_cap: Gauge<u64>,

    /// Total segments force-dropped due to DropOldest retention policy.
    /// Non-zero values indicate data loss.
    #[metric(unit = "{segment}")]
    pub dropped_segments: ObserveCounter<u64>,

    /// Total bundles lost due to force-dropped segments (DropOldest policy).
    /// Non-zero values indicate data loss.
    #[metric(unit = "{bundle}")]
    pub dropped_bundles: ObserveCounter<u64>,

    /// Total individual items (log records, data points, spans) lost due to
    /// force-dropped segments (DropOldest policy). Non-zero values indicate data loss.
    #[metric(unit = "{item}")]
    pub dropped_items: ObserveCounter<u64>,

    /// Total bundles lost due to expired segments (max_age retention).
    /// Non-zero values indicate data aged out before delivery.
    #[metric(unit = "{bundle}")]
    pub expired_bundles: ObserveCounter<u64>,

    /// Total individual items (log records, data points, spans) lost due to
    /// expired segments (max_age retention). Non-zero values indicate data
    /// aged out before delivery.
    #[metric(unit = "{item}")]
    pub expired_items: ObserveCounter<u64>,

    // ─── Retry metrics ──────────────────────────────────────────────────────
    /// Number of retry attempts scheduled.
    #[metric(unit = "{retry}")]
    pub retries_scheduled: Counter<u64>,

    /// Current number of bundles in-flight to downstream.
    #[metric(unit = "{bundle}")]
    pub in_flight: Gauge<u64>,

    // ─── Requeued item metrics (per signal type) ────────────────────────────
    // These count individual items in NACKed bundles when scheduled for retry.
    /// Number of individual log records requeued for retry after NACK.
    #[metric(unit = "{log_record}")]
    pub requeued_log_records: Counter<u64>,

    /// Number of individual metric data points requeued for retry after NACK.
    #[metric(unit = "{data_point}")]
    pub requeued_metric_points: Counter<u64>,

    /// Number of individual spans requeued for retry after NACK.
    #[metric(unit = "{span}")]
    pub requeued_spans: Counter<u64>,

    // ─── Queued item metrics (per signal type) ──────────────────────────────
    /// Current number of log records queued (ingested but not yet ACKed).
    #[metric(unit = "{log_record}")]
    pub queued_log_records: Gauge<u64>,

    /// Current number of metric data points queued (ingested but not yet ACKed).
    #[metric(unit = "{data_point}")]
    pub queued_metric_points: Gauge<u64>,

    /// Current number of spans queued (ingested but not yet ACKed).
    #[metric(unit = "{span}")]
    pub queued_spans: Gauge<u64>,

    // ─── Flush metrics -----------------------------------───────────────────
    /// Number of segment finalization (flush) failures.
    /// Non-zero values indicate data at risk -- check logs for root cause.
    /// Data may still be recoverable via WAL replay on restart.
    #[metric(unit = "{error}")]
    pub flush_failures: Counter<u64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// BundleRef CallData Encoding
// ─────────────────────────────────────────────────────────────────────────────

/// Encode a BundleRef into CallData for ACK/NACK tracking.
///
/// Layout: [segment_seq (u64), bundle_index (u32 packed into u64)]
fn encode_bundle_ref(bundle_ref: BundleRef) -> CallData {
    smallvec![
        Context8u8::from(bundle_ref.segment_seq.raw()),
        Context8u8::from(bundle_ref.bundle_index.raw() as u64),
    ]
}

/// Decode a BundleRef from CallData.
fn decode_bundle_ref(calldata: &CallData) -> Option<BundleRef> {
    if calldata.len() < 2 {
        return None;
    }
    let segment_seq = SegmentSeq::new(u64::from(calldata[0]));
    let bundle_index = BundleIndex::new(u64::from(calldata[1]) as u32);
    Some(BundleRef {
        segment_seq,
        bundle_index,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Pending Bundle Tracking
// ─────────────────────────────────────────────────────────────────────────────

/// State for tracking a pending downstream delivery.
///
/// Holds the Quiver bundle handle to keep the bundle claimed while in-flight.
/// When dropped without explicit ack(), the handle's Drop impl will call
/// on_deferred(), releasing the claim and making the bundle eligible for redelivery.
struct PendingBundle {
    /// The Quiver bundle handle - keeps the bundle claimed while in-flight.
    handle: QuiverBundleHandle,
    /// Number of retries attempted.
    retry_count: u32,
    /// Number of items in this bundle (0 for legacy segments without manifest counts).
    item_count: u64,
    /// Signal type of this bundle.
    signal_type: SignalType,
}

/// Result of attempting to process a bundle with non-blocking send.
enum ProcessBundleResult {
    /// Bundle was successfully sent downstream.
    Sent,
    /// Bundle was skipped (already in-flight or scheduled for retry).
    Skipped,
    /// The downstream channel is full (backpressure).
    /// The bundle has been deferred and will be retried on the next tick.
    Backpressure,
    /// An unrecoverable error occurred.
    Error(Error),
}

// ─────────────────────────────────────────────────────────────────────────────
// DurableBuffer
// ─────────────────────────────────────────────────────────────────────────────

/// Type alias for the bundle handle with our callback type.
type QuiverBundleHandle = BundleHandle<RegistryCallback<SegmentStore>>;

/// State of the Quiver engine (lazy initialization).
enum EngineState {
    /// Engine not yet initialized.
    Uninitialized,
    /// Engine initialized and ready.
    Ready {
        engine: Arc<QuiverEngine>,
        subscriber_id: SubscriberId,
    },
    /// Engine failed to initialize.
    Failed(String),
}

/// Outcome of trying to resume one deferred retry.
enum RetryResumeOutcome {
    /// The retry was re-claimed and sent downstream.
    Sent,
    /// The retry remains deferred because resend is blocked for now.
    Deferred,
    /// The retry no longer needs to be retried on this pass.
    Skipped,
}

/// Cached per-segment signal classification for queued-item gauge computation.
///
/// Populated once per segment (on first access after finalization) and never
/// invalidated — segments are immutable after finalization.  Evicted when the
/// segment is no longer tracked by the subscriber.
struct SegmentMetricsSummary {
    /// Per-bundle `(item_count, signal_type)`, ordered by bundle index.
    bundles: Vec<(u64, Option<SignalType>)>,
    /// Pre-computed totals for the fully-pending case (no bundles resolved).
    total_logs: u64,
    total_metrics: u64,
    total_spans: u64,
}

/// Cached segment summary with recency tracking for bounded eviction.
struct CachedSegmentMetrics {
    summary: SegmentMetricsSummary,
    last_seen_generation: u64,
}

/// Durable buffer that provides crash-resilient buffering via Quiver.
///
/// # Segment Metrics Cache
///
/// To avoid per-tick allocations in `recompute_queued_counters`, this struct
/// maintains a `segment_cache` that maps finalized segment sequences to their
/// pre-computed per-bundle signal classification.  Because segments are
/// **immutable** after finalization, the cache entry for a given segment
/// never needs invalidation — only eviction when the segment is cleaned up.
pub struct DurableBuffer {
    /// The Quiver engine state (lazy initialized on first message).
    engine_state: EngineState,

    /// Map of in-flight bundles awaiting downstream ACK/NACK.
    /// Key is the (segment_seq, bundle_index) pair encoded as a u128 for fast lookup.
    pending_bundles: HashMap<(u64, u32), PendingBundle>,

    /// Processor-local retry deferral state, driven by one wakeup slot plus a
    /// local ordered retry queue.
    deferred_retry_state: DeferredRetryState,

    /// Configuration.
    config: DurableBufferConfig,

    /// Core ID for per-core data directory.
    /// Per ARCHITECTURE.md, each core has its own Quiver instance.
    core_id: usize,

    /// Total number of cores running this pipeline.
    /// Used to divide the retention_size_cap across cores.
    num_cores: usize,

    /// Metrics.
    metrics: MetricSet<DurableBufferMetrics>,

    /// Whether timer has been started.
    timer_started: bool,

    /// Last time we logged a flush warning (rate-limiting).
    last_flush_warn: Option<Instant>,

    /// Last time we logged a backpressure warning (rate-limiting).
    last_backpressure_warn: Option<Instant>,

    /// Cached per-segment signal classification.
    ///
    /// Populated on first access per segment; evicted when the subscriber no
    /// longer references the segment (all bundles resolved and progress cleaned up)
    /// and also evicted by recency (LRU-style) when the bounded cache limit is exceeded.
    segment_cache: HashMap<u64, CachedSegmentMetrics>,

    /// Monotonic generation counter used to track segment cache recency.
    segment_cache_generation: u64,

    /// Segment IDs for which metadata load failure has already been warned.
    ///
    /// Prevents warning spam when `bundle_metadata` repeatedly fails for the
    /// same segment across telemetry ticks.
    metadata_load_warned_segments: HashSet<u64>,
}

impl DurableBuffer {
    /// Creates a new durable buffer with the given configuration.
    ///
    /// Validates that the configured `retention_size_cap` is large enough for
    /// the Quiver engine to operate via [`DiskBudget::minimum_hard_cap()`].
    ///
    /// Note: The Quiver engine is lazily initialized on the first message
    /// to ensure we're running within an async context.
    pub fn new(
        config: DurableBufferConfig,
        pipeline_ctx: &PipelineContext,
    ) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<DurableBufferMetrics>();
        let core_id = pipeline_ctx.core_id();
        let num_cores = pipeline_ctx.num_cores();

        // Validate that per-core budget is large enough for the engine.
        // QuiverEngine::open() enforces this lazily on first message. Catch it
        // here so the pipeline fails fast at construction time.
        let quiver_config = Self::build_quiver_config(&config);
        let total_size_cap = config.size_cap_bytes();
        let per_core_size_cap = total_size_cap / num_cores.max(1) as u64;
        let segment_size = quiver_config.segment.target_size_bytes.get();
        let wal_max = DiskBudget::effective_wal_size(&quiver_config);
        let min_per_core = DiskBudget::minimum_hard_cap(segment_size, wal_max);

        if per_core_size_cap < min_per_core {
            let min_total = min_per_core * num_cores as u64;
            let max_cores = total_size_cap / min_per_core;
            return Err(ConfigError::InvalidUserConfig {
                error: if max_cores == 0 {
                    format!(
                        "retention_size_cap ({total_size_cap} bytes) is too small: \
                         the engine requires at least {min_per_core} bytes per core \
                         (WAL max {wal_max} + 2 * segment size {segment_size}). \
                         Increase retention_size_cap to at least {min_total} bytes",
                    )
                } else {
                    format!(
                        "retention_size_cap ({total_size_cap} bytes) is too small for {num_cores} core(s): \
                         per-core capacity is {per_core_size_cap} bytes, but the engine requires at \
                         least {min_per_core} bytes per core (WAL max {wal_max} + 2 * segment size {segment_size}). \
                         Either increase retention_size_cap to at least {min_total} bytes, \
                         or reduce the core count to at most {max_cores}",
                    )
                },
            });
        }

        Ok(Self {
            engine_state: EngineState::Uninitialized,
            pending_bundles: HashMap::new(),
            deferred_retry_state: DeferredRetryState::new(),
            config,
            core_id,
            num_cores,
            metrics,
            timer_started: false,
            last_flush_warn: None,
            last_backpressure_warn: None,
            segment_cache: HashMap::new(),
            segment_cache_generation: 0,
            metadata_load_warned_segments: HashSet::new(),
        })
    }

    /// Upper bound for cached finalized segment summaries.
    const MAX_SEGMENT_CACHE_ENTRIES: usize = 4096;

    /// Evict least-recently-used cached segment summaries when above the limit.
    fn enforce_segment_cache_bound(&mut self) {
        self.enforce_segment_cache_bound_with_limit(Self::MAX_SEGMENT_CACHE_ENTRIES);
    }

    fn enforce_segment_cache_bound_with_limit(&mut self, max_entries: usize) {
        let len = self.segment_cache.len();
        if len <= max_entries {
            return;
        }

        let evict_count = len - max_entries;
        let mut by_age: Vec<(u64, u64)> = self
            .segment_cache
            .iter()
            .map(|(seq, cached)| (*seq, cached.last_seen_generation))
            .collect();
        by_age.sort_by_key(|(_, generation)| *generation);

        for (seq, _) in by_age.into_iter().take(evict_count) {
            let _ = self.segment_cache.remove(&seq);
            let _ = self.metadata_load_warned_segments.remove(&seq);
        }
    }

    /// Build the [`QuiverConfig`] that will be used for the engine.
    ///
    /// This is the single source of truth for translating [`DurableBufferConfig`]
    /// fields into Quiver engine settings. Both [`new()`](Self::new) (for early
    /// validation) and [`init_engine()`](Self::init_engine) (for construction)
    /// call this so the two can never drift apart.
    ///
    /// The returned config does **not** include the per-core `data_dir`; that is
    /// added by `init_engine` since it depends on `core_id`.
    fn build_quiver_config(config: &DurableBufferConfig) -> QuiverConfig {
        // Exhaustive destructure: if a field is added to DurableBufferConfig,
        // the compiler will force you to handle it here (no `..`).
        let DurableBufferConfig {
            path: _,               // per-core subdir added by init_engine
            retention_size_cap: _, // drives the DiskBudget, not QuiverConfig
            max_age,
            size_cap_policy: _, // drives the DiskBudget policy
            poll_interval: _,   // DurableBuffer timer, not engine config
            otlp_handling: _,   // DurableBuffer adapter concern
            max_segment_open_duration,
            initial_retry_interval: _, // DurableBuffer retry logic
            max_retry_interval: _,     // DurableBuffer retry logic
            retry_multiplier: _,       // DurableBuffer retry logic
            max_in_flight: _,          // DurableBuffer flow control
        } = config;

        let mut quiver_config = QuiverConfig::default();
        quiver_config.segment.max_open_duration = *max_segment_open_duration;
        quiver_config.retention.max_age = *max_age;
        quiver_config
    }

    /// Calculate exponential backoff delay with jitter.
    ///
    /// Formula: min(initial * multiplier^retry_count, max_interval) * (0.5 + random(0.5))
    fn calculate_backoff(&self, retry_count: u32) -> Duration {
        let base = self.config.initial_retry_interval.as_secs_f64()
            * self.config.retry_multiplier.powi(retry_count as i32);
        let capped = base.min(self.config.max_retry_interval.as_secs_f64());

        // Add jitter: 50-100% of the capped value
        // Use a simple deterministic "jitter" based on retry_count to avoid rand dependency
        // This spreads retries but is deterministic for testing
        let jitter_factor = 0.5 + (((retry_count as f64 * 0.618033988749895) % 1.0) * 0.5);
        let with_jitter = capped * jitter_factor;

        Duration::from_secs_f64(with_jitter)
    }

    /// Check if we can send more bundles downstream (respects max_in_flight limit).
    fn can_send_more(&self) -> bool {
        self.pending_bundles.len() < self.config.max_in_flight
    }

    /// Resumes one deferred retry, either by sending it downstream again or by
    /// re-deferring it if downstream/backpressure constraints still apply.
    ///
    /// Guarantees:
    /// - respects `max_in_flight`
    /// - re-defers blocked retries with `poll_interval`
    /// - returns enough outcome information for the caller to decide whether
    ///   the current wakeup pass should keep resuming more due retries
    fn resume_retry(
        &mut self,
        bundle_ref: BundleRef,
        retry_count: u32,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<RetryResumeOutcome, Error> {
        if !self.can_send_more() {
            otel_debug!(
                "durable_buffer.retry.deferred",
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw(),
                in_flight = self.pending_bundles.len(),
                max_in_flight = self.config.max_in_flight
            );

            if !self.deferred_retry_state.schedule_after(
                bundle_ref,
                retry_count,
                self.config.poll_interval,
                effect_handler,
            ) {
                otel_warn!("durable_buffer.retry.reschedule_failed");
            }
            return Ok(RetryResumeOutcome::Deferred);
        }

        let claim_result = {
            let (engine, subscriber_id) = self.engine()?;
            engine.claim_bundle(subscriber_id, bundle_ref)
        };

        match claim_result {
            Ok(handle) => match self.try_process_bundle_handle_with_retry_count(
                handle,
                retry_count,
                effect_handler,
            ) {
                ProcessBundleResult::Sent => {
                    otel_debug!(
                        "durable_buffer.retry.sent",
                        segment_seq = bundle_ref.segment_seq.raw(),
                        bundle_index = bundle_ref.bundle_index.raw(),
                        retry_count = retry_count
                    );
                    Ok(RetryResumeOutcome::Sent)
                }
                ProcessBundleResult::Skipped => {
                    otel_warn!(
                        "durable_buffer.retry.skipped",
                        segment_seq = bundle_ref.segment_seq.raw(),
                        bundle_index = bundle_ref.bundle_index.raw()
                    );
                    Ok(RetryResumeOutcome::Skipped)
                }
                ProcessBundleResult::Backpressure => {
                    otel_debug!(
                        "durable_buffer.retry.backpressure",
                        segment_seq = bundle_ref.segment_seq.raw(),
                        bundle_index = bundle_ref.bundle_index.raw()
                    );

                    if !self.deferred_retry_state.schedule_after(
                        bundle_ref,
                        retry_count,
                        self.config.poll_interval,
                        effect_handler,
                    ) {
                        otel_warn!("durable_buffer.retry.reschedule_failed");
                    }
                    Ok(RetryResumeOutcome::Deferred)
                }
                ProcessBundleResult::Error(e) => Err(e),
            },
            Err(e) => {
                otel_debug!(
                    "durable_buffer.retry.claim_failed",
                    segment_seq = bundle_ref.segment_seq.raw(),
                    bundle_index = bundle_ref.bundle_index.raw(),
                    error = %e
                );
                Ok(RetryResumeOutcome::Skipped)
            }
        }
    }

    /// Lazily initialize the Quiver engine on first use.
    async fn ensure_engine_initialized(&mut self) -> Result<(), Error> {
        match &self.engine_state {
            EngineState::Ready { .. } => Ok(()),
            EngineState::Failed(msg) => Err(Error::InternalError {
                message: msg.clone(),
            }),
            EngineState::Uninitialized => match self.init_engine().await {
                Ok((engine, subscriber_id)) => {
                    // Seed queued gauges immediately so they reflect
                    // persisted state before the first telemetry tick.
                    self.recompute_queued_counters(&engine, &subscriber_id);

                    self.engine_state = EngineState::Ready {
                        engine,
                        subscriber_id,
                    };
                    Ok(())
                }
                Err(e) => {
                    let msg = format!("failed to initialize Quiver engine: {}", e);
                    self.engine_state = EngineState::Failed(msg.clone());
                    Err(Error::InternalError { message: msg })
                }
            },
        }
    }

    /// Recompute the queued item gauges from the subscriber registry.
    ///
    /// Accumulates item counts from two sources:
    /// 1. **Finalized segments**: Queried via subscriber progress and segment store.
    ///    Uses a per-segment cache to avoid re-computing signal type classification.
    /// 2. **Open (accumulating) segment**: Queried directly from the engine to include
    ///    in-flight items that have not yet been finalized.
    ///
    /// The gauges reflect all ingested but not-yet-ACKed items across both sources.
    ///
    /// *Snapshotting is intentionally non-atomic across these two sources.* During
    /// segment finalization there is a brief window where a just-finalized segment
    /// may appear in neither snapshot, causing a temporary under-count. Gauges
    /// will self-correct on the next telemetry tick.
    ///
    /// Uses a segment-level cache (`segment_cache`) for finalized segments to avoid
    /// per-tick allocations. Segments are immutable after finalization, so the
    /// per-bundle `(item_count, signal_type)` classification is computed once and
    /// reused on subsequent ticks.
    ///
    /// **Hot-path optimisation**: when no bundles in a segment have been resolved
    /// yet (the common case for recently-written segments), the pre-computed
    /// per-signal totals are used in O(1) rather than iterating the per-bundle vec.
    ///
    /// # Lock budget
    ///
    /// 1. `pending_segment_progress` — subscriber read-lock (brief clone).
    /// 2. Per cache-miss: `bundle_metadata` — segment-store read-lock.
    /// 3. `open_segment_metrics` — engine open-segment lock (brief snapshot).
    ///
    /// After snapshots are taken, all iteration is lock-free.
    fn recompute_queued_counters(&mut self, engine: &QuiverEngine, subscriber_id: &SubscriberId) {
        self.segment_cache_generation = self.segment_cache_generation.wrapping_add(1);
        let current_generation = self.segment_cache_generation;

        // Step 1: snapshot the subscriber's segment progress (brief lock).
        let progress_snapshot = match engine.registry().pending_segment_progress(subscriber_id) {
            Ok(s) => s,
            Err(e) => {
                otel_warn!(
                    "durable_buffer.queued.recompute_error",
                    error = %e,
                    reason = "Queued item gauges may be stale"
                );
                return;
            }
        };

        let mut logs = 0u64;
        let mut metrics = 0u64;
        let mut spans = 0u64;

        // Step 2: iterate finalized segments, populating cache on miss.
        for (&seg_seq, progress) in &progress_snapshot {
            let seg_raw = seg_seq.raw();

            // Populate cache on first access for this segment.
            let summary = match self.segment_cache.entry(seg_raw) {
                Entry::Occupied(entry) => {
                    let cached = entry.into_mut();
                    cached.last_seen_generation = current_generation;
                    &mut cached.summary
                }
                Entry::Vacant(entry) => {
                    let summary = match engine.segment_store().bundle_metadata(seg_seq) {
                        Ok(metadata) => {
                            // Metadata load recovered (or succeeded first try), so
                            // clear any prior warning marker for this segment.
                            let _ = self.metadata_load_warned_segments.remove(&seg_raw);

                            let mut tl = 0u64;
                            let mut tm = 0u64;
                            let mut ts = 0u64;

                            let bundles: Vec<_> = metadata
                                .iter()
                                .map(|entry| {
                                    let sig = entry
                                        .slot_ids
                                        .iter()
                                        .copied()
                                        .find_map(signal_type_from_slot_id);
                                    match sig {
                                        Some(SignalType::Logs) => tl += entry.item_count,
                                        Some(SignalType::Metrics) => tm += entry.item_count,
                                        Some(SignalType::Traces) => ts += entry.item_count,
                                        None => {}
                                    }
                                    (entry.item_count, sig)
                                })
                                .collect();

                            SegmentMetricsSummary {
                                bundles,
                                total_logs: tl,
                                total_metrics: tm,
                                total_spans: ts,
                            }
                        }
                        Err(e) => {
                            // Warn once per segment to avoid per-tick log spam.
                            if self.metadata_load_warned_segments.insert(seg_raw) {
                                otel_warn!(
                                    "durable_buffer.queued.metadata_skip",
                                    segment = seg_raw,
                                    error = %e,
                                    reason = "Segment metadata unavailable; queued counts may under-report"
                                );
                            }
                            continue;
                        }
                    };
                    &mut entry
                        .insert(CachedSegmentMetrics {
                            summary,
                            last_seen_generation: current_generation,
                        })
                        .summary
                }
            };

            // Fast path: no bundles resolved → use precomputed totals.
            if progress.resolved_count() == 0 {
                logs += summary.total_logs;
                metrics += summary.total_metrics;
                spans += summary.total_spans;
            } else {
                // Slow path: iterate per-bundle, skipping resolved.
                for (idx, &(item_count, signal)) in summary.bundles.iter().enumerate() {
                    let Ok(bundle_idx) = u32::try_from(idx) else {
                        otel_warn!(
                            "durable_buffer.queued.bundle_index_overflow",
                            segment = seg_raw,
                            bundle_index = idx,
                            reason = "Bundle index exceeds u32 range; skipping in queued gauge recompute"
                        );
                        continue;
                    };

                    if !progress.is_resolved(BundleIndex::new(bundle_idx)) {
                        match signal {
                            Some(SignalType::Logs) => logs += item_count,
                            Some(SignalType::Metrics) => metrics += item_count,
                            Some(SignalType::Traces) => spans += item_count,
                            None => {}
                        }
                    }
                }
            }
        }

        // Step 3: add items from the open (accumulating) segment.
        let open_bundles = engine.open_segment_bundle_summaries();
        for bundle in open_bundles {
            if bundle.item_count == 0 {
                continue;
            }

            let signal = bundle
                .slot_ids
                .iter()
                .copied()
                .find_map(signal_type_from_slot_id);

            match signal {
                Some(SignalType::Logs) => logs += bundle.item_count,
                Some(SignalType::Metrics) => metrics += bundle.item_count,
                Some(SignalType::Traces) => spans += bundle.item_count,
                None => {}
            }
        }

        // Step 4: evict cache entries for segments no longer tracked.
        self.segment_cache
            .retain(|seq, _| progress_snapshot.contains_key(&SegmentSeq::new(*seq)));
        self.metadata_load_warned_segments
            .retain(|seq| progress_snapshot.contains_key(&SegmentSeq::new(*seq)));
        self.enforce_segment_cache_bound();

        self.metrics.queued_log_records.set(logs);
        self.metrics.queued_metric_points.set(metrics);
        self.metrics.queued_spans.set(spans);
    }

    /// Initialize the Quiver engine and subscriber.
    ///
    /// Per ARCHITECTURE.md, each core has its own Quiver instance with a
    /// core-specific subdirectory to avoid cross-core locking.
    ///
    /// The configured `retention_size_cap` is divided by `num_cores` so that
    /// the total disk usage across all cores stays within the configured limit.
    async fn init_engine(&self) -> Result<(Arc<QuiverEngine>, SubscriberId), Error> {
        let policy = self.config.retention_policy();

        // Divide the total size cap across all cores.
        // Each core gets an equal share of the configured retention_size_cap.
        let total_size_cap = self.config.size_cap_bytes();
        let num_cores = self.num_cores.max(1) as u64;
        let per_core_size_cap = total_size_cap / num_cores;

        // Build the QuiverConfig from our DurableBufferConfig (same helper used
        // in new() for early validation, ensuring the two stay in sync).
        let core_data_dir = self.config.path.join(format!("core_{}", self.core_id));
        let quiver_config = Self::build_quiver_config(&self.config).with_data_dir(&core_data_dir);

        otel_info!(
            "durable_buffer.engine.init",
            path = %core_data_dir.display(),
            total_size_cap = total_size_cap,
            per_core_size_cap = per_core_size_cap,
            policy = ?policy,
            max_segment_open_duration = ?self.config.max_segment_open_duration,
            max_age = ?self.config.max_age
        );

        // Create disk budget with per-core share of the total cap.
        // DiskBudget::for_config() reads segment/WAL sizes directly from the
        // QuiverConfig, ensuring the budget matches the engine configuration.
        let budget = Arc::new(
            DiskBudget::for_config(per_core_size_cap, &quiver_config, policy).map_err(|e| {
                Error::InternalError {
                    message: format!("invalid budget configuration: {e}"),
                }
            })?,
        );

        // Build the Quiver engine
        let engine = QuiverEngine::builder(quiver_config)
            .with_budget(budget)
            .build()
            .await
            .map_err(|e| Error::InternalError {
                message: format!("failed to create Quiver engine: {}", e),
            })?;

        // Register subscriber
        let subscriber_id = SubscriberId::new(SUBSCRIBER_ID).map_err(|e| Error::InternalError {
            message: format!("invalid subscriber ID: {}", e),
        })?;

        engine
            .register_subscriber(subscriber_id.clone())
            .map_err(|e| Error::InternalError {
                message: format!("failed to register subscriber: {}", e),
            })?;

        engine
            .activate_subscriber(&subscriber_id)
            .map_err(|e| Error::InternalError {
                message: format!("failed to activate subscriber: {}", e),
            })?;

        otel_info!(
            "durable_buffer.engine.ready",
            path = %self.config.path.display(),
            subscriber_id = %subscriber_id.as_str()
        );
        Ok((engine, subscriber_id))
    }

    /// Get the engine and subscriber, assuming they're initialized.
    fn engine(&self) -> Result<(&Arc<QuiverEngine>, &SubscriberId), Error> {
        match &self.engine_state {
            EngineState::Ready {
                engine,
                subscriber_id,
            } => Ok((engine, subscriber_id)),
            EngineState::Failed(msg) => Err(Error::InternalError {
                message: msg.clone(),
            }),
            EngineState::Uninitialized => Err(Error::InternalError {
                message: "engine not initialized".to_string(),
            }),
        }
    }

    /// Handle incoming data by ingesting to Quiver storage.
    ///
    /// # Data Flow
    ///
    /// 1. Data is written to Quiver's durable storage (WAL → segment finalization)
    /// 2. Upstream is ACK'd after successful durable write
    /// 3. Data becomes visible to subscribers after segment finalization
    /// 4. Timer tick polls for finalized bundles and forwards downstream
    ///
    /// Segment finalization occurs when:
    /// - Segment reaches target size (default 32MB), or
    /// - Segment has been open for max_open_duration (default 5 seconds), or
    /// - A subsequent ingest triggers the time/size check
    ///
    /// This design ensures proper at-least-once delivery semantics with
    /// ACK tracking through the Quiver subscriber model.
    async fn handle_data(
        &mut self,
        data: OtapPdata,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let (context, payload) = data.into_parts();

        // Capture signal type before consuming the payload
        let signal_type = payload.signal_type();

        // Get the engine reference - NACK with original payload if unavailable
        let engine = match self.engine() {
            Ok((engine, _)) => engine,
            Err(e) => {
                self.metrics.ingest_errors.add(1);
                otel_error!("durable_buffer.engine.unavailable", error = %e);
                let nack_pdata = OtapPdata::new(context, payload);
                effect_handler
                    .notify_nack(NackMsg::new(
                        format!("engine not available: {}", e),
                        nack_pdata,
                    ))
                    .await?;
                return Ok(());
            }
        };

        // Ingest based on payload type and configuration.
        // Adapters preserve the original payload via into_inner() for NACK on failure.
        // Returns (Result, item_count) - item count tracks individual items for all formats.
        let (ingest_result, item_count): (Result<(), _>, u64) = match payload {
            OtapPayload::OtlpBytes(otlp_bytes) => {
                // OTLP bytes: check configuration for handling mode
                match self.config.otlp_handling {
                    OtlpHandling::PassThrough => {
                        // Store as opaque binary for efficient pass-through.
                        // Item count is computed once inside the adapter constructor
                        // (protobuf wire-format scan, no full deserialization) and cached.
                        match OtlpBytesAdapter::new(otlp_bytes) {
                            Ok(adapter) => {
                                let num_items = adapter.cached_item_count();
                                let result = match engine.ingest(&adapter).await {
                                    Ok(()) => Ok(()),
                                    Err(e) => {
                                        Err((e, OtapPayload::OtlpBytes(adapter.into_inner())))
                                    }
                                };
                                (result, num_items)
                            }
                            Err((e, original_bytes)) => {
                                // Adapter creation failed - NACK with original bytes
                                self.metrics.ingest_errors.add(1);
                                otel_error!("durable_buffer.otlp.adapter_failed", error = %e);

                                let nack_pdata =
                                    OtapPdata::new(context, OtapPayload::OtlpBytes(original_bytes));
                                effect_handler
                                    .notify_nack(NackMsg::new(
                                        format!("OTLP adapter creation failed: {}", e),
                                        nack_pdata,
                                    ))
                                    .await?;
                                return Ok(());
                            }
                        }
                    }
                    OtlpHandling::ConvertToArrow => {
                        // Convert to Arrow for queryability.
                        // Clone bytes for NACK on conversion failure (conversion consumes the input).
                        let bytes_for_nack = otlp_bytes.clone();
                        let conversion_result: Result<OtapArrowRecords, _> =
                            OtapPayload::OtlpBytes(otlp_bytes).try_into();
                        match conversion_result {
                            Ok(records) => {
                                // Count items from Arrow data (cheap - just num_rows)
                                let num_items = records.num_items() as u64;
                                let adapter = OtapRecordBundleAdapter::new(records);
                                let result = match engine.ingest(&adapter).await {
                                    Ok(()) => Ok(()),
                                    // Ingest failed: NACK with the Arrow records we tried to store
                                    Err(e) => Err((
                                        e,
                                        OtapPayload::OtapArrowRecords(adapter.into_inner()),
                                    )),
                                };
                                (result, num_items)
                            }
                            Err(e) => {
                                // Conversion failed - NACK with original bytes so upstream can retry
                                self.metrics.ingest_errors.add(1);
                                otel_error!("durable_buffer.otlp.conversion_failed", error = %e);

                                let nack_pdata =
                                    OtapPdata::new(context, OtapPayload::OtlpBytes(bytes_for_nack));
                                effect_handler
                                    .notify_nack(NackMsg::new(
                                        format!("OTLP to Arrow conversion failed: {}", e),
                                        nack_pdata,
                                    ))
                                    .await?;
                                return Ok(());
                            }
                        }
                    }
                }
            }
            OtapPayload::OtapArrowRecords(records) => {
                // Native Arrow data: count items (cheap) and store directly.
                let num_items = records.num_items() as u64;
                let adapter = OtapRecordBundleAdapter::new(records);
                let result = match engine.ingest(&adapter).await {
                    Ok(()) => Ok(()),
                    Err(e) => Err((e, OtapPayload::OtapArrowRecords(adapter.into_inner()))),
                };
                (result, num_items)
            }
        };

        // Handle ingest result
        match ingest_result {
            Ok(()) => {
                // Track consumed items by signal type
                match signal_type {
                    SignalType::Logs => self.metrics.consumed_log_records.add(item_count),
                    SignalType::Metrics => self.metrics.consumed_metric_points.add(item_count),
                    SignalType::Traces => self.metrics.consumed_spans.add(item_count),
                }

                // ACK upstream after successful durable write.
                // Data will be forwarded downstream via timer tick after segment finalization.
                let ack_pdata = OtapPdata::new(context, OtapPayload::empty(signal_type));
                effect_handler.notify_ack(AckMsg::new(ack_pdata)).await?;
                Ok(())
            }
            Err((e, original_payload)) => {
                if e.is_at_capacity() {
                    // Normal backpressure: soft cap exceeded. Count separately
                    // and rate-limit logging to avoid flooding.
                    self.metrics.ingest_backpressure.add(1);
                    let now = Instant::now();
                    let should_log = self
                        .last_backpressure_warn
                        .is_none_or(|last| now.duration_since(last) >= WARN_RATE_LIMIT);
                    if should_log {
                        self.last_backpressure_warn = Some(now);
                        otel_warn!("durable_buffer.ingest.backpressure", error = %e);
                    }
                } else {
                    self.metrics.ingest_errors.add(1);
                    otel_error!("durable_buffer.ingest.failed", error = %e);
                }

                // Preserve original payload so upstream can retry
                let nack_pdata = OtapPdata::new(context, original_payload);
                effect_handler
                    .notify_nack(NackMsg::new(
                        format!("durable buffer failed: {}", e),
                        nack_pdata,
                    ))
                    .await?;
                Ok(())
            }
        }
    }

    /// Handle timer tick by polling Quiver for available bundles.
    ///
    /// This is the primary path for forwarding data downstream. Bundles become
    /// visible here after segment finalization (triggered by size or time thresholds
    /// during ingest).
    ///
    /// To prevent blocking indefinitely when downstream applies backpressure,
    /// this method uses non-blocking sends. When the downstream channel is full,
    /// the bundle is deferred and processing stops, allowing the processor to
    /// receive new incoming data. Deferred bundles will be retried on the next tick.
    async fn handle_timer_tick(
        &mut self,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Flush to finalize any segments that have exceeded their time threshold.
        // Segment finalization only happens during ingest() calls, so if there's
        // a gap between ingests, we need to explicitly flush to make bundles available.
        {
            let (engine, _) = self.engine()?;
            if let Err(e) = engine.flush().await {
                self.metrics.flush_failures.add(1);
                // Rate-limit flush warnings since the timer tick fires every
                // poll_interval (~100ms).
                let now = Instant::now();
                let should_log = self
                    .last_flush_warn
                    .is_none_or(|last| now.duration_since(last) >= WARN_RATE_LIMIT);
                if should_log {
                    self.last_flush_warn = Some(now);
                    otel_warn!("durable_buffer.flush.failed", error = %e);
                }
            }
        }

        // Time-budgeted draining: spend at most 50% of poll_interval processing bundles.
        // This ensures we yield back promptly to handle incoming data, which is typically
        // higher priority than draining historical data (e.g., during recovery from a
        // network outage, new data reflects current system state).
        let drain_budget = self.config.poll_interval / 2;
        let deadline = Instant::now() + drain_budget;
        let mut bundles_processed = 0usize;

        // Track the first skipped bundle to detect when we've cycled through all
        // available bundles without making progress (all are in-flight or scheduled).
        let mut first_skipped: Option<(u64, u32)> = None;

        loop {
            // Check time budget before each bundle to ensure we yield back for new data
            if Instant::now() >= deadline {
                otel_debug!(
                    "durable_buffer.drain.budget_exhausted",
                    bundles_processed = bundles_processed,
                    budget_ms = drain_budget.as_millis()
                );
                break;
            }

            // Check max_in_flight limit to prevent thundering herd
            if !self.can_send_more() {
                otel_debug!(
                    "durable_buffer.drain.at_capacity",
                    bundles_processed = bundles_processed,
                    in_flight = self.pending_bundles.len(),
                    max_in_flight = self.config.max_in_flight
                );
                break;
            }

            // Get engine inside loop to avoid borrow conflict with self in process_bundle_handle
            let poll_result = {
                let (engine, subscriber_id) = self.engine()?;
                engine.poll_next_bundle(subscriber_id)
            };

            match poll_result {
                Ok(Some(handle)) => {
                    let bundle_key = (
                        handle.bundle_ref().segment_seq.raw(),
                        handle.bundle_ref().bundle_index.raw(),
                    );
                    match self.try_process_bundle_handle(handle, effect_handler) {
                        ProcessBundleResult::Sent => {
                            bundles_processed += 1;
                            first_skipped = None; // Reset on progress
                        }
                        ProcessBundleResult::Skipped => {
                            // Bundle was skipped (in-flight or scheduled for retry).
                            // Check if we've cycled back to the first skipped bundle.
                            if first_skipped == Some(bundle_key) {
                                // We've seen this bundle before - all available bundles
                                // are blocked. Break to avoid busy-looping.
                                otel_debug!(
                                    "durable_buffer.drain.all_blocked",
                                    bundles_processed = bundles_processed,
                                    in_flight = self.pending_bundles.len(),
                                    retry_scheduled = self.deferred_retry_state.scheduled_len()
                                );
                                break;
                            }
                            if first_skipped.is_none() {
                                first_skipped = Some(bundle_key);
                            }
                        }
                        ProcessBundleResult::Backpressure => {
                            // Downstream channel is full, stop processing and let the
                            // processor handle other messages (including incoming data)
                            otel_debug!(
                                "durable_buffer.drain.backpressure",
                                bundles_processed = bundles_processed
                            );
                            break;
                        }
                        ProcessBundleResult::Error(e) => {
                            return Err(e);
                        }
                    }
                }
                Ok(None) => {
                    // No more bundles available
                    break;
                }
                Err(e) => {
                    self.metrics.read_errors.add(1);
                    otel_error!("durable_buffer.poll.failed", error = %e);
                    break;
                }
            }
        }

        // Always run maintenance at the end of each tick to:
        // 1. Flush dirty subscriber progress to disk (for crash recovery)
        // 2. Clean up fully-processed segments to reclaim disk space
        // This is essential for steady-state operation - without periodic maintenance,
        // segments accumulate and eventually fill the storage budget.
        {
            let (engine, _) = self.engine()?;
            if let Err(e) = engine.maintain().await {
                otel_warn!("durable_buffer.maintenance.failed", error = %e);
            }
        }

        Ok(())
    }

    /// Process a bundle handle using non-blocking send.
    ///
    /// Returns `ProcessBundleResult::Sent` if the bundle was successfully sent downstream.
    /// Returns `ProcessBundleResult::Skipped` if the bundle was already in-flight.
    /// Returns `ProcessBundleResult::Backpressure` if the downstream channel is full.
    /// Returns `ProcessBundleResult::Error` for other errors.
    fn try_process_bundle_handle(
        &mut self,
        handle: QuiverBundleHandle,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> ProcessBundleResult {
        self.try_process_bundle_handle_with_retry_count(handle, 0, effect_handler)
    }

    /// Process a bundle handle with a specific retry count.
    ///
    /// This is the core send logic, used by both initial sends (retry_count=0)
    /// and retry attempts (retry_count > 0).
    fn try_process_bundle_handle_with_retry_count(
        &mut self,
        handle: QuiverBundleHandle,
        retry_count: u32,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> ProcessBundleResult {
        let bundle_ref = handle.bundle_ref();
        let key = (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw());

        // Skip if this bundle is already in-flight (waiting for ACK/NACK).
        // This should rarely happen since we keep the handle claimed, but can occur
        // if the bundle was previously sent and is still awaiting response.
        if self.pending_bundles.contains_key(&key) {
            // Bundle is in-flight. Dropping the handle will trigger implicit defer,
            // but since we already hold the original handle, this one is a duplicate claim.
            // This shouldn't happen in normal operation since we keep bundles claimed.
            otel_warn!(
                "durable_buffer.bundle.duplicate",
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw()
            );
            drop(handle); // Implicit defer
            return ProcessBundleResult::Skipped;
        }

        // Skip if this bundle is scheduled for retry (waiting for backoff).
        // This enforces the exponential backoff - poll_next_bundle() returns
        // deferred bundles immediately, but we should wait for the retry delay.
        if self.deferred_retry_state.is_deferred_key(key) {
            // Bundle is waiting for backoff. Release the claim; it will be
            // re-claimed when the single durable-buffer retry wakeup resumes it.
            drop(handle); // Implicit defer
            return ProcessBundleResult::Skipped;
        }

        // Convert the reconstructed bundle to OtapPdata
        match convert_bundle_to_pdata(handle.data()) {
            Ok(mut pdata) => {
                // Use the manifest-derived item count carried by the handle.
                // This avoids re-scanning OTLP bytes on the drain path.
                let item_count = handle.item_count();
                let signal_type = pdata.signal_type();

                // Subscribe for ACK/NACK with BundleRef in calldata
                let calldata = encode_bundle_ref(bundle_ref);
                effect_handler.subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    calldata,
                    &mut pdata,
                );

                // Try non-blocking send downstream
                match effect_handler.try_send_message(pdata) {
                    Ok(()) => {
                        // Track produced items by signal type
                        match signal_type {
                            SignalType::Logs => self.metrics.produced_log_records.add(item_count),
                            SignalType::Metrics => {
                                self.metrics.produced_metric_points.add(item_count)
                            }
                            SignalType::Traces => self.metrics.produced_spans.add(item_count),
                        }

                        otel_debug!(
                            "durable_buffer.bundle.forwarded",
                            segment_seq = bundle_ref.segment_seq.raw(),
                            bundle_index = bundle_ref.bundle_index.raw(),
                            retry_count = retry_count
                        );

                        // Store the handle to keep the bundle claimed until ACK/NACK.
                        // The bundle will not be redelivered while we hold the handle.
                        let _ = self.pending_bundles.insert(
                            key,
                            PendingBundle {
                                handle,
                                retry_count,
                                item_count,
                                signal_type,
                            },
                        );
                        self.metrics
                            .in_flight
                            .set(self.pending_bundles.len() as u64);
                        ProcessBundleResult::Sent
                    }
                    Err(otap_df_engine::error::TypedError::ChannelSendError(
                        otap_df_channel::error::SendError::Full(_pdata),
                    )) => {
                        // Channel is full - release the bundle for retry on next tick.
                        // Dropping the handle triggers implicit defer, making the bundle
                        // eligible for redelivery on the next poll.
                        drop(handle);
                        ProcessBundleResult::Backpressure
                    }
                    Err(otap_df_engine::error::TypedError::ChannelSendError(
                        otap_df_channel::error::SendError::Closed(_pdata),
                    )) => {
                        // Channel is closed - this is a fatal error.
                        // Drop the handle to release the claim (data stays in Quiver).
                        drop(handle);
                        ProcessBundleResult::Error(Error::ChannelSendError {
                            error: "downstream channel closed".to_string(),
                        })
                    }
                    Err(otap_df_engine::error::TypedError::Error(e)) => {
                        // Configuration error (no default port) - this is a fatal error
                        drop(handle);
                        ProcessBundleResult::Error(e)
                    }
                    Err(e) => {
                        // Other TypedError variants
                        drop(handle);
                        ProcessBundleResult::Error(e.into())
                    }
                }
            }
            Err(e) => {
                self.metrics.read_errors.add(1);
                otel_error!("durable_buffer.bundle.conversion_failed", error = %e);
                // Reject the bundle since we can't process it
                handle.reject();
                // Conversion error is not counted as "sent" but also shouldn't stop processing
                ProcessBundleResult::Skipped
            }
        }
    }

    /// Handle ACK from downstream.
    async fn handle_ack(
        &mut self,
        ack: AckMsg<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Extract BundleRef from calldata
        let Some(bundle_ref) = decode_bundle_ref(&ack.unwind.route.calldata) else {
            // Invalid calldata, just forward the ACK upstream
            return effect_handler.notify_ack(ack).await;
        };

        let key = (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw());

        // Remove from pending and acknowledge in Quiver using the stored handle
        if let Some(pending) = self.pending_bundles.remove(&key) {
            pending.handle.ack();
            self.metrics.bundles_acked.add(1);
            self.metrics
                .in_flight
                .set(self.pending_bundles.len() as u64);
            otel_debug!(
                "durable_buffer.bundle.acked",
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw()
            );
        } else {
            otel_warn!(
                "durable_buffer.ack.unknown_bundle",
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw()
            );
        }

        Ok(())
    }

    /// Handle NACK from downstream.
    ///
    /// For permanent NACKs (e.g., malformed data that will never succeed), the bundle
    /// is rejected immediately without retry.
    ///
    /// For transient NACKs, defers the bundle locally with exponential backoff
    /// and ensures the single durable-buffer wakeup tracks the earliest retry.
    async fn handle_nack(
        &mut self,
        nack: NackMsg<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Extract BundleRef from calldata
        let Some(bundle_ref) = decode_bundle_ref(&nack.unwind.route.calldata) else {
            // Invalid calldata, just forward the NACK upstream
            return effect_handler.notify_nack(nack).await;
        };

        let key = (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw());

        // Handle based on whether this is a permanent or transient failure
        if let Some(pending) = self.pending_bundles.remove(&key) {
            self.metrics
                .in_flight
                .set(self.pending_bundles.len() as u64);

            // Permanent failures should not be retried - reject the bundle immediately
            if nack.permanent {
                // Track permanently rejected items by signal type (individual items in NACKed bundles)
                match pending.signal_type {
                    SignalType::Logs => self.metrics.rejected_log_records.add(pending.item_count),
                    SignalType::Metrics => {
                        self.metrics.rejected_metric_points.add(pending.item_count)
                    }
                    SignalType::Traces => self.metrics.rejected_spans.add(pending.item_count),
                }
                self.metrics.bundles_nacked_permanent.add(1);

                otel_warn!(
                    "durable_buffer.bundle.rejected_permanent",
                    segment_seq = bundle_ref.segment_seq.raw(),
                    bundle_index = bundle_ref.bundle_index.raw(),
                    reason = %nack.reason
                );

                // Reject the bundle in Quiver (marks as permanently failed)
                pending.handle.reject();
                return Ok(());
            }

            // Transient failure - schedule retry with exponential backoff
            let retry_count = pending.retry_count + 1;

            // Track requeued items by signal type (individual items in NACKed bundles)
            match pending.signal_type {
                SignalType::Logs => self.metrics.requeued_log_records.add(pending.item_count),
                SignalType::Metrics => self.metrics.requeued_metric_points.add(pending.item_count),
                SignalType::Traces => self.metrics.requeued_spans.add(pending.item_count),
            }
            self.metrics.bundles_nacked_deferred.add(1);

            // Calculate backoff delay with jitter
            let backoff = self.calculate_backoff(retry_count);

            otel_debug!(
                "durable_buffer.bundle.nacked",
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw(),
                retry_count = retry_count,
                backoff_ms = backoff.as_millis(),
                reason = %nack.reason
            );

            // Drop the handle to trigger implicit defer, releasing the claim.
            // The bundle stays in Quiver but is now available for re-claiming.
            // Note: No race with TimerTick here - df_engine processes messages
            // sequentially, so schedule_retry completes before any TimerTick runs.
            drop(pending.handle);

            // Schedule the retry
            if self.deferred_retry_state.schedule_after(
                bundle_ref,
                retry_count,
                backoff,
                effect_handler,
            ) {
                self.metrics.retries_scheduled.add(1);
            } else {
                otel_warn!(
                    "durable_buffer.retry.schedule_failed",
                    segment_seq = bundle_ref.segment_seq.raw(),
                    bundle_index = bundle_ref.bundle_index.raw()
                );
            }
        } else {
            otel_warn!(
                "durable_buffer.nack.unknown_bundle",
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw()
            );
        }

        Ok(())
    }

    /// Handle a retry wakeup.
    ///
    /// Re-claims the bundle from Quiver and attempts redelivery downstream.
    async fn handle_retry_wakeup(
        &mut self,
        slot: WakeupSlot,
        revision: WakeupRevision,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if !self.deferred_retry_state.accept_wakeup(slot, revision) {
            otel_warn!(
                "durable_buffer.retry.unknown_wakeup",
                wakeup_slot = slot.0.to_string(),
                wakeup_revision = revision
            );
            return Ok(());
        }

        let mut rearm_no_earlier_than = None;
        loop {
            let now = Instant::now();
            let Some(retry) = self.deferred_retry_state.take_due_retry(now) else {
                break;
            };

            match self.resume_retry(retry.bundle_ref(), retry.retry_count(), effect_handler)? {
                RetryResumeOutcome::Sent | RetryResumeOutcome::Skipped => {}
                RetryResumeOutcome::Deferred => {
                    rearm_no_earlier_than = Some(now + self.config.poll_interval);
                    break;
                }
            }
        }

        if !self
            .deferred_retry_state
            .rearm_after_processing(effect_handler, rearm_no_earlier_than)
        {
            otel_warn!("durable_buffer.retry.rearm_failed");
        }

        Ok(())
    }

    /// Handle shutdown by flushing the Quiver engine and draining remaining bundles.
    ///
    /// The shutdown sequence is:
    /// 1. Flush to finalize any open segment (makes data visible to subscribers)
    /// 2. Clear deferred-retry gating so parked retry bundles become drainable
    /// 3. Drain remaining bundles to downstream (best-effort, respects deadline)
    /// 4. Engine shutdown (always attempted - also finalizes open segment if flush was skipped)
    ///
    /// Note: Quiver's `shutdown()` internally calls `finalize_current_segment()`, so even
    /// if we skip the explicit flush due to deadline pressure, the engine shutdown will
    /// still persist any buffered data. The explicit flush + drain sequence is for
    /// orderly delivery to downstream, not for data durability.
    ///
    /// The deadline is enforced for the drain loop: if we run out of time, we skip
    /// remaining drain iterations and proceed to engine shutdown.
    async fn handle_shutdown(
        &mut self,
        deadline: Instant,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        otel_info!("durable_buffer.shutdown.start", deadline = ?deadline);

        // Only process if engine was initialized
        if !matches!(self.engine_state, EngineState::Ready { .. }) {
            return Ok(());
        }

        // Shutdown is terminal for this processor instance, so retry backoff no
        // longer matters. Clear local deferred-retry gating up front so bundles
        // that were parked behind backoff become drainable through the normal
        // Quiver poll loop below.
        self.deferred_retry_state.clear_for_shutdown();

        // Check deadline before flush/drain sequence
        if Instant::now() >= deadline {
            otel_warn!("durable_buffer.shutdown.deadline_exceeded");
        } else {
            // Flush to finalize any open segment - this makes buffered data visible
            // for the drain loop below. Even if this is skipped, engine.shutdown()
            // will finalize the segment.
            otel_info!("durable_buffer.shutdown.flushing");
            {
                let (engine, _) = self.engine()?;
                if let Err(e) = engine.flush().await {
                    self.metrics.flush_failures.add(1);
                    otel_error!("durable_buffer.shutdown.flush_failed", error = %e);
                }
            }

            // Drain any remaining bundles that became available after flush
            let mut drained = 0u64;
            loop {
                // Check deadline on each iteration
                if Instant::now() >= deadline {
                    otel_warn!(
                        "durable_buffer.shutdown.drain_deadline",
                        bundles_drained = drained
                    );
                    break;
                }

                let poll_result = {
                    let (engine, subscriber_id) = self.engine()?;
                    engine.poll_next_bundle(subscriber_id)
                };

                match poll_result {
                    Ok(Some(handle)) => {
                        match self.try_process_bundle_handle(handle, effect_handler) {
                            ProcessBundleResult::Sent => drained += 1,
                            ProcessBundleResult::Skipped => {
                                // Continue draining
                            }
                            ProcessBundleResult::Backpressure => {
                                // During shutdown, if we hit backpressure just log and continue
                                // The bundle is deferred and will be picked up on next run
                                otel_warn!("durable_buffer.shutdown.backpressure");
                                break;
                            }
                            ProcessBundleResult::Error(e) => {
                                otel_warn!("durable_buffer.shutdown.bundle_error", error = %e);
                            }
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        otel_warn!("durable_buffer.shutdown.poll_error", error = %e);
                        break;
                    }
                }
            }
            if drained > 0 {
                otel_info!("durable_buffer.shutdown.drained", bundles_drained = drained);
            }
        }

        // Always attempt engine shutdown - this finalizes any open segment and
        // performs cleanup. Even if past deadline, this is fast (especially after
        // flush) and critical for data durability.
        {
            let (engine, _) = self.engine()?;
            if let Err(e) = engine.shutdown().await {
                otel_error!("durable_buffer.shutdown.engine_failed", error = %e);
            } else {
                otel_info!("durable_buffer.shutdown.complete");
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Processor Trait Implementation
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait(?Send)]
impl otap_df_engine::local::processor::Processor<OtapPdata> for DurableBuffer {
    fn runtime_requirements(&self) -> ProcessorRuntimeRequirements {
        ProcessorRuntimeRequirements {
            local_wakeups: Some(LocalWakeupRequirements::new(1)),
        }
    }

    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Ensure engine is initialized on first message (of any type)
        if matches!(self.engine_state, EngineState::Uninitialized) {
            self.ensure_engine_initialized().await?;
        }

        // Start timer on first message (whether control or data) if not already started
        if !self.timer_started {
            let _ = effect_handler
                .start_periodic_timer(self.config.poll_interval)
                .await;
            self.timer_started = true;
            otel_debug!(
                "durable_buffer.timer.started",
                poll_interval = ?self.config.poll_interval
            );
        }

        match msg {
            Message::PData(data) => self.handle_data(data, effect_handler).await,
            Message::Control(control_msg) => match control_msg {
                NodeControlMsg::TimerTick { .. } => self.handle_timer_tick(effect_handler).await,
                NodeControlMsg::Ack(ack) => self.handle_ack(ack, effect_handler).await,
                NodeControlMsg::Nack(nack) => self.handle_nack(nack, effect_handler).await,
                NodeControlMsg::Shutdown { deadline, .. } => {
                    self.handle_shutdown(deadline, effect_handler).await
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => {
                    // Collect Quiver storage metrics
                    // (separate scope to avoid borrow conflicts with self)
                    let quiver_metrics = if let Ok((engine, subscriber_id)) = self.engine() {
                        let budget = engine.budget();
                        Some((
                            budget.used(),
                            budget.hard_cap(),
                            engine.force_dropped_segments(),
                            engine.force_dropped_bundles(),
                            engine.force_dropped_items(),
                            engine.expired_bundles(),
                            engine.expired_items(),
                            engine.clone(),
                            subscriber_id.clone(),
                        ))
                    } else {
                        None
                    };

                    if let Some((
                        used,
                        cap,
                        dropped_segs,
                        dropped_buns,
                        dropped_items,
                        expired_buns,
                        expired_items,
                        engine,
                        subscriber_id,
                    )) = quiver_metrics
                    {
                        self.metrics.storage_bytes_used.set(used);
                        self.metrics.storage_bytes_cap.set(cap);
                        self.metrics.dropped_segments.observe(dropped_segs);
                        self.metrics.dropped_bundles.observe(dropped_buns);
                        self.metrics.dropped_items.observe(dropped_items);
                        self.metrics.expired_bundles.observe(expired_buns);
                        self.metrics.expired_items.observe(expired_items);

                        // Recompute queued item gauges from the subscriber
                        // registry. This is the single source of truth for
                        // these gauges, correctly accounting for ACKs,
                        // force-drops, and expiry.
                        self.recompute_queued_counters(&engine, &subscriber_id);
                    }

                    metrics_reporter
                        .report(&mut self.metrics)
                        .map_err(|e| Error::InternalError {
                            message: e.to_string(),
                        })
                }
                NodeControlMsg::Config { config } => {
                    otel_debug!("durable_buffer.config.update", config = ?config);
                    Ok(())
                }
                NodeControlMsg::MemoryPressureChanged { .. } => Ok(()),
                NodeControlMsg::DrainIngress { .. } => Ok(()),
                NodeControlMsg::Wakeup { slot, revision, .. } => {
                    self.handle_retry_wakeup(slot, revision, effect_handler)
                        .await
                }
                NodeControlMsg::DelayedData { .. } => {
                    otel_warn!("durable_buffer.delayed_data.unexpected");
                    Ok(())
                }
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Factory Registration
// ─────────────────────────────────────────────────────────────────────────────

/// Factory function to create a DurableBuffer.
pub fn create_durable_buffer(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: DurableBufferConfig =
        serde_json::from_value(node_config.config.clone()).map_err(|e| {
            ConfigError::InvalidUserConfig {
                error: format!("failed to parse durable buffer configuration: {}", e),
            }
        })?;

    // Create processor with lazy engine initialization
    // The Quiver engine will be initialized on the first message when we're
    // guaranteed to be in an async context
    let processor = DurableBuffer::new(config, &pipeline_ctx)?;

    Ok(ProcessorWrapper::local(
        processor,
        node,
        node_config,
        processor_config,
    ))
}

/// Register DurableBuffer as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static DURABLE_BUFFER_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: DURABLE_BUFFER_URN,
    create: create_durable_buffer,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<DurableBufferConfig>,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Metric field indices matching `DurableBufferMetrics` declaration order.
    // New metrics MUST be appended to the struct so these never shift.
    const IDX_BUNDLES_ACKED: usize = 0;
    const IDX_BUNDLES_NACKED_DEFERRED: usize = 1;
    const IDX_BUNDLES_NACKED_PERMANENT: usize = 2;
    const IDX_RETRIES_SCHEDULED: usize = 22;
    const IDX_QUEUED_LOG_RECORDS: usize = 27;
    const IDX_QUEUED_METRIC_POINTS: usize = 28;
    const IDX_QUEUED_SPANS: usize = 29;
    const EXPECTED_METRIC_COUNT: usize = 31;

    #[test]
    fn test_bundle_ref_encoding_roundtrip() {
        let bundle_ref = BundleRef {
            segment_seq: SegmentSeq::new(12345),
            bundle_index: BundleIndex::new(42),
        };

        let calldata = encode_bundle_ref(bundle_ref);
        let decoded = decode_bundle_ref(&calldata);

        assert!(decoded.is_some());
        let decoded = decoded.unwrap();
        assert_eq!(decoded.segment_seq.raw(), 12345);
        assert_eq!(decoded.bundle_index.raw(), 42);
    }

    #[test]
    fn test_decode_bundle_ref_empty_calldata() {
        let calldata: CallData = smallvec![];
        assert!(decode_bundle_ref(&calldata).is_none());
    }

    #[test]
    fn test_decode_bundle_ref_insufficient_calldata() {
        let calldata: CallData = smallvec![Context8u8::from(123u64)];
        assert!(decode_bundle_ref(&calldata).is_none());
    }

    /// Scenario: one transient NACK arms a normal processor-local wakeup and the
    /// wakeup control message is later delivered through the processor inbox.
    /// Guarantees: the retry stays deferred until the wakeup arrives, and that
    /// wakeup resumes normal downstream delivery exactly once.
    #[test]
    fn test_retry_wakeup_resumes_retry_logic() {
        use otap_df_config::node::NodeUserConfig;
        use otap_df_engine::config::ProcessorConfig;
        use otap_df_engine::context::ControllerContext;
        use otap_df_engine::control::pipeline_completion_msg_channel;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;
        use otap_df_engine::testing::test_node;
        use otap_df_otap::testing::next_nack;
        use otap_df_pdata::encode::encode_logs_otap_batch;
        use otap_df_pdata::testing::fixtures::DataGenerator;
        use serde_json::json;

        let rt = TestRuntime::new();
        let controller = ControllerContext::new(rt.metrics_registry());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
        let temp_dir = tempfile::tempdir().expect("tempdir");

        let mut node_config = NodeUserConfig::new_processor_config(DURABLE_BUFFER_URN);
        node_config.config = json!({
            "path": temp_dir.path(),
            "retention_size_cap": "256 MiB",
            "poll_interval": "100ms",
            "max_segment_open_duration": "1s",
            "initial_retry_interval": "100ms",
            "max_retry_interval": "100ms",
            "retry_multiplier": 2.0,
            "max_in_flight": 1000
        });

        let processor = create_durable_buffer(
            pipeline_ctx,
            test_node("durable-buffer-retry-wakeup"),
            Arc::new(node_config),
            &ProcessorConfig::new("durable-buffer-retry-wakeup"),
        )
        .expect("create durable buffer");

        rt.set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, _pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(1);
                let input = datagen.generate_logs();
                let rec = encode_logs_otap_batch(&input).expect("encode logs");
                ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                    .await
                    .expect("process input");

                ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                    .await
                    .expect("process timer tick");
                let mut outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 1, "timer tick should emit one bundle");

                let sent = outputs.pop().expect("sent bundle");
                let (_, nack) =
                    next_nack(NackMsg::new("retry", sent)).expect("expected nack subscriber");
                ctx.process(Message::Control(NodeControlMsg::Nack(nack)))
                    .await
                    .expect("process nack");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "nack should defer delivery until wakeup"
                );

                ctx.sleep(Duration::from_millis(200)).await;
                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                    slot: RETRY_WAKEUP_SLOT,
                    when: Instant::now(),
                    revision: 0,
                }))
                .await
                .expect("process retry wakeup");

                let retried = ctx.drain_pdata().await;
                assert_eq!(retried.len(), 1, "wakeup should resume retry delivery");
                assert_eq!(retried[0].signal_type(), SignalType::Logs);
            })
            .validate(|_| async {});
    }

    /// Scenario: an unrelated wakeup arrives while durable-buffer has multiple
    /// deferred retries pending behind its single retry wakeup slot.
    /// Guarantees: the unrelated wakeup does not cause early redelivery or lose
    /// deferred retries; the matching wakeup later resumes all due retries.
    #[test]
    fn test_unknown_wakeup_does_not_lose_deferred_retries() {
        use otap_df_config::node::NodeUserConfig;
        use otap_df_engine::config::ProcessorConfig;
        use otap_df_engine::context::ControllerContext;
        use otap_df_engine::control::pipeline_completion_msg_channel;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;
        use otap_df_engine::testing::test_node;
        use otap_df_otap::testing::next_nack;
        use otap_df_pdata::encode::encode_logs_otap_batch;
        use otap_df_pdata::testing::fixtures::DataGenerator;
        use serde_json::json;

        let rt = TestRuntime::new();
        let controller = ControllerContext::new(rt.metrics_registry());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
        let temp_dir = tempfile::tempdir().expect("tempdir");

        let mut node_config = NodeUserConfig::new_processor_config(DURABLE_BUFFER_URN);
        node_config.config = json!({
            "path": temp_dir.path(),
            "retention_size_cap": "256 MiB",
            "poll_interval": "50ms",
            "max_segment_open_duration": "1s",
            "initial_retry_interval": "100ms",
            "max_retry_interval": "100ms",
            "retry_multiplier": 2.0,
            "max_in_flight": 1000
        });

        let processor = create_durable_buffer(
            pipeline_ctx,
            test_node("durable-buffer-unknown-wakeup"),
            Arc::new(node_config),
            &ProcessorConfig::with_channel_capacities("durable-buffer-unknown-wakeup", 1, 100),
        )
        .expect("create durable buffer");

        rt.set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, _pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(2);
                for _ in 0..2 {
                    let input = datagen.generate_logs();
                    let rec = encode_logs_otap_batch(&input).expect("encode logs");
                    ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                        .await
                        .expect("process input");
                }

                ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                    .await
                    .expect("process timer tick");
                let mut outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 2, "timer tick should emit two bundles");
                for sent in outputs.drain(..) {
                    let (_, nack) =
                        next_nack(NackMsg::new("retry", sent)).expect("expected nack subscriber");
                    ctx.process(Message::Control(NodeControlMsg::Nack(nack)))
                        .await
                        .expect("process nack");
                }

                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                    slot: WakeupSlot(999),
                    when: Instant::now(),
                    revision: 0,
                }))
                .await
                .expect("process unknown wakeup");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "unknown wakeup should not redeliver deferred retries"
                );

                ctx.sleep(Duration::from_millis(200)).await;
                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                    slot: RETRY_WAKEUP_SLOT,
                    when: Instant::now(),
                    revision: 0,
                }))
                .await
                .expect("process shared retry wakeup");
                let retried = ctx.drain_pdata().await;
                assert_eq!(
                    retried.len(),
                    2,
                    "matching wakeup should resume all due deferred retries"
                );
            })
            .validate(|_| async {});
    }

    /// Scenario: two transient NACKs occur and both retries share the durable
    /// buffer's single wakeup slot.
    /// Guarantees: no retry is re-delivered before the shared wakeup fires, and
    /// one matching wakeup resumes all due retries.
    #[test]
    fn test_multiple_retries_share_single_wakeup() {
        use otap_df_config::node::NodeUserConfig;
        use otap_df_engine::config::ProcessorConfig;
        use otap_df_engine::context::ControllerContext;
        use otap_df_engine::control::pipeline_completion_msg_channel;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;
        use otap_df_engine::testing::test_node;
        use otap_df_otap::testing::next_nack;
        use otap_df_pdata::encode::encode_logs_otap_batch;
        use otap_df_pdata::testing::fixtures::DataGenerator;
        use serde_json::json;

        let rt = TestRuntime::new();
        let controller = ControllerContext::new(rt.metrics_registry());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
        let temp_dir = tempfile::tempdir().expect("tempdir");

        let mut node_config = NodeUserConfig::new_processor_config(DURABLE_BUFFER_URN);
        node_config.config = json!({
            "path": temp_dir.path(),
            "retention_size_cap": "256 MiB",
            "poll_interval": "50ms",
            "max_segment_open_duration": "1s",
            "initial_retry_interval": "100ms",
            "max_retry_interval": "100ms",
            "retry_multiplier": 2.0,
            "max_in_flight": 1000
        });

        let processor = create_durable_buffer(
            pipeline_ctx,
            test_node("durable-buffer-shared-retry-wakeup"),
            Arc::new(node_config),
            &ProcessorConfig::with_channel_capacities("durable-buffer-shared-retry-wakeup", 1, 100),
        )
        .expect("create durable buffer");

        rt.set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, _pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(2);
                for _ in 0..2 {
                    let input = datagen.generate_logs();
                    let rec = encode_logs_otap_batch(&input).expect("encode logs");
                    ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                        .await
                        .expect("process input");
                }

                ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                    .await
                    .expect("process timer tick");
                let mut outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 2, "timer tick should emit two bundles");
                for sent in outputs.drain(..) {
                    let (_, nack) =
                        next_nack(NackMsg::new("retry", sent)).expect("expected nack subscriber");
                    ctx.process(Message::Control(NodeControlMsg::Nack(nack)))
                        .await
                        .expect("process nack");
                }

                ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                    .await
                    .expect("process immediate timer tick");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "shared wakeup retries should stay deferred until due"
                );

                ctx.sleep(Duration::from_millis(200)).await;
                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                    slot: RETRY_WAKEUP_SLOT,
                    when: Instant::now(),
                    revision: 0,
                }))
                .await
                .expect("process shared retry wakeup");
                let wakeup_retry = ctx.drain_pdata().await;
                assert_eq!(
                    wakeup_retry.len(),
                    2,
                    "shared wakeup should resume all due retry deliveries"
                );
            })
            .validate(|_| async {});
    }

    /// Scenario: a bundle is transiently NACKed, becomes deferred behind the
    /// durable-buffer retry wakeup, and shutdown starts before that wakeup
    /// fires.
    /// Guarantees: shutdown clears deferred-retry gating so the existing drain
    /// loop can forward that parked bundle instead of leaving it restart-dependent.
    #[test]
    fn test_shutdown_drains_deferred_retry_bundle() {
        use otap_df_config::node::NodeUserConfig;
        use otap_df_engine::config::ProcessorConfig;
        use otap_df_engine::context::ControllerContext;
        use otap_df_engine::control::pipeline_completion_msg_channel;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;
        use otap_df_engine::testing::test_node;
        use otap_df_otap::testing::next_nack;
        use otap_df_pdata::encode::encode_logs_otap_batch;
        use otap_df_pdata::testing::fixtures::DataGenerator;
        use serde_json::json;

        let rt = TestRuntime::new();
        let controller = ControllerContext::new(rt.metrics_registry());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
        let temp_dir = tempfile::tempdir().expect("tempdir");

        let mut node_config = NodeUserConfig::new_processor_config(DURABLE_BUFFER_URN);
        node_config.config = json!({
            "path": temp_dir.path(),
            "retention_size_cap": "256 MiB",
            "poll_interval": "100ms",
            "max_segment_open_duration": "1s",
            "initial_retry_interval": "10s",
            "max_retry_interval": "10s",
            "retry_multiplier": 2.0,
            "max_in_flight": 1000
        });

        let processor = create_durable_buffer(
            pipeline_ctx,
            test_node("durable-buffer-shutdown-drain-deferred"),
            Arc::new(node_config),
            &ProcessorConfig::new("durable-buffer-shutdown-drain-deferred"),
        )
        .expect("create durable buffer");

        rt.set_processor(processor)
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, _pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(1);
                let input = datagen.generate_logs();
                let rec = encode_logs_otap_batch(&input).expect("encode logs");
                ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                    .await
                    .expect("process input");

                ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                    .await
                    .expect("process timer tick");
                let mut outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 1, "timer tick should emit one bundle");

                let sent = outputs.pop().expect("sent bundle");
                let (_, nack) =
                    next_nack(NackMsg::new("retry", sent)).expect("expected nack subscriber");
                ctx.process(Message::Control(NodeControlMsg::Nack(nack)))
                    .await
                    .expect("process nack");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "nack should defer delivery until either wakeup or shutdown drain"
                );

                ctx.process(Message::Control(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "shutdown".to_owned(),
                }))
                .await
                .expect("process shutdown");

                let drained = ctx.drain_pdata().await;
                assert_eq!(
                    drained.len(),
                    1,
                    "shutdown drain should forward the deferred retry bundle"
                );
                assert_eq!(drained[0].signal_type(), SignalType::Logs);
            })
            .validate(|_| async {});
    }

    #[test]
    fn test_backoff_calculation() {
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;

        let registry = TelemetryRegistryHandle::default();
        let controller_ctx = ControllerContext::new(registry);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("test".into(), "test".into(), 0, 1, 0);

        let config = DurableBufferConfig {
            path: std::path::PathBuf::from("/tmp/test"),
            retention_size_cap: byte_unit::Byte::from_u64(256 * 1024 * 1024), // 256 MiB (above min for default WAL/segment sizes)
            max_age: None,
            size_cap_policy: SizeCapPolicy::Backpressure,
            poll_interval: Duration::from_millis(100),
            otlp_handling: OtlpHandling::PassThrough,
            max_segment_open_duration: Duration::from_secs(1),
            initial_retry_interval: Duration::from_secs(1),
            max_retry_interval: Duration::from_secs(30),
            retry_multiplier: 2.0,
            max_in_flight: 1000,
        };

        let processor = DurableBuffer::new(config, &pipeline_ctx).unwrap();

        // retry 0: 1s * 2^0 = 1s (with jitter 0.5-1.0x)
        let backoff0 = processor.calculate_backoff(0);
        assert!(backoff0 >= Duration::from_millis(500));
        assert!(backoff0 <= Duration::from_millis(1000));

        // retry 1: 1s * 2^1 = 2s (with jitter 0.5-1.0x)
        let backoff1 = processor.calculate_backoff(1);
        assert!(backoff1 >= Duration::from_millis(1000));
        assert!(backoff1 <= Duration::from_millis(2000));

        // retry 5: 1s * 2^5 = 32s, capped to 30s (with jitter 0.5-1.0x)
        let backoff5 = processor.calculate_backoff(5);
        assert!(backoff5 >= Duration::from_millis(15000)); // 30s * 0.5
        assert!(backoff5 <= Duration::from_millis(30000)); // 30s * 1.0

        // Large retry count: should stay capped at max_retry_interval
        let backoff100 = processor.calculate_backoff(100);
        assert!(backoff100 >= Duration::from_millis(15000));
        assert!(backoff100 <= Duration::from_millis(30000));
    }

    /// Test that DurableBufferMetrics snapshot correctly reports NACK metrics.
    ///
    /// Verifies:
    /// - bundles_nacked_deferred (index 1) and bundles_nacked_permanent (index 2)
    ///   are distinct counters reported at the correct positions
    /// - retries_scheduled (index 22) is correctly reported
    /// - bundles_acked (index 0) is correctly reported
    /// - Snapshot clears values (delta semantics)
    #[test]
    fn test_nack_metrics_snapshot_field_positions() {
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;
        use otap_df_telemetry::reporter::MetricsReporter;

        let registry = TelemetryRegistryHandle::default();
        let controller_ctx = ControllerContext::new(registry);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("test".into(), "test".into(), 0, 1, 0);

        let config = DurableBufferConfig {
            path: std::path::PathBuf::from("/tmp/test-metrics"),
            retention_size_cap: byte_unit::Byte::from_u64(256 * 1024 * 1024), // 256 MiB
            max_age: None,
            size_cap_policy: SizeCapPolicy::Backpressure,
            poll_interval: Duration::from_millis(100),
            otlp_handling: OtlpHandling::PassThrough,
            max_segment_open_duration: Duration::from_secs(1),
            initial_retry_interval: Duration::from_secs(1),
            max_retry_interval: Duration::from_secs(30),
            retry_multiplier: 2.0,
            max_in_flight: 1000,
        };

        let mut processor = DurableBuffer::new(config, &pipeline_ctx).unwrap();

        // Simulate the metric increments that handle_nack would perform
        // for permanent NACKs:
        processor.metrics.bundles_nacked_permanent.add(3);

        // for transient NACKs:
        processor.metrics.bundles_nacked_deferred.add(5);

        // for retries scheduled (only on transient):
        processor.metrics.retries_scheduled.add(5);

        // for ACKs:
        processor.metrics.bundles_acked.add(10);

        // Take a snapshot and verify field positions
        let (metrics_rx, mut reporter) = MetricsReporter::create_new_and_receiver(1);
        reporter.report(&mut processor.metrics).unwrap();
        let snapshot = metrics_rx.try_recv().unwrap();
        let values = snapshot.get_metrics();

        // Verify total metric count matches DurableBufferMetrics field count
        assert_eq!(
            values.len(),
            EXPECTED_METRIC_COUNT,
            "DurableBufferMetrics should have {EXPECTED_METRIC_COUNT} fields, got {}",
            values.len()
        );

        assert_eq!(
            values[IDX_BUNDLES_ACKED].to_u64_lossy(),
            10,
            "bundles_acked should be 10"
        );

        assert_eq!(
            values[IDX_BUNDLES_NACKED_DEFERRED].to_u64_lossy(),
            5,
            "bundles_nacked_deferred should be 5"
        );

        assert_eq!(
            values[IDX_BUNDLES_NACKED_PERMANENT].to_u64_lossy(),
            3,
            "bundles_nacked_permanent should be 3"
        );

        assert_eq!(
            values[IDX_RETRIES_SCHEDULED].to_u64_lossy(),
            5,
            "retries_scheduled should be 5"
        );

        // Verify delta semantics: after snapshot, counters should be cleared.
        // The NACK-related counter values we set above should now be zero.
        // (Gauges may still report due to Gauge::clear semantics, so we verify
        // counters specifically by taking another snapshot.)
        reporter.report(&mut processor.metrics).unwrap_or(());
        if let Ok(snap2) = metrics_rx.try_recv() {
            let vals2 = snap2.get_metrics();
            assert_eq!(
                vals2[IDX_BUNDLES_ACKED].to_u64_lossy(),
                0,
                "bundles_acked should be 0 after reset"
            );
            assert_eq!(
                vals2[IDX_BUNDLES_NACKED_DEFERRED].to_u64_lossy(),
                0,
                "bundles_nacked_deferred should be 0 after reset"
            );
            assert_eq!(
                vals2[IDX_BUNDLES_NACKED_PERMANENT].to_u64_lossy(),
                0,
                "bundles_nacked_permanent should be 0 after reset"
            );
            assert_eq!(
                vals2[IDX_RETRIES_SCHEDULED].to_u64_lossy(),
                0,
                "retries_scheduled should be 0 after reset"
            );
        }
    }

    /// Test that permanent NACKs decrement the `queued_*` gauges.
    ///
    /// The `queued_log_records` (and siblings) gauge tracks items ingested but
    /// not yet resolved (ACKed or rejected). When a bundle is permanently
    /// NACKed, it must be decremented just like an ACK — otherwise the gauge
    /// drifts upward, giving operators a false picture of backlog.
    #[test]
    fn test_permanent_nack_decrements_queued_gauge() {
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;
        use otap_df_telemetry::reporter::MetricsReporter;

        let registry = TelemetryRegistryHandle::default();
        let controller_ctx = ControllerContext::new(registry);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("test".into(), "test".into(), 0, 1, 0);

        let config = DurableBufferConfig {
            path: std::path::PathBuf::from("/tmp/test-queued-gauge"),
            retention_size_cap: byte_unit::Byte::from_u64(256 * 1024 * 1024),
            max_age: None,
            size_cap_policy: SizeCapPolicy::Backpressure,
            poll_interval: Duration::from_millis(100),
            otlp_handling: OtlpHandling::PassThrough,
            max_segment_open_duration: Duration::from_secs(1),
            initial_retry_interval: Duration::from_secs(1),
            max_retry_interval: Duration::from_secs(30),
            retry_multiplier: 2.0,
            max_in_flight: 1000,
        };

        let mut processor = DurableBuffer::new(config, &pipeline_ctx).unwrap();
        let (metrics_rx, mut reporter) = MetricsReporter::create_new_and_receiver(10);

        // Simulate: 100 log records queued (as if 100 items were ingested).
        let mut queued_log_records = 100u64;
        processor.metrics.queued_log_records.set(queued_log_records);

        // Simulate: permanent NACK path decrements by 30 items
        // (mirrors the code in handle_nack when nack.permanent is true)
        queued_log_records = queued_log_records.saturating_sub(30);
        processor.metrics.queued_log_records.set(queued_log_records);

        // Snapshot should show queued_log_records = 70
        reporter.report(&mut processor.metrics).unwrap();
        let snap = metrics_rx.try_recv().unwrap();
        assert_eq!(
            snap.get_metrics()[IDX_QUEUED_LOG_RECORDS].to_u64_lossy(),
            70,
            "queued_log_records should be 70 after decrementing 30 from 100"
        );

        // Simulate: ACK the remaining 70
        queued_log_records = queued_log_records.saturating_sub(70);
        processor.metrics.queued_log_records.set(queued_log_records);

        reporter.report(&mut processor.metrics).unwrap();
        let snap2 = metrics_rx.try_recv().unwrap();
        assert_eq!(
            snap2.get_metrics()[IDX_QUEUED_LOG_RECORDS].to_u64_lossy(),
            0,
            "queued_log_records should be 0 after all items resolved"
        );

        // Verify the same for metrics and spans signals
        let mut queued_metric_points = 50u64;
        processor
            .metrics
            .queued_metric_points
            .set(queued_metric_points);
        queued_metric_points = queued_metric_points.saturating_sub(50);
        processor
            .metrics
            .queued_metric_points
            .set(queued_metric_points);

        let mut queued_spans = 25u64;
        processor.metrics.queued_spans.set(queued_spans);
        queued_spans = queued_spans.saturating_sub(25);
        processor.metrics.queued_spans.set(queued_spans);

        reporter.report(&mut processor.metrics).unwrap();
        let snap3 = metrics_rx.try_recv().unwrap();
        assert_eq!(
            snap3.get_metrics()[IDX_QUEUED_METRIC_POINTS].to_u64_lossy(),
            0,
            "queued_metric_points should be 0"
        );
        assert_eq!(
            snap3.get_metrics()[IDX_QUEUED_SPANS].to_u64_lossy(),
            0,
            "queued_spans should be 0"
        );
    }

    #[tokio::test]
    async fn test_open_segment_included_in_queued_gauge() {
        use std::sync::Arc;
        use std::time::SystemTime;

        use arrow::array::{Int32Array, StringArray};
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;
        use otap_df_telemetry::reporter::MetricsReporter;
        use quiver::record_bundle::{
            BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
        };

        struct SimpleBundle {
            descriptor: BundleDescriptor,
            batch: RecordBatch,
            fingerprint: SchemaFingerprint,
            slot_id: SlotId,
            item_count: u64,
        }

        impl RecordBundle for SimpleBundle {
            fn descriptor(&self) -> &BundleDescriptor {
                &self.descriptor
            }

            fn ingestion_time(&self) -> SystemTime {
                SystemTime::now()
            }

            fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
                if slot == self.slot_id {
                    Some(PayloadRef {
                        schema_fingerprint: self.fingerprint,
                        batch: &self.batch,
                    })
                } else {
                    None
                }
            }

            fn item_count(&self) -> u64 {
                self.item_count
            }
        }

        let registry = TelemetryRegistryHandle::default();
        let controller_ctx = ControllerContext::new(registry);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("test".into(), "test".into(), 0, 1, 0);

        let temp_dir = tempfile::tempdir().expect("tempdir");
        let config = DurableBufferConfig {
            path: temp_dir.path().to_path_buf(),
            retention_size_cap: byte_unit::Byte::from_u64(256 * 1024 * 1024),
            max_age: None,
            size_cap_policy: SizeCapPolicy::Backpressure,
            poll_interval: Duration::from_millis(100),
            otlp_handling: OtlpHandling::PassThrough,
            max_segment_open_duration: Duration::from_secs(60),
            initial_retry_interval: Duration::from_secs(1),
            max_retry_interval: Duration::from_secs(30),
            retry_multiplier: 2.0,
            max_in_flight: 1000,
        };

        let mut processor = DurableBuffer::new(config, &pipeline_ctx).unwrap();
        let (engine, subscriber_id) = processor.init_engine().await.unwrap();

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, true),
        ]));
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3, 4, 5])),
                Arc::new(StringArray::from(vec![
                    Some("a"),
                    Some("b"),
                    Some("c"),
                    Some("d"),
                    Some("e"),
                ])),
            ],
        )
        .expect("valid batch");

        let slot_id = SlotId::new(30); // Logs slot
        let bundle = SimpleBundle {
            descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(slot_id, "Logs")]),
            batch,
            fingerprint: [0x11u8; 32],
            slot_id,
            item_count: 5,
        };

        engine.ingest(&bundle).await.unwrap();

        // Verify the bundle is actually in the open segment before recomputing gauges
        let open_bundles = engine.open_segment_bundle_summaries();
        assert_eq!(
            open_bundles.len(),
            1,
            "bundle should be in open segment and accessible"
        );
        assert_eq!(
            open_bundles[0].item_count, 5,
            "open segment bundle should have correct item count"
        );
        assert!(
            open_bundles[0].slot_ids.contains(&slot_id),
            "open segment bundle should have the logs slot"
        );

        processor.recompute_queued_counters(&engine, &subscriber_id);

        let (metrics_rx, mut reporter) = MetricsReporter::create_new_and_receiver(1);
        reporter.report(&mut processor.metrics).unwrap();
        let snap = metrics_rx.try_recv().unwrap();

        assert_eq!(
            snap.get_metrics()[IDX_QUEUED_LOG_RECORDS].to_u64_lossy(),
            5,
            "queued_log_records gauge should include open-segment items"
        );
    }

    #[test]
    fn test_segment_cache_bound_evicts_oldest_and_warn_marker() {
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;

        let registry = TelemetryRegistryHandle::default();
        let controller_ctx = ControllerContext::new(registry);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("test".into(), "test".into(), 0, 1, 0);

        let config = DurableBufferConfig {
            path: std::path::PathBuf::from("/tmp/test-cache-bound"),
            retention_size_cap: byte_unit::Byte::from_u64(256 * 1024 * 1024),
            max_age: None,
            size_cap_policy: SizeCapPolicy::Backpressure,
            poll_interval: Duration::from_millis(100),
            otlp_handling: OtlpHandling::PassThrough,
            max_segment_open_duration: Duration::from_secs(1),
            initial_retry_interval: Duration::from_secs(1),
            max_retry_interval: Duration::from_secs(30),
            retry_multiplier: 2.0,
            max_in_flight: 1000,
        };

        let mut processor = DurableBuffer::new(config, &pipeline_ctx).unwrap();
        let _ = processor.segment_cache.insert(
            10,
            CachedSegmentMetrics {
                summary: SegmentMetricsSummary {
                    bundles: vec![],
                    total_logs: 0,
                    total_metrics: 0,
                    total_spans: 0,
                },
                last_seen_generation: 1,
            },
        );
        let _ = processor.segment_cache.insert(
            20,
            CachedSegmentMetrics {
                summary: SegmentMetricsSummary {
                    bundles: vec![],
                    total_logs: 0,
                    total_metrics: 0,
                    total_spans: 0,
                },
                last_seen_generation: 2,
            },
        );
        let _ = processor.segment_cache.insert(
            30,
            CachedSegmentMetrics {
                summary: SegmentMetricsSummary {
                    bundles: vec![],
                    total_logs: 0,
                    total_metrics: 0,
                    total_spans: 0,
                },
                last_seen_generation: 3,
            },
        );
        let _ = processor.metadata_load_warned_segments.insert(10);

        processor.enforce_segment_cache_bound_with_limit(2);

        assert_eq!(processor.segment_cache.len(), 2);
        assert!(
            !processor.segment_cache.contains_key(&10),
            "oldest segment should be evicted"
        );
        assert!(
            !processor.metadata_load_warned_segments.contains(&10),
            "evicted segment warning marker should be removed"
        );
        assert!(processor.segment_cache.contains_key(&20));
        assert!(processor.segment_cache.contains_key(&30));
    }
}
