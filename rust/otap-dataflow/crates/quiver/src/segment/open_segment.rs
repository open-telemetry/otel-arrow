// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! In-memory buffer for an open (not yet finalized) segment.
//!
//! An `OpenSegment` accumulates incoming [`RecordBundle`]s by routing each
//! payload slot to the appropriate [`StreamAccumulator`] based on its
//! `(slot_id, schema_fingerprint)` key. It tracks the batch manifest so that
//! finalized segments can reconstruct the original bundles.
//!
//! # Lifecycle
//!
//! 1. Create with [`OpenSegment::new`].
//! 2. Append bundles via [`OpenSegment::append`]; each call routes payloads
//!    to stream accumulators and records a manifest entry.
//! 3. When ready to finalize (size threshold, duration, or shutdown), call
//!    [`OpenSegment::finalize`] to produce the segment data and metadata.
//!
//! [`RecordBundle`]: crate::record_bundle::RecordBundle

use std::collections::HashMap;
use std::time::Instant;

use arrow_schema::SchemaRef;

use super::error::SegmentError;
use super::stream_accumulator::StreamAccumulator;
use super::types::{
    ManifestEntry, StreamId, StreamKey, StreamMetadata,
    MAX_BUNDLES_PER_SEGMENT, MAX_STREAMS_PER_SEGMENT,
};
use crate::record_bundle::RecordBundle;

/// In-memory buffer for an open segment.
///
/// Routes incoming `RecordBundle` payloads to per-stream accumulators and
/// tracks the batch manifest for later reconstruction.
pub struct OpenSegment {
    /// Maps `(slot_id, schema_fingerprint)` to the stream accumulator.
    streams: HashMap<StreamKey, StreamAccumulator>,
    /// Next stream ID to assign when a new `(slot, schema)` pair is seen.
    next_stream_id: u32,
    /// Ordered list of manifest entries, one per ingested bundle.
    manifest: Vec<ManifestEntry>,
    /// Whether finalize() has been called.
    finalized: bool,
    /// Timestamp when the first bundle was appended (None if empty).
    opened_at: Option<Instant>,
}

