// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Write-ahead-log (WAL) writer for Quiver.
//!
//! The writer is responsible for appending Arrow payloads, rotating chunk
//! files when size thresholds are exceeded, and reclaiming durable space once
//! downstream consumers advance the truncation cursor. Safe offsets are always
//! validated against real entry boundaries so we never expose partially written
//! frames to readers. 
//!
//! # WAL lifecycle at a glance
//!
//! ```text
//!   ingest thread                               wal writer
//! ───────────────────────────────────────────────────────────────────────────
//!   RecordBundle -> encode slots -> entry header -> fs writes -> fsync
//!                                      |                     |
//!                                      v                     v
//!                                truncate cursor <── sidecar flush
//! ```
//!
//! * **Append path** – Every [`RecordBundle`] is serialized slot-by-slot into an
//!   in-memory buffer, hashed (CRC32), and written as one atomic entry. We track
//!   [`WalOffset`] so readers can correlate persisted bytes back to sequences.
//! * **Flush loop** – `flush_interval` and `max_unflushed_bytes` control when we
//!   call `flush()`/`sync_data()`. Tests can inspect thread-local flags to assert
//!   that durability barriers occurred.
//! * **Rotation** – When `rotation_target_bytes` is exceeded the active WAL file
//!   is renamed to `wal.n` (oldest chunks slide toward higher numbers) and a new
//!   header is seeded in-place. [`RotatedChunk`] metadata keeps track of logical
//!   data ranges so stale chunks can be deleted after truncation advances.
//! * **Truncation + sidecar** – Readers compute `WalTruncateCursor` values. The
//!   writer validates cursor boundaries, updates the truncate sidecar on disk,
//!   and relies on chunk rotation to eventually drop fully safe bytes. The
//!   sidecar survives crashes so restarts resume from the last known safe offset
//!   even if a rotation has not run yet.
//! * **Global cap enforcement** – `max_wal_size` is measured across the active
//!   file plus rotated chunks. Exceeding the cap yields `WalAtCapacity` errors
//!   until downstream consumers free space.
//!
//! ## Offset coordinate systems
//!
//! Truncate metadata uses two overlapping coordinate spaces:
//!
//! 1. **Logical/global cursor** – `truncate_state.logical_offset` is the number
//!    of data bytes (headers excluded) that have been validated across the
//!    entire WAL stream. This is the only value persisted in the
//!    `truncate.offset` sidecar so it remains stable across rotations.
//! 2. **Per-file offsets** – Derived from the logical cursor by subtracting
//!    `rotated_prefix_bytes` (the payload currently living in rotated chunks)
//!    and adding `WAL_HEADER_LEN`. This gives the byte location inside the
//!    active file so we can touch OS file descriptors without ambiguity.
//!
//! `purge_rotated_chunks` compares the logical cursor against each chunk's
//! `end_data_offset` so we only delete files once the cursor has moved past
//! their logical range.
//!
//! ```text
//! wal.3 (oldest)    wal.2             wal.1    wal (active)
//! ┌──────────────┬──────────┬────────────────┬───────┬────────────┬──────────┐
//! │ rotated data │ rotated  │ rotated data   │header │ per-file   │ data tail│
//! │   (oldest)   │  data    │   (newest)     │       │ data bytes │          │
//! └──────────────┴──────────┴────────────────┴───────┴────────────┴──────────┘
//! ^                        ^                 ^       ^
//! │                        │                 │       │
//! oldest bytes   logical cursor (sidecar)    │       per-file cursor derived
//!                                            │       from logical offset
//!                                            │ 
//!                                            └── prefix contributed by rotated chunks (rotated_prefix_bytes)
//! ```

//! Single-file case (no rotated chunks yet):
//!
//! ```text
//! wal (active)
//! ┌───────┬──────────────────────────┬──────────────┐
//! │header │ per-file data bytes      │   data tail  │
//! └───────┴──────────────────────────┴──────────────┘
//! ^       ^                          ^
//! │       │                          │
//! header per-file cursor             logical cursor (no header bias)
//! ```
//!
//! When no rotated chunks exist `rotated_prefix_bytes` is `0`, so both arrows land on
//! the same byte boundary within the active file even though one value includes
//! the header prefix and the other does not. As soon as rotation occurs the
//! logical cursor continues to advance across the concatenated stream while the
//! per-file cursor snaps back to `WAL_HEADER_LEN` in the brand-new active file.
//!
//! This separation lets restarts resume from the sidecar-safe position inside
//! the freshly created active file while still honoring previously persisted
//! cursors that might point into older chunks.
//!
//! ## Internal structure
//!
//! * [`WalWriter`] – the façade exposed to call sites. It orchestrates appends,
//!   truncation, and recovery by delegating to the pieces below.
//! * [`WalSegment`] – owns the on-disk file handle plus the transient payload
//!   buffer. All direct I/O (seek, write, flush/fsync) flows through this type
//!   so we only mutate the OS file descriptor in one place.
//! * [`WalCoordinator`] – tracks policy decisions (rotation, global caps, and
//!   truncate sidecar state). It never touches raw I/O directly; instead it
//!   inspects `WalSegment` lengths and requests actions.
//!
//! Splitting responsibilities this way keeps the append path easier to reason
//! about: `WalWriter` asks the coordinator for admission, streams bytes via the
//! segment, then records whatever book-keeping the coordinator dictates.
//!
//! ## Failure and recovery cues
//!
//! ```text
//!            truncate state
//!        ┌──────────────────────┐
//!   WAL  │ cursor + generation  │  sidecar (truncate.offset)
//! header ├──────────────────────┤<──────────────────────────────┐
//!   ...  │   active data tail   │                               │
//!        └──────────────────────┘                               │
//!              ▲           ▲                                    │
//!              │           │                                    │
//!          reclaim request └────── crash-safe checkpoint  ──────┘
//! ```
//!
//! * Failpoints in `test_support` let us simulate crashes before the truncate
//!   sidecar rename. Clearing the crash flag allows resuming normal operation.
//! * Drop flushing is skipped when `test_force_crash` was invoked so tests can
//!   model abrupt exits without double-syncing.
//!
//! These docs focus on the writer-facing lifecycle. See the reader module for
//! how safe cursors are derived from WAL entries.
#![allow(dead_code)]

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use crc32fast::Hasher;

