// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Accumulates Arrow `RecordBatch`es for a single stream within a segment.
//!
//! A `StreamAccumulator` buffers batches in memory during segment accumulation.
//! When the segment is written to disk, the accumulator streams Arrow IPC file
//! bytes that can be memory-mapped for zero-copy reads.
//!
//! # Lifecycle
//!
//! 1. Create with [`StreamAccumulator::new`], providing the stream's schema.
//! 2. Append batches via [`StreamAccumulator::append`]; each call returns the
//!    chunk index for manifest bookkeeping.
//! 3. The accumulator is consumed by [`SegmentWriter::write_segment`] which
//!    streams the IPC data directly to disk.
//!
//! [`SegmentWriter::write_segment`]: super::SegmentWriter::write_segment
//!
//! # Dictionary Handling
//!
//! The accumulator writes batches as-is using Arrow IPC's default
//! `DictionaryHandling::Resend` mode, where each batch includes its full
//! dictionary. This preserves the original dictionary key types (e.g., `UInt8`
//! vs `UInt16`) exactly as received.
//!
//! **Design rationale**: Quiver prioritizes schema fidelity over storage
//! optimization. Dictionary unification would merge vocabularies across batches
//! but could widen key types (e.g., `UInt8` → `UInt16`) when cardinality
//! exceeds the original type's capacity. This would cause readers to receive
//! different schemas than writers sent, breaking round-trip guarantees.
//!
//! Trade-offs of this approach:
//! - **Pro**: Exact schema preservation - readers get back what writers sent
//! - **Pro**: Each batch is self-contained and independently readable
//! - **Con**: Larger file sizes due to duplicate dictionary values, which also
//!   increases memory consumption when segments are memory-mapped for reading
//!
//! This design decision may be revisited if future performance measurements
//! indicate that the size/memory overhead is a significant concern.

use arrow_array::RecordBatch;
use arrow_ipc::writer::{FileWriter, IpcWriteOptions};
use arrow_schema::SchemaRef;

use super::error::SegmentError;
use super::types::{ChunkIndex, MAX_CHUNKS_PER_STREAM, StreamId, StreamKey, StreamMetadata};
use crate::record_bundle::{SchemaFingerprint, SlotId};

/// Accumulates `RecordBatch`es for a single `(slot, schema)` stream.
///
/// This is an in-memory buffer that collects batches during segment
/// accumulation. On finalization, it produces Arrow IPC file bytes
/// suitable for memory-mapped reads.
pub struct StreamAccumulator {
    /// Unique identifier for this stream within the segment.
    stream_id: StreamId,
    /// The slot this stream serves.
    slot_id: SlotId,
    /// Schema fingerprint for routing verification.
    schema_fingerprint: SchemaFingerprint,
    /// Arrow schema for all batches in this stream.
    schema: SchemaRef,
    /// Buffered batches awaiting finalization.
    batches: Vec<RecordBatch>,
    /// Running count of rows across all batches.
    row_count: u64,
    /// Running total of buffer memory used by accumulated batches.
    buffer_size: usize,
    /// Whether finalize() has been called.
    finalized: bool,
}

impl StreamAccumulator {
    /// Creates a new stream accumulator.
    ///
    /// # Arguments
    ///
    /// * `stream_id` - Unique identifier for this stream within the segment.
    /// * `slot_id` - The payload slot this stream serves.
    /// * `schema_fingerprint` - 256-bit hash of the schema for routing.
    /// * `schema` - Arrow schema that all appended batches must match.
    #[must_use]
    pub fn new(
        stream_id: StreamId,
        slot_id: SlotId,
        schema_fingerprint: SchemaFingerprint,
        schema: SchemaRef,
    ) -> Self {
        Self {
            stream_id,
            slot_id,
            schema_fingerprint,
            schema,
            batches: Vec::new(),
            row_count: 0,
            buffer_size: 0,
            finalized: false,
        }
    }

