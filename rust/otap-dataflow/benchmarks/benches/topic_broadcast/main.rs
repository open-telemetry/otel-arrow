// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for topic broadcast delivery against tokio::sync::broadcast.

use std::hint::black_box;
use std::sync::Arc;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_engine::topic::{
    RecvItem, SubscriberOptions, SubscriptionMode, TopicBroker, TopicOptions,
};
use tokio::runtime::Runtime;

const MSG_COUNT: u64 = 10_000;
const MSG_SIZES: [usize; 3] = [32, 256, 4096];
const SUBSCRIBER_COUNTS: [usize; 4] = [1, 2, 4, 8];

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
        .create_in_memory_topic("bench-broadcast", opts)
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

criterion_group!(
    benches,
    bench_topic_broadcast_vs_tokio,
    bench_topic_mixed_broadcast_vs_tokio,
    bench_topic_broadcast_lag_vs_tokio
);
criterion_main!(benches);
