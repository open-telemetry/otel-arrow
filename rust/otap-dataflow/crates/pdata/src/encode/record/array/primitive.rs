// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{
    ArrowPrimitiveType, DictionaryArray, PrimitiveArray, PrimitiveBuilder,
    PrimitiveDictionaryBuilder,
};
use arrow::datatypes::{ArrowDictionaryKeyType, TimestampNanosecondType, UInt8Type, UInt16Type};
use arrow::error::ArrowError;
use std::sync::Arc;

use crate::encode::record::array::dictionary::{DictionaryBuilder, UpdateDictionaryIndexInto};
use crate::encode::record::array::{ArrayAppendNulls, DefaultValueProvider, NoArgs};
use crate::schema::TIMESTAMP_TIME_ZONE;

use super::dictionary::{self, ConvertToNativeHelper, DictionaryArrayAppend};
use super::{ArrayAppend, ArrayBuilder, ArrayBuilderConstructor, ArrayLen, ArrayRef};

impl<T> ArrayAppend for PrimitiveBuilder<T>
where
    T: ArrowPrimitiveType,
{
    type Native
        = T::Native
    where
        T: ArrowPrimitiveType;

    fn append_value(&mut self, value: &<Self as ArrayAppend>::Native) {
        self.append_value(*value);
    }

    fn append_value_n(&mut self, value: &Self::Native, n: usize) {
        self.append_value_n(*value, n);
    }
}

impl<T> ArrayAppendNulls for PrimitiveBuilder<T>
where
    T: ArrowPrimitiveType,
{
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.append_nulls(n);
    }
}

impl<T> DefaultValueProvider<T::Native, NoArgs> for PrimitiveBuilder<T>
where
    T: ArrowPrimitiveType,
{
    fn default_value(_args: NoArgs) -> T::Native {
        T::Native::default()
    }
}

impl<T> ArrayBuilder for PrimitiveBuilder<T>
where
    T: ArrowPrimitiveType,
{
    fn finish(&mut self) -> ArrayRef {
        Arc::new(self.finish())
    }
}

impl<T> ArrayLen for PrimitiveBuilder<T>
where
    T: ArrowPrimitiveType,
{
    fn len(&self) -> usize {
        arrow::array::ArrayBuilder::len(self)
    }
}

impl<K, V> ArrayLen for PrimitiveDictionaryBuilder<K, V>
where
    K: ArrowDictionaryKeyType,
    V: ArrowPrimitiveType,
{
    fn len(&self) -> usize {
        arrow::array::ArrayBuilder::len(self)
    }
}

impl<T> ArrayBuilderConstructor for PrimitiveBuilder<T>
where
    T: ArrowPrimitiveType,
{
    type Args = NoArgs;

    fn new(_args: Self::Args) -> Self {
        Self::new()
    }
}

impl<K, V> ArrayBuilderConstructor for PrimitiveDictionaryBuilder<K, V>
where
    K: ArrowDictionaryKeyType,
    V: ArrowPrimitiveType,
{
    type Args = NoArgs;

    fn new(_args: Self::Args) -> Self {
        Self::new()
    }
}

impl<K, V> DictionaryArrayAppend for PrimitiveDictionaryBuilder<K, V>
where
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
    V: ArrowPrimitiveType,
{
    type Native = V::Native;

    fn append_value(&mut self, value: &Self::Native) -> dictionary::Result<usize> {
        match self.append(*value) {
            Ok(index) => Ok(index.into()),
            Err(ArrowError::DictionaryKeyOverflowError) => {
                Err(dictionary::DictionaryBuilderError::DictOverflow {})
            }

            // safety: shouldn't happen. The only error type append should
            // return should be for dictionary overflows
            Err(e) => panic!("unexpected error type appending to dictionary {e}"),
        }
    }

    fn append_values(&mut self, value: &Self::Native, n: usize) -> dictionary::Result<usize> {
        match self.append_n(*value, n) {
            Ok(index) => Ok(index.into()),
            Err(ArrowError::DictionaryKeyOverflowError) => {
                Err(dictionary::DictionaryBuilderError::DictOverflow {})
            }

            // safety: shouldn't happen. The only error type append should
            // return should be for dictionary overflows
            Err(e) => panic!("unexpected error type appending to dictionary {e}"),
        }
    }
}

impl<K, V> ArrayAppendNulls for PrimitiveDictionaryBuilder<K, V>
where
    K: ArrowDictionaryKeyType,
    V: ArrowPrimitiveType,
{
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.append_nulls(n);
    }
}

