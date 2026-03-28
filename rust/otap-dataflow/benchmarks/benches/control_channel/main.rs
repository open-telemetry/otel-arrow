// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compare the current engine control-channel path against the experimental
//! control-aware channel under heavy Ack/Nack traffic.

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_channel::mpsc;
use otap_df_control_channel::local::{self, LocalNodeControlReceiver, LocalNodeControlSender};
use otap_df_control_channel::shared::{self, SharedNodeControlReceiver, SharedNodeControlSender};
use otap_df_control_channel::{
    AckMsg as ProtoAckMsg, CompletionMsg as ProtoCompletionMsg, ControlChannelConfig, ControlCmd,
    NackMsg as ProtoNackMsg, NodeControlEvent,
};
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::local::message::{LocalReceiver as CurrentLocalReceiver, LocalSender};
use otap_df_engine::shared::message::{
    SharedReceiver as CurrentSharedReceiver, SharedSender as CurrentSharedSender,
};
use otap_df_telemetry::reporter::MetricsReporter;
use std::hint::black_box;
use tokio::runtime::Builder;
use tokio::task::LocalSet;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const COMPLETION_COUNT: usize = 100_000;
const CHANNEL_CAPACITY: usize = 1_024;
const COMPLETION_BATCH_MAX: usize = 32;
const NACK_EVERY: usize = 8;
const TIMER_EVERY: usize = 64;
const TELEMETRY_EVERY: usize = 256;
const CONFIG_EVERY: usize = 1_024;

#[derive(Clone, Copy, Debug)]
enum Scenario {
    AckNackOnly,
    AckNackWithControlNoise,
}

impl Scenario {
    const fn bench_name(self) -> &'static str {
        match self {
            Self::AckNackOnly => "ack_nack_only",
            Self::AckNackWithControlNoise => "ack_nack_with_control_noise",
        }
    }

    const fn has_control_noise(self) -> bool {
        matches!(self, Self::AckNackWithControlNoise)
    }
}

#[derive(Debug, Default)]
struct ObservedWork {
    completions: usize,
    completion_batches: usize,
    timer_ticks: usize,
    telemetry_ticks: usize,
    configs: usize,
}

fn proto_channel_config() -> ControlChannelConfig {
    ControlChannelConfig {
        completion_msg_capacity: CHANNEL_CAPACITY,
        completion_batch_max: COMPLETION_BATCH_MAX,
        completion_burst_limit: COMPLETION_BATCH_MAX,
    }
}

fn build_metrics_reporter() -> MetricsReporter {
    MetricsReporter::create_new_and_receiver(16).1
}

async fn produce_current_local(tx: LocalSender<NodeControlMsg<usize>>, scenario: Scenario) {
    let metrics_reporter = build_metrics_reporter();

    for idx in 0..COMPLETION_COUNT {
        if idx % NACK_EVERY == 0 {
            tx.send(NodeControlMsg::Nack(NackMsg::new("temporary", idx)))
                .await
                .expect("nack should enqueue");
        } else {
            tx.send(NodeControlMsg::Ack(AckMsg::new(idx)))
                .await
                .expect("ack should enqueue");
        }

        if scenario.has_control_noise() {
            if idx % TIMER_EVERY == 0 {
                tx.send(NodeControlMsg::TimerTick {})
                    .await
                    .expect("timer tick should enqueue");
            }
            if idx % TELEMETRY_EVERY == 0 {
                tx.send(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: metrics_reporter.clone(),
                })
                .await
                .expect("telemetry collection should enqueue");
            }
            if idx % CONFIG_EVERY == 0 {
                tx.send(NodeControlMsg::Config {
                    config: serde_json::json!({ "seq": idx }),
                })
                .await
                .expect("config should enqueue");
            }
        }
    }
}

async fn consume_current_local(
    mut rx: CurrentLocalReceiver<NodeControlMsg<usize>>,
) -> ObservedWork {
    let mut observed = ObservedWork::default();

    while let Ok(msg) = rx.recv().await {
        match msg {
            NodeControlMsg::Ack(_) | NodeControlMsg::Nack(_) => observed.completions += 1,
            NodeControlMsg::TimerTick {} => observed.timer_ticks += 1,
            NodeControlMsg::CollectTelemetry { .. } => observed.telemetry_ticks += 1,
            NodeControlMsg::Config { .. } => observed.configs += 1,
            NodeControlMsg::DrainIngress { .. }
            | NodeControlMsg::Shutdown { .. }
            | NodeControlMsg::DelayedData { .. } => {
                panic!("unexpected message in benchmark current local receiver");
            }
        }
    }

    observed
}

