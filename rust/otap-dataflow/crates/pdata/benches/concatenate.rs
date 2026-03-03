// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for concatenation and reindexing.
//!
//! Reindex benchmarks include contiguous, gapped (doubled IDs), and unsorted
//! variants to exercise different code paths.

use std::ops::Mul;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, AsArray, DictionaryArray, PrimitiveArray, RecordBatch, StructArray,
};
use arrow::buffer::ScalarBuffer;
use arrow::datatypes::{ArrowPrimitiveType, DataType, UInt16Type, UInt32Type};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::otap::transform::concatenate::concatenate;
use otap_df_pdata::otap::transform::reindex::reindex;
use otap_df_pdata::otap::transform::reindex_old::reindex as reindex_old;
use otap_df_pdata::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
use otap_df_pdata::schema::consts::{ID, PARENT_ID};
use otap_df_pdata::testing::fixtures::{DataGenerator, LogsConfig, MetricsConfig, TracesConfig};
use otap_df_pdata::testing::round_trip::otlp_to_otap;

const NUM_BATCHES: usize = 10;
const BATCH_SIZES: &[usize] = &[100, 1000];

criterion_group!(benches, bench_all);
criterion_main!(benches);

/// (num_resources, scopes_per_resource, label)
/// Shapes must stay within u16::MAX total items across NUM_BATCHES batches.
/// Each batch generates num_resources * scopes_per_resource * items_per_scope
/// items. With 3r2s and 1000 items: 10 * 3 * 2 * 1000 = 60,000 < 65,535.
const INPUT_SHAPES: &[(usize, usize, &str)] = &[(1, 1, "1r1s"), (3, 2, "3r2s")];

fn bench_all(c: &mut Criterion) {
    for &size in BATCH_SIZES {
        for &(num_res, scopes, shape_label) in INPUT_SHAPES {
            let metrics = generate_metrics(size, num_res, scopes, 10, 5, 3);
            let logs = generate_logs(size, num_res, scopes, 10, 5, 3);
            let traces = generate_traces(size, num_res, scopes, 10, 5, 3);

            let metrics_unsorted: Vec<_> = metrics.iter().map(unsort_ids).collect();
            let logs_unsorted: Vec<_> = logs.iter().map(unsort_ids).collect();
            let traces_unsorted: Vec<_> = traces.iter().map(unsort_ids).collect();

            let metrics_gapped: Vec<_> = metrics.iter().map(introduce_gaps).collect();
            let logs_gapped: Vec<_> = logs.iter().map(introduce_gaps).collect();
            let traces_gapped: Vec<_> = traces.iter().map(introduce_gaps).collect();

            bench_group(
                c,
                &format!("reindex/{size}items/{shape_label}/contiguous"),
                |group| {
                    bench_reindex(group, "metrics/new", reindex, &metrics);
                    bench_reindex(group, "metrics/old", reindex_old, &metrics);
                    bench_reindex(group, "logs/new", reindex, &logs);
                    bench_reindex(group, "logs/old", reindex_old, &logs);
                    bench_reindex(group, "traces/new", reindex, &traces);
                    bench_reindex(group, "traces/old", reindex_old, &traces);
                },
            );

            bench_group(
                c,
                &format!("reindex/{size}items/{shape_label}/unsorted_contiguous"),
                |group| {
                    bench_reindex(group, "metrics/new", reindex, &metrics_unsorted);
                    bench_reindex(group, "metrics/old", reindex_old, &metrics_unsorted);
                    bench_reindex(group, "logs/new", reindex, &logs_unsorted);
                    bench_reindex(group, "logs/old", reindex_old, &logs_unsorted);
                    bench_reindex(group, "traces/new", reindex, &traces_unsorted);
                    bench_reindex(group, "traces/old", reindex_old, &traces_unsorted);
                },
            );

            bench_group(
                c,
                &format!("reindex/{size}items/{shape_label}/gapped"),
                |group| {
                    bench_reindex(group, "metrics/new", reindex, &metrics_gapped);
                    bench_reindex(group, "metrics/old", reindex_old, &metrics_gapped);
                    bench_reindex(group, "logs/new", reindex, &logs_gapped);
                    bench_reindex(group, "logs/old", reindex_old, &logs_gapped);
                    bench_reindex(group, "traces/new", reindex, &traces_gapped);
                    bench_reindex(group, "traces/old", reindex_old, &traces_gapped);
                },
            );

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
                &format!("concatenate/{size}items/{shape_label}/gapped"),
                |group| {
                    bench_concatenate(group, "metrics", &metrics_gapped);
                    bench_concatenate(group, "logs", &logs_gapped);
                    bench_concatenate(group, "traces", &traces_gapped);
                },
            );
        }
    }
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

