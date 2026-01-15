// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types surfaced by the Quiver crate.

use std::borrow::Cow;

/// Common result alias for Quiver operations.
pub type Result<T> = std::result::Result<T, QuiverError>;

/// Errors that can be produced by Quiver APIs.
#[derive(Debug, thiserror::Error)]
pub enum QuiverError {
    /// Raised when a caller provides an invalid configuration.
    #[error("invalid configuration: {message}")]
    InvalidConfig {
        /// Human-readable explanation of the validation failure.
        message: Cow<'static, str>,
    },
    /// Placeholder for functionality that has not yet been implemented.
    #[error("feature not implemented: {context}")]
    Unimplemented {
        /// Context string identifying the missing component.
        context: &'static str,
    },
    /// Raised when storage capacity is exhausted (backpressure signal).
    ///
    /// Callers should slow down or pause ingestion until space is reclaimed.
    /// This is not a fatal error—retry after cleanup or subscriber catch-up.
    #[error(
        "storage at capacity: requested {requested} bytes, only {available} available (cap: {cap})"
    )]
    StorageAtCapacity {
        /// Bytes requested for the operation.
        requested: u64,
        /// Bytes currently available before hitting the cap.
        available: u64,
        /// The configured storage cap.
        cap: u64,
    },
    /// Wrapper for WAL-specific failures.
    #[error("wal error: {source}")]
    Wal {
        /// Underlying WAL error.
        #[from]
        source: crate::wal::WalError,
    },
    /// Wrapper for segment-specific failures.
    #[error("segment error: {source}")]
    Segment {
        /// Underlying segment error.
        #[from]
        source: crate::segment::SegmentError,
    },
}

impl QuiverError {
    /// Helper for creating [`QuiverError::InvalidConfig`] values.
    #[must_use]
    pub fn invalid_config(message: impl Into<Cow<'static, str>>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Helper for creating [`QuiverError::Unimplemented`] values.
    #[must_use]
    pub fn unimplemented(context: &'static str) -> Self {
        Self::Unimplemented { context }
    }

    /// Returns `true` if this is a backpressure signal (storage at capacity).
    ///
    /// Callers can use this to distinguish recoverable capacity errors from
    /// fatal errors and implement appropriate backoff strategies.
    ///
    /// This returns `true` for both disk budget capacity (`StorageAtCapacity`)
    /// and WAL capacity (`WalAtCapacity`) errors.
    #[must_use]
    pub fn is_at_capacity(&self) -> bool {
        match self {
            Self::StorageAtCapacity { .. } => true,
            Self::Wal { source } => source.is_at_capacity(),
            _ => false,
        }
    }
}
