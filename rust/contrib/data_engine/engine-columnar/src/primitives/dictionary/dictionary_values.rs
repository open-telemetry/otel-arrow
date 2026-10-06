// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use ahash::{AHashMap, RandomState};
use arrow::{array::*, buffer::*, datatypes::*};
use chrono::{DateTime, TimeZone, Utc};
use indexmap::IndexSet;
use thiserror::Error;

use crate::*;

pub(crate) type ValueOrRefSet<'a> = GenericSet<ValueOrRef<'a>>;
pub type GenericSet<T> = IndexSet<T, RandomState>;
pub type IndexLookup = Option<AHashMap<usize, Option<usize>>>;

#[derive(Debug, Clone)]
pub enum DictionaryValueArray<'a> {
    Array(DictionaryArrowValueArray),
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
            DictionaryValueArray::Array(a) => a.nulls(),
            DictionaryValueArray::Vec(a) => {
                build_null_buffer(a.iter().map(|value| matches!(value, ValueOrRef::Null)))
            }
            DictionaryValueArray::Set(a) => {
                build_null_buffer(a.iter().map(|value| matches!(value, ValueOrRef::Null)))
            }
            DictionaryValueArray::Boolean => None,
        }
    }
}

impl<'a> DictionaryValueArray<'a> {
    pub fn get_value_at(&self, index: usize) -> Result<ValueOrRef<'a>, DictionaryValueError> {
        match self {
            DictionaryValueArray::Array(a) => a.get_value_at(index),
            DictionaryValueArray::Vec(a) => Ok(a.get(index).cloned().unwrap_or(ValueOrRef::Null)),
            DictionaryValueArray::Set(a) => {
                Ok(a.get_index(index).cloned().unwrap_or(ValueOrRef::Null))
            }
            DictionaryValueArray::Boolean => Ok(match index {
                0 => ValueOrRef::Boolean(false),
                1 => ValueOrRef::Boolean(true),
                _ => ValueOrRef::Null,
            }),
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

#[derive(Debug, Clone)]
pub enum DictionaryArrowValueArray {
    Int8(PrimitiveArray<Int8Type>),
    Int16(PrimitiveArray<Int16Type>),
    Int32(PrimitiveArray<Int32Type>),
    Int64(PrimitiveArray<Int64Type>),

    UInt8(PrimitiveArray<UInt8Type>),
    UInt16(PrimitiveArray<UInt16Type>),
    UInt32(PrimitiveArray<UInt32Type>),
    UInt64(PrimitiveArray<UInt64Type>),

    Float16(PrimitiveArray<Float16Type>),
    Float32(PrimitiveArray<Float32Type>),
    Float64(PrimitiveArray<Float64Type>),

    Utf8(StringArray),
    LargeUtf8(LargeStringArray),

    FixedSizeBinary(FixedSizeBinaryArray),

    TimestampSecond(PrimitiveArray<TimestampSecondType>),
    TimestampMillisecond(PrimitiveArray<TimestampMillisecondType>),
    TimestampMicrosecond(PrimitiveArray<TimestampMicrosecondType>),
    TimestampNanosecond(PrimitiveArray<TimestampNanosecondType>),
}

impl DictionaryArrowValueArray {
    pub fn as_array(&self) -> &dyn Array {
        match self {
            DictionaryArrowValueArray::Int8(a) => a,
            DictionaryArrowValueArray::Int16(a) => a,
            DictionaryArrowValueArray::Int32(a) => a,
            DictionaryArrowValueArray::Int64(a) => a,
            DictionaryArrowValueArray::UInt8(a) => a,
            DictionaryArrowValueArray::UInt16(a) => a,
            DictionaryArrowValueArray::UInt32(a) => a,
            DictionaryArrowValueArray::UInt64(a) => a,
            DictionaryArrowValueArray::Float16(a) => a,
            DictionaryArrowValueArray::Float32(a) => a,
            DictionaryArrowValueArray::Float64(a) => a,
            DictionaryArrowValueArray::Utf8(a) => a,
            DictionaryArrowValueArray::LargeUtf8(a) => a,
            DictionaryArrowValueArray::FixedSizeBinary(a) => a,
            DictionaryArrowValueArray::TimestampSecond(a) => a,
            DictionaryArrowValueArray::TimestampMillisecond(a) => a,
            DictionaryArrowValueArray::TimestampMicrosecond(a) => a,
            DictionaryArrowValueArray::TimestampNanosecond(a) => a,
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

    fn get_value_at(&self, index: usize) -> Result<ValueOrRef<'static>, DictionaryValueError> {
        if index >= self.len() || self.nulls().map(|n| n.is_null(index)).unwrap_or(false) {
            return Ok(ValueOrRef::Null);
        }

        unsafe {
            match self {
                DictionaryArrowValueArray::Int8(array) => {
                    Ok(ValueOrRef::Integer(array.value_unchecked(index) as i64))
                }
                DictionaryArrowValueArray::Int16(array) => {
                    Ok(ValueOrRef::Integer(array.value_unchecked(index) as i64))
                }
                DictionaryArrowValueArray::Int32(array) => {
                    Ok(ValueOrRef::Integer(array.value_unchecked(index) as i64))
                }
                DictionaryArrowValueArray::Int64(array) => {
                    Ok(ValueOrRef::Integer(array.value_unchecked(index)))
                }

                DictionaryArrowValueArray::UInt8(array) => {
                    Ok(ValueOrRef::Integer(array.value_unchecked(index) as i64))
                }
                DictionaryArrowValueArray::UInt16(array) => {
                    Ok(ValueOrRef::Integer(array.value_unchecked(index) as i64))
                }
                DictionaryArrowValueArray::UInt32(array) => {
                    Ok(ValueOrRef::Integer(array.value_unchecked(index) as i64))
                }
                DictionaryArrowValueArray::UInt64(array) => {
                    let value = array.value_unchecked(index);

                    match TryInto::<i64>::try_into(value) {
                        Ok(v) => Ok(ValueOrRef::Integer(v)),
                        Err(_) => Err(DictionaryValueError::IntegerConversionFailure {
                            original_value: value,
                        }),
                    }
                }
                DictionaryArrowValueArray::Float16(array) => {
                    Ok(ValueOrRef::Double(array.value_unchecked(index).into()))
                }
                DictionaryArrowValueArray::Float32(array) => {
                    Ok(ValueOrRef::Double(array.value_unchecked(index) as f64))
                }
                DictionaryArrowValueArray::Float64(array) => {
                    Ok(ValueOrRef::Double(array.value_unchecked(index)))
                }

                DictionaryArrowValueArray::Utf8(strings) => Ok(ValueOrRef::String(
                    StringValueOrRef::new_utf8_unvalidated({
                        let offsets = strings.value_offsets();
                        let end = *offsets.get_unchecked(index + 1) as usize;
                        let start = *offsets.get_unchecked(index) as usize;
                        strings.values().slice_with_length(start, end - start)
                    }),
                )),
                DictionaryArrowValueArray::LargeUtf8(strings) => Ok(ValueOrRef::String(
                    StringValueOrRef::new_utf8_unvalidated({
                        let offsets = strings.value_offsets();
                        let end = *offsets.get_unchecked(index + 1) as usize;
                        let start = *offsets.get_unchecked(index) as usize;
                        strings.values().slice_with_length(start, end - start)
                    }),
                )),

                DictionaryArrowValueArray::TimestampSecond(array) => {
                    let value = array.value_unchecked(index);
                    Self::finalize_date_time_conversion(
                        value,
                        TimeUnit::Second,
                        Utc.timestamp_opt(value, 0).single(),
                    )
                }
                DictionaryArrowValueArray::TimestampMillisecond(array) => {
                    let value = array.value_unchecked(index);
                    Self::finalize_date_time_conversion(
                        value,
                        TimeUnit::Millisecond,
                        Utc.timestamp_millis_opt(value).single(),
                    )
                }
                DictionaryArrowValueArray::TimestampMicrosecond(array) => {
                    let value = array.value_unchecked(index);
                    Self::finalize_date_time_conversion(
                        value,
                        TimeUnit::Microsecond,
                        Utc.timestamp_micros(value).single(),
                    )
                }
                DictionaryArrowValueArray::TimestampNanosecond(array) => {
                    let value = array.value_unchecked(index);
                    Self::finalize_date_time_conversion(
                        value,
                        TimeUnit::Nanosecond,
                        Some(Utc.timestamp_nanos(value)),
                    )
                }

                DictionaryArrowValueArray::FixedSizeBinary(bytes) => {
                    Ok(ValueOrRef::Array(ArrayValueOrRef::Buffer({
                        let start = bytes.value_offset(index) as usize;
                        let buffer = bytes
                            .values()
                            .slice_with_length(start, bytes.value_length() as usize)
                            .clone();
                        BufferArray::new_u8(buffer)
                    })))
                }
            }
        }
    }

    fn finalize_date_time_conversion(
        original_value: i64,
        time_unit: TimeUnit,
        value: Option<DateTime<Utc>>,
    ) -> Result<ValueOrRef<'static>, DictionaryValueError> {
        value.map(|value| ValueOrRef::DateTime(value.into())).ok_or(
            DictionaryValueError::TimestampConversionFailure {
                original_value,
                time_unit,
            },
        )
    }
}

macro_rules! impl_from_value_array {
    ($arrow_ty:ty, $variant:ident) => {
        impl From<$arrow_ty> for DictionaryValueArray<'static> {
            fn from(value: $arrow_ty) -> DictionaryValueArray<'static> {
                DictionaryValueArray::Array(DictionaryArrowValueArray::$variant(value))
            }
        }

        impl From<&$arrow_ty> for DictionaryValueArray<'static> {
            fn from(value: &$arrow_ty) -> DictionaryValueArray<'static> {
                DictionaryValueArray::Array(DictionaryArrowValueArray::$variant(value.clone()))
            }
        }
    };
}