use crate::record_bundle::{PayloadRef, RecordBundle, SchemaFingerprint, SlotId};

use super::header::{WAL_HEADER_LEN, WalHeader};
use super::reader::WalReader;
use super::truncate_sidecar::TruncateSidecar;
use super::{
    ENTRY_HEADER_LEN, ENTRY_TYPE_RECORD_BUNDLE, SCHEMA_FINGERPRINT_LEN, SLOT_HEADER_LEN, WalError,
    WalResult, WalTruncateCursor,
};


/// Low-level tunables that bridge the user-facing [`QuiverConfig`] into the WAL.
#[derive(Debug, Clone)]
pub(crate) struct WalWriterOptions {
    pub path: PathBuf,
    pub segment_cfg_hash: [u8; 16],
    pub flush_interval: Duration,
    pub max_unflushed_bytes: u64,
    pub max_wal_size: u64,
    pub max_chunks: usize,
    pub rotation_target_bytes: u64,
}

impl WalWriterOptions {
    pub fn new(path: PathBuf, segment_cfg_hash: [u8; 16], flush_interval: Duration) -> Self {
        Self {
            path,
            segment_cfg_hash,
            flush_interval,
            max_unflushed_bytes: 0,
            max_wal_size: u64::MAX,
            max_chunks: 8,
            rotation_target_bytes: 64 * 1024 * 1024,
        }
    }

    // Remove when integrated with real replay/compaction path.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_max_unflushed_bytes(mut self, max_bytes: u64) -> Self {
        self.max_unflushed_bytes = max_bytes;
        self
    }

    pub fn with_max_wal_size(mut self, max_bytes: u64) -> Self {
        self.max_wal_size = max_bytes;
        self
    }

    pub fn with_max_chunks(mut self, max_chunks: usize) -> Self {
        self.max_chunks = max_chunks.max(1);
        self
    }

    pub fn with_rotation_target(mut self, target_bytes: u64) -> Self {
        self.rotation_target_bytes = target_bytes.max(1);
        self
    }

}

/// Stateful writer that maintains append position, rotation metadata, and
/// persisted truncate checkpoints. It tracks both the *current* WAL file and
/// any rotated chunks so the total on-disk footprint (`aggregate_bytes`) can be
/// compared against caps before admitting new entries.
#[derive(Debug)]
pub(crate) struct WalWriter {
    /// Encapsulates the active WAL file handle plus its in-memory staging
    /// buffer. All raw I/O flows through this helper.
    segment: WalSegment,
    /// Manages policy, truncate bookkeeping, and rotation metadata.
    coordinator: WalCoordinator,
    /// Next sequence number to assign to an appended entry.
    next_sequence: u64,
    #[cfg(test)]
    /// When true we skip drop-time flushes to simulate a crash.
    test_crashed: bool,
}

#[derive(Debug)]
struct WalSegment {
    /// Active WAL file descriptor.
    file: File,
    /// Scratch buffer used to serialize slot payloads before writing.
    payload_buffer: Vec<u8>,
    /// Timestamp of the most recent flush.
    last_flush: Instant,
    /// Bytes written since the last flush.
    unflushed_bytes: u64,
    /// Current logical file length (includes header and data bytes).
    current_len: u64,
}

#[derive(Debug)]
struct WalCoordinator {
    /// User-provided tuning knobs for the writer.
    options: WalWriterOptions,
    /// Path to the truncate sidecar file on disk.
    sidecar_path: PathBuf,
    /// Cached copy of the persisted truncate offset + rotation generation.
    truncate_state: TruncateSidecar,
    /// Total bytes across the active WAL plus all rotated chunks.
    aggregate_bytes: u64,
    /// Metadata describing each rotated `wal.N` chunk on disk.
    rotated_chunks: VecDeque<RotatedChunk>,
    /// Logical prefix length contributed by all rotated chunks combined.
    rotated_prefix_bytes: u64,
    /// Most recent offset validated to be on an entry boundary.
    active_safe_offset: u64,
    /// Sequence number associated with the last committed truncate cursor.
    last_safe_sequence: Option<u64>,
}

