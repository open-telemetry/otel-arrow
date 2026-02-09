// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Read-side companion to the WAL writer.
//!
//! The reader validates headers, streams entries starting at arbitrary offsets,
//! and exposes helper types for tracking replay progress.
//!
//! # Synchronous I/O Design
//!
//! This module intentionally uses synchronous I/O rather than async for several
//! reasons:
//!
//! 1. **Recovery-only use case**: The WAL reader is only used during engine
//!    startup for crash recovery, not on the hot path during normal operation.
//!
//! 2. **Iterator compatibility**: Rust's [`Iterator`] trait is inherently
//!    synchronous. Using async would require [`Stream`] which adds complexity
//!    for no benefit in the recovery scenario.
//!
//! 3. **Sequential access**: WAL replay reads entries sequentially from disk.
//!    Async I/O provides no benefit here since there's no concurrent work to
//!    overlap with I/O waits.
//!
//! For async WAL operations (writes, flushes), see [`super::WalWriter`] which
//! uses `tokio::fs::File` for non-blocking I/O on the hot path.
//!
//! # Entry Format
//!
//! Each WAL entry has this layout:
//!
//! ```text
//! ┌──────────┬────────────────┬─────────────────┬──────────┐
//! │ len (4)  │ entry_hdr (25) │ slot_data (var) │ crc (4)  │
//! └──────────┴────────────────┴─────────────────┴──────────┘
//! ```
//!
//! - **len**: Size of `entry_hdr + slot_data` (excludes len and crc fields)
//! - **entry_hdr**: Type (1), timestamp (8), sequence (8), slot_bitmap (8)
//! - **slot_data**: For each set bit: slot_id (2), fingerprint (32), rows (4),
//!   payload_len (4), Arrow IPC bytes (payload_len)
//! - **crc**: CRC32 over `entry_hdr + slot_data`
//!
//! # Usage
//!
//! ```ignore
//! let mut reader = WalReader::open("wal/quiver.wal")?;
//! let mut cursor = WalConsumerCursor::default();
//!
//! for result in reader.iter_from(0)? {
//!     let bundle = result?;
//!     println!("seq={} slots={}", bundle.sequence, bundle.slots.len());
//!     cursor.increment(&bundle);  // in-memory only
//! }
//! // cursor now points past the last entry
//! // call writer.persist_cursor(&cursor) to persist
//! ```
#![allow(dead_code)]

#[cfg(test)]
use self::test_support::ReadFailure;
use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crc32fast::Hasher;

use crate::logging::{otel_error, otel_warn};
use crate::record_bundle::{SchemaFingerprint, SlotId};

use super::header::WalHeader;
use super::{
    ENTRY_HEADER_LEN, ENTRY_TYPE_RECORD_BUNDLE, MAX_ROTATION_TARGET_BYTES, SCHEMA_FINGERPRINT_LEN,
    SLOT_HEADER_LEN, WalError, WalOffset, WalResult,
};

/// Maximum allowed entry size, derived from [`MAX_ROTATION_TARGET_BYTES`].
///
/// Since entries cannot span files and files are capped at 256 MiB, no valid
/// entry can exceed this size. The reader rejects larger entries to guard
/// against corrupted or malicious WAL files causing excessive allocation.
const MAX_ENTRY_SIZE: usize = MAX_ROTATION_TARGET_BYTES as usize;

/// Sequential reader that validates the WAL header before exposing iterators
/// over decoded entries.
///
/// # Sync I/O Design
///
/// This reader uses synchronous I/O intentionally. See the [module-level
/// documentation](self) for the design rationale. In brief: WAL reading only
/// occurs during crash recovery at startup, where blocking I/O is acceptable
/// and the iterator-based API is simpler than async streams.
///
/// For async WAL operations (writes, flushes), see [`WalWriter`](super::WalWriter).
#[derive(Debug)]
pub(crate) struct WalReader {
    file: File,
    path: PathBuf,
    segment_cfg_hash: [u8; 16],
    /// Header size in bytes (from the file's header_size field).
    /// Used for coordinate conversions between WAL positions and file offsets.
    header_size: u64,
    /// WAL stream position at the start of this file (from the header).
    /// This is 0 for the very first WAL file created. After rotation, both
    /// rotated files and the new active file have non-zero values representing
    /// the cumulative position in the logical WAL stream.
    wal_position_start: u64,
    /// Current file offset for sequential reading via `read_next_entry`.
    next_file_offset: u64,
    /// Cached file length for bounds checking.
    file_len: u64,
    /// Reusable buffer for entry reading.
    read_buffer: Vec<u8>,
    /// Whether we've hit EOF or an error.
    finished: bool,
}

