// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{cell::OnceCell, rc::Rc, sync::Arc};

use ahash::{AHashMap, RandomState};
use arrow::{array::*, buffer::*, datatypes::*};
use chrono::{TimeZone, Utc};
use indexmap::IndexSet;

use crate::*;

pub(crate) type ValueOrRefSet<'a> = GenericSet<ValueOrRef<'a>>;
pub type GenericSet<T> = IndexSet<T, RandomState>;
pub type IndexLookup = Option<AHashMap<usize, Option<usize>>>;

#[derive(Debug, Clone)]
pub enum DictionaryValueArray<'a> {
    Array(Arc<dyn Array>),
    Vec(Rc<Vec<ValueOrRef<'a>>>),
    Set(Rc<ValueOrRefSet<'a>>),
    Boolean,
}

impl DictionaryValueArray<'_> {
    pub fn len(&self) -> usize {
        match self {
            DictionaryValueArray::Array(a) => a.len(),
            DictionaryValueArray::Vec(a) => a.len(),
            DictionaryValueArray::Set(a) => a.len(),
            DictionaryValueArray::Boolean => 2,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            DictionaryValueArray::Array(a) => a.is_empty(),
            DictionaryValueArray::Vec(a) => a.is_empty(),
            DictionaryValueArray::Set(a) => a.is_empty(),
            DictionaryValueArray::Boolean => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            DictionaryValueArray::Array(a) => a.null_count() == a.len(),
            DictionaryValueArray::Vec(a) => a.iter().all(|v| matches!(v, ValueOrRef::Null)),
            DictionaryValueArray::Set(a) => a.iter().all(|v| matches!(v, ValueOrRef::Null)),
            DictionaryValueArray::Boolean => false,
        }
    }

    pub fn nulls(&self) -> Option<NullBuffer> {
        match self {
            DictionaryValueArray::Array(a) => a.nulls().cloned(),
            DictionaryValueArray::Vec(a) => {
                let mut buffer = OnceCell::new();

                for (index, value) in a.iter().enumerate() {
                    if matches!(value, ValueOrRef::Null) {
                        buffer.get_or_init(|| {
                            let l = a.len();
                            BooleanBufferBuilder::new_from_buffer(
                                MutableBuffer::new_null(a.len()),
                                l,
                            )
                        });
                        let buffer = buffer.get_mut().expect("has buffer");
                        buffer.set_bit(index, true);
                    }
                }

                buffer.take().map(|mut b| {
                    for byte in b.as_slice_mut() {
                        *byte = !*byte;
                    }

                    NullBuffer::new(b.build())
                })
            }
            DictionaryValueArray::Set(a) => {
                let mut buffer = OnceCell::new();

                for (index, value) in a.iter().enumerate() {
                    if matches!(value, ValueOrRef::Null) {
                        buffer.get_or_init(|| {
                            let l = a.len();
                            BooleanBufferBuilder::new_from_buffer(
                                MutableBuffer::new_null(a.len()),
                                l,
                            )
                        });
                        let buffer = buffer.get_mut().expect("has buffer");
                        buffer.set_bit(index, true);
                    }
                }

                buffer.take().map(|mut b| {
                    for byte in b.as_slice_mut() {
                        *byte = !*byte;
                    }

                    NullBuffer::new(b.build())
                })
            }
            DictionaryValueArray::Boolean => None,
        }
    }
}

impl<'a> DictionaryValueArray<'a> {
    pub fn get_value_at(&self, index: usize) -> ValueOrRef<'a> {
        match self {
            DictionaryValueArray::Array(a) => get_value_from_array(a, index),
            DictionaryValueArray::Vec(a) => a.get(index).cloned().unwrap_or(ValueOrRef::Null),
            DictionaryValueArray::Set(a) => a.get_index(index).cloned().unwrap_or(ValueOrRef::Null),
            DictionaryValueArray::Boolean => ValueOrRef::Boolean(index != 0),
        }
    }
}

impl PartialEq for DictionaryValueArray<'_> {
    fn eq(&self, other: &Self) -> bool {
        let length = self.len();

        if length != other.len() {
            return false;
        }

        for index in 0..length {
            if self.get_value_at(index) != other.get_value_at(index) {
                return false;
            }
        }

        true
    }
}

impl<'a, T: Array + 'a> From<&T> for DictionaryValueArray<'a> {
    fn from(value: &T) -> DictionaryValueArray<'a> {
        DictionaryValueArray::Array((value as &dyn Array).slice(0, value.len()))
    }
}

impl<'a> From<&dyn Array> for DictionaryValueArray<'a> {
    fn from(value: &dyn Array) -> DictionaryValueArray<'a> {
        DictionaryValueArray::Array(value.slice(0, value.len()))
    }
}

impl<'a> From<ValueOrRefSet<'a>> for DictionaryValueArray<'a> {
    fn from(value: ValueOrRefSet<'a>) -> DictionaryValueArray<'a> {
        DictionaryValueArray::Set(value.into())
    }
}

