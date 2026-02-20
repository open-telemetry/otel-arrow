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
    /// Raised when an operation is cancelled, typically due to shutdown.
    ///
    /// This is a graceful cancellation, not an error. Operations that receive
    /// this should clean up and return without considering it a failure.
    #[error("operation cancelled: {reason}")]
    Cancelled {
        /// Reason for the cancellation (e.g., "shutdown requested").
        reason: Cow<'static, str>,
    },
    /// Raised when storage capacity is exhausted (backpressure signal).
    ///
    /// Callers should slow down or pause ingestion until space is reclaimed.
    /// This is not a fatal error—retry after cleanup or subscriber catch-up.
    #[error("storage at capacity: {available} bytes available (soft cap: {soft_cap})")]
    StorageAtCapacity {
        /// Bytes currently available before hitting the soft cap.
        available: u64,
        /// The configured soft cap (ingest threshold).
        soft_cap: u64,
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
    pub const fn unimplemented(context: &'static str) -> Self {
        Self::Unimplemented { context }
    }

    /// Helper for creating [`QuiverError::Cancelled`] values.
    #[must_use]
    pub fn cancelled(reason: impl Into<Cow<'static, str>>) -> Self {
        Self::Cancelled {
            reason: reason.into(),
        }
    }

    /// Returns `true` if this error indicates a graceful cancellation.
    ///
    /// Cancelled operations should clean up and return without logging errors.
    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled { .. })
    }

    /// Returns `true` if this is a backpressure signal (storage at capacity).
    ///
    /// Callers can use this to distinguish recoverable capacity errors from
    /// fatal errors and implement appropriate backoff strategies.
    ///
    /// This returns `true` for both disk budget capacity (`StorageAtCapacity`)
    /// and WAL capacity (`WalAtCapacity`) errors.
    #[must_use]
    pub const fn is_at_capacity(&self) -> bool {
        match self {
            Self::StorageAtCapacity { .. } => true,
            Self::Wal { source } => source.is_at_capacity(),
            _ => false,
        }
    }

    /// Returns `true` if this error is recoverable (retry may succeed).
    ///
    /// Recoverable errors include:
    /// - Capacity errors (retry after space is freed)
    /// - Cancelled operations (not really an error)
    ///
    /// Non-recoverable errors include configuration errors, I/O failures,
    /// and data corruption.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        self.is_at_capacity() || self.is_cancelled()
    }
}
