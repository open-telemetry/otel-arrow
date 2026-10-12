// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for dictionary sanitization with dense and sparse live values.

use std::{hint::black_box, sync::Arc};

use arrow::{
    array::{Array, DictionaryArray, Int32Array, RecordBatch, StringArray, UInt16Array},
    datatypes::UInt16Type,
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_pdata::arrays::sanitize::{
    CooperativeBudget, sanitize_record_batch, sanitize_record_batch_cooperative,
};

criterion_group!(benches, bench_sanitize);
criterion_main!(benches);

fn bench_sanitize(criterion: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("benchmark runtime");
    let mut group = criterion.benchmark_group("sanitize");
    let _ = group.throughput(Throughput::Elements(8192));
    let cases: [(&str, Arc<dyn Array>); 3] = [
        ("i32", Arc::new(Int32Array::from_iter_values(0..4097))),
        (
            "utf8_32",
            Arc::new(StringArray::from_iter_values(
                (0..4097).map(|_| "x".repeat(32)),
            )),
        ),
        (
            "utf8_4096",
            Arc::new(StringArray::from_iter_values(
                (0..4097).map(|_| "x".repeat(4096)),
            )),
        ),
    ];
    for (value_type, values) in cases {
        for sparse in [false, true] {
            let keys = UInt16Array::from_iter((0..8192).map(|row| {
                let index = row % 4096;
                let key = if sparse { index / 2 * 2 } else { index } as u16;
                (row % 17 != 0).then_some(key)
            }));
            let dictionary = DictionaryArray::<UInt16Type>::new(keys, Arc::clone(&values));
            let batch =
                RecordBatch::try_from_iter([("value", Arc::new(dictionary) as Arc<dyn Array>)])
                    .expect("benchmark batch");
            let pattern = if sparse { "sparse" } else { "dense" };
            let case = format!("{value_type}/{pattern}");
            let _ = group.bench_with_input(
                BenchmarkId::new("sync", &case),
                &batch,
                |benchmark, batch| {
                    benchmark.iter(|| {
                        sanitize_record_batch(black_box(batch)).expect("unused dictionary values")
                    });
                },
            );
            let _ = group.bench_with_input(
                BenchmarkId::new("cooperative", &case),
                &batch,
                |benchmark, batch| {
                    benchmark.iter(|| {
                        runtime.block_on(async {
                            sanitize_record_batch_cooperative(
                                black_box(batch),
                                &mut CooperativeBudget::default(),
                            )
                            .await
                            .expect("unused dictionary values")
                        })
                    });
                },
            );
        }
    }
    group.finish();
}