impl<'a> From<Vec<ValueOrRef<'a>>> for DictionaryValueArray<'a> {
    fn from(value: Vec<ValueOrRef<'a>>) -> DictionaryValueArray<'a> {
        DictionaryValueArray::Vec(value.into())
    }
}

fn get_value_from_array(value: &Arc<dyn Array>, index: usize) -> ValueOrRef<'static> {
    if index > value.len() || value.nulls().map(|n| n.is_null(index)).unwrap_or(false) {
        return ValueOrRef::Null;
    }

    unsafe {
        match value.data_type() {
            DataType::Int8 => ValueOrRef::Integer(
                *value
                    .as_primitive::<Int8Type>()
                    .values()
                    .get_unchecked(index) as i64,
            ),
            DataType::Int16 => ValueOrRef::Integer(
                *value
                    .as_primitive::<Int16Type>()
                    .values()
                    .get_unchecked(index) as i64,
            ),
            DataType::Int32 => ValueOrRef::Integer(
                *value
                    .as_primitive::<Int32Type>()
                    .values()
                    .get_unchecked(index) as i64,
            ),
            DataType::Int64 => ValueOrRef::Integer(
                *value
                    .as_primitive::<Int64Type>()
                    .values()
                    .get_unchecked(index),
            ),

            DataType::UInt8 => ValueOrRef::Integer(
                *value
                    .as_primitive::<UInt8Type>()
                    .values()
                    .get_unchecked(index) as i64,
            ),
            DataType::UInt16 => ValueOrRef::Integer(
                *value
                    .as_primitive::<UInt16Type>()
                    .values()
                    .get_unchecked(index) as i64,
            ),
            DataType::UInt32 => ValueOrRef::Integer(
                *value
                    .as_primitive::<UInt32Type>()
                    .values()
                    .get_unchecked(index) as i64,
            ),
            DataType::UInt64 => ValueOrRef::Integer(
                *value
                    .as_primitive::<UInt64Type>()
                    .values()
                    .get_unchecked(index) as i64,
            ),

            DataType::Float16 => ValueOrRef::Double(
                (*value
                    .as_primitive::<Float16Type>()
                    .values()
                    .get_unchecked(index))
                .into(),
            ),
            DataType::Float32 => ValueOrRef::Double(
                *value
                    .as_primitive::<Float32Type>()
                    .values()
                    .get_unchecked(index) as f64,
            ),
            DataType::Float64 => ValueOrRef::Double(
                *value
                    .as_primitive::<Float64Type>()
                    .values()
                    .get_unchecked(index),
            ),

            DataType::Utf8 => ValueOrRef::String(StringValueOrRef::new_utf8_unvalidated({
                let strings = value.as_string::<i32>();
                let offsets = strings.value_offsets();
                let end = *offsets.get_unchecked(index + 1) as usize;
                let start = *offsets.get_unchecked(index) as usize;
                strings.values().slice_with_length(start, end - start)
            })),
            DataType::LargeUtf8 => ValueOrRef::String(StringValueOrRef::new_utf8_unvalidated({
                let strings = value.as_string::<i64>();
                let offsets = strings.value_offsets();
                let end = *offsets.get_unchecked(index + 1) as usize;
                let start = *offsets.get_unchecked(index) as usize;
                strings.values().slice_with_length(start, end - start)
            })),

            DataType::Timestamp(time_unit, _) => ValueOrRef::DateTime(match time_unit {
                TimeUnit::Second => {
                    let secs = *value
                        .as_primitive::<TimestampSecondType>()
                        .values()
                        .get_unchecked(index);
                    Utc.timestamp_opt(secs, 0).unwrap().into()
                }
                TimeUnit::Millisecond => {
                    let millis = *value
                        .as_primitive::<TimestampMillisecondType>()
                        .values()
                        .get_unchecked(index);
                    Utc.timestamp_millis_opt(millis).unwrap().into()
                }
                TimeUnit::Microsecond => {
                    let micros = *value
                        .as_primitive::<TimestampMicrosecondType>()
                        .values()
                        .get_unchecked(index);
                    Utc.timestamp_micros(micros).unwrap().into()
                }
                TimeUnit::Nanosecond => {
                    let nanos = *value
                        .as_primitive::<TimestampNanosecondType>()
                        .values()
                        .get_unchecked(index);
                    Utc.timestamp_nanos(nanos).into()
                }
            }),

            DataType::FixedSizeBinary(_) => ValueOrRef::Array(ArrayValueOrRef::Buffer({
                let bytes = value.as_fixed_size_binary();
                let start = bytes.value_offset(index) as usize;
                let buffer = bytes
                    .values()
                    .slice_with_length(start, bytes.value_length() as usize)
                    .clone();
                BufferArray::new_u8(buffer)
            })),

            d => todo!("{d} is not implemented"),
        }
    }
}
