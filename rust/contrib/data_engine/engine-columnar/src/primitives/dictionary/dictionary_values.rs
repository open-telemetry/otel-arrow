// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{cell::OnceCell, rc::Rc, sync::Arc};

use ahash::{AHashMap, RandomState};
use arrow::{array::*, buffer::*, datatypes::*};
use chrono::{TimeZone, Utc};
use indexmap::IndexSet;
use thiserror::Error;

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
    pub fn get_value_at(&self, index: usize) -> Result<ValueOrRef<'a>, ValueError> {
        match self {
            DictionaryValueArray::Array(a) => get_value_from_array(a, index),
            DictionaryValueArray::Vec(a) => Ok(a.get(index).cloned().unwrap_or(ValueOrRef::Null)),
            DictionaryValueArray::Set(a) => {
                Ok(a.get_index(index).cloned().unwrap_or(ValueOrRef::Null))
            }
            DictionaryValueArray::Boolean => match index {
                0 => Ok(ValueOrRef::Boolean(false)),
                1 => Ok(ValueOrRef::Boolean(true)),
                v => Err(ValueError::InvalidBoolean { index_value: v }),
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
            let left = self.get_value_at(index);
            let right = other.get_value_at(index);
            if !match (left, right) {
                (Ok(left), Ok(right)) => left == right,
                (Err(left), Err(right)) => left == right,
                _ => false,
            } {
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

#[derive(Error, Debug, PartialEq)]
pub enum ValueError {
    #[error("UInt64 value '{original_value}' could not be converted into 'Integer'")]
    IntegerConversionFailure { original_value: u64 },
    #[error(
        "Timestamp value '{original_value}' with unit '{time_unit:?}' could not be converted into 'DateTime'"
    )]
    TimestampConversionFailure {
        original_value: i64,
        time_unit: TimeUnit,
    },
    #[error("Index value '{index_value}' could not be converted into 'Boolean'")]
    InvalidBoolean { index_value: usize },
}

fn get_value_from_array(
    value: &Arc<dyn Array>,
    index: usize,
) -> Result<ValueOrRef<'static>, ValueError> {
    if index >= value.len() || value.nulls().map(|n| n.is_null(index)).unwrap_or(false) {
        return Ok(ValueOrRef::Null);
    }

    unsafe {
        match value.data_type() {
            DataType::Int8 => Ok(ValueOrRef::Integer(
                *value
                    .as_primitive::<Int8Type>()
                    .values()
                    .get_unchecked(index) as i64,
            )),
            DataType::Int16 => Ok(ValueOrRef::Integer(
                *value
                    .as_primitive::<Int16Type>()
                    .values()
                    .get_unchecked(index) as i64,
            )),
            DataType::Int32 => Ok(ValueOrRef::Integer(
                *value
                    .as_primitive::<Int32Type>()
                    .values()
                    .get_unchecked(index) as i64,
            )),
            DataType::Int64 => Ok(ValueOrRef::Integer(
                *value
                    .as_primitive::<Int64Type>()
                    .values()
                    .get_unchecked(index),
            )),

            DataType::UInt8 => Ok(ValueOrRef::Integer(
                *value
                    .as_primitive::<UInt8Type>()
                    .values()
                    .get_unchecked(index) as i64,
            )),
            DataType::UInt16 => Ok(ValueOrRef::Integer(
                *value
                    .as_primitive::<UInt16Type>()
                    .values()
                    .get_unchecked(index) as i64,
            )),
            DataType::UInt32 => Ok(ValueOrRef::Integer(
                *value
                    .as_primitive::<UInt32Type>()
                    .values()
                    .get_unchecked(index) as i64,
            )),
            DataType::UInt64 => {
                let value = *value
                    .as_primitive::<UInt64Type>()
                    .values()
                    .get_unchecked(index);

                match TryInto::<i64>::try_into(value) {
                    Ok(v) => Ok(ValueOrRef::Integer(v)),
                    Err(_) => Err(ValueError::IntegerConversionFailure {
                        original_value: value,
                    }),
                }
            }
            DataType::Float16 => Ok(ValueOrRef::Double(
                (*value
                    .as_primitive::<Float16Type>()
                    .values()
                    .get_unchecked(index))
                .into(),
            )),
            DataType::Float32 => Ok(ValueOrRef::Double(
                *value
                    .as_primitive::<Float32Type>()
                    .values()
                    .get_unchecked(index) as f64,
            )),
            DataType::Float64 => Ok(ValueOrRef::Double(
                *value
                    .as_primitive::<Float64Type>()
                    .values()
                    .get_unchecked(index),
            )),

            DataType::Utf8 => Ok(ValueOrRef::String(StringValueOrRef::new_utf8_unvalidated(
                {
                    let strings = value.as_string::<i32>();
                    let offsets = strings.value_offsets();
                    let end = *offsets.get_unchecked(index + 1) as usize;
                    let start = *offsets.get_unchecked(index) as usize;
                    strings.values().slice_with_length(start, end - start)
                },
            ))),
            DataType::LargeUtf8 => Ok(ValueOrRef::String(StringValueOrRef::new_utf8_unvalidated(
                {
                    let strings = value.as_string::<i64>();
                    let offsets = strings.value_offsets();
                    let end = *offsets.get_unchecked(index + 1) as usize;
                    let start = *offsets.get_unchecked(index) as usize;
                    strings.values().slice_with_length(start, end - start)
                },
            ))),

            DataType::Timestamp(time_unit, _) => {
                let (original_value, converted) = match time_unit {
                    TimeUnit::Second => {
                        let value = *value
                            .as_primitive::<TimestampSecondType>()
                            .values()
                            .get_unchecked(index);
                        (value, Utc.timestamp_opt(value, 0).single())
                    }
                    TimeUnit::Millisecond => {
                        let value = *value
                            .as_primitive::<TimestampMillisecondType>()
                            .values()
                            .get_unchecked(index);
                        (value, Utc.timestamp_millis_opt(value).single())
                    }
                    TimeUnit::Microsecond => {
                        let value = *value
                            .as_primitive::<TimestampMicrosecondType>()
                            .values()
                            .get_unchecked(index);
                        (value, Utc.timestamp_micros(value).single())
                    }
                    TimeUnit::Nanosecond => {
                        let value = *value
                            .as_primitive::<TimestampNanosecondType>()
                            .values()
                            .get_unchecked(index);
                        (value, Some(Utc.timestamp_nanos(value)))
                    }
                };

                converted
                    .map(|value| ValueOrRef::DateTime(value.into()))
                    .ok_or(ValueError::TimestampConversionFailure {
                        original_value,
                        time_unit: *time_unit,
                    })
            }

            DataType::FixedSizeBinary(_) => Ok(ValueOrRef::Array(ArrayValueOrRef::Buffer({
                let bytes = value.as_fixed_size_binary();
                let start = bytes.value_offset(index) as usize;
                let buffer = bytes
                    .values()
                    .slice_with_length(start, bytes.value_length() as usize)
                    .clone();
                BufferArray::new_u8(buffer)
            }))),

            d => todo!("{d} is not implemented"),
        }
    }
}

