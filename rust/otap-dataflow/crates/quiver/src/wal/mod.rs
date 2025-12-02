use std::io;

use arrow_schema::ArrowError;
use thiserror::Error;

use crate::record_bundle::SlotId;

mod header;
mod reader;
mod writer;

pub(crate) use reader::{DecodedWalSlot, WalReader, WalRecordBundle, WalTruncateCursor};
pub(crate) use writer::{WalOffset, WalWriter, WalWriterOptions};

pub(crate) const WAL_MAGIC: &[u8; 10] = b"QUIVER\0WAL";
pub(crate) const ENTRY_TYPE_RECORD_BUNDLE: u8 = 0;
pub(crate) const ENTRY_HEADER_LEN: usize = 1 + 8 + 8 + 8;
pub(crate) const SCHEMA_FINGERPRINT_LEN: usize = 32;
pub(crate) const SLOT_HEADER_LEN: usize = 2 + SCHEMA_FINGERPRINT_LEN + 4 + 4;

pub(crate) type WalResult<T> = Result<T, WalError>;

#[derive(Error, Debug)]
pub(crate) enum WalError {
    #[error("wal io error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid wal header: {0}")]
    InvalidHeader(&'static str),
    #[error("slot id {0:?} is out of supported bitmap range (>= 64)")]
    SlotOutOfRange(SlotId),
    #[error("row count {0} exceeds u32::MAX")]
    RowCountOverflow(usize),
    #[error("payload length {0} exceeds u32::MAX")]
    PayloadTooLarge(usize),
    #[error("entry body length {0} exceeds u32::MAX")]
    EntryTooLarge(usize),
    #[error("invalid ingestion timestamp")]
    InvalidTimestamp,
    #[error("wal truncated while reading {0}")]
    UnexpectedEof(&'static str),
    #[error("wal crc mismatch: stored {stored:#010x} computed {computed:#010x}")]
    CrcMismatch { stored: u32, computed: u32 },
    #[error("unsupported wal entry type {0}")]
    UnsupportedEntry(u8),
    #[error("invalid wal entry: {0}")]
    InvalidEntry(&'static str),
    #[error("arrow serialization error: {0}")]
    Arrow(#[from] ArrowError),
}
