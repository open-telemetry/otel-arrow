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
    ArrayAppendNulls, CheckedArrayAppendSlice, DefaultValueProvider,
    dictionary::{
        CheckedDictionaryAppendSlice, CheckedDictionaryArrayAppend, ConvertToNativeHelper,
        DictionaryBuilder, UpdateDictionaryIndexInto,
    },
};

use super::{ArrayBuilder, ArrayBuilderConstructor, ArrayLen, CheckedArrayAppend};

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

impl DefaultValueProvider<Vec<u8>, i32> for FixedSizeBinaryBuilder {
    fn default_value(byte_width: i32) -> Vec<u8> {
        vec![0; byte_width as usize]
    }
}

impl CheckedArrayAppendSlice for FixedSizeBinaryBuilder {
    type Native = u8;

    fn append_slice(&mut self, val: &[Self::Native]) -> Result<(), ArrowError> {
        self.append_value(val)
    }

    fn append_slice_n(&mut self, val: &[Self::Native], n: usize) -> Result<(), ArrowError> {
        for _ in 0..n {
            self.append_value(val)?;
        }

        Ok(())
    }
}

impl ArrayAppendNulls for FixedSizeBinaryBuilder {
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.append_nulls(n);
    }
}

impl ArrayBuilder for FixedSizeBinaryBuilder {
    fn finish(&mut self) -> ArrayRef {
        Arc::new(self.finish())
    }
}

impl ArrayLen for FixedSizeBinaryBuilder {
    fn len(&self) -> usize {
        arrow::array::ArrayBuilder::len(self)
    }
}

impl<K> ArrayLen for FixedSizeBinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    fn len(&self) -> usize {
        arrow::array::ArrayBuilder::len(self)
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
        self.append_slice(value)
    }
}

impl<K> CheckedDictionaryAppendSlice for FixedSizeBinaryDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
{
    type Native = u8;

