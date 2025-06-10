// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, ArrowPrimitiveType, BinaryArray, BinaryBuilder, BinaryDictionaryBuilder,
    DictionaryArray, PrimitiveDictionaryBuilder,
};
use arrow::datatypes::{ArrowDictionaryKeyType, DataType, UInt8Type, UInt16Type};

use crate::encode::record::array::dictionary::{
    ConvertToNativeHelper, DictionaryBuilder, UpdateDictionaryIndexInto,
};
use crate::encode::record::array::{ArrayAppend, NoArgs};

use super::dictionary::{self, DictionaryArrayAppend};
use super::{ArrayBuilder, ArrayBuilderConstructor};

impl ArrayAppend for BinaryBuilder {
    type Native = Vec<u8>;

    fn append_value(&mut self, value: &Self::Native) {
        self.append_value(value);
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
        match self.append(value) {
            Ok(index) => Ok(index.into()),
            Err(arrow::error::ArrowError::DictionaryKeyOverflowError) => {
                Err(dictionary::DictionaryBuilderError::DictOverflow {})
            }
            // safety: shouldn't happen. The only error type append should
            // return should be for dictionary overflows
            Err(e) => panic!("unexpected error type appending to dictionary {}", e),
        }
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
    fn upgrade_into(&mut self) -> BinaryDictionaryBuilder<UInt16Type> {
        // TODO there should be an optimized way to implement this. Thinking we could
        // create a new builder with the same keys (but use `cast` kernel) cast them
        // to u16 then reuse the same values
        // related issue https://github.com/open-telemetry/otel-arrow/issues/536

        let dict_arr = self.finish();

        // safety: DictionaryBuilder returns a dictionary that has Primitive values
        let values = dict_arr
            .downcast_dict::<BinaryArray>()
            .expect("expect values are of type BinaryArray");

        let mut upgraded_builder = BinaryDictionaryBuilder::<UInt16Type>::new();
        for value in values {
            match value {
                Some(value) => upgraded_builder.append_value(value),
                None => {
                    // TODO handle this in https://github.com/open-telemetry/otel-arrow/issues/534
                    todo!("nulls not yet supported by adaptive array builders")
                }
            }
        }

        upgraded_builder
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{ArrayRef, BinaryArray, UInt8Array, UInt8DictionaryArray};
    use arrow::datatypes::{DataType, Field, Schema, UInt8Type};
    use prost::Message;

    #[test]
    fn test_string_builder() {
        let mut binary_builder = BinaryBuilder::new();
        ArrayAppend::append_value(&mut binary_builder, &b"a".to_vec());
        ArrayAppend::append_value(&mut binary_builder, &b"b".to_vec());
        ArrayAppend::append_value(&mut binary_builder, &b"c".to_vec());

        let result = ArrayBuilder::finish(&mut binary_builder);

        assert_eq!(result.data_type(), &DataType::Binary);

        let expected = BinaryArray::from_iter_values(vec![b"a", b"b", b"c"]);
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
                &format!("{}", i).encode_to_vec(),
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
