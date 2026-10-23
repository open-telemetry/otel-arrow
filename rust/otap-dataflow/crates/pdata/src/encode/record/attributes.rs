// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains builders for record batches for attributes.

mod known_keys;

pub use known_keys::{KnownKey, KnownKeyArrayBuilder};

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

    /// Returns the number of attribute rows appended to the builder.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns `true` if no attribute rows have been appended.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        if fields.is_empty() {
            return Ok(RecordBatch::new_empty(Arc::new(Schema::empty())));
        }
        RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
    }
}

/// Key-column backend for the shared string-key attribute record builder.
///
/// The default is [`StringArrayBuilder`]. [`KnownKeyArrayBuilder`] is an opt-in
/// alternative for fixed vocabularies; generic dispatch adds no runtime choice
/// or cache state to ordinary builders.
pub trait AttributeKeyColumnBuilder {
    /// Appends one UTF-8 attribute name.
    fn append_key(&mut self, key: &str);

    /// Finishes the key column, propagating construction errors.
    fn finish_keys(&mut self) -> Result<Option<ArrayRef>, ArrowError>;
}

impl AttributeKeyColumnBuilder for StringArrayBuilder {
    #[inline]
    fn append_key(&mut self, key: &str) {
        self.append_str(key);
    }

    #[inline]
    fn finish_keys(&mut self) -> Result<Option<ArrayRef>, ArrowError> {
        Ok(self.finish())
    }
}

/// Record batch builder for attributes with pre-validated string keys.
///
/// Use [`AttributesRecordBatchBuilder`] for arbitrary byte keys so UTF-8
/// validation can be delayed. The default key backend remains unchanged;
/// `new_with_known_keys` opts into cached fixed-vocabulary names.
pub struct StrKeysAttributesRecordBatchBuilder<
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
    K = StringArrayBuilder,
> {
    keys: K,
    parent_id: PrimitiveArrayBuilder<T::ArrayType>,

    /// Builder for attribute values.
    pub any_values_builder: AnyValuesRecordsBuilder,
}

impl<T> StrKeysAttributesRecordBatchBuilder<T>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    /// Creates an ordinary string-key builder with no known-key cache.
    #[must_use]
    pub fn new() -> Self {
        Self::with_key_builder(StringArrayBuilder::new(ArrayOptions {
            optional: false,
            dictionary_options: Some(DictionaryOptions::dict8()),
            ..Default::default()
        }))
    }
}

impl<T, const N: usize> StrKeysAttributesRecordBatchBuilder<T, KnownKeyArrayBuilder<N>>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
{
    /// Creates an opt-in builder whose dictionary cache resets at finish.
    /// Names are interned on first use, not in vocabulary order.
    ///
    /// ```
    /// use otel_arrow_dfe_pdata::encode::record::attributes::{
    ///     KnownKey, StrKeysAttributesRecordBatchBuilder,
    /// };
    /// let mut attributes = StrKeysAttributesRecordBatchBuilder::<u16, _>::
    ///     new_with_known_keys(&["service.name", "host.name"]);
    /// let keys = KnownKey::<2>::slots();
    /// attributes.append_known_key(keys[0]);
    /// attributes.any_values_builder.append_str(b"frontend");
    /// attributes.append_parent_id(&0);
    /// assert_eq!(attributes.finish().unwrap().num_rows(), 1);
    /// ```
    #[must_use]
    pub fn new_with_known_keys(names: &'static [&'static str; N]) -> Self {
        Self::with_key_builder(KnownKeyArrayBuilder::new(names))
    }

    /// Appends a vocabulary slot without exposing an Arrow dictionary ID.
    pub fn append_known_key(&mut self, key: impl Into<KnownKey<N>>) {
        self.keys.append_known_key(key.into());
    }
}

