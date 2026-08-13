// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Throughput benchmarks for the Kafka exporter.
//!
//! These benchmarks drive a fully-wired [`KafkaExporter`] against the in-process
//! `rdkafka::mocking::MockCluster` (no Docker/external broker) and measure how
//! long it takes to accept, encode, enqueue, deliver, and drain a batch of
//! pdata. They cover the four Section 9 dimensions:
//!
//! 1. Throughput per encoding (OTLP/OTAP) and per signal (logs/traces/metrics).
//! 2. The `max_in_flight` concurrency sweep at 1 and N partitions, quantifying
//!    the delivery-future pipelining win over the serial (`max_in_flight = 1`)
//!    baseline.
//! 3. Poll overhead: steady-state throughput while the producer poll thread runs
//!    at its 1s interval, saturated vs lightly loaded.
//! 4. Slow / unavailable broker: the deadline-bounded nack path never stalls.
//!
//! # Threading
//!
//! The mock broker is `!Send` and must live on its creation thread, so each
//! measured iteration runs on a fresh current-thread Tokio runtime + `LocalSet`
//! (via `bench_support::run_on_local_set`). We use `iter_custom` to time only
//! the drive+drain of the exporter, excluding cluster/exporter construction.

#![allow(missing_docs)]
#![allow(unused_results)]
#![allow(clippy::print_stderr)]

use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use otap_df_contrib_nodes::common::kafka::bench_support::{
    BenchSignalType, Encoding, encoding_otap, encoding_otlp, exporter_config, run_on_local_set,
    sample_pdata, unavailable_broker_config,
};

/// Number of batches pushed per measured iteration.
///
/// Kept modest because each iteration stands up a fresh mock broker + exporter
/// (whose producer polls on a 1s interval) and drains to full delivery, so the
/// wall-clock per iteration is dominated by broker round-trips rather than
/// per-batch CPU. The benches compare relative throughput across encodings,
/// signals, and `max_in_flight`; they are not a raw messages/sec figure.
const BATCHES_PER_ITER: u64 = 20;

/// Graceful-shutdown deadline.
///
/// NOTE: the in-process mock broker's `flush` waits out this full deadline
/// rather than returning the instant the queue drains, so it is kept short to
/// bound per-iteration wall-clock. Records are delivered on the producer poll
/// cycle well within this window; the deadline only caps the flush wait. The
/// benches therefore compare the *relative* cost of the send/encode/enqueue
/// path across configurations rather than reporting an absolute drain latency.
const SHUTDOWN_DEADLINE: Duration = Duration::from_millis(500);

/// Criterion sample size for every group. These benches are broker-bound and
/// comparatively slow per iteration, so a small sample keeps a full run
/// bounded while still producing a stable central estimate.
const SAMPLE_SIZE: usize = 10;

/// Builds a current-thread runtime for the `!Send` mock broker.
fn current_thread_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("bench: build current-thread runtime")
}

/// Drives `BATCHES_PER_ITER` batches through a freshly wired exporter and drains
/// them at shutdown, returning the elapsed time for the drive+drain only.
fn drive_once(
    signal: BenchSignalType,
    encoding: Encoding,
    partitions: i32,
    max_in_flight: usize,
) -> Duration {
    let rt = current_thread_runtime();
    rt.block_on(async move {
        run_on_local_set(&[("bench-topic", partitions)], |cluster| async move {
            // Subscribe before driving so the mock broker establishes topic
            // leadership; without this, delivery (and thus the shutdown flush)
            // would wait out the flush deadline.
            let _consumer = cluster.subscribe(&["bench-topic"]);
            let cfg = exporter_config(
                &cluster.bootstrap_servers(),
                signal,
                "bench-topic",
                encoding,
                max_in_flight,
            );
            let exporter = cluster.start_exporter(cfg);

            // Pre-build one sample pdata and clone it per send so payload
            // construction is not part of the timed region.
            let pdata = sample_pdata(signal);

            // Time only the accept+encode+enqueue+pipeline path. With
            // `max_in_flight` backpressure, this loop still awaits deliveries
            // whenever the in-flight set fills, so it reflects pipelining
            // behavior; the shutdown drain (whose flush waits out the mock's
            // deadline) is deliberately excluded from the measured region.
            let start = Instant::now();
            for _ in 0..BATCHES_PER_ITER {
                exporter.send(pdata.clone()).await;
            }
            let elapsed = start.elapsed();
            exporter.shutdown_and_wait(SHUTDOWN_DEADLINE).await;
            elapsed
        })
        .await
    })
}

