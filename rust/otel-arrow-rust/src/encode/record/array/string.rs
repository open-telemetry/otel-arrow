// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! this module contains the base builders for building strings using the adaptive array builders

use std::sync::Arc;

use arrow::array::{
    ArrayRef, ArrowPrimitiveType, DictionaryArray, StringArray, StringBuilder,
    StringDictionaryBuilder,
};
use arrow::datatypes::{ArrowDictionaryKeyType, UInt8Type, UInt16Type};
use arrow::error::ArrowError;

use crate::encode::record::array::dictionary::{DictionaryArrayAppendStr, DictionaryBuilder};
use crate::encode::record::array::{
    ArrayAppend, ArrayAppendNulls, ArrayAppendStr, DefaultValueProvider, NoArgs,
};

use super::dictionary::{DictionaryArrayAppend, DictionaryBuilderError as Error, Result};
use super::{ArrayBuilder, ArrayBuilderConstructor, dictionary::UpdateDictionaryIndexInto};

impl ArrayBuilderConstructor for StringBuilder {
    type Args = NoArgs;

    fn new(_args: Self::Args) -> Self {
        Self::new()
    }
}

impl ArrayAppend for StringBuilder {
    type Native = String;

    fn append_value(&mut self, value: &Self::Native) {
        self.append_value(value);
    }

    fn append_value_n(&mut self, value: &Self::Native, n: usize) {
        for _ in 0..n {
            self.append_value(value);
        }
    }
}

impl ArrayAppendStr for StringBuilder {
    fn append_str(&mut self, value: &str) {
        self.append_value(value);
    }

    fn append_str_n(&mut self, value: &str, n: usize) {
        for _ in 0..n {
            self.append_value(value);
        }
    }
}

impl ArrayAppendNulls for StringBuilder {
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.append_nulls(n);
    }
}

impl DefaultValueProvider<String, NoArgs> for StringBuilder {
    fn default_value(_args: NoArgs) -> String {
        String::new()
    }
}

impl ArrayBuilder for StringBuilder {
    fn finish(&mut self) -> ArrayRef {
        Arc::new(self.finish())
    }
}

impl<T> ArrayBuilderConstructor for StringDictionaryBuilder<T>
where
    T: ArrowDictionaryKeyType,
{
    type Args = ();

    fn new(_args: Self::Args) -> Self {
        Self::new()
    }
}

impl<T> DictionaryArrayAppend for StringDictionaryBuilder<T>
where
    T: ArrowDictionaryKeyType + ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: Into<usize>,
{
    type Native = String;

    fn append_value(&mut self, value: &Self::Native) -> Result<usize> {
        self.append_str(value.as_str())
    }

    fn append_values(&mut self, value: &Self::Native, n: usize) -> Result<usize> {
        self.append_str_n(value, n)
    }
}

impl<T> ArrayAppendNulls for StringDictionaryBuilder<T>
where
    T: ArrowDictionaryKeyType,
{
    fn append_null(&mut self) {
        self.append_null();
    }

    fn append_nulls(&mut self, n: usize) {
        self.append_nulls(n);
    }
}

impl<T> DictionaryArrayAppendStr for StringDictionaryBuilder<T>
where
    T: ArrowDictionaryKeyType + ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: Into<usize>,
{
    fn append_str(&mut self, value: &str) -> Result<usize> {
        match self.append(value) {
            Ok(index) => Ok(index.into()),
            Err(ArrowError::DictionaryKeyOverflowError) => Err(Error::DictOverflow {}),

            // safety: shouldn't happen. The only error type append should
            // return should be for dictionary overflows
            Err(e) => panic!("unexpected error type appending to dictionary {e}"),
        }
    }

    fn append_str_n(&mut self, value: &str, n: usize) -> Result<usize> {
        match self.append_n(value, n) {
            Ok(index) => Ok(index.into()),
            Err(ArrowError::DictionaryKeyOverflowError) => Err(Error::DictOverflow {}),

            // safety: shouldn't happen. The only error type append should
            // return should be for dictionary overflows
            Err(e) => panic!("unexpected error type appending to dictionary {e}"),
        }
    }
}

impl<K> DictionaryBuilder<K> for StringDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    fn finish(&mut self) -> DictionaryArray<K> {
        self.finish()
    }
}

impl<K> super::dictionary::ConvertToNativeHelper for StringDictionaryBuilder<K>
where
    K: ArrowDictionaryKeyType,
{
    type Accessor = StringArray;
}

impl UpdateDictionaryIndexInto<StringDictionaryBuilder<UInt16Type>>
    for StringDictionaryBuilder<UInt8Type>
{
    fn upgrade_into(self) -> StringDictionaryBuilder<UInt16Type> {
        // safety: `try_new_from_builder` will return an error here if the source key type cannot
        // be upgraded into the source key type. This can happen if going signed -> unsigned and there
        // are negative keys, or if going from a bigger type to smaller and some keys would not fit
        // int the smaller type. This won't happen going u8 to u16
        StringDictionaryBuilder::try_new_from_builder(self).expect("can upgrade from u8 to u16 key")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{Array, UInt8Array, UInt8DictionaryArray};
    use arrow::datatypes::{DataType, UInt8Type};

    use crate::encode::record::array::ArrayBuilder;

    #[test]
    fn test_string_builder() {
        let mut string_builder = StringBuilder::new();
        ArrayAppend::append_value(&mut string_builder, &"a".to_string());
        ArrayAppend::append_value(&mut string_builder, &"b".to_string());
        ArrayAppend::append_value(&mut string_builder, &"c".to_string());
        string_builder.append_str("d");
        string_builder.append_str_n("e", 2);

        let result = ArrayBuilder::finish(&mut string_builder);

        assert_eq!(result.data_type(), &DataType::Utf8);

        let expected = StringArray::from(vec!["a", "b", "c", "d", "e", "e"]);
        assert_eq!(
            result.as_any().downcast_ref::<StringArray>().unwrap(),
            &expected
        );
    }

    #[test]
    fn test_dictionary_builder() {
        let mut dict_builder = StringDictionaryBuilder::<UInt8Type>::new();
        let index =
            DictionaryArrayAppend::append_value(&mut dict_builder, &"a".to_string()).unwrap();
        assert_eq!(index, 0);
        let index =
            DictionaryArrayAppend::append_value(&mut dict_builder, &"a".to_string()).unwrap();
        assert_eq!(index, 0);
        let index =
            DictionaryArrayAppend::append_value(&mut dict_builder, &"b".to_string()).unwrap();
        assert_eq!(index, 1);

        let result = DictionaryBuilder::finish(&mut dict_builder);

        assert_eq!(
            result.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        );

        let mut expected_dict_values = StringBuilder::new();
        expected_dict_values.append_value("a");
        expected_dict_values.append_value("b");
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
        let mut dict_builder = StringDictionaryBuilder::<UInt8Type>::new();
        for i in 0..255 {
            let _ = DictionaryArrayAppend::append_value(&mut dict_builder, &i.to_string()).unwrap();
        }

        // this should be fine
        let _ = DictionaryArrayAppend::append_value(&mut dict_builder, &"a".to_string()).unwrap();

        // this should overflow
        let result = DictionaryArrayAppend::append_value(&mut dict_builder, &"b".to_string());
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, Error::DictOverflow {}));
    }
}
