// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Segment-specific error types.

use std::io;
use std::path::PathBuf;

use arrow_schema::ArrowError;
use thiserror::Error;

use super::types::StreamId;
use crate::record_bundle::SlotId;

/// Errors that can occur during segment operations.
#[derive(Debug, Error)]
pub enum SegmentError {
    /// I/O error during segment file operations.
    #[error("segment I/O error at {path:?}: {source}")]
    Io {
        /// Path to the segment file, if known.
        path: Option<PathBuf>,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// Arrow IPC encoding or decoding error.
    #[error("arrow IPC error: {source}")]
    Arrow {
        /// Underlying Arrow error.
        #[source]
        source: ArrowError,
    },

    /// Segment file has invalid or unrecognized format.
    #[error("invalid segment format: {message}")]
    InvalidFormat {
        /// Description of the format error.
        message: String,
    },

    /// Checksum mismatch indicating data corruption.
    #[error(
        "checksum mismatch in segment {path:?}: expected {expected:#010x}, got {actual:#010x}"
    )]
    ChecksumMismatch {
        /// Path to the corrupted segment file, if known.
        path: Option<PathBuf>,
        /// Expected CRC32 value.
        expected: u32,
        /// Actual computed CRC32 value.
        actual: u32,
    },

    /// Requested stream not found in segment.
    #[error("stream {stream_id:?} not found in segment")]
    StreamNotFound {
        /// The stream that was requested.
        stream_id: StreamId,
    },

    /// Requested slot not found in manifest entry.
    #[error("slot {slot_id:?} not present in bundle {bundle_index}")]
    SlotNotInBundle {
        /// The slot that was requested.
        slot_id: SlotId,
        /// Index of the bundle in the manifest.
        bundle_index: u32,
    },

    /// Segment file is truncated or incomplete.
    #[error("segment file truncated: expected {expected} bytes, found {actual}")]
    Truncated {
        /// Expected file size.
        expected: u64,
        /// Actual file size.
        actual: u64,
    },

    /// Stream accumulator is already finalized.
    #[error("stream accumulator already finalized")]
    AccumulatorFinalized,

    /// Batch schema does not match the stream's expected schema.
    #[error("schema mismatch in stream {stream_id:?}: expected schema {expected}, got {actual}")]
    SchemaMismatch {
        /// The stream where the mismatch occurred.
        stream_id: StreamId,
        /// String representation of the expected schema.
        expected: String,
        /// String representation of the actual batch schema.
        actual: String,
    },

    /// Open segment has no data to finalize.
    #[error("cannot finalize empty segment")]
    EmptySegment,
}

impl SegmentError {
    /// Creates an I/O error with an associated path.
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: Some(path.into()),
            source,
        }
    }

    /// Creates an I/O error without a path.
    #[must_use]
    pub fn io_no_path(source: io::Error) -> Self {
        Self::Io { path: None, source }
    }
}

impl From<ArrowError> for SegmentError {
    fn from(source: ArrowError) -> Self {
        Self::Arrow { source }
    }
}

impl From<io::Error> for SegmentError {
    fn from(source: io::Error) -> Self {
        Self::io_no_path(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_with_path_displays_correctly() {
        let err = SegmentError::io(
            "/data/segments/seg-001.arrow",
            io::Error::new(io::ErrorKind::NotFound, "file not found"),
        );
        let msg = err.to_string();
        assert!(msg.contains("seg-001.arrow"));
        assert!(msg.contains("I/O error"));
    }

    #[test]
    fn io_error_without_path_displays_correctly() {
        let err = SegmentError::io_no_path(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "access denied",
        ));
        let msg = err.to_string();
        assert!(msg.contains("None"));
    }

    #[test]
    fn checksum_mismatch_displays_hex_values() {
        let err = SegmentError::ChecksumMismatch {
            path: Some(PathBuf::from("/data/segments/seg-042.qseg")),
            expected: 0xDEADBEEF,
            actual: 0xCAFEBABE,
        };
        let msg = err.to_string();
        assert!(msg.contains("0xdeadbeef"));
        assert!(msg.contains("0xcafebabe"));
        assert!(msg.contains("seg-042.qseg"));
    }

    #[test]
    fn invalid_format_includes_message() {
        let err = SegmentError::InvalidFormat {
            message: "missing magic bytes".to_string(),
        };
        assert!(err.to_string().contains("missing magic bytes"));
    }

    #[test]
    fn arrow_error_conversion() {
        let arrow_err = ArrowError::ParseError("bad schema".into());
        let seg_err: SegmentError = arrow_err.into();
        assert!(matches!(seg_err, SegmentError::Arrow { .. }));
    }

    #[test]
    fn stream_not_found_displays_stream_id() {
        let err = SegmentError::StreamNotFound {
            stream_id: StreamId::new(7),
        };
        assert!(err.to_string().contains("7"));
    }

    #[test]
    fn slot_not_in_bundle_displays_details() {
        let err = SegmentError::SlotNotInBundle {
            slot_id: SlotId::new(3),
            bundle_index: 10,
        };
        let msg = err.to_string();
        assert!(msg.contains("3"));
        assert!(msg.contains("10"));
    }
}
