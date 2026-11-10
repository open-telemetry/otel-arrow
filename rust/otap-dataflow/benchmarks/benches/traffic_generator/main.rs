// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for the traffic_generator receiver's semantic-conventions data
//! source, comparing the upstream registry (cloned from GitHub at startup) with
//! the vendored, annotated copy under `vendor/semconv/model`.
//!
//! The goal is to measure the cost of the annotation integration. Three kinds
//! of cost are isolated:
//!
//! - `startup/resolve_compile`: resolve the registry and compile it into the
//!   internal model (annotations parsed, samplers built). For `upstream` this
//!   includes the git clone, so it is reported separately from `vendored`.
//! - `pregenerate`: build the full pre-generated payload set once, exactly as
//!   the `pre_generated` strategy does at startup. This uses the producer's own
//!   shaping (a 1:1:1 logs/metrics/traces mix, `1024 * max_batch_size` signals
//!   total, split into separately encoded `max_batch_size` batches) so the
//!   benchmark measures the real retained-payload workload rather than a single
//!   oversized batch.
//! - `fresh`: generate and encode one batch of signals, per signal type (the
//!   per-batch cost paid continuously by the `fresh` strategy), through the same
//!   `OtapPayload` encoder the receiver uses.
//!
//! Memory is reported out-of-band through a counting allocator that wraps
//! jemalloc: per-run lines print bytes retained by the model, bytes allocated
//! during resolve/compile, bytes per fresh batch, bytes retained by the
//! pre-generated payload set, and its per-signal breakdown.
//!
//! Both `upstream` and `vendored` run by default. `upstream` requires network
//! access to clone the registry; a clone failure is a hard error.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::config::{
    Config, TrafficConfig, default_registry_path, vendored_registry_path,
};
use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::producer::{
    SignalGenerator, WeaverGenerator, pregenerate_payloads,
};
use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::semconv_model::SemconvModel;
use otel_arrow_dfe_pdata_codec::OtapPayload;
use weaver_common::vdir::VirtualDirectoryPath;

// Use jemalloc so its statistics can be read through tikv_jemalloc_ctl. The
// counters below are process-wide, so memory lines are only meaningful because
// the benchmark harness runs these regions single-threaded between iterations.
#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Whether a memory report should be printed. jemalloc stats are unavailable on
/// Windows in this configuration.
const MEMORY_REPORT: bool = !cfg!(windows);

/// Snapshot of jemalloc's process-wide allocation statistics (bytes currently
/// allocated to the application, and resident bytes mapped by the allocator).
/// On Windows the fields stay zero and memory lines are omitted.
#[derive(Clone, Copy)]
struct AllocSnapshot {
    allocated: u64,
    resident: u64,
}

impl AllocSnapshot {
    /// Advance jemalloc's stats epoch and read the current counters.
    fn now() -> Self {
        #[cfg(not(windows))]
        {
            use tikv_jemalloc_ctl::{epoch, stats};
            // Refresh cached statistics; ignore epoch errors and fall back to
            // the last readable values.
            let _ = epoch::advance();
            Self {
                allocated: stats::allocated::read().unwrap_or(0) as u64,
                resident: stats::resident::read().unwrap_or(0) as u64,
            }
        }
        #[cfg(windows)]
        {
            Self {
                allocated: 0,
                resident: 0,
            }
        }
    }

    /// Net bytes allocated to the application since `start`.
    fn net_bytes(self, start: Self) -> i64 {
        self.allocated as i64 - start.allocated as i64
    }

    /// Change in resident bytes since `start`.
    fn resident_delta(self, start: Self) -> i64 {
        self.resident as i64 - start.resident as i64
    }
}

/// Registry variants under comparison.
#[derive(Clone, Copy)]
enum Registry {
    Upstream,
    Vendored,
}

impl Registry {
    const ALL: [Registry; 2] = [Registry::Upstream, Registry::Vendored];

