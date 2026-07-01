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
use benchmarks::parquet_study::otap_flat::{self, Layout};
use benchmarks::parquet_study::parquet_io::{read_parquet, to_parquet_ready, write_parquet};
use benchmarks::parquet_study::{Compressor, Scheme, ipc, ipc_flat};

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

/// OTAP-flat study: the cost of presenting the four OTAP record batches as a
/// single columnar view, and the size of that view, for three layouts of the
/// shared resource/scope attributes. `nested` is the baseline flatten (hash join
/// plus full `take`); the `otap-flat-*` rows exploit the fact that OTAP
/// attribute batches are already grouped by `parent_id`, so the per-row log
/// attributes are a zero-copy `List<Struct>` and only the layout of the shared
/// resource/scope sets differs. `convert` is OTAP -> single view; `view-mem` is
/// the in-memory footprint of the view; `pq-write`/`pq-bytes` are the Parquet
/// encode of the view when arrow-rs can write it (materialized only -- REE and
/// dictionary-of-`List<Struct>` are in-memory/query forms).
///
/// Two scenarios hold the record count fixed and vary the attribute mix, because
/// REE and dictionary only save on the *shared* resource/scope attributes:
///
/// - `log-heavy`: many per-record log attributes, few resource/scope attributes.
/// - `resource-heavy`: many resources each with many attributes, few log
///   attributes, which is where storing the shared sets once pays off.
fn print_otap_flat_table() {
    use benchmarks::parquet_study::datagen::{RichGenParams, gen_logs_otap_rich};

    let scenarios = [
        RichGenParams {
            label: "log-heavy",
            num_resources: 1,
            num_scopes: 1,
            num_logs: 60_000,
            num_resource_attrs: 1,
            num_scope_attrs: 2,
            num_log_attrs: 9,
        },
        RichGenParams {
            label: "resource-heavy",
            num_resources: 600,
            num_scopes: 1,
            num_logs: 100,
            num_resource_attrs: 20,
            num_scope_attrs: 5,
            num_log_attrs: 2,
        },
    ];

    println!("\n=== OTAP-flat single columnar view (indicative ms, bytes) ===");
    println!("convert = OTAP -> one RecordBatch; view-mem = in-memory footprint of the view.");
    println!("pq-write/pq-bytes use zstd; REE and dict cannot be written to Parquet by arrow-rs.");
    println!(
        "{:<24} {:>10} {:>12} {:>10} {:>12} {:>6}",
        "contender", "convert-ms", "view-mem", "pq-write", "pq-bytes", "pq-ok"
    );
    for scenario in &scenarios {
        let otap = gen_logs_otap_rich(scenario);
        println!(
            "-- {} : {} logs, {} resources x {} resource-attrs, {} log-attrs --",
            scenario.label,
            scenario.total_logs(),
            scenario.num_resources,
            scenario.num_resource_attrs,
            scenario.num_log_attrs,
        );

        // Baseline: the existing nested flatten (hash join + full take).
        let base_convert = median_ms(|| {
            let _ = Scheme::Nested.flatten(&otap).expect("nested flatten");
        });
        let base_flat = Scheme::Nested.flatten(&otap).expect("nested flatten");
        let base_mem = otap_flat::in_memory_bytes(&base_flat);
        let base_pq_input = to_parquet_ready(&base_flat).expect("parquet-ready");
        let base_pq_ms = median_ms(|| {
            let _ = write_parquet(&base_pq_input, Compressor::Zstd.parquet()).expect("write");
        });
        let base_pq = write_parquet(&base_pq_input, Compressor::Zstd.parquet())
            .expect("write")
            .len();
        println!(
            "{:<24} {:>10.2} {:>12} {:>10.2} {:>12} {:>6}",
            "nested (baseline)", base_convert, base_mem, base_pq_ms, base_pq, "yes"
        );

        for layout in [
            Layout::Materialized,
            Layout::RunEndEncoded,
            Layout::Dictionary,
        ] {
            let convert = median_ms(|| {
                let _ = otap_flat::flatten(&otap, layout).expect("flatten");
            });
            let flat = otap_flat::flatten(&otap, layout).expect("flatten");
            let mem = otap_flat::in_memory_bytes(&flat);
            if layout.parquet_writable() {
                let pq_input = to_parquet_ready(&flat).expect("parquet-ready");
                let pq_ms = median_ms(|| {
                    let _ = write_parquet(&pq_input, Compressor::Zstd.parquet()).expect("write");
                });
                let pq_bytes = write_parquet(&pq_input, Compressor::Zstd.parquet())
                    .expect("write")
                    .len();
                println!(
                    "{:<24} {:>10.2} {:>12} {:>10.2} {:>12} {:>6}",
                    layout.name(),
                    convert,
                    mem,
                    pq_ms,
                    pq_bytes,
                    "yes"
                );
            } else {
                println!(
                    "{:<24} {:>10.2} {:>12} {:>10} {:>12} {:>6}",
                    layout.name(),
                    convert,
                    mem,
                    "n/a",
                    "n/a",
                    "no"
                );
            }
        }
        println!();
    }
}