impl<K, V> DictionaryBuilder<K> for PrimitiveDictionaryBuilder<K, V>
where
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
    V: ArrowPrimitiveType,
{
    fn finish(&mut self) -> DictionaryArray<K> {
        self.finish()
    }
}

impl<K, V> ConvertToNativeHelper for PrimitiveDictionaryBuilder<K, V>
where
    K: ArrowDictionaryKeyType,
    V: ArrowPrimitiveType,
{
    type Accessor = PrimitiveArray<V>;
}

impl<V> UpdateDictionaryIndexInto<PrimitiveDictionaryBuilder<UInt16Type, V>>
    for PrimitiveDictionaryBuilder<UInt8Type, V>
where
    V: ArrowPrimitiveType,
{
    fn upgrade_into(self) -> PrimitiveDictionaryBuilder<UInt16Type, V> {
        // safety: `try_new_from_builder` will return an error here if the source key type cannot
        // be upgraded into the source key type. This can happen if going signed -> unsigned and there
        // are negative keys, or if going from a bigger type to smaller and some keys would not fit
        // int the smaller type. This won't happen going u8 to u16
        PrimitiveDictionaryBuilder::try_new_from_builder(self).expect("can upgrade u8 to u16")
    }
}

/// Builder for OTAP timestamp columns.
///
/// `TimestampNanosecondType::DATA_TYPE` is `Timestamp(Nanosecond, None)`, but
/// the OTAP spec requires producers to tag every timestamp column with the UTC
/// time zone. This wrapper overrides the data type of the underlying builder
/// so the finished array carries `Some("UTC")`.
pub struct TimestampNanosecondBuilder {
    inner: PrimitiveBuilder<TimestampNanosecondType>,
}

impl TimestampNanosecondBuilder {
    /// Creates a builder whose finished array is tagged with the UTC time zone.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: PrimitiveBuilder::<TimestampNanosecondType>::new()
                .with_timezone(TIMESTAMP_TIME_ZONE),
        }
    }
}

impl Default for TimestampNanosecondBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrayBuilderConstructor for TimestampNanosecondBuilder {
    type Args = NoArgs;

    fn new(_args: Self::Args) -> Self {
        Self::new()
    }
}

impl ArrayAppend for TimestampNanosecondBuilder {
    type Native = i64;

    fn append_value(&mut self, value: &Self::Native) {
        self.inner.append_value(*value);
    }

    fn append_value_n(&mut self, value: &Self::Native, n: usize) {
        self.inner.append_value_n(*value, n);
    }
}

impl ArrayAppendNulls for TimestampNanosecondBuilder {
    fn append_null(&mut self) {
        self.inner.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.inner.append_nulls(n);
    }
}

impl DefaultValueProvider<i64, NoArgs> for TimestampNanosecondBuilder {
    fn default_value(_args: NoArgs) -> i64 {
        i64::default()
    }
}

impl ArrayBuilder for TimestampNanosecondBuilder {
    fn finish(&mut self) -> ArrayRef {
        Arc::new(self.inner.finish())
    }
}

impl ArrayLen for TimestampNanosecondBuilder {
    fn len(&self) -> usize {
        arrow::array::ArrayBuilder::len(&self.inner)
    }
}

/// Dictionary builder for OTAP timestamp columns.
///
/// Like [`TimestampNanosecondBuilder`], this overrides the values builder's
/// data type so the dictionary's value type carries the UTC time zone required
/// by section 5.5.2 of the OTAP spec.
///
/// OTAP does not currently permit dictionary-encoded timestamp columns, but the
/// adaptive builder is generic over a dictionary variant, so this keeps the
/// time zone correct on every code path.
pub struct TimestampNanosecondDictionaryBuilder<K: ArrowDictionaryKeyType> {
    inner: PrimitiveDictionaryBuilder<K, TimestampNanosecondType>,
}

impl<K: ArrowDictionaryKeyType> TimestampNanosecondDictionaryBuilder<K> {
    /// Creates a dictionary builder whose values carry the UTC time zone.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: PrimitiveDictionaryBuilder::new_from_empty_builders(
                PrimitiveBuilder::<K>::new(),
                PrimitiveBuilder::<TimestampNanosecondType>::new()
                    .with_timezone(TIMESTAMP_TIME_ZONE),
            ),
        }
    }
}

impl<K: ArrowDictionaryKeyType> Default for TimestampNanosecondDictionaryBuilder<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: ArrowDictionaryKeyType> ArrayBuilderConstructor
    for TimestampNanosecondDictionaryBuilder<K>
{
    type Args = NoArgs;

    fn new(_args: Self::Args) -> Self {
        Self::new()
    }
}

