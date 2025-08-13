// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for functions that transform attributes

use arrow::array::{DictionaryArray, RecordBatch, StringArray, StringBuilder, UInt8Array};
use arrow::compute::kernels::cmp::eq;
use arrow::compute::{filter, filter_record_batch};
use arrow::datatypes::{DataType, Field, Schema};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::sync::Arc;

use otel_arrow_rust::otap::transform::{
    AttributesTransform, rename_attributes, transform_attributes,
};
use otel_arrow_rust::schema::consts;

fn generate_attribute_batch(num_rows: usize, key_gen: impl Fn(usize) -> String) -> RecordBatch {
    let mut keys_arr = StringBuilder::new();
    for i in 0..num_rows {
        // let attr_key = format!("attr{}", i);
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

fn bench_attribute_rename(c: &mut Criterion) {
    let mut group = c.benchmark_group("rename_attribute");
    for size in [128, 1536, 8092] {
        // this will generate a batch that replaces a single row, to simulate if the keys array
        // was dictionary encoded
        let single_replace_input = generate_attribute_batch(size, |i| format!("attr{i}"));
        let _ = group.bench_with_input(
            BenchmarkId::new("single_replace", size),
            &single_replace_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = rename_attributes(
                            input,
                            &BTreeMap::from_iter([("attr100", "attr_100")]),
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        // this will generate a batch that replaces a contiguous block of attributes, to simulate
        // if attrs key was not dictionary encoded and the batch was sorted by key
        let block_replace_input = generate_attribute_batch(size, |i| format!("attr{}", i / 20));
        let _ = group.bench_with_input(
            BenchmarkId::new("block_replace", size),
            &block_replace_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result =
                            rename_attributes(input, &BTreeMap::from_iter([("attr3", "attr_3")]))
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
            generate_attribute_batch(size, |i| format!("attr{}", i % 20));
        let _ = group.bench_with_input(
            BenchmarkId::new("many_non_contiguous_replace", size),
            &multi_non_contiguous_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result =
                            rename_attributes(input, &BTreeMap::from_iter([("attr3", "attr_3")]))
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

fn bench_transform_attributes(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_attributes");

    for size in [128, 1536, 8092] {
        let single_replace_input = generate_attribute_batch(size, |i| format!("attr{i}"));

        // TODO comment on what's the scoop with this one
        let _ = group.bench_with_input(
            BenchmarkId::new("single_replace_single_delete", size),
            &single_replace_input,
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
                                delete: Some(BTreeSet::from_iter(["attr50".into()])),
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        let sr2 = generate_attribute_batch(size, |i| format!("attr{i}"));
        let col = sr2
            .column_by_name(consts::ATTRIBUTE_KEY)
            .cloned()
            .expect("TODO");
        let dict_input = DictionaryArray::new(
            UInt8Array::from_iter_values((0..sr2.num_rows()).map(|u| u as u8)),
            Arc::new(col),
        );

        let dict_input = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ATTRIBUTE_KEY,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                false,
            )])),
            vec![Arc::new(dict_input)],
        )
        .expect("expect no error");

        let _ = group.bench_with_input(
            BenchmarkId::new("single_replace_single_delete_dict", size),
            &dict_input,
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
                                delete: Some(BTreeSet::from_iter(["attr50".into()])),
                            },
                        )
                        .expect("expect no errors");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        // TODO comment on what's the scoop with this one
        // actually, can probably delete it ...
        let _ = group.bench_with_input(
            BenchmarkId::new("single_replace_single_delete_naive", size),
            &single_replace_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = rename_attributes(
                            input,
                            &BTreeMap::from_iter([("attr100", "attr_100")]),
                        )
                        .expect("expect no errors");

                        let key_col = result.column_by_name("key").expect("no fail");
                        let del_mask = eq(key_col, &StringArray::new_scalar("attr50"))
                            .expect("asdflkajlksdfjlkds");

                        let result = filter_record_batch(&result, &del_mask).expect("no fail");
                        _ = black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("single_replace_no_delete", size),
            &single_replace_input,
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
            BenchmarkId::new("no_replace_single_delete", size),
            &single_replace_input,
            |b, input| {
                b.iter_batched(
                    || input,
                    |input| {
                        let result = transform_attributes(
                            input,
                            &AttributesTransform {
                                rename: None,
                                delete: Some(BTreeSet::from_iter(["attr50".into()])),
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
        targets = bench_attribute_rename, bench_transform_attributes
    );
}
criterion_main!(benches::benches);
