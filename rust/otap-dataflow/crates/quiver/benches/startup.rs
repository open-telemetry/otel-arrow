// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for startup retention accounting.
//!
//! Fixture creation is excluded from the timed region. The benchmarks measure:
//! - validation, accounting, and deletion of expired finalized segments
//! - decoding, item counting, and skipping of expired WAL entries

#![allow(missing_docs)]
#![allow(unused_results)]

use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use arrow_array::RecordBatch;
use arrow_array::builder::{Int64Builder, StringBuilder};
use arrow_schema::{DataType, Field, Schema};
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use quiver::budget::DiskBudget;
use quiver::config::{DurabilityMode, QuiverConfig, RetentionConfig, SegmentConfig, WalConfig};
use quiver::engine::{QuiverEngine, WalItemCounter};
use quiver::record_bundle::{BundleDescriptor, PayloadRef, RecordBundle, SlotDescriptor, SlotId};
use quiver::segment::{OpenSegment, SegmentSeq, SegmentWriter};
use tempfile::TempDir;
use tokio::runtime::Runtime;

const EXPIRED_AGE: Duration = Duration::from_secs(3_600);
const MAX_AGE: Duration = Duration::from_secs(60);

struct BenchBundle {
    descriptor: BundleDescriptor,
    batch: RecordBatch,
    ingestion_time: SystemTime,
}

impl BenchBundle {
    fn with_rows(num_rows: usize, ingestion_time: SystemTime) -> Self {
        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::Int64, false),
            Field::new("value", DataType::Int64, false),
            Field::new("message", DataType::Utf8, false),
        ]));

        let mut timestamp = Int64Builder::with_capacity(num_rows);
        let mut value = Int64Builder::with_capacity(num_rows);
        let mut message = StringBuilder::with_capacity(num_rows, num_rows * 64);
        for i in 0..num_rows {
            timestamp.append_value(1_700_000_000_000_i64 + i as i64);
            value.append_value(i as i64);
            message.append_value(format!(
                "startup retention benchmark telemetry payload row {i:08}"
            ));
        }

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(timestamp.finish()),
                Arc::new(value.finish()),
                Arc::new(message.finish()),
            ],
        )
        .expect("valid benchmark batch");

        Self {
            descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(SlotId::new(0), "Logs")]),
            batch,
            ingestion_time,
        }
    }
}

impl RecordBundle for BenchBundle {
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn ingestion_time(&self) -> SystemTime {
        self.ingestion_time
    }

    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
        (slot == SlotId::new(0)).then_some(PayloadRef {
            schema_fingerprint: [0; 32],
            batch: &self.batch,
        })
    }

    fn item_count(&self) -> u64 {
        self.batch.num_rows() as u64
    }
}

fn bench_tempdir() -> TempDir {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let base_dir = home.join(".quiver-benchmarks");
    std::fs::create_dir_all(&base_dir).expect("create benchmark base dir");
    tempfile::Builder::new()
        .prefix("startup-")
        .tempdir_in(&base_dir)
        .expect("create benchmark temp dir")
}

fn unlimited_budget() -> Arc<DiskBudget> {
    Arc::new(DiskBudget::unlimited())
}

fn sync_fixture_dir(path: &Path) {
    for entry in std::fs::read_dir(path).expect("read fixture directory") {
        let entry = entry.expect("fixture directory entry");
        if entry.file_type().expect("fixture file type").is_file() {
            std::fs::File::open(entry.path())
                .expect("open fixture file")
                .sync_all()
                .expect("sync fixture file");
        }
    }

    #[cfg(unix)]
    std::fs::File::open(path)
        .expect("open fixture directory")
        .sync_all()
        .expect("sync fixture directory");
}

fn segment_config(data_dir: &Path, max_age: Option<Duration>) -> QuiverConfig {
    QuiverConfig::builder()
        .data_dir(data_dir)
        .durability(DurabilityMode::SegmentOnly)
        .segment(SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024 * 1024 * 1024).expect("non-zero"),
            max_open_duration: Duration::from_secs(3_600),
            ..Default::default()
        })
        .retention(RetentionConfig { max_age })
        .build()
        .expect("valid segment benchmark config")
}

fn wal_config(data_dir: &Path, max_age: Option<Duration>) -> QuiverConfig {
    QuiverConfig::builder()
        .data_dir(data_dir)
        .wal(WalConfig {
            max_size_bytes: NonZeroU64::new(1024 * 1024 * 1024).expect("non-zero"),
            rotation_target_bytes: NonZeroU64::new(256 * 1024 * 1024).expect("non-zero"),
            flush_interval: Duration::from_secs(3_600),
            ..Default::default()
        })
        .segment(SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024 * 1024 * 1024).expect("non-zero"),
            max_open_duration: Duration::from_secs(3_600),
            ..Default::default()
        })
        .retention(RetentionConfig { max_age })
        .build()
        .expect("valid WAL benchmark config")
}

