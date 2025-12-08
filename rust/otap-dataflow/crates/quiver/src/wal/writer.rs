// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Write-ahead-log (WAL) writer for Quiver.
//!
//! The writer is responsible for appending Arrow payloads, rotating WAL
//! files when size thresholds are exceeded, and reclaiming durable space once
//! downstream consumers advance the consumer checkpoint. Safe offsets are always
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
//!                           consumer checkpoint <── sidecar flush
//! ```
//!
//! * **Append path** – Every [`RecordBundle`] is serialized slot-by-slot into an
//!   in-memory buffer, hashed (CRC32), and written as one atomic entry. We track
//!   [`WalOffset`] so readers can correlate persisted bytes back to sequences.
//! * **Flush loop** – `flush_interval` and `max_unflushed_bytes` control when we
//!   call `flush()`/`sync_data()`. Tests can inspect thread-local flags to assert
//!   that durability barriers occurred.
//! * **Rotation** – When `rotation_target_bytes` is exceeded the active WAL file
//!   is renamed to `wal.n` (oldest files slide toward higher numbers) and a new
//!   header is seeded in-place. [`RotatedWalFile`] metadata keeps track of logical
//!   data ranges so stale files can be deleted after the checkpoint advances.
//! * **Checkpoint + sidecar** – Readers compute `WalConsumerCheckpoint` values. The
//!   writer validates checkpoint boundaries, updates the checkpoint sidecar on disk,
//!   and relies on file rotation to eventually drop fully consumed bytes. The
//!   sidecar survives crashes so restarts resume from the last known safe offset
//!   even if a rotation has not run yet.
//! * **Global cap enforcement** – `max_wal_size` is measured across the active
//!   file plus rotated files. Exceeding the cap yields `WalAtCapacity` errors
//!   until downstream consumers free space.
//!
//! ## Offset coordinate systems (internal)
//!
//! > **Note for operators:** You don't need to understand this section.
//! > Just use `WalConsumerCheckpoint::advance()` to track progress.
//!
//! Internally, checkpoint metadata uses two coordinate spaces:
//!
//! | Coordinate            | Measures                          | Stored in                          |
//! |-----------------------|-----------------------------------|------------------------------------|
//! | `global_data_offset`  | Total data bytes consumed across  | `checkpoint.offset` sidecar        |
//! |                       | the entire WAL (headers excluded) | (stable across rotations)          |
//! | per-file offset       | Byte position within the active   | Derived at runtime via             |
//! |                       | WAL file (includes header)        | `to_active_file_offset()`          |
//!
//! Each [`RotatedWalFile`] stores a `cumulative_data_offset` representing the
//! global data offset at the *end* of that file. This lets `purge_rotated_files`
//! delete a file once `global_data_offset` exceeds its cumulative boundary.
//!
//! **Conversion:** `to_global_offset(file_offset)` adds data bytes from rotated
//! files to the per-file data position. `to_active_file_offset(global)` reverses
//! the transformation, clamping to the active file's bounds.
//!
//! ## Lifecycle events
//!
//! | Event               | Trigger                                              |
//! |---------------------|------------------------------------------------------|
//! | **Rotation**        | Active file exceeds `rotation_target_bytes`          |
//! | **Purge**           | `global_data_offset` passes a rotated file's cumulative end |
//! | **Backpressure**    | `aggregate_bytes > max_wal_size` or rotated file cap hit |
//!
//! ## Internal structure
//!
//! * [`WalWriter`] – the façade exposed to call sites. It orchestrates appends,
//!   compaction, and recovery by delegating to the pieces below.
//! * [`ActiveWalFile`] – owns the on-disk file handle plus the transient payload
//!   buffer. All direct I/O (seek, write, flush/fsync) flows through this type
//!   so we only mutate the OS file descriptor in one place.
//! * [`WalCoordinator`] – tracks policy decisions (rotation, global caps, and
//!   checkpoint sidecar state). It never touches raw I/O directly; instead it
//!   inspects `ActiveWalFile` lengths and requests actions.
//!
//! Splitting responsibilities this way keeps the append path easier to reason
//! about: `WalWriter` asks the coordinator for admission, streams bytes via the
//! active file, then records whatever book-keeping the coordinator dictates.
//!
//! ## Startup and recovery
//!
//! On [`WalWriter::open`]:
//!
//! 1. Read `checkpoint.offset` sidecar → recover `global_data_offset`.
//! 2. Scan for rotated files (`wal.N`) and rebuild the `rotated_files` queue
//!    with cumulative offsets, sorted by rotation id (oldest first).
//! 3. Convert `global_data_offset` → per-file offset inside the active WAL.
//! 4. Detect the highest sequence number across all files; resume from
//!    `highest + 1`.
//! 5. Position the file cursor at EOF and accept new appends.
//!
//! If the sidecar is missing or corrupt, we fall back to starting from offset
//! zero and scanning all entries.
//!
//! ## Testing hooks
//!
//! * Failpoints in `test_support` simulate crashes before the checkpoint sidecar
//!   rename so we can verify idempotent recovery.
//! * `test_force_crash` skips drop-time flushes to model abrupt exits.
//!
//! See the reader module for how [`WalConsumerCheckpoint`] values are derived
//! from WAL entries.
#![allow(dead_code)]

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use crc32fast::Hasher;

