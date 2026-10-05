// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::{array::*, buffer::NullBuffer, datatypes::*};

#[derive(Debug, Clone, PartialEq)]
pub enum DictionaryKeyArray {
    KeyArray(DictionaryArrowKeyArray),
    BooleanArray {
        data_type: DataType,
        values: BooleanArray,
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
            DictionaryKeyArray::KeyArray(array) => array.nulls(),
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
            DictionaryKeyArray::KeyArray(a) => a.data_type(),
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
            DictionaryKeyArray::KeyArray(a) => a.get_value_index_for_key_index(index),
            DictionaryKeyArray::BooleanArray {
                data_type: _,
                values,
            } => get_bool_array_value_index_for_key_index(values, index),
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

#[derive(Debug, Clone, PartialEq)]
pub enum DictionaryArrowKeyArray {
    Int8(PrimitiveArray<Int8Type>),
    Int16(PrimitiveArray<Int16Type>),
    Int32(PrimitiveArray<Int32Type>),
    Int64(PrimitiveArray<Int64Type>),

    UInt8(PrimitiveArray<UInt8Type>),
    UInt16(PrimitiveArray<UInt16Type>),
    UInt32(PrimitiveArray<UInt32Type>),
    UInt64(PrimitiveArray<UInt64Type>),
}

impl DictionaryArrowKeyArray {
    pub fn as_array(&self) -> &dyn Array {
        match self {
            DictionaryArrowKeyArray::Int8(a) => a,
            DictionaryArrowKeyArray::Int16(a) => a,
            DictionaryArrowKeyArray::Int32(a) => a,
            DictionaryArrowKeyArray::Int64(a) => a,
            DictionaryArrowKeyArray::UInt8(a) => a,
            DictionaryArrowKeyArray::UInt16(a) => a,
            DictionaryArrowKeyArray::UInt32(a) => a,
            DictionaryArrowKeyArray::UInt64(a) => a,
        }
    }

    pub fn len(&self) -> usize {
        self.as_array().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_array().is_empty()
    }

    pub fn is_null(&self, key_index: usize) -> bool {
        self.as_array().is_null(key_index)
    }

    pub fn null_count(&self) -> usize {
        self.as_array().null_count()
    }

    pub fn nulls(&self) -> Option<NullBuffer> {
        self.as_array().nulls().cloned()
    }

    pub fn data_type(&self) -> DataType {
        match self {
            DictionaryArrowKeyArray::Int8(_) => DataType::Int8,
            DictionaryArrowKeyArray::Int16(_) => DataType::Int16,
            DictionaryArrowKeyArray::Int32(_) => DataType::Int32,
            DictionaryArrowKeyArray::Int64(_) => DataType::Int64,
            DictionaryArrowKeyArray::UInt8(_) => DataType::UInt8,
            DictionaryArrowKeyArray::UInt16(_) => DataType::UInt16,
            DictionaryArrowKeyArray::UInt32(_) => DataType::UInt32,
            DictionaryArrowKeyArray::UInt64(_) => DataType::UInt64,
        }
    }

    fn get_value_index_for_key_index(&self, key_index: usize) -> Option<usize> {
        if key_index >= self.len() || self.is_null(key_index) {
            return None;
        }

        unsafe {
            Some(match self {
                DictionaryArrowKeyArray::Int8(array) => array.value_unchecked(key_index) as usize,
                DictionaryArrowKeyArray::Int16(array) => array.value_unchecked(key_index) as usize,
                DictionaryArrowKeyArray::Int32(array) => array.value_unchecked(key_index) as usize,
                DictionaryArrowKeyArray::Int64(array) => array.value_unchecked(key_index) as usize,

                DictionaryArrowKeyArray::UInt8(array) => array.value_unchecked(key_index) as usize,
                DictionaryArrowKeyArray::UInt16(array) => array.value_unchecked(key_index) as usize,
                DictionaryArrowKeyArray::UInt32(array) => array.value_unchecked(key_index) as usize,
                DictionaryArrowKeyArray::UInt64(array) => array.value_unchecked(key_index) as usize,
            })
        }
    }
}

macro_rules! impl_from_key_array {
    ($arrow_ty:ty, $variant:ident) => {
        impl From<PrimitiveArray<$arrow_ty>> for DictionaryKeyArray {
            fn from(value: PrimitiveArray<$arrow_ty>) -> DictionaryKeyArray {
                DictionaryKeyArray::KeyArray(DictionaryArrowKeyArray::$variant(value))
            }
        }

        impl<'a> From<&'a PrimitiveArray<$arrow_ty>> for DictionaryKeyArray {
            fn from(value: &'a PrimitiveArray<$arrow_ty>) -> DictionaryKeyArray {
                DictionaryKeyArray::KeyArray(DictionaryArrowKeyArray::$variant(value.clone()))
            }
        }
    };
}

impl_from_key_array!(Int8Type, Int8);
impl_from_key_array!(Int16Type, Int16);
impl_from_key_array!(Int32Type, Int32);
impl_from_key_array!(Int64Type, Int64);
impl_from_key_array!(UInt8Type, UInt8);
impl_from_key_array!(UInt16Type, UInt16);
impl_from_key_array!(UInt32Type, UInt32);
impl_from_key_array!(UInt64Type, UInt64);

fn get_bool_array_value_index_for_key_index(
    array: &BooleanArray,
    key_index: usize,
) -> Option<usize> {
    (key_index < array.len() && !array.is_null(key_index))
        .then(|| unsafe { array.value_unchecked(key_index) })
        .map(|b| b as usize)
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
            values: BooleanArray::from(vec![Some(false), Some(true), None]),
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