/// Opaque marker returned to callers after an append so they can correlate
/// persisted entries with reader positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WalOffset {
    pub position: u64,
    pub sequence: u64,
}

impl WalWriter {
    pub fn open(options: WalWriterOptions) -> WalResult<Self> {
        if let Some(parent) = options.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&options.path)?;

        let metadata = file.metadata()?;
        let metadata_len = if metadata.len() == 0 {
            let header = WalHeader::new(options.segment_cfg_hash);
            header.write_to(&mut file)?;
            file.flush()?;
            WAL_HEADER_LEN as u64
        } else if metadata.len() < WAL_HEADER_LEN as u64 {
            return Err(WalError::InvalidHeader("file smaller than header"));
        } else {
            let header = WalHeader::read_from(&mut file)?;
            if header.segment_cfg_hash != options.segment_cfg_hash {
                return Err(WalError::SegmentConfigMismatch {
                    expected: options.segment_cfg_hash,
                    found: header.segment_cfg_hash,
                });
            }
            metadata.len()
        };

        let _ = file.seek(SeekFrom::End(0))?;
        let sidecar_path = truncate_sidecar_path(&options.path);
        let truncate_state = load_truncate_state(&sidecar_path)?;

        let segment = WalSegment::new(file, metadata_len);
        let mut coordinator = WalCoordinator::new(
            options,
            sidecar_path,
            truncate_state,
            metadata_len,
        );
        coordinator.reload_rotated_chunks(segment.len())?;
        coordinator.restore_safe_offsets(segment.len());

        let mut writer = Self {
            segment,
            coordinator,
            next_sequence: 0,
            #[cfg(test)]
            test_crashed: false,
        };
        writer.next_sequence = writer.coordinator.detect_next_sequence()?;
        Ok(writer)
    }

    fn options(&self) -> &WalWriterOptions {
        self.coordinator.options()
    }

    /// Serializes a [`RecordBundle`] into the active WAL file and returns the
    /// byte offset + sequence number associated with the entry. The writer keeps
    /// internal counters so the next call knows when to flush, rotate, or apply
    /// global caps.
    pub fn append_bundle<B: RecordBundle>(&mut self, bundle: &B) -> WalResult<WalOffset> {
        let descriptor = bundle.descriptor();
        let ingestion_time = bundle.ingestion_time();
        let ingestion_ts_nanos = system_time_to_nanos(ingestion_time)?;

        let mut encoded_slots = Vec::new();
        let mut total_payload_bytes = 0usize;

        self.segment.payload_buffer.clear();

        let sequence = self.next_sequence;

        let mut slot_bitmap = 0u64;

        for slot in &descriptor.slots {
            let slot_index = slot.id.0 as usize;
            if slot_index >= 64 {
                return Err(WalError::SlotOutOfRange(slot.id));
            }
            if let Some(payload) = bundle.payload(slot.id) {
                slot_bitmap |= 1u64 << slot_index;
                let encoded_slot = self.prepare_slot(slot.id, payload)?;
                let slot_size = encoded_slot.serialized_size();
                total_payload_bytes = total_payload_bytes
                    .checked_add(slot_size)
                    .ok_or(WalError::EntryTooLarge(slot_size))?;
                encoded_slots.push(encoded_slot);
            }
        }

        self.segment.payload_buffer.reserve(total_payload_bytes);
        for slot in encoded_slots {
            slot.write_into(&mut self.segment.payload_buffer);
        }

        let mut entry_header = [0u8; ENTRY_HEADER_LEN];
        let mut cursor = 0;
        entry_header[cursor] = ENTRY_TYPE_RECORD_BUNDLE;
        cursor += 1;
        entry_header[cursor..cursor + 8].copy_from_slice(&ingestion_ts_nanos.to_le_bytes());
        cursor += 8;
        entry_header[cursor..cursor + 8].copy_from_slice(&sequence.to_le_bytes());
        cursor += 8;
        entry_header[cursor..cursor + 8].copy_from_slice(&slot_bitmap.to_le_bytes());

        let entry_body_len = ENTRY_HEADER_LEN + self.segment.payload_buffer.len();
        let entry_len =
            u32::try_from(entry_body_len).map_err(|_| WalError::EntryTooLarge(entry_body_len))?;

        let mut hasher = Hasher::new();
        hasher.update(&entry_header);
        hasher.update(&self.segment.payload_buffer);
        let crc = hasher.finalize();

        let entry_total_bytes = 4u64 + u64::from(entry_len) + 4;
        self.coordinator
            .preflight_append(&self.segment, entry_total_bytes)?;

        let mut payload_bytes = std::mem::take(&mut self.segment.payload_buffer);
        let entry_start = self
            .segment
            .write_entry(entry_len, &entry_header, &payload_bytes, crc)?;
        payload_bytes.clear();
        self.segment.payload_buffer = payload_bytes;

        self.next_sequence = self.next_sequence.wrapping_add(1);

        self.segment.current_len = self.segment.current_len.saturating_add(entry_total_bytes);
        self.coordinator.record_append(entry_total_bytes);
        self.segment
            .maybe_flush(self.coordinator.options(), entry_total_bytes)?;

        self.coordinator
            .maybe_rotate_after_append(&mut self.segment)?;
        self.coordinator.enforce_size_cap()?;

        Ok(WalOffset {
            position: entry_start,
            sequence,
        })
    }

    /// Physically truncates the active WAL file to the provided cursor. This is
    /// only possible when the cursor aligns to validated entry boundaries; the
    /// call also refreshes the truncate sidecar so future restarts keep the same
    /// safe point.
    pub fn truncate_to(&mut self, cursor: &WalTruncateCursor) -> WalResult<()> {
        self.coordinator
            .truncate_active_segment(&mut self.segment, cursor)
    }

    /// Updates the truncate sidecar without touching the underlying WAL bytes.
    /// This is used when we merely want to record a new safe cursor so future
    /// rotations know how far readers have progressed.
    pub(crate) fn record_truncate_cursor(&mut self, cursor: &WalTruncateCursor) -> WalResult<()> {
        self.coordinator
            .record_truncate_cursor(&mut self.segment, cursor)
    }

    /// Records a new truncate cursor and lets future rotations delete fully
    /// safe chunks. This validates the cursor just like [`truncate_to`] but
    /// leaves the active file untouched until rotation thresholds are met.
    pub fn reclaim_prefix(&mut self, cursor: &WalTruncateCursor) -> WalResult<()> {
        self.coordinator.reclaim_prefix(&mut self.segment, cursor)
    }

    fn prepare_slot(&mut self, slot_id: SlotId, payload: PayloadRef<'_>) -> WalResult<EncodedSlot> {
        let row_count = u32::try_from(payload.batch.num_rows())
            .map_err(|_| WalError::RowCountOverflow(payload.batch.num_rows()))?;
        let payload_bytes = encode_record_batch(payload.batch)?;
        let payload_len = u32::try_from(payload_bytes.len())
            .map_err(|_| WalError::PayloadTooLarge(payload_bytes.len()))?;

        Ok(EncodedSlot {
            slot_id_raw: slot_id.0,
            schema_fingerprint: payload.schema_fingerprint,
            row_count,
            payload_len,
            payload_bytes,
        })
    }
}