#[cfg(test)]
mod tests {
    use arrow::array::{
        FixedSizeBinaryArray, Float16Array, Float32Array, Float64Array, Int32Array,
        LargeStringArray, StringArray, TimestampMicrosecondArray, TimestampMillisecondArray,
        TimestampNanosecondArray, TimestampSecondArray, UInt64Array,
    };
    use half::f16;

    use super::*;

    /// Scenario: Arrow numeric and string arrays contain ordinary values, nulls, and boundary indexes.
    /// Guarantees: Values are converted to engine primitives while null and out-of-range access returns Null.
    #[test]
    fn arrow_values_convert_numbers_strings_and_nulls() {
        let integers = DictionaryValueArray::from(&Int32Array::from(vec![Some(-7), None, Some(9)]));
        assert_eq!(integers.get_value_at(0), Ok(ValueOrRef::Integer(-7)));
        assert_eq!(integers.get_value_at(1), Ok(ValueOrRef::Null));
        assert_eq!(integers.get_value_at(3), Ok(ValueOrRef::Null));

        let strings = DictionaryValueArray::from(&StringArray::from(vec![
            Some("alpha"),
            None,
            Some("omega"),
        ]));
        assert_eq!(
            strings.get_value_at(0),
            Ok(ValueOrRef::String(StringValueOrRef::new_ref("alpha")))
        );
        assert_eq!(strings.get_value_at(1), Ok(ValueOrRef::Null));
        assert_eq!(strings.get_value_at(3), Ok(ValueOrRef::Null));
    }

