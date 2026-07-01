// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parquet study benchmark.
//!
//! Compares the read/write cost and serialized size of OTAP logs encoded as
//! compressed Arrow IPC (the representation we have today) versus several
//! flattened single-file Parquet layouts, and breaks each pipeline into its
//! sub-steps so the cost of every stage is visible.
//!
//! Starting from an OTAP logs batch (four record batches: Logs, ResourceAttrs,
//! ScopeAttrs, LogAttrs):
//!
//! - OTAP/IPC encode = transport-optimize, then Arrow IPC serialize (+compress).
//!   Decode = IPC deserialize, then transport-decode.
//! - Parquet encode = flatten to one Arrow record batch, then write Parquet.
//!   Decode = read Parquet, then unflatten.
//!
//! Run with:
//!
//! ```bash
//! cargo bench -p benchmarks --bench otap_parquet
//! ```
//!
//! Three tables (size, OTAP/IPC breakdown, Parquet breakdown) are printed to
//! stdout before the timed round-trip benchmarks run.

#![allow(missing_docs)]
// This benchmark intentionally prints comparison tables to stdout before
// running the timed measurements.
#![allow(clippy::print_stdout)]

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::{Duration, Instant};

use benchmarks::parquet_study::datagen::{LogsGenParams, gen_logs_otap};
use benchmarks::parquet_study::parquet_io::{read_parquet, write_parquet};
use benchmarks::parquet_study::{Compressor, Scheme, ipc};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// Input shapes for the breakdown: block sizes larger than a few thousand log
/// records, under a single resource/scope. A single OTAP logs batch caps at
/// 65,535 records because log ids are u16, so these stay below that limit; larger
/// volumes must be streamed as multiple batches (see the streaming table).
fn input_shapes() -> Vec<LogsGenParams> {
    [10_000usize, 30_000, 60_000]
        .into_iter()
        .map(|num_logs| LogsGenParams {
            num_resources: 1,
            num_scopes: 1,
            num_logs,
        })
        .collect()
}

/// Batch sizes for the streaming table, spanning small to large so the fixed
/// per-batch schema/dictionary overhead is visible as a fraction of the batch.
fn streaming_shapes() -> Vec<LogsGenParams> {
    [1_000usize, 10_000, 50_000]
        .into_iter()
        .map(|num_logs| LogsGenParams {
            num_resources: 1,
            num_scopes: 1,
            num_logs,
        })
        .collect()
}

/// Median wall-clock milliseconds of `f` over a few iterations (with one warm-up
/// pass). Indicative only; the Criterion round-trip group gives rigorous totals.
fn median_ms(mut f: impl FnMut()) -> f64 {
    f();
    let iters = 3;
    let mut samples = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = Instant::now();
        f();
        samples.push(start.elapsed().as_secs_f64() * 1e3);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    samples[samples.len() / 2]
}

/// Serialized-size comparison for every contender x compressor x shape.
fn print_size_table(shapes: &[LogsGenParams]) {
    println!("\n=== OTAP logs serialized size (bytes) ===");
    println!(
        "{:<16} {:<8} {:>12} {:>10} {:>10} {:>12}",
        "contender", "comp", "bytes", "vs-otlp", "b/log", "vs-ipc-zstd"
    );
    for shape in shapes {
        let (otap, proto_len) = gen_logs_otap(shape);
        let total_logs = shape.total_logs();
        println!(
            "-- shape {} log records, OTLP proto = {} bytes --",
            total_logs, proto_len
        );
        let ipc_zstd = Scheme::Ipc
            .codec(Compressor::Zstd)
            .write(otap.clone())
            .expect("ipc zstd write")
            .len();
        for scheme in Scheme::all() {
            for &compressor in scheme.compressors() {
                let codec = scheme.codec(compressor);
                let bytes = codec.write(otap.clone()).expect("write").len();
                println!(
                    "{:<16} {:<8} {:>12} {:>9.2}x {:>10.1} {:>11.2}x",
                    codec.name(),
                    compressor.label(),
                    bytes,
                    proto_len as f64 / bytes as f64,
                    bytes as f64 / total_logs as f64,
                    bytes as f64 / ipc_zstd as f64,
                );
            }
        }
        println!();
    }
}

/// Per-step breakdown of the OTAP/IPC encode and decode pipelines.
fn print_ipc_breakdown(shapes: &[LogsGenParams]) {
    println!("\n=== OTAP/IPC pipeline breakdown (indicative ms) ===");
    println!("encode = transport-optimize + Arrow-IPC-serialize(+compress)");
    println!("decode = IPC-deserialize + transport-decode");
    println!(
        "{:<6} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>10}",
        "comp", "t-enc", "ipc-ser", "enc-tot", "ipc-des", "t-dec", "dec-tot", "bytes"
    );
    for shape in shapes {
        let (otap, _) = gen_logs_otap(shape);
        println!("-- shape {} log records --", shape.total_logs());
        for &comp in Scheme::Ipc.compressors() {
            let t_enc = median_ms(|| {
                let mut o = otap.clone();
                ipc::transport_encode(&mut o).expect("transport encode");
            });
            let enc_tot = median_ms(|| {
                let _ = ipc::encode_to_bytes(otap.clone(), comp).expect("encode");
            });
            let ipc_ser = (enc_tot - t_enc).max(0.0);

            let bytes = ipc::encode_to_bytes(otap.clone(), comp).expect("encode");
            let optimized = ipc::deserialize(&bytes).expect("deserialize");
            let ipc_des = median_ms(|| {
                let _ = ipc::deserialize(&bytes).expect("deserialize");
            });
            let t_dec = median_ms(|| {
                let mut o = optimized.clone();
                ipc::transport_decode(&mut o).expect("transport decode");
            });

            println!(
                "{:<6} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>10}",
                comp.label(),
                t_enc,
                ipc_ser,
                enc_tot,
                ipc_des,
                t_dec,
                ipc_des + t_dec,
                bytes.len(),
            );
        }
        println!();
    }
}

