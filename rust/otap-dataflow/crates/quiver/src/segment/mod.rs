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
//! | File       | Purpose                                              |
//! |------------|------------------------------------------------------|
//! | `types.rs` | Core type definitions (StreamId, StreamKey, etc.)   |
//! | `error.rs` | Segment-specific error types                         |

mod error;
mod stream_accumulator;
mod types;

pub use error::SegmentError;
pub use stream_accumulator::StreamAccumulator;
pub use types::{
    ChunkIndex, ManifestEntry, SegmentSeq, SlotChunkRef, StreamId, StreamKey, StreamMetadata,
};