    /// Returns the stream's unique identifier.
    #[must_use]
    pub fn stream_id(&self) -> StreamId {
        self.stream_id
    }

    /// Returns the slot this stream serves.
    #[must_use]
    pub fn slot_id(&self) -> SlotId {
        self.slot_id
    }

    /// Returns the schema fingerprint for this stream.
    #[must_use]
    pub fn schema_fingerprint(&self) -> SchemaFingerprint {
        self.schema_fingerprint
    }

    /// Returns the stream key for this accumulator.
    #[must_use]
    pub fn stream_key(&self) -> StreamKey {
        (self.slot_id, self.schema_fingerprint)
    }

    /// Returns the Arrow schema for this stream.
    #[must_use]
    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    /// Returns the number of chunks (batches) accumulated so far.
    #[must_use]
    pub fn chunk_count(&self) -> u32 {
        self.batches.len() as u32
    }

    /// Returns the total row count across all accumulated batches.
    #[must_use]
    pub fn row_count(&self) -> u64 {
        self.row_count
    }

    /// Returns the total buffer memory used by accumulated batches.
    ///
    /// This is the sum of `get_array_memory_size()` for all appended batches,
    /// representing the actual Arrow buffer memory consumption.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Returns true if this accumulator has no batches.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    /// Returns true if finalize() has been called.
    #[must_use]
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Appends a `RecordBatch` to this stream.
    ///
    /// Returns the chunk index assigned to this batch, which should be
    /// recorded in the batch manifest.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::AccumulatorFinalized`] if the accumulator
    /// has already been finalized.
    /// Returns [`SegmentError::InvalidFormat`] if adding this batch would
    /// exceed the chunk limit.
    /// Returns [`SegmentError::SchemaMismatch`] if the batch schema doesn't
    /// match the stream's expected schema.
    pub fn append(&mut self, batch: RecordBatch) -> Result<ChunkIndex, SegmentError> {
        if self.finalized {
            return Err(SegmentError::AccumulatorFinalized);
        }

        if self.batches.len() >= MAX_CHUNKS_PER_STREAM {
            return Err(SegmentError::InvalidFormat {
                message: format!(
                    "stream {:?} already has {} chunks, cannot exceed limit of {}",
                    self.stream_id,
                    self.batches.len(),
                    MAX_CHUNKS_PER_STREAM
                ),
            });
        }

        // Validate schema matches - this catches routing bugs or data corruption
        if batch.schema() != self.schema {
            return Err(SegmentError::SchemaMismatch {
                stream_id: self.stream_id,
                expected: format!("{:?}", self.schema),
                actual: format!("{:?}", batch.schema()),
            });
        }

        let chunk_index = ChunkIndex::new(self.batches.len() as u32);
        self.row_count += batch.num_rows() as u64;
        self.buffer_size += batch.get_array_memory_size();
        self.batches.push(batch);

        Ok(chunk_index)
    }

    /// Writes all accumulated batches directly to a writer in Arrow IPC file format.
    ///
    /// Streams IPC bytes directly to the destination without buffering the entire
    /// serialized output in memory. After this call, the accumulator is consumed.
    ///
    /// # Arguments
    ///
    /// * `writer` - Destination for the Arrow IPC bytes.
    /// * `byte_offset` - The byte offset where this stream starts in the segment
    ///   file. Used for metadata only.
    ///
    /// # Returns
    ///
    /// The stream metadata including the actual byte length written.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentError::AccumulatorFinalized`] if already finalized.
    /// Returns [`SegmentError::Arrow`] if IPC serialization fails.
    pub fn write_to<W: std::io::Write>(
        mut self,
        writer: &mut W,
        byte_offset: u64,
    ) -> Result<StreamMetadata, SegmentError> {
        if self.finalized {
            return Err(SegmentError::AccumulatorFinalized);
        }
        self.finalized = true;

        let chunk_count = self.chunk_count();
        let row_count = self.row_count;

        // Wrap in CountingWriter to track bytes written
        let mut counting = CountingWriter::new(writer);
        self.write_ipc_to(&mut counting)?;
        let byte_length = counting.bytes_written() as u64;

        Ok(StreamMetadata::new(
            self.stream_id,
            self.slot_id,
            self.schema_fingerprint,
            byte_offset,
            byte_length,
            row_count,
            chunk_count,
        ))
    }

