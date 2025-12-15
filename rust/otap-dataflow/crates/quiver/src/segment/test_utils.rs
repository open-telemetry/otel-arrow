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

/// Returns standard slot descriptors for testing (Logs at slot 0, LogAttrs at slot 1).
#[must_use]
pub fn slot_descriptors() -> Vec<SlotDescriptor> {
    vec![
        SlotDescriptor::new(SlotId::new(0), "Logs"),
        SlotDescriptor::new(SlotId::new(1), "LogAttrs"),
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