impl WalReader {
    /// Opens a WAL file for reading and validates its header.
    ///
    /// Uses synchronous I/O - see the [type-level documentation](Self) for
    /// rationale. This is only called during engine startup/recovery.
    pub fn open(path: impl Into<PathBuf>) -> WalResult<Self> {
        let path = path.into();
        let mut file = File::open(&path)?;
        let header = WalHeader::read_from_sync(&mut file)?;
        let header_size = header.header_size as u64;
        let wal_position_start = header.wal_position_start;
        let file_len = file.metadata()?.len();
        let _ = file.seek(SeekFrom::Start(header_size))?;

        Ok(Self {
            file,
            path,
            segment_cfg_hash: header.segment_cfg_hash,
            header_size,
            wal_position_start,
            next_file_offset: header_size,
            file_len,
            read_buffer: Vec::new(),
            finished: false,
        })
    }

    /// Returns the path to the WAL file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the segment configuration hash from the WAL header.
    pub fn segment_cfg_hash(&self) -> [u8; 16] {
        self.segment_cfg_hash
    }

    /// Returns the header size in bytes for this WAL file.
    pub fn header_size(&self) -> u64 {
        self.header_size
    }

    /// Returns the WAL stream position at the start of this file.
    ///
    /// This value is read from the file header. It is 0 only for the very first
    /// WAL file ever created. After any rotation, both rotated files and the
    /// new active file have non-zero values representing their position in the
    /// logical WAL stream.
    pub const fn wal_position_start(&self) -> u64 {
        self.wal_position_start
    }

    /// Returns the WAL stream position at the end of this file (exclusive).
    ///
    /// This is computed from the file length and header, representing the
    /// position where the next entry would start if appended to this file.
    pub fn wal_position_end(&self) -> WalResult<u64> {
        let data_bytes = self.file_len.saturating_sub(self.header_size);
        Ok(self.wal_position_start.saturating_add(data_bytes))
    }

    /// Seeks to the given WAL position for subsequent `read_next_entry` calls.
    ///
    /// If the position is before this file's start, seeks to the beginning.
    /// If the position is beyond this file's content, `read_next_entry` will
    /// return `None` immediately.
    pub fn seek_to_position(&mut self, wal_position: u64) -> WalResult<()> {
        // Calculate the position within this file's data section
        let position_in_file = wal_position.saturating_sub(self.wal_position_start);
        let file_offset = position_in_file.saturating_add(self.header_size);
        let start = file_offset.max(self.header_size);
        let _ = self.file.seek(SeekFrom::Start(start))?;
        self.next_file_offset = start;
        self.finished = false;
        Ok(())
    }

