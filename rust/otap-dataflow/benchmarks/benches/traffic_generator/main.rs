// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Startup and generation-cost benchmarks for the traffic_generator receiver's
//! semantic-conventions data source, per signal type and with the
//! `annotations` option disabled (`ann_off`) and enabled (`ann_on`).
//!
//! Every benchmark loads the vendored registry under `vendor/semconv/model` so
//! results do not depend on the network. Each config weights a single signal
//! type, has no rate limit, and uses `max_batch_size = 1000`, built from the
//! same JSON shape the receiver parses. Groups:
//!
//! - `startup/<signal>/<variant>`: everything the receiver prepares before it
//!   can start generating data -- `TrafficProducer::from_config` with the
//!   `fresh` strategy (resolve the registry, compile the model, build the
//!   generator and batch shape).
//! - `generation/fresh/<signal>/<variant>`: the cost of each message in the
//!   `fresh` strategy -- generate and encode one batch of 1000 signals through
//!   the producer's run iterator, the receiver's runtime hot path.
//! - `generation/pregen/<signal>/<variant>`: the pre-generation cost of the
//!   `pre_generated` strategy -- generate and encode the full replay set
//!   (1024 batches of 1000 signals) from a newly built model.
//!
//! Set `TRAFFICGEN_BENCH_UPSTREAM=1` to also benchmark the upstream registry
//! (cloned from GitHub on every resolve; requires network access).

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::config::Config;
use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::producer::TrafficProducer;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata_codec::OtapPayload;
use serde_json::json;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Signals per batch (`max_batch_size`).
const BATCH: usize = 1000;
/// Batches built by the `pre_generated` strategy with no rate limit.
const PREGEN_BATCHES: usize = 1024;

mod traffic_generator {
    use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::config::Config;
    use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::semconv_model::SemconvModel;
    use otel_arrow_dfe_dev_nodes::receivers::traffic_generator::semconv_signal;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use weaver_forge::v1::registry::ResolvedRegistry;

    use super::Signal;

    /// Variants under comparison: `(label, annotations)`.
    pub const VARIANTS: &[(&str, Option<bool>)] =
        &[("ann_off", Some(false)), ("ann_on", Some(true))];

    /// Model the generator draws from.
    pub type Model = SemconvModel;

    pub fn resolve(config: &Config) -> ResolvedRegistry {
        config.resolve_registry().expect("registry resolves")
    }

    pub fn compile(resolved: ResolvedRegistry, annotations: Option<bool>) -> Model {
        if annotations == Some(true) {
            SemconvModel::compile(&resolved).unwrap_or_else(|errors| {
                panic!(
                    "{}",
                    otel_arrow_dfe_dev_nodes::receivers::traffic_generator::annotation::format_errors(&errors)
                )
            })
        } else {
            SemconvModel::compile_without_annotations(&resolved)
        }
    }

    pub fn generate(signal: Signal, count: usize, model: &Model) -> OtlpProtoMessage {
        match signal {
            Signal::Logs => OtlpProtoMessage::Logs(semconv_signal::semconv_otlp_logs(count, model)),
            Signal::Metrics => {
                OtlpProtoMessage::Metrics(semconv_signal::semconv_otlp_metrics(count, model))
            }
            Signal::Traces => {
                OtlpProtoMessage::Traces(semconv_signal::semconv_otlp_traces(count, model))
            }
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

    /// `(metric_weight, trace_weight, log_weight)` selecting only this signal.
    fn weights(self) -> (u32, u32, u32) {
        match self {
            Signal::Logs => (0, 0, 1),
            Signal::Metrics => (1, 0, 0),
            Signal::Traces => (0, 1, 0),
        }
    }
}

/// Registry sources under comparison.
#[derive(Clone, Copy)]
enum Registry {
    Vendored,
    Upstream,
}

impl Registry {
    /// `vendored` always; `upstream` only with `TRAFFICGEN_BENCH_UPSTREAM=1`.
    fn selected() -> Vec<Registry> {
        if std::env::var("TRAFFICGEN_BENCH_UPSTREAM").as_deref() == Ok("1") {
            otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
            vec![Registry::Vendored, Registry::Upstream]
        } else {
            vec![Registry::Vendored]
        }
    }

    /// `registry_path` value, or `None` to use the receiver's default (the
    /// upstream git repository).
    fn path(self) -> Option<&'static str> {
        match self {
            Registry::Vendored => Some(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../vendor/semconv/model"
            )),
            Registry::Upstream => None,
        }
    }

    /// Benchmark parameter label for a variant on this registry.
    fn label(self, variant: &str) -> String {
        match self {
            Registry::Vendored => variant.to_owned(),
            Registry::Upstream => format!("{variant}_upstream"),
        }
    }
}

