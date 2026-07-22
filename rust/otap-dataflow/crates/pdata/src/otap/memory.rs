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
        Array, ArrayRef, DictionaryArray, RecordBatch, StringArray, UInt8Array, UInt32Array,
        UInt32Builder,
    };
    use arrow::datatypes::{DataType, Field, Schema, UInt8Type};
    use bytes::Bytes;

    use crate::otap::{Logs, OtapArrowRecords};
    use crate::otlp::OtlpProtoBytes;
    use crate::payload::{OtapPayload, OtapPayloadHelpers};
    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

    use super::{CountedAllocations, record_batch_pinned_bytes};

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

    #[test]
    fn empty_otap_arrow_records_have_no_retained_memory() {
        let records = OtapArrowRecords::Logs(Logs::default());

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

    #[test]
    fn payload_retained_memory_bytes_preserves_num_bytes_semantics() {
        let otlp_bytes = OtlpProtoBytes::ExportLogsRequest(Bytes::from_static(b"abc"));
        let otlp_payload = OtapPayload::OtlpBytes(otlp_bytes.clone());
        let arrow_payload = OtapPayload::OtapArrowRecords(OtapArrowRecords::Logs(Logs::default()));

        assert_eq!(otlp_bytes.retained_memory_bytes(), 3);
        assert_eq!(otlp_payload.retained_memory_bytes(), 3);
        assert_eq!(otlp_payload.num_bytes(), Some(3));
        assert_eq!(arrow_payload.retained_memory_bytes(), 0);
        assert_eq!(arrow_payload.num_bytes(), None);

        // Keep the root payload type reachable on an empty OTAP batch without
        // implying there is retained Arrow memory.
        assert_eq!(
            match arrow_payload {
                OtapPayload::OtapArrowRecords(records) => records.root_payload_type(),
                OtapPayload::OtlpBytes(_) => unreachable!(),
            },
            ArrowPayloadType::Logs
        );
    }
}