    fn append_slice(
        &mut self,
        value: &[Self::Native],
    ) -> super::dictionary::checked::Result<usize> {
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

    fn append_slice_n(
        &mut self,
        val: &[Self::Native],
        n: usize,
    ) -> super::dictionary::checked::Result<usize> {
        // Arrow still validates and inserts dictionary values when the count is zero.
        if n == 0 {
            return Ok(0);
        }

        match self.append_n(val, n) {
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
        self.append_nulls(n);
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
    fn upgrade_into(self) -> FixedSizeBinaryDictionaryBuilder<UInt16Type> {
        // safety: `try_new_from_builder` will return an error here if the source key type cannot
        // be upgraded into the source key type. This can happen if going signed -> unsigned and there
        // are negative keys, or if going from a bigger type to smaller and some keys would not fit
        // int the smaller type. This won't happen going u8 to u16
        FixedSizeBinaryDictionaryBuilder::try_new_from_builder(self)
            .expect("can upgrade from u8 to u16 key")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::encode::record::array::dictionary::{
        AdaptiveDictionaryBuilder, DictionaryOptions, checked::DictionaryBuilderError,
    };
    use arrow::array::{
        Array, FixedSizeBinaryArray, FixedSizeBinaryBuilder, UInt8Array, UInt8DictionaryArray,
    };
    use arrow::datatypes::DataType;

    /// Scenario: Repeated span- and trace-sized values are interleaved with nulls.
    /// Guarantees: Bulk appends preserve scalar indices, key order, nulls, and dictionary values.
    #[test]
    fn test_dict_fsb_bulk_matches_scalar() {
        for width in [8, 16] {
            let mut bulk = FixedSizeBinaryDictionaryBuilder::<UInt8Type>::new(width);
            let mut scalar = FixedSizeBinaryDictionaryBuilder::<UInt8Type>::new(width);
            for (byte, count) in [(1, 1), (2, 4), (1, 3), (3, 1024)] {
                bulk.append_null();
                scalar.append_null();
                let value = vec![byte; width as usize];
                let index = bulk.append_slice_n(&value, count).unwrap();
                for _ in 0..count {
                    assert_eq!(scalar.append_slice(&value).unwrap(), index);
                }
            }
            let bulk = bulk.finish();
            let scalar = scalar.finish();
            assert_eq!(bulk.keys(), scalar.keys());
            assert_eq!(bulk.values().to_data(), scalar.values().to_data());
        }
    }

    /// Scenario: Empty, populated, and full dictionaries receive zero-count bulk appends.
    /// Guarantees: New, existing, and invalid-width values return zero without changing state.
    #[test]
    fn test_dict_fsb_bulk_zero_count_is_noop() {
        for cardinality in [0_u16, 2, 256] {
            let mut builder = FixedSizeBinaryDictionaryBuilder::<UInt8Type>::new(2);
            for value in 0..cardinality {
                let _ = builder.append_slice(&value.to_be_bytes()).unwrap();
            }
            let before = builder.finish_cloned();
            for value in [b"".as_slice(), b"x", b"xyz", &[0, 1], &[255, 255]] {
                assert_eq!(builder.append_slice_n(value, 0).unwrap(), 0);
            }
            let after = builder.finish();
            assert_eq!(after.keys(), before.keys());
            assert_eq!(after.values().to_data(), before.values().to_data());
        }
    }

    /// Scenario: A bulk append has the wrong width, including when dictionary keys are exhausted.
    /// Guarantees: Width errors remain checked Arrow errors and leave keys and values unchanged.
    #[test]
    fn test_dict_fsb_bulk_invalid_width() {
        for cardinality in [0_u16, 256] {
            let mut builder = FixedSizeBinaryDictionaryBuilder::<UInt8Type>::new(2);
            for value in 0..cardinality {
                let _ = builder.append_slice(&value.to_be_bytes()).unwrap();
            }
            let before = builder.finish_cloned();
            for value in [b"".as_slice(), b"x", b"xyz"] {
                let scalar_error = builder.append_slice(value).unwrap_err();
                let bulk_error = builder.append_slice_n(value, 3).unwrap_err();
                let DictionaryBuilderError::CheckedBuilderError { source } = bulk_error else {
                    panic!("expected checked width error");
                };
                assert!(matches!(source, ArrowError::InvalidArgumentError(_)));
                let DictionaryBuilderError::CheckedBuilderError {
                    source: scalar_source,
                } = scalar_error
                else {
                    panic!("expected scalar width error");
                };
                assert_eq!(source.to_string(), scalar_source.to_string());
            }
            let after = builder.finish();
            assert_eq!(after.keys(), before.keys());
            assert_eq!(after.values().to_data(), before.values().to_data());
        }
    }

    /// Scenario: A full u8 dictionary bulk-appends existing values and then one new value.
    /// Guarantees: Existing keys remain usable; overflow returns the checked variant without rows.
    #[test]
    fn test_dict_fsb_bulk_overflow() {
        let mut builder = FixedSizeBinaryDictionaryBuilder::<UInt8Type>::new(2);
        for value in 0_u16..256 {
            let _ = builder.append_slice(&value.to_be_bytes()).unwrap();
        }
        assert_eq!(
            builder.append_slice_n(&255_u16.to_be_bytes(), 3).unwrap(),
            255
        );
        let before = builder.finish_cloned();
        assert!(matches!(
            builder.append_slice_n(&256_u16.to_be_bytes(), 3),
            Err(DictionaryBuilderError::DictOverflow)
        ));
        let after = builder.finish();
        assert_eq!(after.keys(), before.keys());
    }

    /// Scenario: Repeated fixed-size values exhaust u8 keys in an adaptive dictionary.
    /// Guarantees: Upgrading to u16 retries the entire bulk append exactly once with scalar output.
    #[test]
    fn test_dict_fsb_bulk_adaptive_upgrade() {
        let new_builder = || {
            AdaptiveDictionaryBuilder::<
                FixedSizeBinaryDictionaryBuilder<UInt8Type>,
                FixedSizeBinaryDictionaryBuilder<UInt16Type>,
            >::new(&DictionaryOptions::dict8(), 2)
        };
        let mut bulk = new_builder();
        let mut scalar = new_builder();
        for value in 0_u16..258 {
            let value = value.to_be_bytes();
            let index = bulk.append_slice_n_checked(&value, 3).unwrap();
            for _ in 0..3 {
                assert_eq!(scalar.append_slice_checked(&value).unwrap(), index);
            }
        }
        let bulk = bulk.finish();
        let scalar = scalar.finish();
        let bulk = bulk
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>()
            .unwrap();
        let scalar = scalar
            .as_any()
            .downcast_ref::<DictionaryArray<UInt16Type>>()
            .unwrap();
        assert_eq!(bulk.len(), 258 * 3);
        assert_eq!(bulk.keys(), scalar.keys());
        assert_eq!(bulk.values().to_data(), scalar.values().to_data());
    }

    #[test]
    fn test_fsb_builder() {
        let mut fsb_builder = FixedSizeBinaryBuilder::new(4);
        CheckedArrayAppend::append_value(&mut fsb_builder, &b"1234".to_vec()).unwrap();
        CheckedArrayAppend::append_value(&mut fsb_builder, &b"5678".to_vec()).unwrap();
        CheckedArrayAppend::append_value(&mut fsb_builder, &b"9012".to_vec()).unwrap();
        CheckedArrayAppendSlice::append_slice(&mut fsb_builder, b"4180").unwrap();
        CheckedArrayAppendSlice::append_slice_n(&mut fsb_builder, b"5140", 2).unwrap();

        let result = ArrayBuilder::finish(&mut fsb_builder);
        assert_eq!(result.data_type(), &DataType::FixedSizeBinary(4));

        let expected = FixedSizeBinaryArray::try_from_iter(
            [
                b"1234".to_vec(),
                b"5678".to_vec(),
                b"9012".to_vec(),
                b"4180".to_vec(),
                b"5140".to_vec(),
                b"5140".to_vec(),
            ]
            .iter(),
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
        let index = CheckedDictionaryAppendSlice::append_slice(&mut dict_builder, b"c").unwrap();
        assert_eq!(index, 2);
        let index =
            CheckedDictionaryAppendSlice::append_slice_n(&mut dict_builder, b"d", 2).unwrap();
        assert_eq!(index, 3);

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
        assert!(expected_dict_values.append_value(b"c").is_ok());
        assert!(expected_dict_values.append_value(b"d").is_ok());
        assert!(expected_dict_values.append_value(b"d").is_ok());
        let expected_dict_keys = UInt8Array::from_iter_values(vec![0, 0, 1, 2, 3, 3]);
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

    /// Scenario: Scalar appends exhaust the u8 dictionary key space.
    /// Guarantees: The first unrepresentable key returns the checked dictionary overflow error.
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
        assert!(matches!(err, DictionaryBuilderError::DictOverflow));
    }
}
