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

use crate::otlp::attributes::store::AttributeValueType;
use crate::otlp::metric::MetricType;
use arrow::datatypes::DataType;
use arrow::error::ArrowError;
use num_enum::TryFromPrimitiveError;
use snafu::{Location, Snafu};
use std::backtrace::Backtrace;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Cannot find column: {}", name))]
    ColumnNotFound { name: String, backtrace: Backtrace },
    #[snafu(display(
        "Column {} data type mismatch, expect: {}, actual: {}",
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

    #[snafu(display("Currently attribute store value type: {} is not supported", type_name))]
    UnsupportedAttributeValue {
        type_name: String,
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

    #[snafu(display("Invalid List array data type, expect {}, actual {}", expect, actual))]
    InvalidListArray {
        expect: DataType,
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

    #[snafu(display("Failed to read record batch"))]
    ReadRecordBatch {
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

    #[snafu(display("Metric record not found"))]
    MetricRecordNotFound {
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
}