impl WalSegment {
    fn new(file: File, current_len: u64) -> Self {
        Self {
            file,
            payload_buffer: Vec::new(),
            last_flush: Instant::now(),
            unflushed_bytes: 0,
            current_len,
        }
    }

    fn len(&self) -> u64 {
        self.current_len
    }

    fn seek_to_end(&mut self) -> WalResult<u64> {
        let pos = self.file.seek(SeekFrom::End(0))?;
        Ok(pos)
    }

    fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn set_len(&mut self, len: u64) {
        self.current_len = len;
    }

    fn replace_file(&mut self, file: File, new_len: u64) {
        self.file = file;
        self.current_len = new_len;
        self.last_flush = Instant::now();
        self.unflushed_bytes = 0;
    }

    fn write_entry(
        &mut self,
        entry_len: u32,
        entry_header: &[u8; ENTRY_HEADER_LEN],
        payload: &[u8],
        crc: u32,
    ) -> WalResult<u64> {
        let entry_start = self.seek_to_end()?;
        self.file.write_all(&entry_len.to_le_bytes())?;
        self.file.write_all(entry_header)?;
        self.file.write_all(payload)?;
        self.file.write_all(&crc.to_le_bytes())?;
        Ok(entry_start)
    }

    fn maybe_flush(&mut self, options: &WalWriterOptions, bytes_written: u64) -> WalResult<()> {
        self.unflushed_bytes = self.unflushed_bytes.saturating_add(bytes_written);

        if options.flush_interval.is_zero() {
            return self.flush_now();
        }

        if options.max_unflushed_bytes > 0 && self.unflushed_bytes >= options.max_unflushed_bytes {
            return self.flush_now();
        }

        if self.last_flush.elapsed() >= options.flush_interval {
            self.flush_now()?;
        }

        Ok(())
    }

    fn flush_now(&mut self) -> WalResult<()> {
        self.file.flush()?;
        sync_file_data(&self.file)?;
        self.last_flush = Instant::now();
        self.unflushed_bytes = 0;
        #[cfg(test)]
        test_support::record_flush();
        Ok(())
    }
}

