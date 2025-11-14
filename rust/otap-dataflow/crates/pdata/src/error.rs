// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Error and result types

use crate::otlp::metrics::MetricType;
use crate::{
    otlp::attributes::AttributeValueType, proto::opentelemetry::arrow::v1::ArrowPayloadType,
};
use arrow::datatypes::DataType;
use arrow::error::ArrowError;
use num_enum::TryFromPrimitiveError;
use std::num::TryFromIntError;

/// Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Errors related to OTAP or OTLP pipeline data
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Cannot find column: {}", name)]
    ColumnNotFound { name: String },

    #[error(
        "Column `{}` data type mismatch, expect: {}, actual: {}",
        name,
        expect,
        actual
    )]
    ColumnDataTypeMismatch {
        name: String,
        expect: DataType,
        actual: DataType,
    },

    #[error("Failed to compare two columns with unequal length")]
    ColumnLengthMismatch { source: ArrowError },

    #[error("Cannot recognize metric type: {metric_type}: {error}")]
    UnrecognizedMetricType {
        metric_type: i32,
        error: TryFromPrimitiveError<MetricType>,
    },

    #[error("Unable to handle empty metric type")]
    EmptyMetricType,

    #[error("Cannot recognize attribute value type")]
    UnrecognizedAttributeValueType {
        #[from]
        error: TryFromPrimitiveError<AttributeValueType>,
    },

    #[error("Invalid bytes for serialized attribute value")]
    InvalidSerializedAttributeBytes {
        source: ciborium::de::Error<std::io::Error>,
    },

    #[error("Invalid serialized integer attribute value")]
    InvalidSerializedIntAttributeValue { source: TryFromIntError },

    #[error(
        "Invalid serialized map key type, expected: String, actual: {:?}",
        actual
    )]
    InvalidSerializedMapKeyType { actual: ciborium::Value },

    #[error("Serialized attribute {:?} is not supported", actual)]
    UnsupportedSerializedAttributeValue { actual: ciborium::Value },

    #[error("Invalid exemplar data, message: {}", message)]
    InvalidExemplarData { message: String },

    #[error("Invalid span id in exemplar data, message: {}", message)]
    InvalidSpanId { message: String },

    #[error("Invalid trace id in exemplar data, message: {}", message)]
    InvalidTraceId { message: String },

    #[error("Invalid trace id in exemplar data, message: {}", message)]
    InvalidQuantileType { message: String },

    #[error(
        "Invalid List array data type, expect one of {:?}, actual {}",
        expect_oneof,
        actual
    )]
    InvalidListArray {
        expect_oneof: Vec<DataType>,
        actual: DataType,
    },

    #[error("Invalid attribute transform: {}", reason)]
    InvalidAttributeTransform { reason: String },

    #[error("Unsupported parent id type. Expected u16 or u32, got: {}", actual)]
    UnsupportedParentIdType { actual: DataType },

    #[error("Unsupported payload type, got: {}", actual)]
    UnsupportedPayloadType { actual: i32 },

    #[error("Failed to build stream reader")]
    BuildStreamReader { source: ArrowError },

    #[error("Failed to build stream writer")]
    BuildStreamWriter { source: ArrowError },

    #[error("Failed to read record batch")]
    ReadRecordBatch { source: ArrowError },

    #[error("Failed to write record batch")]
    WriteRecordBatch { source: ArrowError },

    #[error("Failed to batch OTAP data")]
    Batching { source: ArrowError },

    #[error("Batch is empty")]
    EmptyBatch,

    #[error("RecordBatch not found: {:?}", payload_type)]
    RecordBatchNotFound { payload_type: ArrowPayloadType },
    #[error("Log record not found")]
    LogRecordNotFound,

    #[error("Metric record not found")]
    MetricRecordNotFound,

    #[error("Span record not found")]
    SpanRecordNotFound,

    #[error("Record batch is in unexpected state. reason: {}", reason)]
    UnexpectedRecordBatchState { reason: String },

    #[error(
        "Unsupported dictionary key type, expect one of {:?}, actual {}",
        expect_oneof,
        actual
    )]
    UnsupportedDictionaryKeyType {
        expect_oneof: Vec<DataType>,
        actual: DataType,
    },

    #[error(
        "Unsupported dictionary value type. expect {:?}, actual {}",
        expect_oneof,
        actual
    )]
    UnsupportedDictionaryValueType {
        expect_oneof: Vec<DataType>,
        actual: DataType,
    },

    #[error("Unsupported string column type, given: {}", data_type)]
    UnsupportedStringColumnType { data_type: DataType },

    #[error("Unsupported string dictionary key type, given: {}", data_type)]
    UnsupportedStringDictKeyType { data_type: DataType },

    #[error("Found duplicate field name: {}", name)]
    DuplicateFieldName { name: String },

    #[error(
        "Invalid byte slice for ID, expect len: {} ,given len: {}",
        expected,
        given
    )]
    InvalidId { expected: usize, given: usize },

    #[error("Mixed signals")]
    MixedSignals,

    #[error("Encoding error: {}", error)]
    Encoding {
        #[from]
        error: crate::encode::Error,
    },
}
