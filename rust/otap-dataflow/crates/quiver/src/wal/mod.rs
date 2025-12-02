use std::io;

use arrow_schema::ArrowError;
use thiserror::Error;

use crate::record_bundle::SlotId;

mod header;
mod writer;

pub(crate) use writer::{WalOffset, WalWriter, WalWriterOptions};

pub(crate) const WAL_MAGIC: &[u8; 10] = b"QUIVER\0WAL";
pub(crate) const ENTRY_TYPE_RECORD_BUNDLE: u8 = 0;

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
    #[error("arrow serialization error: {0}")]
    Arrow(#[from] ArrowError),
}