impl<K> DictionaryArrayAppend for TimestampNanosecondDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
{
    type Native = i64;

    fn append_value(&mut self, value: &Self::Native) -> dictionary::Result<usize> {
        DictionaryArrayAppend::append_value(&mut self.inner, value)
    }

    fn append_values(&mut self, value: &Self::Native, n: usize) -> dictionary::Result<usize> {
        DictionaryArrayAppend::append_values(&mut self.inner, value, n)
    }
}

impl<K: ArrowDictionaryKeyType> ArrayAppendNulls for TimestampNanosecondDictionaryBuilder<K> {
    fn append_null(&mut self) {
        self.inner.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.inner.append_nulls(n);
    }
}

impl<K: ArrowDictionaryKeyType> ArrayLen for TimestampNanosecondDictionaryBuilder<K> {
    fn len(&self) -> usize {
        arrow::array::ArrayBuilder::len(&self.inner)
    }
}

impl<K> DictionaryBuilder<K> for TimestampNanosecondDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
{
    fn finish(&mut self) -> DictionaryArray<K> {
        DictionaryBuilder::finish(&mut self.inner)
    }
}

impl<K: ArrowDictionaryKeyType> ConvertToNativeHelper for TimestampNanosecondDictionaryBuilder<K> {
    type Accessor = PrimitiveArray<TimestampNanosecondType>;
}

impl UpdateDictionaryIndexInto<TimestampNanosecondDictionaryBuilder<UInt16Type>>
    for TimestampNanosecondDictionaryBuilder<UInt8Type>
{
    fn upgrade_into(self) -> TimestampNanosecondDictionaryBuilder<UInt16Type> {
        TimestampNanosecondDictionaryBuilder {
            // safety: upgrading u8 keys to u16 keys always fits
            inner: PrimitiveDictionaryBuilder::try_new_from_builder(self.inner)
                .expect("can upgrade u8 to u16"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{Array, UInt8Array, UInt8Builder, UInt8DictionaryArray};
    use arrow::datatypes::{DataType, UInt8Type};

    #[test]
    fn test_primitive_builder() {
        let mut builder = PrimitiveBuilder::<UInt8Type>::new();
        ArrayAppend::append_value(&mut builder, &1);
        ArrayAppend::append_value(&mut builder, &2);
        ArrayAppend::append_value(&mut builder, &3);

        let result = ArrayBuilder::finish(&mut builder);
        assert_eq!(result.data_type(), &DataType::UInt8);

        let expected = PrimitiveArray::<UInt8Type>::from(vec![1, 2, 3]);
        assert_eq!(
            result.as_any().downcast_ref::<UInt8Array>().unwrap(),
            &expected
        );
    }

    #[test]
    fn test_primitive_dictionary_builder() {
        let mut builder = PrimitiveDictionaryBuilder::<UInt8Type, UInt8Type>::new();
        let index = DictionaryArrayAppend::append_value(&mut builder, &42).unwrap();
        assert_eq!(index, 0);
        let index = DictionaryArrayAppend::append_value(&mut builder, &42).unwrap();
        assert_eq!(index, 0);
        let index = DictionaryArrayAppend::append_value(&mut builder, &43).unwrap();
        assert_eq!(index, 1);

        let result = DictionaryBuilder::finish(&mut builder);
        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(
                Box::new(UInt8Type::DATA_TYPE),
                Box::new(UInt8Type::DATA_TYPE)
            )
        );
        assert_eq!(result.len(), 3);

        let mut expected_dict_values = UInt8Builder::new();
        expected_dict_values.append_value(42);
        expected_dict_values.append_value(43);
        let expected_dict_keys = UInt8Array::from_iter_values(vec![0, 0, 1]);
        let expected =
            UInt8DictionaryArray::new(expected_dict_keys, Arc::new(expected_dict_values.finish()));

        assert_eq!(
            result
                .as_any()
                .downcast_ref::<UInt8DictionaryArray>()
                .unwrap(),
            &expected
        );
    }

    #[test]
    fn test_dictionary_builder_overflow() {
        let mut dict_builder = PrimitiveDictionaryBuilder::<UInt8Type, UInt16Type>::new();
        for i in 0..255 {
            let _ = DictionaryArrayAppend::append_value(&mut dict_builder, &i).unwrap();
        }

        // this should be fine
        let _ = DictionaryArrayAppend::append_value(&mut dict_builder, &256).unwrap();

        // this should overflow
        let result = DictionaryArrayAppend::append_value(&mut dict_builder, &257);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(
            err,
            dictionary::DictionaryBuilderError::DictOverflow
        ));
    }
}
