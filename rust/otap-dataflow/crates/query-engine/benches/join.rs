// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! benchmarks for expression eval join implementations

use arrow::array::{UInt16Array, UInt32Array};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otel_arrow_dfe_query_engine::pipeline::bench_support::join::{
    U16IdLookupBenchWrapper, U32IdLookupBenchWrapper,
};
use std::collections::HashMap;

/// Benchmark the data structure currently used by the joins for the build side (`IdJoinLookup`)
/// against hashmap, to determine which is more performant.
fn bench_id_join_lookup(c: &mut Criterion) {
    let batch_sizes = [128u16, 1024, 8096];

    let mut group = c.benchmark_group("u16_insert");
    for batch_size in batch_sizes {
        let inputs = (0..batch_size).collect::<Vec<_>>();
        let input_arr = UInt16Array::from_iter_values(inputs);

        let benchmark_id = BenchmarkId::new("idlookup/batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &input_arr, |b, inputs| {
            b.iter(|| {
                let lookup = U16IdLookupBenchWrapper::new(inputs);
                std::hint::black_box(lookup)
            })
        });

        let benchmark_id = BenchmarkId::new("hashmap/batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &input_arr, |b, inputs| {
            b.iter(|| {
                let lookup: HashMap<u16, usize> = HashMap::from_iter(
                    inputs
                        .iter()
                        .flatten()
                        .enumerate()
                        .map(|(idx, id)| (id, idx)),
                );
                std::hint::black_box(lookup)
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("u16_lookup");
    for batch_size in batch_sizes {
        let inputs = (0..batch_size).collect::<Vec<_>>();
        let input_arr = UInt16Array::from_iter_values(inputs.clone());

        let id_lookup = U16IdLookupBenchWrapper::new(&input_arr);
        let hashmap_lookup: HashMap<u16, usize> = HashMap::from_iter(
            input_arr
                .iter()
                .flatten()
                .enumerate()
                .map(|(idx, id)| (id, idx)),
        );

        let benchmark_id = BenchmarkId::new("idlookup/batch_size", batch_size);
        let _ = group.bench_with_input(
            benchmark_id,
            &(id_lookup, inputs.clone()),
            |b, (input, lookup)| {
                b.iter(|| {
                    for input_id in lookup {
                        _ = std::hint::black_box(input.lookup(*input_id));
                    }
                })
            },
        );

        let benchmark_id = BenchmarkId::new("hashmap/batch_size", batch_size);
        let _ = group.bench_with_input(
            benchmark_id,
            &(hashmap_lookup, inputs.clone()),
            |b, (input, lookup)| {
                b.iter(|| {
                    for input_id in lookup {
                        _ = std::hint::black_box(input.get(input_id));
                    }
                })
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("u32_insert");
    for batch_size in batch_sizes {
        let inputs = (0..batch_size).map(|i| i as u32 * 10).collect::<Vec<_>>();
        let input_arr = UInt32Array::from_iter_values(inputs);

        let benchmark_id = BenchmarkId::new("idlookup/batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &input_arr, |b, inputs| {
            b.iter(|| {
                let lookup = U32IdLookupBenchWrapper::new(inputs);
                std::hint::black_box(lookup)
            })
        });

        let benchmark_id = BenchmarkId::new("hashmap/batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &input_arr, |b, inputs| {
            b.iter(|| {
                let lookup: HashMap<u32, usize> = HashMap::from_iter(
                    inputs
                        .iter()
                        .flatten()
                        .enumerate()
                        .map(|(idx, id)| (id, idx)),
                );
                std::hint::black_box(lookup)
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("u32_lookup");
    for batch_size in batch_sizes {
        let inputs = (0..batch_size).map(|i| i as u32 * 10).collect::<Vec<_>>();
        let input_arr = UInt32Array::from_iter_values(inputs.clone());

        let id_lookup = U32IdLookupBenchWrapper::new(&input_arr);
        let hashmap_lookup: HashMap<u32, usize> = HashMap::from_iter(
            input_arr
                .iter()
                .flatten()
                .enumerate()
                .map(|(idx, id)| (id, idx)),
        );

        let benchmark_id = BenchmarkId::new("idlookup/batch_size", batch_size);
        let _ = group.bench_with_input(
            benchmark_id,
            &(id_lookup, inputs.clone()),
            |b, (input, lookup)| {
                b.iter(|| {
                    for input_id in lookup {
                        _ = std::hint::black_box(input.lookup(*input_id));
                    }
                })
            },
        );

        let benchmark_id = BenchmarkId::new("hashmap/batch_size", batch_size);
        let _ = group.bench_with_input(
            benchmark_id,
            &(hashmap_lookup, inputs.clone()),
            |b, (input, lookup)| {
                b.iter(|| {
                    for input_id in lookup {
                        _ = std::hint::black_box(input.get(input_id));
                    }
                })
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
        targets = bench_id_join_lookup
    );
}

criterion_main!(benches::benches);
