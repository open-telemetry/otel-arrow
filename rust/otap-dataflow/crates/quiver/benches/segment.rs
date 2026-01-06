// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Criterion micro-benchmarks for segment writing and reading.
//!
//! These benchmarks measure:
//! - OpenSegment accumulation performance
//! - Segment finalization latency
//! - Multi-stream segment writing
//! - SegmentReader open vs open_mmap performance
//! - Bundle reconstruction throughput

#![allow(missing_docs)]
#![allow(unused_results)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use arrow_array::RecordBatch;
use arrow_array::builder::{Int64Builder, StringBuilder};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use quiver::record_bundle::{
    BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
};
use quiver::segment::{OpenSegment, SegmentReader, SegmentSeq, SegmentWriter};

/// Test bundle with configurable row count and schema fingerprint.
struct BenchBundle {
    descriptor: BundleDescriptor,
    payloads: HashMap<SlotId, (SchemaFingerprint, RecordBatch)>,
}

impl BenchBundle {
    fn single_slot(num_rows: usize, fingerprint: [u8; 32]) -> Self {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int64,
            false,
        )]));

        let mut builder = Int64Builder::with_capacity(num_rows);
        for i in 0..num_rows {
            builder.append_value(i as i64);
        }
        let array = builder.finish();

        let batch = RecordBatch::try_new(schema, vec![Arc::new(array)]).expect("valid batch");

        let mut payloads = HashMap::new();
        payloads.insert(SlotId::new(0), (fingerprint, batch));

        Self {
            descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(SlotId::new(0), "Logs")]),
            payloads,
        }
    }

    /// Creates a 4-slot bundle representative of OTAP logs payloads:
    /// - ResourceAttrs: Resource-level attributes (typically small, shared across many logs)
    /// - ScopeAttrs: Instrumentation scope attributes
    /// - Logs: Main log records
    /// - LogAttrs: Per-log attributes
    fn multi_slot(num_rows: usize) -> Self {
        // ResourceAttrs - typically few rows, string attributes
        let resource_schema: SchemaRef = Arc::new(Schema::new(vec![
            Field::new("resource_id", DataType::Int64, false),
            Field::new("service_name", DataType::Utf8, true),
        ]));

        // ScopeAttrs - instrumentation scope info
        let scope_schema: SchemaRef = Arc::new(Schema::new(vec![
            Field::new("scope_id", DataType::Int64, false),
            Field::new("scope_name", DataType::Utf8, true),
        ]));

        // Logs - main log records
        let logs_schema: SchemaRef = Arc::new(Schema::new(vec![
            Field::new("log_id", DataType::Int64, false),
            Field::new("timestamp", DataType::Int64, false),
            Field::new("body", DataType::Utf8, true),
        ]));

        // LogAttrs - per-log attributes
        let log_attrs_schema: SchemaRef = Arc::new(Schema::new(vec![
            Field::new("log_id", DataType::Int64, false),
            Field::new("key", DataType::Utf8, false),
            Field::new("value", DataType::Utf8, true),
        ]));

        // ResourceAttrs typically has fewer rows (resource-level, shared)
        let resource_rows = (num_rows / 10).max(1);
        let mut resource_id_builder = Int64Builder::with_capacity(resource_rows);
        let mut service_name_builder =
            StringBuilder::with_capacity(resource_rows, resource_rows * 20);
        for i in 0..resource_rows {
            resource_id_builder.append_value(i as i64);
            service_name_builder.append_value(format!("service_{}", i));
        }
        let resource_batch = RecordBatch::try_new(
            resource_schema,
            vec![
                Arc::new(resource_id_builder.finish()),
                Arc::new(service_name_builder.finish()),
            ],
        )
        .expect("valid resource batch");

        // ScopeAttrs - also typically fewer rows
        let scope_rows = (num_rows / 5).max(1);
        let mut scope_id_builder = Int64Builder::with_capacity(scope_rows);
        let mut scope_name_builder = StringBuilder::with_capacity(scope_rows, scope_rows * 30);
        for i in 0..scope_rows {
            scope_id_builder.append_value(i as i64);
            scope_name_builder.append_value(format!("io.opentelemetry.scope_{}", i));
        }
        let scope_batch = RecordBatch::try_new(
            scope_schema,
            vec![
                Arc::new(scope_id_builder.finish()),
                Arc::new(scope_name_builder.finish()),
            ],
        )
        .expect("valid scope batch");

        // Logs - main records, full row count
        let mut log_id_builder = Int64Builder::with_capacity(num_rows);
        let mut timestamp_builder = Int64Builder::with_capacity(num_rows);
        let mut body_builder = StringBuilder::with_capacity(num_rows, num_rows * 50);
        for i in 0..num_rows {
            log_id_builder.append_value(i as i64);
            timestamp_builder.append_value(1700000000000000000i64 + i as i64);
            body_builder.append_value(format!("Log message content for entry {}", i));
        }
        let logs_batch = RecordBatch::try_new(
            logs_schema,
            vec![
                Arc::new(log_id_builder.finish()),
                Arc::new(timestamp_builder.finish()),
                Arc::new(body_builder.finish()),
            ],
        )
        .expect("valid logs batch");

        // LogAttrs - typically more rows than logs (multiple attrs per log)
        let attr_rows = num_rows * 3; // ~3 attributes per log on average
        let mut attr_log_id_builder = Int64Builder::with_capacity(attr_rows);
        let mut attr_key_builder = StringBuilder::with_capacity(attr_rows, attr_rows * 15);
        let mut attr_value_builder = StringBuilder::with_capacity(attr_rows, attr_rows * 20);
        for i in 0..attr_rows {
            attr_log_id_builder.append_value((i / 3) as i64);
            attr_key_builder.append_value(format!("attr_key_{}", i % 3));
            attr_value_builder.append_value(format!("attr_value_{}", i));
        }
        let log_attrs_batch = RecordBatch::try_new(
            log_attrs_schema,
            vec![
                Arc::new(attr_log_id_builder.finish()),
                Arc::new(attr_key_builder.finish()),
                Arc::new(attr_value_builder.finish()),
            ],
        )
        .expect("valid log attrs batch");

        let mut payloads = HashMap::new();
        // Use slot IDs matching OTAP ArrowPayloadType enum values
        payloads.insert(SlotId::new(1), ([1u8; 32], resource_batch)); // RESOURCE_ATTRS = 1
        payloads.insert(SlotId::new(2), ([2u8; 32], scope_batch)); // SCOPE_ATTRS = 2
        payloads.insert(SlotId::new(30), ([30u8; 32], logs_batch)); // LOGS = 30
        payloads.insert(SlotId::new(31), ([31u8; 32], log_attrs_batch)); // LOG_ATTRS = 31

        Self {
            descriptor: BundleDescriptor::new(vec![
                SlotDescriptor::new(SlotId::new(1), "ResourceAttrs"),
                SlotDescriptor::new(SlotId::new(2), "ScopeAttrs"),
                SlotDescriptor::new(SlotId::new(30), "Logs"),
                SlotDescriptor::new(SlotId::new(31), "LogAttrs"),
            ]),
            payloads,
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
        self.payloads.get(&slot).map(|(fp, batch)| PayloadRef {
            schema_fingerprint: *fp,
            batch,
        })
    }
}