fn prepare_segments(
    rt: &Runtime,
    segment_count: usize,
    bundles_per_segment: usize,
    rows_per_bundle: usize,
    expired: bool,
) -> (TempDir, QuiverConfig, u64, u64) {
    let temp_dir = bench_tempdir();
    let segment_dir = temp_dir.path().join("segments");
    std::fs::create_dir_all(&segment_dir).expect("create segment directory");

    let bundle = BenchBundle::with_rows(rows_per_bundle, SystemTime::now() - EXPIRED_AGE);
    let mut total_bytes = 0;
    for raw_seq in 0..segment_count as u64 {
        let seq = SegmentSeq::new(raw_seq);
        let mut segment = OpenSegment::new();
        for _ in 0..bundles_per_segment {
            segment.append(&bundle).expect("append benchmark bundle");
        }
        let path = segment_dir.join(format!("{}.qseg", seq.to_filename_component()));
        let writer = SegmentWriter::new(seq, false);
        rt.block_on(writer.write_segment(&path, segment))
            .expect("write benchmark segment");
        total_bytes += std::fs::metadata(path)
            .expect("benchmark segment metadata")
            .len();
    }
    sync_fixture_dir(&segment_dir);

    let max_age = if expired {
        // Segment expiry uses file modification time, so make the positive
        // max_age boundary unambiguous outside the measured region.
        std::thread::sleep(Duration::from_millis(2));
        Duration::from_nanos(1)
    } else {
        MAX_AGE
    };
    let config = segment_config(temp_dir.path(), Some(max_age));
    let total_items = (segment_count * bundles_per_segment * rows_per_bundle) as u64;
    (temp_dir, config, total_bytes, total_items)
}

fn prepare_wal(
    rt: &Runtime,
    entry_count: usize,
    rows_per_entry: usize,
    expired: bool,
) -> (TempDir, QuiverConfig, u64, u64) {
    let temp_dir = bench_tempdir();
    let write_config = wal_config(temp_dir.path(), None);
    let ingestion_time = if expired {
        SystemTime::now() - EXPIRED_AGE
    } else {
        SystemTime::now()
    };
    let bundle = BenchBundle::with_rows(rows_per_entry, ingestion_time);

    rt.block_on(async {
        let engine = QuiverEngine::open(write_config, unlimited_budget())
            .await
            .expect("open WAL fixture engine");
        for _ in 0..entry_count {
            engine.ingest(&bundle).await.expect("write WAL fixture");
        }
        drop(engine);
    });

    let wal_dir = temp_dir.path().join("wal");
    sync_fixture_dir(&wal_dir);
    let wal_bytes = std::fs::read_dir(&wal_dir)
        .expect("read WAL directory")
        .map(|entry| {
            entry
                .expect("WAL directory entry")
                .metadata()
                .expect("WAL metadata")
                .len()
        })
        .sum();
    let config = wal_config(temp_dir.path(), Some(MAX_AGE));
    let total_items = (entry_count * rows_per_entry) as u64;
    (temp_dir, config, wal_bytes, total_items)
}

fn startup_empty(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");
    c.bench_function("startup_empty", |b| {
        b.iter_batched(
            || {
                let temp_dir = bench_tempdir();
                let config = segment_config(temp_dir.path(), Some(MAX_AGE));
                (temp_dir, config)
            },
            |(temp_dir, config)| {
                let engine = rt
                    .block_on(QuiverEngine::open(config, unlimited_budget()))
                    .expect("open empty engine");
                drop((engine, temp_dir));
            },
            BatchSize::PerIteration,
        );
    });
}

fn startup_segments(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_segments");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    let rt = Runtime::new().expect("tokio runtime");

    for expired in [false, true] {
        for (segment_count, bundles_per_segment, rows_per_bundle) in
            [(1, 16, 1_000), (4, 16, 1_000), (1, 16, 8_192)]
        {
            let probe = prepare_segments(
                &rt,
                segment_count,
                bundles_per_segment,
                rows_per_bundle,
                expired,
            );
            let total_bytes = probe.2;
            drop(probe);

            group.throughput(Throughput::Bytes(total_bytes));
            let state = if expired { "expired" } else { "fresh" };
            let id = format!(
                "{state}/{segment_count}_segments/{bundles_per_segment}_bundles/{rows_per_bundle}_rows"
            );
            group.bench_function(BenchmarkId::new("open", id), |b| {
                b.iter_batched(
                    || {
                        prepare_segments(
                            &rt,
                            segment_count,
                            bundles_per_segment,
                            rows_per_bundle,
                            expired,
                        )
                    },
                    |(temp_dir, config, _, expected_items)| {
                        let engine = rt
                            .block_on(QuiverEngine::open(config, unlimited_budget()))
                            .expect("open engine with benchmark segments");
                        let loss = engine.retention_loss_snapshot().expired;
                        if expired {
                            assert_eq!(loss.segments, segment_count as u64);
                            assert_eq!(loss.items, expected_items);
                        } else {
                            assert_eq!(loss.segments, 0);
                            assert_eq!(loss.items, 0);
                        }
                        drop((engine, temp_dir));
                    },
                    BatchSize::PerIteration,
                );
            });
        }
    }

    group.finish();
}

