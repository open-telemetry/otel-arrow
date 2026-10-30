// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Byte-sizing helpers for OTAP Arrow record batches.
//!
//! [`record_batch_logical_bytes`] estimates the bytes associated with the
//! active Arrow array ranges using [`ArrayData::get_slice_memory_size`]. This
//! is the OTAP representation's `num_bytes()` value; it is not an encoded OTAP
//! wire size.
//!
//! [`record_batch_pinned_bytes`] counts deduped Arrow-owned buffer capacity
//! retained by record batches. This is an estimate of retained memory, not
//! process RSS: it excludes allocator, struct, and `Arc` overhead.
//!
//! Buffer allocations are deduped by [`arrow::buffer::Buffer::data_ptr`], which
//! returns the allocation base and ignores slice offsets. This matters because
//! OTAP transforms such as `otap::transform::split` use
//! [`RecordBatch::slice`](arrow::array::RecordBatch::slice), so multiple slices
//! can share the same parent allocation.
//!
//! Retained-memory accounting uses buffer `capacity()`, not `len()`: a small
//! slice pins the whole parent allocation until it is dropped. One known
//! limitation is that externally owned Arrow buffers report `capacity() == 0`;
//! IPC-decoded OTAP batches are Rust-allocated today, but future zero-copy or
//! mmap ingest would be under-counted by retained-memory accounting.
//!
//! `OtapArrowRecords` does not cache these sizes. Its stores are cloneable and
//! mutable through `set()` and `remove()`, so a cache owned by the records could
//! become stale. Runtime payload wrappers cache logical size outside this
//! low-level data model. Consumers retaining bare records should compute once
//! when retention starts and store the value with their retained state or ticket.
//!
//! Sizing performance is proportional to the number of arrays and buffers, not
//! to the number of rows or byte values. Logical sizing walks typed arrays
//! directly and only falls back to `to_data()` for array types OTAP does not
//! use, because `to_data()` allocates a structural clone of every array (and
//! recursively of every child) and showed up in batch processor profiles with
//! many small inputs. Retained-memory accounting still calls `to_data()` per
//! column and uses a `HashSet` to dedupe buffers; callers can reuse and clear
//! a [`CountedAllocations`] value across calls if this ever shows up in
//! profiles.

use std::{collections::HashSet, ptr::NonNull};

use arrow::array::{Array, ArrayData, ArrayRef, AsArray, OffsetSizeTrait, RecordBatch};
use arrow::datatypes::DataType;
use arrow::error::ArrowError;
use arrow::util::bit_util;

/// Buffer allocations already counted during one retained-memory accounting
/// call.
#[derive(Debug, Default)]
pub struct CountedAllocations(HashSet<NonNull<u8>>);

/// Returns deduped Arrow-owned buffer capacity retained by `batch`.
///
/// Buffers shared by multiple arrays in the same accounting call are counted
/// once.
#[must_use]
pub fn record_batch_pinned_bytes(batch: &RecordBatch, seen: &mut CountedAllocations) -> usize {
    batch
        .columns()
        .iter()
        .map(|array| array_data_pinned_bytes(&array.to_data(), seen))
        .sum()
}

/// Returns the logical Arrow buffer bytes associated with `batch`.
///
/// The definition is [`ArrayData::get_slice_memory_size`], computed without
/// materializing `ArrayData` (see [`array_slice_memory_size`]). Direct buffers
/// are measured using the array's active offset and length, nested child
/// arrays follow Arrow's own recursive sizing semantics, and shared buffers
/// are counted once per logical array reference.
pub fn record_batch_logical_bytes(batch: &RecordBatch) -> Result<usize, ArrowError> {
    batch.columns().iter().try_fold(0usize, |total, array| {
        checked_add(total, array_slice_memory_size(array.as_ref())?)
    })
}

fn array_data_pinned_bytes(data: &ArrayData, seen: &mut CountedAllocations) -> usize {
    let mut total = 0;

    for buffer in data.buffers() {
        if seen.0.insert(buffer.data_ptr()) {
            total += buffer.capacity();
        }
    }

    if let Some(nulls) = data.nulls() {
        let buffer = nulls.buffer();
        if seen.0.insert(buffer.data_ptr()) {
            total += buffer.capacity();
        }
    }

    total
        + data
            .child_data()
            .iter()
            .map(|child| array_data_pinned_bytes(child, seen))
            .sum::<usize>()
}

