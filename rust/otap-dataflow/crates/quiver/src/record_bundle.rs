// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal traits that adapters implement to feed data into Quiver.

use std::borrow::Cow;
use std::time::SystemTime;

use arrow_array::RecordBatch;

/// Logical identifier for a payload slot inside a bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SlotId(pub u16);

impl SlotId {
    /// Helper for constructing identifiers from primitive values.
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }
}

/// Metadata describing a slot that may appear inside a [`RecordBundle`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotDescriptor {
    /// Identifier used to reference the slot.
    pub id: SlotId,
    /// Human-friendly label (e.g. `Logs`).
    pub label: Cow<'static, str>,
}

impl SlotDescriptor {
    /// Creates a descriptor with the provided id + label.
    pub fn new(id: SlotId, label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

/// Metadata that every bundle must expose alongside payload accessors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundleDescriptor {
    /// Ordered slot metadata for the bundle's adapter.
    pub slots: Vec<SlotDescriptor>,
}

impl BundleDescriptor {
    /// Creates a descriptor using the provided slots.
    #[must_use]
    pub fn new(slots: Vec<SlotDescriptor>) -> Self {
        Self { slots }
    }

    /// Looks up the descriptor for a slot id, if present.
    #[must_use]
    pub fn get(&self, id: SlotId) -> Option<&SlotDescriptor> {
        self.slots.iter().find(|slot| slot.id == id)
    }
}

/// Stable fingerprint for a payload schema (currently 128 bits).
pub type SchemaFingerprint = [u8; 16];

/// Borrowed view into a payload slot; callers reuse their in-memory `RecordBatch`
/// instances to avoid copies.
pub struct PayloadRef<'a> {
    /// Logical schema fingerprint for the payload.
    pub schema_fingerprint: SchemaFingerprint,
    /// Borrowed Arrow `RecordBatch` to be serialized.
    pub batch: &'a RecordBatch,
}

/// Trait implemented by adapters that hand off telemetry batches to Quiver.
pub trait RecordBundle {
    /// Returns metadata describing supported slots.
    fn descriptor(&self) -> &BundleDescriptor;
    /// Wall-clock timestamp for retention metrics.
    fn ingestion_time(&self) -> SystemTime;
    /// Returns the payload for the requested slot if it is populated.
    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_array::RecordBatch;
    use arrow_schema::{DataType, Field, Schema};
    use std::sync::Arc;

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
                descriptor: BundleDescriptor::new(vec![SlotDescriptor::new(
                    SlotId::new(0),
                    "Logs",
                )]),
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

    #[test]
    fn descriptor_lookup_succeeds() {
        let bundle = DummyBundle::new();
        assert!(bundle.descriptor.get(SlotId::new(0)).is_some());
    }

    #[test]
    fn ingestion_time_is_not_in_future() {
        let bundle = DummyBundle::new();
        let observed = bundle.ingestion_time();
        assert!(observed.elapsed().is_ok());
    }

    #[test]
    fn payload_round_trip_matches_expected_slot() {
        let bundle = DummyBundle::new();
        let payload = bundle
            .payload(SlotId::new(0))
            .expect("slot 0 should be populated");

        assert_eq!(payload.schema_fingerprint, [0; 16]);
        assert!(std::ptr::eq(payload.batch, &bundle.batch));
    }

    #[test]
    fn payload_returns_none_for_missing_slot() {
        let bundle = DummyBundle::new();
        assert!(bundle.payload(SlotId::new(42)).is_none());
    }
}
