// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for concatenating OTAP batches the way the batch processor
//! does: reindex the ID columns so they do not collide, then concatenate.
//!
//! Scenarios (`concatenate/{size}items/{shape}/{scenario}/{signal}`):
//!
//! - `contiguous`: freshly encoded batches with dense, sorted IDs. Reindexing
//!   can shift each batch by a uniform offset.
//! - `presplit`: each batch is first split into slices of about a third of
//!   its rows, as the batch processor does before concatenating. Slices share
//!   their dictionary values arrays. Metrics split at metric boundaries, so
//!   this scenario only differs from `contiguous` at the 3r2s shape.
//! - `gapped`: every ID is doubled so the ID ranges no longer fit the u16 ID
//!   budget when uniformly offset, which forces the compaction path. This only
//!   triggers for logs and traces at the largest shape, so it is only run
//!   there.

use std::num::NonZeroU32;
use std::ops::Mul;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, AsArray, DictionaryArray, PrimitiveArray, RecordBatch, StructArray,
};
use arrow::buffer::ScalarBuffer;
use arrow::datatypes::{ArrowPrimitiveType, DataType, UInt16Type, UInt32Type};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use otel_arrow_dfe_pdata::otap::transform::concatenate::{ConcatOptions, concatenate};
use otel_arrow_dfe_pdata::otap::transform::split::split;
use otel_arrow_dfe_pdata::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
use otel_arrow_dfe_pdata::schema::consts::{ID, PARENT_ID};
use otel_arrow_dfe_pdata::testing::fixtures::{
    DataGenerator, LogsConfig, MetricsConfig, TracesConfig,
};
use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;

const NUM_BATCHES: usize = 10;
const BATCH_SIZES: &[usize] = &[100, 1000];

criterion_group!(benches, bench_all);
criterion_main!(benches);

/// (num_resources, scopes_per_resource, label)
/// Shapes must stay within u16::MAX total items across NUM_BATCHES batches.
/// Each batch generates num_resources * scopes_per_resource * items_per_scope
/// items. Combinations that would overflow u16::MAX are skipped at runtime.
const INPUT_SHAPES: &[(usize, usize, &str)] = &[(1, 1, "1r1s"), (3, 2, "3r2s")];

/// The only size/shape where `gapped` inputs overflow the ID budget.
const GAPPED_SIZE: usize = 1000;
const GAPPED_SHAPE: &str = "3r2s";

fn bench_all(c: &mut Criterion) {
    for &size in BATCH_SIZES {
        for &(num_res, scopes, shape_label) in INPUT_SHAPES {
            // Skip shapes that would overflow u16::MAX total items.
            let total_items = size * num_res * scopes * NUM_BATCHES;
            if total_items > 65000 {
                continue;
            }

            let metrics = generate_metrics(size, num_res, scopes, 10, 5, 3);
            let logs = generate_logs(size, num_res, scopes, 10, 5, 3);
            let traces = generate_traces(size, num_res, scopes, 10, 5, 3);

            bench_group(
                c,
                &format!("concatenate/{size}items/{shape_label}/contiguous"),
                |group| {
                    bench_concatenate(group, "metrics", &metrics);
                    bench_concatenate(group, "logs", &logs);
                    bench_concatenate(group, "traces", &traces);
                },
            );

            bench_group(
                c,
                &format!("concatenate/{size}items/{shape_label}/presplit"),
                |group| {
                    // Single-metric inputs are not split further, so presplit
                    // metrics only differs from contiguous at the 3r2s shape.
                    if num_res * scopes > 1 {
                        bench_concatenate(group, "metrics", &presplit(&metrics));
                    }
                    bench_concatenate(group, "logs", &presplit(&logs));
                    bench_concatenate(group, "traces", &presplit(&traces));
                },
            );

            if size == GAPPED_SIZE && shape_label == GAPPED_SHAPE {
                let logs_gapped: Vec<_> = logs.iter().map(introduce_gaps).collect();
                let traces_gapped: Vec<_> = traces.iter().map(introduce_gaps).collect();
                bench_group(
                    c,
                    &format!("concatenate/{size}items/{shape_label}/gapped"),
                    |group| {
                        bench_concatenate(group, "logs", &logs_gapped);
                        bench_concatenate(group, "traces", &traces_gapped);
                    },
                );
            }
        }
    }
}

/// Split every input into slices of roughly a third of its root rows. This mimics
/// the batch processor, where concatenation inputs are zero-copy slices whose
/// dictionary values arrays are shared with other slices.
fn presplit<const N: usize>(batches: &[[Option<RecordBatch>; N]]) -> Vec<[Option<RecordBatch>; N]> {
    // Root payload is at index 2 for every signal.
    let items = batches[0][2].as_ref().map_or(3, RecordBatch::num_rows);
    let max = NonZeroU32::new((items / 3).max(1) as u32).expect("non-zero");
    let mut batches = batches.to_vec();
    split::<N>(&mut batches, max).expect("split failed")
}

fn bench_group(
    c: &mut Criterion,
    name: &str,
    f: impl FnOnce(&mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>),
) {
    let mut group = c.benchmark_group(name);
    f(&mut group);
    group.finish();
}