use crate::record_bundle::{PayloadRef, RecordBundle, SlotId};

use super::checkpoint_sidecar::CheckpointSidecar;
use super::header::{WAL_HEADER_LEN, WalHeader};
use super::reader::WalReader;
use super::{
    ENTRY_HEADER_LEN, ENTRY_TYPE_RECORD_BUNDLE, MAX_ROTATION_TARGET_BYTES, WalConsumerCheckpoint,
    WalError, WalResult,
};

// ---------------------------------------------------------------------------
// Default configuration constants
//
// These defaults balance durability, I/O efficiency, and recovery time. See
// ARCHITECTURE.md § "Configuration defaults and rationale" for background.
// ---------------------------------------------------------------------------

/// Default maximum aggregate size of active + rotated WAL files.
///
/// `u64::MAX` means unlimited—backpressure is only applied by
/// `max_rotated_files`.
pub const DEFAULT_MAX_WAL_SIZE: u64 = u64::MAX;

/// Default maximum number of rotated WAL files to retain.
///
/// New rotations are blocked when this limit is reached; the writer returns
/// `WalAtCapacity` until the checkpoint advances and older files are purged.
/// Eight rotated files is a reasonable trade-off: it keeps enough history for
/// slow consumers while limiting disk footprint and recovery scan time.
pub const DEFAULT_MAX_ROTATED_FILES: usize = 8;

/// Default size threshold (in bytes) that triggers WAL file rotation.
///
/// 64 MiB keeps individual files manageable for sequential scans while
/// amortizing rotation overhead (header writes, renames).
pub const DEFAULT_ROTATION_TARGET_BYTES: u64 = 64 * 1024 * 1024;

/// Controls when the WAL flushes data to disk.
///
/// Flushing calls `fsync()` to ensure data reaches stable storage. More frequent
/// flushes improve durability but reduce throughput.
#[derive(Debug, Clone, Default)]
pub(crate) enum FlushPolicy {
    /// Flush after every write. Safest but slowest.
    #[default]
    Immediate,
    /// Flush when unflushed bytes exceed the threshold.
    EveryNBytes(u64),
    /// Flush when elapsed time since last flush exceeds the duration.
    EveryDuration(Duration),
    /// Flush when either bytes or duration threshold is exceeded.
    /// This is the recommended policy for production.
    BytesOrDuration { bytes: u64, duration: Duration },
}

/// Low-level tunables for the WAL writer.
///
/// Most users should use [`WalWriterOptions::new()`] with defaults, then
/// customize via the builder methods:
///
/// ```ignore
/// let options = WalWriterOptions::new(path, hash, FlushPolicy::Immediate)
///     .with_max_wal_size(1024 * 1024 * 1024)  // 1 GB cap
///     .with_rotation_target_bytes(64 * 1024 * 1024);  // 64 MB per file
/// ```
#[derive(Debug, Clone)]
pub(crate) struct WalWriterOptions {
    /// Path to the active WAL file (e.g., `wal/quiver.wal`).
    pub path: PathBuf,
    /// Hash of segment configuration; mismatches reject existing files.
    pub segment_cfg_hash: [u8; 16],
    /// When to call `fsync()` for durability.
    pub flush_policy: FlushPolicy,
    /// Maximum aggregate size of active + rotated files (default: unlimited).
    pub max_wal_size: u64,
    /// Maximum number of rotated files to keep (default: 8).
    pub max_rotated_files: usize,
    /// Rotate the active file when it exceeds this size (default: 64 MB).
    pub rotation_target_bytes: u64,
}

impl WalWriterOptions {
    pub fn new(path: PathBuf, segment_cfg_hash: [u8; 16], flush_policy: FlushPolicy) -> Self {
        Self {
            path,
            segment_cfg_hash,
            flush_policy,
            max_wal_size: DEFAULT_MAX_WAL_SIZE,
            max_rotated_files: DEFAULT_MAX_ROTATED_FILES,
            rotation_target_bytes: DEFAULT_ROTATION_TARGET_BYTES,
        }
    }

    pub fn with_flush_policy(mut self, policy: FlushPolicy) -> Self {
        self.flush_policy = policy;
        self
    }