/// Returns exactly what `array.to_data().get_slice_memory_size()` returns,
/// without allocating the intermediate `ArrayData`.
///
/// Mirrors arrow-data's `get_slice_memory_size`: each layout buffer counts
/// `len` fixed-width slots (`len + 1` for offsets), variable-width data counts
/// the bytes between the first and last active offset, a validity bitmap
/// counts `ceil(len / 8)`, and child arrays are sized recursively. Children
/// are what `to_data()` would produce: struct fields are already sliced, while
/// dictionary values and list values are the full child arrays. Types OTAP
/// does not produce fall back to `to_data()` so the result stays identical.
pub fn array_slice_memory_size(array: &dyn Array) -> Result<usize, ArrowError> {
    let len = array.len();
    let data_type = array.data_type();
    let own = match data_type {
        DataType::Boolean => bit_util::ceil(len, 8),
        DataType::Utf8 => byte_array_size(array.as_string::<i32>().value_offsets())?,
        DataType::LargeUtf8 => byte_array_size(array.as_string::<i64>().value_offsets())?,
        DataType::Binary => byte_array_size(array.as_binary::<i32>().value_offsets())?,
        DataType::LargeBinary => byte_array_size(array.as_binary::<i64>().value_offsets())?,
        DataType::FixedSizeBinary(width) => fixed_width_size(len, *width as usize)?,
        DataType::Dictionary(key_type, _) => {
            let key_width = key_type
                .primitive_width()
                .ok_or_else(|| invalid_layout(data_type))?;
            let values = array.as_any_dictionary().values();
            checked_add(
                fixed_width_size(len, key_width)?,
                array_slice_memory_size(values.as_ref())?,
            )?
        }
        DataType::Struct(_) => array
            .as_struct()
            .columns()
            .iter()
            .try_fold(0usize, |total, child| {
                checked_add(total, array_slice_memory_size(child.as_ref())?)
            })?,
        DataType::List(_) => {
            let list = array.as_list::<i32>();
            list_size(list.value_offsets(), list.values())?
        }
        DataType::LargeList(_) => {
            let list = array.as_list::<i64>();
            list_size(list.value_offsets(), list.values())?
        }
        _ if data_type.is_primitive() => {
            let width = data_type
                .primitive_width()
                .ok_or_else(|| invalid_layout(data_type))?;
            fixed_width_size(len, width)?
        }
        _ => return array.to_data().get_slice_memory_size(),
    };

    // `ArrayDataBuilder::build` drops validity buffers without nulls (e.g. a
    // slice of a nullable array that selects only valid rows), so arrow only
    // counts the bitmap when the active range contains a null.
    let null_bytes = if array.nulls().is_some_and(|nulls| nulls.null_count() > 0) {
        bit_util::ceil(len, 8)
    } else {
        0
    };
    checked_add(own, null_bytes)
}

/// Offsets buffer (`len + 1` slots) plus the active value bytes.
fn byte_array_size<O: OffsetSizeTrait>(offsets: &[O]) -> Result<usize, ArrowError> {
    debug_assert!(!offsets.is_empty(), "offsets always hold len + 1 entries");
    let value_bytes = (offsets[offsets.len() - 1] - offsets[0]).as_usize();
    checked_add(
        fixed_width_size(offsets.len(), size_of::<O>())?,
        value_bytes,
    )
}

/// Offsets buffer (`len + 1` slots) plus the full values child array.
fn list_size<O: OffsetSizeTrait>(offsets: &[O], values: &ArrayRef) -> Result<usize, ArrowError> {
    debug_assert!(!offsets.is_empty(), "offsets always hold len + 1 entries");
    checked_add(
        fixed_width_size(offsets.len(), size_of::<O>())?,
        array_slice_memory_size(values.as_ref())?,
    )
}

fn fixed_width_size(slots: usize, byte_width: usize) -> Result<usize, ArrowError> {
    slots.checked_mul(byte_width).ok_or_else(overflow_error)
}

fn checked_add(a: usize, b: usize) -> Result<usize, ArrowError> {
    a.checked_add(b).ok_or_else(overflow_error)
}

fn overflow_error() -> ArrowError {
    ArrowError::ComputeError("Integer overflow computing logical Arrow byte size".to_string())
}

