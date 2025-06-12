// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::{
    array::{
        ArrayRef, ArrowPrimitiveType, DictionaryArray, FixedSizeBinaryArray,
        FixedSizeBinaryBuilder, FixedSizeBinaryDictionaryBuilder,
    },
    datatypes::{ArrowDictionaryKeyType, UInt8Type, UInt16Type},
    error::ArrowError,
};

use crate::encode::record::array::{
    ArrayAppendNulls,
    dictionary::{
        CheckedDictionaryArrayAppend, ConvertToNativeHelper, DictionaryBuilder,
        UpdateDictionaryIndexInto,
    },
};

use super::{ArrayBuilder, ArrayBuilderConstructor, CheckedArrayAppend};

impl ArrayBuilderConstructor for FixedSizeBinaryBuilder {
    type Args = i32;

    fn new(byte_width: Self::Args) -> Self {
        Self::new(byte_width)
    }
}

impl CheckedArrayAppend for FixedSizeBinaryBuilder {
    type Native = Vec<u8>;

    fn append_value(&mut self, value: &Self::Native) -> Result<(), ArrowError> {
        self.append_value(value)
    }
}

impl ArrayAppendNulls for FixedSizeBinaryBuilder {
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        // TODO - after the next release of arrow-rs we should revisit this and call append_nulls
        // on the base builder. Waiting on these changes to be released:
        // https://github.com/apache/arrow-rs/pull/7606
        for _ in 0..n {
            self.append_null();
        }
    }
}

impl ArrayBuilder for FixedSizeBinaryBuilder {
    fn finish(&mut self) -> ArrayRef {
        Arc::new(self.finish())
    }
}

impl<K> ArrayBuilderConstructor for FixedSizeBinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    type Args = i32;

    fn new(byte_width: Self::Args) -> Self {
        Self::new(byte_width)
    }
}

impl<K> CheckedDictionaryArrayAppend for FixedSizeBinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
{
    type Native = Vec<u8>;

    fn append_value(&mut self, value: &Self::Native) -> super::dictionary::checked::Result<usize> {
        match self.append(value) {
            Ok(index) => Ok(index.into()),
            Err(ArrowError::DictionaryKeyOverflowError) => {
                Err(super::dictionary::checked::DictionaryBuilderError::DictOverflow {})
            }
            Err(e) => Err(
                super::dictionary::checked::DictionaryBuilderError::CheckedBuilderError {
                    source: e,
                },
            ),
        }
    }
}

impl<K> ArrayAppendNulls for FixedSizeBinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        // TODO - after the next release of arrow-rs we should revisit this and call append_nulls
        // on the base builder. Waiting on these changes to be released:
        // https://github.com/apache/arrow-rs/pull/7606
        for _ in 0..n {
            self.append_null();
        }
    }
}

impl<K> DictionaryBuilder<K> for FixedSizeBinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    fn finish(&mut self) -> DictionaryArray<K> {
        self.finish()
    }
}

impl<K> ConvertToNativeHelper for FixedSizeBinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    type Accessor = FixedSizeBinaryArray;
}

impl UpdateDictionaryIndexInto<FixedSizeBinaryDictionaryBuilder<UInt16Type>>
    for FixedSizeBinaryDictionaryBuilder<UInt8Type>
{
    fn upgrade_into(&mut self) -> FixedSizeBinaryDictionaryBuilder<UInt16Type> {
        // TODO there should be an optimized way to implement this. Thinking we could
        // create a new builder with the same keys (but use `cast` kernel) cast them
        // to u16 then reuse the same values
        // related issue https://github.com/open-telemetry/otel-arrow/issues/536

        let dict_arr = self.finish();

        // safety: DictionaryBuilder returns a dictionary that has Primitive values
        let values = dict_arr
            .downcast_dict::<FixedSizeBinaryArray>()
            .expect("expect values are of type FixedSizeBinary");

        let mut upgraded_builder =
            FixedSizeBinaryDictionaryBuilder::<UInt16Type>::new(values.values().value_length());
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
    use arrow::array::{
        Array, FixedSizeBinaryArray, FixedSizeBinaryBuilder, UInt8Array, UInt8DictionaryArray,
    };
    use arrow::datatypes::DataType;

    #[test]
    fn test_fsb_builder() {
        let mut fsb_builder = FixedSizeBinaryBuilder::new(4);
        CheckedArrayAppend::append_value(&mut fsb_builder, &b"1234".to_vec()).unwrap();
        CheckedArrayAppend::append_value(&mut fsb_builder, &b"5678".to_vec()).unwrap();
        CheckedArrayAppend::append_value(&mut fsb_builder, &b"9012".to_vec()).unwrap();
        let result = ArrayBuilder::finish(&mut fsb_builder);
        assert_eq!(result.data_type(), &DataType::FixedSizeBinary(4));

        let expected = FixedSizeBinaryArray::try_from_iter(
            [b"1234".to_vec(), b"5678".to_vec(), b"9012".to_vec()].iter(),
        )
        .unwrap();

        assert_eq!(
            result
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap(),
            &expected
        );
    }

    #[test]
    fn test_dict_fsb_builder() {
        let mut dict_builder = FixedSizeBinaryDictionaryBuilder::<UInt8Type>::new(1);
        let index =
            CheckedDictionaryArrayAppend::append_value(&mut dict_builder, &b"a".to_vec()).unwrap();
        assert_eq!(index, 0);
        let index =
            CheckedDictionaryArrayAppend::append_value(&mut dict_builder, &b"a".to_vec()).unwrap();
        assert_eq!(index, 0);
        let index =
            CheckedDictionaryArrayAppend::append_value(&mut dict_builder, &b"b".to_vec()).unwrap();
        assert_eq!(index, 1);

        let result = DictionaryBuilder::finish(&mut dict_builder);

        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(
                Box::new(DataType::UInt8),
                Box::new(DataType::FixedSizeBinary(1))
            )
        );

        let mut expected_dict_values = FixedSizeBinaryBuilder::new(1);
        assert!(expected_dict_values.append_value(b"a").is_ok());
        assert!(expected_dict_values.append_value(b"b").is_ok());
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
    fn test_dict_fsb_builder_overflow() {
        let mut dict_builder = FixedSizeBinaryDictionaryBuilder::<UInt8Type>::new(2);
        for i in 0..255 {
            let _ =
                CheckedDictionaryArrayAppend::append_value(&mut dict_builder, &vec![0, i]).unwrap();
        }

        // this should be fine
        let _ = CheckedDictionaryArrayAppend::append_value(&mut dict_builder, &vec![1, 0]).unwrap();

        // this should overflow
        let result = CheckedDictionaryArrayAppend::append_value(&mut dict_builder, &vec![1, 1]);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(
            err,
            super::super::dictionary::checked::DictionaryBuilderError::DictOverflow {}
        ));
    }
}