    /// Scenario: Arrow UInt64 values straddle the largest value representable by the engine's i64 integer type.
    /// Guarantees: Representable values are preserved while larger values, including u64::MAX, return conversion errors without wrapping.
    #[test]
    fn uint64_values_outside_i64_range_return_errors() {
        let values = DictionaryValueArray::from(&UInt64Array::from(vec![
            Some(0),
            Some(i64::MAX as u64),
            Some(i64::MAX as u64 + 1),
            Some(u64::MAX),
            None,
        ]));

        assert_eq!(values.get_value_at(0), Ok(ValueOrRef::Integer(0)));
        assert_eq!(values.get_value_at(1), Ok(ValueOrRef::Integer(i64::MAX)));
        assert_eq!(
            values.get_value_at(2),
            Err(ValueError::IntegerConversionFailure {
                original_value: i64::MAX as u64 + 1,
            })
        );
        assert_eq!(
            values.get_value_at(3),
            Err(ValueError::IntegerConversionFailure {
                original_value: u64::MAX,
            })
        );
        assert_eq!(values.get_value_at(4), Ok(ValueOrRef::Null));
    }

    /// Scenario: Arrow Float16, Float32, and Float64 arrays contain finite values, infinities, negative zero, NaN, and null.
    /// Guarantees: Every width converts to f64 without losing special-value semantics or changing null handling.
    #[test]
    fn arrow_float_values_preserve_special_values() {
        let float16 = DictionaryValueArray::from(&Float16Array::from(vec![
            Some(f16::from_f32(1.5)),
            Some(f16::NEG_INFINITY),
            None,
        ]));
        assert_eq!(float16.get_value_at(0), Ok(ValueOrRef::Double(1.5)));
        assert_eq!(
            float16.get_value_at(1),
            Ok(ValueOrRef::Double(f64::NEG_INFINITY))
        );
        assert_eq!(float16.get_value_at(2), Ok(ValueOrRef::Null));

        let float32_nan = f32::from_bits(0x7fc0_0001);
        let float32 =
            DictionaryValueArray::from(&Float32Array::from(vec![Some(-0.0), Some(float32_nan)]));
        assert_eq!(float32.get_value_at(0), Ok(ValueOrRef::Double(-0.0)));
        assert_eq!(
            float32.get_value_at(1),
            Ok(ValueOrRef::Double(float32_nan as f64))
        );

        let float64_nan = f64::from_bits(0x7ff8_0000_0000_0001);
        let float64 = DictionaryValueArray::from(&Float64Array::from(vec![
            Some(f64::INFINITY),
            Some(float64_nan),
        ]));
        assert_eq!(
            float64.get_value_at(0),
            Ok(ValueOrRef::Double(f64::INFINITY))
        );
        assert_eq!(float64.get_value_at(1), Ok(ValueOrRef::Double(float64_nan)));
    }

    /// Scenario: A LargeUtf8 Arrow array contains adjacent ASCII and multibyte strings with an intervening null.
    /// Guarantees: Large offsets select exactly each value's UTF-8 bytes and preserve null entries.
    #[test]
    fn large_utf8_values_preserve_offsets_and_multibyte_text() {
        let values = DictionaryValueArray::from(&LargeStringArray::from(vec![
            Some("prefix"),
            Some("\u{e9}\u{65e5}"),
            None,
            Some("suffix"),
        ]));

        assert_eq!(
            values.get_value_at(0),
            Ok(ValueOrRef::String(StringValueOrRef::new_ref("prefix")))
        );
        assert_eq!(
            values.get_value_at(1),
            Ok(ValueOrRef::String(StringValueOrRef::new_ref(
                "\u{e9}\u{65e5}"
            )))
        );
        assert_eq!(values.get_value_at(2), Ok(ValueOrRef::Null));
        assert_eq!(
            values.get_value_at(3),
            Ok(ValueOrRef::String(StringValueOrRef::new_ref("suffix")))
        );
    }

