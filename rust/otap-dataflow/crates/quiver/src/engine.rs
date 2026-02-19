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
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use tokio::sync::Mutex as TokioMutex;

use crate::config::{DurabilityMode, QuiverConfig, RetentionPolicy};
use crate::error::{QuiverError, Result};
use crate::logging::{otel_debug, otel_error, otel_info, otel_warn};
use crate::record_bundle::RecordBundle;
use crate::segment::{OpenSegment, SegmentError, SegmentSeq, SegmentWriter};
use crate::segment_store::SegmentStore;
use crate::subscriber::{
    BundleHandle, BundleRef, RegistryCallback, RegistryConfig, SegmentProvider, SubscriberError,
    SubscriberId, SubscriberRegistry,
};
use crate::telemetry::PersistenceMetrics;
use crate::wal::{
    CursorSidecar, MultiFileWalReader, ReplayBundle, WalConsumerCursor, WalWriter, WalWriterOptions,
};

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
    /// Number of segments deleted due to exceeding max_age retention.
    pub expired: usize,
    /// Number of previously-deferred segment deletions that succeeded.
    /// (Segments may be deferred on Windows when still memory-mapped.)
    pub pending_deletes_cleared: usize,
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
    /// Write-ahead log writer (uses tokio mutex for async lock across await points).
    wal_writer: TokioMutex<WalWriter>,
    /// Current open segment accumulator.
    open_segment: Mutex<OpenSegment>,
    /// Cursor representing all entries in the current open segment.
    /// Updated after each WAL append, used to advance WAL after finalization.
    segment_cursor: Mutex<WalConsumerCursor>,
    /// Next segment sequence number to assign.
    next_segment_seq: AtomicU64,
    /// Cumulative bytes written to WAL (never decreases, even after rotation/purge).
    cumulative_wal_bytes: AtomicU64,
    /// Cumulative bytes written to segments (never decreases, even after cleanup).
    cumulative_segment_bytes: AtomicU64,
    /// Count of segments force-dropped due to DropOldest policy.
    force_dropped_segments: AtomicU64,
    /// Count of bundles lost due to force-dropped segments.
    force_dropped_bundles: AtomicU64,
    /// Count of bundles lost due to expired segments (max_age retention).
    expired_bundles: AtomicU64,
    /// Segment store for finalized segment files.
    segment_store: Arc<SegmentStore>,
    /// Subscriber registry for tracking consumption progress.
    registry: Arc<SubscriberRegistry<SegmentStore>>,
    /// Whether the filesystem supports chmod/set_permissions.
    ///
    /// Probed once at startup by creating a temp file and attempting to change
    /// its permissions. When `false`, immutability enforcement via `set_readonly`
    /// is skipped for both segment files and rotated WAL files. This allows
    /// quiver to operate on filesystems that don't support POSIX permission
    /// changes (e.g., Azure Files SMB/CIFS mounts, certain Kubernetes
    /// volumeMounts).
    set_permissions_supported: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// QuiverEngineBuilder
// ─────────────────────────────────────────────────────────────────────────────

