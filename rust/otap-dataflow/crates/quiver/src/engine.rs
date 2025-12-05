// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Placeholder engine entry-point so other crates can begin wiring dependencies.

use std::path::PathBuf;

use blake3::Hasher;
use parking_lot::Mutex;

use crate::config::QuiverConfig;
use crate::error::{QuiverError, Result};
use crate::record_bundle::RecordBundle;
use crate::telemetry::PersistenceMetrics;
use crate::wal::{WalWriter, WalWriterOptions};

/// Primary entry point for the persistence engine.
#[derive(Debug)]
pub struct QuiverEngine {
    config: QuiverConfig,
    metrics: PersistenceMetrics,
    wal_writer: Mutex<WalWriter>,
}

impl QuiverEngine {
    /// Validates configuration and returns a placeholder engine instance.
    pub fn new(config: QuiverConfig) -> Result<Self> {
        config.validate()?;
        let wal_writer = initialize_wal_writer(&config)?;

        Ok(Self {
            config,
            metrics: PersistenceMetrics::new(),
            wal_writer: Mutex::new(wal_writer),
        })
    }

    /// Returns the configuration backing this engine.
    pub fn config(&self) -> &QuiverConfig {
        &self.config
    }

    /// Returns metric counters for instrumentation layers.
    pub fn metrics(&self) -> &PersistenceMetrics {
        &self.metrics
    }

    /// Placeholder ingest method; later phases will implement WAL + segment logic.
    pub fn ingest<B: RecordBundle>(&self, bundle: &B) -> Result<()> {
        self.metrics.record_ingest_attempt();

        {
            let mut writer = self.wal_writer.lock();
            let _wal_offset = writer.append_bundle(bundle)?;
        }

        let descriptor = bundle.descriptor();
        let _ingestion_time = bundle.ingestion_time();

        for slot in &descriptor.slots {
            if let Some(payload) = bundle.payload(slot.id) {
                let _ = payload.schema_fingerprint;
            }
        }

        Err(QuiverError::unimplemented("ingest path"))
    }
}

fn initialize_wal_writer(config: &QuiverConfig) -> Result<WalWriter> {
    let wal_path = wal_path(config);
    let options = WalWriterOptions::new(
        wal_path,
        segment_cfg_hash(config),
        config.wal.flush_interval,
    )
    .with_max_wal_size(config.wal.max_size_bytes.get())
    .with_max_rotated_files(config.wal.max_rotated_files as usize)
    .with_rotation_target(config.wal.rotation_target_bytes.get());
    Ok(WalWriter::open(options)?)
}

fn wal_path(config: &QuiverConfig) -> PathBuf {
    config.data_dir.join("wal").join("quiver.wal")
}

fn segment_cfg_hash(config: &QuiverConfig) -> [u8; 16] {
    // Placeholder fingerprint derived from segment configuration; later phases will
    // mix in adapter-specific layout metadata once available.
    let mut hasher = Hasher::new();
    let _ = hasher.update(&config.segment.target_size_bytes.get().to_le_bytes());
    let _ = hasher.update(&config.segment.max_stream_count.to_le_bytes());
    let _ = hasher.update(&config.segment.max_open_duration.as_nanos().to_le_bytes());

    let digest = hasher.finalize();
    let mut hash = [0u8; 16];
    hash.copy_from_slice(&digest.as_bytes()[..16]);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record_bundle::{
        BundleDescriptor, PayloadRef, RecordBundle, SlotDescriptor, SlotId,
    };
    use crate::wal::WalReader;
    use arrow_schema::{DataType, Field, Schema};
    use std::sync::Arc;
    use tempfile::tempdir;

    struct DummyBundle {
        descriptor: BundleDescriptor,
        batch: arrow_array::RecordBatch,
    }

    impl DummyBundle {
        fn new() -> Self {
            let schema = Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::Int64,
                false,
            )]));
            Self {
                descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(
                    SlotId::new(0),
                    "Logs",
                )]),
                batch: arrow_array::RecordBatch::new_empty(schema),
            }
        }
    }

    impl RecordBundle for DummyBundle {
        fn descriptor(&self) -> &BundleDescriptor {
            &self.descriptor
        }

        fn ingestion_time(&self) -> std::time::SystemTime {
            std::time::SystemTime::now()
        }

        fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
            if slot == SlotId::new(0) {
                Some(PayloadRef {
                    schema_fingerprint: [0; 32],
                    batch: &self.batch,
                })
            } else {
                None
            }
        }
    }

    #[test]
    fn ingest_is_currently_unimplemented() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());
        let engine = QuiverEngine::new(config).expect("config valid");
        let bundle = DummyBundle::new();
        let err = engine.ingest(&bundle).expect_err("not implemented");
        assert!(matches!(err, QuiverError::Unimplemented { .. }));
        assert_eq!(engine.metrics().ingest_attempts(), 1);
    }

    #[test]
    fn config_returns_engine_configuration() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::builder()
            .data_dir(temp_dir.path())
            .build()
            .expect("builder should produce valid config");
        let engine = QuiverEngine::new(config.clone()).expect("config valid");

        assert_eq!(engine.config(), &config);
    }

    #[test]
    fn ingest_appends_to_wal_before_placeholder_error() {
        let temp_dir = tempdir().expect("tempdir");
        let config = QuiverConfig::default().with_data_dir(temp_dir.path());
        let engine = QuiverEngine::new(config).expect("config valid");
        let bundle = DummyBundle::new();

        let err = engine
            .ingest(&bundle)
            .expect_err("segment still unimplemented");
        assert!(matches!(err, QuiverError::Unimplemented { .. }));

        drop(engine);

        let wal_path = temp_dir.path().join("wal").join("quiver.wal");
        let mut reader = WalReader::open(&wal_path).expect("wal opens");
        let mut iter = reader.iter_from(0).expect("iterator");
        let entry = iter.next().expect("entry exists").expect("entry decodes");

        assert_eq!(entry.sequence, 0);
        assert_eq!(entry.slots.len(), 1);
        assert_eq!(entry.slot_bitmap.count_ones(), 1);
    }

    #[test]
    fn dummy_bundle_payload_handles_missing_slot() {
        let bundle = DummyBundle::new();
        assert!(bundle.payload(SlotId::new(10)).is_none());
    }
}
