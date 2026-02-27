// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for concatenation (reindex + merge) of OTAP record batches.
//!
//! 1000 batches per benchmark. All signal types vary data points per batch
//! (5, 100, 1000).
//!
//! Reindex benchmarks include contiguous (encoder-produced) and gapped
//! (doubled IDs) variants to exercise different code paths.

use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, AsArray, DictionaryArray, PrimitiveArray, RecordBatch, StructArray,
};
use arrow::buffer::ScalarBuffer;
use arrow::datatypes::{DataType, UInt16Type, UInt32Type};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::otap::transform::concatenate::concatenate;
use otap_df_pdata::otap::transform::reindex::{reindex_logs, reindex_metrics, reindex_traces};
use otap_df_pdata::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
use otap_df_pdata::schema::consts::{ID, PARENT_ID};
use otap_df_pdata::testing::fixtures::{DataGenerator, LogsConfig, MetricsConfig, TracesConfig};
use otap_df_pdata::testing::round_trip::otlp_to_otap;

const NUM_BATCHES: usize = 10;
const BATCH_SIZES: &[usize] = &[5, 100, 1000];

criterion_group!(benches, bench_metrics, bench_logs, bench_traces);
criterion_main!(benches);

fn bench_metrics(c: &mut Criterion) {
    for points in BATCH_SIZES {
        let contiguous = generate_metrics(*points);
        let gapped: Vec<_> = contiguous.iter().map(|b| introduce_gaps(b)).collect();

        let mut group = c.benchmark_group(format!("metrics/{points}pts"));
        bench_signal(&mut group, &contiguous, &gapped, |batches| {
            reindex_metrics::<{ Metrics::COUNT }>(batches).expect("reindex failed")
        });
        group.finish();
    }
}

fn bench_logs(c: &mut Criterion) {
    for num_logs in BATCH_SIZES {
        let contiguous = generate_logs(*num_logs);
        let gapped: Vec<_> = contiguous.iter().map(|b| introduce_gaps(b)).collect();

        let mut group = c.benchmark_group(format!("logs/{num_logs}logs"));
        bench_signal(&mut group, &contiguous, &gapped, |batches| {
            reindex_logs::<{ Logs::COUNT }>(batches).expect("reindex failed")
        });
        group.finish();
    }
}

fn bench_traces(c: &mut Criterion) {
    for num_spans in BATCH_SIZES {
        let contiguous = generate_traces(*num_spans);
        let gapped: Vec<_> = contiguous.iter().map(|b| introduce_gaps(b)).collect();

        let mut group = c.benchmark_group(format!("traces/{num_spans}spans"));
        bench_signal(&mut group, &contiguous, &gapped, |batches| {
            reindex_traces::<{ Traces::COUNT }>(batches).expect("reindex failed")
        });
        group.finish();
    }
}

fn bench_signal<const N: usize>(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    contiguous: &[[Option<RecordBatch>; N]],
    gapped: &[[Option<RecordBatch>; N]],
    reindex_fn: impl Fn(&mut Vec<[Option<RecordBatch>; N]>) + Clone,
) {
    let reindex_clone = reindex_fn.clone();

    let _ = group.bench_with_input(
        BenchmarkId::new("reindex", "contiguous"),
        contiguous,
        |b, data| {
            let f = reindex_fn.clone();
            b.iter_batched(
                || data.to_vec(),
                |mut batches| f(&mut batches),
                BatchSize::SmallInput,
            )
        },
    );

    let _ = group.bench_with_input(BenchmarkId::new("reindex", "gapped"), gapped, |b, data| {
        let f = reindex_clone.clone();
        b.iter_batched(
            || data.to_vec(),
            |mut batches| f(&mut batches),
            BatchSize::SmallInput,
        )
    });

    let _ = group.bench_with_input(
        BenchmarkId::new("concatenate", "contiguous"),
        contiguous,
        |b, data| {
            b.iter_batched(
                || data.to_vec(),
                |mut batches| concatenate::<N>(&mut batches).expect("concat failed"),
                BatchSize::SmallInput,
            )
        },
    );

    let _ = group.bench_with_input(
        BenchmarkId::new("concatenate", "gapped"),
        gapped,
        |b, data| {
            b.iter_batched(
                || data.to_vec(),
                |mut batches| concatenate::<N>(&mut batches).expect("concat failed"),
                BatchSize::SmallInput,
            )
        },
    );
}

fn generate_metrics(points_per_gauge: usize) -> Vec<[Option<RecordBatch>; Metrics::COUNT]> {
    let mut datagen = DataGenerator::with_metrics_config(
        MetricsConfig::new().with_gauges(vec![points_per_gauge]),
    );
    (0..NUM_BATCHES)
        .map(|_| {
            let data = datagen.generate_metrics_from_config();
            match otlp_to_otap(&data.into()) {
                OtapArrowRecords::Metrics(m) => m.into_batches(),
                _ => unreachable!(),
            }
        })
        .collect()
}

