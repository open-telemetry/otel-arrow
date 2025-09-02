// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::{
    ArrayRef, ArrowPrimitiveType, BinaryArray, BinaryBuilder, BinaryDictionaryBuilder,
    DictionaryArray,
};
use arrow::datatypes::{ArrowDictionaryKeyType, UInt8Type, UInt16Type};

use crate::encode::record::array::dictionary::{
    ConvertToNativeHelper, DictionaryArrayAppendSlice, DictionaryBuilder, UpdateDictionaryIndexInto,
};
use crate::encode::record::array::{
    ArrayAppend, ArrayAppendNulls, ArrayAppendSlice, DefaultValueProvider, NoArgs,
};

use super::dictionary::{self, DictionaryArrayAppend};
use super::{ArrayBuilder, ArrayBuilderConstructor};

impl ArrayAppend for BinaryBuilder {
    type Native = Vec<u8>;

    #[inline(always)]
    fn append_value(&mut self, value: &Self::Native) {
        self.append_value(value);
    }

    #[inline(always)]
    fn append_value_n(&mut self, value: &Self::Native, n: usize) {
        for _ in 0..n {
            self.append_value(value);
        }
    }
}

impl ArrayAppendSlice for BinaryBuilder {
    type Native = u8;

    #[inline(always)]
    fn append_slice(&mut self, value: &[Self::Native]) {
        self.append_value(value);
    }

    #[inline(always)]
    fn append_slice_n(&mut self, value: &[Self::Native], n: usize) {
        for _ in 0..n {
            self.append_value(value);
        }
    }
}

impl ArrayAppendNulls for BinaryBuilder {
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.append_nulls(n);
    }
}

impl DefaultValueProvider<Vec<u8>, NoArgs> for BinaryBuilder {
    fn default_value(_args: NoArgs) -> Vec<u8> {
        Vec::new()
    }
}

impl ArrayBuilder for BinaryBuilder {
    fn finish(&mut self) -> ArrayRef {
        Arc::new(self.finish())
    }
}

impl ArrayBuilderConstructor for BinaryBuilder {
    type Args = NoArgs;

    fn new(_args: Self::Args) -> Self {
        Self::new()
    }
}

impl<K> ArrayBuilderConstructor for BinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    type Args = ();

    fn new(_args: Self::Args) -> Self {
        Self::new()
    }
}

impl<K> DictionaryArrayAppend for BinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
{
    type Native = Vec<u8>;

    fn append_value(&mut self, value: &Self::Native) -> dictionary::Result<usize> {
        self.append_slice(value)
    }

    fn append_values(&mut self, value: &Self::Native, n: usize) -> dictionary::Result<usize> {
        self.append_slice_n(value, n)
    }
}

impl<K> DictionaryArrayAppendSlice for BinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
{
    type Native = u8;

    fn append_slice(&mut self, value: &[Self::Native]) -> dictionary::Result<usize> {
        match self.append(value) {
            Ok(index) => Ok(index.into()),
            Err(arrow::error::ArrowError::DictionaryKeyOverflowError) => {
                Err(dictionary::DictionaryBuilderError::DictOverflow {})
            }
            // safety: shouldn't happen. The only error type append should
            // return should be for dictionary overflows
            Err(e) => panic!("unexpected error type appending to dictionary {e}"),
        }
    }

    fn append_slice_n(&mut self, value: &[Self::Native], n: usize) -> dictionary::Result<usize> {
        match self.append_n(value, n) {
            Ok(index) => Ok(index.into()),
            Err(arrow::error::ArrowError::DictionaryKeyOverflowError) => {
                Err(dictionary::DictionaryBuilderError::DictOverflow {})
            }
            // safety: shouldn't happen. The only error type append should
            // return should be for dictionary overflows
            Err(e) => panic!("unexpected error type appending to dictionary {e}"),
        }
    }
}

impl<K> ArrayAppendNulls for BinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.append_nulls(n);
    }
}

impl<K> DictionaryBuilder<K> for BinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    fn finish(&mut self) -> DictionaryArray<K> {
        self.finish()
    }
}

impl<T> ConvertToNativeHelper for BinaryDictionaryBuilder<T>
where
    T: ArrowDictionaryKeyType,
{
    type Accessor = BinaryArray;
}

impl UpdateDictionaryIndexInto<BinaryDictionaryBuilder<UInt16Type>>
    for BinaryDictionaryBuilder<UInt8Type>
{
    fn upgrade_into(self) -> BinaryDictionaryBuilder<UInt16Type> {
        // safety: `try_new_from_builder` will return an error here if the source key type cannot
        // be upgraded into the source key type. This can happen if going signed -> unsigned and there
        // are negative keys, or if going from a bigger type to smaller and some keys would not fit
        // int the smaller type. This won't happen going u8 to u16
        BinaryDictionaryBuilder::try_new_from_builder(self).expect("can upgrade from u8 to u16 key")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{Array, BinaryArray, UInt8Array, UInt8DictionaryArray};
    use arrow::datatypes::{DataType, UInt8Type};
    use prost::Message;

    #[test]
    fn test_string_builder() {
        let mut binary_builder = BinaryBuilder::new();
        ArrayAppend::append_value(&mut binary_builder, &b"a".to_vec());
        ArrayAppend::append_value(&mut binary_builder, &b"b".to_vec());
        ArrayAppend::append_value(&mut binary_builder, &b"c".to_vec());
        binary_builder.append_slice(b"d");
        binary_builder.append_slice_n(b"e", 2);

        let result = ArrayBuilder::finish(&mut binary_builder);

        assert_eq!(result.data_type(), &DataType::Binary);

        let expected = BinaryArray::from_iter_values(vec![b"a", b"b", b"c", b"d", b"e", b"e"]);
        assert_eq!(
            result.as_any().downcast_ref::<BinaryArray>().unwrap(),
            &expected
        );
    }

    #[test]
    fn test_dictionary_builder() {
        let mut dict_builder = BinaryDictionaryBuilder::<UInt8Type>::new();
        let index = DictionaryArrayAppend::append_value(&mut dict_builder, &b"a".to_vec()).unwrap();
        assert_eq!(index, 0);
        let index = DictionaryArrayAppend::append_value(&mut dict_builder, &b"a".to_vec()).unwrap();
        assert_eq!(index, 0);
        let index = DictionaryArrayAppend::append_value(&mut dict_builder, &b"b".to_vec()).unwrap();
        assert_eq!(index, 1);

        let result = DictionaryBuilder::finish(&mut dict_builder);

        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Binary))
        );

        let mut expected_dict_values = BinaryBuilder::new();
        expected_dict_values.append_value(b"a");
        expected_dict_values.append_value(b"b");
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
        let mut dict_builder = BinaryDictionaryBuilder::<UInt8Type>::new();
        for i in 0..255 {
            let _ = DictionaryArrayAppend::append_value(
                &mut dict_builder,
                &format!("{i}").encode_to_vec(),
            )
            .unwrap();
        }

        // this should be fine
        let _ = DictionaryArrayAppend::append_value(&mut dict_builder, &b"a".to_vec()).unwrap();

        // this should overflow
        let result = DictionaryArrayAppend::append_value(&mut dict_builder, &b"b".to_vec());
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(
            err,
            dictionary::DictionaryBuilderError::DictOverflow {}
        ));
    }
}
