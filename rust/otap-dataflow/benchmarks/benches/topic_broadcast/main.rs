// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Broadcast topic benchmarks against `tokio::sync::broadcast`.
//!
//! This module provides five benchmark families:
//! - `topic_broadcast_vs_tokio`: `BroadcastOnly` topic vs tokio broadcast in
//!   steady-state no-lag conditions (large capacity)
//! - `topic_mixed_broadcast_vs_tokio`: broadcast path of `TopicOptions::Mixed`
//!   vs tokio broadcast
//! - `topic_broadcast_lag_vs_tokio`: forced-lag scenario with tiny capacity to
//!   exercise lag accounting paths
//! - `topic_broadcast_tracked`: tracked-publish path comparing `first` ack mode
//!   against `all` (consensus) ack mode over an identical workload, isolating
//!   the consensus-tracking overhead
//! - `topic_broadcast_consensus_cleanup`: worst-case disconnect and lag cleanup
//!   with many outstanding consensus requirements
//!
//! Workload model:
//! - one publisher sends `MSG_COUNT` messages of fixed-size payloads
//! - `N` broadcast subscribers
//! - no-lag groups assert full fan-out delivery
//!   (`sum(received_by_subscribers) == MSG_COUNT * N`)
//! - lag group asserts lag is observed (`lagged > 0`)
//!
//! What is measured:
//! - Criterion throughput uses `Elements(MSG_COUNT)`, so results are messages
//!   published per second
//!
//! Out of scope:
//! - network I/O, serialization, and downstream processing

use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_engine::topic::{
    InMemoryBackend, RecvItem, SubscriberOptions, SubscriptionMode, TopicBroadcastAckMode,
    TopicBroadcastOnLagPolicy, TopicBroker, TopicOptions, TopicPublishOutcomeConfig,
};
use tokio::runtime::Runtime;

const MSG_COUNT: u64 = 10_000;
const MSG_SIZES: [usize; 3] = [32, 256, 4096];
const SUBSCRIBER_COUNTS: [usize; 4] = [1, 2, 4, 8];
const CLEANUP_PENDING_COUNTS: [usize; 3] = [1, 64, 1024];

const BROADCAST_CAPACITY: usize = 16_384;
const LAG_CAPACITY: usize = 64;
const LAG_SUBSCRIBERS: usize = 4;

fn make_payload(size: usize) -> Arc<Vec<u8>> {
    Arc::new(vec![42u8; size])
}

#[derive(Clone, Copy)]
struct BenchCase {
    msg_size: usize,
    num_subs: usize,
}

async fn run_topic_broadcast_case(case: BenchCase, opts: TopicOptions) {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic("bench-broadcast", opts, InMemoryBackend)
        .expect("benchmark topic creation failed");

    let mut subs: Vec<_> = (0..case.num_subs)
        .map(|_| {
            topic
                .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
                .expect("benchmark subscription failed")
        })
        .collect();

    let mut sub_handles = Vec::new();
    for mut sub in subs.drain(..) {
        sub_handles.push(tokio::spawn(async move {
            let mut count = 0u64;
            while let Ok(item) = sub.recv().await {
                match item {
                    RecvItem::Message(env) => {
                        _ = black_box(&env.payload);
                        count += 1;
                    }
                    RecvItem::Lagged { missed } => {
                        panic!("unexpected lag in no-lag broadcast benchmark: missed={missed}");
                    }
                }
            }
            count
        }));
    }

    let payload = make_payload(case.msg_size);
    for _ in 0..MSG_COUNT {
        topic
            .publish(Arc::clone(&payload))
            .await
            .expect("benchmark publish failed");
    }
    topic.close();

    let mut total = 0u64;
    for h in sub_handles {
        total += h.await.expect("subscriber task panicked");
    }
    assert_eq!(total, MSG_COUNT * case.num_subs as u64);
}