fn generate_logs(num_logs: usize) -> Vec<[Option<RecordBatch>; Logs::COUNT]> {
    let mut datagen = DataGenerator::with_logs_config(LogsConfig::new(num_logs));
    (0..NUM_BATCHES)
        .map(|_| {
            let data = datagen.generate_logs_from_config();
            match otlp_to_otap(&data.into()) {
                OtapArrowRecords::Logs(l) => l.into_batches(),
                _ => unreachable!(),
            }
        })
        .collect()
}

fn generate_traces(num_spans: usize) -> Vec<[Option<RecordBatch>; Traces::COUNT]> {
    let mut datagen = DataGenerator::with_traces_config(TracesConfig::new(num_spans));
    (0..NUM_BATCHES)
        .map(|_| {
            let data = datagen.generate_traces_from_config();
            match otlp_to_otap(&data.into()) {
                OtapArrowRecords::Traces(t) => t.into_batches(),
                _ => unreachable!(),
            }
        })
        .collect()
}

fn introduce_gaps<const N: usize>(batches: &[Option<RecordBatch>; N]) -> [Option<RecordBatch>; N] {
    std::array::from_fn(|i| batches[i].as_ref().map(double_id_columns))
}

fn double_id_columns(rb: &RecordBatch) -> RecordBatch {
    let schema = rb.schema();
    let mut columns: Vec<ArrayRef> = rb.columns().to_vec();
    for (i, field) in schema.fields().iter().enumerate() {
        match field.name().as_str() {
            ID | PARENT_ID => columns[i] = double_array(&columns[i]),
            _ if matches!(field.data_type(), DataType::Struct(_)) => {
                columns[i] = Arc::new(double_struct_ids(columns[i].as_struct()));
            }
            _ => {}
        }
    }
    RecordBatch::try_new(schema, columns).unwrap()
}

fn double_struct_ids(arr: &StructArray) -> StructArray {
    let fields = arr.fields();
    let mut columns: Vec<ArrayRef> = arr.columns().to_vec();
    for (i, field) in fields.iter().enumerate() {
        if field.name() == ID || field.name() == PARENT_ID {
            columns[i] = double_array(&columns[i]);
        }
    }
    StructArray::try_new(fields.clone(), columns, arr.nulls().cloned()).unwrap()
}

fn double_array(arr: &ArrayRef) -> ArrayRef {
    match arr.data_type() {
        DataType::UInt16 => {
            let v = arr.as_primitive::<UInt16Type>().values();
            let doubled: Vec<u16> = v.iter().map(|x| x * 2).collect();
            Arc::new(PrimitiveArray::<UInt16Type>::new(
                ScalarBuffer::from(doubled),
                None,
            ))
        }
        DataType::UInt32 => {
            let v = arr.as_primitive::<UInt32Type>().values();
            let doubled: Vec<u32> = v.iter().map(|x| x * 2).collect();
            Arc::new(PrimitiveArray::<UInt32Type>::new(
                ScalarBuffer::from(doubled),
                None,
            ))
        }
        DataType::Dictionary(key_type, value_type) => {
            match (key_type.as_ref(), value_type.as_ref()) {
                (_, DataType::UInt16) => double_dict_values::<UInt16Type>(arr, key_type),
                (_, DataType::UInt32) => double_dict_values::<UInt32Type>(arr, key_type),
                _ => arr.clone(),
            }
        }
        _ => arr.clone(),
    }
}

fn double_dict_values<V>(arr: &ArrayRef, key_type: &DataType) -> ArrayRef
where
    V: arrow::datatypes::ArrowPrimitiveType,
    V::Native: std::ops::Mul<Output = V::Native> + From<u8>,
{
    let two = V::Native::from(2u8);
    match key_type {
        DataType::UInt8 => {
            let dict = arr.as_dictionary::<arrow::datatypes::UInt8Type>();
            let vals = dict.values().as_primitive::<V>().values();
            let doubled: Vec<V::Native> = vals.iter().map(|x| *x * two).collect();
            let new_vals = PrimitiveArray::<V>::new(ScalarBuffer::from(doubled), None);
            Arc::new(DictionaryArray::new(
                dict.keys().clone(),
                Arc::new(new_vals),
            ))
        }
        DataType::UInt16 => {
            let dict = arr.as_dictionary::<UInt16Type>();
            let vals = dict.values().as_primitive::<V>().values();
            let doubled: Vec<V::Native> = vals.iter().map(|x| *x * two).collect();
            let new_vals = PrimitiveArray::<V>::new(ScalarBuffer::from(doubled), None);
            Arc::new(DictionaryArray::new(
                dict.keys().clone(),
                Arc::new(new_vals),
            ))
        }
        _ => arr.clone(),
    }
}
