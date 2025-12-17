// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pdata::Context;
use ahash::{AHashMap as HashMap, AHashSet as HashSet};
use otap_df_pdata::OtapPayload;

/// Tracks relationships between batches ⇄ messages + their data.
/// High-perf: uses AHashMap/AHashSet (fastest hashing for u64 keys).
pub struct AzureMonitorExporterState {
    /// batch_id → set of msg_ids
    pub batch_to_msg: HashMap<u64, HashSet<u64>>,

    /// msg_id → set of batch_ids
    pub msg_to_batch: HashMap<u64, HashSet<u64>>,

    /// msg_id → (context, optional payload for ack/nack)
    pub msg_to_data: HashMap<u64, (Context, OtapPayload)>,
}

impl AzureMonitorExporterState {
    /// Create state with preallocated capacity for high throughput.
    pub fn new() -> Self {
        Self {
            batch_to_msg: HashMap::with_capacity(262144),
            msg_to_batch: HashMap::with_capacity(262144),
            msg_to_data: HashMap::with_capacity(262144),
        }
    }

    /// Insert a message and associate it with a batch.
    /// If the msg already exists, its data will NOT be overwritten.
    #[inline]
    pub fn add_batch_msg_relationship(&mut self, batch_id: u64, msg_id: u64) {
        // Batch → Msg
        _ = self
            .batch_to_msg
            .entry(batch_id)
            .or_default()
            .insert(msg_id);

        // Msg → Batch
        _ = self
            .msg_to_batch
            .entry(msg_id)
            .or_default()
            .insert(batch_id);
    }

    #[inline]
    pub fn delete_msg_data_if_orphaned(
        &mut self,
        msg_id: u64,
    ) -> Option<(Context, OtapPayload)> {
        match self.msg_to_batch.get(&msg_id) {
            Some(batches) if !batches.is_empty() => None, // Has batches, not orphaned
            _ => {
                _ = self.msg_to_batch.remove(&msg_id);
                self.msg_to_data.remove(&msg_id)
            }
        }
    }

    #[inline]
    pub fn add_msg_to_data(
        &mut self,
        msg_id: u64,
        context: Context,
        otap_payload: OtapPayload,
    ) {
        _ = self
            .msg_to_data
            .entry(msg_id)
            .or_insert((context, otap_payload));
    }

    #[inline]
    pub fn remove_msg_to_data(&mut self, msg_id: u64) -> Option<(Context, OtapPayload)> {
        self.msg_to_data.remove(&msg_id)
    }

    /// Remove a batch on SUCCESS - only returns messages with no remaining batches.
    pub fn remove_batch_success(
        &mut self,
        batch_id: u64,
    ) -> Vec<(u64, Context, OtapPayload)> {
        let mut orphaned = Vec::new();

        if let Some(msgs) = self.batch_to_msg.remove(&batch_id) {
            for msg_id in msgs {
                if let Some(batches) = self.msg_to_batch.get_mut(&msg_id) {
                    _ = batches.remove(&batch_id);

                    // Only return if no remaining batches
                    if batches.is_empty() {
                        _ = self.msg_to_batch.remove(&msg_id);
                        if let Some((context, otap_payload)) = self.msg_to_data.remove(&msg_id) {
                            orphaned.push((msg_id, context, otap_payload));
                        }
                    }
                }
            }
        }

        orphaned
    }

    /// Remove a batch on FAILURE - returns ALL messages in batch, removing them entirely.
    /// Messages are removed from all their batch associations.
    pub fn remove_batch_failure(
        &mut self,
        batch_id: u64,
    ) -> Vec<(u64, Context, OtapPayload)> {
        let mut failed = Vec::new();

        if let Some(msgs) = self.batch_to_msg.remove(&batch_id) {
            for msg_id in msgs {
                // Remove this message from ALL batches it belongs to
                if let Some(other_batches) = self.msg_to_batch.remove(&msg_id) {
                    for other_batch_id in other_batches {
                        if other_batch_id != batch_id {
                            if let Some(other_batch_msgs) =
                                self.batch_to_msg.get_mut(&other_batch_id)
                            {
                                _ = other_batch_msgs.remove(&msg_id);
                            }
                        }
                    }
                }

                // Take the message data
                if let Some((context, otap_payload)) = self.msg_to_data.remove(&msg_id) {
                    failed.push((msg_id, context, otap_payload));
                }
            }
        }

        failed
    }

    /// Drain all remaining message data (for shutdown cleanup).
    /// Returns all messages that still have data, regardless of batch associations.
    pub fn drain_all(&mut self) -> Vec<(u64, Context, OtapPayload)> {
        // Clear batch relationships
        self.batch_to_msg.clear();
        self.msg_to_batch.clear();

        // Drain and return all message data
        self.msg_to_data
            .drain()
            .map(|(msg_id, (context, otap_payload))| (msg_id, context, otap_payload))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdata::Context;
    use bytes::Bytes;
    use otap_df_pdata::otlp::OtlpProtoBytes;

    /// Helper to create a test OtapPayload from bytes
    fn test_payload(data: &'static [u8]) -> OtapPayload {
        OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(Bytes::from_static(data)))
    }