impl WalCoordinator {
    fn new(
        options: WalWriterOptions,
        sidecar_path: PathBuf,
        truncate_state: TruncateSidecar,
        current_len: u64,
    ) -> Self {
        Self {
            options,
            sidecar_path,
            truncate_state,
            aggregate_bytes: current_len,
            rotated_chunks: VecDeque::new(),
            rotated_prefix_bytes: 0,
            active_safe_offset: WAL_HEADER_LEN as u64,
            last_safe_sequence: None,
        }
    }

    fn options(&self) -> &WalWriterOptions {
        &self.options
    }

    fn reload_rotated_chunks(&mut self, active_len: u64) -> WalResult<()> {
        let discovered = discover_rotated_chunk_files(&self.options.path)?;
        if discovered.is_empty() {
            self.aggregate_bytes = active_len;
            self.rotated_chunks.clear();
            self.rotated_prefix_bytes = 0;
            return Ok(());
        }

        let mut chunks = VecDeque::with_capacity(discovered.len());
        let mut aggregate = active_len;
        let mut cumulative_data = 0u64;

        for (index, len) in discovered.into_iter().rev() {
            aggregate = aggregate.saturating_add(len);
            let data_bytes = len.saturating_sub(WAL_HEADER_LEN as u64);
            cumulative_data = cumulative_data.saturating_add(data_bytes);
            chunks.push_back(RotatedChunk {
                path: rotated_path(&self.options.path, index),
                file_bytes: len,
                end_data_offset: cumulative_data,
            });
        }

        self.rotated_chunks = chunks;
        self.aggregate_bytes = aggregate;
        self.rotated_prefix_bytes = cumulative_data;
        Ok(())
    }

    fn restore_safe_offsets(&mut self, active_len: u64) {
        let header = WAL_HEADER_LEN as u64;
        let active_data = active_len.saturating_sub(header);
        let total_logical = self
            .rotated_prefix_bytes
            .saturating_add(active_data);

        if self.truncate_state.logical_offset > total_logical {
            self.truncate_state.logical_offset = total_logical;
        }

        self.active_safe_offset =
            self.per_file_offset_from_global(self.truncate_state.logical_offset, active_len);
    }

    fn record_append(&mut self, entry_total_bytes: u64) {
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(entry_total_bytes);
    }

    fn preflight_append(&mut self, segment: &WalSegment, entry_total_bytes: u64) -> WalResult<()> {
        let will_rotate = segment
            .len()
            .saturating_add(entry_total_bytes)
            .saturating_sub(WAL_HEADER_LEN as u64)
            > self.options.rotation_target_bytes;

        if will_rotate && self.rotated_chunks.len() >= self.options.max_chunks {
            return Err(WalError::WalAtCapacity(
                "wal chunk cap reached; truncate more data before rotating",
            ));
        }

        let mut projected = self.aggregate_bytes.saturating_add(entry_total_bytes);
        if will_rotate {
            projected = projected.saturating_add(WAL_HEADER_LEN as u64);
        }

        if projected > self.options.max_wal_size {
            return Err(WalError::WalAtCapacity(
                "wal size cap exceeded; truncate finalized segments",
            ));
        }

        Ok(())
    }

    fn truncate_active_segment(
        &mut self,
        segment: &mut WalSegment,
        cursor: &WalTruncateCursor,
    ) -> WalResult<()> {
        let safe_offset = self.resolve_truncate_cursor(segment, cursor)?;
        segment.file_mut().set_len(safe_offset)?;
        let _ = segment.file_mut().seek(SeekFrom::Start(safe_offset))?;
        segment.set_len(safe_offset);
        self.recalculate_aggregate_bytes(segment.len());
        self.commit_truncate_state(cursor, safe_offset)
    }

    fn record_truncate_cursor(
        &mut self,
        segment: &mut WalSegment,
        cursor: &WalTruncateCursor,
    ) -> WalResult<()> {
        let safe_offset = self.resolve_truncate_cursor(segment, cursor)?;
        self.commit_truncate_state(cursor, safe_offset)
    }

    fn reclaim_prefix(
        &mut self,
        segment: &mut WalSegment,
        cursor: &WalTruncateCursor,
    ) -> WalResult<()> {
        let safe_offset = self.resolve_truncate_cursor(segment, cursor)?;
        self.commit_truncate_state(cursor, safe_offset)
    }

    fn resolve_truncate_cursor(
        &mut self,
        segment: &mut WalSegment,
        cursor: &WalTruncateCursor,
    ) -> WalResult<u64> {
        let requested_offset = cursor.safe_offset.max(WAL_HEADER_LEN as u64);
        let file_len = segment.file().metadata()?.len();
        if requested_offset > file_len {
            return Err(WalError::InvalidTruncateCursor(
                "safe offset beyond wal tail",
            ));
        }
        if let Some(last_seq) = self.last_safe_sequence {
            if cursor.safe_sequence < last_seq {
                return Err(WalError::InvalidTruncateCursor("safe sequence regressed"));
            }
        }
        self.ensure_entry_boundary(segment, requested_offset)?;
        Ok(requested_offset)
    }

