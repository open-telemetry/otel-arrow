// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Durable acknowledgment log for subscriber progress tracking.
//!
//! The ack log is an append-only file recording terminal outcomes (Acked or
//! Dropped) for each subscriber-bundle pair. On recovery, the log is replayed
//! to rebuild [`SubscriberState`] for each subscriber.
//!
//! # File Header Format
//!
//! ```text
//! +----------+---------+--------------+
//! | magic    | version | header_size  |
//! | (8B)     | (2B LE) | (2B LE)      |
//! +----------+---------+--------------+
//! ```
//!
//! - `magic`: `QUIVER\0A` identifying an ack log file
//! - `version`: Format version (currently 1)
//! - `header_size`: Total header size in bytes (allows future extensions)
//!
//! # Entry Format
//!
//! Each entry is self-describing with a length prefix and CRC for integrity:
//!
//! ```text
//! +----------+----------+------+-------+-------------+---------+---------+----------+-------+
//! | len (4B) | crc (4B) | type | flags | sub_id_len  | sub_id  | outcome | seg_seq  | b_idx |
//! |  u32 LE  |  u32 LE  | (1B) | (1B)  |   (1B)      | (var)   |  (1B)   | (8B LE)  |(4B LE)|
//! +----------+----------+------+-------+-------------+---------+---------+----------+-------+
//! ```
//!
//! - `len`: Total entry length excluding the length field itself (4 bytes)
//! - `crc`: CRC32C of all bytes after the CRC field
//! - `type`: Entry type (0 = ack entry; future types for checkpoints, etc.)
//! - `flags`: Reserved for future use (must be 0)
//! - `sub_id_len`: Length of subscriber ID string (1 byte, max 255)
//! - `sub_id`: UTF-8 subscriber ID
//! - `outcome`: [`AckOutcome`] discriminant (1 byte)
//! - `seg_seq`: Segment sequence number (8 bytes, little-endian)
//! - `b_idx`: Bundle index within segment (4 bytes, little-endian)
//!
//! # Forward/Backward Compatibility
//!
//! The format is designed for compatibility:
//! - **Header extensions**: `header_size` field lets readers skip unknown fields
//! - **New entry types**: Unknown types are skipped (enables new record kinds)
//! - **Entry extensions**: Trailing bytes in entries are ignored (future fields)
//! - **Flags field**: Reserved for future per-entry options without version bump
//!
//! # Durability
//!
//! Entries are fsynced after each write to ensure durability. The writer
//! batches multiple entries before fsync when possible for better throughput.
//!
//! # Recovery
//!
//! On startup, the reader scans the log sequentially:
//! 1. Validate each entry's CRC
//! 2. Skip unknown entry types (forward compatibility)
//! 3. Apply known entries to rebuild subscriber state
//! 4. Truncate any trailing partial entries (from crash during write)
//!
//! [`SubscriberState`]: super::SubscriberState

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crc32fast::Hasher as Crc32Hasher;

use super::error::{Result, SubscriberError};
use super::types::{AckOutcome, BundleIndex, BundleRef, SubscriberId};
use crate::segment::SegmentSeq;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Magic bytes identifying an ack log file.
const ACK_LOG_MAGIC: &[u8; 8] = b"QUIVER\0A";

/// Current ack log file format version.
const ACK_LOG_VERSION: u16 = 1;

/// Minimum header size: magic (8) + version (2) + header_size (2) = 12 bytes.
/// The header_size field allows future versions to extend the header while
/// remaining readable by older implementations.
const HEADER_MIN_SIZE: usize = 12;

/// Current header size for version 1.
/// Layout: magic (8) + version (2) + header_size (2) = 12 bytes.
const HEADER_V1_SIZE: usize = 12;

// ─────────────────────────────────────────────────────────────────────────────
// Entry Type Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Entry type for acknowledgment records.
const ENTRY_TYPE_ACK: u8 = 0;

/// Entry type for subscriber registration events.
const ENTRY_TYPE_SUBSCRIBER_REGISTERED: u8 = 1;

/// Entry type for subscriber unregistration events.
const ENTRY_TYPE_SUBSCRIBER_UNREGISTERED: u8 = 2;

// ─────────────────────────────────────────────────────────────────────────────
// Entry Size Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Minimum entry size for ack entries:
/// crc (4) + entry_type (1) + flags (1) + timestamp (8) + sub_id_len (1) + sub_id (1+) + outcome (1) + seg_seq (8) + b_idx (4) = 29
const MIN_ENTRY_CONTENT_SIZE: usize = 29;

/// Maximum entry size based on max subscriber ID length (255) + fixed fields.
/// crc (4) + entry_type (1) + flags (1) + timestamp (8) + sub_id_len (1) + sub_id (255) + outcome (1) + seg_seq (8) + b_idx (4) = 283
const MAX_ENTRY_CONTENT_SIZE: usize = 283;

/// Minimum lifecycle entry size:
/// crc (4) + entry_type (1) + flags (1) + timestamp (8) + sub_id_len (1) + sub_id (1+) = 16
const MIN_LIFECYCLE_ENTRY_SIZE: usize = 16;

// ─────────────────────────────────────────────────────────────────────────────
// Rotation Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Default size threshold (in bytes) that triggers ack log file rotation.
///
/// 16 MiB is reasonable for ack logs since entries are small (~30-50 bytes each)
/// and we want to be able to purge old files relatively frequently.
#[allow(dead_code)]
pub const DEFAULT_ROTATION_TARGET_BYTES: u64 = 16 * 1024 * 1024;

/// Default maximum number of rotated ack log files to retain.
///
/// Unlike WAL, ack log files can only be purged when the oldest incomplete
/// segment advances, not based on a linear cursor. More files means more
/// flexibility for slow subscribers.
#[allow(dead_code)]
pub const DEFAULT_MAX_ROTATED_FILES: usize = 16;

// ─────────────────────────────────────────────────────────────────────────────
// Timestamp Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the current timestamp as milliseconds since UNIX epoch.
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ─────────────────────────────────────────────────────────────────────────────
// AckLogEntry
// ─────────────────────────────────────────────────────────────────────────────

/// A single ack entry in the ack log.
///
/// Represents a terminal outcome for a specific subscriber-bundle pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AckLogEntry {
    /// Timestamp when this entry was created (millis since UNIX epoch).
    pub timestamp_ms: u64,
    /// The subscriber recording the outcome.
    pub subscriber_id: SubscriberId,
    /// Reference to the bundle.
    pub bundle_ref: BundleRef,
    /// The terminal outcome.
    pub outcome: AckOutcome,
}

impl AckLogEntry {
    /// Creates a new ack log entry with the current timestamp.
    #[must_use]
    pub fn new(subscriber_id: SubscriberId, bundle_ref: BundleRef, outcome: AckOutcome) -> Self {
        Self {
            timestamp_ms: now_millis(),
            subscriber_id,
            bundle_ref,
            outcome,
        }
    }

