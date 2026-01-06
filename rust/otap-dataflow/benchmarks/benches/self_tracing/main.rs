// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! Benchmarks for self-tracing OTLP bytes encoding and formatting.
//!
//! # Benchmark Design
//!
//! These benchmarks emit a single tracing event but perform N encoding/formatting
//! operations inside the callback. This amortizes tracing dispatch overhead to noise,
//! allowing us to measure the true cost of encoding.
//!
//! # Interpreting Results
//!
//! Benchmark names follow the pattern: `group/description/N_encodings`
//!
//! To get per-event cost: `measured_time / N`
//!
//! Example: `encode_otlp/3_attrs/1000_events` = 265 µs → 265 ns per event

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

use otap_df_telemetry::self_tracing::{
    OtlpBytesFormattingLayer, StatefulDirectEncoder, encode_resource_bytes_from_attrs,
};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// =============================================================================
// ISOLATED ENCODING BENCHMARK
// Emit 1 event, encode it N times inside the callback
// =============================================================================

/// Layer that encodes the same event N times to measure pure encoding cost.
struct IsolatedEncoderLayer {
    /// Number of times to encode each event
    iterations: usize,
    /// Pre-encoded resource bytes
    resource_bytes: Vec<u8>,
}

impl IsolatedEncoderLayer {
    fn new(iterations: usize) -> Self {
        Self {
            iterations,
            resource_bytes: encode_resource_bytes_from_attrs(&[
                ("service.name", "benchmark"),
            ]),
        }
    }
}

impl<S> Layer<S> for IsolatedEncoderLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Encode the same event N times using StatefulDirectEncoder
        for _ in 0..self.iterations {
            let mut encoder = StatefulDirectEncoder::new(4096, self.resource_bytes.clone());
            encoder.encode_event(event);
            let _ = encoder.flush();
        }
    }
}