async fn produce_current_shared(
    tx: CurrentSharedSender<NodeControlMsg<usize>>,
    scenario: Scenario,
) {
    let metrics_reporter = build_metrics_reporter();

    for idx in 0..COMPLETION_COUNT {
        if idx % NACK_EVERY == 0 {
            tx.send(NodeControlMsg::Nack(NackMsg::new("temporary", idx)))
                .await
                .expect("nack should enqueue");
        } else {
            tx.send(NodeControlMsg::Ack(AckMsg::new(idx)))
                .await
                .expect("ack should enqueue");
        }

        if scenario.has_control_noise() {
            if idx % TIMER_EVERY == 0 {
                tx.send(NodeControlMsg::TimerTick {})
                    .await
                    .expect("timer tick should enqueue");
            }
            if idx % TELEMETRY_EVERY == 0 {
                tx.send(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: metrics_reporter.clone(),
                })
                .await
                .expect("telemetry collection should enqueue");
            }
            if idx % CONFIG_EVERY == 0 {
                tx.send(NodeControlMsg::Config {
                    config: serde_json::json!({ "seq": idx }),
                })
                .await
                .expect("config should enqueue");
            }
        }
    }
}

async fn consume_current_shared(
    mut rx: CurrentSharedReceiver<NodeControlMsg<usize>>,
) -> ObservedWork {
    let mut observed = ObservedWork::default();

    while let Ok(msg) = rx.recv().await {
        match msg {
            NodeControlMsg::Ack(_) | NodeControlMsg::Nack(_) => observed.completions += 1,
            NodeControlMsg::TimerTick {} => observed.timer_ticks += 1,
            NodeControlMsg::CollectTelemetry { .. } => observed.telemetry_ticks += 1,
            NodeControlMsg::Config { .. } => observed.configs += 1,
            NodeControlMsg::DrainIngress { .. }
            | NodeControlMsg::Shutdown { .. }
            | NodeControlMsg::DelayedData { .. } => {
                panic!("unexpected message in benchmark current shared receiver");
            }
        }
    }

    observed
}

async fn send_proto_completion_local(tx: &LocalNodeControlSender<usize>, idx: usize) {
    let result = if idx.is_multiple_of(NACK_EVERY) {
        tx.send(ControlCmd::Nack(ProtoNackMsg::new("temporary", idx)))
            .await
    } else {
        tx.send(ControlCmd::Ack(ProtoAckMsg::new(idx))).await
    };
    let _ = result.expect("prototype local completion send should succeed");
}

async fn send_proto_completion_shared(tx: &SharedNodeControlSender<usize>, idx: usize) {
    let result = if idx.is_multiple_of(NACK_EVERY) {
        tx.send(ControlCmd::Nack(ProtoNackMsg::new("temporary", idx)))
            .await
    } else {
        tx.send(ControlCmd::Ack(ProtoAckMsg::new(idx))).await
    };
    let _ = result.expect("prototype shared completion send should succeed");
}

async fn produce_proto_local(tx: LocalNodeControlSender<usize>, scenario: Scenario) {
    for idx in 0..COMPLETION_COUNT {
        send_proto_completion_local(&tx, idx).await;

        if scenario.has_control_noise() {
            if idx % TIMER_EVERY == 0 {
                let result = tx.try_send(ControlCmd::TimerTick);
                assert!(result.is_ok(), "timer tick should not fail");
            }
            if idx % TELEMETRY_EVERY == 0 {
                let result = tx.try_send(ControlCmd::CollectTelemetry);
                assert!(result.is_ok(), "telemetry tick should not fail");
            }
            if idx % CONFIG_EVERY == 0 {
                let result = tx.try_send(ControlCmd::Config {
                    config: serde_json::json!({ "seq": idx }),
                });
                assert!(result.is_ok(), "config should not fail");
            }
        }
    }
}

async fn produce_proto_shared(tx: SharedNodeControlSender<usize>, scenario: Scenario) {
    for idx in 0..COMPLETION_COUNT {
        send_proto_completion_shared(&tx, idx).await;

        if scenario.has_control_noise() {
            let timer_result = if idx % TIMER_EVERY == 0 {
                Some(tx.try_send(ControlCmd::TimerTick))
            } else {
                None
            };
            if let Some(Err(err)) = timer_result {
                panic!("timer tick should not fail: {err}");
            }

            let telemetry_result = if idx % TELEMETRY_EVERY == 0 {
                Some(tx.try_send(ControlCmd::CollectTelemetry))
            } else {
                None
            };
            if let Some(Err(err)) = telemetry_result {
                panic!("telemetry tick should not fail: {err}");
            }

            let config_result = if idx % CONFIG_EVERY == 0 {
                Some(tx.try_send(ControlCmd::Config {
                    config: serde_json::json!({ "seq": idx }),
                }))
            } else {
                None
            };
            if let Some(Err(err)) = config_result {
                panic!("config should not fail: {err}");
            }
        }
    }
}