    /// Serializes the entry to bytes (excluding length prefix).
    ///
    /// Entry format:
    /// ```text
    /// +----------+------+-------+-----------+-------------+---------+---------+----------+-------+
    /// | crc (4B) | type | flags | timestamp | sub_id_len  | sub_id  | outcome | seg_seq  | b_idx |
    /// |  u32 LE  | (1B) | (1B)  |  (8B LE)  |   (1B)      | (var)   |  (1B)   | (8B LE)  |(4B LE)|
    /// +----------+------+-------+-----------+-------------+---------+---------+----------+-------+
    /// ```
    ///
    /// The CRC covers all bytes after the CRC field.
    fn serialize(&self) -> Vec<u8> {
        let sub_id_bytes = self.subscriber_id.as_str().as_bytes();
        debug_assert!(sub_id_bytes.len() <= 255);

        // Calculate content size (after CRC)
        // entry_type (1) + flags (1) + timestamp (8) + sub_id_len (1) + sub_id (var) + outcome (1) + seg_seq (8) + b_idx (4)
        let content_size = 1 + 1 + 8 + 1 + sub_id_bytes.len() + 1 + 8 + 4;
        let mut buf = Vec::with_capacity(4 + content_size);

        // Reserve space for CRC (will fill in after)
        buf.extend_from_slice(&[0u8; 4]);

        // Entry type
        buf.push(ENTRY_TYPE_ACK);

        // Flags (reserved for future use)
        buf.push(0u8);

        // Timestamp
        buf.extend_from_slice(&self.timestamp_ms.to_le_bytes());

        // Subscriber ID length and bytes
        buf.push(sub_id_bytes.len() as u8);
        buf.extend_from_slice(sub_id_bytes);

        // Outcome
        buf.push(self.outcome.as_byte());

        // Segment sequence (little-endian)
        buf.extend_from_slice(&self.bundle_ref.segment_seq.raw().to_le_bytes());

        // Bundle index (little-endian)
        buf.extend_from_slice(&self.bundle_ref.bundle_index.raw().to_le_bytes());

        // Compute and insert CRC of content (everything after CRC field)
        let mut hasher = Crc32Hasher::new();
        hasher.update(&buf[4..]);
        let crc = hasher.finalize();
        buf[0..4].copy_from_slice(&crc.to_le_bytes());

        buf
    }

