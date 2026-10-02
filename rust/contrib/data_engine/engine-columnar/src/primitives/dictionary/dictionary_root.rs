// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::{self, Write};

use arrow::{array::*, buffer::*, datatypes::*};
use otel_arrow_contrib_data_engine_expressions::*;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dictionary<'a> {
    keys: DictionaryKeyArray,
    values: DictionaryValueArray<'a>,
}

impl Dictionary<'_> {
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn is_null(&self) -> bool {
        self.keys.is_null() || self.values.is_null()
    }

    pub fn nulls(&self) -> Option<NullBuffer> {
        let key_nulls = self.keys.nulls();

        if let Some(v) = key_nulls.as_ref()
            && v.null_count() == self.keys.len()
        {
            return key_nulls;
        }

        if let Some(value_nulls) = self.values.nulls()
            && value_nulls.null_count() > 0
        {
            let key_length = self.keys.len();

            let mut builder: BooleanBufferBuilder = key_nulls.map_or_else(
                || {
                    let mut buffer = MutableBuffer::new_null(key_length);
                    buffer.fill(0xFF);
                    BooleanBufferBuilder::new_from_buffer(buffer, key_length)
                },
                |v| {
                    let mut b = MutableBuffer::with_capacity(v.len());
                    b.extend_from_slice(v.validity());
                    BooleanBufferBuilder::new_from_buffer(b, v.len())
                },
            );

            for key_index in 0..key_length {
                if let Some(value_index) = self.keys.get_value_index_for_key_index(key_index)
                    && value_nulls.is_null(value_index)
                {
                    builder.set_bit(key_index, false);
                }
            }

            return Some(NullBuffer::new(builder.into()));
        }

        key_nulls
    }

    pub fn keys(&self) -> &DictionaryKeyArray {
        &self.keys
    }

    pub fn get_value_index(&self, key_index: usize) -> Option<usize> {
        self.keys.get_value_index_for_key_index(key_index)
    }

    pub fn diagnostic_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{keys:[")?;
        for key in 0..self.keys.len() {
            if key > 0 {
                f.write_char(',')?;
            }
            write!(f, "{:?}", self.get_value_index(key))?;
        }
        f.write_str("],values:[")?;
        for value in 0..self.values.len() {
            if value > 0 {
                f.write_char(',')?;
            }
            self.values
                .get_value_at(value)
                .to_value()
                .diagnostic_fmt(f)?;
        }
        f.write_str("]}")
    }
}

impl<'a> Dictionary<'a> {
    pub fn new(keys: DictionaryKeyArray, values: DictionaryValueArray<'a>) -> Dictionary<'a> {
        Self { keys, values }
    }

    pub fn from_array<K: ArrowDictionaryKeyType, V: ArrowPrimitiveType>(
        values: &PrimitiveArray<V>,
    ) -> Dictionary<'a> {
        Self {
            keys: DictionaryKeyArray::UniqueValues {
                data_type: K::DATA_TYPE,
                length: values.len(),
            },
            values: (values as &dyn Array).into(),
        }
    }

    pub fn new_scalar_with_data_type(
        key_data_type: DataType,
        key_count: usize,
        value: ValueOrRef<'a>,
    ) -> Dictionary<'a> {
        match key_data_type {
            DataType::Int8 => Self::new_scalar::<Int8Type>(key_count, value),
            DataType::Int16 => Self::new_scalar::<Int16Type>(key_count, value),
            DataType::Int32 => Self::new_scalar::<Int32Type>(key_count, value),
            DataType::Int64 => Self::new_scalar::<Int64Type>(key_count, value),

            DataType::UInt8 => Self::new_scalar::<UInt8Type>(key_count, value),
            DataType::UInt16 => Self::new_scalar::<UInt16Type>(key_count, value),
            DataType::UInt32 => Self::new_scalar::<UInt32Type>(key_count, value),
            DataType::UInt64 => Self::new_scalar::<UInt64Type>(key_count, value),

            d => panic!("Unexpected dictionary key type '{d}' encountered"),
        }
    }

    pub fn new_scalar<K: ArrowDictionaryKeyType>(
        key_count: usize,
        value: ValueOrRef<'a>,
    ) -> Dictionary<'a> {
        Dictionary::new(
            DictionaryKeyArray::SingleValue {
                data_type: K::DATA_TYPE,
                length: key_count,
                value_index: Some(0),
            },
            vec![value].into(),
        )
    }

    pub fn new_null_with_data_type(count: usize, data_type: DataType) -> Dictionary<'a> {
        match data_type {
            DataType::Int8 => Self::new_null::<Int8Type>(count),
            DataType::Int16 => Self::new_null::<Int16Type>(count),
            DataType::Int32 => Self::new_null::<Int32Type>(count),
            DataType::Int64 => Self::new_null::<Int64Type>(count),

            DataType::UInt8 => Self::new_null::<UInt8Type>(count),
            DataType::UInt16 => Self::new_null::<UInt16Type>(count),
            DataType::UInt32 => Self::new_null::<UInt32Type>(count),
            DataType::UInt64 => Self::new_null::<UInt64Type>(count),

            d => panic!("Unexpected dictionary key type '{d}' encountered"),
        }
    }

    pub fn new_null<K: ArrowDictionaryKeyType>(count: usize) -> Dictionary<'a> {
        Dictionary::new(
            DictionaryKeyArray::SingleValue {
                data_type: K::DATA_TYPE,
                length: count,
                value_index: None,
            },
            vec![].into(),
        )
    }

    pub fn values(&self) -> &DictionaryValueArray<'a> {
        &self.values
    }

    pub fn into_parts(self) -> (DictionaryKeyArray, DictionaryValueArray<'a>) {
        (self.keys, self.values)
    }

    pub fn get_value(&self, key_index: usize) -> ValueOrRef<'a> {
        if let Some(value_index) = self.get_value_index(key_index) {
            return self.values.get_value_at(value_index);
        }

        ValueOrRef::Null
    }
}