/// Build the receiver config exactly as the receiver would parse it.
fn config(
    registry: Registry,
    signal: Signal,
    annotations: Option<bool>,
    pre_generated: bool,
) -> Config {
    let (metric_weight, trace_weight, log_weight) = signal.weights();
    let mut value = json!({
        "traffic_config": {
            "signals_per_second": null,
            "max_batch_size": BATCH,
            "metric_weight": metric_weight,
            "trace_weight": trace_weight,
            "log_weight": log_weight,
        },
        "data_source": "semantic_conventions",
        "generation_strategy": if pre_generated { "pre_generated" } else { "fresh" },
    });
    if let Some(path) = registry.path() {
        value["registry_path"] = json!(path);
    }
    if let Some(annotations) = annotations {
        value["annotations"] = json!(annotations);
    }
    serde_json::from_value(value).expect("benchmark config parses")
}

/// Generate and encode the replay set (`PREGEN_BATCHES` batches of `BATCH`
/// signals) from `model`, as the `pre_generated` strategy does at startup.
fn pregenerate(signal: Signal, model: &traffic_generator::Model) -> Vec<OtapPayload> {
    (0..PREGEN_BATCHES)
        .map(|_| {
            let message: OtlpProtoMessage = traffic_generator::generate(signal, BATCH, model);
            message.try_into().expect("payload encodes")
        })
        .collect()
}

/// Scenario: The receiver prepares to generate one signal type, for each
/// annotations variant.
/// Guarantees: Measures everything done before data generation can begin
/// (`TrafficProducer::from_config` with `fresh`: resolve, compile, shape), so
/// startup-latency regressions are visible per signal and per variant.
fn bench_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    let _ = group.sample_size(20);
    for registry in Registry::selected() {
        for &(variant, annotations) in traffic_generator::VARIANTS {
            for signal in Signal::ALL {
                let cfg = config(registry, signal, annotations, false);
                let id = BenchmarkId::new(signal.id(), registry.label(variant));
                let _ = group.bench_function(id, |b| {
                    b.iter_with_large_drop(|| {
                        black_box(TrafficProducer::from_config(&cfg).expect("producer starts"))
                    });
                });
            }
        }
    }
    group.finish();
}

/// Scenario: A started `fresh` producer emits messages through its run
/// iterator, for each signal type and annotations variant.
/// Guarantees: Measures the per-message runtime cost of the `fresh` strategy
/// (generate and encode one batch of `BATCH` signals) on the same path the
/// receiver uses, excluding startup.
fn bench_generation_fresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation/fresh");
    let _ = group.throughput(Throughput::Elements(BATCH as u64));
    for registry in Registry::selected() {
        for &(variant, annotations) in traffic_generator::VARIANTS {
            for signal in Signal::ALL {
                let cfg = config(registry, signal, annotations, false);
                let mut producer = TrafficProducer::from_config(&cfg).expect("producer starts");
                let id = BenchmarkId::new(signal.id(), registry.label(variant));
                let _ = group.bench_function(id, |b| {
                    b.iter(|| {
                        let mut run = producer
                            .next_run()
                            .expect("run starts")
                            .expect("no signal limit");
                        black_box(
                            run.next()
                                .expect("run has a batch")
                                .expect("batch generates"),
                        )
                    });
                });
            }
        }
    }
    group.finish();
}

/// Scenario: The `pre_generated` strategy builds its full replay set for one
/// signal type from a newly built model, for each annotations variant.
/// Guarantees: Measures the pre-generation cost (generate and encode all
/// `PREGEN_BATCHES` batches) separately from startup; model construction runs
/// untimed before each iteration so every sample starts from a new model as
/// the receiver does.
fn bench_generation_pregen(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation/pregen");
    let _ = group.sample_size(10);
    let _ = group.throughput(Throughput::Elements((BATCH * PREGEN_BATCHES) as u64));
    for registry in Registry::selected() {
        for &(variant, annotations) in traffic_generator::VARIANTS {
            for signal in Signal::ALL {
                let cfg = config(registry, signal, annotations, true);
                let id = BenchmarkId::new(signal.id(), registry.label(variant));
                let _ = group.bench_function(id, |b| {
                    b.iter_batched(
                        || traffic_generator::compile(traffic_generator::resolve(&cfg), annotations),
                        |model| {
                            // Return the model with the payloads so both are
                            // dropped outside the timed region.
                            let payloads = pregenerate(signal, &model);
                            black_box((payloads, model))
                        },
                        BatchSize::PerIteration,
                    );
                });
            }
        }
    }
    group.finish();
}

criterion_group!(
    traffic_generator,
    bench_startup,
    bench_generation_fresh,
    bench_generation_pregen
);
criterion_main!(traffic_generator);
