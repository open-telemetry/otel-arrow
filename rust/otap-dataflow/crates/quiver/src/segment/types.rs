// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core type definitions for segment storage.
//!
//! These types form the vocabulary used throughout the segment module for
//! tracking streams, manifests, and segment metadata. See the module-level
//! documentation in [`super`] for terminology definitions.

use std::collections::HashMap;

use crate::record_bundle::{SchemaFingerprint, SlotId};

use super::error::SegmentError;

// ─────────────────────────────────────────────────────────────────────────────
// Segment Format Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Magic bytes identifying a Quiver segment file.
pub(super) const SEGMENT_MAGIC: &[u8; 8] = b"QUIVER\0S";

/// Current segment file format version.
pub(super) const SEGMENT_VERSION: u16 = 1;

/// Size of the fixed trailer at the end of the segment file.
/// Layout: footer_size (4) + magic (8) + crc32 (4) = 16 bytes
pub(super) const TRAILER_SIZE: usize = 16;

/// Size of the footer for version 1.
/// Layout: version (2) + stream_count (4) + bundle_count (4) +
///         directory_offset (8) + directory_length (4) +
///         manifest_offset (8) + manifest_length (4) = 34 bytes
pub(super) const FOOTER_V1_SIZE: usize = 34;

// ─────────────────────────────────────────────────────────────────────────────
// Segment Limits
//
// These limits are enforced by both writers (to prevent creating invalid
// segments) and readers (to protect against malicious/corrupted files).
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum number of distinct streams in a single segment.
///
/// A stream is created for each unique `(slot_id, schema_fingerprint)` pair.
/// This limit prevents resource exhaustion from segments with excessive schema
/// diversity.
pub(super) const MAX_STREAMS_PER_SEGMENT: usize = 100_000;

/// Maximum number of bundles that can be stored in a single segment.
///
/// Each bundle represents one ingested `RecordBundle`. This limit bounds
/// manifest size and prevents resource exhaustion.
pub(super) const MAX_BUNDLES_PER_SEGMENT: usize = 10_000_000;

/// Maximum number of payload slots per bundle.
///
/// This matches the slot bitmap width (u64) used in manifest encoding.
/// Slots beyond this limit cannot be represented in the bitmap.
pub(super) const MAX_SLOTS_PER_BUNDLE: usize = 64;

/// Maximum number of Arrow IPC dictionaries per stream.
///
/// Limits resource consumption when parsing dictionary-encoded data.
pub(super) const MAX_DICTIONARIES_PER_STREAM: usize = 10_000;

/// Maximum number of Arrow IPC record batches (chunks) per stream.
///
/// Each chunk is an Arrow RecordBatch. This limit prevents excessive
/// memory allocation when building batch indices.
pub(super) const MAX_CHUNKS_PER_STREAM: usize = 10_000_000;

// ─────────────────────────────────────────────────────────────────────────────
// Stream Identification
// ─────────────────────────────────────────────────────────────────────────────

/// Unique identifier for a stream within a segment.
///
/// A stream is an ordered sequence of Arrow IPC messages for a specific
/// `(slot, schema_fingerprint)` pairing. Multiple [`RecordBundle`]s may
/// contribute chunks (Arrow `RecordBatch`es) to the same stream if they
/// share the same slot and schema.
///
/// [`RecordBundle`]: crate::record_bundle::RecordBundle
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StreamId(u32);

impl StreamId {
    /// Creates a new stream identifier.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw numeric value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for StreamId {
    fn from(raw: u32) -> Self {
        Self::new(raw)
    }
}

impl From<StreamId> for u32 {
    fn from(id: StreamId) -> Self {
        id.0
    }
}

/// Composite key identifying a stream by its slot and schema fingerprint.
///
/// Used internally to route incoming payloads to the correct stream accumulator.
/// Two payloads with the same `StreamKey` can share a stream; different keys
/// require separate streams.
///
/// The tuple contains `(SlotId, SchemaFingerprint)`.
pub type StreamKey = (SlotId, SchemaFingerprint);