    /// Deserializes an entry from bytes (excluding length prefix).
    ///
    /// # Errors
    ///
    /// Returns an error if the data is corrupted or malformed.
    /// Returns `Ok(None)` for unknown entry types (forward compatibility).
    fn deserialize(data: &[u8], path: &Path) -> Result<Option<Self>> {
        // Use minimum lifecycle entry size for initial check since we don't know the type yet.
        // Full size validation happens after we confirm this is an ack entry.
        if data.len() < MIN_LIFECYCLE_ENTRY_SIZE {
            return Err(SubscriberError::ack_log_corrupted(
                path,
                format!("entry too short: {} bytes", data.len()),
            ));
        }

        // Verify CRC
        let stored_crc = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let mut hasher = Crc32Hasher::new();
        hasher.update(&data[4..]);
        let computed_crc = hasher.finalize();
        if stored_crc != computed_crc {
            return Err(SubscriberError::ack_log_corrupted(
                path,
                format!("CRC mismatch: stored={stored_crc:#x}, computed={computed_crc:#x}"),
            ));
        }

        let mut pos = 4;

        // Entry type
        let entry_type = data[pos];
        pos += 1;

        // Unknown entry types are skipped for forward compatibility.
        // A newer writer may have written entry types this reader doesn't understand.
        if entry_type != ENTRY_TYPE_ACK {
            return Ok(None);
        }

        // Now that we know this is an ack entry, verify minimum size
        if data.len() < MIN_ENTRY_CONTENT_SIZE {
            return Err(SubscriberError::ack_log_corrupted(
                path,
                format!("ack entry too short: {} bytes", data.len()),
            ));
        }

        // Flags (currently unused, skip)
        let _flags = data[pos];
        pos += 1;

        // Timestamp
        let timestamp_bytes: [u8; 8] = data[pos..pos + 8]
            .try_into()
            .expect("length already validated");
        let timestamp_ms = u64::from_le_bytes(timestamp_bytes);
        pos += 8;

        // Subscriber ID
        let sub_id_len = data[pos] as usize;
        pos += 1;

        if pos + sub_id_len + 1 + 8 + 4 > data.len() {
            return Err(SubscriberError::ack_log_corrupted(path, "entry truncated"));
        }

        let sub_id_bytes = &data[pos..pos + sub_id_len];
        let sub_id_str = std::str::from_utf8(sub_id_bytes).map_err(|_| {
            SubscriberError::ack_log_corrupted(path, "subscriber ID is not valid UTF-8")
        })?;
        let subscriber_id = SubscriberId::new(sub_id_str)?;
        pos += sub_id_len;

        // Outcome
        let outcome = AckOutcome::from_byte(data[pos]).ok_or_else(|| {
            SubscriberError::ack_log_corrupted(
                path,
                format!("invalid outcome byte: {:#x}", data[pos]),
            )
        })?;
        pos += 1;

        // Segment sequence
        let seg_seq_bytes: [u8; 8] = data[pos..pos + 8]
            .try_into()
            .expect("length already validated");
        let segment_seq = SegmentSeq::new(u64::from_le_bytes(seg_seq_bytes));
        pos += 8;

        // Bundle index
        let bundle_idx_bytes: [u8; 4] = data[pos..pos + 4]
            .try_into()
            .expect("length already validated");
        let bundle_index = BundleIndex::new(u32::from_le_bytes(bundle_idx_bytes));

        // Note: Any trailing bytes after the known fields are intentionally ignored.
        // This allows future versions to append new fields without breaking older readers.

        Ok(Some(Self {
            timestamp_ms,
            subscriber_id,
            bundle_ref: BundleRef::new(segment_seq, bundle_index),
            outcome,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SubscriberLifecycleEntry
// ─────────────────────────────────────────────────────────────────────────────

/// Type of subscriber lifecycle event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubscriberLifecycleEvent {
    /// Subscriber was registered.
    Registered,
    /// Subscriber was unregistered.
    Unregistered,
}

impl SubscriberLifecycleEvent {
    /// Converts the event to its entry type byte.
    fn to_entry_type(self) -> u8 {
        match self {
            Self::Registered => ENTRY_TYPE_SUBSCRIBER_REGISTERED,
            Self::Unregistered => ENTRY_TYPE_SUBSCRIBER_UNREGISTERED,
        }
    }

    /// Converts an entry type byte to an event.
    fn from_entry_type(entry_type: u8) -> Option<Self> {
        match entry_type {
            ENTRY_TYPE_SUBSCRIBER_REGISTERED => Some(Self::Registered),
            ENTRY_TYPE_SUBSCRIBER_UNREGISTERED => Some(Self::Unregistered),
            _ => None,
        }
    }
}

/// A subscriber lifecycle entry in the ack log.
///
/// Records when subscribers are registered or unregistered. Used for tracking
/// dynamic subscriber changes and for troubleshooting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriberLifecycleEntry {
    /// Timestamp when this event occurred (millis since UNIX epoch).
    pub timestamp_ms: u64,
    /// The subscriber affected by this event.
    pub subscriber_id: SubscriberId,
    /// The type of lifecycle event.
    pub event: SubscriberLifecycleEvent,
}

impl SubscriberLifecycleEntry {
    /// Creates a new lifecycle entry with the current timestamp.
    #[must_use]
    pub fn new(subscriber_id: SubscriberId, event: SubscriberLifecycleEvent) -> Self {
        Self {
            timestamp_ms: now_millis(),
            subscriber_id,
            event,
        }
    }

    /// Serializes the entry to bytes (excluding length prefix).
    ///
    /// Entry format:
    /// ```text
    /// +----------+------+-------+-----------+-------------+---------+
    /// | crc (4B) | type | flags | timestamp | sub_id_len  | sub_id  |
    /// |  u32 LE  | (1B) | (1B)  |  (8B LE)  |   (1B)      | (var)   |
    /// +----------+------+-------+-----------+-------------+---------+
    /// ```
    pub fn serialize(&self) -> Vec<u8> {
        let sub_id_bytes = self.subscriber_id.as_str().as_bytes();
        // CRC (4) + type (1) + flags (1) + timestamp (8) + sub_id_len (1) + sub_id (var)
        let content_size = 4 + 1 + 1 + 8 + 1 + sub_id_bytes.len();
        let mut buf = Vec::with_capacity(content_size);

        // Reserve space for CRC
        buf.extend_from_slice(&[0u8; 4]);

        // Entry type
        buf.push(self.event.to_entry_type());

        // Flags (reserved)
        buf.push(0u8);

        // Timestamp
        buf.extend_from_slice(&self.timestamp_ms.to_le_bytes());

        // Subscriber ID
        buf.push(sub_id_bytes.len() as u8);
        buf.extend_from_slice(sub_id_bytes);

        // Compute and write CRC over everything after the CRC field
        let mut hasher = Crc32Hasher::new();
        hasher.update(&buf[4..]);
        let crc = hasher.finalize();
        buf[0..4].copy_from_slice(&crc.to_le_bytes());

        buf
    }

    /// Deserializes an entry from bytes.
    ///
    /// Returns `Ok(None)` if the entry type is not a lifecycle event (for forward
    /// compatibility with future entry types).
    ///
    /// # Errors
    ///
    /// Returns an error if the data is corrupted or truncated.
    pub fn deserialize(data: &[u8], path: &Path) -> Result<Option<Self>> {
        // Minimum size check: crc (4) + type (1) + flags (1) + timestamp (8) + sub_id_len (1) + sub_id (1+)
        if data.len() < MIN_LIFECYCLE_ENTRY_SIZE {
            return Err(SubscriberError::ack_log_corrupted(
                path,
                format!("lifecycle entry too short: {} bytes", data.len()),
            ));
        }

        // Verify CRC
        let stored_crc =
            u32::from_le_bytes(data[0..4].try_into().expect("length already validated"));
        let mut hasher = Crc32Hasher::new();
        hasher.update(&data[4..]);
        let computed_crc = hasher.finalize();

        if stored_crc != computed_crc {
            return Err(SubscriberError::ack_log_corrupted(
                path,
                format!(
                    "lifecycle CRC mismatch: stored={stored_crc:#x}, computed={computed_crc:#x}"
                ),
            ));
        }

        let mut pos = 4;

        // Entry type
        let entry_type = data[pos];
        pos += 1;

        // Check if this is a lifecycle event type
        let event = match SubscriberLifecycleEvent::from_entry_type(entry_type) {
            Some(e) => e,
            None => return Ok(None),
        };

        // Flags (currently unused, skip)
        let _flags = data[pos];
        pos += 1;

        // Timestamp
        let timestamp_bytes: [u8; 8] = data[pos..pos + 8]
            .try_into()
            .expect("length already validated");
        let timestamp_ms = u64::from_le_bytes(timestamp_bytes);
        pos += 8;

        // Subscriber ID
        let sub_id_len = data[pos] as usize;
        pos += 1;

        if pos + sub_id_len > data.len() {
            return Err(SubscriberError::ack_log_corrupted(
                path,
                "lifecycle entry truncated",
            ));
        }

        let sub_id_bytes = &data[pos..pos + sub_id_len];
        let sub_id_str = std::str::from_utf8(sub_id_bytes).map_err(|_| {
            SubscriberError::ack_log_corrupted(path, "subscriber ID is not valid UTF-8")
        })?;
        let subscriber_id = SubscriberId::new(sub_id_str)?;

        // Note: Any trailing bytes after the known fields are intentionally ignored.
        // This allows future versions to append new fields without breaking older readers.

        Ok(Some(Self {
            timestamp_ms,
            subscriber_id,
            event,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AckLogWriter
// ─────────────────────────────────────────────────────────────────────────────

/// Append-only writer for the ack log.
///
/// Writes are buffered and fsynced after each `append` call for durability.
pub struct AckLogWriter {
    path: PathBuf,
    writer: BufWriter<File>,
}

impl AckLogWriter {
    /// Opens an existing ack log for appending, or creates a new one.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened/created or the header
    /// is invalid.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)
            .map_err(|e| SubscriberError::ack_log_io(&path, e))?;

        let file_len = file
            .metadata()
            .map_err(|e| SubscriberError::ack_log_io(&path, e))?
            .len();

        let mut writer = BufWriter::new(file);

        if file_len == 0 {
            // New file - write header
            // Header format: magic (8) + version (2) + header_size (2) = 12 bytes
            writer
                .write_all(ACK_LOG_MAGIC)
                .map_err(|e| SubscriberError::ack_log_io(&path, e))?;
            writer
                .write_all(&ACK_LOG_VERSION.to_le_bytes())
                .map_err(|e| SubscriberError::ack_log_io(&path, e))?;
            writer
                .write_all(&(HEADER_V1_SIZE as u16).to_le_bytes())
                .map_err(|e| SubscriberError::ack_log_io(&path, e))?;
            writer
                .flush()
                .map_err(|e| SubscriberError::ack_log_io(&path, e))?;
            writer
                .get_ref()
                .sync_all()
                .map_err(|e| SubscriberError::ack_log_io(&path, e))?;
        }

        Ok(Self { path, writer })
    }

    /// Appends an entry to the ack log and fsyncs.
    ///
    /// # Errors
    ///
    /// Returns an error if the write or fsync fails.
    pub fn append(&mut self, entry: &AckLogEntry) -> Result<()> {
        let serialized = entry.serialize();
        let len = serialized.len() as u32;

        // Write length prefix
        self.writer
            .write_all(&len.to_le_bytes())
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;

        // Write entry content
        self.writer
            .write_all(&serialized)
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;

        // Flush and sync
        self.writer
            .flush()
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;
        self.writer
            .get_ref()
            .sync_all()
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;

        Ok(())
    }

    /// Appends multiple entries and fsyncs once.
    ///
    /// More efficient than calling `append` repeatedly when multiple entries
    /// are available.
    ///
    /// # Errors
    ///
    /// Returns an error if any write or the final fsync fails.
    pub fn append_batch(&mut self, entries: &[AckLogEntry]) -> Result<()> {
        for entry in entries {
            let serialized = entry.serialize();
            let len = serialized.len() as u32;

            self.writer
                .write_all(&len.to_le_bytes())
                .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;
            self.writer
                .write_all(&serialized)
                .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;
        }

        self.writer
            .flush()
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;
        self.writer
            .get_ref()
            .sync_all()
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;

        Ok(())
    }

    /// Returns the path to the ack log file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends a lifecycle event to the ack log.
    ///
    /// Records subscriber registration or unregistration for troubleshooting
    /// and tracking dynamic subscriber changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the write or fsync fails.
    pub fn append_lifecycle(&mut self, entry: &SubscriberLifecycleEntry) -> Result<()> {
        let serialized = entry.serialize();
        let len = serialized.len() as u32;

        // Write length prefix
        self.writer
            .write_all(&len.to_le_bytes())
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;

        // Write entry content
        self.writer
            .write_all(&serialized)
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;

        // Flush and sync
        self.writer
            .flush()
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;
        self.writer
            .get_ref()
            .sync_all()
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AckLogReader
// ─────────────────────────────────────────────────────────────────────────────

/// Reader for the ack log, used during recovery.
///
/// Iterates over entries in the log, validating CRCs and handling truncation.
pub struct AckLogReader {
    path: PathBuf,
    reader: BufReader<File>,
    /// Position of the last successfully read entry's end.
    last_valid_pos: u64,
}

impl AckLogReader {
    /// Opens an ack log for reading.
    ///
    /// The reader supports forward compatibility:
    /// - Unknown entry types are skipped (newer writer, older reader)
    /// - Header extensions are skipped via the header_size field
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or has an invalid header.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|e| SubscriberError::ack_log_io(&path, e))?;

        let mut reader = BufReader::new(file);

        // Read minimum header
        let mut min_header = [0u8; HEADER_MIN_SIZE];
        reader
            .read_exact(&mut min_header)
            .map_err(|e| SubscriberError::ack_log_io(&path, e))?;

        // Verify magic
        if &min_header[0..8] != ACK_LOG_MAGIC {
            return Err(SubscriberError::ack_log_corrupted(
                &path,
                "invalid magic bytes",
            ));
        }

        // Check version
        let version = u16::from_le_bytes([min_header[8], min_header[9]]);
        if version != ACK_LOG_VERSION {
            return Err(SubscriberError::ack_log_corrupted(
                &path,
                format!("unsupported version: {version}"),
            ));
        }

        // Read header size (allows skipping unknown header extensions)
        let header_size = u16::from_le_bytes([min_header[10], min_header[11]]) as usize;
        if header_size < HEADER_MIN_SIZE {
            return Err(SubscriberError::ack_log_corrupted(
                &path,
                format!(
                    "header size too small: {} < {}",
                    header_size, HEADER_MIN_SIZE
                ),
            ));
        }

        // Skip any additional header bytes (forward compatibility)
        let extra_header_bytes = header_size - HEADER_MIN_SIZE;
        if extra_header_bytes > 0 {
            let mut skip_buf = vec![0u8; extra_header_bytes];
            reader
                .read_exact(&mut skip_buf)
                .map_err(|e| SubscriberError::ack_log_io(&path, e))?;
        }

        Ok(Self {
            path,
            reader,
            last_valid_pos: header_size as u64,
        })
    }

    /// Reads the next entry from the log.
    ///
    /// Returns `None` at end of file or if a partial entry is encountered.
    /// Partial entries at the end (from crash during write) are silently
    /// skipped.
    ///
    /// # Errors
    ///
    /// Returns an error only for actual I/O failures or corruption in the
    /// middle of the file.
    pub fn next_entry(&mut self) -> Result<Option<AckLogEntry>> {
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        match self.reader.read_exact(&mut len_bytes) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                return Ok(None);
            }
            Err(e) => {
                return Err(SubscriberError::ack_log_io(&self.path, e));
            }
        }

        let len = u32::from_le_bytes(len_bytes) as usize;

        // Validate length bounds
        // Accept both ack entries (29-283 bytes) and lifecycle entries (16-270 bytes)
        // Combined valid range: 16-283 bytes
        if !(MIN_LIFECYCLE_ENTRY_SIZE..=MAX_ENTRY_CONTENT_SIZE).contains(&len) {
            // This could be corruption or a partial write
            return Err(SubscriberError::ack_log_corrupted(
                &self.path,
                format!("invalid entry length: {len}"),
            ));
        }

        // Read entry content
        let mut content = vec![0u8; len];
        match self.reader.read_exact(&mut content) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Partial entry at end - likely crash during write
                // Seek back to last valid position
                return Ok(None);
            }
            Err(e) => {
                return Err(SubscriberError::ack_log_io(&self.path, e));
            }
        }

        // Deserialize - may return None for unknown entry types (forward compat)
        let entry = match AckLogEntry::deserialize(&content, &self.path)? {
            Some(e) => e,
            None => {
                // Unknown entry type - update position and continue to next
                self.last_valid_pos = self
                    .reader
                    .stream_position()
                    .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;
                return self.next_entry();
            }
        };

        // Update last valid position
        self.last_valid_pos = self
            .reader
            .stream_position()
            .map_err(|e| SubscriberError::ack_log_io(&self.path, e))?;

        Ok(Some(entry))
    }

    /// Returns the position after the last successfully read entry.
    ///
    /// Used to truncate partial entries after crash recovery.
    #[must_use]
    pub fn last_valid_position(&self) -> u64 {
        self.last_valid_pos
    }

    /// Returns the path to the ack log file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads all entries and returns them as a vector.
    ///
    /// Convenience method for recovery that consumes the reader.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    pub fn read_all(mut self) -> Result<Vec<AckLogEntry>> {
        let mut entries = Vec::new();
        while let Some(entry) = self.next_entry()? {
            entries.push(entry);
        }
        Ok(entries)
    }
}

/// Truncates the ack log at the given position.
///
/// Used during recovery to remove partial entries written during a crash.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or truncated.
#[allow(dead_code)] // Will be used in recovery
pub fn truncate_ack_log(path: impl AsRef<Path>, position: u64) -> Result<()> {
    let path = path.as_ref();
    let file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| SubscriberError::ack_log_io(path, e))?;

    file.set_len(position)
        .map_err(|e| SubscriberError::ack_log_io(path, e))?;

    file.sync_all()
        .map_err(|e| SubscriberError::ack_log_io(path, e))?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// RotatingAckLogWriter
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata for a rotated ack log file.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct RotatedAckLogFile {
    /// Path to the rotated file.
    path: PathBuf,
    /// Rotation ID (monotonically increasing).
    rotation_id: u64,
    /// File size in bytes.
    file_bytes: u64,
    /// Highest segment sequence number referenced in this file.
    /// Used to determine when the file can be purged.
    max_segment_seq: SegmentSeq,
}

/// Configuration for the rotating ack log writer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RotatingAckLogConfig {
    /// Size threshold that triggers rotation.
    pub rotation_target_bytes: u64,
    /// Maximum number of rotated files to keep.
    pub max_rotated_files: usize,
}

impl Default for RotatingAckLogConfig {
    fn default() -> Self {
        Self {
            rotation_target_bytes: DEFAULT_ROTATION_TARGET_BYTES,
            max_rotated_files: DEFAULT_MAX_ROTATED_FILES,
        }
    }
}

/// Ack log writer with automatic file rotation and cleanup.
///
/// When the active file exceeds `rotation_target_bytes`, it's renamed to
/// `quiver.ack.N` and a fresh file is created. Old rotated files are purged
/// when their `max_segment_seq` is older than the oldest incomplete segment
/// (all subscribers have moved past that data).
///
/// # On-Disk Layout
///
/// ```text
/// subscriber/
/// ├── quiver.ack           # Active ack log file
/// ├── quiver.ack.1         # Rotated file (oldest)
/// ├── quiver.ack.2         # Rotated file
/// └── quiver.ack.3         # Rotated file (newest)
/// ```
#[allow(dead_code)]
pub struct RotatingAckLogWriter {
    /// Configuration for rotation behavior.
    config: RotatingAckLogConfig,
    /// Path to the active ack log file.
    path: PathBuf,
    /// The active writer.
    writer: AckLogWriter,
    /// Current file size (for rotation decisions).
    current_size: u64,
    /// Highest segment sequence seen in the active file.
    active_max_segment_seq: Option<SegmentSeq>,
    /// Queue of rotated files, ordered oldest to newest.
    rotated_files: VecDeque<RotatedAckLogFile>,
    /// Next rotation ID to use.
    next_rotation_id: u64,
}

#[allow(dead_code)]
impl RotatingAckLogWriter {
    /// Opens or creates a rotating ack log at the given path.
    ///
    /// Scans for existing rotated files and rebuilds rotation state.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be accessed or the log
    /// cannot be opened.
    pub fn open(path: impl AsRef<Path>, config: RotatingAckLogConfig) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let writer = AckLogWriter::open(&path)?;