fn bench_reindex<const N: usize>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    signal_name: &str,
    reindex_fn: fn(&mut [[Option<RecordBatch>; N]]) -> otap_df_pdata::error::Result<()>,
    data: &[[Option<RecordBatch>; N]],
) {
    let _ = group.bench_with_input(BenchmarkId::from_parameter(signal_name), data, |b, data| {
        b.iter_batched(
            || data.to_vec(),
            |mut batches| reindex_fn(&mut batches).expect("reindex failed"),
            BatchSize::SmallInput,
        )
    });
}

fn bench_concatenate<const N: usize>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    signal_name: &str,
    data: &[[Option<RecordBatch>; N]],
) {
    let _ = group.bench_with_input(BenchmarkId::from_parameter(signal_name), data, |b, data| {
        b.iter_batched(
            || data.to_vec(),
            |mut batches| {
                let _ = concatenate::<N>(&mut batches).expect("concat failed");
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

fn unsort_ids<const N: usize>(batches: &[Option<RecordBatch>; N]) -> [Option<RecordBatch>; N] {
    std::array::from_fn(|i| batches[i].as_ref().map(swap_first_last_ids))
}

fn introduce_gaps<const N: usize>(batches: &[Option<RecordBatch>; N]) -> [Option<RecordBatch>; N] {
    std::array::from_fn(|i| batches[i].as_ref().map(double_id_columns))
}

/// Swap the first and last values in every ID/PARENT_ID column so that the
/// column is no longer sorted, defeating the is_sorted() fast path.
fn swap_first_last_ids(rb: &RecordBatch) -> RecordBatch {
    let schema = rb.schema();
    let mut columns: Vec<ArrayRef> = rb.columns().to_vec();
    for (i, field) in schema.fields().iter().enumerate() {
        match field.name().as_str() {
            ID | PARENT_ID => columns[i] = swap_first_last_array(&columns[i]),
            _ if matches!(field.data_type(), DataType::Struct(_)) => {
                columns[i] = Arc::new(swap_first_last_struct_ids(columns[i].as_struct()));
            }
            _ => {}
        }
    }
    RecordBatch::try_new(schema, columns).expect("create record batch")
}

fn swap_first_last_struct_ids(arr: &StructArray) -> StructArray {
    let fields = arr.fields();
    let mut columns: Vec<ArrayRef> = arr.columns().to_vec();
    for (i, field) in fields.iter().enumerate() {
        if field.name() == ID || field.name() == PARENT_ID {
            columns[i] = swap_first_last_array(&columns[i]);
        }
    }
    StructArray::try_new(fields.clone(), columns, arr.nulls().cloned()).expect("create struct")
}

fn swap_first_last_primitive<T: ArrowPrimitiveType>(arr: &PrimitiveArray<T>) -> PrimitiveArray<T> {
    if arr.len() < 2 {
        return arr.clone();
    }
    let mut values: Vec<T::Native> = arr.values().to_vec();
    let last = values.len() - 1;
    values.swap(0, last);
    PrimitiveArray::<T>::new(ScalarBuffer::from(values), arr.nulls().cloned())
}

fn swap_first_last_array(arr: &ArrayRef) -> ArrayRef {
    match arr.data_type() {
        DataType::UInt16 => Arc::new(swap_first_last_primitive(arr.as_primitive::<UInt16Type>())),
        DataType::UInt32 => Arc::new(swap_first_last_primitive(arr.as_primitive::<UInt32Type>())),
        DataType::Dictionary(key_type, value_type) => {
            match (key_type.as_ref(), value_type.as_ref()) {
                (_, DataType::UInt16) => swap_first_last_dict_values::<UInt16Type>(arr, key_type),
                (_, DataType::UInt32) => swap_first_last_dict_values::<UInt32Type>(arr, key_type),
                _ => panic!("Unexpected id column type"),
            }
        }
        _ => panic!("Unexpected id column type"),
    }
}

fn swap_first_last_dict_values<V: ArrowPrimitiveType>(
    arr: &ArrayRef,
    key_type: &DataType,
) -> ArrayRef {
    match key_type {
        DataType::UInt8 => {
            let dict = arr.as_dictionary::<arrow::datatypes::UInt8Type>();
            let new_vals = swap_first_last_primitive::<V>(dict.values().as_primitive());
            Arc::new(DictionaryArray::new(
                dict.keys().clone(),
                Arc::new(new_vals),
            ))
        }
        DataType::UInt16 => {
            let dict = arr.as_dictionary::<UInt16Type>();
            let new_vals = swap_first_last_primitive::<V>(dict.values().as_primitive());
            Arc::new(DictionaryArray::new(
                dict.keys().clone(),
                Arc::new(new_vals),
            ))
        }
        _ => arr.clone(),
    }
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