/// Accumulates `drive_once` across the iteration count Criterion requests.
fn drive_iters(
    iters: u64,
    signal: BenchSignalType,
    encoding: Encoding,
    partitions: i32,
    max_in_flight: usize,
) -> Duration {
    let mut total = Duration::ZERO;
    for _ in 0..iters {
        total += drive_once(signal, encoding, partitions, max_in_flight);
    }
    total
}

/// Number of batches for the `max_in_flight` sweep. Larger than
/// `BATCHES_PER_ITER` so that -- with an injected per-request broker latency --
/// the in-flight set genuinely saturates and the overlap of delivery waits
/// dominates the fixed per-iteration construction cost.
const MIF_BATCHES: u64 = 64;

/// Per-request broker round-trip latency injected for the `max_in_flight`
/// sweep. Serial (`max_in_flight = 1`) pays this once per batch; a pipelined
/// run overlaps up to `max_in_flight` of these waits, which is exactly the
/// throughput win the sweep is meant to expose. Kept small so a 64-batch serial
/// run stays bounded (~64 * this).
const MIF_ROUND_TRIP: Duration = Duration::from_millis(2);

/// Drives `MIF_BATCHES` batches with an injected broker round-trip latency and
/// times the send loop, which is where the `max_in_flight` bound takes effect:
/// with `max_in_flight = 1` each send blocks on the previous delivery (so the
/// injected round trip is serialized), while a higher bound lets that many sends
/// proceed before blocking (so the delivery waits overlap). This exposes the
/// pipelining win on the otherwise near-zero-latency mock broker.
///
/// NOTE: the custom producer polls delivery callbacks on a 1s interval, so a
/// serial run's per-batch stall is dominated by that poll quantization rather
/// than the 2ms injected latency; the sweep therefore measures the *relative*
/// send-loop throughput across `max_in_flight`, not an absolute broker latency.
fn drive_with_latency_once(partitions: i32, max_in_flight: usize) -> Duration {
    let rt = current_thread_runtime();
    rt.block_on(async move {
        run_on_local_set(&[("bench-topic", partitions)], |cluster| async move {
            // Subscribe to establish topic leadership on the mock broker.
            let _consumer = cluster.subscribe(&["bench-topic"]);
            // Inject broker latency so serial sends serialize the wait and
            // pipelined sends overlap it.
            cluster.inject_round_trip_latency(MIF_ROUND_TRIP);
            let cfg = exporter_config(
                &cluster.bootstrap_servers(),
                BenchSignalType::Logs,
                "bench-topic",
                encoding_otlp(),
                max_in_flight,
            );
            let exporter = cluster.start_exporter(cfg);
            let pdata = sample_pdata(BenchSignalType::Logs);

            // Timed region: the send loop only. With `max_in_flight = 1` every
            // send after the first blocks until the previous delivery resolves
            // (in-flight set full), serializing the injected round trip; a higher
            // bound lets up to `max_in_flight` sends proceed before blocking, so
            // the loop overlaps the delivery waits.
            let start = Instant::now();
            for _ in 0..MIF_BATCHES {
                exporter.send(pdata.clone()).await;
            }
            let elapsed = start.elapsed();

            exporter.shutdown_and_wait(SHUTDOWN_DEADLINE).await;
            elapsed
        })
        .await
    })
}

/// Accumulates `drive_with_latency_once` across Criterion's iteration count.
fn drive_with_latency_iters(iters: u64, partitions: i32, max_in_flight: usize) -> Duration {
    let mut total = Duration::ZERO;
    for _ in 0..iters {
        total += drive_with_latency_once(partitions, max_in_flight);
    }
    total
}

/// Dimension 1: throughput per encoding x signal (single partition, serial).
fn bench_encoding_and_signal(c: &mut Criterion) {
    let mut group = c.benchmark_group("kafka_exporter/encoding_signal");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(BATCHES_PER_ITER));

    let signals = [
        ("logs", BenchSignalType::Logs),
        ("traces", BenchSignalType::Traces),
        ("metrics", BenchSignalType::Metrics),
    ];
    let encodings = [("otlp", encoding_otlp()), ("otap", encoding_otap())];

    for (sig_name, signal) in signals {
        for (enc_name, encoding) in encodings {
            group.bench_function(
                BenchmarkId::from_parameter(format!("{enc_name}/{sig_name}")),
                |b| {
                    b.iter_custom(|iters| drive_iters(iters, signal, encoding, 1, 1));
                },
            );
        }
    }
    group.finish();
}