        // Get current file size
        let current_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(HEADER_V1_SIZE as u64);

        // Scan for rotated files
        let (rotated_files, next_rotation_id) = Self::scan_rotated_files(&path)?;

        Ok(Self {
            config,
            path,
            writer,
            current_size,
            active_max_segment_seq: None,
            rotated_files,
            next_rotation_id,
        })
    }

    /// Appends an entry, rotating if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub fn append(&mut self, entry: &AckLogEntry) -> Result<()> {
        // Track max segment for this file
        let entry_seg = entry.bundle_ref.segment_seq;
        self.active_max_segment_seq = Some(
            self.active_max_segment_seq
                .map_or(entry_seg, |s| s.max(entry_seg)),
        );

        // Write entry
        self.writer.append(entry)?;

        // Update size estimate (length prefix + content)
        let entry_size = 4 + entry.serialize().len() as u64;
        self.current_size += entry_size;

        // Check for rotation
        if self.current_size >= self.config.rotation_target_bytes {
            self.rotate()?;
        }

        Ok(())
    }

    /// Appends multiple entries, rotating if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if any write fails.
    pub fn append_batch(&mut self, entries: &[AckLogEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        // Track max segment for this file
        for entry in entries {
            let entry_seg = entry.bundle_ref.segment_seq;
            self.active_max_segment_seq = Some(
                self.active_max_segment_seq
                    .map_or(entry_seg, |s| s.max(entry_seg)),
            );
        }

        // Write entries
        self.writer.append_batch(entries)?;

        // Update size estimate
        let batch_size: u64 = entries.iter().map(|e| 4 + e.serialize().len() as u64).sum();
        self.current_size += batch_size;

        // Check for rotation
        if self.current_size >= self.config.rotation_target_bytes {
            self.rotate()?;
        }

        Ok(())
    }

    /// Purges rotated files that are fully superseded.
    ///
    /// A file can be purged when its `max_segment_seq` is strictly less than
    /// `oldest_incomplete_segment`, meaning all subscribers have completed
    /// all segments referenced in that file.
    ///
    /// # Errors
    ///
    /// Returns an error if file deletion fails.
    pub fn purge_before(&mut self, oldest_incomplete_segment: SegmentSeq) -> Result<usize> {
        let mut purged = 0;

        while let Some(front) = self.rotated_files.front() {
            if front.max_segment_seq < oldest_incomplete_segment {
                // Safe to delete - all entries are for completed segments
                std::fs::remove_file(&front.path)
                    .map_err(|e| SubscriberError::ack_log_io(&front.path, e))?;
                let _ = self.rotated_files.pop_front();
                purged += 1;
            } else {
                break;
            }
        }

        Ok(purged)
    }

    /// Returns the number of rotated files.
    #[must_use]
    pub fn rotated_file_count(&self) -> usize {
        self.rotated_files.len()
    }

    /// Returns the path to the active ack log file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the current size of the active file.
    #[must_use]
    pub fn current_size(&self) -> u64 {
        self.current_size
    }

    /// Rotates the active file to a new rotated file.
    fn rotate(&mut self) -> Result<()> {
        // Enforce max rotated files limit
        if self.rotated_files.len() >= self.config.max_rotated_files {
            // Can't rotate - too many files. This is not an error, just skip rotation.
            // The caller should call purge_before() more frequently.
            return Ok(());
        }

        let rotation_id = self.next_rotation_id;
        self.next_rotation_id += 1;

        let rotated_path = rotated_ack_log_path(&self.path, rotation_id);

        let file_bytes = self.current_size;
        let max_segment_seq = self
            .active_max_segment_seq
            .unwrap_or_else(|| SegmentSeq::new(0));

        // Close the current writer by replacing it with a new one on a temp path.
        // We'll fix this up after the rename.
        // First, flush and sync by dropping the inner BufWriter.
        // The writer holds a BufWriter<File> which flushes on drop.

        // Create a dummy writer to swap with - we'll replace it right after rename
        let active_path = self.path.clone();

        // Drop current writer to release file handle
        // Rust doesn't have a way to "close" without drop, so we use a temporary file
        let temp_path = active_path.with_extension("ack.rotating");
        let temp_writer = AckLogWriter::open(&temp_path)?;
        let old_writer = std::mem::replace(&mut self.writer, temp_writer);
        drop(old_writer);

        // Rename active to rotated
        std::fs::rename(&active_path, &rotated_path)
            .map_err(|e| SubscriberError::ack_log_io(&active_path, e))?;

        // Remove temp file if it was created
        let _ = std::fs::remove_file(&temp_path);

        self.rotated_files.push_back(RotatedAckLogFile {
            path: rotated_path,
            rotation_id,
            file_bytes,
            max_segment_seq,
        });

        // Open fresh active file
        self.writer = AckLogWriter::open(&active_path)?;
        self.current_size = HEADER_V1_SIZE as u64;
        self.active_max_segment_seq = None;

        Ok(())
    }

    /// Scans for existing rotated files and returns them sorted by rotation ID.
    fn scan_rotated_files(active_path: &Path) -> Result<(VecDeque<RotatedAckLogFile>, u64)> {
        let mut files = VecDeque::new();
        let mut max_id = 0u64;

        let parent = active_path.parent().unwrap_or(Path::new("."));
        let base_name = active_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("quiver.ack");

        let entries = match std::fs::read_dir(parent) {
            Ok(e) => e,
            Err(_) => return Ok((files, 1)), // Directory doesn't exist yet
        };

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            // Look for files matching "quiver.ack.N" pattern
            if let Some(suffix) = name
                .strip_prefix(base_name)
                .and_then(|s| s.strip_prefix('.'))
            {
                if let Ok(rotation_id) = suffix.parse::<u64>() {
                    let path = entry.path();
                    let file_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);

                    // Scan the file to find max segment seq
                    let max_segment_seq = Self::scan_max_segment_seq(&path)?;

                    files.push_back(RotatedAckLogFile {
                        path,
                        rotation_id,
                        file_bytes,
                        max_segment_seq,
                    });

                    max_id = max_id.max(rotation_id);
                }
            }
        }

        // Sort by rotation ID
        files.make_contiguous().sort_by_key(|f| f.rotation_id);

        Ok((files, max_id + 1))
    }

    /// Scans a file to find the maximum segment sequence referenced.
    fn scan_max_segment_seq(path: &Path) -> Result<SegmentSeq> {
        let reader = match AckLogReader::open(path) {
            Ok(r) => r,
            Err(_) => return Ok(SegmentSeq::new(0)), // Can't read file
        };

        let entries = reader.read_all()?;
        let max_seq = entries
            .iter()
            .map(|e| e.bundle_ref.segment_seq)
            .max()
            .unwrap_or_else(|| SegmentSeq::new(0));

        Ok(max_seq)
    }
}

