// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::{array::*, buffer::NullBuffer, datatypes::*};

#[derive(Debug, Clone, PartialEq)]
pub enum DictionaryKeyArray {
    KeyArray(Arc<dyn Array>),
    BooleanArray {
        data_type: DataType,
        values: Arc<dyn Array>,
    },
    UniqueValues {
        data_type: DataType,
        length: usize,
    },
    SingleValue {
        data_type: DataType,
        length: usize,
        value_index: Option<usize>,
    },
}

impl DictionaryKeyArray {
    pub fn len(&self) -> usize {
        match self {
            DictionaryKeyArray::KeyArray(a) => a.len(),
            DictionaryKeyArray::BooleanArray {
                data_type: _,
                values,
            } => values.len(),
            DictionaryKeyArray::UniqueValues {
                data_type: _,
                length,
            } => *length,
            DictionaryKeyArray::SingleValue {
                data_type: _,
                length,
                value_index: _,
            } => *length,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            DictionaryKeyArray::KeyArray(a) => a.is_empty(),
            DictionaryKeyArray::BooleanArray {
                data_type: _,
                values,
            } => values.is_empty(),
            DictionaryKeyArray::UniqueValues {
                data_type: _,
                length,
            } => *length == 0,
            DictionaryKeyArray::SingleValue {
                data_type: _,
                length,
                value_index: _,
            } => *length == 0,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            DictionaryKeyArray::KeyArray(array) => array.null_count() == array.len(),
            DictionaryKeyArray::BooleanArray {
                data_type: _,
                values,
            } => values.null_count() == values.len(),
            DictionaryKeyArray::UniqueValues {
                data_type: _,
                length: _,
            } => false,
            DictionaryKeyArray::SingleValue {
                data_type: _,
                length: _,
                value_index,
            } => value_index.is_none(),
        }
    }

    pub fn nulls(&self) -> Option<NullBuffer> {
        match self {
            DictionaryKeyArray::KeyArray(array) => array.nulls().cloned(),
            DictionaryKeyArray::BooleanArray {
                data_type: _,
                values,
            } => values.nulls().cloned(),
            DictionaryKeyArray::UniqueValues {
                data_type: _,
                length: _,
            } => None,
            DictionaryKeyArray::SingleValue {
                data_type: _,
                length,
                value_index,
            } => {
                if value_index.is_none() {
                    Some(NullBuffer::new_null(*length))
                } else {
                    None
                }
            }
        }
    }

    pub fn data_type(&self) -> DataType {
        match self {
            DictionaryKeyArray::KeyArray(a) => a.data_type().clone(),
            DictionaryKeyArray::BooleanArray {
                data_type,
                values: _,
            } => data_type.clone(),
            DictionaryKeyArray::UniqueValues {
                data_type,
                length: _,
            } => data_type.clone(),
            DictionaryKeyArray::SingleValue {
                data_type,
                length: _,
                value_index: _,
            } => data_type.clone(),
        }
    }

    pub fn get_value_index_for_key_index(&self, index: usize) -> Option<usize> {
        match self {
            DictionaryKeyArray::KeyArray(a) => get_key_array_value_index_for_key_index(a, index),
            DictionaryKeyArray::BooleanArray {
                data_type: _,
                values,
            } => get_bool_array_value_index_for_key_index(values.as_boolean(), index),
            DictionaryKeyArray::UniqueValues {
                data_type: _,
                length,
            } => {
                if index > *length {
                    None
                } else {
                    Some(index)
                }
            }
            DictionaryKeyArray::SingleValue {
                data_type: _,
                length,
                value_index,
            } => {
                if index > *length {
                    None
                } else {
                    *value_index
                }
            }
        }
    }
}

impl<T: ArrowDictionaryKeyType> From<PrimitiveArray<T>> for DictionaryKeyArray {
    fn from(value: PrimitiveArray<T>) -> DictionaryKeyArray {
        DictionaryKeyArray::KeyArray(Arc::new(value))
    }
}

impl From<&dyn Array> for DictionaryKeyArray {
    fn from(value: &dyn Array) -> DictionaryKeyArray {
        DictionaryKeyArray::KeyArray(value.slice(0, value.len()))
    }
}

impl<'a, T: ArrowPrimitiveType> From<&'a PrimitiveArray<T>> for DictionaryKeyArray {
    fn from(value: &'a PrimitiveArray<T>) -> DictionaryKeyArray {
        DictionaryKeyArray::KeyArray((value as &dyn Array).slice(0, value.len()))
    }
}

fn get_key_array_value_index_for_key_index(array: &dyn Array, key_index: usize) -> Option<usize> {
    if key_index > array.len() || array.is_null(key_index) {
        return None;
    }

    unsafe {
        Some(match array.data_type() {
            DataType::Int8 => array
                .as_any()
                .downcast_ref::<PrimitiveArray<Int8Type>>()
                .unwrap()
                .value_unchecked(key_index) as usize,
            DataType::Int16 => array
                .as_any()
                .downcast_ref::<PrimitiveArray<Int16Type>>()
                .unwrap()
                .value_unchecked(key_index) as usize,
            DataType::Int32 => array
                .as_any()
                .downcast_ref::<PrimitiveArray<Int32Type>>()
                .unwrap()
                .value_unchecked(key_index) as usize,
            DataType::Int64 => array
                .as_any()
                .downcast_ref::<PrimitiveArray<Int64Type>>()
                .unwrap()
                .value_unchecked(key_index) as usize,

            DataType::UInt8 => array
                .as_any()
                .downcast_ref::<PrimitiveArray<UInt8Type>>()
                .unwrap()
                .value_unchecked(key_index) as usize,
            DataType::UInt16 => array
                .as_any()
                .downcast_ref::<PrimitiveArray<UInt16Type>>()
                .unwrap()
                .value_unchecked(key_index) as usize,
            DataType::UInt32 => array
                .as_any()
                .downcast_ref::<PrimitiveArray<UInt32Type>>()
                .unwrap()
                .value_unchecked(key_index) as usize,
            DataType::UInt64 => array
                .as_any()
                .downcast_ref::<PrimitiveArray<UInt64Type>>()
                .unwrap()
                .value_unchecked(key_index) as usize,

            d => panic!("Key type '{d}' is not supported"),
        })
    }
}

fn get_bool_array_value_index_for_key_index(
    array: &BooleanArray,
    key_index: usize,
) -> Option<usize> {
    if key_index > array.len() || array.is_null(key_index) {
        return None;
    }
    Some(match unsafe { array.value_unchecked(key_index) } {
        true => 1,
        false => 0,
    })
}