    /// Helper to create an empty OtapPayload
    fn empty_payload() -> OtapPayload {
        OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(Bytes::new()))
    }

    #[test]
    fn test_new() {
        let state = AzureMonitorExporterState::new();
        assert!(state.batch_to_msg.is_empty());
        assert!(state.msg_to_batch.is_empty());
        assert!(state.msg_to_data.is_empty());
    }

    #[test]
    fn test_add_relationships_and_data() {
        let mut state = AzureMonitorExporterState::new();
        let msg_id = 1;
        let batch_id = 100;
        let payload = test_payload(b"test");

        state.add_batch_msg_relationship(batch_id, msg_id);
        state.add_msg_to_data(msg_id, Context::default(), payload);

        assert!(state.batch_to_msg.contains_key(&batch_id));
        assert!(state.batch_to_msg.get(&batch_id).unwrap().contains(&msg_id));

        assert!(state.msg_to_batch.contains_key(&msg_id));
        assert!(state.msg_to_batch.get(&msg_id).unwrap().contains(&batch_id));

        assert!(state.msg_to_data.contains_key(&msg_id));
    }

    #[test]
    fn test_delete_msg_data_if_orphaned() {
        let mut state = AzureMonitorExporterState::new();
        let msg_id = 1;

        // Case 1: Message has no batches (orphaned)
        state.add_msg_to_data(msg_id, Context::default(), test_payload(b"test"));
        let removed = state.delete_msg_data_if_orphaned(msg_id);
        assert!(removed.is_some());
        // Verify the payload matches
        let (_, payload) = removed.unwrap();
        match payload {
            OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) => {
                assert_eq!(bytes.as_ref(), b"test");
            }
            _ => panic!("Expected OtlpBytes::ExportLogsRequest"),
        }
        assert!(!state.msg_to_data.contains_key(&msg_id));

        // Case 2: Message has batches (not orphaned)
        state.add_msg_to_data(msg_id, Context::default(), test_payload(b"test"));
        state.add_batch_msg_relationship(100, msg_id);

        let removed = state.delete_msg_data_if_orphaned(msg_id);
        assert!(removed.is_none());
        assert!(state.msg_to_data.contains_key(&msg_id));
    }

    #[test]
    fn test_delete_msg_data_if_orphaned_with_empty_payload() {
        let mut state = AzureMonitorExporterState::new();
        let msg_id = 1;

        // Test with empty payload
        state.add_msg_to_data(msg_id, Context::default(), empty_payload());
        let removed = state.delete_msg_data_if_orphaned(msg_id);
        assert!(removed.is_some());
        let (_, payload) = removed.unwrap();
        match payload {
            OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) => {
                assert!(bytes.is_empty());
            }
            _ => panic!("Expected OtlpBytes::ExportLogsRequest"),
        }
    }

    #[test]
    fn test_remove_batch_success() {
        let mut state = AzureMonitorExporterState::new();
        let msg1 = 1;
        let msg2 = 2;
        let batch1 = 100;
        let batch2 = 101;

        // Setup:
        // msg1 is in batch1 only
        // msg2 is in batch1 AND batch2
        state.add_batch_msg_relationship(batch1, msg1);
        state.add_msg_to_data(msg1, Context::default(), test_payload(b"msg1"));

        state.add_batch_msg_relationship(batch1, msg2);
        state.add_batch_msg_relationship(batch2, msg2);
        state.add_msg_to_data(msg2, Context::default(), test_payload(b"msg2"));

        // Remove batch1 success
        let orphaned = state.remove_batch_success(batch1);

        // msg1 should be returned (it was only in batch1)
        assert_eq!(orphaned.len(), 1);
        assert_eq!(orphaned[0].0, msg1);
        assert!(!state.msg_to_data.contains_key(&msg1));

        // msg2 should NOT be returned (it is still in batch2)
        assert!(state.msg_to_data.contains_key(&msg2));

        // Verify msg2 relationships updated
        let msg2_batches = state.msg_to_batch.get(&msg2).unwrap();
        assert!(!msg2_batches.contains(&batch1));
        assert!(msg2_batches.contains(&batch2));

        // Remove batch2 success
        let orphaned2 = state.remove_batch_success(batch2);

        // msg2 should now be returned
        assert_eq!(orphaned2.len(), 1);
        assert_eq!(orphaned2[0].0, msg2);
        assert!(!state.msg_to_data.contains_key(&msg2));
    }

    #[test]
    fn test_remove_batch_failure() {
        let mut state = AzureMonitorExporterState::new();
        let msg1 = 1;
        let msg2 = 2;
        let batch1 = 100;
        let batch2 = 101;

        // Setup:
        // msg1 is in batch1 only
        // msg2 is in batch1 AND batch2
        state.add_batch_msg_relationship(batch1, msg1);
        state.add_msg_to_data(msg1, Context::default(), test_payload(b"msg1"));

        state.add_batch_msg_relationship(batch1, msg2);
        state.add_batch_msg_relationship(batch2, msg2);
        state.add_msg_to_data(msg2, Context::default(), test_payload(b"msg2"));

        // Remove batch1 failure
        // Should return ALL messages in batch1, even if they are in other batches
        let failed = state.remove_batch_failure(batch1);

        assert_eq!(failed.len(), 2);
        let ids: HashSet<u64> = failed.iter().map(|(id, _, _)| *id).collect();
        assert!(ids.contains(&msg1));
        assert!(ids.contains(&msg2));

        // Data should be gone
        assert!(!state.msg_to_data.contains_key(&msg1));
        assert!(!state.msg_to_data.contains_key(&msg2));

        // msg2 should be removed from batch2's list as well
        if let Some(batch2_msgs) = state.batch_to_msg.get(&batch2) {
            assert!(!batch2_msgs.contains(&msg2));
        }
    }

    #[test]
    fn test_drain_all() {
        let mut state = AzureMonitorExporterState::new();
        state.add_msg_to_data(1, Context::default(), test_payload(b"1"));
        state.add_msg_to_data(2, Context::default(), empty_payload()); // Test with empty payload

        let drained = state.drain_all();
        assert_eq!(drained.len(), 2);
        let ids: HashSet<u64> = drained.iter().map(|(id, _, _)| *id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }
}