impl_from_value_array!(PrimitiveArray<Int8Type>, Int8);
impl_from_value_array!(PrimitiveArray<Int16Type>, Int16);
impl_from_value_array!(PrimitiveArray<Int32Type>, Int32);
impl_from_value_array!(PrimitiveArray<Int64Type>, Int64);
impl_from_value_array!(PrimitiveArray<UInt8Type>, UInt8);
impl_from_value_array!(PrimitiveArray<UInt16Type>, UInt16);
impl_from_value_array!(PrimitiveArray<UInt32Type>, UInt32);
impl_from_value_array!(PrimitiveArray<UInt64Type>, UInt64);
impl_from_value_array!(PrimitiveArray<Float16Type>, Float16);
impl_from_value_array!(PrimitiveArray<Float32Type>, Float32);
impl_from_value_array!(PrimitiveArray<Float64Type>, Float64);
impl_from_value_array!(PrimitiveArray<TimestampSecondType>, TimestampSecond);
impl_from_value_array!(
    PrimitiveArray<TimestampMillisecondType>,
    TimestampMillisecond
);
impl_from_value_array!(
    PrimitiveArray<TimestampMicrosecondType>,
    TimestampMicrosecond
);
impl_from_value_array!(PrimitiveArray<TimestampNanosecondType>, TimestampNanosecond);
impl_from_value_array!(FixedSizeBinaryArray, FixedSizeBinary);
impl_from_value_array!(StringArray, Utf8);
impl_from_value_array!(LargeStringArray, LargeUtf8);

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
pub enum DictionaryValueError {
    #[error("UInt64 value '{original_value}' could not be converted into 'Integer'")]
    IntegerConversionFailure { original_value: u64 },
    #[error(
        "Timestamp value '{original_value}' with unit '{time_unit:?}' could not be converted into 'DateTime'"
    )]
    TimestampConversionFailure {
        original_value: i64,
        time_unit: TimeUnit,
    },
}

