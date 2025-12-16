// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Segment storage for Quiver.
//!
//! A **segment** is a container file holding multiple Arrow IPC streams plus a
//! manifest that describes how those streams reassemble back into the
//! [`RecordBundle`](crate::record_bundle::RecordBundle) abstraction used by
//! the embedding application. Segments are immutable once finalized and support
//! zero-copy memory-mapped reads.
//!
//! # Why a Custom Format Instead of Plain Arrow IPC?
//!
//! Arrow IPC (both streaming and file formats) requires all `RecordBatch`es in
//! a single stream to share the same schema. This conflicts with OTAP's data
//! model:
//!
//! 1. **Multiple payload types per bundle**: Each `RecordBundle` contains
//!    multiple payload slots (`Logs`, `LogAttrs`, `ScopeAttrs`, etc.), each
//!    with a completely different schema.
//!
//! 2. **Schema evolution within a payload type**: Even for a single slot, the
//!    schema can change between bundles—optional columns may appear/disappear,
//!    and dictionary-encoded columns may switch between `Dictionary<u8, Utf8>`,
//!    `Dictionary<u16, Utf8>`, or native `Utf8` based on cardinality.
//!
//! 3. **Optional payloads**: Some slots may be absent entirely for a given
//!    bundle (e.g., no `ScopeAttrs` when scope attributes are empty).
//!
//! The Quiver segment format addresses this by interleaving multiple Arrow IPC
//! *file* streams (one per `(slot, schema_fingerprint)` pair) inside a single
//! container, with a manifest recording how to reconstruct each original
//! `RecordBundle`. This preserves:
//!
//! - **Standard Arrow IPC reading**: Each stream is a valid Arrow IPC file
//!   readable via `arrow_ipc::reader::FileReader` (on a memory-mapped slice).
//! - **Efficient storage**: Batches with the same schema share a stream,
//!   enabling dictionary delta encoding and avoiding repeated schema metadata.
//! - **Zero-copy access**: The entire segment can be memory-mapped; readers
//!   seek to stream offsets without copying data.
//!
//! # Terminology
//!
//! The following terms are used throughout this module. See also the
//! Terminology section in `ARCHITECTURE.md` for additional context.
//!
//! | Term                 | Definition                                                                                                                                                                             |
//! |----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
//! | **Segment**          | A container file grouping multiple `RecordBundle` arrivals. Contains one or more *streams* plus metadata (stream directory + batch manifest). Target size is 8-64 MB.                  |
//! | **Stream**           | An ordered sequence of *chunks* for a specific `(slot, schema_fingerprint)` pairing. All chunks in a stream share a schema and unified dictionary vocabulary. Serialized as Arrow IPC. |
//! | **Chunk**            | A single Arrow `RecordBatch` within a stream. Chunks share the stream's unified dictionary vocabulary and are individually addressable via the batch manifest.                         |
//! | **Stream Directory** | Metadata table at the segment level listing every stream's id, slot, schema fingerprint, byte offset, byte length, and statistics.                                                     |
//! | **Batch Manifest**   | Ordered list of `RecordBundle` arrivals within a segment. Each entry maps payload slots to `(stream_id, chunk_index)` pairs for reconstruction.                                        |
//! | **Slot**             | A stable identifier for a payload type within a `RecordBundle` (e.g., slot 0 = Logs, slot 1 = LogAttrs). See [`SlotId`](crate::record_bundle::SlotId).                                 |
//!
//! # Relationship to Arrow Concepts
//!
//! - **Arrow `RecordBatch`**: The fundamental unit of columnar data in Arrow.
//!   Each payload slot in a `RecordBundle` contains a `RecordBatch`.
//! - **Arrow IPC**: The serialization format used to persist `RecordBatch`
//!   data. Quiver uses the IPC *streaming* format during accumulation and
//!   writes IPC *file* format on finalization for memory-mapped reads.
//! - **Schema Fingerprint**: A 256-bit hash identifying the Arrow schema of
//!   a payload. Payloads with different schemas route to different streams.
//!
//! # Module Organization
//!
//! | File                    | Purpose                                             |
//! |-------------------------|-----------------------------------------------------|
//! | `types.rs`              | Core type definitions (StreamId, StreamKey, etc.)   |
//! | `error.rs`              | Segment-specific error types                        |
//! | `stream_accumulator.rs` | Per-stream batch accumulation                       |
//! | `open_segment.rs`       | Open segment buffer routing payloads to streams     |
//! | `writer.rs`             | Segment file writer                                  |
//! | `reader.rs`             | Segment file reader with CRC validation              |

mod error;
mod open_segment;
mod reader;
mod stream_accumulator;
#[cfg(test)]
pub(crate) mod test_utils;
mod types;
mod writer;

pub use error::SegmentError;
pub use open_segment::OpenSegment;
pub use reader::{ReconstructedBundle, SegmentReader};
pub use stream_accumulator::StreamAccumulator;
pub use types::{
    ChunkIndex, ManifestEntry, SegmentSeq, SlotChunkRef, StreamId, StreamKey, StreamMetadata,
};
pub use writer::SegmentWriter;
