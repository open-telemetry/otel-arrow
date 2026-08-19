// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Retained-memory sizing helpers for OTAP Arrow record batches.
//!
//! These helpers count deduped Arrow-owned buffer capacity retained by record
//! batches. The value is logical retained memory, not process RSS: it excludes
//! allocator, struct, and `Arc` overhead. It is intended for callers that need
//! to account for retained work, while `num_bytes()` remains encoded/wire-size
//! semantics.
//!
//! Buffer allocations are deduped by [`arrow::buffer::Buffer::data_ptr`], which
//! returns the allocation base and ignores slice offsets. This matters because
//! OTAP transforms such as `otap::transform::split` use
//! [`RecordBatch::slice`](arrow::array::RecordBatch::slice), so multiple slices
//! can share the same parent allocation.
//!
//! The accounting uses buffer `capacity()`, not `len()`: a small slice pins the
//! whole parent allocation until it is dropped. One known limitation is that
//! externally owned Arrow buffers report `capacity() == 0`; IPC-decoded OTAP
//! batches are Rust-allocated today, but future zero-copy or mmap ingest would
//! be under-counted by these helpers.
//!
//! This module does not cache sizes inside pdata. `OtapArrowRecords` and its
//! stores are cloneable and mutable through `set()` and `remove()`, so an
//! internal cache would be easy to stale. Consumers that need charge/refund
//! symmetry should compute once when retention starts and store the value with
//! their retained state or ticket.
//!
//! Performance is proportional to the number of arrays and buffers, not to the
//! number of rows or byte values. Each column calls `to_data()`, which performs
//! a small structural clone of `Arc`-backed Arrow metadata and does not copy
//! buffer contents. Each accounting call also creates a fresh `HashSet` for
//! deduping buffers; if this ever shows up in profiles, callers can reuse and
//! clear a [`CountedAllocations`] value across accounting calls.

use std::{collections::HashSet, ptr::NonNull};

use arrow::array::{ArrayData, RecordBatch};
use arrow::error::ArrowError;

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
/// This delegates the definition to [`ArrayData::get_slice_memory_size`].
/// Direct buffers are measured using the array's active offset and length,
/// nested child arrays follow Arrow's own recursive sizing semantics, and
/// shared buffers are counted once per logical array reference.
pub fn record_batch_logical_bytes(batch: &RecordBatch) -> Result<usize, ArrowError> {
    batch.columns().iter().try_fold(0usize, |total, array| {
        let array_bytes = array.to_data().get_slice_memory_size()?;
        total.checked_add(array_bytes).ok_or_else(|| {
            ArrowError::ComputeError(
                "Integer overflow computing logical Arrow byte size".to_string(),
            )
        })
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

#[cfg(test)]
mod tests {
    use std::{mem::size_of, sync::Arc};

    use arrow::array::{
        Array, ArrayRef, DictionaryArray, ListArray, RecordBatch, StringArray, UInt8Array,
        UInt32Array, UInt32Builder,
    };
    use arrow::datatypes::{DataType, Field, Int32Type, Schema, UInt8Type};
    use bytes::Bytes;

    use crate::otap::{Logs, OtapArrowRecords};
    use crate::otlp::OtlpProtoBytes;
    use crate::payload::{OtapPayload, OtapPayloadHelpers, PayloadData};
    use crate::proto::OtlpProtoMessage;
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::testing::fixtures::logs_with_full_resource_and_scope;
    use crate::testing::round_trip::{otlp_message_to_bytes, otlp_to_otap};

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
    /// Guarantees: logical sizing follows Arrow's active-slice accounting.
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
        let utf8_offset_bytes = 2 * size_of::<i32>();
        let utf8_value_bytes = "betagamma".len();

        assert_eq!(
            logical_bytes(&slice),
            primitive_bytes + utf8_offset_bytes + utf8_value_bytes
        );
    }

    /// Scenario: two dictionary columns share one dictionary values array.
    /// Guarantees: logical sizing counts the dictionary for each logical array reference.
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
        let dictionary_bytes = 2 * size_of::<i32>() + "alphabeta".len();

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

    /// Scenario: a serialized OTLP payload is wrapped in the opaque payload type.
    /// Guarantees: OTLP logical and retained byte estimates both equal the protobuf length.
    #[test]
    fn otlp_payload_num_bytes_matches_retained_memory_bytes() {
        let otlp_bytes = OtlpProtoBytes::ExportLogsRequest(Bytes::from_static(b"abc"));
        let otlp_payload: OtapPayload = otlp_bytes.clone().into();

        assert_eq!(otlp_bytes.num_bytes(), otlp_bytes.retained_memory_bytes());
        assert_eq!(
            otlp_payload.num_bytes(),
            Some(otlp_payload.retained_memory_bytes())
        );
    }

    /// Scenario: an empty OTAP logs payload is converted into and out of the opaque payload type.
    /// Guarantees: the conversion preserves the payload's logs root type.
    #[test]
    fn empty_otap_payload_preserves_root_type() {
        let arrow_payload: OtapPayload = OtapArrowRecords::Logs(Logs::default()).into();

        assert_eq!(
            match arrow_payload.into_data() {
                PayloadData::OtapArrowRecords(records) => records.root_payload_type(),
                PayloadData::OtlpBytes(_) => unreachable!(),
            },
            ArrowPayloadType::Logs
        );
    }

    /// Scenario: a ListArray slice selects one list while retaining its original child array.
    /// Guarantees: logical sizing preserves Arrow's recursive child-array semantics.
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
        let list_offset_bytes = size_of::<i32>();
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
