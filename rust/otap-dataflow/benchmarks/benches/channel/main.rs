// SPDX-License-Identifier: Apache-2.0

//! This benchmark compares the performance of different async channels
//! - `tokio mpsc`,
//! - `flume mpsc`,
//! - our own !Send `local_mpsc`.

#![allow(missing_docs)]

use std::rc::Rc;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::{SinkExt, StreamExt};
use futures_channel::mpsc as futures_mpsc;
use tokio::task::LocalSet;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const MSG_COUNT: usize = 100_000;
const CHANNEL_SIZE: usize = 256;

fn bench_compare(c: &mut Criterion) {
    // Use a single-threaded Tokio runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    // Pin the current thread to a core
    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    let core = cores.iter().last().expect("no cores found");
    _ = core_affinity::set_for_current(*core);

    let mut group = c.benchmark_group("async_channel");
    _ = group.throughput(Throughput::Elements(MSG_COUNT as u64));
    
    // Benchmark tokio mpsc channel
    let _ = group.bench_function(BenchmarkId::new("tokio_mpsc", MSG_COUNT), |b| {
        b.to_async(&rt).iter(|| async {
            let (mut tx, mut rx) = futures_mpsc::channel(CHANNEL_SIZE);
            let pdata = Rc::new("test".to_string());

            let local = LocalSet::new();
            let _ = local.spawn_local(async move {
                for _ in 0..MSG_COUNT { _ = tx.send(pdata.clone()).await; }
            });

            let _ = local.run_until(async {
                let mut _sum = 0;
                while let Some(_v) = rx.next().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
            });
        });
    });

    // Benchmark flume mpsc channel
    let _ = group.bench_function(BenchmarkId::new("flume_mpsc", MSG_COUNT), |b| {
        b.to_async(&rt)
            .iter(|| async {
                let (tx, rx) = flume::bounded(CHANNEL_SIZE);
                let pdata = Rc::new("test".to_string());

                let local = LocalSet::new();
                let _ = local.spawn_local(async move {
                    for _ in 0..MSG_COUNT { _ = tx.send_async(pdata.clone()).await; }
                });

                let _ = local.run_until(async {
                    let mut _sum = 0;
                    while let Ok(_v) = rx.recv_async().await {
                        _sum += 1;
                    }
                    assert_eq!(_sum, MSG_COUNT);
                });

            });
    });

    // Benchmark local mpsc channel
    let _ = group.bench_function(BenchmarkId::new("local_mpsc", MSG_COUNT), |b| {
        b.to_async(&rt).iter(|| async {
            let (tx, rx) = otap_df_channel::mpsc::Channel::new(CHANNEL_SIZE);
            let pdata = Rc::new("test".to_string());

            let local = LocalSet::new();
            let _ = local.spawn_local(async move {
                for _ in 0..MSG_COUNT { _ = tx.send_async(pdata.clone()).await; }
            });

            let _ = local.run_until(async {
                let mut _sum = 0;
                while let Ok(_v) = rx.recv().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
            });

        });
    });

    // Benchmark local-sync (monoio) mpsc channel
    let _ = group.bench_function(BenchmarkId::new("local_sync_mpsc", MSG_COUNT), |b| {
        b.to_async(&rt).iter(|| async {
            let (tx, mut rx) = local_sync::mpsc::bounded::channel(CHANNEL_SIZE);
            let pdata = Rc::new("test".to_string());

            let local = LocalSet::new();
            let _ = local.spawn_local(async move {
                for _ in 0..MSG_COUNT { _ = tx.send(pdata.clone()).await; }
            });

            let _ = local.run_until(async {
                let mut _sum = 0;
                while let Some(_v) = rx.recv().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
            });

        });
    });

    // Benchmark async unsync mpsc channel
    let _ = group.bench_function(BenchmarkId::new("async_unsync_mpsc", MSG_COUNT), |b| {
        b.to_async(&rt).iter(|| async {
            let (tx, mut rx) = async_unsync::bounded::channel(CHANNEL_SIZE).into_split();
            let pdata = Rc::new("test".to_string());

            let local = LocalSet::new();
            let _ = local.spawn_local(async move {
                for _ in 0..MSG_COUNT { _ = tx.send(pdata.clone()); }
            });

            let _ = local.run_until(async {
                let mut _sum = 0;
                while let Some(_v) = rx.recv().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
            });

        });
    });

    // Benchmark unsync mpsc channel
    let _ = group.bench_function(BenchmarkId::new("unsync_mpsc", MSG_COUNT), |b| {
        b.to_async(&rt).iter(|| async {
            let (mut tx, mut rx) = unsync::spsc::channel(CHANNEL_SIZE);
            let pdata = Rc::new("test".to_string());

            let local = LocalSet::new();
            let _ = local.spawn_local(async move {
                for _ in 0..MSG_COUNT { _ = tx.send(pdata.clone()); }
            });

            let _ = local.run_until(async {
                let mut _sum = 0;
                while let Some(_v) = rx.recv().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
            });

        });
    });

    group.finish();
}

criterion_group!(benches, bench_compare);
criterion_main!(benches);