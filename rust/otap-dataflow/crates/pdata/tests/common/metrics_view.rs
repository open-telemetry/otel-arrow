// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Synthetic numeric metrics shared by the view regression tests and benchmark.

use std::sync::Arc;

use arrow::{
    array::{
        ArrayRef, BooleanArray, Int32Array, Int64Array, PrimitiveDictionaryBuilder, RecordBatch,
        StringArray, StructArray, UInt8Array, UInt16Array, UInt32Array,
    },
    datatypes::{DataType, Field, UInt16Type, UInt32Type},
};
use otel_arrow_dfe_pdata::{
    otap::OtapArrowRecords, proto::opentelemetry::arrow::v1::ArrowPayloadType, schema::consts,
};

pub const KEYS: [&str; 8] = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

#[derive(Clone, Copy, Debug)]
pub enum Layout {
    Grouped,
    Scattered,
    Mixed,
}

pub fn records(points: usize, layout: Layout, dictionary: bool) -> OtapArrowRecords {
    let structure = |ids| -> ArrayRef {
        Arc::new(StructArray::from(vec![(
            Arc::new(Field::new(consts::ID, DataType::UInt16, false)),
            Arc::new(UInt16Array::from(ids)) as ArrayRef,
        )]))
    };
    let root = RecordBatch::try_from_iter([
        (
            consts::ID,
            Arc::new(UInt16Array::from(vec![0, 1])) as ArrayRef,
        ),
        (
            consts::NAME,
            Arc::new(StringArray::from(vec!["gauge", "counter"])),
        ),
        (consts::METRIC_TYPE, Arc::new(UInt8Array::from(vec![1, 2]))),
        (
            consts::IS_MONOTONIC,
            Arc::new(BooleanArray::from(vec![false, true])),
        ),
        (
            consts::AGGREGATION_TEMPORALITY,
            Arc::new(Int32Array::from(vec![0, 2])),
        ),
        (consts::RESOURCE, structure(vec![5u16, 9])),
        (consts::SCOPE, structure(vec![9u16, 3])),
    ])
    .expect("valid synthetic metrics fixture");
    // Sparse, descending IDs deliberately differ from row offsets, including ID 0.
    let ids: Vec<u32> = (0..points)
        .map(|row| u32::try_from(points - row - 1).expect("valid synthetic metrics fixture") * 37)
        .collect();
    let numbers = RecordBatch::try_from_iter([
        (
            consts::ID,
            Arc::new(UInt32Array::from(ids.clone())) as ArrayRef,
        ),
        (
            consts::PARENT_ID,
            Arc::new(UInt16Array::from_iter_values(
                (0..points).map(|row| (row % 2) as u16),
            )),
        ),
        (
            consts::INT_VALUE,
            Arc::new(Int64Array::from_iter_values(
                ids.iter().map(|&id| i64::from(id)),
            )),
        ),
    ])
    .expect("valid synthetic metrics fixture");
    let mut parents = Vec::with_capacity(points * KEYS.len());
    let mut keys = Vec::with_capacity(parents.capacity());
    let mut values = Vec::with_capacity(parents.capacity());
    let mut append = |row: usize, key: usize| {
        parents.push(ids[row]);
        keys.push(KEYS[key]);
        values.push(i64::from(ids[row]) * 16 + key as i64);
    };
    match layout {
        Layout::Grouped => {
            for row in 0..points {
                for key in 0..KEYS.len() {
                    append(row, key);
                }
            }
        }
        Layout::Scattered => {
            for key in 0..KEYS.len() {
                for row in 0..points {
                    append(row, key);
                }
            }
        }
        Layout::Mixed => {
            for row in 0..points {
                for key in 0..4 {
                    append(row, key);
                }
            }
            for key in 4..KEYS.len() {
                for row in 0..points {
                    append(row, key);
                }
            }
        }
    }
    let parents: ArrayRef = if dictionary {
        let mut builder = PrimitiveDictionaryBuilder::<UInt16Type, UInt32Type>::new();
        for parent in parents {
            let _ = builder
                .append(parent)
                .expect("valid synthetic metrics fixture");
        }
        Arc::new(builder.finish())
    } else {
        Arc::new(UInt32Array::from(parents))
    };
    let attributes = RecordBatch::try_from_iter([
        (consts::PARENT_ID, parents),
        (consts::ATTRIBUTE_KEY, Arc::new(StringArray::from(keys))),
        (
            consts::ATTRIBUTE_TYPE,
            Arc::new(UInt8Array::from(vec![2; values.len()])),
        ),
        (consts::ATTRIBUTE_INT, Arc::new(Int64Array::from(values))),
    ])
    .expect("valid synthetic metrics fixture");
    let mut records = OtapArrowRecords::Metrics(Default::default());
    records
        .set(ArrowPayloadType::UnivariateMetrics, root)
        .expect("valid synthetic metrics fixture");
    records
        .set(ArrowPayloadType::NumberDataPoints, numbers)
        .expect("valid synthetic metrics fixture");
    records
        .set(ArrowPayloadType::NumberDpAttrs, attributes)
        .expect("valid synthetic metrics fixture");
    records
}

// Shared with the timing benchmark, which keeps System as its allocator and
// intentionally does not use this measurement support.
#[allow(dead_code)]
pub mod allocations {
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
    };

    thread_local! {
        // Counts successful requests and requested bytes, including full realloc sizes.
        // These are not measurements of retained heap or process memory.
        static COUNTS: Cell<Option<(u64, u64)>> = const { Cell::new(None) };
    }

    pub struct CountingAllocator;

    fn record(size: usize) {
        let _ = COUNTS.try_with(|counts| {
            if let Some((calls, bytes)) = counts.get() {
                counts.set(Some((calls + 1, bytes + size as u64)));
            }
        });
    }

    // SAFETY: Pointer, ownership and layout contracts are delegated unchanged to
    // System. Successful requests update only allocation-free thread-local cells.
    #[allow(unsafe_code)] // GlobalAlloc requires unsafe; confined to measurement targets.
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // SAFETY: Forward the caller's valid layout unchanged.
            let pointer = unsafe { System.alloc(layout) };
            if !pointer.is_null() {
                record(layout.size());
            }
            pointer
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            // SAFETY: Forward the caller's valid layout unchanged.
            let pointer = unsafe { System.alloc_zeroed(layout) };
            if !pointer.is_null() {
                record(layout.size());
            }
            pointer
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            // SAFETY: Forward the original system pointer, layout and valid new size.
            let next = unsafe { System.realloc(pointer, layout, size) };
            if !next.is_null() {
                record(size);
            }
            next
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            // SAFETY: Forward the original system pointer and layout unchanged.
            unsafe { System.dealloc(pointer, layout) };
        }
    }

    struct MeasurementScope;

    impl Drop for MeasurementScope {
        fn drop(&mut self) {
            let _ = COUNTS.try_with(|counts| counts.set(None));
        }
    }

    /// Measure allocation requests made by a synchronous operation on this thread.
    pub fn measure<T>(operation: impl FnOnce() -> T) -> (T, u64, u64) {
        COUNTS.with(|counts| counts.set(Some((0, 0))));
        let scope = MeasurementScope;
        let result = operation();
        let (calls, bytes) = COUNTS.with(|counts| {
            counts
                .replace(None)
                .expect("allocation measurement is active")
        });
        drop(scope);
        (result, calls, bytes)
    }
}
