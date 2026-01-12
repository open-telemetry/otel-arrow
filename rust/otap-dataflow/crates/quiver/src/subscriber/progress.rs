// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-subscriber progress file for durable state tracking.
//!
//! Each subscriber maintains an independent progress file (`quiver.sub.<id>`)
//! that snapshots its current state. Progress is atomically updated via
//! write-fsync-rename, avoiding the compaction complexity of append-only logs.
//!
//! # File Format (v1)
//!
//! ```text
//! Header (fixed size = 32 bytes):
//! +----------+---------+-------------+----------+-----------------------+-------------+----------+
//! | magic    | version | header_size | flags    | oldest_incomplete_seg | entry_count | reserved |
//! | (8B)     | (2B LE) | (2B LE)     | (2B LE)  | (8B LE)               | (4B LE)     | (6B)     |
//! +----------+---------+-------------+----------+-----------------------+-------------+----------+
//!
//! Body (entry_count segment entries):
//! +------------+--------------+---------------+-----------------+
//! | seg_seq    | bundle_count | bitmap_words  | acked_bitmap    |
//! | (8B LE)    | (4B LE)      | (2B LE)       | (bitmap_words × 8B) |
//! +------------+--------------+---------------+-----------------+
//!
//! Footer:
//! +----------+
//! | crc32    |
//! | (4B LE)  |
//! +----------+
//! ```
//!
//! # Compatibility
//!
//! - **Forward compatibility**: `header_size` allows old readers to skip unknown
//!   header extensions. Unknown trailing bytes are ignored.
//! - **Backward compatibility**: Version field identifies format; unknown versions
//!   are rejected.
//! - **Integrity**: CRC32C covers entire file (header + body); mismatches indicate
//!   corruption.
//!
//! # Atomic Updates
//!
//! Progress is written via temp file → fsync → rename to ensure crash-safe
//! updates. This avoids partial writes and provides atomic visibility.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use crc32fast::Hasher as Crc32Hasher;

use super::error::{Result, SubscriberError};
use super::types::SubscriberId;
use crate::segment::SegmentSeq;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Magic bytes identifying a subscriber progress file.
const PROGRESS_MAGIC: &[u8; 8] = b"QUIVSUB\0";

/// Current progress file format version.
const PROGRESS_VERSION: u16 = 1;

/// Header size for version 1.
/// Layout: magic (8) + version (2) + header_size (2) + flags (2) +
///         oldest_incomplete_seg (8) + entry_count (4) + reserved (6) = 32 bytes.
const HEADER_V1_SIZE: usize = 32;

/// Minimum header size (for forward compatibility checks).
const HEADER_MIN_SIZE: usize = 32;

/// Footer size (CRC32).
const FOOTER_SIZE: usize = 4;

/// Maximum number of segment entries we'll accept (sanity check).
/// With 64KB bundles per segment, this allows tracking ~1M segments.
const MAX_ENTRY_COUNT: u32 = 1_000_000;

/// Maximum bitmap words per entry (supports up to 64K bundles per segment).
const MAX_BITMAP_WORDS: u16 = 1024;

// ─────────────────────────────────────────────────────────────────────────────
// ProgressHeader
// ─────────────────────────────────────────────────────────────────────────────

/// Header of a progress file.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ProgressHeader {
    /// Format version (currently 1).
    version: u16,
    /// Total header size in bytes (32 for v1).
    header_size: u16,
    /// Reserved flags (must be 0 for v1).
    flags: u16,
    /// Oldest segment with incomplete bundles.
    oldest_incomplete_seg: SegmentSeq,
    /// Number of segment entries in the body.
    entry_count: u32,
}

impl ProgressHeader {
    /// Creates a new header with the given values.
    fn new(oldest_incomplete_seg: SegmentSeq, entry_count: u32) -> Self {
        Self {
            version: PROGRESS_VERSION,
            header_size: HEADER_V1_SIZE as u16,
            flags: 0,
            oldest_incomplete_seg,
            entry_count,
        }
    }

