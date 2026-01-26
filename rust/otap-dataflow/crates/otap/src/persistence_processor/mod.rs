// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Persistence processor for durable buffering of OTAP data via Quiver.
//!
//! This processor provides crash-resilient persistence by writing incoming
//! telemetry data to Quiver's write-ahead log and segment storage before
//! forwarding downstream. On acknowledgement from downstream, the data is
//! marked as consumed in Quiver; on rejection, data can be replayed.
//!
//! # Architecture
//!
//! ```text
//! Upstream → PersistenceProcessor → Downstream
//!                    ↓
//!              QuiverEngine
//!                    ↓
//!            WAL + Segments
//! ```
//!
//! # Per-Core Isolation
//!
//! Each processor instance (one per CPU core) has its own isolated Quiver engine
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
//! - `Message::Data`: Ingested to Quiver, ACK sent upstream after WAL fsync
//! - `TimerTick`: Poll Quiver for bundles, send downstream
//! - `Ack`: Extract BundleRef from calldata, call handle.ack()
//! - `Nack`: Call handle.defer() and schedule retry via delay_data()
//! - `Shutdown`: Flush Quiver engine

mod bundle_adapter;
mod config;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use linkme::distributed_slice;
use quiver::budget::DiskBudget;
use quiver::segment::SegmentSeq;
use quiver::segment_store::SegmentStore;
use quiver::subscriber::{BundleHandle, BundleIndex, BundleRef, RegistryCallback, SubscriberId};
use quiver::{QuiverConfig, QuiverEngine};
use smallvec::smallvec;
use tracing::{debug, error, info, warn};

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;

use bundle_adapter::{OtapRecordBundleAdapter, OtlpBytesAdapter, convert_bundle_to_pdata};
pub use config::{OtlpHandling, PersistenceProcessorConfig, SizeCapPolicy};

use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
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
use otap_df_pdata::OtapPayload;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;

/// URN for the persistence processor.
pub const PERSISTENCE_PROCESSOR_URN: &str = "urn:otap:processor:persistence";

/// Subscriber ID used by this processor.
const SUBSCRIBER_ID: &str = "persistence-processor";

// ─────────────────────────────────────────────────────────────────────────────
// Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Metrics for the persistence processor.
///
/// Follows RFC-aligned telemetry conventions:
/// - Metric set name follows `otelcol.<entity>` pattern
/// - Tracks both bundles and individual items (spans, data points, log records)
/// - Per-signal breakdown for consumed and produced items
#[metric_set(name = "otelcol.node.persistence")]
#[derive(Debug, Default, Clone)]
pub struct PersistenceProcessorMetrics {
    // ─── Bundle-level metrics ───────────────────────────────────────────────
    /// Number of bundles ingested to Quiver.
    #[metric(unit = "{bundle}")]
    pub bundles_ingested: Counter<u64>,

    /// Number of bundles sent downstream.
    #[metric(unit = "{bundle}")]
    pub bundles_sent: Counter<u64>,

    /// Number of bundles acknowledged by downstream.
    #[metric(unit = "{bundle}")]
    pub bundles_acked: Counter<u64>,

    /// Number of bundles rejected (deferred for retry) by downstream.
    #[metric(unit = "{bundle}")]
    pub bundles_nacked: Counter<u64>,

    // ─── Consumed item metrics (per signal type) ────────────────────────────
    /// Number of log records consumed (ingested to WAL).
    #[metric(unit = "{log}")]
    pub consumed_items_logs: Counter<u64>,

    /// Number of metric data points consumed (ingested to WAL).
    #[metric(unit = "{metric}")]
    pub consumed_items_metrics: Counter<u64>,

    /// Number of trace spans consumed (ingested to WAL).
    #[metric(unit = "{span}")]
    pub consumed_items_traces: Counter<u64>,

    // ─── Produced item metrics (per signal type) ────────────────────────────
    /// Number of log records produced (sent downstream).
    #[metric(unit = "{log}")]
    pub produced_items_logs: Counter<u64>,

