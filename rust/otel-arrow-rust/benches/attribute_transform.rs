// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for functions that transform attributes

use arrow::array::{RecordBatch, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;

use otel_arrow_rust::otap::transform::rename_attr;
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
                        let result = rename_attr(input, "attr100", "attr_100").expect("expect no error");
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
                        let result = rename_attr(input, "attr3", "attr_3").expect("expect no error");
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
                        let result = rename_attr(input, "attr3", "attr_3").expect("expect no error");
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
    criterion_group!(benches, bench_attribute_rename);
}
criterion_main!(benches::benches);
