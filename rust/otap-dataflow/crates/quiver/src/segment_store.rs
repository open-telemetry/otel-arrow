// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Segment store for managing finalized segments.
//!
//! The [`SegmentStore`] tracks finalized segment files and provides read access
//! via memory mapping or standard file I/O. It implements [`SegmentProvider`]
//! for integration with the subscriber registry.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
}

impl SegmentHandle {
    /// Opens a segment file using the specified read mode.
    fn open(seq: SegmentSeq, path: PathBuf, mode: SegmentReadMode) -> Result<Self> {
        // Get file size before opening (for budget tracking)
        let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

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
    /// # Errors
    ///
    /// Returns an error if the file cannot be deleted (e.g., permissions).
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
            std::fs::remove_file(&path)?;

            // Release bytes from budget after successful deletion
            if let (Some(budget), Some(size)) = (&self.budget, file_size) {
                budget.release(size);
            }
        }
        Ok(())
    }

    /// Returns the path for a segment file.
    fn segment_path(&self, seq: SegmentSeq) -> PathBuf {
        self.segment_dir
            .join(format!("{}.qseg", seq.to_filename_component()))
    }

    /// Scans the segment directory and loads existing segments.
    ///
    /// Called during startup to discover segments from a previous run.
    ///
    /// # Errors
    ///
    /// Returns an error if directory scanning fails.
    pub fn scan_existing(&self) -> Result<Vec<(SegmentSeq, u32)>> {
        let mut found = Vec::new();

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
}
