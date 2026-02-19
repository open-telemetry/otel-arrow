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
//! - `Message::Data`: Ingested to storage, ACK sent upstream after WAL fsync
//! - `TimerTick`: Poll storage for bundles, send downstream
//! - `Ack`: Extract BundleRef from calldata, call handle.ack()
//! - `Nack`: Call handle.defer() and schedule retry via delay_data()
//! - `Shutdown`: Flush storage engine
//!
//! # Retry Behavior and Error Handling
//!
//! On NACK from downstream, bundles are retried with exponential backoff until
//! either delivery succeeds or the data is evicted by the configured retention
//! policy (`retention_size_cap` + `drop_oldest`).
//!
//! There is no `max_retries` limit: without machine-readable error classification
//! in NACKs, we cannot distinguish temporary failures (network outage—retry will
//! succeed) from permanent failures (malformed data—retry is futile). A retry
//! limit would cause **data loss** during legitimate extended outages.
//!
//! **Operational guidance:**
//!
//! - Monitor `retries_scheduled` metric to detect persistently failing data
//! - Use `retention_size_cap` to bound storage; `drop_oldest` policy evicts
//!   stuck data when space is needed for new data
//! - `max_in_flight` limit prevents thundering herd after recovery
//!
//! **Future improvement:** When NACK messages carry error categorization
//! (e.g., `retryable: bool` or error codes), we can drop permanently-failed
//! data immediately while still retrying transient failures.

mod bundle_adapter;
mod config;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use linkme::distributed_slice;
use quiver::budget::DiskBudget;
use quiver::segment::SegmentSeq;
use quiver::segment_store::SegmentStore;
use quiver::subscriber::{BundleHandle, BundleIndex, BundleRef, RegistryCallback, SubscriberId};
use quiver::{QuiverConfig, QuiverEngine};
use smallvec::smallvec;

use otap_df_telemetry::{otel_debug, otel_error, otel_info, otel_warn};

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;

use bundle_adapter::{OtapRecordBundleAdapter, OtlpBytesAdapter, convert_bundle_to_pdata};
pub use config::{DurableBufferConfig, OtlpHandling, SizeCapPolicy};

use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::{SignalFormat, SignalType};
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::Context8u8;
use otap_df_engine::control::{AckMsg, CallData, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::local::processor::EffectHandler;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, Interests, ProcessorFactory, ProducerEffectHandlerExtension,
};
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;

/// URN for the durable buffer.
pub const DURABLE_BUFFER_URN: &str = "urn:otel:durable_buffer:processor";

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
/// - Metric set name follows `otelcol.<entity>` pattern
/// - Channel metrics already track bundle send/receive counts
/// - This tracks ACK/NACK status, Arrow item counts, storage, and retries
#[metric_set(name = "otelcol.node.durable_buffer")]
#[derive(Debug, Default, Clone)]
pub struct DurableBufferMetrics {
    // ─── ACK/NACK tracking ──────────────────────────────────────────────────
    // Note: Bundle send/receive counts are tracked by channel metrics.
    // These metrics track downstream acknowledgement status.
    /// Number of bundles acknowledged by downstream.
    #[metric(unit = "{bundle}")]
    pub bundles_acked: Counter<u64>,

    /// Number of bundles rejected (deferred for retry) by downstream.
    #[metric(unit = "{bundle}")]
    pub bundles_nacked: Counter<u64>,

    // ─── Consumed Arrow item metrics (per signal type) ──────────────────────
    /// Number of Arrow log records consumed (ingested to WAL).
    /// OTLP pass-through mode skips counting to avoid parsing overhead.
    #[metric(unit = "{log}")]
    pub consumed_arrow_logs: Counter<u64>,

    /// Number of Arrow metric data points consumed (ingested to WAL).
    /// OTLP pass-through mode skips counting to avoid parsing overhead.
    #[metric(unit = "{metric}")]
    pub consumed_arrow_metrics: Counter<u64>,

    /// Number of Arrow trace spans consumed (ingested to WAL).
    /// OTLP pass-through mode skips counting to avoid parsing overhead.
    #[metric(unit = "{span}")]
    pub consumed_arrow_traces: Counter<u64>,

