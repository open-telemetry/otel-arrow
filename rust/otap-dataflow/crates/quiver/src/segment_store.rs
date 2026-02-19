// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Segment store for managing finalized segments.
//!
//! The [`SegmentStore`] tracks finalized segment files and provides read access
//! via memory mapping or standard file I/O. It implements [`SegmentProvider`]
//! for integration with the subscriber registry.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use parking_lot::{Mutex, RwLock};

use crate::budget::DiskBudget;
use crate::logging::{otel_debug, otel_error, otel_warn};
use crate::segment::{ReconstructedBundle, SegmentReader, SegmentSeq};
use crate::subscriber::{BundleIndex, BundleRef, SegmentProvider, SubscriberError};

/// Maximum number of maintenance cycles a pending delete is retried before
/// the entry is abandoned and the budget is forcibly released.
///
/// When a deletion consistently fails (e.g. permanent permission error),
/// the segment's budget bytes are released after this many attempts to
/// avoid a permanent budget leak, even though the file may still be on
/// disk.  An error-level log is emitted so operators can investigate.
const MAX_DELETE_ATTEMPTS: u32 = 10;

/// Base interval for the exponential backoff between delete retries.
/// Each failure doubles the wait time: 1 s → 2 s → 4 s → … capped at
/// [`MAX_RETRY_INTERVAL`].
const BASE_RETRY_INTERVAL: Duration = Duration::from_secs(1);

/// Upper bound on the exponential backoff interval so that retries do
/// not stall for excessively long periods.
const MAX_RETRY_INTERVAL: Duration = Duration::from_secs(60);

/// A segment whose file deletion was deferred for later retry.
#[derive(Debug, Clone, Copy)]
struct PendingDelete {
    /// Size in bytes tracked by the budget for this segment.
    file_size: u64,
    /// How many times `retry_pending_deletes` has attempted to remove
    /// this file (starts at 0 on first deferral).
    attempts: u32,
    /// Current backoff interval.  Doubled (up to [`MAX_RETRY_INTERVAL`])
    /// after each failed attempt.
    backoff: Duration,
    /// Earliest [`Instant`] at which the next retry should be attempted.
    /// Entries whose `next_retry_at` is in the future are skipped by
    /// [`SegmentStore::retry_pending_deletes`], implementing exponential
    /// backoff.
    next_retry_at: Instant,
}

/// Result of scanning the segment directory during startup.
///
/// Contains both the segments that were successfully loaded and the
/// sequence numbers of segments that were deleted due to expiration.
/// The deleted IDs are needed so the subscriber registry can mark them
/// as completed, preventing subscribers from trying to read missing files.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// Segments that were found and registered, sorted by sequence number.
    /// Each entry contains the segment sequence and its bundle count.
    pub found: Vec<(SegmentSeq, u32)>,
    /// Segment sequences that were deleted during scan (e.g., expired by max_age).
    pub deleted: Vec<SegmentSeq>,
}

/// Type alias for subscriber-related results.
type Result<T> = std::result::Result<T, SubscriberError>;

/// Read mode for segment files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SegmentReadMode {
    /// Standard file I/O (reads entire file into memory)
    #[cfg_attr(not(feature = "mmap"), default)]
    Standard,
    /// Memory-mapped I/O (zero-copy, lazy loading)
    #[cfg_attr(feature = "mmap", default)]
    #[cfg(feature = "mmap")]
    Mmap,
}