    fn ensure_entry_boundary(&mut self, segment: &mut WalSegment, target: u64) -> WalResult<()> {
        if target == self.active_safe_offset {
            return Ok(());
        }
        if target < self.active_safe_offset {
            return Err(WalError::InvalidTruncateCursor("safe offset regressed"));
        }

        let original_pos = segment.file_mut().stream_position()?;
        let mut cursor = self.active_safe_offset;
        let _ = segment.file_mut().seek(SeekFrom::Start(cursor))?;
        while cursor < target {
            let mut len_buf = [0u8; 4];
            segment.file_mut().read_exact(&mut len_buf)?;
            let entry_len = u32::from_le_bytes(len_buf) as u64;
            let entry_total = 4u64
                .checked_add(entry_len)
                .and_then(|val| val.checked_add(4))
                .ok_or(WalError::InvalidTruncateCursor("entry length overflow"))?;
            cursor = cursor
                .checked_add(entry_total)
                .ok_or(WalError::InvalidTruncateCursor("safe offset overflow"))?;
            if cursor > target {
                let _ = segment.file_mut().seek(SeekFrom::Start(original_pos))?;
                return Err(WalError::InvalidTruncateCursor(
                    "safe offset splits entry boundary",
                ));
            }
            let _ = segment
                .file_mut()
                .seek(SeekFrom::Current(entry_len as i64 + 4))?;
        }
        let _ = segment.file_mut().seek(SeekFrom::Start(original_pos))?;
        Ok(())
    }

    fn commit_truncate_state(
        &mut self,
        cursor: &WalTruncateCursor,
        recorded_offset: u64,
    ) -> WalResult<()> {
        let global_offset = self.global_safe_offset(recorded_offset);
        if global_offset < self.truncate_state.logical_offset {
            return Err(WalError::InvalidTruncateCursor("safe offset regressed"));
        }
        self.record_logical_offset(global_offset)?;
        self.active_safe_offset = recorded_offset;
        self.last_safe_sequence = Some(cursor.safe_sequence);
        self.purge_rotated_chunks()
    }

    fn record_logical_offset(&mut self, logical_offset: u64) -> WalResult<()> {
        if self.truncate_state.logical_offset == logical_offset && self.sidecar_path.exists() {
            return Ok(());
        }
        self.truncate_state.logical_offset = logical_offset;
        TruncateSidecar::write_to(&self.sidecar_path, &self.truncate_state)
    }

    fn maybe_rotate_after_append(&mut self, segment: &mut WalSegment) -> WalResult<()> {
        let active_data_bytes = segment.len().saturating_sub(WAL_HEADER_LEN as u64);
        if active_data_bytes <= self.options.rotation_target_bytes {
            return Ok(());
        }
        self.rotate_active_file(segment)
    }

    fn rotate_active_file(&mut self, segment: &mut WalSegment) -> WalResult<()> {
        if self.rotated_chunks.len() >= self.options.max_chunks {
            return Err(WalError::WalAtCapacity(
                "wal chunk cap reached; truncate more data before rotating",
            ));
        }

        segment.flush_now()?;
        let old_len = segment.len();
        if old_len <= WAL_HEADER_LEN as u64 {
            return Ok(());
        }
        self.aggregate_bytes = self.aggregate_bytes.saturating_sub(old_len);

        shift_rotated_files(&self.options.path, self.rotated_chunks.len())?;
        self.update_rotated_chunk_paths();

        let new_chunk_path = rotated_path(&self.options.path, 1);
        std::fs::rename(&self.options.path, &new_chunk_path)?;

        let data_bytes = old_len.saturating_sub(WAL_HEADER_LEN as u64);
        let end_data_offset = self.rotated_prefix_bytes.saturating_add(data_bytes);
        self.rotated_chunks.push_back(RotatedChunk {
            path: new_chunk_path,
            file_bytes: old_len,
            end_data_offset,
        });
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(old_len);
        self.rotated_prefix_bytes = end_data_offset;

        let mut file = reopen_wal_file(&self.options.path, self.options.segment_cfg_hash)?;
        let _ = file.seek(SeekFrom::End(0))?; // ensure positioned at end
        segment.replace_file(file, WAL_HEADER_LEN as u64);
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(segment.len());
        self.active_safe_offset = WAL_HEADER_LEN as u64;

        self.truncate_state.rotation_generation =
            self.truncate_state.rotation_generation.saturating_add(1);
        TruncateSidecar::write_to(&self.sidecar_path, &self.truncate_state)?;
        Ok(())
    }

    fn update_rotated_chunk_paths(&mut self) {
        let existing = self.rotated_chunks.len();
        if existing == 0 {
            return;
        }
        for (idx, chunk) in self.rotated_chunks.iter_mut().enumerate() {
            let disk_index = existing - idx + 1;
            chunk.path = rotated_path(&self.options.path, disk_index);
        }
    }

    fn enforce_size_cap(&mut self) -> WalResult<()> {
        if self.aggregate_bytes <= self.options.max_wal_size {
            return Ok(());
        }
        Err(WalError::WalAtCapacity(
            "wal size cap exceeded; truncate finalized segments",
        ))
    }

