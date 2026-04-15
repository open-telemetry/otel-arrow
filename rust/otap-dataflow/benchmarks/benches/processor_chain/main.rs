// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks comparing processor chain vs. separate processors.
//!
//! Compares the throughput of N processors wired as:
//! 1. A single `ProcessorChainNode` (all processors in one task, no inter-channels)
//! 2. Separate processors in the same task, connected by mpsc channels —
//!    isolating per-message channel send/recv overhead without pipelining.
//!
//! Both variants process messages sequentially: one message completes all
//! N stages before the next starts.  This ensures the comparison measures
//! the chain's elimination of inter-processor channels rather than
//! concurrent-vs-sequential scheduling differences.
//!
//! The chain's value is the ability to produce a correct composite duration
//! metric (min/max/sum/count) across all sub-processors, which is impossible
//! with separate processors.
//!
//! ## Expected results
//!
//! **`processor_chain_high_work` group** (~100µs simulated work per processor):
//! Both approaches show near-identical throughput — the chain's overhead is
//! negligible relative to real processor work.
//!
//! **`processor_chain_low_work` group** (100ns simulated work per processor):
//! The chain is slower for `len>=2` because it inserts `yield_now().await`
//! between stages for scheduling fairness.  Each yield is a ~1µs context
//! switch, which dominates when per-stage work is only 100ns.  The separate
//! variant has channel send/recv overhead (~200ns/stage) but no yield.
//! At production work levels (~100µs+) the yield cost is negligible.

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_channel::mpsc;
use otap_df_config::PortName;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::error::Error;
use otap_df_engine::inline_processor::{InlineOutput, InlineProcessor};
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
/// Set to ~100µs to approximate a real attribute transform on an Arrow
/// record batch while keeping benchmark runtime reasonable.
const SIMULATED_WORK_NS: u64 = 100_000;

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

fn make_sub(suffix: &str) -> Box<dyn InlineProcessor<String>> {
    make_sub_with_work(suffix, SIMULATED_WORK_NS)
}

fn make_sub_with_work(suffix: &str, work_ns: u64) -> Box<dyn InlineProcessor<String>> {
    Box::new(SuffixProcessor {
        suffix: suffix.to_string(),
        work_ns,
    })
}

// ── Mock processor ───────────────────────────────────────────────────

/// A processor that appends a suffix with simulated compute work.
/// The busy-spin approximates the cost of a real Arrow attribute transform.
struct SuffixProcessor {
    suffix: String,
    work_ns: u64,
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

impl InlineProcessor<String> for SuffixProcessor {
    fn process_inline(&mut self, data: String) -> Result<InlineOutput<String>, Error> {
        simulate_work(self.work_ns);
        Ok(InlineOutput::Forward(format!("{}{}", data, self.suffix)))
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
            simulate_work(self.work_ns);
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

fn bench_processor_chain_high_work(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    if let Some(core) = cores.iter().last() {
        let _ = core_affinity::set_for_current(*core);
    }

    let mut group = c.benchmark_group("processor_chain_high_work");
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

                        let mut chain = ProcessorChainNode::new(subs, cd, false);
                        let (mut eh, _rx) = make_effect_handler();

                        for _ in 0..BATCH_COUNT {
                            chain
                                .process(Message::PData(black_box("hello".to_string())), &mut eh)
                                .await
                                .expect("chain process failed");
                        }
                    })
                    .await;
            });
        });

        // ── Separate: N processors in same task, channels between each ──
        // Each message passes through all N processors sequentially,
        // with channel send/recv between each stage. This isolates the
        // per-message channel overhead without pipelining effects.
        let _ = group.bench_function(BenchmarkId::new("separate", chain_len), |b| {
            b.to_async(&rt).iter(|| async {
                let local = LocalSet::new();
                local
                    .run_until(async {
                        // Build N (processor, effect_handler, receiver) tuples.
                        let mut stages: Vec<(
                            SuffixProcessor,
                            EffectHandler<String>,
                            mpsc::Receiver<String>,
                        )> = (0..chain_len)
                            .map(|i| {
                                let proc = SuffixProcessor {
                                    suffix: format!("_{i}"),
                                    work_ns: SIMULATED_WORK_NS,
                                };
                                let (eh, rx) = make_effect_handler();
                                (proc, eh, rx)
                            })
                            .collect();

                        for _ in 0..BATCH_COUNT {
                            let mut current = black_box("hello".to_string());
                            for (proc, eh, rx) in stages.iter_mut() {
                                Processor::process(proc, Message::PData(current), eh)
                                    .await
                                    .expect("process failed");
                                current = rx.try_recv().expect("missing output");
                            }
                            let _ = black_box(current);
                        }
                    })
                    .await;
            });
        });
    }

    group.finish();
}

/// Low-work variant: 100ns per processor to surface inter-task overhead.
const LOW_WORK_NS: u64 = 100;

fn bench_processor_chain_low_work(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    if let Some(core) = cores.iter().last() {
        let _ = core_affinity::set_for_current(*core);
    }

    let mut group = c.benchmark_group("processor_chain_low_work");
    let _ = group.throughput(Throughput::Elements(BATCH_COUNT as u64));

    for chain_len in [1, 2, 3] {
        // ── Chained ──
        let _ = group.bench_function(BenchmarkId::new("chained", chain_len), |b| {
            b.to_async(&rt).iter(|| async {
                let local = LocalSet::new();
                local
                    .run_until(async {
                        let ctx = test_pipeline_ctx();
                        let cd = ComputeDuration::new(&ctx);
                        let subs: Vec<_> = (0..chain_len)
                            .map(|i| make_sub_with_work(&format!("_{i}"), LOW_WORK_NS))
                            .collect();

                        let mut chain = ProcessorChainNode::new(subs, cd, false);
                        let (mut eh, _rx) = make_effect_handler();

                        for _ in 0..BATCH_COUNT {
                            chain
                                .process(Message::PData(black_box("hello".to_string())), &mut eh)
                                .await
                                .expect("chain process failed");
                        }
                    })
                    .await;
            });
        });

        // ── Separate ──
        let _ = group.bench_function(BenchmarkId::new("separate", chain_len), |b| {
            b.to_async(&rt).iter(|| async {
                let local = LocalSet::new();
                local
                    .run_until(async {
                        let mut stages: Vec<(
                            SuffixProcessor,
                            EffectHandler<String>,
                            mpsc::Receiver<String>,
                        )> = (0..chain_len)
                            .map(|i| {
                                let proc = SuffixProcessor {
                                    suffix: format!("_{i}"),
                                    work_ns: LOW_WORK_NS,
                                };
                                let (eh, rx) = make_effect_handler();
                                (proc, eh, rx)
                            })
                            .collect();

                        for _ in 0..BATCH_COUNT {
                            let mut current = black_box("hello".to_string());
                            for (proc, eh, rx) in stages.iter_mut() {
                                Processor::process(proc, Message::PData(current), eh)
                                    .await
                                    .expect("process failed");
                                current = rx.try_recv().expect("missing output");
                            }
                            let _ = black_box(current);
                        }
                    })
                    .await;
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_processor_chain_high_work,
    bench_processor_chain_low_work
);
criterion_main!(benches);
