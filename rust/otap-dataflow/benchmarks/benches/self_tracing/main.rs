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
use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
use otap_df_telemetry::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};
use otap_df_telemetry::event::LogEvent;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::self_tracing::{
    DirectLogRecordEncoder, LogContext, LogRecord, ScopeToBytesMap, encode_export_logs_request,
    format_log_record_to_string,
};
use std::time::SystemTime;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Test scope attributes for entity benchmarks
static BENCH_SCOPE_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
    name: "BenchScope",
    fields: &[
        AttributeField {
            key: "pipeline.name",
            r#type: AttributeValueType::String,
            brief: "Pipeline name",
        },
        AttributeField {
            key: "cpu.id",
            r#type: AttributeValueType::Int,
            brief: "CPU ID",
        },
    ],
};

/// Mock attribute set for benchmarking scope attributes.
#[derive(Debug)]
struct BenchScopeAttributes {
    values: Vec<AttributeValue>,
}

impl BenchScopeAttributes {
    fn new(name: &str, id: i64) -> Self {
        Self {
            values: vec![AttributeValue::String(name.into()), AttributeValue::Int(id)],
        }
    }
}

impl AttributeSetHandler for BenchScopeAttributes {
    fn descriptor(&self) -> &'static AttributesDescriptor {
        &BENCH_SCOPE_ATTRIBUTES_DESCRIPTOR
    }

    fn attribute_values(&self) -> &[AttributeValue] {
        &self.values
    }
}

/// The operation to perform on each event within the layer.  The cost
/// of generating the timestamp is not included in the  measurement.
#[derive(Clone, Copy)]
enum BenchOp {
    /// Encode the event body into a new LogRecord.  This includes
    /// encoding the body and attributes, not callsite details or
    /// timestamp.
    NewRecord,
    /// Encode once, then format standard representation (with
    /// timestamp) N times.
    Format,
    /// Encode and format standard representation (with timestamp) N
    /// times.
    EncodeAndFormat,
    /// Encode to complete protobuf (with timestamp) N times.
    EncodeProto,
    /// Encode to complete protobuf with scope cache (with entity context) N times.
    /// Measures overhead of scope attribute encoding/caching.
    EncodeProtoWithScope,
    /// Format with entity context N times.
    /// Measures overhead of formatting entity keys.
    FormatWithEntity,
}

/// A layer that performs a configurable operation N times per event.
struct BenchLayer {
    iterations: usize,
    op: BenchOp,
    /// Optional entity context for scope benchmarks.
    entity_context: Option<EntityContext>,
}

/// Entity context for scope benchmarks (registry, entity key, resource bytes).
struct EntityContext {
    registry: TelemetryRegistryHandle,
    context: LogContext,
    resource_bytes: bytes::Bytes,
}

impl BenchLayer {
    fn new(iterations: usize, op: BenchOp) -> Self {
        Self {
            iterations,
            op,
            entity_context: None,
        }
    }

    fn with_entity(iterations: usize, op: BenchOp) -> Self {
        let registry = TelemetryRegistryHandle::new();
        let entity_key = registry.register_entity(BenchScopeAttributes::new("bench-pipeline", 42));
        // Empty resource bytes - we're benchmarking scope encoding, not resource encoding
        let resource_bytes = bytes::Bytes::new();
        Self {
            iterations,
            op,
            entity_context: Some(EntityContext {
                registry,
                context: LogContext::from_buf([entity_key]),
                resource_bytes,
            }),
        }
    }
}