    fn purge_rotated_chunks(&mut self) -> WalResult<()> {
        while let Some(front) = self.rotated_chunks.front() {
            if front.end_data_offset <= self.truncate_state.logical_offset {
                std::fs::remove_file(&front.path)?;
                self.aggregate_bytes = self.aggregate_bytes.saturating_sub(front.total_bytes());
                let _ = self.rotated_chunks.pop_front();
            } else {
                break;
            }
        }
        Ok(())
    }

    fn recalculate_aggregate_bytes(&mut self, active_len: u64) {
        let rotated_total: u64 = self
            .rotated_chunks
            .iter()
            .map(|chunk| chunk.total_bytes())
            .sum();
        self.aggregate_bytes = rotated_total.saturating_add(active_len);
    }

    fn global_safe_offset(&self, cursor_offset: u64) -> u64 {
        let data_offset = cursor_offset.saturating_sub(WAL_HEADER_LEN as u64);
        self.rotated_prefix_bytes.saturating_add(data_offset)
    }

    fn per_file_offset_from_global(&self, global_offset: u64, active_len: u64) -> u64 {
        let header = WAL_HEADER_LEN as u64;
        let active_data_len = active_len.saturating_sub(header);

        if global_offset <= self.rotated_prefix_bytes {
            return header;
        }

        let data_within_active = global_offset
            .saturating_sub(self.rotated_prefix_bytes)
            .min(active_data_len);
        header.saturating_add(data_within_active)
    }

    fn detect_next_sequence(&self) -> WalResult<u64> {
        let mut highest = self.scan_file_last_sequence(&self.options.path)?;
        for chunk in &self.rotated_chunks {
            if let Some(seq) = self.scan_file_last_sequence(&chunk.path)? {
                highest = Some(match highest {
                    Some(current) => current.max(seq),
                    None => seq,
                });
            }
        }
        Ok(highest.map_or(0, |seq| seq.wrapping_add(1)))
    }

    fn scan_file_last_sequence(&self, path: &Path) -> WalResult<Option<u64>> {
        if !path.exists() {
            return Ok(None);
        }
        let mut reader = WalReader::open(path)?;
        let mut iter = reader.iter_from(0)?;
        let mut last = None;
        while let Some(entry) = iter.next() {
            match entry {
                Ok(bundle) => last = Some(bundle.sequence),
                Err(WalError::UnexpectedEof(_)) | Err(WalError::InvalidEntry(_)) => {
                    break;
                }
                Err(err) => return Err(err),
            }
        }
        Ok(last)
    }
}

impl Drop for WalWriter {
    fn drop(&mut self) {
        #[cfg(test)]
        if self.test_crashed {
            return;
        }
        if self.segment.unflushed_bytes == 0 {
            return;
        }

        let _ = self.segment.flush_now();
        #[cfg(test)]
        test_support::record_drop_flush();
    }
}

#[cfg(test)]
impl WalWriter {
    pub(crate) fn test_set_last_flush(&mut self, instant: Instant) {
        self.segment.last_flush = instant;
    }

    pub(crate) fn test_last_flush(&self) -> Instant {
        self.segment.last_flush
    }

    /// Simulates a process crash by flagging Drop to skip flush/fsync logic.
    #[cfg(test)]
    pub(crate) fn test_force_crash(mut self) {
        self.test_crashed = true;
    }
}

fn system_time_to_nanos(ts: SystemTime) -> WalResult<i64> {
    let duration = ts
        .duration_since(UNIX_EPOCH)
        .map_err(|_| WalError::InvalidTimestamp)?;
    i64::try_from(duration.as_nanos()).map_err(|_| WalError::InvalidTimestamp)
}

fn encode_record_batch(batch: &RecordBatch) -> WalResult<Vec<u8>> {
    let schema = batch.schema();
    let mut buffer = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut buffer, &schema).map_err(WalError::Arrow)?;
        writer.write(batch).map_err(WalError::Arrow)?;
        writer.finish().map_err(WalError::Arrow)?;
    }
    Ok(buffer)
}

fn sync_file_data(file: &File) -> WalResult<()> {
    #[cfg(test)]
    test_support::record_sync_data();
    file.sync_data()?;
    Ok(())
}

fn truncate_sidecar_path(wal_path: &Path) -> PathBuf {
    wal_path
        .parent()
        .map(|parent| parent.join("truncate.offset"))
        .unwrap_or_else(|| PathBuf::from("truncate.offset"))
}

fn load_truncate_state(path: &Path) -> WalResult<TruncateSidecar> {
    match TruncateSidecar::read_from(path) {
        Ok(state) => Ok(state),
        Err(WalError::InvalidTruncateSidecar(_)) => Ok(default_truncate_state()),
        Err(WalError::Io(err))
            if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::UnexpectedEof) =>
        {
            Ok(default_truncate_state())
        }
        Err(err) => Err(err),
    }
}

fn default_truncate_state() -> TruncateSidecar {
    TruncateSidecar::new(0, 0)
}