/// Transfer study: how OTAP-flat compares to OTAP-standard and to Parquet when
/// the question is moving data between two large services. Each row reports the
/// serialized wire size under zstd and lz4, the encode cost from an OTAP batch to
/// wire bytes, the decode cost from wire bytes to the receiver's working form,
/// and what that working form is.
///
/// - `ipc-standard` is the OTAP representation today: the transport-optimized
///   `Producer` over the four normalized batches, decoded back to normalized
///   OTAP.
/// - `ipc-flat-*` is one flat record batch serialized as plain Arrow IPC. Arrow
///   IPC can carry `RunEndEncoded` and dictionary columns, so the compact
///   resource/scope layouts survive on the wire. The receiver gets a single
///   query-ready table; projecting it back to normalized OTAP is an extra
///   `otap_flat::unflatten` that this table does not include.
/// - `parquet-flat` is the same flat batch written as a Parquet file.
///
/// encode and decode are timed with zstd.
fn print_transfer_table() {
    use benchmarks::parquet_study::datagen::{RichGenParams, gen_logs_otap_rich};

    let scenarios = [
        RichGenParams {
            label: "log-heavy",
            num_resources: 1,
            num_scopes: 1,
            num_logs: 60_000,
            num_resource_attrs: 1,
            num_scope_attrs: 2,
            num_log_attrs: 9,
        },
        RichGenParams {
            label: "resource-heavy",
            num_resources: 600,
            num_scopes: 1,
            num_logs: 100,
            num_resource_attrs: 20,
            num_scope_attrs: 5,
            num_log_attrs: 2,
        },
    ];

    println!("\n=== Transfer between two services: wire size and CPU (indicative) ===");
    println!("encode = OTAP batch -> wire bytes; decode = wire bytes -> receiver working form.");
    println!("ipc-flat carries REE/dict on the wire (Parquet cannot); decode yields one table.");
    println!(
        "{:<22} {:>12} {:>12} {:>10} {:>10} {:<18}",
        "contender", "zstd-bytes", "lz4-bytes", "encode-ms", "decode-ms", "receiver-form"
    );
    for scenario in &scenarios {
        let otap = gen_logs_otap_rich(scenario);
        println!(
            "-- {} : {} logs, {} resources x {} resource-attrs, {} log-attrs --",
            scenario.label,
            scenario.total_logs(),
            scenario.num_resources,
            scenario.num_resource_attrs,
            scenario.num_log_attrs,
        );

        // OTAP standard: transport-optimized Producer over the normalized batches.
        {
            let zstd_bytes = ipc::encode_to_bytes(otap.clone(), Compressor::Zstd)
                .expect("encode")
                .len();
            let lz4_bytes = ipc::encode_to_bytes(otap.clone(), Compressor::Lz4)
                .expect("encode")
                .len();
            let encode_ms = median_ms(|| {
                let _ = ipc::encode_to_bytes(otap.clone(), Compressor::Zstd).expect("encode");
            });
            let bytes = ipc::encode_to_bytes(otap.clone(), Compressor::Zstd).expect("encode");
            let decode_ms = median_ms(|| {
                let mut o = ipc::deserialize(&bytes).expect("deserialize");
                ipc::transport_decode(&mut o).expect("transport decode");
            });
            println!(
                "{:<22} {:>12} {:>12} {:>10.2} {:>10.2} {:<18}",
                "ipc-standard", zstd_bytes, lz4_bytes, encode_ms, decode_ms, "normalized OTAP"
            );
        }

        // OTAP flat: one Arrow IPC batch, three shared-attribute layouts.
        for layout in [
            Layout::Materialized,
            Layout::RunEndEncoded,
            Layout::Dictionary,
        ] {
            let flat = otap_flat::flatten(&otap, layout).expect("flatten");
            let zstd_bytes = ipc_flat::write_ipc(&flat, Compressor::Zstd)
                .expect("write ipc")
                .len();
            let lz4_bytes = ipc_flat::write_ipc(&flat, Compressor::Lz4)
                .expect("write ipc")
                .len();
            let encode_ms = median_ms(|| {
                let f = otap_flat::flatten(&otap, layout).expect("flatten");
                let _ = ipc_flat::write_ipc(&f, Compressor::Zstd).expect("write ipc");
            });
            let bytes = ipc_flat::write_ipc(&flat, Compressor::Zstd).expect("write ipc");
            let decode_ms = median_ms(|| {
                let _ = ipc_flat::read_ipc(&bytes).expect("read ipc");
            });
            let name = format!("ipc-flat-{}", layout_suffix(layout));
            println!(
                "{:<22} {:>12} {:>12} {:>10.2} {:>10.2} {:<18}",
                name, zstd_bytes, lz4_bytes, encode_ms, decode_ms, "flat table"
            );
        }

        // Parquet: the same flat batch as a Parquet file. Arrow dictionaries are
        // materialized first because arrow-rs cannot read a dictionary-encoded
        // FixedSizeBinary (trace_id/span_id) back from Parquet.
        {
            let flat = Scheme::Nested.flatten(&otap).expect("nested flatten");
            let pq_input = to_parquet_ready(&flat).expect("parquet-ready");
            let zstd_bytes = write_parquet(&pq_input, Compressor::Zstd.parquet())
                .expect("write")
                .len();
            let lz4_bytes = write_parquet(&pq_input, Compressor::Lz4.parquet())
                .expect("write")
                .len();
            let encode_ms = median_ms(|| {
                let f = Scheme::Nested.flatten(&otap).expect("nested flatten");
                let d = to_parquet_ready(&f).expect("parquet-ready");
                let _ = write_parquet(&d, Compressor::Zstd.parquet()).expect("write");
            });
            let bytes = write_parquet(&pq_input, Compressor::Zstd.parquet()).expect("write");
            let decode_ms = median_ms(|| {
                let _ = read_parquet(&bytes).expect("read");
            });
            println!(
                "{:<22} {:>12} {:>12} {:>10.2} {:>10.2} {:<18}",
                "parquet-flat", zstd_bytes, lz4_bytes, encode_ms, decode_ms, "flat table"
            );
        }
        println!();
    }
}

