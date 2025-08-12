// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains builders for record batches for logs.

use std::sync::Arc;

use arrow::{
    array::{Array, NullBufferBuilder, RecordBatch, StructArray},
    datatypes::{Field, Fields, Schema},
    error::ArrowError,
};

use crate::{
    encode::record::array::{
        ArrayAppend, ArrayAppendNulls, ArrayAppendSlice, ArrayAppendStr, ArrayOptions,
        BinaryArrayBuilder, CheckedArrayAppendSlice, FixedSizeBinaryArrayBuilder,
        Float64ArrayBuilder, Int32ArrayBuilder, Int64ArrayBuilder, StringArrayBuilder,
        TimestampNanosecondArrayBuilder, UInt8ArrayBuilder, UInt16ArrayBuilder, UInt32ArrayBuilder,
        boolean::{AdaptiveBooleanArrayBuilder, BooleanBuilderOptions},
        dictionary::DictionaryOptions,
    },
    otlp::attributes::store::AttributeValueType,
    schema::{FieldExt, consts},
};

/// Record batch builder for logs
pub struct LogsRecordBatchBuilder {
    id: UInt16ArrayBuilder,

    /// the builder for the resource struct for this log record batch
    pub resource: ResourceBuilder,

    /// the builder for the scope struct for this log record batch
    pub scope: ScopeBuilder,

    /// the builder for the body of the log record
    pub body: LogsBodyBuilder,

    schema_url: StringArrayBuilder,
    time_unix_nano: TimestampNanosecondArrayBuilder,
    observed_time_unix_nano: TimestampNanosecondArrayBuilder,
    severity_number: Int32ArrayBuilder,
    severity_text: StringArrayBuilder,
    dropped_attributes_count: UInt32ArrayBuilder,
    flags: UInt32ArrayBuilder,
    trace_id: FixedSizeBinaryArrayBuilder,
    span_id: FixedSizeBinaryArrayBuilder,
    // TODO event_name https://github.com/open-telemetry/otel-arrow/issues/422ame
}

