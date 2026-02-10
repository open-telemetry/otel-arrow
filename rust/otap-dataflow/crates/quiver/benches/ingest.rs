// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for end-to-end engine ingestion.
//!
//! These benchmarks measure the complete ingest path including:
//! - WAL append
//! - Open segment accumulation
//! - Segment finalization (when thresholds are exceeded)
//!
//! Benchmark directories are created in `~/.quiver-benchmarks/` to avoid
//! `/tmp` which may be tmpfs (RAM-backed) and would not reflect real disk I/O.
//!
//! Uses tokio runtime to call async APIs via `block_on`.

#![allow(missing_docs)]
#![allow(unused_results)]

use std::num::NonZeroU64;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use arrow_array::RecordBatch;
use arrow_array::builder::{Int64Builder, StringBuilder};
use arrow_schema::{DataType, Field, Schema};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use quiver::budget::DiskBudget;
use quiver::config::{QuiverConfig, RetentionPolicy, SegmentConfig, WalConfig};
use quiver::engine::QuiverEngine;
use quiver::record_bundle::{BundleDescriptor, PayloadRef, RecordBundle, SlotDescriptor, SlotId};
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// Creates a large test budget (1 GB) for benchmarks.
fn bench_budget() -> Arc<DiskBudget> {
    Arc::new(DiskBudget::new(
        1024 * 1024 * 1024,
        64 * 1024 * 1024, // 64 MB segment headroom
        RetentionPolicy::Backpressure,
    ))
}

/// Creates a temp directory in ~/.quiver-benchmarks/ to avoid tmpfs.
fn bench_tempdir() -> TempDir {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let base_dir = home.join(".quiver-benchmarks");
    std::fs::create_dir_all(&base_dir).expect("create benchmark base dir");
    tempfile::Builder::new()
        .prefix("bench-")
        .tempdir_in(&base_dir)
        .expect("create benchmark temp dir")
}

/// Realistic test bundle with configurable size.
struct BenchBundle {
    descriptor: BundleDescriptor,
    batch: RecordBatch,
    fingerprint: [u8; 32],
}

impl BenchBundle {
    fn with_rows(num_rows: usize) -> Self {
        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::Int64, false),
            Field::new("value", DataType::Int64, false),
            Field::new("message", DataType::Utf8, true),
        ]));

        let mut ts_builder = Int64Builder::with_capacity(num_rows);
        let mut val_builder = Int64Builder::with_capacity(num_rows);
        let mut msg_builder = StringBuilder::with_capacity(num_rows, num_rows * 50);

        for i in 0..num_rows {
            ts_builder.append_value(1700000000000 + i as i64);
            val_builder.append_value(i as i64 * 100);
            msg_builder.append_value(format!("Log message number {} with some content", i));
        }

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(ts_builder.finish()),
                Arc::new(val_builder.finish()),
                Arc::new(msg_builder.finish()),
            ],
        )
        .expect("valid batch");

        Self {
            descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(SlotId::new(0), "Logs")]),
            batch,
            fingerprint: [0u8; 32],
        }
    }
}

impl RecordBundle for BenchBundle {
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn ingestion_time(&self) -> SystemTime {
        SystemTime::now()
    }

    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
        if slot == SlotId::new(0) {
            Some(PayloadRef {
                schema_fingerprint: self.fingerprint,
                batch: &self.batch,
            })
        } else {
            None
        }
    }
}

/// Create a test config optimized for benchmarking.
fn bench_config(temp_dir: &std::path::Path, segment_size_mb: u64) -> QuiverConfig {
    QuiverConfig::builder()
        .data_dir(temp_dir)
        .wal(WalConfig {
            flush_interval: Duration::from_secs(60), // Don't flush during benchmark
            ..Default::default()
        })
        .segment(SegmentConfig {
            target_size_bytes: NonZeroU64::new(segment_size_mb * 1024 * 1024)
                .expect("segment size must be non-zero"),
            max_open_duration: Duration::from_secs(3600), // Don't time-finalize
            ..Default::default()
        })
        .build()
        .expect("valid config")
}