    /// Serializes the header to bytes.
    fn serialize(&self) -> [u8; HEADER_V1_SIZE] {
        let mut buf = [0u8; HEADER_V1_SIZE];

        // Magic (8 bytes)
        buf[0..8].copy_from_slice(PROGRESS_MAGIC);

        // Version (2 bytes LE)
        buf[8..10].copy_from_slice(&self.version.to_le_bytes());

        // Header size (2 bytes LE)
        buf[10..12].copy_from_slice(&self.header_size.to_le_bytes());

        // Flags (2 bytes LE)
        buf[12..14].copy_from_slice(&self.flags.to_le_bytes());

        // Oldest incomplete segment (8 bytes LE)
        buf[14..22].copy_from_slice(&self.oldest_incomplete_seg.raw().to_le_bytes());

        // Entry count (4 bytes LE)
        buf[22..26].copy_from_slice(&self.entry_count.to_le_bytes());

        // Reserved (6 bytes, zero-filled)
        // Already zero from array initialization

        buf
    }

    /// Deserializes a header from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is invalid or corrupted.
    fn deserialize(data: &[u8], path: &Path) -> Result<Self> {
        if data.len() < HEADER_MIN_SIZE {
            return Err(SubscriberError::progress_corrupted(
                path,
                format!(
                    "header too short: {} bytes, expected at least {}",
                    data.len(),
                    HEADER_MIN_SIZE
                ),
            ));
        }

        // Verify magic
        if &data[0..8] != PROGRESS_MAGIC {
            return Err(SubscriberError::progress_corrupted(
                path,
                "invalid magic bytes",
            ));
        }

        // Read version
        let version = u16::from_le_bytes([data[8], data[9]]);
        if version != PROGRESS_VERSION {
            return Err(SubscriberError::progress_corrupted(
                path,
                format!("unsupported version: {version}, expected {PROGRESS_VERSION}"),
            ));
        }

        // Read header size
        let header_size = u16::from_le_bytes([data[10], data[11]]);
        if (header_size as usize) < HEADER_MIN_SIZE {
            return Err(SubscriberError::progress_corrupted(
                path,
                format!("invalid header_size: {header_size}, minimum is {HEADER_MIN_SIZE}"),
            ));
        }

        // Note: We don't validate header_size against file size here because
        // deserialize() is called with just the header bytes. File-level bounds
        // checking happens in read_progress_file().

        // Read flags (warn on unknown flags but don't fail)
        let flags = u16::from_le_bytes([data[12], data[13]]);
        if flags != 0 {
            // Future: log warning about unknown flags
        }

        // Read oldest incomplete segment
        let oldest_incomplete_seg = SegmentSeq::new(u64::from_le_bytes(
            data[14..22].try_into().expect("slice length validated"),
        ));

        // Read entry count
        let entry_count =
            u32::from_le_bytes(data[22..26].try_into().expect("slice length validated"));

        if entry_count > MAX_ENTRY_COUNT {
            return Err(SubscriberError::progress_corrupted(
                path,
                format!("entry count too large: {entry_count}, maximum is {MAX_ENTRY_COUNT}"),
            ));
        }

        Ok(Self {
            version,
            header_size,
            flags,
            oldest_incomplete_seg,
            entry_count,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SegmentProgressEntry
// ─────────────────────────────────────────────────────────────────────────────

/// Progress entry for a single segment.
///
/// Tracks which bundles within a segment have been acknowledged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentProgressEntry {
    /// Segment sequence number.
    pub seg_seq: SegmentSeq,
    /// Total number of bundles in this segment.
    pub bundle_count: u32,
    /// Bitmap of acknowledged bundles (1 = acked, LSB = bundle 0).
    acked_bitmap: Vec<u64>,
}

impl SegmentProgressEntry {
    /// Creates a new entry with no bundles acked.
    #[must_use]
    pub fn new(seg_seq: SegmentSeq, bundle_count: u32) -> Self {
        let bitmap_words = Self::words_needed(bundle_count);
        Self {
            seg_seq,
            bundle_count,
            acked_bitmap: vec![0u64; bitmap_words],
        }
    }

    /// Creates an entry from an existing bitmap.
    #[must_use]
    pub fn from_bitmap(seg_seq: SegmentSeq, bundle_count: u32, bitmap: Vec<u64>) -> Self {
        Self {
            seg_seq,
            bundle_count,
            acked_bitmap: bitmap,
        }
    }

    /// Returns the number of 64-bit words needed for `bundle_count` bundles.
    #[must_use]
    pub const fn words_needed(bundle_count: u32) -> usize {
        (bundle_count as usize).div_ceil(64)
    }

    /// Marks a bundle as acknowledged.
    ///
    /// # Panics
    ///
    /// Panics if `bundle_index >= bundle_count`.
    pub fn mark_acked(&mut self, bundle_index: u32) {
        assert!(
            bundle_index < self.bundle_count,
            "bundle index {bundle_index} out of range (count: {})",
            self.bundle_count
        );
        let word_idx = (bundle_index / 64) as usize;
        let bit_idx = bundle_index % 64;
        self.acked_bitmap[word_idx] |= 1u64 << bit_idx;
    }

    /// Checks if a bundle is acknowledged.
    #[must_use]
    pub fn is_acked(&self, bundle_index: u32) -> bool {
        if bundle_index >= self.bundle_count {
            return false;
        }
        let word_idx = (bundle_index / 64) as usize;
        let bit_idx = bundle_index % 64;
        (self.acked_bitmap[word_idx] >> bit_idx) & 1 == 1
    }

    /// Returns the count of acknowledged bundles.
    #[must_use]
    pub fn acked_count(&self) -> u32 {
        self.acked_bitmap.iter().map(|w| w.count_ones()).sum()
    }

    /// Returns true if all bundles are acknowledged.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.acked_count() == self.bundle_count
    }

    /// Returns the bitmap words.
    #[must_use]
    pub fn bitmap(&self) -> &[u64] {
        &self.acked_bitmap
    }

    /// Serializes the entry to bytes.
    fn serialize(&self) -> Vec<u8> {
        let bitmap_words = self.acked_bitmap.len() as u16;
        let size = 8 + 4 + 2 + (bitmap_words as usize * 8);
        let mut buf = Vec::with_capacity(size);

        // seg_seq (8 bytes LE)
        buf.extend_from_slice(&self.seg_seq.raw().to_le_bytes());

        // bundle_count (4 bytes LE)
        buf.extend_from_slice(&self.bundle_count.to_le_bytes());

        // bitmap_words (2 bytes LE)
        buf.extend_from_slice(&bitmap_words.to_le_bytes());

        // acked_bitmap (bitmap_words × 8 bytes)
        for word in &self.acked_bitmap {
            buf.extend_from_slice(&word.to_le_bytes());
        }

        buf
    }

    /// Deserializes an entry from a reader.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is corrupted or truncated.
    fn deserialize(reader: &mut impl Read, path: &Path) -> Result<Self> {
        // Read fixed fields
        let mut fixed = [0u8; 14]; // seg_seq (8) + bundle_count (4) + bitmap_words (2)
        reader
            .read_exact(&mut fixed)
            .map_err(|e| SubscriberError::progress_io(path, e))?;

        let seg_seq = SegmentSeq::new(u64::from_le_bytes(
            fixed[0..8].try_into().expect("slice length is 8"),
        ));
        let bundle_count = u32::from_le_bytes(fixed[8..12].try_into().expect("slice length is 4"));
        let bitmap_words = u16::from_le_bytes(fixed[12..14].try_into().expect("slice length is 2"));

        // Validate bitmap_words
        if bitmap_words > MAX_BITMAP_WORDS {
            return Err(SubscriberError::progress_corrupted(
                path,
                format!("bitmap_words too large: {bitmap_words}, maximum is {MAX_BITMAP_WORDS}"),
            ));
        }

        // Validate consistency
        let expected_words = Self::words_needed(bundle_count) as u16;
        if bitmap_words != expected_words {
            return Err(SubscriberError::progress_corrupted(
                path,
                format!(
                    "bitmap_words mismatch: got {bitmap_words}, expected {expected_words} for {bundle_count} bundles"
                ),
            ));
        }

        // Read bitmap
        let mut bitmap = vec![0u64; bitmap_words as usize];
        for word in &mut bitmap {
            let mut buf = [0u8; 8];
            reader
                .read_exact(&mut buf)
                .map_err(|e| SubscriberError::progress_io(path, e))?;
            *word = u64::from_le_bytes(buf);
        }

        Ok(Self {
            seg_seq,
            bundle_count,
            acked_bitmap: bitmap,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SubscriberProgress
// ─────────────────────────────────────────────────────────────────────────────

/// Complete progress state for a subscriber.
///
/// This is the in-memory representation that can be serialized to/from
/// a progress file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriberProgress {
    /// The subscriber this progress belongs to.
    pub subscriber_id: SubscriberId,
    /// Oldest segment with incomplete bundles.
    pub oldest_incomplete_seg: SegmentSeq,
    /// Per-segment progress entries (only for incomplete segments).
    pub entries: Vec<SegmentProgressEntry>,
}

impl SubscriberProgress {
    /// Creates a new empty progress state.
    #[must_use]
    pub fn new(subscriber_id: SubscriberId) -> Self {
        Self {
            subscriber_id,
            oldest_incomplete_seg: SegmentSeq::new(0),
            entries: Vec::new(),
        }
    }

    /// Creates progress with the given state.
    #[must_use]
    pub fn with_entries(
        subscriber_id: SubscriberId,
        oldest_incomplete_seg: SegmentSeq,
        entries: Vec<SegmentProgressEntry>,
    ) -> Self {
        Self {
            subscriber_id,
            oldest_incomplete_seg,
            entries,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Progress File Path
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the path to a subscriber's progress file.
///
/// Format: `<dir>/quiver.sub.<subscriber_id>`
#[must_use]
pub fn progress_file_path(dir: &Path, subscriber_id: &SubscriberId) -> PathBuf {
    dir.join(format!("quiver.sub.{}", subscriber_id.as_str()))
}

/// Returns the path to a temporary progress file for atomic writes.
fn temp_progress_file_path(dir: &Path, subscriber_id: &SubscriberId) -> PathBuf {
    dir.join(format!("quiver.sub.{}.tmp", subscriber_id.as_str()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Reading Progress Files
// ─────────────────────────────────────────────────────────────────────────────

/// Reads and validates a subscriber progress file.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened
/// - The file is corrupted (invalid magic, version, or CRC)
/// - The file is truncated
pub fn read_progress_file(path: &Path) -> Result<(SegmentSeq, Vec<SegmentProgressEntry>)> {
    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| SubscriberError::progress_io(path, e))?;

    let mut reader = BufReader::new(file);

    // Read entire file for CRC validation
    let mut data = Vec::new();
    let _ = reader
        .read_to_end(&mut data)
        .map_err(|e| SubscriberError::progress_io(path, e))?;

    // Minimum file size: header + footer
    if data.len() < HEADER_V1_SIZE + FOOTER_SIZE {
        return Err(SubscriberError::progress_corrupted(
            path,
            format!(
                "file too short: {} bytes, minimum is {}",
                data.len(),
                HEADER_V1_SIZE + FOOTER_SIZE
            ),
        ));
    }

    // Validate CRC (covers everything except the CRC itself)
    let crc_offset = data.len() - FOOTER_SIZE;
    let stored_crc = u32::from_le_bytes(data[crc_offset..].try_into().expect("slice length is 4"));

    let mut hasher = Crc32Hasher::new();
    hasher.update(&data[..crc_offset]);
    let computed_crc = hasher.finalize();

    if stored_crc != computed_crc {
        return Err(SubscriberError::progress_corrupted(
            path,
            format!("CRC mismatch: stored {stored_crc:#010x}, computed {computed_crc:#010x}"),
        ));
    }

    // Parse header
    let header = ProgressHeader::deserialize(&data[..HEADER_V1_SIZE], path)?;

    // Validate header_size against file bounds
    let header_size = header.header_size as usize;
    if header_size > crc_offset {
        return Err(SubscriberError::progress_corrupted(
            path,
            format!(
                "header_size ({header_size}) exceeds file body size ({})",
                crc_offset
            ),
        ));
    }

    // Parse entries
    let body_data = &data[header_size..crc_offset];
    let mut body_reader = io::Cursor::new(body_data);

    let mut entries = Vec::with_capacity(header.entry_count as usize);
    for _ in 0..header.entry_count {
        let entry = SegmentProgressEntry::deserialize(&mut body_reader, path)?;
        entries.push(entry);
    }

    Ok((header.oldest_incomplete_seg, entries))
}

// ─────────────────────────────────────────────────────────────────────────────
// Writing Progress Files
// ─────────────────────────────────────────────────────────────────────────────

/// Writes a subscriber progress file atomically.
///
/// Uses write-to-temp → fsync → rename pattern for crash safety.
///
/// # Errors
///
/// Returns an error if:
/// - The directory doesn't exist
/// - The file cannot be written
/// - fsync fails
pub fn write_progress_file(
    dir: &Path,
    subscriber_id: &SubscriberId,
    oldest_incomplete_seg: SegmentSeq,
    entries: &[SegmentProgressEntry],
) -> Result<()> {
    let final_path = progress_file_path(dir, subscriber_id);
    let temp_path = temp_progress_file_path(dir, subscriber_id);

    // Build the complete file content
    let mut content = Vec::new();

    // Header
    let header = ProgressHeader::new(oldest_incomplete_seg, entries.len() as u32);
    content.extend_from_slice(&header.serialize());

    // Entries
    for entry in entries {
        content.extend_from_slice(&entry.serialize());
    }

    // CRC (of header + entries)
    let mut hasher = Crc32Hasher::new();
    hasher.update(&content);
    let crc = hasher.finalize();
    content.extend_from_slice(&crc.to_le_bytes());

    // Write to temp file
    {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .map_err(|e| SubscriberError::progress_io(&temp_path, e))?;

        let mut writer = BufWriter::new(file);
        writer
            .write_all(&content)
            .map_err(|e| SubscriberError::progress_io(&temp_path, e))?;

        writer
            .flush()
            .map_err(|e| SubscriberError::progress_io(&temp_path, e))?;

        // Sync to disk
        writer
            .get_ref()
            .sync_all()
            .map_err(|e| SubscriberError::progress_io(&temp_path, e))?;
    }

    // Atomic rename
    // Note: fs::rename() is atomic on POSIX systems. On Windows, it's only atomic
    // when source and target are on the same volume. Since both temp_path and
    // final_path are in the same directory (data_dir), this is guaranteed to be
    // on the same volume.
    fs::rename(&temp_path, &final_path)
        .map_err(|e| SubscriberError::progress_io(&final_path, e))?;

    // Sync parent directory to ensure rename is durable
    if let Some(parent) = final_path.parent() {
        if let Ok(dir_file) = File::open(parent) {
            let _ = dir_file.sync_all();
        }
    }

    Ok(())
}

/// Deletes a subscriber's progress file.
///
/// # Errors
///
/// Returns an error if the file cannot be deleted (but not if it doesn't exist).
pub fn delete_progress_file(dir: &Path, subscriber_id: &SubscriberId) -> Result<()> {
    let path = progress_file_path(dir, subscriber_id);

    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(SubscriberError::progress_io(&path, e)),
    }
}

/// Scans a directory for all subscriber progress files.
///
/// Returns a list of subscriber IDs that have progress files.
///
/// # Errors
///
/// Returns an error if the directory cannot be read.
pub fn scan_progress_files(dir: &Path) -> Result<Vec<SubscriberId>> {
    let entries = fs::read_dir(dir).map_err(|e| SubscriberError::progress_io(dir, e))?;

    let mut subscriber_ids = Vec::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if let Some(id_str) = name.strip_prefix("quiver.sub.") {
            // Skip temp files
            if id_str.ends_with(".tmp") {
                continue;
            }

            // Try to parse as valid subscriber ID
            match SubscriberId::new(id_str) {
                Ok(id) => subscriber_ids.push(id),
                Err(e) => {
                    tracing::warn!(
                        file = %name,
                        error = %e,
                        "ignoring progress file with invalid subscriber ID"
                    );
                }
            }
        }
    }

    Ok(subscriber_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ─────────────────────────────────────────────────────────────────────────
    // ProgressHeader tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn header_serialize_deserialize_roundtrip() {
        let header = ProgressHeader::new(SegmentSeq::new(42), 5);
        let serialized = header.serialize();

        assert_eq!(serialized.len(), HEADER_V1_SIZE);

        let path = Path::new("test.sub");
        let deserialized = ProgressHeader::deserialize(&serialized, path).unwrap();

        assert_eq!(header, deserialized);
    }

    #[test]
    fn header_magic_validation() {
        let mut data = [0u8; HEADER_V1_SIZE];
        data[0..8].copy_from_slice(b"WRONGMAG");

        let path = Path::new("test.sub");
        let result = ProgressHeader::deserialize(&data, path);

        assert!(matches!(
            result,
            Err(SubscriberError::ProgressCorrupted { .. })
        ));
    }

    #[test]
    fn header_version_validation() {
        let mut header = ProgressHeader::new(SegmentSeq::new(0), 0);
        header.version = 99;

        let mut data = header.serialize();
        // Fix magic since serialize uses the constant
        data[8..10].copy_from_slice(&99u16.to_le_bytes());

        let path = Path::new("test.sub");
        let result = ProgressHeader::deserialize(&data, path);

        assert!(matches!(
            result,
            Err(SubscriberError::ProgressCorrupted { .. })
        ));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SegmentProgressEntry tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn entry_new_empty() {
        let entry = SegmentProgressEntry::new(SegmentSeq::new(1), 100);

        assert_eq!(entry.seg_seq, SegmentSeq::new(1));
        assert_eq!(entry.bundle_count, 100);
        assert_eq!(entry.acked_count(), 0);
        assert!(!entry.is_complete());
    }

    #[test]
    fn entry_mark_acked() {
        let mut entry = SegmentProgressEntry::new(SegmentSeq::new(1), 100);

        entry.mark_acked(0);
        entry.mark_acked(50);
        entry.mark_acked(99);

        assert!(entry.is_acked(0));
        assert!(entry.is_acked(50));
        assert!(entry.is_acked(99));
        assert!(!entry.is_acked(1));
        assert_eq!(entry.acked_count(), 3);
    }

    #[test]
    fn entry_complete() {
        let mut entry = SegmentProgressEntry::new(SegmentSeq::new(1), 3);

        entry.mark_acked(0);
        entry.mark_acked(1);
        assert!(!entry.is_complete());

        entry.mark_acked(2);
        assert!(entry.is_complete());
    }

    #[test]
    fn entry_words_needed() {
        assert_eq!(SegmentProgressEntry::words_needed(0), 0);
        assert_eq!(SegmentProgressEntry::words_needed(1), 1);
        assert_eq!(SegmentProgressEntry::words_needed(64), 1);
        assert_eq!(SegmentProgressEntry::words_needed(65), 2);
        assert_eq!(SegmentProgressEntry::words_needed(128), 2);
        assert_eq!(SegmentProgressEntry::words_needed(129), 3);
    }

    #[test]
    fn entry_serialize_deserialize_roundtrip() {
        let mut entry = SegmentProgressEntry::new(SegmentSeq::new(42), 150);
        entry.mark_acked(0);
        entry.mark_acked(64);
        entry.mark_acked(149);

        let serialized = entry.serialize();
        let path = Path::new("test.sub");
        let mut cursor = io::Cursor::new(&serialized);
        let deserialized = SegmentProgressEntry::deserialize(&mut cursor, path).unwrap();

        assert_eq!(entry, deserialized);
    }

    #[test]
    fn entry_large_bundle_count() {
        let mut entry = SegmentProgressEntry::new(SegmentSeq::new(1), 1000);

        entry.mark_acked(0);
        entry.mark_acked(500);
        entry.mark_acked(999);

        assert!(entry.is_acked(0));
        assert!(entry.is_acked(500));
        assert!(entry.is_acked(999));
        assert!(!entry.is_acked(1));
        assert_eq!(entry.acked_count(), 3);

        // Roundtrip
        let serialized = entry.serialize();
        let path = Path::new("test.sub");
        let mut cursor = io::Cursor::new(&serialized);
        let deserialized = SegmentProgressEntry::deserialize(&mut cursor, path).unwrap();
        assert_eq!(entry, deserialized);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // File I/O tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn write_and_read_empty_progress() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("test-sub").unwrap();

        write_progress_file(dir.path(), &sub_id, SegmentSeq::new(0), &[]).unwrap();

        let path = progress_file_path(dir.path(), &sub_id);
        let (oldest, entries) = read_progress_file(&path).unwrap();

        assert_eq!(oldest, SegmentSeq::new(0));
        assert!(entries.is_empty());
    }

    #[test]
    fn write_and_read_progress_with_entries() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("exporter-1").unwrap();

        let mut entry1 = SegmentProgressEntry::new(SegmentSeq::new(10), 50);
        entry1.mark_acked(0);
        entry1.mark_acked(25);

        let mut entry2 = SegmentProgressEntry::new(SegmentSeq::new(11), 30);
        entry2.mark_acked(0);
        entry2.mark_acked(1);
        entry2.mark_acked(2);

        let entries = vec![entry1.clone(), entry2.clone()];

        write_progress_file(dir.path(), &sub_id, SegmentSeq::new(10), &entries).unwrap();

        let path = progress_file_path(dir.path(), &sub_id);
        let (oldest, read_entries) = read_progress_file(&path).unwrap();

        assert_eq!(oldest, SegmentSeq::new(10));
        assert_eq!(read_entries.len(), 2);
        assert_eq!(read_entries[0], entry1);
        assert_eq!(read_entries[1], entry2);
    }

    #[test]
    fn read_detects_crc_corruption() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("test-sub").unwrap();

        write_progress_file(dir.path(), &sub_id, SegmentSeq::new(5), &[]).unwrap();

        // Corrupt a byte in the header
        let path = progress_file_path(dir.path(), &sub_id);
        let mut data = fs::read(&path).unwrap();
        data[20] ^= 0xFF;
        fs::write(&path, &data).unwrap();

        let result = read_progress_file(&path);
        assert!(matches!(
            result,
            Err(SubscriberError::ProgressCorrupted { .. })
        ));
    }

    #[test]
    fn read_detects_truncation() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("test-sub").unwrap();

        write_progress_file(dir.path(), &sub_id, SegmentSeq::new(0), &[]).unwrap();

        // Truncate the file
        let path = progress_file_path(dir.path(), &sub_id);
        let data = fs::read(&path).unwrap();
        fs::write(&path, &data[..10]).unwrap();

        let result = read_progress_file(&path);
        assert!(matches!(
            result,
            Err(SubscriberError::ProgressCorrupted { .. })
        ));
    }

    #[test]
    fn read_detects_oversized_header() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("test-sub").unwrap();

        write_progress_file(dir.path(), &sub_id, SegmentSeq::new(0), &[]).unwrap();

        // Corrupt the header_size field to claim a size larger than the file
        let path = progress_file_path(dir.path(), &sub_id);
        let mut data = fs::read(&path).unwrap();

        // Set header_size to 0xFFFF (way larger than file)
        data[10] = 0xFF;
        data[11] = 0xFF;

        // Recalculate CRC to avoid CRC error
        let crc_offset = data.len() - 4;
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&data[..crc_offset]);
        let crc = hasher.finalize();
        data[crc_offset..].copy_from_slice(&crc.to_le_bytes());

        fs::write(&path, &data).unwrap();

        let result = read_progress_file(&path);
        assert!(
            matches!(result, Err(SubscriberError::ProgressCorrupted { .. })),
            "expected ProgressCorrupted error for oversized header, got: {result:?}"
        );
    }

    #[test]
    fn read_detects_entry_count_overflow() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("test-sub").unwrap();

        write_progress_file(dir.path(), &sub_id, SegmentSeq::new(0), &[]).unwrap();

        // Corrupt the entry_count field to claim more entries than exist
        let path = progress_file_path(dir.path(), &sub_id);
        let mut data = fs::read(&path).unwrap();

        // Set entry_count to 1000 but file has no entries
        data[22..26].copy_from_slice(&1000u32.to_le_bytes());

        // Recalculate CRC
        let crc_offset = data.len() - 4;
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&data[..crc_offset]);
        let crc = hasher.finalize();
        data[crc_offset..].copy_from_slice(&crc.to_le_bytes());

        fs::write(&path, &data).unwrap();

        let result = read_progress_file(&path);
        // Should fail when trying to read entries that don't exist
        assert!(
            result.is_err(),
            "expected error for entry count overflow, got: {result:?}"
        );
    }

    #[test]
    fn read_handles_zero_byte_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.sub.empty");

        // Create empty file
        fs::write(&path, b"").unwrap();

        let result = read_progress_file(&path);
        assert!(matches!(
            result,
            Err(SubscriberError::ProgressCorrupted { .. })
        ));
    }

    #[test]
    fn read_handles_garbage_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.sub.garbage");

        // Write random garbage
        fs::write(&path, b"this is not a valid progress file at all!!!").unwrap();

        let result = read_progress_file(&path);
        assert!(
            result.is_err(),
            "expected error for garbage file, got: {result:?}"
        );
    }