/// Tracked-publish broadcast path: every message is a tracked publish whose
/// upstream outcome is awaited. Parameterized by `ack_mode` so the *identical*
/// workload (same message count, same payload content, every subscriber
/// receives and acks every message) can be run in both `First` and `All`
/// (consensus) modes. The only difference between the two is the registry
/// snapshot + consensus registration + N-way resolution performed by `All`,
/// so the delta isolates the pure consensus-tracking overhead.
async fn run_topic_broadcast_tracked_case(case: BenchCase, ack_mode: TopicBroadcastAckMode) {
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bench-broadcast-tracked",
            TopicOptions::BroadcastOnly {
                capacity: BROADCAST_CAPACITY,
                on_lag: TopicBroadcastOnLagPolicy::Disconnect,
                ack_mode,
            },
            InMemoryBackend,
        )
        .expect("benchmark topic creation failed");

    let publisher = topic.tracked_publisher();

    let mut sub_handles = Vec::new();
    for _ in 0..case.num_subs {
        let mut sub = topic
            .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
            .expect("benchmark subscription failed");
        sub_handles.push(tokio::spawn(async move {
            let mut count = 0u64;
            while let Ok(item) = sub.recv().await {
                match item {
                    RecvItem::Message(env) => {
                        _ = black_box(&env.payload);
                        _ = sub.ack(env.id);
                        count += 1;
                    }
                    RecvItem::Lagged { missed } => {
                        panic!("unexpected lag in tracked broadcast benchmark: missed={missed}");
                    }
                }
            }
            count
        }));
    }

    let payload = make_payload(case.msg_size);
    let mut receipts = Vec::with_capacity(MSG_COUNT as usize);
    for _ in 0..MSG_COUNT {
        receipts.push(
            publisher
                .publish(Arc::clone(&payload))
                .await
                .expect("benchmark tracked publish failed"),
        );
    }
    for receipt in receipts {
        _ = black_box(receipt.wait_for_outcome().await);
    }
    topic.close();

    let mut total = 0u64;
    for h in sub_handles {
        total += h.await.expect("subscriber task panicked");
    }
    assert_eq!(total, MSG_COUNT * case.num_subs as u64);
}

async fn setup_consensus_cleanup_case(
    pending_count: usize,
    on_lag: TopicBroadcastOnLagPolicy,
) -> (
    otap_df_engine::topic::Subscription<Vec<u8>>,
    Vec<otap_df_engine::topic::TrackedPublishReceipt>,
    otap_df_engine::topic::TopicHandle<Vec<u8>>,
) {
    let capacity = pending_count.max(2).next_power_of_two();
    let broker = TopicBroker::new();
    let topic = broker
        .create_topic(
            "bench-consensus-cleanup",
            TopicOptions::BroadcastOnly {
                capacity,
                on_lag,
                ack_mode: TopicBroadcastAckMode::All,
            },
            InMemoryBackend,
        )
        .expect("benchmark topic creation failed");
    let publisher = topic.tracked_publisher_with_config(TopicPublishOutcomeConfig {
        max_in_flight: pending_count,
        timeout: Duration::from_secs(30),
    });
    let sub = topic
        .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
        .expect("benchmark subscription failed");
    let payload = make_payload(32);
    let mut receipts = Vec::with_capacity(pending_count);
    for _ in 0..pending_count {
        receipts.push(
            publisher
                .publish(Arc::clone(&payload))
                .await
                .expect("benchmark tracked publish failed"),
        );
    }
    (sub, receipts, topic)
}

async fn run_tokio_broadcast_case(case: BenchCase) {
    let (tx, _rx) = tokio::sync::broadcast::channel::<Arc<Vec<u8>>>(BROADCAST_CAPACITY);

    let mut sub_handles = Vec::new();
    for _ in 0..case.num_subs {
        let mut rx = tx.subscribe();
        sub_handles.push(tokio::spawn(async move {
            let mut count = 0u64;
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        _ = black_box(&msg);
                        count += 1;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                        panic!("unexpected lag in no-lag tokio benchmark: missed={missed}");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            count
        }));
    }

    let payload = make_payload(case.msg_size);
    for _ in 0..MSG_COUNT {
        _ = tx.send(Arc::clone(&payload));
    }
    drop(tx);

    let mut total = 0u64;
    for h in sub_handles {
        total += h.await.expect("subscriber task panicked");
    }
    assert_eq!(total, MSG_COUNT * case.num_subs as u64);
}

