use bytes::Bytes;
use crate::pdata::Context;
use ahash::{AHashMap as HashMap, AHashSet as HashSet};

/// Tracks relationships between batches ⇄ messages + their data.
/// High-perf: uses AHashMap/AHashSet (fastest hashing for u64 keys).
pub struct AzureMonitorExporterState {
    /// batch_id → set of msg_ids
    pub batch_to_msg: HashMap<u64, HashSet<u64>>,

    /// msg_id → set of batch_ids
    pub msg_to_batch: HashMap<u64, HashSet<u64>>,

    /// msg_id → (context, bytes)
    pub msg_data: HashMap<u64, (Context, Bytes)>,
}

impl AzureMonitorExporterState {
    /// Create state with preallocated capacity for high throughput.
    pub fn new() -> Self {
        Self {
            batch_to_msg: HashMap::with_capacity(262144),
            msg_to_batch: HashMap::with_capacity(262144),
            msg_data: HashMap::with_capacity(262144),
        }
    }

    /// Insert a message and associate it with a batch.
    /// If the msg already exists, its data will NOT be overwritten.
    #[inline]
    pub fn add_batch_msg_relationship(
        &mut self,
        batch_id: u64,
        msg_id: u64
    ) {
        // Batch → Msg
        _ = self.batch_to_msg
            .entry(batch_id)
            .or_default()
            .insert(msg_id);

        // Msg → Batch
        _ = self.msg_to_batch
            .entry(msg_id)
            .or_default()
            .insert(batch_id);
    }

    #[inline]
    pub fn delete_msg_data_if_orphaned(&mut self, msg_id: u64) -> Option<(Context, Bytes)> {
        match self.msg_to_batch.get(&msg_id) {
            Some(batches) if !batches.is_empty() => None, // Has batches, not orphaned
            _ => {
                _ = self.msg_to_batch.remove(&msg_id);
                self.msg_data.remove(&msg_id)
            }
        }
    }

    #[inline]
    pub fn add_msg_data(
        &mut self,
        msg_id: u64,
        context: Context,
        bytes: Bytes,
    ) {
        _ = self.msg_data
            .entry(msg_id)
            .or_insert((context, bytes));
    }

    /// Remove a batch on SUCCESS - only returns messages with no remaining batches.
    pub fn remove_batch_success(&mut self, batch_id: u64) -> Vec<(u64, Context, Bytes)> {
        let mut orphaned = Vec::new();

        if let Some(msgs) = self.batch_to_msg.remove(&batch_id) {
            for msg_id in msgs {
                if let Some(batches) = self.msg_to_batch.get_mut(&msg_id) {
                    _ = batches.remove(&batch_id);

                    // Only return if no remaining batches
                    if batches.is_empty() {
                        _ = self.msg_to_batch.remove(&msg_id);
                        if let Some((context, bytes)) = self.msg_data.remove(&msg_id) {
                            orphaned.push((msg_id, context, bytes));
                        }
                    }
                }
            }
        }

        orphaned
    }

    /// Remove a batch on FAILURE - returns ALL messages in batch, removing them entirely.
    /// Messages are removed from all their batch associations.
    pub fn remove_batch_failure(&mut self, batch_id: u64) -> Vec<(u64, Context, Bytes)> {
        let mut failed = Vec::new();

        if let Some(msgs) = self.batch_to_msg.remove(&batch_id) {
            for msg_id in msgs {
                // Remove this message from ALL batches it belongs to
                if let Some(other_batches) = self.msg_to_batch.remove(&msg_id) {
                    for other_batch_id in other_batches {
                        if other_batch_id != batch_id {
                            if let Some(other_batch_msgs) = self.batch_to_msg.get_mut(&other_batch_id) {
                                _ = other_batch_msgs.remove(&msg_id);
                            }
                        }
                    }
                }

                // Take the message data
                if let Some((context, bytes)) = self.msg_data.remove(&msg_id) {
                    failed.push((msg_id, context, bytes));
                }
            }
        }

        failed
    }

    /// Drain all remaining message data (for shutdown cleanup).
    /// Returns all messages that still have data, regardless of batch associations.
    pub fn drain_all(&mut self) -> Vec<(u64, Context, Bytes)> {
        // Clear batch relationships
        self.batch_to_msg.clear();
        self.msg_to_batch.clear();
        
        // Drain and return all message data
        self.msg_data
            .drain()
            .map(|(msg_id, (context, bytes))| (msg_id, context, bytes))
            .collect()
    }
}
