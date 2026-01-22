// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! WAL replay support for crash recovery.
//!
//! This module provides types for decoding WAL entries back into `RecordBundle`
//! implementations that can be replayed through the normal engine ingest path.

use std::io::Cursor;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arrow_array::RecordBatch;
use arrow_ipc::reader::StreamReader;

use crate::record_bundle::{
    BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
};

use super::reader::{DecodedWalSlot, WalRecordBundle};

/// A bundle reconstructed from WAL entries during recovery.
///
/// This type implements [`RecordBundle`] to allow replaying WAL entries
/// back through the normal engine ingest path during startup.
pub(crate) struct ReplayBundle {
    /// Slot descriptors for all slots in this bundle.
    descriptor: BundleDescriptor,
    /// The ingestion timestamp from the WAL entry.
    ingestion_time: SystemTime,
    /// Decoded slots with their Arrow RecordBatch data.
    slots: Vec<ReplaySlot>,
}

/// A single slot decoded from a WAL entry.
struct ReplaySlot {
    slot_id: SlotId,
    schema_fingerprint: SchemaFingerprint,
    batch: RecordBatch,
}

impl ReplayBundle {
    /// Creates a ReplayBundle from a WAL entry by decoding all slot payloads.
    ///
    /// Returns `None` if any slot payload fails to decode, along with the
    /// slot ID that failed for diagnostic purposes.
    ///
    /// # Slot Names
    ///
    /// The WAL format does not store slot names (labels like "Logs", "LogAttrs"),
    /// only the numeric `slot_id` and `schema_fingerprint`. This has **no semantic
    /// impact** because:
    ///
    /// - Stream routing uses `(slot_id, schema_fingerprint)` as the key, not names
    /// - Segment storage indexes by `slot_id`, not names
    /// - Names are purely for human debugging/observability
    ///
    /// All recovered slots use the placeholder name "recovered". This is only
    /// visible in debug logs during the brief replay window at startup.
    pub(crate) fn from_wal_entry(entry: &WalRecordBundle) -> Option<Self> {
        let mut slots = Vec::with_capacity(entry.slots.len());
        let mut descriptors = Vec::with_capacity(entry.slots.len());

        for wal_slot in &entry.slots {
            let batch = match decode_slot_payload(wal_slot) {
                Some(b) => b,
                None => {
                    tracing::debug!(
                        slot_id = wal_slot.slot_id.raw(),
                        sequence = entry.sequence,
                        payload_len = wal_slot.payload_len,
                        "failed to decode WAL slot payload"
                    );
                    return None;
                }
            };
            // Slot names are not in WAL; "recovered" is a placeholder with no semantic impact
            // (routing uses slot_id + schema_fingerprint, not the label)
            descriptors.push(SlotDescriptor::new(wal_slot.slot_id, "recovered"));
            slots.push(ReplaySlot {
                slot_id: wal_slot.slot_id,
                schema_fingerprint: wal_slot.schema_fingerprint,
                batch,
            });
        }

        // Convert nanosecond timestamp to SystemTime
        let ingestion_time = if entry.ingestion_ts_nanos >= 0 {
            UNIX_EPOCH + Duration::from_nanos(entry.ingestion_ts_nanos as u64)
        } else {
            // For negative timestamps (before epoch), just use epoch
            UNIX_EPOCH
        };

        Some(Self {
            descriptor: BundleDescriptor::new(descriptors),
            ingestion_time,
            slots,
        })
    }
}

impl RecordBundle for ReplayBundle {
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn ingestion_time(&self) -> SystemTime {
        self.ingestion_time
    }

    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
        self.slots
            .iter()
            .find(|s| s.slot_id == slot)
            .map(|s| PayloadRef {
                schema_fingerprint: s.schema_fingerprint,
                batch: &s.batch,
            })
    }
}