/// Benchmark OpenSegment append with varying bundle sizes.
///
/// Row counts are representative of OTel Collector batch sizes:
/// - 100: Small/frequent batches (real-time streaming)
/// - 1,000: Typical production batch size
/// - 8,192: OTel Collector default `send_batch_size`
fn segment_append_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_append");

    for num_rows in [100, 1_000, 8_192] {
        let bundle = BenchBundle::single_slot(num_rows, [0u8; 32]);

        // Estimate bytes for throughput
        let estimated_bytes = num_rows * 8;

        group.throughput(Throughput::Bytes(estimated_bytes as u64));
        group.bench_with_input(BenchmarkId::new("rows", num_rows), &bundle, |b, bundle| {
            b.iter_batched(
                OpenSegment::new,
                |mut segment| {
                    segment.append(bundle).expect("append succeeds");
                    segment
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark accumulating many bundles into a segment.
///
/// Uses 1,000 rows per bundle (typical production batch size) to simulate
/// realistic segment accumulation patterns.
fn segment_accumulate_many(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_accumulate");

    let bundle = BenchBundle::single_slot(1_000, [0u8; 32]);

    // Accumulate 100 bundles of 1,000 rows each = 100,000 total rows
    group.throughput(Throughput::Elements(100));
    group.bench_function("100_bundles_1000_rows", |b| {
        b.iter_batched(
            OpenSegment::new,
            |mut segment| {
                for _ in 0..100 {
                    segment.append(&bundle).expect("append succeeds");
                }
                segment
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark end-to-end segment write performance.
///
/// All write benchmarks use 1,000 rows per bundle (typical production batch
/// size) and measure the cost of building the batch manifest, serializing
/// stream data, and writing to disk.
///
/// Sub-benchmarks:
/// - `single_slot`: Varying bundle count with single-slot bundles
/// - `multi_stream`: 10 different schemas (schema evolution scenario)
/// - `multi_slot`: OTAP-style 4-slot bundles (realistic structure)
fn segment_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_write");

    // ── Single-slot write (varying bundle counts) ──
    for num_bundles in [10, 50, 100] {
        let bundle = BenchBundle::single_slot(1_000, [0u8; 32]);

        group.throughput(Throughput::Elements(num_bundles as u64));
        group.bench_with_input(
            BenchmarkId::new("single_slot", num_bundles),
            &num_bundles,
            |b, &num_bundles| {
                b.iter_batched(
                    || {
                        let temp_dir = tempfile::tempdir().expect("create temp dir");
                        let mut segment = OpenSegment::new();
                        for _ in 0..num_bundles {
                            segment.append(&bundle).expect("append succeeds");
                        }
                        (temp_dir, segment)
                    },
                    |(temp_dir, segment)| {
                        let segment_path = temp_dir.path().join("bench_segment.qseg");
                        let writer = SegmentWriter::new(SegmentSeq::new(1));
                        writer
                            .write_segment(&segment_path, segment)
                            .expect("write succeeds");
                        temp_dir // Keep temp_dir alive until write completes
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    // ── Multi-stream write (schema evolution scenario) ──
    // 10 different fingerprints = 10 different streams
    {
        let bundles: Vec<BenchBundle> = (0..10u8)
            .map(|i| BenchBundle::single_slot(1_000, [i; 32]))
            .collect();

        group.throughput(Throughput::Elements(10));
        group.bench_function("multi_stream/10_schemas", |b| {
            b.iter_batched(
                || {
                    let temp_dir = tempfile::tempdir().expect("create temp dir");
                    let mut segment = OpenSegment::new();
                    for bundle in &bundles {
                        segment.append(bundle).expect("append succeeds");
                    }
                    (temp_dir, segment)
                },
                |(temp_dir, segment)| {
                    let segment_path = temp_dir.path().join("bench_segment.qseg");
                    let writer = SegmentWriter::new(SegmentSeq::new(1));
                    writer
                        .write_segment(&segment_path, segment)
                        .expect("write succeeds");
                    temp_dir
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    // ── Multi-slot write (OTAP logs payload structure) ──
    // 4 slots: ResourceAttrs, ScopeAttrs, Logs, LogAttrs
    {
        let bundle = BenchBundle::multi_slot(1_000);

        group.throughput(Throughput::Elements(50));
        group.bench_function("multi_slot/50_bundles", |b| {
            b.iter_batched(
                || {
                    let temp_dir = tempfile::tempdir().expect("create temp dir");
                    let mut segment = OpenSegment::new();
                    for _ in 0..50 {
                        segment.append(&bundle).expect("append succeeds");
                    }
                    (temp_dir, segment)
                },
                |(temp_dir, segment)| {
                    let segment_path = temp_dir.path().join("bench_segment.qseg");
                    let writer = SegmentWriter::new(SegmentSeq::new(1));
                    writer
                        .write_segment(&segment_path, segment)
                        .expect("write succeeds");
                    temp_dir
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Read benchmarks
// ─────────────────────────────────────────────────────────────────────────────

/// Helper to create a segment file for read benchmarks.
fn create_test_segment(num_bundles: usize, num_rows: usize) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let segment_path = temp_dir.path().join("test_segment.qseg");

    // Create and populate segment
    let mut segment = OpenSegment::new();
    let bundle = BenchBundle::multi_slot(num_rows);
    for _ in 0..num_bundles {
        segment.append(&bundle).expect("append succeeds");
    }

    // Write to file
    let writer = SegmentWriter::new(SegmentSeq::new(1));
    writer
        .write_segment(&segment_path, segment)
        .expect("write succeeds");

    (temp_dir, segment_path)
}

/// Benchmark SegmentReader::open (read into memory) vs open_mmap (memory-mapped).
///
/// Uses 1,000 rows per bundle to reflect typical production batch sizes.
fn segment_reader_open(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_reader_open");

    for num_bundles in [10, 50, 100] {
        let (_temp_dir, segment_path) = create_test_segment(num_bundles, 1_000);

        // Get file size for throughput calculation
        let file_size = std::fs::metadata(&segment_path)
            .expect("segment file exists")
            .len();
        group.throughput(Throughput::Bytes(file_size));

        // Benchmark open (read into memory)
        group.bench_with_input(
            BenchmarkId::new("read", num_bundles),
            &segment_path,
            |b, path| {
                b.iter(|| SegmentReader::open(path).expect("open succeeds"));
            },
        );

        // Benchmark open_mmap (memory-mapped)
        #[cfg(feature = "mmap")]
        group.bench_with_input(
            BenchmarkId::new("mmap", num_bundles),
            &segment_path,
            |b, path| {
                b.iter(|| SegmentReader::open_mmap(path).expect("open_mmap succeeds"));
            },
        );
    }

    group.finish();
}

/// Benchmark reading all bundles from a segment.
/// Opens the reader once, then measures bundle reconstruction throughput.
///
/// Uses 1,000 rows per bundle to reflect typical production batch sizes.
fn segment_read_bundles(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_read_bundles");

    for num_bundles in [10, 50, 100] {
        let (_temp_dir, segment_path) = create_test_segment(num_bundles, 1_000);

        group.throughput(Throughput::Elements(num_bundles as u64));

        // Read all bundles using standard open
        {
            let reader = SegmentReader::open(&segment_path).expect("open succeeds");

            group.bench_function(BenchmarkId::new("read", num_bundles), |b| {
                b.iter(|| {
                    for entry in reader.manifest() {
                        let _bundle = reader.read_bundle(entry).expect("read bundle");
                    }
                });
            });
        }

        // Read all bundles using mmap
        #[cfg(feature = "mmap")]
        {
            let reader = SegmentReader::open_mmap(&segment_path).expect("open_mmap succeeds");

            group.bench_function(BenchmarkId::new("mmap", num_bundles), |b| {
                b.iter(|| {
                    for entry in reader.manifest() {
                        let _bundle = reader.read_bundle(entry).expect("read bundle");
                    }
                });
            });
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    segment_append_throughput,
    segment_accumulate_many,
    segment_write,
    segment_reader_open,
    segment_read_bundles,
);
criterion_main!(benches);