/// Dimension 2: the `max_in_flight` sweep at 1 and 4 partitions (logs/OTLP),
/// quantifying the pipelining win over the serial baseline.
///
/// A per-request broker round-trip latency ([`MIF_ROUND_TRIP`]) is injected and
/// the measured region includes delivery confirmation, so that serial
/// (`max_in_flight = 1`) runs serialize the delivery waits while pipelined runs
/// overlap them. Throughput therefore rises with `max_in_flight` until it
/// saturates at the broker/partition concurrency limit. Absolute numbers are
/// mock-relative; the meaningful signal is the *relative* improvement across the
/// sweep.
fn bench_max_in_flight_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("kafka_exporter/max_in_flight");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(MIF_BATCHES));

    for partitions in [1, 4] {
        for max_in_flight in [1usize, 5, 32] {
            group.bench_function(
                BenchmarkId::from_parameter(format!("p{partitions}/mif{max_in_flight}")),
                |b| {
                    b.iter_custom(|iters| {
                        drive_with_latency_iters(iters, partitions, max_in_flight)
                    });
                },
            );
        }
    }
    group.finish();
}

/// Dimension 3: poll overhead. Compares a lightly loaded run (few batches, so
/// the producer poll thread's 1s interval dominates the idle profile) against
/// the saturated baseline, both at the serial default.
fn bench_poll_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("kafka_exporter/poll_overhead");
    group.sample_size(SAMPLE_SIZE);

    // Saturated: the full batch count (throughput-normalized).
    group.throughput(Throughput::Elements(BATCHES_PER_ITER));
    group.bench_function("saturated", |b| {
        b.iter_custom(|iters| drive_iters(iters, BenchSignalType::Logs, encoding_otlp(), 1, 8));
    });

    // Lightly loaded: a single batch per iteration exercises the
    // construct/enqueue/poll/drain fixed cost that the poll thread contributes.
    group.throughput(Throughput::Elements(1));
    group.bench_function("single_batch", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                let rt = current_thread_runtime();
                total += rt.block_on(async {
                    run_on_local_set(&[("bench-topic", 1)], |cluster| async move {
                        let _consumer = cluster.subscribe(&["bench-topic"]);
                        let cfg = exporter_config(
                            &cluster.bootstrap_servers(),
                            BenchSignalType::Logs,
                            "bench-topic",
                            encoding_otlp(),
                            1,
                        );
                        let exporter = cluster.start_exporter(cfg);
                        let pdata = sample_pdata(BenchSignalType::Logs);
                        let start = Instant::now();
                        exporter.send(pdata).await;
                        let elapsed = start.elapsed();
                        exporter.shutdown_and_wait(SHUTDOWN_DEADLINE).await;
                        elapsed
                    })
                    .await
                });
            }
            total
        });
    });
    group.finish();
}

/// Dimension 4: slow / unavailable broker. Points the exporter at an unroutable
/// broker with a short timeout; every send fails within the bound and becomes a
/// transient nack, so the drive+drain stays deadline-bounded rather than
/// hanging. Measures the nack-path throughput.
fn bench_unavailable_broker(c: &mut Criterion) {
    let mut group = c.benchmark_group("kafka_exporter/unavailable_broker");
    group.throughput(Throughput::Elements(BATCHES_PER_ITER));
    group.sample_size(SAMPLE_SIZE);

    group.bench_function("transient_nack_path", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                let rt = current_thread_runtime();
                total += rt.block_on(async {
                    // Point at an unroutable address with a short message
                    // timeout so each delivery fails quickly. The mock broker is
                    // still created (harness threading requires a LocalSet) but
                    // is unused since the config targets 127.0.0.1:1.
                    run_on_local_set(&[("bench-topic", 1)], |cluster| async move {
                        let cfg = unavailable_broker_config("127.0.0.1:1", "bench-topic", 8, 100);
                        let exporter = cluster.start_exporter(cfg);
                        let pdata = sample_pdata(BenchSignalType::Logs);
                        let start = Instant::now();
                        for _ in 0..BATCHES_PER_ITER {
                            exporter.send(pdata.clone()).await;
                        }
                        let elapsed = start.elapsed();
                        // Short shutdown deadline: the unroutable broker cannot
                        // deliver, so the flush is bounded then purged. Excluded
                        // from the measured region.
                        exporter.shutdown_and_wait(Duration::from_millis(500)).await;
                        elapsed
                    })
                    .await
                });
            }
            total
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_encoding_and_signal,
    bench_max_in_flight_sweep,
    bench_poll_overhead,
    bench_unavailable_broker,
);
criterion_main!(benches);
