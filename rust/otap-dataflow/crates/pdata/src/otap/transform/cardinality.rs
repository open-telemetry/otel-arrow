// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Cardinality estimation for dictionary-encoding decisions.
//!
//! This module provides [`FieldInfo`] and [`estimate_cardinality`], which
//! estimate the number of distinct values across a set of arrays so a caller
//! can decide whether a column fits within a `u8` or `u16` dictionary key.
//!
//! This is used by the query engine when it materializes evaluation results and
//! needs to choose a dictionary key width. It is intentionally separate from the
//! concatenation schema-unification path, which selects dictionary widths from
//! the summed physical value counts (a cheaper, more conservative bound) and
//! therefore does not need exact cardinality estimation.

use ahash::AHashSet;
use arrow::array::{Array, ArrayRef, ArrowPrimitiveType, AsArray, OffsetSizeTrait};
use arrow::datatypes::{
    ArrowNativeType, DataType, DurationMicrosecondType, DurationMillisecondType,
    DurationNanosecondType, DurationSecondType, Float64Type, GenericBinaryType, Int64Type,
    TimestampMicrosecondType, TimestampMillisecondType, TimestampNanosecondType,
    TimestampSecondType, UInt64Type,
};
use itertools::Either;
use roaring::RoaringBitmap;
use std::sync::Arc;

/// These are one less than the maximum cardinality of the key type. We should be
/// able to go up to 256/65536 without overflow, but there is a bug in arrow-rs
/// for some value types.
///
/// See:
///     - https://github.com/apache/arrow-rs/issues/9366
///     - github.com/open-telemetry/otel-arrow/issues/1971
pub(crate) const MAX_U8_CARDINALITY: usize = 255;
pub(crate) const MAX_U16_CARDINALITY: usize = 65535;

/// Accumulated information about a single field across one or more arrays, used
/// as input to [`estimate_cardinality`].
#[derive(Debug)]
pub struct FieldInfo<'a> {
    // The value type of the column. This must be some primitive type; the
    // estimator does not support struct or dictionary value types directly.
    value_type: &'a DataType,
    // Set if this originated from a dictionary column. The query engine always
    // constructs from a plain array so this is currently always `None`, but it
    // is retained because `check_cardinality` uses it to decide when an early
    // exit is sound.
    smallest_key_type: Option<DataType>,
    // The total number of values, excluding nulls
    total_value_count: usize,
    // The values arrays for the type, some of these may come from dictionary array values.
    values: Vec<ArrayRef>,
}

impl<'a> FieldInfo<'a> {
    /// Construct a [`FieldInfo`] describing a single array.
    #[must_use]
    pub fn new_from_array(array: &'a ArrayRef) -> Self {
        Self {
            value_type: array.data_type(),
            smallest_key_type: None,
            total_value_count: array.len() - array.null_count(),
            values: vec![Arc::clone(array)],
        }
    }
}

/// Estimate of the cardinality of a field
#[derive(PartialEq)]
pub enum Cardinality {
    WithinU8,
    WithinU16,
    GreaterThanU16,
}

impl Cardinality {
    const fn from_exact(count: usize) -> Cardinality {
        match count {
            count if count <= MAX_U8_CARDINALITY => Cardinality::WithinU8,
            count if count <= MAX_U16_CARDINALITY => Cardinality::WithinU16,
            _ => Cardinality::GreaterThanU16,
        }
    }
}

