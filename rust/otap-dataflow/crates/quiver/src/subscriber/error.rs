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

    /// Segment not found.
    #[error("segment {segment_seq} not found")]
    SegmentNotFound {
        /// The segment sequence number that was not found.
        segment_seq: u64,
    },

    /// Segment I/O error.
    #[error("segment I/O error at {path}: {source}")]
    SegmentIo {
        /// Path to the segment file or directory.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },

    /// Registry is shutting down.
    #[error("subscriber registry is shutting down")]
    ShuttingDown,

    /// Progress file I/O error.
    #[error("progress file I/O error at {path}: {source}")]
    ProgressIo {
        /// Path to the progress file.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },

    /// Progress file corruption detected.
    #[error("progress file corrupted at {path}: {message}")]
    ProgressCorrupted {
        /// Path to the progress file.
        path: PathBuf,
        /// Description of the corruption.
        message: Cow<'static, str>,
    },
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

    /// Creates a new [`SubscriberError::SegmentNotFound`] error.
    #[must_use]
    pub fn segment_not_found(segment_seq: u64) -> Self {
        Self::SegmentNotFound { segment_seq }
    }

    /// Creates a new [`SubscriberError::SegmentIo`] error.
    #[must_use]
    pub fn segment_io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::SegmentIo {
            path: path.into(),
            source,
        }
    }

    /// Creates a new [`SubscriberError::ProgressIo`] error.
    #[must_use]
    pub fn progress_io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::ProgressIo {
            path: path.into(),
            source,
        }
    }

    /// Creates a new [`SubscriberError::ProgressCorrupted`] error.
    #[must_use]
    pub fn progress_corrupted(
        path: impl Into<PathBuf>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::ProgressCorrupted {
            path: path.into(),
            message: message.into(),
        }
    }
}
