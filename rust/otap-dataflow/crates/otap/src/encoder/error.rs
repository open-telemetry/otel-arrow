// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! this module contains definitions for the various errors that can happen encoding OTAP data

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An arrow error occurred encoding error record batch: {0}")]
    ArrowError(#[from] arrow::error::ArrowError),

    #[error("An error occurred serializing value as CBOR: {error}")]
    CborError {
        /// The error that occurred
        error: String,
    },
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