    /// Number of metric data points produced (sent downstream).
    #[metric(unit = "{metric}")]
    pub produced_items_metrics: Counter<u64>,

    /// Number of trace spans produced (sent downstream).
    #[metric(unit = "{span}")]
    pub produced_items_traces: Counter<u64>,

    // ─── Error and backpressure metrics ─────────────────────────────────────
    /// Number of ingest errors.
    #[metric(unit = "{error}")]
    pub ingest_errors: Counter<u64>,

    /// Number of read errors.
    #[metric(unit = "{error}")]
    pub read_errors: Counter<u64>,

    // ─── Quiver storage metrics (updated on telemetry collection) ───────────
    /// Current bytes used by Quiver storage (WAL + segments).
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
}

// ─────────────────────────────────────────────────────────────────────────────
// BundleRef CallData Encoding
// ─────────────────────────────────────────────────────────────────────────────

/// Encode a BundleRef into CallData.
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
}

/// Result of attempting to process a bundle with non-blocking send.
enum ProcessBundleResult {
    /// Bundle was successfully sent downstream.
    Sent,
    /// Bundle was skipped (already in-flight, waiting for ACK/NACK).
    Skipped,
    /// The downstream channel is full (backpressure).
    /// The bundle has been deferred and will be retried on the next tick.
    Backpressure,
    /// An unrecoverable error occurred.
    Error(Error),
}

// ─────────────────────────────────────────────────────────────────────────────
// PersistenceProcessor
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

/// Persistence processor that provides durable buffering via Quiver.
pub struct PersistenceProcessor {
    /// The Quiver engine state (lazy initialized on first message).
    engine_state: EngineState,

    /// Map of in-flight bundles awaiting downstream ACK/NACK.
    /// Key is the (segment_seq, bundle_index) pair encoded as a u128 for fast lookup.
    pending_bundles: HashMap<(u64, u32), PendingBundle>,

    /// Configuration.
    config: PersistenceProcessorConfig,

    /// Core ID for per-core data directory.
    /// Per ARCHITECTURE.md, each core has its own Quiver instance.
    core_id: usize,

    /// Metrics.
    metrics: MetricSet<PersistenceProcessorMetrics>,

    /// Whether timer has been started.
    timer_started: bool,
}

impl PersistenceProcessor {
    /// Creates a new persistence processor with the given configuration.
    ///
    /// Note: The Quiver engine is lazily initialized on the first message
    /// to ensure we're running within an async context.
    pub fn new(
        config: PersistenceProcessorConfig,
        pipeline_ctx: &PipelineContext,
    ) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<PersistenceProcessorMetrics>();
        let core_id = pipeline_ctx.core_id();

