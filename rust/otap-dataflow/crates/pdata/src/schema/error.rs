// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error types for OTAP schema validation.

use arrow::datatypes::DataType;

use crate::schema::schema::DataType as OtapDataType;

/// Result type for schema validation.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when validating a [`RecordBatch`](arrow::array::RecordBatch)
/// against an OTAP [`Schema`](crate::schema::schema::Schema) definition.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A column is present in the batch but not defined in the schema.
    #[error("Extraneous column `{name}`")]
    ExtraneousField {
        /// Name of the extraneous column.
        name: String,
    },

    /// One or more required columns are missing from the batch.
    #[error("Missing required columns {names:?}")]
    MissingRequiredFields {
        /// Names of the missing required columns.
        names: Vec<String>,
    },

    /// A column's Arrow data type does not match the OTAP schema definition.
    #[error("Column `{name}` type mismatch: expected {expected:?}, actual: {actual}")]
    FieldTypeMismatch {
        /// Name of the mismatched column.
        name: String,
        /// The type expected by the OTAP schema.
        expected: OtapDataType,
        /// The actual Arrow data type found.
        actual: DataType,
    },
}
