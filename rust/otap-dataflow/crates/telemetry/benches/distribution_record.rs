// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compares the hot-path recording cost of ITS distribution tiers.
//!
//! Run with:
//!
//! ```text
//! cargo bench -p otap-df-telemetry --bench distribution_record
//! ```

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_telemetry::instrument::{Counter, HistogramDetailed, HistogramNormal, Mmsc};
use std::hint::black_box;

const OBSERVATIONS: usize = 1_024;

fn observations() -> [f64; OBSERVATIONS] {
    std::array::from_fn(|index| {
        let magnitude = (index % 64) as i32 - 16;
        let mantissa = 1.0 + ((index * 17) % 100) as f64 / 100.0;
        mantissa * 2.0_f64.powi(magnitude)
    })
}

fn record_distributions(c: &mut Criterion) {
    let values = observations();
    let mut group = c.benchmark_group("distribution_record");
    let _ = group.throughput(Throughput::Elements(OBSERVATIONS as u64));

    let _ = group.bench_function("counter_f64", |b| {
        b.iter_batched(
            Counter::<f64>::default,
            |mut counter| {
                for value in &values {
                    counter.add(black_box(*value));
                }
                black_box(counter)
            },
            BatchSize::SmallInput,
        );
    });

    let _ = group.bench_function("mmsc", |b| {
        b.iter_batched(
            Mmsc::default,
            |mut mmsc| {
                for value in &values {
                    mmsc.record(black_box(*value));
                }
                black_box(mmsc)
            },
            BatchSize::SmallInput,
        );
    });

    let _ = group.bench_function("histogram_normal", |b| {
        b.iter_batched(
            HistogramNormal::default,
            |mut histogram| {
                for value in &values {
                    histogram.record(black_box(*value));
                }
                black_box(histogram)
            },
            BatchSize::SmallInput,
        );
    });

    let _ = group.bench_function("histogram_detailed", |b| {
        b.iter_batched(
            HistogramDetailed::default,
            |mut histogram| {
                for value in &values {
                    histogram.record(black_box(*value));
                }
                black_box(histogram)
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, record_distributions);
criterion_main!(benches);
