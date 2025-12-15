// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Segment file writer for Quiver.
//!
//! This module provides [`SegmentWriter`], which takes the output from
//! [`OpenSegment::finalize`] and writes a complete segment file to disk.
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
//! ```ignore
//! use quiver::segment::{OpenSegment, SegmentWriter};
//!
//! // Finalize an open segment
//! let (streams, manifest) = open_segment.finalize()?;
//!
//! // Write to disk
//! let writer = SegmentWriter::new(segment_seq);
//! let (bytes_written, checksum) = writer.write_to_file(&path, streams, manifest)?;
//! ```
//!
//! [`OpenSegment::finalize`]: super::OpenSegment::finalize

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_array::builder::{
    FixedSizeBinaryBuilder, StringBuilder, UInt16Builder, UInt32Builder, UInt64Builder,
};
use arrow_ipc::writer::FileWriter;
use arrow_schema::{DataType, Field, Schema};
use crc32fast::Hasher;

use super::error::SegmentError;
use super::types::{
    Footer, ManifestEntry, SEGMENT_VERSION, SegmentSeq, StreamMetadata, TRAILER_SIZE, Trailer,
    MAX_BUNDLES_PER_SEGMENT, MAX_SLOTS_PER_BUNDLE, MAX_STREAMS_PER_SEGMENT,
};

// ─────────────────────────────────────────────────────────────────────────────
// SegmentWriter
// ─────────────────────────────────────────────────────────────────────────────

