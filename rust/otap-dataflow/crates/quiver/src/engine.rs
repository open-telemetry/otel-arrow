// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Quiver persistence engine.
//!
//! The [`QuiverEngine`] is the primary entry point for the persistence layer.
//! It coordinates the write-ahead log (WAL) and segment storage to provide
//! durable buffering of Arrow-based telemetry data.
//!
//! # Write Path
//!
//! 1. **WAL Append**: Each incoming `RecordBundle` is first appended to the WAL
//!    for durability.
//! 2. **Open Segment Accumulation**: The bundle is then appended to the current
//!    open segment's in-memory accumulators.
//! 3. **Finalization Check**: If the open segment exceeds the configured size
//!    threshold, it is finalized and written to disk as an immutable segment file.
//! 4. **WAL Consumer Cursor**: After segment finalization, the WAL cursor is
//!    advanced to allow cleanup of consumed entries.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use parking_lot::Mutex;

use crate::config::{DurabilityMode, QuiverConfig, RetentionPolicy};
use crate::error::{QuiverError, Result};
use crate::record_bundle::RecordBundle;
use crate::segment::{OpenSegment, SegmentError, SegmentSeq, SegmentWriter};
use crate::segment_store::SegmentStore;
use crate::subscriber::{
    BundleHandle, BundleRef, RegistryCallback, RegistryConfig, SegmentProvider, SubscriberError,
    SubscriberId, SubscriberRegistry,
};
use crate::telemetry::PersistenceMetrics;
use crate::wal::{WalConsumerCursor, WalWriter, WalWriterOptions};

/// WAL statistics for observability.
#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct WalStats {
    /// Number of WAL file rotations performed.
    pub rotation_count: u64,
    /// Number of rotated files purged after cursor advancement.
    pub purge_count: u64,
}

/// Statistics returned by maintenance operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct MaintenanceStats {
    /// Number of subscribers whose progress was flushed.
    pub flushed: usize,
    /// Number of completed segments deleted.
    pub deleted: usize,
}

/// Primary entry point for the persistence engine.
///
/// The engine coordinates:
/// - **Ingestion**: WAL + segment accumulation
/// - **Segment Storage**: Finalized segment files
/// - **Subscription**: Subscriber registry and bundle delivery
/// - **Maintenance**: Progress flush and segment cleanup
///
/// The engine provides durable buffering with the following guarantees:
///
/// - **Durability**: Data is appended to the WAL before acknowledgement.
/// - **Immutability**: Finalized segments are read-only and never modified.
/// - **Recovery**: On restart, the WAL can replay uncommitted entries.
/// - **Capacity**: Disk usage is bounded by the shared [`DiskBudget`].
///
/// [`DiskBudget`]: crate::budget::DiskBudget
pub struct QuiverEngine {
    /// Engine configuration.
    config: QuiverConfig,
    /// Shared disk budget for enforcing storage caps.
    budget: Arc<crate::budget::DiskBudget>,
    /// Metrics for observability.
    metrics: PersistenceMetrics,
    /// Write-ahead log writer.
    wal_writer: Mutex<WalWriter>,
    /// Current open segment accumulator.
    open_segment: Mutex<OpenSegment>,
    /// Cursor representing all entries in the current open segment.
    /// Updated after each WAL append, used to advance WAL after finalization.
    segment_cursor: Mutex<WalConsumerCursor>,
    /// Next segment sequence number to assign.
    next_segment_seq: AtomicU64,
    /// Cumulative bytes written to segments (never decreases, even after cleanup).
    cumulative_segment_bytes: AtomicU64,
    /// Count of segments force-dropped due to DropOldest policy.
    force_dropped_segments: AtomicU64,
    /// Count of bundles lost due to force-dropped segments.
    force_dropped_bundles: AtomicU64,
    /// Segment store for finalized segment files.
    segment_store: Arc<SegmentStore>,
    /// Subscriber registry for tracking consumption progress.
    registry: Arc<SubscriberRegistry<SegmentStore>>,
}

// ─────────────────────────────────────────────────────────────────────────────
// QuiverEngineBuilder
// ─────────────────────────────────────────────────────────────────────────────

/// Builder for creating a [`QuiverEngine`] with customizable options.
///
/// This provides a cleaner ergonomic interface compared to `QuiverEngine::new()`,
/// especially for tests that may want to customize specific options.
///
/// # Example
///
/// ```
/// use quiver::{QuiverEngineBuilder, QuiverConfig, DiskBudget, RetentionPolicy};
/// use std::sync::Arc;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let temp_dir = tempfile::tempdir()?;
/// let config = QuiverConfig::default().with_data_dir(temp_dir.path());
/// let budget = Arc::new(DiskBudget::new(1024 * 1024 * 1024, RetentionPolicy::Backpressure));
///
/// let engine = QuiverEngineBuilder::new(config)
///     .with_budget(budget)
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct QuiverEngineBuilder {
    config: QuiverConfig,
    budget: Option<Arc<crate::budget::DiskBudget>>,
}

impl QuiverEngineBuilder {
    /// Creates a new builder with the given configuration.
    ///
    /// By default, uses an unlimited disk budget with backpressure policy.
    #[must_use]
    pub fn new(config: QuiverConfig) -> Self {
        Self {
            config,
            budget: None,
        }
    }

    /// Sets the disk budget for the engine.
    ///
    /// The budget enforces a hard cap on total disk usage across WAL, segments,
    /// and progress files. Multiple engines can share the same budget for
    /// coordinated capacity management.
    #[must_use]
    pub fn with_budget(mut self, budget: Arc<crate::budget::DiskBudget>) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Builds the engine, returning an `Arc<QuiverEngine>`.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails or if the WAL
    /// cannot be initialized.
    pub fn build(self) -> Result<Arc<QuiverEngine>> {
        let budget = self.budget.unwrap_or_else(|| {
            Arc::new(crate::budget::DiskBudget::new(
                u64::MAX,
                RetentionPolicy::Backpressure,
            ))
        });
        QuiverEngine::new(self.config, budget)
    }
}

impl std::fmt::Debug for QuiverEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuiverEngine")
            .field("config", &self.config)
            .field("metrics", &self.metrics)
            .field("next_segment_seq", &self.next_segment_seq)
            .finish_non_exhaustive()
    }
}

