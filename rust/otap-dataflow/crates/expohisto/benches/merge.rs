// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compares sequential and packed exponential histogram merging.

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_expohisto::HistogramNN;
use std::hint::black_box;

struct Scenario<const N: usize, const M: usize> {
    name: &'static str,
    destination: HistogramNN<N>,
    source: HistogramNN<M>,
}

fn build<const N: usize>(values: &[f64]) -> HistogramNN<N> {
    let mut histogram = HistogramNN::new();
    for &value in values {
        histogram
            .update(value)
            .expect("benchmark observations are finite");
    }
    histogram
}

fn assert_equivalent<const N: usize>(optimized: &HistogramNN<N>, sequential: &HistogramNN<N>) {
    assert_eq!(optimized.current_settings(), sequential.current_settings());

    let optimized = optimized.view();
    let sequential = sequential.view();
    let optimized_stats = optimized.stats();
    let sequential_stats = sequential.stats();
    assert_eq!(optimized_stats.count, sequential_stats.count);
    assert_eq!(optimized_stats.sum, sequential_stats.sum);
    assert_eq!(optimized_stats.min, sequential_stats.min);
    assert_eq!(optimized_stats.max, sequential_stats.max);

    let optimized = optimized.positive();
    let sequential = sequential.positive();
    assert_eq!(optimized.width(), sequential.width());
    assert_eq!(optimized.offset(), sequential.offset());
    assert_eq!(optimized.len(), sequential.len());
    assert_eq!(
        optimized.iter().collect::<Vec<_>>(),
        sequential.iter().collect::<Vec<_>>()
    );
}

fn bench_scenario<const N: usize, const M: usize>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    scenario: Scenario<N, M>,
) {
    let mut optimized = scenario.destination.clone();
    optimized
        .merge_from(&scenario.source)
        .expect("prepared counts do not overflow");
    let mut sequential = scenario.destination.clone();
    sequential
        .merge_from_sequential_reference(&scenario.source)
        .expect("prepared counts do not overflow");
    assert_equivalent(&optimized, &sequential);

    let _ = group.throughput(Throughput::Elements(scenario.source.view().stats().count));
    let _ = group.bench_with_input(
        BenchmarkId::new(scenario.name, "optimized"),
        &scenario,
        |b, scenario| {
            b.iter_batched(
                || (),
                |()| {
                    let mut destination = black_box(scenario.destination.clone());
                    destination
                        .merge_from(black_box(&scenario.source))
                        .expect("prepared counts do not overflow");
                    black_box(destination)
                },
                BatchSize::SmallInput,
            );
        },
    );
    let _ = group.bench_with_input(
        BenchmarkId::new(scenario.name, "sequential"),
        &scenario,
        |b, scenario| {
            b.iter_batched(
                || (),
                |()| {
                    let mut destination = black_box(scenario.destination.clone());
                    destination
                        .merge_from_sequential_reference(black_box(&scenario.source))
                        .expect("prepared counts do not overflow");
                    black_box(destination)
                },
                BatchSize::SmallInput,
            );
        },
    );
}

fn merge_benchmarks(c: &mut Criterion) {
    let dense_left = (0..4_096)
        .map(|index| 0.75 + (index % 128) as f64 / 128.0)
        .collect::<Vec<_>>();
    let dense_right = (0..4_096)
        .map(|index| 0.875 + (index % 128) as f64 / 128.0)
        .collect::<Vec<_>>();

    let mut distant_left = vec![1.0; 1_024];
    distant_left.extend((0..128).map(|index| 1.0 + index as f64 / 128.0));
    let distant_right = (0..256)
        .map(|index| 2.0_f64.powi(18 + (index % 4)))
        .collect::<Vec<_>>();

    let negative_left = (0..2_048)
        .map(|index| 0.25 + (index % 64) as f64 / 512.0)
        .collect::<Vec<_>>();
    let negative_right = (0..2_048)
        .map(|index| 0.375 + (index % 64) as f64 / 512.0)
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("expohisto_merge");
    bench_scenario(
        &mut group,
        Scenario::<64, 32> {
            name: "overlapping_dense",
            destination: build(&dense_left),
            source: build(&dense_right),
        },
    );
    bench_scenario(
        &mut group,
        Scenario::<64, 8> {
            name: "counter_width_growth",
            destination: build(&[1.0; 10]),
            source: build(&[1.0; 10]),
        },
    );
    bench_scenario(
        &mut group,
        Scenario::<16, 8> {
            name: "differing_scale_and_range",
            destination: build(&distant_left),
            source: build(&distant_right),
        },
    );
    bench_scenario(
        &mut group,
        Scenario::<32, 16> {
            name: "negative_circular_indices",
            destination: build(&negative_left),
            source: build(&negative_right),
        },
    );
    group.finish();
}

criterion_group!(benches, merge_benchmarks);
criterion_main!(benches);