/// Decodes the Arrow IPC payload from a WAL slot.
fn decode_slot_payload(slot: &DecodedWalSlot) -> Option<RecordBatch> {
    let cursor = Cursor::new(&slot.payload);
    let mut reader = StreamReader::try_new(cursor, None).ok()?;
    reader.next()?.ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_array::Int64Array;
    use arrow_ipc::writer::StreamWriter;
    use arrow_schema::{DataType, Field, Schema};
    use std::sync::Arc;

    use super::super::WalOffset;

    /// Creates a test schema for use in tests.
    fn test_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int64,
            false,
        )]))
    }

    /// Creates a test RecordBatch with the given values.
    fn test_batch(values: &[i64]) -> RecordBatch {
        let schema = test_schema();
        let array = Int64Array::from(values.to_vec());
        RecordBatch::try_new(schema, vec![Arc::new(array)]).unwrap()
    }

    /// Encodes a RecordBatch as Arrow IPC stream bytes.
    fn encode_batch_to_ipc(batch: &RecordBatch) -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buffer, batch.schema().as_ref()).unwrap();
            writer.write(batch).unwrap();
            writer.finish().unwrap();
        }
        buffer
    }

    /// Creates a DecodedWalSlot for testing.
    fn make_decoded_slot(slot_id: u16, fingerprint_seed: u8, values: &[i64]) -> DecodedWalSlot {
        let batch = test_batch(values);
        let payload = encode_batch_to_ipc(&batch);
        let payload_len = payload.len() as u32;
        DecodedWalSlot {
            slot_id: SlotId::new(slot_id),
            schema_fingerprint: [fingerprint_seed; 32],
            row_count: values.len() as u32,
            payload_len,
            payload,
        }
    }

    /// Creates a WalRecordBundle for testing.
    fn make_wal_bundle(
        sequence: u64,
        ingestion_ts_nanos: i64,
        slots: Vec<DecodedWalSlot>,
    ) -> WalRecordBundle {
        // Build slot bitmap from slot IDs
        let slot_bitmap: u64 = slots
            .iter()
            .map(|s| 1u64 << s.slot_id.raw())
            .fold(0, |acc, bit| acc | bit);

        WalRecordBundle {
            offset: WalOffset {
                position: 0,
                next_offset: 100,
                sequence,
            },
            next_offset: 100,
            ingestion_ts_nanos,
            sequence,
            slot_bitmap,
            slots,
        }
    }

    #[test]
    fn decode_slot_payload_succeeds() {
        let slot = make_decoded_slot(0, 42, &[1, 2, 3, 4, 5]);
        let batch = decode_slot_payload(&slot);

        assert!(batch.is_some(), "decode should succeed");
        let batch = batch.unwrap();
        assert_eq!(batch.num_rows(), 5);
        assert_eq!(batch.num_columns(), 1);
    }

    #[test]
    fn decode_slot_payload_fails_on_invalid_ipc() {
        let slot = DecodedWalSlot {
            slot_id: SlotId::new(0),
            schema_fingerprint: [0; 32],
            row_count: 5,
            payload_len: 4,
            payload: vec![0, 1, 2, 3], // Invalid IPC bytes
        };
        let batch = decode_slot_payload(&slot);
        assert!(batch.is_none(), "decode should fail on invalid IPC");
    }

    #[test]
    fn decode_slot_payload_fails_on_empty_payload() {
        let slot = DecodedWalSlot {
            slot_id: SlotId::new(0),
            schema_fingerprint: [0; 32],
            row_count: 0,
            payload_len: 0,
            payload: vec![],
        };
        let batch = decode_slot_payload(&slot);
        assert!(batch.is_none(), "decode should fail on empty payload");
    }

    #[test]
    fn replay_bundle_from_wal_entry_single_slot() {
        let slot = make_decoded_slot(0, 42, &[10, 20, 30]);
        let entry = make_wal_bundle(1, 1_000_000_000, vec![slot]);

        let bundle = ReplayBundle::from_wal_entry(&entry);
        assert!(bundle.is_some(), "should create bundle");

        let bundle = bundle.unwrap();
        assert_eq!(bundle.descriptor().slots.len(), 1);
        assert_eq!(bundle.descriptor().slots[0].id, SlotId::new(0));

        let payload = bundle.payload(SlotId::new(0));
        assert!(payload.is_some());
        let payload = payload.unwrap();
        assert_eq!(payload.batch.num_rows(), 3);
        assert_eq!(payload.schema_fingerprint, [42; 32]);
    }

    #[test]
    fn replay_bundle_from_wal_entry_multiple_slots() {
        let slot0 = make_decoded_slot(0, 10, &[1, 2]);
        let slot1 = make_decoded_slot(1, 20, &[3, 4, 5]);
        let slot2 = make_decoded_slot(2, 30, &[6]);
        let entry = make_wal_bundle(1, 2_000_000_000, vec![slot0, slot1, slot2]);

        let bundle = ReplayBundle::from_wal_entry(&entry);
        assert!(bundle.is_some());

        let bundle = bundle.unwrap();
        assert_eq!(bundle.descriptor().slots.len(), 3);

        // Verify each slot
        let p0 = bundle.payload(SlotId::new(0)).unwrap();
        assert_eq!(p0.batch.num_rows(), 2);
        assert_eq!(p0.schema_fingerprint, [10; 32]);

        let p1 = bundle.payload(SlotId::new(1)).unwrap();
        assert_eq!(p1.batch.num_rows(), 3);
        assert_eq!(p1.schema_fingerprint, [20; 32]);

        let p2 = bundle.payload(SlotId::new(2)).unwrap();
        assert_eq!(p2.batch.num_rows(), 1);
        assert_eq!(p2.schema_fingerprint, [30; 32]);

        // Non-existent slot returns None
        assert!(bundle.payload(SlotId::new(99)).is_none());
    }

    #[test]
    fn replay_bundle_from_wal_entry_fails_if_slot_decode_fails() {
        let good_slot = make_decoded_slot(0, 10, &[1, 2, 3]);
        let bad_slot = DecodedWalSlot {
            slot_id: SlotId::new(1),
            schema_fingerprint: [20; 32],
            row_count: 5,
            payload_len: 2,
            payload: vec![0xFF, 0xFE], // Invalid IPC
        };
        let entry = make_wal_bundle(1, 1_000_000_000, vec![good_slot, bad_slot]);

        let bundle = ReplayBundle::from_wal_entry(&entry);
        assert!(bundle.is_none(), "should fail if any slot fails to decode");
    }

    #[test]
    fn replay_bundle_ingestion_time_positive() {
        let slot = make_decoded_slot(0, 0, &[1]);
        // 1 second after epoch
        let entry = make_wal_bundle(1, 1_000_000_000, vec![slot]);

        let bundle = ReplayBundle::from_wal_entry(&entry).unwrap();
        let expected = UNIX_EPOCH + Duration::from_secs(1);
        assert_eq!(bundle.ingestion_time(), expected);
    }

    #[test]
    fn replay_bundle_ingestion_time_negative_uses_epoch() {
        let slot = make_decoded_slot(0, 0, &[1]);
        // Negative timestamp (before epoch)
        let entry = make_wal_bundle(1, -1_000_000_000, vec![slot]);

        let bundle = ReplayBundle::from_wal_entry(&entry).unwrap();
        assert_eq!(bundle.ingestion_time(), UNIX_EPOCH);
    }

    #[test]
    fn replay_bundle_empty_slots() {
        // Entry with no slots - edge case
        let entry = make_wal_bundle(1, 1_000_000_000, vec![]);

        let bundle = ReplayBundle::from_wal_entry(&entry);
        assert!(bundle.is_some(), "empty slots should still create bundle");

        let bundle = bundle.unwrap();
        assert!(bundle.descriptor().slots.is_empty());
    }
}