    pub fn with_max_wal_size(mut self, max_bytes: u64) -> Self {
        self.max_wal_size = max_bytes;
        self
    }

    pub fn with_max_rotated_files(mut self, max_files: usize) -> Self {
        self.max_rotated_files = max_files.max(1);
        self
    }

    pub fn with_rotation_target(mut self, target_bytes: u64) -> Self {
        self.rotation_target_bytes = target_bytes.clamp(1, MAX_ROTATION_TARGET_BYTES);
        self
    }
}

/// Stateful writer that maintains append position, rotation metadata, and
/// persisted consumer checkpoints. It tracks both the *current* WAL file and
/// any rotated files so the total on-disk footprint (`aggregate_bytes`) can be
/// compared against caps before admitting new entries.
#[derive(Debug)]
pub(crate) struct WalWriter {
    /// Encapsulates the active WAL file handle plus its in-memory staging
    /// buffer. All raw I/O flows through this helper.
    active_file: ActiveWalFile,
    /// Manages policy, checkpoint bookkeeping, and rotation metadata.
    coordinator: WalCoordinator,
    /// Next sequence number to assign to an appended entry.
    next_sequence: u64,
    #[cfg(test)]
    /// When true we skip drop-time flushes to simulate a crash.
    test_crashed: bool,
}

#[derive(Debug)]
struct ActiveWalFile {
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
    /// Path to the checkpoint sidecar file on disk.
    sidecar_path: PathBuf,
    /// Cached copy of the persisted checkpoint offset + rotation generation.
    checkpoint_state: CheckpointSidecar,
    /// Total bytes across the active WAL plus all rotated files.
    aggregate_bytes: u64,
    /// Metadata describing each rotated `wal.N` file on disk, ordered oldest-to-newest.
    rotated_files: VecDeque<RotatedWalFile>,
    /// Next rotation id to use when rotating (initialized to max existing + 1).
    next_rotation_id: u64,
    /// Most recent offset validated to be on an entry boundary in the active file.
    active_file_checkpoint_offset: u64,
    /// Sequence number associated with the last committed consumer checkpoint.
    last_checkpoint_sequence: Option<u64>,
}