    /// Reads the next WAL entry at the current file position.
    ///
    /// Returns `None` when EOF is reached. After an error, subsequent calls
    /// will continue to return `None`.
    ///
    /// This is more efficient than `iter_from` when reading entries one at a
    /// time across multiple files, as it avoids re-seeking on each call.
    pub fn read_next_entry(&mut self) -> Option<WalResult<WalRecordBundle>> {
        if self.finished {
            return None;
        }

        let entry_start = self.next_file_offset;
        let mut len_buf = [0u8; 4];
        match read_exact_or_eof(&mut self.file, &mut len_buf) {
            Ok(ReadStatus::Eof) => {
                self.finished = true;
                return None;
            }
            Ok(ReadStatus::Filled) => {}
            Err(err) => {
                self.finished = true;
                return Some(Err(err));
            }
        }

        let entry_len = u32::from_le_bytes(len_buf) as usize;

        // Guard against malicious or corrupted length values
        if entry_len > MAX_ENTRY_SIZE {
            self.finished = true;
            return Some(Err(WalError::InvalidEntry(
                "entry length exceeds maximum allowed size",
            )));
        }

        let remaining_bytes = self.file_len.saturating_sub(entry_start + 4) as usize;
        if entry_len > remaining_bytes {
            self.finished = true;
            return Some(Err(WalError::InvalidEntry(
                "entry length exceeds remaining file size",
            )));
        }

        self.read_buffer.resize(entry_len, 0);
        if let Err(err) = read_entry_body(&mut self.file, &mut self.read_buffer) {
            self.finished = true;
            let wal_err = if err.kind() == ErrorKind::UnexpectedEof {
                WalError::UnexpectedEof("entry body")
            } else {
                err.into()
            };
            return Some(Err(wal_err));
        }

        let mut crc_buf = [0u8; 4];
        if let Err(err) = read_entry_crc(&mut self.file, &mut crc_buf) {
            self.finished = true;
            let wal_err = if err.kind() == ErrorKind::UnexpectedEof {
                WalError::UnexpectedEof("entry crc")
            } else {
                err.into()
            };
            return Some(Err(wal_err));
        }

        let stored_crc = u32::from_le_bytes(crc_buf);
        let mut hasher = Hasher::new();
        hasher.update(&self.read_buffer);
        let computed_crc = hasher.finalize();
        if stored_crc != computed_crc {
            self.finished = true;
            return Some(Err(WalError::CrcMismatch {
                stored: stored_crc,
                computed: computed_crc,
            }));
        }

        let next_offset = entry_start
            .saturating_add(4)
            .saturating_add(entry_len as u64)
            .saturating_add(4);

        self.next_file_offset = next_offset;

        match decode_entry(
            entry_start,
            next_offset,
            &self.read_buffer,
            self.header_size,
            self.wal_position_start,
        ) {
            Ok(entry) => Some(Ok(entry)),
            Err(err) => {
                self.finished = true;
                Some(Err(err))
            }
        }
    }

    /// Returns an iterator that starts at the given WAL position.
    ///
    /// WAL positions are logical coordinates that span across rotated files.
    /// Pass 0 to start from the first entry in this file (regardless of
    /// wal_position_start).
    ///
    /// If the requested position is before this file's `wal_position_start`,
    /// the iterator starts from the beginning of this file.
    /// If the requested position is beyond this file's content, the iterator
    /// will be empty (will immediately return `None`).
    pub fn iter_from(&mut self, wal_position: u64) -> WalResult<WalEntryIter<'_>> {
        // Calculate the position within this file's data section
        // If wal_position is before this file starts, begin at the start
        let position_in_file = wal_position.saturating_sub(self.wal_position_start);

        // Convert to file offset (add header length)
        let file_offset = position_in_file.saturating_add(self.header_size);
        let start = file_offset.max(self.header_size);
        // Refresh file_len in case file grew since open
        self.file_len = self.file.metadata()?.len();
        let _ = self.file.seek(SeekFrom::Start(start))?;
        Ok(WalEntryIter::new(
            &mut self.file,
            start,
            self.file_len,
            self.header_size,
            self.wal_position_start,
        ))
    }

    /// Seeks back to the entry immediately after the header so a fresh scan can
    /// start from the beginning.
    pub fn rewind(&mut self) -> WalResult<()> {
        let _ = self.file.seek(SeekFrom::Start(self.header_size))?;
        Ok(())
    }
}

/// Iterator that yields decoded [`WalRecordBundle`] instances while keeping
/// track of the next byte offset so callers can build consumer cursors.
pub(crate) struct WalEntryIter<'a> {
    file: &'a mut File,
    buffer: Vec<u8>,
    next_offset: u64,
    /// Known file size at iterator creation, used to reject impossibly large entries.
    file_len: u64,
    /// Header size in bytes, used for coordinate conversions.
    header_size: u64,
    /// WAL position at the start of this file, used for computing global positions.
    wal_position_start: u64,
    finished: bool,
}