/// Per-step breakdown of the Parquet encode and decode pipelines.
fn print_parquet_breakdown(shapes: &[LogsGenParams]) {
    println!("\n=== Parquet pipeline breakdown (indicative ms) ===");
    println!("encode = flatten + parquet-write   decode = parquet-read + unflatten");
    println!(
        "{:<16} {:<8} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>10}",
        "scheme", "comp", "flatten", "pq-write", "enc-tot", "pq-read", "unflat", "dec-tot", "bytes"
    );
    for shape in shapes {
        let (otap, _) = gen_logs_otap(shape);
        println!("-- shape {} log records --", shape.total_logs());
        for scheme in Scheme::flattened() {
            let flat = scheme.flatten(&otap).expect("flatten");
            let flatten_t = median_ms(|| {
                let _ = scheme.flatten(&otap).expect("flatten");
            });
            for compressor in Compressor::ALL {
                let write_t = median_ms(|| {
                    let _ = write_parquet(&flat, compressor.parquet()).expect("write");
                });
                let bytes = write_parquet(&flat, compressor.parquet()).expect("write");
                let read_flat = read_parquet(&bytes).expect("read");
                let read_t = median_ms(|| {
                    let _ = read_parquet(&bytes).expect("read");
                });
                let unflatten_t = median_ms(|| {
                    let _ = scheme.unflatten(&read_flat).expect("unflatten");
                });
                println!(
                    "{:<16} {:<8} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>10}",
                    scheme.name(),
                    compressor.label(),
                    flatten_t,
                    write_t,
                    flatten_t + write_t,
                    read_t,
                    unflatten_t,
                    read_t + unflatten_t,
                    bytes.len(),
                );
            }
        }
        println!();
    }
}

/// OTAP/IPC streaming amortization: cold (first) versus warm (steady-state)
/// per-batch size when a single long-lived Producer streams many batches, with
/// the equivalent single Parquet file for reference.
fn print_streaming_table(shapes: &[LogsGenParams]) {
    println!("\n=== OTAP/IPC streaming amortization (bytes per batch) ===");
    println!("One long-lived Producer streams batches: schema once, delta dictionaries.");
    println!("cold = first batch (schema + full dictionaries + data); warm = steady-state batch.");
    println!(
        "pq-nested is the same batch as one Parquet file, which has no per-batch amortization."
    );
    println!(
        "{:<8} {:<6} {:>12} {:>12} {:>10} {:>12} {:>9}",
        "logs", "comp", "cold", "warm", "saved", "pq-nested", "warm/pq"
    );
    for shape in shapes {
        let (otap, _) = gen_logs_otap(shape);
        let flat = Scheme::Nested.flatten(&otap).expect("flatten");
        for &comp in Scheme::Ipc.compressors() {
            let sizes = ipc::stream_batch_sizes(&otap, comp, 6).expect("stream sizes");
            let cold = sizes[0];
            let warm = *sizes.last().expect("non-empty");
            let pq = write_parquet(&flat, comp.parquet())
                .expect("parquet write")
                .len();
            println!(
                "{:<8} {:<6} {:>12} {:>12} {:>10} {:>12} {:>8.2}x",
                shape.total_logs(),
                comp.label(),
                cold,
                warm,
                cold - warm,
                pq,
                warm as f64 / pq as f64,
            );
        }
    }
    println!();
}

fn bench_round_trip(c: &mut Criterion) {
    let shapes = input_shapes();
    print_size_table(&shapes);
    print_ipc_breakdown(&shapes);
    print_parquet_breakdown(&shapes);
    print_streaming_table(&streaming_shapes());

    let mut write_group = c.benchmark_group("parquet_study/write");
    let _ = write_group.sample_size(10);
    let _ = write_group.warm_up_time(Duration::from_millis(500));
    let _ = write_group.measurement_time(Duration::from_secs(3));
    for shape in &shapes {
        let (otap, _) = gen_logs_otap(shape);
        for scheme in Scheme::all() {
            for &compressor in scheme.compressors() {
                let codec = scheme.codec(compressor);
                let id = BenchmarkId::new(
                    format!("{}/{}", codec.name(), compressor.label()),
                    shape.total_logs(),
                );
                let _ = write_group.bench_with_input(id, shape, |b, _| {
                    b.iter_batched(
                        || otap.clone(),
                        |input| black_box(codec.write(input).expect("write")),
                        BatchSize::SmallInput,
                    );
                });
            }
        }
    }
    write_group.finish();

    let mut read_group = c.benchmark_group("parquet_study/read");
    let _ = read_group.sample_size(10);
    let _ = read_group.warm_up_time(Duration::from_millis(500));
    let _ = read_group.measurement_time(Duration::from_secs(3));
    for shape in &shapes {
        let (otap, _) = gen_logs_otap(shape);
        for scheme in Scheme::all() {
            for &compressor in scheme.compressors() {
                let codec = scheme.codec(compressor);
                let bytes = codec.write(otap.clone()).expect("write");
                let id = BenchmarkId::new(
                    format!("{}/{}", codec.name(), compressor.label()),
                    shape.total_logs(),
                );
                let _ = read_group.bench_with_input(id, shape, |b, _| {
                    b.iter(|| black_box(codec.read(&bytes).expect("read")));
                });
            }
        }
    }
    read_group.finish();
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;
    criterion_group!(benches, bench_round_trip);
}

criterion_main!(bench_entry::benches);