async fn consume_proto_local(mut rx: LocalNodeControlReceiver<usize>) -> ObservedWork {
    let mut observed = ObservedWork::default();

    while let Some(event) = rx.recv().await {
        match event {
            NodeControlEvent::CompletionBatch(batch) => {
                observed.completion_batches += 1;
                observed.completions += batch.len();
                for completion in batch {
                    match completion {
                        ProtoCompletionMsg::Ack(_) | ProtoCompletionMsg::Nack(_) => {}
                    }
                }
            }
            NodeControlEvent::TimerTick => observed.timer_ticks += 1,
            NodeControlEvent::CollectTelemetry => observed.telemetry_ticks += 1,
            NodeControlEvent::Config { .. } => observed.configs += 1,
            NodeControlEvent::Shutdown(_) => {
                panic!("unexpected event in prototype local benchmark receiver");
            }
        }
    }

    observed
}

async fn consume_proto_shared(mut rx: SharedNodeControlReceiver<usize>) -> ObservedWork {
    let mut observed = ObservedWork::default();

    while let Some(event) = rx.recv().await {
        match event {
            NodeControlEvent::CompletionBatch(batch) => {
                observed.completion_batches += 1;
                observed.completions += batch.len();
                for completion in batch {
                    match completion {
                        ProtoCompletionMsg::Ack(_) | ProtoCompletionMsg::Nack(_) => {}
                    }
                }
            }
            NodeControlEvent::TimerTick => observed.timer_ticks += 1,
            NodeControlEvent::CollectTelemetry => observed.telemetry_ticks += 1,
            NodeControlEvent::Config { .. } => observed.configs += 1,
            NodeControlEvent::Shutdown(_) => {
                panic!("unexpected event in prototype shared benchmark receiver");
            }
        }
    }

    observed
}

async fn run_current_local_workload(scenario: Scenario) -> ObservedWork {
    let (tx_raw, rx_raw) = mpsc::Channel::new(CHANNEL_CAPACITY);
    let tx = LocalSender::mpsc(tx_raw);
    let rx = CurrentLocalReceiver::mpsc(rx_raw);

    let ((), observed) = tokio::join!(
        produce_current_local(tx, scenario),
        consume_current_local(rx)
    );
    assert_eq!(observed.completions, COMPLETION_COUNT);
    observed
}

async fn run_current_shared_workload(scenario: Scenario) -> ObservedWork {
    let (tx_raw, rx_raw) = tokio::sync::mpsc::channel(CHANNEL_CAPACITY);
    let tx = CurrentSharedSender::mpsc(tx_raw);
    let rx = CurrentSharedReceiver::mpsc(rx_raw);

    let ((), observed) = tokio::join!(
        produce_current_shared(tx, scenario),
        consume_current_shared(rx)
    );
    assert_eq!(observed.completions, COMPLETION_COUNT);
    observed
}

async fn run_proto_local_workload(scenario: Scenario) -> ObservedWork {
    let (tx, rx) =
        local::node_channel(proto_channel_config()).expect("prototype channel config valid");

    let ((), observed) = tokio::join!(produce_proto_local(tx, scenario), consume_proto_local(rx));
    assert_eq!(observed.completions, COMPLETION_COUNT);
    observed
}

async fn run_proto_shared_workload(scenario: Scenario) -> ObservedWork {
    let (tx, rx) =
        shared::node_channel(proto_channel_config()).expect("prototype channel config valid");

    let ((), observed) = tokio::join!(produce_proto_shared(tx, scenario), consume_proto_shared(rx));
    assert_eq!(observed.completions, COMPLETION_COUNT);
    observed
}

fn bench_control_channels(c: &mut Criterion) {
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    let core = cores.iter().last().expect("no cores found");
    _ = core_affinity::set_for_current(*core);

    let mut group = c.benchmark_group("control_channel_ack_nack");
    _ = group.throughput(Throughput::Elements(COMPLETION_COUNT as u64));

    for scenario in [Scenario::AckNackOnly, Scenario::AckNackWithControlNoise] {
        let _ = group.bench_function(
            BenchmarkId::new("current_local", scenario.bench_name()),
            |b| {
                b.to_async(&rt).iter(|| async {
                    let local = LocalSet::new();
                    let observed = local
                        .run_until(async { run_current_local_workload(scenario).await })
                        .await;
                    let _ = black_box(observed);
                });
            },
        );

        let _ = group.bench_function(
            BenchmarkId::new("prototype_local", scenario.bench_name()),
            |b| {
                b.to_async(&rt).iter(|| async {
                    let local = LocalSet::new();
                    let observed = local
                        .run_until(async { run_proto_local_workload(scenario).await })
                        .await;
                    let _ = black_box(observed);
                });
            },
        );

        let _ = group.bench_function(
            BenchmarkId::new("current_shared", scenario.bench_name()),
            |b| {
                b.to_async(&rt).iter(|| async {
                    let observed = run_current_shared_workload(scenario).await;
                    let _ = black_box(observed);
                });
            },
        );

        let _ = group.bench_function(
            BenchmarkId::new("prototype_shared", scenario.bench_name()),
            |b| {
                b.to_async(&rt).iter(|| async {
                    let observed = run_proto_shared_workload(scenario).await;
                    let _ = black_box(observed);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_control_channels);
criterion_main!(benches);
