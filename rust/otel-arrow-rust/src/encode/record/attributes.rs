// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains builders for record batches for attributes.

use std::sync::Arc;

use arrow::{
    array::{ArrayRef, ArrowPrimitiveType, RecordBatch},
    datatypes::{Field, Schema},
    error::ArrowError,
};

use crate::{
    encode::record::array::{
        ArrayAppend, ArrayAppendNulls, ArrayAppendSlice, ArrayAppendStr, ArrayOptions,
        BinaryArrayBuilder, Float64ArrayBuilder, Int64ArrayBuilder, PrimitiveArrayBuilder,
        StringArrayBuilder, UInt8ArrayBuilder, binary_to_utf8_array,
        boolean::{AdaptiveBooleanArrayBuilder, BooleanBuilderOptions},
        dictionary::DictionaryOptions,
    },
    otlp::attributes::{AttributeValueType, parent_id::ParentId},
    schema::{FieldExt, consts},
};

/// Record batch builder for attributes
pub struct AttributesRecordBatchBuilder<T: ParentId + AttributesRecordBatchBuilderConstructorHelper>
{
    keys: BinaryArrayBuilder,
    parent_id: PrimitiveArrayBuilder<T::ArrayType>,

    /// builder for attribute values
    pub any_values_builder: AnyValuesRecordsBuilder,
}

