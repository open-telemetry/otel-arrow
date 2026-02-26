// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for `sort_by_parent_then_id`.

use std::sync::Arc;

use arrow::array::{ArrayRef, DictionaryArray, PrimitiveArray, RecordBatch, UInt16Array, UInt32Array};
use arrow::datatypes::{DataType, Field, Schema, UInt16Type, UInt8Type};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use otap_df_pdata::otap::transform::util::sort_by_parent_then_id;

const NUM_ROWS: usize = 1_000;
const SEED: u64 = 42;

// ---------------------------------------------------------------------------
// Data generation helpers
// ---------------------------------------------------------------------------

fn random_u16_array(rng: &mut StdRng, n: usize) -> UInt16Array {
    PrimitiveArray::from_iter_values((0..n).map(|_| rng.random::<u16>()))
}

fn sorted_u16_array(n: usize) -> UInt16Array {
    PrimitiveArray::from_iter_values((0..n).map(|i| i as u16))
}

fn random_u32_array(rng: &mut StdRng, n: usize) -> UInt32Array {
    PrimitiveArray::from_iter_values((0..n).map(|_| rng.random::<u32>()))
}

fn random_u16_parent_id(rng: &mut StdRng, n: usize) -> UInt16Array {
    PrimitiveArray::from_iter_values((0..n).map(|_| rng.random::<u16>()))
}

fn random_u32_parent_id(rng: &mut StdRng, n: usize) -> UInt32Array {
    PrimitiveArray::from_iter_values((0..n).map(|_| rng.random::<u32>()))
}

/// Build a Dict<UInt8, UInt32> array with `n` rows, `n_values` distinct values.
fn random_dict_u8_u32(rng: &mut StdRng, n: usize, n_values: usize) -> ArrayRef {
    let values: Vec<u32> = (0..n_values).map(|_| rng.random::<u32>()).collect();
    let keys: Vec<u8> = (0..n).map(|_| rng.random_range(0..n_values as u8)).collect();
    let keys_arr = PrimitiveArray::<UInt8Type>::from_iter_values(keys);
    let values_arr = Arc::new(UInt32Array::from(values));
    Arc::new(DictionaryArray::new(keys_arr, values_arr))
}

/// Build a Dict<UInt16, UInt32> array with `n` rows, `n_values` distinct values.
fn random_dict_u16_u32(rng: &mut StdRng, n: usize, n_values: usize) -> ArrayRef {
    let values: Vec<u32> = (0..n_values).map(|_| rng.random::<u32>()).collect();
    let keys: Vec<u16> = (0..n).map(|_| rng.random_range(0..n_values as u16)).collect();
    let keys_arr = PrimitiveArray::<UInt16Type>::from_iter_values(keys);
    let values_arr = Arc::new(UInt32Array::from(values));
    Arc::new(DictionaryArray::new(keys_arr, values_arr))
}

