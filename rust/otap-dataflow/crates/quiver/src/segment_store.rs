// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Segment store for managing finalized segments.
//!
//! The [`SegmentStore`] tracks finalized segment files and provides read access
//! via memory mapping or standard file I/O. It implements [`SegmentProvider`]
//! for integration with the subscriber registry.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use parking_lot::{Mutex, RwLock};

use crate::budget::DiskBudget;
use crate::segment::{ReconstructedBundle, SegmentReader, SegmentSeq};
use crate::subscriber::{BundleIndex, BundleRef, SegmentProvider, SubscriberError};

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
    /// Segments pending deletion (e.g., file deletion failed due to sharing violation on Windows).
    /// These will be retried during the next maintenance cycle.
    pending_deletes: Mutex<HashSet<SegmentSeq>>,
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
            pending_deletes: Mutex::new(HashSet::new()),
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
            pending_deletes: Mutex::new(HashSet::new()),
        }
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
    pub fn read_mode(&self) -> SegmentReadMode {
        self.read_mode
    }

    /// Registers a newly finalized segment.
    ///
    /// Called by the engine after writing a segment file. If a callback was
    /// set via [`set_on_segment_registered`](Self::set_on_segment_registered),
    /// it will be invoked with the segment sequence and bundle count.
    ///
    /// The segment's file size is recorded to the disk budget if one was configured.
    /// Use this for loading existing segments at startup.
    /// For newly written segments where budget was already reserved, use
    /// [`register_new_segment`](Self::register_new_segment) instead.
    ///
    /// # Errors
    ///
    /// Returns an error if the segment file cannot be opened.
    pub fn register_segment(&self, seq: SegmentSeq) -> Result<u32> {
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
            budget.record_existing(file_size);
        }

        // Notify callback (outside of segments lock)
        if let Some(callback) = self.on_segment_registered.lock().as_ref() {
            callback(seq, bundle_count);
        }

        Ok(bundle_count)
    }

    /// Registers a newly written segment without budget accounting.
    ///
    /// Use this when the segment was just written and budget was already reserved
    /// via the reservation pattern. This avoids double-counting bytes.
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

        // Note: No budget recording - caller already committed the reservation

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
    /// The segment's file size is released from the disk budget if one was configured.
    ///
    /// If the file cannot be deleted (e.g., due to a sharing violation on Windows
    /// when the segment is still memory-mapped), the segment is added to a pending
    /// delete list and will be retried during the next maintenance cycle via
    /// [`retry_pending_deletes`](Self::retry_pending_deletes).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be deleted (e.g., permissions).
    /// Note: On Windows, sharing violations are handled gracefully by deferring deletion.
    pub fn delete_segment(&self, seq: SegmentSeq) -> std::io::Result<()> {
        // First remove from in-memory map and get file size for budget release
        let file_size = {
            let mut segments = self.segments.write();
            segments.remove(&seq).map(|h| h.file_size_bytes)
        };

        // Delete the file from disk
        let path = self.segment_path(seq);
        if path.exists() {
            // Segment files are read-only after finalization, make writable first
            if let Ok(metadata) = path.metadata() {
                let mut perms = metadata.permissions();
                #[allow(clippy::permissions_set_readonly_false)]
                perms.set_readonly(false);
                let _ = std::fs::set_permissions(&path, perms);
            }

            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if Self::is_sharing_violation(&e) => {
                    // On Windows, the file may still be memory-mapped by outstanding
                    // BundleHandles. Add to pending deletes for retry later.
                    tracing::debug!(
                        segment = seq.raw(),
                        "Segment file in use, deferring deletion"
                    );
                    let _ = self.pending_deletes.lock().insert(seq);
                    // Don't return error - we've handled it by deferring
                }
                Err(e) => return Err(e),
            }
        }

        // Release bytes from budget regardless of whether the file existed on disk.
        // If the file was externally deleted, we still need to release our accounting.
        if let (Some(budget), Some(size)) = (&self.budget, file_size) {
            budget.release(size);
        }

        Ok(())
    }

    /// Checks if an I/O error is a sharing violation (Windows-specific).
    ///
    /// On Windows, this occurs when trying to delete a file that's still open
    /// (e.g., memory-mapped by outstanding `BundleHandle`s).
    fn is_sharing_violation(error: &std::io::Error) -> bool {
        // Windows error code 32: ERROR_SHARING_VIOLATION
        // "The process cannot access the file because it is being used by another process."
        #[cfg(windows)]
        {
            error.raw_os_error() == Some(32)
        }
        #[cfg(not(windows))]
        {
            let _ = error;
            false
        }
    }

    /// Retries deletion of segments that previously failed due to sharing violations.
    ///
    /// Returns the number of segments successfully deleted.
    pub fn retry_pending_deletes(&self) -> usize {
        let pending: Vec<SegmentSeq> = {
            let pending = self.pending_deletes.lock();
            pending.iter().copied().collect()
        };

        if pending.is_empty() {
            return 0;
        }

        let mut deleted = 0;
        for seq in pending {
            let path = self.segment_path(seq);
            if !path.exists() {
                // File was deleted externally
                let _ = self.pending_deletes.lock().remove(&seq);
                deleted += 1;
                continue;
            }

            // Try to make writable and delete
            if let Ok(metadata) = path.metadata() {
                let mut perms = metadata.permissions();
                #[allow(clippy::permissions_set_readonly_false)]
                perms.set_readonly(false);
                let _ = std::fs::set_permissions(&path, perms);
            }

            match std::fs::remove_file(&path) {
                Ok(()) => {
                    let _ = self.pending_deletes.lock().remove(&seq);
                    tracing::debug!(
                        segment = seq.raw(),
                        "Successfully deleted previously deferred segment"
                    );
                    deleted += 1;
                }
                Err(e) if Self::is_sharing_violation(&e) => {
                    // Still in use, keep in pending list
                    tracing::trace!(
                        segment = seq.raw(),
                        "Segment file still in use, will retry later"
                    );
                }
                Err(e) => {
                    // Other error - remove from pending and log
                    let _ = self.pending_deletes.lock().remove(&seq);
                    tracing::warn!(
                        segment = seq.raw(),
                        error = %e,
                        "Failed to delete deferred segment"
                    );
                }
            }
        }

        deleted
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
    pub fn scan_existing(&self) -> Result<Vec<(SegmentSeq, u32)>> {
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
    pub fn scan_existing_with_max_age(
        &self,
        max_age: Option<Duration>,
    ) -> Result<Vec<(SegmentSeq, u32)>> {
        let mut found = Vec::new();
        let now = SystemTime::now();

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
                    if let Some(max_age) = max_age {
                        if Self::is_file_expired(&path, max_age, now) {
                            // Delete expired segment without loading it
                            if let Err(e) = Self::delete_segment_file(&path, &self.budget) {
                                tracing::warn!(
                                    path = %path.display(),
                                    error = %e,
                                    "failed to delete expired segment during scan"
                                );
                            } else {
                                tracing::info!(
                                    segment = seq.raw(),
                                    max_age_secs = max_age.as_secs(),
                                    "deleted expired segment during startup scan"
                                );
                            }
                            continue;
                        }
                    }

                    match self.register_segment(seq) {
                        Ok(bundle_count) => found.push((seq, bundle_count)),
                        Err(e) => {
                            // Use debug level since this is expected during concurrent cleanup
                            tracing::debug!(
                                path = %path.display(),
                                error = %e,
                                "failed to load segment, skipping"
                            );
                        }
                    }
                }
            }
        }

        // Sort by sequence number
        found.sort_by_key(|(seq, _)| *seq);

        Ok(found)
    }

    /// Checks if a file is older than `max_age` based on its modification time.
    ///
    /// Returns `true` if the file should be considered expired, or `false` if the
    /// metadata cannot be read (fail-safe: don't delete if we can't check).
    fn is_file_expired(path: &Path, max_age: Duration, now: SystemTime) -> bool {
        match std::fs::metadata(path) {
            Ok(metadata) => match metadata.modified() {
                Ok(mtime) => now.duration_since(mtime).is_ok_and(|age| age > max_age),
                Err(_) => false, // Can't determine age, don't expire
            },
            Err(_) => false, // Can't read metadata, don't expire
        }
    }

    /// Deletes a segment file from disk without loading it.
    ///
    /// Used for cleaning up expired segments during scan without the overhead
    /// of opening and parsing the segment.
    fn delete_segment_file(path: &Path, budget: &Option<Arc<DiskBudget>>) -> std::io::Result<()> {
        // Get file size for budget release before deleting
        let file_size = std::fs::metadata(path).map(|m| m.len()).ok();

        // Segment files are read-only after finalization, make writable first
        if let Ok(metadata) = std::fs::metadata(path) {
            let mut perms = metadata.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(path, perms);
        }

        std::fs::remove_file(path)?;

        // Release bytes from budget
        if let (Some(budget), Some(size)) = (budget, file_size) {
            budget.release(size);
        }

        Ok(())
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
        let segments = self.segments.read();

        segments
            .iter()
            .filter_map(|(seq, handle)| {
                // Calculate age since finalization
                let age = now.duration_since(handle.finalized_at).ok()?;
                if age > max_age { Some(*seq) } else { None }
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
    fn is_file_expired_checks_mtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        let now = SystemTime::now();

        // File just created, should not be expired with 1 hour max_age
        assert!(!SegmentStore::is_file_expired(
            &file_path,
            Duration::from_secs(3600),
            now
        ));

        // With a very short max_age and sleep, it should be expired
        std::thread::sleep(Duration::from_millis(50));
        let later = SystemTime::now();
        assert!(SegmentStore::is_file_expired(
            &file_path,
            Duration::from_millis(10),
            later
        ));
    }

    #[test]
    fn is_file_expired_nonexistent_file() {
        // Non-existent file should not be considered expired (returns false)
        let nonexistent = Path::new("/nonexistent/file.txt");
        assert!(!SegmentStore::is_file_expired(
            nonexistent,
            Duration::from_secs(1),
            SystemTime::now()
        ));
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
            let _ = pending.insert(SegmentSeq::new(1));
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
            let _ = pending.insert(seq);
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

        let err = std::io::Error::new(std::io::ErrorKind::Other, "generic error");
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
    fn delete_segment_file_helper() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.qseg");
        std::fs::write(&file_path, "test data").unwrap();
        assert!(file_path.exists());

        // Delete without budget
        SegmentStore::delete_segment_file(&file_path, &None).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn delete_segment_file_releases_budget() {
        use crate::budget::DiskBudget;
        use crate::config::RetentionPolicy;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.qseg");
        let file_content = "test data with some content";
        std::fs::write(&file_path, file_content).unwrap();
        let file_size = file_content.len() as u64;

        // Create a budget and record the file's usage
        let budget = Arc::new(DiskBudget::new(1024 * 1024, RetentionPolicy::Backpressure));
        budget.record_existing(file_size);
        let usage_before = budget.used();
        assert_eq!(usage_before, file_size);

        // Delete with budget tracking
        SegmentStore::delete_segment_file(&file_path, &Some(budget.clone())).unwrap();
        assert!(!file_path.exists());

        // Budget should have released the bytes
        assert_eq!(
            budget.used(),
            0,
            "budget should release file size after deletion"
        );
    }

    #[test]
    fn delete_segment_file_nonexistent_errors() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.qseg");

        // Deleting a non-existent file returns an error
        let result = SegmentStore::delete_segment_file(&file_path, &None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    /// Test that future mtime (clock skew) does not cause spurious expiration.
    ///
    /// If a file's mtime is in the future (e.g., due to clock skew during creation),
    /// duration_since() will fail, and we should treat the file as NOT expired.
    #[test]
    fn is_file_expired_future_mtime_not_expired() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        // Simulate a "now" that is before the file's actual mtime
        // (as if the file was created with a clock that was ahead)
        let past = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);

        // Even with a tiny max_age, the file should NOT be considered expired
        // because duration_since() will return Err when mtime > now
        assert!(
            !SegmentStore::is_file_expired(&file_path, Duration::from_secs(1), past),
            "file with mtime in the future should not be considered expired"
        );
    }
}
