// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Balanced topic benchmarks against a flume MPMC baseline.
//!
//! Workload model:
//! - one publisher sends `MSG_COUNT` messages of fixed-size payloads
//! - one balanced consumer group (`g1`) with `N` subscribers
//! - bounded queue capacity is aligned across implementations (`4096`)
//!
//! What is measured:
//! - Criterion throughput is configured as `Elements(MSG_COUNT)`, so reported
//!   throughput is messages published per second
//! - each message must be delivered exactly once across all subscribers
//!   (`sum(received_by_subscribers) == MSG_COUNT`)
//!
//! Interpretation:
//! - isolates the cost of balanced dispatch and receiver contention as
//!   subscriber count increases
//! - does not include network I/O, serialization, or downstream processing

use std::hint::black_box;
use std::sync::Arc;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_engine::topic::{
    InMemoryBackend, RecvItem, SubscriberOptions, SubscriptionMode, TopicBroker, TopicOptions,
};
use tokio::runtime::Runtime;

const MSG_COUNT: u64 = 10_000;
const MSG_SIZES: [usize; 3] = [32, 256, 4096];
const SUBSCRIBER_COUNTS: [usize; 4] = [1, 2, 4, 8];

fn make_payload(size: usize) -> Arc<Vec<u8>> {
    Arc::new(vec![42u8; size])
}

#[derive(Clone, Copy)]
struct BenchCase {
    msg_size: usize,
    num_subs: usize,
}

async fn run_broker_case(case: BenchCase) {
    let broker = TopicBroker::new();
    let opts = TopicOptions::BalancedOnly { capacity: 4096 };
    let topic = broker
        .create_topic("bench", opts, InMemoryBackend)
        .expect("benchmark topic creation failed");

    // Create subscribers.
    let mut subs: Vec<_> = (0..case.num_subs)
        .map(|_| {
            topic
                .subscribe(
                    SubscriptionMode::Balanced { group: "g1".into() },
                    SubscriberOptions::default(),
                )
                .expect("benchmark subscription failed")
        })
        .collect();

    // Spawn subscriber tasks.
    let mut sub_handles = Vec::new();
    for mut sub in subs.drain(..) {
        sub_handles.push(tokio::spawn(async move {
            let mut count = 0u64;
            while let Ok(RecvItem::Message(env)) = sub.recv().await {
                _ = black_box(&env.payload);
                count += 1;
            }
            count
        }));
    }

    // Publish.
    let payload = make_payload(case.msg_size);
    for _ in 0..MSG_COUNT {
        topic
            .publish(Arc::clone(&payload))
            .await
            .expect("benchmark publish failed");
    }
    topic.close();

    // Wait for subscribers.
    let mut total = 0u64;
    for h in sub_handles {
        total += h.await.expect("subscriber task panicked");
    }
    assert_eq!(total, MSG_COUNT);
}

async fn run_flume_case(case: BenchCase) {
    let (tx, rx) = flume::bounded::<Arc<Vec<u8>>>(4096);

    // Spawn subscriber tasks.
    let mut sub_handles = Vec::new();
    for _ in 0..case.num_subs {
        let rx = rx.clone();
        sub_handles.push(tokio::spawn(async move {
            let mut count = 0u64;
            while let Ok(msg) = rx.recv_async().await {
                _ = black_box(&msg);
                count += 1;
            }
            count
        }));
    }
    // Drop the extra rx so flume knows when senders are done.
    drop(rx);

    // Publish.
    let payload = make_payload(case.msg_size);
    for _ in 0..MSG_COUNT {
        tx.send_async(Arc::clone(&payload))
            .await
            .expect("flume send failed");
    }
    drop(tx);

    // Wait for subscribers.
    let mut total = 0u64;
    for h in sub_handles {
        total += h.await.expect("subscriber task panicked");
    }
    assert_eq!(total, MSG_COUNT);
}

/// Benchmark broker and flume side-by-side for each balanced-case parameter set.
fn bench_topic_balanced_vs_flume(c: &mut Criterion) {
    for &msg_size in &MSG_SIZES {
        let mut group = c.benchmark_group(format!("topic_balanced_vs_flume/{}B", msg_size));
        _ = group.throughput(Throughput::Elements(MSG_COUNT));

        for &num_subs in &SUBSCRIBER_COUNTS {
            let case = BenchCase { msg_size, num_subs };

            _ = group.bench_with_input(BenchmarkId::new("broker", num_subs), &case, |b, case| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt).iter(|| run_broker_case(*case));
            });

            _ = group.bench_with_input(BenchmarkId::new("flume", num_subs), &case, |b, case| {
                let rt = Runtime::new().expect("tokio runtime creation failed");
                b.to_async(&rt).iter(|| run_flume_case(*case));
            });
        }

        group.finish();
    }
}

criterion_group!(benches, bench_topic_balanced_vs_flume);
criterion_main!(benches);
