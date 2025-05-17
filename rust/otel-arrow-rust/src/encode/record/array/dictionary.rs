use arrow::{
    array::{ArrowPrimitiveType, DictionaryArray},
    datatypes::{ArrowDictionaryKeyType, UInt8Type, UInt16Type},
};
use snafu::Snafu;

use crate::arrays::NullableArrayAccessor;

use super::{ArrayBuilder, ArrayBuilderConstructor};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum DictionaryBuilderError {
    #[snafu(display("dict overflow"))]
    DictOverflow {},
}

pub type Result<T> = std::result::Result<T, DictionaryBuilderError>;

// This is the base trait for array builder implementations that are
// used to construct dictionary arrays
pub trait DictionaryArrayBuilder {
    type Native;

    // Append a new value to the dictionary, and return the index of
    // the keys array. The returned index can by AdaptiveDictionaryBuilder
    // to determine if the dictionary overflows.
    //
    // If the implementing builder can determine internally that the dictionary
    // would overflow, it can also return `DictOverflow` error
    fn append_value(&mut self, value: &Self::Native) -> Result<usize>;

    fn finish(&mut self) -> super::ArrayWithType;
}

// This †rait is used to help the AdaptiveDictionaryBuilder to convert the dictionary
// array constructed by the underlying builder to the native type. It will use the
// associated `Accessor` type to downcast the values array, so by implementing this trait
// implementations of `DictionaryArrayBuilder` have a way to signal to
// `AdaptiveDictionaryBuilder` what is the underlying type of the values array.
pub trait ConvertToNativeHelper {
    type Accessor;
}

// Implementaions of this trait are used to upgrade from a builder for a dictionary
// keyed by a smaller index type into a larger typer. E.g. a builder for
// DictionaryArray<u8> -> DictionaryArray<u16>
pub trait UpdateDictionaryIndexInto<T> {
    fn upgrade_into(&mut self) -> T;
}

pub struct DictionaryOptions {
    pub max_cardinality: u16,
    pub min_cardinality: u16,
    // TODO there's something called reset_threshold in the golang code
    // that maybe we need to add here?
}

enum DictIndexVariant<T8, T16> {
    UInt8(T8),
    UInt16(T16),
}

pub struct AdaptiveDictionaryBuilder<T8, T16> {
    max_cardinality: u16,
    variant: DictIndexVariant<T8, T16>,

    // This is the index of the key array in the builder at which an
    // overflow was detected. This will be set when the underlying builder
    // could not detect a-priori that inserting the value would cause an
    // an overflow e.g. b/c it didn't know about this parent builder's
    // max_cardinality.
    overflow_index: Option<usize>,
}

impl<T8, T16> AdaptiveDictionaryBuilder<T8, T16>
where
    T8: ArrayBuilderConstructor + UpdateDictionaryIndexInto<T16>,
    T16: ArrayBuilderConstructor,
{
    pub fn new(options: &DictionaryOptions) -> Self {
        // choose the default dictionary index type to be the smallest that can
        // hold the min cardinality
        let variant = if options.min_cardinality <= u8::MAX.into() {
            DictIndexVariant::UInt8(T8::new())
        } else {
            DictIndexVariant::UInt16(T16::new())
        };

        Self {
            max_cardinality: options.max_cardinality,
            variant,
            overflow_index: None,
        }
    }

    fn upgrade_key(&mut self) -> Result<()> {
        match &mut self.variant {
            DictIndexVariant::UInt8(dict_builder) => {
                // if the max cardinality is less than what the next bigger
                // index type can hold, we don't want to upgrade
                if self.max_cardinality <= u8::MAX.into() {
                    return DictOverflowSnafu.fail();
                }

                let next_bigger = dict_builder.upgrade_into();
                self.variant = DictIndexVariant::UInt16(next_bigger);

                Ok(())
            }
            _ => DictOverflowSnafu.fail(),
        }
    }
}

