// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! Benchmarks for OTAP batch-processor flush work.

use arrow::array::RecordBatch;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_config::SignalType;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::batching::make_item_batches;
use otap_df_pdata::otap::transform::concatenate::concatenate;
use otap_df_pdata::otap::transform::reindex::reindex;
use otap_df_pdata::otap::{Logs, OtapBatchStore};
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::fixtures::{DataGenerator, LogsConfig};
use otap_df_pdata::testing::round_trip::otlp_to_otap;
use std::hint::black_box;
use std::num::NonZeroU64;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn logs_batch(item_count: usize) -> OtapArrowRecords {
    let logs = DataGenerator::with_logs_config(
        LogsConfig::new(item_count)
            .with_resources(1)
            .with_scopes_per_resource(1)
            .with_resource_attrs(4)
            .with_scope_attrs(2)
            .with_log_attrs(4),
    )
    .generate_logs_from_config();
    otlp_to_otap(&OtlpProtoMessage::Logs(logs))
}

fn logs_store(item_count: usize) -> [Option<RecordBatch>; Logs::COUNT] {
    let OtapArrowRecords::Logs(logs) = logs_batch(item_count) else {
        unreachable!("logs generator must produce logs")
    };
    logs.into_batches()
}

fn bench_otap_logs_flush(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processor/otap_logs_flush");
    _ = group.sample_size(30);

    for (input_batches, items_per_batch, max_items) in [
        (1, 8192, None),
        (2, 4096, None),
        (8, 1024, None),
        (32, 256, None),
        (8, 1024, Some(2048)),
    ] {
        let input = vec![logs_batch(items_per_batch); input_batches];
        let total_items = input_batches * items_per_batch;
        let name = match max_items {
            None => format!("{input_batches}x{items_per_batch}/unbounded"),
            Some(max_items) => format!("{input_batches}x{items_per_batch}/max_{max_items}"),
        };

        _ = group.throughput(Throughput::Elements(total_items as u64));
        _ = group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, input| {
            b.iter_batched(
                || input.clone(),
                |pending| {
                    black_box(
                        make_item_batches(
                            SignalType::Logs,
                            max_items.and_then(NonZeroU64::new),
                            pending,
                        )
                        .expect("OTAP logs batching must succeed"),
                    )
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_otap_logs_stages(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processor/otap_logs_stages/8x1024");
    _ = group.sample_size(30);
    _ = group.throughput(Throughput::Elements(8192));

    let input = vec![logs_store(1024); 8];
    _ = group.bench_function("reindex", |b| {
        b.iter_batched(
            || input.clone(),
            |mut pending| {
                reindex(&mut pending).expect("reindexing must succeed");
                black_box(pending)
            },
            BatchSize::SmallInput,
        );
    });

    let mut reindexed = input;
    reindex(&mut reindexed).expect("reindexing must succeed");
    _ = group.bench_function("concatenate", |b| {
        b.iter_batched(
            || reindexed.clone(),
            |mut pending| black_box(concatenate(&mut pending).expect("concatenation must succeed")),
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_otap_logs_flush, bench_otap_logs_stages);
criterion_main!(benches);