    /// Writes all accumulated batches to Arrow IPC file format.
    fn write_ipc_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), SegmentError> {
        // Use default IPC write options
        let options = IpcWriteOptions::default();

        let mut ipc_writer = FileWriter::try_new_with_options(writer, &self.schema, options)?;

        for batch in &self.batches {
            ipc_writer.write(batch)?;
        }

        ipc_writer.finish()?;

        Ok(())
    }
}

/// A writer wrapper that counts bytes written.
struct CountingWriter<W> {
    inner: W,
    bytes_written: usize,
}

impl<W> CountingWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            bytes_written: 0,
        }
    }

    fn bytes_written(&self) -> usize {
        self.bytes_written
    }
}

impl<W: std::io::Write> std::io::Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes_written += n;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl std::fmt::Debug for StreamAccumulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamAccumulator")
            .field("stream_id", &self.stream_id)
            .field("slot_id", &self.slot_id)
            .field("chunk_count", &self.chunk_count())
            .field("row_count", &self.row_count)
            .field("finalized", &self.finalized)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Arc;

    use arrow_array::{Int32Array, RecordBatch, StringArray};
    use arrow_ipc::reader::FileReader;
    use arrow_schema::{DataType, Field, Schema};

    use super::*;
    use crate::segment::test_utils::{make_batch, test_fingerprint, test_schema};

    #[test]
    fn new_accumulator_is_empty() {
        let schema = test_schema();
        let acc =
            StreamAccumulator::new(StreamId::new(0), SlotId::new(0), test_fingerprint(), schema);

        assert!(acc.is_empty());
        assert_eq!(acc.chunk_count(), 0);
        assert_eq!(acc.row_count(), 0);
        assert!(!acc.is_finalized());
    }

    #[test]
    fn append_returns_sequential_chunk_indices() {
        let schema = test_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 = make_batch(&schema, &[1, 2], &["a", "b"]);
        let batch2 = make_batch(&schema, &[3], &["c"]);
        let batch3 = make_batch(&schema, &[4, 5, 6], &["d", "e", "f"]);

        assert_eq!(acc.append(batch1).unwrap(), ChunkIndex::new(0));
        assert_eq!(acc.append(batch2).unwrap(), ChunkIndex::new(1));
        assert_eq!(acc.append(batch3).unwrap(), ChunkIndex::new(2));

        assert_eq!(acc.chunk_count(), 3);
        assert_eq!(acc.row_count(), 6); // 2 + 1 + 3
    }

    #[test]
    fn append_after_finalize_fails() {
        let schema = test_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch = make_batch(&schema, &[1], &["a"]);
        let _ = acc.append(batch).unwrap();

        // Finalize consumes self, so we need a new accumulator to test the error
        let mut acc2 = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema),
        );
        acc2.finalized = true; // Simulate already finalized

        let batch2 = make_batch(&schema, &[2], &["b"]);
        let result = acc2.append(batch2);
        assert!(matches!(result, Err(SegmentError::AccumulatorFinalized)));
    }

    #[test]
    fn write_to_produces_valid_arrow_ipc() {
        let schema = test_schema();
        let mut acc = StreamAccumulator::new(
            StreamId::new(5),
            SlotId::new(2),
            test_fingerprint(),
            Arc::clone(&schema),
        );

        let batch1 = make_batch(&schema, &[1, 2], &["alice", "bob"]);
        let batch2 = make_batch(&schema, &[3, 4, 5], &["charlie", "diana", "eve"]);

        let _ = acc.append(batch1).unwrap();
        let _ = acc.append(batch2).unwrap();

        // Write to a buffer using write_to
        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 1024).unwrap();

        // Verify metadata
        assert_eq!(metadata.id, StreamId::new(5));
        assert_eq!(metadata.slot_id, SlotId::new(2));
        assert_eq!(metadata.schema_fingerprint, test_fingerprint());
        assert_eq!(metadata.byte_offset, 1024);
        assert!(metadata.byte_length > 0);
        assert_eq!(metadata.row_count, 5);
        assert_eq!(metadata.chunk_count, 2);

        // Verify IPC bytes are readable
        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");

        // Schema should match
        assert_eq!(reader.schema(), schema);

        // Read all batches
        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert_eq!(batches.len(), 2);

        // Verify batch contents
        assert_eq!(batches[0].num_rows(), 2);
        assert_eq!(batches[1].num_rows(), 3);
    }

    #[test]
    fn write_to_empty_accumulator_produces_valid_ipc() {
        let schema = test_schema();
        let acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            schema.clone(),
        );

        let mut buffer = Vec::new();
        let metadata = acc.write_to(&mut buffer, 0).unwrap();

        assert_eq!(metadata.chunk_count, 0);
        assert_eq!(metadata.row_count, 0);

        // Empty IPC file should still be readable
        let cursor = Cursor::new(buffer);
        let reader = FileReader::try_new(cursor, None).expect("valid IPC file");
        assert_eq!(reader.schema(), schema);

        let batches: Vec<_> = reader.map(|r| r.expect("valid batch")).collect();
        assert!(batches.is_empty());
    }

    #[test]
    fn stream_key_matches_constructor_args() {
        let schema = test_schema();
        let fp = [0x42u8; 32];
        const SLOT_ID: u16 = 3;
        let acc = StreamAccumulator::new(StreamId::new(7), SlotId::new(SLOT_ID), fp, schema);

        let key = acc.stream_key();
        assert_eq!(key.0, SlotId::new(SLOT_ID));
        assert_eq!(key.1, fp);
    }

    #[test]
    fn accessors_return_expected_values() {
        let schema = test_schema();
        let fp = [0x99u8; 32];
        let acc =
            StreamAccumulator::new(StreamId::new(10), SlotId::new(5), fp, Arc::clone(&schema));

        assert_eq!(acc.stream_id(), StreamId::new(10));
        assert_eq!(acc.slot_id(), SlotId::new(5));
        assert_eq!(acc.schema_fingerprint(), fp);
        assert_eq!(acc.schema(), &schema);
    }

    #[test]
    fn debug_impl_does_not_panic() {
        let schema = test_schema();
        let acc =
            StreamAccumulator::new(StreamId::new(0), SlotId::new(0), test_fingerprint(), schema);
        let debug_str = format!("{:?}", acc);
        assert!(debug_str.contains("StreamAccumulator"));
        assert!(debug_str.contains("stream_id"));
    }

    #[test]
    fn append_rejects_schema_mismatch() {
        let schema1 = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));

        let schema2 = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("extra", DataType::Utf8, true),
        ]));

        let mut acc = StreamAccumulator::new(
            StreamId::new(0),
            SlotId::new(0),
            test_fingerprint(),
            Arc::clone(&schema1),
        );

        // Create a batch with a different schema
        let batch = RecordBatch::try_new(
            Arc::clone(&schema2),
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(StringArray::from(vec!["a", "b"])),
            ],
        )
        .unwrap();

        let result = acc.append(batch);

        match result {
            Err(SegmentError::SchemaMismatch {
                stream_id,
                expected,
                actual,
            }) => {
                assert_eq!(stream_id, StreamId::new(0));
                // Expected schema has 1 field, actual has 2
                assert!(expected.contains("id"));
                assert!(!expected.contains("extra"));
                assert!(actual.contains("id"));
                assert!(actual.contains("extra"));
            }
            other => panic!("Expected SchemaMismatch error, got: {:?}", other),
        }
    }
}