fn invalid_layout(data_type: &DataType) -> ArrowError {
    ArrowError::InvalidArgumentError(format!("No fixed-width layout for data type {data_type}"))
}

#[cfg(test)]
mod tests {
    use std::{mem::size_of, sync::Arc};

    use crate::otap::{Logs, OtapArrowRecords};
    use crate::proto::OtlpProtoMessage;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::testing::fixtures::logs_with_full_resource_and_scope;
    use crate::testing::round_trip::{otlp_message_to_bytes, otlp_to_otap};
    use arrow::array::{
        Array, ArrayRef, DictionaryArray, ListArray, RecordBatch, StringArray, UInt8Array,
        UInt32Array, UInt32Builder,
    };
    use arrow::datatypes::{DataType, Field, Int32Type, Schema, UInt8Type};

    use super::{CountedAllocations, record_batch_logical_bytes, record_batch_pinned_bytes};

    fn batch_with_columns(columns: Vec<(&str, DataType, ArrayRef)>) -> RecordBatch {
        let fields = columns
            .iter()
            .map(|(name, data_type, _)| Field::new(*name, data_type.clone(), true))
            .collect::<Vec<_>>();
        let arrays = columns
            .into_iter()
            .map(|(_, _, array)| array)
            .collect::<Vec<_>>();

        RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays).unwrap()
    }

    fn pinned_bytes(batch: &RecordBatch) -> usize {
        let mut seen = CountedAllocations::default();
        record_batch_pinned_bytes(batch, &mut seen)
    }

    fn logical_bytes(batch: &RecordBatch) -> usize {
        record_batch_logical_bytes(batch).unwrap()
    }

    fn repeated_logs(record_count: usize) -> LogsData {
        let records: Vec<_> = (0..record_count)
            .map(|index| {
                LogRecord::build()
                    .time_unix_nano(index as u64 + 1)
                    .severity_number(SeverityNumber::Info as i32)
                    .event_name("repeated.event")
                    .body(AnyValue::new_string("the same repeated log body"))
                    .attributes(vec![KeyValue::new(
                        "deployment.environment",
                        AnyValue::new_string("production"),
                    )])
                    .finish()
            })
            .collect();

        LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![KeyValue::new(
                    "service.name",
                    AnyValue::new_string("size-comparison"),
                )])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .name("size-test")
                    .version("1.0.0")
                    .finish(),
                records,
            )],
        )])
    }

    #[test]
    fn fresh_unshared_batch_matches_arrow_buffer_memory_size_sum() {
        let batch = batch_with_columns(vec![
            (
                "number",
                DataType::UInt32,
                Arc::new(UInt32Array::from(vec![1, 2, 3, 4])) as ArrayRef,
            ),
            (
                "text",
                DataType::Utf8,
                Arc::new(StringArray::from(vec!["alpha", "beta", "gamma", "delta"])) as ArrayRef,
            ),
        ]);

        let arrow_sum = batch
            .columns()
            .iter()
            .map(|array| array.to_data().get_buffer_memory_size())
            .sum::<usize>();

        assert_eq!(pinned_bytes(&batch), arrow_sum);
    }

    #[test]
    fn slices_count_shared_parent_allocation_once_in_one_call() {
        let parent = batch_with_columns(vec![(
            "number",
            DataType::UInt32,
            Arc::new(UInt32Array::from_iter_values(0..16)) as ArrayRef,
        )]);
        let first = parent.slice(0, 8);
        let second = parent.slice(8, 8);

        let mut seen = CountedAllocations::default();
        let split_total = record_batch_pinned_bytes(&first, &mut seen)
            + record_batch_pinned_bytes(&second, &mut seen);

        assert_eq!(split_total, pinned_bytes(&parent));
    }

    #[test]
    fn shared_dictionary_values_are_counted_once_in_one_call() {
        let values: ArrayRef = Arc::new(StringArray::from(vec!["alpha", "beta"]));
        let keys_a = UInt8Array::from(vec![0, 1, 0, 1]);
        let keys_b = UInt8Array::from(vec![1, 0, 1, 0]);
        let expected = keys_a.to_data().get_buffer_memory_size()
            + keys_b.to_data().get_buffer_memory_size()
            + values.to_data().get_buffer_memory_size();

        let dict_a: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
            keys_a,
            Arc::clone(&values),
        ));
        let dict_b: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
            keys_b,
            Arc::clone(&values),
        ));

        let batch = batch_with_columns(vec![
            (
                "dict_a",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                dict_a,
            ),
            (
                "dict_b",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                dict_b,
            ),
        ]);

        assert_eq!(pinned_bytes(&batch), expected);
    }

    #[test]
    fn nullable_array_counts_null_buffer() {
        let array = UInt32Array::from(vec![Some(1), None, Some(3), None]);
        let data = array.to_data();
        let value_buffer_bytes = data
            .buffers()
            .iter()
            .map(|buffer| buffer.capacity())
            .sum::<usize>();
        let null_buffer_bytes = data.nulls().unwrap().buffer().capacity();
        assert!(null_buffer_bytes > 0);

        let batch = batch_with_columns(vec![("number", DataType::UInt32, Arc::new(array))]);

        assert_eq!(pinned_bytes(&batch), value_buffer_bytes + null_buffer_bytes);
    }

    #[test]
    fn primitive_builder_excess_capacity_is_counted() {
        let mut builder = UInt32Builder::with_capacity(16);
        builder.append_value(1);
        builder.append_value(2);
        builder.append_value(3);
        let array = builder.finish();
        let row_bytes = array.len() * size_of::<u32>();

        let batch = batch_with_columns(vec![("number", DataType::UInt32, Arc::new(array))]);

        assert!(pinned_bytes(&batch) > row_bytes);
    }

    /// Scenario: a primitive array has more allocated capacity than active values.
    /// Guarantees: logical sizing counts active values rather than retained capacity.
    #[test]
    fn logical_bytes_ignore_excess_primitive_capacity() {
        let mut builder = UInt32Builder::with_capacity(16);
        builder.append_value(1);
        builder.append_value(2);
        builder.append_value(3);
        let array = builder.finish();
        let batch = batch_with_columns(vec![("number", DataType::UInt32, Arc::new(array))]);

        assert_eq!(logical_bytes(&batch), 3 * size_of::<u32>());
        assert!(pinned_bytes(&batch) > logical_bytes(&batch));
    }

    /// Scenario: a record batch slice selects subsets of primitive and UTF-8 arrays.
    /// Guarantees: logical sizing counts active values and the UTF-8 terminal offset.
    #[test]
    fn logical_bytes_follow_primitive_and_utf8_slices() {
        let batch = batch_with_columns(vec![
            (
                "number",
                DataType::UInt32,
                Arc::new(UInt32Array::from(vec![1, 2, 3, 4])) as ArrayRef,
            ),
            (
                "text",
                DataType::Utf8,
                Arc::new(StringArray::from(vec!["alpha", "beta", "gamma", "delta"])) as ArrayRef,
            ),
        ]);

        let slice = batch.slice(1, 2);
        let primitive_bytes = 2 * size_of::<u32>();
        let utf8_offset_bytes = (slice.num_rows() + 1) * size_of::<i32>();
        let utf8_value_bytes = "betagamma".len();

        assert_eq!(
            logical_bytes(&slice),
            primitive_bytes + utf8_offset_bytes + utf8_value_bytes
        );
    }

    /// Scenario: two dictionary columns share one dictionary values array.
    /// Guarantees: each logical reference counts dictionary values and their terminal offset.
    #[test]
    fn logical_bytes_count_shared_dictionary_per_array_reference() {
        let values: ArrayRef = Arc::new(StringArray::from(vec!["alpha", "beta"]));
        let dict_a: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
            UInt8Array::from(vec![0, 1, 0, 1]),
            Arc::clone(&values),
        ));
        let dict_b: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
            UInt8Array::from(vec![1, 0, 1, 0]),
            Arc::clone(&values),
        ));
        let batch = batch_with_columns(vec![
            (
                "dict_a",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                dict_a,
            ),
            (
                "dict_b",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                dict_b,
            ),
        ]);
        let keys_bytes = 4 * size_of::<u8>();
        let dictionary_bytes = (values.len() + 1) * size_of::<i32>() + "alphabeta".len();

        assert_eq!(logical_bytes(&batch), 2 * (keys_bytes + dictionary_bytes));
    }

    /// Scenario: an OTAP logs payload contains no Arrow record batches.
    /// Guarantees: both its logical and retained byte counts are zero.
    #[test]
    fn empty_otap_arrow_records_have_zero_byte_sizes() {
        let records = OtapArrowRecords::Logs(Logs::default());

        assert_eq!(records.logical_arrow_bytes().unwrap(), 0);
        assert_eq!(records.retained_memory_bytes(), 0);
    }

    #[test]
    fn all_none_payload_slots_are_skipped() {
        let records = OtapArrowRecords::Logs(Logs::default());

        for payload_type in records.allowed_payload_types() {
            assert_eq!(records.get(*payload_type), None);
        }
        assert_eq!(records.retained_memory_bytes(), 0);
    }

    /// Scenario: a ListArray slice selects one list while retaining its original child array.
    /// Guarantees: logical sizing counts the terminal offset and the full retained child array.
    #[test]
    fn logical_bytes_follow_arrow_list_child_semantics() {
        let array = ListArray::from_iter_primitive::<Int32Type, _, _>(vec![
            Some(vec![Some(0), Some(1), Some(2)]),
            Some(vec![Some(3), Some(4), Some(5)]),
            Some(vec![Some(6), Some(7)]),
        ]);
        let data_type = array.data_type().clone();
        let slice = array.slice(1, 1);
        let batch = batch_with_columns(vec![("list", data_type, Arc::new(slice) as ArrayRef)]);
        let list_offset_bytes = (batch.num_rows() + 1) * size_of::<i32>();
        let child_bytes = size_of::<[i32; 8]>();

        assert_eq!(logical_bytes(&batch), list_offset_bytes + child_bytes);
    }

    /// Scenario: a small equivalent logs payload is represented as OTLP bytes and OTAP arrays.
    /// Guarantees: protobuf has less representation overhead for this small payload.
    #[test]
    fn small_equivalent_logs_are_smaller_as_otlp() {
        let message = OtlpProtoMessage::Logs(logs_with_full_resource_and_scope());
        let otlp_bytes = otlp_message_to_bytes(&message).num_bytes();
        let otap = otlp_to_otap(&message);
        let otap_bytes = otap.logical_arrow_bytes().unwrap();
        let retained_bytes = otap.retained_memory_bytes();

        assert!(
            otlp_bytes < otap_bytes,
            "expected small OTLP payload ({otlp_bytes}) to be smaller than OTAP ({otap_bytes})"
        );
        assert!(
            otap_bytes < retained_bytes,
            "expected OTAP logical bytes ({otap_bytes}) to be smaller than retained bytes ({retained_bytes})"
        );
    }

    /// Scenario: many equivalent logs repeat the same body, event, and attributes.
    /// Guarantees: OTAP columnar and dictionary representations reduce logical bytes.
    #[test]
    fn large_repetitive_equivalent_logs_are_smaller_as_otap() {
        let message = OtlpProtoMessage::Logs(repeated_logs(1_000));
        let otlp_bytes = otlp_message_to_bytes(&message).num_bytes();
        let otap = otlp_to_otap(&message);
        let otap_bytes = otap.logical_arrow_bytes().unwrap();
        let retained_bytes = otap.retained_memory_bytes();

        assert!(
            otap_bytes < otlp_bytes,
            "expected repetitive OTAP payload ({otap_bytes}) to be smaller than OTLP ({otlp_bytes})"
        );
        assert!(
            otap_bytes < retained_bytes,
            "expected OTAP logical bytes ({otap_bytes}) to be smaller than retained bytes ({retained_bytes})"
        );
    }
}

