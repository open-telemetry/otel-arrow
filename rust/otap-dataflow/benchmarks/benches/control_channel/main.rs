// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compare the current engine control-channel path against the standalone
//! control-aware channel under heavy Ack/Nack traffic.

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_channel::mpsc;
use otap_df_control_channel::{
    AckMsg as ControlAwareAckMsg, CompletionMsg as ControlAwareCompletionMsg, ControlChannelConfig,
    ControlCmd, NackMsg as ControlAwareNackMsg, NodeControlEvent, NodeControlReceiver,
    NodeControlSender, node_channel,
};
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::local::message::{LocalReceiver as CurrentLocalReceiver, LocalSender};
use otap_df_engine::shared::message::{
    SharedReceiver as CurrentSharedReceiver, SharedSender as CurrentSharedSender,
};
use otap_df_telemetry::reporter::MetricsReporter;
use std::future::{Future, poll_fn};
use std::hint::black_box;
use std::pin::Pin;
use std::task::Poll;
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
const CANCELED_BLOCKED_SENDS: usize = 100_000;

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

fn control_aware_channel_config() -> ControlChannelConfig {
    ControlChannelConfig {
        completion_msg_capacity: CHANNEL_CAPACITY,
        completion_batch_max: COMPLETION_BATCH_MAX,
        completion_burst_limit: COMPLETION_BATCH_MAX,
    }
}

async fn is_pending_once<F>(mut future: Pin<&mut F>) -> bool
where
    F: Future,
{
    poll_fn(|cx| Poll::Ready(future.as_mut().poll(cx).is_pending())).await
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
            | NodeControlMsg::MemoryPressureChanged { .. }
            | NodeControlMsg::Shutdown { .. }
            | NodeControlMsg::DelayedData { .. }
            | NodeControlMsg::Wakeup { .. } => {
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
            | NodeControlMsg::MemoryPressureChanged { .. }
            | NodeControlMsg::Shutdown { .. }
            | NodeControlMsg::DelayedData { .. }
            | NodeControlMsg::Wakeup { .. } => {
                panic!("unexpected message in benchmark current shared receiver");
            }
        }
    }

    observed
}

async fn send_control_aware_completion(tx: &NodeControlSender<usize>, idx: usize) {
    let result = if idx.is_multiple_of(NACK_EVERY) {
        tx.send(ControlCmd::Nack(ControlAwareNackMsg::new("temporary", idx)))
            .await
    } else {
        tx.send(ControlCmd::Ack(ControlAwareAckMsg::new(idx))).await
    };
    let _ = result.expect("control-aware completion send should succeed");
}

async fn produce_control_aware(tx: NodeControlSender<usize>, scenario: Scenario) {
    for idx in 0..COMPLETION_COUNT {
        send_control_aware_completion(&tx, idx).await;

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

async fn consume_control_aware(mut rx: NodeControlReceiver<usize>) -> ObservedWork {
    let mut observed = ObservedWork::default();

    while let Some(event) = rx.recv().await {
        match event {
            NodeControlEvent::CompletionBatch(batch) => {
                observed.completion_batches += 1;
                observed.completions += batch.len();
                for completion in batch {
                    match completion {
                        ControlAwareCompletionMsg::Ack(_) | ControlAwareCompletionMsg::Nack(_) => {}
                    }
                }
            }
            NodeControlEvent::TimerTick => observed.timer_ticks += 1,
            NodeControlEvent::CollectTelemetry => observed.telemetry_ticks += 1,
            NodeControlEvent::Config { .. } => observed.configs += 1,
            NodeControlEvent::Shutdown(_) => {
                panic!("unexpected event in control-aware benchmark receiver");
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

async fn run_control_aware_workload(scenario: Scenario) -> ObservedWork {
    let (tx, rx) =
        node_channel(control_aware_channel_config()).expect("control-aware channel config valid");

    let ((), observed) = tokio::join!(
        produce_control_aware(tx, scenario),
        consume_control_aware(rx)
    );
    assert_eq!(observed.completions, COMPLETION_COUNT);
    observed
}

async fn run_control_aware_canceled_sender_churn() -> usize {
    let (tx, mut rx) =
        node_channel(control_aware_channel_config()).expect("control-aware channel config valid");

    let _ = tx
        .try_send(ControlCmd::Ack(ControlAwareAckMsg::new(0)))
        .expect("seed completion should enqueue");

    for idx in 0..CANCELED_BLOCKED_SENDS {
        let mut blocked =
            std::pin::pin!(tx.send(ControlCmd::Ack(ControlAwareAckMsg::new(idx + 1))));
        assert!(is_pending_once(blocked.as_mut()).await);
    }

    let mut live = std::pin::pin!(tx.send(ControlCmd::Ack(ControlAwareAckMsg::new(
        CANCELED_BLOCKED_SENDS + 1,
    ))));
    assert!(is_pending_once(live.as_mut()).await);

    let first_batch = rx.recv().await;
    assert!(matches!(
        first_batch,
        Some(NodeControlEvent::CompletionBatch(_))
    ));
    let _ = live
        .await
        .expect("live blocked send should complete after capacity is freed");
    let second_batch = rx.recv().await;
    assert!(matches!(
        second_batch,
        Some(NodeControlEvent::CompletionBatch(_))
    ));

    CANCELED_BLOCKED_SENDS
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
            BenchmarkId::new("control_aware", scenario.bench_name()),
            |b| {
                b.to_async(&rt).iter(|| async {
                    let local = LocalSet::new();
                    let observed = local
                        .run_until(async { run_control_aware_workload(scenario).await })
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
    }

    group.finish();

    let mut churn_group = c.benchmark_group("control_channel_blocked_sender_churn");
    let _ = churn_group.throughput(Throughput::Elements(CANCELED_BLOCKED_SENDS as u64));
    let _ = churn_group.bench_function("control_aware", |b| {
        b.to_async(&rt).iter(|| async {
            let local = LocalSet::new();
            let churned = local
                .run_until(async { run_control_aware_canceled_sender_churn().await })
                .await;
            let _ = black_box(churned);
        });
    });
    churn_group.finish();
}

criterion_group!(benches, bench_control_channels);
criterion_main!(benches);
