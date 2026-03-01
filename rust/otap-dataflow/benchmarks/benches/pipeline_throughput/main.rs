// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline throughput benchmark.
//!
//! Measures the cost of passing empty requests through a minimal pipeline
//! (receiver → noop exporter) under four conditions:
//!
//! (a) **fire_and_forget** — no backpressure (`wait_for_result=false` equivalent).
//! (b) **backpressure** — receiver subscribes for ACKs, waits for each ack.
//! (c) **backpressure_normal_metrics** — same with `MetricLevel::Normal` overhead.
//! (d) **backpressure_detailed_metrics** — same with `MetricLevel::Detailed` overhead.

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_channel::mpsc;
use otap_df_config::SignalType;
use otap_df_engine::Interests;
use otap_df_engine::control::{
    AckMsg, NodeControlMsg, PipelineControlMsg, UserCallData, nanos_since_epoch,
};
use otap_df_engine::message::Sender;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::OtlpProtoBytes;
use std::rc::Rc;
use tokio::task::LocalSet;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const MSG_COUNT: usize = 100_000;
const CHANNEL_SIZE: usize = 256;
const PRODUCER_NODE_ID: usize = 0;

/// Create an empty OtapPdata with no payload bytes.
fn empty_pdata() -> OtapPdata {
    OtapPdata::new(
        Context::with_capacity(2),
        OtlpProtoBytes::empty(SignalType::Logs).into(),
    )
}

fn bench_pipeline_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    let core = cores.iter().last().expect("no cores found");
    _ = core_affinity::set_for_current(*core);

    let mut group = c.benchmark_group("pipeline_throughput");
    _ = group.throughput(Throughput::Elements(MSG_COUNT as u64));

    // (a) Fire-and-forget: send empty pdata, consumer drains without acking.
    _ = group.bench_function(
        BenchmarkId::new("fire_and_forget", MSG_COUNT),
        |b| {
            b.to_async(&rt).iter(|| async {
                let (pdata_tx, pdata_rx) = mpsc::Channel::new(CHANNEL_SIZE);
                let template = Rc::new(empty_pdata());

                let local = LocalSet::new();
                let t = template.clone();
                _ = local.spawn_local(async move {
                    for _ in 0..MSG_COUNT {
                        _ = pdata_tx.send_async((*t).clone()).await;
                    }
                });

                _ = local.spawn_local(async move {
                    for _ in 0..MSG_COUNT {
                        let _ = pdata_rx.recv().await;
                    }
                });

                local.await;
            });
        },
    );

    // (b)-(d): Backpressure with ack routing via a lightweight controller loop.
    let scenarios: &[(&str, Interests)] = &[
        ("backpressure", Interests::ACKS),
        (
            "backpressure_normal_metrics",
            Interests::ACKS | Interests::PIPELINE_METRICS,
        ),
        (
            "backpressure_detailed_metrics",
            Interests::ACKS | Interests::PIPELINE_METRICS | Interests::ENTRY_TIMESTAMP,
        ),
    ];

    for &(name, interests) in scenarios {
        _ = group.bench_function(BenchmarkId::new(name, MSG_COUNT), |b| {
            b.to_async(&rt).iter(|| {
                let interests = interests;
                async move {
                    // pdata channel: producer → consumer
                    let (pdata_tx, pdata_rx) = mpsc::Channel::<OtapPdata>::new(CHANNEL_SIZE);

                    // pipeline-ctrl channel: consumer → controller
                    let (ctrl_tx, ctrl_rx) =
                        mpsc::Channel::<PipelineControlMsg<OtapPdata>>::new(CHANNEL_SIZE);

                    // ack channel: controller → producer
                    let (ack_tx, ack_rx) =
                        mpsc::Channel::<NodeControlMsg<OtapPdata>>::new(CHANNEL_SIZE);

                    let local = LocalSet::new();

                    // Consumer: receive pdata, send DeliverAck to controller.
                    _ = local.spawn_local(async move {
                        for _ in 0..MSG_COUNT {
                            let pdata = pdata_rx.recv().await.expect("recv pdata");
                            _ = ctrl_tx
                                .send_async(PipelineControlMsg::DeliverAck {
                                    ack: AckMsg::new(pdata),
                                })
                                .await;
                        }
                    });

                    // Controller: pop frame from context, route ack to producer.
                    let ack_sender = Sender::new_local_mpsc_sender(ack_tx);
                    _ = local.spawn_local(async move {
                        for _ in 0..MSG_COUNT {
                            let msg = ctrl_rx.recv().await.expect("recv ctrl");
                            if let PipelineControlMsg::DeliverAck { ack } = msg {
                                if let Some((_node_id, ack)) = Context::next_ack(ack) {
                                    _ = ack_sender.send(NodeControlMsg::Ack(ack)).await;
                                }
                            }
                        }
                    });

                    // Producer: build pdata with subscription, send, wait for ack.
                    let stamp_time = interests.contains(Interests::ENTRY_TIMESTAMP);
                    _ = local.spawn_local(async move {
                        for _ in 0..MSG_COUNT {
                            let mut pdata = empty_pdata();
                            pdata = pdata.test_subscribe_to(
                                interests,
                                UserCallData::default(),
                                PRODUCER_NODE_ID,
                            );
                            if stamp_time {
                                pdata.test_stamp_top_time(nanos_since_epoch());
                            }
                            _ = pdata_tx.send_async(pdata).await;
                            let _ack = ack_rx.recv().await.expect("recv ack");
                        }
                    });

                    local.await;
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_pipeline_throughput);
criterion_main!(benches);
