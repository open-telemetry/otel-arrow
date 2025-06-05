// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::{
    array::{
        Array, ArrayRef, ArrowPrimitiveType, DictionaryArray, FixedSizeBinaryArray,
        FixedSizeBinaryBuilder, FixedSizeBinaryDictionaryBuilder,
    },
    datatypes::{ArrowDictionaryKeyType, DataType, UInt8Type, UInt16Type},
    error::ArrowError,
};

use crate::encode::record::array::dictionary::{
    CheckedDictionaryArrayAppend, ConvertToNativeHelper, DictionaryArrayAppend, DictionaryBuilder,
    UpdateDictionaryIndexInto,
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
    use arrow::array::{FixedSizeBinaryArray, FixedSizeBinaryBuilder};
    use arrow::datatypes::DataType;

    #[test]
    fn test_fsb_builder() {
        // let mut fsb_builder = FixedSizeBinaryBuilder::new(4);
        // ArrayBuilder::append_value(&mut fsb_builder, &b"1234".to_vec());
        // ArrayBuilder::append_value(&mut fsb_builder, &b"5678".to_vec());
        // ArrayBuilder::append_value(&mut fsb_builder, &b"9012".to_vec());
        // let result = ArrayBuilder::finish(&mut fsb_builder);
        // assert_eq!(result.data_type, DataType::FixedSizeBinary(4));

        // let expected = FixedSizeBinaryArray::try_from_iter(
        //     vec![b"1234".to_vec(), b"5678".to_vec(), b"9012".to_vec()].iter(),
        // )
        // .unwrap();

        // assert_eq!(
        //     result
        //         .array
        //         .as_any()
        //         .downcast_ref::<FixedSizeBinaryArray>()
        //         .unwrap(),
        //     &expected
        // );
    }
}
