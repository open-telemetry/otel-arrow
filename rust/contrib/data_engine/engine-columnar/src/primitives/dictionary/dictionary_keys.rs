// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use arrow::{
    array::*,
    buffer::{MutableBuffer, NullBuffer},
    datatypes::*,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DictionaryKeyArray {
    KeyArray(DictionaryArrowKeyArray),
    BooleanArray {
        data_type: DictionaryKeyType,
        values: BooleanArray,
    },
    UniqueValues {
        data_type: DictionaryKeyType,
        length: usize,
    },
    SingleValue {
        data_type: DictionaryKeyType,
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

    pub fn data_type(&self) -> DictionaryKeyType {
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

    pub fn data_type(&self) -> DictionaryKeyType {
        match self {
            DictionaryArrowKeyArray::Int8(_) => DictionaryKeyType::Int8,
            DictionaryArrowKeyArray::Int16(_) => DictionaryKeyType::Int16,
            DictionaryArrowKeyArray::Int32(_) => DictionaryKeyType::Int32,
            DictionaryArrowKeyArray::Int64(_) => DictionaryKeyType::Int64,
            DictionaryArrowKeyArray::UInt8(_) => DictionaryKeyType::UInt8,
            DictionaryArrowKeyArray::UInt16(_) => DictionaryKeyType::UInt16,
            DictionaryArrowKeyArray::UInt32(_) => DictionaryKeyType::UInt32,
            DictionaryArrowKeyArray::UInt64(_) => DictionaryKeyType::UInt64,
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

    pub fn iter(&self) -> DictionaryArrowKeyArrayIter<'_> {
        match self {
            DictionaryArrowKeyArray::Int8(a) => DictionaryArrowKeyArrayIter::Int8(a.iter()),
            DictionaryArrowKeyArray::Int16(a) => DictionaryArrowKeyArrayIter::Int16(a.iter()),
            DictionaryArrowKeyArray::Int32(a) => DictionaryArrowKeyArrayIter::Int32(a.iter()),
            DictionaryArrowKeyArray::Int64(a) => DictionaryArrowKeyArrayIter::Int64(a.iter()),
            DictionaryArrowKeyArray::UInt8(a) => DictionaryArrowKeyArrayIter::UInt8(a.iter()),
            DictionaryArrowKeyArray::UInt16(a) => DictionaryArrowKeyArrayIter::UInt16(a.iter()),
            DictionaryArrowKeyArray::UInt32(a) => DictionaryArrowKeyArrayIter::UInt32(a.iter()),
            DictionaryArrowKeyArray::UInt64(a) => DictionaryArrowKeyArrayIter::UInt64(a.iter()),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictionaryKeyType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
}

pub enum DictionaryArrowKeyArrayIter<'a> {
    Int8(PrimitiveIter<'a, Int8Type>),
    Int16(PrimitiveIter<'a, Int16Type>),
    Int32(PrimitiveIter<'a, Int32Type>),
    Int64(PrimitiveIter<'a, Int64Type>),

    UInt8(PrimitiveIter<'a, UInt8Type>),
    UInt16(PrimitiveIter<'a, UInt16Type>),
    UInt32(PrimitiveIter<'a, UInt32Type>),
    UInt64(PrimitiveIter<'a, UInt64Type>),
}

impl<'a> Iterator for DictionaryArrowKeyArrayIter<'a> {
    type Item = Option<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DictionaryArrowKeyArrayIter::Int8(i) => i.next().map(|v| v.map(|v| v as usize)),
            DictionaryArrowKeyArrayIter::Int16(i) => i.next().map(|v| v.map(|v| v as usize)),
            DictionaryArrowKeyArrayIter::Int32(i) => i.next().map(|v| v.map(|v| v as usize)),
            DictionaryArrowKeyArrayIter::Int64(i) => i.next().map(|v| v.map(|v| v as usize)),
            DictionaryArrowKeyArrayIter::UInt8(i) => i.next().map(|v| v.map(|v| v as usize)),
            DictionaryArrowKeyArrayIter::UInt16(i) => i.next().map(|v| v.map(|v| v as usize)),
            DictionaryArrowKeyArrayIter::UInt32(i) => i.next().map(|v| v.map(|v| v as usize)),
            DictionaryArrowKeyArrayIter::UInt64(i) => i.next().map(|v| v.map(|v| v as usize)),
        }
    }
}

fn get_bool_array_value_index_for_key_index(
    array: &BooleanArray,
    key_index: usize,
) -> Option<usize> {
    (key_index < array.len() && !array.is_null(key_index))
        .then(|| unsafe { array.value_unchecked(key_index) })
        .map(|b| b as usize)
}

pub(crate) struct DictionaryKeyArrayBuilder<K: ArrowDictionaryKeyType> {
    key_length: usize,
    key_bit_length: usize,
    key_buffer: MutableBuffer,
    null_buffer: Option<MutableBuffer>,
    marker: PhantomData<K>,
}

impl<K: ArrowDictionaryKeyType> DictionaryKeyArrayBuilder<K> {
    pub fn new(key_length: usize) -> DictionaryKeyArrayBuilder<K> {
        let key_bit_length = arrow::util::bit_util::ceil(key_length, 8);

        Self {
            key_length,
            key_bit_length,
            key_buffer: MutableBuffer::from_len_zeroed(size_of::<K::Native>() * key_length),
            null_buffer: None,
            marker: Default::default(),
        }
    }

    pub fn new_null(key_length: usize) -> DictionaryKeyArrayBuilder<K> {
        let key_bit_length = arrow::util::bit_util::ceil(key_length, 8);

        Self {
            key_length,
            key_bit_length,
            key_buffer: MutableBuffer::from_len_zeroed(size_of::<K::Native>() * key_length),
            null_buffer: Some(MutableBuffer::from_len_zeroed(key_bit_length)),
            marker: Default::default(),
        }
    }

    pub fn get_key_length(&self) -> usize {
        self.key_length
    }

    pub fn get_writer(&mut self) -> DictionaryKeyArrayWriter<'_, K> {
        DictionaryKeyArrayWriter {
            key_bit_length: self.key_bit_length,
            key_builder: self.key_buffer.typed_data_mut::<K::Native>().as_mut_ptr(),
            null_buffer: &mut self.null_buffer,
        }
    }

    pub fn has_nulls(&self) -> bool {
        self.null_buffer.is_some()
    }

    pub fn finish(self) -> PrimitiveArray<K> {
        PrimitiveArray::<K>::new(
            self.key_buffer.into(),
            self.null_buffer
                .and_then(|v| NullBufferBuilder::new_from_buffer(v, self.key_length).build()),
        )
    }
}

pub(crate) struct DictionaryKeyArrayWriter<'a, K: ArrowDictionaryKeyType> {
    key_bit_length: usize,
    key_builder: *mut K::Native,
    null_buffer: &'a mut Option<MutableBuffer>,
}

impl<'a, K: ArrowDictionaryKeyType> DictionaryKeyArrayWriter<'a, K> {
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*.
    pub unsafe fn set_value_index_unchecked(&mut self, key_index: usize, value_index: usize) {
        unsafe {
            *self.key_builder.add(key_index) =
                <K as ArrowPrimitiveType>::Native::from_usize(value_index)
                    .expect("transformed value index converted to output size")
        }
    }

    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*.
    pub unsafe fn set_value_index_typed_unchecked(
        &mut self,
        key_index: usize,
        value_index: K::Native,
    ) {
        unsafe { *self.key_builder.add(key_index) = value_index }
    }

    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*.
    pub unsafe fn set_null_unchecked(&mut self, key_index: usize) {
        unsafe {
            ensure_null_buffer_and_push_null(self.null_buffer, key_index, self.key_bit_length)
        }
    }

    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is *[undefined behavior]*.
    pub unsafe fn set_nonnull_unchecked(&mut self, key_index: usize) {
        if let Some(nulls) = self.null_buffer {
            let ptr = nulls.typed_data_mut::<u8>().as_mut_ptr();

            let i = key_index / 8;
            let b = 1 << (key_index % 8);

            unsafe { *ptr.add(i) |= b };
        }
    }
}

pub(crate) unsafe fn ensure_null_buffer_and_push_null(
    null_buffer: &mut Option<MutableBuffer>,
    index: usize,
    key_bit_length: usize,
) {
    if let Some(buffer) = null_buffer {
        let ptr = buffer.typed_data_mut::<u8>().as_mut_ptr();

        let i = index / 8;
        let b = 1 << (index % 8);

        unsafe { *ptr.add(i) &= !b };
    } else {
        let mut buffer = MutableBuffer::new(key_bit_length).with_bitset(key_bit_length, true);

        let ptr = buffer.typed_data_mut::<u8>().as_mut_ptr();

        let i = index / 8;
        let b = 1 << (index % 8);

        unsafe { *ptr.add(i) &= !b };

        *null_buffer = Some(buffer);
    }
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
            data_type: DictionaryKeyType::Int8,
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
            data_type: DictionaryKeyType::UInt16,
            length: 2,
        };
        assert_eq!(unique.len(), 2);
        assert!(!unique.is_empty());
        assert!(!unique.is_null());
        assert!(unique.nulls().is_none());
        assert_eq!(unique.data_type(), DictionaryKeyType::UInt16);
        assert_eq!(unique.get_value_index_for_key_index(1), Some(1));
        assert_eq!(unique.get_value_index_for_key_index(2), None);

        let null = DictionaryKeyArray::SingleValue {
            data_type: DictionaryKeyType::Int32,
            length: 2,
            value_index: None,
        };
        assert!(null.is_null());
        assert_eq!(null.nulls().unwrap().null_count(), 2);
        assert_eq!(null.get_value_index_for_key_index(2), None);
    }
}