    fn id(self) -> &'static str {
        match self {
            Registry::Upstream => "upstream",
            Registry::Vendored => "vendored",
        }
    }

    fn path(self) -> VirtualDirectoryPath {
        match self {
            Registry::Upstream => default_registry_path(),
            Registry::Vendored => vendored_registry_path(),
        }
    }
}

/// Signal types under comparison.
#[derive(Clone, Copy)]
enum Signal {
    Logs,
    Metrics,
    Traces,
}

impl Signal {
    const ALL: [Signal; 3] = [Signal::Logs, Signal::Metrics, Signal::Traces];

    fn id(self) -> &'static str {
        match self {
            Signal::Logs => "logs",
            Signal::Metrics => "metrics",
            Signal::Traces => "traces",
        }
    }

    /// Generate `count` signals as one encoded `OtapPayload`, through the same
    /// `OtlpProtoMessage` -> `OtapPayload` path the receiver uses.
    fn generate_encoded(self, count: usize, generator: &mut dyn SignalGenerator) -> OtapPayload {
        let result = match self {
            Signal::Logs => generator.generate_logs(count),
            Signal::Metrics => generator.generate_metrics(count),
            Signal::Traces => generator.generate_traces(count),
        };
        result.expect("signal generation and encoding succeed")
    }
}

/// Signals per `fresh` batch.
const FRESH_BATCH: usize = 1000;

/// Build a `TrafficConfig` with all three signal weights enabled; the generator
/// code paths below select a single signal type explicitly.
fn traffic_config() -> TrafficConfig {
    // signals_per_second = None (open), max_signal_count = None, batch 1000,
    // metric/trace/log weights all 1.
    TrafficConfig::new(None, None, FRESH_BATCH, 1, 1, 1)
}

/// Resolve and compile the registry into a model, panicking on failure.
fn compile_model(registry: Registry) -> SemconvModel {
    let config = Config::new(traffic_config(), registry.path());
    config
        .get_semconv_model()
        .unwrap_or_else(|e| panic!("resolve/compile {} registry: {e}", registry.id()))
        .expect("semantic_conventions data source returns Some model")
}

/// weaver clones the upstream registry over rustls without choosing a crypto
/// provider, so one must be installed before resolving.
fn ensure_crypto() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
}

#[allow(clippy::print_stdout)]
fn bench_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup/resolve_compile");
    // Resolving (and, for upstream, cloning) is slow; keep the sample small.
    let _ = group.sample_size(10);

    for registry in Registry::ALL {
        if MEMORY_REPORT {
            let before = AllocSnapshot::now();
            let model = compile_model(registry);
            let after = AllocSnapshot::now();
            // net_bytes is bytes still allocated to the application after
            // resolve/compile completes (resolver scratch has been freed, the
            // model is retained); resident_delta is the change in bytes mapped
            // by the allocator, which includes fragmentation and retained
            // arenas.
            println!(
                "mem startup/{}: resolve_compile net={} B resident_delta={} B",
                registry.id(),
                after.net_bytes(before),
                after.resident_delta(before),
            );
            // Measure bytes retained by the model alone: snapshot around a
            // clone so transient resolver state is excluded.
            let before_model = AllocSnapshot::now();
            let cloned = black_box(model.clone());
            let after_model = AllocSnapshot::now();
            println!(
                "mem startup/{}: model_retained net={} B",
                registry.id(),
                after_model.net_bytes(before_model),
            );
            drop(cloned);
            drop(model);
        }

        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(registry.id()),
            &registry,
            |b, &registry| {
                b.iter(|| black_box(compile_model(registry)));
            },
        );
    }
    group.finish();
}

