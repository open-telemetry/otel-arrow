// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Segment file writer for Quiver.
//!
//! This module provides [`SegmentWriter`], which takes an [`OpenSegment`]
//! and writes a complete segment file to disk using streaming serialization.
//!
//! # Segment File Layout
//!
//! A segment file has the following structure:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         Stream Data Region                              │
//! │  ┌──────────────────────────────────────────────────────────────────┐   │
//! │  │ Stream 0: Arrow IPC File bytes                                   │   │
//! │  ├──────────────────────────────────────────────────────────────────┤   │
//! │  │ Stream 1: Arrow IPC File bytes                                   │   │
//! │  ├──────────────────────────────────────────────────────────────────┤   │
//! │  │ ...                                                              │   │
//! │  └──────────────────────────────────────────────────────────────────┘   │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                         Stream Directory                                │
//! │  Encoded as Arrow IPC (self-describing schema)                          │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                         Batch Manifest                                  │
//! │  Encoded as Arrow IPC (self-describing schema)                          │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                         Footer (variable size, version-dependent)       │
//! │  - Version: u16                                                         │
//! │  - Stream count: u32                                                    │
//! │  - Bundle count: u32                                                    │
//! │  - Directory offset: u64                                                │
//! │  - Directory length: u32                                                │
//! │  - Manifest offset: u64                                                 │
//! │  - Manifest length: u32                                                 │
//! │  - (Future versions may add more fields here)                           │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                         Trailer (fixed 16 bytes)                        │
//! │  - Footer size: u32 (size of footer, not including trailer)             │
//! │  - Magic: b"QUIVER\0S" (8 bytes)                                        │
//! │  - CRC32: u32 (covers footer + trailer except CRC itself)               │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! The trailer is always at the end of the file with a fixed size, allowing
//! readers to locate the variable-size footer regardless of version. This
//! enables future versions to add fields to the footer without breaking
//! backwards compatibility.
//!
//! # Usage
//!
//! The preferred API uses [`write_segment`](SegmentWriter::write_segment), which
//! streams IPC data directly to disk without buffering all serialized data in
//! memory:
//!
//! ```ignore
//! use quiver::segment::{OpenSegment, SegmentWriter, SegmentSeq};
//!
//! // Build an open segment
//! let mut open_segment = OpenSegment::new();
//! open_segment.append(&bundle)?;
//!
//! // Write directly to disk (streaming serialization)
//! let writer = SegmentWriter::new(SegmentSeq::new(1));
//! let (bytes_written, checksum) = writer.write_segment_sync(&path, open_segment)?;
//! ```
//!
//! [`OpenSegment`]: super::OpenSegment

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_array::builder::{
    FixedSizeBinaryBuilder, ListBuilder, StructBuilder, UInt16Builder, UInt32Builder, UInt64Builder,
};
use arrow_ipc::writer::FileWriter;
use arrow_schema::{DataType, Field, Fields, Schema};
use crc32fast::Hasher;

use super::error::SegmentError;
use super::types::{
    ChunkIndex, Footer, MAX_BUNDLES_PER_SEGMENT, MAX_STREAMS_PER_SEGMENT, ManifestEntry,
    SEGMENT_VERSION, SegmentSeq, StreamId, StreamMetadata, TRAILER_SIZE, Trailer,
};
use crate::record_bundle::{ArrowPrimitive, SlotId};

/// Alignment boundary for stream data within a segment file (in bytes).
///
/// Each stream's Arrow IPC data starts at a 64-byte aligned offset. This provides:
///
/// - **Cache-line alignment**: Modern x86 and ARM CPUs use 64-byte cache lines.
///   Aligned access avoids cache-line splits and improves prefetching.
/// - **SIMD optimization**: AVX-512 operates on 64-byte vectors. Aligned data
///   enables use of aligned load/store instructions.
/// - **Memory-mapped I/O**: Aligned regions work better with page boundaries
///   and enable efficient zero-copy access via `mmap`.
///
/// This matches Arrow's recommended buffer alignment. See:
/// <https://arrow.apache.org/docs/format/Columnar.html#buffer-alignment-and-padding>
const STREAM_ALIGNMENT: u64 = 64;

