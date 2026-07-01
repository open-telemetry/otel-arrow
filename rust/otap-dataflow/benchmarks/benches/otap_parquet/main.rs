// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parquet study benchmark.
//!
//! Compares the read/write cost and serialized size of OTAP logs encoded as
//! compressed Arrow IPC (the representation we have today) versus several
//! flattened single-file Parquet layouts, across the `zstd`, `lz4`, `snappy`,
//! and `none` compressors.
//!
//! Two questions are answered:
//!
//! 1. Round-trip cost and size of each representation (the `write` / `read` and
//!    size-table output).
//! 2. Where the OTAP -> Parquet conversion CPU lands, given the data ends up as
//!    Parquet on the server either way (the `server_cost` output): the server
//!    converting received OTAP/IPC (Option A) versus accepting Parquet the
//!    client already produced (Option B).
//!
//! Run with:
//!
//! ```bash
//! cargo bench -p benchmarks --bench otap_parquet
//! ```
//!
//! Two tables are printed to stdout before the timed benchmarks run.

#![allow(missing_docs)]
// This benchmark intentionally prints comparison tables to stdout before
// running the timed measurements.
#![allow(clippy::print_stdout)]

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::{Duration, Instant};

use benchmarks::parquet_study::datagen::{LogsGenParams, gen_logs_otap};
use benchmarks::parquet_study::{Compressor, Scheme, server};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// Input shapes covering a few resource/scope/log fan-outs. The first is the
/// "many small groups" shape from the `otap_encoder` bench; the rest scale the
/// number of log records under a single resource/scope (so resource/scope
/// attribute denormalization is amortized over more rows).
fn input_shapes() -> Vec<LogsGenParams> {
    vec![
        LogsGenParams {
            num_resources: 5,
            num_scopes: 10,
            num_logs: 5,
        },
        LogsGenParams {
            num_resources: 50,
            num_scopes: 2,
            num_logs: 10,
        },
        LogsGenParams {
            num_resources: 1,
            num_scopes: 1,
            num_logs: 1000,
        },
        LogsGenParams {
            num_resources: 1,
            num_scopes: 1,
            num_logs: 5000,
        },
    ]
}

/// Print a serialized-size comparison table to stdout (not part of the timed
/// measurements, but the most important output for the "on the wire" debate).
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
            "-- shape {} : {} log records, OTLP proto = {} bytes --",
            shape.label(),
            total_logs,
            proto_len
        );

        let ipc_zstd = Scheme::Ipc
            .codec(Compressor::Zstd)
            .write(otap.clone())
            .expect("ipc zstd write")
            .len();

        for scheme in Scheme::ALL {
            for &compressor in scheme.compressors() {
                let codec = scheme.codec(compressor);
                let bytes = codec
                    .write(otap.clone())
                    .unwrap_or_else(|e| panic!("{} write failed: {e}", codec.name()))
                    .len();
                let vs_otlp = proto_len as f64 / bytes as f64;
                let per_log = bytes as f64 / total_logs as f64;
                let vs_ipc = bytes as f64 / ipc_zstd as f64;
                println!(
                    "{:<16} {:<8} {:>12} {:>9.2}x {:>10.1} {:>11.2}x",
                    codec.name(),
                    compressor.label(),
                    bytes,
                    vs_otlp,
                    per_log,
                    vs_ipc
                );
            }
        }
        println!();
    }
}

/// Median wall-clock milliseconds of `f` over a few iterations (with warm-up).
/// Indicative only; the `server_cost` Criterion group provides rigorous numbers.
fn median_ms(mut f: impl FnMut()) -> f64 {
    for _ in 0..2 {
        f();
    }
    let iters = 11;
    let mut samples = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = Instant::now();
        f();
        samples.push(start.elapsed().as_secs_f64() * 1e3);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    samples[samples.len() / 2]
}