impl<T> AttributesRecordBatchBuilder<T>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    /// create a new instance of [`AttributesRecordBatchBuilder`]
    #[must_use]
    pub fn new() -> Self {
        Self {
            keys: BinaryArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            parent_id: PrimitiveArrayBuilder::new(T::parent_id_array_options()),
            any_values_builder: AnyValuesRecordsBuilder::new(),
        }
    }

    /// Append the parent ID to the builder for the parent_id array
    pub fn append_parent_id(
        &mut self,
        val: &<<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native,
    ) {
        self.parent_id.append_value(val);
    }

    /// Append the attribute key to the builder for this array
    pub fn append_key(&mut self, val: &[u8]) {
        self.keys.append_slice(val);
    }

    /// Finish this builder and produce the resulting RecordBatch
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut columns = vec![];
        let mut fields = vec![];

        if let Some(array) = self.parent_id.finish() {
            fields.push(
                Field::new(consts::PARENT_ID, array.data_type().clone(), false)
                    .with_plain_encoding(),
            );

            columns.push(array);
        }

        if let Some(array) = self.keys.finish() {
            let array = binary_to_utf8_array(&array)?;
            fields.push(Field::new(
                consts::ATTRIBUTE_KEY,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        self.any_values_builder.finish(&mut columns, &mut fields)?;
        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

/// Record batch builder for attributes where the key column is pre-validated string.
//
// Ordinarily, it would be better to use [`AttributesRecordBatchBuilder`] when constructing OTAP
// batches for attributes where the keys are not known ahead of time and we're receiving them as
// an array of arbitrary bytes (e.g. from OTLP proto bytes). The reason this is usually better is
// because we can delay UTF validation until we construct the array.
//
// However, in some cases the keys may be known ahead of time, in which case, it makes more sense
// to put them directly into a StringArrayBuilder instead of putting them into a BinaryArrayBuilder
// and validate UTF-8 lazily
pub struct StrKeysAttributesRecordBatchBuilder<
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
> {
    keys: StringArrayBuilder,
    parent_id: PrimitiveArrayBuilder<T::ArrayType>,

    /// builder for attribute values
    pub any_values_builder: AnyValuesRecordsBuilder,
}

impl<T> StrKeysAttributesRecordBatchBuilder<T>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    /// create a new instance of [`AttributesRecordBatchBuilder`]
    #[must_use]
    pub fn new() -> Self {
        Self {
            keys: StringArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: Some(DictionaryOptions::dict8()),
                ..Default::default()
            }),
            parent_id: PrimitiveArrayBuilder::new(T::parent_id_array_options()),
            any_values_builder: AnyValuesRecordsBuilder::new(),
        }
    }

    /// Append the parent ID to the builder for the parent_id array
    pub fn append_parent_id(
        &mut self,
        val: &<<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native,
    ) {
        self.parent_id.append_value(val);
    }

    /// Append the attribute key to the builder for this array
    pub fn append_key(&mut self, val: &str) {
        self.keys.append_str(val);
    }

    /// Finish this builder and produce the resulting RecordBatch
    pub fn finish(&mut self) -> Result<RecordBatch, ArrowError> {
        let mut columns = vec![];
        let mut fields = vec![];

        if let Some(array) = self.parent_id.finish() {
            fields.push(
                Field::new(consts::PARENT_ID, array.data_type().clone(), false)
                    .with_plain_encoding(),
            );

            columns.push(array);
        }

        if let Some(array) = self.keys.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_KEY,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        self.any_values_builder.finish(&mut columns, &mut fields)?;
        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

/// Record batch builder for arrays representing AnyValue
pub struct AnyValuesRecordsBuilder {
    value_type: UInt8ArrayBuilder,
    string_value: BinaryArrayBuilder,
    int_value: Int64ArrayBuilder,
    double_value: Float64ArrayBuilder,
    bool_value: AdaptiveBooleanArrayBuilder,
    bytes_value: BinaryArrayBuilder,
    ser_value: BinaryArrayBuilder,
    // Track pending null counts for each type for efficient batching
    pending_string_nulls: usize,
    pending_int_nulls: usize,
    pending_double_nulls: usize,
    pending_bool_nulls: usize,
    pending_bytes_nulls: usize,
    pending_ser_nulls: usize,
}

impl AnyValuesRecordsBuilder {
    /// Create a new instance of `AttributesRecordBatchBuilder`
    #[must_use]
    pub fn new() -> Self {
        Self {
            value_type: UInt8ArrayBuilder::new(ArrayOptions {
                optional: false,
                dictionary_options: None,
                ..Default::default()
            }),
            string_value: BinaryArrayBuilder::new(ArrayOptions {
                optional: true,
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
            pending_string_nulls: 0,
            pending_int_nulls: 0,
            pending_double_nulls: 0,
            pending_bool_nulls: 0,
            pending_bytes_nulls: 0,
            pending_ser_nulls: 0,
        }
    }

    /// Append a string value to the body.
    pub fn append_str(&mut self, val: &[u8]) {
        self.value_type
            .append_value(&(AttributeValueType::Str as u8));

        // Flush pending nulls for string array and append the actual value
        if self.pending_string_nulls > 0 {
            self.string_value.append_nulls(self.pending_string_nulls);
            self.pending_string_nulls = 0;
        }
        self.string_value.append_slice(val);

        // Increment pending nulls for all other arrays
        self.pending_int_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.pending_ser_nulls += 1;
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
    }

    /// Append an empty value to the body.
    pub fn append_empty(&mut self) {
        self.value_type
            .append_value(&(AttributeValueType::Empty as u8));

        // For empty values, just increment pending nulls for all arrays
        self.pending_string_nulls += 1;
        self.pending_int_nulls += 1;
        self.pending_double_nulls += 1;
        self.pending_bool_nulls += 1;
        self.pending_bytes_nulls += 1;
        self.pending_ser_nulls += 1;
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

    /// Finish this builder and produce the resulting RecordBatch
    pub fn finish(
        &mut self,
        columns: &mut Vec<ArrayRef>,
        fields: &mut Vec<Field>,
    ) -> Result<(), ArrowError> {
        // Ensure all arrays have the same length by bulk appending nulls where needed
        self.fill_missing_nulls();

        if let Some(array) = self.value_type.finish() {
            fields.push(Field::new(
                consts::ATTRIBUTE_TYPE,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        if let Some(array) = self.string_value.finish() {
            let array = binary_to_utf8_array(&array)?;
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

        Ok(())
    }
}

impl<T> Default for AttributesRecordBatchBuilder<T>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    fn default() -> Self {
        Self::new()
    }
}

/// trait that helps with the construction of AttributeRecordBatchBuilder
pub trait AttributesRecordBatchBuilderConstructorHelper {
    /// Supply the array options that define the behaviour of the parent ID column builder
    fn parent_id_array_options() -> ArrayOptions;
}

impl AttributesRecordBatchBuilderConstructorHelper for u16 {
    fn parent_id_array_options() -> ArrayOptions {
        ArrayOptions {
            optional: false,
            dictionary_options: None,
            ..Default::default()
        }
    }
}

impl AttributesRecordBatchBuilderConstructorHelper for u32 {
    fn parent_id_array_options() -> ArrayOptions {
        ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions::dict8()),
            ..Default::default()
        }
    }
}