impl<'a> WalEntryIter<'a> {
    fn new(
        file: &'a mut File,
        offset: u64,
        file_len: u64,
        header_size: u64,
        wal_position_start: u64,
    ) -> Self {
        Self {
            file,
            buffer: Vec::new(),
            next_offset: offset,
            file_len,
            header_size,
            wal_position_start,
            finished: false,
        }
    }
}

impl<'a> Iterator for WalEntryIter<'a> {
    type Item = WalResult<WalRecordBundle>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let entry_start = self.next_offset;
        let mut len_buf = [0u8; 4];
        match read_exact_or_eof(self.file, &mut len_buf) {
            Ok(ReadStatus::Eof) => {
                self.finished = true;
                return None;
            }
            Ok(ReadStatus::Filled) => {}
            Err(err) => {
                self.finished = true;
                return Some(Err(err));
            }
        }

        let entry_len = u32::from_le_bytes(len_buf) as usize;

        // Guard against malicious or corrupted length values. Check the hard
        // cap first (avoids allocation bombs even with huge files), then verify
        // against actual remaining bytes.
        if entry_len > MAX_ENTRY_SIZE {
            self.finished = true;
            return Some(Err(WalError::InvalidEntry(
                "entry length exceeds maximum allowed size",
            )));
        }

        let remaining_bytes = self.file_len.saturating_sub(entry_start + 4) as usize;
        if entry_len > remaining_bytes {
            self.finished = true;
            return Some(Err(WalError::InvalidEntry(
                "entry length exceeds remaining file size",
            )));
        }

        self.buffer.resize(entry_len, 0);
        if let Err(err) = read_entry_body(self.file, &mut self.buffer) {
            self.finished = true;
            let wal_err = if err.kind() == ErrorKind::UnexpectedEof {
                WalError::UnexpectedEof("entry body")
            } else {
                err.into()
            };
            return Some(Err(wal_err));
        }

        let mut crc_buf = [0u8; 4];
        if let Err(err) = read_entry_crc(self.file, &mut crc_buf) {
            self.finished = true;
            let wal_err = if err.kind() == ErrorKind::UnexpectedEof {
                WalError::UnexpectedEof("entry crc")
            } else {
                err.into()
            };
            return Some(Err(wal_err));
        }

        let stored_crc = u32::from_le_bytes(crc_buf);
        let mut hasher = Hasher::new();
        hasher.update(&self.buffer);
        let computed_crc = hasher.finalize();
        if stored_crc != computed_crc {
            self.finished = true;
            return Some(Err(WalError::CrcMismatch {
                stored: stored_crc,
                computed: computed_crc,
            }));
        }

        let next_offset = entry_start
            .saturating_add(4)
            .saturating_add(entry_len as u64)
            .saturating_add(4);

        self.next_offset = next_offset;

        match decode_entry(
            entry_start,
            next_offset,
            &self.buffer,
            self.header_size,
            self.wal_position_start,
        ) {
            Ok(entry) => Some(Ok(entry)),
            Err(err) => {
                self.finished = true;
                Some(Err(err))
            }
        }
    }
}

/// Fully decoded WAL entry that the engine can replay without touching the raw
/// on-disk representation.
#[derive(Debug, Clone)]
pub(crate) struct WalRecordBundle {
    pub offset: WalOffset,
    pub next_offset: u64,
    pub ingestion_ts_nanos: i64,
    pub sequence: u64,
    pub slot_bitmap: u64,
    pub slots: Vec<DecodedWalSlot>,
}

/// Arrow payload captured for a single slot inside a WAL entry.
#[derive(Debug, Clone)]
pub(crate) struct DecodedWalSlot {
    pub slot_id: SlotId,
    pub schema_fingerprint: SchemaFingerprint,
    pub row_count: u32,
    pub payload_len: u32,
    pub payload: Vec<u8>,
}