    /// Scenario: Arrow timestamps use second, millisecond, microsecond, and nanosecond units with negative, positive, and null values.
    /// Guarantees: Every unit converts to the same instant represented by its source integer and Arrow nulls remain Null.
    #[test]
    fn arrow_timestamp_units_preserve_instants_and_nulls() {
        let seconds =
            DictionaryValueArray::from(&TimestampSecondArray::from(vec![Some(-1), None, Some(1)]));
        match seconds.get_value_at(0).unwrap() {
            ValueOrRef::DateTime(value) => assert_eq!(value.timestamp(), -1),
            value => panic!("expected datetime, got {value:?}"),
        }
        assert_eq!(seconds.get_value_at(1), Ok(ValueOrRef::Null));
        match seconds.get_value_at(2).unwrap() {
            ValueOrRef::DateTime(value) => assert_eq!(value.timestamp(), 1),
            value => panic!("expected datetime, got {value:?}"),
        }

        let milliseconds =
            DictionaryValueArray::from(&TimestampMillisecondArray::from(vec![-1_500]));
        match milliseconds.get_value_at(0).unwrap() {
            ValueOrRef::DateTime(value) => assert_eq!(value.timestamp_millis(), -1_500),
            value => panic!("expected datetime, got {value:?}"),
        }

        let microseconds =
            DictionaryValueArray::from(&TimestampMicrosecondArray::from(vec![1_500_001]));
        match microseconds.get_value_at(0).unwrap() {
            ValueOrRef::DateTime(value) => assert_eq!(value.timestamp_micros(), 1_500_001),
            value => panic!("expected datetime, got {value:?}"),
        }

        let nanoseconds =
            DictionaryValueArray::from(&TimestampNanosecondArray::from(vec![i64::MIN, i64::MAX]));
        match nanoseconds.get_value_at(0).unwrap() {
            ValueOrRef::DateTime(value) => {
                assert_eq!(value.timestamp_nanos_opt(), Some(i64::MIN))
            }
            value => panic!("expected datetime, got {value:?}"),
        }
        match nanoseconds.get_value_at(1).unwrap() {
            ValueOrRef::DateTime(value) => {
                assert_eq!(value.timestamp_nanos_opt(), Some(i64::MAX))
            }
            value => panic!("expected datetime, got {value:?}"),
        }
    }

    /// Scenario: Second, millisecond, and microsecond Arrow timestamps exceed Chrono's representable calendar range.
    /// Guarantees: Both signed extremes return typed conversion errors instead of panicking or changing the timestamp.
    #[test]
    fn out_of_range_arrow_timestamps_return_conversion_errors() {
        let seconds =
            DictionaryValueArray::from(&TimestampSecondArray::from(vec![i64::MIN, i64::MAX]));
        let milliseconds =
            DictionaryValueArray::from(&TimestampMillisecondArray::from(vec![i64::MIN, i64::MAX]));
        let microseconds =
            DictionaryValueArray::from(&TimestampMicrosecondArray::from(vec![i64::MIN, i64::MAX]));

        for (values, time_unit) in [
            (seconds, TimeUnit::Second),
            (milliseconds, TimeUnit::Millisecond),
            (microseconds, TimeUnit::Microsecond),
        ] {
            assert_eq!(
                values.get_value_at(0),
                Err(ValueError::TimestampConversionFailure {
                    original_value: i64::MIN,
                    time_unit,
                })
            );
            assert_eq!(
                values.get_value_at(1),
                Err(ValueError::TimestampConversionFailure {
                    original_value: i64::MAX,
                    time_unit,
                })
            );
        }
    }

    /// Scenario: Arrow timestamp and fixed-size binary arrays are used as dictionary values.
    /// Guarantees: Time units and binary byte sequences are preserved by conversion.
    #[test]
    fn arrow_values_convert_timestamps_and_fixed_binary() {
        let timestamps =
            DictionaryValueArray::from(&TimestampMillisecondArray::from(vec![1_500_i64]));
        let timestamp = timestamps.get_value_at(0).unwrap();
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
            Ok(ValueOrRef::Array(ArrayValueOrRef::from([
                ValueOrRef::Integer(b'd' as i64),
                ValueOrRef::Integer(b'e' as i64),
                ValueOrRef::Integer(b'f' as i64),
            ])))
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
        assert_eq!(boolean.get_value_at(0), Ok(ValueOrRef::Boolean(false)));
        assert_eq!(boolean.get_value_at(1), Ok(ValueOrRef::Boolean(true)));
        assert_eq!(
            boolean.get_value_at(2),
            Err(ValueError::InvalidBoolean { index_value: 2 })
        );
    }
}