// ─────────────────────────────────────────────────────────────────────────────
// Stream Metadata
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata for a single stream within a segment.
///
/// This information is stored in the segment's **stream directory** and allows
/// readers to locate and interpret stream data without scanning the entire file.
/// Each stream contains Arrow IPC data for a specific `(slot, schema)` pairing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamMetadata {
    /// Unique identifier for this stream within the segment.
    pub id: StreamId,
    /// The slot this stream serves.
    pub slot_id: SlotId,
    /// Schema fingerprint for all batches in this stream.
    pub schema_fingerprint: SchemaFingerprint,
    /// Byte offset from segment start to the stream's Arrow IPC data.
    pub byte_offset: u64,
    /// Total byte length of the stream's Arrow IPC data.
    pub byte_length: u64,
    /// Total number of rows across all chunks in this stream.
    pub row_count: u64,
    /// Number of chunks in this stream. Each chunk corresponds to one
    /// Arrow `RecordBatch` written from a [`RecordBundle`](crate::record_bundle::RecordBundle) payload.
    pub chunk_count: u32,
}

impl StreamMetadata {
    /// Creates new stream metadata.
    #[must_use]
    pub fn new(
        id: StreamId,
        slot_id: SlotId,
        schema_fingerprint: SchemaFingerprint,
        byte_offset: u64,
        byte_length: u64,
        row_count: u64,
        chunk_count: u32,
    ) -> Self {
        Self {
            id,
            slot_id,
            schema_fingerprint,
            byte_offset,
            byte_length,
            row_count,
            chunk_count,
        }
    }

    /// Returns the stream key for this metadata.
    #[must_use]
    pub fn stream_key(&self) -> StreamKey {
        (self.slot_id, self.schema_fingerprint)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Chunk References
// ─────────────────────────────────────────────────────────────────────────────

/// Zero-based index of a chunk within a stream.
///
/// Each Arrow `RecordBatch` written to a stream becomes a chunk at a
/// specific index. When a [`RecordBundle`] payload is appended to a stream,
/// the resulting chunk index is recorded in the batch manifest so the
/// original bundle can be reconstructed during reads.
///
/// [`RecordBundle`]: crate::record_bundle::RecordBundle
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkIndex(u32);

impl ChunkIndex {
    /// Creates a new chunk index.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw numeric value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns the next sequential chunk index.
    ///
    /// Uses saturating addition to prevent overflow. In practice, chunk indices
    /// are bounded by [`MAX_CHUNKS_PER_STREAM`] which is well below u32::MAX.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl From<u32> for ChunkIndex {
    fn from(raw: u32) -> Self {
        Self::new(raw)
    }
}

impl From<ChunkIndex> for u32 {
    fn from(idx: ChunkIndex) -> Self {
        idx.0
    }
}

/// Reference to a specific chunk (Arrow `RecordBatch`) within a stream.
///
/// Used in [`ManifestEntry`] to map a slot in a bundle to the exact
/// location of its Arrow data within the segment. The combination of
/// `stream_id` and `chunk_index` uniquely identifies a `RecordBatch`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotChunkRef {
    /// The stream containing this chunk.
    pub stream_id: StreamId,
    /// Index of the chunk within the stream.
    pub chunk_index: ChunkIndex,
}

impl SlotChunkRef {
    /// Creates a new slot chunk reference.
    #[must_use]
    pub const fn new(stream_id: StreamId, chunk_index: ChunkIndex) -> Self {
        Self {
            stream_id,
            chunk_index,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Batch Manifest
// ─────────────────────────────────────────────────────────────────────────────

/// Entry in the **batch manifest** representing a single ingested
/// [`RecordBundle`](crate::record_bundle::RecordBundle).
///
/// Each entry maps the bundle's populated slots to their corresponding
/// stream and chunk locations, enabling reconstruction of the original
/// bundle from the segment's Arrow IPC streams. Slots that were not
/// populated in the original bundle have no entry in the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestEntry {
    /// Zero-based index of this bundle within the segment.
    pub bundle_index: u32,
    /// Mapping from slot to the stream/chunk containing that slot's data.
    slot_refs: HashMap<SlotId, SlotChunkRef>,
}

impl ManifestEntry {
    /// Creates a new manifest entry for the given bundle index.
    #[must_use]
    pub fn new(bundle_index: u32) -> Self {
        Self {
            bundle_index,
            slot_refs: HashMap::new(),
        }
    }

    /// Adds a slot reference to this manifest entry.
    pub fn add_slot(&mut self, slot_id: SlotId, stream_id: StreamId, chunk_index: ChunkIndex) {
        let _ = self
            .slot_refs
            .insert(slot_id, SlotChunkRef::new(stream_id, chunk_index));
    }

    /// Returns the chunk reference for a slot, if present.
    #[must_use]
    pub fn get_slot(&self, slot_id: SlotId) -> Option<&SlotChunkRef> {
        self.slot_refs.get(&slot_id)
    }

    /// Returns the number of populated slots in this bundle.
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slot_refs.len()
    }

    /// Returns true if this bundle has no populated slots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slot_refs.is_empty()
    }

