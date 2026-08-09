// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compares the hot-path recording cost of ITS distribution tiers.
//!
//! A component owns its instruments for the life of the pipeline and records
//! into the same ones over and over, so each case here records a reporting
//! interval's worth of observations into one long-lived instrument and then
//! resets it. Allocating a fresh instrument per iteration would instead
//! measure cold memory, which swamps the recording cost being compared.
//!
//! Run with:
//!
//! ```text
//! cargo bench -p otap-df-telemetry --bench distribution_record
//! ```

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
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

    let mut counter = Counter::<f64>::default();
    let _ = group.bench_function("counter_f64", |b| {
        b.iter(|| {
            for value in &values {
                counter.add(black_box(*value));
            }
            counter.reset();
        });
    });

    let mut mmsc = Mmsc::default();
    let _ = group.bench_function("mmsc", |b| {
        b.iter(|| {
            for value in &values {
                mmsc.record(black_box(*value));
            }
            mmsc.reset();
        });
    });

    let mut normal = HistogramNormal::default();
    let _ = group.bench_function("histogram_normal", |b| {
        b.iter(|| {
            for value in &values {
                normal.record(black_box(*value));
            }
            normal.reset();
        });
    });

    let mut detailed = HistogramDetailed::default();
    let _ = group.bench_function("histogram_detailed", |b| {
        b.iter(|| {
            for value in &values {
                detailed.record(black_box(*value));
            }
            detailed.reset();
        });
    });

    group.finish();
}

criterion_group!(benches, record_distributions);
criterion_main!(benches);