impl<T, T8, T16> AdaptiveDictionaryBuilder<T8, T16>
where
    T8: DictionaryArrayBuilder<Native = T> + ConvertToNativeHelper,
    <T8 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
    T16: DictionaryArrayBuilder<Native = T> + ConvertToNativeHelper,
    <T16 as ConvertToNativeHelper>::Accessor: NullableArrayAccessor<Native = T> + 'static,
{
    pub fn to_native<TN>(&mut self, builder: &mut TN)
    where
        TN: ArrayBuilder<Native = T>,
    {
        match &mut self.variant {
            DictIndexVariant::UInt8(dict_builder) => {
                let result = dict_builder.finish();
                let dict_arr = result
                    .array
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    // TODO - instead of unwrapping here, we might want to enforce that this builder returns a DictArray<UInt8Type>
                    .unwrap();
                populate_native_builder::<_, UInt8Type, <T8 as ConvertToNativeHelper>::Accessor, _>(
                    dict_arr,
                    builder,
                    self.overflow_index,
                );
            }

            DictIndexVariant::UInt16(dict_builder) => {
                let result = dict_builder.finish();
                // TODO - see comment above related to unwrap the downcast
                let dict_arr = result
                    .array
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt16Type>>()
                    .unwrap();
                populate_native_builder::<_, UInt16Type, <T16 as ConvertToNativeHelper>::Accessor, _>(
                    dict_arr,
                    builder,
                    self.overflow_index,
                );
            }
        }
    }
}

// This helper function populates the native builder from the dict values in a way
// that is generic over the type of dictionary key
fn populate_native_builder<T, K, V, TN>(
    dict_arr: &DictionaryArray<K>,
    builder: &mut TN,
    overflow_index: Option<usize>,
) where
    TN: ArrayBuilder<Native = T>,
    K: ArrowDictionaryKeyType,
    <K as ArrowPrimitiveType>::Native: Into<usize>,
    V: NullableArrayAccessor<Native = T> + 'static,
{
    let keys = dict_arr.keys();
    let values = dict_arr.values().as_any().downcast_ref::<V>().unwrap();

    for i in 0..dict_arr.len() {
        // TODO need to handle nulls here
        let key = keys.value(i);
        let index = key.into();

        // break if we find the index that caused the overflow
        if overflow_index == Some(index) {
            break;
        }

        // TODO no unwrap here when we handle nulls
        let value = values.value_at(index).unwrap();
        builder.append_value(&value);
    }
}

