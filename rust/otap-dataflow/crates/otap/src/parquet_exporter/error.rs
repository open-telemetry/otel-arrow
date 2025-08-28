// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Definition of errors that could happen when exporting OTAP batches to Parquet
#[derive(thiserror::Error, Debug)]
pub enum ParquetExporterError {
    #[error("Invalid record batch: {error}")]
    InvalidRecordBatch { error: String },

    #[error("Unknown error occurred: {error}")]
    UnknownError { error: String },
}
