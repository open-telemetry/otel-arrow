// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This benchmark compares the performance of different async channels
//! - `tokio mpsc`,
//! - `flume mpsc`,
//! - our own !Send `local_mpsc`.

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{SinkExt, StreamExt};
use futures_channel::mpsc as futures_mpsc;
use std::rc::Rc;
use tokio::runtime::LocalOptions;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const MSG_COUNT: usize = 100_000;
const CHANNEL_SIZE: usize = 256;

fn bench_compare(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .name("async-channel-bench")
        .build_local(LocalOptions::default())
        .expect("failed to build local Tokio runtime");

    // Pin the current thread to a core
    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    let core = cores.iter().last().expect("no cores found");
    _ = core_affinity::set_for_current(*core);

    let mut group = c.benchmark_group("async_channel");
    _ = group.throughput(Throughput::Elements(MSG_COUNT as u64));

    // Benchmark tokio mpsc channel
    let _ = group.bench_function(BenchmarkId::new("tokio_mpsc", MSG_COUNT), |b| {
        b.iter(|| {
            rt.block_on(async {
                let (mut tx, mut rx) = futures_mpsc::channel(CHANNEL_SIZE);
                let pdata = Rc::new("test".to_string());

                let send_handle = tokio::task::spawn_local(async move {
                    for _ in 0..MSG_COUNT {
                        _ = tx.send(pdata.clone()).await;
                    }
                });

                let mut _sum = 0;
                while let Some(_v) = rx.next().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
                send_handle.await.expect("sender task should join");
            });
        });
    });

    // Benchmark flume mpsc channel
    let _ = group.bench_function(BenchmarkId::new("flume_mpsc", MSG_COUNT), |b| {
        b.iter(|| {
            rt.block_on(async {
                let (tx, rx) = flume::bounded(CHANNEL_SIZE);
                let pdata = Rc::new("test".to_string());

                let send_handle = tokio::task::spawn_local(async move {
                    for _ in 0..MSG_COUNT {
                        _ = tx.send_async(pdata.clone()).await;
                    }
                });

                let mut _sum = 0;
                while let Ok(_v) = rx.recv_async().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
                send_handle.await.expect("sender task should join");
            });
        });
    });

    // Benchmark local mpsc channel
    let _ = group.bench_function(BenchmarkId::new("local_mpsc", MSG_COUNT), |b| {
        b.iter(|| {
            rt.block_on(async {
                let (tx, rx) = otap_df_channel::mpsc::Channel::new(CHANNEL_SIZE);
                let pdata = Rc::new("test".to_string());

                let send_handle = tokio::task::spawn_local(async move {
                    for _ in 0..MSG_COUNT {
                        _ = tx.send_async(pdata.clone()).await;
                    }
                });

                let mut _sum = 0;
                while let Ok(_v) = rx.recv().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
                send_handle.await.expect("sender task should join");
            });
        });
    });

    // Benchmark local-sync (monoio) mpsc channel
    let _ = group.bench_function(BenchmarkId::new("local_sync_mpsc", MSG_COUNT), |b| {
        b.iter(|| {
            rt.block_on(async {
                let (tx, mut rx) = local_sync::mpsc::bounded::channel(CHANNEL_SIZE);
                let pdata = Rc::new("test".to_string());

                let send_handle = tokio::task::spawn_local(async move {
                    for _ in 0..MSG_COUNT {
                        _ = tx.send(pdata.clone()).await;
                    }
                });

                let mut _sum = 0;
                while let Some(_v) = rx.recv().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
                send_handle.await.expect("sender task should join");
            });
        });
    });

    // Benchmark async unsync mpsc channel
    let _ = group.bench_function(BenchmarkId::new("async_unsync_mpsc", MSG_COUNT), |b| {
        b.iter(|| {
            rt.block_on(async {
                let (tx, mut rx) = async_unsync::bounded::channel(CHANNEL_SIZE).into_split();
                let pdata = Rc::new("test".to_string());

                let send_handle = tokio::task::spawn_local(async move {
                    for _ in 0..MSG_COUNT {
                        _ = tx.send(pdata.clone());
                    }
                });

                let mut _sum = 0;
                while let Some(_v) = rx.recv().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
                send_handle.await.expect("sender task should join");
            });
        });
    });

    // Benchmark unsync mpsc channel
    let _ = group.bench_function(BenchmarkId::new("unsync_mpsc", MSG_COUNT), |b| {
        b.iter(|| {
            rt.block_on(async {
                let (mut tx, mut rx) = unsync::spsc::channel(CHANNEL_SIZE);
                let pdata = Rc::new("test".to_string());

                let send_handle = tokio::task::spawn_local(async move {
                    for _ in 0..MSG_COUNT {
                        _ = tx.send(pdata.clone());
                    }
                });

                let mut _sum = 0;
                while let Some(_v) = rx.recv().await {
                    _sum += 1;
                }
                assert_eq!(_sum, MSG_COUNT);
                send_handle.await.expect("sender task should join");
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_compare);
criterion_main!(benches);