    // ─── Produced Arrow item metrics (per signal type) ──────────────────────
    /// Number of Arrow log records produced (sent downstream).
    /// OTLP pass-through mode skips counting to avoid parsing overhead.
    #[metric(unit = "{log}")]
    pub produced_arrow_logs: Counter<u64>,

    /// Number of Arrow metric data points produced (sent downstream).
    /// OTLP pass-through mode skips counting to avoid parsing overhead.
    #[metric(unit = "{metric}")]
    pub produced_arrow_metrics: Counter<u64>,

    /// Number of Arrow trace spans produced (sent downstream).
    /// OTLP pass-through mode skips counting to avoid parsing overhead.
    #[metric(unit = "{span}")]
    pub produced_arrow_traces: Counter<u64>,

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
    pub dropped_segments: Gauge<u64>,

    /// Total bundles lost due to force-dropped segments (DropOldest policy).
    /// Non-zero values indicate data loss.
    #[metric(unit = "{bundle}")]
    pub dropped_bundles: Gauge<u64>,

    // ─── Retry metrics ──────────────────────────────────────────────────────
    /// Number of retry attempts scheduled.
    #[metric(unit = "{retry}")]
    pub retries_scheduled: Counter<u64>,

    /// Current number of bundles in-flight to downstream.
    #[metric(unit = "{bundle}")]
    pub in_flight: Gauge<u64>,
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

/// Encode a retry ticket into CallData for DelayedData scheduling.
///
/// Layout: [segment_seq (u64), bundle_index (u32), retry_count (u32) packed into u64]
fn encode_retry_ticket(bundle_ref: BundleRef, retry_count: u32) -> CallData {
    // Pack bundle_index (low 32 bits) and retry_count (high 32 bits) into one u64
    let packed = (bundle_ref.bundle_index.raw() as u64) | ((retry_count as u64) << 32);
    smallvec![
        Context8u8::from(bundle_ref.segment_seq.raw()),
        Context8u8::from(packed),
    ]
}

/// Decode a retry ticket from CallData.
///
/// Returns (BundleRef, retry_count) if valid.
fn decode_retry_ticket(calldata: &CallData) -> Option<(BundleRef, u32)> {
    if calldata.len() < 2 {
        return None;
    }
    let segment_seq = SegmentSeq::new(u64::from(calldata[0]));
    let packed = u64::from(calldata[1]);
    let bundle_index = BundleIndex::new((packed & 0xFFFF_FFFF) as u32);
    let retry_count = (packed >> 32) as u32;
    Some((
        BundleRef {
            segment_seq,
            bundle_index,
        },
        retry_count,
    ))
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

/// Durable buffer that provides crash-resilient buffering via Quiver.
pub struct DurableBuffer {
    /// The Quiver engine state (lazy initialized on first message).
    engine_state: EngineState,

    /// Map of in-flight bundles awaiting downstream ACK/NACK.
    /// Key is the (segment_seq, bundle_index) pair encoded as a u128 for fast lookup.
    pending_bundles: HashMap<(u64, u32), PendingBundle>,

    /// Bundles scheduled for retry via delay_data.
    /// These are skipped by poll_next_bundle to enforce backoff.
    /// Removed when the delay fires and claim_bundle is called.
    retry_scheduled: HashSet<(u64, u32)>,

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
            retry_scheduled: HashSet::new(),
            config,
            core_id,
            num_cores,
            metrics,
            timer_started: false,
            last_flush_warn: None,
            last_backpressure_warn: None,
        })
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

