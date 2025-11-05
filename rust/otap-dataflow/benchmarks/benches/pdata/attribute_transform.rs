// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for functions that transform attributes

use arrow::array::{DictionaryArray, PrimitiveBuilder, RecordBatch, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::sync::Arc;

use otel_arrow_rust::otap::transform::{
    AttributesTransform, DeleteTransform, RenameTransform, transform_attributes,
};
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
    // Pre-create AttributesTransform instances to avoid measuring their creation cost
    let single_replace_no_delete = AttributesTransform {
        rename: Some(RenameTransform::new(BTreeMap::from_iter([(
            "attr24".into(),
            "attr_24".into(),
        )]))),
        delete: None,
    };

    let single_replace_single_delete = AttributesTransform {
        rename: Some(RenameTransform::new(BTreeMap::from_iter([(
            "attr24".into(),
            "attr_24".into(),
        )]))),
        delete: Some(DeleteTransform::new(BTreeSet::from_iter(["attr15".into()]))),
    };

    let no_replace_single_delete = AttributesTransform {
        rename: None,
        delete: Some(DeleteTransform::new(BTreeSet::from_iter(["attr15".into()]))),
    };

    let attr3_replace_no_delete = AttributesTransform {
        rename: Some(RenameTransform::new(BTreeMap::from_iter([(
            "attr3".into(),
            "attr_3".into(),
        )]))),
        delete: None,
    };

    let no_replace_attr9_delete = AttributesTransform {
        rename: None,
        delete: Some(DeleteTransform::new(BTreeSet::from_iter(["attr9".into()]))),
    };

    let attr3_replace_attr9_delete = AttributesTransform {
        rename: Some(RenameTransform::new(BTreeMap::from_iter([(
            "attr3".into(),
            "attr_3".into(),
        )]))),
        delete: Some(DeleteTransform::new(BTreeSet::from_iter(["attr9".into()]))),
    };

    let mut group = c.benchmark_group("transform_attributes_dict_keys");
    for (num_keys, num_rows) in [
        (32, 128),   // 32 keys, 128 rows, 4 rows/key
        (32, 1536),  // 32 keys, 1536 rows, 48 rows/key
        (32, 8192),  // 32 keys, 8192 rows, 256 rows/key
        (128, 128),  // 128 keys, 128 rows, 1 rows/key
        (128, 1536), // 128 keys, 1536 rows, 12 rows/key
        (128, 8192), // 128 keys, 8192 rows, 64 rows/key
    ] {
        let rows_per_key = num_rows / num_keys;
        let dict_transform_input =
            generate_dict_keys_attribute_batch(num_keys, |i| format!("attr{i}"), rows_per_key);

        let benchmark_id_param =
            format!("keys={num_keys},rows={num_rows},rows_per_key={rows_per_key}");

        let _ = group.bench_with_input(
            BenchmarkId::new("single_replace_no_deletes", &benchmark_id_param),
            &dict_transform_input,
            |b: &mut criterion::Bencher<'_>, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        transform_attributes(black_box(input), &single_replace_no_delete)
                            .expect("expect no errors")
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
                        transform_attributes(black_box(input), &single_replace_single_delete)
                            .expect("expect no errors")
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
                        transform_attributes(black_box(input), &no_replace_single_delete)
                            .expect("expect no errors")
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();

    let mut group = c.benchmark_group("transform_attributes_native_keys");
    for num_rows in [128, 1536, 8192] {
        // this will generate a batch that replaces a contiguous block of attributes, to simulate
        // if attrs key was not dictionary encoded and the batch was sorted by key
        let block_transform_input =
            generate_native_keys_attr_batch(num_rows, |i| format!("attr{}", i / 20));
        let benchmark_id_param = format!("rows={num_rows}");

        let _ = group.bench_with_input(
            BenchmarkId::new("block_replace_no_delete", &benchmark_id_param),
            &block_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        transform_attributes(black_box(input), &attr3_replace_no_delete)
                            .expect("expect no errors")
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        let _ = group.bench_with_input(
            BenchmarkId::new("no_replace_block_delete", &benchmark_id_param),
            &block_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        transform_attributes(black_box(input), &no_replace_attr9_delete)
                            .expect("expect no errors")
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        let _ = group.bench_with_input(
            BenchmarkId::new("block_replace_block_delete", &benchmark_id_param),
            &block_transform_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        transform_attributes(black_box(input), &attr3_replace_attr9_delete)
                            .expect("expect no errors")
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        // this will generate a batch that replaces many non-contiguous rows, to simulate if the
        // attrs key was not dictionary encoded and the batch was not sorted by key. This could
        // happen when we encode OTAP from OTLP, where the attributes end up sorted by parent ID
        let multi_non_contiguous_input =
            generate_native_keys_attr_batch(num_rows, |i| format!("attr{}", i % 20));
        let _ = group.bench_with_input(
            BenchmarkId::new("many_non_contiguous_replace_no_delete", &benchmark_id_param),
            &multi_non_contiguous_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        transform_attributes(black_box(input), &attr3_replace_no_delete)
                            .expect("expect no errors")
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("no_replace_many_non_contiguous_delete", &benchmark_id_param),
            &multi_non_contiguous_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        transform_attributes(black_box(input), &no_replace_attr9_delete)
                            .expect("expect no errors")
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("many_contiguous_replace_and_delete", &benchmark_id_param),
            &multi_non_contiguous_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        transform_attributes(black_box(input), &attr3_replace_attr9_delete)
                            .expect("expect no errors")
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