/// Opaque cursor describing how much of the WAL has been seen.
///
/// This is an **in-memory only** cursor. Updating it has no durability effect.
/// To persist progress and allow WAL cleanup, pass the cursor to
/// [`WalWriter::persist_cursor()`].
///
/// Usage:
/// 1. Start with `WalConsumerCursor::default()` (beginning of WAL)
/// 2. Call `cursor.increment(&bundle)` after processing each entry (in-memory)
/// 3. Call `writer.persist_cursor(&cursor)` to persist and enable cleanup
///
/// The writer validates that cursors land on entry boundaries and rejects
/// stale or regressed values.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WalConsumerCursor {
    /// Internal: byte position within the current WAL file.
    pub(super) safe_offset: u64,
    /// Internal: monotonic sequence number for ordering.
    pub(super) safe_sequence: u64,
}

impl WalConsumerCursor {
    /// Creates a cursor positioned immediately after the given bundle.
    ///
    /// This is equivalent to `WalConsumerCursor::default()` followed by
    /// `increment(bundle)`, but clearer when you only need to track a
    /// single entry.
    pub fn after(bundle: &WalRecordBundle) -> Self {
        Self::from_offset(&bundle.offset)
    }

    /// Creates a cursor from a `WalOffset` returned by the writer.
    ///
    /// This allows tracking progress directly from write operations without
    /// needing to read the entry back.
    pub fn from_offset(offset: &WalOffset) -> Self {
        Self {
            safe_offset: offset.next_offset,
            safe_sequence: offset.sequence,
        }
    }

    /// Moves the cursor past the provided bundle (in-memory only).
    ///
    /// This does **not** persist the cursor or trigger WAL cleanup.
    /// Call [`WalWriter::persist_cursor()`] to make progress durable.
    ///
    /// Typical usage in a replay loop:
    /// ```ignore
    /// let mut cursor = WalConsumerCursor::default();
    /// for bundle in reader.iter_from(0)? {
    ///     process(&bundle?);
    ///     cursor.increment(&bundle);  // in-memory only
    /// }
    /// writer.persist_cursor(&cursor)?;  // persist + cleanup
    /// ```
    pub fn increment(&mut self, bundle: &WalRecordBundle) {
        self.safe_offset = bundle.next_offset;
        self.safe_sequence = bundle.sequence;
    }
}

enum ReadStatus {
    Filled,
    Eof,
}

fn read_exact_or_eof(file: &mut File, buf: &mut [u8]) -> WalResult<ReadStatus> {
    let mut read = 0;
    while read < buf.len() {
        match read_length_chunk(file, &mut buf[read..]) {
            Ok(0) if read == 0 => return Ok(ReadStatus::Eof),
            Ok(0) => return Err(WalError::UnexpectedEof("entry length")),
            Ok(n) => read += n,
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(err.into()),
        }
    }
    Ok(ReadStatus::Filled)
}

fn read_length_chunk(file: &mut File, buf: &mut [u8]) -> std::io::Result<usize> {
    #[cfg(test)]
    if let Some(err) = test_support::take_failure(ReadFailure::Length) {
        return Err(err);
    }
    file.read(buf)
}

fn read_entry_body(file: &mut File, buffer: &mut [u8]) -> std::io::Result<()> {
    #[cfg(test)]
    if let Some(err) = test_support::take_failure(ReadFailure::Body) {
        return Err(err);
    }
    file.read_exact(buffer)
}

fn read_entry_crc(file: &mut File, buffer: &mut [u8; 4]) -> std::io::Result<()> {
    #[cfg(test)]
    if let Some(err) = test_support::take_failure(ReadFailure::Crc) {
        return Err(err);
    }
    file.read_exact(buffer)
}