impl LogsRecordBatchBuilder {
    /// Create a new instance of `LogRecordBatchBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self {
            resource: ResourceBuilder::new(),
            scope: ScopeBuilder::new(),
            body: LogsBodyBuilder::new(),
            id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            schema_url: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            observed_time_unix_nano: TimestampNanosecondArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            severity_number: Int32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            severity_text: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            flags: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            trace_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: true,
                    dictionary_options: Some(DictionaryOptions::dict8()),
                    ..Default::default()
                },
                16,
            ),
            span_id: FixedSizeBinaryArrayBuilder::new_with_args(
                ArrayOptions {
                    optional: true,
                    dictionary_options: Some(DictionaryOptions::dict8()),
                    ..Default::default()
                },
                8,
            ),
        }
    }

    /// append a value to the `id` array
    pub fn append_id(&mut self, val: Option<u16>) {
        if let Some(val) = val {
            self.id.append_value(&val);
        } else {
            self.id.append_null();
        }
    }

    /// append a value to the `time_unix_nano` array
    pub fn append_time_unix_nano(&mut self, val: Option<i64>) {
        let val = val.unwrap_or(0);
        self.time_unix_nano.append_value(&val);
    }

    /// append a value to the `observed_time_unix_nano` array
    pub fn append_observed_time_unix_nano(&mut self, val: Option<i64>) {
        let val = val.unwrap_or(0);
        self.observed_time_unix_nano.append_value(&val);
    }

    /// append a value to the `severity_number` array
    pub fn append_severity_number(&mut self, val: Option<i32>) {
        if let Some(val) = val {
            self.severity_number.append_value(&val);
        } else {
            self.severity_number.append_null();
        }
    }

    /// append a value to the `schema_url` array
    pub fn append_schema_url(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.schema_url.append_str(val);
        } else {
            self.schema_url.append_null();
        }
    }

    /// append a value to the `schema_url` array `n` times
    pub fn append_schema_url_n(&mut self, val: Option<&str>, n: usize) {
        if let Some(val) = val {
            self.schema_url.append_str_n(val, n);
        } else {
            self.schema_url.append_nulls(n);
        }
    }

    /// append a value to the `severity_text` array
    pub fn append_severity_text(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.severity_text.append_str(val)
        } else {
            self.severity_text.append_null();
        }
    }

    /// append a value to the `dropped_attributes_count` array
    pub fn append_dropped_attributes_count(&mut self, val: u32) {
        self.dropped_attributes_count.append_value(&val);
    }

    /// append a value to the `flags` array
    pub fn append_flags(&mut self, val: Option<u32>) {
        if let Some(val) = val {
            self.flags.append_value(&val);
        } else {
            self.flags.append_null();
        }
    }

    /// append a value to the `trace_id` array
    pub fn append_trace_id(&mut self, val: Option<&[u8]>) -> Result<(), ArrowError> {
        if let Some(val) = val {
            self.trace_id.append_slice(val)
        } else {
            self.trace_id.append_null();
            Ok(())
        }
    }

    /// append a value to the `span_id` array
    pub fn append_span_id(&mut self, val: Option<&[u8]>) -> Result<(), ArrowError> {
        if let Some(val) = val {
            self.span_id.append_slice(val)
        } else {
            self.span_id.append_null();
            Ok(())
        }
    }

    /// construct an OTAP Logs record batch from the array builders
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut fields = vec![];
        let mut columns = vec![];

        if let Some(array) = self.id.finish() {
            fields.push(
                Field::new(consts::ID, array.data_type().clone(), true).with_plain_encoding(),
            );
            columns.push(array);
        }

        let resources = self.resource.finish()?;
        fields.push(Field::new(
            consts::RESOURCE,
            resources.data_type().clone(),
            true,
        ));
        columns.push(Arc::new(resources));

        let scopes = self.scope.finish()?;
        fields.push(Field::new(consts::SCOPE, scopes.data_type().clone(), true));
        columns.push(Arc::new(scopes));

        if let Some(array) = self.schema_url.finish() {
            fields.push(Field::new(
                consts::SCHEMA_URL,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.time_unix_nano.finish() {
            fields.push(Field::new(
                consts::TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.observed_time_unix_nano.finish() {
            fields.push(Field::new(
                consts::OBSERVED_TIME_UNIX_NANO,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.trace_id.finish() {
            fields.push(Field::new(
                consts::TRACE_ID,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.span_id.finish() {
            fields.push(Field::new(consts::SPAN_ID, array.data_type().clone(), true));
            columns.push(array);
        }

        if let Some(array) = self.severity_number.finish() {
            fields.push(Field::new(
                consts::SEVERITY_NUMBER,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.severity_text.finish() {
            fields.push(Field::new(
                consts::SEVERITY_TEXT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(body) = self.body.finish().transpose()? {
            fields.push(Field::new(consts::BODY, body.data_type().clone(), true));
            columns.push(Arc::new(body));
        }

        if let Some(array) = self.dropped_attributes_count.finish() {
            fields.push(Field::new(
                consts::DROPPED_ATTRIBUTES_COUNT,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.flags.finish() {
            fields.push(Field::new(consts::FLAGS, array.data_type().clone(), true));
            columns.push(array);
        }

        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

/// Builder for the body of a log record.
pub struct LogsBodyBuilder {
    value_type: UInt8ArrayBuilder,
    string_value: StringArrayBuilder,
    int_value: Int64ArrayBuilder,
    double_value: Float64ArrayBuilder,
    bool_value: AdaptiveBooleanArrayBuilder,
    bytes_value: BinaryArrayBuilder,
    ser_value: BinaryArrayBuilder,
    nulls: NullBufferBuilder,
    // Track pending null counts for each type for efficient batching
    pending_string_nulls: usize,
    pending_int_nulls: usize,
    pending_double_nulls: usize,
    pending_bool_nulls: usize,
    pending_bytes_nulls: usize,
    pending_ser_nulls: usize,
}

impl LogsBodyBuilder {
    /// Create a new instance of `LogsBodyBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self {
            value_type: UInt8ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            string_value: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict16()),
                ..Default::default()
            }),
            int_value: Int64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict16()),
                ..Default::default()
            }),
            double_value: Float64ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
            bool_value: AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions { optional: true }),
            bytes_value: BinaryArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict16()),
                ..Default::default()
            }),
            ser_value: BinaryArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict16()),
                ..Default::default()
            }),
            nulls: NullBufferBuilder::new(0),
            pending_string_nulls: 0,
            pending_int_nulls: 0,
            pending_double_nulls: 0,
            pending_bool_nulls: 0,
            pending_bytes_nulls: 0,
            pending_ser_nulls: 0,
        }
    }

    /// Append a string value to the body.
    pub fn append_str(&mut self, val: &str) {
        self.value_type
            .append_value(&(AttributeValueType::Str as u8));

        // Flush pending nulls for string array and append the actual value
        if self.pending_string_nulls > 0 {
            self.string_value.append_nulls(self.pending_string_nulls);
            self.pending_string_nulls = 0;
        }
        self.string_value.append_str(val);

        // Increment pending nulls for all other arrays
        self.pending_int_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.pending_ser_nulls += 1;
        self.nulls.append(true);
    }

    /// Append a boolean value to the body..
    pub fn append_bool(&mut self, val: bool) {
        self.value_type
            .append_value(&(AttributeValueType::Bool as u8));

        // Flush pending nulls for bool array and append the actual value
        if self.pending_bool_nulls > 0 {
            self.bool_value.append_nulls(self.pending_bool_nulls);
            self.pending_bool_nulls = 0;
        }
        self.bool_value.append_value(val);

        // Increment pending nulls for all other arrays
        self.pending_string_nulls += 1;
        self.pending_int_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.pending_ser_nulls += 1;
        self.nulls.append(true);
    }

    /// Append an integer value to the body.
    pub fn append_int(&mut self, val: i64) {
        self.value_type
            .append_value(&(AttributeValueType::Int as u8));

        // Flush pending nulls for int array and append the actual value
        if self.pending_int_nulls > 0 {
            self.int_value.append_nulls(self.pending_int_nulls);
            self.pending_int_nulls = 0;
        }
        self.int_value.append_value(&val);

        // Increment pending nulls for all other arrays
        self.pending_string_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.pending_ser_nulls += 1;
        self.nulls.append(true);
    }

    /// Append a double value to the body.
    pub fn append_double(&mut self, val: f64) {
        self.value_type
            .append_value(&(AttributeValueType::Double as u8));

        // Flush pending nulls for double array and append the actual value
        if self.pending_double_nulls > 0 {
            self.double_value.append_nulls(self.pending_double_nulls);
            self.pending_double_nulls = 0;
        }
        self.double_value.append_value(&val);

        // Increment pending nulls for all other arrays
        self.pending_string_nulls += 1;
        self.pending_int_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.pending_ser_nulls += 1;
        self.nulls.append(true);
    }

    /// Append a bytes value to the body.
    pub fn append_bytes(&mut self, val: &[u8]) {
        self.value_type
            .append_value(&(AttributeValueType::Bytes as u8));

        // Flush pending nulls for bytes array and append the actual value
        if self.pending_bytes_nulls > 0 {
            self.bytes_value.append_nulls(self.pending_bytes_nulls);
            self.pending_bytes_nulls = 0;
        }
        self.bytes_value.append_slice(val);

        // Increment pending nulls for all other arrays
        self.pending_string_nulls += 1;
        self.pending_int_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_ser_nulls += 1;
        self.nulls.append(true);
    }

    /// Append a slice value to the body. The bytes should be the value serialized as CBOR
    pub fn append_slice(&mut self, val: &[u8]) {
        self.value_type
            .append_value(&(AttributeValueType::Slice as u8));

        // Flush pending nulls for ser array and append the actual value
        if self.pending_ser_nulls > 0 {
            self.ser_value.append_nulls(self.pending_ser_nulls);
            self.pending_ser_nulls = 0;
        }
        self.ser_value.append_slice(val);

        // Increment pending nulls for all other arrays
        self.pending_string_nulls += 1;
        self.pending_int_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.nulls.append(true);
    }

    /// Append a map value to the body. The bytes should be the value serialized as CBOR
    pub fn append_map(&mut self, val: &[u8]) {
        self.value_type
            .append_value(&(AttributeValueType::Map as u8));

        // Flush pending nulls for ser array and append the actual value
        if self.pending_ser_nulls > 0 {
            self.ser_value.append_nulls(self.pending_ser_nulls);
            self.pending_ser_nulls = 0;
        }
        self.ser_value.append_slice(val);

        // Increment pending nulls for all other arrays
        self.pending_string_nulls += 1;
        self.pending_int_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.nulls.append(true);
    }

    /// Append a null value to the body
    pub fn append_null(&mut self) {
        self.value_type.append_null();

        // For null values, just increment pending nulls for all arrays
        self.pending_string_nulls += 1;
        self.pending_int_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.pending_ser_nulls += 1;
        self.nulls.append_null();
    }

    /// Fill arrays with nulls to ensure they all have the same length and maintain correct ordering
    fn fill_missing_nulls(&mut self) {
        // Simply append any remaining pending nulls to each array
        if self.pending_string_nulls > 0 {
            self.string_value.append_nulls(self.pending_string_nulls);
            self.pending_string_nulls = 0;
        }
        if self.pending_int_nulls > 0 {
            self.int_value.append_nulls(self.pending_int_nulls);
            self.pending_int_nulls = 0;
        }
        if self.pending_double_nulls > 0 {
            self.double_value.append_nulls(self.pending_double_nulls);
            self.pending_double_nulls = 0;
        }
        if self.pending_bool_nulls > 0 {
            self.bool_value.append_nulls(self.pending_bool_nulls);
            self.pending_bool_nulls = 0;
        }
        if self.pending_bytes_nulls > 0 {
            self.bytes_value.append_nulls(self.pending_bytes_nulls);
            self.pending_bytes_nulls = 0;
        }
        if self.pending_ser_nulls > 0 {
            self.ser_value.append_nulls(self.pending_ser_nulls);
            self.pending_ser_nulls = 0;
        }
    }

    /// Finish this builder try to build the resulting `StructArray` for the log body
    fn finish(&mut self) -> Option<Result<StructArray, ArrowError>> {
        // Ensure all arrays have the same length by bulk appending nulls where needed
        self.fill_missing_nulls();

        let len = self.nulls.len();
        let nulls = self.nulls.finish();

        // if it's all null, don't bother creating the struct array
        if let Some(nulls) = &nulls {
            if nulls.null_count() == len {
                return None;
            }
        }

        let mut fields = vec![];
        let mut columns = vec![];

        if let Some(array) = self.value_type.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_TYPE,
                array.data_type().clone(),
                // TODO shouldn't be nullable?
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.string_value.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_STR,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.int_value.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_INT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.double_value.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_DOUBLE,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.bool_value.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_BOOL,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.bytes_value.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_BYTES,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.ser_value.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_SER,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        Some(StructArray::try_new(Fields::from(fields), columns, nulls))
    }
}

/// Builder for the `resource` struct column of the logs OTAP record.
pub struct ResourceBuilder {
    id: UInt16ArrayBuilder,
    schema_url: StringArrayBuilder,
    dropped_attributes_count: UInt32ArrayBuilder,
}

impl ResourceBuilder {
    /// Create a new instance of this resource builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            schema_url: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `id` array
    pub fn append_id(&mut self, val: Option<u16>) {
        if let Some(val) = val {
            self.id.append_value(&val);
        } else {
            self.id.append_null();
        }
    }

    /// Append the value of `id` to the builder `n` times
    pub fn append_id_n(&mut self, val: u16, n: usize) {
        self.id.append_value_n(&val, n);
    }

    /// Append a value to the `schema_url` array
    pub fn append_schema_url(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.schema_url.append_str(val)
        } else {
            self.schema_url.append_null();
        }
    }

    /// Append a value to the `schema_url` array `n` times
    pub fn append_schema_url_n(&mut self, val: Option<&str>, n: usize) {
        if let Some(val) = val {
            self.schema_url.append_str_n(val, n);
        } else {
            self.schema_url.append_nulls(n);
        }
    }

    /// Append a value to the `dropped_attributes_count` array
    pub fn append_dropped_attributes_count(&mut self, val: u32) {
        self.dropped_attributes_count.append_value(&val);
    }

    /// Append a value to the `dropped_attributes_count` array `n` times
    pub fn append_dropped_attributes_count_n(&mut self, val: u32, n: usize) {
        self.dropped_attributes_count.append_value_n(&val, n);
    }

    /// Finish this builder and build the resulting `StructArray` for the resource
    pub fn finish(&mut self) -> Result<StructArray, ArrowError> {
        let mut fields = vec![];
        let mut columns = vec![];

        if let Some(array) = self.id.finish() {
            fields.push(
                Field::new(consts::ID, array.data_type().clone(), true).with_plain_encoding(),
            );
            columns.push(array);
        }

        if let Some(array) = self.schema_url.finish() {
            fields.push(Field::new(
                consts::SCHEMA_URL,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        if let Some(array) = self.dropped_attributes_count.finish() {
            fields.push(Field::new(
                consts::DROPPED_ATTRIBUTES_COUNT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        StructArray::try_new(Fields::from(fields), columns, None)
    }
}

/// Builder for the scope struct column of the logs OTAP batch
pub struct ScopeBuilder {
    id: UInt16ArrayBuilder,
    name: StringArrayBuilder,
    version: StringArrayBuilder,
    dropped_attributes_count: UInt32ArrayBuilder,
}

impl ScopeBuilder {
    /// Create a new instance of this scope builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: UInt16ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                default_values_optional: false,
            }),
            name: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            version: StringArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            dropped_attributes_count: UInt32ArrayBuilder::new(ArrayOptions {
                optional: true,
                dictionary_options: None,
                ..Default::default()
            }),
        }
    }

    /// Append a value to the `id` array
    pub fn append_id(&mut self, val: Option<u16>) {
        if let Some(val) = val {
            self.id.append_value(&val);
        } else {
            self.id.append_null();
        }
    }

    /// Append the value of `id` to the builder `n` times
    pub fn append_id_n(&mut self, val: u16, n: usize) {
        self.id.append_value_n(&val, n);
    }

    /// Append a value to the `name` array
    pub fn append_name(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.name.append_str(val)
        } else {
            self.name.append_null();
        }
    }

    /// Append a value to the `name` array n times
    pub fn append_name_n(&mut self, val: Option<&str>, n: usize) {
        if let Some(val) = val {
            self.name.append_str_n(val, n);
        } else {
            self.name.append_nulls(n);
        }
    }

    /// Append a value to the `version` array
    pub fn append_version(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.version.append_str(val);
        } else {
            self.version.append_null();
        }
    }

    /// Append a value to the `version` array n times`
    pub fn append_version_n(&mut self, val: Option<&str>, n: usize) {
        if let Some(val) = val {
            self.version.append_str_n(val, n);
        } else {
            self.version.append_nulls(n);
        }
    }

    /// Append a value to the `dropped_attributes_count` array
    pub fn append_dropped_attributes_count(&mut self, val: u32) {
        self.dropped_attributes_count.append_value(&val);
    }

    /// Append a value to the `dropped_attributes_count` array `n` times
    pub fn append_dropped_attributes_count_n(&mut self, val: u32, n: usize) {
        self.dropped_attributes_count.append_value_n(&val, n);
    }

    /// Finish this builder and build the resulting `StructArray` for the scope
    pub fn finish(&mut self) -> Result<StructArray, ArrowError> {
        let mut fields = vec![];
        let mut columns = vec![];

        if let Some(array) = self.id.finish() {
            fields.push(
                Field::new(consts::ID, array.data_type().clone(), true).with_plain_encoding(),
            );
            columns.push(array);
        }

        if let Some(array) = self.name.finish() {
            fields.push(Field::new(consts::NAME, array.data_type().clone(), true));
            columns.push(array);
        }

        if let Some(array) = self.version.finish() {
            fields.push(Field::new(consts::VERSION, array.data_type().clone(), true));
            columns.push(array);
        }

        if let Some(array) = self.dropped_attributes_count.finish() {
            fields.push(Field::new(
                consts::DROPPED_ATTRIBUTES_COUNT,
                array.data_type().clone(),
                true,
            ));
            columns.push(array);
        }

        StructArray::try_new(Fields::from(fields), columns, None)
    }
}