impl OpenSegment {
    /// Creates a new open segment buffer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
            next_stream_id: 0,
            manifest: Vec::new(),
            finalized: false,
            opened_at: None,
        }
    }

    /// Returns the number of bundles accumulated so far.
    #[must_use]
    pub fn bundle_count(&self) -> usize {
        self.manifest.len()
    }

    /// Returns the number of distinct streams (unique `(slot, schema)` pairs).
    #[must_use]
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Returns true if no bundles have been appended.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.manifest.is_empty()
    }

    /// Returns true if finalize() has been called.
    #[must_use]
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Returns the instant when the first bundle was appended, if any.
    ///
    /// This is used for time-based finalization decisions.
    #[must_use]
    pub fn opened_at(&self) -> Option<Instant> {
        self.opened_at
    }

    /// Estimates the current in-memory size of accumulated data.
    ///
    /// This is a rough estimate based on the number of rows and streams,
    /// suitable for triggering finalization decisions. It does not account
    /// for Arrow buffer overhead or manifest size.
    #[must_use]
    pub fn estimated_size_bytes(&self) -> usize {
        // TODO: Track actual byte sizes as batches are appended.
        // For now, return a placeholder based on row counts.
        self.streams
            .values()
            .map(|acc| {
                // Rough estimate: assume 100 bytes per row as a placeholder
                acc.row_count() as usize * 100
            })
            .sum()
    }

    /// Appends a `RecordBundle` to this open segment.
    ///
    /// Each populated slot in the bundle is routed to the appropriate stream
    /// accumulator based on its `(slot_id, schema_fingerprint)` key. Returns
    /// the manifest entry recording where each payload was stored.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::AccumulatorFinalized`] if the segment has
    /// already been finalized.
    /// Returns [`SegmentError::InvalidFormat`] if adding this bundle would
    /// exceed segment limits.
    pub fn append<B: RecordBundle>(&mut self, bundle: &B) -> Result<ManifestEntry, SegmentError> {
        if self.finalized {
            return Err(SegmentError::AccumulatorFinalized);
        }

        // Check bundle limit before appending
        if self.manifest.len() >= MAX_BUNDLES_PER_SEGMENT {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "segment already has {} bundles, cannot exceed limit of {}",
                    self.manifest.len(),
                    MAX_BUNDLES_PER_SEGMENT
                ),
            });
        }

        // Track when the first bundle was appended for time-based finalization
        if self.opened_at.is_none() {
            self.opened_at = Some(Instant::now());
        }

        let bundle_index = self.manifest.len() as u32;
        let mut entry = ManifestEntry::new(bundle_index);

        // Iterate over all slots defined in the bundle's descriptor
        for slot_desc in &bundle.descriptor().slots {
            let slot_id = slot_desc.id;

            // Check if this slot is populated
            if let Some(payload) = bundle.payload(slot_id) {
                let stream_key: StreamKey = (slot_id, payload.schema_fingerprint);

                // Check stream limit before potentially creating a new stream
                if !self.streams.contains_key(&stream_key)
                    && self.streams.len() >= MAX_STREAMS_PER_SEGMENT
                {
                    return Err(SegmentError::InvalidFormat {
                        message: format!(
                            "segment already has {} streams, cannot exceed limit of {}",
                            self.streams.len(),
                            MAX_STREAMS_PER_SEGMENT
                        ),
                    });
                }

                // Get or create the stream accumulator for this (slot, schema) pair
                let accumulator =
                    self.get_or_create_accumulator(stream_key, payload.batch.schema());

                // Append the batch and record the chunk index
                let chunk_index = accumulator.append(payload.batch.clone())?;
                entry.add_slot(slot_id, accumulator.stream_id(), chunk_index);
            }
        }

        self.manifest.push(entry.clone());
        Ok(entry)
    }

    /// Gets or creates a stream accumulator for the given key.
    fn get_or_create_accumulator(
        &mut self,
        key: StreamKey,
        schema: SchemaRef,
    ) -> &mut StreamAccumulator {
        let (slot_id, schema_fingerprint) = key;
        if !self.streams.contains_key(&key) {
            let stream_id = StreamId::new(self.next_stream_id);
            self.next_stream_id += 1;

            let accumulator =
                StreamAccumulator::new(stream_id, slot_id, schema_fingerprint, schema);
            let _ = self.streams.insert(key, accumulator);
        }

        self.streams.get_mut(&key).expect("just inserted or exists")
    }

    /// Finalizes the open segment, producing stream data and metadata.
    ///
    /// Returns a tuple of:
    /// - `Vec<(Vec<u8>, StreamMetadata)>`: The serialized Arrow IPC data and
    ///   metadata for each stream.
    /// - `Vec<ManifestEntry>`: The batch manifest entries.
    ///
    /// After finalization, no more bundles can be appended.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::AccumulatorFinalized`] if already finalized.
    /// Returns [`SegmentError::EmptySegment`] if no bundles were appended.
    pub fn finalize(
        mut self,
    ) -> Result<(Vec<(Vec<u8>, StreamMetadata)>, Vec<ManifestEntry>), SegmentError> {
        if self.finalized {
            return Err(SegmentError::AccumulatorFinalized);
        }
        if self.manifest.is_empty() {
            return Err(SegmentError::EmptySegment);
        }
        self.finalized = true;

        let mut streams_data = Vec::with_capacity(self.streams.len());
        let mut current_offset: u64 = 0;

        // Finalize each stream accumulator
        // Sort by stream ID for deterministic output order
        let mut stream_entries: Vec<_> = self.streams.into_iter().collect();
        stream_entries.sort_by_key(|(_, acc)| acc.stream_id());

        for (_, accumulator) in stream_entries {
            let (ipc_bytes, metadata) = accumulator.finalize(current_offset)?;
            current_offset += metadata.byte_length;
            streams_data.push((ipc_bytes, metadata));
        }

        Ok((streams_data, self.manifest))
    }

    /// Returns an iterator over the manifest entries.
    pub fn manifest(&self) -> impl Iterator<Item = &ManifestEntry> {
        self.manifest.iter()
    }
}

