// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Write-Ahead Log (WAL) for Quiver crash recovery.
//!
//! This module provides durable, append-only storage for Arrow record batches.
//! On crash, the WAL replays uncommitted data to restore in-memory state.
//!
//! # Quick Start
//!
//! ```ignore
//! // Writing
//! let mut writer = WalWriter::open(options)?;
//! let offset = writer.append_bundle(&bundle)?;
//!
//! // Reading (for replay)
//! let mut reader = WalReader::open(&path)?;
//! let mut cursor = WalConsumerCursor::default();
//! for entry in reader.iter_from(0)? {
//!     let bundle = entry?;
//!     // ... rebuild state from bundle ...
//!     cursor.increment(&bundle);  // in-memory only
//! }
//!
//! // Persisting cursor (after downstream confirms durability)
//! writer.persist_cursor(&cursor)?;  // persists + enables cleanup
//! ```
//!
//! # Module Organization
//!
//! | File                    | Purpose                                          |
//! |-------------------------|--------------------------------------------------|
//! | `writer.rs`             | Append entries, rotate files, manage cursors     |
//! | `reader.rs`             | Iterate entries, decode payloads, track progress |
//! | `header.rs`             | WAL file header format (magic, version, config)  |
//! | `cursor_sidecar.rs`     | Crash-safe WAL cursor persistence                |
//! | `tests.rs`              | Integration tests and crash simulation           |
//!
//! # On-Disk Layout
//!
//! ```text
//! wal/
//! ├── quiver.wal           # Active WAL file (append target)
//! ├── quiver.wal.1         # Rotated file (oldest)
//! ├── quiver.wal.2         # Rotated file
//! └── quiver.wal.cursor    # Consumer progress (24 bytes, CRC-protected)
//! ```
//!
//! # Key Concepts
//!
//! - **Entry**: One [`RecordBundle`] serialized with CRC32 integrity check
//! - **Rotation**: When the active file exceeds `rotation_target_bytes`, it's
//!   renamed to `quiver.wal.N` and a fresh file starts
//! - **Cursor**: Consumers call [`WalConsumerCursor::increment()`] while
//!   iterating (in-memory), then [`WalWriter::persist_cursor()`] to persist
//! - **Purge**: Rotated files are deleted once fully covered by the cursor
//!
//! See [`writer`] module docs for detailed lifecycle documentation.

use std::io;

use arrow_schema::ArrowError;
use thiserror::Error;

use crate::record_bundle::SlotId;

mod cursor_sidecar;
mod header;
mod reader;
#[cfg(test)]
mod tests;
mod writer;

// Keep reader exports visible even though only tests consume them today.
#[allow(unused_imports)]
pub(crate) use reader::{DecodedWalSlot, WalConsumerCursor, WalReader, WalRecordBundle};
// Writer is used broadly soon; suppress warnings while integration lands.
#[allow(unused_imports)]
pub(crate) use writer::{FlushPolicy, WalOffset, WalWriter, WalWriterOptions};
// Re-export for integration tests that need to verify cursor state.
#[allow(unused_imports)]
pub(crate) use cursor_sidecar::CursorSidecar;

// ─────────────────────────────────────────────────────────────────────────────
// WAL Format Constants
//
// See ARCHITECTURE.md § "Write-Ahead Log" for the full on-disk layout.
// ─────────────────────────────────────────────────────────────────────────────

/// Magic bytes identifying a Quiver WAL file.
///
/// The file header starts with these 10 bytes: `b"QUIVER\0WAL"`.
/// See ARCHITECTURE.md: "File header: fixed-width preamble (`b\"QUIVER\\0WAL\"`)"
pub(crate) const WAL_MAGIC: &[u8; 10] = b"QUIVER\0WAL";

/// Entry type marker for a serialized [`RecordBundle`].
///
/// Currently the only defined entry type. Future versions may add additional
/// types (e.g., for schema evolution or control records).
/// See ARCHITECTURE.md: "Entry header (`u8 entry_type`, currently `0 = RecordBundle`)"
pub(crate) const ENTRY_TYPE_RECORD_BUNDLE: u8 = 0;

/// Size of the entry header in bytes: `entry_type(1) + timestamp(8) + sequence(8) + bitmap(8)`.
///
/// Layout: `{ u8 entry_type, i64 ingestion_ts_nanos, u64 per_core_sequence, u64 slot_bitmap }`
/// See ARCHITECTURE.md § "Framed entries" for the complete entry structure.
pub(crate) const ENTRY_HEADER_LEN: usize = 1 + 8 + 8 + 8;

/// Size of a schema fingerprint (BLAKE3 truncated to 256 bits).
pub(crate) const SCHEMA_FINGERPRINT_LEN: usize = 32;

/// Size of per-slot metadata: `payload_type_id(2) + fingerprint(32) + row_count(4) + payload_len(4)`.
///
/// Layout: `{ u16 payload_type_id, [u8;32] schema_fingerprint, u32 row_count, u32 payload_len }`
/// See ARCHITECTURE.md § "Framed entries" → SlotMeta block.
pub(crate) const SLOT_HEADER_LEN: usize = 2 + SCHEMA_FINGERPRINT_LEN + 4 + 4;

/// Maximum allowed rotation target size (256 MiB).
///
/// This bounds the maximum size of any single WAL file and, by extension, the
/// maximum size of any single entry. Both writer and reader enforce this limit:
/// - Writer clamps `rotation_target_bytes` to this value
/// - Reader rejects entries exceeding this size
///
/// 256 MiB is generous for telemetry workloads while preventing excessive
/// memory allocation from corrupted or malicious WAL files.
pub(crate) const MAX_ROTATION_TARGET_BYTES: u64 = 256 * 1024 * 1024;

pub(crate) type WalResult<T> = Result<T, WalError>;

/// Errors produced while reading or writing WAL data.
///
/// Most variants include context about where the failure occurred.
/// [`WalError::Io`] wraps underlying filesystem errors.
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
    /// Cursor sidecar contains invalid or corrupted bytes.
    #[error("invalid cursor sidecar: {0}")]
    InvalidCursorSidecar(&'static str),
    /// Consumer cursor failed validation.
    #[error("invalid consumer cursor: {0}")]
    InvalidConsumerCursor(&'static str),
    /// Arrow serialization/deserialization failure.
    #[error("arrow serialization error: {0}")]
    Arrow(#[from] ArrowError),
    /// Writer cannot proceed because configured capacity limits were reached.
    #[error("wal at capacity: {0}")]
    WalAtCapacity(&'static str),
    /// Configuration parameter is invalid.
    #[error("invalid wal configuration: {0}")]
    InvalidConfig(&'static str),
    /// Test-only failure that simulates a crash at a specific point.
    #[error("wal crash injected: {0}")]
    InjectedCrash(&'static str),
}

impl WalError {
    /// Returns `true` if this is a capacity/backpressure error.
    ///
    /// WAL capacity errors are recoverable—the caller should wait for
    /// segment finalization to advance the cursor and reclaim space.
    #[must_use]
    pub fn is_at_capacity(&self) -> bool {
        matches!(self, Self::WalAtCapacity(_))
    }
}
