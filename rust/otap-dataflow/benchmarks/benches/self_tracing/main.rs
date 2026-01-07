// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for the compact log formatter.
//!
//! These benchmarks emit a single tracing event but perform N
//! encoding or encoding-and-formatting operations inside the callback
//!
//! Benchmark names follow the pattern: `group/description/N_events`
//!
//! Example: `encode/3_attrs/1000_events` = 300 µs → 300 ns per event

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_telemetry::self_tracing::{
    ConsoleWriter, DirectLogRecordEncoder, LogRecord, SavedCallsite,
};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// The operation to perform on each event within the layer.
#[derive(Clone, Copy)]
enum BenchOp {
    /// Encode the event into a LogRecord only.
    Encode,
    /// Encode once, then format N times.
    Format,
    /// Encode and format together N times.
    EncodeAndFormat,
    /// Encode to protobuf N times.
    EncodeProto,
}

/// A layer that performs a configurable operation N times per event.
struct BenchLayer {
    iterations: usize,
    op: BenchOp,
}

impl BenchLayer {
    fn new(iterations: usize, op: BenchOp) -> Self {
        Self { iterations, op }
    }
}

impl<S> Layer<S> for BenchLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        match self.op {
            BenchOp::Encode => {
                for _ in 0..self.iterations {
                    let record = LogRecord::new(event);
                    let _ = std::hint::black_box(record);
                }
            }
            BenchOp::Format => {
                // Encode once, format N times
                let record = LogRecord::new(event);
                let writer = ConsoleWriter::no_color();
                let callsite = SavedCallsite::new(event.metadata());

                for _ in 0..self.iterations {
                    let line = writer.format_log_record(&record, &callsite);
                    let _ = std::hint::black_box(line);
                }
            }
            BenchOp::EncodeAndFormat => {
                let writer = ConsoleWriter::no_color();

                for _ in 0..self.iterations {
                    let record = LogRecord::new(event);
                    let callsite = SavedCallsite::new(event.metadata());
                    let line = writer.format_log_record(&record, &callsite);
                    let _ = std::hint::black_box(line);
                }
            }
            BenchOp::EncodeProto => {
                let mut buf = ProtoBuffer::new();
                let mut encoder = DirectLogRecordEncoder::new(&mut buf);
                let callsite = SavedCallsite::new(event.metadata());

                for _ in 0..self.iterations {
                    encoder.clear();
                    let size = encoder.encode_log_record(LogRecord::new(event), &callsite);
                    let _ = std::hint::black_box(size);
                }
            }
        }
    }
}

/// Macro to generate benchmark functions for different attribute counts.
/// Each variant emits a consistent log statement for fair comparison.
macro_rules! emit_log {
    (0) => {
        tracing::info!("benchmark message")
    };
    (3) => {
        tracing::info!(
            attr_str = "value",
            attr_int = 42,
            attr_bool = true,
            "benchmark message"
        )
    };
    (10) => {
        tracing::info!(
            attr_str1 = "string1",
            attr_bool1 = true,
            attr_str2 = "string2",
            attr_float1 = 3.14,
            attr_int1 = 42i64,
            attr_str3 = "string3",
            attr_bool2 = false,
            attr_float2 = 2.718,
            attr_int2 = 100u64,
            attr_str4 = "string4",
            "benchmark message"
        )
    };
}

/// Run a benchmark with the given layer, invoking the log emitter.
fn run_bench<L, F>(b: &mut criterion::Bencher, layer: L, emit: F)
where
    L: Layer<tracing_subscriber::Registry> + 'static,
    F: Fn(),
{
    let subscriber = tracing_subscriber::registry().with(layer);
    let dispatch = tracing::Dispatch::new(subscriber);

    b.iter(|| {
        tracing::dispatcher::with_default(&dispatch, &emit);
        std::hint::black_box(());
    });
}

/// Benchmark a specific operation across different iteration counts.
fn bench_op(c: &mut Criterion, group_name: &str, op: BenchOp) {
    let mut group = c.benchmark_group(group_name);

    for &iterations in &[100, 1000] {
        for &(attr_count, attr_label) in &[(0, "0_attrs"), (3, "3_attrs"), (10, "10_attrs")] {
            let id = BenchmarkId::new(attr_label, format!("{}_events", iterations));

            group.bench_with_input(id, &iterations, |b, &iters| {
                let layer = BenchLayer::new(iters, op);
                match attr_count {
                    0 => run_bench(b, layer, || emit_log!(0)),
                    3 => run_bench(b, layer, || emit_log!(3)),
                    _ => run_bench(b, layer, || emit_log!(10)),
                }
            });
        }
    }

    group.finish();
}

fn bench_encode(c: &mut Criterion) {
    bench_op(c, "encode", BenchOp::Encode);
}

fn bench_format(c: &mut Criterion) {
    bench_op(c, "format", BenchOp::Format);
}

fn bench_encode_and_format(c: &mut Criterion) {
    bench_op(c, "encode_and_format", BenchOp::EncodeAndFormat);
}

fn bench_encode_proto(c: &mut Criterion) {
    bench_op(c, "encode_proto", BenchOp::EncodeProto);
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_encode, bench_format, bench_encode_and_format, bench_encode_proto
    );
}

criterion_main!(bench_entry::benches);