impl Default for OpenSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for OpenSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenSegment")
            .field("stream_count", &self.stream_count())
            .field("bundle_count", &self.bundle_count())
            .field("finalized", &self.finalized)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use arrow_ipc::reader::FileReader;

    use super::*;
    use crate::record_bundle::SlotId;
    use crate::segment::ChunkIndex;
    use crate::segment::test_utils::{TestBundle, make_batch, slot_descriptors, test_schema};

    #[test]
    fn new_segment_is_empty() {
        let seg = OpenSegment::new();
        assert!(seg.is_empty());
        assert_eq!(seg.bundle_count(), 0);
        assert_eq!(seg.stream_count(), 0);
        assert!(!seg.is_finalized());
        assert!(seg.opened_at().is_none());
    }

    #[test]
    fn opened_at_is_set_on_first_append() {
        let mut seg = OpenSegment::new();
        let schema = test_schema();
        let batch = make_batch(&schema, &[1], &["a"]);
        let fp = [0x11u8; 32];

        // Before append, opened_at should be None
        assert!(seg.opened_at().is_none());

        let bundle = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch);
        let _ = seg.append(&bundle).expect("append succeeds");

        // After append, opened_at should be set
        assert!(seg.opened_at().is_some());
        let opened_at = seg.opened_at().unwrap();

        // A second append should not change opened_at
        let batch2 = make_batch(&test_schema(), &[2], &["b"]);
        let bundle2 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch2);
        let _ = seg.append(&bundle2).expect("append succeeds");

        assert_eq!(
            seg.opened_at().unwrap(),
            opened_at,
            "opened_at should not change after first bundle"
        );
    }

    #[test]
    fn append_single_bundle_single_slot() {
        let mut seg = OpenSegment::new();
        let schema = test_schema();
        let batch = make_batch(&schema, &[1, 2, 3], &["a", "b", "c"]);
        let fp = [0x11u8; 32];

        let bundle = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch);

        let entry = seg.append(&bundle).expect("append should succeed");

        assert_eq!(seg.bundle_count(), 1);
        assert_eq!(seg.stream_count(), 1);
        assert_eq!(entry.bundle_index, 0);
        assert_eq!(entry.slot_count(), 1);
        assert!(entry.get_slot(SlotId::new(0)).is_some());
        assert!(entry.get_slot(SlotId::new(1)).is_none());
    }

    #[test]
    fn append_single_bundle_multiple_slots() {
        let mut seg = OpenSegment::new();
        let schema = test_schema();
        let batch0 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch1 = make_batch(&schema, &[3, 4, 5], &["c", "d", "e"]);
        let fp0 = [0x11u8; 32];
        let fp1 = [0x22u8; 32];

        let bundle = TestBundle::new(slot_descriptors())
            .with_payload(SlotId::new(0), fp0, batch0)
            .with_payload(SlotId::new(1), fp1, batch1);

        let entry = seg.append(&bundle).expect("append should succeed");

        assert_eq!(seg.bundle_count(), 1);
        assert_eq!(seg.stream_count(), 2);
        assert_eq!(entry.slot_count(), 2);

        // Each slot should map to a different stream
        let ref0 = entry.get_slot(SlotId::new(0)).unwrap();
        let ref1 = entry.get_slot(SlotId::new(1)).unwrap();
        assert_ne!(ref0.stream_id, ref1.stream_id);
    }

    #[test]
    fn multiple_bundles_same_schema_share_stream() {
        let mut seg = OpenSegment::new();
        let schema = test_schema();
        let fp = [0x33u8; 32];

        let batch1 = make_batch(&schema, &[1], &["a"]);
        let batch2 = make_batch(&schema, &[2], &["b"]);
        let batch3 = make_batch(&schema, &[3], &["c"]);

        let bundle1 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch1);
        let bundle2 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch2);
        let bundle3 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch3);

        let entry1 = seg.append(&bundle1).unwrap();
        let entry2 = seg.append(&bundle2).unwrap();
        let entry3 = seg.append(&bundle3).unwrap();

        // All bundles should use the same stream
        assert_eq!(seg.stream_count(), 1);
        assert_eq!(seg.bundle_count(), 3);

        let stream_id = entry1.get_slot(SlotId::new(0)).unwrap().stream_id;
        assert_eq!(
            entry2.get_slot(SlotId::new(0)).unwrap().stream_id,
            stream_id
        );
        assert_eq!(
            entry3.get_slot(SlotId::new(0)).unwrap().stream_id,
            stream_id
        );

        // But each should have a different chunk index
        assert_eq!(
            entry1.get_slot(SlotId::new(0)).unwrap().chunk_index,
            ChunkIndex::new(0)
        );
        assert_eq!(
            entry2.get_slot(SlotId::new(0)).unwrap().chunk_index,
            ChunkIndex::new(1)
        );
        assert_eq!(
            entry3.get_slot(SlotId::new(0)).unwrap().chunk_index,
            ChunkIndex::new(2)
        );
    }

    #[test]
    fn different_schemas_create_different_streams() {
        let mut seg = OpenSegment::new();
        let schema = test_schema();
        let fp1 = [0x11u8; 32];
        let fp2 = [0x22u8; 32]; // Different fingerprint = different schema

        let batch1 = make_batch(&schema, &[1], &["a"]);
        let batch2 = make_batch(&schema, &[2], &["b"]);

        let bundle1 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp1, batch1);
        let bundle2 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp2, batch2);

        let entry1 = seg.append(&bundle1).unwrap();
        let entry2 = seg.append(&bundle2).unwrap();

        // Different fingerprints should create different streams
        assert_eq!(seg.stream_count(), 2);

        let stream_id1 = entry1.get_slot(SlotId::new(0)).unwrap().stream_id;
        let stream_id2 = entry2.get_slot(SlotId::new(0)).unwrap().stream_id;
        assert_ne!(stream_id1, stream_id2);
    }

    #[test]
    fn finalize_produces_valid_ipc_streams() {
        let mut seg = OpenSegment::new();
        let schema = test_schema();
        let fp0 = [0x44u8; 32];
        let fp1 = [0x55u8; 32];

        // Stream 0 (slot 0): two batches with 2 and 1 rows
        let batch0_1 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch0_2 = make_batch(&schema, &[3], &["c"]);

        // Stream 1 (slot 1): one batch with 4 rows
        let batch1_1 = make_batch(&schema, &[10, 20, 30, 40], &["w", "x", "y", "z"]);

        let bundle1 = TestBundle::new(slot_descriptors())
            .with_payload(SlotId::new(0), fp0, batch0_1)
            .with_payload(SlotId::new(1), fp1, batch1_1);
        let bundle2 =
            TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp0, batch0_2);

        let _ = seg.append(&bundle1).unwrap();
        let _ = seg.append(&bundle2).unwrap();

        let (streams, manifest) = seg.finalize().expect("finalize should succeed");

        assert_eq!(streams.len(), 2);
        assert_eq!(manifest.len(), 2);

        // Streams are sorted by stream_id; stream 0 was created first (slot 0)
        // Stream 0: slot 0, 2 chunks, 3 rows total
        let (ipc_bytes_0, metadata_0) = &streams[0];
        assert_eq!(metadata_0.slot_id, SlotId::new(0));
        assert_eq!(metadata_0.chunk_count, 2);
        assert_eq!(metadata_0.row_count, 3);

        let cursor_0 = Cursor::new(ipc_bytes_0.as_slice());
        let reader_0 = FileReader::try_new(cursor_0, None).expect("valid IPC stream 0");
        let batches_0: Vec<_> = reader_0.map(|r| r.unwrap()).collect();
        assert_eq!(batches_0.len(), 2);
        assert_eq!(batches_0[0].num_rows(), 2);
        assert_eq!(batches_0[1].num_rows(), 1);

        // Stream 1: slot 1, 1 chunk, 4 rows
        let (ipc_bytes_1, metadata_1) = &streams[1];
        assert_eq!(metadata_1.slot_id, SlotId::new(1));
        assert_eq!(metadata_1.chunk_count, 1);
        assert_eq!(metadata_1.row_count, 4);

        let cursor_1 = Cursor::new(ipc_bytes_1.as_slice());
        let reader_1 = FileReader::try_new(cursor_1, None).expect("valid IPC stream 1");
        let batches_1: Vec<_> = reader_1.map(|r| r.unwrap()).collect();
        assert_eq!(batches_1.len(), 1);
        assert_eq!(batches_1[0].num_rows(), 4);
    }

    #[test]
    fn finalize_empty_segment_fails() {
        let seg = OpenSegment::new();
        let result = seg.finalize();
        assert!(matches!(result, Err(SegmentError::EmptySegment)));
    }

    #[test]
    fn append_after_finalize_fails() {
        let mut seg = OpenSegment::new();
        seg.finalized = true; // Simulate finalized state

        let schema = test_schema();
        let batch = make_batch(&schema, &[1], &["a"]);
        let bundle =
            TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), [0u8; 32], batch);

        let result = seg.append(&bundle);
        assert!(matches!(result, Err(SegmentError::AccumulatorFinalized)));
    }

    #[test]
    fn manifest_iterator_returns_entries_in_order() {
        let mut seg = OpenSegment::new();
        let schema = test_schema();
        let fp = [0x55u8; 32];

        for i in 0..5 {
            let batch = make_batch(&schema, &[i], &["x"]);
            let bundle =
                TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch);
            let _ = seg.append(&bundle).unwrap();
        }

        let indices: Vec<_> = seg.manifest().map(|e| e.bundle_index).collect();
        assert_eq!(indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn estimated_size_increases_with_data() {
        let mut seg = OpenSegment::new();
        let schema = test_schema();
        let fp = [0x66u8; 32];

        let initial_size = seg.estimated_size_bytes();
        assert_eq!(initial_size, 0);

        let batch = make_batch(&schema, &[1, 2, 3, 4, 5], &["a", "b", "c", "d", "e"]);
        let bundle = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch);
        let _ = seg.append(&bundle).unwrap();

        let after_append = seg.estimated_size_bytes();
        assert!(after_append > initial_size);
    }

    #[test]
    fn debug_impl_does_not_panic() {
        let seg = OpenSegment::new();
        let debug_str = format!("{:?}", seg);
        assert!(debug_str.contains("OpenSegment"));
    }

    #[test]
    fn default_creates_empty_segment() {
        let seg = OpenSegment::default();
        assert!(seg.is_empty());
    }
}
