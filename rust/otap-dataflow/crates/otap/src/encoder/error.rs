// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! this module contains definitions for the various errors that can happen encoding OTAP data

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An arrow error occurred encoding error record batch: {0}")]
    ArrowError(#[from] arrow::error::ArrowError),

    #[error("An error occured serializing value as CBOR: {0}")]
    CborError(#[from] ciborium::ser::Error<std::io::Error>),
}