impl<'a, T: ArrowDictionaryKeyType> From<&DictionaryArray<T>> for Dictionary<'a> {
    fn from(value: &DictionaryArray<T>) -> Self {
        Dictionary {
            keys: value.keys().into(),
            values: (value.values() as &dyn Array).into(),
        }
    }
}

impl<'a, 'b, K: ArrowDictionaryKeyType, V> From<TypedDictionaryArray<'b, K, V>> for Dictionary<'a>
where
    DictionaryValueArray<'a>: From<&'b V>,
{
    fn from(value: TypedDictionaryArray<'b, K, V>) -> Self {
        Dictionary {
            keys: value.keys().into(),
            values: value.values().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Dictionary keys include both null keys and keys that point to a null dictionary value.
    /// Guarantees: Row validity combines key and value validity, and value lookup returns Null for both cases.
    #[test]
    fn dictionary_combines_key_and_value_nulls() {
        let dictionary = Dictionary::new(
            DictionaryKeyArray::from(Int8Array::from(vec![Some(0), Some(1), None, Some(0)])),
            DictionaryValueArray::from(vec![ValueOrRef::Integer(10), ValueOrRef::Null]),
        );

        let nulls = dictionary.nulls().unwrap();
        assert_eq!(nulls.null_count(), 2);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_null(1));
        assert!(nulls.is_null(2));
        assert!(nulls.is_valid(3));
        assert_eq!(dictionary.get_value(0), ValueOrRef::Integer(10));
        assert_eq!(dictionary.get_value(1), ValueOrRef::Null);
        assert_eq!(dictionary.get_value(2), ValueOrRef::Null);
        assert_eq!(dictionary.get_value(4), ValueOrRef::Null);
    }

    /// Scenario: Scalar and null dictionaries are synthesized for a requested Arrow key type.
    /// Guarantees: Every row resolves to the scalar or Null and retains the requested key data type.
    #[test]
    fn scalar_and_null_dictionary_constructors_preserve_shape() {
        let scalar =
            Dictionary::new_scalar_with_data_type(DataType::UInt16, 3, ValueOrRef::Integer(42));
        assert_eq!(scalar.len(), 3);
        assert_eq!(scalar.keys().data_type(), DataType::UInt16);
        assert!(!scalar.is_null());
        assert_eq!(scalar.get_value(2), ValueOrRef::Integer(42));
        assert_eq!(scalar.get_value(3), ValueOrRef::Null);

        let null = Dictionary::new_null_with_data_type(3, DataType::Int32);
        assert_eq!(null.len(), 3);
        assert_eq!(null.keys().data_type(), DataType::Int32);
        assert!(null.is_null());
        assert_eq!(null.nulls().unwrap().null_count(), 3);
        assert_eq!(null.get_value(0), ValueOrRef::Null);
    }

    /// Scenario: A primitive Arrow array is represented as a dictionary with one unique key per row.
    /// Guarantees: Row indexes map directly to source values and the first boundary index is rejected.
    #[test]
    fn primitive_array_dictionary_uses_unique_keys() {
        let values = Int32Array::from(vec![5, 8, 13]);
        let dictionary = Dictionary::from_array::<UInt8Type, Int32Type>(&values);

        assert_eq!(dictionary.len(), 3);
        assert_eq!(dictionary.get_value_index(2), Some(2));
        assert_eq!(dictionary.get_value(2), ValueOrRef::Integer(13));
        assert_eq!(dictionary.get_value_index(3), None);
        assert_eq!(dictionary.get_value(3), ValueOrRef::Null);
    }
}