/// Decodes a WAL entry body into a `WalRecordBundle`.
///
/// The `file_offset_start` and `file_offset_end` parameters are file offsets (including header).
/// They are converted to global WAL positions for the returned `WalOffset` by:
/// 1. Subtracting the header size to get position within file data
/// 2. Adding `wal_position_start` to get the global WAL stream position
fn decode_entry(
    file_offset_start: u64,
    file_offset_end: u64,
    body: &[u8],
    header_size: u64,
    wal_position_start: u64,
) -> WalResult<WalRecordBundle> {
    if body.len() < ENTRY_HEADER_LEN {
        return Err(WalError::InvalidEntry("body shorter than header"));
    }

    // Convert file offsets to global WAL positions:
    // file_offset - header_size = position within file's data section
    // + wal_position_start = global WAL stream position
    let position_in_file = file_offset_start.saturating_sub(header_size);
    let next_position_in_file = file_offset_end.saturating_sub(header_size);
    let wal_position = wal_position_start.saturating_add(position_in_file);
    let wal_next_position = wal_position_start.saturating_add(next_position_in_file);

    let mut cursor = 0;
    let entry_type = body[cursor];
    cursor += 1;
    if entry_type != ENTRY_TYPE_RECORD_BUNDLE {
        return Err(WalError::UnsupportedEntry(entry_type));
    }

    let ingestion_ts_nanos = read_i64(body, &mut cursor, "ingestion timestamp")?;
    let sequence = read_u64(body, &mut cursor, "sequence")?;
    let slot_bitmap = read_u64(body, &mut cursor, "slot bitmap")?;

    let expected_slots = slot_bitmap.count_ones() as usize;
    let mut slots = Vec::with_capacity(expected_slots);

    for _ in 0..expected_slots {
        if cursor + SLOT_HEADER_LEN > body.len() {
            return Err(WalError::InvalidEntry("slot header truncated"));
        }

        let slot_id = SlotId(read_u16(body, &mut cursor, "slot id")?);

        let mut schema_fingerprint = [0u8; SCHEMA_FINGERPRINT_LEN];
        schema_fingerprint.copy_from_slice(slice_bytes(
            body,
            &mut cursor,
            SCHEMA_FINGERPRINT_LEN,
            "schema fingerprint",
        )?);

        let row_count = read_u32(body, &mut cursor, "row count")?;
        let payload_len = read_u32(body, &mut cursor, "payload length")?;
        let payload_len_usize = usize::try_from(payload_len)
            .map_err(|_| WalError::InvalidEntry("payload length overflow"))?;

        let payload = slice_bytes(body, &mut cursor, payload_len_usize, "slot payload")?.to_vec();

        slots.push(DecodedWalSlot {
            slot_id,
            schema_fingerprint,
            row_count,
            payload_len,
            payload,
        });
    }

    if cursor != body.len() {
        return Err(WalError::InvalidEntry("unexpected trailing bytes"));
    }

    Ok(WalRecordBundle {
        offset: WalOffset {
            position: wal_position,
            next_offset: wal_next_position,
            sequence,
        },
        next_offset: wal_next_position,
        ingestion_ts_nanos,
        sequence,
        slot_bitmap,
        slots,
    })
}

fn slice_bytes<'a>(
    body: &'a [u8],
    cursor: &mut usize,
    len: usize,
    ctx: &'static str,
) -> WalResult<&'a [u8]> {
    if *cursor + len > body.len() {
        return Err(WalError::InvalidEntry(ctx));
    }
    let slice = &body[*cursor..*cursor + len];
    *cursor += len;
    Ok(slice)
}