/// Estimate the cardinality of a set of arrays
#[must_use]
pub fn estimate_cardinality<'a>(info: &FieldInfo<'a>) -> Cardinality {
    // Small types
    match info.value_type.primitive_width() {
        Some(1) => return estimate_cardinality_small_type::<u8>(info),
        Some(2) => return estimate_cardinality_small_type::<u16>(info),
        Some(4) => return estimate_cardinality_small_type::<u32>(info),
        _ => {}
    };

    // Large types
    match info.value_type {
        DataType::UInt64 => estimate_cardinality_primitive_type::<UInt64Type, 8>(info),
        DataType::Int64 => estimate_cardinality_primitive_type::<Int64Type, 8>(info),
        DataType::Duration(unit) => match unit {
            arrow_schema::TimeUnit::Second => {
                estimate_cardinality_primitive_type::<DurationSecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Millisecond => {
                estimate_cardinality_primitive_type::<DurationMillisecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Microsecond => {
                estimate_cardinality_primitive_type::<DurationMicrosecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Nanosecond => {
                estimate_cardinality_primitive_type::<DurationNanosecondType, 8>(info)
            }
        },
        DataType::Timestamp(unit, _) => match unit {
            arrow_schema::TimeUnit::Second => {
                estimate_cardinality_primitive_type::<TimestampSecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Millisecond => {
                estimate_cardinality_primitive_type::<TimestampMillisecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Microsecond => {
                estimate_cardinality_primitive_type::<TimestampMicrosecondType, 8>(info)
            }
            arrow_schema::TimeUnit::Nanosecond => {
                estimate_cardinality_primitive_type::<TimestampNanosecondType, 8>(info)
            }
        },
        DataType::Float64 => estimate_cardinality_primitive_type::<Float64Type, 8>(info),
        DataType::FixedSizeBinary(8) => estimate_cardinality_fixed_size_type::<8>(info),
        DataType::FixedSizeBinary(16) => estimate_cardinality_fixed_size_type::<16>(info),
        DataType::Utf8 => estimate_cardinality_string_type::<i32>(info),
        DataType::LargeUtf8 => estimate_cardinality_string_type::<i64>(info),
        DataType::LargeBinary => {
            let iter = info
                .values
                .iter()
                .flat_map(|v| v.as_bytes::<GenericBinaryType<i64>>().iter().flatten());
            estimate_cardinality_generic(info, iter)
        }
        _ => unreachable!("Unexpected type: {:?}", info.value_type),
    }
}

fn estimate_cardinality_fixed_size_type<'a, const ELEMENT_WIDTH: usize>(
    info: &FieldInfo<'a>,
) -> Cardinality {
    estimate_cardinality_from_bytes::<ELEMENT_WIDTH>(info, |array| {
        array.as_fixed_size_binary().values()
    })
}

fn estimate_cardinality_primitive_type<'a, T, const ELEMENT_WIDTH: usize>(
    info: &FieldInfo<'a>,
) -> Cardinality
where
    T: ArrowPrimitiveType,
{
    estimate_cardinality_from_bytes::<ELEMENT_WIDTH>(info, |array| {
        array.as_primitive::<T>().values().inner()
    })
}

fn estimate_cardinality_from_bytes<'a, const ELEMENT_WIDTH: usize>(
    info: &'a FieldInfo<'a>,
    get_buffer: impl Fn(&'a ArrayRef) -> &'a [u8],
) -> Cardinality {
    let iter = info.values.iter().flat_map(|array| {
        let nulls = array.nulls();
        let buf = get_buffer(array);

        match nulls {
            Some(nulls) => Either::Left(nulls.valid_slices().flat_map(move |(start, end)| {
                let range = start * ELEMENT_WIDTH..end * ELEMENT_WIDTH;
                buf[range]
                    .as_chunks::<ELEMENT_WIDTH>()
                    .0
                    .iter()
                    .map(|chunk| chunk.as_slice())
            })),
            None => Either::Right(
                buf.as_chunks::<ELEMENT_WIDTH>()
                    .0
                    .iter()
                    .map(|chunk| chunk.as_slice()),
            ),
        }
    });

    estimate_cardinality_generic(info, iter)
}

fn estimate_cardinality_string_type<'a, T: OffsetSizeTrait>(info: &FieldInfo<'a>) -> Cardinality {
    let iter = info
        .values
        .iter()
        .flat_map(|v| v.as_string::<T>().iter().flatten().map(|s| s.as_bytes()));
    estimate_cardinality_generic(info, iter)
}

fn estimate_cardinality_generic<'a>(
    info: &FieldInfo<'a>,
    values: impl Iterator<Item = &'a [u8]>,
) -> Cardinality {
    // TODO: Consider re-use this across cardinality calculations
    let capacity = if info.total_value_count <= u8::MAX as usize {
        u8::MAX as usize
    } else {
        // TODO: is this too big?
        u16::MAX as usize
    };

    let mut set = AHashSet::with_capacity(capacity);
    let mut visited_element_count = 0;

    for value in values {
        _ = set.insert(value);
        visited_element_count += 1;

        let maybe_cardinality = check_cardinality(info, visited_element_count, set.len() as u64);
        if let Some(c) = maybe_cardinality {
            return c;
        }
    }

    Cardinality::from_exact(set.len())
}

fn estimate_cardinality_small_type<'a, T>(info: &FieldInfo<'a>) -> Cardinality
where
    T: ArrowNativeType + Into<u32>,
{
    // TODO: Play around with optimizing bitmap here
    let mut bitmap = RoaringBitmap::new();
    let mut visited_element_count = 0;

    for array in info.values.iter() {
        let value_data = array.to_data();
        let value_buf = value_data.buffer::<T>(0);
        match array.nulls() {
            Some(nulls) => {
                for (start, end) in nulls.valid_slices() {
                    let cardinality = visit_native_values(
                        &value_buf[start..end],
                        info,
                        &mut bitmap,
                        &mut visited_element_count,
                    );
                    if let Some(c) = cardinality {
                        return c;
                    }
                }
            }
            None => {
                let cardinality =
                    visit_native_values(value_buf, info, &mut bitmap, &mut visited_element_count);
                if let Some(c) = cardinality {
                    return c;
                }
            }
        }

        // TODO: Consider when to call bitmap.optimize(). This seemed to regress
        // things for high numbers of small batches. Maybe we can be smarter about
        // calling this only under certain conditions.
        // _ = bitmap.optimize();
    }

    Cardinality::from_exact(bitmap.len() as usize)
}

