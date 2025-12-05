// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Read-side companion to the WAL writer.
//!
//! The reader validates headers, streams entries starting at arbitrary offsets,
//! and exposes helper types such as [`WalConsumerCheckpoint`] so higher layers can
//! describe how much of the log is safe to reclaim. Like the writer, it is
//! currently exercised by tests until the runtime wires in replay logic.
#![allow(dead_code)]

#[cfg(test)]
use self::test_support::ReadFailure;
use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crc32fast::Hasher;

use crate::record_bundle::{SchemaFingerprint, SlotId};

use super::header::{WAL_HEADER_LEN, WalHeader};
use super::{
    ENTRY_HEADER_LEN, ENTRY_TYPE_RECORD_BUNDLE, SCHEMA_FINGERPRINT_LEN, SLOT_HEADER_LEN, WalError,
    WalOffset, WalResult,
};

/// Sequential reader that validates the WAL header before exposing iterators
/// over decoded entries.
#[derive(Debug)]
pub(crate) struct WalReader {
    file: File,
    path: PathBuf,
    segment_cfg_hash: [u8; 16],
}

impl WalReader {
    pub fn open(path: impl Into<PathBuf>) -> WalResult<Self> {
        let path = path.into();
        let mut file = File::open(&path)?;
        let header = WalHeader::read_from(&mut file)?;
        let _ = file.seek(SeekFrom::Start(WAL_HEADER_LEN as u64))?;

        Ok(Self {
            file,
            path,
            segment_cfg_hash: header.segment_cfg_hash,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn segment_cfg_hash(&self) -> [u8; 16] {
        self.segment_cfg_hash
    }

    /// Returns an iterator that starts at `offset`, clamped to the minimum
    /// position right after the WAL header.
    pub fn iter_from(&mut self, offset: u64) -> WalResult<WalEntryIter<'_>> {
        let start = offset.max(WAL_HEADER_LEN as u64);
        let _ = self.file.seek(SeekFrom::Start(start))?;
        Ok(WalEntryIter::new(&mut self.file, start))
    }

    /// Seeks back to the entry immediately after the header so a fresh scan can
    /// start from the beginning.
    pub fn rewind(&mut self) -> WalResult<()> {
        let _ = self.file.seek(SeekFrom::Start(WAL_HEADER_LEN as u64))?;
        Ok(())
    }
}

/// Iterator that yields decoded [`WalRecordBundle`] instances while keeping
/// track of the next byte offset so callers can build consumer checkpoints.
pub(crate) struct WalEntryIter<'a> {
    file: &'a mut File,
    buffer: Vec<u8>,
    next_offset: u64,
    finished: bool,
}

impl<'a> WalEntryIter<'a> {
    fn new(file: &'a mut File, offset: u64) -> Self {
        Self {
            file,
            buffer: Vec::new(),
            next_offset: offset,
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

        match decode_entry(entry_start, next_offset, &self.buffer) {
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

/// Consumer checkpoint used by writers and readers to describe how much of the WAL is
/// durably processed. `safe_offset` is expressed in absolute bytes (including
/// the header) while `safe_sequence` guards against replay regressions.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WalConsumerCheckpoint {
    pub safe_offset: u64,
    pub safe_sequence: u64,
}

impl WalConsumerCheckpoint {
    /// Advances the cursor to cover the provided bundle. Callers typically run
    /// this inside a replay loop and later hand the cursor back to the writer so
    /// it can truncate or reclaim the prefix.
    pub fn advance(&mut self, bundle: &WalRecordBundle) {
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

fn decode_entry(entry_start: u64, next_offset: u64, body: &[u8]) -> WalResult<WalRecordBundle> {
    if body.len() < ENTRY_HEADER_LEN {
        return Err(WalError::InvalidEntry("body shorter than header"));
    }

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
            position: entry_start,
            sequence,
        },
        next_offset,
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
