// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{
    ArrowPrimitiveType, DictionaryArray, PrimitiveArray, PrimitiveBuilder,
    PrimitiveDictionaryBuilder,
};
use arrow::datatypes::{ArrowDictionaryKeyType, UInt8Type, UInt16Type};
use arrow::error::ArrowError;
use std::sync::Arc;

use crate::encode::record::array::dictionary::{DictionaryBuilder, UpdateDictionaryIndexInto};
use crate::encode::record::array::{ArrayAppendNulls, DefaultValueProvider, NoArgs};

use super::dictionary::{self, ConvertToNativeHelper, DictionaryArrayAppend};
use super::{ArrayAppend, ArrayBuilder, ArrayBuilderConstructor, ArrayRef};

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
    fn upgrade_into(&mut self) -> PrimitiveDictionaryBuilder<UInt16Type, V> {
        // TODO there should be an optimized way to implement this. Thinking we could
        // create a new builder with the same keys (but use `cast` kernel) cast them
        // to u16 then reuse the same values
        // related issue https://github.com/open-telemetry/otel-arrow/issues/536

        let dict_arr = self.finish();

        // safety: DictionaryBuilder returns a dictionary that has Primitive values
        let values = dict_arr
            .downcast_dict::<PrimitiveArray<V>>()
            .expect("expect values are of type primitive V");

        let mut upgraded_builder = PrimitiveDictionaryBuilder::<UInt16Type, V>::new();
        for value in values {
            match value {
                Some(value) => upgraded_builder.append_value(value),
                None => upgraded_builder.append_null(),
            }
        }

        upgraded_builder
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
            dictionary::DictionaryBuilderError::DictOverflow {}
        ));
    }
}