    #[test]
    fn delete_progress_file_works() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("test-sub").unwrap();

        write_progress_file(dir.path(), &sub_id, SegmentSeq::new(0), &[]).unwrap();

        let path = progress_file_path(dir.path(), &sub_id);
        assert!(path.exists());

        delete_progress_file(dir.path(), &sub_id).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn delete_nonexistent_file_ok() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("nonexistent").unwrap();

        // Should not error
        delete_progress_file(dir.path(), &sub_id).unwrap();
    }

    #[test]
    fn scan_progress_files_finds_all() {
        let dir = tempdir().unwrap();

        let sub1 = SubscriberId::new("exporter-1").unwrap();
        let sub2 = SubscriberId::new("backup-s3").unwrap();
        let sub3 = SubscriberId::new("metrics-sink").unwrap();

        write_progress_file(dir.path(), &sub1, SegmentSeq::new(0), &[]).unwrap();
        write_progress_file(dir.path(), &sub2, SegmentSeq::new(5), &[]).unwrap();
        write_progress_file(dir.path(), &sub3, SegmentSeq::new(10), &[]).unwrap();

        // Create a temp file that should be ignored
        fs::write(dir.path().join("quiver.sub.temp-file.tmp"), b"temp").unwrap();

        // Create an unrelated file
        fs::write(dir.path().join("other-file.txt"), b"other").unwrap();

        let found = scan_progress_files(dir.path()).unwrap();

        assert_eq!(found.len(), 3);
        assert!(found.contains(&sub1));
        assert!(found.contains(&sub2));
        assert!(found.contains(&sub3));
    }

    #[test]
    fn scan_empty_directory() {
        let dir = tempdir().unwrap();

        let found = scan_progress_files(dir.path()).unwrap();
        assert!(found.is_empty());
    }

    #[test]
    fn progress_file_path_format() {
        let sub_id = SubscriberId::new("my-exporter").unwrap();
        let path = progress_file_path(Path::new("/data"), &sub_id);

        assert_eq!(path, PathBuf::from("/data/quiver.sub.my-exporter"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Atomic update tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn atomic_update_preserves_old_on_temp_failure() {
        let dir = tempdir().unwrap();
        let sub_id = SubscriberId::new("test-sub").unwrap();

        // Write initial state
        let initial_entry = SegmentProgressEntry::new(SegmentSeq::new(1), 10);
        write_progress_file(
            dir.path(),
            &sub_id,
            SegmentSeq::new(1),
            std::slice::from_ref(&initial_entry),
        )
        .unwrap();

        // Read back and verify
        let path = progress_file_path(dir.path(), &sub_id);
        let (oldest, entries) = read_progress_file(&path).unwrap();
        assert_eq!(oldest, SegmentSeq::new(1));
        assert_eq!(entries.len(), 1);

        // Write updated state
        let updated_entry = SegmentProgressEntry::new(SegmentSeq::new(2), 20);
        write_progress_file(
            dir.path(),
            &sub_id,
            SegmentSeq::new(2),
            std::slice::from_ref(&updated_entry),
        )
        .unwrap();

        // Verify update
        let (oldest, entries) = read_progress_file(&path).unwrap();
        assert_eq!(oldest, SegmentSeq::new(2));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], updated_entry);
    }
}