/// Parity between [`array_slice_memory_size`] and arrow's
/// `to_data().get_slice_memory_size()`.
#[cfg(test)]
mod slice_size_parity_tests {
    use std::io::Cursor;
    use std::sync::Arc;

    use arrow::array::{
        Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
        Int32Array, LargeListArray, LargeStringArray, ListArray, MapBuilder, NullArray,
        RecordBatch, StringArray, StringBuilder, StructArray, UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::buffer::NullBuffer;
    use arrow::datatypes::{DataType, Field, Fields, Int32Type, UInt8Type, UInt16Type};
    use arrow_ipc::reader::StreamReader;
    use arrow_ipc::writer::StreamWriter;

    use super::{array_slice_memory_size, record_batch_logical_bytes};
    use crate::otap::OtapArrowRecords;
    use crate::proto::OtlpProtoMessage;
    use crate::testing::fixtures::{self, DataGenerator, LogsConfig, MetricsConfig, TracesConfig};
    use crate::testing::round_trip::otlp_to_otap;

    /// Scenario: arrays of every type OTAP produces, plus fallback types, each
    /// whole and sliced at several offsets.
    /// Guarantees: the allocation-free size equals arrow's `ArrayData` size.
    #[test]
    fn targeted_arrays_match_arrow_slice_memory_size() {
        for array in targeted_arrays() {
            assert_parity_with_slices(array.as_ref());
        }
    }

    /// Scenario: every column of OTAP logs, traces and metrics batches, as
    /// encoded and after an IPC round trip (the receiver's representation).
    /// Guarantees: column and batch sizes equal the `to_data()` implementation.
    #[test]
    fn otap_batches_match_arrow_slice_memory_size() {
        for records in otap_fixture_records() {
            for payload_type in records.allowed_payload_types() {
                let Some(batch) = records.get(*payload_type) else {
                    continue;
                };
                for batch in [batch.clone(), ipc_round_trip(batch)] {
                    assert_batch_parity(&batch);
                    let rows = batch.num_rows();
                    assert_batch_parity(&batch.slice(rows / 3, rows - rows / 3));
                    assert_batch_parity(&batch.slice(0, rows / 2));
                }
            }
        }
    }

    fn assert_batch_parity(batch: &RecordBatch) {
        assert_eq!(
            record_batch_logical_bytes(batch).unwrap(),
            batch
                .columns()
                .iter()
                .map(|column| column.to_data().get_slice_memory_size().unwrap())
                .sum::<usize>(),
            "schema {:?}",
            batch.schema()
        );
        for column in batch.columns() {
            assert_parity_with_slices(column.as_ref());
        }
    }

    fn assert_parity_with_slices(array: &dyn Array) {
        assert_parity(array);
        let len = array.len();
        let mut slices = vec![(0, len / 2), (len / 3, len - len / 3), (len / 2, 0)];
        if len > 0 {
            slices.push((len - 1, 1));
        }
        for (offset, slice_len) in slices {
            assert_parity(array.slice(offset, slice_len).as_ref());
        }
    }

    fn assert_parity(array: &dyn Array) {
        assert_eq!(
            array_slice_memory_size(array).unwrap(),
            array.to_data().get_slice_memory_size().unwrap(),
            "data type {} len {} offset {}",
            array.data_type(),
            array.len(),
            array.offset()
        );
    }

    fn targeted_arrays() -> Vec<ArrayRef> {
        let utf8_values: ArrayRef = Arc::new(StringArray::from(vec!["alpha", "beta", "gamma"]));
        let binary_values: ArrayRef =
            Arc::new(BinaryArray::from(vec![b"ab".as_ref(), b"cde".as_ref()]));
        let fsb_values: ArrayRef = Arc::new(
            FixedSizeBinaryArray::try_from_iter([[1u8; 16], [2u8; 16]].into_iter()).unwrap(),
        );

        let dict_u8_utf8: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
            UInt8Array::from(vec![Some(0), None, Some(2), Some(1), Some(0)]),
            Arc::clone(&utf8_values),
        ));
        let dict_u16_binary: ArrayRef = Arc::new(DictionaryArray::<UInt16Type>::new(
            UInt16Array::from(vec![1, 0, 1, 1]),
            binary_values,
        ));
        let dict_u8_fsb: ArrayRef = Arc::new(DictionaryArray::<UInt8Type>::new(
            UInt8Array::from(vec![0, 1, 1]),
            fsb_values,
        ));

