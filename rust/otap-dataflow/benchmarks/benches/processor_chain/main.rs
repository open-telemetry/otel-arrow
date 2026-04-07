// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks comparing processor chain vs. separate processors.
//!
//! Compares the throughput of N processors wired as:
//! 1. A single `ProcessorChainNode` (all processors in one task, no inter-channels)
//! 2. Separate processors connected via channels (simulated by calling each
//!    processor sequentially with channel send/recv between them)
//!
//! This isolates the overhead of the chain's staging buffer approach vs.
//! the baseline of individual processor calls with channel round-trips.
//!
//! ## Current Results (1,000 batches, ~300µs simulated work per processor, single-threaded LocalSet)
//!
//! | Chain len | Chained (ms) | Separate (ms) | Overhead |
//! |-----------|-------------|---------------|----------|
//! | 1         | 301.3       | 300.6         | +0.2%    |
//! | 2         | 602.2       | 605.2         | -0.5%    |
//! | 3         | 904.2       | 906.1         | -0.2%    |
//!
//! With realistic per-processor compute (~300µs, approximating a real
//! Arrow attribute transform), the chain overhead is within noise —
//! effectively zero. The staging buffer and mpsc channel costs (~500ns
//! per stage) are negligible relative to the processor work.
//!
//! The chain's value is not raw throughput — it's the ability to produce a
//! correct composite duration metric (min/max/sum/count) across all
//! sub-processors, which is impossible with separate processors.

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_channel::mpsc;
use otap_df_config::PortName;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::error::Error;
use otap_df_engine::local::processor::{EffectHandler, Processor};
use otap_df_engine::local::processor_chain::ProcessorChainNode;
use otap_df_engine::message::{Message, Sender};
use otap_df_engine::node::NodeId;
use otap_df_engine::process_duration::ComputeDuration;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::HashMap;
use std::hint::black_box;
use tokio::task::LocalSet;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const BATCH_COUNT: usize = 1_000;

/// Simulated compute work per processor in nanoseconds.
/// Set to ~300µs to approximate a real attribute transform on an Arrow
/// record batch. This ensures the benchmark reflects production-like
/// overhead ratios rather than being dominated by chain bookkeeping.
const SIMULATED_WORK_NS: u64 = 300_000;

// ── Helpers ──────────────────────────────────────────────────────────

fn test_pipeline_ctx() -> otap_df_engine::context::PipelineContext {
    let registry = TelemetryRegistryHandle::new();
    let controller = ControllerContext::new(registry);
    controller
        .pipeline_context_with("bench".into(), "bench".into(), 0, 1, 0)
        .with_node_context(
            "bench_node".into(),
            "urn:otel:processor:attribute".into(),
            otap_df_config::node::NodeKind::Processor,
            HashMap::new(),
        )
}

fn node_id(name: &str) -> NodeId {
    NodeId {
        index: 0,
        name: name.to_string().into(),
    }
}

fn make_effect_handler() -> (EffectHandler<String>, mpsc::Receiver<String>) {
    let (sender, receiver) = mpsc::Channel::new(BATCH_COUNT + 128);
    let port: PortName = "default".into();
    let mut senders: HashMap<PortName, Sender<String>> = HashMap::new();
    let _ = senders.insert(port.clone(), Sender::new_local_mpsc_sender(sender));
    let (_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
    let eh = EffectHandler::new(node_id("bench"), senders, Some(port), reporter);
    (eh, receiver)
}

fn make_sub(suffix: &str) -> (Box<dyn Processor<String>>, NodeId, MetricsReporter) {
    let (_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
    (
        Box::new(SuffixProcessor {
            suffix: suffix.to_string(),
        }),
        node_id("sub"),
        reporter,
    )
}

// ── Mock processor ───────────────────────────────────────────────────

/// A processor that appends a suffix with simulated compute work.
/// The busy-spin approximates the cost of a real Arrow attribute transform.
struct SuffixProcessor {
    suffix: String,
}

/// Busy-spin for approximately `ns` nanoseconds of CPU work.
/// Uses `Instant` to avoid being optimized away.
fn simulate_work(ns: u64) {
    let start = std::time::Instant::now();
    let target = std::time::Duration::from_nanos(ns);
    while start.elapsed() < target {
        std::hint::spin_loop();
    }
}

#[async_trait::async_trait(?Send)]
impl Processor<String> for SuffixProcessor {
    async fn process(
        &mut self,
        msg: Message<String>,
        effect_handler: &mut EffectHandler<String>,
    ) -> Result<(), Error> {
        if let Message::PData(data) = msg {
            simulate_work(SIMULATED_WORK_NS);
            let result = format!("{}{}", data, self.suffix);
            effect_handler
                .send_message(result)
                .await
                .map_err(Error::from)?;
        }
        Ok(())
    }
}

// ── Benchmarks ───────────────────────────────────────────────────────

fn bench_processor_chain(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    if let Some(core) = cores.iter().last() {
        let _ = core_affinity::set_for_current(*core);
    }

    let mut group = c.benchmark_group("processor_chain");
    let _ = group.throughput(Throughput::Elements(BATCH_COUNT as u64));

    for chain_len in [1, 2, 3] {
        // ── Chained: all processors inside a ProcessorChainNode ──
        let _ = group.bench_function(BenchmarkId::new("chained", chain_len), |b| {
            b.to_async(&rt).iter(|| async {
                let local = LocalSet::new();
                local
                    .run_until(async {
                        let ctx = test_pipeline_ctx();
                        let cd = ComputeDuration::new(&ctx);
                        let subs: Vec<_> =
                            (0..chain_len).map(|i| make_sub(&format!("_{i}"))).collect();

                        let mut chain = ProcessorChainNode::new(subs, cd, BATCH_COUNT + 128);
                        let (mut eh, _rx) = make_effect_handler();

                        for _ in 0..BATCH_COUNT {
                            chain
                                .process(Message::PData(black_box("hello".to_string())), &mut eh)
                                .await
                                .unwrap();
                        }
                    })
                    .await;
            });
        });

        // ── Separate: N processors with channel send/recv between each ──
        let _ = group.bench_function(BenchmarkId::new("separate", chain_len), |b| {
            b.to_async(&rt).iter(|| async {
                let local = LocalSet::new();
                local
                    .run_until(async {
                        let mut processors: Vec<SuffixProcessor> = (0..chain_len)
                            .map(|i| SuffixProcessor {
                                suffix: format!("_{i}"),
                            })
                            .collect();

                        let mut channels: Vec<(EffectHandler<String>, mpsc::Receiver<String>)> =
                            (0..chain_len).map(|_| make_effect_handler()).collect();

                        for _ in 0..BATCH_COUNT {
                            let mut data = black_box("hello".to_string());

                            for (proc, (eh, rx)) in processors.iter_mut().zip(channels.iter_mut()) {
                                proc.process(Message::PData(data), eh).await.unwrap();
                                data = rx.try_recv().unwrap();
                            }
                            let _ = black_box(data);
                        }
                    })
                    .await;
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_processor_chain);
criterion_main!(benches);