/// Writes finalized segment data to a file.
///
/// Takes the output of [`OpenSegment::finalize`](super::OpenSegment::finalize)
/// and produces a complete segment file with stream data, directory, manifest,
/// and footer.
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

    /// Writes a complete segment file to disk.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the segment file will be written
    /// * `streams` - Stream data and metadata from `OpenSegment::finalize`
    /// * `manifest` - Batch manifest entries from `OpenSegment::finalize`
    ///
    /// # Returns
    ///
    /// A tuple of (bytes_written, crc32_checksum).
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::Io`] if file operations fail.
    /// Returns [`SegmentError::Arrow`] if IPC encoding fails.
    pub fn write_to_file(
        &self,
        path: impl AsRef<Path>,
        streams: Vec<(Vec<u8>, StreamMetadata)>,
        manifest: Vec<ManifestEntry>,
    ) -> Result<(u64, u32), SegmentError> {
        let path = path.as_ref();
        let file = File::create(path).map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        let mut writer = BufWriter::new(file);

        let result = self.write_to(&mut writer, streams, manifest, path)?;

        // Set the file to read-only after writing to enforce immutability.
        // This is a defense-in-depth measure; segment files should never be
        // modified after finalization.
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

        Ok(result)
    }

    /// Writes segment data to any `Write` implementation.
    ///
    /// This is the core write logic, separated from file handling for testability.
    fn write_to<W: Write>(
        &self,
        writer: &mut W,
        streams: Vec<(Vec<u8>, StreamMetadata)>,
        manifest: Vec<ManifestEntry>,
        path: &Path,
    ) -> Result<(u64, u32), SegmentError> {
        // Validate limits before writing to prevent creating unreadable segments
        if streams.len() > MAX_STREAMS_PER_SEGMENT {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "segment has {} streams, exceeds limit of {}",
                    streams.len(),
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
        // 1. Write stream data region
        // ─────────────────────────────────────────────────────────────────────
        let mut stream_metadata_list: Vec<StreamMetadata> = Vec::with_capacity(streams.len());

        for (ipc_bytes, mut metadata) in streams {
            // Align stream start to 8-byte boundary for zero-copy mmap reads.
            // Arrow IPC internally uses 8-byte alignment for data buffers,
            // but those offsets are relative to the IPC file start. If the IPC
            // file itself starts at an unaligned offset within the mmap region,
            // Arrow must copy the data to achieve alignment. Padding here
            // ensures each stream starts aligned, preserving zero-copy behavior.
            let padding = (8 - (offset % 8)) % 8;
            if padding > 0 {
                let pad_bytes = vec![0u8; padding as usize];
                writer
                    .write_all(&pad_bytes)
                    .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
                hasher.update(&pad_bytes);
                offset += padding;
            }

            // Update metadata with actual (aligned) offset
            metadata.byte_offset = offset;
            metadata.byte_length = ipc_bytes.len() as u64;

            // Write stream data
            writer
                .write_all(&ipc_bytes)
                .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
            hasher.update(&ipc_bytes);
            offset += ipc_bytes.len() as u64;

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
        // CRC covers entire file (streams, directory, manifest, footer, trailer)
        // except the final 4-byte CRC field itself
        hasher.update(&trailer_bytes[..TRAILER_SIZE - 4]);
        let crc = hasher.finalize();

        // Write trailer with CRC
        writer
            .write_all(&trailer_bytes[..TRAILER_SIZE - 4])
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        writer
            .write_all(&crc.to_le_bytes())
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;
        offset += TRAILER_SIZE as u64;

        // Flush to ensure data is written
        writer
            .flush()
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;

        Ok((offset, crc))
    }

    /// Encodes the stream directory as Arrow IPC.
    ///
    /// Schema:
    /// - stream_id: UInt32
    /// - slot_id: UInt8
    /// - schema_fingerprint: FixedSizeBinary(32)
    /// - byte_offset: UInt64
    /// - byte_length: UInt64
    /// - row_count: UInt64
    /// - chunk_count: UInt32
    fn encode_stream_directory(&self, streams: &[StreamMetadata]) -> Result<Vec<u8>, SegmentError> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("stream_id", DataType::UInt32, false),
            Field::new("slot_id", DataType::UInt16, false),
            Field::new("schema_fingerprint", DataType::FixedSizeBinary(32), false),
            Field::new("byte_offset", DataType::UInt64, false),
            Field::new("byte_length", DataType::UInt64, false),
            Field::new("row_count", DataType::UInt64, false),
            Field::new("chunk_count", DataType::UInt32, false),
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
    /// Schema:
    /// - bundle_index: UInt32
    /// - slot_bitmap: UInt64 (which slots are populated)
    /// - slot_refs: UTF-8 (JSON-encoded sparse map of slot_id -> {stream_id, chunk_index})
    ///
    /// We use a JSON string for slot_refs to handle the sparse nature of bundles
    /// without requiring a fixed-width schema for all possible slots.
    fn encode_manifest(&self, entries: &[ManifestEntry]) -> Result<Vec<u8>, SegmentError> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("bundle_index", DataType::UInt32, false),
            Field::new("slot_bitmap", DataType::UInt64, false),
            Field::new("slot_refs", DataType::Utf8, false),
        ]));

        let mut bundle_index_builder = UInt32Builder::with_capacity(entries.len());
        let mut slot_bitmap_builder = UInt64Builder::with_capacity(entries.len());
        let mut slot_refs_builder = StringBuilder::with_capacity(entries.len(), entries.len() * 64);

        for entry in entries {
            bundle_index_builder.append_value(entry.bundle_index);

            // Build slot bitmap and refs JSON
            let mut bitmap: u64 = 0;
            let mut refs: Vec<String> = Vec::new();

            for (slot_id, chunk_ref) in entry.slots() {
                let slot_raw = slot_id.raw() as usize;
                if slot_raw < MAX_SLOTS_PER_BUNDLE {
                    bitmap |= 1u64 << slot_raw;
                }
                // Format: "slot_id:stream_id:chunk_index"
                refs.push(format!(
                    "{}:{}:{}",
                    slot_id.raw(),
                    chunk_ref.stream_id.raw(),
                    chunk_ref.chunk_index.raw()
                ));
            }

            slot_bitmap_builder.append_value(bitmap);
            slot_refs_builder.append_value(refs.join(","));
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(bundle_index_builder.finish()),
                Arc::new(slot_bitmap_builder.finish()),
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
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::fs;

    use arrow_array::RecordBatch;
    use arrow_schema::SchemaRef;
    use tempfile::tempdir;

    use super::*;
    use crate::record_bundle::SlotId;
    use crate::segment::test_utils::{make_batch, test_schema};
    use crate::segment::types::{ChunkIndex, FOOTER_V1_SIZE, SEGMENT_MAGIC, StreamId};

    fn make_stream_ipc(schema: &SchemaRef, batches: &[RecordBatch]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut writer = FileWriter::try_new(&mut buf, schema.as_ref()).expect("create writer");
            for batch in batches {
                writer.write(batch).expect("write batch");
            }
            writer.finish().expect("finish");
        }
        buf
    }

    #[test]
    fn write_empty_segment_creates_valid_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.qseg");

        let writer = SegmentWriter::new(SegmentSeq::new(1));

        // Empty streams and manifest
        let streams: Vec<(Vec<u8>, StreamMetadata)> = vec![];
        let manifest: Vec<ManifestEntry> = vec![];

        let (bytes_written, crc) = writer.write_to_file(&path, streams, manifest).unwrap();

        // Should have written just the empty directory, manifest, footer, and trailer
        assert!(bytes_written >= (FOOTER_V1_SIZE + TRAILER_SIZE) as u64);
        assert!(crc != 0);

        // File should exist
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.len(), bytes_written);
    }

    #[test]
    fn write_segment_with_single_stream() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.qseg");

        let schema = test_schema();
        let batch1 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch2 = make_batch(&schema, &[3], &["c"]);
        let ipc_bytes = make_stream_ipc(&schema, &[batch1, batch2]);

        let stream_meta = StreamMetadata::new(
            StreamId::new(0),
            SlotId::new(0),
            [0x11u8; 32],
            0, // Will be updated by writer
            ipc_bytes.len() as u64,
            3, // row_count
            2, // chunk_count
        );

        let mut manifest_entry = ManifestEntry::new(0);
        manifest_entry.add_slot(SlotId::new(0), StreamId::new(0), ChunkIndex::new(0));

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let streams = vec![(ipc_bytes.clone(), stream_meta)];
        let manifest = vec![manifest_entry];

        let (bytes_written, _crc) = writer.write_to_file(&path, streams, manifest).unwrap();

        // File should be larger than just the IPC bytes
        assert!(bytes_written > ipc_bytes.len() as u64);

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
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.qseg");

        let schema = test_schema();

        // Stream 0: slot 0
        let batch0 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let ipc_bytes0 = make_stream_ipc(&schema, &[batch0]);
        let stream_meta0 = StreamMetadata::new(
            StreamId::new(0),
            SlotId::new(0),
            [0x11u8; 32],
            0,
            ipc_bytes0.len() as u64,
            2,
            1,
        );

        // Stream 1: slot 1
        let batch1 = make_batch(&schema, &[3, 4, 5], &["c", "d", "e"]);
        let ipc_bytes1 = make_stream_ipc(&schema, &[batch1]);
        let stream_meta1 = StreamMetadata::new(
            StreamId::new(1),
            SlotId::new(1),
            [0x22u8; 32],
            0, // Will be updated
            ipc_bytes1.len() as u64,
            3,
            1,
        );

        let mut manifest_entry = ManifestEntry::new(0);
        manifest_entry.add_slot(SlotId::new(0), StreamId::new(0), ChunkIndex::new(0));
        manifest_entry.add_slot(SlotId::new(1), StreamId::new(1), ChunkIndex::new(0));

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let streams = vec![
            (ipc_bytes0.clone(), stream_meta0),
            (ipc_bytes1.clone(), stream_meta1),
        ];
        let manifest = vec![manifest_entry];

        let (bytes_written, _crc) = writer.write_to_file(&path, streams, manifest).unwrap();

        // File should contain both streams plus metadata
        let total_stream_size = ipc_bytes0.len() + ipc_bytes1.len();
        assert!(bytes_written > total_stream_size as u64);
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
}