impl std::fmt::Display for SegmentReadMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            #[cfg(feature = "mmap")]
            Self::Mmap => write!(f, "mmap"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SegmentHandle
// ─────────────────────────────────────────────────────────────────────────────

/// Handle to a loaded segment, providing read access.
struct SegmentHandle {
    /// The segment sequence number.
    seq: SegmentSeq,
    /// Path to the segment file.
    path: PathBuf,
    /// The segment reader.
    reader: SegmentReader,
    /// Number of bundles in this segment.
    bundle_count: u32,
    /// Size of the segment file in bytes.
    file_size_bytes: u64,
    /// Time when the segment was finalized (from file modification time).
    finalized_at: SystemTime,
}

impl SegmentHandle {
    /// Opens a segment file using the specified read mode.
    fn open(seq: SegmentSeq, path: PathBuf, mode: SegmentReadMode) -> Result<Self> {
        // Get file metadata for size and modification time (finalization timestamp)
        let metadata = std::fs::metadata(&path).map_err(|e| SubscriberError::SegmentIo {
            path: path.clone(),
            source: e,
        })?;
        let file_size_bytes = metadata.len();
        let finalized_at = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        let reader = match mode {
            SegmentReadMode::Standard => SegmentReader::open(&path),
            #[cfg(feature = "mmap")]
            SegmentReadMode::Mmap => SegmentReader::open_mmap(&path),
        }
        .map_err(|e| SubscriberError::SegmentIo {
            path: path.clone(),
            source: std::io::Error::other(e.to_string()),
        })?;

        let bundle_count = reader.manifest().len() as u32;

        Ok(Self {
            seq,
            path,
            reader,
            bundle_count,
            file_size_bytes,
            finalized_at,
        })
    }

    /// Reads a bundle by index.
    fn read_bundle(&self, bundle_index: BundleIndex) -> Result<ReconstructedBundle> {
        let idx = bundle_index.raw() as usize;
        let manifest = self.reader.manifest();

        if idx >= manifest.len() {
            return Err(SubscriberError::bundle_not_available(format!(
                "bundle index {} out of range (segment {} has {} bundles)",
                idx,
                self.seq,
                manifest.len()
            )));
        }

        self.reader
            .read_bundle(&manifest[idx])
            .map_err(|e| SubscriberError::SegmentIo {
                path: self.path.clone(),
                source: std::io::Error::other(e.to_string()),
            })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SegmentStore
// ─────────────────────────────────────────────────────────────────────────────

/// Callback type for segment finalization notifications.
pub type SegmentCallback = Box<dyn Fn(SegmentSeq, u32) + Send + Sync>;

/// Store for finalized segments.
///
/// Tracks segment files and provides read access. Thread-safe for concurrent
/// access from multiple subscribers.
pub struct SegmentStore {
    /// Directory containing segment files.
    segment_dir: PathBuf,
    /// Read mode for segment files.
    read_mode: SegmentReadMode,
    /// Loaded segments, keyed by sequence number.
    segments: RwLock<BTreeMap<SegmentSeq, Arc<SegmentHandle>>>,
    /// Optional callback invoked when a segment is registered.
    on_segment_registered: Mutex<Option<SegmentCallback>>,
    /// Optional disk budget for tracking segment storage usage.
    budget: Option<Arc<DiskBudget>>,
    /// Segments pending deletion (e.g., file still in use or transient I/O error).
    /// Entries are retried each maintenance cycle and evicted after
    /// [`MAX_DELETE_ATTEMPTS`] failures (releasing the budget with an error log).
    pending_deletes: Mutex<HashMap<SegmentSeq, PendingDelete>>,
}

impl std::fmt::Debug for SegmentStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let segments = self.segments.read();
        f.debug_struct("SegmentStore")
            .field("segment_dir", &self.segment_dir)
            .field("read_mode", &self.read_mode)
            .field("segment_count", &segments.len())
            .finish()
    }
}

impl SegmentStore {
    /// Creates a new segment store for the given directory.
    ///
    /// Uses the default read mode (mmap if available, otherwise standard).
    pub fn new(segment_dir: impl Into<PathBuf>) -> Self {
        Self::with_mode(segment_dir, SegmentReadMode::default())
    }

    /// Creates a new segment store with a specific read mode.
    pub fn with_mode(segment_dir: impl Into<PathBuf>, read_mode: SegmentReadMode) -> Self {
        Self {
            segment_dir: segment_dir.into(),
            read_mode,
            segments: RwLock::new(BTreeMap::new()),
            on_segment_registered: Mutex::new(None),
            budget: None,
            pending_deletes: Mutex::new(HashMap::new()),
        }
    }

    /// Creates a new segment store with a specific read mode and disk budget.
    ///
    /// The budget will be updated when segments are registered or deleted.
    pub fn with_budget(
        segment_dir: impl Into<PathBuf>,
        read_mode: SegmentReadMode,
        budget: Arc<DiskBudget>,
    ) -> Self {
        Self {
            segment_dir: segment_dir.into(),
            read_mode,
            segments: RwLock::new(BTreeMap::new()),
            on_segment_registered: Mutex::new(None),
            budget: Some(budget),
            pending_deletes: Mutex::new(HashMap::new()),
        }
    }

    /// Inserts or replaces a pending-delete entry, preserving the attempt
    /// count if the segment was already deferred.
    fn defer_delete(&self, seq: SegmentSeq, file_size: u64) {
        let mut pending = self.pending_deletes.lock();
        let _ = pending
            .entry(seq)
            .and_modify(|p| p.file_size = file_size)
            .or_insert(PendingDelete {
                file_size,
                attempts: 0,
                backoff: BASE_RETRY_INTERVAL,
                next_retry_at: Instant::now(),
            });
    }

    /// Sets a callback to be invoked when a segment is registered.
    ///
    /// The callback receives the segment sequence number and bundle count.
    /// This is typically used to notify a [`SubscriberRegistry`] of new segments.
    ///
    /// [`SubscriberRegistry`]: crate::subscriber::SubscriberRegistry
    pub fn set_on_segment_registered<F>(&self, callback: F)
    where
        F: Fn(SegmentSeq, u32) + Send + Sync + 'static,
    {
        *self.on_segment_registered.lock() = Some(Box::new(callback));
    }

    /// Returns the read mode being used.
    #[must_use]
    pub const fn read_mode(&self) -> SegmentReadMode {
        self.read_mode
    }

    /// Registers an existing segment discovered at startup.
    ///
    /// Called during `scan_existing` to load segments from a previous run.
    /// The segment's file size is recorded to the disk budget via
    /// `budget.add()` so that startup accounting reflects on-disk state.
    ///
    /// For newly written segments where `budget.add()` was already called
    /// by the engine, use [`register_new_segment`](Self::register_new_segment)
    /// instead to avoid double-counting.
    ///
    /// # Errors
    ///
    /// Returns an error if the segment file cannot be opened.
    pub fn register_existing_segment(&self, seq: SegmentSeq) -> Result<u32> {
        let path = self.segment_path(seq);
        let handle = SegmentHandle::open(seq, path, self.read_mode)?;
        let bundle_count = handle.bundle_count;
        let file_size = handle.file_size_bytes;

        {
            let mut segments = self.segments.write();
            let _ = segments.insert(seq, Arc::new(handle));
        }

        // Record file size to budget (for existing files loaded at startup)
        if let Some(ref budget) = self.budget {
            budget.add(file_size);
        }

        // Notify callback (outside of segments lock)
        if let Some(callback) = self.on_segment_registered.lock().as_ref() {
            callback(seq, bundle_count);
        }

        Ok(bundle_count)
    }

    /// Registers a newly written segment without budget accounting.
    ///
    /// Use this when the segment was just written and `budget.add()` was
    /// already called by the engine. This avoids double-counting bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the segment file cannot be opened.
    pub fn register_new_segment(&self, seq: SegmentSeq) -> Result<u32> {
        let path = self.segment_path(seq);
        let handle = SegmentHandle::open(seq, path, self.read_mode)?;
        let bundle_count = handle.bundle_count;

        {
            let mut segments = self.segments.write();
            let _ = segments.insert(seq, Arc::new(handle));
        }

        // Note: No budget recording - caller already called budget.add()

        // Notify callback (outside of segments lock)
        if let Some(callback) = self.on_segment_registered.lock().as_ref() {
            callback(seq, bundle_count);
        }

        Ok(bundle_count)
    }

    /// Removes a segment from the store.
    ///
    /// Called when all subscribers have completed a segment and it can be
    /// deleted. This only removes the segment from the in-memory map;
    /// use [`delete_segment`](Self::delete_segment) to also delete the file.
    pub fn unregister_segment(&self, seq: SegmentSeq) {
        let mut segments = self.segments.write();
        let _ = segments.remove(&seq);
    }

    /// Deletes a segment file from disk and removes it from the store.
    ///
    /// This is called when all subscribers have completed consuming a segment
    /// and it is safe to permanently delete.
    ///
    /// The segment's file size is released from the disk budget only when the
    /// file is actually removed (or confirmed already absent). If deletion
    /// fails (sharing violation, permission error, etc.), the segment is added
    /// to a pending-delete list and the budget is released later when
    /// [`retry_pending_deletes`](Self::retry_pending_deletes) succeeds.
    pub fn delete_segment(&self, seq: SegmentSeq) -> std::io::Result<()> {
        // Remove from in-memory map and capture file size for budget accounting.
        let file_size = {
            let mut segments = self.segments.write();
            segments.remove(&seq).map(|h| h.file_size_bytes)
        };

        // Attempt to delete the file from disk.
        let path = self.segment_path(seq);
        if path.exists() {
            match Self::remove_readonly_file(&path) {
                Ok(()) => {
                    // File deleted — release budget bytes.
                    if let (Some(budget), Some(size)) = (&self.budget, file_size) {
                        budget.remove(size);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // Raced with external deletion — file is gone, release budget.
                    if let (Some(budget), Some(size)) = (&self.budget, file_size) {
                        budget.remove(size);
                    }
                }
                Err(e) => {
                    // File still on disk (sharing violation, permission error, etc.).
                    // Defer deletion and keep the budget charged until the file is
                    // actually removed.
                    if Self::is_sharing_violation(&e) {
                        otel_debug!(
                            "quiver.segment.drop",
                            segment = seq.raw(),
                            phase = "deferred",
                        );
                    } else {
                        otel_warn!(
                        "quiver.segment.drop",
                            segment = seq.raw(),
                            error = %e,
                            error_type = "io",
                            phase = "deferred",
                            message = "Failed to delete segment, deferring for retry",
                        );
                    }
                    self.defer_delete(seq, file_size.unwrap_or(0));
                }
            }
        } else {
            // File doesn't exist on disk — release budget.
            if let (Some(budget), Some(size)) = (&self.budget, file_size) {
                budget.remove(size);
            }
        }

        Ok(())
    }

    /// Checks if an I/O error is a sharing violation (Windows-specific).
    ///
    /// On Windows, this occurs when trying to delete a file that's still open
    /// (e.g., memory-mapped by outstanding `BundleHandle`s).
    #[cfg(windows)]
    fn is_sharing_violation(error: &std::io::Error) -> bool {
        // Windows error code 32: ERROR_SHARING_VIOLATION
        // "The process cannot access the file because it is being used by another process."
        error.raw_os_error() == Some(32)
    }

    #[cfg(not(windows))]
    const fn is_sharing_violation(error: &std::io::Error) -> bool {
        let _ = error;
        false
    }

    /// Removes a read-only file from disk.
    ///
    /// On non-Unix platforms, defensively clears the read-only attribute before
    /// deletion. On Unix, file deletion depends on directory write permissions,
    /// not file permissions, so this step is skipped.
    ///
    /// Note: since Rust 1.85 (https://github.com/rust-lang/rust/pull/134679),
    /// `std::fs::remove_file` natively handles readonly files on Windows, so
    /// the attribute clearing is only needed for compatibility with older toolchains.
    fn remove_readonly_file(path: &Path) -> std::io::Result<()> {
        #[cfg(not(unix))]
        if let Ok(metadata) = std::fs::metadata(path) {
            let mut perms = metadata.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(path, perms);
        }
        std::fs::remove_file(path)
    }

    /// Retries deletion of segments that previously failed.
    ///
    /// On success (or if the file has been removed externally), releases the
    /// segment's tracked bytes from the disk budget.  On failure the attempt
    /// counter is incremented and the next retry is scheduled with
    /// exponential backoff (see [`BASE_RETRY_INTERVAL`] /
    /// [`MAX_RETRY_INTERVAL`]).  After [`MAX_DELETE_ATTEMPTS`] consecutive
    /// failures the entry is **evicted**: the budget bytes are released and
    /// an error is logged so operators can investigate the orphaned file.
    ///
    /// Entries whose backoff timer has not yet elapsed are skipped so that
    /// callers can invoke this method on every ingest cycle without
    /// hammering the filesystem.
    ///
    /// Returns the number of segments whose budget was released (successful
    /// deletes + abandoned entries).
    pub fn retry_pending_deletes(&self) -> usize {
        let now = Instant::now();
        let pending: Vec<(SegmentSeq, PendingDelete)> = {
            let pending = self.pending_deletes.lock();
            pending
                .iter()
                .filter(|(_, pd)| now >= pd.next_retry_at)
                .map(|(&seq, &pd)| (seq, pd))
                .collect()
        };

        if pending.is_empty() {
            return 0;
        }

        let mut cleared = 0;
        for (seq, pd) in pending {
            let path = self.segment_path(seq);
            if !path.exists() {
                // File was deleted externally — release budget and remove from pending.
                let _ = self.pending_deletes.lock().remove(&seq);
                if let Some(budget) = &self.budget {
                    budget.remove(pd.file_size);
                }
                cleared += 1;
                continue;
            }

            match Self::remove_readonly_file(&path) {
                Ok(()) => {
                    let _ = self.pending_deletes.lock().remove(&seq);
                    if let Some(budget) = &self.budget {
                        budget.remove(pd.file_size);
                    }
                    otel_debug!(
                        "quiver.segment.drop",
                        segment = seq.raw(),
                        phase = "deferred",
                        message = "Successfully deleted deferred segment on retry",
                    );
                    cleared += 1;
                }
                Err(e) if Self::is_sharing_violation(&e) => {
                    // Still in use — schedule next retry with backoff.
                    let mut pending = self.pending_deletes.lock();
                    if let Some(entry) = pending.get_mut(&seq) {
                        entry.attempts += 1;
                        entry.next_retry_at = now + entry.backoff;
                        entry.backoff = (entry.backoff * 2).min(MAX_RETRY_INTERVAL);
                    }
                    otel_debug!(
                        "quiver.segment.drop",
                        segment = seq.raw(),
                        phase = "deferred",
                        attempt = pd.attempts + 1,
                        message = "Segment still in use, will retry deletion after backoff",
                    );
                }
                Err(e) => {
                    // Other error — schedule next retry with backoff.
                    let mut pending = self.pending_deletes.lock();
                    if let Some(entry) = pending.get_mut(&seq) {
                        entry.attempts += 1;
                        entry.next_retry_at = now + entry.backoff;
                        entry.backoff = (entry.backoff * 2).min(MAX_RETRY_INTERVAL);
                    }
                    otel_warn!(
                        "quiver.segment.drop",
                        segment = seq.raw(),
                        error = %e,
                        error_type = "io",
                        phase = "deferred",
                        attempt = pd.attempts + 1,
                        message = "Failed to delete deferred segment, will retry after backoff",
                    );
                }
            }
        }

        // Evict entries that have exceeded the maximum retry count.
        // Release their budget bytes so persistent errors don't cause a
        // permanent budget leak.
        {
            let mut pending = self.pending_deletes.lock();
            let abandoned: Vec<(SegmentSeq, u64)> = pending
                .iter()
                .filter(|(_, pd)| pd.attempts >= MAX_DELETE_ATTEMPTS)
                .map(|(&seq, pd)| (seq, pd.file_size))
                .collect();
            for (seq, size) in abandoned {
                let _ = pending.remove(&seq);
                if let Some(budget) = &self.budget {
                    budget.remove(size);
                }
                otel_error!(
                    "quiver.segment.drop",
                    segment = seq.raw(),
                    phase = "abandoned",
                    attempts = MAX_DELETE_ATTEMPTS,
                    message = "Giving up on segment deletion after max retries; \
                               budget released but file may remain on disk",
                );
                cleared += 1;
            }
        }

        cleared
    }

    /// Returns the number of segments pending deletion.
    #[must_use]
    pub fn pending_delete_count(&self) -> usize {
        self.pending_deletes.lock().len()
    }

    /// Returns the path for a segment file.
    fn segment_path(&self, seq: SegmentSeq) -> PathBuf {
        self.segment_dir
            .join(format!("{}.qseg", seq.to_filename_component()))
    }

    /// Scans the segment directory and loads existing segments.
    ///
    /// Called during startup to discover segments from a previous run.
    /// Use [`scan_existing_with_max_age`](Self::scan_existing_with_max_age) to
    /// automatically skip loading segments that have exceeded the retention age.
    ///
    /// # Errors
    ///
    /// Returns an error if directory scanning fails.
    pub fn scan_existing(&self) -> Result<ScanResult> {
        self.scan_existing_with_max_age(None)
    }

    /// Scans the segment directory and loads existing segments, optionally
    /// filtering out expired segments without loading them.
    ///
    /// When `max_age` is provided, segment files whose modification time
    /// exceeds the age limit are deleted without being loaded into memory.
    /// This is more efficient than loading all segments and then calling
    /// [`cleanup_expired_segments`](crate::QuiverEngine::cleanup_expired_segments).
    ///
    /// # Arguments
    ///
    /// * `max_age` - If provided, segments older than this duration are deleted
    ///   without being loaded.
    ///
    /// # Errors
    ///
    /// Returns an error if directory scanning fails.
    pub fn scan_existing_with_max_age(&self, max_age: Option<Duration>) -> Result<ScanResult> {
        let mut found = Vec::new();
        let mut deleted = Vec::new();
        let now = SystemTime::now();

        // Pre-compute cutoff time: segments modified before this are expired.
        let cutoff = max_age.and_then(|age| now.checked_sub(age));

        let entries =
            std::fs::read_dir(&self.segment_dir).map_err(|e| SubscriberError::SegmentIo {
                path: self.segment_dir.clone(),
                source: e,
            })?;

        for entry in entries {
            let entry = entry.map_err(|e| SubscriberError::SegmentIo {
                path: self.segment_dir.clone(),
                source: e,
            })?;

            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "qseg") {
                if let Some(seq) = Self::parse_segment_filename(&path) {
                    // Check if segment is expired before loading
                    if let Some(cutoff) = cutoff {
                        if Self::is_file_before_cutoff(&path, cutoff) {
                            // Delete expired segment without loading it.
                            // No budget release needed - file was never registered.
                            if let Err(e) = Self::remove_readonly_file(&path) {
                                otel_warn!(
                                    "quiver.segment.scan",
                                    path = %path.display(),
                                    error = %e,
                                    error_type = "io",
                                );
                            } else {
                                otel_debug!("quiver.segment.scan", segment = seq.raw(),);
                                deleted.push(seq);
                            }
                            continue;
                        }
                    }

                    match self.register_existing_segment(seq) {
                        Ok(bundle_count) => found.push((seq, bundle_count)),
                        Err(e) => {
                            otel_error!(
                                "quiver.segment.scan",
                                path = %path.display(),
                                error = %e,
                                error_type = "io",
                                message = "segment data is inaccessible and may indicate corruption",
                            );
                        }
                    }
                }
            }
        }

        // Sort by sequence number
        found.sort_by_key(|(seq, _)| *seq);
        deleted.sort();

        Ok(ScanResult { found, deleted })
    }

    /// Checks if a file's modification time is before the cutoff time.
    ///
    /// Returns `true` if the file should be considered expired, or `false` if the
    /// metadata cannot be read (fail-safe: don't delete if we can't check).
    fn is_file_before_cutoff(path: &Path, cutoff: SystemTime) -> bool {
        match std::fs::metadata(path) {
            Ok(metadata) => match metadata.modified() {
                Ok(mtime) => mtime < cutoff,
                Err(_) => false, // Can't determine age, don't expire
            },
            Err(_) => false, // Can't read metadata, don't expire
        }
    }

    /// Parses a segment sequence number from a filename.
    fn parse_segment_filename(path: &Path) -> Option<SegmentSeq> {
        let stem = path.file_stem()?.to_str()?;
        let raw: u64 = stem.parse().ok()?;
        Some(SegmentSeq::new(raw))
    }

    /// Returns the number of loaded segments.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segments.read().len()
    }

    /// Returns all loaded segment sequences.
    #[must_use]
    pub fn segment_sequences(&self) -> Vec<SegmentSeq> {
        self.segments.read().keys().copied().collect()
    }

    /// Returns segments that were finalized more than `max_age` ago.
    ///
    /// This is used for age-based retention to identify segments that have
    /// exceeded the configured maximum retention time. The returned segments
    /// are candidates for deletion via [`cleanup_expired_segments`].
    ///
    /// [`cleanup_expired_segments`]: crate::QuiverEngine::cleanup_expired_segments
    #[must_use]
    pub fn segments_older_than(&self, max_age: Duration) -> Vec<SegmentSeq> {
        let now = SystemTime::now();

        let Some(cutoff) = now.checked_sub(max_age) else {
            // max_age exceeds time since UNIX_EPOCH - nothing can be expired
            return Vec::new();
        };

        let segments = self.segments.read();

        segments
            .iter()
            .filter_map(|(seq, handle)| {
                if handle.finalized_at < cutoff {
                    Some(*seq)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl SegmentProvider for SegmentStore {
    fn bundle_count(&self, segment_seq: SegmentSeq) -> Result<u32> {
        let segments = self.segments.read();
        segments
            .get(&segment_seq)
            .map(|h| h.bundle_count)
            .ok_or_else(|| SubscriberError::segment_not_found(segment_seq.raw()))
    }

    fn read_bundle(&self, bundle_ref: BundleRef) -> Result<ReconstructedBundle> {
        let segments = self.segments.read();
        let handle = segments
            .get(&bundle_ref.segment_seq)
            .ok_or_else(|| SubscriberError::segment_not_found(bundle_ref.segment_seq.raw()))?;

        handle.read_bundle(bundle_ref.bundle_index)
    }

    fn available_segments(&self) -> Vec<SegmentSeq> {
        self.segment_sequences()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RetentionPolicy;
    use tempfile::tempdir;

    #[test]
    fn segment_store_empty() {
        let dir = tempdir().unwrap();
        let store = SegmentStore::new(dir.path().join("segments"));
        assert_eq!(store.segment_count(), 0);
        assert!(store.segment_sequences().is_empty());
    }

    #[test]
    fn segment_store_missing_segment() {
        let dir = tempdir().unwrap();
        let store = SegmentStore::new(dir.path().join("segments"));

        let result = store.bundle_count(SegmentSeq::new(1));
        assert!(matches!(
            result,
            Err(SubscriberError::SegmentNotFound { .. })
        ));
    }

    #[test]
    fn parse_segment_filename() {
        let seq = SegmentStore::parse_segment_filename(Path::new("0000000000000042.qseg"));
        assert_eq!(seq, Some(SegmentSeq::new(42)));

        let seq = SegmentStore::parse_segment_filename(Path::new("invalid.qseg"));
        assert_eq!(seq, None);

        // Note: parse_segment_filename only parses the numeric stem.
        // Extension filtering (.qseg) is done by scan_existing().
        let seq = SegmentStore::parse_segment_filename(Path::new("0000000000000001.txt"));
        assert_eq!(seq, Some(SegmentSeq::new(1)));
    }

    #[test]
    fn segments_older_than_empty_store() {
        let dir = tempdir().unwrap();
        let store = SegmentStore::new(dir.path().join("segments"));

        // No segments, should return empty
        let expired = store.segments_older_than(Duration::from_secs(1));
        assert!(expired.is_empty());
    }

    #[test]
    fn is_file_before_cutoff_checks_mtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        let now = SystemTime::now();

        // File just created, cutoff 1 hour ago - file is NOT before cutoff
        let cutoff_1hr_ago = now.checked_sub(Duration::from_secs(3600)).unwrap();
        assert!(!SegmentStore::is_file_before_cutoff(
            &file_path,
            cutoff_1hr_ago
        ));

        // With a very short max_age and sleep, file should be before cutoff
        std::thread::sleep(Duration::from_millis(50));
        let later = SystemTime::now();
        let cutoff_10ms_ago = later.checked_sub(Duration::from_millis(10)).unwrap();
        assert!(SegmentStore::is_file_before_cutoff(
            &file_path,
            cutoff_10ms_ago
        ));
    }

    #[test]
    fn is_file_before_cutoff_nonexistent_file() {
        // Non-existent file should not be considered expired (returns false)
        let nonexistent = Path::new("/nonexistent/file.txt");
        let cutoff = SystemTime::now();
        assert!(!SegmentStore::is_file_before_cutoff(nonexistent, cutoff));
    }

    #[test]
    fn pending_delete_count_initially_zero() {
        let dir = tempdir().unwrap();
        let store = SegmentStore::new(dir.path().join("segments"));
        assert_eq!(store.pending_delete_count(), 0);
    }

    #[test]
    fn retry_pending_deletes_empty() {
        let dir = tempdir().unwrap();
        let store = SegmentStore::new(dir.path().join("segments"));

        // No pending deletes, should return 0
        let deleted = store.retry_pending_deletes();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn retry_pending_deletes_file_already_gone() {
        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();
        let store = SegmentStore::new(&segment_dir);

        // Manually add a segment to pending deletes (simulating a failed delete)
        {
            let mut pending = store.pending_deletes.lock();
            let _ = pending.insert(
                SegmentSeq::new(1),
                PendingDelete {
                    file_size: 0,
                    attempts: 0,
                    backoff: BASE_RETRY_INTERVAL,
                    next_retry_at: Instant::now(),
                },
            );
        }
        assert_eq!(store.pending_delete_count(), 1);

        // The file doesn't exist, so retry should succeed (removes from pending)
        let deleted = store.retry_pending_deletes();
        assert_eq!(deleted, 1);
        assert_eq!(store.pending_delete_count(), 0);
    }

    #[test]
    fn retry_pending_deletes_file_exists_deletable() {
        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();
        let store = SegmentStore::new(&segment_dir);

        // Create a dummy segment file
        let seq = SegmentSeq::new(42);
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        std::fs::write(&path, "dummy segment data").unwrap();
        assert!(path.exists());

        // Manually add to pending deletes
        {
            let mut pending = store.pending_deletes.lock();
            let _ = pending.insert(
                seq,
                PendingDelete {
                    file_size: 0,
                    attempts: 0,
                    backoff: BASE_RETRY_INTERVAL,
                    next_retry_at: Instant::now(),
                },
            );
        }
        assert_eq!(store.pending_delete_count(), 1);

        // Retry should delete the file
        let deleted = store.retry_pending_deletes();
        assert_eq!(deleted, 1);
        assert_eq!(store.pending_delete_count(), 0);
        assert!(!path.exists());
    }

    #[test]
    fn is_sharing_violation_always_false_on_unix() {
        // On non-Windows, is_sharing_violation should always return false
        let err = std::io::Error::other("test error");
        assert!(!SegmentStore::is_sharing_violation(&err));

        #[cfg(not(windows))]
        {
            let err = std::io::Error::from_raw_os_error(32); // Would be sharing violation on Windows
            assert!(!SegmentStore::is_sharing_violation(&err));
        }
    }

    #[cfg(windows)]
    #[test]
    fn is_sharing_violation_detects_error_code_32_on_windows() {
        // On Windows, error code 32 is ERROR_SHARING_VIOLATION
        let err = std::io::Error::from_raw_os_error(32);
        assert!(SegmentStore::is_sharing_violation(&err));

        // Other error codes should not be detected as sharing violations
        let err = std::io::Error::from_raw_os_error(5); // ERROR_ACCESS_DENIED
        assert!(!SegmentStore::is_sharing_violation(&err));

        let err = std::io::Error::other("generic error");
        assert!(!SegmentStore::is_sharing_violation(&err));
    }

    #[cfg(windows)]
    #[test]
    fn delete_segment_defers_on_sharing_violation_windows() {
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;

        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();
        let store = SegmentStore::new(&segment_dir);

        // Create a segment file
        let seq = SegmentSeq::new(99);
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        std::fs::write(&path, "segment data for windows test").unwrap();

        // Open the file with a handle that prevents deletion (sharing violation)
        // On Windows, opening without FILE_SHARE_DELETE will cause delete to fail
        let _held_handle = OpenOptions::new()
            .read(true)
            .share_mode(0) // No sharing - this prevents deletion on Windows
            .open(&path)
            .expect("should open file");

        // Add segment to the store's in-memory map (simulating a loaded segment)
        // We can't use register_segment since the file format won't be valid,
        // so we just test delete_segment directly which removes from map first
        assert_eq!(store.pending_delete_count(), 0);

        // Try to delete - should fail with sharing violation and add to pending
        let result = store.delete_segment(seq);
        assert!(
            result.is_ok(),
            "delete_segment should not error on sharing violation"
        );
        assert_eq!(
            store.pending_delete_count(),
            1,
            "segment should be added to pending deletes"
        );

        // File should still exist
        assert!(
            path.exists(),
            "file should still exist after deferred delete"
        );

        // Drop the handle to release the file
        drop(_held_handle);

        // Now retry should succeed
        let deleted = store.retry_pending_deletes();
        assert_eq!(deleted, 1, "retry should successfully delete");
        assert_eq!(store.pending_delete_count(), 0);
        assert!(!path.exists(), "file should be deleted after retry");
    }

    #[test]
    fn remove_readonly_file_deletes_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.qseg");
        std::fs::write(&file_path, "test data").unwrap();
        assert!(file_path.exists());

        SegmentStore::remove_readonly_file(&file_path).unwrap();
        assert!(!file_path.exists());
    }

    /// Verifies that `remove_readonly_file` successfully deletes readonly files.
    ///
    /// Note: since Rust 1.85 (https://github.com/rust-lang/rust/pull/134679),
    /// `std::fs::remove_file` natively handles readonly files on Windows, so
    /// the attribute clearing is only needed for compatibility with older toolchains.
    #[test]
    fn remove_readonly_file_handles_readonly() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("readonly.qseg");
        let content = "readonly content";
        std::fs::write(&file_path, content).unwrap();

        // Make the file read-only
        let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&file_path, perms).unwrap();

        // Verify it's actually read-only
        assert!(
            std::fs::metadata(&file_path)
                .unwrap()
                .permissions()
                .readonly()
        );

        // Should still be able to delete it
        SegmentStore::remove_readonly_file(&file_path).unwrap();
        assert!(!file_path.exists(), "read-only file should be deleted");
    }

    #[test]
    fn remove_readonly_file_nonexistent_errors() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.qseg");

        let result = SegmentStore::remove_readonly_file(&file_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    /// Test that future mtime (clock skew) does not cause spurious expiration.
    ///
    /// If a file's mtime is in the future relative to the cutoff time,
    /// it should not be considered expired.
    #[test]
    fn is_file_before_cutoff_future_mtime_not_expired() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        // Use a cutoff time far in the past - the file's mtime will be after it
        let ancient_cutoff = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);

        // File created recently should NOT be before the ancient cutoff
        assert!(
            !SegmentStore::is_file_before_cutoff(&file_path, ancient_cutoff),
            "file with mtime after cutoff should not be considered expired"
        );
    }

    // ── Budget interaction tests ─────────────────────────────────────────

    /// Helper: creates a store with a budget and writes a real segment file
    /// that can be registered. Returns (store, budget, segment_seq, file_size).
    fn store_with_budget_and_segment() -> (SegmentStore, Arc<DiskBudget>, SegmentSeq, u64) {
        use crate::segment::OpenSegment;
        use crate::segment::SegmentWriter;
        use crate::segment::test_utils::{
            TestBundle, make_batch, slot_descriptors, test_fingerprint, test_schema,
        };

        let dir = tempdir().unwrap();
        let segment_dir = dir.keep().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();

        let budget = Arc::new(DiskBudget::new(
            10_000_000,
            1_000_000,
            RetentionPolicy::Backpressure,
        ));
        let store =
            SegmentStore::with_budget(&segment_dir, SegmentReadMode::Standard, budget.clone());

        // Build a minimal segment file via the writer.
        let seq = SegmentSeq::new(1);
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        let schema = test_schema();
        let batch = make_batch(&schema, &[1, 2], &["a", "b"]);
        let fp = test_fingerprint();
        let mut open_segment = OpenSegment::new();
        let bundle = TestBundle::new(slot_descriptors()).with_payload(
            crate::record_bundle::SlotId::new(0),
            fp,
            batch,
        );
        let _ = open_segment.append(&bundle).unwrap();
        let writer = SegmentWriter::new(seq, true);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (bytes_written, _) = rt
            .block_on(writer.write_segment(&path, open_segment))
            .unwrap();
        assert!(bytes_written > 0);

        (store, budget, seq, bytes_written)
    }

    #[test]
    fn delete_segment_releases_budget_on_success() {
        let (store, budget, seq, file_size) = store_with_budget_and_segment();

        // Register the segment (adds file_size to budget via register_existing_segment).
        let _ = store.register_existing_segment(seq).unwrap();
        assert_eq!(budget.used(), file_size);

        // Delete the segment — file should be removed and budget released.
        store.delete_segment(seq).unwrap();
        assert_eq!(
            budget.used(),
            0,
            "budget should be zero after successful delete"
        );
        assert_eq!(store.pending_delete_count(), 0);
    }

    #[test]
    fn delete_segment_releases_budget_when_file_already_gone() {
        let (store, budget, seq, file_size) = store_with_budget_and_segment();

        // Register the segment.
        let _ = store.register_existing_segment(seq).unwrap();
        assert_eq!(budget.used(), file_size);

        // Externally remove the file before calling delete_segment.
        let path = store.segment_path(seq);
        std::fs::remove_file(&path).unwrap();

        // delete_segment should still release budget (file doesn't exist on disk).
        store.delete_segment(seq).unwrap();
        assert_eq!(
            budget.used(),
            0,
            "budget should be released even if file was already gone"
        );
        assert_eq!(store.pending_delete_count(), 0);
    }

    #[test]
    fn retry_pending_deletes_releases_budget_on_success() {
        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();

        let budget = Arc::new(DiskBudget::new(
            10_000_000,
            1_000_000,
            RetentionPolicy::Backpressure,
        ));
        let store =
            SegmentStore::with_budget(&segment_dir, SegmentReadMode::Standard, budget.clone());

        // Simulate a segment whose delete was deferred: file on disk, size in budget.
        let seq = SegmentSeq::new(42);
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        let tracked_size: u64 = 5000;
        std::fs::write(&path, vec![0u8; tracked_size as usize]).unwrap();

        // Pre-charge budget and insert into pending_deletes.
        budget.add(tracked_size);
        assert_eq!(budget.used(), tracked_size);
        {
            let mut pending = store.pending_deletes.lock();
            let _ = pending.insert(
                seq,
                PendingDelete {
                    file_size: tracked_size,
                    attempts: 0,
                    backoff: BASE_RETRY_INTERVAL,
                    next_retry_at: Instant::now(),
                },
            );
        }

        // Retry should delete the file and release the budget.
        let deleted = store.retry_pending_deletes();
        assert_eq!(deleted, 1);
        assert_eq!(store.pending_delete_count(), 0);
        assert!(!path.exists());
        assert_eq!(
            budget.used(),
            0,
            "budget should be released after retry succeeds"
        );
    }

    #[test]
    fn retry_pending_deletes_releases_budget_when_file_externally_removed() {
        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();

        let budget = Arc::new(DiskBudget::new(
            10_000_000,
            1_000_000,
            RetentionPolicy::Backpressure,
        ));
        let store =
            SegmentStore::with_budget(&segment_dir, SegmentReadMode::Standard, budget.clone());

        // Simulate deferred delete where the file no longer exists.
        let seq = SegmentSeq::new(7);
        let tracked_size: u64 = 3000;
        budget.add(tracked_size);
        {
            let mut pending = store.pending_deletes.lock();
            let _ = pending.insert(
                seq,
                PendingDelete {
                    file_size: tracked_size,
                    attempts: 0,
                    backoff: BASE_RETRY_INTERVAL,
                    next_retry_at: Instant::now(),
                },
            );
        }

        // File doesn't exist — retry should release budget and clear pending.
        let deleted = store.retry_pending_deletes();
        assert_eq!(deleted, 1);
        assert_eq!(store.pending_delete_count(), 0);
        assert_eq!(
            budget.used(),
            0,
            "budget should be released when file is externally gone"
        );
    }

    #[test]
    fn delete_segment_without_budget_does_not_panic() {
        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();

        // Store without a budget.
        let store = SegmentStore::new(&segment_dir);

        // Create a dummy segment file and manually insert into pending deletes.
        let seq = SegmentSeq::new(1);
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        std::fs::write(&path, "test data").unwrap();

        // delete_segment should succeed without a budget configured.
        store.delete_segment(seq).unwrap();
        assert!(!path.exists());
        assert_eq!(store.pending_delete_count(), 0);
    }

    #[test]
    fn retry_pending_deletes_abandons_after_max_attempts() {
        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();

        let budget = Arc::new(DiskBudget::new(
            10_000_000,
            1_000_000,
            RetentionPolicy::Backpressure,
        ));
        let store =
            SegmentStore::with_budget(&segment_dir, SegmentReadMode::Standard, budget.clone());

        // Create a segment file that we'll make undeletable (simulated via
        // pre-setting attempts to the threshold).
        let seq = SegmentSeq::new(99);
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        let tracked_size: u64 = 7000;
        std::fs::write(&path, vec![0u8; tracked_size as usize]).unwrap();

        budget.add(tracked_size);
        assert_eq!(budget.used(), tracked_size);

        // Simulate that we've already retried MAX_DELETE_ATTEMPTS times.
        {
            let mut pending = store.pending_deletes.lock();
            let _ = pending.insert(
                seq,
                PendingDelete {
                    file_size: tracked_size,
                    attempts: MAX_DELETE_ATTEMPTS,
                    backoff: MAX_RETRY_INTERVAL,
                    next_retry_at: Instant::now(),
                },
            );
        }

        // On the next retry cycle, the entry should be evicted and budget released.
        let cleared = store.retry_pending_deletes();
        // retry_pending_deletes will first successfully delete the file (it IS
        // deletable in this test env), so it counts as cleared = 1 that way.
        // But if we truly can't delete, it evicts.  Either way the budget is freed.
        assert!(cleared >= 1, "entry should be cleared");
        assert_eq!(store.pending_delete_count(), 0);
        assert_eq!(
            budget.used(),
            0,
            "budget should be released after abandoning"
        );
    }

    #[test]
    fn retry_pending_deletes_increments_attempts_on_failure() {
        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();

        let budget = Arc::new(DiskBudget::new(
            10_000_000,
            1_000_000,
            RetentionPolicy::Backpressure,
        ));
        let store =
            SegmentStore::with_budget(&segment_dir, SegmentReadMode::Standard, budget.clone());

        // Create a read-only directory segment entry that can't be deleted
        // by making the segment path a subdirectory (remove_file fails on dirs).
        let seq = SegmentSeq::new(77);
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        std::fs::create_dir_all(&path).unwrap();

        let tracked_size: u64 = 4000;
        budget.add(tracked_size);
        {
            let mut pending = store.pending_deletes.lock();
            let _ = pending.insert(
                seq,
                PendingDelete {
                    file_size: tracked_size,
                    attempts: 0,
                    backoff: BASE_RETRY_INTERVAL,
                    next_retry_at: Instant::now(),
                },
            );
        }

        // First retry: deletion fails (it's a directory), attempts incremented.
        let cleared = store.retry_pending_deletes();
        assert_eq!(cleared, 0, "should not be cleared on first failure");
        assert_eq!(store.pending_delete_count(), 1);
        assert_eq!(
            budget.used(),
            tracked_size,
            "budget should remain charged while retrying"
        );

        // Verify attempts incremented and backoff doubled.
        {
            let pending = store.pending_deletes.lock();
            let pd = pending.get(&seq).expect("should still be pending");
            assert_eq!(pd.attempts, 1);
            assert_eq!(pd.backoff, BASE_RETRY_INTERVAL * 2);
            assert!(
                pd.next_retry_at > Instant::now() - Duration::from_millis(100),
                "next_retry_at should have been updated"
            );
        }

        // Clean up the directory so the test doesn't leave artifacts.
        let _ = std::fs::remove_dir(&path);
    }

    #[test]
    fn retry_pending_deletes_skips_entries_with_future_backoff() {
        let dir = tempdir().unwrap();
        let segment_dir = dir.path().join("segments");
        std::fs::create_dir_all(&segment_dir).unwrap();

        let budget = Arc::new(DiskBudget::new(
            10_000_000,
            1_000_000,
            RetentionPolicy::Backpressure,
        ));
        let store =
            SegmentStore::with_budget(&segment_dir, SegmentReadMode::Standard, budget.clone());

        // Create a real deletable file but set next_retry_at far in the future.
        let seq = SegmentSeq::new(50);
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        let tracked_size: u64 = 2000;
        std::fs::write(&path, vec![0u8; tracked_size as usize]).unwrap();

        budget.add(tracked_size);
        {
            let mut pending = store.pending_deletes.lock();
            let _ = pending.insert(
                seq,
                PendingDelete {
                    file_size: tracked_size,
                    attempts: 1,
                    backoff: BASE_RETRY_INTERVAL * 2,
                    next_retry_at: Instant::now() + Duration::from_secs(3600),
                },
            );
        }

        // Retry should skip this entry because the backoff timer hasn't elapsed.
        let cleared = store.retry_pending_deletes();
        assert_eq!(
            cleared, 0,
            "should skip entries whose backoff has not elapsed"
        );
        assert_eq!(
            store.pending_delete_count(),
            1,
            "entry should remain pending"
        );
        assert!(path.exists(), "file should still exist");
        assert_eq!(budget.used(), tracked_size, "budget should remain charged");
    }
}