/// Opaque marker returned after an append.
///
/// Contains the byte position and sequence number of the written entry.
/// Useful for correlating writer positions with reader cursors during testing
/// or debugging. Operators typically don't need to inspect these values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WalOffset {
    /// Byte offset where the entry starts in the active WAL file.
    pub position: u64,
    /// Monotonically increasing sequence number assigned to this entry.
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
        let is_new_file = metadata.len() == 0;

        if is_new_file {
            let header = WalHeader::new(options.segment_cfg_hash);
            header.write_to(&mut file)?;
            file.flush()?;
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
        }

        let sidecar_path = checkpoint_sidecar_path(&options.path);
        let checkpoint_state = load_checkpoint_state(&sidecar_path)?;

        // Create coordinator first to scan for valid entries
        let mut coordinator = WalCoordinator::new(
            options,
            sidecar_path,
            checkpoint_state,
            WAL_HEADER_LEN as u64,
        );
        coordinator.reload_rotated_files(metadata.len())?;

        // Scan to find the last valid entry and truncate any trailing garbage
        let (next_sequence, valid_offset) = coordinator.detect_next_sequence()?;
        let current_file_len = file.metadata()?.len();

        if valid_offset < current_file_len {
            // Truncate trailing garbage from a partial write (e.g., crash mid-write)
            file.set_len(valid_offset)?;
            file.sync_all()?;
        }

        // Position at the end of valid data
        let _ = file.seek(SeekFrom::Start(valid_offset))?;

        let active_file = ActiveWalFile::new(file, valid_offset);
        coordinator.restore_checkpoint_offsets(active_file.len());
        coordinator.recalculate_aggregate_bytes(active_file.len());

        Ok(Self {
            active_file,
            coordinator,
            next_sequence,
            #[cfg(test)]
            test_crashed: false,
        })
    }

    /// Serializes a [`RecordBundle`] into the active WAL file and returns the
    /// byte offset + sequence number associated with the entry. The writer keeps
    /// internal counters so the next call knows when to flush, rotate, or apply
    /// global caps.
    pub fn append_bundle<B: RecordBundle>(&mut self, bundle: &B) -> WalResult<WalOffset> {
        let descriptor = bundle.descriptor();
        let ingestion_time = bundle.ingestion_time();
        let ingestion_ts_nanos = system_time_to_nanos(ingestion_time)?;

        self.active_file.payload_buffer.clear();

        let sequence = self.next_sequence;

        let mut slot_bitmap = 0u64;

        // Single pass: validate slots and write directly to payload_buffer.
        // The buffer is reused across calls, so after a few appends it stabilizes
        // at a capacity that fits typical bundles without reallocation.
        for slot in &descriptor.slots {
            let slot_index = slot.id.0 as usize;
            if slot_index >= 64 {
                return Err(WalError::SlotOutOfRange(slot.id));
            }
            if let Some(payload) = bundle.payload(slot.id) {
                slot_bitmap |= 1u64 << slot_index;
                self.encode_slot_into_buffer(slot.id, payload)?;
            }
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

        let entry_body_len = ENTRY_HEADER_LEN + self.active_file.payload_buffer.len();
        let entry_len =
            u32::try_from(entry_body_len).map_err(|_| WalError::EntryTooLarge(entry_body_len))?;

        let mut hasher = Hasher::new();
        hasher.update(&entry_header);
        hasher.update(&self.active_file.payload_buffer);
        let crc = hasher.finalize();

        let entry_total_bytes = 4u64 + u64::from(entry_len) + 4;
        self.coordinator
            .preflight_append(&self.active_file, entry_total_bytes)?;

        let mut payload_bytes = std::mem::take(&mut self.active_file.payload_buffer);
        let entry_start =
            self.active_file
                .write_entry(entry_len, &entry_header, &payload_bytes, crc)?;
        payload_bytes.clear();
        self.active_file.payload_buffer = payload_bytes;

        self.next_sequence = self.next_sequence.wrapping_add(1);

        self.active_file.current_len = self
            .active_file
            .current_len
            .saturating_add(entry_total_bytes);
        self.coordinator.record_append(entry_total_bytes);
        self.active_file
            .maybe_flush(&self.coordinator.options().flush_policy, entry_total_bytes)?;

        self.coordinator
            .maybe_rotate_after_append(&mut self.active_file)?;

        Ok(WalOffset {
            position: entry_start,
            sequence,
        })
    }

    /// Persists the cursor position and enables cleanup of consumed WAL data.
    ///
    /// This validates the cursor, writes to the checkpoint sidecar with fsync,
    /// and purges any rotated files fully covered by the new position.
    ///
    /// Call this after downstream has confirmed durability (e.g., segment flush).
    pub(crate) fn checkpoint_cursor(
        &mut self,
        checkpoint: &WalConsumerCheckpoint,
    ) -> WalResult<()> {
        self.coordinator
            .checkpoint_cursor(&mut self.active_file, checkpoint)
    }

    /// Encodes a slot directly into `payload_buffer`, avoiding intermediate allocations.
    ///
    /// Writes the slot header (id, fingerprint, row_count, payload_len) followed by
    /// the Arrow IPC-encoded payload bytes.
    fn encode_slot_into_buffer(&mut self, slot_id: SlotId, payload: PayloadRef<'_>) -> WalResult<()> {
        let row_count = u32::try_from(payload.batch.num_rows())
            .map_err(|_| WalError::RowCountOverflow(payload.batch.num_rows()))?;
        let payload_bytes = encode_record_batch(payload.batch)?;
        let payload_len = u32::try_from(payload_bytes.len())
            .map_err(|_| WalError::PayloadTooLarge(payload_bytes.len()))?;

        let buf = &mut self.active_file.payload_buffer;

        // Write slot header
        buf.extend_from_slice(&slot_id.0.to_le_bytes());
        buf.extend_from_slice(&payload.schema_fingerprint);
        buf.extend_from_slice(&row_count.to_le_bytes());
        buf.extend_from_slice(&payload_len.to_le_bytes());

        // Write payload
        buf.extend_from_slice(&payload_bytes);

        Ok(())
    }
}

impl ActiveWalFile {
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

    fn set_len(&mut self, len: u64) {
        self.current_len = len;
    }

    fn replace_file(&mut self, file: File, new_len: u64) {
        self.file = file;
        self.current_len = new_len;
        self.last_flush = Instant::now();
        self.unflushed_bytes = 0;
    }

    /// Writes a complete WAL entry to disk and returns the starting byte offset.
    ///
    /// Entry layout:
    /// ```text
    /// ┌──────────┬──────────────┬─────────────────┬──────────┐
    /// │ u32 len  │ entry header │  payload bytes  │ u32 crc  │
    /// └──────────┴──────────────┴─────────────────┴──────────┘
    /// ```
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