/// Benchmark: Pure encoding cost (N encodings per single event dispatch)
fn bench_isolated_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_otlp");

    for iterations in [100, 1000].iter() {
        let _ = group.bench_with_input(
            BenchmarkId::new("3_attrs", format!("{}_events", iterations)),
            iterations,
            |b, &iters| {
                b.iter(|| {
                    let layer = IsolatedEncoderLayer::new(iters);
                    let subscriber = tracing_subscriber::registry().with(layer);
                    let dispatch = tracing::Dispatch::new(subscriber);

                    tracing::dispatcher::with_default(&dispatch, || {
                        // Single event, encoded `iters` times inside the callback
                        tracing::info!(
                            key1 = "value1",
                            key2 = 42,
                            key3 = true,
                            "Benchmark message"
                        );
                    });

                    std::hint::black_box(())
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// ISOLATED ENCODE + FORMAT BENCHMARK
// Emit 1 event, encode and format it N times
// =============================================================================

/// Layer that encodes and formats the same event N times.
struct IsolatedEncodeFormatLayer {
    iterations: usize,
    resource_bytes: Vec<u8>,
}

impl IsolatedEncodeFormatLayer {
    fn new(iterations: usize) -> Self {
        Self {
            iterations,
            resource_bytes: encode_resource_bytes_from_attrs(&[
                ("service.name", "benchmark"),
            ]),
        }
    }
}

impl<S> Layer<S> for IsolatedEncodeFormatLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let formatter = OtlpBytesFormattingLayer::new(std::io::sink);

        // Encode and format N times
        for _ in 0..self.iterations {
            // Use StatefulDirectEncoder to produce full OTLP envelope
            let mut encoder = StatefulDirectEncoder::new(4096, self.resource_bytes.clone());
            encoder.encode_event(event);
            let bytes = encoder.flush();
            
            // Format the complete OTLP bytes
            let _ = formatter.format_otlp_bytes(&bytes);
        }
    }
}

/// Benchmark: Encoding + formatting cost
fn bench_isolated_encode_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_and_format_otlp");

    for iterations in [100, 1000].iter() {
        let _ = group.bench_with_input(
            BenchmarkId::new("3_attrs", format!("{}_events", iterations)),
            iterations,
            |b, &iters| {
                b.iter(|| {
                    let layer = IsolatedEncodeFormatLayer::new(iters);
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

                    std::hint::black_box(())
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// ISOLATED FORMAT-ONLY BENCHMARK
// Pre-encode bytes, format them N times
// =============================================================================

/// Layer that encodes once, then formats N times.
struct IsolatedFormatLayer {
    format_iterations: usize,
    resource_bytes: Vec<u8>,
}

impl IsolatedFormatLayer {
    fn new(format_iterations: usize) -> Self {
        Self {
            format_iterations,
            resource_bytes: encode_resource_bytes_from_attrs(&[
                ("service.name", "benchmark"),
            ]),
        }
    }
}

impl<S> Layer<S> for IsolatedFormatLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // Encode once using StatefulDirectEncoder to get full OTLP envelope
        let mut encoder = StatefulDirectEncoder::new(4096, self.resource_bytes.clone());
        encoder.encode_event(event);
        let bytes = encoder.flush();

        // Format N times
        let formatter = OtlpBytesFormattingLayer::new(std::io::sink);
        for _ in 0..self.format_iterations {
            let _ = formatter.format_otlp_bytes(&bytes);
        }
    }
}

/// Benchmark: Pure formatting cost (encode once, format N times)
fn bench_isolated_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_otlp_only");

    for iterations in [100, 1000].iter() {
        let _ = group.bench_with_input(
            BenchmarkId::new("3_attrs", format!("{}_formats", iterations)),
            iterations,
            |b, &iters| {
                b.iter(|| {
                    let layer = IsolatedFormatLayer::new(iters);
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

                    std::hint::black_box(())
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// ATTRIBUTE COMPLEXITY BENCHMARK
// =============================================================================

/// Layer that encodes events with varying attribute counts.
struct AttributeComplexityLayer {
    iterations: usize,
    resource_bytes: Vec<u8>,
}

impl AttributeComplexityLayer {
    fn new(iterations: usize) -> Self {
        Self {
            iterations,
            resource_bytes: encode_resource_bytes_from_attrs(&[
                ("service.name", "benchmark"),
            ]),
        }
    }
}

impl<S> Layer<S> for AttributeComplexityLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        for _ in 0..self.iterations {
            let mut encoder = StatefulDirectEncoder::new(4096, self.resource_bytes.clone());
            encoder.encode_event(event);
            let _ = encoder.flush();
        }
    }
}

/// Benchmark: Encoding cost with different attribute counts
fn bench_attribute_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_otlp_by_attrs");
    let iterations = 1000;

    // No attributes
    let _ = group.bench_function("0_attrs/1000_events", |b| {
        b.iter(|| {
            let layer = AttributeComplexityLayer::new(iterations);
            let subscriber = tracing_subscriber::registry().with(layer);
            let dispatch = tracing::Dispatch::new(subscriber);

            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!("message only");
            });

            std::hint::black_box(())
        })
    });

    // 3 attributes
    let _ = group.bench_function("3_attrs/1000_events", |b| {
        b.iter(|| {
            let layer = AttributeComplexityLayer::new(iterations);
            let subscriber = tracing_subscriber::registry().with(layer);
            let dispatch = tracing::Dispatch::new(subscriber);

            tracing::dispatcher::with_default(&dispatch, || {
                tracing::info!(a1 = "value", a2 = 42, a3 = true, "with 3 attributes");
            });

            std::hint::black_box(())
        })
    });

    // 10 attributes
    let _ = group.bench_function("10_attrs/1000_events", |b| {
        b.iter(|| {
            let layer = AttributeComplexityLayer::new(iterations);
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

            std::hint::black_box(())
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
        targets = bench_isolated_encode, bench_isolated_encode_format, 
                  bench_isolated_format, bench_attribute_complexity
    );
}

criterion_main!(bench_entry::benches);
