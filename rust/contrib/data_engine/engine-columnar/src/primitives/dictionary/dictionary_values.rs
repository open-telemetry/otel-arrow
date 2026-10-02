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
            DictionaryValueArray::Boolean => match index {
                0 => ValueOrRef::Boolean(false),
                1 => ValueOrRef::Boolean(true),
                _ => ValueOrRef::Null,
            },
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
    if index >= value.len() || value.nulls().map(|n| n.is_null(index)).unwrap_or(false) {
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
            DataType::UInt64 => match TryInto::<i64>::try_into(
                *value
                    .as_primitive::<UInt64Type>()
                    .values()
                    .get_unchecked(index),
            ) {
                Ok(v) => ValueOrRef::Integer(v),
                Err(_) => ValueOrRef::Null,
            },
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

#[cfg(test)]
mod tests {
    use arrow::array::{
        FixedSizeBinaryArray, Int32Array, StringArray, TimestampMillisecondArray, UInt64Array,
    };

    use super::*;

    /// Scenario: Arrow numeric and string arrays contain ordinary values, nulls, and boundary indexes.
    /// Guarantees: Values are converted to engine primitives while null and out-of-range access returns Null.
    #[test]
    fn arrow_values_convert_numbers_strings_and_nulls() {
        let integers = DictionaryValueArray::from(&Int32Array::from(vec![Some(-7), None, Some(9)]));
        assert_eq!(integers.get_value_at(0), ValueOrRef::Integer(-7));
        assert_eq!(integers.get_value_at(1), ValueOrRef::Null);
        assert_eq!(integers.get_value_at(3), ValueOrRef::Null);

        let strings = DictionaryValueArray::from(&StringArray::from(vec![
            Some("alpha"),
            None,
            Some("omega"),
        ]));
        assert_eq!(
            strings.get_value_at(0),
            ValueOrRef::String(StringValueOrRef::new_ref("alpha"))
        );
        assert_eq!(strings.get_value_at(1), ValueOrRef::Null);
        assert_eq!(strings.get_value_at(3), ValueOrRef::Null);
    }

    /// Scenario: Arrow UInt64 values straddle the largest value representable by the engine's i64 integer type.
    /// Guarantees: Representable values are preserved while larger values, including u64::MAX, convert to Null without wrapping.
    #[test]
    fn uint64_values_outside_i64_range_convert_to_null() {
        let values = DictionaryValueArray::from(&UInt64Array::from(vec![
            Some(0),
            Some(i64::MAX as u64),
            Some(i64::MAX as u64 + 1),
            Some(u64::MAX),
            None,
        ]));

        assert_eq!(values.get_value_at(0), ValueOrRef::Integer(0));
        assert_eq!(values.get_value_at(1), ValueOrRef::Integer(i64::MAX));
        assert_eq!(values.get_value_at(2), ValueOrRef::Null);
        assert_eq!(values.get_value_at(3), ValueOrRef::Null);
        assert_eq!(values.get_value_at(4), ValueOrRef::Null);
    }

    /// Scenario: Arrow timestamp and fixed-size binary arrays are used as dictionary values.
    /// Guarantees: Time units and binary byte sequences are preserved by conversion.
    #[test]
    fn arrow_values_convert_timestamps_and_fixed_binary() {
        let timestamps =
            DictionaryValueArray::from(&TimestampMillisecondArray::from(vec![1_500_i64]));
        let timestamp = timestamps.get_value_at(0);
        match timestamp {
            ValueOrRef::DateTime(value) => assert_eq!(value.timestamp_millis(), 1_500),
            value => panic!("expected datetime, got {value:?}"),
        }

        let binary =
            FixedSizeBinaryArray::try_from_iter([b"abc".as_slice(), b"def".as_slice()].into_iter())
                .unwrap();
        let values = DictionaryValueArray::from(&binary);
        assert_eq!(
            values.get_value_at(1),
            ValueOrRef::Array(ArrayValueOrRef::from([
                ValueOrRef::Integer(b'd' as i64),
                ValueOrRef::Integer(b'e' as i64),
                ValueOrRef::Integer(b'f' as i64),
            ]))
        );
    }

    /// Scenario: Engine-owned and Boolean dictionary values are queried for validity and bounds.
    /// Guarantees: Validity marks only Null entries and Boolean lookup rejects indexes outside its two-value domain.
    #[test]
    fn owned_and_boolean_values_report_nulls_and_bounds() {
        let values = DictionaryValueArray::from(vec![
            ValueOrRef::Integer(1),
            ValueOrRef::Null,
            ValueOrRef::Integer(3),
        ]);
        let nulls = values.nulls().unwrap();
        assert_eq!(nulls.null_count(), 1);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_null(1));
        assert!(nulls.is_valid(2));

        let boolean = DictionaryValueArray::Boolean;
        assert_eq!(boolean.get_value_at(0), ValueOrRef::Boolean(false));
        assert_eq!(boolean.get_value_at(1), ValueOrRef::Boolean(true));
        assert_eq!(boolean.get_value_at(2), ValueOrRef::Null);
    }
}