fn make_batch(fields: Vec<(&str, DataType, ArrayRef)>) -> RecordBatch {
    let schema = Arc::new(Schema::new(
        fields
            .iter()
            .map(|(name, dt, _)| Field::new(*name, dt.clone(), true))
            .collect::<Vec<_>>(),
    ));
    let columns: Vec<ArrayRef> = fields.into_iter().map(|(_, _, arr)| arr).collect();
    RecordBatch::try_new(schema, columns).unwrap()
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_by_parent_then_id");
    let mut rng = StdRng::seed_from_u64(SEED);

    // 1) u16 id only - random order
    {
        let id_col = Arc::new(random_u16_array(&mut rng, NUM_ROWS)) as ArrayRef;
        let batch = make_batch(vec![("id", DataType::UInt16, id_col)]);
        let _ = group.bench_with_input(
            BenchmarkId::new("u16_id_only", "random"),
            &batch,
            |b, batch| {
                b.iter(|| sort_by_parent_then_id(batch.clone()).unwrap());
            },
        );
    }

    // 2) u16 id only - already sorted
    {
        let id_col = Arc::new(sorted_u16_array(NUM_ROWS)) as ArrayRef;
        let batch = make_batch(vec![("id", DataType::UInt16, id_col)]);
        let _ = group.bench_with_input(
            BenchmarkId::new("u16_id_only", "sorted"),
            &batch,
            |b, batch| {
                b.iter(|| sort_by_parent_then_id(batch.clone()).unwrap());
            },
        );
    }

    // 3) u16 parent_id + u16 id - random (native)
    {
        let pid = Arc::new(random_u16_parent_id(&mut rng, NUM_ROWS)) as ArrayRef;
        let id_col = Arc::new(random_u16_array(&mut rng, NUM_ROWS)) as ArrayRef;
        let batch = make_batch(vec![
            ("parent_id", DataType::UInt16, pid),
            ("id", DataType::UInt16, id_col),
        ]);
        let _ = group.bench_with_input(
            BenchmarkId::new("u16_pid_u16_id_native", "random"),
            &batch,
            |b, batch| {
                b.iter(|| sort_by_parent_then_id(batch.clone()).unwrap());
            },
        );
    }

    // 4) u32 id + u16 parent_id - random (native)
    {
        let pid = Arc::new(random_u16_parent_id(&mut rng, NUM_ROWS)) as ArrayRef;
        let id_col = Arc::new(random_u32_array(&mut rng, NUM_ROWS)) as ArrayRef;
        let batch = make_batch(vec![
            ("parent_id", DataType::UInt16, pid),
            ("id", DataType::UInt32, id_col),
        ]);
        let _ = group.bench_with_input(
            BenchmarkId::new("u16_pid_u32_id_native", "random"),
            &batch,
            |b, batch| {
                b.iter(|| sort_by_parent_then_id(batch.clone()).unwrap());
            },
        );
    }

    // 5) u32 id + u32 parent_id - native
    {
        let pid = Arc::new(random_u32_parent_id(&mut rng, NUM_ROWS)) as ArrayRef;
        let id_col = Arc::new(random_u32_array(&mut rng, NUM_ROWS)) as ArrayRef;
        let batch = make_batch(vec![
            ("parent_id", DataType::UInt32, pid),
            ("id", DataType::UInt32, id_col),
        ]);
        let _ = group.bench_with_input(
            BenchmarkId::new("u32_pid_u32_id_native", "random"),
            &batch,
            |b, batch| {
                b.iter(|| sort_by_parent_then_id(batch.clone()).unwrap());
            },
        );
    }

    // 6) u32 id + Dict<UInt8, UInt32> parent_id
    {
        let pid = random_dict_u8_u32(&mut rng, NUM_ROWS, 50);
        let id_col = Arc::new(random_u32_array(&mut rng, NUM_ROWS)) as ArrayRef;
        let dict_dt = DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32));
        let batch = make_batch(vec![
            ("parent_id", dict_dt, pid),
            ("id", DataType::UInt32, id_col),
        ]);
        let _ = group.bench_with_input(
            BenchmarkId::new("u32_id_dict_u8_u32_pid", "random"),
            &batch,
            |b, batch| {
                b.iter(|| sort_by_parent_then_id(batch.clone()).unwrap());
            },
        );
    }

    // 7) u32 id + Dict<UInt16, UInt32> parent_id
    {
        let pid = random_dict_u16_u32(&mut rng, NUM_ROWS, 200);
        let id_col = Arc::new(random_u32_array(&mut rng, NUM_ROWS)) as ArrayRef;
        let dict_dt = DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::UInt32));
        let batch = make_batch(vec![
            ("parent_id", dict_dt, pid),
            ("id", DataType::UInt32, id_col),
        ]);
        let _ = group.bench_with_input(
            BenchmarkId::new("u32_id_dict_u16_u32_pid", "random"),
            &batch,
            |b, batch| {
                b.iter(|| sort_by_parent_then_id(batch.clone()).unwrap());
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_sort);
criterion_main!(benches);
