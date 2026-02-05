// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for concatenating OTAP record batches
//!
//! ## Running
//!
//! ```bash
//! cargo bench --bench concatenate
//! ```
//!
//! ```

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::otap::transform::concatenate::concatenate;
use otap_df_pdata::otap::{Metrics, OtapArrowRecords, OtapBatchStore};
use otap_df_pdata::testing::fixtures::{DataGenerator, MetricsConfig};
use otap_df_pdata::testing::round_trip::otlp_to_otap;

/// Helper function to generate test data and convert to batch arrays
fn generate_metrics_batches(
    num_inputs: usize,
    points_per_metric: usize,
) -> Vec<[Option<RecordBatch>; Metrics::COUNT]> {
    let mut datagen = DataGenerator::with_metrics_config(
        MetricsConfig::new().with_gauges(vec![points_per_metric]),
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
fn bench_concatenate(c: &mut Criterion) {
    let mut group = c.benchmark_group("concatenate_comparison");

    for num_inputs in [10, 100, 1000] {
        for points in [5, 50] {
            let test_data = generate_metrics_batches(num_inputs, points);
            let _ = group.bench_with_input(
                BenchmarkId::new("new", format!("{}inputs_{}points", num_inputs, points)),
                &test_data,
                |b, data| {
                    b.iter_batched(
                        || data.clone(),
                        |mut batches| {
                            concatenate::<{ Metrics::COUNT }>(&mut batches)
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
fn bench_concatenate_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("concatenate_mixed_comparison");

    for num_inputs in [10, 100, 1000] {
        let test_data = generate_mixed_metrics_batches(num_inputs);
        let _ = group.bench_with_input(
            BenchmarkId::new("new", format!("{}inputs", num_inputs)),
            &test_data,
            |b, data| {
                b.iter_batched(
                    || data.clone(),
                    |mut batches| {
                        concatenate::<{ Metrics::COUNT }>(&mut batches)
                            .expect("Concatenation failed")
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_concatenate, bench_concatenate_mixed);
criterion_main!(benches);