fn build_null_buffer(nulls: impl ExactSizeIterator<Item = bool>) -> Option<NullBuffer> {
    let len = nulls.len();
    let mut builder = None;

    for (index, is_null) in nulls.enumerate() {
        if is_null {
            let builder = builder.get_or_insert_with(|| {
                let mut builder = BooleanBufferBuilder::new(len);
                builder.append_n(index, true);
                builder
            });
            builder.append(false);
        } else if let Some(builder) = builder.as_mut() {
            builder.append(true);
        }
    }

    builder.map(|builder| NullBuffer::new(builder.build()))
}

#[cfg(test)]
mod tests {
    use arrow::array::{
        FixedSizeBinaryArray, Float16Array, Float32Array, Float64Array, Int32Array,
        LargeStringArray, StringArray, TimestampMicrosecondArray, TimestampMillisecondArray,
        TimestampNanosecondArray, TimestampSecondArray, UInt64Array,
    };
    use chrono::DateTime;
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

    /// Scenario: Values are accessed from Arrow, Vec, Set, and Boolean dictionary storage at valid, null, and invalid indexes.
    /// Guarantees: Every storage variant returns values and Null consistently while invalid Boolean indexes remain typed errors.
    #[test]
    fn dictionary_value_storage_variants_share_result_contract() {
        let array = DictionaryValueArray::from(&Int32Array::from(vec![Some(7), None, Some(9)]));
        assert_eq!(array.get_value_at(0), Ok(ValueOrRef::Integer(7)));
        assert_eq!(array.get_value_at(1), Ok(ValueOrRef::Null));
        assert_eq!(array.get_value_at(3), Ok(ValueOrRef::Null));

        let values = DictionaryValueArray::from(vec![
            ValueOrRef::Integer(7),
            ValueOrRef::Null,
            ValueOrRef::Integer(9),
        ]);
        assert_eq!(values.get_value_at(0), Ok(ValueOrRef::Integer(7)));
        assert_eq!(values.get_value_at(1), Ok(ValueOrRef::Null));
        assert_eq!(values.get_value_at(3), Ok(ValueOrRef::Null));

        let mut set = ValueOrRefSet::default();
        set.insert(ValueOrRef::Integer(7));
        set.insert(ValueOrRef::Null);
        let set = DictionaryValueArray::from(set);
        assert_eq!(set.get_value_at(0), Ok(ValueOrRef::Integer(7)));
        assert_eq!(set.get_value_at(1), Ok(ValueOrRef::Null));
        assert_eq!(set.get_value_at(2), Ok(ValueOrRef::Null));

        let boolean = DictionaryValueArray::Boolean;
        assert_eq!(boolean.get_value_at(0), Ok(ValueOrRef::Boolean(false)));
        assert_eq!(boolean.get_value_at(1), Ok(ValueOrRef::Boolean(true)));
        assert_eq!(boolean.get_value_at(2), Ok(ValueOrRef::Null));
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
            Err(DictionaryValueError::IntegerConversionFailure {
                original_value: i64::MAX as u64 + 1,
            })
        );
        assert_eq!(
            values.get_value_at(3),
            Err(DictionaryValueError::IntegerConversionFailure {
                original_value: u64::MAX,
            })
        );
        assert_eq!(values.get_value_at(4), Ok(ValueOrRef::Null));
    }

    /// Scenario: Dictionary arrays produce matching errors, different integer errors, different timestamp-unit errors, and Null.
    /// Guarantees: Equality compares typed conversion failures exactly and never equates a conversion failure with Null.
    #[test]
    fn dictionary_value_equality_distinguishes_conversion_failures() {
        let integer_error =
            DictionaryValueArray::from(&UInt64Array::from(vec![i64::MAX as u64 + 1]));
        let same_integer_error =
            DictionaryValueArray::from(&UInt64Array::from(vec![i64::MAX as u64 + 1]));
        let different_integer_error =
            DictionaryValueArray::from(&UInt64Array::from(vec![u64::MAX]));
        let second_error = DictionaryValueArray::from(&TimestampSecondArray::from(vec![i64::MAX]));
        let millisecond_error =
            DictionaryValueArray::from(&TimestampMillisecondArray::from(vec![i64::MAX]));
        let null = DictionaryValueArray::from(&UInt64Array::from(vec![None]));

        assert_eq!(integer_error, same_integer_error);
        assert_ne!(integer_error, different_integer_error);
        assert_ne!(second_error, millisecond_error);
        assert_ne!(integer_error, null);
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
                Err(DictionaryValueError::TimestampConversionFailure {
                    original_value: i64::MIN,
                    time_unit,
                })
            );
            assert_eq!(
                values.get_value_at(1),
                Err(DictionaryValueError::TimestampConversionFailure {
                    original_value: i64::MAX,
                    time_unit,
                })
            );
        }
    }

    /// Scenario: Timestamp values sit at and immediately beyond Chrono's exact second, millisecond, and microsecond limits.
    /// Guarantees: The final representable values succeed while their adjacent out-of-range values return typed errors.
    #[test]
    fn arrow_timestamps_enforce_exact_chrono_boundaries() {
        let min = DateTime::<Utc>::MIN_UTC;
        let max = DateTime::<Utc>::MAX_UTC;

        let cases = [
            (
                DictionaryValueArray::from(&TimestampSecondArray::from(vec![
                    min.timestamp(),
                    min.timestamp() - 1,
                    max.timestamp(),
                    max.timestamp() + 1,
                ])),
                TimeUnit::Second,
                min.timestamp() - 1,
                max.timestamp() + 1,
            ),
            (
                DictionaryValueArray::from(&TimestampMillisecondArray::from(vec![
                    min.timestamp_millis(),
                    min.timestamp_millis() - 1,
                    max.timestamp_millis(),
                    max.timestamp_millis() + 1,
                ])),
                TimeUnit::Millisecond,
                min.timestamp_millis() - 1,
                max.timestamp_millis() + 1,
            ),
            (
                DictionaryValueArray::from(&TimestampMicrosecondArray::from(vec![
                    min.timestamp_micros(),
                    min.timestamp_micros() - 1,
                    max.timestamp_micros(),
                    max.timestamp_micros() + 1,
                ])),
                TimeUnit::Microsecond,
                min.timestamp_micros() - 1,
                max.timestamp_micros() + 1,
            ),
        ];

        for (values, time_unit, below_min, above_max) in cases {
            assert!(matches!(
                values.get_value_at(0),
                Ok(ValueOrRef::DateTime(_))
            ));
            assert_eq!(
                values.get_value_at(1),
                Err(DictionaryValueError::TimestampConversionFailure {
                    original_value: below_min,
                    time_unit,
                })
            );
            assert!(matches!(
                values.get_value_at(2),
                Ok(ValueOrRef::DateTime(_))
            ));
            assert_eq!(
                values.get_value_at(3),
                Err(DictionaryValueError::TimestampConversionFailure {
                    original_value: above_max,
                    time_unit,
                })
            );
        }
    }

    /// Scenario: Equal Arrow timestamp values carry no timezone, UTC, and a non-UTC timezone metadata string.
    /// Guarantees: Timezone metadata does not alter the represented instant during conversion to the engine DateTime value.
    #[test]
    fn arrow_timestamp_timezone_metadata_preserves_instant() {
        let without_timezone = DictionaryValueArray::from(&TimestampSecondArray::from(vec![1_500]));
        let utc = DictionaryValueArray::from(
            &TimestampSecondArray::from(vec![1_500]).with_timezone("UTC"),
        );
        let offset = DictionaryValueArray::from(
            &TimestampSecondArray::from(vec![1_500]).with_timezone("+02:00"),
        );

        let expected = without_timezone.get_value_at(0).unwrap();
        assert_eq!(utc.get_value_at(0), Ok(expected.clone()));
        assert_eq!(offset.get_value_at(0), Ok(expected));
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
            ValueOrRef::Null,
            ValueOrRef::Integer(1),
            ValueOrRef::Null,
            ValueOrRef::Integer(3),
        ]);
        let nulls = values.nulls().unwrap();
        assert_eq!(nulls.null_count(), 2);
        assert!(nulls.is_null(0));
        assert!(nulls.is_valid(1));
        assert!(nulls.is_null(2));
        assert!(nulls.is_valid(3));

        let mut set = ValueOrRefSet::default();
        set.insert(ValueOrRef::Integer(1));
        set.insert(ValueOrRef::Null);
        set.insert(ValueOrRef::Integer(3));
        let set = DictionaryValueArray::from(set);
        let nulls = set.nulls().unwrap();
        assert_eq!(nulls.null_count(), 1);
        assert!(nulls.is_valid(0));
        assert!(nulls.is_null(1));
        assert!(nulls.is_valid(2));

        let boolean = DictionaryValueArray::Boolean;
        assert_eq!(boolean.get_value_at(0), Ok(ValueOrRef::Boolean(false)));
        assert_eq!(boolean.get_value_at(1), Ok(ValueOrRef::Boolean(true)));
        assert_eq!(boolean.get_value_at(2), Ok(ValueOrRef::Null));
    }
}
