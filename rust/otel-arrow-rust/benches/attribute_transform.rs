// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for functions that transform attributes

use arrow::array::{DictionaryArray, PrimitiveBuilder, RecordBatch, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::sync::Arc;

use otel_arrow_rust::otap::transform::{AttributesTransform, transform_attributes};
use otel_arrow_rust::schema::consts;

fn generate_native_keys_attr_batch(
    num_rows: usize,
    key_gen: impl Fn(usize) -> String,
) -> RecordBatch {
    let mut keys_arr = StringBuilder::new();
    for i in 0..num_rows {
        let attr_key = key_gen(i);
        keys_arr.append_value(attr_key);
    }

    RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::ATTRIBUTE_KEY,
            DataType::Utf8,
            false,
        )])),
        vec![Arc::new(keys_arr.finish())],
    )
    .expect("expect no error")
}

fn generate_dict_keys_attribute_batch(
    num_keys: usize,
    key_gen: impl Fn(usize) -> String,
    rows_per_key: usize,
) -> RecordBatch {
    let mut keys_dict_values_arr = StringBuilder::new();
    let mut keys_dict_keys_arr = PrimitiveBuilder::<UInt16Type>::new();
    for i in 0..num_keys {
        let attr_key = key_gen(i);
        keys_dict_values_arr.append_value(attr_key);
        keys_dict_keys_arr.append_value_n(i as u16, rows_per_key);
    }

    let keys_arr = DictionaryArray::new(
        keys_dict_keys_arr.finish(),
        Arc::new(keys_dict_values_arr.finish()),
    );

    let schema = Arc::new(Schema::new(vec![Field::new(
        consts::ATTRIBUTE_KEY,
        DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
        false,
    )]));
    RecordBatch::try_new(schema, vec![Arc::new(keys_arr)]).expect("expect no error")
}

fn bench_transform_attributes(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_attributes_dict_keys");
    for (num_keys, num_rows) in [
        (128, 512),  // 128 keys, 512 rows, 4 rows/key
        (128, 1536), // 128 keys, 1536 rows, 24 rows/key
        (128, 8192), // 128 keys, 8192 rows, 128 rows/key
    ] {
        let rows_per_key = num_rows / num_keys;
        let dict_transform_input =
            generate_dict_keys_attribute_batch(num_keys, |i| format!("attr{i}"), rows_per_key);

        let benchmark_id_param = format!("keys={num_keys},rows={num_rows}");

        let _ = group.bench_with_input(
            BenchmarkId::new("single_replace_no_deletes", &benchmark_id_param),
            &dict_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: Some(BTreeMap::from_iter([(
                                    "attr100".into(),
                                    "attr_100".into(),
                                )])),
                                delete: None,
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("single_replace_single_delete", &benchmark_id_param),
            &dict_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: Some(BTreeMap::from_iter([(
                                    "attr100".into(),
                                    "attr_100".into(),
                                )])),
                                delete: Some(BTreeSet::from_iter(["attr80".into()])),
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        let _ = group.bench_with_input(
            BenchmarkId::new("no_replace_single_delete", &benchmark_id_param),
            &dict_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: None,
                                delete: Some(BTreeSet::from_iter(["attr80".into()])),
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();

    let mut group = c.benchmark_group("transform_attributes_native_keys");
    for size in [128, 1536, 8192] {
        // this will generate a batch that replaces a contiguous block of attributes, to simulate
        // if attrs key was not dictionary encoded and the batch was sorted by key
        let block_transform_input =
            generate_native_keys_attr_batch(size, |i| format!("attr{}", i / 20));
        let _ = group.bench_with_input(
            BenchmarkId::new("block_replace_no_delete", size),
            &block_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: Some(BTreeMap::from_iter([(
                                    "attr3".into(),
                                    "attr_3".into(),
                                )])),
                                delete: None,
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        let _ = group.bench_with_input(
            BenchmarkId::new("no_replace_block_delete", size),
            &block_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: None,
                                delete: Some(BTreeSet::from_iter(["attr9".into()])),
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        let _ = group.bench_with_input(
            BenchmarkId::new("block_replace_block_delete", size),
            &block_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: Some(BTreeMap::from_iter([(
                                    "attr3".into(),
                                    "attr_3".into(),
                                )])),
                                delete: Some(BTreeSet::from_iter(["attr9".into()])),
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        // this will generate a batch that replaces many non-contiguous rows, to simulate if the
        // attrs key was not dictionary encoded and the batch was not sorted by key. This could
        // happen when we encode OTAP from OTLP, where the attributes end up sorted by parent ID
        let multi_non_contiguous_input =
            generate_native_keys_attr_batch(size, |i| format!("attr{}", i % 20));
        let _ = group.bench_with_input(
            BenchmarkId::new("many_non_contiguous_replace_no_delete", size),
            &multi_non_contiguous_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: Some(BTreeMap::from_iter([(
                                    "attr3".into(),
                                    "attr_3".into(),
                                )])),
                                delete: None,
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("no_replace_many_non_contiguous_delete", size),
            &multi_non_contiguous_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: None,
                                delete: Some(BTreeSet::from_iter(["attr9".into()])),
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("many_contiguous_replace_and_delete", size),
            &multi_non_contiguous_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: Some(BTreeMap::from_iter([(
                                    "attr3".into(),
                                    "attr_3".into(),
                                )])),
                                delete: Some(BTreeSet::from_iter(["attr9".into()])),
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

#[allow(missing_docs)]
mod benches {
    use super::*;
    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_transform_attributes
    );
}
criterion_main!(benches::benches);
