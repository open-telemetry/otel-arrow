// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Segment file reader for Quiver.
//!
//! This module provides [`SegmentReader`] for reading segment files written
//! by [`SegmentWriter`](super::SegmentWriter). The primary use case is
//! reconstructing [`RecordBundle`](crate::record_bundle::RecordBundle)s from
//! their stored payload slots.
//!
//! # Zero-Copy Reading (requires `mmap` feature)
//!
//! With the `mmap` feature enabled, [`SegmentReader`] memory-maps the segment
//! file and returns [`ReconstructedBundle`]s whose Arrow `RecordBatch`es
//! reference the mapped memory directly. This avoids allocation and copying
//! for high-throughput replay scenarios.
//!
//! ```ignore
//! use quiver::segment::SegmentReader;
//!
//! // Open with memory mapping (requires `mmap` feature)
//! let reader = SegmentReader::open_mmap(&path)?;
//!
//! // Read bundles - RecordBatches reference mmap directly
//! for entry in reader.manifest() {
//!     let bundle = reader.read_bundle(entry)?;
//!     // bundle.payloads() returns HashMap<SlotId, RecordBatch>
//!     // The underlying memory stays valid as long as `bundle` exists
//! }
//! // When all bundles are dropped, the mmap is unmapped
//! ```
//!
//! # Standard Reading (no `mmap` feature)
//!
//! Without the `mmap` feature, [`SegmentReader`] reads data into allocated
//! buffers. This is suitable for testing or platforms without mmap support.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use arrow_array::{Array, RecordBatch};
use arrow_buffer::Buffer;
use arrow_ipc::convert::fb_to_schema;
use arrow_ipc::reader::{FileDecoder, read_footer_length};
use arrow_ipc::{Block, root_as_footer};
use crc32fast::Hasher;

use super::error::SegmentError;
use super::types::{ChunkIndex, ManifestEntry, StreamId, StreamMetadata};
use crate::record_bundle::SlotId;

// ─────────────────────────────────────────────────────────────────────────────
// Constants (must match writer.rs)
// ─────────────────────────────────────────────────────────────────────────────

/// Magic bytes identifying a Quiver segment file.
const SEGMENT_MAGIC: &[u8; 8] = b"QUIVER\0S";

/// Current segment file format version.
const SEGMENT_VERSION: u16 = 1;

/// Size of the fixed trailer at the end of the segment file.
const TRAILER_SIZE: usize = 16;

/// Size of the footer for version 1.
const FOOTER_V1_SIZE: usize = 34;

// ─────────────────────────────────────────────────────────────────────────────
// ReconstructedBundle
// ─────────────────────────────────────────────────────────────────────────────

/// A reconstructed bundle with Arrow `RecordBatch`es for each payload slot.
///
/// When using memory-mapped I/O, the `RecordBatch`es reference the underlying
/// mapped memory directly (zero-copy). The bundle keeps a reference to the
/// backing buffer, ensuring the memory remains valid as long as this struct
/// exists.
///
/// When all `ReconstructedBundle`s from a segment are dropped, the underlying
/// memory (or mmap) is automatically released.
#[derive(Debug, Clone)]
pub struct ReconstructedBundle {
    /// The bundle index from the manifest.
    bundle_index: u32,
    /// Payload batches by slot ID.
    payloads: HashMap<SlotId, RecordBatch>,
    /// Keeps the backing buffer alive (may be mmap or heap allocation).
    /// RecordBatches reference slices of this buffer.
    _backing: Arc<Buffer>,
}

impl ReconstructedBundle {
    /// Returns the bundle index from the manifest.
    #[must_use]
    pub fn bundle_index(&self) -> u32 {
        self.bundle_index
    }

    /// Returns the payload batches by slot ID.
    #[must_use]
    pub fn payloads(&self) -> &HashMap<SlotId, RecordBatch> {
        &self.payloads
    }

    /// Returns a specific payload by slot ID.
    #[must_use]
    pub fn payload(&self, slot_id: SlotId) -> Option<&RecordBatch> {
        self.payloads.get(&slot_id)
    }