fn startup_segments_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_segments_large");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));
    let rt = Runtime::new().expect("tokio runtime");

    for expired in [false, true] {
        for (segment_count, bundles_per_segment, rows_per_bundle) in
            [(32, 16, 1_000), (8, 16, 8_192)]
        {
            let probe = prepare_segments(
                &rt,
                segment_count,
                bundles_per_segment,
                rows_per_bundle,
                expired,
            );
            let total_bytes = probe.2;
            drop(probe);

            group.throughput(Throughput::Bytes(total_bytes));
            let state = if expired { "expired" } else { "fresh" };
            let id = format!(
                "{state}/{segment_count}_segments/{bundles_per_segment}_bundles/{rows_per_bundle}_rows"
            );
            group.bench_function(BenchmarkId::new("open", id), |b| {
                b.iter_batched(
                    || {
                        prepare_segments(
                            &rt,
                            segment_count,
                            bundles_per_segment,
                            rows_per_bundle,
                            expired,
                        )
                    },
                    |(temp_dir, config, _, expected_items)| {
                        let engine = rt
                            .block_on(QuiverEngine::open(config, unlimited_budget()))
                            .expect("open engine with large benchmark segments");
                        let loss = engine.retention_loss_snapshot().expired;
                        if expired {
                            assert_eq!(loss.segments, segment_count as u64);
                            assert_eq!(loss.items, expected_items);
                        } else {
                            assert_eq!(loss.segments, 0);
                            assert_eq!(loss.items, 0);
                        }
                        drop((engine, temp_dir));
                    },
                    BatchSize::PerIteration,
                );
            });
        }
    }

    group.finish();
}

fn startup_wal(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_wal");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    let rt = Runtime::new().expect("tokio runtime");
    let counter: WalItemCounter = Arc::new(|bundle| {
        bundle
            .payload(SlotId::new(0))
            .map(|payload| payload.batch.num_rows() as u64)
    });

    for expired in [false, true] {
        for (entry_count, rows_per_entry) in [(100, 100), (100, 1_000), (1_000, 100)] {
            let probe = prepare_wal(&rt, entry_count, rows_per_entry, expired);
            let total_bytes = probe.2;
            drop(probe);

            group.throughput(Throughput::Bytes(total_bytes));
            let state = if expired { "expired" } else { "fresh" };
            let id = format!("{state}/{entry_count}_entries/{rows_per_entry}_rows");
            group.bench_function(BenchmarkId::new("open", id), |b| {
                b.iter_batched(
                    || prepare_wal(&rt, entry_count, rows_per_entry, expired),
                    |(temp_dir, config, _, expected_items)| {
                        let engine = rt
                            .block_on(
                                QuiverEngine::builder(config)
                                    .with_budget(unlimited_budget())
                                    .with_wal_item_counter(Arc::clone(&counter))
                                    .build(),
                            )
                            .expect("open engine with benchmark WAL");
                        let loss = engine.retention_loss_snapshot().expired;
                        if expired {
                            assert_eq!(loss.bundles, entry_count as u64);
                            assert_eq!(loss.items, expected_items);
                        } else {
                            assert_eq!(loss.bundles, 0);
                            assert_eq!(loss.items, 0);
                        }
                        drop((engine, temp_dir));
                    },
                    BatchSize::PerIteration,
                );
            });
        }
    }

    group.finish();
}

fn startup_wal_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_wal_large");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));
    let rt = Runtime::new().expect("tokio runtime");
    let counter: WalItemCounter = Arc::new(|bundle| {
        bundle
            .payload(SlotId::new(0))
            .map(|payload| payload.batch.num_rows() as u64)
    });

    for expired in [false, true] {
        for (entry_count, rows_per_entry) in [(1_000, 1_000), (10_000, 100)] {
            let probe = prepare_wal(&rt, entry_count, rows_per_entry, expired);
            let total_bytes = probe.2;
            drop(probe);

            group.throughput(Throughput::Bytes(total_bytes));
            let state = if expired { "expired" } else { "fresh" };
            let id = format!("{state}/{entry_count}_entries/{rows_per_entry}_rows");
            group.bench_function(BenchmarkId::new("open", id), |b| {
                b.iter_batched(
                    || prepare_wal(&rt, entry_count, rows_per_entry, expired),
                    |(temp_dir, config, _, expected_items)| {
                        let engine = rt
                            .block_on(
                                QuiverEngine::builder(config)
                                    .with_budget(unlimited_budget())
                                    .with_wal_item_counter(Arc::clone(&counter))
                                    .build(),
                            )
                            .expect("open engine with large benchmark WAL");
                        let loss = engine.retention_loss_snapshot().expired;
                        if expired {
                            assert_eq!(loss.bundles, entry_count as u64);
                            assert_eq!(loss.items, expected_items);
                        } else {
                            assert_eq!(loss.bundles, 0);
                            assert_eq!(loss.items, 0);
                        }
                        drop((engine, temp_dir));
                    },
                    BatchSize::PerIteration,
                );
            });
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    startup_empty,
    startup_segments,
    startup_segments_large,
    startup_wal,
    startup_wal_large,
);
criterion_main!(benches);