/// Benchmark single ingest operations with varying bundle sizes.
///
/// Row counts are representative of OTel Collector batch sizes:
/// - 100: Small/frequent batches (real-time streaming)
/// - 1,000: Typical production batch size
/// - 8,192: OTel Collector default `send_batch_size`
fn ingest_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingest_single");
    let rt = Runtime::new().expect("tokio runtime");

    for num_rows in [100, 1_000, 8_192] {
        let bundle = BenchBundle::with_rows(num_rows);

        // Estimate bytes: 2 int64 columns + ~50 bytes per string
        let estimated_bytes = num_rows * (8 + 8 + 50);

        group.throughput(Throughput::Bytes(estimated_bytes as u64));
        group.bench_with_input(BenchmarkId::new("rows", num_rows), &bundle, |b, bundle| {
            b.iter_batched(
                || {
                    let temp_dir = bench_tempdir();
                    let config = bench_config(temp_dir.path(), 100); // Large segment to avoid finalization
                    let engine = rt.block_on(async {
                        QuiverEngine::open(config, bench_budget())
                            .await
                            .expect("engine")
                    });
                    (engine, temp_dir)
                },
                |(engine, _temp_dir): (Arc<QuiverEngine>, TempDir)| {
                    rt.block_on(async {
                        engine.ingest(bundle).await.expect("ingest succeeds");
                    });
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark sustained ingestion (many bundles, includes segment finalization).
///
/// Uses 1,000 rows per bundle to reflect typical production batch sizes.
fn ingest_sustained(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingest_sustained");
    group.sample_size(20); // Fewer samples for longer-running benchmark
    let rt = Runtime::new().expect("tokio runtime");

    // Ingest 100 bundles of 1,000 rows each = 100,000 total rows
    let num_bundles = 100;
    let rows_per_bundle = 1_000;

    let bundle = BenchBundle::with_rows(rows_per_bundle);
    let total_rows = num_bundles * rows_per_bundle;
    let estimated_bytes = total_rows * (8 + 8 + 50);

    group.throughput(Throughput::Bytes(estimated_bytes as u64));
    group.bench_function("100_bundles_1000_rows_with_finalization", |b| {
        b.iter_batched(
            || {
                // Create fresh temp dir per iteration to avoid read-only segment conflicts
                let temp_dir = bench_tempdir();
                let config = bench_config(temp_dir.path(), 1); // 1 MB segments
                let engine = rt.block_on(async {
                    QuiverEngine::open(config, bench_budget())
                        .await
                        .expect("engine")
                });
                (engine, temp_dir) // Keep temp_dir alive
            },
            |(engine, _temp_dir): (Arc<QuiverEngine>, TempDir)| {
                rt.block_on(async {
                    for _ in 0..num_bundles {
                        engine.ingest(&bundle).await.expect("ingest succeeds");
                    }
                    engine.shutdown().await.expect("shutdown succeeds");
                });
            },
            criterion::BatchSize::PerIteration,
        );
    });

    group.finish();
}

/// Benchmark with frequent segment writes to measure finalization overhead.
///
/// Uses 1,000 rows per bundle to reflect typical production batch sizes.
fn ingest_with_frequent_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingest_finalization");
    group.sample_size(30);
    let rt = Runtime::new().expect("tokio runtime");

    let bundle = BenchBundle::with_rows(1_000);

    group.throughput(Throughput::Elements(20));
    group.bench_function("20_bundles_with_frequent_finalization", |b| {
        b.iter_batched(
            || {
                // Create fresh temp dir per iteration to avoid read-only segment conflicts
                let temp_dir = bench_tempdir();
                // Tiny segment to force frequent finalization
                let config = QuiverConfig::builder()
                    .data_dir(temp_dir.path())
                    .wal(WalConfig {
                        flush_interval: Duration::from_secs(60),
                        ..Default::default()
                    })
                    .segment(SegmentConfig {
                        // ~100 bytes per row * 1,000 rows = 100KB per bundle
                        // 500KB target = finalize every ~5 bundles
                        target_size_bytes: NonZeroU64::new(500 * 1024)
                            .expect("segment size must be non-zero"),
                        max_open_duration: Duration::from_secs(3600),
                        ..Default::default()
                    })
                    .build()
                    .expect("valid config");
                let engine = rt.block_on(async {
                    QuiverEngine::open(config, bench_budget())
                        .await
                        .expect("engine")
                });
                (engine, temp_dir) // Keep temp_dir alive
            },
            |(engine, _temp_dir): (Arc<QuiverEngine>, TempDir)| {
                rt.block_on(async {
                    for _ in 0..20 {
                        engine.ingest(&bundle).await.expect("ingest succeeds");
                    }
                    engine.shutdown().await.expect("shutdown succeeds");
                });
            },
            criterion::BatchSize::PerIteration,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    ingest_single,
    ingest_sustained,
    ingest_with_frequent_writes
);
criterion_main!(benches);