    /// Returns the number of populated slots in this bundle.
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.payloads.len()
    }

    /// Consumes the bundle and returns the payloads map.
    #[must_use]
    pub fn into_payloads(self) -> HashMap<SlotId, RecordBatch> {
        self.payloads
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// StreamDecoder - decodes a single Arrow IPC stream from a buffer slice
// ─────────────────────────────────────────────────────────────────────────────

/// Decodes Arrow IPC batches from a buffer slice (zero-copy).
struct StreamDecoder {
    /// The buffer slice containing the IPC stream.
    buffer: Buffer,
    /// Arrow's file decoder.
    decoder: FileDecoder,
    /// Block locations within the buffer.
    batches: Vec<Block>,
}

impl StreamDecoder {
    /// Creates a decoder for an Arrow IPC stream in the given buffer.
    fn new(buffer: Buffer) -> Result<Self, SegmentError> {
        if buffer.len() < 10 {
            return Err(SegmentError::InvalidFormat {
                message: "IPC stream too short".to_string(),
            });
        }

        let trailer_start = buffer.len() - 10;
        let footer_len = read_footer_length(buffer[trailer_start..].try_into().map_err(|_| {
            SegmentError::InvalidFormat {
                message: "invalid IPC trailer".to_string(),
            }
        })?)
        .map_err(|e| SegmentError::Arrow { source: e })?;

        let footer_start = trailer_start.saturating_sub(footer_len);
        let footer = root_as_footer(&buffer[footer_start..trailer_start]).map_err(|e| {
            SegmentError::InvalidFormat {
                message: format!("invalid IPC footer: {}", e),
            }
        })?;

        let schema = footer.schema().ok_or_else(|| SegmentError::InvalidFormat {
            message: "IPC footer missing schema".to_string(),
        })?;
        let schema = fb_to_schema(schema);

        let mut decoder = FileDecoder::new(Arc::new(schema), footer.version());

        // Read dictionaries
        for block in footer.dictionaries().iter().flatten() {
            let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
            let data = buffer.slice_with_length(block.offset() as usize, block_len);
            decoder
                .read_dictionary(block, &data)
                .map_err(|e| SegmentError::Arrow { source: e })?;
        }

        let batches = footer
            .recordBatches()
            .map(|b| b.iter().copied().collect())
            .unwrap_or_default();

        Ok(Self {
            buffer,
            decoder,
            batches,
        })
    }

    /// Returns the number of batches in this stream.
    fn num_batches(&self) -> usize {
        self.batches.len()
    }

    /// Returns the batch at the given index (zero-copy from buffer).
    fn get_batch(&self, index: usize) -> Result<Option<RecordBatch>, SegmentError> {
        if index >= self.batches.len() {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "batch index {} out of bounds (stream has {} batches)",
                    index,
                    self.batches.len()
                ),
            });
        }

        let block = &self.batches[index];
        let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
        let data = self
            .buffer
            .slice_with_length(block.offset() as usize, block_len);

        self.decoder
            .read_record_batch(block, &data)
            .map_err(|e| SegmentError::Arrow { source: e })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SegmentReader
// ─────────────────────────────────────────────────────────────────────────────

/// Reads segment files written by [`SegmentWriter`](super::SegmentWriter).
///
/// Provides zero-copy access to stream data when using memory-mapped I/O.
/// The primary API is [`read_bundle`](Self::read_bundle) which reconstructs
/// a [`ReconstructedBundle`] from its manifest entry.
#[derive(Debug)]
pub struct SegmentReader {
    /// The backing buffer (may be mmap or heap allocation).
    buffer: Arc<Buffer>,
    /// Parsed footer metadata.
    footer: Footer,
    /// Stream directory (parsed on open).
    streams: Vec<StreamMetadata>,
    /// Stream lookup by ID.
    stream_by_id: HashMap<StreamId, usize>,
    /// Batch manifest (parsed on open).
    manifest: Vec<ManifestEntry>,
}

