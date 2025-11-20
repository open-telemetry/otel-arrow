// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for the columnar query engine.

use arrow::error::ArrowError;
use datafusion::error::DataFusionError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error which the pipeline can return if there was a problem encountered during execution
    #[error("Pipeline execution error: {cause}")]
    ExecutionError { cause: String },

    /// Error for syntax/query state that should be valid but is not yet supported by this engine
    #[error("Operation not yet supported by columnar engine: {message}")]
    NotYetSupportedError { message: String },
}

impl From<ArrowError> for Error {
    fn from(error: ArrowError) -> Self {
        Self::ExecutionError {
            cause: format!("ArrowError: {error:?}"),
        }
    }
}

impl From<DataFusionError> for Error {
    fn from(error: DataFusionError) -> Self {
        Self::ExecutionError {
            cause: format!("DataFusionError: {error:?}"),
        }
    }
}
