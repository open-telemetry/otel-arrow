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
    /// Wrapper for WAL-specific failures.
    #[error("wal error: {source}")]
    Wal {
        /// Underlying WAL error.
        #[from]
        source: crate::wal::WalError,
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
}