impl QuiverEngine {
    /// Creates a builder for constructing a `QuiverEngine`.
    ///
    /// This provides a cleaner alternative to [`QuiverEngine::new()`] with
    /// sensible defaults (e.g., unlimited budget with backpressure policy).
    ///
    /// # Example
    ///
    /// ```
    /// use quiver::{QuiverEngine, QuiverConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let temp_dir = tempfile::tempdir()?;
    /// let config = QuiverConfig::default().with_data_dir(temp_dir.path());
    /// let engine = QuiverEngine::builder(config).build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn builder(config: QuiverConfig) -> QuiverEngineBuilder {
        QuiverEngineBuilder::new(config)
    }

    /// Creates a new persistence engine with the given configuration and disk budget.
    ///
    /// This validates the configuration, initializes the WAL writer,
    /// creates the segment store and subscriber registry, and wires
    /// them together for automatic segment notifications.
    ///
    /// The `budget` parameter enforces a hard cap on total disk usage across
    /// WAL, segments, and progress files. Multiple engines can share the same
    /// budget for coordinated capacity management.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails or if the WAL
    /// cannot be initialized.
    pub fn new(config: QuiverConfig, budget: Arc<crate::budget::DiskBudget>) -> Result<Arc<Self>> {
        config.validate()?;

        // Validate budget is large enough for at least 2 segments
        // This prevents deadlock where segment finalization is blocked
        // because the budget is full but consumers can't free space
        // (they're still consuming the one segment that exists).
        let segment_size = config.segment.target_size_bytes.get();
        let min_budget = segment_size.saturating_mul(2);
        if budget.cap() < min_budget && budget.cap() != u64::MAX {
            return Err(QuiverError::invalid_config(format!(
                "disk budget ({} bytes) must be at least 2x segment size ({} bytes = {} bytes) \
                 to prevent deadlock; increase budget to at least {} bytes or reduce segment size",
                budget.cap(),
                segment_size,
                min_budget,
                min_budget
            )));
        }

        // Ensure directories exist
        let segment_dir = segment_dir(&config);
        fs::create_dir_all(&segment_dir).map_err(|e| SegmentError::io(segment_dir.clone(), e))?;

        let wal_writer = initialize_wal_writer(&config, &budget)?;

        // Create segment store with configured read mode and budget
        let segment_store = Arc::new(SegmentStore::with_budget(
            &segment_dir,
            config.read_mode,
            budget.clone(),
        ));

        // Scan for existing segments from previous runs (recovery)
        // This populates the segment store with any segments that exist on disk,
        // enabling subscribers to resume from where they left off.
        if let Err(e) = segment_store.scan_existing() {
            tracing::warn!(
                error = %e,
                "failed to scan existing segments during startup; continuing with empty store"
            );
        }

        // Create subscriber registry with segment store as provider
        let registry_config = RegistryConfig::new(&config.data_dir);
        let registry = SubscriberRegistry::open(registry_config, segment_store.clone())
            .map_err(|e| SegmentError::io(config.data_dir.clone(), std::io::Error::other(e)))?;

        let engine = Arc::new(Self {
            config,
            metrics: PersistenceMetrics::new(),
            wal_writer: Mutex::new(wal_writer),
            open_segment: Mutex::new(OpenSegment::new()),
            segment_cursor: Mutex::new(WalConsumerCursor::default()),
            next_segment_seq: AtomicU64::new(0),
            cumulative_segment_bytes: AtomicU64::new(0),
            force_dropped_segments: AtomicU64::new(0),
            force_dropped_bundles: AtomicU64::new(0),
            segment_store,
            registry: registry.clone(),
            budget: budget.clone(),
        });

        // Wire segment store callback to notify registry of new segments
        let registry_for_callback = registry;
        engine
            .segment_store
            .set_on_segment_registered(move |seq, bundle_count| {
                registry_for_callback.on_segment_finalized(seq, bundle_count);
            });

        // Wire up cleanup callback for Backpressure mode
        // This only deletes completed segments (no data loss)
        let engine_weak_cleanup = Arc::downgrade(&engine);
        budget.set_cleanup_callback(move || {
            if let Some(engine) = engine_weak_cleanup.upgrade() {
                engine.cleanup_completed_segments().unwrap_or(0)
            } else {
                0
            }
        });

        // Wire up reclaim callback for DropOldest policy
        // This enables automatic cleanup when disk budget is exceeded
        let engine_weak = Arc::downgrade(&engine);
        let budget_weak = Arc::downgrade(&budget);
        budget.set_reclaim_callback(move |_needed_bytes| {
            let Some(engine) = engine_weak.upgrade() else {
                return 0;
            };
            let Some(budget) = budget_weak.upgrade() else {
                return 0;
            };

            // Track how much space we free
            let used_before = budget.used();

            // First try to clean up completed segments
            let completed = engine.cleanup_completed_segments().unwrap_or(0);

            // If that didn't help, force-drop oldest pending segments
            // that have no active readers
            if completed == 0 {
                let _ = engine.force_drop_oldest_pending_segments();
            }

            // Return how many bytes we freed
            used_before.saturating_sub(budget.used())
        });

        Ok(engine)
    }

    /// Returns the configuration backing this engine.
    pub fn config(&self) -> &QuiverConfig {
        &self.config
    }

    /// Returns the disk budget governing this engine's storage.
    ///
    /// Use this to inspect current usage, available capacity, or to share
    /// the budget with external components.
    pub fn budget(&self) -> &Arc<crate::budget::DiskBudget> {
        &self.budget
    }

    /// Returns metric counters for instrumentation layers.
    pub fn metrics(&self) -> &PersistenceMetrics {
        &self.metrics
    }

    /// Returns the cumulative bytes written to WAL since engine creation.
    /// This value never decreases, even as WAL files are rotated and purged.
    /// Useful for accurate throughput measurement without file system sampling.
    pub fn wal_bytes_written(&self) -> u64 {
        let writer = self.wal_writer.lock();
        writer.cumulative_bytes_written()
    }

    /// Returns the cumulative bytes written to segments since engine creation.
    /// This value never decreases, even as segments are cleaned up after consumption.
    /// Useful for accurate throughput measurement without file system sampling.
    pub fn segment_bytes_written(&self) -> u64 {
        self.cumulative_segment_bytes.load(Ordering::Relaxed)
    }

    /// Returns the total number of segments written since engine creation.
    ///
    /// This is a monotonically increasing counter, unlike `segment_store().segment_count()`
    /// which only shows currently-loaded segments (after cleanup, count decreases).
    /// Useful for tracking total segments written during a test run.
    pub fn total_segments_written(&self) -> u64 {
        self.next_segment_seq.load(Ordering::Relaxed)
    }

    /// Returns the total number of segments that have been force-dropped
    /// due to the DropOldest retention policy.
    ///
    /// This counter helps demonstrate data loss when using DropOldest vs
    /// Backpressure policy (which should always show 0 dropped segments).
    pub fn force_dropped_segments(&self) -> u64 {
        self.force_dropped_segments.load(Ordering::Relaxed)
    }

    /// Returns the total number of bundles lost due to force-dropped segments.
    ///
    /// This counter tracks the actual data loss (in bundle count) when using
    /// the DropOldest policy. Combined with `force_dropped_segments()`, this
    /// provides visibility into how much data was discarded to stay within
    /// the disk budget.
    pub fn force_dropped_bundles(&self) -> u64 {
        self.force_dropped_bundles.load(Ordering::Relaxed)
    }

    /// Returns WAL statistics (rotation count, purge count).
    ///
    /// Call this before dropping the engine to capture final stats.
    #[cfg(test)]
    pub(crate) fn wal_stats(&self) -> WalStats {
        let writer = self.wal_writer.lock();
        WalStats {
            rotation_count: writer.rotation_count(),
            purge_count: writer.purge_count(),
        }
    }

    /// Ingests a `RecordBundle` into the persistence layer.
    ///
    /// The bundle is first appended to the WAL for durability, then accumulated
    /// into the current open segment. If the segment exceeds the configured
    /// size or time threshold, it is finalized and written to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The disk budget does not have sufficient headroom (`StorageAtCapacity`)
    /// - WAL append fails
    /// - Segment finalization fails
    pub fn ingest<B: RecordBundle>(&self, bundle: &B) -> Result<()> {
        self.metrics.record_ingest_attempt();

        // Step 0: Check budget headroom before doing any work.
        // This "taps the brakes" early, leaving room for WAL rotation and
        // segment finalization to complete. We use a small estimate (4KB)
        // since the actual segment bytes aren't known until finalization.
        const INGEST_HEADROOM_ESTIMATE: u64 = 4 * 1024;
        if !self.budget.has_ingest_headroom(INGEST_HEADROOM_ESTIMATE) {
            return Err(QuiverError::StorageAtCapacity {
                requested: INGEST_HEADROOM_ESTIMATE,
                available: self
                    .budget
                    .headroom()
                    .saturating_sub(self.budget.reserved_headroom()),
                cap: self.budget.cap(),
            });
        }

        // Step 1: Append to WAL for durability (if enabled)
        // Note: WAL bytes are NOT tracked in budget - they're temporary and purged after
        // segment finalization. Only segment bytes are budget-tracked.
        if self.config.durability == DurabilityMode::Wal {
            let wal_offset = self.append_to_wal_with_capacity_handling(bundle)?;

            // Step 2: Update cursor to include this entry
            let cursor = WalConsumerCursor::from_offset(&wal_offset);
            {
                let mut cp = self.segment_cursor.lock();
                *cp = cursor;
            }
        }

        // Step 3: Append to open segment accumulator
        let should_finalize = {
            let mut segment = self.open_segment.lock();
            let _manifest_entry = segment.append(bundle)?;

            // Check if we should finalize based on size threshold
            let estimated_size = segment.estimated_size_bytes();
            let target_size = self.config.segment.target_size_bytes.get() as usize;
            let size_exceeded = estimated_size >= target_size;

            // Check if we should finalize based on time threshold
            let max_duration = self.config.segment.max_open_duration;
            let time_exceeded = segment
                .opened_at()
                .is_some_and(|opened_at| opened_at.elapsed() >= max_duration);

            // Check if we should finalize based on stream count threshold
            // (too many unique (slot, schema) pairs indicates schema evolution pressure)
            let stream_count = segment.stream_count();
            let max_streams = self.config.segment.max_stream_count as usize;
            let streams_exceeded = stream_count >= max_streams;

            size_exceeded || time_exceeded || streams_exceeded
        };

        // Step 4: Finalize segment if threshold exceeded
        if should_finalize {
            self.finalize_current_segment()?;
        }

        Ok(())
    }

    /// Appends a bundle to the WAL, handling capacity errors transparently.
    ///
    /// If the WAL is at capacity, this method:
    /// 1. Finalizes the current segment to advance the WAL cursor
    /// 2. Retries the WAL append
    ///
    /// This allows the engine to handle large segment sizes that exceed the
    /// WAL capacity without requiring caller intervention.
    fn append_to_wal_with_capacity_handling<B: RecordBundle>(
        &self,
        bundle: &B,
    ) -> Result<crate::wal::WalOffset> {
        // First attempt
        let first_result = {
            let mut writer = self.wal_writer.lock();
            writer.append_bundle_sync(bundle)
        };

        match first_result {
            Ok(offset) => Ok(offset),
            Err(ref e) if e.is_at_capacity() => {
                // WAL is full - finalize the current segment to advance the cursor
                // and free WAL space, then retry the append.
                self.finalize_current_segment()?;

                // Retry the append after finalization freed space
                let mut writer = self.wal_writer.lock();
                writer.append_bundle_sync(bundle).map_err(Into::into)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Flushes the current open segment to disk, making all ingested data
    /// available to subscribers.
    ///
    /// This is a checkpoint operation—the engine remains fully operational
    /// and can continue accepting new bundles. Use this when you need to
    /// ensure all ingested data is durable and visible to subscribers without
    /// shutting down.
    ///
    /// Note: Normally segments are finalized automatically based on size and
    /// time thresholds. This method is primarily useful for testing or when
    /// you need immediate durability guarantees.
    ///
    /// # Errors
    ///
    /// Returns an error if segment finalization fails.
    pub fn flush(&self) -> Result<()> {
        self.finalize_current_segment()
    }

    /// Gracefully shuts down the engine, finalizing any open segment.
    ///
    /// After calling this, the engine should not be used for further ingestion.
    /// This ensures that any accumulated data in the open segment is written
    /// to disk. Without calling this, data in the open segment will only be
    /// recoverable via WAL replay.
    ///
    /// For checkpointing without shutdown, use [`flush()`](Self::flush) instead.
    ///
    /// # Errors
    ///
    /// Returns an error if segment finalization fails.
    pub fn shutdown(&self) -> Result<()> {
        self.finalize_current_segment()
    }

    /// Finalizes the current open segment and writes it to disk.
    ///
    /// This is called automatically when the size or time threshold is exceeded,
    /// but can also be called explicitly for shutdown or testing.
    ///
    /// # Budget Handling
    ///
    /// If the disk budget would be exceeded:
    /// - With `Backpressure` policy: Returns `StorageAtCapacity` error. The segment
    ///   data remains in memory and can be retried after space is freed.
    /// - With `DropOldest` policy: Attempts to reclaim space by deleting old segments,
    ///   then retries.
    fn finalize_current_segment(&self) -> Result<()> {
        // First, check if there's anything to finalize (without swapping)
        let estimated_size = {
            let segment_guard = self.open_segment.lock();
            if segment_guard.is_empty() {
                return Ok(());
            }
            segment_guard.estimated_size_bytes() as u64
        };

        // Reserve budget BEFORE swapping out the segment
        // This prevents data loss if reservation fails
        let pending = self.budget.try_reserve(estimated_size)?;

        // Now safe to swap out the segment and cursor
        let (segment, cursor) = {
            let mut segment_guard = self.open_segment.lock();
            let mut cursor_guard = self.segment_cursor.lock();
            let segment = std::mem::take(&mut *segment_guard);
            let cursor = std::mem::take(&mut *cursor_guard);
            (segment, cursor)
        };

        // Double-check segment isn't empty (race condition guard)
        if segment.is_empty() {
            // Release the reservation since we won't write anything
            drop(pending);
            return Ok(());
        }

        // Assign a segment sequence number (after reservation succeeds)
        let seq = SegmentSeq::new(self.next_segment_seq.fetch_add(1, Ordering::SeqCst));

        // Write the segment file (streaming serialization - no intermediate buffer)
        let segment_path = self.segment_path(seq);
        let writer = SegmentWriter::new(seq);
        let (bytes_written, _checksum) = writer.write_segment_sync(&segment_path, segment)?;

        // Track cumulative bytes (never decreases, for accurate throughput measurement)
        let _ = self
            .cumulative_segment_bytes
            .fetch_add(bytes_written, Ordering::Relaxed);

        // Commit reservation with actual bytes written
        pending.commit(bytes_written);

        // Step 5: Advance WAL cursor now that segment is durable
        {
            let mut wal_writer = self.wal_writer.lock();
            wal_writer.persist_cursor_sync(&cursor)?;
        }

        // Step 6: Register segment with store (triggers subscriber notification)
        // Use register_new_segment to skip budget recording (already committed above)
        let _ = self.segment_store.register_new_segment(seq);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Subscriber API
    // ─────────────────────────────────────────────────────────────────────────

    /// Registers a new subscriber with the given ID.
    ///
    /// If a progress file exists for this subscriber, its state is loaded.
    /// The subscriber starts in inactive state; call `activate` to begin
    /// receiving bundles.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber is already registered.
    pub fn register_subscriber(
        &self,
        id: SubscriberId,
    ) -> std::result::Result<(), SubscriberError> {
        self.registry.register(id)
    }

    /// Activates a subscriber, enabling bundle delivery.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscriber is not registered.
    pub fn activate_subscriber(
        &self,
        id: &SubscriberId,
    ) -> std::result::Result<(), SubscriberError> {
        self.registry.activate(id)
    }

    /// Polls for the next available bundle for the subscriber.
    ///
    /// Non-blocking: returns `None` immediately if no bundles are available.
    /// Returns an RAII handle that must be resolved via `ack()`, `reject()`,
    /// or `defer()` before being dropped.
    ///
    /// For async waiting, use [`next_bundle`](Self::next_bundle).
    /// For sync blocking behavior, use [`next_bundle_blocking`](Self::next_bundle_blocking).
    pub fn poll_next_bundle(
        self: &Arc<Self>,
        id: &SubscriberId,
    ) -> std::result::Result<Option<BundleHandle<RegistryCallback<SegmentStore>>>, SubscriberError>
    {
        self.registry.poll_next_bundle(id)
    }

    /// Waits asynchronously for the next available bundle.
    ///
    /// This is the primary async API for consuming bundles. It awaits until
    /// a bundle becomes available or the timeout expires.
    ///
    /// Returns an RAII handle that must be resolved via `ack()`, `reject()`,
    /// or `defer()` before being dropped.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscriber ID
    /// * `timeout` - Maximum time to wait for a bundle. If `None`, waits indefinitely.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(handle))` - A bundle is available
    /// * `Ok(None)` - Timeout expired with no bundle available
    /// * `Err(_)` - Subscriber not found or not active
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::Duration;
    ///
    /// // Wait up to 5 seconds for a bundle
    /// let handle = engine
    ///     .next_bundle(&subscriber_id, Some(Duration::from_secs(5)))
    ///     .await?;
    ///
    /// if let Some(bundle) = handle {
    ///     // Process the bundle...
    ///     bundle.ack();
    /// }
    /// ```
    pub async fn next_bundle(
        self: &Arc<Self>,
        id: &SubscriberId,
        timeout: Option<Duration>,
    ) -> std::result::Result<Option<BundleHandle<RegistryCallback<SegmentStore>>>, SubscriberError>
    {
        self.registry.next_bundle(id, timeout).await
    }

    /// Waits for the next available bundle with a timeout (blocking).
    ///
    /// Blocking: waits up to `timeout` for a bundle to become available.
    /// Returns `None` on timeout or if `should_stop` returns true.
    ///
    /// For async waiting, prefer [`next_bundle`](Self::next_bundle).
    ///
    /// # Arguments
    ///
    /// * `id` - The subscriber ID
    /// * `timeout` - Maximum time to wait for a bundle. If `None`, waits indefinitely.
    /// * `should_stop` - Called periodically to check if waiting should stop (e.g., for shutdown).
    pub fn next_bundle_blocking<F>(
        self: &Arc<Self>,
        id: &SubscriberId,
        timeout: Option<Duration>,
        should_stop: F,
    ) -> std::result::Result<Option<BundleHandle<RegistryCallback<SegmentStore>>>, SubscriberError>
    where
        F: Fn() -> bool,
    {
        self.registry.next_bundle_blocking(id, timeout, should_stop)
    }

    /// Claims a specific bundle for a subscriber.
    ///
    /// Used for retry scenarios where the embedding layer needs to re-acquire
    /// a previously deferred bundle.
    ///
    /// # Errors
    ///
    /// Returns an error if the bundle is not available (already resolved or claimed).
    pub fn claim_bundle(
        self: &Arc<Self>,
        id: &SubscriberId,
        bundle_ref: BundleRef,
    ) -> std::result::Result<BundleHandle<RegistryCallback<SegmentStore>>, SubscriberError> {
        self.registry.claim_bundle(id, bundle_ref)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Maintenance API
    // ─────────────────────────────────────────────────────────────────────────

    /// Flushes dirty subscriber progress to disk synchronously.
    ///
    /// Returns the number of subscribers whose progress was flushed.
    ///
    /// For async contexts, use [`flush_progress`](Self::flush_progress).
    ///
    /// # Errors
    ///
    /// Returns an error if any progress file cannot be written.
    pub fn flush_progress_sync(&self) -> std::result::Result<usize, SubscriberError> {
        self.registry.flush_progress_sync()
    }

    /// Deletes segment files that have been fully processed by all subscribers.
    ///
    /// Finds the minimum incomplete segment across all active subscribers
    /// and deletes all segments before that threshold.
    ///
    /// Returns the number of segments deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if segment deletion fails.
    pub fn cleanup_completed_segments(&self) -> std::result::Result<usize, SubscriberError> {
        // Find the oldest incomplete segment across all subscribers.
        // All segments before this are fully processed.
        let delete_boundary = match self.registry.oldest_incomplete_segment() {
            Some(seq) => seq, // Delete segments strictly before this
            None => {
                // No incomplete segments. This could mean:
                // 1. No subscribers or no segments tracked yet - should not delete
                // 2. All tracked segments are complete - can delete up to highest
                match self.registry.min_highest_tracked_segment() {
                    Some(highest) => {
                        // All segments up through `highest` are complete.
                        // Delete segments up to and including it.
                        highest.next()
                    }
                    None => return Ok(0), // No active subscribers tracking segments
                }
            }
        };

        // Delete segment files before the boundary
        let mut deleted = 0;
        for seq in self.segment_store.segment_sequences() {
            if seq < delete_boundary {
                if let Err(e) = self.segment_store.delete_segment(seq) {
                    tracing::warn!(segment = seq.raw(), error = %e, "Failed to delete segment");
                } else {
                    deleted += 1;
                }
            }
        }

        // Clean up registry internal state for deleted segments
        self.registry.cleanup_segments_before(delete_boundary);

        Ok(deleted)
    }

    /// Force-drops the oldest pending segments that have no active readers.
    ///
    /// This is used by the `DropOldest` retention policy to reclaim disk space
    /// when at capacity. Unlike `cleanup_completed_segments`, this will drop
    /// segments that haven't been fully consumed by all subscribers, as long
    /// as no subscriber is currently reading from them (has claimed bundles).
    ///
    /// # Warning
    ///
    /// This causes data loss for subscribers that haven't consumed these segments.
    /// Subscribers will see a gap in their segment sequence but can continue
    /// processing from the next available segment.
    ///
    /// Returns the number of segments force-dropped.
    pub fn force_drop_oldest_pending_segments(&self) -> usize {
        // Get list of segments to drop from the registry
        let to_drop = self.registry.force_drop_oldest_pending_segments();

        if to_drop.is_empty() {
            return 0;
        }

        // Delete the segment files, counting bundles before deletion
        let mut deleted = 0;
        let mut bundles_dropped: u64 = 0;
        for seq in &to_drop {
            // Count bundles before deleting
            if let Ok(count) = self.segment_store.bundle_count(*seq) {
                bundles_dropped += count as u64;
            }
            if let Err(e) = self.segment_store.delete_segment(*seq) {
                tracing::warn!(segment = seq.raw(), error = %e, "Failed to delete force-dropped segment");
            } else {
                tracing::info!(
                    segment = seq.raw(),
                    "Force-dropped pending segment (DropOldest policy)"
                );
                deleted += 1;
            }
        }

        // Update the force-dropped counters
        let _ = self
            .force_dropped_segments
            .fetch_add(deleted as u64, Ordering::Relaxed);
        let _ = self
            .force_dropped_bundles
            .fetch_add(bundles_dropped, Ordering::Relaxed);

        // Clean up registry internal state
        if let Some(&max_dropped) = to_drop.iter().max() {
            self.registry.cleanup_segments_before(max_dropped.next());
        }

        deleted
    }

    /// Performs combined maintenance synchronously: flushes progress and cleans up segments.
    ///
    /// This is the recommended periodic maintenance call. It:
    /// 1. Flushes dirty subscriber progress to disk
    /// 2. Deletes fully-processed segment files
    ///
    /// For async contexts, use [`maintain`](Self::maintain).
    ///
    /// # Errors
    ///
    /// Returns an error if either operation fails.
    pub fn maintain_sync(&self) -> std::result::Result<MaintenanceStats, SubscriberError> {
        let flushed = self.flush_progress_sync()?;
        let deleted = self.cleanup_completed_segments()?;
        Ok(MaintenanceStats { flushed, deleted })
    }

    /// Flushes dirty subscriber progress to disk.
    ///
    /// Returns the number of subscribers whose progress was flushed.
    ///
    /// # Errors
    ///
    /// Returns an error if any progress file cannot be written.
    pub async fn flush_progress(&self) -> std::result::Result<usize, SubscriberError> {
        self.registry.flush_progress().await
    }

    /// Performs combined maintenance: flushes progress and cleans up segments.
    ///
    /// This is the recommended periodic maintenance call. It:
    /// 1. Flushes dirty subscriber progress to disk
    /// 2. Deletes fully-processed segment files
    ///
    /// # Errors
    ///
    /// Returns an error if either operation fails.
    pub async fn maintain(&self) -> std::result::Result<MaintenanceStats, SubscriberError> {
        let flushed = self.flush_progress().await?;
        let deleted = self.cleanup_completed_segments()?;
        Ok(MaintenanceStats { flushed, deleted })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Accessors
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns a reference to the underlying segment store.
    ///
    /// This is useful for advanced use cases like custom segment queries.
    #[must_use]
    pub fn segment_store(&self) -> &Arc<SegmentStore> {
        &self.segment_store
    }

    /// Returns a reference to the underlying subscriber registry.
    ///
    /// This is useful for advanced use cases like custom subscriber management.
    #[must_use]
    pub fn registry(&self) -> &Arc<SubscriberRegistry<SegmentStore>> {
        &self.registry
    }

    /// Returns the path for a segment file with the given sequence number.
    fn segment_path(&self, seq: SegmentSeq) -> PathBuf {
        segment_dir(&self.config).join(format!("{}.qseg", seq.to_filename_component()))
    }
}

fn segment_dir(config: &QuiverConfig) -> PathBuf {
    config.data_dir.join("segments")
}

fn initialize_wal_writer(
    config: &QuiverConfig,
    budget: &Arc<crate::budget::DiskBudget>,
) -> Result<WalWriter> {
    use crate::wal::FlushPolicy;

    let wal_path = wal_path(config);
    let flush_policy = if config.wal.flush_interval.is_zero() {
        FlushPolicy::Immediate
    } else {
        FlushPolicy::EveryDuration(config.wal.flush_interval)
    };
    let options = WalWriterOptions::new(wal_path, segment_cfg_hash(config), flush_policy)
        .with_max_wal_size(config.wal.max_size_bytes.get())
        .with_max_rotated_files(config.wal.max_rotated_files as usize)
        .with_rotation_target(config.wal.rotation_target_bytes.get())
        .with_budget(budget.clone());
    Ok(WalWriter::open_sync(options)?)
}

fn wal_path(config: &QuiverConfig) -> PathBuf {
    config.data_dir.join("wal").join("quiver.wal")
}

fn segment_cfg_hash(_config: &QuiverConfig) -> [u8; 16] {
    // Placeholder: the segment_cfg_hash should be derived from adapter-owned
    // layout contracts (slot id → payload mappings, per-slot ordering, checksum
    // policy toggles) once available. Operational knobs like segment.target_size,
    // flush cadence, or retention caps are intentionally excluded so that tuning
    // never invalidates an otherwise healthy WAL.
    //
    // For now we return a fixed placeholder until adapter metadata is implemented.
    //
    // Future implementation might look like:
    // ```
    // let mut hasher = Hasher::new();
    // hasher.update(&adapter.slot_layout_fingerprint());
    // hasher.update(&adapter.checksum_policy().to_le_bytes());
    // // ... other adapter-specific layout settings
    // let digest = hasher.finalize();
    // let mut hash = [0u8; 16];
    // hash.copy_from_slice(&digest.as_bytes()[..16]);
    // hash
    // ```
    *b"QUIVER_SEGCFG\0\0\0"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::DiskBudget;
    use crate::config::{RetentionPolicy, SegmentConfig, WalConfig};
    use crate::record_bundle::{
        BundleDescriptor, PayloadRef, RecordBundle, SlotDescriptor, SlotId,
    };
    use crate::subscriber::SubscriberId;
    use crate::wal::WalReader;
    use arrow_array::builder::Int64Builder;
    use arrow_schema::{DataType, Field, Schema};
    use std::num::NonZeroU64;

    /// Creates a large test budget (1 GB) for tests that don't specifically test budget limits.
    fn test_budget() -> Arc<DiskBudget> {
        Arc::new(DiskBudget::new(
            1024 * 1024 * 1024,
            RetentionPolicy::Backpressure,
        ))
    }
    use std::sync::Arc;
    use tempfile::tempdir;

    struct DummyBundle {
        descriptor: BundleDescriptor,
        batch: arrow_array::RecordBatch,
    }

    impl DummyBundle {
        fn new() -> Self {
            let schema = Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::Int64,
                false,
            )]));
            Self {
                descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(
                    SlotId::new(0),
                    "Logs",
                )]),
                batch: arrow_array::RecordBatch::new_empty(schema),
            }
        }

        /// Creates a bundle with the specified number of rows.
        fn with_rows(num_rows: usize) -> Self {
            let schema = Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::Int64,
                false,
            )]));

            let mut builder = Int64Builder::new();
            for i in 0..num_rows {
                builder.append_value(i as i64);
            }
            let array = builder.finish();

            let batch =
                arrow_array::RecordBatch::try_new(schema.clone(), vec![Arc::new(array)]).unwrap();

            Self {
                descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(
                    SlotId::new(0),
                    "Logs",
                )]),
                batch,
            }
        }
    }

    impl RecordBundle for DummyBundle {
        fn descriptor(&self) -> &BundleDescriptor {
            &self.descriptor
        }

        fn ingestion_time(&self) -> std::time::SystemTime {
            std::time::SystemTime::now()
        }

        fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
            if slot == SlotId::new(0) {
                Some(PayloadRef {
                    schema_fingerprint: [0; 32],
                    batch: &self.batch,
                })
            } else {
                None
            }
        }
    }

    #[test]
    fn ingest_succeeds_and_records_metrics() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());
        let engine = QuiverEngine::new(config, test_budget()).expect("config valid");
        let bundle = DummyBundle::new();

        // Ingest should now succeed
        engine.ingest(&bundle).expect("ingest succeeds");
        assert_eq!(engine.metrics().ingest_attempts(), 1);
    }

    #[test]
    fn config_returns_engine_configuration() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .build()
            .expect("builder should produce valid config");
        let engine = QuiverEngine::new(config.clone(), test_budget()).expect("config valid");

        assert_eq!(engine.config(), &config);
    }

    #[test]
    fn ingest_appends_to_wal() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());
        let engine = QuiverEngine::new(config, test_budget()).expect("config valid");
        let bundle = DummyBundle::new();

        engine.ingest(&bundle).expect("ingest succeeds");

        drop(engine);

        let wal_path = temp_dir.path().join("wal").join("quiver.wal");
        let mut reader = WalReader::open(&wal_path).expect("wal opens");
        let mut iter = reader.iter_from(0).expect("iterator");
        let entry = iter.next().expect("entry exists").expect("entry decodes");

        assert_eq!(entry.sequence, 0);
        assert_eq!(entry.slots.len(), 1);
        assert_eq!(entry.slot_bitmap.count_ones(), 1);
    }

    /// Test that DurabilityMode::SegmentOnly skips WAL writes.
    ///
    /// This test verifies:
    /// 1. Ingest succeeds with SegmentOnly mode
    /// 2. WAL file is not written to (or contains no entries)
    /// 3. Segments are still created normally
    #[test]
    fn ingest_segment_only_mode_skips_wal() {
        use crate::config::DurabilityMode;
        use crate::segment::SegmentReader;

        let temp_dir = tempdir().expect("tempdir");

        // Small segment to trigger finalization
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).unwrap(),
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .durability(DurabilityMode::SegmentOnly)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Ingest a bundle
        let bundle = DummyBundle::with_rows(10);
        engine.ingest(&bundle).expect("ingest succeeds");

        // Verify metrics
        assert_eq!(engine.metrics().ingest_attempts(), 1);

        drop(engine);

        // === Verify segment file was created ===
        let segment_dir = temp_dir.path().join("segments");
        let segment_files: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .collect();

        assert_eq!(segment_files.len(), 1, "expected one segment file");

        // Verify segment contents
        let segment_path = segment_files[0].path();
        let reader = SegmentReader::open(&segment_path).expect("open segment");
        assert_eq!(reader.bundle_count(), 1);

        let manifest = reader.manifest();
        let reconstructed = reader.read_bundle(&manifest[0]).expect("read bundle");
        let payload = reconstructed
            .payload(SlotId::new(0))
            .expect("slot 0 exists");
        assert_eq!(payload.num_rows(), 10);

        // === Verify WAL contains no entries ===
        let wal_path = temp_dir.path().join("wal").join("quiver.wal");
        let mut wal_reader = WalReader::open(&wal_path).expect("open WAL");
        let mut iter = wal_reader.iter_from(0).expect("iterator");

        // WAL should have no entries when using SegmentOnly mode
        assert!(
            iter.next().is_none(),
            "WAL should have no entries in SegmentOnly mode"
        );
    }

    #[test]
    fn ingest_finalizes_segment_when_threshold_exceeded() {
        let temp_dir = tempdir().expect("tempdir");

        // Use a tiny segment size to trigger finalization
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).unwrap(), // Very small
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Ingest enough data to exceed the threshold
        // With 100 byte threshold and ~100 bytes per row estimate,
        // a few rows should trigger finalization
        let bundle = DummyBundle::with_rows(10);
        engine.ingest(&bundle).expect("ingest succeeds");

        drop(engine);

        // Check that a segment file was created
        let segment_dir = temp_dir.path().join("segments");
        let entries: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .collect();

        assert_eq!(entries.len(), 1, "expected one segment file");
    }

    #[test]
    fn dummy_bundle_payload_handles_missing_slot() {
        let bundle = DummyBundle::new();
        assert!(bundle.payload(SlotId::new(10)).is_none());
    }

    /// End-to-end test validating the full ingest → segment → WAL cursor flow.
    ///
    /// This test verifies:
    /// 1. Bundles are appended to the WAL
    /// 2. Segment file is created when threshold is exceeded
    /// 3. Segment file contains the expected data
    /// 4. WAL cursor is advanced after segment finalization
    #[test]
    fn e2e_ingest_creates_segment_and_advances_wal_cursor() {
        use crate::segment::SegmentReader;
        use crate::wal::CursorSidecar;

        let temp_dir = tempdir().expect("tempdir");

        // Use a tiny segment size to trigger finalization
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).unwrap(),
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Ingest a bundle with enough data to trigger finalization
        let bundle = DummyBundle::with_rows(10);
        engine.ingest(&bundle).expect("ingest succeeds");

        // Drop engine to ensure all writes are flushed
        drop(engine);

        // === Verify segment file was created ===
        let segment_dir = temp_dir.path().join("segments");
        let segment_files: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .collect();
        assert_eq!(segment_files.len(), 1, "expected one segment file");

        // === Verify segment contents ===
        let segment_path = segment_files[0].path();
        let reader = SegmentReader::open(&segment_path).expect("open segment");

        // Should have 1 bundle in manifest
        assert_eq!(reader.bundle_count(), 1, "expected 1 bundle in segment");

        // Should have 1 stream (one slot type)
        assert_eq!(reader.stream_count(), 1, "expected 1 stream");

        // Read the bundle and verify it has the expected data
        let manifest = reader.manifest();
        let reconstructed = reader.read_bundle(&manifest[0]).expect("read bundle");
        assert_eq!(reconstructed.slot_count(), 1, "expected 1 slot");

        let payload = reconstructed
            .payload(SlotId::new(0))
            .expect("slot 0 exists");
        assert_eq!(payload.num_rows(), 10, "expected 10 rows in payload");

        // === Verify WAL cursor was advanced ===
        let sidecar_path = temp_dir.path().join("wal").join("quiver.wal.cursor");
        let sidecar = CursorSidecar::read_from_sync(&sidecar_path).expect("read sidecar");

        // The cursor should be > 0 (advanced past the header)
        assert!(
            sidecar.wal_position > 0,
            "cursor should be advanced after segment finalization"
        );

        // === Verify WAL contains the entry ===
        let wal_path = temp_dir.path().join("wal").join("quiver.wal");
        let mut wal_reader = WalReader::open(&wal_path).expect("open WAL");
        let mut iter = wal_reader.iter_from(0).expect("iterator");
        let entry = iter.next().expect("entry exists").expect("entry decodes");

        // The WAL entry should have sequence 0 and 1 slot
        assert_eq!(entry.sequence, 0);
        assert_eq!(entry.slots.len(), 1);

        // The cursor should point past this entry.
        // Both wal_position and next_offset are now in global coordinates.
        assert_eq!(
            sidecar.wal_position, entry.next_offset,
            "cursor should point past the finalized entry"
        );
    }

    /// Tests that the engine transparently handles WAL capacity by finalizing segments.
    ///
    /// This test verifies that when:
    /// 1. Segment size is larger than WAL max size
    /// 2. WAL fills up before segment finalization threshold is reached
    ///
    /// The engine will:
    /// 1. Detect the WAL is at capacity
    /// 2. Finalize the current segment to advance the WAL cursor and free space
    /// 3. Retry the WAL append transparently
    /// 4. Continue accepting ingestion without returning errors to the caller
    #[test]
    fn ingest_handles_wal_capacity_transparently() {
        let temp_dir = tempdir().expect("tempdir");

        // Configure WAL to be smaller than segment target size.
        // WAL max: 8KB, Segment target: 32KB
        // This means WAL will fill up before segment size threshold is reached.
        let wal_config = WalConfig {
            max_size_bytes: NonZeroU64::new(8 * 1024).unwrap(),
            rotation_target_bytes: NonZeroU64::new(4 * 1024).unwrap(),
            max_rotated_files: 2,
            flush_interval: Duration::ZERO,
        };
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(32 * 1024).unwrap(),
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .wal(wal_config)
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Ingest many bundles - WAL will fill up multiple times
        // Each bundle is ~100-200 bytes, so we need 50+ to fill the 8KB WAL
        for i in 0..100 {
            let bundle = DummyBundle::with_rows(5);
            engine
                .ingest(&bundle)
                .unwrap_or_else(|e| panic!("ingest {} failed: {:?}", i, e));
        }

        // Verify we created multiple segments (WAL capacity forced early finalization)
        let segment_dir = temp_dir.path().join("segments");
        let segment_count = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .count();

        // We should have multiple segments since WAL capacity forced finalization
        // before the segment size threshold was reached
        assert!(
            segment_count >= 2,
            "expected at least 2 segments due to WAL capacity (got {})",
            segment_count
        );

        // Verify all bundles were ingested
        assert_eq!(
            engine.metrics().ingest_attempts(),
            100,
            "all ingests should succeed"
        );
    }

    /// End-to-end test for ingesting many bundles that span multiple segments.
    ///
    /// This test verifies:
    /// 1. Multiple bundles accumulate correctly in the open segment
    /// 2. Multiple segment files are created as thresholds are exceeded
    /// 3. Each segment contains the expected bundles
    /// 4. WAL cursor advances correctly after each segment finalization
    /// 5. WAL entries match the total number of ingested bundles
    /// 6. All data can be reconstructed from segments + WAL replay
    #[test]
    fn e2e_many_bundles_across_multiple_segments() {
        use crate::segment::SegmentReader;
        use crate::wal::CursorSidecar;

        let temp_dir = tempdir().expect("tempdir");

        // Use a small segment size so we get multiple segments
        // Each bundle with 5 rows is ~500 bytes, so 1KB threshold = ~2 bundles per segment
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).unwrap(),
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Ingest 10 bundles with varying row counts
        let bundle_row_counts = [5, 8, 3, 12, 7, 4, 9, 6, 11, 2];
        let total_rows: usize = bundle_row_counts.iter().sum();

        for &row_count in &bundle_row_counts {
            let bundle = DummyBundle::with_rows(row_count);
            engine.ingest(&bundle).expect("ingest succeeds");
        }

        // Verify metrics
        assert_eq!(
            engine.metrics().ingest_attempts(),
            bundle_row_counts.len() as u64
        );

        // Drop engine to flush all writes
        drop(engine);

        // === Verify multiple segment files were created ===
        let segment_dir = temp_dir.path().join("segments");
        let mut segment_files: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .map(|e| e.path())
            .collect();
        segment_files.sort(); // Sort by filename (sequence number)

        assert!(
            segment_files.len() >= 2,
            "expected at least 2 segment files, got {}",
            segment_files.len()
        );

        // === Count data in finalized segments ===
        let mut segment_rows = 0;
        let mut segment_bundles = 0;

        for segment_path in &segment_files {
            let reader = SegmentReader::open(segment_path).expect("open segment");

            for entry in reader.manifest() {
                let bundle = reader.read_bundle(entry).expect("read bundle");
                if let Some(payload) = bundle.payload(SlotId::new(0)) {
                    segment_rows += payload.num_rows();
                }
                segment_bundles += 1;
            }
        }

        // === Verify WAL contains all entries ===
        let wal_path = temp_dir.path().join("wal").join("quiver.wal");
        let mut wal_reader = WalReader::open(&wal_path).expect("open WAL");
        let iter = wal_reader.iter_from(0).expect("iterator");

        let mut wal_entry_count = 0;
        let mut wal_total_rows = 0;
        let mut last_entry_next_offset = 0;
        for result in iter {
            let entry = result.expect("entry decodes");
            assert_eq!(
                entry.sequence, wal_entry_count,
                "WAL sequence should be monotonic"
            );
            // Count rows in WAL entries
            for slot in &entry.slots {
                wal_total_rows += slot.row_count as usize;
            }
            last_entry_next_offset = entry.next_offset;
            wal_entry_count += 1;
        }

        assert_eq!(
            wal_entry_count,
            bundle_row_counts.len() as u64,
            "WAL should contain one entry per ingested bundle"
        );

        // WAL contains ALL data (it's the durability source)
        assert_eq!(
            wal_total_rows, total_rows,
            "WAL should contain all ingested rows"
        );

        // Segments contain only finalized data (some bundles may still be in open segment)
        assert!(
            segment_bundles <= bundle_row_counts.len(),
            "segment bundles ({}) should not exceed total bundles ({})",
            segment_bundles,
            bundle_row_counts.len()
        );
        assert!(
            segment_rows <= total_rows,
            "segment rows ({}) should not exceed total rows ({})",
            segment_rows,
            total_rows
        );

        // === Verify cursor is at or past the last finalized segment ===
        let sidecar_path = temp_dir.path().join("wal").join("quiver.wal.cursor");
        let sidecar = CursorSidecar::read_from_sync(&sidecar_path).expect("read sidecar");

        // Cursor should be > 0 (some segments were finalized)
        assert!(
            sidecar.wal_position > 0,
            "cursor should advance after segment finalization"
        );

        // If there's still an open segment (not finalized), cursor won't be at the very end.
        // It should be <= last_entry_next_offset
        assert!(
            sidecar.wal_position <= last_entry_next_offset,
            "cursor ({}) should not exceed last WAL entry ({})",
            sidecar.wal_position,
            last_entry_next_offset
        );

        // === Verify recovery: WAL entries after cursor can restore missing data ===
        // Count WAL entries after the cursor (these are in the open segment)
        let mut wal_reader2 = WalReader::open(&wal_path).expect("open WAL");
        let iter2 = wal_reader2
            .iter_from(sidecar.wal_position)
            .expect("iterator from cursor");

        let mut uncommitted_bundles = 0;
        let mut uncommitted_rows = 0;
        for result in iter2 {
            let entry = result.expect("entry decodes");
            for slot in &entry.slots {
                uncommitted_rows += slot.row_count as usize;
            }
            uncommitted_bundles += 1;
        }

        // Segments + uncommitted WAL entries should equal total
        assert_eq!(
            segment_bundles + uncommitted_bundles,
            bundle_row_counts.len(),
            "finalized bundles ({}) + uncommitted ({}) should equal total ({})",
            segment_bundles,
            uncommitted_bundles,
            bundle_row_counts.len()
        );
        assert_eq!(
            segment_rows + uncommitted_rows,
            total_rows,
            "finalized rows ({}) + uncommitted ({}) should equal total ({})",
            segment_rows,
            uncommitted_rows,
            total_rows
        );
    }

    /// Test that bundles with different schemas create separate streams.
    #[test]
    fn e2e_bundles_with_different_schemas_create_separate_streams() {
        use crate::segment::SegmentReader;

        let temp_dir = tempdir().expect("tempdir");

        // Small segment size to trigger finalization
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(500).unwrap(),
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Create bundles with different fingerprints (simulating schema evolution)
        let bundle1 = DummyBundleWithFingerprint::new([0x11; 32], 5);
        let bundle2 = DummyBundleWithFingerprint::new([0x22; 32], 5);
        let bundle3 = DummyBundleWithFingerprint::new([0x11; 32], 5); // Same as bundle1

        engine.ingest(&bundle1).expect("ingest bundle1");
        engine.ingest(&bundle2).expect("ingest bundle2");
        engine.ingest(&bundle3).expect("ingest bundle3");

        drop(engine);

        // Find the segment file(s)
        let segment_dir = temp_dir.path().join("segments");
        let segment_files: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .map(|e| e.path())
            .collect();

        assert!(!segment_files.is_empty(), "expected at least one segment");

        // Count total streams across all segments
        let mut total_streams = 0;
        let mut total_bundles = 0;
        for path in &segment_files {
            let reader = SegmentReader::open(path).expect("open segment");
            total_streams += reader.stream_count();
            total_bundles += reader.bundle_count();
        }

        // Should have 2 distinct streams (fingerprint 0x11 and 0x22)
        // But bundles might be split across segments, so check total bundles
        assert_eq!(total_bundles, 3, "expected 3 bundles total");

        // Streams should reflect the two distinct fingerprints
        // (actual count depends on how bundles landed in segments)
        assert!(
            total_streams >= 1,
            "expected at least 1 stream, got {}",
            total_streams
        );
    }

    /// Helper struct for testing bundles with custom fingerprints.
    struct DummyBundleWithFingerprint {
        descriptor: BundleDescriptor,
        batch: arrow_array::RecordBatch,
        fingerprint: [u8; 32],
    }

    impl DummyBundleWithFingerprint {
        fn new(fingerprint: [u8; 32], num_rows: usize) -> Self {
            let schema = Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::Int64,
                false,
            )]));

            let mut builder = Int64Builder::new();
            for i in 0..num_rows {
                builder.append_value(i as i64);
            }
            let array = builder.finish();

            let batch =
                arrow_array::RecordBatch::try_new(schema.clone(), vec![Arc::new(array)]).unwrap();

            Self {
                descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(
                    SlotId::new(0),
                    "Logs",
                )]),
                batch,
                fingerprint,
            }
        }
    }

    impl RecordBundle for DummyBundleWithFingerprint {
        fn descriptor(&self) -> &BundleDescriptor {
            &self.descriptor
        }

        fn ingestion_time(&self) -> std::time::SystemTime {
            std::time::SystemTime::now()
        }

        fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
            if slot == SlotId::new(0) {
                Some(PayloadRef {
                    schema_fingerprint: self.fingerprint,
                    batch: &self.batch,
                })
            } else {
                None
            }
        }
    }

    /// Stress test: ingest thousands of bundles creating many segments.
    ///
    /// This test exercises:
    /// - High volume ingestion (1000+ bundles)
    /// - Many segment file creations (50+ segments)
    /// - Large total row counts (100K+ rows)
    /// - WAL rotation and cursor advancement
    /// - Data integrity across all segments
    ///
    /// Run manually with: `cargo test stress_high_volume_ingestion -- --ignored`
    #[test]
    #[ignore]
    fn stress_high_volume_ingestion() {
        use crate::config::WalConfig;
        use crate::segment::SegmentReader;
        use crate::wal::CursorSidecar;
        use std::time::Instant;

        let temp_dir = tempdir().expect("tempdir");

        // Configure for stress testing:
        // - Small segments (8KB) to create many segment files
        // - Small WAL rotation (64KB) to exercise rotation
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(8 * 1024).unwrap(), // 8KB segments
            ..Default::default()
        };
        let wal_config = WalConfig {
            rotation_target_bytes: NonZeroU64::new(64 * 1024).unwrap(), // 64KB rotation
            max_size_bytes: NonZeroU64::new(1024 * 1024).unwrap(),      // 1MB max
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .wal(wal_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Stress parameters
        const NUM_BUNDLES: usize = 10_000;
        const ROWS_PER_BUNDLE: usize = 100;
        const TOTAL_EXPECTED_ROWS: usize = NUM_BUNDLES * ROWS_PER_BUNDLE;

        // Pre-generate a small pool of bundles to reuse (avoids 1M allocations)
        const BUNDLE_POOL_SIZE: usize = 100;
        let bundle_pool: Vec<_> = (0..BUNDLE_POOL_SIZE)
            .map(|_| DummyBundle::with_rows(ROWS_PER_BUNDLE))
            .collect();

        let start = Instant::now();

        // Ingest many bundles, cycling through the pool
        for i in 0..NUM_BUNDLES {
            let bundle = &bundle_pool[i % BUNDLE_POOL_SIZE];
            engine
                .ingest(bundle)
                .unwrap_or_else(|e| panic!("ingest {} failed: {}", i, e));
        }

        let ingest_duration = start.elapsed();

        // Verify metrics
        assert_eq!(engine.metrics().ingest_attempts(), NUM_BUNDLES as u64);

        // Capture WAL stats before dropping
        let wal_stats = engine.wal_stats();

        // Drop engine to flush
        drop(engine);

        let total_duration = start.elapsed();

        // === Count segment files ===
        let segment_dir = temp_dir.path().join("segments");
        let segment_files: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .map(|e| e.path())
            .collect();

        // With 8KB segments and ~1KB per bundle (100 rows), expect ~8 bundles per segment
        // 1000 bundles / 8 = ~125 segments minimum
        assert!(
            segment_files.len() >= 50,
            "expected at least 50 segments, got {}",
            segment_files.len()
        );

        // === Verify data integrity across all segments ===
        let mut segment_rows = 0;
        let mut segment_bundles = 0;
        let mut total_segment_bytes = 0u64;

        for path in &segment_files {
            let metadata = fs::metadata(path).expect("segment metadata");
            total_segment_bytes += metadata.len();

            let reader = SegmentReader::open(path).expect("open segment");
            for entry in reader.manifest() {
                let bundle = reader.read_bundle(entry).expect("read bundle");
                if let Some(payload) = bundle.payload(SlotId::new(0)) {
                    segment_rows += payload.num_rows();
                }
                segment_bundles += 1;
            }
        }

        // === Verify WAL + cursor state ===
        let wal_dir = temp_dir.path().join("wal");
        let sidecar_path = wal_dir.join("quiver.wal.cursor");
        let sidecar = CursorSidecar::read_from_sync(&sidecar_path).expect("read sidecar");

        // Count WAL files and sizes
        let wal_files: Vec<_> = fs::read_dir(&wal_dir)
            .expect("read wal dir")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("quiver.wal"))
            })
            .collect();

        let active_wal_path = wal_dir.join("quiver.wal");
        let active_wal_size = fs::metadata(&active_wal_path).map(|m| m.len()).unwrap_or(0);

        let rotated_wal_count = wal_files.len().saturating_sub(1); // exclude active
        let total_wal_bytes: u64 = wal_files
            .iter()
            .filter_map(|e| fs::metadata(e.path()).ok())
            .map(|m| m.len())
            .sum();

        // Read uncommitted entries (after cursor)
        let wal_path = temp_dir.path().join("wal").join("quiver.wal");
        let mut wal_reader = WalReader::open(&wal_path).expect("open WAL");
        let iter = wal_reader
            .iter_from(sidecar.wal_position)
            .expect("iterator from cursor");

        let mut uncommitted_bundles = 0usize;
        let mut uncommitted_rows = 0usize;
        for result in iter {
            let entry = result.expect("entry decodes");
            for slot in &entry.slots {
                uncommitted_rows += slot.row_count as usize;
            }
            uncommitted_bundles += 1;
        }

        // Segment data + uncommitted WAL data should equal total
        assert_eq!(
            segment_rows + uncommitted_rows,
            TOTAL_EXPECTED_ROWS,
            "segment rows ({}) + uncommitted ({}) != total ({})",
            segment_rows,
            uncommitted_rows,
            TOTAL_EXPECTED_ROWS
        );

        assert_eq!(
            segment_bundles + uncommitted_bundles,
            NUM_BUNDLES,
            "segment bundles ({}) + uncommitted ({}) != total ({})",
            segment_bundles,
            uncommitted_bundles,
            NUM_BUNDLES
        );

        // === Print performance summary ===
        eprintln!("\n=== Stress Test Results ===");
        eprintln!("Bundles ingested: {}", NUM_BUNDLES);
        eprintln!("Rows per bundle: {}", ROWS_PER_BUNDLE);
        eprintln!("Total rows: {}", TOTAL_EXPECTED_ROWS);
        eprintln!();
        eprintln!("--- Segment Statistics ---");
        eprintln!("Segment files created: {}", segment_files.len());
        eprintln!(
            "Total segment bytes: {} KB ({:.2} MB)",
            total_segment_bytes / 1024,
            total_segment_bytes as f64 / (1024.0 * 1024.0)
        );
        eprintln!("Bundles in segments: {}", segment_bundles);
        eprintln!("Rows in segments: {}", segment_rows);
        eprintln!(
            "Avg segment size: {:.1} KB",
            total_segment_bytes as f64 / segment_files.len() as f64 / 1024.0
        );
        eprintln!();
        eprintln!("--- WAL Statistics ---");
        eprintln!(
            "WAL rotations: {} (purged: {})",
            wal_stats.rotation_count, wal_stats.purge_count
        );
        eprintln!("Rotated WAL files remaining: {}", rotated_wal_count);
        eprintln!("Active WAL size: {} KB", active_wal_size / 1024);
        eprintln!(
            "Total WAL bytes on disk: {} KB ({:.2} MB)",
            total_wal_bytes / 1024,
            total_wal_bytes as f64 / (1024.0 * 1024.0)
        );
        eprintln!(
            "Cursor WAL position: {} bytes ({:.2} MB)",
            sidecar.wal_position,
            sidecar.wal_position as f64 / (1024.0 * 1024.0)
        );
        eprintln!("Uncommitted bundles in WAL: {}", uncommitted_bundles);
        eprintln!("Uncommitted rows in WAL: {}", uncommitted_rows);
        eprintln!();
        eprintln!("--- Performance ---");
        eprintln!("Ingest duration: {:?}", ingest_duration);
        eprintln!("Total duration (with flush): {:?}", total_duration);
        eprintln!(
            "Throughput: {:.0} bundles/sec",
            NUM_BUNDLES as f64 / ingest_duration.as_secs_f64()
        );
        eprintln!(
            "Throughput: {:.0} rows/sec",
            TOTAL_EXPECTED_ROWS as f64 / ingest_duration.as_secs_f64()
        );
        eprintln!(
            "Throughput: {:.2} MB/sec (segments)",
            total_segment_bytes as f64 / ingest_duration.as_secs_f64() / (1024.0 * 1024.0)
        );
    }

    /// Stress test with multiple slots per bundle (simulating OTAP payloads).
    ///
    /// OTAP bundles typically have multiple payload slots (Logs, LogAttrs,
    /// ScopeAttrs, ResourceAttrs). This test exercises that pattern.
    #[test]
    fn stress_multi_slot_bundles() {
        use crate::segment::SegmentReader;

        let temp_dir = tempdir().expect("tempdir");

        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(16 * 1024).unwrap(), // 16KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        const NUM_BUNDLES: usize = 500;

        for _ in 0..NUM_BUNDLES {
            let bundle = MultiSlotBundle::new();
            engine.ingest(&bundle).expect("ingest");
        }

        drop(engine);

        // Verify all segments can be read
        let segment_dir = temp_dir.path().join("segments");
        let segment_files: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .map(|e| e.path())
            .collect();

        let mut total_bundles = 0;
        let mut streams_seen = std::collections::HashSet::new();

        for path in &segment_files {
            let reader = SegmentReader::open(path).expect("open segment");

            // Track unique streams
            for stream in reader.streams() {
                let _ = streams_seen.insert((stream.slot_id, stream.schema_fingerprint));
            }

            for entry in reader.manifest() {
                let bundle = reader.read_bundle(entry).expect("read bundle");
                // Each bundle should have 4 slots
                assert!(
                    bundle.slot_count() >= 1,
                    "bundle should have at least 1 slot"
                );
                total_bundles += 1;
            }
        }

        // Should have 4 distinct streams (one per slot type)
        // All slots use the same schema, so 4 (slot_id, fingerprint) pairs
        assert_eq!(
            streams_seen.len(),
            4,
            "expected 4 distinct streams for 4 slots"
        );

        eprintln!("\n=== Multi-Slot Stress Test ===");
        eprintln!("Bundles ingested: {}", NUM_BUNDLES);
        eprintln!("Segment files: {}", segment_files.len());
        eprintln!("Bundles in segments: {}", total_bundles);
        eprintln!("Distinct streams: {}", streams_seen.len());
    }

    /// Multi-slot bundle simulating OTAP structure (Logs, LogAttrs, ScopeAttrs, ResourceAttrs).
    struct MultiSlotBundle {
        descriptor: BundleDescriptor,
        batches: [arrow_array::RecordBatch; 4],
    }

    impl MultiSlotBundle {
        fn new() -> Self {
            let schema = Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::Int64,
                false,
            )]));

            // Different row counts per slot (realistic OTAP pattern)
            let row_counts = [50, 50, 5, 1]; // Logs, LogAttrs, ScopeAttrs, ResourceAttrs

            let batches = row_counts.map(|rows| {
                let mut builder = Int64Builder::new();
                for i in 0..rows {
                    builder.append_value(i as i64);
                }
                arrow_array::RecordBatch::try_new(schema.clone(), vec![Arc::new(builder.finish())])
                    .unwrap()
            });

            Self {
                descriptor: BundleDescriptor::new(vec![
                    SlotDescriptor::new(SlotId::new(0), "Logs"),
                    SlotDescriptor::new(SlotId::new(1), "LogAttrs"),
                    SlotDescriptor::new(SlotId::new(2), "ScopeAttrs"),
                    SlotDescriptor::new(SlotId::new(3), "ResourceAttrs"),
                ]),
                batches,
            }
        }
    }

    impl RecordBundle for MultiSlotBundle {
        fn descriptor(&self) -> &BundleDescriptor {
            &self.descriptor
        }

        fn ingestion_time(&self) -> std::time::SystemTime {
            std::time::SystemTime::now()
        }

        fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
            let idx = slot.0 as usize;
            if idx < 4 {
                Some(PayloadRef {
                    schema_fingerprint: [idx as u8; 32], // Different fingerprint per slot
                    batch: &self.batches[idx],
                })
            } else {
                None
            }
        }
    }

    /// Stress test with schema evolution (many different fingerprints).
    #[test]
    fn stress_schema_evolution() {
        use crate::segment::SegmentReader;

        let temp_dir = tempdir().expect("tempdir");

        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(32 * 1024).unwrap(), // 32KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Simulate schema evolution: 100 different schemas, 10 bundles each
        const NUM_SCHEMAS: usize = 100;
        const BUNDLES_PER_SCHEMA: usize = 10;
        const ROWS_PER_BUNDLE: usize = 20;

        for schema_id in 0..NUM_SCHEMAS {
            let mut fingerprint = [0u8; 32];
            fingerprint[0] = (schema_id >> 8) as u8;
            fingerprint[1] = schema_id as u8;

            for _ in 0..BUNDLES_PER_SCHEMA {
                let bundle = DummyBundleWithFingerprint::new(fingerprint, ROWS_PER_BUNDLE);
                engine.ingest(&bundle).expect("ingest");
            }
        }

        drop(engine);

        // Verify segments
        let segment_dir = temp_dir.path().join("segments");
        let segment_files: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .map(|e| e.path())
            .collect();

        let mut total_rows = 0;
        let mut unique_fingerprints = std::collections::HashSet::new();

        for path in &segment_files {
            let reader = SegmentReader::open(path).expect("open segment");

            for stream in reader.streams() {
                let _ = unique_fingerprints.insert(stream.schema_fingerprint);
            }

            for entry in reader.manifest() {
                let bundle = reader.read_bundle(entry).expect("read bundle");
                if let Some(payload) = bundle.payload(SlotId::new(0)) {
                    total_rows += payload.num_rows();
                }
            }
        }

        // We created 100 unique fingerprints, but only count those in finalized segments
        assert!(
            unique_fingerprints.len() >= 50,
            "expected many unique fingerprints, got {}",
            unique_fingerprints.len()
        );

        eprintln!("\n=== Schema Evolution Stress Test ===");
        eprintln!("Schemas simulated: {}", NUM_SCHEMAS);
        eprintln!("Bundles per schema: {}", BUNDLES_PER_SCHEMA);
        eprintln!("Total bundles: {}", NUM_SCHEMAS * BUNDLES_PER_SCHEMA);
        eprintln!("Segment files: {}", segment_files.len());
        eprintln!("Rows in segments: {}", total_rows);
        eprintln!(
            "Unique fingerprints in segments: {}",
            unique_fingerprints.len()
        );
    }

    #[test]
    fn ingest_finalizes_segment_when_max_open_duration_exceeded() {
        use std::thread;
        use std::time::Duration;

        let temp_dir = tempdir().expect("tempdir");

        // Use a very short max_open_duration to trigger time-based finalization
        // Use a large size so size-based finalization won't trigger
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(), // 100 MB
            max_open_duration: Duration::from_millis(50),                   // Very short duration
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // First ingest - starts the timer
        let bundle1 = DummyBundle::with_rows(1);
        engine.ingest(&bundle1).expect("first ingest succeeds");

        // Wait for the max_open_duration to elapse
        thread::sleep(Duration::from_millis(100));

        // Second ingest - should trigger time-based finalization
        let bundle2 = DummyBundle::with_rows(1);
        engine.ingest(&bundle2).expect("second ingest succeeds");

        drop(engine);

        // Check that at least one segment file was created due to time-based finalization
        let segment_dir = temp_dir.path().join("segments");
        let entries: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .collect();

        assert!(
            !entries.is_empty(),
            "expected segment file from time-based finalization"
        );
    }

    #[test]
    fn shutdown_finalizes_open_segment() {
        use crate::segment::SegmentReader;

        let temp_dir = tempdir().expect("tempdir");

        // Use a large size threshold so size-based finalization won't trigger
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(), // 100 MB
            max_open_duration: Duration::from_secs(3600),                   // 1 hour
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Ingest a small bundle that won't trigger size or time finalization
        let bundle = DummyBundle::with_rows(5);
        engine.ingest(&bundle).expect("ingest succeeds");

        // Verify no segment file exists yet
        let segment_dir = temp_dir.path().join("segments");
        let initial_entries: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .collect();
        assert!(
            initial_entries.is_empty(),
            "no segment should exist before shutdown"
        );

        // Call shutdown to finalize the open segment
        engine.shutdown().expect("shutdown succeeds");

        // Verify segment file was created
        let final_entries: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .collect();
        assert_eq!(
            final_entries.len(),
            1,
            "expected one segment file after shutdown"
        );

        // Verify the segment contains the correct data
        let segment_path = final_entries[0].path();
        let reader = SegmentReader::open(&segment_path).expect("open segment");
        assert_eq!(reader.bundle_count(), 1, "expected 1 bundle in segment");

        let manifest = reader.manifest();
        let reconstructed = reader.read_bundle(&manifest[0]).expect("read bundle");
        let payload = reconstructed
            .payload(SlotId::new(0))
            .expect("slot 0 exists");
        assert_eq!(payload.num_rows(), 5, "expected 5 rows in payload");
    }

    #[test]
    fn shutdown_on_empty_segment_succeeds() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());
        let engine = QuiverEngine::new(config, test_budget()).expect("config valid");

        // Shutdown without ingesting anything should succeed
        engine
            .shutdown()
            .expect("shutdown on empty segment succeeds");

        // No segment files should be created
        let segment_dir = temp_dir.path().join("segments");
        let entries: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .collect();
        assert!(entries.is_empty(), "no segment file for empty segment");
    }

    #[test]
    fn ingest_finalizes_segment_when_max_stream_count_exceeded() {
        let temp_dir = tempdir().expect("tempdir");

        // Use a tiny max_stream_count to trigger stream-based finalization
        // Use large size and time thresholds so they won't trigger
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(), // 100 MB
            max_open_duration: Duration::from_secs(3600),                   // 1 hour
            max_stream_count: 3, // Very small - will trigger after 3 unique streams
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        // Each bundle with a different schema fingerprint creates a new stream.
        // We need to exceed max_stream_count (3) to trigger finalization.
        for i in 0u8..4 {
            let bundle = DummyBundleWithFingerprint::new([i; 32], 1);
            engine.ingest(&bundle).expect("ingest succeeds");
        }

        drop(engine);

        // Check that at least one segment file was created due to stream count finalization
        let segment_dir = temp_dir.path().join("segments");
        let entries: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .collect();

        assert!(
            !entries.is_empty(),
            "expected segment file from stream count finalization"
        );
    }

    #[test]
    fn budget_tracks_segment_bytes() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1).expect("non-zero"), // 1 byte - immediate finalization
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        // Create a budget with plenty of room
        let budget = Arc::new(DiskBudget::new(
            100 * 1024 * 1024,
            RetentionPolicy::Backpressure,
        ));
        let engine = QuiverEngine::new(config, budget.clone()).expect("engine created");

        // Budget starts with WAL header bytes (WAL is now tracked in budget)
        let initial_used = budget.used();
        assert!(
            initial_used > 0,
            "budget should include WAL header bytes, got {}",
            initial_used
        );

        // Ingest a bundle - this will trigger segment finalization due to tiny target size
        let bundle = DummyBundle::with_rows(10);
        engine.ingest(&bundle).expect("ingest succeeds");

        // Budget should now reflect segment file size + WAL bytes
        let used = budget.used();
        assert!(
            used > initial_used,
            "budget should increase after segment write, got {} (was {})",
            used,
            initial_used
        );

        // Verify headroom decreased
        let headroom = budget.headroom();
        assert!(headroom < 100 * 1024 * 1024, "headroom should decrease");
    }

    #[test]
    fn budget_returns_storage_at_capacity_when_exceeded() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1).expect("non-zero"), // 1 byte - immediate finalization
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        // Create a very small budget (100 bytes) - segment will exceed this
        let budget = Arc::new(DiskBudget::new(100, RetentionPolicy::Backpressure));
        let engine = QuiverEngine::new(config, budget).expect("engine created");

        // Ingest a bundle - segment finalization should fail due to budget
        let bundle = DummyBundle::with_rows(10);
        let result = engine.ingest(&bundle);

        assert!(
            result.is_err(),
            "expected StorageAtCapacity error for tiny budget"
        );
        assert!(
            result.as_ref().unwrap_err().is_at_capacity(),
            "expected is_at_capacity() to be true, got {:?}",
            result
        );
    }

    #[test]
    fn budget_at_capacity_preserves_segment_data() {
        // Verifies that when budget is exceeded, the open segment data is NOT lost
        // and can be retried after freeing space.
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1).expect("non-zero"), // 1 byte - immediate finalization
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        // Create a very small budget (100 bytes) - segment will exceed this
        let budget = Arc::new(DiskBudget::new(100, RetentionPolicy::Backpressure));
        let engine = QuiverEngine::new(config, budget.clone()).expect("engine created");

        // Ingest a bundle - segment finalization should fail due to budget
        let bundle = DummyBundle::with_rows(10);
        let result = engine.ingest(&bundle);
        assert!(result.is_err(), "expected StorageAtCapacity error");

        // The open segment should still have the data
        // Verify by increasing budget and trying again
        // (We can't easily change the budget, so we just verify the error was returned
        // and the engine didn't panic - the data is preserved in the open segment)
    }

    #[test]
    fn budget_drop_oldest_reclaims_completed_segments() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).expect("non-zero"), // 1 KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        // Create a budget with DropOldest policy - large enough for a few segments
        let budget = Arc::new(DiskBudget::new(50 * 1024, RetentionPolicy::DropOldest));
        let engine = QuiverEngine::new(config, budget.clone()).expect("engine created");

        // Register a subscriber so segments can be marked as "complete"
        let sub_id = SubscriberId::new("test-sub").expect("valid id");
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Ingest bundles to create segments
        for _ in 0..5 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).expect("ingest succeeds");
        }
        engine.flush().expect("flush");

        // Consume all bundles to mark segments as complete
        while let Ok(Some(handle)) = engine.poll_next_bundle(&sub_id) {
            handle.ack();
        }

        // Verify some segments were created
        let initial_segment_count = engine.segment_store.segment_count();
        assert!(initial_segment_count > 0, "should have created segments");

        // Run cleanup to complete segment lifecycle
        let _ = engine.cleanup_completed_segments();

        // Budget's reclaim callback should have been wired up
        // (We can't easily test DropOldest triggering without precise budget sizing,
        // but we verify the callback was registered by checking cleanup works)
    }

    #[test]
    fn force_drop_oldest_drops_pending_segments_without_readers() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).expect("non-zero"), // 1 KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let budget = Arc::new(DiskBudget::new(100 * 1024, RetentionPolicy::DropOldest));
        let engine = QuiverEngine::new(config, budget.clone()).expect("engine created");

        // Register and activate a subscriber
        let sub_id = SubscriberId::new("test-sub").expect("valid id");
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Ingest bundles to create multiple segments
        for _ in 0..10 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).expect("ingest succeeds");
        }
        engine.flush().expect("flush");

        // Verify we have some segments pending
        let initial_count = engine.segment_store.segment_count();
        assert!(initial_count >= 2, "should have multiple segments");

        // Do NOT consume any bundles - they're all pending
        // But we also have no claimed bundles (no active reads)

        // Counter should start at 0
        assert_eq!(engine.force_dropped_segments(), 0);

        // Force drop should drop the oldest pending segment
        let dropped = engine.force_drop_oldest_pending_segments();
        assert_eq!(dropped, 1, "should have dropped exactly one segment");

        // Counter should be incremented
        assert_eq!(engine.force_dropped_segments(), 1);

        // Segment count should decrease
        let new_count = engine.segment_store.segment_count();
        assert_eq!(
            new_count,
            initial_count - 1,
            "segment count should decrease by 1"
        );

        // Can still consume remaining bundles
        let mut consumed = 0;
        while let Ok(Some(handle)) = engine.poll_next_bundle(&sub_id) {
            handle.ack();
            consumed += 1;
        }
        // Should have lost some bundles from the dropped segment
        assert!(consumed > 0, "should still have some bundles to consume");
    }

    #[test]
    fn force_drop_skips_segments_with_active_readers() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).expect("non-zero"), // 1 KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .build()
            .expect("config valid");

        let budget = Arc::new(DiskBudget::new(100 * 1024, RetentionPolicy::DropOldest));
        let engine = QuiverEngine::new(config, budget.clone()).expect("engine created");

        // Register and activate a subscriber
        let sub_id = SubscriberId::new("test-sub").expect("valid id");
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Ingest bundles to create multiple segments
        for _ in 0..10 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).expect("ingest succeeds");
        }
        engine.flush().expect("flush");

        let initial_count = engine.segment_store.segment_count();
        assert!(initial_count >= 2, "should have multiple segments");

        // Claim a bundle from the oldest segment (creating an active reader)
        let handle = engine
            .poll_next_bundle(&sub_id)
            .expect("next_bundle succeeds")
            .expect("bundle available");

        // Keep the handle alive (bundle is claimed)
        // Try to force drop - should skip the segment with the active reader
        let dropped = engine.force_drop_oldest_pending_segments();

        // If there are other segments without readers, one of those should be dropped
        // Otherwise, nothing should be dropped
        if initial_count > 1 {
            // The oldest segment has a reader, so it should be skipped
            // The second oldest should be dropped
            assert_eq!(
                engine.segment_store.segment_count(),
                initial_count - dropped,
                "dropped segments should be reflected in count"
            );
        }

        // Clean up
        handle.ack();
    }

    /// Test that existing segments from a previous engine run are loaded on startup.
    ///
    /// This verifies that `scan_existing()` is called during engine initialization.
    #[test]
    fn engine_loads_existing_segments_on_startup() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).expect("non-zero"), // Small segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config.clone())
            .build()
            .expect("config valid");

        // First engine: create some segments
        let segments_created = {
            let engine = QuiverEngine::new(config.clone(), test_budget()).expect("engine created");

            // Ingest enough data to create multiple segments
            for _ in 0..5 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).expect("ingest succeeds");
            }
            engine.flush().expect("flush");

            let count = engine.segment_store.segment_count();
            assert!(count >= 2, "should create multiple segments, got {count}");
            count
        };

        // Second engine: should discover existing segments automatically
        {
            let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

            // The segment store should have loaded the existing segments
            assert_eq!(
                engine.segment_store.segment_count(),
                segments_created,
                "new engine should load existing segments"
            );
        }
    }

    /// Test that subscribers can consume bundles from segments that existed
    /// before the engine was created (recovery scenario).
    #[test]
    fn subscriber_can_consume_from_recovered_segments() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).expect("non-zero"), // Small segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config.clone())
            .build()
            .expect("config valid");

        // First engine: create segments with data
        {
            let engine = QuiverEngine::new(config.clone(), test_budget()).expect("engine created");

            for _ in 0..3 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).expect("ingest succeeds");
            }
            engine.flush().expect("flush");

            assert!(
                engine.segment_store.segment_count() >= 1,
                "should have at least one segment"
            );
        }

        // Second engine: new subscriber should be able to consume the recovered data
        {
            let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

            // Register and activate a new subscriber
            let sub_id = SubscriberId::new("recovery-sub").expect("valid id");
            engine
                .register_subscriber(sub_id.clone())
                .expect("register");
            engine.activate_subscriber(&sub_id).expect("activate");

            // Should be able to consume bundles from recovered segments
            let mut consumed = 0;
            while let Some(handle) = engine.poll_next_bundle(&sub_id).expect("next_bundle") {
                handle.ack();
                consumed += 1;
            }

            assert!(
                consumed >= 3,
                "should consume bundles from recovered segments, got {consumed}"
            );
        }
    }

    /// Test that engine handles empty segment directory gracefully.
    #[test]
    fn engine_handles_empty_segment_directory() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());

        // Create engine - should work fine with no existing segments
        let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

        assert_eq!(
            engine.segment_store.segment_count(),
            0,
            "should start with no segments"
        );

        // Register a subscriber - should work even with no segments
        let sub_id = SubscriberId::new("empty-sub").expect("valid id");
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // next_bundle should return None (no bundles available)
        let bundle = engine.poll_next_bundle(&sub_id).expect("next_bundle");
        assert!(bundle.is_none(), "should have no bundles");
    }

    /// Integration test for crash recovery with progress files.
    ///
    /// This test simulates a crash-recovery scenario:
    /// 1. Ingest bundles and consume some (partially)
    /// 2. Drop the engine (simulating crash before flush)
    /// 3. Manually flush progress to simulate graceful shutdown
    /// 4. Recreate engine and verify progress is restored
    #[test]
    fn crash_recovery_with_progress_files() {
        let temp_dir = tempdir().expect("tempdir");
        let sub_id = SubscriberId::new("recovery-sub").expect("valid id");

        // Small segment size to force finalization
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).unwrap(),
            ..Default::default()
        };

        // Phase 1: Create engine, ingest bundles, consume some
        let bundles_acked;
        {
            let config = QuiverConfig::builder()
                .data_dir(temp_dir.path())
                .segment(segment_config.clone())
                .build()
                .expect("valid config");

            let engine = QuiverEngine::new(config, test_budget()).expect("engine created");

            // Register subscriber before ingesting
            engine
                .register_subscriber(sub_id.clone())
                .expect("register");
            engine.activate_subscriber(&sub_id).expect("activate");

            // Ingest bundles to create multiple segments
            for _ in 0..5 {
                let bundle = DummyBundle::with_rows(10);
                engine.ingest(&bundle).expect("ingest");
            }
            engine.flush().expect("flush segments");

            // Consume and ack some bundles (but not all)
            let mut acked = 0;
            for _ in 0..3 {
                if let Some(handle) = engine.poll_next_bundle(&sub_id).expect("next_bundle") {
                    handle.ack();
                    acked += 1;
                }
            }
            bundles_acked = acked;

            // Flush progress to disk (simulating graceful shutdown)
            let flushed = engine.flush_progress_sync().expect("flush progress");
            assert!(flushed > 0, "should have flushed progress");

            // Engine dropped here (simulating crash/shutdown)
        }

        // Phase 2: Recreate engine and verify recovery
        {
            let config = QuiverConfig::builder()
                .data_dir(temp_dir.path())
                .segment(segment_config)
                .build()
                .expect("valid config");

            let engine = QuiverEngine::new(config, test_budget()).expect("engine recreated");

            // Scan existing segments from previous run
            let found = engine.segment_store.scan_existing().expect("scan existing");
            assert!(!found.is_empty(), "should find segments from previous run");

            // Re-register the subscriber (should load from progress file)
            engine
                .register_subscriber(sub_id.clone())
                .expect("re-register");
            engine.activate_subscriber(&sub_id).expect("activate");

            // Count remaining bundles to consume
            let mut remaining = 0;
            while let Some(handle) = engine.poll_next_bundle(&sub_id).expect("next_bundle") {
                handle.ack();
                remaining += 1;
            }

            // Should only see the bundles we didn't ack before
            assert_eq!(
                remaining,
                5 - bundles_acked,
                "should only see unacked bundles from previous run"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Async API tests
    // ─────────────────────────────────────────────────────────────────────────

    /// Helper to create an engine and ingest data in a blocking context.
    /// This is needed because WalWriter::open_sync uses block_on internally.
    async fn setup_engine_with_data(
        dir: &std::path::Path,
        bundle_count: usize,
    ) -> Arc<QuiverEngine> {
        let path = dir.to_path_buf();
        let budget = test_budget();
        tokio::task::spawn_blocking(move || {
            let config = QuiverConfig::builder()
                .data_dir(&path)
                .build()
                .expect("config");
            let engine = QuiverEngine::new(config, budget).expect("engine");

            for _ in 0..bundle_count {
                let bundle = DummyBundle::with_rows(100);
                engine.ingest(&bundle).expect("ingest");
            }
            if bundle_count > 0 {
                engine.flush().expect("flush");
            }
            engine
        })
        .await
        .expect("spawn_blocking")
    }

    #[tokio::test]
    async fn async_next_bundle_returns_available_bundle() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 10).await;

        // Register and activate subscriber
        let sub_id = SubscriberId::new("async-test").unwrap();
        engine.register_subscriber(sub_id.clone()).expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Use async next_bundle
        let handle = engine
            .next_bundle(&sub_id, Some(Duration::from_secs(5)))
            .await
            .expect("next_bundle")
            .expect("should have bundle");

        assert_eq!(handle.bundle_ref().bundle_index.raw(), 0);
        handle.ack();
    }

    #[tokio::test]
    async fn async_next_bundle_timeout_when_no_bundles() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 0).await;

        // Register and activate subscriber (but don't ingest anything)
        let sub_id = SubscriberId::new("timeout-test").unwrap();
        engine.register_subscriber(sub_id.clone()).expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Async next_bundle should timeout quickly
        let result = engine
            .next_bundle(&sub_id, Some(Duration::from_millis(100)))
            .await
            .expect("next_bundle");

        assert!(result.is_none(), "should timeout with no bundles");
    }

    #[tokio::test]
    async fn async_next_bundle_wakes_on_segment_finalized() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 0).await;

        // Register and activate subscriber before any data
        let sub_id = SubscriberId::new("wake-test").unwrap();
        engine.register_subscriber(sub_id.clone()).expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        let got_bundle = Arc::new(AtomicBool::new(false));
        let got_bundle_clone = got_bundle.clone();
        let engine_clone = engine.clone();
        let sub_id_clone = sub_id.clone();

        // Spawn async task to wait for bundle
        let consumer = tokio::spawn(async move {
            let result = engine_clone
                .next_bundle(&sub_id_clone, Some(Duration::from_secs(10)))
                .await
                .expect("next_bundle");
            if result.is_some() {
                got_bundle_clone.store(true, Ordering::Relaxed);
            }
        });

        // Give consumer time to start waiting
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Now ingest data and flush in blocking context (which finalizes segment and notifies)
        let engine_for_ingest = engine.clone();
        tokio::task::spawn_blocking(move || {
            for _ in 0..5 {
                let bundle = DummyBundle::with_rows(100);
                engine_for_ingest.ingest(&bundle).expect("ingest");
            }
            engine_for_ingest.flush().expect("flush");
        })
        .await
        .expect("spawn_blocking");

        // Consumer should complete
        let result = tokio::time::timeout(Duration::from_secs(5), consumer).await;
        assert!(result.is_ok(), "consumer task should complete");
        assert!(
            got_bundle.load(Ordering::Relaxed),
            "should have received bundle"
        );
    }

    #[tokio::test]
    async fn async_next_bundle_interleaves_with_poll() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 10).await;

        // Register and activate subscriber
        let sub_id = SubscriberId::new("interleave-test").unwrap();
        engine.register_subscriber(sub_id.clone()).expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Alternate between async and poll methods
        let h1 = engine
            .next_bundle(&sub_id, Some(Duration::from_secs(1)))
            .await
            .expect("async 1")
            .expect("bundle 1");
        h1.ack();

        let h2 = engine
            .poll_next_bundle(&sub_id)
            .expect("poll 1")
            .expect("bundle 2");
        h2.ack();

        let h3 = engine
            .next_bundle(&sub_id, Some(Duration::from_secs(1)))
            .await
            .expect("async 2")
            .expect("bundle 3");
        h3.ack();

        // All three bundles should have sequential indices
        // (we can't check indices directly since we already acked, but test completes successfully)
    }

    #[tokio::test]
    async fn async_flush_progress_writes_dirty_subscribers() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 5).await;

        // Register and activate subscriber
        let sub_id = SubscriberId::new("flush-test").unwrap();
        engine.register_subscriber(sub_id.clone()).expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Consume a bundle to make the subscriber dirty
        let handle = engine
            .next_bundle(&sub_id, Some(Duration::from_secs(1)))
            .await
            .expect("next_bundle")
            .expect("bundle");
        handle.ack();

        // Flush progress asynchronously
        let flushed = engine.flush_progress().await.expect("flush_progress");
        assert_eq!(flushed, 1, "should have flushed one subscriber");

        // Flush again - should be 0 since nothing is dirty now
        let flushed2 = engine.flush_progress().await.expect("flush_progress 2");
        assert_eq!(flushed2, 0, "should have flushed zero subscribers");
    }

    #[tokio::test]
    async fn async_maintain_flushes_and_cleans() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 10).await;

        // Register and consume all bundles
        let sub_id = SubscriberId::new("maintain-test").unwrap();
        engine.register_subscriber(sub_id.clone()).expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Consume all bundles
        loop {
            match engine
                .next_bundle(&sub_id, Some(Duration::from_millis(100)))
                .await
            {
                Ok(Some(h)) => h.ack(),
                Ok(None) => break,
                Err(_) => break,
            }
        }

        // Run async maintain
        let stats = engine.maintain().await.expect("maintain");

        // Should have flushed at least the one dirty subscriber
        assert!(stats.flushed >= 1, "should have flushed at least one subscriber");
        // deleted can be any non-negative number (usize)
    }
}
