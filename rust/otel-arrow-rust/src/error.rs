// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Error and result types

use crate::otlp::attributes::store::AttributeValueType;
use crate::otlp::metrics::MetricType;
use arrow::datatypes::DataType;
use arrow::error::ArrowError;
use num_enum::TryFromPrimitiveError;
use snafu::{Location, Snafu};
use std::{backtrace::Backtrace, num::TryFromIntError};

/// Result type
pub type Result<T> = std::result::Result<T, Error>;

#[allow(missing_docs)]
#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Cannot find column: {}", name))]
    ColumnNotFound { name: String, backtrace: Backtrace },
    #[snafu(display(
        "Column `{}` data type mismatch, expect: {}, actual: {}",
        name,
        expect,
        actual
    ))]
    ColumnDataTypeMismatch {
        name: String,
        expect: DataType,
        actual: DataType,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Cannot recognize metric type: {}", metric_type))]
    UnrecognizedMetricType {
        metric_type: i32,
        #[snafu(source)]
        error: TryFromPrimitiveError<MetricType>,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Unable to handle empty metric type"))]
    EmptyMetricType {
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Cannot recognize attribute value type"))]
    UnrecognizedAttributeValueType {
        #[snafu(source)]
        error: TryFromPrimitiveError<AttributeValueType>,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid bytes for serialized attribute value"))]
    InvalidSerializedAttributeBytes {
        source: ciborium::de::Error<std::io::Error>,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid serialized integer attribute value"))]
    InvalidSerializedIntAttributeValue {
        source: TryFromIntError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(
        "Invalid serialized map key type, expected: String, actual: {:?}",
        actual
    ))]
    InvalidSerializedMapKeyType {
        actual: ciborium::Value,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Serialized attribute {:?} is not supported", actual))]
    UnsupportedSerializedAttributeValue {
        actual: ciborium::Value,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid exemplar data, message: {}", message))]
    InvalidExemplarData {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid span id in exemplar data, message: {}", message))]
    InvalidSpanId {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid trace id in exemplar data, message: {}", message))]
    InvalidTraceId {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid trace id in exemplar data, message: {}", message))]
    InvalidQuantileType {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(
        "Invalid List array data type, expect one of {:?}, actual {}",
        expect_oneof,
        actual
    ))]
    InvalidListArray {
        expect_oneof: Vec<DataType>,
        actual: DataType,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid attribute transform: {}", reason))]
    InvalidAttributeTransform { reason: String },

    #[snafu(display("Unsupported parent id type. Expected u16 or u32, got: {}", actual))]
    UnsupportedParentIdType {
        actual: DataType,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Unsupported payload type, got: {}", actual))]
    UnsupportedPayloadType {
        actual: i32,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to build stream reader"))]
    BuildStreamReader {
        #[snafu(source)]
        source: ArrowError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to build stream writer"))]
    BuildStreamWriter {
        #[snafu(source)]
        source: ArrowError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to read record batch"))]
    ReadRecordBatch {
        #[snafu(source)]
        source: ArrowError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to write record batch"))]
    WriteRecordBatch {
        #[snafu(source)]
        source: ArrowError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to batch OTAP data"))]
    Batching {
        #[snafu(source)]
        source: ArrowError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Batch is empty"))]
    EmptyBatch {
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Log record not found"))]
    LogRecordNotFound {
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Metric record not found"))]
    MetricRecordNotFound {
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Span record not found"))]
    SpanRecordNotFound {
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Record batch is in unexpected state. reason: {}", reason))]
    UnexpectedRecordBatchState {
        reason: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(
        "Unsupported dictionary key type, expect one of {:?}, actual {}",
        expect_oneof,
        actual
    ))]
    UnsupportedDictionaryKeyType {
        expect_oneof: Vec<DataType>,
        actual: DataType,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(
        "Unsupported dictionary value type. expect {:?}, actual {}",
        expect_oneof,
        actual
    ))]
    UnsupportedDictionaryValueType {
        expect_oneof: Vec<DataType>,
        actual: DataType,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Unsupported string column type, given: {}", data_type))]
    UnsupportedStringColumnType {
        data_type: DataType,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Unsupported string dictionary key type, given: {}", data_type))]
    UnsupportedStringDictKeyType {
        data_type: DataType,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Found duplicate field name: {}", name))]
    DuplicateFieldName {
        name: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(
        "Invalid byte slice for ID, expect len: {} ,given len: {}",
        expected,
        given
    ))]
    InvalidId {
        expected: usize,
        given: usize,
        #[snafu(implicit)]
        location: Location,
    },
}