fn shift_rotated_files(base_path: &Path, existing: usize) -> WalResult<()> {
    for idx in (1..=existing).rev() {
        let src = rotated_path(base_path, idx);
        let dst = rotated_path(base_path, idx + 1);
        if src.exists() {
            std::fs::rename(src, &dst)?;
        }
    }
    Ok(())
}

fn rotated_path(base_path: &Path, index: usize) -> PathBuf {
    let mut name = base_path.as_os_str().to_os_string();
    name.push(format!(".{index}"));
    PathBuf::from(name)
}

fn discover_rotated_chunk_files(base_path: &Path) -> WalResult<Vec<(usize, u64)>> {
    let mut discovered = Vec::new();
    let mut index = 1usize;
    loop {
        let path = rotated_path(base_path, index);
        match std::fs::metadata(&path) {
            Ok(metadata) => {
                discovered.push((index, metadata.len()));
                index += 1;
            }
            Err(err) if err.kind() == ErrorKind::NotFound => break,
            Err(err) => return Err(WalError::Io(err)),
        }
    }
    Ok(discovered)
}

fn reopen_wal_file(path: &Path, segment_hash: [u8; 16]) -> WalResult<File> {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    WalHeader::new(segment_hash).write_to(&mut file)?;
    Ok(file)
}

struct EncodedSlot {
    slot_id_raw: u16,
    schema_fingerprint: SchemaFingerprint,
    row_count: u32,
    payload_len: u32,
    payload_bytes: Vec<u8>,
}

/// Metadata describing an on-disk rotated chunk. We retain enough information
/// to decide when the chunk can be deleted once readers have safely advanced
/// past its logical range.
#[derive(Clone, Debug)]
struct RotatedChunk {
    path: PathBuf,
    file_bytes: u64,
    end_data_offset: u64,
}

impl RotatedChunk {
    fn total_bytes(&self) -> u64 {
        self.file_bytes
    }
}

impl EncodedSlot {
    fn serialized_size(&self) -> usize {
        SLOT_HEADER_LEN + self.payload_bytes.len()
    }

    fn write_into(self, buffer: &mut Vec<u8>) {
        let total = self.serialized_size();
        let start = buffer.len();
        buffer.resize(start + total, 0);

        let mut cursor = start;
        buffer[cursor..cursor + 2].copy_from_slice(&self.slot_id_raw.to_le_bytes());
        cursor += 2;
        buffer[cursor..cursor + SCHEMA_FINGERPRINT_LEN].copy_from_slice(&self.schema_fingerprint);
        cursor += SCHEMA_FINGERPRINT_LEN;
        buffer[cursor..cursor + 4].copy_from_slice(&self.row_count.to_le_bytes());
        cursor += 4;
        buffer[cursor..cursor + 4].copy_from_slice(&self.payload_len.to_le_bytes());
        cursor += 4;
        buffer[cursor..cursor + self.payload_bytes.len()].copy_from_slice(&self.payload_bytes);
    }
}

#[cfg(test)]
pub(super) mod test_support {
    use std::cell::Cell;

    thread_local! {
        static FLUSH_NOTIFIED: Cell<bool> = const { Cell::new(false) };
        static DROP_FLUSH_NOTIFIED: Cell<bool> = const { Cell::new(false) };
        static SYNC_DATA_NOTIFIED: Cell<bool> = const { Cell::new(false) };
        static NEXT_CRASH: Cell<Option<CrashInjection>> = const { Cell::new(None) };
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum CrashInjection {
        BeforeSidecarRename,
    }

    pub fn record_flush() {
        FLUSH_NOTIFIED.with(|cell| cell.set(true));
    }

    pub fn record_drop_flush() {
        DROP_FLUSH_NOTIFIED.with(|cell| cell.set(true));
    }

    pub fn take_drop_flush_notification() -> bool {
        DROP_FLUSH_NOTIFIED.with(|cell| {
            let notified = cell.get();
            cell.set(false);
            notified
        })
    }

    pub fn reset_flush_notifications() {
        FLUSH_NOTIFIED.with(|cell| cell.set(false));
        DROP_FLUSH_NOTIFIED.with(|cell| cell.set(false));
        SYNC_DATA_NOTIFIED.with(|cell| cell.set(false));
        NEXT_CRASH.with(|cell| cell.set(None));
    }

    pub fn record_sync_data() {
        SYNC_DATA_NOTIFIED.with(|cell| cell.set(true));
    }

    pub fn take_sync_data_notification() -> bool {
        SYNC_DATA_NOTIFIED.with(|cell| {
            let notified = cell.get();
            cell.set(false);
            notified
        })
    }

    pub fn inject_crash(point: CrashInjection) {
        NEXT_CRASH.with(|cell| cell.set(Some(point)));
    }

    pub fn take_crash(point: CrashInjection) -> bool {
        NEXT_CRASH.with(|cell| {
            if cell.get() == Some(point) {
                cell.set(None);
                true
            } else {
                false
            }
        })
    }
}