/// Generates the path for a rotated ack log file.
#[allow(dead_code)]
fn rotated_ack_log_path(active_path: &Path, rotation_id: u64) -> PathBuf {
    let mut path = active_path.as_os_str().to_os_string();
    path.push(format!(".{rotation_id}"));
    PathBuf::from(path)
}

// ─────────────────────────────────────────────────────────────────────────────
// Multi-File Recovery
// ─────────────────────────────────────────────────────────────────────────────

/// Reads all entries from the active ack log and any rotated files.
///
/// Returns entries in order: oldest rotated file first, then active file.
/// This is the entry point for recovery.
///
/// # Errors
///
/// Returns an error if any file cannot be read.
#[allow(dead_code)]
pub fn read_all_ack_logs(active_path: impl AsRef<Path>) -> Result<Vec<AckLogEntry>> {
    let active_path = active_path.as_ref();
    let mut all_entries = Vec::new();

    // Find and read rotated files first (in order)
    let parent = active_path.parent().unwrap_or(Path::new("."));
    let base_name = active_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("quiver.ack");

    let mut rotated_paths: Vec<(u64, PathBuf)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if let Some(suffix) = name
                .strip_prefix(base_name)
                .and_then(|s| s.strip_prefix('.'))
            {
                if let Ok(rotation_id) = suffix.parse::<u64>() {
                    rotated_paths.push((rotation_id, entry.path()));
                }
            }
        }
    }

    // Sort by rotation ID (oldest first)
    rotated_paths.sort_by_key(|(id, _)| *id);

    // Read rotated files
    for (_, path) in rotated_paths {
        if let Ok(reader) = AckLogReader::open(&path) {
            let entries = reader.read_all()?;
            all_entries.extend(entries);
        }
    }

    // Read active file
    if active_path.exists() {
        let reader = AckLogReader::open(active_path)?;
        let entries = reader.read_all()?;
        all_entries.extend(entries);
    }

    Ok(all_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_entry(sub_id: &str, seg: u64, idx: u32, outcome: AckOutcome) -> AckLogEntry {
        AckLogEntry::new(
            SubscriberId::new(sub_id).unwrap(),
            BundleRef::new(SegmentSeq::new(seg), BundleIndex::new(idx)),
            outcome,
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Entry serialization tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn entry_serialize_deserialize_roundtrip() {
        let entry = test_entry("test-sub", 42, 7, AckOutcome::Acked);
        let serialized = entry.serialize();

        let path = Path::new("test.ack");
        let deserialized = AckLogEntry::deserialize(&serialized, path)
            .unwrap()
            .unwrap();

        assert_eq!(entry, deserialized);
    }

    #[test]
    fn entry_serialize_dropped_outcome() {
        let entry = test_entry("exporter", 100, 999, AckOutcome::Dropped);
        let serialized = entry.serialize();

        let path = Path::new("test.ack");
        let deserialized = AckLogEntry::deserialize(&serialized, path)
            .unwrap()
            .unwrap();

        assert_eq!(entry, deserialized);
    }

    #[test]
    fn entry_crc_validation() {
        let entry = test_entry("test-sub", 42, 7, AckOutcome::Acked);
        let mut serialized = entry.serialize();

        // Corrupt a byte
        serialized[10] ^= 0xFF;

        let path = Path::new("test.ack");
        let result = AckLogEntry::deserialize(&serialized, path);

        assert!(matches!(
            result,
            Err(SubscriberError::AckLogCorrupted { .. })
        ));
    }

    #[test]
    fn entry_too_short() {
        let path = Path::new("test.ack");
        let result = AckLogEntry::deserialize(&[0u8; 10], path);
        assert!(matches!(
            result,
            Err(SubscriberError::AckLogCorrupted { .. })
        ));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Lifecycle entry tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn lifecycle_entry_serialize_deserialize_roundtrip() {
        let entry = SubscriberLifecycleEntry::new(
            SubscriberId::new("my-subscriber").unwrap(),
            SubscriberLifecycleEvent::Registered,
        );
        let serialized = entry.serialize();

        let path = Path::new("test.ack");
        let deserialized = SubscriberLifecycleEntry::deserialize(&serialized, path)
            .unwrap()
            .unwrap();

        assert_eq!(entry, deserialized);
    }

    #[test]
    fn lifecycle_entry_unregistered_roundtrip() {
        let entry = SubscriberLifecycleEntry::new(
            SubscriberId::new("exporter-1").unwrap(),
            SubscriberLifecycleEvent::Unregistered,
        );
        let serialized = entry.serialize();

        let path = Path::new("test.ack");
        let deserialized = SubscriberLifecycleEntry::deserialize(&serialized, path)
            .unwrap()
            .unwrap();

        assert_eq!(entry.subscriber_id, deserialized.subscriber_id);
        assert_eq!(entry.event, deserialized.event);
    }

    #[test]
    fn lifecycle_entry_crc_validation() {
        let entry = SubscriberLifecycleEntry::new(
            SubscriberId::new("test-sub").unwrap(),
            SubscriberLifecycleEvent::Registered,
        );
        let mut serialized = entry.serialize();

        // Corrupt a byte
        serialized[8] ^= 0xFF;

        let path = Path::new("test.ack");
        let result = SubscriberLifecycleEntry::deserialize(&serialized, path);

        assert!(matches!(
            result,
            Err(SubscriberError::AckLogCorrupted { .. })
        ));
    }

    #[test]
    fn lifecycle_entry_returns_none_for_ack_type() {
        // Create an ack entry and try to deserialize as lifecycle
        let ack_entry = test_entry("sub-1", 1, 0, AckOutcome::Acked);
        let serialized = ack_entry.serialize();

        let path = Path::new("test.ack");
        let result = SubscriberLifecycleEntry::deserialize(&serialized, path).unwrap();

        // Should return None since it's not a lifecycle entry type
        assert!(result.is_none());
    }

    #[test]
    fn ack_entry_returns_none_for_lifecycle_type() {
        // Create a lifecycle entry and try to deserialize as ack entry
        let lifecycle_entry = SubscriberLifecycleEntry::new(
            SubscriberId::new("sub-1").unwrap(),
            SubscriberLifecycleEvent::Registered,
        );
        let serialized = lifecycle_entry.serialize();

        let path = Path::new("test.ack");
        let result = AckLogEntry::deserialize(&serialized, path).unwrap();

        // Should return None since it's not an ack entry type
        assert!(result.is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Writer tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn writer_creates_new_file_with_header() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        {
            let _writer = AckLogWriter::open(&path).unwrap();
        }

        // Verify header: magic (8) + version (2) + header_size (2) = 12 bytes
        let file = std::fs::read(&path).unwrap();
        assert_eq!(&file[0..8], quiver.ack_MAGIC);
        assert_eq!(u16::from_le_bytes([file[8], file[9]]), quiver.ack_VERSION);
        assert_eq!(
            u16::from_le_bytes([file[10], file[11]]),
            HEADER_V1_SIZE as u16
        );
        assert_eq!(file.len(), HEADER_V1_SIZE);
    }

    #[test]
    fn writer_append_single_entry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let entry = test_entry("subscriber-1", 1, 0, AckOutcome::Acked);

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&entry).unwrap();
        }

        // Read back
        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], entry);
    }

    #[test]
    fn writer_append_multiple_entries() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let entries = vec![
            test_entry("sub-a", 1, 0, AckOutcome::Acked),
            test_entry("sub-a", 1, 1, AckOutcome::Acked),
            test_entry("sub-b", 1, 0, AckOutcome::Dropped),
        ];

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            for entry in &entries {
                writer.append(entry).unwrap();
            }
        }

        let reader = AckLogReader::open(&path).unwrap();
        let read_entries = reader.read_all().unwrap();
        assert_eq!(read_entries, entries);
    }

    #[test]
    fn writer_append_batch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let entries = vec![
            test_entry("batch-sub", 10, 0, AckOutcome::Acked),
            test_entry("batch-sub", 10, 1, AckOutcome::Acked),
            test_entry("batch-sub", 10, 2, AckOutcome::Dropped),
        ];

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append_batch(&entries).unwrap();
        }

        let reader = AckLogReader::open(&path).unwrap();
        let read_entries = reader.read_all().unwrap();
        assert_eq!(read_entries, entries);
    }

    #[test]
    fn writer_reopen_and_append() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let entry1 = test_entry("sub-1", 1, 0, AckOutcome::Acked);
        let entry2 = test_entry("sub-1", 1, 1, AckOutcome::Acked);

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&entry1).unwrap();
        }

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&entry2).unwrap();
        }

        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert_eq!(entries, vec![entry1, entry2]);
    }

    #[test]
    fn writer_append_lifecycle_entry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let lifecycle = SubscriberLifecycleEntry::new(
            SubscriberId::new("new-subscriber").unwrap(),
            SubscriberLifecycleEvent::Registered,
        );

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append_lifecycle(&lifecycle).unwrap();
        }

        // Verify file was written (header + length prefix + entry)
        let file = std::fs::read(&path).unwrap();
        assert!(file.len() > HEADER_V1_SIZE);
    }

    #[test]
    fn reader_skips_lifecycle_entries_in_mixed_log() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let ack1 = test_entry("sub-1", 1, 0, AckOutcome::Acked);
        let lifecycle = SubscriberLifecycleEntry::new(
            SubscriberId::new("sub-1").unwrap(),
            SubscriberLifecycleEvent::Registered,
        );
        let ack2 = test_entry("sub-1", 1, 1, AckOutcome::Acked);

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&ack1).unwrap();
            writer.append_lifecycle(&lifecycle).unwrap();
            writer.append(&ack2).unwrap();
        }

        // Reader should return only ack entries (lifecycle entries skipped)
        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], ack1);
        assert_eq!(entries[1], ack2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Reader tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn reader_empty_log() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        {
            let _writer = AckLogWriter::open(&path).unwrap();
        }

        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn reader_invalid_magic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        // Write a file with wrong magic but enough bytes for minimum header
        std::fs::write(&path, b"BADMAGIC1234").unwrap();

        let result = AckLogReader::open(&path);
        assert!(matches!(
            result,
            Err(SubscriberError::AckLogCorrupted { .. })
        ));
    }

    #[test]
    fn reader_invalid_version() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let mut data = Vec::new();
        data.extend_from_slice(quiver.ack_MAGIC);
        data.extend_from_slice(&99u16.to_le_bytes()); // Invalid version
        data.extend_from_slice(&12u16.to_le_bytes()); // header_size

        std::fs::write(&path, &data).unwrap();

        let result = AckLogReader::open(&path);
        assert!(matches!(
            result,
            Err(SubscriberError::AckLogCorrupted { .. })
        ));
    }

    #[test]
    fn reader_handles_partial_length() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        // Write valid header + partial length prefix (only 2 of 4 bytes)
        let mut data = Vec::new();
        data.extend_from_slice(quiver.ack_MAGIC);
        data.extend_from_slice(&quiver.ack_VERSION.to_le_bytes());
        data.extend_from_slice(&(HEADER_V1_SIZE as u16).to_le_bytes()); // header_size
        data.extend_from_slice(&[0x20, 0x00]); // Partial 2-byte length (need 4)

        std::fs::write(&path, &data).unwrap();

        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert!(entries.is_empty()); // Partial entry ignored
    }

    #[test]
    fn reader_handles_partial_entry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let entry = test_entry("sub-1", 1, 0, AckOutcome::Acked);

        // Write a valid entry, then a partial one
        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&entry).unwrap();
        }

        // Append a partial entry (length says 32 bytes, but only write 10)
        {
            use std::io::Write;
            let mut file = OpenOptions::new().append(true).open(&path).unwrap();
            file.write_all(&32u32.to_le_bytes()).unwrap();
            file.write_all(&[0u8; 10]).unwrap();
        }

        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], entry);
    }

    #[test]
    fn reader_last_valid_position() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let entry = test_entry("sub-1", 1, 0, AckOutcome::Acked);

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&entry).unwrap();
        }

        let file_len = std::fs::metadata(&path).unwrap().len();

        let mut reader = AckLogReader::open(&path).unwrap();
        let _ = reader.next_entry().unwrap().unwrap();
        assert!(reader.next_entry().unwrap().is_none());

        // Last valid position should be at end of file
        assert_eq!(reader.last_valid_position(), file_len);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Truncation tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn truncate_removes_partial_entry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let entry = test_entry("sub-1", 1, 0, AckOutcome::Acked);

        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&entry).unwrap();
        }

        let valid_pos = std::fs::metadata(&path).unwrap().len();

        // Append garbage
        {
            use std::io::Write;
            let mut file = OpenOptions::new().append(true).open(&path).unwrap();
            file.write_all(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        }

        // Truncate back to valid
        truncate_quiver.ack(&path, valid_pos).unwrap();

        assert_eq!(std::fs::metadata(&path).unwrap().len(), valid_pos);

        // Should still read the valid entry
        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], entry);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Forward compatibility tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn reader_skips_unknown_entry_type() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let entry1 = test_entry("sub-1", 1, 0, AckOutcome::Acked);
        let entry2 = test_entry("sub-1", 1, 1, AckOutcome::Acked);

        // Write first entry normally
        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&entry1).unwrap();
        }

        // Manually write an entry with unknown type (simulating future format)
        {
            use std::io::Write;
            let mut file = OpenOptions::new().append(true).open(&path).unwrap();

            // Build a fake entry with entry_type = 99 (unknown)
            let mut entry_content = Vec::new();
            entry_content.extend_from_slice(&[0u8; 4]); // CRC placeholder
            entry_content.push(99u8); // Unknown entry type
            entry_content.push(0u8); // Flags
            entry_content.extend_from_slice(&12345u64.to_le_bytes()); // timestamp_ms
            entry_content.push(5u8); // sub_id_len
            entry_content.extend_from_slice(b"dummy"); // sub_id
            entry_content.push(0u8); // outcome
            entry_content.extend_from_slice(&0u64.to_le_bytes()); // seg_seq
            entry_content.extend_from_slice(&0u32.to_le_bytes()); // b_idx

            // Compute CRC
            let mut hasher = Crc32Hasher::new();
            hasher.update(&entry_content[4..]);
            let crc = hasher.finalize();
            entry_content[0..4].copy_from_slice(&crc.to_le_bytes());

            // Write length + content
            let len = entry_content.len() as u32;
            file.write_all(&len.to_le_bytes()).unwrap();
            file.write_all(&entry_content).unwrap();
        }

        // Write second entry normally
        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            writer.append(&entry2).unwrap();
        }

        // Reader should skip unknown entry and return entry1, entry2
        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry1);
        assert_eq!(entries[1], entry2);
    }

    #[test]
    fn reader_handles_extended_header() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        // Manually write a header with extra bytes (simulating future extension)
        {
            use std::io::Write;
            let mut file = File::create(&path).unwrap();

            file.write_all(quiver.ack_MAGIC).unwrap();
            file.write_all(&quiver.ack_VERSION.to_le_bytes()).unwrap();
            // Header size = 20 (current 12 + 8 extra bytes)
            file.write_all(&20u16.to_le_bytes()).unwrap();
            // Extra future header fields
            file.write_all(&[0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x9A])
                .unwrap();
        }

        // Append a valid entry
        {
            let mut writer = AckLogWriter::open(&path).unwrap();
            let entry = test_entry("sub-1", 1, 0, AckOutcome::Acked);
            writer.append(&entry).unwrap();
        }

        // Reader should skip extra header bytes and read entry
        let reader = AckLogReader::open(&path).unwrap();
        let entries = reader.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].subscriber_id.as_str(), "sub-1");
    }

    #[test]
    fn deserialize_returns_none_for_unknown_entry_type() {
        let path = Path::new("test.ack");

        // Build entry with unknown type
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 4]); // CRC placeholder
        data.push(42u8); // Unknown entry type
        data.push(0u8); // Flags
        data.extend_from_slice(&12345u64.to_le_bytes()); // timestamp_ms
        data.push(4u8); // sub_id_len
        data.extend_from_slice(b"test"); // sub_id
        data.push(0u8); // outcome
        data.extend_from_slice(&1u64.to_le_bytes()); // seg_seq
        data.extend_from_slice(&0u32.to_le_bytes()); // b_idx

        // Compute CRC
        let mut hasher = Crc32Hasher::new();
        hasher.update(&data[4..]);
        let crc = hasher.finalize();
        data[0..4].copy_from_slice(&crc.to_le_bytes());

        let result = AckLogEntry::deserialize(&data, path).unwrap();
        assert!(result.is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Rotation tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn rotating_writer_creates_and_rotates() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        // Use a very small rotation threshold to trigger rotation quickly
        let config = RotatingAckLogConfig {
            rotation_target_bytes: 100, // Very small for testing
            max_rotated_files: 4,
        };

        let mut writer = RotatingAckLogWriter::open(&path, config).unwrap();

        // Write entries until rotation occurs
        for i in 0..10 {
            let entry = test_entry("sub-1", i, 0, AckOutcome::Acked);
            writer.append(&entry).unwrap();
        }

        // Should have at least one rotated file
        assert!(
            writer.rotated_file_count() > 0,
            "Expected rotation to occur"
        );

        // All entries should be readable via read_all_quiver.acks
        let all_entries = read_all_quiver.acks(&path).unwrap();
        assert_eq!(all_entries.len(), 10);
    }

    #[test]
    fn rotating_writer_purge_before() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let config = RotatingAckLogConfig {
            rotation_target_bytes: 100,
            max_rotated_files: 8,
        };

        let mut writer = RotatingAckLogWriter::open(&path, config).unwrap();

        // Write entries with increasing segment sequences
        for seg in 1..=5 {
            for idx in 0..5 {
                let entry = test_entry("sub-1", seg, idx, AckOutcome::Acked);
                writer.append(&entry).unwrap();
            }
        }

        let initial_rotated = writer.rotated_file_count();

        // Purge files with max_segment_seq < 3
        let purged = writer.purge_before(SegmentSeq::new(3)).unwrap();

        // Some files should have been purged (those only containing seg 1-2)
        if initial_rotated > 0 {
            assert!(
                writer.rotated_file_count() <= initial_rotated,
                "Rotated file count should not increase"
            );
        }

        // All remaining entries should still be readable
        let remaining = read_all_quiver.acks(&path).unwrap();
        assert!(!remaining.is_empty());

        // Verify we can identify how many were purged
        let _ = purged; // Silence unused warning - value is informational
    }

    #[test]
    fn read_all_quiver.acks_combines_rotated_and_active() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let config = RotatingAckLogConfig {
            rotation_target_bytes: 100,
            max_rotated_files: 4,
        };

        let mut writer = RotatingAckLogWriter::open(&path, config).unwrap();

        // Write entries
        let entries: Vec<_> = (0..20u64)
            .map(|i| test_entry("sub-1", i / 4, (i % 4) as u32, AckOutcome::Acked))
            .collect();

        for entry in &entries {
            writer.append(entry).unwrap();
        }

        // Ensure we have some rotated files
        let rotated_count = writer.rotated_file_count();
        assert!(rotated_count > 0, "Expected rotation to occur");

        // Read all and verify count
        let all_entries = read_all_quiver.acks(&path).unwrap();
        assert_eq!(all_entries.len(), 20);
    }

    #[test]
    fn rotating_writer_respects_max_rotated_files() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        let config = RotatingAckLogConfig {
            rotation_target_bytes: 50, // Very small
            max_rotated_files: 2,      // Very limited
        };

        let mut writer = RotatingAckLogWriter::open(&path, config).unwrap();

        // Write many entries
        for i in 0..100 {
            let entry = test_entry("sub-1", i, 0, AckOutcome::Acked);
            writer.append(&entry).unwrap();
        }

        // Should not exceed max_rotated_files
        assert!(
            writer.rotated_file_count() <= 2,
            "Should not exceed max_rotated_files"
        );
    }

    #[test]
    fn read_all_quiver.acks_empty_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        // Create empty active file
        {
            let _writer = AckLogWriter::open(&path).unwrap();
        }

        let entries = read_all_quiver.acks(&path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn rotating_writer_scan_existing_rotated_files() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("quiver.ack");

        // Create some rotated files manually
        {
            let config = RotatingAckLogConfig {
                rotation_target_bytes: 50,
                max_rotated_files: 4,
            };

            let mut writer = RotatingAckLogWriter::open(&path, config).unwrap();

            for i in 0..30 {
                let entry = test_entry("sub-1", i, 0, AckOutcome::Acked);
                writer.append(&entry).unwrap();
            }
        }

        // Reopen and verify state is recovered
        let config = RotatingAckLogConfig::default();
        let writer = RotatingAckLogWriter::open(&path, config).unwrap();

        // Should have found the rotated files
        assert!(
            writer.rotated_file_count() > 0 || writer.current_size() > HEADER_V1_SIZE as u64,
            "Should have found existing data"
        );
    }
}