async fn run_topic_broadcast_lag_case(msg_size: usize) {
    let broker = TopicBroker::new();
    let topic = broker
        .create_in_memory_topic(
            "bench-broadcast-lag",
            TopicOptions::BroadcastOnly {
                capacity: LAG_CAPACITY,
                on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                ack_mode: TopicBroadcastAckMode::First,
            },
        )
        .expect("benchmark topic creation failed");

    let mut subs: Vec<_> = (0..LAG_SUBSCRIBERS)
        .map(|_| {
            topic
                .subscribe(SubscriptionMode::Broadcast, SubscriberOptions::default())
                .expect("benchmark subscription failed")
        })
        .collect();

    let payload = make_payload(msg_size);
    for _ in 0..MSG_COUNT {
        topic
            .publish(Arc::clone(&payload))
            .await
            .expect("benchmark publish failed");
    }
    topic.close();

    let mut received = 0u64;
    let mut lagged = 0u64;
    for mut sub in subs.drain(..) {
        loop {
            match sub.recv().await {
                Ok(RecvItem::Message(env)) => {
                    _ = black_box(&env.payload);
                    received += 1;
                }
                Ok(RecvItem::Lagged { missed }) => {
                    lagged += missed;
                }
                Err(_) => break,
            }
        }
    }

    assert!(lagged > 0);
    _ = black_box((received, lagged));
}

async fn run_tokio_broadcast_lag_case(msg_size: usize) {
    let (tx, _rx) = tokio::sync::broadcast::channel::<Arc<Vec<u8>>>(LAG_CAPACITY);
    let mut receivers: Vec<_> = (0..LAG_SUBSCRIBERS).map(|_| tx.subscribe()).collect();

    let payload = make_payload(msg_size);
    for _ in 0..MSG_COUNT {
        _ = tx.send(Arc::clone(&payload));
    }
    drop(tx);

    let mut received = 0u64;
    let mut lagged = 0u64;
    for rx in &mut receivers {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    _ = black_box(&msg);
                    received += 1;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                    lagged += missed;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    }

    assert!(lagged > 0);
    _ = black_box((received, lagged));
}

/// Compare broadcast-only topic mode against tokio::sync::broadcast.
fn bench_topic_broadcast_vs_tokio(c: &mut Criterion) {
    for &msg_size in &MSG_SIZES {
        let mut group = c.benchmark_group(format!("topic_broadcast_vs_tokio/{}B", msg_size));
        _ = group.throughput(Throughput::Elements(MSG_COUNT));

        for &num_subs in &SUBSCRIBER_COUNTS {
            let case = BenchCase { msg_size, num_subs };

            _ = group.bench_with_input(BenchmarkId::new("broker", num_subs), &case, |b, case| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt).iter(|| {
                    run_topic_broadcast_case(
                        *case,
                        TopicOptions::BroadcastOnly {
                            capacity: BROADCAST_CAPACITY,
                            on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                            ack_mode: TopicBroadcastAckMode::First,
                        },
                    )
                });
            });

            _ = group.bench_with_input(BenchmarkId::new("tokio", num_subs), &case, |b, case| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt).iter(|| run_tokio_broadcast_case(*case));
            });
        }

        group.finish();
    }
}

/// Compare mixed topic broadcast path against tokio::sync::broadcast.
fn bench_topic_mixed_broadcast_vs_tokio(c: &mut Criterion) {
    for &msg_size in &MSG_SIZES {
        let mut group = c.benchmark_group(format!("topic_mixed_broadcast_vs_tokio/{}B", msg_size));
        _ = group.throughput(Throughput::Elements(MSG_COUNT));

        for &num_subs in &SUBSCRIBER_COUNTS {
            let case = BenchCase { msg_size, num_subs };

            _ = group.bench_with_input(BenchmarkId::new("mixed", num_subs), &case, |b, case| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt).iter(|| {
                    run_topic_broadcast_case(
                        *case,
                        TopicOptions::Mixed {
                            balanced_capacity: TopicOptions::DEFAULT_BALANCED_CAPACITY,
                            broadcast_capacity: BROADCAST_CAPACITY,
                            on_lag: TopicBroadcastOnLagPolicy::DropOldest,
                            ack_mode: TopicBroadcastAckMode::First,
                        },
                    )
                });
            });

            _ = group.bench_with_input(BenchmarkId::new("tokio", num_subs), &case, |b, case| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt).iter(|| run_tokio_broadcast_case(*case));
            });
        }

        group.finish();
    }
}