// ─────────────────────────────────────────────────────────────────────────────
// SegmentWriter
// ─────────────────────────────────────────────────────────────────────────────

/// Writes segment data to a file.
///
/// Use [`write_segment`](Self::write_segment) to write an [`OpenSegment`]
/// directly to disk with streaming serialization, avoiding the need to buffer
/// all serialized IPC data in memory.
///
/// [`OpenSegment`]: super::OpenSegment
#[derive(Debug)]
pub struct SegmentWriter {
    /// Sequence number for this segment.
    segment_seq: SegmentSeq,
}

impl SegmentWriter {
    /// Creates a new segment writer.
    #[must_use]
    pub fn new(segment_seq: SegmentSeq) -> Self {
        Self { segment_seq }
    }

    /// Returns the segment sequence number.
    #[must_use]
    pub fn segment_seq(&self) -> SegmentSeq {
        self.segment_seq
    }

    /// Writes an open segment directly to disk with streaming serialization.
    ///
    /// This method streams each stream's IPC data directly to disk, avoiding
    /// Writes an open segment to disk synchronously.
    ///
    /// This is the synchronous version of [`write_segment`](Self::write_segment).
    /// Prefer the async version when running in an async context.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the segment file will be written.
    /// * `segment` - The open segment to finalize and write.
    ///
    /// # Returns
    ///
    /// A tuple of (bytes_written, crc32_checksum).
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::Io`] if file operations fail.
    /// Returns [`SegmentError::Arrow`] if IPC encoding fails.
    /// Returns [`SegmentError::EmptySegment`] if the segment has no bundles.
    pub fn write_segment_sync(
        &self,
        path: impl AsRef<Path>,
        segment: super::OpenSegment,
    ) -> Result<(u64, u32), SegmentError> {
        let path = path.as_ref();
        let (accumulators, manifest) = segment.into_parts()?;

        let file = File::create(path).map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        let mut writer = BufWriter::new(file);

        let result = self.write_streaming(&mut writer, accumulators, manifest, path)?;

        // Ensure durability: fsync the file to guarantee data is persisted to disk.
        // This is critical for crash recovery - without fsync, data may be lost
        // if the system crashes before the OS flushes its caches.
        let file = writer
            .into_inner()
            .map_err(|e| SegmentError::io(path.to_path_buf(), e.into_error()))?;
        file.sync_all()
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;

        Self::set_readonly(path)?;

        Ok(result)
    }

    /// Writes an open segment to disk.
    ///
    /// This method streams each stream's IPC data directly to disk, avoiding
    /// the need to buffer all serialized data in memory. Uses async I/O for
    /// fsync while Arrow IPC serialization runs synchronously (Arrow doesn't
    /// support async I/O).
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the segment file will be written.
    /// * `segment` - The open segment to finalize and write.
    ///
    /// # Returns
    ///
    /// A tuple of (bytes_written, crc32_checksum).
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::Io`] if file operations fail.
    /// Returns [`SegmentError::Arrow`] if IPC encoding fails.
    /// Returns [`SegmentError::EmptySegment`] if the segment has no bundles.
    pub async fn write_segment(
        &self,
        path: impl AsRef<Path>,
        segment: super::OpenSegment,
    ) -> Result<(u64, u32), SegmentError> {
        let path = path.as_ref();
        let (accumulators, manifest) = segment.into_parts()?;

        // Create file and write data (sync - Arrow IPC doesn't support async)
        let file = File::create(path).map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        let mut writer = BufWriter::new(file);

        let result = self.write_streaming(&mut writer, accumulators, manifest, path)?;

        // Extract the underlying file for async fsync
        let file = writer
            .into_inner()
            .map_err(|e| SegmentError::io(path.to_path_buf(), e.into_error()))?;

        // Convert to tokio File for async fsync
        let async_file = tokio::fs::File::from_std(file);
        async_file
            .sync_all()
            .await
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;

        Self::set_readonly(path)?;

        Ok(result)
    }

    /// Sets the file to read-only after writing to enforce immutability.
    fn set_readonly(path: &Path) -> Result<(), SegmentError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // 0o440 = r--r----- (read-only for owner and group, no access for others)
            let permissions = std::fs::Permissions::from_mode(0o440);
            std::fs::set_permissions(path, permissions)
                .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        }

        #[cfg(not(unix))]
        {
            // On non-Unix platforms, use the portable read-only flag
            let mut permissions = std::fs::metadata(path)
                .map_err(|e| SegmentError::io(path.to_path_buf(), e))?
                .permissions();
            permissions.set_readonly(true);
            std::fs::set_permissions(path, permissions)
                .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        }

        Ok(())
    }

    /// Writes segment data with streaming IPC serialization.
    ///
    /// Serializes each stream directly to the writer, avoiding the need to
    /// buffer all IPC bytes in memory before writing.
    fn write_streaming<W: Write>(
        &self,
        writer: &mut W,
        accumulators: Vec<super::StreamAccumulator>,
        manifest: Vec<ManifestEntry>,
        path: &Path,
    ) -> Result<(u64, u32), SegmentError> {
        // Validate limits before writing to prevent creating unreadable segments
        if accumulators.len() > MAX_STREAMS_PER_SEGMENT {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "segment has {} streams, exceeds limit of {}",
                    accumulators.len(),
                    MAX_STREAMS_PER_SEGMENT
                ),
            });
        }
        if manifest.len() > MAX_BUNDLES_PER_SEGMENT {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "segment has {} bundles, exceeds limit of {}",
                    manifest.len(),
                    MAX_BUNDLES_PER_SEGMENT
                ),
            });
        }

        let mut hasher = Hasher::new();
        let mut offset: u64 = 0;

        // ─────────────────────────────────────────────────────────────────────
        // 1. Write stream data region (streaming)
        // ─────────────────────────────────────────────────────────────────────
        let mut stream_metadata_list: Vec<StreamMetadata> = Vec::with_capacity(accumulators.len());

        for accumulator in accumulators {
            // Align stream start for optimal SIMD/cache access when memory-mapping.
            let padding = (STREAM_ALIGNMENT - (offset % STREAM_ALIGNMENT)) % STREAM_ALIGNMENT;
            if padding > 0 {
                let pad_bytes = vec![0u8; padding as usize];
                writer
                    .write_all(&pad_bytes)
                    .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
                hasher.update(&pad_bytes);
                offset += padding;
            }

            // Write stream directly to output, wrapped in a hashing writer
            let mut hashing_writer = HashingWriter::new(writer, &mut hasher);
            let metadata = accumulator.write_to(&mut hashing_writer, offset)?;

            offset += metadata.byte_length;
            stream_metadata_list.push(metadata);
        }

        // ─────────────────────────────────────────────────────────────────────
        // 2. Write stream directory (as Arrow IPC)
        // ─────────────────────────────────────────────────────────────────────
        let directory_offset = offset;
        let directory_bytes = self.encode_stream_directory(&stream_metadata_list)?;
        writer
            .write_all(&directory_bytes)
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        hasher.update(&directory_bytes);
        offset += directory_bytes.len() as u64;
        let directory_length = directory_bytes.len() as u32;

        // ─────────────────────────────────────────────────────────────────────
        // 3. Write batch manifest (as Arrow IPC)
        // ─────────────────────────────────────────────────────────────────────
        let manifest_offset = offset;
        let manifest_bytes = self.encode_manifest(&manifest)?;
        writer
            .write_all(&manifest_bytes)
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        hasher.update(&manifest_bytes);
        offset += manifest_bytes.len() as u64;
        let manifest_length = manifest_bytes.len() as u32;

        // ─────────────────────────────────────────────────────────────────────
        // 4. Write footer (variable size, version-dependent)
        // ─────────────────────────────────────────────────────────────────────
        let footer = Footer {
            version: SEGMENT_VERSION,
            stream_count: stream_metadata_list.len() as u32,
            bundle_count: manifest.len() as u32,
            directory_offset,
            directory_length,
            manifest_offset,
            manifest_length,
        };

        let footer_bytes = footer.encode();
        writer
            .write_all(&footer_bytes)
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        hasher.update(&footer_bytes);
        offset += footer_bytes.len() as u64;

        // ─────────────────────────────────────────────────────────────────────
        // 5. Write trailer (fixed 16 bytes)
        // ─────────────────────────────────────────────────────────────────────
        let trailer = Trailer {
            footer_size: footer_bytes.len() as u32,
        };

        let trailer_bytes = trailer.encode();
        hasher.update(&trailer_bytes[..TRAILER_SIZE - 4]);
        let crc = hasher.finalize();

        writer
            .write_all(&trailer_bytes[..TRAILER_SIZE - 4])
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        writer
            .write_all(&crc.to_le_bytes())
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        offset += TRAILER_SIZE as u64;

        writer
            .flush()
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;

        Ok((offset, crc))
    }

    /// Encodes the stream directory as Arrow IPC.
    ///
    /// Schema:
    /// - stream_id: UInt32
    /// - slot_id: UInt16
    /// - schema_fingerprint: FixedSizeBinary(32)
    /// - byte_offset: UInt64
    /// - byte_length: UInt64
    /// - row_count: UInt64
    /// - chunk_count: UInt32
    fn encode_stream_directory(&self, streams: &[StreamMetadata]) -> Result<Vec<u8>, SegmentError> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("stream_id", StreamId::arrow_data_type(), false),
            Field::new("slot_id", SlotId::arrow_data_type(), false),
            Field::new("schema_fingerprint", DataType::FixedSizeBinary(32), false),
            Field::new("byte_offset", DataType::UInt64, false),
            Field::new("byte_length", DataType::UInt64, false),
            Field::new("row_count", DataType::UInt64, false),
            Field::new("chunk_count", ChunkIndex::arrow_data_type(), false),
        ]));

        let mut stream_id_builder = UInt32Builder::with_capacity(streams.len());
        let mut slot_id_builder = UInt16Builder::with_capacity(streams.len());
        let mut fingerprint_builder = FixedSizeBinaryBuilder::with_capacity(streams.len(), 32);
        let mut byte_offset_builder = UInt64Builder::with_capacity(streams.len());
        let mut byte_length_builder = UInt64Builder::with_capacity(streams.len());
        let mut row_count_builder = UInt64Builder::with_capacity(streams.len());
        let mut chunk_count_builder = UInt32Builder::with_capacity(streams.len());

        for meta in streams {
            stream_id_builder.append_value(meta.id.raw());
            slot_id_builder.append_value(meta.slot_id.raw());
            fingerprint_builder
                .append_value(meta.schema_fingerprint)
                .map_err(|e| SegmentError::Arrow { source: e })?;
            byte_offset_builder.append_value(meta.byte_offset);
            byte_length_builder.append_value(meta.byte_length);
            row_count_builder.append_value(meta.row_count);
            chunk_count_builder.append_value(meta.chunk_count);
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(stream_id_builder.finish()),
                Arc::new(slot_id_builder.finish()),
                Arc::new(fingerprint_builder.finish()),
                Arc::new(byte_offset_builder.finish()),
                Arc::new(byte_length_builder.finish()),
                Arc::new(row_count_builder.finish()),
                Arc::new(chunk_count_builder.finish()),
            ],
        )
        .map_err(|e| SegmentError::Arrow { source: e })?;

        encode_as_ipc(&batch)
    }

    /// Encodes the batch manifest as Arrow IPC.
    ///
    /// # Schema
    ///
    /// - `bundle_index`: UInt32 — ordinal of the bundle within the segment
    /// - `slot_refs`: List<Struct<...>> — references to stream chunks
    ///   - `slot_id`: UInt16 — which payload slot this reference is for
    ///   - `stream_id`: UInt32 — index into the stream directory
    ///   - `chunk_index`: UInt32 — which Arrow RecordBatch within the stream
    ///
    /// Using Arrow's native List and Struct types enables zero-copy decoding
    /// and avoids string parsing overhead compared to the previous JSON encoding.
    ///
    /// This is decoded by [`SegmentReader::read_manifest`](super::SegmentReader).
    ///
    /// # Panics
    ///
    /// The `expect()` calls in this function cannot panic because the field
    /// builders are created with the exact same types in the same order as
    /// the indices used to access them.
    fn encode_manifest(&self, entries: &[ManifestEntry]) -> Result<Vec<u8>, SegmentError> {
        // Define the inner struct type for slot references.
        // Uses ArrowPrimitive::arrow_data_type() to ensure the Arrow schema stays
        // synchronized with the Rust primitive types.
        let slot_ref_fields = Fields::from(vec![
            Field::new("slot_id", SlotId::arrow_data_type(), false),
            Field::new("stream_id", StreamId::arrow_data_type(), false),
            Field::new("chunk_index", ChunkIndex::arrow_data_type(), false),
        ]);

        // Note: The list item field must be nullable to match what ListBuilder produces
        let schema = Arc::new(Schema::new(vec![
            Field::new("bundle_index", DataType::UInt32, false),
            Field::new(
                "slot_refs",
                DataType::List(Arc::new(Field::new_struct(
                    "item",
                    slot_ref_fields.clone(),
                    true,
                ))),
                false,
            ),
        ]));

        let mut bundle_index_builder = UInt32Builder::with_capacity(entries.len());

        // Create the list builder with a struct builder inside
        let struct_builder = StructBuilder::from_fields(
            slot_ref_fields,
            entries.iter().map(|e| e.slot_count()).sum(),
        );
        let mut slot_refs_builder = ListBuilder::new(struct_builder);

        for entry in entries {
            bundle_index_builder.append_value(entry.bundle_index);

            // Get the struct builder from the list builder
            let struct_builder = slot_refs_builder.values();

            for (slot_id, chunk_ref) in entry.slots() {
                // Append values to each field builder within the struct.
                // These expect() calls cannot fail because we created the StructBuilder
                // with exactly these field types in the same order.
                struct_builder
                    .field_builder::<UInt16Builder>(0)
                    .expect("slot_id field at index 0")
                    .append_value(slot_id.raw());
                struct_builder
                    .field_builder::<UInt32Builder>(1)
                    .expect("stream_id field at index 1")
                    .append_value(chunk_ref.stream_id.raw());
                struct_builder
                    .field_builder::<UInt32Builder>(2)
                    .expect("chunk_index field at index 2")
                    .append_value(chunk_ref.chunk_index.raw());
                struct_builder.append(true);
            }

            slot_refs_builder.append(true);
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(bundle_index_builder.finish()),
                Arc::new(slot_refs_builder.finish()),
            ],
        )
        .map_err(|e| SegmentError::Arrow { source: e })?;

        encode_as_ipc(&batch)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Encodes a RecordBatch as Arrow IPC file format bytes.
