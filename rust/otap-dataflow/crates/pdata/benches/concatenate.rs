// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for concatenating OTAP record batches
//!
//! This benchmark compares two implementations for concatenating OTAP records:
//! 1. The new `concatenate` function from `transform/concatenate.rs`
//! 2. The old `generic_schemaless_concatenate` function from `groups.rs`
//!
//! **Important**: Both implementations are benchmarked on the exact same data to ensure
//! fair comparison. Data is generated once upfront for each configuration, then cloned
//! for each benchmark iteration.
//!
//! ## Running the benchmarks
//!
//! To run all concatenation benchmarks:
//! ```bash
//! cargo bench --bench concatenate
//! ```
//!
//! To run only single-metric-type benchmarks:
//! ```bash
//! cargo bench --bench concatenate concatenate_comparison
//! ```
//!
//! To run only mixed-metric-type benchmarks:
//! ```bash
//! cargo bench --bench concatenate mixed_comparison
//! ```
//!
//! To test that the benchmarks work without running full benchmarks:
//! ```bash
//! cargo bench --bench concatenate -- --test
//! ```

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::otap::groups::generic_schemaless_concatenate;
use otap_df_pdata::otap::transform::concatenate::concatenate;
use otap_df_pdata::otap::{Metrics, OtapArrowRecords, OtapBatchStore};
use otap_df_pdata::testing::fixtures::{DataGenerator, MetricsConfig};
use otap_df_pdata::testing::round_trip::otlp_to_otap;

/// Helper function to generate test data and convert to batch arrays
fn generate_metrics_batches(
    num_inputs: usize,
    points_per_metric: usize,
) -> Vec<[Option<RecordBatch>; 19]> {
    let mut datagen = DataGenerator::with_metrics_config(
        MetricsConfig::new().with_gauges(vec![points_per_metric]),
    );

    (0..num_inputs)
        .map(|_| {
            let metrics_data = datagen.generate_metrics_from_config();
            let otap_records = otlp_to_otap(&metrics_data.into());

            // Extract batches from OtapArrowRecords::Metrics
            match otap_records {
                OtapArrowRecords::Metrics(metrics) => metrics.into_batches(),
                _ => panic!("Expected metrics"),
            }
        })
        .collect()
}

/// Helper function to generate mixed metric types
fn generate_mixed_metrics_batches(num_inputs: usize) -> Vec<[Option<RecordBatch>; 19]> {
    let mut datagen = DataGenerator::with_metrics_config(
        MetricsConfig::new()
            .with_gauges(vec![5, 10])
            .with_sums(vec![3, 7])
            .with_histograms(vec![2, 4]),
    );

    (0..num_inputs)
        .map(|_| {
            let metrics_data = datagen.generate_metrics_from_config();
            let otap_records = otlp_to_otap(&metrics_data.into());

            match otap_records {
                OtapArrowRecords::Metrics(metrics) => metrics.into_batches(),
                _ => panic!("Expected metrics"),
            }
        })
        .collect()
}

/// Benchmark comparing new vs old concatenate implementations
fn bench_concatenate_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("concatenate_comparison");

    // Test different numbers of input batches
    for num_inputs in [10, 100, 1000] {
        // Test different batch sizes
        for points in [5, 50] {
            // Generate data ONCE upfront for this configuration
            let test_data = generate_metrics_batches(num_inputs, points);

            // Benchmark new implementation
            let _ = group.bench_with_input(
                BenchmarkId::new("new", format!("{}inputs_{}points", num_inputs, points)),
                &test_data,
                |b, data| {
                    b.iter_batched(
                        || {
                            // Clone the pre-generated data for each iteration
                            data.clone()
                        },
                        |mut batches| {
                            // Benchmark: Run concatenation
                            concatenate::<{ Metrics::COUNT }>(&mut batches)
                                .expect("Concatenation failed")
                        },
                        BatchSize::SmallInput,
                    )
                },
            );

            // Benchmark old implementation with the SAME data
            let _ = group.bench_with_input(
                BenchmarkId::new("old", format!("{}inputs_{}points", num_inputs, points)),
                &test_data,
                |b, data| {
                    b.iter_batched(
                        || {
                            // Clone the same pre-generated data for each iteration
                            data.clone()
                        },
                        |mut batches| {
                            // Benchmark: Run old concatenation implementation
                            generic_schemaless_concatenate::<{ Metrics::COUNT }>(&mut batches)
                                .expect("Concatenation failed")
                        },
                        BatchSize::SmallInput,
                    )
                },
            );
        }
    }

    group.finish();
}

use arrow::array::RecordBatch;

/// Benchmark comparing new vs old implementations with mixed metric types
fn bench_concatenate_mixed_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("concatenate_mixed_comparison");

    for num_inputs in [10, 100, 1000] {
        // Generate mixed metrics data ONCE upfront for this configuration
        let test_data = generate_mixed_metrics_batches(num_inputs);

        // Benchmark new implementation
        let _ = group.bench_with_input(
            BenchmarkId::new("new", format!("{}inputs", num_inputs)),
            &test_data,
            |b, data| {
                b.iter_batched(
                    || {
                        // Clone the pre-generated data for each iteration
                        data.clone()
                    },
                    |mut batches| {
                        concatenate::<{ Metrics::COUNT }>(&mut batches)
                            .expect("Concatenation failed")
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        // Benchmark old implementation with the SAME data
        let _ = group.bench_with_input(
            BenchmarkId::new("old", format!("{}inputs", num_inputs)),
            &test_data,
            |b, data| {
                b.iter_batched(
                    || {
                        // Clone the same pre-generated data for each iteration
                        data.clone()
                    },
                    |mut batches| {
                        generic_schemaless_concatenate::<{ Metrics::COUNT }>(&mut batches)
                            .expect("Concatenation failed")
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_concatenate_comparison,
    bench_concatenate_mixed_comparison
);
criterion_main!(benches);
