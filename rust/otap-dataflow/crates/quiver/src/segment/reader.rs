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
use super::types::{
    ChunkIndex, Footer, MAX_BUNDLES_PER_SEGMENT, MAX_CHUNKS_PER_STREAM,
    MAX_DICTIONARIES_PER_STREAM, MAX_SLOTS_PER_BUNDLE, ManifestEntry, StreamId, StreamMetadata,
    TRAILER_SIZE, Trailer,
};
use crate::record_bundle::SlotId;

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

        let footer_start =
            trailer_start
                .checked_sub(footer_len)
                .ok_or_else(|| SegmentError::InvalidFormat {
                    message: format!(
                        "IPC footer length {} exceeds available space {}",
                        footer_len, trailer_start
                    ),
                })?;
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

        // Read dictionaries with bounds checking
        let dictionaries: Vec<_> = footer.dictionaries().iter().flatten().collect();
        if dictionaries.len() > MAX_DICTIONARIES_PER_STREAM {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "IPC stream has {} dictionaries, exceeds limit of {}",
                    dictionaries.len(),
                    MAX_DICTIONARIES_PER_STREAM
                ),
            });
        }

        for block in dictionaries {
            let block_offset = block.offset() as usize;
            let block_len = (block.bodyLength() as usize)
                .checked_add(block.metaDataLength() as usize)
                .ok_or_else(|| SegmentError::InvalidFormat {
                    message: "dictionary block length overflow".to_string(),
                })?;
            let block_end =
                block_offset
                    .checked_add(block_len)
                    .ok_or_else(|| SegmentError::InvalidFormat {
                        message: "dictionary block offset+length overflow".to_string(),
                    })?;
            if block_end > buffer.len() {
                return Err(SegmentError::InvalidFormat {
                    message: format!(
                        "dictionary block extends beyond buffer: offset={}, len={}, buffer_len={}",
                        block_offset,
                        block_len,
                        buffer.len()
                    ),
                });
            }
            let data = buffer.slice_with_length(block_offset, block_len);
            decoder
                .read_dictionary(block, &data)
                .map_err(|e| SegmentError::Arrow { source: e })?;
        }

        let batches: Vec<Block> = footer
            .recordBatches()
            .map(|b| b.iter().copied().collect())
            .unwrap_or_default();

        if batches.len() > MAX_CHUNKS_PER_STREAM {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "IPC stream has {} batches, exceeds limit of {}",
                    batches.len(),
                    MAX_CHUNKS_PER_STREAM
                ),
            });
        }

        // Eagerly validate all batch block offsets and lengths
        for (i, block) in batches.iter().enumerate() {
            let block_offset = block.offset() as usize;
            let block_len = (block.bodyLength() as usize)
                .checked_add(block.metaDataLength() as usize)
                .ok_or_else(|| SegmentError::InvalidFormat {
                    message: format!("batch {} block length overflow", i),
                })?;
            let block_end =
                block_offset
                    .checked_add(block_len)
                    .ok_or_else(|| SegmentError::InvalidFormat {
                        message: format!("batch {} block offset+length overflow", i),
                    })?;
            if block_end > buffer.len() {
                return Err(SegmentError::InvalidFormat {
                    message: format!(
                        "batch {} block extends beyond buffer: offset={}, len={}, buffer_len={}",
                        i,
                        block_offset,
                        block_len,
                        buffer.len()
                    ),
                });
            }
        }

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
        // Note: block bounds were validated in new(), but we still use checked_add
        // for defense-in-depth and to avoid issues if blocks are modified.
        let block_len = (block.bodyLength() as usize)
            .checked_add(block.metaDataLength() as usize)
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("batch {} block length overflow", index),
            })?;
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
        Self::from_buffer(buffer)
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

        Self::from_buffer(buffer)
    }

    /// Creates a reader from a pre-loaded buffer.
    fn from_buffer(buffer: Buffer) -> Result<Self, SegmentError> {
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

        // 2. Validate footer_size doesn't cause underflow
        let footer_size = trailer.footer_size as usize;
        let available_for_footer = file_size.saturating_sub(TRAILER_SIZE);
        if footer_size > available_for_footer {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "footer_size {} exceeds available space {} (file_size={}, trailer={})",
                    footer_size, available_for_footer, file_size, TRAILER_SIZE
                ),
            });
        }
        let footer_start = file_size - TRAILER_SIZE - footer_size;
        let footer = Self::read_footer(&buffer, footer_start, footer_size)?;

        // 3. Validate CRC (covers entire file except last 4 bytes)
        let computed_crc = Self::compute_crc(&buffer);
        if computed_crc != stored_crc {
            return Err(SegmentError::ChecksumMismatch {
                segment_seq: None,
                expected: stored_crc,
                actual: computed_crc,
            });
        }

        // 4. Validate directory and manifest offsets before reading
        Self::validate_region(
            file_size,
            footer.directory_offset,
            footer.directory_length as u64,
            "stream directory",
        )?;
        Self::validate_region(
            file_size,
            footer.manifest_offset,
            footer.manifest_length as u64,
            "batch manifest",
        )?;

        // 5. Read stream directory
        let streams = Self::read_stream_directory(
            &buffer,
            footer.directory_offset as usize,
            footer.directory_length as usize,
        )?;

        // Build stream lookup
        let stream_by_id: HashMap<_, _> =
            streams.iter().enumerate().map(|(i, s)| (s.id, i)).collect();

        // 6. Read batch manifest
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

            // Validate stream region before slicing
            Self::validate_region(
                self.buffer.len(),
                stream_meta.byte_offset,
                stream_meta.byte_length,
                "stream data",
            )?;

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

        // Validate stream region before slicing
        Self::validate_region(
            self.buffer.len(),
            stream_meta.byte_offset,
            stream_meta.byte_length,
            "stream data",
        )?;

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

    /// Validates that a region (offset, length) fits within the buffer.
    fn validate_region(
        buffer_len: usize,
        offset: u64,
        length: u64,
        region_name: &str,
    ) -> Result<(), SegmentError> {
        let end = offset
            .checked_add(length)
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!(
                    "{} offset+length overflow: offset={}, length={}",
                    region_name, offset, length
                ),
            })?;
        if end > buffer_len as u64 {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "{} extends beyond buffer: offset={}, length={}, end={}, buffer_len={}",
                    region_name, offset, length, end, buffer_len
                ),
            });
        }
        Ok(())
    }

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
        Self::validate_region(buffer.len(), offset as u64, length as u64, "footer")?;
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
        let stream_ids =
            Self::get_primitive_column::<arrow_array::types::UInt32Type>(&batch, "stream_id")?;
        let slot_ids =
            Self::get_primitive_column::<arrow_array::types::UInt16Type>(&batch, "slot_id")?;
        let fingerprints = Self::get_fixed_binary_column(&batch, "schema_fingerprint", 32)?;
        let byte_offsets =
            Self::get_primitive_column::<arrow_array::types::UInt64Type>(&batch, "byte_offset")?;
        let byte_lengths =
            Self::get_primitive_column::<arrow_array::types::UInt64Type>(&batch, "byte_length")?;
        let row_counts =
            Self::get_primitive_column::<arrow_array::types::UInt64Type>(&batch, "row_count")?;
        let chunk_counts =
            Self::get_primitive_column::<arrow_array::types::UInt32Type>(&batch, "chunk_count")?;

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
        let bundle_indices =
            Self::get_primitive_column::<arrow_array::types::UInt32Type>(&batch, "bundle_index")?;
        let slot_refs_strs = Self::get_string_column(&batch, "slot_refs")?;

        // Derived limit for slot_refs string length.
        // Each slot ref is "slot:stream:chunk" which is at most ~30 chars, plus comma.
        // With MAX_SLOTS_PER_BUNDLE slots, this gives a reasonable upper bound.
        const MAX_SLOT_REFS_STRING_LEN: usize = MAX_SLOTS_PER_BUNDLE * 32;

        if batch.num_rows() > MAX_BUNDLES_PER_SEGMENT {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "manifest has {} bundles, exceeds limit of {}",
                    batch.num_rows(),
                    MAX_BUNDLES_PER_SEGMENT
                ),
            });
        }

        let mut entries = Vec::with_capacity(batch.num_rows());
        for i in 0..batch.num_rows() {
            let mut entry = ManifestEntry::new(bundle_indices[i]);

            // Parse slot_refs string: "slot_id:stream_id:chunk_index,..."
            let refs_str = slot_refs_strs[i];

            // Guard against extremely long slot_refs strings
            if refs_str.len() > MAX_SLOT_REFS_STRING_LEN {
                return Err(SegmentError::InvalidFormat {
                    message: format!(
                        "slot_refs string length {} exceeds limit of {} in bundle {}",
                        refs_str.len(),
                        MAX_SLOT_REFS_STRING_LEN,
                        i
                    ),
                });
            }

            if !refs_str.is_empty() {
                let mut slot_count = 0;
                for part in refs_str.split(',') {
                    slot_count += 1;
                    if slot_count > MAX_SLOTS_PER_BUNDLE {
                        return Err(SegmentError::InvalidFormat {
                            message: format!(
                                "bundle {} has more than {} slots",
                                i, MAX_SLOTS_PER_BUNDLE
                            ),
                        });
                    }

                    let parts: Vec<&str> = part.split(':').collect();
                    if parts.len() != 3 {
                        return Err(SegmentError::InvalidFormat {
                            message: format!(
                                "invalid slot_ref format in bundle {}: expected 3 parts, got {}",
                                i,
                                parts.len()
                            ),
                        });
                    }

                    let slot =
                        parts[0]
                            .parse::<u16>()
                            .map_err(|_| SegmentError::InvalidFormat {
                                message: format!("invalid slot_id in bundle {}: {:?}", i, parts[0]),
                            })?;
                    let stream =
                        parts[1]
                            .parse::<u32>()
                            .map_err(|_| SegmentError::InvalidFormat {
                                message: format!(
                                    "invalid stream_id in bundle {}: {:?}",
                                    i, parts[1]
                                ),
                            })?;
                    let chunk =
                        parts[2]
                            .parse::<u32>()
                            .map_err(|_| SegmentError::InvalidFormat {
                                message: format!(
                                    "invalid chunk_index in bundle {}: {:?}",
                                    i, parts[2]
                                ),
                            })?;

                    entry.add_slot(
                        SlotId::new(slot),
                        StreamId::new(stream),
                        ChunkIndex::new(chunk),
                    );
                }
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Extracts a primitive column from a RecordBatch as a Vec.
    fn get_primitive_column<T>(
        batch: &RecordBatch,
        name: &str,
    ) -> Result<Vec<T::Native>, SegmentError>
    where
        T: arrow_array::types::ArrowPrimitiveType,
    {
        use arrow_array::cast::AsArray;

        let col = batch
            .column_by_name(name)
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!("missing column: {}", name),
            })?;

        let arr = col
            .as_primitive_opt::<T>()
            .ok_or_else(|| SegmentError::InvalidFormat {
                message: format!(
                    "column {} has type {:?}, expected {:?}",
                    name,
                    col.data_type(),
                    T::DATA_TYPE
                ),
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
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{DictionaryArray, Int32Array};
    use arrow_schema::{DataType, Field, Schema};
    use tempfile::tempdir;

    use super::*;
    use crate::record_bundle::SlotDescriptor;
    use crate::segment::test_utils::{TestBundle, make_batch, slot_descriptors, test_schema};
    use crate::segment::types::FOOTER_V1_SIZE;
    use crate::segment::{OpenSegment, SegmentSeq, SegmentWriter};

    /// Test helper that creates a segment file in a temporary directory.
    /// Returns handles to the tempdir (which must be kept alive) and segment path.
    struct TestSegment {
        /// Must be kept alive to prevent cleanup of the temp directory.
        #[allow(dead_code)]
        dir: tempfile::TempDir,
        /// Path to the segment file.
        path: std::path::PathBuf,
        /// Number of streams in the segment.
        stream_count: usize,
        /// Number of bundles in the segment.
        bundle_count: usize,
    }

    impl TestSegment {
        /// Creates a test segment with 2 bundles in a single stream.
        fn new() -> Self {
            let dir = tempdir().expect("tempdir");
            let path = dir.path().join("test.qseg");

            let schema = test_schema();
            let fp = [0x11u8; 32];

            let batch1 = make_batch(&schema, &[1, 2], &["a", "b"]);
            let batch2 = make_batch(&schema, &[3, 4, 5], &["c", "d", "e"]);

            let mut open_segment = OpenSegment::new();

            let bundle1 =
                TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch1);
            let bundle2 =
                TestBundle::new(slot_descriptors()).with_payload(SlotId::new(0), fp, batch2);

            let _ = open_segment.append(&bundle1);
            let _ = open_segment.append(&bundle2);

            let stream_count = open_segment.stream_count();
            let bundle_count = open_segment.bundle_count();

            let writer = SegmentWriter::new(SegmentSeq::new(1));
            let _ = writer.write_segment(&path, open_segment).expect("write");

            Self {
                dir,
                path,
                stream_count,
                bundle_count,
            }
        }
    }

    #[test]
    fn reader_opens_valid_segment() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");

        assert_eq!(reader.stream_count(), seg.stream_count);
        assert_eq!(reader.bundle_count(), seg.bundle_count);
        assert_eq!(reader.version(), 1);
    }

    #[test]
    fn reader_returns_stream_metadata() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");
        let streams = reader.streams();

        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].slot_id, SlotId::new(0));
        assert_eq!(streams[0].chunk_count, 2);
        assert_eq!(streams[0].row_count, 5); // 2 + 3
    }

    #[test]
    fn reader_returns_manifest_entries() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");
        let manifest = reader.manifest();

        assert_eq!(manifest.len(), 2);
        assert_eq!(manifest[0].bundle_index, 0);
        assert_eq!(manifest[1].bundle_index, 1);
    }

    #[test]
    fn reader_reads_bundle() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");
        let entry = reader.manifest()[0].clone();

        let bundle = reader.read_bundle(&entry).expect("read_bundle");

        assert_eq!(bundle.bundle_index(), 0);
        assert_eq!(bundle.slot_count(), 1);
        assert!(bundle.payload(SlotId::new(0)).is_some());
        assert_eq!(
            bundle.payload(SlotId::new(0)).map(|b| b.num_rows()),
            Some(2)
        );

        // Test payloads() method - returns HashMap of all payloads
        let payloads = bundle.payloads();
        assert_eq!(payloads.len(), 1);
        assert!(payloads.contains_key(&SlotId::new(0)));
    }

    #[test]
    fn reader_file_size() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");

        // file_size() should match the actual file size on disk
        let file_metadata = std::fs::metadata(&seg.path).expect("metadata");
        assert_eq!(reader.file_size(), file_metadata.len() as usize);
        assert!(reader.file_size() > 0);
    }

    #[test]
    fn reader_reads_chunk() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");
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

        assert!(
            matches!(result, Err(SegmentError::InvalidFormat { message }) if message.contains("too short to contain version"))
        );
    }

    #[test]
    fn footer_decode_rejects_unsupported_version() {
        // Create a buffer with version = 99 (unsupported)
        let mut buf = [0u8; FOOTER_V1_SIZE];
        buf[0] = 99; // version low byte
        buf[1] = 0; // version high byte

        let result = Footer::decode(&buf);

        assert!(
            matches!(result, Err(SegmentError::InvalidFormat { message }) if message.contains("unsupported segment version: 99"))
        );
    }

    #[test]
    fn footer_decode_rejects_buffer_too_short_for_v1() {
        // Create a buffer with correct version (1) but too short for full v1 footer
        let mut buf = [0u8; 10]; // Less than FOOTER_V1_SIZE (34 bytes)
        buf[0] = 1; // version = 1
        buf[1] = 0;

        let result = Footer::decode(&buf);

        assert!(
            matches!(result, Err(SegmentError::InvalidFormat { message }) if message.contains("footer too short for version 1"))
        );
    }

    #[test]
    fn stream_decoder_rejects_buffer_too_short() {
        // StreamDecoder needs at least 10 bytes for the IPC trailer
        let buf = Buffer::from_vec(vec![0u8; 5]);
        let result = StreamDecoder::new(buf);

        assert!(
            matches!(result, Err(SegmentError::InvalidFormat { message }) if message.contains("IPC stream too short"))
        );
    }

    #[test]
    fn stream_decoder_rejects_invalid_footer() {
        // Create a buffer that's long enough but has garbage data
        // The IPC trailer is the last 10 bytes, which contains the footer length
        let buf = Buffer::from_vec(vec![0xFFu8; 100]);
        let result = StreamDecoder::new(buf);

        // This will fail when trying to parse the footer (either invalid footer length
        // or invalid FlatBuffer data)
        assert!(result.is_err());
        match result {
            Err(SegmentError::InvalidFormat { message }) => {
                assert!(
                    message.contains("IPC footer") || message.contains("IPC"),
                    "expected IPC-related error, got: {}",
                    message
                );
            }
            Err(SegmentError::Arrow { .. }) => {
                // Arrow errors are also acceptable for malformed IPC data
            }
            Err(other) => panic!("unexpected error type: {:?}", other),
            Ok(_) => panic!("expected error for invalid IPC data"),
        }
    }

    #[test]
    fn roundtrip_dictionary_encoded_data() {
        // This test exercises the dictionary reading code path in StreamDecoder::new
        // (lines 174-178) by using dictionary-encoded string columns.
        use arrow_array::types::Int32Type;

        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("dict_encoded.qseg");

        // Create a schema with a dictionary-encoded string column
        let dict_field = Field::new(
            "category",
            DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
            false,
        );
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            dict_field,
        ]));

        // Create dictionary-encoded data with repeated values (good for compression)
        let categories = ["apple", "banana", "apple", "cherry", "banana"];
        let dict_array: DictionaryArray<Int32Type> = categories.into_iter().collect();

        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3, 4, 5])),
                Arc::new(dict_array),
            ],
        )
        .expect("valid batch");

        let fp = [0xDDu8; 32];
        let bundle = TestBundle::new(vec![SlotDescriptor::new(SlotId::new(0), "DictData")])
            .with_payload(SlotId::new(0), fp, batch.clone());

        let mut open_segment = OpenSegment::new();
        let _ = open_segment.append(&bundle);

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let _ = writer.write_segment(&path, open_segment).expect("write");

        // Read back and verify
        let reader = SegmentReader::open(&path).expect("open");
        assert_eq!(reader.bundle_count(), 1);

        let entry = reader.manifest()[0].clone();
        let reconstructed = reader.read_bundle(&entry).expect("read_bundle");

        let payload = reconstructed.payload(SlotId::new(0)).expect("payload");
        assert_eq!(payload.num_rows(), 5);
        assert_eq!(payload.num_columns(), 2);

        // Verify the dictionary column was correctly reconstructed
        let dict_col = payload
            .column(1)
            .as_any()
            .downcast_ref::<DictionaryArray<Int32Type>>()
            .expect("dictionary column");
        assert_eq!(dict_col.len(), 5);
    }

    #[test]
    fn reader_detects_checksum_mismatch() {
        let seg = TestSegment::new();

        // Make the file writable so we can corrupt it for testing
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o644);
            std::fs::set_permissions(&seg.path, permissions).expect("set permissions");
        }
        #[cfg(not(unix))]
        {
            let mut permissions = std::fs::metadata(&seg.path)
                .expect("metadata")
                .permissions();
            permissions.set_readonly(false);
            std::fs::set_permissions(&seg.path, permissions).expect("set permissions");
        }

        // Corrupt a byte in the footer
        let mut bytes = std::fs::read(&seg.path).expect("read");
        let footer_pos = bytes.len() - 20; // Somewhere in footer
        bytes[footer_pos] ^= 0xFF;
        std::fs::write(&seg.path, &bytes).expect("write");

        let result = SegmentReader::open(&seg.path);
        assert!(matches!(result, Err(SegmentError::ChecksumMismatch { .. })));
    }

    #[test]
    fn reader_stream_lookup_by_id() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");
        let stream_id = reader.streams()[0].id;

        let meta = reader.stream(stream_id);
        assert!(meta.is_some());
        assert_eq!(meta.map(|m| m.id), Some(stream_id));

        let missing = reader.stream(StreamId::new(999));
        assert!(missing.is_none());
    }

    #[test]
    fn reader_chunk_out_of_bounds() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");
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

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let _ = writer.write_segment(&path, open_segment).expect("write");

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
        let seg = TestSegment::new();

        let reader = SegmentReader::open(&seg.path).expect("open");
        let entry = reader.manifest()[0].clone();

        let bundle = reader.read_bundle(&entry).expect("read_bundle");
        let payloads = bundle.into_payloads();

        assert_eq!(payloads.len(), 1);
        assert!(payloads.contains_key(&SlotId::new(0)));
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn reader_opens_mmap() {
        let seg = TestSegment::new();

        let reader = SegmentReader::open_mmap(&seg.path).expect("open_mmap");

        assert_eq!(reader.stream_count(), seg.stream_count);
        assert_eq!(reader.bundle_count(), seg.bundle_count);
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_bundle_outlives_reader() {
        let seg = TestSegment::new();

        let bundle = {
            let reader = SegmentReader::open_mmap(&seg.path).expect("open_mmap");
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

        let writer = SegmentWriter::new(SegmentSeq::new(1));
        let _ = writer.write_segment(&path, open_segment).expect("write");

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