/// Print the server-side CPU model: where the OTAP -> Parquet conversion lands.
fn print_server_cost_table(shapes: &[LogsGenParams]) {
    println!(
        "\n=== Server-side CPU: where does OTAP->Parquet conversion land? (indicative ms) ==="
    );
    println!("Option A (client sends OTAP/IPC): server = IPC-decode + flatten + Parquet-encode");
    println!(
        "Option B (client sends Parquet):  server = persist (~0 CPU) or reparse Parquet->Arrow"
    );
    println!("IPC input fixed at zstd; 'comp' is the Parquet output compressor.");
    println!(
        "{:<16} {:<8} {:>12} {:>12} {:>13} {:>15}",
        "flatten", "comp", "A:convert", "B:reparse", "save(store)", "save(reparse)"
    );

    for shape in shapes {
        let (otap, _) = gen_logs_otap(shape);
        println!(
            "-- shape {} : {} log records --",
            shape.label(),
            shape.total_logs()
        );
        let ipc_bytes = Scheme::Ipc
            .codec(Compressor::Zstd)
            .write(otap.clone())
            .expect("ipc write");

        for scheme in Scheme::PARQUET {
            for compressor in Compressor::ALL {
                let pcodec = scheme.codec(compressor);
                let parquet_bytes = pcodec.write(otap.clone()).expect("parquet write");

                let convert_a = median_ms(|| {
                    let _ = server::convert_ipc_to_parquet(&ipc_bytes, &*pcodec).expect("convert");
                });
                let reparse_b = median_ms(|| {
                    let _ = server::reparse_parquet(&parquet_bytes).expect("reparse");
                });

                println!(
                    "{:<16} {:<8} {:>10.3}ms {:>10.3}ms {:>11.3}ms {:>13.3}ms",
                    scheme.name(),
                    compressor.label(),
                    convert_a,
                    reparse_b,
                    convert_a,
                    convert_a - reparse_b
                );
            }
        }
        println!();
    }
}

fn bench_round_trip(c: &mut Criterion) {
    let shapes = input_shapes();
    print_size_table(&shapes);
    print_server_cost_table(&shapes);

    let mut write_group = c.benchmark_group("parquet_study/write");
    let _ = write_group.sample_size(20);
    let _ = write_group.warm_up_time(Duration::from_millis(500));
    let _ = write_group.measurement_time(Duration::from_secs(2));
    for shape in &shapes {
        let (otap, _) = gen_logs_otap(shape);
        for scheme in Scheme::ALL {
            for &compressor in scheme.compressors() {
                let codec = scheme.codec(compressor);
                let id = BenchmarkId::new(
                    format!("{}/{}", codec.name(), compressor.label()),
                    shape.label(),
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
    let _ = read_group.sample_size(20);
    let _ = read_group.warm_up_time(Duration::from_millis(500));
    let _ = read_group.measurement_time(Duration::from_secs(2));
    for shape in &shapes {
        let (otap, _) = gen_logs_otap(shape);
        for scheme in Scheme::ALL {
            for &compressor in scheme.compressors() {
                let codec = scheme.codec(compressor);
                let bytes = codec.write(otap.clone()).expect("write");
                let id = BenchmarkId::new(
                    format!("{}/{}", codec.name(), compressor.label()),
                    shape.label(),
                );
                let _ = read_group.bench_with_input(id, shape, |b, _| {
                    b.iter(|| black_box(codec.read(&bytes).expect("read")));
                });
            }
        }
    }
    read_group.finish();
}

fn bench_server_cost(c: &mut Criterion) {
    let shapes = input_shapes();

    let mut group = c.benchmark_group("parquet_study/server_cost");
    let _ = group.sample_size(20);
    let _ = group.warm_up_time(Duration::from_millis(500));
    let _ = group.measurement_time(Duration::from_secs(2));

    for shape in &shapes {
        let (otap, _) = gen_logs_otap(shape);
        // Client -> server wire is OTAP/IPC; decode auto-detects compression.
        let ipc_bytes = Scheme::Ipc
            .codec(Compressor::Zstd)
            .write(otap.clone())
            .expect("ipc write");

        for scheme in Scheme::PARQUET {
            for compressor in Compressor::ALL {
                let pcodec = scheme.codec(compressor);

                // Option A: server converts received OTAP/IPC to Parquet.
                let id_a = BenchmarkId::new(
                    format!("convert-A/{}/{}", scheme.name(), compressor.label()),
                    shape.label(),
                );
                let _ = group.bench_with_input(id_a, shape, |b, _| {
                    b.iter(|| {
                        black_box(
                            server::convert_ipc_to_parquet(&ipc_bytes, &*pcodec).expect("convert"),
                        )
                    });
                });

                // Option B: server reparses client-precomputed Parquet.
                let parquet_bytes = pcodec.write(otap.clone()).expect("parquet write");
                let id_b = BenchmarkId::new(
                    format!("accept-B/{}/{}", scheme.name(), compressor.label()),
                    shape.label(),
                );
                let _ = group.bench_with_input(id_b, shape, |b, _| {
                    b.iter(|| black_box(server::reparse_parquet(&parquet_bytes).expect("reparse")));
                });
            }
        }
    }
    group.finish();
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;
    criterion_group!(benches, bench_round_trip, bench_server_cost);
}

criterion_main!(bench_entry::benches);