    fn maybe_flush(&mut self, policy: &FlushPolicy, bytes_written: u64) -> WalResult<()> {
        self.unflushed_bytes = self.unflushed_bytes.saturating_add(bytes_written);

        match policy {
            FlushPolicy::Immediate => self.flush_now(),
            FlushPolicy::EveryNBytes(threshold) => {
                if self.unflushed_bytes >= *threshold {
                    self.flush_now()
                } else {
                    Ok(())
                }
            }
            FlushPolicy::EveryDuration(interval) => {
                if self.last_flush.elapsed() >= *interval {
                    self.flush_now()
                } else {
                    Ok(())
                }
            }
            FlushPolicy::BytesOrDuration { bytes, duration } => {
                if self.unflushed_bytes >= *bytes || self.last_flush.elapsed() >= *duration {
                    self.flush_now()
                } else {
                    Ok(())
                }
            }
        }
    }

    fn flush_now(&mut self) -> WalResult<()> {
        self.file.flush()?;
        sync_file_data(&self.file)?;
        self.last_flush = Instant::now();
        self.unflushed_bytes = 0;
        Ok(())
    }
}

impl WalCoordinator {
    fn new(
        options: WalWriterOptions,
        sidecar_path: PathBuf,
        checkpoint_state: CheckpointSidecar,
        current_len: u64,
    ) -> Self {
        Self {
            options,
            sidecar_path,
            checkpoint_state,
            aggregate_bytes: current_len,
            rotated_files: VecDeque::new(),
            next_rotation_id: 1,
            active_file_checkpoint_offset: WAL_HEADER_LEN as u64,
            last_checkpoint_sequence: None,
        }
    }

    /// Returns the total data bytes (excluding headers) across all rotated files.
    /// Derived from the last rotated file's cumulative offset.
    fn rotated_data_bytes(&self) -> u64 {
        self.rotated_files
            .back()
            .map_or(0, |f| f.cumulative_data_offset)
    }

    fn options(&self) -> &WalWriterOptions {
        &self.options
    }

    fn reload_rotated_files(&mut self, active_len: u64) -> WalResult<()> {
        let discovered = discover_rotated_wal_files(&self.options.path)?;
        if discovered.is_empty() {
            self.aggregate_bytes = active_len;
            self.rotated_files.clear();
            self.next_rotation_id = 1;
            return Ok(());
        }

        // Files are returned sorted oldest-to-newest by rotation_id
        let mut files = VecDeque::with_capacity(discovered.len());
        let mut aggregate = active_len;
        let mut cumulative_data = 0u64;
        let mut max_rotation_id = 0u64;

        for (rotation_id, len) in &discovered {
            aggregate = aggregate.saturating_add(*len);
            let data_bytes = len.saturating_sub(WAL_HEADER_LEN as u64);
            cumulative_data = cumulative_data.saturating_add(data_bytes);
            files.push_back(RotatedWalFile {
                path: rotated_wal_path(&self.options.path, *rotation_id),
                rotation_id: *rotation_id,
                file_bytes: *len,
                cumulative_data_offset: cumulative_data,
            });
            max_rotation_id = max_rotation_id.max(*rotation_id);
        }

        self.rotated_files = files;
        self.aggregate_bytes = aggregate;
        self.next_rotation_id = max_rotation_id.saturating_add(1);
        Ok(())
    }

    fn restore_checkpoint_offsets(&mut self, active_len: u64) {
        let header = WAL_HEADER_LEN as u64;
        let active_data = active_len.saturating_sub(header);
        let total_logical = self.rotated_data_bytes().saturating_add(active_data);

        if self.checkpoint_state.global_data_offset > total_logical {
            self.checkpoint_state.global_data_offset = total_logical;
        }

        self.active_file_checkpoint_offset =
            self.to_active_file_offset(self.checkpoint_state.global_data_offset, active_len);
    }

    fn record_append(&mut self, entry_total_bytes: u64) {
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(entry_total_bytes);
    }

    fn preflight_append(
        &mut self,
        active_file: &ActiveWalFile,
        entry_total_bytes: u64,
    ) -> WalResult<()> {
        let will_rotate = active_file
            .len()
            .saturating_add(entry_total_bytes)
            .saturating_sub(WAL_HEADER_LEN as u64)
            > self.options.rotation_target_bytes;

        if will_rotate && self.rotated_files.len() >= self.options.max_rotated_files {
            return Err(WalError::WalAtCapacity(
                "rotated wal file cap reached; advance checkpoint before rotating",
            ));
        }

        let mut projected = self.aggregate_bytes.saturating_add(entry_total_bytes);
        if will_rotate {
            projected = projected.saturating_add(WAL_HEADER_LEN as u64);
        }

        if projected > self.options.max_wal_size {
            return Err(WalError::WalAtCapacity(
                "wal size cap exceeded; advance checkpoint to reclaim space",
            ));
        }

        Ok(())
    }

