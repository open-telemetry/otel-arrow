use std::sync::Arc;

use arrow::array::{
    Array, ArrowPrimitiveType, DictionaryArray, StringArray, StringBuilder, StringDictionaryBuilder,
};
use arrow::datatypes::{ArrowDictionaryKeyType, DataType, UInt8Type, UInt16Type};
use arrow::error::ArrowError;

use crate::encode::record::array::dictionary::DictionaryArrayWithType;

use super::dictionary::{DictionaryArrayBuilder, DictionaryBuilderError as Error, Result};
use super::{ArrayBuilder, ArrayBuilderConstructor, dictionary::UpdateDictionaryIndexInto};

impl ArrayBuilderConstructor for StringBuilder {
    fn new() -> Self {
        Self::new()
    }
}

impl ArrayBuilder for StringBuilder {
    type Native = String;

    fn append_value(&mut self, value: &Self::Native) {
        self.append_value(value);
    }

    fn finish(&mut self) -> super::ArrayWithType {
        super::ArrayWithType {
            data_type: DataType::Utf8,
            array: Arc::new(self.finish()),
        }
    }
}

impl<T> ArrayBuilderConstructor for StringDictionaryBuilder<T>
where
    T: ArrowDictionaryKeyType,
{
    fn new() -> Self {
        Self::new()
    }
}

impl<T> DictionaryArrayBuilder<T> for StringDictionaryBuilder<T>
where
    T: ArrowDictionaryKeyType + ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: Into<usize>,
{
    type Native = String;

    fn append_value(&mut self, value: &Self::Native) -> Result<usize> {
        match self.append(value) {
            Ok(index) => Ok(index.into()),
            Err(ArrowError::DictionaryKeyOverflowError) => Err(Error::DictOverflow {}),

            // safety: shouldn't happen. The only error type append should
            // return should be for dictionary overflows
            Err(e) => panic!("unexpected error type appending to dictionary {}", e),
        }
    }

    fn finish(&mut self) -> DictionaryArrayWithType<T> {
        DictionaryArrayWithType {
            data_type: DataType::Dictionary(Box::new(T::DATA_TYPE), Box::new(DataType::Utf8)),
            array: Arc::new(self.finish()),
        }
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
    fn upgrade_into(&mut self) -> StringDictionaryBuilder<UInt16Type> {
        // TODO there should be an optimized way to implement this. Thinking we could
        // create a new builder with the same keys (but use `cast` kernel) cast them
        // to u16 then reuse the same values

        let dict_arr = self.finish();

        // safety: StringDictionaryBuilder returns a dictionary that has String values
        let str_values = dict_arr
            .downcast_dict::<StringArray>()
            .expect("expect values are of type string");

        let mut upgraded_builder = StringDictionaryBuilder::<UInt16Type>::new();
        for str_value in str_values {
            match str_value {
                Some(str_value) => upgraded_builder.append_value(str_value),
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

    use arrow::array::{UInt8Array, UInt8DictionaryArray};
    use arrow::datatypes::UInt8Type;

    use crate::encode::record::array::ArrayBuilder;

    #[test]
    fn test_string_builder() {
        let mut string_builder = StringBuilder::new();
        ArrayBuilder::append_value(&mut string_builder, &"a".to_string());
        ArrayBuilder::append_value(&mut string_builder, &"b".to_string());
        ArrayBuilder::append_value(&mut string_builder, &"c".to_string());

        let result = ArrayBuilder::finish(&mut string_builder);

        assert_eq!(result.data_type, DataType::Utf8);

        let expected = StringArray::from(vec!["a", "b", "c"]);
        assert_eq!(
            result.array.as_any().downcast_ref::<StringArray>().unwrap(),
            &expected
        );
    }

    #[test]
    fn test_dictionary_builder() {
        let mut dict_builder = StringDictionaryBuilder::<UInt8Type>::new();
        let index =
            DictionaryArrayBuilder::append_value(&mut dict_builder, &"a".to_string()).unwrap();
        assert_eq!(index, 0);
        let index =
            DictionaryArrayBuilder::append_value(&mut dict_builder, &"a".to_string()).unwrap();
        assert_eq!(index, 0);
        let index =
            DictionaryArrayBuilder::append_value(&mut dict_builder, &"b".to_string()).unwrap();
        assert_eq!(index, 1);

        let result = DictionaryArrayBuilder::finish(&mut dict_builder);

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
    fn test_dictionary_builder_overflow() {
        let mut dict_builder = StringDictionaryBuilder::<UInt8Type>::new();
        for i in 0..255 {
            let _ =
                DictionaryArrayBuilder::append_value(&mut dict_builder, &i.to_string()).unwrap();
        }

        // this should be fine
        let _ = DictionaryArrayBuilder::append_value(&mut dict_builder, &"a".to_string()).unwrap();

        // this should overflow
        let result = DictionaryArrayBuilder::append_value(&mut dict_builder, &"b".to_string());
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, Error::DictOverflow {}));
    }
}