        Ok(Self {
            engine_state: EngineState::Uninitialized,
            pending_bundles: HashMap::new(),
            config,
            core_id,
            metrics,
            timer_started: false,
        })
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
    async fn init_engine(&self) -> Result<(Arc<QuiverEngine>, SubscriberId), Error> {
        let size_cap = self.config.size_cap_bytes();
        let policy = self.config.retention_policy();

        // Create per-core data directory: {base_path}/core_{core_id}
        let core_data_dir = self.config.path.join(format!("core_{}", self.core_id));

        info!(
            path = %core_data_dir.display(),
            core_id = self.core_id,
            size_cap = size_cap,
            policy = ?policy,
            max_segment_open_duration = ?self.config.max_segment_open_duration,
            "initializing Quiver engine"
        );

        // Create Quiver configuration with per-core data directory
        let mut quiver_config = QuiverConfig::default().with_data_dir(&core_data_dir);
        quiver_config.segment.max_open_duration = self.config.max_segment_open_duration;

        // Create disk budget
        let budget = Arc::new(DiskBudget::new(size_cap, policy));

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

        info!(
            path = %self.config.path.display(),
            subscriber_id = %subscriber_id.as_str(),
            "Quiver engine initialized successfully"
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
        // Ensure engine is initialized - NACK if initialization fails
        if let Err(e) = self.ensure_engine_initialized().await {
            self.metrics.ingest_errors.add(1);
            error!(error = %e, "engine initialization failed");
            effect_handler
                .notify_nack(NackMsg::new(format!("engine not available: {}", e), data))
                .await?;
            return Ok(());
        }

        let (context, payload) = data.into_parts();

        // Capture signal type and item count before consuming the payload
        let signal_type = payload.signal_type();
        let num_items = payload.num_items() as u64;

        // Get the engine reference - NACK if unavailable
        let engine = match self.engine() {
            Ok((engine, _)) => engine,
            Err(e) => {
                self.metrics.ingest_errors.add(1);
                error!(error = %e, "failed to get engine reference");
                let nack_pdata = OtapPdata::new(context, OtapPayload::empty(signal_type));
                effect_handler
                    .notify_nack(NackMsg::new(
                        format!("engine not available: {}", e),
                        nack_pdata,
                    ))
                    .await?;
                return Ok(());
            }
        };

        // Ingest based on payload type and configuration
        let ingest_result = match payload {
            OtapPayload::OtlpBytes(otlp_bytes) => {
                // OTLP bytes: check configuration for handling mode
                match self.config.otlp_handling {
                    OtlpHandling::PassThrough => {
                        // Store as opaque binary for efficient pass-through
                        let adapter = OtlpBytesAdapter::new(otlp_bytes);
                        engine.ingest(&adapter).await
                    }
                    OtlpHandling::ConvertToArrow => {
                        // Convert to Arrow for queryability
                        match OtapPayload::OtlpBytes(otlp_bytes).try_into() {
                            Ok(records) => {
                                let adapter = OtapRecordBundleAdapter::new(records);
                                engine.ingest(&adapter).await
                            }
                            Err(e) => {
                                // Conversion failed - NACK upstream so they can retry or handle
                                self.metrics.ingest_errors.add(1);
                                error!(error = %e, "failed to convert OTLP to Arrow");

                                let nack_pdata =
                                    OtapPdata::new(context, OtapPayload::empty(signal_type));
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
                // Native Arrow data: store directly
                let adapter = OtapRecordBundleAdapter::new(records);
                engine.ingest(&adapter).await
            }
        };

        // Handle ingest result
        match ingest_result {
            Ok(()) => {
                self.metrics.bundles_ingested.add(1);

                // Track consumed items by signal type
                match signal_type {
                    SignalType::Logs => self.metrics.consumed_items_logs.add(num_items),
                    SignalType::Metrics => self.metrics.consumed_items_metrics.add(num_items),
                    SignalType::Traces => self.metrics.consumed_items_traces.add(num_items),
                }

                // ACK upstream after successful WAL write.
                // Data will be forwarded downstream via timer tick after segment finalization.
                let ack_pdata = OtapPdata::new(context, OtapPayload::empty(signal_type));
                effect_handler.notify_ack(AckMsg::new(ack_pdata)).await?;
                Ok(())
            }
            Err(e) => {
                self.metrics.ingest_errors.add(1);
                error!(error = %e, "failed to ingest bundle to Quiver");

                let nack_pdata = OtapPdata::new(context, OtapPayload::empty(signal_type));
                effect_handler
                    .notify_nack(NackMsg::new(
                        format!("persistence failed: {}", e),
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
        // Ensure engine is initialized before polling
        self.ensure_engine_initialized().await?;

        // Flush to finalize any segments that have exceeded their time threshold.
        // Segment finalization only happens during ingest() calls, so if there's
        // a gap between ingests, we need to explicitly flush to make bundles available.
        {
            let (engine, _) = self.engine()?;
            if let Err(e) = engine.flush().await {
                warn!(error = %e, "failed to flush engine during timer tick");
            }
        }

        // Time-budgeted draining: spend at most 50% of poll_interval processing bundles.
        // This ensures we yield back promptly to handle incoming data, which is typically
        // higher priority than draining historical data (e.g., during recovery from a
        // network outage, new data reflects current system state).
        let drain_budget = self.config.poll_interval / 2;
        let deadline = Instant::now() + drain_budget;
        let mut bundles_processed = 0usize;

        loop {
            // Check time budget before each bundle to ensure we yield back for new data
            if Instant::now() >= deadline {
                debug!(
                    bundles_processed = bundles_processed,
                    budget_ms = drain_budget.as_millis(),
                    "drain time budget exhausted, deferring remaining to next tick"
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
                    match self.try_process_bundle_handle(handle, effect_handler) {
                        ProcessBundleResult::Sent => {
                            bundles_processed += 1;
                        }
                        ProcessBundleResult::Skipped => {
                            // Bundle was already in-flight (waiting for ACK/NACK).
                            // Break out of the loop - we need to wait for downstream
                            // to respond before sending more bundles.
                            // This prevents busy-looping when all available bundles
                            // are already pending.
                            break;
                        }
                        ProcessBundleResult::Backpressure => {
                            // Downstream channel is full, stop processing and let the
                            // processor handle other messages (including incoming data)
                            debug!(
                                bundles_processed = bundles_processed,
                                "downstream backpressure detected, deferring remaining bundles"
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
                    error!(error = %e, "failed to poll for bundle");
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
                warn!(error = %e, "maintenance error");
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
        let bundle_ref = handle.bundle_ref();
        let key = (bundle_ref.segment_seq.raw(), bundle_ref.bundle_index.raw());

        // Skip if this bundle is already in-flight (waiting for ACK/NACK).
        // This should rarely since we keep the handle claimed, but can occur
        // if the bundle was previously sent and is still awaiting response.
        if self.pending_bundles.contains_key(&key) {
            // Bundle is in-flight. Dropping the handle will trigger implicit defer,
            // but since we already hold the original handle, this one is a duplicate claim.
            // This shouldn't happen in normal operation since we keep bundles claimed.
            warn!(
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw(),
                "received duplicate handle for in-flight bundle"
            );
            drop(handle); // Implicit defer
            return ProcessBundleResult::Skipped;
        }

        // Convert the reconstructed bundle to OtapPdata
        match convert_bundle_to_pdata(handle.data()) {
            Ok(mut pdata) => {
                // Capture signal type and item count for metrics before moving pdata
                let signal_type = pdata.signal_type();
                let num_items = pdata.num_items() as u64;

                // Subscribe for ACK/NACK with BundleRef in calldata
                let calldata = encode_bundle_ref(bundle_ref);
                effect_handler.subscribe_to(
                    Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA,
                    calldata,
                    &mut pdata,
                );

                // Try non-blocking send downstream
                match effect_handler.try_send_message(pdata) {
                    Ok(()) => {
                        self.metrics.bundles_sent.add(1);

                        // Track produced items by signal type
                        match signal_type {
                            SignalType::Logs => self.metrics.produced_items_logs.add(num_items),
                            SignalType::Metrics => {
                                self.metrics.produced_items_metrics.add(num_items)
                            }
                            SignalType::Traces => self.metrics.produced_items_traces.add(num_items),
                        }

                        debug!(
                            core_id = self.core_id,
                            segment_seq = bundle_ref.segment_seq.raw(),
                            bundle_index = bundle_ref.bundle_index.raw(),
                            "forwarded bundle downstream from finalized segment"
                        );

                        // Store the handle to keep the bundle claimed until ACK/NACK.
                        // The bundle will not be redelivered while we hold the handle.
                        let _ = self.pending_bundles.insert(
                            key,
                            PendingBundle {
                                handle,
                                retry_count: 0,
                            },
                        );
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
                error!(error = %e, "failed to convert bundle to pdata");
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
            debug!(
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw(),
                "bundle acknowledged in Quiver"
            );
        } else {
            warn!(
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw(),
                "received ACK for unknown bundle"
            );
        }

        Ok(())
    }

    /// Handle NACK from downstream.
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

        // Handle retry
        if let Some(pending) = self.pending_bundles.remove(&key) {
            let retry_count = pending.retry_count + 1;
            self.metrics.bundles_nacked.add(1);

            debug!(
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw(),
                retry_count = retry_count,
                reason = %nack.reason,
                "bundle nacked, will retry on next poll"
            );

            // Drop the handle to trigger implicit defer, releasing the claim.
            // The bundle will be redelivered on the next poll_next_bundle() call.
            // We rely on the timer tick to retry naturally.
            //
            // TODO: For more sophisticated retry with exponential backoff, the embedding
            // layer could use effect_handler.delay_data(). We could also track retry_count
            // separately and reject bundles that exceed a maximum retry limit.
            drop(pending.handle);
        } else {
            warn!(
                segment_seq = bundle_ref.segment_seq.raw(),
                bundle_index = bundle_ref.bundle_index.raw(),
                "received NACK for unknown bundle"
            );
        }

        Ok(())
    }

    /// Handle shutdown by flushing the Quiver engine and draining remaining bundles.
    ///
    /// Flush is critical here: it finalizes any open segment so that data
    /// in the WAL becomes visible to subscribers. Without flush, data that
    /// hasn't triggered segment finalization would be lost.
    async fn handle_shutdown(
        &mut self,
        deadline: Instant,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        info!(deadline = ?deadline, "shutting down persistence processor");

        // Only process if engine was initialized
        if !matches!(self.engine_state, EngineState::Ready { .. }) {
            return Ok(());
        }

        // Flush to finalize any open segment - this makes buffered data visible
        info!("flushing open segment to finalize pending data");
        {
            let (engine, _) = self.engine()?;
            if let Err(e) = engine.flush().await {
                error!(error = %e, "failed to flush Quiver engine");
            }
        }

        // Drain any remaining bundles that became available after flush
        let mut drained = 0u64;
        loop {
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
                            warn!(
                                "backpressure during shutdown drain, some bundles may not be forwarded"
                            );
                            break;
                        }
                        ProcessBundleResult::Error(e) => {
                            warn!(error = %e, "failed to process bundle during shutdown drain");
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    warn!(error = %e, "error polling during shutdown drain");
                    break;
                }
            }
        }
        if drained > 0 {
            info!(
                bundles_drained = drained,
                "drained remaining bundles during shutdown"
            );
        }

        // Now shutdown the engine
        {
            let (engine, _) = self.engine()?;
            if let Err(e) = engine.shutdown().await {
                error!(error = %e, "failed to shutdown Quiver engine");
            } else {
                info!("Quiver engine shutdown complete");
            }
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Processor Trait Implementation
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait(?Send)]
impl otap_df_engine::local::processor::Processor<OtapPdata> for PersistenceProcessor {
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
            debug!(
                poll_interval = ?self.config.poll_interval,
                "started periodic timer for subscriber polling"
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
                            budget.cap(),
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
                    debug!(config = ?config, "received config update (ignored)");
                    Ok(())
                }
                NodeControlMsg::DelayedData { data, .. } => {
                    // Handle delayed data as regular data
                    self.handle_data(*data, effect_handler).await
                }
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Factory Registration
// ─────────────────────────────────────────────────────────────────────────────

/// Factory function to create a PersistenceProcessor.
pub fn create_persistence_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: PersistenceProcessorConfig = serde_json::from_value(node_config.config.clone())
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("failed to parse persistence configuration: {}", e),
        })?;

    // Create processor with lazy engine initialization
    // The Quiver engine will be initialized on the first message when we're
    // guaranteed to be in an async context
    let processor = PersistenceProcessor::new(config, &pipeline_ctx)?;

    Ok(ProcessorWrapper::local(
        processor,
        node,
        node_config,
        processor_config,
    ))
}

/// Register PersistenceProcessor as an OTAP processor factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static PERSISTENCE_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: PERSISTENCE_PROCESSOR_URN,
    create: create_persistence_processor,
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
}