        let list: ArrayRef = Arc::new(ListArray::from_iter_primitive::<Int32Type, _, _>(vec![
            Some(vec![Some(1), None, Some(3)]),
            None,
            Some(vec![]),
            Some(vec![Some(4), Some(5)]),
        ]));
        let large_list: ArrayRef =
            Arc::new(LargeListArray::from_iter_primitive::<Int32Type, _, _>(
                vec![Some(vec![Some(1)]), Some(vec![Some(2), Some(3)])],
            ));

        let struct_fields = Fields::from(vec![
            Field::new("id", DataType::UInt16, true),
            Field::new(
                "name",
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ),
            Field::new("list", list.data_type().clone(), true),
        ]);
        let nested_struct: ArrayRef = Arc::new(StructArray::new(
            struct_fields.clone(),
            vec![
                Arc::new(UInt16Array::from(vec![Some(1), None, Some(3), Some(4)])),
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0, 1, 2, 0]),
                    utf8_values,
                )),
                Arc::clone(&list),
            ],
            Some(NullBuffer::from(vec![true, false, true, true])),
        ));
        let outer_struct: ArrayRef = Arc::new(StructArray::new(
            Fields::from(vec![
                Field::new("inner", nested_struct.data_type().clone(), true),
                Field::new("flag", DataType::Boolean, true),
            ]),
            vec![
                Arc::clone(&nested_struct),
                Arc::new(BooleanArray::from(vec![
                    Some(true),
                    None,
                    Some(false),
                    Some(true),
                ])),
            ],
            None,
        ));

        let mut map_builder = MapBuilder::new(None, StringBuilder::new(), StringBuilder::new());
        map_builder.keys().append_value("k");
        map_builder.values().append_value("v");
        map_builder.append(true).unwrap();
        map_builder.append(false).unwrap();
        let map: ArrayRef = Arc::new(map_builder.finish());

        vec![
            Arc::new(UInt32Array::from(vec![1, 2, 3, 4, 5])),
            Arc::new(Int32Array::from(vec![Some(1), None, Some(3)])),
            Arc::new(UInt32Array::from(Vec::<u32>::new())),
            Arc::new(BooleanArray::from(vec![Some(true), None, Some(false)])),
            Arc::new(StringArray::from(vec![
                Some("a"),
                None,
                Some("ccc"),
                Some(""),
            ])),
            Arc::new(StringArray::from(Vec::<&str>::new())),
            Arc::new(LargeStringArray::from(vec!["x", "yy", "zzz"])),
            Arc::new(BinaryArray::from(vec![
                Some(b"a".as_ref()),
                None,
                Some(b"bc".as_ref()),
            ])),
            Arc::new(
                FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                    [Some([1u8; 8]), None, Some([3u8; 8])].into_iter(),
                    8,
                )
                .unwrap(),
            ),
            dict_u8_utf8,
            dict_u16_binary,
            dict_u8_fsb,
            list,
            large_list,
            nested_struct,
            outer_struct,
            map,
            Arc::new(NullArray::new(5)),
        ]
    }

    fn otap_fixture_records() -> Vec<OtapArrowRecords> {
        let mut messages: Vec<OtlpProtoMessage> = vec![
            fixtures::logs_with_full_resource_and_scope().into(),
            fixtures::logs_multiple_resources_mixed_content().into(),
            fixtures::traces_with_full_resource_and_scope().into(),
            fixtures::traces_multiple_resources_mixed_content().into(),
            fixtures::metrics_sum_with_full_resource_and_scope().into(),
            fixtures::metrics_multiple_sums_no_resource().into(),
        ];

        let mut logs = DataGenerator::with_logs_config(
            LogsConfig::new(8)
                .with_resources(2)
                .with_scopes_per_resource(2)
                .with_resource_attrs(4)
                .with_scope_attrs(3)
                .with_log_attrs(3),
        );
        messages.push(logs.generate_logs_from_config().into());

        let mut traces = DataGenerator::with_traces_config(
            TracesConfig::new(8)
                .with_resources(2)
                .with_scopes_per_resource(2)
                .with_resource_attrs(4)
                .with_scope_attrs(3)
                .with_span_attrs(3),
        );
        messages.push(traces.generate_traces_from_config().into());

        let mut metrics = DataGenerator::with_metrics_config(
            MetricsConfig::new()
                .with_gauges(vec![4])
                .with_sums(vec![3])
                .with_histograms(vec![4])
                .with_summaries(vec![2])
                .with_resource_attrs(4)
                .with_scope_attrs(3)
                .with_metric_attrs(3),
        );
        messages.push(metrics.generate_metrics_from_config().into());

        messages.iter().map(otlp_to_otap).collect()
    }

    fn ipc_round_trip(batch: &RecordBatch) -> RecordBatch {
        let mut buf = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buf, &batch.schema()).unwrap();
            writer.write(batch).unwrap();
            writer.finish().unwrap();
        }
        let mut reader = StreamReader::try_new(Cursor::new(buf), None).unwrap();
        reader.next().unwrap().unwrap()
    }
}
