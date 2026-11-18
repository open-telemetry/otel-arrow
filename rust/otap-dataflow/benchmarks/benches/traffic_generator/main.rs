// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! Benchmarks the traffic generator's synthetic data path: building one batch
//! of signals as OTAP record batches and encoding it to OTLP bytes with the
//! OTAP->OTLP encoder.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::otap_batch::OtlpEncoder;
use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::synthetic_signal;
use otel_arrow_dfe_pdata_codec::OtapPayload;
use std::hint::black_box;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const BATCH_SIZES: &[usize] = &[50, 512];

/// Log generation parameters: (label, body size, attribute count, trace context).
const LOG_CASES: &[(&str, Option<usize>, Option<usize>, bool)] = &[
    ("default", None, None, false),
    ("body1k", Some(1024), None, false),
    ("body1k_attrs10_tc", Some(1024), Some(10), true),
];

/// Metric generation parameters: (label, attribute count, data points per metric).
const METRIC_CASES: &[(&str, Option<usize>, Option<usize>)] =
    &[("default", None, None), ("attrs6_dps3", Some(6), Some(3))];

fn bench_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic_generator/logs");
    for &batch in BATCH_SIZES {
        _ = group.throughput(Throughput::Elements(batch as u64));
        for &(label, body, attrs, tc) in LOG_CASES {
            _ = group.bench_with_input(BenchmarkId::new(label, batch), &batch, |b, &n| {
                let mut encoder = OtlpEncoder::new();
                b.iter(|| {
                    let records = synthetic_signal::static_otap_logs(n, body, attrs, tc, None)
                        .expect("build logs");
                    let payload: OtapPayload = encoder.encode(records).expect("encode logs").into();
                    black_box(payload)
                })
            });
        }
    }
    group.finish();
}

fn bench_traces(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic_generator/traces");
    for &batch in BATCH_SIZES {
        _ = group.throughput(Throughput::Elements(batch as u64));
        _ = group.bench_with_input(BenchmarkId::new("default", batch), &batch, |b, &n| {
            let mut encoder = OtlpEncoder::new();
            b.iter(|| {
                let records = synthetic_signal::static_otap_traces(n, None).expect("build traces");
                let payload: OtapPayload = encoder.encode(records).expect("encode traces").into();
                black_box(payload)
            })
        });
    }
    group.finish();
}

fn bench_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic_generator/metrics");
    for &batch in BATCH_SIZES {
        _ = group.throughput(Throughput::Elements(batch as u64));
        for &(label, attrs, dps) in METRIC_CASES {
            _ = group.bench_with_input(BenchmarkId::new(label, batch), &batch, |b, &n| {
                let mut encoder = OtlpEncoder::new();
                b.iter(|| {
                    let records = synthetic_signal::static_otap_metrics(n, attrs, dps, None)
                        .expect("build metrics");
                    let payload: OtapPayload =
                        encoder.encode(records).expect("encode metrics").into();
                    black_box(payload)
                })
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_logs, bench_traces, bench_metrics);
criterion_main!(benches);