fn read_u16(body: &[u8], cursor: &mut usize, ctx: &'static str) -> WalResult<u16> {
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(slice_bytes(body, cursor, 2, ctx)?);
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32(body: &[u8], cursor: &mut usize, ctx: &'static str) -> WalResult<u32> {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(slice_bytes(body, cursor, 4, ctx)?);
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(body: &[u8], cursor: &mut usize, ctx: &'static str) -> WalResult<u64> {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(slice_bytes(body, cursor, 8, ctx)?);
    Ok(u64::from_le_bytes(bytes))
}

fn read_i64(body: &[u8], cursor: &mut usize, ctx: &'static str) -> WalResult<i64> {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(slice_bytes(body, cursor, 8, ctx)?);
    Ok(i64::from_le_bytes(bytes))
}

// ─────────────────────────────────────────────────────────────────────────────
// Multi-File WAL Reader
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata about a WAL file discovered during scan.
#[derive(Debug, Clone)]
struct WalFileInfo {
    /// Path to the WAL file.
    path: PathBuf,
    /// Rotation ID (None for active file, Some(n) for rotated files).
    rotation_id: Option<u64>,
    /// WAL position at the start of this file.
    wal_position_start: u64,
    /// WAL position at the end of this file (exclusive).
    wal_position_end: u64,
}

/// A reader that iterates across multiple WAL files (rotated + active) in order.
///
/// This is used during WAL replay to read entries that may span across rotated
/// files and the active file. Files are read in order from oldest to newest.
///
/// # Usage
///
/// ```ignore
/// let reader = MultiFileWalReader::open("/path/to/wal/quiver.wal")?;
/// for entry in reader.iter_from(cursor_position)? {
///     let bundle = entry?;
///     // ... process bundle ...
/// }
/// ```
pub(crate) struct MultiFileWalReader {
    /// WAL files sorted by wal_position_start (oldest first).
    files: Vec<WalFileInfo>,
}

impl MultiFileWalReader {
    /// Opens all WAL files (rotated + active) at the given base path.
    ///
    /// Discovers rotated files by scanning for `<base>.N` pattern and reads
    /// headers to determine WAL position ranges for each file.
    pub fn open(base_path: impl Into<PathBuf>) -> WalResult<Self> {
        let base_path = base_path.into();
        let mut files = Vec::new();

        // Discover rotated files
        let rotated = discover_rotated_wal_files(&base_path)?;
        for (rotation_id, path) in rotated {
            match WalReader::open(&path) {
                Ok(reader) => {
                    let wal_position_end = reader.wal_position_end()?;
                    files.push(WalFileInfo {
                        path,
                        rotation_id: Some(rotation_id),
                        wal_position_start: reader.wal_position_start(),
                        wal_position_end,
                    });
                }
                Err(e) => {
                    otel_error!(
                        "quiver.wal.file.init",
                        path = %path.display(),
                        rotation_id,
                        error = %e,
                        error_type = "io",
                        file_type = "rotated",
                        message = "entries in this file will be lost during replay",
                    );
                }
            }
        }

        // Open active file if it exists
        if base_path.exists() {
            match WalReader::open(&base_path) {
                Ok(reader) => {
                    let wal_position_end = reader.wal_position_end()?;
                    files.push(WalFileInfo {
                        path: base_path,
                        rotation_id: None,
                        wal_position_start: reader.wal_position_start(),
                        wal_position_end,
                    });
                }
                Err(e) => {
                    otel_warn!(
                        "quiver.wal.file.init",
                        path = %base_path.display(),
                        error = %e,
                        error_type = "io",
                        file_type = "active",
                    );
                    // If we have no files at all, return the error
                    if files.is_empty() {
                        return Err(e);
                    }
                }
            }
        } else if files.is_empty() {
            return Err(WalError::Io(std::io::Error::new(
                ErrorKind::NotFound,
                format!("WAL file not found: {}", base_path.display()),
            )));
        }

        // Sort by wal_position_start to ensure correct ordering
        files.sort_by_key(|f| f.wal_position_start);

        Ok(Self { files })
    }

    /// Returns the end position of the WAL stream (exclusive).
    ///
    /// This is the position where the next entry would be written.
    /// If the WAL is empty (header only), this returns 0.
    pub fn wal_end_position(&self) -> u64 {
        self.files.last().map(|f| f.wal_position_end).unwrap_or(0)
    }

    /// Returns an iterator that reads entries starting at the given WAL position.
    ///
    /// The iterator will read from the appropriate file(s) based on where the
    /// position falls, and automatically transition to the next file when needed.
    pub fn iter_from(self, wal_position: u64) -> WalResult<MultiFileWalIter> {
        // Find the first file that contains or comes after the requested position
        let start_idx = self
            .files
            .iter()
            .position(|f| f.wal_position_end > wal_position)
            .unwrap_or(self.files.len());

        Ok(MultiFileWalIter {
            files: self.files,
            current_idx: start_idx,
            current_reader: None,
            start_position: wal_position,
        })
    }
}

/// Discovers rotated WAL files by scanning the directory for files matching
/// `<base>.N` pattern. Returns a list of (rotation_id, path) tuples
/// sorted by rotation_id (oldest first).
fn discover_rotated_wal_files(base_path: &Path) -> WalResult<Vec<(u64, PathBuf)>> {
    let parent = match base_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };

    let base_name = base_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            WalError::Io(std::io::Error::new(
                ErrorKind::InvalidInput,
                "WAL path has invalid filename",
            ))
        })?;

    let prefix = format!("{base_name}.");
    let mut discovered = Vec::new();

    let entries = match std::fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(discovered),
        Err(err) => return Err(WalError::Io(err)),
    };

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(name_str) = file_name.to_str() else {
            continue;
        };
        let Some(suffix) = name_str.strip_prefix(&prefix) else {
            continue;
        };
        let Ok(rotation_id) = suffix.parse::<u64>() else {
            continue;
        };
        discovered.push((rotation_id, entry.path()));
    }

    // Sort by rotation_id (oldest first)
    discovered.sort_by_key(|(id, _)| *id);
    Ok(discovered)
}

