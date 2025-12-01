// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Placeholder engine entry-point so other crates can begin wiring dependencies.

use crate::config::QuiverConfig;
use crate::error::{QuiverError, Result};
use crate::record_bundle::RecordBundle;
use crate::telemetry::PersistenceMetrics;

/// Primary entry point for the persistence engine.
#[derive(Debug)]
pub struct QuiverEngine {
    config: QuiverConfig,
    metrics: PersistenceMetrics,
}

impl QuiverEngine {
    /// Validates configuration and returns a placeholder engine instance.
    pub fn new(config: QuiverConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            metrics: PersistenceMetrics::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record_bundle::{
        BundleDescriptor, PayloadRef, RecordBundle, SlotDescriptor, SlotId,
    };
    use arrow_schema::{DataType, Field, Schema};
    use std::sync::Arc;

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
                    schema_fingerprint: [0; 16],
                    batch: &self.batch,
                })
            } else {
                None
            }
        }
    }

    #[test]
    fn ingest_is_currently_unimplemented() {
        let engine = QuiverEngine::new(QuiverConfig::default()).expect("config valid");
        let bundle = DummyBundle::new();
        let err = engine.ingest(&bundle).expect_err("not implemented");
        assert!(matches!(err, QuiverError::Unimplemented { .. }));
        assert_eq!(engine.metrics().ingest_attempts(), 1);
    }

    #[test]
    fn config_returns_engine_configuration() {
        let config = QuiverConfig::builder()
            .data_dir("./config_return_test")
            .build()
            .expect("builder should produce valid config");
        let engine = QuiverEngine::new(config.clone()).expect("config valid");

        assert_eq!(engine.config(), &config);
    }

    #[test]
    fn dummy_bundle_payload_handles_missing_slot() {
        let bundle = DummyBundle::new();
        assert!(bundle.payload(SlotId::new(10)).is_none());
    }
}