    fn checkpoint_cursor(
        &mut self,
        active_file: &mut ActiveWalFile,
        checkpoint: &WalConsumerCheckpoint,
    ) -> WalResult<()> {
        let safe_offset = self.resolve_consumer_checkpoint(active_file, checkpoint)?;
        self.persist_checkpoint(checkpoint, safe_offset)
    }

    fn resolve_consumer_checkpoint(
        &mut self,
        active_file: &mut ActiveWalFile,
        checkpoint: &WalConsumerCheckpoint,
    ) -> WalResult<u64> {
        let requested_offset = checkpoint.safe_offset.max(WAL_HEADER_LEN as u64);
        let file_len = active_file.file_mut().metadata()?.len();
        if requested_offset > file_len {
            return Err(WalError::InvalidConsumerCheckpoint(
                "safe offset beyond wal tail",
            ));
        }
        if let Some(last_seq) = self.last_checkpoint_sequence {
            if checkpoint.safe_sequence < last_seq {
                return Err(WalError::InvalidConsumerCheckpoint(
                    "safe sequence regressed",
                ));
            }
        }
        self.ensure_entry_boundary(active_file, requested_offset)?;
        Ok(requested_offset)
    }

    fn ensure_entry_boundary(
        &mut self,
        active_file: &mut ActiveWalFile,
        target: u64,
    ) -> WalResult<()> {
        if target == self.active_file_checkpoint_offset {
            return Ok(());
        }
        if target < self.active_file_checkpoint_offset {
            return Err(WalError::InvalidConsumerCheckpoint("safe offset regressed"));
        }

        let original_pos = active_file.file_mut().stream_position()?;
        let mut cursor = self.active_file_checkpoint_offset;
        let _ = active_file.file_mut().seek(SeekFrom::Start(cursor))?;
        while cursor < target {
            let mut len_buf = [0u8; 4];
            active_file.file_mut().read_exact(&mut len_buf)?;
            let entry_len = u32::from_le_bytes(len_buf) as u64;
            let entry_total = 4u64
                .checked_add(entry_len)
                .and_then(|val| val.checked_add(4))
                .ok_or(WalError::InvalidConsumerCheckpoint("entry length overflow"))?;
            cursor = cursor
                .checked_add(entry_total)
                .ok_or(WalError::InvalidConsumerCheckpoint("safe offset overflow"))?;
            if cursor > target {
                let _ = active_file.file_mut().seek(SeekFrom::Start(original_pos))?;
                return Err(WalError::InvalidConsumerCheckpoint(
                    "safe offset splits entry boundary",
                ));
            }
            let _ = active_file
                .file_mut()
                .seek(SeekFrom::Current(entry_len as i64 + 4))?;
        }
        let _ = active_file.file_mut().seek(SeekFrom::Start(original_pos))?;
        Ok(())
    }

    fn persist_checkpoint(
        &mut self,
        checkpoint: &WalConsumerCheckpoint,
        recorded_offset: u64,
    ) -> WalResult<()> {
        let global_offset = self.to_global_offset(recorded_offset);
        if global_offset < self.checkpoint_state.global_data_offset {
            return Err(WalError::InvalidConsumerCheckpoint("safe offset regressed"));
        }
        self.record_global_offset(global_offset)?;
        self.active_file_checkpoint_offset = recorded_offset;
        self.last_checkpoint_sequence = Some(checkpoint.safe_sequence);
        self.purge_rotated_files()
    }

    fn record_global_offset(&mut self, global_offset: u64) -> WalResult<()> {
        if self.checkpoint_state.global_data_offset == global_offset && self.sidecar_path.exists() {
            return Ok(());
        }
        self.checkpoint_state.global_data_offset = global_offset;
        CheckpointSidecar::write_to(&self.sidecar_path, &self.checkpoint_state)
    }

    fn maybe_rotate_after_append(&mut self, active_file: &mut ActiveWalFile) -> WalResult<()> {
        let active_data_bytes = active_file.len().saturating_sub(WAL_HEADER_LEN as u64);
        if active_data_bytes <= self.options.rotation_target_bytes {
            return Ok(());
        }
        self.rotate_active_file(active_file)
    }