impl<T, K> StrKeysAttributesRecordBatchBuilder<T, K>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
    K: AttributeKeyColumnBuilder,
{
    fn with_key_builder(keys: K) -> Self {
        Self {
            keys,
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

    /// Append the same parent ID for `n` consecutive attribute rows.
    ///
    /// This avoids repeating adaptive dispatch and dictionary lookup for every
    /// attribute belonging to the same record. A zero count appends nothing.
    pub fn append_parent_id_n(
        &mut self,
        val: &<<T as ParentId>::ArrayType as ArrowPrimitiveType>::Native,
        n: usize,
    ) {
        self.parent_id.append_value_n(val, n);
    }

    /// Append the attribute key to the builder for this array
    pub fn append_key(&mut self, val: &str) {
        self.keys.append_key(val);
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

        if let Some(array) = self.keys.finish_keys()? {
            fields.push(Field::new(
                consts::ATTRIBUTE_KEY,
                array.data_type().clone(),
                false,
            ));
            columns.push(array);
        }

        self.any_values_builder.finish(&mut columns, &mut fields)?;
        if fields.is_empty() {
            return Ok(RecordBatch::new_empty(Arc::new(Schema::empty())));
        }
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
            bool_value: AdaptiveBooleanArrayBuilder::new(BooleanBuilderOptions {
                optional: true,
                skip_all_false: false,
            }),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_bulk_parent_ids<T>()
    where
        T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
        <T::ArrayType as ArrowPrimitiveType>::Native: From<u16>,
    {
        let mut bulk = StrKeysAttributesRecordBatchBuilder::<T>::new();
        let mut scalar = StrKeysAttributesRecordBatchBuilder::<T>::new();
        for id in 0..1000u16 {
            let parent = id.into();
            let count = usize::from(id % 4);
            bulk.append_parent_id_n(&parent, count);
            for _ in 0..count {
                scalar.append_parent_id(&parent);
                bulk.append_key("key");
                scalar.append_key("key");
                bulk.any_values_builder.append_int(i64::from(id));
                scalar.any_values_builder.append_int(i64::from(id));
            }
        }
        assert_eq!(bulk.finish().unwrap(), scalar.finish().unwrap());
    }

    /// Scenario: Parent IDs repeat zero or more times across native and dictionary columns.
    /// Guarantees: Bulk appends preserve rows and schema across capacity growth and 8-bit dictionary overflow.
    #[test]
    fn bulk_parent_ids_match_scalar_appends() {
        compare_bulk_parent_ids::<u16>();
        compare_bulk_parent_ids::<u32>();
    }

    fn compare_known_attribute_records<T>()
    where
        T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
        <T::ArrayType as ArrowPrimitiveType>::Native: From<u16>,
    {
        let mut known =
            StrKeysAttributesRecordBatchBuilder::<T, _>::new_with_known_keys(&["first", "second"]);
        let keys = KnownKey::<2>::slots();
        for count in [0u16, 1000] {
            let mut ordinary = StrKeysAttributesRecordBatchBuilder::<T>::new();
            for id in 0..count {
                let parent = id.into();
                known.append_known_key(keys[1]);
                ordinary.append_key("second");
                known.any_values_builder.append_int(i64::from(id));
                ordinary.any_values_builder.append_int(i64::from(id));
                known.append_key("first");
                ordinary.append_key("first");
                known.any_values_builder.append_str(b"dynamic collision");
                ordinary.any_values_builder.append_str(b"dynamic collision");
                known.append_known_key(keys[0]);
                ordinary.append_key("first");
                known.any_values_builder.append_empty();
                ordinary.any_values_builder.append_empty();
                known.append_parent_id_n(&parent, 3);
                ordinary.append_parent_id_n(&parent, 3);
            }
            let actual = known.finish().unwrap();
            let expected = ordinary.finish().unwrap();
            assert_eq!(actual, expected);
            for (actual, expected) in actual.columns().iter().zip(expected.columns()) {
                // Compare physical dictionary ordering and keys, not just decoded equality.
                assert_eq!(actual.to_data(), expected.to_data());
            }
        }
    }

    /// Scenario: Opt-in builders create empty and mixed batches with u16 and u32 parent IDs.
    /// Guarantees: Shared assembly preserves schema, metadata, physical arrays, bulk parents, and values across finish reuse.
    #[test]
    fn known_attribute_records_match_default_builders() {
        compare_known_attribute_records::<u16>();
        compare_known_attribute_records::<u32>();
    }
}