/// Iterator that reads WAL entries across multiple files.
pub(crate) struct MultiFileWalIter {
    /// All WAL files, sorted by wal_position_start.
    files: Vec<WalFileInfo>,
    /// Index of the current file being read.
    current_idx: usize,
    /// The current file's reader, positioned for sequential reading.
    current_reader: Option<WalReader>,
    /// The WAL position to start reading from in each new file.
    start_position: u64,
}

impl Iterator for MultiFileWalIter {
    type Item = WalResult<WalRecordBundle>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we've exhausted all files, we're done
            if self.current_idx >= self.files.len() {
                return None;
            }

            // Open and seek the current file if we haven't yet
            if self.current_reader.is_none() {
                let file_info = &self.files[self.current_idx];
                match WalReader::open(&file_info.path) {
                    Ok(mut reader) => {
                        // Seek to appropriate position within this file
                        let seek_pos = self.start_position.max(file_info.wal_position_start);
                        if let Err(e) = reader.seek_to_position(seek_pos) {
                            otel_error!(
                                "quiver.wal.file.init",
                                path = %file_info.path.display(),
                                error = %e,
                                error_type = "io",
                                file_type = "iteration",
                                message = "entries in this file will be lost during replay",
                            );
                            self.current_idx += 1;
                            continue;
                        }
                        self.current_reader = Some(reader);
                    }
                    Err(e) => {
                        otel_error!(
                            "quiver.wal.file.init",
                            path = %file_info.path.display(),
                            error = %e,
                            error_type = "io",
                            file_type = "iteration",
                            message = "entries in this file will be lost during replay",
                        );
                        self.current_idx += 1;
                        continue;
                    }
                }
            }

            // Read the next entry from the current file
            let reader = self
                .current_reader
                .as_mut()
                .expect("current_reader was set above");

            match reader.read_next_entry() {
                Some(Ok(entry)) => {
                    // Update start_position so if we move to next file, we start correctly
                    self.start_position = entry.next_offset;
                    return Some(Ok(entry));
                }
                Some(Err(e)) => {
                    // Error reading entry
                    return Some(Err(e));
                }
                None => {
                    // Current file exhausted, move to next
                    self.current_reader = None;
                    self.current_idx += 1;
                    // Loop will try the next file
                }
            }
        }
    }
}

#[cfg(test)]
pub(super) mod test_support {
    use std::cell::Cell;
    use std::io::Error;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ReadFailure {
        Length,
        Body,
        Crc,
    }

    thread_local! {
        static NEXT_FAILURE: Cell<Option<ReadFailure>> = const { Cell::new(None) };
    }

    pub fn fail_next_read(stage: ReadFailure) {
        NEXT_FAILURE.with(|slot| slot.set(Some(stage)));
    }

    pub fn take_failure(stage: ReadFailure) -> Option<Error> {
        NEXT_FAILURE.with(|slot| {
            if slot.get() == Some(stage) {
                slot.set(None);
                Some(Error::other("wal reader injected read failure"))
            } else {
                None
            }
        })
    }

    pub fn reset_failures() {
        NEXT_FAILURE.with(|slot| slot.set(None));
    }
}
