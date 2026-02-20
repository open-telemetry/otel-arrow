// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for functions that transform attributes

use arrow::array::{
    ArrayRef, DictionaryArray, PrimitiveBuilder, RecordBatch, StringBuilder,
    StringDictionaryBuilder, UInt8Builder, UInt16Builder,
};
use arrow::compute::cast;
use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::otap::transform::transport_optimize::apply_transport_optimized_encodings;
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::sync::Arc;

use otap_df_pdata::otap::transform::{
    AttributesTransform, DeleteTransform, LiteralValue, RenameTransform,
    UpsertTransform, transform_attributes, transform_attributes_with_stats,
};
use otap_df_pdata::schema::{FieldExt, consts};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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
        insert: None,
        rename: Some(RenameTransform::new(BTreeMap::from_iter([(
            "attr24".into(),
            "attr_24".into(),
        )]))),
        delete: None,
        upsert: None,
    };

    let single_replace_single_delete = AttributesTransform {
        insert: None,
        rename: Some(RenameTransform::new(BTreeMap::from_iter([(
            "attr24".into(),
            "attr_24".into(),
        )]))),
        delete: Some(DeleteTransform::new(BTreeSet::from_iter(["attr15".into()]))),
        upsert: None,
    };

    let no_replace_single_delete = AttributesTransform {
        insert: None,
        rename: None,
        delete: Some(DeleteTransform::new(BTreeSet::from_iter(["attr15".into()]))),
        upsert: None,
    };

    let attr3_replace_no_delete = AttributesTransform {
        insert: None,
        rename: Some(RenameTransform::new(BTreeMap::from_iter([(
            "attr3".into(),
            "attr_3".into(),
        )]))),
        delete: None,
        upsert: None,
    };

    let no_replace_attr9_delete = AttributesTransform {
        insert: None,
        rename: None,
        delete: Some(DeleteTransform::new(BTreeSet::from_iter(["attr9".into()]))),
        upsert: None,
    };

    let attr3_replace_attr9_delete = AttributesTransform {
        insert: None,
        rename: Some(RenameTransform::new(BTreeMap::from_iter([(
            "attr3".into(),
            "attr_3".into(),
        )]))),
        delete: Some(DeleteTransform::new(BTreeSet::from_iter(["attr9".into()]))),
        upsert: None,
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

fn gen_transport_optimized_bench_batch(
    num_rows: usize,
    dict_encoded_keys: bool,
    transport_encode: bool,
) -> RecordBatch {
    let mut parent_id_builder = UInt16Builder::with_capacity(num_rows);
    let mut type_builder = UInt8Builder::with_capacity(num_rows);
    let mut keys_builder = StringBuilder::new();
    let mut values_builder = StringDictionaryBuilder::<UInt16Type>::new();

    // generate a batch with 8 different attr keys, 4 attrs per parent this is a bit arbitrary,
    // but it should allow us to create something that the renaming will break delta encoding
    // if not handled correctly, which triggers the code path we're of which trying to measure
    // the overhead
    for i in 0..num_rows {
        parent_id_builder.append_value(i as u16 / 4);
        type_builder.append_value(AttributeValueType::Str as u8);
        keys_builder.append_value(format!("key_{}", i % 4));
        values_builder.append_value("val");
    }

    let key_array: ArrayRef = if dict_encoded_keys {
        let keys_arr = keys_builder.finish();
        let keys_arr_dict = cast(
            &keys_arr,
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
        )
        .expect("can cast to dict");
        Arc::new(keys_arr_dict)
    } else {
        Arc::new(keys_builder.finish())
    };

    let schema = Arc::new(Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false)
            .with_encoding(consts::metadata::encodings::PLAIN),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_KEY, key_array.data_type().clone(), false),
        Field::new(
            consts::ATTRIBUTE_STR,
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            true,
        ),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(parent_id_builder.finish()),
            Arc::new(type_builder.finish()),
            key_array,
            Arc::new(values_builder.finish()),
        ],
    )
    .expect("record batch OK");

    if transport_encode {
        let (result, _) = apply_transport_optimized_encodings(&ArrowPayloadType::LogAttrs, &batch)
            .expect("transport optimize encoding apply");

        result
    } else {
        batch
    }
}