#[allow(clippy::print_stdout)]
fn bench_fresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("fresh");
    let _ = group.throughput(Throughput::Elements(FRESH_BATCH as u64));

    for registry in Registry::ALL {
        let mut generator = WeaverGenerator::new(compile_model(registry));
        for signal in Signal::ALL {
            if MEMORY_REPORT {
                let before = AllocSnapshot::now();
                let mut encoded = black_box(signal.generate_encoded(FRESH_BATCH, &mut generator));
                let after = AllocSnapshot::now();
                let bytes = encoded.num_bytes().unwrap_or(0);
                println!(
                    "mem fresh/{}/{}: batch_net={} B encoded={} B encoded_per_signal={:.1} B",
                    signal.id(),
                    registry.id(),
                    after.net_bytes(before),
                    bytes,
                    bytes as f64 / FRESH_BATCH as f64,
                );
            }

            let id = BenchmarkId::new(signal.id(), registry.id());
            let _ = group.bench_with_input(id, &registry, |b, _registry| {
                b.iter(|| black_box(signal.generate_encoded(FRESH_BATCH, &mut generator)));
            });
        }
    }
    group.finish();
}

/// Build the full pre-generated payload set for `registry`, exactly as the
/// `pre_generated` strategy does at startup: a 1:1:1 logs/metrics/traces mix of
/// `FRESH_BATCH * 1024` signals, split into separately encoded `FRESH_BATCH`
/// batches and retained as `OtapPayload`s.
fn pregenerate_set(registry: Registry) -> Vec<OtapPayload> {
    let mut generator = WeaverGenerator::new(compile_model(registry));
    pregenerate_payloads(&mut generator, &traffic_config())
        .unwrap_or_else(|e| panic!("pregenerate {} payloads: {e}", registry.id()))
}

/// Sum encoded bytes and batch counts per signal type for a payload set.
fn report_pregenerate(registry: Registry, net: i64, payloads: &mut [OtapPayload]) {
    let mut batches = [0usize; 3];
    let mut bytes = [0usize; 3];
    let mut signals = [0usize; 3];
    for payload in payloads.iter_mut() {
        let idx = match payload.signal_type() {
            SignalType::Logs => 0,
            SignalType::Metrics => 1,
            SignalType::Traces => 2,
        };
        batches[idx] += 1;
        bytes[idx] += payload.num_bytes().unwrap_or(0);
        signals[idx] += payload.num_items();
    }
    let total_bytes: usize = bytes.iter().sum();
    let total_batches: usize = batches.iter().sum();
    #[allow(clippy::print_stdout)]
    {
        println!(
            "mem pregenerate/{}: encoded_retained={total_bytes} B heap_net={net} B batches={total_batches}",
            registry.id(),
        );
        for (idx, name) in ["logs", "metrics", "traces"].iter().enumerate() {
            let per_signal = if signals[idx] == 0 {
                0.0
            } else {
                bytes[idx] as f64 / signals[idx] as f64
            };
            println!(
                "mem pregenerate/{}/{}: encoded={} B batches={} signals={} per_signal={:.1} B",
                name,
                registry.id(),
                bytes[idx],
                batches[idx],
                signals[idx],
                per_signal,
            );
        }
    }
}

fn bench_pregenerate(c: &mut Criterion) {
    let mut group = c.benchmark_group("pregenerate");
    // The pre_generated strategy builds max_batch_size * 1024 signals up front
    // when signals_per_second is None. Mirror that volume and 1:1:1 mix here.
    let pregen_count = FRESH_BATCH * 1024;
    let _ = group.throughput(Throughput::Elements(pregen_count as u64));
    // One run already generates a million+ signals; keep the sample small.
    let _ = group.sample_size(10);

    for registry in Registry::ALL {
        if MEMORY_REPORT {
            let before = AllocSnapshot::now();
            let mut payloads = black_box(pregenerate_set(registry));
            let after = AllocSnapshot::now();
            report_pregenerate(registry, after.net_bytes(before), &mut payloads);
            drop(payloads);
        }

        let id = BenchmarkId::from_parameter(registry.id());
        let _ = group.bench_with_input(id, &registry, |b, &registry| {
            // Freeing the retained payload set is excluded from the timed
            // region so the measurement reflects generation and encoding only.
            b.iter_with_large_drop(|| black_box(pregenerate_set(registry)));
        });
    }
    group.finish();
}

fn benches(c: &mut Criterion) {
    ensure_crypto();
    bench_startup(c);
    bench_fresh(c);
    bench_pregenerate(c);
}

criterion_group!(traffic_generator, benches);
criterion_main!(traffic_generator);
