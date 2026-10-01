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