    /// Schedule a retry for a bundle via delay_data.
    ///
    /// This is the single point of coordination between `delay_data` scheduling
    /// and `retry_scheduled` tracking. Always use this method instead of calling
    /// `delay_data` directly to ensure the two stay in sync.
    ///
    /// Returns true if scheduling succeeded, false if it failed (caller should
    /// let poll_next_bundle pick up the bundle instead).
    async fn schedule_retry(
        &mut self,
        bundle_ref: BundleRef,
        retry_count: u32,
        delay: Duration,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> bool {
        let key = (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw());

        // Create a lightweight retry ticket
        // TODO(#1472): Replace with proper timer support when available.
        // Currently we abuse delay_data() with an empty payload as a workaround
        // for the lack of a native "schedule callback" primitive.
        let retry_ticket = OtapPdata::new(
            Default::default(),
            OtapPayload::empty(SignalType::Traces), // Signal type doesn't matter for empty payload
        );
        let calldata = encode_retry_ticket(bundle_ref, retry_count);
        let mut retry_ticket = Box::new(retry_ticket);
        effect_handler.subscribe_to(Interests::empty(), calldata, &mut retry_ticket);

        let retry_at = Instant::now() + delay;
        if effect_handler
            .delay_data(retry_at, retry_ticket)
            .await
            .is_ok()
        {
            // Track that this bundle is scheduled - poll_next_bundle will skip it
            let _ = self.retry_scheduled.insert(key);
            true
        } else {
            // Failed to schedule - don't add to retry_scheduled, poll will pick it up
            false
        }
    }

    /// Remove a bundle from retry_scheduled tracking.
    ///
    /// Call this when the delay has fired and we're about to process the retry.
    fn unschedule_retry(&mut self, bundle_ref: BundleRef) {
        let key = (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw());
        let _ = self.retry_scheduled.remove(&key);
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

    /// Handle incoming data by ingesting to Quiver's WAL.
    ///
    /// # Data Flow
    ///
    /// 1. Data is written to Quiver's WAL for durability
    /// 2. Upstream is ACK'd after successful WAL write
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
        // Returns (Result, Option<item_count>) - item count is only available for Arrow data.
        let (ingest_result, item_count) = match payload {
            OtapPayload::OtlpBytes(otlp_bytes) => {
                // OTLP bytes: check configuration for handling mode
                match self.config.otlp_handling {
                    OtlpHandling::PassThrough => {
                        // Store as opaque binary for efficient pass-through.
                        // Skip item counting to avoid parsing overhead.
                        match OtlpBytesAdapter::new(otlp_bytes) {
                            Ok(adapter) => {
                                let result = match engine.ingest(&adapter).await {
                                    Ok(()) => Ok(()),
                                    Err(e) => {
                                        Err((e, OtapPayload::OtlpBytes(adapter.into_inner())))
                                    }
                                };
                                (result, None) // No item count for pass-through
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
                                (result, Some(num_items))
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
                (result, Some(num_items))
            }
        };

        // Handle ingest result
        match ingest_result {
            Ok(()) => {
                // Track consumed Arrow items by signal type
                if let Some(num_items) = item_count {
                    match signal_type {
                        SignalType::Logs => self.metrics.consumed_arrow_logs.add(num_items),
                        SignalType::Metrics => self.metrics.consumed_arrow_metrics.add(num_items),
                        SignalType::Traces => self.metrics.consumed_arrow_traces.add(num_items),
                    }
                }

                // ACK upstream after successful WAL write.
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
                                    retry_scheduled = self.retry_scheduled.len()
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
        // deferred bundles immediately, but we should wait for delay_data to fire.
        if self.retry_scheduled.contains(&key) {
            // Bundle is waiting for backoff. Release the claim; it will be
            // re-claimed when the delay_data retry ticket fires.
            drop(handle); // Implicit defer
            return ProcessBundleResult::Skipped;
        }

        // Convert the reconstructed bundle to OtapPdata
        match convert_bundle_to_pdata(handle.data()) {
            Ok(mut pdata) => {
                // Get item count for Arrow data (cheap); skip for OTLP bytes (expensive)
                let item_count = match pdata.signal_format() {
                    SignalFormat::OtapRecords => Some(pdata.num_items() as u64),
                    SignalFormat::OtlpBytes => None,
                };
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
                        // Track produced Arrow items by signal type
                        if let Some(num_items) = item_count {
                            match signal_type {
                                SignalType::Logs => self.metrics.produced_arrow_logs.add(num_items),
                                SignalType::Metrics => {
                                    self.metrics.produced_arrow_metrics.add(num_items)
                                }
                                SignalType::Traces => {
                                    self.metrics.produced_arrow_traces.add(num_items)
                                }
                            }
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
        let Some(bundle_ref) = decode_bundle_ref(&ack.calldata) else {
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
    /// Schedules a retry with exponential backoff using `delay_data()`.
    /// The bundle is deferred in Quiver (releasing the claim) and a lightweight
    /// retry ticket is scheduled. When the delay expires, `handle_delayed_retry`
    /// will re-claim the bundle and attempt redelivery.
    async fn handle_nack(
        &mut self,
        nack: NackMsg<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Extract BundleRef from calldata
        let Some(bundle_ref) = decode_bundle_ref(&nack.calldata) else {
            // Invalid calldata, just forward the NACK upstream
            return effect_handler.notify_nack(nack).await;
        };

        let key = (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw());

        // Handle retry scheduling
        if let Some(pending) = self.pending_bundles.remove(&key) {
            let retry_count = pending.retry_count + 1;
            self.metrics.bundles_nacked.add(1);
            self.metrics
                .in_flight
                .set(self.pending_bundles.len() as u64);

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
            if self
                .schedule_retry(bundle_ref, retry_count, backoff, effect_handler)
                .await
            {
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

    /// Handle a delayed retry ticket.
    ///
    /// Re-claims the bundle from Quiver and attempts redelivery downstream.
    async fn handle_delayed_retry(
        &mut self,
        retry_ticket: Box<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Decode the retry ticket
        let Some(calldata) = retry_ticket.source_calldata() else {
            otel_warn!("durable_buffer.retry.missing_calldata");
            return Ok(());
        };

        let Some((bundle_ref, retry_count)) = decode_retry_ticket(&calldata) else {
            otel_warn!("durable_buffer.retry.invalid_calldata");
            return Ok(());
        };

        // Check max_in_flight limit
        if !self.can_send_more() {
            // At capacity - re-schedule with a short delay.
            // Bundle stays in retry_scheduled (wasn't removed yet).
            otel_debug!(
                "durable_buffer.retry.deferred",
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw(),
                in_flight = self.pending_bundles.len(),
                max_in_flight = self.config.max_in_flight
            );

            // Re-schedule - note: bundle is still in retry_scheduled, schedule_retry
            // will just update it (insert is idempotent for HashSet)
            if !self
                .schedule_retry(
                    bundle_ref,
                    retry_count,
                    self.config.poll_interval,
                    effect_handler,
                )
                .await
            {
                // Failed to re-schedule - remove from retry_scheduled so poll can pick it up
                self.unschedule_retry(bundle_ref);
                otel_warn!("durable_buffer.retry.reschedule_failed");
            }
            return Ok(());
        }

        // Backoff period has elapsed and we have capacity - remove from retry_scheduled.
        // This allows poll_next_bundle to see it again if claim_bundle fails.
        self.unschedule_retry(bundle_ref);

        // Re-claim the bundle from Quiver
        let claim_result = {
            let (engine, subscriber_id) = self.engine()?;
            engine.claim_bundle(subscriber_id, bundle_ref)
        };

        match claim_result {
            Ok(handle) => {
                // Successfully re-claimed, now send downstream
                match self.try_process_bundle_handle_with_retry_count(
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
                    }
                    ProcessBundleResult::Skipped => {
                        // Shouldn't happen - we just claimed it and removed from retry_scheduled
                        otel_warn!(
                            "durable_buffer.retry.skipped",
                            segment_seq = bundle_ref.segment_seq.raw(),
                            bundle_index = bundle_ref.bundle_index.raw()
                        );
                    }
                    ProcessBundleResult::Backpressure => {
                        // Channel full - the handle was dropped (deferred).
                        // Re-schedule retry with a short delay.
                        otel_debug!(
                            "durable_buffer.retry.backpressure",
                            segment_seq = bundle_ref.segment_seq.raw(),
                            bundle_index = bundle_ref.bundle_index.raw()
                        );

                        // Short delay for backpressure (not exponential - this isn't a failure).
                        // If scheduling fails, poll will pick it up.
                        let _ = self
                            .schedule_retry(
                                bundle_ref,
                                retry_count,
                                self.config.poll_interval,
                                effect_handler,
                            )
                            .await;
                    }
                    ProcessBundleResult::Error(e) => {
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                // Claim failed - bundle may have been resolved or segment dropped
                otel_debug!(
                    "durable_buffer.retry.claim_failed",
                    segment_seq = bundle_ref.segment_seq.raw(),
                    bundle_index = bundle_ref.bundle_index.raw(),
                    error = %e
                );
            }
        }

        Ok(())
    }

    /// Handle shutdown by flushing the Quiver engine and draining remaining bundles.
    ///
    /// The shutdown sequence is:
    /// 1. Flush to finalize any open segment (makes data visible to subscribers)
    /// 2. Drain remaining bundles to downstream (best-effort, respects deadline)
    /// 3. Engine shutdown (always attempted - also finalizes open segment if flush was skipped)
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
                    // Collect Quiver storage metrics (separate scope to avoid borrow conflicts)
                    let quiver_metrics = if let Ok((engine, _)) = self.engine() {
                        let budget = engine.budget();
                        Some((
                            budget.used(),
                            budget.hard_cap(),
                            engine.force_dropped_segments(),
                            engine.force_dropped_bundles(),
                        ))
                    } else {
                        None
                    };

                    // Update metrics from collected values
                    if let Some((used, cap, dropped_segs, dropped_buns)) = quiver_metrics {
                        self.metrics.storage_bytes_used.set(used);
                        self.metrics.storage_bytes_cap.set(cap);
                        self.metrics.dropped_segments.set(dropped_segs);
                        self.metrics.dropped_bundles.set(dropped_buns);
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
                NodeControlMsg::DelayedData { data, .. } => {
                    // Check if this is a retry ticket (has BundleRef + retry_count in calldata)
                    if let Some(calldata) = data.source_calldata() {
                        if decode_retry_ticket(&calldata).is_some() {
                            // This is a retry ticket - handle retry
                            return self.handle_delayed_retry(data, effect_handler).await;
                        }
                    }
                    // Not a retry ticket - shouldn't happen, but handle gracefully
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

    #[test]
    fn test_retry_ticket_encoding_roundtrip() {
        let bundle_ref = BundleRef {
            segment_seq: SegmentSeq::new(98765),
            bundle_index: BundleIndex::new(123),
        };
        let retry_count = 7u32;

        let calldata = encode_retry_ticket(bundle_ref, retry_count);
        let decoded = decode_retry_ticket(&calldata);

        assert!(decoded.is_some());
        let (decoded_ref, decoded_count) = decoded.unwrap();
        assert_eq!(decoded_ref.segment_seq.raw(), 98765);
        assert_eq!(decoded_ref.bundle_index.raw(), 123);
        assert_eq!(decoded_count, 7);
    }

    #[test]
    fn test_retry_ticket_encoding_max_values() {
        let bundle_ref = BundleRef {
            segment_seq: SegmentSeq::new(u64::MAX),
            bundle_index: BundleIndex::new(u32::MAX),
        };
        let retry_count = u32::MAX;

        let calldata = encode_retry_ticket(bundle_ref, retry_count);
        let decoded = decode_retry_ticket(&calldata);

        assert!(decoded.is_some());
        let (decoded_ref, decoded_count) = decoded.unwrap();
        assert_eq!(decoded_ref.segment_seq.raw(), u64::MAX);
        assert_eq!(decoded_ref.bundle_index.raw(), u32::MAX);
        assert_eq!(decoded_count, u32::MAX);
    }

    #[test]
    fn test_decode_retry_ticket_empty_calldata() {
        let calldata: CallData = smallvec![];
        assert!(decode_retry_ticket(&calldata).is_none());
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
            retention_size_cap: byte_unit::Byte::from_u64(256 * 1024 * 1024), // 256 MB (above min for default WAL/segment sizes)
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
}
