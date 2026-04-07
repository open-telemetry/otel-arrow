// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks comparing processor chain vs. separate processors.
//!
//! Compares the throughput of N processors wired as:
//! 1. A single `ProcessorChainNode` (all processors in one task, no inter-channels)
//! 2. Separate processors, each running in its own `spawn_local` task and
//!    connected by mpsc channels — matching how the real engine wires
//!    individual processors via `ProcessorWrapper::start()`.
//!
//! This isolates the overhead of the chain's in-memory staging-buffer approach
//! vs. the baseline of individual processor tasks with inter-task channel
//! coordination (async send/recv, task wake-ups, executor scheduling).
//!
//! The chain's value is twofold:
//! - Eliminates inter-task wake-up and channel overhead between sub-processors.
//! - Produces a correct composite duration metric (min/max/sum/count) across
//!   all sub-processors, which is impossible with separate processors.
//!
//! ## Expected results
//!
//! **`processor_chain` group** (300µs simulated work per processor):
//! Inter-task channel overhead (~500ns per hop) is <0.2% of work — well
//! within noise. Both approaches show identical throughput.
//!
//! **`processor_chain_low_work` group** (100ns simulated work per processor):
//! The chain is actually *slower* than separate tasks (~20-25% for len=1,
//! ~10% for len=3) because `ProcessorChainNode`'s internal staging buffers
//! and intermediate `BufferSlot` channels add more per-message bookkeeping
//! than a simple spawned task doing `recv → process → send`.
//!
//! This confirms the chain's value is solely the composite duration metric,
//! not throughput. At production workloads the overhead is negligible.

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
    make_sub_with_work(suffix, SIMULATED_WORK_NS)
}

fn make_sub_with_work(
    suffix: &str,
    work_ns: u64,
) -> (Box<dyn Processor<String>>, NodeId, MetricsReporter) {
    let (_rx, reporter) = MetricsReporter::create_new_and_receiver(1);
    (
        Box::new(SuffixProcessor {
            suffix: suffix.to_string(),
            work_ns,
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
                                .expect("chain process failed");
                        }
                    })
                    .await;
            });
        });

        // ── Separate: N processors each in its own spawned task ──
        let _ = group.bench_function(BenchmarkId::new("separate", chain_len), |b| {
            b.to_async(&rt).iter(|| async {
                let local = LocalSet::new();
                local
                    .run_until(async {
                        // Build a pipeline of N tasks connected by channels:
                        //   producer_tx -> [proc_0] -> [proc_1] -> ... -> [proc_N-1] -> final_rx
                        let (producer_tx, first_rx) =
                            mpsc::Channel::<String>::new(BATCH_COUNT + 128);

                        let mut prev_rx = Some(first_rx);
                        let mut final_rx: Option<mpsc::Receiver<String>> = None;

                        for i in 0..chain_len {
                            let inbox = prev_rx.take().expect("prev_rx must be set");
                            let (mut eh, out_rx) = make_effect_handler();

                            let suffix = format!("_{i}");
                            let work_ns = SIMULATED_WORK_NS;
                            let _ = tokio::task::spawn_local(async move {
                                let mut proc = SuffixProcessor { suffix, work_ns };
                                while let Ok(data) = inbox.recv().await {
                                    proc.process(Message::PData(data), &mut eh)
                                        .await
                                        .expect("process failed");
                                }
                            });

                            if i < chain_len - 1 {
                                // Intermediate: next processor reads from this one's output
                                prev_rx = Some(out_rx);
                            } else {
                                // Last: benchmark drains the final output
                                final_rx = Some(out_rx);
                            }
                        }

                        let final_rx = final_rx.expect("chain_len must be >= 1");

                        // Feed all messages into the pipeline.
                        for _ in 0..BATCH_COUNT {
                            producer_tx
                                .send(black_box("hello".to_string()))
                                .expect("send failed");
                        }
                        // Drop the producer so the first task eventually sees channel close.
                        drop(producer_tx);

                        // Drain all outputs from the last processor.
                        let mut received = 0;
                        while let Ok(data) = final_rx.recv().await {
                            let _ = black_box(data);
                            received += 1;
                        }
                        assert_eq!(received, BATCH_COUNT, "lost messages in separate pipeline");
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

                        let mut chain = ProcessorChainNode::new(subs, cd, BATCH_COUNT + 128);
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
                        let (producer_tx, first_rx) =
                            mpsc::Channel::<String>::new(BATCH_COUNT + 128);

                        let mut prev_rx = Some(first_rx);
                        let mut final_rx: Option<mpsc::Receiver<String>> = None;

                        for i in 0..chain_len {
                            let inbox = prev_rx.take().expect("prev_rx must be set");
                            let (mut eh, out_rx) = make_effect_handler();

                            let suffix = format!("_{i}");
                            let work_ns = LOW_WORK_NS;
                            let _ = tokio::task::spawn_local(async move {
                                let mut proc = SuffixProcessor { suffix, work_ns };
                                while let Ok(data) = inbox.recv().await {
                                    proc.process(Message::PData(data), &mut eh)
                                        .await
                                        .expect("process failed");
                                }
                            });

                            if i < chain_len - 1 {
                                prev_rx = Some(out_rx);
                            } else {
                                final_rx = Some(out_rx);
                            }
                        }

                        let final_rx = final_rx.expect("chain_len must be >= 1");

                        for _ in 0..BATCH_COUNT {
                            producer_tx
                                .send(black_box("hello".to_string()))
                                .expect("send failed");
                        }
                        drop(producer_tx);

                        let mut received = 0;
                        while let Ok(data) = final_rx.recv().await {
                            let _ = black_box(data);
                            received += 1;
                        }
                        assert_eq!(received, BATCH_COUNT, "lost messages in separate pipeline");
                    })
                    .await;
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_processor_chain,
    bench_processor_chain_low_work
);
criterion_main!(benches);
