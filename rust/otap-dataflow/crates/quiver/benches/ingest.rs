//! Criterion micro-benchmark for the placeholder Quiver ingest path.

#![allow(missing_docs)]

use std::sync::Arc;
use std::time::SystemTime;

use arrow_array::RecordBatch;
use arrow_schema::{DataType, Field, Schema};
use criterion::{criterion_group, criterion_main, Criterion};
use quiver::config::QuiverConfig;
use quiver::engine::QuiverEngine;
use quiver::record_bundle::{BundleDescriptor, PayloadRef, RecordBundle, SlotDescriptor, SlotId};

struct DummyBundle {
    descriptor: BundleDescriptor,
    batch: RecordBatch,
}

impl DummyBundle {
    fn new() -> Self {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int64,
            false,
        )]));
        Self {
            descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(SlotId::new(0), "Logs")]),
            batch: RecordBatch::new_empty(schema),
        }
    }
}

impl RecordBundle for DummyBundle {
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn ingestion_time(&self) -> SystemTime {
        SystemTime::now()
    }

    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
        if slot == SlotId::new(0) {
            Some(PayloadRef {
                schema_fingerprint: [0; 16],
                batch: &self.batch,
            })
        } else {
            None
        }
    }
}

/// Measures the placeholder ingest path so we can wire perf tooling later.
fn ingest_placeholder(c: &mut Criterion) {
    let engine = QuiverEngine::new(QuiverConfig::default()).expect("config valid");
    let bundle = DummyBundle::new();

    let _ = c.bench_function("ingest_placeholder", |b| {
        b.iter(|| {
            let _ = engine.ingest(&bundle);
        });
    });
}

criterion_group!(benches, ingest_placeholder);
criterion_main!(benches);