impl<S> Layer<S> for BenchLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let now = SystemTime::now();
        match self.op {
            BenchOp::NewRecord => {
                for _ in 0..self.iterations {
                    let record = LogRecord::new(event, LogContext::new());
                    let _ = std::hint::black_box(record);
                }
            }
            BenchOp::Format => {
                // Encode once, format N times
                let record = LogRecord::new(event, LogContext::new());

                for _ in 0..self.iterations {
                    let line = format_log_record_to_string(Some(now), &record);
                    let _ = std::hint::black_box(line);
                }
            }
            BenchOp::EncodeAndFormat => {
                for _ in 0..self.iterations {
                    let record = LogRecord::new(event, LogContext::new());
                    let line = format_log_record_to_string(Some(now), &record);
                    let _ = std::hint::black_box(line);
                }
            }
            BenchOp::EncodeProto => {
                let mut buf = ProtoBuffer::new();
                let mut encoder = DirectLogRecordEncoder::new(&mut buf);

                for _ in 0..self.iterations {
                    encoder.clear();
                    let size =
                        encoder.encode_log_record(now, &LogRecord::new(event, LogContext::new()));
                    let _ = std::hint::black_box(size);
                }
            }
            BenchOp::EncodeProtoWithScope => {
                let ctx = self
                    .entity_context
                    .as_ref()
                    .expect("entity context required");
                let mut scope_cache = ScopeToBytesMap::new(ctx.registry.clone());
                let mut buf = ProtoBuffer::with_capacity(512);

                for _ in 0..self.iterations {
                    let record = LogRecord::new(event, ctx.context.clone());
                    let log_event = LogEvent { time: now, record };
                    encode_export_logs_request(
                        &mut buf,
                        &log_event,
                        &ctx.resource_bytes,
                        &mut scope_cache,
                    );
                    let _ = std::hint::black_box(buf.len());
                }
            }
            BenchOp::FormatWithEntity => {
                let ctx = self
                    .entity_context
                    .as_ref()
                    .expect("entity context required");

                for _ in 0..self.iterations {
                    let record = LogRecord::new(event, ctx.context.clone());
                    let line = format_log_record_to_string(Some(now), &record);
                    let _ = std::hint::black_box(line);
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
            attr_float1 = 1.234,
            attr_int1 = 42i64,
            attr_str3 = "string3",
            attr_bool2 = false,
            attr_float2 = 5.678,
            attr_int2 = 100u64,
            attr_str4 = "string4",
            "benchmark message"
        )
    };
}

/// Run a benchmark with the given layer, invoking the log emitter.
fn run_bench<L, F>(b: &mut criterion::Bencher<'_>, layer: L, emit: F)
where
    L: Layer<tracing_subscriber::Registry> + Send + Sync + 'static,
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

            let _ = group.bench_with_input(id, &iterations, |b, &iters| {
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

fn bench_new_record(c: &mut Criterion) {
    bench_op(c, "new_record", BenchOp::NewRecord);
}

fn bench_format(c: &mut Criterion) {
    bench_op(c, "format", BenchOp::Format);
}

fn bench_format_new_record(c: &mut Criterion) {
    bench_op(c, "format_new_record", BenchOp::EncodeAndFormat);
}

fn bench_encode_proto(c: &mut Criterion) {
    bench_op(c, "encode_proto", BenchOp::EncodeProto);
}

/// Benchmark entity-aware operations across different iteration counts.
fn bench_op_with_entity(c: &mut Criterion, group_name: &str, op: BenchOp) {
    let mut group = c.benchmark_group(group_name);

    for &iterations in &[100, 1000] {
        for &(attr_count, attr_label) in &[(0, "0_attrs"), (3, "3_attrs"), (10, "10_attrs")] {
            let id = BenchmarkId::new(attr_label, format!("{}_events", iterations));

            let _ = group.bench_with_input(id, &iterations, |b, &iters| {
                let layer = BenchLayer::with_entity(iters, op);
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

fn bench_encode_proto_with_scope(c: &mut Criterion) {
    bench_op_with_entity(c, "encode_proto_with_scope", BenchOp::EncodeProtoWithScope);
}

fn bench_format_with_entity(c: &mut Criterion) {
    bench_op_with_entity(c, "format_with_entity", BenchOp::FormatWithEntity);
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_new_record, bench_format, bench_format_new_record, bench_encode_proto,
                  bench_encode_proto_with_scope, bench_format_with_entity
    );
}

criterion_main!(bench_entry::benches);