/// Compare lag handling with a tiny broadcast buffer in broker vs tokio baseline.
fn bench_topic_broadcast_lag_vs_tokio(c: &mut Criterion) {
    for &msg_size in &MSG_SIZES {
        let mut group = c.benchmark_group(format!("topic_broadcast_lag_vs_tokio/{}B", msg_size));
        _ = group.throughput(Throughput::Elements(MSG_COUNT));

        _ = group.bench_with_input(
            BenchmarkId::new("broker", LAG_SUBSCRIBERS),
            &msg_size,
            |b, msg_size| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt)
                    .iter(|| run_topic_broadcast_lag_case(*msg_size));
            },
        );

        _ = group.bench_with_input(
            BenchmarkId::new("tokio", LAG_SUBSCRIBERS),
            &msg_size,
            |b, msg_size| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt)
                    .iter(|| run_tokio_broadcast_lag_case(*msg_size));
            },
        );

        group.finish();
    }
}

/// Measure broadcast tracked-publish overhead across fan-out widths, comparing
/// `first` ack mode (resolve on first ack, no registry) against `all`
/// (consensus) mode (resolve after every subscriber acks). Both run the
/// identical workload - same message count, same payload, every subscriber
/// receives and acks every message - so the `first` -> `all` delta isolates the
/// consensus-tracking cost.
fn bench_topic_broadcast_tracked(c: &mut Criterion) {
    for &msg_size in &MSG_SIZES {
        let mut group = c.benchmark_group(format!("topic_broadcast_tracked/{}B", msg_size));
        _ = group.throughput(Throughput::Elements(MSG_COUNT));

        for &num_subs in &SUBSCRIBER_COUNTS {
            let case = BenchCase { msg_size, num_subs };

            _ = group.bench_with_input(BenchmarkId::new("first", num_subs), &case, |b, case| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt)
                    .iter(|| run_topic_broadcast_tracked_case(*case, TopicBroadcastAckMode::First));
            });

            _ = group.bench_with_input(BenchmarkId::new("all", num_subs), &case, |b, case| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt)
                    .iter(|| run_topic_broadcast_tracked_case(*case, TopicBroadcastAckMode::All));
            });
        }

        group.finish();
    }
}

/// Measure worst-case consensus cleanup when one subscriber owes many messages.
fn bench_topic_broadcast_consensus_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("topic_broadcast_consensus_cleanup");

    for &pending_count in &CLEANUP_PENDING_COUNTS {
        _ = group.throughput(Throughput::Elements(pending_count as u64));

        _ = group.bench_with_input(
            BenchmarkId::new("disconnect", pending_count),
            &pending_count,
            |b, pending_count| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.iter_batched(
                    || {
                        rt.block_on(setup_consensus_cleanup_case(
                            *pending_count,
                            TopicBroadcastOnLagPolicy::Disconnect,
                        ))
                    },
                    |(sub, receipts, topic)| {
                        drop(sub);
                        topic.close();
                        _ = black_box(receipts);
                        _ = black_box(topic);
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        _ = group.bench_with_input(
            BenchmarkId::new("drop_oldest_lag", pending_count),
            &pending_count,
            |b, pending_count| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.iter_batched(
                    || {
                        let (sub, receipts, topic) = rt.block_on(setup_consensus_cleanup_case(
                            *pending_count,
                            TopicBroadcastOnLagPolicy::DropOldest,
                        ));
                        let payload = make_payload(32);
                        rt.block_on(async {
                            for _ in 0..pending_count.saturating_add(1) {
                                topic
                                    .publish(Arc::clone(&payload))
                                    .await
                                    .expect("benchmark untracked publish failed");
                            }
                        });
                        (sub, receipts, topic)
                    },
                    |(mut sub, receipts, topic)| {
                        let item = rt
                            .block_on(sub.recv())
                            .expect("benchmark lag receive failed");
                        assert!(matches!(item, RecvItem::Lagged { .. }));
                        topic.close();
                        _ = black_box((sub, receipts, topic));
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_topic_broadcast_vs_tokio,
    bench_topic_mixed_broadcast_vs_tokio,
    bench_topic_broadcast_lag_vs_tokio,
    bench_topic_broadcast_tracked,
    bench_topic_broadcast_consensus_cleanup
);
criterion_main!(benches);
