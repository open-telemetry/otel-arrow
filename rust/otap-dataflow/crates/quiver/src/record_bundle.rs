// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal traits that adapters implement to feed data into Quiver.

use std::borrow::Cow;
use std::time::SystemTime;

use arrow_array::RecordBatch;
use arrow_array::types::{ArrowPrimitiveType, UInt16Type};
use arrow_schema::DataType;

// ─────────────────────────────────────────────────────────────────────────────
// Arrow Type Mapping
// ─────────────────────────────────────────────────────────────────────────────

/// Trait for newtypes that wrap an Arrow primitive type.
///
/// This trait bridges Quiver's domain-specific newtypes (like [`SlotId`],
/// [`StreamId`](crate::segment::StreamId), [`ChunkIndex`](crate::segment::ChunkIndex))
/// with Arrow's type system. It ensures the Arrow schema stays synchronized
/// with the Rust representation.
///
/// # Why This Pattern?
///
/// When encoding segment metadata as Arrow IPC, we need to construct Arrow
/// schemas with the correct `DataType` for each field. Without this trait,
/// the schema definition and the actual Rust type could drift apart:
///
/// ```ignore
/// // Problem: SlotId is u16, but if someone changes it to u32,
/// // this schema definition becomes wrong:
/// Field::new("slot_id", DataType::UInt16, false)  // Hardcoded!
/// ```
///
/// With `ArrowPrimitive`, the schema always derives from the Rust type:
///
/// ```ignore
/// Field::new("slot_id", SlotId::arrow_data_type(), false)
/// ```
///
/// # Compile-Time Safety
///
/// Use `assert_arrow_type_matches!` after implementing this trait to verify
/// the `ArrowType` matches the newtype's inner primitive at compile time.
///
/// # Example
///
/// ```ignore
/// pub struct SlotId(u16);
///
/// impl ArrowPrimitive for SlotId {
///     type ArrowType = UInt16Type;
/// }
///
/// // Compile-time check that u16 == UInt16Type::Native
/// assert_arrow_type_matches!(SlotId, u16, UInt16Type);
///
/// // In schema construction:
/// Field::new("slot_id", SlotId::arrow_data_type(), false)
/// ```
///
/// [`StreamId`]: crate::segment::StreamId
/// [`ChunkIndex`]: crate::segment::ChunkIndex
pub trait ArrowPrimitive {
    /// The Arrow primitive type corresponding to this Rust type.
    ///
    /// This must implement [`ArrowPrimitiveType`], which provides `DATA_TYPE`
    /// and `Native` associated items.
    type ArrowType: ArrowPrimitiveType;

    /// Returns the Arrow `DataType` for schema construction.
    ///
    /// This delegates to `<Self::ArrowType as ArrowPrimitiveType>::DATA_TYPE`.
    #[must_use]
    fn arrow_data_type() -> DataType {
        Self::ArrowType::DATA_TYPE
    }
}

/// Compile-time assertion that a newtype's inner primitive matches its Arrow type.
///
/// This macro generates a const assertion that fails at compile time if the
/// newtype's inner type doesn't match `<ArrowType as ArrowPrimitiveType>::Native`.
/// It checks both size and alignment to catch type mismatches.
///
/// # Usage
///
/// Call this immediately after implementing [`ArrowPrimitive`] for a newtype:
///
/// ```ignore
/// pub struct SlotId(u16);
///
/// impl ArrowPrimitive for SlotId {
///     type ArrowType = UInt16Type;
/// }
///
/// // Must match: inner type (u16), Arrow type (UInt16Type)
/// assert_arrow_type_matches!(SlotId, u16, UInt16Type);
/// ```
///
/// # Failure Example
///
/// If someone changes the newtype but forgets to update the Arrow type:
///
/// ```ignore
/// pub struct SlotId(u32);  // Changed from u16!
/// impl ArrowPrimitive for SlotId { type ArrowType = UInt16Type; }  // Not updated!
/// assert_arrow_type_matches!(SlotId, u32, UInt16Type);  // COMPILE ERROR!
/// // Error: "ArrowType::Native size doesn't match inner type"
/// ```
#[macro_export]
macro_rules! assert_arrow_type_matches {
    ($newtype:ty, $inner:ty, $arrow_type:ty) => {
        const _: () = {
            // Assert that ArrowType::Native == $inner at compile time
            const fn check_same_type<T, U>() {
                assert!(
                    std::mem::size_of::<T>() == std::mem::size_of::<U>(),
                    "ArrowType::Native size doesn't match inner type"
                );
                assert!(
                    std::mem::align_of::<T>() == std::mem::align_of::<U>(),
                    "ArrowType::Native alignment doesn't match inner type"
                );
            }
            check_same_type::<
                <$arrow_type as arrow_array::types::ArrowPrimitiveType>::Native,
                $inner,
            >();
        };
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Slot Identification
// ─────────────────────────────────────────────────────────────────────────────

/// Logical identifier for a payload slot inside a bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SlotId(pub u16);

impl SlotId {
    /// Helper for constructing identifiers from primitive values.
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    /// Returns the raw numeric value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

impl ArrowPrimitive for SlotId {
    type ArrowType = UInt16Type;
}

// Compile-time check: SlotId's inner u16 must match UInt16Type::Native
assert_arrow_type_matches!(SlotId, u16, UInt16Type);

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

/// Stable fingerprint for a payload schema (currently 256 bits).
pub type SchemaFingerprint = [u8; 32];

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
                    schema_fingerprint: [0; 32],
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

        assert_eq!(payload.schema_fingerprint, [0; 32]);
        assert!(std::ptr::eq(payload.batch, &bundle.batch));
    }

    #[test]
    fn payload_returns_none_for_missing_slot() {
        let bundle = DummyBundle::new();
        assert!(bundle.payload(SlotId::new(42)).is_none());
    }

    #[test]
    fn descriptor_trait_method_returns_bundle_descriptor() {
        let bundle = DummyBundle::new();
        let descriptor = RecordBundle::descriptor(&bundle);

        assert_eq!(descriptor.slots.len(), 1);
        assert_eq!(descriptor.get(SlotId::new(0)).unwrap().label, "Logs");
    }
}