    fn rotate_active_file(&mut self, active_file: &mut ActiveWalFile) -> WalResult<()> {
        if self.rotated_files.len() >= self.options.max_rotated_files {
            return Err(WalError::WalAtCapacity(
                "rotated wal file cap reached; advance checkpoint before rotating",
            ));
        }

        active_file.flush_now()?;
        let old_len = active_file.len();
        if old_len <= WAL_HEADER_LEN as u64 {
            return Ok(());
        }
        self.aggregate_bytes = self.aggregate_bytes.saturating_sub(old_len);

        // Use monotonic naming: rename to wal.{next_rotation_id}
        let rotation_id = self.next_rotation_id;
        self.next_rotation_id = self.next_rotation_id.saturating_add(1);

        let new_rotated_path = rotated_wal_path(&self.options.path, rotation_id);
        std::fs::rename(&self.options.path, &new_rotated_path)?;
        sync_parent_dir(&self.options.path)?;

        let data_bytes = old_len.saturating_sub(WAL_HEADER_LEN as u64);
        let cumulative_data_offset = self.rotated_data_bytes().saturating_add(data_bytes);
        self.rotated_files.push_back(RotatedWalFile {
            path: new_rotated_path,
            rotation_id,
            file_bytes: old_len,
            cumulative_data_offset,
        });
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(old_len);

        let mut file = reopen_wal_file(&self.options.path, self.options.segment_cfg_hash)?;
        let _ = file.seek(SeekFrom::End(0))?; // ensure positioned at end
        active_file.replace_file(file, WAL_HEADER_LEN as u64);
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(active_file.len());
        self.active_file_checkpoint_offset = WAL_HEADER_LEN as u64;

        CheckpointSidecar::write_to(&self.sidecar_path, &self.checkpoint_state)?;
        Ok(())
    }

    fn purge_rotated_files(&mut self) -> WalResult<()> {
        while let Some(front) = self.rotated_files.front() {
            if front.cumulative_data_offset <= self.checkpoint_state.global_data_offset {
                std::fs::remove_file(&front.path)?;
                self.aggregate_bytes = self.aggregate_bytes.saturating_sub(front.total_bytes());
                let _ = self.rotated_files.pop_front();
            } else {
                break;
            }
        }
        Ok(())
    }

    fn recalculate_aggregate_bytes(&mut self, active_len: u64) {
        let rotated_total: u64 = self.rotated_files.iter().map(|f| f.total_bytes()).sum();
        self.aggregate_bytes = rotated_total.saturating_add(active_len);
    }

    fn to_global_offset(&self, file_offset: u64) -> u64 {
        let data_offset = file_offset.saturating_sub(WAL_HEADER_LEN as u64);
        self.rotated_data_bytes().saturating_add(data_offset)
    }

    fn to_active_file_offset(&self, global_offset: u64, active_len: u64) -> u64 {
        let header = WAL_HEADER_LEN as u64;
        let active_data_len = active_len.saturating_sub(header);
        let rotated_data = self.rotated_data_bytes();

        if global_offset <= rotated_data {
            return header;
        }

        let data_within_active = global_offset
            .saturating_sub(rotated_data)
            .min(active_data_len);
        header.saturating_add(data_within_active)
    }

    /// Determines the next sequence number and last valid offset in the active file.
    ///
    /// Since sequence numbers are monotonically increasing, the highest sequence
    /// is always in the active file (or the most recent rotated file if the active
    /// file is empty after a rotation). Returns (next_sequence, active_file_valid_offset).
    fn detect_next_sequence(&self) -> WalResult<(u64, u64)> {
        let (active_seq, active_valid_offset) = self.scan_file_last_sequence(&self.options.path)?;

        // If active file has entries, use its sequence
        if let Some(seq) = active_seq {
            return Ok((seq.wrapping_add(1), active_valid_offset));
        }

        // Active file is empty - check the most recent rotated file (highest rotation_id)
        if let Some(most_recent) = self.rotated_files.iter().max_by_key(|f| f.rotation_id) {
            let (seq, _) = self.scan_file_last_sequence(&most_recent.path)?;
            if let Some(s) = seq {
                return Ok((s.wrapping_add(1), active_valid_offset));
            }
        }

        // No entries anywhere - start at sequence 0
        Ok((0, active_valid_offset))
    }

    /// Scans a WAL file and returns the last valid sequence number and the
    /// byte offset immediately after the last valid entry (i.e., where new
    /// writes should begin).
    fn scan_file_last_sequence(&self, path: &Path) -> WalResult<(Option<u64>, u64)> {
        if !path.exists() {
            return Ok((None, 0));
        }
        let mut reader = WalReader::open(path)?;
        let iter = reader.iter_from(0)?;
        let mut last_seq = None;
        let mut last_valid_offset = WAL_HEADER_LEN as u64;
        for entry in iter {
            match entry {
                Ok(bundle) => {
                    last_seq = Some(bundle.sequence);
                    last_valid_offset = bundle.next_offset;
                }
                Err(WalError::UnexpectedEof(_)) | Err(WalError::InvalidEntry(_)) => {
                    // Partial or corrupted entry - stop here
                    break;
                }
                Err(err) => return Err(err),
            }
        }
        Ok((last_seq, last_valid_offset))
    }
}