/// Short suffix for a shared-attribute layout, used in transfer row names.
fn layout_suffix(layout: Layout) -> &'static str {
    match layout {
        Layout::Materialized => "materialized",
        Layout::RunEndEncoded => "ree",
        Layout::Dictionary => "dict",
    }
}

/// Conversion-cost matrix: every directed edge of the pipeline, timed in
/// isolation, so the cost of moving between representations is visible edge by
/// edge. The nodes are OTAP-standard (`S`, four normalized batches),
/// OTAP-flat/REE (`F`, one batch with run-end resource/scope columns),
/// OTAP/IPC-standard (`Ws`), OTAP/IPC-flat (`Wf`), and Parquet (`P`). The flat to
/// Parquet edge is split into the parquet-ready transform, which expands the
/// run-end columns and materializes `trace_id`/`span_id`, and the Parquet write.
fn print_conversion_matrix() {
    use benchmarks::parquet_study::datagen::{RichGenParams, gen_logs_otap_rich};
    use otap_df_pdata::otap::OtapArrowRecords;

    let scenarios = [
        RichGenParams {
            label: "log-heavy",
            num_resources: 1,
            num_scopes: 1,
            num_logs: 60_000,
            num_resource_attrs: 1,
            num_scope_attrs: 2,
            num_log_attrs: 9,
        },
        RichGenParams {
            label: "resource-heavy",
            num_resources: 600,
            num_scopes: 1,
            num_logs: 100,
            num_resource_attrs: 20,
            num_scope_attrs: 5,
            num_log_attrs: 2,
        },
    ];

    let edges = [
        "S  -> F   flatten to REE",
        "F  -> S   unflatten",
        "S  -> Ws  standard serialize",
        "Ws -> S   standard deserialize",
        "F  -> Wf  flat serialize",
        "Wf -> F   flat deserialize",
        "F  -> P   parquet-ready (REE+FSB)",
        "F  -> P   parquet write",
        "P  -> F   parquet read",
    ];

    fn measure(otap: &OtapArrowRecords) -> Vec<f64> {
        let flat = otap_flat::flatten(otap, Layout::RunEndEncoded).expect("flatten");
        let ws = ipc::encode_to_bytes(otap.clone(), Compressor::Zstd).expect("ws");
        let wf = ipc_flat::write_ipc(&flat, Compressor::Zstd).expect("wf");
        let ready = to_parquet_ready(&flat).expect("ready");
        let pq = write_parquet(&ready, Compressor::Zstd.parquet()).expect("pq");

        vec![
            median_ms(|| {
                let _ = otap_flat::flatten(otap, Layout::RunEndEncoded).expect("flatten");
            }),
            median_ms(|| {
                let _ = otap_flat::unflatten(&flat).expect("unflatten");
            }),
            median_ms(|| {
                let _ = ipc::encode_to_bytes(otap.clone(), Compressor::Zstd).expect("ws");
            }),
            median_ms(|| {
                let mut o = ipc::deserialize(&ws).expect("des");
                ipc::transport_decode(&mut o).expect("tdec");
            }),
            median_ms(|| {
                let _ = ipc_flat::write_ipc(&flat, Compressor::Zstd).expect("wf");
            }),
            median_ms(|| {
                let _ = ipc_flat::read_ipc(&wf).expect("rf");
            }),
            median_ms(|| {
                let _ = to_parquet_ready(&flat).expect("ready");
            }),
            median_ms(|| {
                let _ = write_parquet(&ready, Compressor::Zstd.parquet()).expect("pq");
            }),
            median_ms(|| {
                let _ = read_parquet(&pq).expect("read");
            }),
        ]
    }

    println!("\n=== Conversion cost matrix (indicative ms) ===");
    println!(
        "S=OTAP-standard, F=OTAP-flat(REE), Ws=OTAP/IPC-standard, Wf=OTAP/IPC-flat, P=Parquet."
    );
    let log = measure(&gen_logs_otap_rich(&scenarios[0]));
    let res = measure(&gen_logs_otap_rich(&scenarios[1]));
    println!(
        "{:<34} {:>12} {:>14}",
        "edge", "log-heavy", "resource-heavy"
    );
    for (i, name) in edges.iter().enumerate() {
        println!("{:<34} {:>12.2} {:>14.2}", name, log[i], res[i]);
    }
    println!();
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
    print_otap_flat_table();
    print_transfer_table();
    print_conversion_matrix();
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