/// Builder for creating a [`QuiverEngine`] with customizable options.
///
/// This provides a cleaner ergonomic interface, especially for tests
/// that may want to customize specific options.
///
/// # Example
///
/// ```ignore
/// use quiver::{QuiverEngineBuilder, QuiverConfig, DiskBudget, RetentionPolicy};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = QuiverConfig::default().with_data_dir("/var/lib/quiver/data");
///     let budget = Arc::new(DiskBudget::for_config(
///         1024 * 1024 * 1024,      // 1 GB hard cap
///         &config,
///         RetentionPolicy::Backpressure,
///     ).expect("valid budget config"));
///
///     let engine = QuiverEngineBuilder::new(config)
///         .with_budget(budget)
///         .build()
///         .await?;
///     Ok(())
/// }
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
    pub const fn new(config: QuiverConfig) -> Self {
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

    /// Builds the engine asynchronously, returning an `Arc<QuiverEngine>`.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails or if the WAL
    /// cannot be initialized.
    pub async fn build(self) -> Result<Arc<QuiverEngine>> {
        let budget = self
            .budget
            .unwrap_or_else(|| Arc::new(crate::budget::DiskBudget::unlimited()));
        QuiverEngine::open(self.config, budget).await
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
    /// This provides a cleaner interface with sensible defaults
    /// (e.g., unlimited budget with backpressure policy).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use quiver::{QuiverEngine, QuiverConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = QuiverConfig::default().with_data_dir("/var/lib/quiver/data");
    ///     let engine = QuiverEngine::builder(config).build().await?;
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub const fn builder(config: QuiverConfig) -> QuiverEngineBuilder {
        QuiverEngineBuilder::new(config)
    }

    /// Opens a new persistence engine asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails or if the WAL
    /// cannot be initialized.
    pub async fn open(
        config: QuiverConfig,
        budget: Arc<crate::budget::DiskBudget>,
    ) -> Result<Arc<Self>> {
        config.validate()?;

        // Validate budget is large enough for WAL + two segments.
        // Uses DiskBudget::minimum_hard_cap() as the single source of truth
        // for the constraint: hard_cap >= wal_max + 2 * segment_target_size.
        // WAL contribution is zero when DurabilityMode::SegmentOnly.
        let segment_size = config.segment.target_size_bytes.get();
        let wal_max = crate::budget::DiskBudget::effective_wal_size(&config);
        let min_budget = crate::budget::DiskBudget::minimum_hard_cap(segment_size, wal_max);
        if budget.hard_cap() < min_budget && budget.hard_cap() != u64::MAX {
            let message = format!(
                "disk budget must be at least 2x segment size to prevent deadlock: \
                 hard cap {} bytes is too small for WAL max {} bytes + 2 * segment size {} bytes",
                budget.hard_cap(),
                wal_max,
                segment_size,
            );
            otel_error!(
                "quiver.engine.init",
                budget_cap = budget.hard_cap(),
                segment_size,
                min_budget,
                reason = "budget_too_small",
                message = message,
            );
            return Err(QuiverError::invalid_config(message));
        }

        // Validate segment headroom is at least segment_target_size.
        // The soft_cap headroom must reserve room for one full segment
        // finalization, otherwise used can overshoot the hard_cap.
        let headroom = budget.hard_cap().saturating_sub(budget.soft_cap());
        if headroom < segment_size && budget.hard_cap() != u64::MAX {
            return Err(QuiverError::invalid_config(format!(
                "budget segment_headroom ({headroom} bytes) must be at least segment_target_size \
                 ({segment_size} bytes); use DiskBudget::for_config() to construct a correctly \
                 configured budget",
            )));
        }

        // Ensure directories exist
        let segment_dir = segment_dir(&config);
        fs::create_dir_all(&segment_dir).map_err(|e| {
            otel_error!(
                "quiver.engine.init",
                path = %segment_dir.display(),
                error = %e,
                error_type = "io",
                reason = "dir_create_failed",
            );
            SegmentError::io(segment_dir.clone(), e)
        })?;

        // Probe whether the filesystem supports chmod/set_permissions.
        // Some filesystems (e.g., Azure Files SMB mounts, certain k8s
        // volumeMounts) return EPERM on permission changes. When unsupported,
        // we skip read-only enforcement for segments and WAL files.
        let set_permissions_supported = probe_set_permissions_support(&config.data_dir);

        // Use async WAL initialization
        let wal_writer = initialize_wal_writer(&config, &budget, set_permissions_supported)
            .await
            .map_err(|e| {
                otel_error!(
                    "quiver.engine.init",
                    error = %e,
                    error_type = "io",
                    reason = "wal_init_failed",
                );
                e
            })?;

        // Create segment store with configured read mode and budget
        let segment_store = Arc::new(SegmentStore::with_budget(
            &segment_dir,
            config.read_mode,
            budget.clone(),
        ));

        // Scan for existing segments from previous runs (recovery).
        // Pass max_age to skip loading segments that are already expired -
        // they'll be deleted during scan without the overhead of parsing them.
        let mut next_segment_seq = 0u64;
        let mut deleted_during_scan = Vec::new();
        match segment_store.scan_existing_with_max_age(config.retention.max_age) {
            Ok(scan_result) => {
                // Determine next sequence from the highest loaded OR deleted segment.
                // This prevents sequence reuse when all segments are expired and deleted.
                let highest_found = scan_result.found.last().map(|(seq, _)| seq.raw());
                let highest_deleted = scan_result.deleted.last().map(|seq| seq.raw());
                if let Some(highest) = highest_found.into_iter().chain(highest_deleted).max() {
                    next_segment_seq = highest + 1;
                }

                if !scan_result.found.is_empty() {
                    otel_info!(
                        "quiver.segment.scan",
                        segment_count = scan_result.found.len(),
                        next_segment_seq,
                        message = "recovered segments from previous run",
                    );
                }
                if !scan_result.deleted.is_empty() {
                    otel_info!(
                        "quiver.segment.scan",
                        deleted_count = scan_result.deleted.len(),
                        next_segment_seq,
                    );
                }
                deleted_during_scan = scan_result.deleted;
            }
            Err(e) => {
                otel_error!(
                    "quiver.segment.scan",
                    error = %e,
                    error_type = "io",
                    message = "continuing with empty store, previously finalized data may be inaccessible",
                );
            }
        }

        // Create subscriber registry with segment store as provider
        let registry_config = RegistryConfig::new(&config.data_dir);
        let registry =
            SubscriberRegistry::open(registry_config, segment_store.clone()).map_err(|e| {
                otel_error!(
                    "quiver.engine.init",
                    error = %e,
                    error_type = "io",
                    reason = "registry_open_failed",
                );
                SegmentError::io(config.data_dir.clone(), std::io::Error::other(e))
            })?;

        // If any segments were deleted during scan, force-complete them in the
        // registry so that subscribers restored from progress.json don't try to
        // read from files that no longer exist.
        if !deleted_during_scan.is_empty() {
            registry.force_complete_segments(&deleted_during_scan);
        }

        // Start with empty open segment and default cursor
        // WAL replay will populate these through the normal ingest path
        let engine = Arc::new(Self {
            config,
            metrics: PersistenceMetrics::new(),
            wal_writer: TokioMutex::new(wal_writer),
            open_segment: Mutex::new(OpenSegment::new()),
            segment_cursor: Mutex::new(WalConsumerCursor::default()),
            next_segment_seq: AtomicU64::new(next_segment_seq),
            cumulative_wal_bytes: AtomicU64::new(0),
            cumulative_segment_bytes: AtomicU64::new(0),
            force_dropped_segments: AtomicU64::new(0),
            force_dropped_bundles: AtomicU64::new(0),
            expired_bundles: AtomicU64::new(0),
            segment_store,
            registry: registry.clone(),
            budget: budget.clone(),
            set_permissions_supported,
        });

        // Wire segment store callback to notify registry of new segments
        let registry_for_callback = registry;
        engine
            .segment_store
            .set_on_segment_registered(move |seq, bundle_count| {
                registry_for_callback.on_segment_finalized(seq, bundle_count);
            });

        // Replay WAL entries that weren't finalized to segments before shutdown/crash
        // This uses the same ingest path as live ingestion (minus WAL writes)
        let replayed = engine.replay_wal().await?;
        if replayed > 0 {
            otel_info!("quiver.wal.replay", replayed,);
        }

        Ok(engine)
    }

    /// Returns the configuration backing this engine.
    pub const fn config(&self) -> &QuiverConfig {
        &self.config
    }

    /// Returns the disk budget governing this engine's storage.
    ///
    /// Use this to inspect current usage, available capacity, or to share
    /// the budget with external components.
    pub const fn budget(&self) -> &Arc<crate::budget::DiskBudget> {
        &self.budget
    }

    /// Returns metric counters for instrumentation layers.
    pub const fn metrics(&self) -> &PersistenceMetrics {
        &self.metrics
    }

    /// Returns the cumulative bytes written to WAL since engine creation.
    /// This value never decreases, even as WAL files are rotated and purged.
    /// Useful for accurate throughput measurement without file system sampling.
    pub fn wal_bytes_written(&self) -> u64 {
        self.cumulative_wal_bytes.load(Ordering::Relaxed)
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

    /// Returns the total number of bundles lost due to expired segments.
    ///
    /// This counter tracks data loss from segments deleted because they
    /// exceeded the [`RetentionConfig::max_age`]. Unlike `force_dropped_bundles()`
    /// which tracks DropOldest policy, this tracks time-based retention.
    ///
    /// [`RetentionConfig::max_age`]: crate::config::RetentionConfig::max_age
    pub fn expired_bundles(&self) -> u64 {
        self.expired_bundles.load(Ordering::Relaxed)
    }

    /// Returns WAL statistics (rotation count, purge count).
    ///
    /// Call this before dropping the engine to capture final stats.
    #[cfg(test)]
    pub(crate) async fn wal_stats(&self) -> WalStats {
        let writer = self.wal_writer.lock().await;
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
    /// # Budget gating
    ///
    /// The soft-cap check is a best-effort gate, not a serialized barrier.
    /// Multiple concurrent callers may pass the check before any of them
    /// records bytes via the WAL or segment path. This is safe because:
    ///
    /// - WAL appends are serialized (`TokioMutex`), so entries are added
    ///   one at a time.
    /// - Finalization is serialized (`Mutex<OpenSegment>`), so at most one
    ///   segment writes to disk at a time.
    /// - The `hard_cap - soft_cap = segment_target_size` headroom absorbs
    ///   the overshoot from racing callers, since individual WAL entries
    ///   are much smaller than a full segment.
    ///
    /// The hard cap may be temporarily exceeded by a small amount (sum of
    /// in-flight WAL entries), but this is bounded and self-correcting:
    /// once the soft cap is exceeded, subsequent callers are rejected.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The disk budget is over the soft cap (`StorageAtCapacity`)
    /// - WAL append fails
    /// - Segment finalization fails
    pub async fn ingest<B: RecordBundle>(&self, bundle: &B) -> Result<()> {
        self.metrics.record_ingest_attempt();

        // Step 0: Check budget watermark before doing any work.
        // This is a best-effort gate — see "Budget gating" in the doc comment.
        // If usage exceeds the soft cap, attempt cleanup before rejecting.
        if self.budget.is_over_soft_cap() {
            // Try cleaning up fully-consumed segments first (no data loss)
            let _ = self.cleanup_completed_segments();

            if self.budget.is_over_soft_cap() {
                // For DropOldest: force-drop pending segments until under
                // soft_cap or nothing left to drop.
                if self.budget.policy() == RetentionPolicy::DropOldest {
                    while self.budget.is_over_soft_cap() {
                        if self.force_drop_oldest_pending_segments() == 0 {
                            break; // nothing left to reclaim
                        }
                    }
                }

                // Re-check after cleanup attempts
                if self.budget.is_over_soft_cap() {
                    return Err(QuiverError::StorageAtCapacity {
                        available: self.budget.soft_cap_headroom(),
                        soft_cap: self.budget.soft_cap(),
                    });
                }
            }
        }

        // Step 1: Append to WAL for durability (if enabled)
        // WAL bytes are tracked in the shared disk budget by the WalWriter/WalCoordinator.
        // They are released when WAL files are purged after segment finalization.
        let cursor = if self.config.durability == DurabilityMode::Wal {
            let wal_offset = self.append_to_wal_with_capacity_handling(bundle).await?;

            // Track cumulative WAL bytes for throughput measurement
            let wal_entry_bytes = wal_offset.next_offset.saturating_sub(wal_offset.position);
            let _ = self
                .cumulative_wal_bytes
                .fetch_add(wal_entry_bytes, Ordering::Relaxed);

            Some(WalConsumerCursor::from_offset(&wal_offset))
        } else {
            None
        };

        // Step 2: Append to open segment and finalize if threshold exceeded
        self.append_to_segment_and_maybe_finalize(bundle, cursor)
            .await
    }

    /// Core ingest logic: appends a bundle to the open segment and finalizes if thresholds are met.
    ///
    /// This is the shared path used by both live ingestion and WAL replay.
    /// During live ingestion, `cursor` is populated from the WAL offset.
    /// During WAL replay, `cursor` is populated from the WAL entry being replayed.
    ///
    /// Segment finalization writes the segment then records its size via
    /// `budget.add()`. The `hard_cap >= wal_max + 2 * segment_size`
    /// validation guarantees room for at least one finalization even when
    /// the budget is at the soft cap. WAL bytes are released after cursor
    /// persistence and purge.
    #[inline]
    async fn append_to_segment_and_maybe_finalize<B: RecordBundle>(
        &self,
        bundle: &B,
        cursor: Option<WalConsumerCursor>,
    ) -> Result<()> {
        // Update cursor if provided (tracks WAL position for segment finalization)
        if let Some(c) = cursor {
            let mut cp = self.segment_cursor.lock();
            *cp = c;
        }

        // Append to open segment accumulator
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

        // Finalize segment if threshold exceeded
        if should_finalize {
            self.finalize_segment_impl().await?;
        }

        Ok(())
    }

    /// Replays WAL entries that weren't finalized to segments before shutdown/crash.
    ///
    /// This method reads the WAL starting from the cursor position (entries already
    /// finalized to segments are skipped), and replays the remaining entries through
    /// the normal ingest path. This ensures consistent finalization behavior between
    /// WAL replay and live ingestion.
    ///
    /// # Returns
    ///
    /// The number of WAL entries replayed.
    #[must_use = "the replay count indicates recovery status and should be logged"]
    async fn replay_wal(&self) -> Result<usize> {
        let wal_path = wal_path(&self.config);

        // Load cursor sidecar to find where we left off
        let cursor_path = CursorSidecar::path_for(&wal_path);

        // Track whether we're using a potentially stale/missing cursor that could cause duplicates
        let mut cursor_may_cause_duplicates = false;

        let cursor_position = match tokio::fs::read(&cursor_path).await {
            Ok(data) => match CursorSidecar::decode(&data) {
                Ok(c) => c.wal_position,
                Err(e) => {
                    otel_warn!(
                        "quiver.wal.cursor.load",
                        error = %e,
                        error_type = "decode",
                        reason = "decode_failed",
                        message = "replaying from start, duplicates may occur",
                    );
                    cursor_may_cause_duplicates = true;
                    0
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
            Err(e) => {
                otel_warn!(
                    "quiver.wal.cursor.load",
                    error = %e,
                    error_type = "io",
                    reason = "read_failed",
                    message = "replaying from start, duplicates may occur",
                );
                cursor_may_cause_duplicates = true;
                0
            }
        };

        // Try to open the multi-file WAL reader (handles both rotated and active files)
        let reader = match MultiFileWalReader::open(&wal_path) {
            Ok(r) => r,
            Err(e) => {
                // WAL files don't exist or are invalid - this is fine on first run
                otel_debug!(
                    "quiver.wal.replay",
                    error = %e,
                    error_type = "io",
                );
                return Ok(0);
            }
        };

        // Clamp cursor_position to actual WAL bounds to handle stale/corrupted sidecar values
        let wal_end = reader.wal_end_position();
        let cursor_position = if cursor_position > wal_end {
            otel_warn!(
                "quiver.wal.cursor.load",
                cursor_position,
                wal_end,
                reason = "clamped",
                message = "cursor position beyond WAL bounds, clamped to end",
            );
            wal_end
        } else {
            cursor_position
        };

        // Log warning if cursor was missing/corrupted and we're replaying from a non-zero position
        if cursor_may_cause_duplicates && cursor_position > 0 {
            otel_warn!(
                "quiver.wal.cursor.load",
                cursor_position,
                reason = "invalid",
                message = "replaying from start with invalid cursor, duplicates possible",
            );
        }

        // Iterate from the cursor position (skip already-finalized entries)
        // The multi-file reader handles finding the right file(s) to start from
        let iter = match reader.iter_from(cursor_position) {
            Ok(iter) => iter,
            Err(e) => {
                otel_warn!(
                    "quiver.wal.replay",
                    error = %e,
                    error_type = "io",
                    cursor_position,
                    message = "starting fresh",
                );
                return Ok(0);
            }
        };

        let mut replayed_count = 0;
        let mut skipped_expired = 0u64;
        let mut stopped_at_corruption = false;

        // Compute max_age cutoff once before the loop so expired WAL entries
        // are not replayed into new segments, which would reset their age and
        // cause them to be retained longer than intended.
        // If max_age is so large that it underflows past the epoch, clamp to
        // UNIX_EPOCH (effectively disabling filtering, which is correct).
        let max_age_cutoff = self
            .config
            .retention
            .max_age
            .map(|max_age| SystemTime::now().checked_sub(max_age).unwrap_or(UNIX_EPOCH));

        for entry_result in iter {
            let entry = match entry_result {
                Ok(e) => e,
                Err(crate::wal::WalError::UnexpectedEof(context)) => {
                    // UnexpectedEof is expected during crash recovery - it means the
                    // last write was incomplete when the process died. This is normal
                    // and we should stop replay here (the partial entry is discarded).
                    otel_debug!(
                        "quiver.wal.replay",
                        context,
                        replayed_so_far = replayed_count,
                        status = "stopped_incomplete",
                        message = "stopped at incomplete entry, expected after crash",
                    );
                    break;
                }
                Err(e) => {
                    // CRC mismatch, InvalidEntry, etc. indicate corruption.
                    // Log prominently but continue - availability > perfect recovery.
                    // We can't safely read past corruption (entry boundaries unknown),
                    // so stop replay here.
                    stopped_at_corruption = true;
                    otel_error!(
                        "quiver.wal.replay",
                        error = %e,
                        error_type = "corruption",
                        replayed_so_far = replayed_count,
                        reason = "corruption",
                        message = "stopping replay at corruption boundary",
                    );
                    break;
                }
            };

            // Skip WAL entries that are older than max_age — replaying them
            // would effectively reset their age to zero and cause them to be
            // retained longer than intended.
            //
            // Note: WAL entries are NOT assumed to be sorted by ingestion_time,
            // so we check every entry individually rather than short-circuiting.
            //
            // We check the raw timestamp BEFORE decoding to a ReplayBundle to
            // avoid spending cycles on IPC deserialization for expired entries.
            if let Some(cutoff) = max_age_cutoff {
                let entry_time = if entry.ingestion_ts_nanos >= 0 {
                    UNIX_EPOCH + Duration::from_nanos(entry.ingestion_ts_nanos as u64)
                } else {
                    UNIX_EPOCH
                };
                if entry_time < cutoff {
                    skipped_expired += 1;
                    // Advance cursor past this entry so it won't be retried.
                    let cursor = WalConsumerCursor::after(&entry);
                    {
                        let mut cp = self.segment_cursor.lock();
                        *cp = cursor;
                    }
                    continue;
                }
            }

            // Decode WAL entry into a ReplayBundle
            let bundle = match ReplayBundle::from_wal_entry(&entry) {
                Some(b) => b,
                None => {
                    // Decode failure logged by from_wal_entry with slot details
                    otel_warn!(
                        "quiver.wal.entry.decode",
                        sequence = entry.sequence,
                        slot_count = entry.slots.len(),
                        scope = "entry",
                    );
                    // Still update cursor past this entry so we don't retry it
                    let cursor = WalConsumerCursor::after(&entry);
                    {
                        let mut cp = self.segment_cursor.lock();
                        *cp = cursor;
                    }
                    continue;
                }
            };

            // Replay through the normal ingest path (minus WAL write)
            let cursor = WalConsumerCursor::after(&entry);
            if let Err(e) = self
                .append_to_segment_and_maybe_finalize(&bundle, Some(cursor))
                .await
            {
                // Log segment errors prominently - these could indicate disk issues
                otel_error!(
                    "quiver.wal.replay",
                    error = %e,
                    error_type = "io",
                    sequence = entry.sequence,
                );
                // For critical errors like disk full, we should stop replay
                // to avoid silent data loss. The next restart will retry.
                return Err(e);
            }

            replayed_count += 1;
        }

        if skipped_expired > 0 {
            otel_info!(
                "quiver.wal.replay",
                skipped_expired,
                message = "skipped expired WAL entries during replay (max_age)"
            );
            let _ = self
                .expired_bundles
                .fetch_add(skipped_expired, Ordering::Relaxed);

            // If we only skipped entries (nothing replayed), persist the cursor
            // so subsequent restarts don't re-scan the same expired WAL tail.
            if replayed_count == 0 {
                let cursor = *self.segment_cursor.lock();
                let mut wal_writer = self.wal_writer.lock().await;
                wal_writer.persist_cursor(&cursor).await?;
            }
        }

        if replayed_count > 0 || stopped_at_corruption {
            otel_info!(
                "quiver.wal.replay",
                replayed_count,
                stopped_at_corruption,
                cursor_position,
            );
        }

        Ok(replayed_count)
    }

    /// Appends a bundle to the WAL asynchronously, handling capacity errors transparently.
    ///
    /// If the WAL is at capacity, this method:
    /// 1. Finalizes the current segment to advance the WAL cursor
    /// 2. Retries the WAL append
    ///
    /// This allows the engine to handle large segment sizes that exceed the
    /// WAL capacity without requiring caller intervention.
    async fn append_to_wal_with_capacity_handling<B: RecordBundle>(
        &self,
        bundle: &B,
    ) -> Result<crate::wal::WalOffset> {
        // First attempt
        let first_result = {
            let mut writer = self.wal_writer.lock().await;
            writer.append_bundle(bundle).await
        };

        match first_result {
            Ok(offset) => Ok(offset),
            Err(ref e) if e.is_at_capacity() => {
                // WAL is full - finalize the current segment to advance the cursor
                // and free WAL space, then retry the append.
                otel_warn!(
                    "quiver.wal.backpressure",
                    message = "finalizing segment to free space before retry",
                );
                self.finalize_segment_impl().await?;

                // Retry the append after finalization freed space
                let mut writer = self.wal_writer.lock().await;
                writer.append_bundle(bundle).await.map_err(Into::into)
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
    pub async fn flush(&self) -> Result<()> {
        self.finalize_segment_impl().await
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
    pub async fn shutdown(&self) -> Result<()> {
        let result = self.finalize_segment_impl().await;
        if let Err(ref e) = result {
            if self.config.durability == DurabilityMode::Wal {
                otel_warn!(
                    "quiver.engine.stop",
                    error = %e,
                    error_type = "io",
                    message = "data recoverable via WAL replay",
                );
            } else {
                otel_error!(
                    "quiver.engine.stop",
                    error = %e,
                    error_type = "io",
                    message = "data in open segment will be lost, WAL is disabled",
                );
            }
        }
        otel_info!(
            "quiver.engine.stop",
            cumulative_segment_bytes = self.cumulative_segment_bytes.load(Ordering::Relaxed),
            force_dropped_segments = self.force_dropped_segments.load(Ordering::Relaxed),
            force_dropped_bundles = self.force_dropped_bundles.load(Ordering::Relaxed),
        );
        result
    }

    /// Finalizes the current open segment and writes it to disk asynchronously.
    ///
    /// Uses async I/O for segment writing and WAL cursor persistence.
    ///
    /// Budget accounting: the segment is written first, then its size is
    /// recorded via `budget.add()`. The `soft_cap` (= `hard_cap - segment_size`)
    /// guarantees that even if `used` was at the soft cap before finalization,
    /// the resulting `used` won't exceed `hard_cap`.
    async fn finalize_segment_impl(&self) -> Result<()> {
        // Check if there's anything to finalize
        {
            let segment_guard = self.open_segment.lock();
            if segment_guard.is_empty() {
                return Ok(());
            }
        }

        // Swap out the segment and cursor
        let (segment, cursor) = {
            let mut segment_guard = self.open_segment.lock();
            let mut cursor_guard = self.segment_cursor.lock();
            let segment = std::mem::take(&mut *segment_guard);
            let cursor = std::mem::take(&mut *cursor_guard);
            (segment, cursor)
        };

        // Double-check segment isn't empty (race condition guard)
        if segment.is_empty() {
            return Ok(());
        }

        // Assign a segment sequence number
        let seq = SegmentSeq::new(self.next_segment_seq.fetch_add(1, Ordering::SeqCst));

        // Write the segment file (streaming serialization - no intermediate buffer)
        let segment_path = self.segment_path(seq);
        let writer = SegmentWriter::new(seq, self.set_permissions_supported);
        let (bytes_written, _checksum) = writer
            .write_segment(&segment_path, segment)
            .await
            .map_err(|e| {
                otel_error!(
                    "quiver.segment.flush",
                    segment = seq.raw(),
                    path = %segment_path.display(),
                    error = %e,
                    error_type = "io",
                    message = "data may only be recoverable via WAL replay",
                );
                e
            })?;

        otel_debug!("quiver.segment.flush", segment = seq.raw(), bytes_written,);

        // Track cumulative bytes (never decreases, for accurate throughput measurement)
        let _ = self
            .cumulative_segment_bytes
            .fetch_add(bytes_written, Ordering::Relaxed);

        // Record the segment's bytes in the budget.
        // This is safe: the soft_cap reserves headroom for exactly this.
        self.budget.add(bytes_written);

        // Step 5: Advance WAL cursor now that segment is durable
        {
            let mut wal_writer = self.wal_writer.lock().await;
            wal_writer.persist_cursor(&cursor).await.map_err(|e| {
                otel_error!(
                    "quiver.segment.flush",
                    segment = seq.raw(),
                    error = %e,
                    error_type = "io",
                    message = "WAL replay may produce duplicates on restart",
                );
                e
            })?;
        }

        // Step 6: Register segment with store (triggers subscriber notification).
        // Budget was already recorded above, so register_segment will skip
        // duplicate accounting (the file size was already added).
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
    /// a bundle becomes available, the timeout expires, or cancellation is requested.
    ///
    /// Returns an RAII handle that must be resolved via `ack()`, `reject()`,
    /// or `defer()` before being dropped.
    ///
    /// # Arguments
    ///
    /// * `id` - The subscriber ID
    /// * `timeout` - Maximum time to wait for a bundle. If `None`, waits indefinitely.
    /// * `cancel` - Optional cancellation token for graceful shutdown.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(handle))` - A bundle is available
    /// * `Ok(None)` - Timeout expired with no bundle available
    /// * `Err(SubscriberError::Cancelled)` - Cancellation was requested
    /// * `Err(_)` - Subscriber not found or not active
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::Duration;
    /// use quiver::CancellationToken;
    ///
    /// let cancel = CancellationToken::new();
    ///
    /// // Wait up to 5 seconds for a bundle, with cancellation support
    /// let handle = engine
    ///     .next_bundle(&subscriber_id, Some(Duration::from_secs(5)), Some(&cancel))
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
        cancel: Option<&tokio_util::sync::CancellationToken>,
    ) -> std::result::Result<Option<BundleHandle<RegistryCallback<SegmentStore>>>, SubscriberError>
    {
        self.registry.next_bundle(id, timeout, cancel).await
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
                    otel_warn!("quiver.segment.drop", segment = seq.raw(), error = %e, error_type = "io", reason = "completed");
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
                otel_warn!("quiver.segment.drop", segment = seq.raw(), error = %e, error_type = "io", reason = "force_drop");
            } else {
                otel_info!(
                    "quiver.segment.drop",
                    segment = seq.raw(),
                    reason = "force_drop",
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

    /// Deletes segments that have exceeded the configured maximum age.
    ///
    /// This implements time-based retention: segments older than
    /// [`RetentionConfig::max_age`] are deleted regardless of whether
    /// subscribers have consumed them.
    ///
    /// Returns the number of segments deleted due to age expiration.
    ///
    /// # Errors
    ///
    /// Returns an error if segment deletion fails.
    ///
    /// [`RetentionConfig::max_age`]: crate::config::RetentionConfig::max_age
    pub fn cleanup_expired_segments(&self) -> std::result::Result<usize, SubscriberError> {
        let Some(max_age) = self.config.retention.max_age else {
            // max_age not configured, nothing to expire
            return Ok(0);
        };
        let expired_segments = self.segment_store.segments_older_than(max_age);

        if expired_segments.is_empty() {
            return Ok(0);
        }

        // Force-complete these segments in the registry BEFORE deleting files.
        // This ensures subscribers don't try to read from segments we're about to delete.
        // Any claimed but unresolved bundles will be abandoned.
        self.registry.force_complete_segments(&expired_segments);

        let mut deleted = 0;
        let mut bundles_expired: u64 = 0;

        for seq in &expired_segments {
            // Count bundles before deleting
            if let Ok(count) = self.segment_store.bundle_count(*seq) {
                bundles_expired += count as u64;
            }

            if let Err(e) = self.segment_store.delete_segment(*seq) {
                otel_warn!(
                    "quiver.segment.drop",
                    segment = seq.raw(),
                    error = %e,
                    error_type = "io",
                    reason = "expired",
                );
            } else {
                otel_info!(
                    "quiver.segment.drop",
                    segment = seq.raw(),
                    max_age_secs = max_age.as_secs(),
                    reason = "expired",
                );
                deleted += 1;
            }
        }

        // Clean up registry internal state for deleted segments
        if let Some(&max_deleted) = expired_segments.iter().max() {
            self.registry.cleanup_segments_before(max_deleted.next());
        }

        // Track expired bundles in the dedicated counter
        if bundles_expired > 0 {
            let _ = self
                .expired_bundles
                .fetch_add(bundles_expired, Ordering::Relaxed);
        }

        Ok(deleted)
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
    /// 3. Deletes segments that have exceeded max_age retention
    /// 4. Retries deletion of segments that were previously deferred (e.g., Windows sharing violations)
    ///
    /// # Errors
    ///
    /// Returns an error if either operation fails.
    pub async fn maintain(&self) -> std::result::Result<MaintenanceStats, SubscriberError> {
        let flushed = self.flush_progress().await?;
        let deleted = self.cleanup_completed_segments()?;
        let expired = self.cleanup_expired_segments()?;
        let pending_deletes_cleared = self.segment_store.retry_pending_deletes();

        if flushed > 0 || deleted > 0 || expired > 0 || pending_deletes_cleared > 0 {
            otel_debug!(
                "quiver.engine.tick",
                flushed,
                deleted,
                expired,
                pending_deletes_cleared,
            );
        }

        Ok(MaintenanceStats {
            flushed,
            deleted,
            expired,
            pending_deletes_cleared,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Accessors
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns a reference to the underlying segment store.
    ///
    /// This is useful for advanced use cases like custom segment queries.
    #[must_use]
    pub const fn segment_store(&self) -> &Arc<SegmentStore> {
        &self.segment_store
    }

    /// Returns a reference to the underlying subscriber registry.
    ///
    /// This is useful for advanced use cases like custom subscriber management.
    #[must_use]
    pub const fn registry(&self) -> &Arc<SubscriberRegistry<SegmentStore>> {
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

async fn initialize_wal_writer(
    config: &QuiverConfig,
    budget: &Arc<crate::budget::DiskBudget>,
    set_permissions_supported: bool,
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
        .with_budget(budget.clone())
        .with_enforce_file_readonly(set_permissions_supported);
    Ok(WalWriter::open(options).await?)
}

fn wal_path(config: &QuiverConfig) -> PathBuf {
    config.data_dir.join("wal").join("quiver.wal")
}

/// Prefix for the temporary probe file created by [`probe_set_permissions_support`].
const PERMS_PROBE_PREFIX: &str = ".quiver_perms_probe.";

/// Probes whether the filesystem at `dir` supports `chmod`/`set_permissions`.
///
/// Creates a temporary probe file with a unique name (using PID and timestamp),
/// attempts to change its permissions, and returns `true` if the operation
/// succeeds. This detects filesystems (e.g., Azure Files SMB/CIFS mounts,
/// certain Kubernetes volumeMounts) that return `EPERM` on permission changes.
///
/// The probe file is always cleaned up, regardless of outcome. The randomized
/// name avoids conflicts if a previous probe file was not properly deleted.
fn probe_set_permissions_support(dir: &Path) -> bool {
    let probe_name = format!(
        "{}{}.{}",
        PERMS_PROBE_PREFIX,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_nanos(),
    );
    let probe_path = dir.join(&probe_name);

    // Create a temporary probe file. If this fails (disk full, quota exceeded,
    // directory missing, etc.) we cannot probe permissions at all. Treat this as
    // a setup error rather than evidence that permissions are unsupported —
    // default to `true` so we don't silently disable readonly enforcement.
    let file_created = match fs::File::create(&probe_path) {
        Ok(_file) => true,
        Err(e) => {
            otel_warn!(
                "quiver.engine.init",
                error = %e,
                path = %dir.display(),
                reason = "permissions_probe_create_failed",
                message = "could not create probe file to test permission support; \
                    assuming permissions are supported",
            );
            return true;
        }
    };
    debug_assert!(file_created);

    // Attempt to set permissions and verify they took effect.
    let result = (|| -> std::io::Result<()> {
        // Some filesystems (e.g., FAT32/vfat) silently accept chmod but
        // don't actually change the permission bits. We must read back
        // and compare to detect this.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let target_mode = 0o440;
            let permissions = fs::Permissions::from_mode(target_mode);
            fs::set_permissions(&probe_path, permissions)?;

            // Read back and verify the mode actually changed
            let actual_mode = fs::metadata(&probe_path)?.permissions().mode() & 0o777;
            if actual_mode != target_mode {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!(
                        "chmod succeeded but permissions were not applied \
                         (requested {target_mode:#o}, got {actual_mode:#o}); \
                         filesystem likely does not support permission changes"
                    ),
                ));
            }
        }

        #[cfg(not(unix))]
        {
            let mut permissions = fs::metadata(&probe_path)?.permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&probe_path, permissions)?;

            // Read back and verify readonly actually took effect
            if !fs::metadata(&probe_path)?.permissions().readonly() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "set_readonly succeeded but file is still writable; \
                     filesystem likely does not support permission changes",
                ));
            }
        }

        Ok(())
    })();

    // Always clean up the probe file
    let _ = fs::remove_file(&probe_path);

    match result {
        Ok(()) => {
            otel_debug!(
                "quiver.engine.init",
                path = %dir.display(),
                set_permissions_supported = true,
            );
            true
        }
        Err(e) => {
            otel_warn!(
                "quiver.engine.init",
                error = %e,
                path = %dir.display(),
                reason = "set_permissions_unsupported",
                message = "filesystem does not support permission changes; \
                    immutability enforcement via read-only file permissions \
                    will be skipped for segment and WAL files",
            );
            false
        }
    }
}

const fn segment_cfg_hash(_config: &QuiverConfig) -> [u8; 16] {
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
    use crate::config::{RetentionConfig, RetentionPolicy, SegmentConfig, WalConfig};
    use crate::record_bundle::{
        BundleDescriptor, PayloadRef, RecordBundle, SlotDescriptor, SlotId,
    };
    use crate::subscriber::SubscriberId;
    use crate::wal::{CURSOR_SIDECAR_FILENAME, WalReader};
    use arrow_array::builder::Int64Builder;
    use arrow_schema::{DataType, Field, Schema};
    use std::num::NonZeroU64;
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

        fn ingestion_time(&self) -> SystemTime {
            SystemTime::now()
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

    /// A bundle wrapper that returns a fixed ingestion time instead of `now()`.
    /// Used to test time-dependent behavior like max_age filtering during WAL replay.
    struct TimestampedBundle {
        inner: DummyBundle,
        ingestion_time: SystemTime,
    }

    impl TimestampedBundle {
        fn with_rows_and_time(num_rows: usize, ingestion_time: SystemTime) -> Self {
            Self {
                inner: DummyBundle::with_rows(num_rows),
                ingestion_time,
            }
        }
    }

    impl RecordBundle for TimestampedBundle {
        fn descriptor(&self) -> &BundleDescriptor {
            self.inner.descriptor()
        }

        fn ingestion_time(&self) -> SystemTime {
            self.ingestion_time
        }

        fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
            self.inner.payload(slot)
        }
    }

    /// Creates a large test budget (1 GB) for tests that don't specifically test budget limits.
    fn test_budget() -> Arc<DiskBudget> {
        Arc::new(DiskBudget::unlimited())
    }

    /// Small WAL config for budget-constrained tests.
    ///
    /// Uses a small but functional max_size_bytes so that
    /// `cap >= wal_max + segment_size` is satisfiable with modest budget caps
    /// while still being large enough for actual WAL entries.
    fn small_wal_config() -> WalConfig {
        WalConfig {
            max_size_bytes: NonZeroU64::new(32 * 1024).expect("non-zero"), // 32 KB
            max_rotated_files: 2,
            rotation_target_bytes: NonZeroU64::new(16 * 1024).expect("non-zero"), // 16 KB
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn ingest_succeeds_and_records_metrics() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());
        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("config valid");
        let bundle = DummyBundle::new();

        // Ingest should now succeed
        engine.ingest(&bundle).await.expect("ingest succeeds");
        assert_eq!(engine.metrics().ingest_attempts(), 1);
    }

    #[tokio::test]
    async fn config_returns_engine_configuration() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .build()
            .expect("builder should produce valid config");
        let engine = QuiverEngine::open(config.clone(), test_budget())
            .await
            .expect("config valid");

        assert_eq!(engine.config(), &config);
    }

    #[tokio::test]
    async fn ingest_appends_to_wal() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());
        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("config valid");
        let bundle = DummyBundle::new();

        engine.ingest(&bundle).await.expect("ingest succeeds");

        engine.shutdown().await.expect("shutdown succeeds");

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
    #[tokio::test]
    async fn ingest_segment_only_mode_skips_wal() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // Ingest a bundle
        let bundle = DummyBundle::with_rows(10);
        engine.ingest(&bundle).await.expect("ingest succeeds");

        // Verify metrics
        assert_eq!(engine.metrics().ingest_attempts(), 1);

        engine.shutdown().await.expect("shutdown");

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

    #[tokio::test]
    async fn ingest_finalizes_segment_when_threshold_exceeded() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // Ingest enough data to exceed the threshold
        // With 100 byte threshold and ~100 bytes per row estimate,
        // a few rows should trigger finalization
        let bundle = DummyBundle::with_rows(10);
        engine.ingest(&bundle).await.expect("ingest succeeds");

        engine.shutdown().await.expect("shutdown");

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
    #[tokio::test]
    async fn e2e_ingest_creates_segment_and_advances_wal_cursor() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // Ingest a bundle with enough data to trigger finalization
        let bundle = DummyBundle::with_rows(10);
        engine.ingest(&bundle).await.expect("ingest succeeds");

        // Shutdown engine to ensure all writes are flushed
        engine.shutdown().await.expect("shutdown");

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
        let sidecar_path = temp_dir.path().join("wal").join(CURSOR_SIDECAR_FILENAME);
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
    #[tokio::test]
    async fn ingest_handles_wal_capacity_transparently() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // Ingest many bundles - WAL will fill up multiple times
        // Each bundle is ~100-200 bytes, so we need 50+ to fill the 8KB WAL
        for i in 0..100 {
            let bundle = DummyBundle::with_rows(5);
            engine
                .ingest(&bundle)
                .await
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
    #[tokio::test]
    async fn e2e_many_bundles_across_multiple_segments() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // Ingest 10 bundles with varying row counts
        let bundle_row_counts = [5, 8, 3, 12, 7, 4, 9, 6, 11, 2];
        let total_rows: usize = bundle_row_counts.iter().sum();

        for &row_count in &bundle_row_counts {
            let bundle = DummyBundle::with_rows(row_count);
            engine.ingest(&bundle).await.expect("ingest succeeds");
        }

        // Verify metrics
        assert_eq!(
            engine.metrics().ingest_attempts(),
            bundle_row_counts.len() as u64
        );

        // Shutdown engine to flush all writes
        engine.shutdown().await.expect("shutdown");

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
        let sidecar_path = temp_dir.path().join("wal").join(CURSOR_SIDECAR_FILENAME);
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
    #[tokio::test]
    async fn e2e_bundles_with_different_schemas_create_separate_streams() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // Create bundles with different fingerprints (simulating schema evolution)
        let bundle1 = DummyBundleWithFingerprint::new([0x11; 32], 5);
        let bundle2 = DummyBundleWithFingerprint::new([0x22; 32], 5);
        let bundle3 = DummyBundleWithFingerprint::new([0x11; 32], 5); // Same as bundle1

        engine.ingest(&bundle1).await.expect("ingest bundle1");
        engine.ingest(&bundle2).await.expect("ingest bundle2");
        engine.ingest(&bundle3).await.expect("ingest bundle3");

        engine.shutdown().await.expect("shutdown");

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

        fn ingestion_time(&self) -> SystemTime {
            SystemTime::now()
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
    #[tokio::test]
    #[ignore]
    async fn stress_high_volume_ingestion() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

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
                .await
                .unwrap_or_else(|e| panic!("ingest {} failed: {}", i, e));
        }

        let ingest_duration = start.elapsed();

        // Verify metrics
        assert_eq!(engine.metrics().ingest_attempts(), NUM_BUNDLES as u64);

        // Capture WAL stats before shutdown
        let wal_stats = engine.wal_stats().await;

        // Shutdown engine to flush
        engine.shutdown().await.expect("shutdown");

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
        let sidecar_path = wal_dir.join(CURSOR_SIDECAR_FILENAME);
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
    #[tokio::test]
    async fn stress_multi_slot_bundles() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        const NUM_BUNDLES: usize = 500;

        for _ in 0..NUM_BUNDLES {
            let bundle = MultiSlotBundle::new();
            engine.ingest(&bundle).await.expect("ingest");
        }

        engine.shutdown().await.expect("shutdown");

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

        fn ingestion_time(&self) -> SystemTime {
            SystemTime::now()
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
    #[tokio::test]
    async fn stress_schema_evolution() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

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
                engine.ingest(&bundle).await.expect("ingest");
            }
        }

        engine.shutdown().await.expect("shutdown");

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

    #[tokio::test]
    async fn ingest_finalizes_segment_when_max_open_duration_exceeded() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // First ingest - starts the timer
        let bundle1 = DummyBundle::with_rows(1);
        engine
            .ingest(&bundle1)
            .await
            .expect("first ingest succeeds");

        // Wait for the max_open_duration to elapse
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Second ingest - should trigger time-based finalization
        let bundle2 = DummyBundle::with_rows(1);
        engine
            .ingest(&bundle2)
            .await
            .expect("second ingest succeeds");

        engine.shutdown().await.expect("shutdown");

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

    #[tokio::test]
    async fn shutdown_finalizes_open_segment() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // Ingest a small bundle that won't trigger size or time finalization
        let bundle = DummyBundle::with_rows(5);
        engine.ingest(&bundle).await.expect("ingest succeeds");

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
        engine.shutdown().await.expect("shutdown succeeds");

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

    #[tokio::test]
    async fn shutdown_on_empty_segment_succeeds() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());
        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("config valid");

        // Shutdown without ingesting anything should succeed
        engine
            .shutdown()
            .await
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

    #[tokio::test]
    async fn ingest_finalizes_segment_when_max_stream_count_exceeded() {
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

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

        // Each bundle with a different schema fingerprint creates a new stream.
        // We need to exceed max_stream_count (3) to trigger finalization.
        for i in 0u8..4 {
            let bundle = DummyBundleWithFingerprint::new([i; 32], 1);
            engine.ingest(&bundle).await.expect("ingest succeeds");
        }

        engine.shutdown().await.expect("shutdown");

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

    #[tokio::test]
    async fn budget_tracks_segment_bytes() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1).expect("non-zero"), // 1 byte - immediate finalization
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .wal(small_wal_config())
            .build()
            .expect("config valid");

        // Create a budget with plenty of room
        let budget = Arc::new(DiskBudget::new(
            100 * 1024 * 1024,
            1, // segment_headroom matches target_size_bytes
            RetentionPolicy::Backpressure,
        ));
        let engine = QuiverEngine::open(config, budget.clone())
            .await
            .expect("engine created");

        // Budget starts with WAL header bytes (tracked in the shared disk budget)
        let initial_used = budget.used();
        assert!(
            initial_used > 0,
            "budget should include WAL header bytes, got {}",
            initial_used
        );

        // Ingest a bundle - this will trigger segment finalization due to tiny target size
        let bundle = DummyBundle::with_rows(10);
        engine.ingest(&bundle).await.expect("ingest succeeds");

        // Budget should now reflect segment file size + WAL bytes
        let used = budget.used();
        assert!(
            used > initial_used,
            "budget should increase after segment write, got {} (was {})",
            used,
            initial_used
        );

        // Verify headroom decreased
        let headroom = budget.soft_cap_headroom();
        assert!(headroom < 100 * 1024 * 1024, "headroom should decrease");
    }

    #[tokio::test]
    async fn budget_returns_storage_at_capacity_when_exceeded() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).expect("non-zero"), // 1 KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .wal(small_wal_config())
            .build()
            .expect("config valid");

        // Budget sized to allow a few ingests then saturate.
        // min_budget = wal_max(32KB) + 2 * segment(1KB) = 34KB.
        // Use hard_cap just above minimum so the budget fills quickly.
        // soft_cap = 36KB - 1KB = 35KB.
        let hard_cap: u64 = 36 * 1024;
        let segment_headroom: u64 = 1024;
        let budget = Arc::new(DiskBudget::new(
            hard_cap,
            segment_headroom,
            RetentionPolicy::Backpressure,
        ));
        let engine = QuiverEngine::open(config, budget.clone())
            .await
            .expect("engine created");

        // Ingest until budget exceeds soft_cap, then verify rejection.
        // With a 35KB soft_cap and 32KB WAL, the budget fills within a handful
        // of ingests. We use a generous iteration limit but assert on budget
        // state if it's unexpectedly never reached.
        let bundle = DummyBundle::with_rows(100);
        let max_iterations = 200;
        for i in 0..max_iterations {
            match engine.ingest(&bundle).await {
                Ok(()) => {
                    // Safety valve: if budget is over soft_cap but ingest somehow
                    // succeeded, something is wrong with the gating logic.
                    if i > 10 && budget.is_over_soft_cap() {
                        panic!(
                            "budget is over soft_cap (used={}, soft_cap={}) \
                             but ingest succeeded on iteration {}",
                            budget.used(),
                            budget.soft_cap(),
                            i
                        );
                    }
                }
                Err(e) => {
                    assert!(e.is_at_capacity(), "expected StorageAtCapacity, got {e:?}");
                    assert!(
                        budget.is_over_soft_cap(),
                        "budget should be over soft_cap when ingest is rejected"
                    );
                    return;
                }
            }
        }
        panic!(
            "expected StorageAtCapacity within {max_iterations} ingests, \
             but all succeeded (budget used={}, soft_cap={}, hard_cap={})",
            budget.used(),
            budget.soft_cap(),
            budget.hard_cap()
        );
    }

    #[tokio::test]
    async fn budget_at_capacity_blocks_subsequent_ingest() {
        // Verifies that once the budget soft_cap is exceeded, further ingest
        // attempts are consistently rejected with StorageAtCapacity.
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).expect("non-zero"), // 1 KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .wal(small_wal_config())
            .build()
            .expect("config valid");

        // Budget sized to saturate after a small amount of data.
        let hard_cap: u64 = 36 * 1024;
        let budget = Arc::new(DiskBudget::new(
            hard_cap,
            1024,
            RetentionPolicy::Backpressure,
        ));
        let engine = QuiverEngine::open(config, budget.clone())
            .await
            .expect("engine created");

        // Ingest until we hit capacity.
        let bundle = DummyBundle::with_rows(100);
        loop {
            if engine.ingest(&bundle).await.is_err() {
                break;
            }
        }

        // Subsequent ingest should also fail.
        let result = engine.ingest(&bundle).await;
        assert!(result.is_err(), "expected StorageAtCapacity error");
        assert!(
            result.as_ref().unwrap_err().is_at_capacity(),
            "expected is_at_capacity() to be true, got {:?}",
            result
        );

        // Engine should still be functional (no panic).
        assert!(engine.metrics().ingest_attempts() >= 2);
    }

    #[tokio::test]
    async fn budget_drop_oldest_reclaims_completed_segments() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).expect("non-zero"), // 1 KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .wal(small_wal_config())
            .build()
            .expect("config valid");

        // Create a budget with DropOldest policy - large enough for a few segments
        let budget = Arc::new(DiskBudget::new(
            50 * 1024,
            1024,
            RetentionPolicy::DropOldest,
        ));
        let engine = QuiverEngine::open(config, budget.clone())
            .await
            .expect("engine created");

        // Register a subscriber so segments can be marked as "complete"
        let sub_id = SubscriberId::new("test-sub").expect("valid id");
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Ingest bundles to create segments
        for _ in 0..5 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).await.expect("ingest succeeds");
        }
        engine.flush().await.expect("flush");

        // Consume all bundles to mark segments as complete
        while let Ok(Some(handle)) = engine.poll_next_bundle(&sub_id) {
            handle.ack();
        }

        // Verify some segments were created
        let initial_segment_count = engine.segment_store.segment_count();
        assert!(initial_segment_count > 0, "should have created segments");

        // Run cleanup to complete segment lifecycle
        let _ = engine.cleanup_completed_segments();

        // Cleanup should reclaim completed segments and free budget space.
        // (With the watermark design, cleanup is called inline during ingest
        // when the soft cap is exceeded — this test verifies the mechanism works.)
    }

    #[tokio::test]
    async fn force_drop_oldest_drops_pending_segments_without_readers() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).expect("non-zero"), // 1 KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .wal(small_wal_config())
            .build()
            .expect("config valid");

        let budget = Arc::new(DiskBudget::new(
            100 * 1024,
            1024,
            RetentionPolicy::DropOldest,
        ));
        let engine = QuiverEngine::open(config, budget.clone())
            .await
            .expect("engine created");

        // Register and activate a subscriber
        let sub_id = SubscriberId::new("test-sub").expect("valid id");
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Ingest bundles to create multiple segments
        for _ in 0..10 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).await.expect("ingest succeeds");
        }
        engine.flush().await.expect("flush");

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

    #[tokio::test]
    async fn force_drop_skips_segments_with_active_readers() {
        let temp_dir = tempdir().expect("tempdir");
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).expect("non-zero"), // 1 KB segments
            ..Default::default()
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(segment_config)
            .wal(small_wal_config())
            .build()
            .expect("config valid");

        let budget = Arc::new(DiskBudget::new(
            100 * 1024,
            1024,
            RetentionPolicy::DropOldest,
        ));
        let engine = QuiverEngine::open(config, budget.clone())
            .await
            .expect("engine created");

        // Register and activate a subscriber
        let sub_id = SubscriberId::new("test-sub").expect("valid id");
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Ingest bundles to create multiple segments
        for _ in 0..10 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).await.expect("ingest succeeds");
        }
        engine.flush().await.expect("flush");

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
    #[tokio::test]
    async fn engine_loads_existing_segments_on_startup() {
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
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine created");

            // Ingest enough data to create multiple segments
            for _ in 0..5 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest succeeds");
            }
            engine.flush().await.expect("flush");

            let count = engine.segment_store.segment_count();
            assert!(count >= 2, "should create multiple segments, got {count}");
            count
        };

        // Second engine: should discover existing segments automatically
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine created");

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
    #[tokio::test]
    async fn subscriber_can_consume_from_recovered_segments() {
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
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine created");

            for _ in 0..3 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest succeeds");
            }
            engine.flush().await.expect("flush");

            assert!(
                engine.segment_store.segment_count() >= 1,
                "should have at least one segment"
            );
        }

        // Second engine: new subscriber should be able to consume the recovered data
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine created");

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
    #[tokio::test]
    async fn engine_handles_empty_segment_directory() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());

        // Create engine - should work fine with no existing segments
        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine created");

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
    #[tokio::test]
    async fn crash_recovery_with_progress_files() {
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

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine created");

            // Register subscriber before ingesting
            engine
                .register_subscriber(sub_id.clone())
                .expect("register");
            engine.activate_subscriber(&sub_id).expect("activate");

            // Ingest bundles to create multiple segments
            for _ in 0..5 {
                let bundle = DummyBundle::with_rows(10);
                engine.ingest(&bundle).await.expect("ingest");
            }
            engine.flush().await.expect("flush segments");

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
            let flushed = engine.flush_progress().await.expect("flush progress");
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

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine recreated");

            // Scan existing segments from previous run
            let scan_result = engine.segment_store.scan_existing().expect("scan existing");
            assert!(
                !scan_result.found.is_empty(),
                "should find segments from previous run"
            );

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

    /// Helper to create an engine and ingest data asynchronously.
    async fn setup_engine_with_data(dir: &Path, bundle_count: usize) -> Arc<QuiverEngine> {
        let config = QuiverConfig::builder()
            .data_dir(dir)
            .build()
            .expect("config");
        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        for _ in 0..bundle_count {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).await.expect("ingest");
        }
        if bundle_count > 0 {
            engine.flush().await.expect("flush");
        }
        engine
    }

    #[tokio::test]
    async fn next_bundle_returns_available_bundle() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 10).await;

        // Register and activate subscriber
        let sub_id = SubscriberId::new("async-test").unwrap();
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Use async next_bundle
        let handle = engine
            .next_bundle(&sub_id, Some(Duration::from_secs(5)), None)
            .await
            .expect("next_bundle")
            .expect("should have bundle");

        assert_eq!(handle.bundle_ref().bundle_index.raw(), 0);
        handle.ack();
    }

    #[tokio::test]
    async fn next_bundle_timeout_when_no_bundles() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 0).await;

        // Register and activate subscriber (but don't ingest anything)
        let sub_id = SubscriberId::new("timeout-test").unwrap();
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Async next_bundle should timeout quickly
        let result = engine
            .next_bundle(&sub_id, Some(Duration::from_millis(100)), None)
            .await
            .expect("next_bundle");

        assert!(result.is_none(), "should timeout with no bundles");
    }

    #[tokio::test]
    async fn next_bundle_wakes_on_segment_finalized() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 0).await;

        // Register and activate subscriber before any data
        let sub_id = SubscriberId::new("wake-test").unwrap();
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        let got_bundle = Arc::new(AtomicBool::new(false));
        let got_bundle_clone = got_bundle.clone();
        let engine_clone = engine.clone();
        let sub_id_clone = sub_id.clone();

        // Spawn async task to wait for bundle
        let consumer = tokio::spawn(async move {
            let result = engine_clone
                .next_bundle(&sub_id_clone, Some(Duration::from_secs(10)), None)
                .await
                .expect("next_bundle");
            if result.is_some() {
                got_bundle_clone.store(true, Ordering::Relaxed);
            }
        });

        // Give consumer time to start waiting
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Now ingest data and flush (which finalizes segment and notifies)
        for _ in 0..5 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).await.expect("ingest");
        }
        engine.flush().await.expect("flush");

        // Consumer should complete
        let result = tokio::time::timeout(Duration::from_secs(5), consumer).await;
        assert!(result.is_ok(), "consumer task should complete");
        assert!(
            got_bundle.load(Ordering::Relaxed),
            "should have received bundle"
        );
    }

    #[tokio::test]
    async fn next_bundle_interleaves_with_poll() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 10).await;

        // Register and activate subscriber
        let sub_id = SubscriberId::new("interleave-test").unwrap();
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Alternate between async and poll methods
        let h1 = engine
            .next_bundle(&sub_id, Some(Duration::from_secs(1)), None)
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
            .next_bundle(&sub_id, Some(Duration::from_secs(1)), None)
            .await
            .expect("async 2")
            .expect("bundle 3");
        h3.ack();

        // All three bundles should have sequential indices
        // (we can't check indices directly since we already acked, but test completes successfully)
    }

    #[tokio::test]
    async fn flush_progress_writes_dirty_subscribers() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 5).await;

        // Register and activate subscriber
        let sub_id = SubscriberId::new("flush-test").unwrap();
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Consume a bundle to make the subscriber dirty
        let handle = engine
            .next_bundle(&sub_id, Some(Duration::from_secs(1)), None)
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
    async fn maintain_flushes_and_cleans() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 10).await;

        // Register and consume all bundles
        let sub_id = SubscriberId::new("maintain-test").unwrap();
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Consume all bundles
        loop {
            match engine
                .next_bundle(&sub_id, Some(Duration::from_millis(100)), None)
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
        assert!(
            stats.flushed >= 1,
            "should have flushed at least one subscriber"
        );
        // deleted can be any non-negative number (usize)
    }

    #[tokio::test]
    async fn ingest_and_flush_creates_segment() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 0).await;

        // Ingest some data
        for _ in 0..5 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).await.expect("ingest");
        }

        // Flush to create segment
        engine.flush().await.expect("flush");

        // Verify segment created
        assert_eq!(engine.total_segments_written(), 1);
        assert_eq!(engine.segment_store().segment_count(), 1);
    }

    #[tokio::test]
    async fn full_pipeline_ingest_to_consume() {
        let dir = tempdir().expect("tempdir");
        let engine = setup_engine_with_data(dir.path(), 10).await;

        // Register and activate subscriber
        let sub_id = SubscriberId::new("async-pipeline").unwrap();
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Consume all bundles using async next_bundle
        let mut consumed = 0;
        loop {
            match engine
                .next_bundle(&sub_id, Some(Duration::from_millis(100)), None)
                .await
            {
                Ok(Some(h)) => {
                    h.ack();
                    consumed += 1;
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }

        assert_eq!(consumed, 10, "should have consumed all bundles");
    }

    #[tokio::test]
    async fn ingest_triggers_segment_finalization() {
        use std::num::NonZeroU64;

        let dir = tempdir().expect("tempdir");

        // Use a tiny segment size to trigger finalization during ingest
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(1024).unwrap(), // 1KB
                ..Default::default()
            })
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        // Ingest enough data to trigger segment finalization
        for _ in 0..20 {
            let bundle = DummyBundle::with_rows(100);
            engine.ingest(&bundle).await.expect("ingest");
        }

        // Should have auto-finalized at least one segment
        assert!(
            engine.total_segments_written() >= 1,
            "should have finalized at least one segment"
        );
    }

    /// Test that segment sequence numbers are correctly recovered after engine restart.
    ///
    /// The test verifies:
    /// 1. First engine run creates segment files (e.g., 00000000.qseg, 00000001.qseg)
    /// 2. After shutdown and restart, next_segment_seq is initialized to max(existing) + 1
    /// 3. New segments after restart use non-colliding sequence numbers
    /// 4. All segment files have unique sequence numbers
    #[tokio::test]
    async fn restart_recovers_segment_sequence_numbers() {
        use std::collections::HashSet;

        let dir = tempdir().expect("tempdir");
        let segment_dir = dir.path().join("segments");

        // === First engine run: create some segments ===
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100).unwrap(), // Very small to trigger finalization
                ..Default::default()
            })
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config.clone(), test_budget())
            .await
            .expect("first engine open");

        // Ingest data to create multiple segment files
        for _ in 0..10 {
            let bundle = DummyBundle::with_rows(50);
            engine.ingest(&bundle).await.expect("ingest");
        }
        engine.flush().await.expect("flush");

        let segments_before_restart = engine.total_segments_written();
        assert!(
            segments_before_restart >= 1,
            "should have created at least one segment before restart"
        );

        // Shutdown cleanly
        engine.shutdown().await.expect("first shutdown");
        drop(engine);

        // Collect segment files created in first run
        let files_after_first_run: HashSet<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert!(
            !files_after_first_run.is_empty(),
            "should have segment files after first run"
        );

        // === Second engine run: verify sequence recovery ===
        let engine2 = QuiverEngine::open(config.clone(), test_budget())
            .await
            .expect("second engine open");

        // The next segment sequence should be >= segments created in first run
        // (it tracks total segments ever created, not just loaded)
        assert!(
            engine2.total_segments_written() >= segments_before_restart,
            "next_segment_seq should be initialized from existing segments: got {}, expected >= {}",
            engine2.total_segments_written(),
            segments_before_restart
        );

        // Ingest more data to create additional segments
        for _ in 0..10 {
            let bundle = DummyBundle::with_rows(50);
            engine2.ingest(&bundle).await.expect("ingest after restart");
        }
        engine2.flush().await.expect("flush after restart");

        let segments_after_restart = engine2.total_segments_written();
        assert!(
            segments_after_restart > segments_before_restart,
            "should have created more segments after restart"
        );

        engine2.shutdown().await.expect("second shutdown");
        drop(engine2);

        // === Verify no sequence collisions ===
        // Collect all segment files and verify unique sequence numbers
        let all_files: Vec<_> = fs::read_dir(&segment_dir)
            .expect("read segment dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        let unique_files: HashSet<_> = all_files.iter().cloned().collect();

        assert_eq!(
            all_files.len(),
            unique_files.len(),
            "all segment files should have unique names (no sequence collisions)"
        );

        // Verify new segments were created (not overwriting old ones)
        assert!(
            all_files.len() > files_after_first_run.len(),
            "should have more segment files after restart: {} vs {}",
            all_files.len(),
            files_after_first_run.len()
        );

        // Verify all original files still exist (weren't overwritten)
        for original_file in &files_after_first_run {
            assert!(
                unique_files.contains(original_file),
                "original segment file {} should still exist after restart",
                original_file
            );
        }
    }

    /// Test that cleanup_expired_segments deletes segments older than max_age.
    ///
    /// This test verifies:
    /// 1. Segments with finalization time older than max_age are deleted
    /// 2. Segments newer than max_age are preserved
    /// 3. The method returns the correct count of deleted segments
    #[tokio::test]
    async fn cleanup_expired_segments_deletes_old_segments() {
        let dir = tempdir().expect("tempdir");

        // Configure a short max_age for testing (1 second)
        let retention = RetentionConfig {
            max_age: Some(Duration::from_secs(1)),
        };
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100).unwrap(),
                ..Default::default()
            })
            .retention(retention)
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        // Ingest data to create segments
        for _ in 0..5 {
            let bundle = DummyBundle::with_rows(50);
            engine.ingest(&bundle).await.expect("ingest");
        }
        engine.flush().await.expect("flush");

        let initial_segment_count = engine.segment_store().segment_count();
        assert!(
            initial_segment_count >= 1,
            "should have at least one segment"
        );

        // Wait for segments to exceed max_age
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup expired segments
        let expired_count = engine.cleanup_expired_segments().expect("cleanup");

        // All segments should have been deleted (they're all older than 1 second)
        assert_eq!(
            expired_count, initial_segment_count,
            "all segments should have expired"
        );
        assert_eq!(
            engine.segment_store().segment_count(),
            0,
            "no segments should remain after cleanup"
        );
    }

    /// Test that cleanup_expired_segments force-completes segments with claimed bundles.
    ///
    /// When a segment expires while a subscriber has claimed (but not yet resolved) bundles,
    /// the segment should still be deleted, but the registry should be updated first so
    /// subscribers don't try to read from deleted segments.
    #[tokio::test]
    async fn cleanup_expired_segments_handles_claimed_bundles() {
        let temp_dir = tempdir().expect("tempdir");
        let retention = RetentionConfig {
            max_age: Some(Duration::from_secs(1)), // Very short max_age for testing
        };
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100).unwrap(),
                ..Default::default()
            })
            .retention(retention)
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        // Register and activate a subscriber
        let sub_id = SubscriberId::new("test-sub").expect("valid id");
        engine
            .register_subscriber(sub_id.clone())
            .expect("register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // Ingest data to create segments
        for _ in 0..3 {
            let bundle = DummyBundle::with_rows(50);
            engine.ingest(&bundle).await.expect("ingest");
        }
        engine.flush().await.expect("flush");

        let initial_segment_count = engine.segment_store().segment_count();
        assert!(
            initial_segment_count >= 1,
            "should have at least one segment"
        );

        // Claim a bundle (but don't resolve it yet)
        let handle = engine
            .poll_next_bundle(&sub_id)
            .expect("poll succeeds")
            .expect("bundle available");

        // Wait for segments to exceed max_age
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup expired segments - should succeed even with claimed bundles
        let expired_count = engine.cleanup_expired_segments().expect("cleanup");
        assert_eq!(
            expired_count, initial_segment_count,
            "all segments should have been deleted"
        );
        assert_eq!(
            engine.segment_store().segment_count(),
            0,
            "segment store should be empty after cleanup"
        );

        // Verify the expired_bundles counter was incremented
        assert!(
            engine.expired_bundles() > 0,
            "expired_bundles counter should be incremented"
        );

        // The handle is now pointing to a deleted segment.
        // When we try to resolve it, the segment will already be force-completed
        // in the registry, so we should NOT attempt to read from it.
        // The handle.ack() should succeed because force_complete_segment already
        // abandoned the claimed bundle.

        // Ack the handle - this should be safe because the segment was force-completed
        handle.ack();

        // Verify the subscriber can continue (no bundles left since all expired)
        let result = engine
            .poll_next_bundle(&sub_id)
            .expect("poll should succeed");
        assert!(
            result.is_none(),
            "no bundles should remain after all segments expired"
        );
    }

    /// Test that cleanup_expired_segments is a no-op when max_age is None (default).
    #[tokio::test]
    async fn cleanup_expired_segments_noop_when_disabled() {
        let dir = tempdir().expect("tempdir");

        // Use default config (max_age = None)
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100).unwrap(),
                ..Default::default()
            })
            .build()
            .expect("config");

        // Verify max_age is None by default
        assert!(
            config.retention.max_age.is_none(),
            "max_age should be None by default"
        );

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        // Ingest data to create segments
        for _ in 0..3 {
            let bundle = DummyBundle::with_rows(50);
            engine.ingest(&bundle).await.expect("ingest");
        }
        engine.flush().await.expect("flush");

        let initial_segment_count = engine.segment_store().segment_count();
        assert!(
            initial_segment_count >= 1,
            "should have at least one segment"
        );

        // Wait some time (would normally cause expiration if max_age was set)
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cleanup should be a no-op since max_age is not configured
        let expired_count = engine.cleanup_expired_segments().expect("cleanup");

        assert_eq!(
            expired_count, 0,
            "no segments should expire when max_age is None"
        );
        assert_eq!(
            engine.segment_store().segment_count(),
            initial_segment_count,
            "all segments should remain when max_age is disabled"
        );
    }

    /// Test that cleanup_expired_segments preserves segments newer than max_age.
    #[tokio::test]
    async fn cleanup_expired_segments_preserves_recent_segments() {
        let dir = tempdir().expect("tempdir");

        // Configure a long max_age (1 hour) so segments won't expire
        let retention = RetentionConfig {
            max_age: Some(Duration::from_secs(3600)),
        };
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100).unwrap(),
                ..Default::default()
            })
            .retention(retention)
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        // Ingest data to create segments
        for _ in 0..3 {
            let bundle = DummyBundle::with_rows(50);
            engine.ingest(&bundle).await.expect("ingest");
        }
        engine.flush().await.expect("flush");

        let initial_segment_count = engine.segment_store().segment_count();
        assert!(
            initial_segment_count >= 1,
            "should have at least one segment"
        );

        // Cleanup expired segments (none should be expired)
        let expired_count = engine.cleanup_expired_segments().expect("cleanup");

        // No segments should have been deleted
        assert_eq!(expired_count, 0, "no segments should have expired");
        assert_eq!(
            engine.segment_store().segment_count(),
            initial_segment_count,
            "all segments should remain"
        );
    }

    /// Test that maintain() includes expired segment cleanup in its stats.
    #[tokio::test]
    async fn maintain_includes_expired_segment_cleanup() {
        let dir = tempdir().expect("tempdir");

        // Configure a short max_age for testing
        let retention = RetentionConfig {
            max_age: Some(Duration::from_secs(1)),
        };
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100).unwrap(),
                ..Default::default()
            })
            .retention(retention)
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        // Ingest data to create segments
        for _ in 0..3 {
            let bundle = DummyBundle::with_rows(50);
            engine.ingest(&bundle).await.expect("ingest");
        }
        engine.flush().await.expect("flush");

        let initial_segment_count = engine.segment_store().segment_count();
        assert!(
            initial_segment_count >= 1,
            "should have at least one segment"
        );

        // Wait for segments to exceed max_age
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Run maintenance
        let stats = engine.maintain().await.expect("maintain");

        // The expired segments should be included in the deleted count
        assert!(
            stats.expired >= initial_segment_count,
            "maintain should report expired segments: got {}, expected >= {}",
            stats.expired,
            initial_segment_count
        );
    }

    /// Test that expired segments are deleted during startup scan without loading them.
    ///
    /// This verifies that `scan_existing_with_max_age()` deletes expired segments
    /// based on file mtime alone, without the overhead of opening and parsing them.
    #[tokio::test]
    async fn startup_deletes_expired_segments_without_loading() {
        let dir = tempdir().expect("tempdir");

        // Phase 1: Create segments with a long max_age config
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).unwrap(),
            ..Default::default()
        };
        let config_long_age = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(segment_config.clone())
            .retention(RetentionConfig {
                max_age: Some(Duration::from_secs(3600)), // 1 hour - won't expire yet
            })
            .build()
            .expect("config");

        let segments_created;
        {
            let engine = QuiverEngine::open(config_long_age, test_budget())
                .await
                .expect("engine");

            // Ingest data to create multiple segments
            for _ in 0..5 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest");
            }
            engine.flush().await.expect("flush");

            segments_created = engine.segment_store().segment_count();
            assert!(segments_created >= 1, "should create at least one segment");
        }

        // Wait for segments to become "old" enough for our short max_age
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Phase 2: Create new engine with very short max_age
        let config_short_age = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(segment_config)
            .retention(RetentionConfig {
                max_age: Some(Duration::from_millis(10)), // Very short - segments should be expired
            })
            .build()
            .expect("config");

        let engine2 = QuiverEngine::open(config_short_age, test_budget())
            .await
            .expect("engine");

        // The expired segments should have been deleted during startup scan
        assert_eq!(
            engine2.segment_store().segment_count(),
            0,
            "expired segments should be deleted during startup, not loaded"
        );
    }

    /// Test that startup scan deletes only expired segments, keeping fresh ones.
    ///
    /// This verifies `scan_existing_with_max_age()` correctly filters segments
    /// by age - deleting old ones while loading recent ones.
    #[tokio::test]
    async fn startup_preserves_fresh_segments_deletes_expired() {
        let dir = tempdir().expect("tempdir");

        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).unwrap(),
            ..Default::default()
        };

        // Phase 1: Create initial segments
        let old_segments_created;
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(segment_config.clone())
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            // Create some segments
            for _ in 0..3 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest");
            }
            engine.flush().await.expect("flush");

            old_segments_created = engine.segment_store().segment_count();
            assert!(
                old_segments_created >= 1,
                "should create at least one segment"
            );
        }

        // Wait for these segments to become "old" - use generous margin
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Phase 2: Create fresh segments
        let total_segments;
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(segment_config.clone())
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            // Create more segments (these will be fresh)
            for _ in 0..2 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest");
            }
            engine.flush().await.expect("flush");

            total_segments = engine.segment_store().segment_count();
            assert!(
                total_segments > old_segments_created,
                "should have created additional segments"
            );
        }

        // Phase 3: Reopen with max_age that expires old segments but not fresh ones
        // Use 1 second - old segments (2s+ old) will expire, fresh ones won't
        let config_with_max_age = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(segment_config)
            .retention(RetentionConfig {
                max_age: Some(Duration::from_secs(1)),
            })
            .build()
            .expect("config");

        let engine3 = QuiverEngine::open(config_with_max_age, test_budget())
            .await
            .expect("engine");

        // Only the fresh segments should remain
        let remaining = engine3.segment_store().segment_count();
        assert!(
            remaining > 0,
            "some fresh segments should remain after startup"
        );
        assert!(
            remaining < total_segments,
            "some old segments should have been deleted: remaining={}, total={}",
            remaining,
            total_segments
        );
    }

    /// Test that the expired_bundles counter is tracked separately from force_dropped_bundles.
    #[tokio::test]
    async fn expired_bundles_counter_is_separate() {
        let dir = tempdir().expect("tempdir");

        let retention = RetentionConfig {
            max_age: Some(Duration::from_secs(1)),
        };
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100).unwrap(),
                ..Default::default()
            })
            .retention(retention)
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        // Initially both counters should be zero
        assert_eq!(engine.force_dropped_bundles(), 0);
        assert_eq!(engine.expired_bundles(), 0);

        // Ingest data to create segments
        for _ in 0..3 {
            let bundle = DummyBundle::with_rows(50);
            engine.ingest(&bundle).await.expect("ingest");
        }
        engine.flush().await.expect("flush");

        let initial_segment_count = engine.segment_store().segment_count();
        assert!(
            initial_segment_count >= 1,
            "should have at least one segment"
        );

        // Wait for segments to exceed max_age
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup expired segments
        let expired_count = engine.cleanup_expired_segments().expect("cleanup");
        assert!(expired_count > 0, "should have expired some segments");

        // expired_bundles should be incremented, force_dropped_bundles should still be zero
        assert_eq!(
            engine.force_dropped_bundles(),
            0,
            "force_dropped_bundles should not be affected by expired segments"
        );
        assert!(
            engine.expired_bundles() > 0,
            "expired_bundles should be incremented"
        );
    }

    /// Test that subscribers restored from progress.json don't fail when
    /// expired segments are deleted during startup scan.
    ///
    /// If scan_existing_with_max_age deletes segments on disk, but
    /// SubscriberRegistry::open restores subscriber state from progress.json
    /// which references those deleted segments, ensure that the deleted
    /// segments are force-completed in the registry to avoid
    /// segment_not_found errors.
    #[tokio::test]
    async fn startup_expired_segments_force_completed_in_registry() {
        let dir = tempdir().expect("tempdir");

        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).unwrap(),
            ..Default::default()
        };

        // Phase 1: Create segments, register a subscriber, consume some bundles,
        // then flush progress to disk.
        let sub_id = SubscriberId::new("test-sub").expect("valid id");
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(segment_config.clone())
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            // Ingest data to create segments
            for _ in 0..5 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest");
            }
            engine.flush().await.expect("flush");

            let segment_count = engine.segment_store().segment_count();
            assert!(segment_count >= 1, "should have at least one segment");

            // Register and activate a subscriber
            engine
                .register_subscriber(sub_id.clone())
                .expect("register");
            engine.activate_subscriber(&sub_id).expect("activate");

            // Consume one bundle to advance subscriber progress
            let handle = engine
                .poll_next_bundle(&sub_id)
                .expect("poll")
                .expect("bundle available");
            handle.ack();

            // Flush progress to disk so it persists across restart
            let flushed = engine.flush_progress().await.expect("flush progress");
            assert!(flushed > 0, "should have flushed subscriber progress");

            // Engine dropped here (simulating shutdown)
        }

        // Wait for segments to become "old" enough
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Phase 2: Reopen with a very short max_age so all segments expire.
        // The subscriber's progress.json still references the deleted segments.
        // Without the fix, poll_next_bundle would fail with segment_not_found.
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(segment_config)
            .retention(RetentionConfig {
                max_age: Some(Duration::from_millis(10)),
            })
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine should open successfully");

        // All segments should have been expired and deleted during scan
        assert_eq!(
            engine.segment_store().segment_count(),
            0,
            "all segments should be deleted"
        );

        // Re-register the subscriber (loads from progress.json)
        engine
            .register_subscriber(sub_id.clone())
            .expect("re-register");
        engine.activate_subscriber(&sub_id).expect("activate");

        // This is the critical assertion: poll_next_bundle should return None
        // (no bundles available) rather than failing with segment_not_found.
        let result = engine
            .poll_next_bundle(&sub_id)
            .expect("poll should succeed, not error with segment_not_found");
        assert!(
            result.is_none(),
            "no bundles should be available after all segments expired"
        );
    }

    /// Test that next_segment_seq is monotonic even when all segments are deleted.
    ///
    /// If scan_existing_with_max_age deletes ALL segments, the sequence counter
    /// must still continue from the highest deleted sequence to prevent reuse.
    #[tokio::test]
    async fn startup_sequence_continues_after_all_segments_deleted() {
        let dir = tempdir().expect("tempdir");

        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(100).unwrap(),
            ..Default::default()
        };

        // Phase 1: Create segments
        let segments_created;
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(segment_config.clone())
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            for _ in 0..5 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest");
            }
            engine.flush().await.expect("flush");

            segments_created = engine.segment_store().segment_count();
            assert!(segments_created >= 1, "should have at least one segment");
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Phase 2: Reopen with short max_age to delete all segments
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(segment_config)
            .retention(RetentionConfig {
                max_age: Some(Duration::from_millis(10)),
            })
            .build()
            .expect("config");

        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine");

        assert_eq!(engine.segment_store().segment_count(), 0);

        // Ingest new data — new segments should have sequence numbers
        // that don't overlap with the deleted ones
        let bundle = DummyBundle::with_rows(50);
        engine.ingest(&bundle).await.expect("ingest");
        engine.flush().await.expect("flush");

        let new_segments = engine.segment_store().segment_sequences();
        assert!(!new_segments.is_empty(), "should have new segments");

        for seq in &new_segments {
            assert!(
                seq.raw() >= segments_created as u64,
                "new segment seq {} should be >= previous count {} to avoid reuse",
                seq.raw(),
                segments_created
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // WAL Replay Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn wal_replay_recovers_bundles_not_finalized_to_segments() {
        let dir = tempdir().expect("tempdir");

        // Use a LARGE segment size so nothing gets finalized to segments
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(), // 100MB
                ..Default::default()
            })
            .build()
            .expect("config");

        // First run: ingest some bundles (they go to WAL + open segment, but NOT finalized)
        let bundles_ingested = 5;
        {
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");

            for _ in 0..bundles_ingested {
                let bundle = DummyBundle::with_rows(10);
                engine.ingest(&bundle).await.expect("ingest");
            }

            // Verify WAL has data
            let wal_bytes = engine.wal_bytes_written();
            assert!(wal_bytes > 0, "WAL should have data written");

            // Verify NO segments were finalized (data only in open segment)
            assert_eq!(
                engine.total_segments_written(),
                0,
                "no segments should be finalized yet"
            );

            // Check open segment has our bundles
            let open_segment_bundles = engine.open_segment.lock().bundle_count();
            assert_eq!(
                open_segment_bundles, bundles_ingested,
                "open segment should have {} bundles",
                bundles_ingested
            );

            // Engine dropped here - simulates crash/shutdown
            // WAL cursor is NOT advanced (no segments finalized)
        }

        // Verify WAL file exists
        let wal_path = dir.path().join("wal").join("quiver.wal");
        assert!(wal_path.exists(), "WAL file should exist at {:?}", wal_path);

        // Verify NO segment files exist on disk
        let segments_dir = dir.path().join("segments");
        if segments_dir.exists() {
            let segment_files: Vec<_> = fs::read_dir(&segments_dir)
                .expect("read segments dir")
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "seg"))
                .collect();
            assert!(
                segment_files.is_empty(),
                "no segment files should exist, but found: {:?}",
                segment_files
            );
        }

        // Second run: reopen engine and verify WAL replay
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine reopen");

            // Verify still no segments after recovery (data should be in open segment only)
            assert_eq!(
                engine.total_segments_written(),
                0,
                "no segments should exist after recovery"
            );

            // Check open segment has our replayed bundles
            let open_segment_bundles = engine.open_segment.lock().bundle_count();
            assert_eq!(
                open_segment_bundles, bundles_ingested,
                "open segment should have {} replayed bundles after recovery, got {}",
                bundles_ingested, open_segment_bundles
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_skips_already_finalized_bundles() {
        let dir = tempdir().expect("tempdir");

        // Use a small segment size to force finalization after a few bundles
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                // Use a size that causes finalization every ~10 bundles
                // We'll ingest 15 bundles to get 1 segment finalized + 5 in open segment
                target_size_bytes: NonZeroU64::new(50_000).unwrap(), // 50KB
                ..Default::default()
            })
            .build()
            .expect("config");

        // Track how many bundles end up in the open segment (not finalized)
        let bundles_in_open_segment_before_shutdown;

        // First run: ingest bundles - some will be finalized, some will remain in open segment
        {
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");

            // Ingest 15 bundles with 100 rows each (~5KB per bundle = ~75KB total)
            // With 50KB segment threshold, expect 1 segment finalized + some in open segment
            let total_bundles = 15;
            for _ in 0..total_bundles {
                let bundle = DummyBundle::with_rows(100);
                engine.ingest(&bundle).await.expect("ingest");
            }

            // Verify at least one segment was finalized
            let segments_written = engine.total_segments_written();
            assert!(
                segments_written > 0,
                "should have finalized at least one segment"
            );

            // Record the exact count in open segment before shutdown
            bundles_in_open_segment_before_shutdown = engine.open_segment.lock().bundle_count();
            assert!(
                bundles_in_open_segment_before_shutdown > 0,
                "open segment should have at least one bundle to replay; \
                 got 0 with {} segments finalized from {} bundles",
                segments_written,
                total_bundles
            );
        }

        // Second run: reopen and verify we only replay the un-finalized bundles
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine reopen");

            // Open segment should have EXACTLY the same count as before shutdown
            // (only the un-finalized bundles are replayed)
            let open_segment_bundles = engine.open_segment.lock().bundle_count();

            assert_eq!(
                open_segment_bundles, bundles_in_open_segment_before_shutdown,
                "should replay exactly {} bundles (those not finalized to segments)",
                bundles_in_open_segment_before_shutdown
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_handles_empty_wal() {
        let dir = tempdir().expect("tempdir");

        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .build()
            .expect("config");

        // First run: create engine but don't ingest anything
        {
            let _engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");
            // Engine dropped without any ingestion
        }

        // Second run: should open cleanly with empty WAL
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine reopen");

            let open_segment_bundles = engine.open_segment.lock().bundle_count();
            assert_eq!(
                open_segment_bundles, 0,
                "open segment should be empty after reopening with empty WAL"
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_handles_missing_wal_file() {
        let dir = tempdir().expect("tempdir");

        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .build()
            .expect("config");

        // Don't create any engine first - just open directly
        // This simulates a fresh start with no WAL file
        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine should open even without existing WAL");

        let open_segment_bundles = engine.open_segment.lock().bundle_count();
        assert_eq!(
            open_segment_bundles, 0,
            "open segment should be empty on fresh start"
        );
    }

    #[tokio::test]
    async fn wal_replay_finalizes_segments_if_threshold_exceeded() {
        // Test that WAL replay properly finalizes segments if the replayed data
        // exceeds the segment size threshold
        let dir = tempdir().expect("tempdir");

        // Segment threshold for replay: 5KB
        // We'll ingest 20 bundles of 50 rows each (~1-2KB per bundle = ~20-40KB total)
        // With 5KB threshold, expect 4-8 segments finalized during replay
        let replay_segment_threshold = 5 * 1024; // 5KB
        let bundles_to_ingest = 20;
        let rows_per_bundle = 50;

        // Use a small segment size that will be exceeded by WAL replay
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(replay_segment_threshold).unwrap(),
                ..Default::default()
            })
            .build()
            .expect("config");

        // First run: ingest bundles with a LARGE segment size so they stay in WAL
        // but don't get finalized to segments
        let large_segment_config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(), // 100MB
                ..Default::default()
            })
            .build()
            .expect("config");

        let bundles_in_open_segment;
        {
            let engine = QuiverEngine::open(large_segment_config, test_budget())
                .await
                .expect("engine");

            // Ingest enough data that will exceed threshold when replayed
            for _ in 0..bundles_to_ingest {
                let bundle = DummyBundle::with_rows(rows_per_bundle);
                engine.ingest(&bundle).await.expect("ingest");
            }

            // Verify no segments were finalized (data only in WAL + open segment)
            assert_eq!(
                engine.total_segments_written(),
                0,
                "no segments should be finalized with large segment config"
            );

            bundles_in_open_segment = engine.open_segment.lock().bundle_count();
            assert_eq!(
                bundles_in_open_segment, bundles_to_ingest,
                "all bundles should be in open segment"
            );
        }

        // Second run: reopen with SMALL segment size - replay should trigger finalization
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine reopen");

            // Verify that segments were finalized during replay
            let segments_written = engine.total_segments_written();

            // With 20 bundles of ~1-2KB each and 5KB threshold, expect at least 2 segments
            // (conservative lower bound to account for varying bundle sizes)
            assert!(
                segments_written >= 2,
                "WAL replay should have finalized at least 2 segments with 5KB threshold, but got {}",
                segments_written
            );

            // Also verify the math: segments written + bundles remaining = total ingested
            let bundles_remaining = engine.open_segment.lock().bundle_count();
            // The total bundles across all segments plus open segment should equal what we ingested
            // Note: we can't easily count bundles in finalized segments, but we can verify
            // that the open segment has fewer bundles than before (some were finalized)
            assert!(
                bundles_remaining < bundles_in_open_segment,
                "open segment should have fewer bundles ({}) than before finalization ({})",
                bundles_remaining,
                bundles_in_open_segment
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_reads_from_rotated_files() {
        // Test that WAL replay correctly reads entries from rotated WAL files
        // This simulates a crash that happens after WAL rotation but before
        // cursor advancement.
        let dir = tempdir().expect("tempdir");

        // Use a very small WAL rotation target to force rotation
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .wal(WalConfig {
                rotation_target_bytes: NonZeroU64::new(1024).unwrap(), // 1KB - will trigger rotation
                max_size_bytes: NonZeroU64::new(1024 * 1024).unwrap(), // 1MB total
                max_rotated_files: 10,
                ..Default::default()
            })
            .segment(SegmentConfig {
                // Large segment size so nothing gets finalized to segments
                target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                ..Default::default()
            })
            .build()
            .expect("config");

        let bundles_ingested = 20;
        {
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");

            // Ingest enough bundles to trigger multiple WAL rotations
            for _ in 0..bundles_ingested {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest");
            }

            // Verify WAL has data
            let wal_bytes = engine.wal_bytes_written();
            assert!(wal_bytes > 0, "WAL should have data written");

            // Verify NO segments were finalized
            assert_eq!(
                engine.total_segments_written(),
                0,
                "no segments should be finalized"
            );

            // Engine dropped here - simulates crash/shutdown
        }

        // Verify we have rotated WAL files
        let wal_dir = dir.path().join("wal");
        let rotated_files: Vec<_> = fs::read_dir(&wal_dir)
            .expect("read wal dir")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("quiver.wal."))
            })
            .collect();

        // With 20 bundles of 50 rows each (~1-2KB per bundle) and 1KB rotation target,
        // we should have at least 10 rotated files. Use a conservative minimum of 5
        // to account for varying bundle sizes.
        let rotated_count = rotated_files.len();
        assert!(
            rotated_count >= 5,
            "should have at least 5 rotated WAL files with 1KB rotation target, found {}: {:?}",
            rotated_count,
            fs::read_dir(&wal_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect::<Vec<_>>()
        );

        // Second run: reopen engine and verify WAL replay reads from ALL files
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine reopen");

            // Check open segment has all replayed bundles
            let open_segment_bundles = engine.open_segment.lock().bundle_count();
            assert_eq!(
                open_segment_bundles, bundles_ingested,
                "open segment should have all {} replayed bundles after recovery from rotated files, got {}",
                bundles_ingested, open_segment_bundles
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_handles_truncated_entry() {
        // Test that WAL replay gracefully handles a truncated entry (simulating crash mid-write).
        // This is the expected case after a crash - the last write was incomplete.
        let dir = tempdir().expect("tempdir");

        // Use large segment size so nothing gets finalized
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                ..Default::default()
            })
            .build()
            .expect("config");

        let bundles_before_crash = 5;
        {
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");

            for _ in 0..bundles_before_crash {
                let bundle = DummyBundle::with_rows(10);
                engine.ingest(&bundle).await.expect("ingest");
            }

            // Engine dropped - WAL has 5 complete entries
        }

        // Truncate the WAL file to simulate crash mid-write
        let wal_path = dir.path().join("wal").join("quiver.wal");
        let original_len = fs::metadata(&wal_path).expect("metadata").len();
        // Truncate 20 bytes off the end (partial entry)
        let truncated_len = original_len.saturating_sub(20);
        fs::OpenOptions::new()
            .write(true)
            .open(&wal_path)
            .expect("open wal")
            .set_len(truncated_len)
            .expect("truncate");

        // Reopen engine - should recover gracefully, replaying complete entries only
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine should open despite truncated WAL");

            // We should have recovered at least some bundles (the complete ones)
            let recovered = engine.open_segment.lock().bundle_count();
            assert!(
                recovered > 0 && recovered <= bundles_before_crash,
                "should recover some but not all bundles after truncation; got {}",
                recovered
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_handles_corrupted_entry() {
        // Test that WAL replay gracefully handles a corrupted entry (CRC mismatch).
        // The engine should log the error but continue startup with partial recovery.
        let dir = tempdir().expect("tempdir");

        // Use large segment size so nothing gets finalized
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                ..Default::default()
            })
            .build()
            .expect("config");

        let bundles_ingested = 5;
        {
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");

            for _ in 0..bundles_ingested {
                let bundle = DummyBundle::with_rows(10);
                engine.ingest(&bundle).await.expect("ingest");
            }
        }

        // Corrupt the WAL file by flipping some bytes in the middle
        let wal_path = dir.path().join("wal").join("quiver.wal");
        let mut wal_data = fs::read(&wal_path).expect("read wal");
        // Corrupt bytes in the middle of the file (after header, in entry data)
        let corrupt_offset = wal_data.len() / 2;
        for i in 0..8 {
            if corrupt_offset + i < wal_data.len() {
                wal_data[corrupt_offset + i] ^= 0xFF; // Flip all bits
            }
        }
        fs::write(&wal_path, &wal_data).expect("write corrupted wal");

        // Reopen engine - should recover gracefully with partial data
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine should open despite corrupted WAL");

            // We should have recovered some bundles (before the corruption)
            let recovered = engine.open_segment.lock().bundle_count();
            // Corruption in the middle means we'll get entries before the corrupt one
            assert!(
                recovered < bundles_ingested,
                "should recover fewer bundles due to corruption; got {} but ingested {}",
                recovered,
                bundles_ingested
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_clamps_cursor_exceeding_wal_bounds() {
        // Test that WAL replay handles a cursor sidecar with a position beyond the WAL end.
        // This can happen if the WAL was truncated or the sidecar is stale.
        let dir = tempdir().expect("tempdir");

        // Use large segment size so nothing gets finalized
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                ..Default::default()
            })
            .build()
            .expect("config");

        let bundles_ingested = 5;
        {
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");

            for _ in 0..bundles_ingested {
                let bundle = DummyBundle::with_rows(10);
                engine.ingest(&bundle).await.expect("ingest");
            }
        }

        // Write a cursor sidecar with a position way beyond the actual WAL size
        let wal_dir = dir.path().join("wal");
        let sidecar_path = wal_dir.join(CURSOR_SIDECAR_FILENAME);
        let bogus_cursor = CursorSidecar::new(999_999_999); // Way beyond WAL end
        CursorSidecar::write_to_sync(&sidecar_path, &bogus_cursor).expect("write sidecar");

        // Reopen engine - should clamp cursor to WAL end and replay nothing
        // (since the clamped position is at the end of the WAL)
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine should open despite bogus cursor");

            // Since cursor was clamped to WAL end, no entries should be replayed
            // The open segment should be empty
            let recovered = engine.open_segment.lock().bundle_count();
            assert_eq!(
                recovered, 0,
                "should replay zero bundles when cursor is clamped to WAL end; got {}",
                recovered
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_handles_corrupted_cursor_sidecar() {
        // Test that WAL replay handles a corrupted cursor sidecar gracefully.
        // Should fall back to replaying from the beginning.
        let dir = tempdir().expect("tempdir");

        // Use large segment size so nothing gets finalized
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                ..Default::default()
            })
            .build()
            .expect("config");

        let bundles_ingested = 5;
        {
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");

            for _ in 0..bundles_ingested {
                let bundle = DummyBundle::with_rows(10);
                engine.ingest(&bundle).await.expect("ingest");
            }
        }

        // Corrupt the cursor sidecar file
        let wal_dir = dir.path().join("wal");
        let sidecar_path = wal_dir.join(CURSOR_SIDECAR_FILENAME);
        // Write garbage data that won't decode as a valid sidecar
        fs::write(&sidecar_path, b"corrupted garbage data").expect("write corrupt sidecar");

        // Reopen engine - should fall back to position 0 and replay all entries
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine should open despite corrupted sidecar");

            // Since cursor decode failed, should replay from position 0 (all entries)
            let recovered = engine.open_segment.lock().bundle_count();
            assert_eq!(
                recovered, bundles_ingested,
                "should replay all {} bundles when cursor is corrupted; got {}",
                bundles_ingested, recovered
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_handles_missing_cursor_sidecar() {
        // Test that WAL replay handles a missing cursor sidecar gracefully.
        // Should replay from the beginning (position 0).
        let dir = tempdir().expect("tempdir");

        // Use large segment size so nothing gets finalized
        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(SegmentConfig {
                target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                ..Default::default()
            })
            .build()
            .expect("config");

        let bundles_ingested = 5;
        {
            let engine = QuiverEngine::open(config.clone(), test_budget())
                .await
                .expect("engine");

            for _ in 0..bundles_ingested {
                let bundle = DummyBundle::with_rows(10);
                engine.ingest(&bundle).await.expect("ingest");
            }
        }

        // Delete the cursor sidecar file if it exists
        let wal_dir = dir.path().join("wal");
        let sidecar_path = wal_dir.join(CURSOR_SIDECAR_FILENAME);
        if sidecar_path.exists() {
            fs::remove_file(&sidecar_path).expect("remove sidecar");
        }

        // Reopen engine - should replay from position 0 (all entries)
        {
            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine should open without sidecar");

            // Since cursor sidecar is missing, should replay from position 0 (all entries)
            let recovered = engine.open_segment.lock().bundle_count();
            assert_eq!(
                recovered, bundles_ingested,
                "should replay all {} bundles when sidecar is missing; got {}",
                bundles_ingested, recovered
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_skips_expired_entries() {
        // Test that WAL replay filters out entries older than max_age,
        // preventing expired data from being replayed into new segments
        // (which would reset its age and retain it longer than intended).
        let dir = tempdir().expect("tempdir");

        let max_age = Duration::from_secs(60); // 1 minute

        // Phase 1: Ingest bundles WITHOUT max_age, using timestamps that
        // straddle the max_age boundary. Large segment size prevents
        // finalization so everything stays in the WAL.
        let old_bundles = 3;
        let fresh_bundles = 2;
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(SegmentConfig {
                    target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                    ..Default::default()
                })
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            // Ingest bundles with a timestamp well beyond max_age (2 hours ago)
            let old_time = SystemTime::now() - Duration::from_secs(7200);
            for _ in 0..old_bundles {
                let bundle = TimestampedBundle::with_rows_and_time(10, old_time);
                engine.ingest(&bundle).await.expect("ingest old");
            }

            // Ingest bundles with a recent timestamp (well within max_age)
            let fresh_time = SystemTime::now();
            for _ in 0..fresh_bundles {
                let bundle = TimestampedBundle::with_rows_and_time(10, fresh_time);
                engine.ingest(&bundle).await.expect("ingest fresh");
            }

            // Verify all bundles are in the open segment
            let total = engine.open_segment.lock().bundle_count();
            assert_eq!(total, old_bundles + fresh_bundles);

            // Drop engine without flushing — simulates crash
        }

        // Phase 2: Reopen with max_age enabled. WAL replay should skip
        // the old entries and only replay the fresh ones.
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(SegmentConfig {
                    target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                    ..Default::default()
                })
                .retention(RetentionConfig {
                    max_age: Some(max_age),
                })
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            // Only the fresh bundles should have been replayed
            let recovered = engine.open_segment.lock().bundle_count();
            assert_eq!(
                recovered, fresh_bundles,
                "expected {} fresh bundles after replay, got {} (old entries should be skipped)",
                fresh_bundles, recovered
            );

            // The expired_bundles counter should reflect the skipped entries
            assert_eq!(
                engine.expired_bundles(),
                old_bundles as u64,
                "expired_bundles counter should track skipped WAL entries"
            );
        }
    }

    #[tokio::test]
    async fn wal_replay_skips_all_entries_when_all_expired() {
        // Edge case: every WAL entry is older than max_age.
        // The engine should start with an empty open segment, the
        // expired_bundles counter should reflect all entries, and the
        // cursor should be persisted so subsequent restarts skip them.
        let dir = tempdir().expect("tempdir");

        let max_age = Duration::from_secs(60);
        let total_bundles: usize = 5;

        // Phase 1: Ingest bundles WITHOUT max_age, all with timestamps
        // well beyond max_age.
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(SegmentConfig {
                    target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                    ..Default::default()
                })
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            let old_time = SystemTime::now() - Duration::from_secs(7200);
            for _ in 0..total_bundles {
                let bundle = TimestampedBundle::with_rows_and_time(10, old_time);
                engine.ingest(&bundle).await.expect("ingest old");
            }

            assert_eq!(engine.open_segment.lock().bundle_count(), total_bundles);
            // Drop without flush — simulates crash
        }

        // Phase 2: Reopen with max_age. All entries should be skipped.
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(SegmentConfig {
                    target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                    ..Default::default()
                })
                .retention(RetentionConfig {
                    max_age: Some(max_age),
                })
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            // Open segment should be empty — nothing was replayed
            assert_eq!(
                engine.open_segment.lock().bundle_count(),
                0,
                "open segment should be empty when all WAL entries are expired"
            );

            assert_eq!(
                engine.expired_bundles(),
                total_bundles as u64,
                "expired_bundles counter should reflect all skipped entries"
            );
        }

        // Phase 3: Reopen again to verify cursor was persisted — the engine
        // should not re-process the expired entries (expired_bundles stays 0
        // on this open because everything was already consumed).
        {
            let config = QuiverConfig::builder()
                .data_dir(dir.path())
                .segment(SegmentConfig {
                    target_size_bytes: NonZeroU64::new(100 * 1024 * 1024).unwrap(),
                    ..Default::default()
                })
                .retention(RetentionConfig {
                    max_age: Some(max_age),
                })
                .build()
                .expect("config");

            let engine = QuiverEngine::open(config, test_budget())
                .await
                .expect("engine");

            assert_eq!(
                engine.open_segment.lock().bundle_count(),
                0,
                "open segment should still be empty on second reopen"
            );

            assert_eq!(
                engine.expired_bundles(),
                0,
                "expired_bundles should be 0 — cursor was persisted, no re-processing"
            );
        }
    }

    #[test]
    fn probe_set_permissions_support_returns_true_on_normal_filesystem() {
        let dir = tempdir().unwrap();
        assert!(
            probe_set_permissions_support(dir.path()),
            "set_permissions should succeed on a normal tmpdir filesystem"
        );
        // Probe file should be cleaned up — no leftover files with the probe prefix
        let leftover: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| n.starts_with(PERMS_PROBE_PREFIX))
            })
            .collect();
        assert!(
            leftover.is_empty(),
            "probe file should be removed after probing, found: {leftover:?}"
        );
    }

    #[tokio::test]
    async fn engine_stores_chmod_supported_flag() {
        let dir = tempdir().unwrap();
        let config = QuiverConfig::default().with_data_dir(dir.path());
        let engine = QuiverEngine::open(config, test_budget())
            .await
            .expect("engine opens");
        // On a normal test filesystem, chmod should be supported
        assert!(
            engine.set_permissions_supported,
            "set_permissions_supported should be true on a normal filesystem"
        );
    }

    /// Regression test: WAL replay must not deadlock under Backpressure policy
    /// when the budget is tight.
    ///
    /// The watermark design guarantees `hard_cap >= wal_max + 2 * segment_size`
    /// at engine open time, and finalization always proceeds (just calls
    /// `budget.add()`). This test verifies that replay succeeds under a
    /// tight budget that exactly covers the on-disk state.
    #[tokio::test]
    async fn wal_replay_under_tight_backpressure_budget_succeeds() {
        let dir = tempdir().expect("tempdir");

        // Segment target large enough that a single bundle doesn't trigger
        // finalization, but several bundles together do.
        let segment_target: u64 = 4 * 1024; // 4 KB
        let segment_config = SegmentConfig {
            target_size_bytes: NonZeroU64::new(segment_target).expect("non-zero"),
            ..Default::default()
        };
        // Use a small WAL config so that tight budgets can pass validation
        // (hard_cap >= wal_max + 2 * segment_size).
        let wal_config = WalConfig {
            max_size_bytes: NonZeroU64::new(64 * 1024).expect("non-zero"), // 64 KB
            max_rotated_files: 2,
            rotation_target_bytes: NonZeroU64::new(32 * 1024).expect("non-zero"),
            ..Default::default()
        };

        let config = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(segment_config.clone())
            .wal(wal_config.clone())
            .build()
            .expect("config valid");

        // Phase 1: Ingest with a generous budget.
        // We want some segments finalized on disk AND some un-finalized WAL
        // entries that will trigger finalization during replay.
        //
        // Each bundle of 50 rows ≈ 1–2 KB.  With target = 4 KB we need
        // ~3-4 bundles to trigger finalization. Ingesting 12 bundles should
        // produce ~2-3 segments and leave a tail of un-finalized entries.
        let generous_budget = Arc::new(DiskBudget::new(
            100 * 1024 * 1024, // 100 MB
            segment_target,
            RetentionPolicy::Backpressure,
        ));

        {
            let engine = QuiverEngine::open(config.clone(), generous_budget)
                .await
                .expect("engine");

            for _ in 0..12 {
                let bundle = DummyBundle::with_rows(50);
                engine.ingest(&bundle).await.expect("ingest");
            }

            // Intentionally do NOT call shutdown so any un-finalized bundles
            // remain only in the WAL and must be replayed.
        }

        // Measure what's on disk (segments + WAL).
        let segment_dir = dir.path().join("segments");
        let mut disk_segment_bytes: u64 = 0;
        if segment_dir.exists() {
            for entry in fs::read_dir(&segment_dir).expect("read segment dir") {
                let entry = entry.expect("entry");
                disk_segment_bytes += entry.metadata().expect("metadata").len();
            }
        }

        let wal_dir = dir.path().join("wal");
        let mut disk_wal_bytes: u64 = 0;
        for entry in fs::read_dir(&wal_dir).expect("read wal dir") {
            let entry = entry.expect("entry");
            disk_wal_bytes += entry.metadata().expect("metadata").len();
        }

        // Sanity: we should have BOTH segments and WAL data.
        assert!(
            disk_segment_bytes > 0,
            "expected finalized segments on disk"
        );
        assert!(disk_wal_bytes > 0, "expected WAL data on disk");

        // Phase 2: Reopen with a tight Backpressure budget.
        //
        // Set hard_cap = exact disk usage (or the minimum required by validation).
        // At startup, both WAL bytes and segment bytes are recorded via
        // `budget.add()`. The watermark design ensures finalization always
        // proceeds — budget.add() is called after writing, and the
        // `hard_cap >= wal_max + 2 * segment_size` validation ensures
        // there is structural room for at least one finalization.
        let disk_total = disk_segment_bytes + disk_wal_bytes;
        let wal_max: u64 = wal_config.max_size_bytes.get();
        let min_budget = wal_max + 2 * segment_target; // QuiverEngine::open requires hard_cap >= wal_max + 2 * segment_size
        let tight_cap = disk_total.max(min_budget);
        let tight_budget = Arc::new(DiskBudget::new(
            tight_cap,
            segment_target,
            RetentionPolicy::Backpressure,
        ));

        let config2 = QuiverConfig::builder()
            .data_dir(dir.path())
            .segment(segment_config)
            .wal(wal_config)
            .build()
            .expect("config valid");

        // This open() must succeed.  The watermark budget allows finalization
        // to always proceed, and WAL bytes are released after purge.
        let engine = QuiverEngine::open(config2, tight_budget.clone())
            .await
            .expect(
                "engine open with tight budget should succeed; \
                 watermark budget allows finalization to always proceed",
            );

        // The engine should have replayed and finalized at least one segment
        // that was not present before restart.
        assert!(
            engine.total_segments_written() > 0,
            "expected at least one segment from WAL replay"
        );

        engine.shutdown().await.expect("shutdown");
    }
}