impl SegmentReader {
    /// Opens a segment file by reading it into memory.
    ///
    /// This allocates a buffer and reads the entire file. For zero-copy
    /// memory-mapped access, use [`open_mmap`](Self::open_mmap) instead
    /// (requires the `mmap` feature).
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::Io`] if file operations fail.
    /// Returns [`SegmentError::InvalidFormat`] if the file format is invalid.
    /// Returns [`SegmentError::ChecksumMismatch`] if CRC validation fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SegmentError> {
        let path = path.as_ref();
        let mut file = File::open(path).map_err(|e| SegmentError::io(path.to_path_buf(), e))?;

        let file_size = file
            .metadata()
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?
            .len();

        // Read entire file into buffer
        let mut data = vec![0u8; file_size as usize];
        file.read_exact(&mut data)
            .map_err(|e| SegmentError::io(path.to_path_buf(), e))?;

        let buffer = Buffer::from(data);
        Self::from_buffer(buffer, path)
    }

    /// Opens a segment file with memory mapping for zero-copy access.
    ///
    /// The returned `SegmentReader` and any [`ReconstructedBundle`]s read from
    /// it will reference the memory-mapped file directly. The file is unmapped
    /// when all references are dropped.
    ///
    /// # Safety
    ///
    /// Memory mapping is inherently unsafe because the underlying file could
    /// be modified or truncated while mapped, leading to undefined behavior.
    /// This function is safe to call, but the `mmap` feature must be explicitly
    /// enabled.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::Io`] if file operations fail.
    /// Returns [`SegmentError::InvalidFormat`] if the file format is invalid.
    /// Returns [`SegmentError::ChecksumMismatch`] if CRC validation fails.
    #[cfg(feature = "mmap")]
    pub fn open_mmap(path: impl AsRef<Path>) -> Result<Self, SegmentError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| SegmentError::io(path.to_path_buf(), e))?;

        // SAFETY: We require the `mmap` feature to be explicitly enabled.
        // The caller accepts responsibility for ensuring the file is not
        // modified while mapped.
        #[allow(unsafe_code)]
        let mmap = unsafe {
            memmap2::Mmap::map(&file).map_err(|e| SegmentError::io(path.to_path_buf(), e))?
        };

        // Convert mmap -> Bytes -> Buffer (all zero-copy)
        let bytes = bytes::Bytes::from_owner(mmap);
        let buffer = Buffer::from(bytes);

        Self::from_buffer(buffer, path)
    }

    /// Creates a reader from a pre-loaded buffer.
    fn from_buffer(buffer: Buffer, _path: &Path) -> Result<Self, SegmentError> {
        let file_size = buffer.len();

        // Need at least trailer size
        if file_size < TRAILER_SIZE {
            return Err(SegmentError::Truncated {
                expected: TRAILER_SIZE as u64,
                actual: file_size as u64,
            });
        }

        // 1. Read and validate trailer
        let (trailer, stored_crc) = Self::read_trailer(&buffer)?;

        // 2. Read footer
        let footer_start = file_size - TRAILER_SIZE - trailer.footer_size as usize;
        let footer = Self::read_footer(&buffer, footer_start, trailer.footer_size as usize)?;

        // 3. Validate CRC (covers entire file except last 4 bytes)
        let computed_crc = Self::compute_crc(&buffer);
        if computed_crc != stored_crc {
            return Err(SegmentError::ChecksumMismatch {
                segment_seq: None,
                expected: stored_crc,
                actual: computed_crc,
            });
        }

        // 4. Read stream directory
        let streams = Self::read_stream_directory(
            &buffer,
            footer.directory_offset as usize,
            footer.directory_length as usize,
        )?;

        // Build stream lookup
        let stream_by_id: HashMap<_, _> =
            streams.iter().enumerate().map(|(i, s)| (s.id, i)).collect();

        // 5. Read batch manifest
        let manifest = Self::read_manifest(
            &buffer,
            footer.manifest_offset as usize,
            footer.manifest_length as usize,
        )?;

        Ok(Self {
            buffer: Arc::new(buffer),
            footer,
            streams,
            stream_by_id,
            manifest,
        })
    }

    /// Returns the number of streams in this segment.
    #[must_use]
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Returns the number of bundles in this segment.
    #[must_use]
    pub fn bundle_count(&self) -> usize {
        self.manifest.len()
    }

    /// Returns the format version of this segment file.
    #[must_use]
    pub fn version(&self) -> u16 {
        self.footer.version
    }

    /// Returns the total file size in bytes.
    #[must_use]
    pub fn file_size(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the stream directory.
    #[must_use]
    pub fn streams(&self) -> &[StreamMetadata] {
        &self.streams
    }

    /// Returns the batch manifest.
    #[must_use]
    pub fn manifest(&self) -> &[ManifestEntry] {
        &self.manifest
    }

    /// Returns a reference to the underlying buffer.
    ///
    /// This is primarily useful for testing zero-copy behavior by verifying
    /// that record batch data buffers point into the mmap region.
    #[cfg(all(test, feature = "mmap"))]
    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Returns metadata for a specific stream by ID.
    #[must_use]
    pub fn stream(&self, id: StreamId) -> Option<&StreamMetadata> {
        self.stream_by_id.get(&id).map(|&i| &self.streams[i])
    }

    /// Reads a bundle from the manifest, reconstructing all payload slots.
    ///
    /// Returns a [`ReconstructedBundle`] containing zero-copy `RecordBatch`es
    /// for each populated slot. The bundle maintains a reference to the
    /// underlying buffer, keeping it alive.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::StreamNotFound`] if a referenced stream doesn't exist.
    /// Returns [`SegmentError::Arrow`] if IPC decoding fails.
    pub fn read_bundle(&self, entry: &ManifestEntry) -> Result<ReconstructedBundle, SegmentError> {
        let mut payloads = HashMap::new();

        for (slot_id, chunk_ref) in entry.slots() {
            let stream_meta =
                self.stream(chunk_ref.stream_id)
                    .ok_or_else(|| SegmentError::StreamNotFound {
                        stream_id: chunk_ref.stream_id,
                    })?;

            // Get the stream's buffer slice
            let stream_buffer = self.buffer.slice_with_length(
                stream_meta.byte_offset as usize,
                stream_meta.byte_length as usize,
            );

            // Decode the specific batch
            let decoder = StreamDecoder::new(stream_buffer)?;
            let batch = decoder
                .get_batch(chunk_ref.chunk_index.raw() as usize)?
                .ok_or_else(|| SegmentError::InvalidFormat {
                    message: format!(
                        "chunk {} in stream {:?} returned None",
                        chunk_ref.chunk_index.raw(),
                        chunk_ref.stream_id
                    ),
                })?;

            let _ = payloads.insert(slot_id, batch);
        }

        Ok(ReconstructedBundle {
            bundle_index: entry.bundle_index,
            payloads,
            _backing: Arc::clone(&self.buffer),
        })
    }

    /// Reads a specific chunk from a stream.
    ///
    /// This is a lower-level API; prefer [`read_bundle`](Self::read_bundle)
    /// for reconstructing complete bundles.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::StreamNotFound`] if the stream doesn't exist.
    /// Returns [`SegmentError::InvalidFormat`] if the chunk index is out of bounds.
    pub fn read_chunk(
        &self,
        stream_id: StreamId,
        chunk_index: ChunkIndex,
    ) -> Result<RecordBatch, SegmentError> {
        let stream_meta = self
            .stream(stream_id)
            .ok_or_else(|| SegmentError::StreamNotFound { stream_id })?;

        let stream_buffer = self.buffer.slice_with_length(
            stream_meta.byte_offset as usize,
            stream_meta.byte_length as usize,
        );

        let decoder = StreamDecoder::new(stream_buffer)?;
        decoder
            .get_batch(chunk_index.raw() as usize)?
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!(
                    "chunk {} in stream {:?} returned None",
                    chunk_index.raw(),
                    stream_id
                ),
            })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn read_trailer(buffer: &Buffer) -> Result<(Trailer, u32), SegmentError> {
        let trailer_start = buffer.len() - TRAILER_SIZE;
        let trailer_bytes: &[u8; TRAILER_SIZE] =
            buffer[trailer_start..]
                .try_into()
                .map_err(|_| SegmentError::InvalidFormat {
                    message: "trailer read failed".to_string(),
                })?;

        Trailer::decode(trailer_bytes)
    }

    fn read_footer(buffer: &Buffer, offset: usize, length: usize) -> Result<Footer, SegmentError> {
        if offset + length > buffer.len() {
            return Err(SegmentError::InvalidFormat {
                message: "footer extends beyond buffer".to_string(),
            });
        }
        Footer::decode(&buffer[offset..offset + length])
    }

    fn compute_crc(buffer: &Buffer) -> u32 {
        // CRC covers entire file except last 4 bytes (the CRC itself)
        let crc_length = buffer.len() - 4;
        let mut hasher = Hasher::new();
        hasher.update(&buffer[..crc_length]);
        hasher.finalize()
    }

    fn read_stream_directory(
        buffer: &Buffer,
        offset: usize,
        length: usize,
    ) -> Result<Vec<StreamMetadata>, SegmentError> {
        let ipc_buffer = buffer.slice_with_length(offset, length);
        let decoder = StreamDecoder::new(ipc_buffer)?;

        if decoder.num_batches() == 0 {
            return Err(SegmentError::InvalidFormat {
                message: "empty stream directory".to_string(),
            });
        }

        let batch = decoder
            .get_batch(0)?
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: "stream directory batch is None".to_string(),
            })?;

        // Parse stream directory columns
        let stream_ids = Self::get_u32_column(&batch, "stream_id")?;
        let slot_ids = Self::get_u16_column(&batch, "slot_id")?;
        let fingerprints = Self::get_fixed_binary_column(&batch, "schema_fingerprint", 32)?;
        let byte_offsets = Self::get_u64_column(&batch, "byte_offset")?;
        let byte_lengths = Self::get_u64_column(&batch, "byte_length")?;
        let row_counts = Self::get_u64_column(&batch, "row_count")?;
        let chunk_counts = Self::get_u32_column(&batch, "chunk_count")?;

        let mut streams = Vec::with_capacity(batch.num_rows());
        for i in 0..batch.num_rows() {
            let mut fingerprint = [0u8; 32];
            fingerprint.copy_from_slice(fingerprints[i]);

            streams.push(StreamMetadata::new(
                StreamId::new(stream_ids[i]),
                SlotId::new(slot_ids[i]),
                fingerprint,
                byte_offsets[i],
                byte_lengths[i],
                row_counts[i],
                chunk_counts[i],
            ));
        }

        Ok(streams)
    }

    fn read_manifest(
        buffer: &Buffer,
        offset: usize,
        length: usize,
    ) -> Result<Vec<ManifestEntry>, SegmentError> {
        let ipc_buffer = buffer.slice_with_length(offset, length);
        let decoder = StreamDecoder::new(ipc_buffer)?;

        if decoder.num_batches() == 0 {
            return Err(SegmentError::InvalidFormat {
                message: "empty manifest".to_string(),
            });
        }

        let batch = decoder
            .get_batch(0)?
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: "manifest batch is None".to_string(),
            })?;

        // Parse manifest columns
        let bundle_indices = Self::get_u32_column(&batch, "bundle_index")?;
        let slot_refs_strs = Self::get_string_column(&batch, "slot_refs")?;

        let mut entries = Vec::with_capacity(batch.num_rows());
        for i in 0..batch.num_rows() {
            let mut entry = ManifestEntry::new(bundle_indices[i]);

            // Parse slot_refs string: "slot_id:stream_id:chunk_index,..."
            let refs_str = slot_refs_strs[i];
            if !refs_str.is_empty() {
                for part in refs_str.split(',') {
                    let parts: Vec<&str> = part.split(':').collect();
                    if parts.len() == 3 {
                        if let (Ok(slot), Ok(stream), Ok(chunk)) = (
                            parts[0].parse::<u16>(),
                            parts[1].parse::<u32>(),
                            parts[2].parse::<u32>(),
                        ) {
                            entry.add_slot(
                                SlotId::new(slot),
                                StreamId::new(stream),
                                ChunkIndex::new(chunk),
                            );
                        }
                    }
                }
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    fn get_u32_column(batch: &RecordBatch, name: &str) -> Result<Vec<u32>, SegmentError> {
        use arrow_array::cast::AsArray;

        let col = batch
            .column_by_name(name)
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("missing column: {}", name),
            })?;

        let arr = col
            .as_primitive_opt::<arrow_array::types::UInt32Type>()
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("column {} is not UInt32", name),
            })?;

        Ok(arr.values().to_vec())
    }

    fn get_u16_column(batch: &RecordBatch, name: &str) -> Result<Vec<u16>, SegmentError> {
        use arrow_array::cast::AsArray;

        let col = batch
            .column_by_name(name)
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("missing column: {}", name),
            })?;

        let arr = col
            .as_primitive_opt::<arrow_array::types::UInt16Type>()
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("column {} is not UInt16", name),
            })?;

        Ok(arr.values().to_vec())
    }

    fn get_u64_column(batch: &RecordBatch, name: &str) -> Result<Vec<u64>, SegmentError> {
        use arrow_array::cast::AsArray;

        let col = batch
            .column_by_name(name)
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("missing column: {}", name),
            })?;

        let arr = col
            .as_primitive_opt::<arrow_array::types::UInt64Type>()
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("column {} is not UInt64", name),
            })?;

        Ok(arr.values().to_vec())
    }

    fn get_fixed_binary_column<'a>(
        batch: &'a RecordBatch,
        name: &str,
        expected_size: i32,
    ) -> Result<Vec<&'a [u8]>, SegmentError> {
        let col = batch
            .column_by_name(name)
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("missing column: {}", name),
            })?;

        let arr = col
            .as_any()
            .downcast_ref::<arrow_array::FixedSizeBinaryArray>()
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("column {} is not FixedSizeBinary", name),
            })?;

        if arr.value_length() != expected_size {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "column {} has size {}, expected {}",
                    name,
                    arr.value_length(),
                    expected_size
                ),
            });
        }

        Ok((0..arr.len()).map(|i| arr.value(i)).collect())
    }

    fn get_string_column<'a>(
        batch: &'a RecordBatch,
        name: &str,
    ) -> Result<Vec<&'a str>, SegmentError> {
        let col = batch
            .column_by_name(name)
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("missing column: {}", name),
            })?;

        let arr = col
            .as_any()
            .downcast_ref::<arrow_array::StringArray>()
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("column {} is not Utf8", name),
            })?;

        Ok((0..arr.len()).map(|i| arr.value(i)).collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Footer
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Footer {
    version: u16,
    #[allow(dead_code)]
    stream_count: u32,
    #[allow(dead_code)]
    bundle_count: u32,
    directory_offset: u64,
    directory_length: u32,
    manifest_offset: u64,
    manifest_length: u32,
}

impl Footer {
    fn decode(buf: &[u8]) -> Result<Self, SegmentError> {
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

        let mut pos = 2;

        let stream_count = u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
        pos += 4;

        let bundle_count = u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
        pos += 4;

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

        let directory_length =
            u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
        pos += 4;

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

#[derive(Debug, Clone)]
struct Trailer {
    footer_size: u32,
}

impl Trailer {
    fn decode(buf: &[u8; TRAILER_SIZE]) -> Result<(Self, u32), SegmentError> {
        if &buf[4..12] != SEGMENT_MAGIC {
            return Err(SegmentError::InvalidFormat {
                message: "invalid segment magic bytes in trailer".to_string(),
            });
        }

        let footer_size = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let crc = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);

        Ok((Trailer { footer_size }, crc))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;

    use arrow_array::{Int32Array, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use tempfile::tempdir;

    use super::*;
    use crate::record_bundle::{BundleDescriptor, PayloadRef, RecordBundle, SlotDescriptor};
    use crate::segment::{OpenSegment, SegmentSeq, SegmentWriter};

    fn test_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, true),
        ]))
    }

    fn make_batch(schema: &Arc<Schema>, ids: &[i32], names: &[&str]) -> RecordBatch {
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

    fn slot_descriptors() -> Vec<SlotDescriptor> {
        vec![
            SlotDescriptor::new(SlotId::new(0), "Logs"),
            SlotDescriptor::new(SlotId::new(1), "LogAttrs"),
        ]
    }

    struct TestBundle {
        descriptor: BundleDescriptor,
        payloads: HashMap<SlotId, ([u8; 32], RecordBatch)>,
    }

    impl TestBundle {
        fn new(slots: Vec<SlotDescriptor>) -> Self {
            Self {
                descriptor: BundleDescriptor::new(slots),
                payloads: HashMap::new(),
            }
        }

        fn with_payload(
            mut self,
            slot_id: SlotId,
            fingerprint: [u8; 32],
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

    fn write_test_segment(path: &Path) -> (usize, usize) {
        let schema = test_schema();
        let fp = [0x11u8; 32];

        let batch1 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch2 = make_batch(&schema, &[3, 4, 5], &["c", "d", "e"]);

        let mut open_segment = OpenSegment::new();

        let bundle1 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch1);
        let bundle2 = TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch2);

        let _ = open_segment.append(&bundle1);
        let _ = open_segment.append(&bundle2);

        let stream_count = open_segment.stream_count();
        let bundle_count = open_segment.bundle_count();

        let (streams, manifest) = open_segment.finalize().expect("finalize");

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let _ = writer
            .write_to_file(path, streams, manifest)
            .expect("write");

        (stream_count, bundle_count)
    }

    #[test]
    fn reader_opens_valid_segment() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let (expected_streams, expected_bundles) = write_test_segment(&path);

        let reader = SegmentReader::open(&path).expect("open");

        assert_eq!(reader.stream_count(), expected_streams);
        assert_eq!(reader.bundle_count(), expected_bundles);
        assert_eq!(reader.version(), 1);
    }

    #[test]
    fn reader_returns_stream_metadata() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        let reader = SegmentReader::open(&path).expect("open");
        let streams = reader.streams();

        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].slot_id, SlotId::new(0));
        assert_eq!(streams[0].chunk_count, 2);
        assert_eq!(streams[0].row_count, 5); // 2 + 3
    }

    #[test]
    fn reader_returns_manifest_entries() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        let reader = SegmentReader::open(&path).expect("open");
        let manifest = reader.manifest();

        assert_eq!(manifest.len(), 2);
        assert_eq!(manifest[0].bundle_index, 0);
        assert_eq!(manifest[1].bundle_index, 1);
    }

    #[test]
    fn reader_reads_bundle() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        let reader = SegmentReader::open(&path).expect("open");
        let entry = reader.manifest()[0].clone();

        let bundle = reader.read_bundle(&entry).expect("read_bundle");

        assert_eq!(bundle.bundle_index(), 0);
        assert_eq!(bundle.slot_count(), 1);
        assert!(bundle.payload(SlotId::new(0)).is_some());
        assert_eq!(
            bundle.payload(SlotId::new(0)).map(|b| b.num_rows()),
            Some(2)
        );
    }

    #[test]
    fn reader_reads_chunk() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        let reader = SegmentReader::open(&path).expect("open");
        let stream_id = reader.streams()[0].id;

        let batch0 = reader
            .read_chunk(stream_id, ChunkIndex::new(0))
            .expect("chunk 0");
        let batch1 = reader
            .read_chunk(stream_id, ChunkIndex::new(1))
            .expect("chunk 1");

        assert_eq!(batch0.num_rows(), 2);
        assert_eq!(batch1.num_rows(), 3);
    }

    #[test]
    fn reader_rejects_invalid_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("invalid.qseg");

        std::fs::write(&path, b"not a valid segment file").expect("write");

        let result = SegmentReader::open(&path);
        assert!(result.is_err());
    }

    #[test]
    fn reader_rejects_truncated_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("truncated.qseg");

        std::fs::write(&path, b"short").expect("write");

        let result = SegmentReader::open(&path);
        assert!(matches!(result, Err(SegmentError::Truncated { .. })));
    }

    #[test]
    fn footer_decode_rejects_buffer_too_short_for_version() {
        // Footer needs at least 2 bytes for the version field
        let buf = [0u8; 1];
        let result = Footer::decode(&buf);

        assert!(matches!(result, Err(SegmentError::InvalidFormat { message }) if message.contains("too short to contain version")));
    }

    #[test]
    fn footer_decode_rejects_unsupported_version() {
        // Create a buffer with version = 99 (unsupported)
        let mut buf = [0u8; FOOTER_V1_SIZE];
        buf[0] = 99; // version low byte
        buf[1] = 0; // version high byte

        let result = Footer::decode(&buf);

        assert!(matches!(result, Err(SegmentError::InvalidFormat { message }) if message.contains("unsupported segment version: 99")));
    }

    #[test]
    fn footer_decode_rejects_buffer_too_short_for_v1() {
        // Create a buffer with correct version (1) but too short for full v1 footer
        let mut buf = [0u8; 10]; // Less than FOOTER_V1_SIZE (34 bytes)
        buf[0] = 1; // version = 1
        buf[1] = 0;

        let result = Footer::decode(&buf);

        assert!(matches!(result, Err(SegmentError::InvalidFormat { message }) if message.contains("footer too short for version 1")));
    }

    #[test]
    fn reader_detects_checksum_mismatch() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        // Make the file writable so we can corrupt it for testing
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o644);
            std::fs::set_permissions(&path, permissions).expect("set permissions");
        }
        #[cfg(not(unix))]
        {
            let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
            permissions.set_readonly(false);
            std::fs::set_permissions(&path, permissions).expect("set permissions");
        }

        // Corrupt a byte in the footer
        let mut bytes = std::fs::read(&path).expect("read");
        let footer_pos = bytes.len() - 20; // Somewhere in footer
        bytes[footer_pos] ^= 0xFF;
        std::fs::write(&path, &bytes).expect("write");

        let result = SegmentReader::open(&path);
        assert!(matches!(result, Err(SegmentError::ChecksumMismatch { .. })));
    }

    #[test]
    fn reader_stream_lookup_by_id() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        let reader = SegmentReader::open(&path).expect("open");
        let stream_id = reader.streams()[0].id;

        let meta = reader.stream(stream_id);
        assert!(meta.is_some());
        assert_eq!(meta.map(|m| m.id), Some(stream_id));

        let missing = reader.stream(StreamId::new(999));
        assert!(missing.is_none());
    }

    #[test]
    fn reader_chunk_out_of_bounds() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        let reader = SegmentReader::open(&path).expect("open");
        let stream_id = reader.streams()[0].id;

        let result = reader.read_chunk(stream_id, ChunkIndex::new(99));
        assert!(matches!(result, Err(SegmentError::InvalidFormat { .. })));
    }

    #[test]
    fn roundtrip_multiple_slots() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("multi_slot.qseg");

        let schema = test_schema();
        let fp0 = [0x11u8; 32];
        let fp1 = [0x22u8; 32];

        let batch0 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch1 = make_batch(&schema, &[10, 20, 30], &["x", "y", "z"]);

        let mut open_segment = OpenSegment::new();

        let bundle = TestBundle::new(slot_descriptors())
            .with_payload(SlotId::new(0), fp0, batch0.clone())
            .with_payload(SlotId::new(1), fp1, batch1.clone());

        let _ = open_segment.append(&bundle);

        let (streams, manifest) = open_segment.finalize().expect("finalize");

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let _ = writer
            .write_to_file(&path, streams, manifest)
            .expect("write");

        // Read back
        let reader = SegmentReader::open(&path).expect("open");

        assert_eq!(reader.stream_count(), 2);
        assert_eq!(reader.bundle_count(), 1);

        let entry = reader.manifest()[0].clone();
        let bundle = reader.read_bundle(&entry).expect("read_bundle");

        assert_eq!(bundle.slot_count(), 2);
        assert_eq!(
            bundle.payload(SlotId::new(0)).map(|b| b.num_rows()),
            Some(2)
        );
        assert_eq!(
            bundle.payload(SlotId::new(1)).map(|b| b.num_rows()),
            Some(3)
        );
    }

    #[test]
    fn reconstructed_bundle_into_payloads() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        let reader = SegmentReader::open(&path).expect("open");
        let entry = reader.manifest()[0].clone();

        let bundle = reader.read_bundle(&entry).expect("read_bundle");
        let payloads = bundle.into_payloads();

        assert_eq!(payloads.len(), 1);
        assert!(payloads.contains_key(&SlotId::new(0)));
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn reader_opens_mmap() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let (expected_streams, expected_bundles) = write_test_segment(&path);

        let reader = SegmentReader::open_mmap(&path).expect("open_mmap");

        assert_eq!(reader.stream_count(), expected_streams);
        assert_eq!(reader.bundle_count(), expected_bundles);
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_bundle_outlives_reader() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.qseg");

        let _ = write_test_segment(&path);

        let bundle = {
            let reader = SegmentReader::open_mmap(&path).expect("open_mmap");
            let entry = reader.manifest()[0].clone();
            reader.read_bundle(&entry).expect("read_bundle")
            // reader dropped here
        };

        // Bundle should still be valid because it holds Arc<Buffer> which holds the mmap
        assert_eq!(bundle.slot_count(), 1);
        assert_eq!(
            bundle.payload(SlotId::new(0)).map(|b| b.num_rows()),
            Some(2)
        );
    }

    /// Tests mmap reading with multiple streams to verify Arrow IPC alignment
    /// works correctly when streams are concatenated in the segment file.
    ///
    /// This test also verifies that data is truly zero-copy by checking that
    /// the returned RecordBatch data buffers point into the mmap region rather
    /// than to copied memory.
    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_multi_stream_alignment() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("multi_stream_mmap.qseg");

        let schema = test_schema();
        let fp0 = [0x11u8; 32];
        let fp1 = [0x22u8; 32];
        let fp2 = [0x33u8; 32];

        // Create batches with different sizes to exercise alignment edge cases
        let batch0 = make_batch(&schema, &[1, 2, 3], &["a", "b", "c"]);
        let batch1 = make_batch(&schema, &[10, 20], &["xx", "yy"]);
        let batch2 = make_batch(&schema, &[100], &["zzzzz"]);

        let mut open_segment = OpenSegment::new();

        // Each slot gets a different fingerprint, creating 3 separate streams
        let bundle = TestBundle::new(vec![
            SlotDescriptor::new(SlotId::new(0), "Logs"),
            SlotDescriptor::new(SlotId::new(1), "LogAttrs"),
            SlotDescriptor::new(SlotId::new(2), "ScopeAttrs"),
        ])
        .with_payload(SlotId::new(0), fp0, batch0.clone())
        .with_payload(SlotId::new(1), fp1, batch1.clone())
        .with_payload(SlotId::new(2), fp2, batch2.clone());

        let _ = open_segment.append(&bundle);

        let (streams, manifest) = open_segment.finalize().expect("finalize");

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let _ = writer
            .write_to_file(&path, streams, manifest)
            .expect("write");

        // Read back with mmap
        let reader = SegmentReader::open_mmap(&path).expect("open_mmap");

        assert_eq!(reader.stream_count(), 3);
        assert_eq!(reader.bundle_count(), 1);

        let entry = reader.manifest()[0].clone();
        let bundle = reader.read_bundle(&entry).expect("read_bundle");

        assert_eq!(bundle.slot_count(), 3);

        // Verify each slot's data is correctly reconstructed from mmap
        let p0 = bundle.payload(SlotId::new(0)).expect("slot 0");
        assert_eq!(p0.num_rows(), 3);

        let p1 = bundle.payload(SlotId::new(1)).expect("slot 1");
        assert_eq!(p1.num_rows(), 2);

        let p2 = bundle.payload(SlotId::new(2)).expect("slot 2");
        assert_eq!(p2.num_rows(), 1);

        // Verify actual column data to ensure no corruption
        let col0 = p0.column(0).as_any().downcast_ref::<Int32Array>().unwrap();
        assert_eq!(col0.values(), &[1, 2, 3]);

        let col1 = p1.column(0).as_any().downcast_ref::<Int32Array>().unwrap();
        assert_eq!(col1.values(), &[10, 20]);

        let col2 = p2.column(0).as_any().downcast_ref::<Int32Array>().unwrap();
        assert_eq!(col2.values(), &[100]);

        // Verify zero-copy: check that array data buffers point into the mmap region.
        // Get the mmap base address and length
        let mmap_ptr = reader.buffer().as_ptr() as usize;
        let mmap_end = mmap_ptr + reader.buffer().len();

        // Check that each column's data buffer is within the mmap region
        for slot_id in [0, 1, 2] {
            let payload = bundle.payload(SlotId::new(slot_id)).unwrap();
            let col = payload.column(0);
            let data = col.to_data();
            for buffer in data.buffers() {
                let buf_ptr = buffer.as_ptr() as usize;
                let buf_end = buf_ptr + buffer.len();
                // Buffer should be within mmap region (zero-copy)
                assert!(
                    buf_ptr >= mmap_ptr && buf_end <= mmap_end,
                    "slot {} buffer at {:x}..{:x} is outside mmap {:x}..{:x} - data was copied!",
                    slot_id,
                    buf_ptr,
                    buf_end,
                    mmap_ptr,
                    mmap_end
                );
            }
        }
    }
}