/// benchmarks for transforming attributes when batches are transport optimized encoded. This
/// requires some extra handling to ensure we don't break the interpretation of the IDs column
fn bench_transport_optimized_transform_attributes(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_attributes_transport_optimized");

    for dict_encoded_keys in [false, true] {
        for num_rows in [128, 1536, 8092] {
            let benchmark_id_param =
                format!("num_rows={num_rows},dict_keys={dict_encoded_keys},rename,decode=true");
            let input = gen_transport_optimized_bench_batch(num_rows, dict_encoded_keys, true);

            let _ = group.bench_with_input(benchmark_id_param, &input, |b, input| {
                b.iter_batched(
                    || {
                        let transform =
                            AttributesTransform::default().with_rename(RenameTransform::new(
                                [("key_2".into(), "key_3".into())].into_iter().collect(),
                            ));

                        (input, transform)
                    },
                    |(input, transform)| {
                        let result = transform_attributes(input, &transform).expect("no error");
                        black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }

    for dict_encoded_keys in [false, true] {
        for num_rows in [128, 1536, 8092] {
            let benchmark_id_param =
                format!("num_rows={num_rows},dict_keys={dict_encoded_keys},rename,decode=false");
            let input = gen_transport_optimized_bench_batch(num_rows, dict_encoded_keys, true);

            let _ = group.bench_with_input(benchmark_id_param, &input, |b, input| {
                b.iter_batched(
                    || {
                        let transform =
                            AttributesTransform::default().with_rename(RenameTransform::new(
                                [("key_2".into(), "key_4".into())].into_iter().collect(),
                            ));

                        (input, transform)
                    },
                    |(input, transform)| {
                        let result = transform_attributes(input, &transform).expect("no error");
                        black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }

    for dict_encoded_keys in [false, true] {
        for num_rows in [128, 1536, 8092] {
            let benchmark_id_param =
                format!("num_rows={num_rows},dict_keys={dict_encoded_keys},delete,decode=true");
            let input = gen_transport_optimized_bench_batch(num_rows, dict_encoded_keys, true);
            let transform = AttributesTransform::default().with_rename(RenameTransform::new(
                [("key_1".into(), "key_3".into())].into_iter().collect(),
            ));
            let input = transform_attributes(&input, &transform).expect("no error");

            let _ = group.bench_with_input(benchmark_id_param, &input, |b, input| {
                b.iter_batched(
                    || {
                        let transform = AttributesTransform::default().with_delete(
                            DeleteTransform::new([("key_2".into())].into_iter().collect()),
                        );

                        (input, transform)
                    },
                    |(input, transform)| {
                        let result = transform_attributes(input, &transform).expect("no error");
                        black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }

    for dict_encoded_keys in [false, true] {
        for num_rows in [128, 1536, 8092] {
            let benchmark_id_param =
                format!("num_rows={num_rows},dict_keys={dict_encoded_keys},delete,decode=false");
            let input = gen_transport_optimized_bench_batch(num_rows, dict_encoded_keys, true);

            let _ = group.bench_with_input(benchmark_id_param, &input, |b, input| {
                b.iter_batched(
                    || {
                        let transform = AttributesTransform::default().with_delete(
                            DeleteTransform::new([("key_2".into())].into_iter().collect()),
                        );

                        (input, transform)
                    },
                    |(input, transform)| {
                        let result = transform_attributes(input, &transform).expect("no error");
                        black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }

    for dict_encoded_keys in [false, true] {
        for num_rows in [128, 1536, 8092] {
            let benchmark_id_param =
                format!("num_rows={num_rows},dict_keys={dict_encoded_keys},rename,no_encode");
            let input = gen_transport_optimized_bench_batch(num_rows, dict_encoded_keys, false);

            let _ = group.bench_with_input(benchmark_id_param, &input, |b, input| {
                b.iter_batched(
                    || {
                        let transform =
                            AttributesTransform::default().with_rename(RenameTransform::new(
                                [("key_2".into(), "key_3".into())].into_iter().collect(),
                            ));

                        (input, transform)
                    },
                    |(input, transform)| {
                        let result = transform_attributes(input, &transform).expect("no error");
                        black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }

    for dict_encoded_keys in [false, true] {
        for num_rows in [128, 1536, 8092] {
            let benchmark_id_param =
                format!("num_rows={num_rows},dict_keys={dict_encoded_keys},delete,no_encode");
            let input = gen_transport_optimized_bench_batch(num_rows, dict_encoded_keys, false);

            let _ = group.bench_with_input(benchmark_id_param, &input, |b, input| {
                b.iter_batched(
                    || {
                        let transform = AttributesTransform::default().with_delete(
                            DeleteTransform::new([("key_2".into())].into_iter().collect()),
                        );

                        (input, transform)
                    },
                    |(input, transform)| {
                        let result = transform_attributes(input, &transform).expect("no error");
                        black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }

    for dict_encoded_keys in [false, true] {
        for num_rows in [128, 1536, 8092] {
            let benchmark_id_param =
                format!("num_rows={num_rows},dict_keys={dict_encoded_keys},rename,no_encode,stat");
            let input = gen_transport_optimized_bench_batch(num_rows, dict_encoded_keys, false);

            let _ = group.bench_with_input(benchmark_id_param, &input, |b, input| {
                b.iter_batched(
                    || {
                        let transform =
                            AttributesTransform::default().with_rename(RenameTransform::new(
                                [("key_2".into(), "key_3".into())].into_iter().collect(),
                            ));

                        (input, transform)
                    },
                    |(input, transform)| {
                        let result =
                            transform_attributes_with_stats(input, &transform).expect("no error");
                        black_box(result)
                    },
                    BatchSize::SmallInput,
                )
            });
        }
    }

    group.finish();
}

/// Benchmarks for upsert transforms (insert-or-update semantics).
/// Measures both the case where keys already exist (update path) and where they don't (insert
/// path), as well as combined upsert+delete scenarios.
fn bench_upsert_attributes(c: &mut Criterion) {
    let mut group = c.benchmark_group("upsert_attributes");

    // Upsert a single key that does NOT exist in any row (pure insert path)
    let upsert_new_key = AttributesTransform::default().with_upsert(UpsertTransform::new(
        [("new_key".into(), LiteralValue::Str("new_val".into()))]
            .into_iter()
            .collect(),
    ));

    // Upsert a single key that DOES exist in every row (update/overwrite path)
    let upsert_existing_key = AttributesTransform::default().with_upsert(UpsertTransform::new(
        [("key_2".into(), LiteralValue::Str("updated_val".into()))]
            .into_iter()
            .collect(),
    ));

    // Upsert + delete combined
    let upsert_with_delete = AttributesTransform::default()
        .with_upsert(UpsertTransform::new(
            [("key_1".into(), LiteralValue::Str("upserted_val".into()))]
                .into_iter()
                .collect(),
        ))
        .with_delete(DeleteTransform::new(
            [("key_3".into())].into_iter().collect(),
        ));

    // Upsert multiple keys (mix of new and existing)
    let upsert_multiple = AttributesTransform::default().with_upsert(UpsertTransform::new(
        [
            ("key_0".into(), LiteralValue::Str("updated_0".into())),
            ("key_2".into(), LiteralValue::Str("updated_2".into())),
            ("inserted_key".into(), LiteralValue::Str("new".into())),
        ]
        .into_iter()
        .collect(),
    ));

    for dict_encoded_keys in [false, true] {
        for num_rows in [128, 1536, 8192] {
            let input = gen_transport_optimized_bench_batch(num_rows, dict_encoded_keys, false);
            let id_prefix = format!("rows={num_rows},dict_keys={dict_encoded_keys}");

            let _ = group.bench_with_input(
                BenchmarkId::new("upsert_new_key", &id_prefix),
                &input,
                |b, input| {
                    b.iter_batched(
                        || input,
                        |input| {
                            transform_attributes(black_box(input), &upsert_new_key)
                                .expect("no error")
                        },
                        BatchSize::SmallInput,
                    )
                },
            );

            let _ = group.bench_with_input(
                BenchmarkId::new("upsert_existing_key", &id_prefix),
                &input,
                |b, input| {
                    b.iter_batched(
                        || input,
                        |input| {
                            transform_attributes(black_box(input), &upsert_existing_key)
                                .expect("no error")
                        },
                        BatchSize::SmallInput,
                    )
                },
            );

            let _ = group.bench_with_input(
                BenchmarkId::new("upsert_with_delete", &id_prefix),
                &input,
                |b, input| {
                    b.iter_batched(
                        || input,
                        |input| {
                            transform_attributes(black_box(input), &upsert_with_delete)
                                .expect("no error")
                        },
                        BatchSize::SmallInput,
                    )
                },
            );

            let _ = group.bench_with_input(
                BenchmarkId::new("upsert_multiple_keys", &id_prefix),
                &input,
                |b, input| {
                    b.iter_batched(
                        || input,
                        |input| {
                            transform_attributes(black_box(input), &upsert_multiple)
                                .expect("no error")
                        },
                        BatchSize::SmallInput,
                    )
                },
            );
        }
    }

    group.finish();
}

#[allow(missing_docs)]
mod benches {
    use super::*;
    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_transform_attributes, bench_transport_optimized_transform_attributes, bench_upsert_attributes
    );
}
criterion_main!(benches::benches);