impl<T, T8, T16> DictionaryArrayBuilder for AdaptiveDictionaryBuilder<T8, T16>
where
    T8: DictionaryArrayBuilder<Native = T>
        + ArrayBuilderConstructor
        + UpdateDictionaryIndexInto<T16>,
    T16: DictionaryArrayBuilder<Native = T> + ArrayBuilderConstructor,
{
    type Native = T;

    fn append_value(&mut self, value: &Self::Native) -> Result<usize> {
        let append_result = match &mut self.variant {
            DictIndexVariant::UInt8(dict_builder) => dict_builder.append_value(value),
            DictIndexVariant::UInt16(dict_builder) => dict_builder.append_value(value),
        };

        match append_result {
            Ok(index) => {
                if index + 1 > self.max_cardinality as usize {
                    // if we're here, it means we did append successfully to the underlying builder
                    // but we shouldn't have, because have overflowed the configured max cardinality
                    self.overflow_index = Some(index);
                    Err(DictionaryBuilderError::DictOverflow {})
                } else {
                    Ok(index)
                }
            }
            Err(DictionaryBuilderError::DictOverflow {}) => {
                self.upgrade_key()?;
                self.append_value(value)
            }
        }
    }

    fn finish(&mut self) -> super::ArrayWithType {
        match &mut self.variant {
            DictIndexVariant::UInt8(u8_dict_builder) => u8_dict_builder.finish(),
            DictIndexVariant::UInt16(u16_dict_builder) => u16_dict_builder.finish(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::sync::Arc;
    use std::u16;

    use arrow::array::{
        StringBuilder, StringDictionaryBuilder, UInt8Array, UInt8DictionaryArray, UInt16Array,
        UInt16DictionaryArray,
    };
    use arrow::datatypes::{DataType, UInt8Type, UInt16Type};

    type TestDictBuilder = AdaptiveDictionaryBuilder<
        StringDictionaryBuilder<UInt8Type>,
        StringDictionaryBuilder<UInt16Type>,
    >;

    #[test]
    fn test_dict_builder() {
        let mut dict_builder = TestDictBuilder::new(&DictionaryOptions {
            max_cardinality: u16::MAX,
            min_cardinality: u8::MAX.into(),
        });

        let index = dict_builder.append_value(&"a".to_string()).unwrap();
        assert_eq!(index, 0);
        let index = dict_builder.append_value(&"a".to_string()).unwrap();
        assert_eq!(index, 0);
        let index = dict_builder.append_value(&"b".to_string()).unwrap();
        assert_eq!(index, 1);

        let result = dict_builder.finish();

        assert_eq!(
            result.data_type,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        );

        let mut expected_dict_values = StringBuilder::new();
        expected_dict_values.append_value("a");
        expected_dict_values.append_value("b");
        let expected_dict_keys = UInt8Array::from_iter_values(vec![0, 0, 1]);
        let expected =
            UInt8DictionaryArray::new(expected_dict_keys, Arc::new(expected_dict_values.finish()));

        assert_eq!(
            result
                .array
                .as_any()
                .downcast_ref::<UInt8DictionaryArray>()
                .unwrap(),
            &expected
        );
    }

    #[test]
    fn test_dict_builder_update_index_type() {
        let mut dict_builder = TestDictBuilder::new(&DictionaryOptions {
            max_cardinality: u16::MAX,
            min_cardinality: u8::MAX.into(),
        });

        for i in 0..257 {
            let _ = dict_builder.append_value(&i.to_string()).unwrap();
        }

        let result = dict_builder.finish();

        assert_eq!(
            result.data_type,
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8))
        );

        // check that the dictionary is the correct type
        let dict_array = result
            .array
            .as_any()
            .downcast_ref::<UInt16DictionaryArray>();
        assert!(dict_array.is_some(), "Expected a UInt16DictionaryArray");
    }

    #[test]
    fn test_dict_max_cardinality() {
        let mut dict_builder = TestDictBuilder::new(&DictionaryOptions {
            max_cardinality: u8::MAX as u16 + 1,
            min_cardinality: u8::MAX as u16 + 1,
        });

        for i in 0..u8::MAX {
            let _ = dict_builder.append_value(&i.to_string()).unwrap();
        }

        // this should be fine
        let _ = dict_builder.append_value(&"a".to_string()).unwrap();

        // should overflow the max cardinality
        let result = dict_builder.append_value(&"b".to_string());
        assert!(
            result.is_err(),
            "Expected an error due to exceeding max cardinality"
        );
        assert!(
            matches!(result.unwrap_err(), DictionaryBuilderError::DictOverflow {}),
            "Expected a DictOverflow error"
        );
    }

    #[test]
    fn test_dict_min_cardinality() {
        // test that we can force the dictionary index to be bigger type than is needed
        // by specifying the min cardinality.
        let mut dict_builder = TestDictBuilder::new(&DictionaryOptions {
            max_cardinality: u16::MAX,
            min_cardinality: u16::MAX,
        });

        let _ = dict_builder.append_value(&"a".to_string()).unwrap();
        let _ = dict_builder.append_value(&"a".to_string()).unwrap();
        let _ = dict_builder.append_value(&"b".to_string()).unwrap();

        let result = dict_builder.finish();

        assert_eq!(
            result.data_type,
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8))
        );

        let mut expected_dict_values = StringBuilder::new();
        expected_dict_values.append_value("a");
        expected_dict_values.append_value("b");
        let expected_dict_keys = UInt16Array::from_iter_values(vec![0, 0, 1]);
        let expected =
            UInt16DictionaryArray::new(expected_dict_keys, Arc::new(expected_dict_values.finish()));

        assert_eq!(
            result
                .array
                .as_any()
                .downcast_ref::<UInt16DictionaryArray>()
                .unwrap(),
            &expected
        );
    }

    #[test]
    fn test_dict_arbitrary_max_cardinality() {
        // check that we support a max-cardinality that is arbitrarily aligned
        // e.g. not necessarily alighed to u8/u16 max values
        let mut dict_builder = TestDictBuilder::new(&DictionaryOptions {
            max_cardinality: 4,
            min_cardinality: 4,
        });

        let _ = dict_builder.append_value(&"a".to_string()).unwrap();
        let _ = dict_builder.append_value(&"b".to_string()).unwrap();
        let _ = dict_builder.append_value(&"c".to_string()).unwrap();
        let _ = dict_builder.append_value(&"d".to_string()).unwrap();

        // this should be OK, we are re-adding an existing value so it should not
        // affect the size of the dictionary
        let _ = dict_builder.append_value(&"d".to_string()).unwrap();

        // this should exceed the max cardinality:
        let result = dict_builder.append_value(&"e".to_string());

        assert!(
            matches!(result.unwrap_err(), DictionaryBuilderError::DictOverflow {}),
            "Expected a DictOverflow error"
        );
    }
}
