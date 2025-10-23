// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use datafusion::error::DataFusionError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error the engine will produce if it receives a pipeline expression that is not valid
    #[error("Invalid pipeline error: {reason}")]
    InvalidPipelineError { reason: String },

    #[error("Engine received invalid record batch: {reason}")]
    InvalidBatchError { reason: String },

    /// Error for syntax that should be valid but is not yet supported by this engine
    #[error("Operation not yet supported by columnar engine: {message}")]
    NotYetSupportedError { message: String },

    #[error("Error occurred in DataFusion: {0}")]
    DataFusion(#[from] DataFusionError),
}
