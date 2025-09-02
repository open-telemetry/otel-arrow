// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! this module contains definitions for the various errors that can happen encoding OTAP data

/// Result type with encoder Error type
pub type Result<T> = std::result::Result<T, Error>;

/// All errors that can occur when encoding OTAP data
#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    /// Wrapper for errors that occurred in arrow-rs
    #[error("An arrow error occurred encoding error record batch: {0}")]
    ArrowError(#[from] arrow::error::ArrowError),

    /// Wrapper for errors related to encoding attributes as CBOR
    #[error("An error occurred serializing value as CBOR: {error}")]
    CborError {
        /// The error that occurred
        error: String,
    },

    /// u16 underflow error
    #[error("An error occurred packing more than 2**16 - 1 entries into a record batch")]
    U16OverflowError,

    /// u32 underflow error
    #[error("An error occurred packing more than 2**32 - 1 entries into a record batch")]
    U32OverflowError,
}

impl From<ciborium::ser::Error<std::io::Error>> for Error {
    fn from(e: ciborium::ser::Error<std::io::Error>) -> Self {
        Self::CborError {
            error: format!("{e}"),
        }
    }
}

impl From<serde_cbor::Error> for Error {
    fn from(e: serde_cbor::Error) -> Self {
        Self::CborError {
            error: format!("{e}"),
        }
    }
}
