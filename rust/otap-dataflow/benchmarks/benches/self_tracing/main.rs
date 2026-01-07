// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! Benchmarks for the compact log formatter.
//!
//! These benchmarks emit a single tracing event but perform N
//! encoding or encoding-and-formatting operations inside the callback
//!
//! Benchmark names follow the pattern: `group/description/N_events`
//!
//! Example: `compact_encode/3_attrs/1000_events` = 300 µs → 300 ns per event

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

use otap_df_telemetry::self_tracing::{
    CallsiteCache, CompactLogRecord, encode_body_and_attrs, format_log_record,
};

use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

struct EncodeOnlyLayer {
    iterations: usize,
}

impl EncodeOnlyLayer {
    fn new(iterations: usize) -> Self {
        Self { iterations }
    }
}

impl<S> Layer<S> for EncodeOnlyLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        for _ in 0..self.iterations {
            let bytes = encode_body_and_attrs(event);
            let _ = std::hint::black_box(bytes);
        }
    }
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for iterations in [100, 1000].iter() {
        let _ = group.bench_with_input(
            BenchmarkId::new("3_attrs", format!("{}_events", iterations)),
            iterations,
            |b, &iters| {
                b.iter(|| {
                    let layer = EncodeOnlyLayer::new(iters);
                    let subscriber = tracing_subscriber::registry().with(layer);
                    let dispatch = tracing::Dispatch::new(subscriber);

                    tracing::dispatcher::with_default(&dispatch, || {
                        tracing::info!(
                            key1 = "value1",
                            key2 = 42,
                            key3 = true,
                            "Benchmark message"
                        );
                    });

                    let _ = std::hint::black_box(());
                })
            },
        );
    }

    group.finish();
}

struct FormatOnlyLayer {
    iterations: usize,
}

impl FormatOnlyLayer {
    fn new(iterations: usize) -> Self {
        Self { iterations }
    }
}

impl<S> Layer<S> for FormatOnlyLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();

        // Build cache with this callsite
        let mut cache = CallsiteCache::new();
        cache.register(metadata);

        // Encode once
        let body_attrs_bytes = encode_body_and_attrs(event);
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let record = CompactLogRecord {
            callsite_id: metadata.callsite(),
            timestamp_ns,
            severity_number: match *metadata.level() {
                Level::TRACE => 1,
                Level::DEBUG => 5,
                Level::INFO => 9,
                Level::WARN => 13,
                Level::ERROR => 17,
            },
            severity_text: metadata.level().as_str(),
            body_attrs_bytes,
        };

        for _ in 0..self.iterations {
            let line = format_log_record(&record, &cache, true);
            let _ = std::hint::black_box(line);
        }
    }
}

fn bench_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("format");

    for iterations in [100, 1000].iter() {
        let _ = group.bench_with_input(
            BenchmarkId::new("3_attrs", format!("{}_events", iterations)),
            iterations,
            |b, &iters| {
                b.iter(|| {
                    let layer = FormatOnlyLayer::new(iters);
                    let subscriber = tracing_subscriber::registry().with(layer);
                    let dispatch = tracing::Dispatch::new(subscriber);

                    tracing::dispatcher::with_default(&dispatch, || {
                        tracing::info!(
                            key1 = "value1",
                            key2 = 42,
                            key3 = true,
                            "Benchmark message"
                        );
                    });

                    let _ = std::hint::black_box(());
                })
            },
        );
    }

    group.finish();
}

struct EncodeFormatLayer {
    iterations: usize,
}

impl EncodeFormatLayer {
    fn new(iterations: usize) -> Self {
        Self { iterations }
    }
}

impl<S> Layer<S> for EncodeFormatLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();

        // Build cache with this callsite
        let mut cache = CallsiteCache::new();
        cache.register(metadata);

        // Encode + format N times
        for _ in 0..self.iterations {
            let body_attrs_bytes = encode_body_and_attrs(event);
            let timestamp_ns = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;

            let record = CompactLogRecord {
                callsite_id: metadata.callsite(),
                timestamp_ns,
                severity_number: match *metadata.level() {
                    Level::TRACE => 1,
                    Level::DEBUG => 5,
                    Level::INFO => 9,
                    Level::WARN => 13,
                    Level::ERROR => 17,
                },
                severity_text: metadata.level().as_str(),
                body_attrs_bytes,
            };

            let line = format_log_record(&record, &cache, true);
            let _ = std::hint::black_box(line);
        }
    }
}

fn bench_encode_and_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_and_format");

    for iterations in [100, 1000].iter() {
        let _ = group.bench_with_input(
            BenchmarkId::new("3_attrs", format!("{}_events", iterations)),
            iterations,
            |b, &iters| {
                b.iter(|| {
                    let layer = EncodeFormatLayer::new(iters);
                    let subscriber = tracing_subscriber::registry().with(layer);
                    let dispatch = tracing::Dispatch::new(subscriber);

                    tracing::dispatcher::with_default(&dispatch, || {
                        tracing::info!(
                            key1 = "value1",
                            key2 = 42,
                            key3 = true,
                            "Benchmark message"
                        );
                    });

                    let _ = std::hint::black_box(());
                })
            },
        );
    }

    group.finish();
}

fn bench_encode_attrs(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_attrs");
    let iterations = 1000;

    // No attributes
    let _ = group.bench_function("0_attrs/1000_events", |b| {
        b.iter(|| {
            let layer = EncodeOnlyLayer::new(iterations);
            let subscriber = tracing_subscriber::registry().with(layer);
            let dispatch = tracing::Dispatch::new(subscriber);

            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!("message only");
            });

            let _ = std::hint::black_box(());
        })
    });

    // 3 attributes
    let _ = group.bench_function("3_attrs/1000_events", |b| {
        b.iter(|| {
            let layer = EncodeOnlyLayer::new(iterations);
            let subscriber = tracing_subscriber::registry().with(layer);
            let dispatch = tracing::Dispatch::new(subscriber);

            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!(a1 = "value", a2 = 42, a3 = true, "with 3 attributes");
            });

            let _ = std::hint::black_box(());
        })
    });

    // 10 attributes
    let _ = group.bench_function("10_attrs/1000_events", |b| {
        b.iter(|| {
            let layer = EncodeOnlyLayer::new(iterations);
            let subscriber = tracing_subscriber::registry().with(layer);
            let dispatch = tracing::Dispatch::new(subscriber);

            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!(
                    a1 = "string1",
                    a2 = true,
                    a3 = "string2",
                    a4 = 3.14,
                    a5 = 42i64,
                    a6 = "string3",
                    a7 = false,
                    a8 = 2.718,
                    a9 = 100u64,
                    a10 = "string4",
                    "with 10 attributes"
                );
            });

            let _ = std::hint::black_box(());
        })
    });

    group.finish();
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_encode, bench_format, bench_encode_and_format, bench_encode_attrs
    );
}

criterion_main!(bench_entry::benches);
