// SPDX-License-Identifier: Apache-2.0

//! Benchmark tests for channel implementations

#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkGroup, BenchmarkId, Criterion};
use criterion::measurement::WallTime;
use futures::{SinkExt, StreamExt};
use futures_channel::mpsc as futures_mpsc;
//use kanal::mpmc;
use tokio::sync::mpsc as tokio_mpsc;
use tokio::runtime::{Builder, Runtime};

const MSG_COUNT: usize = 100_000;

fn make_runtime() -> Runtime {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime")
}

fn bench_tokio(c: &mut Criterion, rt: &Runtime) {
    let mut group = c.benchmark_group("tokio_mpsc");
    let _ = group.bench_function(BenchmarkId::new("tokio", MSG_COUNT), |b| {
        b.to_async(rt).iter(|| async {
            let (tx, mut rx) = tokio_mpsc::channel::<usize>(1024);
            let _ = tokio::spawn(async move {
                for i in 0..MSG_COUNT { tx.send(i).await.unwrap(); }
            });
            let mut _sum = 0;
            while let Some(v) = rx.recv().await {
                _sum += v;
            }
        });
    });
    group.finish();
}

fn bench_futures(c: &mut Criterion, rt: &Runtime) {
    let mut group = c.benchmark_group("futures_mpsc");
    let _ = group.bench_function(BenchmarkId::new("futures", MSG_COUNT), |b| {
        b.to_async(rt).iter(|| async {
            let (mut tx, mut rx) = futures_mpsc::channel::<usize>(1024);
            let _ = tokio::spawn(async move {
                for i in 0..MSG_COUNT { tx.send(i).await.unwrap(); }
                drop(tx);
            });
            let mut _sum = 0;
            while let Some(v) = rx.next().await {
                _sum += v;
            }
        });
    });
    group.finish();
}


// async fn bench_kanal(c: &mut Criterion) {
//     let mut group = c.benchmark_group("kanal_mpmc");
//     group.bench_function(BenchmarkId::new("kanal", MSG_COUNT), |b| {
//         b.to_async(current_thread_runtime())
//             .iter(|| async {
//                 let (tx, rx) = mpmc::unbounded::<usize>();
//                 tokio::spawn(async move {
//                     for i in 0..MSG_COUNT { tx.send(i).unwrap(); }
//                 });
//                 let mut sum = 0;
//                 for v in rx {
//                     sum += v;
//                 }
//             });
//     });
//     group.finish();
// }

fn criterion_benchmark(c: &mut Criterion) {
    let rt = make_runtime();

    bench_tokio(c, &rt);
    bench_futures(c, &rt);
    //bench_kanal(c, &rt);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);