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
                if index >= *length {
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
                if index >= *length {
                    None
                } else {
                    *value_index
                }
            }
        }
    }

    pub(crate) fn has_value_index_out_of_bounds(&self, value_count: usize) -> bool {
        match self {
            DictionaryKeyArray::KeyArray(_) | DictionaryKeyArray::BooleanArray { .. } => {
                (0..self.len()).any(|key_index| {
                    self.get_value_index_for_key_index(key_index)
                        .is_some_and(|value_index| value_index >= value_count)
                })
            }
            DictionaryKeyArray::UniqueValues { length, .. } => *length > value_count,
            DictionaryKeyArray::SingleValue {
                length,
                value_index,
                ..
            } => *length > 0 && value_index.is_some_and(|value_index| value_index >= value_count),
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
    if key_index >= array.len() || array.is_null(key_index) {
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
    if key_index >= array.len() || array.is_null(key_index) {
        return None;
    }
    Some(match unsafe { array.value_unchecked(key_index) } {
        true => 1,
        false => 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Primitive and Boolean key arrays contain valid keys, null keys, and boundary indexes.
    /// Guarantees: Key lookup maps valid values, preserves nulls, and safely rejects the first out-of-range index.
    #[test]
    fn key_arrays_map_values_and_enforce_bounds() {
        let primitive = DictionaryKeyArray::from(Int16Array::from(vec![Some(2), None, Some(0)]));
        assert_eq!(primitive.get_value_index_for_key_index(0), Some(2));
        assert_eq!(primitive.get_value_index_for_key_index(1), None);
        assert_eq!(primitive.get_value_index_for_key_index(3), None);

        let boolean = DictionaryKeyArray::BooleanArray {
            data_type: DataType::Int8,
            values: Arc::new(BooleanArray::from(vec![Some(false), Some(true), None])),
        };
        assert_eq!(boolean.get_value_index_for_key_index(0), Some(0));
        assert_eq!(boolean.get_value_index_for_key_index(1), Some(1));
        assert_eq!(boolean.get_value_index_for_key_index(2), None);
        assert_eq!(boolean.get_value_index_for_key_index(3), None);
    }

    /// Scenario: Synthetic unique, scalar, and null key layouts represent dictionary columns without Arrow buffers.
    /// Guarantees: Metadata, validity, and boundary lookup are correct for every synthetic layout.
    #[test]
    fn synthetic_key_arrays_report_metadata_and_validity() {
        let unique = DictionaryKeyArray::UniqueValues {
            data_type: DataType::UInt16,
            length: 2,
        };
        assert_eq!(unique.len(), 2);
        assert!(!unique.is_empty());
        assert!(!unique.is_null());
        assert!(unique.nulls().is_none());
        assert_eq!(unique.data_type(), DataType::UInt16);
        assert_eq!(unique.get_value_index_for_key_index(1), Some(1));
        assert_eq!(unique.get_value_index_for_key_index(2), None);

        let null = DictionaryKeyArray::SingleValue {
            data_type: DataType::Int32,
            length: 2,
            value_index: None,
        };
        assert!(null.is_null());
        assert_eq!(null.nulls().unwrap().null_count(), 2);
        assert_eq!(null.get_value_index_for_key_index(2), None);
    }

    /// Scenario: Synthetic unique and scalar key layouts are checked against shorter and equal-length value tables.
    /// Guarantees: Their out-of-bounds fast paths detect invalid references without scanning individual rows.
    #[test]
    fn synthetic_key_arrays_detect_out_of_bounds_value_indices() {
        let unique = DictionaryKeyArray::UniqueValues {
            data_type: DataType::UInt16,
            length: 2,
        };
        assert!(!unique.has_value_index_out_of_bounds(2));
        assert!(unique.has_value_index_out_of_bounds(1));

        let scalar = DictionaryKeyArray::SingleValue {
            data_type: DataType::Int32,
            length: 3,
            value_index: Some(1),
        };
        assert!(!scalar.has_value_index_out_of_bounds(2));
        assert!(scalar.has_value_index_out_of_bounds(1));

        let empty = DictionaryKeyArray::SingleValue {
            data_type: DataType::Int32,
            length: 0,
            value_index: Some(1),
        };
        assert!(!empty.has_value_index_out_of_bounds(0));
    }
}
