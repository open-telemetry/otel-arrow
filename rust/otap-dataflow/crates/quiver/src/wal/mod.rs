// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::io;

use arrow_schema::ArrowError;
use thiserror::Error;

use crate::record_bundle::SlotId;

mod header;
mod reader;
#[cfg(test)]
mod tests;
mod truncate_sidecar;
mod writer;

// Keep reader exports visible even though only tests consume them today.
#[allow(unused_imports)]
pub(crate) use reader::{DecodedWalSlot, WalReader, WalRecordBundle, WalTruncateCursor};
// Writer is used broadly soon; suppress warnings while integration lands.
#[allow(unused_imports)]
pub(crate) use writer::{WalOffset, WalWriter, WalWriterOptions};

pub(crate) const WAL_MAGIC: &[u8; 10] = b"QUIVER\0WAL";
pub(crate) const ENTRY_TYPE_RECORD_BUNDLE: u8 = 0;
pub(crate) const ENTRY_HEADER_LEN: usize = 1 + 8 + 8 + 8;
pub(crate) const SCHEMA_FINGERPRINT_LEN: usize = 32;
pub(crate) const SLOT_HEADER_LEN: usize = 2 + SCHEMA_FINGERPRINT_LEN + 4 + 4;

pub(crate) type WalResult<T> = Result<T, WalError>;

/// Errors produced while reading or writing WAL data.
#[derive(Error, Debug)]
pub enum WalError {
    /// Underlying filesystem failure.
    #[error("wal io error: {0}")]
    Io(#[from] io::Error),
    /// File header contained unexpected bytes.
    #[error("invalid wal header: {0}")]
    InvalidHeader(&'static str),
    /// Slot id exceeded the current bitmap encoding.
    #[error("slot id {0:?} is out of supported bitmap range (>= 64)")]
    SlotOutOfRange(SlotId),
    /// Payload row count cannot be encoded as `u32`.
    #[error("row count {0} exceeds u32::MAX")]
    RowCountOverflow(usize),
    /// Serialized payload exceeds allowed size.
    #[error("payload length {0} exceeds u32::MAX")]
    PayloadTooLarge(usize),
    /// Entry body is larger than the framing supports.
    #[error("entry body length {0} exceeds u32::MAX")]
    EntryTooLarge(usize),
    /// Ingestion timestamp could not be normalized.
    #[error("invalid ingestion timestamp")]
    InvalidTimestamp,
    /// Encountered an unexpected EOF while parsing.
    #[error("wal truncated while reading {0}")]
    UnexpectedEof(&'static str),
    /// CRC mismatch detected during validation.
    #[error("wal crc mismatch: stored {stored:#010x} computed {computed:#010x}")]
    CrcMismatch {
        /// CRC value persisted alongside the entry.
        stored: u32,
        /// CRC recomputed from the decoded entry.
        computed: u32,
    },
    /// Entry type not supported by this build.
    #[error("unsupported wal entry type {0}")]
    UnsupportedEntry(u8),
    /// Entry body failed structural validation.
    #[error("invalid wal entry: {0}")]
    InvalidEntry(&'static str),
    /// Existing WAL header does not match expected segment configuration.
    #[error("segment config mismatch: expected {expected:02x?}, found {found:02x?}")]
    SegmentConfigMismatch {
        /// The hash the caller expected.
        expected: [u8; 16],
        /// The hash stored in the WAL header.
        found: [u8; 16],
    },
    /// Truncate sidecar contains invalid or corrupted bytes.
    #[error("invalid truncate sidecar: {0}")]
    InvalidTruncateSidecar(&'static str),
    /// Arrow serialization/deserialization failure.
    #[error("arrow serialization error: {0}")]
    Arrow(#[from] ArrowError),
    /// Writer cannot proceed because configured capacity limits were reached.
    #[error("wal at capacity: {0}")]
    WalAtCapacity(&'static str),
}