impl Drop for WalWriter {
    fn drop(&mut self) {
        #[cfg(test)]
        if self.test_crashed {
            return;
        }
        if self.active_file.unflushed_bytes == 0 {
            return;
        }

        let _ = self.active_file.flush_now();
        #[cfg(test)]
        test_support::record_drop_flush();
    }
}

#[cfg(test)]
impl WalWriter {
    pub(crate) fn test_set_last_flush(&mut self, instant: Instant) {
        self.active_file.last_flush = instant;
    }

    pub(crate) fn test_last_flush(&self) -> Instant {
        self.active_file.last_flush
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

/// Syncs the parent directory to ensure rename durability.
///
/// On POSIX systems, `rename()` is atomic but not necessarily durable until the
/// parent directory is fsynced. This matters on filesystems without automatic
/// rename barriers (older ext3, NFS, non-default mount options).
///
/// On non-Unix platforms this is a no-op since directory sync semantics differ.
pub(super) fn sync_parent_dir(path: &Path) -> WalResult<()> {
    #[cfg(unix)]
    {
        if let Some(parent) = path.parent() {
            let dir = File::open(parent)?;
            dir.sync_all()?;
        }
    }
    #[cfg(not(unix))]
    let _ = path; // silence unused warning
    Ok(())
}

fn checkpoint_sidecar_path(wal_path: &Path) -> PathBuf {
    wal_path
        .parent()
        .map(|parent| parent.join("checkpoint.offset"))
        .unwrap_or_else(|| PathBuf::from("checkpoint.offset"))
}

fn load_checkpoint_state(path: &Path) -> WalResult<CheckpointSidecar> {
    match CheckpointSidecar::read_from(path) {
        Ok(state) => Ok(state),
        Err(WalError::InvalidCheckpointSidecar(_)) => Ok(default_checkpoint_state()),
        Err(WalError::Io(err))
            if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::UnexpectedEof) =>
        {
            Ok(default_checkpoint_state())
        }
        Err(err) => Err(err),
    }
}

fn default_checkpoint_state() -> CheckpointSidecar {
    CheckpointSidecar::new(0)
}

fn rotated_wal_path(base_path: &Path, rotation_id: u64) -> PathBuf {
    let mut name = base_path.as_os_str().to_os_string();
    name.push(format!(".{rotation_id}"));
    PathBuf::from(name)
}

/// Discovers rotated WAL files by scanning the directory for files matching
/// `<base>.N` pattern. Returns a list of (rotation_id, file_size) tuples
/// sorted by rotation_id (oldest first).
fn discover_rotated_wal_files(base_path: &Path) -> WalResult<Vec<(u64, u64)>> {
    let parent = base_path.parent().ok_or_else(|| {
        WalError::Io(std::io::Error::new(
            ErrorKind::InvalidInput,
            "WAL path has no parent directory",
        ))
    })?;
    let base_name = base_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            WalError::Io(std::io::Error::new(
                ErrorKind::InvalidInput,
                "WAL path has invalid filename",
            ))
        })?;

    let prefix = format!("{base_name}.");
    let mut discovered = Vec::new();

    let entries = match std::fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(discovered),
        Err(err) => return Err(WalError::Io(err)),
    };

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(name_str) = file_name.to_str() else {
            continue;
        };
        let Some(suffix) = name_str.strip_prefix(&prefix) else {
            continue;
        };
        let Ok(rotation_id) = suffix.parse::<u64>() else {
            continue;
        };
        let metadata = entry.metadata()?;
        discovered.push((rotation_id, metadata.len()));
    }

    // Sort by rotation_id (oldest first)
    discovered.sort_by_key(|(id, _)| *id);
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

/// Metadata describing an on-disk rotated WAL file. We retain enough information
/// to decide when the file can be deleted once readers have safely advanced
/// past its logical range.
#[derive(Clone, Debug)]
struct RotatedWalFile {
    path: PathBuf,
    /// Monotonic rotation id for this file (extracted from the suffix, e.g., wal.5 has id=5).
    rotation_id: u64,
    file_bytes: u64,
    /// Global data offset at the *end* of this file (cumulative across older
    /// rotated files). When `global_data_offset >= cumulative_data_offset` the
    /// file is fully consumed and can be deleted.
    cumulative_data_offset: u64,
}

impl RotatedWalFile {
    fn total_bytes(&self) -> u64 {
        self.file_bytes
    }
}

#[cfg(test)]
pub(super) mod test_support {
    use std::cell::Cell;

    thread_local! {
        static DROP_FLUSH_NOTIFIED: Cell<bool> = const { Cell::new(false) };
        static SYNC_DATA_NOTIFIED: Cell<bool> = const { Cell::new(false) };
        static NEXT_CRASH: Cell<Option<CrashInjection>> = const { Cell::new(None) };
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum CrashInjection {
        BeforeSidecarRename,
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
