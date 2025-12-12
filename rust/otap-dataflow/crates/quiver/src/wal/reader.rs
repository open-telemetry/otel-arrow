// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Read-side companion to the WAL writer.
//!
//! The reader validates headers, streams entries starting at arbitrary offsets,
//! and exposes helper types for tracking replay progress.
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
#[derive(Debug)]
pub(crate) struct WalReader {
    file: File,
    path: PathBuf,
    segment_cfg_hash: [u8; 16],
    /// Header size in bytes (from the file's header_size field).
    /// Used for coordinate conversions between WAL positions and file offsets.
    header_size: u64,
}

impl WalReader {
    pub fn open(path: impl Into<PathBuf>) -> WalResult<Self> {
        let path = path.into();
        let mut file = File::open(&path)?;
        let header = WalHeader::read_from(&mut file)?;
        let header_size = header.header_size as u64;
        let _ = file.seek(SeekFrom::Start(header_size))?;

        Ok(Self {
            file,
            path,
            segment_cfg_hash: header.segment_cfg_hash,
            header_size,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn segment_cfg_hash(&self) -> [u8; 16] {
        self.segment_cfg_hash
    }

    /// Returns the header size in bytes for this WAL file.
    pub fn header_size(&self) -> u64 {
        self.header_size
    }

    /// Returns an iterator that starts at the given WAL position.
    ///
    /// WAL positions exclude the WAL header; pass 0 to start from the first entry.
    /// The position is converted to a file offset internally.
    pub fn iter_from(&mut self, wal_position: u64) -> WalResult<WalEntryIter<'_>> {
        // Convert WAL position to file offset (add header length)
        let file_offset = wal_position.saturating_add(self.header_size);
        let start = file_offset.max(self.header_size);
        let file_len = self.file.metadata()?.len();
        let _ = self.file.seek(SeekFrom::Start(start))?;
        Ok(WalEntryIter::new(&mut self.file, start, file_len, self.header_size))
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
    finished: bool,
}

impl<'a> WalEntryIter<'a> {
    fn new(file: &'a mut File, offset: u64, file_len: u64, header_size: u64) -> Self {
        Self {
            file,
            buffer: Vec::new(),
            next_offset: offset,
            file_len,
            header_size,
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

        match decode_entry(entry_start, next_offset, &self.buffer, self.header_size) {
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
        Self {
            safe_offset: bundle.next_offset,
            safe_sequence: bundle.sequence,
        }
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
/// They are converted to WAL positions (excluding header) for the returned `WalOffset`.
fn decode_entry(
    file_offset_start: u64,
    file_offset_end: u64,
    body: &[u8],
    header_size: u64,
) -> WalResult<WalRecordBundle> {
    if body.len() < ENTRY_HEADER_LEN {
        return Err(WalError::InvalidEntry("body shorter than header"));
    }

    // Convert file offsets to WAL positions (subtract header length)
    let wal_position = file_offset_start.saturating_sub(header_size);
    let wal_next_position = file_offset_end.saturating_sub(header_size);

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