    /// Returns an iterator over populated slot IDs.
    pub fn slot_ids(&self) -> impl Iterator<Item = SlotId> + '_ {
        self.slot_refs.keys().copied()
    }

    /// Returns an iterator over (slot_id, chunk_ref) pairs.
    pub fn slots(&self) -> impl Iterator<Item = (SlotId, &SlotChunkRef)> + '_ {
        self.slot_refs.iter().map(|(k, v)| (*k, v))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Segment Identification
// ─────────────────────────────────────────────────────────────────────────────

/// Monotonically increasing segment sequence number.
///
/// Assigned when a segment is finalized. Used for ordering, gap detection,
/// and as part of the segment filename.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SegmentSeq(u64);

impl SegmentSeq {
    /// Creates a new segment sequence number.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw numeric value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Returns the next sequential segment number.
    ///
    /// Uses saturating addition to prevent overflow. In practice, u64 overflow
    /// is not a realistic concern (would require 18 quintillion segments).
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Formats the sequence number for use in filenames.
    ///
    /// Returns a zero-padded 16-digit string for lexicographic ordering.
    #[must_use]
    pub fn to_filename_component(self) -> String {
        format!("{:016}", self.0)
    }
}

impl From<u64> for SegmentSeq {
    fn from(raw: u64) -> Self {
        Self::new(raw)
    }
}

impl From<SegmentSeq> for u64 {
    fn from(seq: SegmentSeq) -> Self {
        seq.0
    }
}

impl std::fmt::Display for SegmentSeq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Footer
// ─────────────────────────────────────────────────────────────────────────────

/// Segment file footer structure (version 1).
///
/// The footer contains metadata needed to locate and interpret the segment's
/// stream directory and batch manifest. Future versions may add additional
/// fields; the trailer's `footer_size` field allows readers to handle
/// variable-sized footers.
#[derive(Debug, Clone)]
pub(super) struct Footer {
    pub version: u16,
    pub stream_count: u32,
    pub bundle_count: u32,
    pub directory_offset: u64,
    pub directory_length: u32,
    pub manifest_offset: u64,
    pub manifest_length: u32,
}

impl Footer {
    /// Encodes the footer to bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = vec![0u8; FOOTER_V1_SIZE];
        let mut pos = 0;

        // Version (2 bytes)
        buf[pos..pos + 2].copy_from_slice(&self.version.to_le_bytes());
        pos += 2;

        // Stream count (4 bytes)
        buf[pos..pos + 4].copy_from_slice(&self.stream_count.to_le_bytes());
        pos += 4;