fn visit_native_values<'a, T>(
    values: &[T],
    info: &FieldInfo<'a>,
    bitmap: &mut RoaringBitmap,
    visited_element_count: &mut usize,
) -> Option<Cardinality>
where
    T: ArrowNativeType + Into<u32>,
{
    const CHUNK_SIZE: usize = 256;

    for chunk in values.chunks(CHUNK_SIZE) {
        bitmap.extend(chunk.iter().copied().map(|v| v.into()));
        *visited_element_count += chunk.len();

        let maybe_cardinality = check_cardinality(info, *visited_element_count, bitmap.len());
        if let Some(c) = maybe_cardinality {
            return Some(c);
        }
    }

    None
}

fn check_cardinality<'a>(
    info: &'a FieldInfo<'a>,
    visited_count: usize,
    current_cardinality: u64,
) -> Option<Cardinality> {
    if current_cardinality > MAX_U16_CARDINALITY as u64 {
        return Some(Cardinality::GreaterThanU16);
    }

    let duplicates_visited = visited_count - current_cardinality as usize;
    let max_possible_cardinality = info.total_value_count - duplicates_visited;

    // If the smallest key type is u8 then it's possible as we keep processing
    // values that we can reduce the size further, so we can't return.
    if max_possible_cardinality <= MAX_U16_CARDINALITY
        && info.smallest_key_type == Some(DataType::UInt16)
    {
        return Some(Cardinality::WithinU16);
    }

    if max_possible_cardinality <= MAX_U8_CARDINALITY
        && info.smallest_key_type == Some(DataType::UInt8)
    {
        return Some(Cardinality::WithinU8);
    }

    None
}
