// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Before/after microbenchmark of repeated fixed-size binary dictionary appends.
//!
//! Run with `cargo bench -p otel-arrow-dfe-pdata --bench fixed_size_binary_dictionary`.
//! `scalar_loop` reproduces the previous checked helper; `append_n` calls the current helper.
//! Each timed operation appends one value n times to a fresh u8-keyed dictionary
//! containing one seed entry. "new" inserts a second entry; "existing" reuses the seed.
//! Setup, allocation of key/value capacity, finishing, assertions, and destruction are
//! outside timing. Hash-table insertion (for new entries) and key writes remain timed.
//! Batches contain 64 independent builders to bound memory and amortize timer overhead.
//! These are helper timings, not end-to-end throughput or production pipeline gains.

use std::hint::black_box;
use std::time::Duration;

use arrow::array::FixedSizeBinaryDictionaryBuilder;
use arrow::datatypes::UInt8Type;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_pdata::encode::record::array::dictionary::{
    CheckedDictionaryAppendSlice, checked,
};

type Builder = FixedSizeBinaryDictionaryBuilder<UInt8Type>;

fn scalar_loop(builder: &mut Builder, value: &[u8], n: usize) -> checked::Result<usize> {
    let mut index = 0;
    for _ in 0..n {
        index = builder.append_slice(value)?;
    }
    Ok(index)
}

fn fixture(width: i32, n: usize, seed: &[u8]) -> Builder {
    let mut builder = Builder::with_capacity(n + 1, 2, width);
    let _ = builder.append_slice(seed).expect("valid seed");
    builder
}

fn bench_fixed_size_binary_dictionary(c: &mut Criterion) {
    for width in [8, 16] {
        let seed = vec![1; width as usize];
        for (entry, byte) in [("new", 2), ("existing", 1)] {
            let value = vec![byte; width as usize];
            let mut group = c.benchmark_group(format!("fixed_size_binary/{width}/{entry}"));
            let _ = group
                .sample_size(100)
                .warm_up_time(Duration::from_secs(1))
                .measurement_time(Duration::from_secs(3));

            for n in [1, 8, 64, 1024] {
                // Check equivalence once per fixture, outside Criterion's measurement.
                let mut before = fixture(width, n, &seed);
                let mut after = fixture(width, n, &seed);
                assert_eq!(
                    scalar_loop(&mut before, &value, n).expect("valid scalar append"),
                    after.append_slice_n(&value, n).expect("valid bulk append")
                );
                assert_eq!(before.finish(), after.finish());

                let _ = group.throughput(Throughput::Elements(n as u64));
                let _ = group.bench_with_input(BenchmarkId::new("scalar_loop", n), &n, |b, &n| {
                    b.iter_batched_ref(
                        || fixture(width, n, &seed),
                        |builder| {
                            black_box(
                                scalar_loop(black_box(builder), black_box(&value), black_box(n))
                                    .expect("valid repeated value"),
                            )
                        },
                        BatchSize::NumIterations(64),
                    );
                });
                let _ = group.bench_with_input(BenchmarkId::new("append_n", n), &n, |b, &n| {
                    b.iter_batched_ref(
                        || fixture(width, n, &seed),
                        |builder| {
                            black_box(
                                black_box(builder)
                                    .append_slice_n(black_box(&value), black_box(n))
                                    .expect("valid repeated value"),
                            )
                        },
                        BatchSize::NumIterations(64),
                    );
                });
            }
            group.finish();
        }
    }
}

criterion_group!(benches, bench_fixed_size_binary_dictionary);
criterion_main!(benches);
