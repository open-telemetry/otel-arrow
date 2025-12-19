// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for subscriber operations.

use std::borrow::Cow;
use std::io;
use std::path::PathBuf;

use super::types::InvalidSubscriberId;

/// Common result alias for subscriber operations.
pub type Result<T> = std::result::Result<T, SubscriberError>;

/// Errors that can occur during subscriber operations.
#[derive(Debug, thiserror::Error)]
pub enum SubscriberError {
    /// Invalid subscriber ID.
    #[error("invalid subscriber ID: {source}")]
    InvalidId {
        /// The underlying validation error.
        #[from]
        source: InvalidSubscriberId,
    },

    /// Subscriber not found.
    #[error("subscriber not found: {id}")]
    NotFound {
        /// The subscriber ID that was not found.
        id: String,
    },

    /// Subscriber already exists.
    #[error("subscriber already registered: {id}")]
    AlreadyExists {
        /// The subscriber ID that already exists.
        id: String,
    },

    /// Bundle not found or already resolved.
    #[error("bundle not available: {message}")]
    BundleNotAvailable {
        /// Description of why the bundle is not available.
        message: Cow<'static, str>,
    },

    /// Ack log I/O error.
    #[error("ack log I/O error at {path}: {source}")]
    AckLogIo {
        /// Path to the ack log file.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },

    /// Ack log corruption detected.
    #[error("ack log corrupted at {path}: {message}")]
    AckLogCorrupted {
        /// Path to the ack log file.
        path: PathBuf,
        /// Description of the corruption.
        message: Cow<'static, str>,
    },

    /// Segment not found.
    #[error("segment {segment_seq} not found")]
    SegmentNotFound {
        /// The segment sequence number that was not found.
        segment_seq: u64,
    },

    /// Registry is shutting down.
    #[error("subscriber registry is shutting down")]
    ShuttingDown,
}

impl SubscriberError {
    /// Creates a new [`SubscriberError::NotFound`] error.
    #[must_use]
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound { id: id.into() }
    }

    /// Creates a new [`SubscriberError::AlreadyExists`] error.
    #[must_use]
    pub fn already_exists(id: impl Into<String>) -> Self {
        Self::AlreadyExists { id: id.into() }
    }

    /// Creates a new [`SubscriberError::BundleNotAvailable`] error.
    #[must_use]
    pub fn bundle_not_available(message: impl Into<Cow<'static, str>>) -> Self {
        Self::BundleNotAvailable {
            message: message.into(),
        }
    }

    /// Creates a new [`SubscriberError::AckLogIo`] error.
    #[must_use]
    pub fn ack_log_io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::AckLogIo {
            path: path.into(),
            source,
        }
    }

    /// Creates a new [`SubscriberError::AckLogCorrupted`] error.
    #[must_use]
    pub fn ack_log_corrupted(
        path: impl Into<PathBuf>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::AckLogCorrupted {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Creates a new [`SubscriberError::SegmentNotFound`] error.
    #[must_use]
    pub fn segment_not_found(segment_seq: u64) -> Self {
        Self::SegmentNotFound { segment_seq }
    }
}
