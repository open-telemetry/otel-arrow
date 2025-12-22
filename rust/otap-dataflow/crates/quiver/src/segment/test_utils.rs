// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared test utilities for segment module tests.
//!
//! This module provides common test helpers used across multiple test modules
//! within the segment subsystem, reducing duplication and ensuring consistency.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use arrow_array::{Int32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};

use crate::record_bundle::{
    BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
};

/// Creates a standard test schema with `id: Int32` and `name: Utf8` columns.
#[must_use]
pub fn test_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, true),
    ]))
}

/// Creates a standard fingerprint for testing.
#[must_use]
pub fn test_fingerprint() -> SchemaFingerprint {
    [0xABu8; 32]
}

/// Creates a `RecordBatch` with the given IDs and names using the test schema.
#[must_use]
pub fn make_batch(schema: &SchemaRef, ids: &[i32], names: &[&str]) -> RecordBatch {
    RecordBatch::try_new(
        Arc::clone(schema),
        vec![
            Arc::new(Int32Array::from(ids.to_vec())),
            Arc::new(StringArray::from(
                names.iter().map(|s| Some(*s)).collect::<Vec<_>>(),
            )),
        ],
    )
    .expect("valid batch")
}

/// Returns standard slot descriptors for testing.
///
/// Creates 20 slots matching typical OTAP metrics payloads:
/// - Resource, Scope, and metric type tables
/// - Attribute tables for each level
/// - Exemplar tables
///
/// This exercises realistic slot counts that will be common in production.
#[must_use]
pub fn slot_descriptors() -> Vec<SlotDescriptor> {
    vec![
        // Core metric tables
        SlotDescriptor::new(SlotId::new(0), "ResourceMetrics"),
        SlotDescriptor::new(SlotId::new(1), "Resource"),
        SlotDescriptor::new(SlotId::new(2), "ResourceAttrs"),
        SlotDescriptor::new(SlotId::new(3), "ScopeMetrics"),
        SlotDescriptor::new(SlotId::new(4), "Scope"),
        SlotDescriptor::new(SlotId::new(5), "ScopeAttrs"),
        // Gauge
        SlotDescriptor::new(SlotId::new(6), "Gauge"),
        SlotDescriptor::new(SlotId::new(7), "GaugeDataPoints"),
        SlotDescriptor::new(SlotId::new(8), "GaugeAttrs"),
        SlotDescriptor::new(SlotId::new(9), "GaugeExemplars"),
        // Sum
        SlotDescriptor::new(SlotId::new(10), "Sum"),
        SlotDescriptor::new(SlotId::new(11), "SumDataPoints"),
        SlotDescriptor::new(SlotId::new(12), "SumAttrs"),
        SlotDescriptor::new(SlotId::new(13), "SumExemplars"),
        // Histogram
        SlotDescriptor::new(SlotId::new(14), "Histogram"),
        SlotDescriptor::new(SlotId::new(15), "HistogramDataPoints"),
        SlotDescriptor::new(SlotId::new(16), "HistogramAttrs"),
        SlotDescriptor::new(SlotId::new(17), "HistogramExemplars"),
        // ExpHistogram
        SlotDescriptor::new(SlotId::new(18), "ExpHistogram"),
        SlotDescriptor::new(SlotId::new(19), "ExpHistogramDataPoints"),
    ]
}

/// A simple `RecordBundle` implementation for testing.
///
/// Allows setting up bundles with specific slot payloads for segment tests.
pub struct TestBundle {
    descriptor: BundleDescriptor,
    payloads: HashMap<SlotId, (SchemaFingerprint, RecordBatch)>,
}

impl TestBundle {
    /// Creates a new test bundle with the given slot descriptors.
    #[must_use]
    pub fn new(slots: Vec<SlotDescriptor>) -> Self {
        Self {
            descriptor: BundleDescriptor::new(slots),
            payloads: HashMap::new(),
        }
    }

    /// Adds a payload to the bundle for the specified slot.
    #[must_use]
    pub fn with_payload(
        mut self,
        slot_id: SlotId,
        fingerprint: SchemaFingerprint,
        batch: RecordBatch,
    ) -> Self {
        let _ = self.payloads.insert(slot_id, (fingerprint, batch));
        self
    }
}

impl RecordBundle for TestBundle {
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