        // Bundle count (4 bytes)
        buf[pos..pos + 4].copy_from_slice(&self.bundle_count.to_le_bytes());
        pos += 4;

        // Directory offset (8 bytes)
        buf[pos..pos + 8].copy_from_slice(&self.directory_offset.to_le_bytes());
        pos += 8;

        // Directory length (4 bytes)
        buf[pos..pos + 4].copy_from_slice(&self.directory_length.to_le_bytes());
        pos += 4;

        // Manifest offset (8 bytes)
        buf[pos..pos + 8].copy_from_slice(&self.manifest_offset.to_le_bytes());
        pos += 8;

        // Manifest length (4 bytes)
        buf[pos..pos + 4].copy_from_slice(&self.manifest_length.to_le_bytes());
        // pos += 4;

        buf
    }

    /// Decodes a version 1 footer from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the version is unsupported or the buffer is too short.
    pub fn decode(buf: &[u8]) -> Result<Self, SegmentError> {
        if buf.len() < 2 {
            return Err(SegmentError::InvalidFormat {
                message: "footer too short to contain version".to_string(),
            });
        }

        let version = u16::from_le_bytes([buf[0], buf[1]]);
        if version != SEGMENT_VERSION {
            return Err(SegmentError::InvalidFormat {
                message: format!("unsupported segment version: {}", version),
            });
        }

        if buf.len() < FOOTER_V1_SIZE {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "footer too short for version 1: expected {} bytes, got {}",
                    FOOTER_V1_SIZE,
                    buf.len()
                ),
            });
        }

        let mut pos = 2; // Skip version

        // Stream count (4 bytes)
        let stream_count = u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
        pos += 4;

        // Bundle count (4 bytes)
        let bundle_count = u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
        pos += 4;

        // Directory offset (8 bytes)
        let directory_offset = u64::from_le_bytes([
            buf[pos],
            buf[pos + 1],
            buf[pos + 2],
            buf[pos + 3],
            buf[pos + 4],
            buf[pos + 5],
            buf[pos + 6],
            buf[pos + 7],
        ]);
        pos += 8;

        // Directory length (4 bytes)
        let directory_length =
            u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
        pos += 4;

        // Manifest offset (8 bytes)
        let manifest_offset = u64::from_le_bytes([
            buf[pos],
            buf[pos + 1],
            buf[pos + 2],
            buf[pos + 3],
            buf[pos + 4],
            buf[pos + 5],
            buf[pos + 6],
            buf[pos + 7],
        ]);
        pos += 8;

        // Manifest length (4 bytes)
        let manifest_length =
            u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);

        Ok(Footer {
            version,
            stream_count,
            bundle_count,
            directory_offset,
            directory_length,
            manifest_offset,
            manifest_length,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Trailer
// ─────────────────────────────────────────────────────────────────────────────

/// Fixed-size trailer at the end of every segment file.
///
/// The trailer allows readers to locate the variable-size footer regardless
/// of version. It contains the footer size, magic bytes for identification,
/// and a CRC32 checksum covering the footer and trailer.
#[derive(Debug, Clone)]
pub(super) struct Trailer {
    /// Size of the footer in bytes (not including trailer).
    pub footer_size: u32,
}

impl Trailer {
    /// Encodes the trailer to bytes (CRC placeholder at end).
    pub fn encode(&self) -> [u8; TRAILER_SIZE] {
        let mut buf = [0u8; TRAILER_SIZE];

        // Footer size (4 bytes)
        buf[0..4].copy_from_slice(&self.footer_size.to_le_bytes());

        // Magic (8 bytes)
        buf[4..12].copy_from_slice(SEGMENT_MAGIC);

        // CRC placeholder (4 bytes) - filled by caller
        // buf[12..16] remains zeroed

        buf
    }

    /// Decodes a trailer from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the magic bytes don't match.
    pub fn decode(buf: &[u8; TRAILER_SIZE]) -> Result<(Self, u32), SegmentError> {
        // Magic (8 bytes) at offset 4
        if &buf[4..12] != SEGMENT_MAGIC {
            return Err(SegmentError::InvalidFormat {
                message: "invalid segment magic bytes in trailer".to_string(),
            });
        }

        // Footer size (4 bytes)
        let footer_size = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);

        // CRC (4 bytes)
        let crc = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);

        Ok((Trailer { footer_size }, crc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // StreamId tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn stream_id_round_trip() {
        let id = StreamId::new(42);
        assert_eq!(id.raw(), 42);
        assert_eq!(u32::from(id), 42);
    }

    #[test]
    fn stream_id_from_u32() {
        let id: StreamId = 123u32.into();
        assert_eq!(id.raw(), 123);
    }

    #[test]
    fn stream_id_ordering() {
        let a = StreamId::new(1);
        let b = StreamId::new(2);
        assert!(a < b);
    }

    #[test]
    fn stream_id_equality() {
        let a = StreamId::new(5);
        let b = StreamId::new(5);
        let c = StreamId::new(6);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StreamKey tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn stream_key_equality() {
        let fp1 = [1u8; 32];
        let fp2 = [2u8; 32];

        let k1: StreamKey = (SlotId::new(0), fp1);
        let k2: StreamKey = (SlotId::new(0), fp1);
        let k3: StreamKey = (SlotId::new(0), fp2);
        let k4: StreamKey = (SlotId::new(1), fp1);

        assert_eq!(k1, k2);
        assert_ne!(k1, k3); // different fingerprint
        assert_ne!(k1, k4); // different slot
    }

    #[test]
    fn stream_key_hash_equality() {
        use std::collections::HashSet;

        let fp = [42u8; 32];
        let k1: StreamKey = (SlotId::new(0), fp);
        let k2: StreamKey = (SlotId::new(0), fp);

        let mut set = HashSet::new();
        assert!(set.insert(k1));
        assert!(set.contains(&k2));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StreamMetadata tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn stream_metadata_construction() {
        let fp = [0xABu8; 32];
        let meta = StreamMetadata::new(
            StreamId::new(0),
            SlotId::new(1),
            fp,
            1024, // offset
            4096, // length
            1000, // rows
            5,    // chunks
        );

        assert_eq!(meta.id, StreamId::new(0));
        assert_eq!(meta.slot_id, SlotId::new(1));
        assert_eq!(meta.schema_fingerprint, fp);
        assert_eq!(meta.byte_offset, 1024);
        assert_eq!(meta.byte_length, 4096);
        assert_eq!(meta.row_count, 1000);
        assert_eq!(meta.chunk_count, 5);
    }

    #[test]
    fn stream_metadata_stream_key() {
        let fp = [0xCDu8; 32];
        let meta = StreamMetadata::new(StreamId::new(0), SlotId::new(2), fp, 0, 0, 0, 0);

        let key = meta.stream_key();
        assert_eq!(key.0, SlotId::new(2));
        assert_eq!(key.1, fp);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ChunkIndex tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn chunk_index_round_trip() {
        let idx = ChunkIndex::new(7);
        assert_eq!(idx.raw(), 7);
        assert_eq!(u32::from(idx), 7);
    }

    #[test]
    fn chunk_index_next() {
        let idx = ChunkIndex::new(0);
        assert_eq!(idx.next(), ChunkIndex::new(1));
        assert_eq!(idx.next().next(), ChunkIndex::new(2));
    }

    #[test]
    fn chunk_index_ordering() {
        let a = ChunkIndex::new(0);
        let b = ChunkIndex::new(1);
        assert!(a < b);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SlotChunkRef tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn slot_chunk_ref_construction() {
        let r = SlotChunkRef::new(StreamId::new(3), ChunkIndex::new(5));
        assert_eq!(r.stream_id, StreamId::new(3));
        assert_eq!(r.chunk_index, ChunkIndex::new(5));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ManifestEntry tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn manifest_entry_empty() {
        let entry = ManifestEntry::new(0);
        assert_eq!(entry.bundle_index, 0);
        assert!(entry.is_empty());
        assert_eq!(entry.slot_count(), 0);
    }

    #[test]
    fn manifest_entry_add_and_get_slot() {
        let mut entry = ManifestEntry::new(5);
        entry.add_slot(SlotId::new(0), StreamId::new(1), ChunkIndex::new(2));
        entry.add_slot(SlotId::new(1), StreamId::new(3), ChunkIndex::new(0));

        assert!(!entry.is_empty());
        assert_eq!(entry.slot_count(), 2);

        let slot0 = entry.get_slot(SlotId::new(0)).unwrap();
        assert_eq!(slot0.stream_id, StreamId::new(1));
        assert_eq!(slot0.chunk_index, ChunkIndex::new(2));

        let slot1 = entry.get_slot(SlotId::new(1)).unwrap();
        assert_eq!(slot1.stream_id, StreamId::new(3));
        assert_eq!(slot1.chunk_index, ChunkIndex::new(0));

        assert!(entry.get_slot(SlotId::new(99)).is_none());
    }

    #[test]
    fn manifest_entry_slots_iterator() {
        let mut entry = ManifestEntry::new(0);
        entry.add_slot(SlotId::new(2), StreamId::new(0), ChunkIndex::new(0));
        entry.add_slot(SlotId::new(5), StreamId::new(1), ChunkIndex::new(0));

        let mut slot_ids: Vec<_> = entry.slot_ids().collect();
        slot_ids.sort();
        assert_eq!(slot_ids, vec![SlotId::new(2), SlotId::new(5)]);

        // Also test the slots() method that returns (SlotId, &SlotChunkRef) pairs
        let slots: Vec<_> = entry.slots().collect();
        assert_eq!(slots.len(), 2);
    }

    #[test]
    fn manifest_entry_overwrite_slot() {
        let mut entry = ManifestEntry::new(0);
        entry.add_slot(SlotId::new(0), StreamId::new(1), ChunkIndex::new(0));
        entry.add_slot(SlotId::new(0), StreamId::new(2), ChunkIndex::new(1));

        // Should overwrite, not add
        assert_eq!(entry.slot_count(), 1);
        let slot = entry.get_slot(SlotId::new(0)).unwrap();
        assert_eq!(slot.stream_id, StreamId::new(2));
        assert_eq!(slot.chunk_index, ChunkIndex::new(1));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SegmentSeq tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn segment_seq_round_trip() {
        let seq = SegmentSeq::new(12345);
        assert_eq!(seq.raw(), 12345);
        assert_eq!(u64::from(seq), 12345);
    }

    #[test]
    fn segment_seq_next() {
        let seq = SegmentSeq::new(0);
        assert_eq!(seq.next(), SegmentSeq::new(1));
    }

    #[test]
    fn segment_seq_ordering() {
        let a = SegmentSeq::new(1);
        let b = SegmentSeq::new(2);
        assert!(a < b);
    }

    #[test]
    fn segment_seq_filename_component() {
        assert_eq!(
            SegmentSeq::new(0).to_filename_component(),
            "0000000000000000"
        );
        assert_eq!(
            SegmentSeq::new(1).to_filename_component(),
            "0000000000000001"
        );
        assert_eq!(
            SegmentSeq::new(123456789).to_filename_component(),
            "0000000123456789"
        );
    }

    #[test]
    fn segment_seq_display() {
        let seq = SegmentSeq::new(42);
        assert_eq!(format!("{}", seq), "42");
    }

    #[test]
    fn segment_seq_from_u64() {
        let seq: SegmentSeq = 999u64.into();
        assert_eq!(seq.raw(), 999);
    }
}
