// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Single-file checkpoint container: header, complete snapshot, then WAL transactions.

use crate::primitives::{FILELOG_FORMAT_VERSION, Reader, Writer};
use crate::snapshot::encode_snapshot_into;
use crate::{
    DecodeError, EncodeError, SNAPSHOT_FOOTER_BYTES, SNAPSHOT_HEADER_BYTES, Snapshot,
    SnapshotRecord, crc32c, decode_snapshot,
};

const CHECKPOINT_MAGIC: &[u8; 8] = b"FLOGCHK\0";
/// Exact width of the immutable version 1 container header.
pub const CHECKPOINT_HEADER_BYTES: usize = 24;
const MIN_SNAPSHOT_BYTES: u64 = (SNAPSHOT_HEADER_BYTES + SNAPSHOT_FOOTER_BYTES) as u64;

/// Validated section boundary from a checkpoint container header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointHeader {
    /// Absolute file offset of the first WAL transaction, or EOF for an empty WAL.
    pub wal_offset: u64,
}

/// Validates an exact header before a store reads or allocates its snapshot section.
///
/// `max_snapshot_bytes` is the caller's configured snapshot byte budget, excluding
/// this header. A valid header does not establish physical EOF, snapshot validity,
/// namespace ownership, or durable publication. The WAL has no separate header.
pub fn decode_checkpoint_header(
    bytes: &[u8],
    max_snapshot_bytes: u64,
) -> Result<CheckpointHeader, DecodeError> {
    if bytes.len() != CHECKPOINT_HEADER_BYTES {
        return Err(DecodeError::InvalidLength {
            context: "checkpoint header",
            expected: CHECKPOINT_HEADER_BYTES,
            actual: bytes.len(),
        });
    }
    let mut input = Reader::new(bytes);
    if input.exact(8)? != CHECKPOINT_MAGIC {
        return Err(DecodeError::BadMagic {
            context: "checkpoint header",
        });
    }
    let version = input.u16()?;
    if version != FILELOG_FORMAT_VERSION {
        return Err(DecodeError::UnsupportedVersion {
            context: "checkpoint header",
            found: version,
        });
    }
    let flags = input.u16()?;
    if flags != 0 {
        return Err(DecodeError::ReservedFieldNonZero {
            field: "checkpoint.flags",
            value: u64::from(flags),
        });
    }
    let wal_offset = input.u64()?;
    let stored = input.u32()?;
    let computed = crc32c(&bytes[..20]);
    if stored != computed {
        return Err(DecodeError::ChecksumMismatch {
            context: "checkpoint header",
            stored,
            computed,
        });
    }
    if wal_offset < CHECKPOINT_HEADER_BYTES as u64 + MIN_SNAPSHOT_BYTES {
        return Err(DecodeError::InvalidWalOffset { found: wal_offset });
    }
    let snapshot_bytes = wal_offset - CHECKPOINT_HEADER_BYTES as u64;
    if snapshot_bytes > max_snapshot_bytes {
        return Err(DecodeError::LengthExceedsMaximum {
            field: "checkpoint.snapshot_bytes",
            declared: snapshot_bytes,
            max: max_snapshot_bytes,
        });
    }
    Ok(CheckpointHeader { wal_offset })
}

/// Encodes a checkpoint with a complete snapshot and an empty WAL.
///
/// Callers enforce their configured snapshot byte and record budgets before
/// publication. Append encoded transactions only after this immutable prefix;
/// the first transaction sequence is one. This does not publish or sync a file.
pub fn encode_checkpoint(
    generation: u64,
    checkpoint_id: &str,
    records: &[SnapshotRecord],
) -> Result<Vec<u8>, EncodeError> {
    let mut out = Writer::new();
    out.bytes(&[0; CHECKPOINT_HEADER_BYTES]);
    encode_snapshot_into(&mut out, generation, checkpoint_id, records)?;
    let mut bytes = out.finish();
    let wal_offset = u64::try_from(bytes.len()).map_err(|_| EncodeError::ArithmeticOverflow {
        context: "checkpoint WAL offset",
    })?;
    bytes[..8].copy_from_slice(CHECKPOINT_MAGIC);
    bytes[8..10].copy_from_slice(&FILELOG_FORMAT_VERSION.to_be_bytes());
    bytes[12..20].copy_from_slice(&wal_offset.to_be_bytes());
    let header_crc = crc32c(&bytes[..20]);
    bytes[20..24].copy_from_slice(&header_crc.to_be_bytes());
    Ok(bytes)
}

/// Decodes the immutable prefix and returns the absolute WAL start offset.
///
/// Input must contain the complete header and snapshot; any following WAL bytes
/// are left unexamined. A store can read the fixed header, validate its byte
/// budget with [`decode_checkpoint_header`], then read only through `wal_offset`.
/// It can release this input after decoding and scan the remaining file with
/// [`crate::scan_next_transaction`] one transaction at a time. This function
/// neither replays nor validates the WAL and never authorizes tail truncation.
pub fn decode_checkpoint_snapshot(
    bytes: &[u8],
    expected_namespace_digest: &[u8; 32],
    max_records: u32,
    max_snapshot_bytes: u64,
) -> Result<(Snapshot, usize), DecodeError> {
    let header_bytes = bytes
        .get(..CHECKPOINT_HEADER_BYTES)
        .ok_or(DecodeError::Truncated {
            needed: CHECKPOINT_HEADER_BYTES,
            available: bytes.len(),
        })?;
    let header = decode_checkpoint_header(header_bytes, max_snapshot_bytes)?;
    let wal_offset =
        usize::try_from(header.wal_offset).map_err(|_| DecodeError::ArithmeticOverflow {
            context: "checkpoint WAL offset to usize",
        })?;
    if wal_offset > bytes.len() {
        return Err(DecodeError::Truncated {
            needed: wal_offset,
            available: bytes.len(),
        });
    }
    let snapshot = decode_snapshot(
        &bytes[CHECKPOINT_HEADER_BYTES..wal_offset],
        expected_namespace_digest,
        max_records,
    )?;
    Ok((snapshot, wal_offset))
}