fn encode_as_ipc(batch: &RecordBatch) -> Result<Vec<u8>, SegmentError> {
    let mut buf = Vec::new();
    {
        let mut writer = FileWriter::try_new(&mut buf, batch.schema().as_ref())
            .map_err(|e| SegmentError::Arrow { source: e })?;
        writer
            .write(batch)
            .map_err(|e| SegmentError::Arrow { source: e })?;
        writer
            .finish()
            .map_err(|e| SegmentError::Arrow { source: e })?;
    }
    Ok(buf)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Types
// ─────────────────────────────────────────────────────────────────────────────

/// A writer wrapper that updates a CRC32 hasher as data is written.
struct HashingWriter<'a, W> {
    inner: &'a mut W,
    hasher: &'a mut Hasher,
}

impl<'a, W> HashingWriter<'a, W> {
    fn new(inner: &'a mut W, hasher: &'a mut Hasher) -> Self {
        Self { inner, hasher }
    }
}

impl<W: Write> Write for HashingWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::record_bundle::SlotId;
    use crate::segment::test_utils::{make_batch, test_schema};
    use crate::segment::types::{ChunkIndex, FOOTER_V1_SIZE, SEGMENT_MAGIC, StreamId};

    #[test]
    fn write_empty_segment_fails() {
        use crate::segment::{OpenSegment, SegmentError};

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.qseg");

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let open_segment = OpenSegment::new();

        // Empty segment should fail
        let result = writer.write_segment_sync(&path, open_segment);
        assert!(matches!(result, Err(SegmentError::EmptySegment)));
    }

    #[test]
    fn write_segment_with_single_stream() {
        use crate::segment::OpenSegment;
        use crate::segment::test_utils::{TestBundle, slot_descriptors, test_fingerprint};

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.qseg");

        let schema = test_schema();
        let batch1 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch2 = make_batch(&schema, &[3], &["c"]);

        // Build an open segment with two batches in the same stream
        let mut open_segment = OpenSegment::new();
        let fp = test_fingerprint();

        let bundle1 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch1);
        let bundle2 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch2);

        let _ = open_segment.append(&bundle1).unwrap();
        let _ = open_segment.append(&bundle2).unwrap();

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let (bytes_written, _crc) = writer.write_segment_sync(&path, open_segment).unwrap();

        // File should be non-empty
        assert!(bytes_written > 0);

        // Verify we can read back the trailer
        let file_bytes = fs::read(&path).unwrap();
        assert_eq!(file_bytes.len(), bytes_written as usize);

        // Check trailer magic (at offset 4 within the 16-byte trailer)
        let trailer_start = file_bytes.len() - TRAILER_SIZE;
        assert_eq!(
            &file_bytes[trailer_start + 4..trailer_start + 12],
            SEGMENT_MAGIC
        );
    }

    #[test]
    fn write_segment_with_multiple_streams() {
        use crate::segment::OpenSegment;
        use crate::segment::test_utils::{TestBundle, slot_descriptors};

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.qseg");

        let schema = test_schema();

        // Stream 0: slot 0
        let batch0 = make_batch(&schema, &[1, 2], &["a", "b"]);
        // Stream 1: slot 1
        let batch1 = make_batch(&schema, &[3, 4, 5], &["c", "d", "e"]);

        // Build an open segment with two streams
        let mut open_segment = OpenSegment::new();
        let fp0 = [0x11u8; 32];
        let fp1 = [0x22u8; 32];

        let bundle = TestBundle::new(slot_descriptors())
            .with_payload(SlotId::new(0), fp0, batch0)
            .with_payload(SlotId::new(1), fp1, batch1);

        let _ = open_segment.append(&bundle).unwrap();

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let (bytes_written, _crc) = writer.write_segment_sync(&path, open_segment).unwrap();

        // File should be non-empty
        assert!(bytes_written > 0);
    }

    #[test]
    fn write_segment_streaming_produces_readable_file() {
        use crate::segment::test_utils::{TestBundle, slot_descriptors, test_fingerprint};
        use crate::segment::{OpenSegment, SegmentReader};

        let dir = tempdir().unwrap();
        let path = dir.path().join("streaming.qseg");

        // Build an open segment with data
        let schema = test_schema();
        let descriptors = slot_descriptors();
        let batch = make_batch(&schema, &[1, 2, 3], &["a", "b", "c"]);

        let mut open_segment = OpenSegment::new();
        let bundle = TestBundle::new(descriptors).with_payload(
            SlotId::new(0),
            test_fingerprint(),
            batch.clone(),
        );
        let _ = open_segment.append(&bundle).unwrap();

        // Write using the streaming API
        let writer = SegmentWriter::new(SegmentSeq::new(42));
        let (bytes_written, crc) = writer.write_segment_sync(&path, open_segment).unwrap();

        assert!(bytes_written > 0);
        assert!(crc != 0);

        // Verify the file is readable
        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(reader.bundle_count(), 1);
        assert_eq!(reader.stream_count(), 1);

        // Read back the bundle
        let manifest = reader.manifest();
        let reconstructed = reader.read_bundle(&manifest[0]).unwrap();
        let payload = reconstructed.payload(SlotId::new(0)).unwrap();
        assert_eq!(payload.num_rows(), 3);
    }

    #[test]
    fn footer_encode_decode_roundtrip() {
        let footer = Footer {
            version: SEGMENT_VERSION,
            stream_count: 5,
            bundle_count: 100,
            directory_offset: 12345678,
            directory_length: 1024,
            manifest_offset: 12346702,
            manifest_length: 2048,
        };

        let encoded = footer.encode();
        assert_eq!(encoded.len(), FOOTER_V1_SIZE);

        let decoded = Footer::decode(&encoded).unwrap();

        assert_eq!(decoded.version, footer.version);
        assert_eq!(decoded.stream_count, footer.stream_count);
        assert_eq!(decoded.bundle_count, footer.bundle_count);
        assert_eq!(decoded.directory_offset, footer.directory_offset);
        assert_eq!(decoded.directory_length, footer.directory_length);
        assert_eq!(decoded.manifest_offset, footer.manifest_offset);
        assert_eq!(decoded.manifest_length, footer.manifest_length);
    }

    #[test]
    fn trailer_encode_decode_roundtrip() {
        let trailer = Trailer { footer_size: 34 };

        let mut encoded = trailer.encode();
        // Add a fake CRC
        let fake_crc: u32 = 0xDEADBEEF;
        encoded[12..16].copy_from_slice(&fake_crc.to_le_bytes());

        let (decoded, crc) = Trailer::decode(&encoded).unwrap();

        assert_eq!(decoded.footer_size, trailer.footer_size);
        assert_eq!(crc, fake_crc);
    }

    #[test]
    fn trailer_decode_rejects_invalid_magic() {
        let mut buf = [0u8; TRAILER_SIZE];
        buf[4..12].copy_from_slice(b"INVALID!");

        let result = Trailer::decode(&buf);
        assert!(matches!(result, Err(SegmentError::InvalidFormat { .. })));
    }

    #[test]
    fn footer_decode_rejects_unsupported_version() {
        let mut buf = vec![0u8; FOOTER_V1_SIZE];
        buf[0..2].copy_from_slice(&99u16.to_le_bytes()); // Unsupported version

        let result = Footer::decode(&buf);
        assert!(matches!(result, Err(SegmentError::InvalidFormat { .. })));
    }

    #[test]
    fn footer_decode_rejects_short_buffer() {
        let buf = vec![0u8; 10]; // Too short

        let result = Footer::decode(&buf);
        assert!(matches!(result, Err(SegmentError::InvalidFormat { .. })));
    }

    #[test]
    fn encode_stream_directory_produces_valid_ipc() {
        let writer = SegmentWriter::new(SegmentSeq::new(1));

        let streams = vec![
            StreamMetadata::new(
                StreamId::new(0),
                SlotId::new(0),
                [0x11u8; 32],
                0,
                1000,
                100,
                5,
            ),
            StreamMetadata::new(
                StreamId::new(1),
                SlotId::new(1),
                [0x22u8; 32],
                1000,
                2000,
                200,
                10,
            ),
        ];

        let ipc_bytes = writer.encode_stream_directory(&streams).unwrap();

        // Should produce non-empty IPC
        assert!(!ipc_bytes.is_empty());

        // Should be readable
        use arrow_ipc::reader::FileReader;
        use std::io::Cursor;
        let cursor = Cursor::new(&ipc_bytes);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC");
        let batches: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].num_rows(), 2);
    }

    #[test]
    fn encode_manifest_produces_valid_ipc() {
        let writer = SegmentWriter::new(SegmentSeq::new(1));

        let mut entry0 = ManifestEntry::new(0);
        entry0.add_slot(SlotId::new(0), StreamId::new(0), ChunkIndex::new(0));

        let mut entry1 = ManifestEntry::new(1);
        entry1.add_slot(SlotId::new(0), StreamId::new(0), ChunkIndex::new(1));
        entry1.add_slot(SlotId::new(1), StreamId::new(1), ChunkIndex::new(0));

        let entries = vec![entry0, entry1];

        let ipc_bytes = writer.encode_manifest(&entries).unwrap();

        // Should produce non-empty IPC
        assert!(!ipc_bytes.is_empty());

        // Should be readable
        use arrow_ipc::reader::FileReader;
        use std::io::Cursor;
        let cursor = Cursor::new(&ipc_bytes);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC");
        let batches: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].num_rows(), 2);
    }

    #[test]
    fn segment_seq_accessor_returns_correct_value() {
        let writer = SegmentWriter::new(SegmentSeq::new(42));
        assert_eq!(writer.segment_seq(), SegmentSeq::new(42));
    }

    #[test]
    fn segment_constants_have_expected_values() {
        assert_eq!(SEGMENT_MAGIC, b"QUIVER\0S");
        assert_eq!(SEGMENT_VERSION, 1);
        assert_eq!(TRAILER_SIZE, 16);
        assert_eq!(FOOTER_V1_SIZE, 34);
    }

    #[tokio::test]
    async fn async_write_segment_with_single_stream() {
        use crate::segment::OpenSegment;
        use crate::segment::test_utils::{TestBundle, slot_descriptors, test_fingerprint};

        let dir = tempdir().unwrap();
        let path = dir.path().join("async_test.qseg");

        let schema = test_schema();
        let batch1 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch2 = make_batch(&schema, &[3], &["c"]);

        // Build an open segment with two batches in the same stream
        let mut open_segment = OpenSegment::new();
        let fp = test_fingerprint();

        let bundle1 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch1);
        let bundle2 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch2);

        let _ = open_segment.append(&bundle1).unwrap();
        let _ = open_segment.append(&bundle2).unwrap();

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let (bytes_written, _crc) = writer.write_segment(&path, open_segment).await.unwrap();

        // File should be non-empty
        assert!(bytes_written > 0);

        // Verify file exists and has content
        let metadata = tokio::fs::metadata(&path).await.expect("file should exist");
        assert_eq!(metadata.len(), bytes_written);
    }

    #[tokio::test]
    async fn async_write_segment_matches_sync_output() {
        use crate::segment::OpenSegment;
        use crate::segment::test_utils::{TestBundle, slot_descriptors, test_fingerprint};

        let dir = tempdir().unwrap();
        let sync_path = dir.path().join("sync.qseg");
        let async_path = dir.path().join("async.qseg");

        let schema = test_schema();
        let batch = make_batch(&schema, &[1, 2, 3], &["a", "b", "c"]);

        let fp = test_fingerprint();
        let bundle = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch);

        // Write with sync method
        let mut open_segment_sync = OpenSegment::new();
        let _ = open_segment_sync.append(&bundle).unwrap();
        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let (sync_bytes, sync_crc) = writer
            .write_segment_sync(&sync_path, open_segment_sync)
            .unwrap();

        // Write with async method (same data)
        let batch2 = make_batch(&schema, &[1, 2, 3], &["a", "b", "c"]);
        let bundle2 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch2);
        let mut open_segment_async = OpenSegment::new();
        let _ = open_segment_async.append(&bundle2).unwrap();
        let writer2 = SegmentWriter::new(SegmentSeq::new(1));
        let (async_bytes, async_crc) = writer2
            .write_segment(&async_path, open_segment_async)
            .await
            .unwrap();

        // Both should produce the same size and CRC
        assert_eq!(sync_bytes, async_bytes);
        assert_eq!(sync_crc, async_crc);
    }
}