/// Time reindexing and concatenation together, as the batch processor runs
/// them.
fn bench_concatenate<const N: usize>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    signal_name: &str,
    data: &[[Option<RecordBatch>; N]],
) {
    let _ = group.bench_with_input(BenchmarkId::from_parameter(signal_name), data, |b, data| {
        b.iter_batched(
            || data.to_vec(),
            |mut batches| {
                let _ = concatenate::<N>(&mut batches, ConcatOptions::reindex())
                    .expect("concat failed");
            },
            BatchSize::SmallInput,
        )
    });
}

fn generate_metrics(
    points_per_gauge: usize,
    num_resources: usize,
    scopes_per_resource: usize,
    resource_attrs: usize,
    scope_attrs: usize,
    metric_attrs: usize,
) -> Vec<[Option<RecordBatch>; Metrics::COUNT]> {
    let mut datagen = DataGenerator::with_metrics_config(
        MetricsConfig::new()
            .with_gauges(vec![points_per_gauge])
            .with_resources(num_resources)
            .with_scopes_per_resource(scopes_per_resource)
            .with_resource_attrs(resource_attrs)
            .with_scope_attrs(scope_attrs)
            .with_metric_attrs(metric_attrs),
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

fn generate_logs(
    logs_per_scope: usize,
    num_resources: usize,
    scopes_per_resource: usize,
    resource_attrs: usize,
    scope_attrs: usize,
    log_attrs: usize,
) -> Vec<[Option<RecordBatch>; Logs::COUNT]> {
    let mut datagen = DataGenerator::with_logs_config(
        LogsConfig::new(logs_per_scope)
            .with_resources(num_resources)
            .with_scopes_per_resource(scopes_per_resource)
            .with_resource_attrs(resource_attrs)
            .with_scope_attrs(scope_attrs)
            .with_log_attrs(log_attrs),
    );
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

fn generate_traces(
    spans_per_scope: usize,
    num_resources: usize,
    scopes_per_resource: usize,
    resource_attrs: usize,
    scope_attrs: usize,
    span_attrs: usize,
) -> Vec<[Option<RecordBatch>; Traces::COUNT]> {
    let mut datagen = DataGenerator::with_traces_config(
        TracesConfig::new(spans_per_scope)
            .with_resources(num_resources)
            .with_scopes_per_resource(scopes_per_resource)
            .with_resource_attrs(resource_attrs)
            .with_scope_attrs(scope_attrs)
            .with_span_attrs(span_attrs),
    );
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
    RecordBatch::try_new(schema, columns).expect("create record batch")
}

fn double_struct_ids(arr: &StructArray) -> StructArray {
    let fields = arr.fields();
    let mut columns: Vec<ArrayRef> = arr.columns().to_vec();
    for (i, field) in fields.iter().enumerate() {
        if field.name() == ID || field.name() == PARENT_ID {
            columns[i] = double_array(&columns[i]);
        }
    }
    StructArray::try_new(fields.clone(), columns, arr.nulls().cloned()).expect("create struct")
}

fn double_primitive<T>(arr: &PrimitiveArray<T>) -> PrimitiveArray<T>
where
    T: ArrowPrimitiveType,
    T::Native: Mul<Output = T::Native> + From<u8>,
{
    let two = T::Native::from(2u8);
    let doubled: Vec<T::Native> = arr.values().iter().map(|x| *x * two).collect();
    PrimitiveArray::<T>::new(ScalarBuffer::from(doubled), None)
}

fn double_array(arr: &ArrayRef) -> ArrayRef {
    match arr.data_type() {
        DataType::UInt16 => Arc::new(double_primitive(arr.as_primitive::<UInt16Type>())),
        DataType::UInt32 => Arc::new(double_primitive(arr.as_primitive::<UInt32Type>())),
        DataType::Dictionary(key_type, value_type) => {
            match (key_type.as_ref(), value_type.as_ref()) {
                (_, DataType::UInt16) => double_dict_values::<UInt16Type>(arr, key_type),
                (_, DataType::UInt32) => double_dict_values::<UInt32Type>(arr, key_type),
                _ => panic!("Unexpected id column type"),
            }
        }
        _ => panic!("Unexpected id column type"),
    }
}

fn double_dict_values<V>(arr: &ArrayRef, key_type: &DataType) -> ArrayRef
where
    V: ArrowPrimitiveType,
    V::Native: Mul<Output = V::Native> + From<u8>,
{
    match key_type {
        DataType::UInt8 => {
            let dict = arr.as_dictionary::<arrow::datatypes::UInt8Type>();
            let new_vals = double_primitive::<V>(dict.values().as_primitive());
            Arc::new(DictionaryArray::new(
                dict.keys().clone(),
                Arc::new(new_vals),
            ))
        }
        DataType::UInt16 => {
            let dict = arr.as_dictionary::<UInt16Type>();
            let new_vals = double_primitive::<V>(dict.values().as_primitive());
            Arc::new(DictionaryArray::new(
                dict.keys().clone(),
                Arc::new(new_vals),
            ))
        }
        _ => panic!("Unexpected id column type"),
    }
}
