// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Write-ahead-log (WAL) writer for Quiver.
//!
//! The writer is responsible for appending Arrow payloads, rotating WAL
//! files when size thresholds are exceeded, and reclaiming durable space once
//! downstream consumers advance the consumer cursor. Safe offsets are always
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
//!                              consumer cursor <── sidecar flush
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
//!   data ranges so stale files can be deleted after the cursor advances.
//! * **Cursor + sidecar** – Readers compute `WalConsumerCursor` values. The
//!   writer validates cursor boundaries, updates the cursor sidecar on disk,
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
//! > Just use `WalConsumerCursor::advance()` to track progress.
//!
//! Internally, cursor metadata uses two coordinate spaces:
//!
//! | Coordinate            | Measures                          | Stored in                          |
//! |-----------------------|-----------------------------------|------------------------------------|
//! | `wal_position`        | Position in the logical WAL       | `quiver.wal.cursor` sidecar        |
//! |                       | stream (stable across rotations)  |                                    |
//! | file offset           | Byte position within the active   | Derived at runtime via             |
//! |                       | WAL file (includes header)        | `to_file_offset()`                 |
//!
//! Each [`RotatedWalFile`] stores a `wal_position_end` representing the
//! WAL stream position at the *end* of that file. This lets `purge_rotated_files`
//! delete a file once `wal_position` exceeds its boundary.
//!
//! **Conversion:** `to_wal_position(file_offset)` adds data bytes from rotated
//! files to the per-file data position. `to_file_offset(wal_position)` reverses
//! the transformation, clamping to the active file's bounds.
//!
//! ## Lifecycle events
//!
//! | Event               | Trigger                                              |
//! |---------------------|------------------------------------------------------|
//! | **Rotation**        | Active file exceeds `rotation_target_bytes`          |
//! | **Purge**           | `wal_position` passes a rotated file's end           |
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
//!   cursor sidecar state). It never touches raw I/O directly; instead it
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
//! 1. Read `quiver.wal.cursor` sidecar → recover `wal_position`.
//! 2. Scan for rotated files (`wal.N`) and rebuild the `rotated_files` queue
//!    with WAL positions, sorted by rotation id (oldest first).
//! 3. Convert `wal_position` → per-file offset inside the active WAL.
//! 4. Detect the highest sequence number across all files; resume from
//!    `highest + 1`.
//! 5. Position the file cursor at EOF and accept new appends.
//!
//! If the sidecar is missing or corrupt, we fall back to starting from offset
//! zero and scanning all entries.
//!
//! ## Testing hooks
//!
//! * Failpoints in `test_support` simulate crashes before the cursor sidecar
//!   rename so we can verify idempotent recovery.
//! * `test_force_crash` skips drop-time flushes to model abrupt exits.
//!
//! See the reader module for how [`WalConsumerCursor`] values are derived
//! from WAL entries.
#![allow(dead_code)]

use std::collections::VecDeque;
use std::io::{ErrorKind, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use arrow_ipc::writer::StreamWriter;
use crc32fast::Hasher;

use crate::budget::DiskBudget;
use crate::logging::{otel_debug, otel_error, otel_info, otel_warn};
use crate::record_bundle::{PayloadRef, RecordBundle, SlotId};

use super::cursor_sidecar::CursorSidecar;
use super::header::{WAL_HEADER_MIN_LEN, WalHeader};
use super::reader::WalReader;
use super::{
    ENTRY_HEADER_LEN, ENTRY_TYPE_RECORD_BUNDLE, MAX_ROTATION_TARGET_BYTES, WalConsumerCursor,
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

/// Minimum buffer capacity (bytes) below which we skip shrinking logic.
///
/// Shrinking tiny buffers isn't worth the bookkeeping overhead.
const SHRINK_THRESHOLD: usize = 64 * 1024; // 64 KB

/// Headroom added to high-water mark when shrinking the payload buffer.
///
/// Provides slack so minor fluctuations don't trigger immediate reallocation.
const SHRINK_HEADROOM: usize = 4 * 1024; // 4 KB

/// Default decay rate for the payload buffer high-water mark.
///
/// Expressed as a fraction (numerator, denominator). After each append,
/// `high_water = high_water * numerator / denominator`. The default (15, 16)
/// gives ~6% decay per append, meaning ~37 appends to decay to 10% of peak.
///
/// Slower decay (e.g., 31/32) holds memory longer but is safer against thrashing.
/// Faster decay (e.g., 7/8) reclaims memory faster but may reallocate more often.
pub const DEFAULT_BUFFER_DECAY_RATE: (usize, usize) = (15, 16);

/// Default maximum number of rotated WAL files to retain.
///
/// New rotations are blocked when this limit is reached; the writer returns
/// `WalAtCapacity` until the cursor advances and older files are purged.
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
    /// Decay rate for the payload buffer high-water mark (numerator, denominator).
    ///
    /// Controls how quickly the buffer shrinks after a spike in usage.
    /// Default is (15, 16) for ~6% decay per append.
    pub buffer_decay_rate: (usize, usize),
    /// Optional shared disk budget for tracking WAL usage alongside segments.
    ///
    /// When provided, WAL bytes are recorded in this budget, enabling unified
    /// disk capacity management across all engine components.
    pub budget: Option<Arc<DiskBudget>>,
    /// Whether to set rotated WAL files to read-only after rotation.
    ///
    /// When `true` (the default), rotated WAL files are `chmod`-ed to `0o440`
    /// (Unix) or marked read-only (other platforms) to prevent accidental
    /// corruption.  When `false`, the permission change is skipped — this is
    /// necessary on filesystems that do not support `chmod` (e.g., SMB/CIFS
    /// mounts, certain Kubernetes volumeMounts).
    pub enforce_file_readonly: bool,
}

impl WalWriterOptions {
    pub const fn new(path: PathBuf, segment_cfg_hash: [u8; 16], flush_policy: FlushPolicy) -> Self {
        Self {
            path,
            segment_cfg_hash,
            flush_policy,
            max_wal_size: DEFAULT_MAX_WAL_SIZE,
            max_rotated_files: DEFAULT_MAX_ROTATED_FILES,
            rotation_target_bytes: DEFAULT_ROTATION_TARGET_BYTES,
            buffer_decay_rate: DEFAULT_BUFFER_DECAY_RATE,
            budget: None,
            enforce_file_readonly: true,
        }
    }

    pub const fn with_flush_policy(mut self, policy: FlushPolicy) -> Self {
        self.flush_policy = policy;
        self
    }

    pub const fn with_max_wal_size(mut self, max_bytes: u64) -> Self {
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

    /// Sets the shared disk budget for unified capacity tracking.
    ///
    /// When provided, WAL bytes are recorded in this budget, allowing the
    /// `DiskBudget` to track total disk usage across WAL files, segments,
    /// and other engine components.
    pub fn with_budget(mut self, budget: Arc<DiskBudget>) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Controls whether rotated WAL files are set to read-only.
    ///
    /// Pass `false` on filesystems that don't support `chmod`
    /// (e.g., certain Kubernetes volumeMounts, Azure Files SMB mounts).
    pub const fn with_enforce_file_readonly(mut self, enforce: bool) -> Self {
        self.enforce_file_readonly = enforce;
        self
    }

    /// Sets the decay rate for the payload buffer high-water mark.
    ///
    /// Expressed as a fraction (numerator, denominator). After each append,
    /// `high_water = high_water * numerator / denominator`.
    ///
    /// - Slower decay (e.g., 31/32): holds memory longer, safer against thrashing
    /// - Faster decay (e.g., 7/8): reclaims memory faster, may reallocate more
    ///
    /// Default is (15, 16) for ~6% decay per append.
    ///
    /// # Errors
    ///
    /// Returns `InvalidConfig` from `WalWriter::open()` if:
    /// - `denominator` is zero
    /// - `numerator >= denominator` (no decay would occur)
    pub const fn with_buffer_decay_rate(mut self, numerator: usize, denominator: usize) -> Self {
        self.buffer_decay_rate = (numerator, denominator);
        self
    }

    /// Validates the configuration, returning an error if any values are invalid.
    fn validate(&self) -> WalResult<()> {
        let (numerator, denominator) = self.buffer_decay_rate;
        if denominator == 0 {
            otel_error!("quiver.wal.init", reason = "invalid_config",);
            return Err(WalError::InvalidConfig(
                "buffer_decay_rate denominator must be positive",
            ));
        }
        if numerator >= denominator {
            otel_error!(
                "quiver.wal.init",
                numerator,
                denominator,
                reason = "invalid_config",
                message = "numerator must be less than denominator for decay",
            );
            return Err(WalError::InvalidConfig(
                "buffer_decay_rate numerator must be less than denominator for decay",
            ));
        }
        Ok(())
    }
}

/// Stateful writer that maintains append position, rotation metadata, and
/// persisted consumer cursors. It tracks both the *current* WAL file and
/// any rotated files so the total on-disk footprint (`aggregate_bytes`) can be
/// compared against caps before admitting new entries.
#[derive(Debug)]
pub(crate) struct WalWriter {
    /// Encapsulates the active WAL file handle plus its in-memory staging
    /// buffer. All raw I/O flows through this helper.
    active_file: ActiveWalFile,
    /// Manages policy, cursor bookkeeping, and rotation metadata.
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
    ///
    /// Wrapped in `Option` to allow temporary ownership transfer during
    /// synchronous flush in `Drop` (where we convert to std::fs::File).
    /// Always `Some` during normal operation; only `None` transiently.
    file: Option<File>,
    /// Scratch buffer used to serialize slot payloads before writing.
    payload_buffer: Vec<u8>,
    /// Scratch buffer for building complete WAL entries before writing.
    /// This allows a single write syscall per entry instead of multiple small writes.
    entry_buffer: Vec<u8>,
    /// Rolling high-water mark for payload buffer size (bytes).
    ///
    /// Tracks typical peak usage over recent appends. Used to decide when
    /// shrinking is safe (capacity significantly exceeds high-water) without
    /// thrashing after one-off large bundles. Decays slowly each append.
    payload_high_water: usize,
    /// Rolling high-water mark for entry buffer size (bytes).
    ///
    /// Same adaptive sizing logic as `payload_high_water`, applied to the
    /// entry serialization buffer to prevent unbounded growth.
    entry_high_water: usize,
    /// Decay rate for the high-water mark (numerator, denominator).
    buffer_decay_rate: (usize, usize),
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
    /// Path to the cursor sidecar file on disk.
    sidecar_path: PathBuf,
    /// Cached copy of the persisted cursor position + rotation generation.
    cursor_state: CursorSidecar,
    /// Total bytes across the active WAL plus all rotated files.
    aggregate_bytes: u64,
    /// Cumulative bytes written to WAL since writer opened (never decreases).
    /// Used for accurate throughput measurement across rotations.
    cumulative_bytes_written: u64,
    /// Metadata describing each rotated `wal.N` file on disk, ordered oldest-to-newest.
    rotated_files: VecDeque<RotatedWalFile>,
    /// WAL stream position at the start of the active file.
    /// Initialized from the active file's header `wal_position_start` at startup,
    /// and updated on each rotation to match the new active file's start position.
    /// Used as a fallback in `active_wal_position_start()` when no rotated files exist.
    wal_position_start: u64,
    /// Next rotation id to use when rotating (initialized to max existing + 1).
    next_rotation_id: u64,
    /// Header size of the active WAL file in bytes.
    /// Read from the active file's header; used for coordinate conversions.
    /// Updated on rotation when a new file is created.
    active_header_size: u64,
    /// File offset in the active WAL up to which entry boundaries have been validated.
    /// Used by `ensure_entry_boundary()` to avoid re-scanning from the file start.
    /// Reset to `active_header_size` on rotation since the new file has no entries yet.
    active_file_validated_entry_boundary: u64,
    /// In-memory index of entry end positions (file offsets) in the active WAL file.
    /// Populated during append; used by `ensure_entry_boundary()` to validate cursors
    /// without reading from disk. Cleared on rotation.
    entry_boundaries: Vec<u64>,
    /// Sequence number associated with the last committed consumer cursor.
    last_cursor_sequence: Option<u64>,
    /// Count of WAL file rotations performed during this writer's lifetime.
    rotation_count: u64,
    /// Count of rotated files purged during this writer's lifetime.
    purge_count: u64,
}

/// Opaque marker returned after an append.
///
/// Contains the WAL position and sequence number of the written entry.
/// These are logical positions in the WAL coordinate system, abstracting
/// away file headers and rotation boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WalOffset {
    /// WAL position where the entry starts (excludes file headers).
    pub position: u64,
    /// WAL position immediately after the entry (used for cursor tracking).
    pub next_offset: u64,
    /// Monotonically increasing sequence number assigned to this entry.
    pub sequence: u64,
}

impl WalWriter {
    pub async fn open(options: WalWriterOptions) -> WalResult<Self> {
        if let Some(parent) = options.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&options.path)
            .await?;

        let metadata = file.metadata().await?;
        let is_new_file = metadata.len() == 0;

        // Read or create header, extracting wal_position_start and header_size
        let (active_wal_start, active_header_size) = if is_new_file {
            let header = WalHeader::new(options.segment_cfg_hash);
            header.write_to(&mut file).await?;
            file.flush().await?;
            (0, header.encoded_len()) // New file starts at WAL position 0
        } else if metadata.len() < WAL_HEADER_MIN_LEN as u64 {
            otel_error!(
                "quiver.wal.init",
                path = %options.path.display(),
                file_size = metadata.len(),
                min_header_size = WAL_HEADER_MIN_LEN,
                error_type = "corruption",
                reason = "invalid_header",
                message = "file may be corrupt",
            );
            return Err(WalError::InvalidHeader("file smaller than minimum header"));
        } else {
            let header = WalHeader::read_from(&mut file).await?;
            if header.segment_cfg_hash != options.segment_cfg_hash {
                otel_error!(
                    "quiver.wal.init",
                    path = %options.path.display(),
                    expected = ?options.segment_cfg_hash,
                    found = ?header.segment_cfg_hash,
                    error_type = "config",
                    reason = "config_mismatch",
                    message = "WAL was created with a different configuration",
                );
                return Err(WalError::SegmentConfigMismatch {
                    expected: options.segment_cfg_hash,
                    found: header.segment_cfg_hash,
                });
            }
            (header.wal_position_start, header.encoded_len())
        };

        options.validate()?;

        let sidecar_path = CursorSidecar::path_for(&options.path);
        let cursor_state = load_cursor_state(&sidecar_path).await?;
        let buffer_decay_rate = options.buffer_decay_rate;

        // Create coordinator first to scan for valid entries
        let mut coordinator = WalCoordinator::new(
            options,
            sidecar_path,
            cursor_state,
            active_header_size,
            active_wal_start,
        );
        coordinator.reload_rotated_files(metadata.len()).await?;

        // Scan to find the last valid entry and truncate any trailing garbage
        let (next_sequence, valid_offset) = coordinator.detect_next_sequence().await?;
        let current_file_len = file.metadata().await?.len();

        if valid_offset < current_file_len {
            // Truncate trailing garbage from a partial write (e.g., crash mid-write)
            file.set_len(valid_offset).await?;
            file.sync_all().await?;
        }

        // Position at the end of valid data
        let _ = file.seek(SeekFrom::Start(valid_offset)).await?;

        let active_file = ActiveWalFile::new(file, valid_offset, buffer_decay_rate);
        coordinator.restore_cursor_offsets(active_file.len());
        coordinator.recalculate_aggregate_bytes(active_file.len());

        // Record existing WAL bytes in the shared disk budget (if provided)
        if let Some(ref budget) = coordinator.options.budget {
            budget.add(coordinator.aggregate_bytes);
        }

        otel_info!(
            "quiver.wal.init",
            is_new_file,
            rotated_file_count = coordinator.rotated_files.len(),
            cursor_position = coordinator.cursor_state.wal_position,
            next_sequence,
            aggregate_bytes = coordinator.aggregate_bytes,
        );

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
    pub async fn append_bundle<B: RecordBundle>(&mut self, bundle: &B) -> WalResult<WalOffset> {
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
        let payload_len = payload_bytes.len();
        let entry_start = self
            .active_file
            .write_entry(entry_len, &entry_header, &payload_bytes, crc)
            .await?;
        payload_bytes.clear();
        self.active_file.payload_buffer = payload_bytes;
        self.active_file.maybe_shrink_payload_buffer(payload_len);

        self.next_sequence = self.next_sequence.wrapping_add(1);

        let entry_end_offset = entry_start.saturating_add(entry_total_bytes);
        self.active_file.current_len = self
            .active_file
            .current_len
            .saturating_add(entry_total_bytes);
        self.coordinator
            .record_append(entry_end_offset, entry_total_bytes);
        self.active_file
            .maybe_flush(&self.coordinator.options().flush_policy, entry_total_bytes)
            .await?;

        // Convert file offsets to WAL positions BEFORE rotation
        // (rotation changes active_wal_position_start, which would corrupt the calculation)
        let wal_position = self.coordinator.to_wal_position(entry_start);
        let wal_next_position = self
            .coordinator
            .to_wal_position(entry_start.saturating_add(entry_total_bytes));

        self.coordinator
            .maybe_rotate_after_append(&mut self.active_file)
            .await?;

        Ok(WalOffset {
            position: wal_position,
            next_offset: wal_next_position,
            sequence,
        })
    }

    /// Persists the cursor position and enables cleanup of consumed WAL data.
    ///
    /// This validates the cursor, writes to the cursor sidecar with fsync,
    /// and purges any rotated files fully covered by the new position.
    ///
    /// Call this after downstream has confirmed durability (e.g., segment flush).
    pub(crate) async fn persist_cursor(&mut self, cursor: &WalConsumerCursor) -> WalResult<()> {
        self.coordinator
            .persist_cursor(&mut self.active_file, cursor)
            .await
    }

    /// Returns the number of WAL file rotations performed during this writer's lifetime.
    pub(crate) const fn rotation_count(&self) -> u64 {
        self.coordinator.rotation_count
    }

    /// Returns the number of rotated files purged during this writer's lifetime.
    pub(crate) const fn purge_count(&self) -> u64 {
        self.coordinator.purge_count
    }

    /// Returns the cumulative bytes written to WAL since this writer opened.
    /// This value never decreases, even as WAL files are rotated and purged.
    pub(crate) const fn cumulative_bytes_written(&self) -> u64 {
        self.coordinator.cumulative_bytes_written
    }

    /// Encodes a slot directly into `payload_buffer`, avoiding intermediate allocations.
    ///
    /// Writes the slot header (id, fingerprint, row_count, payload_len) followed by
    /// the Arrow IPC-encoded payload bytes. The payload is written directly to the
    /// buffer, and the length is patched in afterwards.
    fn encode_slot_into_buffer(
        &mut self,
        slot_id: SlotId,
        payload: PayloadRef<'_>,
    ) -> WalResult<()> {
        let row_count = u32::try_from(payload.batch.num_rows())
            .map_err(|_| WalError::RowCountOverflow(payload.batch.num_rows()))?;

        // Write slot header with placeholder for payload_len
        self.active_file
            .payload_buffer
            .extend_from_slice(&slot_id.0.to_le_bytes());
        self.active_file
            .payload_buffer
            .extend_from_slice(&payload.schema_fingerprint);
        self.active_file
            .payload_buffer
            .extend_from_slice(&row_count.to_le_bytes());
        let payload_len_offset = self.active_file.payload_buffer.len();
        self.active_file
            .payload_buffer
            .extend_from_slice(&0u32.to_le_bytes()); // placeholder

        // Encode Arrow IPC directly into buffer
        let payload_start = self.active_file.payload_buffer.len();
        {
            let schema = payload.batch.schema();
            let mut writer = StreamWriter::try_new(&mut self.active_file.payload_buffer, &schema)
                .map_err(WalError::Arrow)?;
            writer.write(payload.batch).map_err(WalError::Arrow)?;
            writer.finish().map_err(WalError::Arrow)?;
        }
        let payload_len = self.active_file.payload_buffer.len() - payload_start;

        // Patch the payload length
        let payload_len_u32 =
            u32::try_from(payload_len).map_err(|_| WalError::PayloadTooLarge(payload_len))?;
        self.active_file.payload_buffer[payload_len_offset..payload_len_offset + 4]
            .copy_from_slice(&payload_len_u32.to_le_bytes());

        Ok(())
    }
}

impl ActiveWalFile {
    fn new(file: File, current_len: u64, buffer_decay_rate: (usize, usize)) -> Self {
        Self {
            file: Some(file),
            payload_buffer: Vec::new(),
            entry_buffer: Vec::new(),
            payload_high_water: 0,
            entry_high_water: 0,
            buffer_decay_rate,
            last_flush: Instant::now(),
            unflushed_bytes: 0,
            current_len,
        }
    }

    const fn len(&self) -> u64 {
        self.current_len
    }

    /// Returns a mutable reference to the file, or an error if unavailable.
    ///
    /// The file is only `None` transiently during `flush_now_sync()` in Drop.
    /// During normal async operations it should always be present.
    fn file_ref(&mut self) -> WalResult<&mut File> {
        self.file
            .as_mut()
            .ok_or(WalError::InternalError("file handle unavailable"))
    }

    async fn seek_to_end(&mut self) -> WalResult<u64> {
        let file = self.file_ref()?;
        let pos = file.seek(SeekFrom::End(0)).await?;
        Ok(pos)
    }

    fn file_mut(&mut self) -> WalResult<&mut File> {
        self.file_ref()
    }

    const fn set_len(&mut self, len: u64) {
        self.current_len = len;
    }

    fn replace_file(&mut self, file: File, new_len: u64) {
        self.file = Some(file);
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
    ///
    /// Builds the complete entry in `entry_buffer` and writes with a single syscall.
    /// The buffer is reused across calls with adaptive shrinking to balance memory
    /// usage against reallocation overhead.
    async fn write_entry(
        &mut self,
        entry_len: u32,
        entry_header: &[u8; ENTRY_HEADER_LEN],
        payload: &[u8],
        crc: u32,
    ) -> WalResult<u64> {
        // Record entry start position before writing
        let entry_start = self.current_len;

        // Calculate total entry size: 4 (len) + header + payload + 4 (crc)
        let total_size = 4 + ENTRY_HEADER_LEN + payload.len() + 4;

        // Reuse the buffer: clear sets len=0 but preserves capacity.
        // Only reserve additional capacity if needed (reserve is a no-op if
        // capacity is already sufficient).
        self.entry_buffer.clear();
        self.entry_buffer.reserve(total_size);

        // Build complete entry
        self.entry_buffer
            .extend_from_slice(&entry_len.to_le_bytes());
        self.entry_buffer.extend_from_slice(entry_header);
        self.entry_buffer.extend_from_slice(payload);
        self.entry_buffer.extend_from_slice(&crc.to_le_bytes());

        // Single write syscall for the complete entry
        // Note: we access file.as_mut() directly here to avoid borrow checker issues
        // with self.entry_buffer being borrowed.
        let file = self
            .file
            .as_mut()
            .ok_or(WalError::InternalError("file handle unavailable"))?;
        file.write_all(&self.entry_buffer).await?;

        // Adaptive shrinking: keep buffer sized appropriately for typical usage
        self.maybe_shrink_entry_buffer(total_size);

        Ok(entry_start)
    }

    /// Updates the high-water mark and potentially shrinks the entry buffer.
    ///
    /// Uses the same adaptive algorithm as `maybe_shrink_payload_buffer`.
    fn maybe_shrink_entry_buffer(&mut self, used_len: usize) {
        // Update high-water with current usage
        self.entry_high_water = self.entry_high_water.max(used_len);

        // Apply decay so high-water adapts to reduced usage
        let (numerator, denominator) = self.buffer_decay_rate;
        self.entry_high_water = self.entry_high_water.saturating_mul(numerator) / denominator;

        let capacity = self.entry_buffer.capacity();
        let target = self.entry_high_water.saturating_add(SHRINK_HEADROOM);

        // Only shrink if:
        // - Capacity significantly exceeds the target (2× headroom)
        // - Buffer is large enough to bother (> SHRINK_THRESHOLD)
        if capacity > target.saturating_mul(2) && capacity > SHRINK_THRESHOLD {
            self.entry_buffer.shrink_to(target);
        }
    }

    async fn maybe_flush(&mut self, policy: &FlushPolicy, bytes_written: u64) -> WalResult<()> {
        self.unflushed_bytes = self.unflushed_bytes.saturating_add(bytes_written);

        match policy {
            FlushPolicy::Immediate => self.flush_now().await,
            FlushPolicy::EveryNBytes(threshold) => {
                if self.unflushed_bytes >= *threshold {
                    self.flush_now().await
                } else {
                    Ok(())
                }
            }
            FlushPolicy::EveryDuration(interval) => {
                if self.last_flush.elapsed() >= *interval {
                    self.flush_now().await
                } else {
                    Ok(())
                }
            }
            FlushPolicy::BytesOrDuration { bytes, duration } => {
                if self.unflushed_bytes >= *bytes || self.last_flush.elapsed() >= *duration {
                    self.flush_now().await
                } else {
                    Ok(())
                }
            }
        }
    }

    async fn flush_now(&mut self) -> WalResult<()> {
        let file = self.file_ref()?;
        file.flush().await?;
        sync_file_data(file).await?;
        self.last_flush = Instant::now();
        self.unflushed_bytes = 0;
        Ok(())
    }

    /// Synchronous flush for use in `Drop` where async is not available.
    ///
    /// This temporarily converts the tokio File to a std File to perform
    /// a blocking sync_data call, then converts it back.
    fn flush_now_sync(&mut self) {
        // Take ownership of the file temporarily. If already taken (shouldn't
        // happen in Drop), skip the sync.
        let Some(tokio_file) = self.file.take() else {
            otel_warn!("quiver.wal.drop.flush", reason = "no_handle",);
            return;
        };

        // try_into_std fails if the file has pending async operations.
        // In Drop, we expect the file to be idle.
        let std_file = match tokio_file.try_into_std() {
            Ok(f) => f,
            Err(tokio_file) => {
                // Restore the file and give up - pending async ops
                self.file = Some(tokio_file);
                otel_warn!("quiver.wal.drop.flush", reason = "pending_async_ops",);
                return;
            }
        };

        // Perform sync_data
        #[cfg(test)]
        test_support::record_sync_data();
        if let Err(e) = std_file.sync_data() {
            otel_warn!("quiver.wal.drop.flush", error = %e, error_type = "io", reason = "sync_data_failed");
        }

        // Convert back to tokio::fs::File
        self.file = Some(File::from_std(std_file));
        self.last_flush = Instant::now();
        self.unflushed_bytes = 0;
    }

    /// Updates the high-water mark and potentially shrinks the payload buffer.
    ///
    /// Call this after writing an entry (when `payload_buffer` has been cleared).
    /// The algorithm:
    /// 1. Update high-water to max(high_water, used_len)
    /// 2. Apply decay: high_water = high_water * numerator / denominator
    /// 3. Shrink if capacity > 2 * high_water + SHRINK_HEADROOM and capacity > SHRINK_THRESHOLD
    ///
    /// The decay ensures we eventually reclaim memory if usage drops permanently.
    /// The 2× threshold and headroom prevent thrashing on minor fluctuations.
    fn maybe_shrink_payload_buffer(&mut self, used_len: usize) {
        // Update high-water with current usage
        self.payload_high_water = self.payload_high_water.max(used_len);

        // Apply decay so high-water adapts to reduced usage
        let (numerator, denominator) = self.buffer_decay_rate;
        self.payload_high_water = self.payload_high_water.saturating_mul(numerator) / denominator;

        let capacity = self.payload_buffer.capacity();
        let target = self.payload_high_water.saturating_add(SHRINK_HEADROOM);

        // Only shrink if:
        // - Capacity significantly exceeds the target (2× headroom)
        // - Buffer is large enough to bother (> SHRINK_THRESHOLD)
        if capacity > target.saturating_mul(2) && capacity > SHRINK_THRESHOLD {
            self.payload_buffer.shrink_to(target);
        }
    }
}

/// Result of scanning a WAL file for entries.
struct ScanResult {
    /// The last valid sequence number found, if any.
    last_sequence: Option<u64>,
    /// File offset immediately after the last valid entry (where new writes begin).
    last_valid_offset: u64,
    /// File offsets at the end of each entry (for cursor validation).
    entry_boundaries: Vec<u64>,
}

impl WalCoordinator {
    const fn new(
        options: WalWriterOptions,
        sidecar_path: PathBuf,
        cursor_state: CursorSidecar,
        active_header_size: u64,
        active_wal_start: u64,
    ) -> Self {
        Self {
            options,
            sidecar_path,
            cursor_state,
            aggregate_bytes: active_header_size,
            cumulative_bytes_written: 0,
            rotated_files: VecDeque::new(),
            // The active file's header tells us the WAL position at the start
            // of this file, which represents data from prior rotated files.
            wal_position_start: active_wal_start,
            next_rotation_id: 1,
            active_header_size,
            active_file_validated_entry_boundary: active_header_size,
            entry_boundaries: Vec::new(),
            last_cursor_sequence: None,
            rotation_count: 0,
            purge_count: 0,
        }
    }

    /// Returns the WAL stream position at the start of the active file.
    /// This is the sum of data bytes from all rotated files (including purged ones).
    fn active_wal_position_start(&self) -> u64 {
        self.rotated_files
            .back()
            .map_or(self.wal_position_start, |f| f.wal_position_end)
    }

    const fn options(&self) -> &WalWriterOptions {
        &self.options
    }

    fn restore_cursor_offsets(&mut self, active_len: u64) {
        let active_data = active_len.saturating_sub(self.active_header_size);

        // Get the WAL position at the end of rotated files
        let rotated_end = self.rotated_files.back().map_or(0, |f| f.wal_position_end);

        // Total logical bytes currently on disk (rotated + active data)
        let total_on_disk = rotated_end.saturating_add(active_data);

        // Handle cursor position beyond what's on disk
        if self.cursor_state.wal_position > total_on_disk {
            // The cursor claims more data was consumed than currently exists.
            // This means the cursor is stale (from before a crash, data never written).
            // Since wal_position_start is correctly initialized from the
            // active file's header, we only need to handle stale
            // cursors by clamping to actual data.
            self.cursor_state.wal_position = total_on_disk;
        }

        // Initialize validated boundary to match the cursor position in the active file,
        // so entry boundary scanning can start from there.
        self.active_file_validated_entry_boundary =
            self.to_file_offset(self.cursor_state.wal_position, active_len);
    }

    fn record_append(&mut self, entry_end_offset: u64, entry_total_bytes: u64) {
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(entry_total_bytes);
        self.cumulative_bytes_written = self
            .cumulative_bytes_written
            .saturating_add(entry_total_bytes);

        // Record the new WAL bytes in the shared disk budget (if provided)
        if let Some(ref budget) = self.options.budget {
            budget.add(entry_total_bytes);
        }

        // Record the entry end position for in-memory cursor validation
        self.entry_boundaries.push(entry_end_offset);

        // Warn if entry_boundaries is growing large relative to expected WAL capacity.
        // This vector is cleared on rotation and pruned on cursor advancement.
        // Under normal operation with 64MB rotation target and ~1KB bundles, we'd expect
        // at most ~65,000 entries per active file. A threshold of 100,000 indicates either:
        // - Very small bundles with a large rotation target, or
        // - The consumer cursor is not advancing (entries aren't being pruned)
        // Memory impact: 100,000 entries × 8 bytes = 800KB (modest but worth monitoring)
        const ENTRY_BOUNDARIES_WARNING_THRESHOLD: usize = 100_000;
        if self.entry_boundaries.len() == ENTRY_BOUNDARIES_WARNING_THRESHOLD {
            otel_warn!(
                "quiver.wal.backpressure",
                entry_count = self.entry_boundaries.len(),
                rotation_target_bytes = self.options.rotation_target_bytes,
                reason = "stale_cursor",
                message = "consumer cursor may be stale or not advancing",
            );
        }
    }

    fn preflight_append(
        &mut self,
        active_file: &ActiveWalFile,
        entry_total_bytes: u64,
    ) -> WalResult<()> {
        let will_rotate = active_file
            .len()
            .saturating_add(entry_total_bytes)
            .saturating_sub(self.active_header_size)
            > self.options.rotation_target_bytes;

        if will_rotate && self.rotated_files.len() >= self.options.max_rotated_files {
            otel_warn!(
                "quiver.wal.backpressure",
                reason = "rotated_files_cap",
                phase = "preflight_append",
                rotated_file_count = self.rotated_files.len(),
                max_rotated_files = self.options.max_rotated_files,
                aggregate_bytes = self.aggregate_bytes,
            );
            return Err(WalError::WalAtCapacity(
                "rotated wal file cap reached; advance cursor before rotating",
            ));
        }

        let mut projected = self.aggregate_bytes.saturating_add(entry_total_bytes);
        if will_rotate {
            // New file will have a fresh header
            let new_header_size = WalHeader::new(self.options.segment_cfg_hash).encoded_len();
            projected = projected.saturating_add(new_header_size);
        }

        if projected > self.options.max_wal_size {
            otel_warn!(
                "quiver.wal.backpressure",
                reason = "max_wal_size",
                projected_bytes = projected,
                max_wal_size = self.options.max_wal_size,
                aggregate_bytes = self.aggregate_bytes,
            );
            return Err(WalError::WalAtCapacity(
                "wal size cap exceeded; advance cursor to reclaim space",
            ));
        }

        // Note: WAL does NOT check shared disk budget here. The engine applies
        // backpressure at the ingestion boundary based on budget headroom,
        // leaving room for WAL rotation and segment finalization to complete.
        // This simpler approach avoids potential deadlocks from WAL trying to
        // trigger cleanup while holding locks.

        Ok(())
    }

    /// Validates that `target` is a valid entry boundary in the active WAL file.
    ///
    /// Uses the in-memory `entry_boundaries` index for O(log n) validation without
    /// reading from disk. The index is populated during append operations.
    fn ensure_entry_boundary(
        &mut self,
        _active_file: &mut ActiveWalFile,
        target: u64,
    ) -> WalResult<()> {
        // Fast path: target matches the last validated boundary
        if target == self.active_file_validated_entry_boundary {
            return Ok(());
        }

        // Regression check
        if target < self.active_file_validated_entry_boundary {
            return Err(WalError::InvalidConsumerCursor(
                "safe offset regressed (file offset)",
            ));
        }

        // Target must be at header (start of entries) or at an entry boundary
        if target == self.active_header_size {
            return Ok(());
        }

        // Binary search for target in the entry boundaries index
        // entry_boundaries contains the end offset of each entry
        match self.entry_boundaries.binary_search(&target) {
            Ok(_) => Ok(()), // Exact match - target is a valid entry boundary
            Err(_) => Err(WalError::InvalidConsumerCursor(
                "safe offset splits entry boundary",
            )),
        }
    }

    fn recalculate_aggregate_bytes(&mut self, active_len: u64) {
        let rotated_total: u64 = self.rotated_files.iter().map(|f| f.total_bytes()).sum();
        self.aggregate_bytes = rotated_total.saturating_add(active_len);
    }

    fn to_wal_position(&self, file_offset: u64) -> u64 {
        let data_offset = file_offset.saturating_sub(self.active_header_size);
        self.active_wal_position_start().saturating_add(data_offset)
    }

    fn to_file_offset(&self, wal_position: u64, active_len: u64) -> u64 {
        let active_data_len = active_len.saturating_sub(self.active_header_size);
        let active_start = self.active_wal_position_start();

        if wal_position <= active_start {
            return self.active_header_size;
        }

        let data_within_active = wal_position
            .saturating_sub(active_start)
            .min(active_data_len);
        self.active_header_size.saturating_add(data_within_active)
    }

    /// Scans a WAL file and returns information about its entries.
    ///
    /// Returns file offsets (not WAL positions). The entry boundaries can be
    /// used to initialize the `entry_boundaries` vector for cursor validation
    /// during WAL replay.
    ///
    /// Note: Uses sync I/O via `WalReader` - this is acceptable because it's
    /// only called during startup/recovery.
    fn scan_file_for_entries(&self, path: &Path) -> WalResult<ScanResult> {
        if !path.exists() {
            return Ok(ScanResult {
                last_sequence: None,
                last_valid_offset: 0,
                entry_boundaries: Vec::new(),
            });
        }
        let mut reader = WalReader::open(path)?;
        let header_size = reader.header_size();
        let wal_position_start = reader.wal_position_start();
        let iter = reader.iter_from(0)?;
        let mut last_seq = None;
        let mut last_valid_offset = header_size;
        let mut boundaries = Vec::new();
        for entry in iter {
            match entry {
                Ok(bundle) => {
                    last_seq = Some(bundle.sequence);
                    // bundle.next_offset is now a global WAL position
                    // Convert back to file offset for internal use:
                    // file_offset = (global_position - wal_position_start) + header_size
                    let position_in_file = bundle.next_offset.saturating_sub(wal_position_start);
                    last_valid_offset = position_in_file + header_size;
                    boundaries.push(last_valid_offset);
                }
                Err(WalError::UnexpectedEof(_))
                | Err(WalError::InvalidEntry(_))
                | Err(WalError::CrcMismatch { .. }) => {
                    // Partial or corrupted entry - stop scanning here.
                    // This handles crash recovery (UnexpectedEof) and corruption
                    // (InvalidEntry, CrcMismatch) by using entries before the damage.
                    break;
                }
                Err(err) => return Err(err),
            }
        }
        Ok(ScanResult {
            last_sequence: last_seq,
            last_valid_offset,
            entry_boundaries: boundaries,
        })
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Async methods
    // ─────────────────────────────────────────────────────────────────────────────

    /// Determines the next sequence number and last valid offset in the active file.
    /// Also populates `entry_boundaries` with all entry end offsets for cursor validation.
    ///
    /// Since sequence numbers are monotonically increasing, the highest sequence
    /// is always in the active file (or the most recent rotated file if the active
    /// file is empty after a rotation). Returns (next_sequence, active_file_valid_offset).
    ///
    /// Note: Uses sync I/O via `WalReader` internally - this is acceptable because
    /// this method is only called during startup/recovery, not on the hot path.
    async fn detect_next_sequence(&mut self) -> WalResult<(u64, u64)> {
        let scan = self.scan_file_for_entries(&self.options.path)?;

        // Populate entry_boundaries so cursor validation works during WAL replay
        self.entry_boundaries = scan.entry_boundaries;

        // If active file has entries, use its sequence
        if let Some(seq) = scan.last_sequence {
            return Ok((seq.wrapping_add(1), scan.last_valid_offset));
        }

        // Active file is empty - check the most recent rotated file (highest rotation_id)
        if let Some(most_recent) = self.rotated_files.iter().max_by_key(|f| f.rotation_id) {
            let rotated_scan = self.scan_file_for_entries(&most_recent.path)?;
            if let Some(s) = rotated_scan.last_sequence {
                return Ok((s.wrapping_add(1), scan.last_valid_offset));
            }
        }

        // No entries anywhere - start at sequence 0
        Ok((0, scan.last_valid_offset))
    }

    async fn reload_rotated_files(&mut self, active_len: u64) -> WalResult<()> {
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
        let mut wal_position = 0u64;
        let mut max_rotation_id = 0u64;

        for (rotation_id, len) in &discovered {
            let path = rotated_wal_path(&self.options.path, *rotation_id);
            // Read header to get actual header size for this file
            let mut file = File::open(&path).await?;
            let header_size = WalHeader::read_header_size(&mut file).await? as u64;

            aggregate = aggregate.saturating_add(*len);
            let data_bytes = len.saturating_sub(header_size);
            wal_position = wal_position.saturating_add(data_bytes);
            files.push_back(RotatedWalFile {
                path,
                rotation_id: *rotation_id,
                file_bytes: *len,
                wal_position_end: wal_position,
            });
            max_rotation_id = max_rotation_id.max(*rotation_id);
        }

        self.rotated_files = files;
        self.aggregate_bytes = aggregate;
        self.next_rotation_id = max_rotation_id.saturating_add(1);
        Ok(())
    }

    async fn maybe_rotate_after_append(
        &mut self,
        active_file: &mut ActiveWalFile,
    ) -> WalResult<()> {
        let active_data_bytes = active_file.len().saturating_sub(self.active_header_size);
        if active_data_bytes <= self.options.rotation_target_bytes {
            return Ok(());
        }
        self.rotate_active_file(active_file).await
    }

    async fn rotate_active_file(&mut self, active_file: &mut ActiveWalFile) -> WalResult<()> {
        if self.rotated_files.len() >= self.options.max_rotated_files {
            otel_warn!(
                "quiver.wal.backpressure",
                reason = "rotated_files_cap",
                phase = "rotate_active_file",
                rotated_file_count = self.rotated_files.len(),
                max_rotated_files = self.options.max_rotated_files,
                aggregate_bytes = self.aggregate_bytes,
            );
            return Err(WalError::WalAtCapacity(
                "rotated wal file cap reached; advance cursor before rotating",
            ));
        }

        active_file.flush_now().await?;
        let old_len = active_file.len();
        if old_len <= self.active_header_size {
            return Ok(());
        }
        self.aggregate_bytes = self.aggregate_bytes.saturating_sub(old_len);

        // Use monotonic naming: rename to wal.{next_rotation_id}
        let rotation_id = self.next_rotation_id;
        self.next_rotation_id = self.next_rotation_id.saturating_add(1);

        let new_rotated_path = rotated_wal_path(&self.options.path, rotation_id);
        tokio::fs::rename(&self.options.path, &new_rotated_path).await?;

        // Set rotated file to read-only to prevent accidental corruption.
        // Rotated WAL files should never be modified.
        // Skipped when the filesystem doesn't support chmod (e.g., certain
        // Kubernetes volumeMounts, Azure Files SMB mounts).
        if self.options.enforce_file_readonly {
            // Note: Permission changes use std::fs as tokio doesn't wrap these
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = std::fs::Permissions::from_mode(0o440);
                std::fs::set_permissions(&new_rotated_path, permissions)?;
            }

            #[cfg(not(unix))]
            {
                let mut permissions = std::fs::metadata(&new_rotated_path)?.permissions();
                permissions.set_readonly(true);
                std::fs::set_permissions(&new_rotated_path, permissions)?;
            }
        }

        sync_parent_dir(&self.options.path).await?;

        let data_bytes = old_len.saturating_sub(self.active_header_size);
        let new_wal_position_end = self.active_wal_position_start().saturating_add(data_bytes);
        self.rotated_files.push_back(RotatedWalFile {
            path: new_rotated_path,
            rotation_id,
            file_bytes: old_len,
            wal_position_end: new_wal_position_end,
        });
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(old_len);

        // New file's start position is the end of the last rotated file.
        // Update wal_position_start to match the new active file's header.
        let new_wal_start = new_wal_position_end;
        self.wal_position_start = new_wal_start;
        let mut file = reopen_wal_file(
            &self.options.path,
            self.options.segment_cfg_hash,
            new_wal_start,
        )
        .await?;
        let _ = file.seek(SeekFrom::End(0)).await?; // ensure positioned at end

        // New file has a fresh header - update active_header_size
        let new_header_size = WalHeader::new(self.options.segment_cfg_hash).encoded_len();
        self.active_header_size = new_header_size;
        active_file.replace_file(file, new_header_size);
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(active_file.len());
        self.active_file_validated_entry_boundary = new_header_size;
        // Clear entry boundaries index - new file has no entries yet
        self.entry_boundaries.clear();
        self.rotation_count = self.rotation_count.saturating_add(1);

        otel_info!(
            "quiver.wal.rotate",
            rotation_id,
            rotated_file_bytes = old_len,
            rotated_file_count = self.rotated_files.len(),
            aggregate_bytes = self.aggregate_bytes,
        );

        CursorSidecar::write_to(&self.sidecar_path, &self.cursor_state).await?;
        Ok(())
    }

    async fn persist_cursor(
        &mut self,
        active_file: &mut ActiveWalFile,
        cursor: &WalConsumerCursor,
    ) -> WalResult<()> {
        self.validate_cursor(active_file, cursor).await?;
        self.write_cursor_sidecar(cursor).await
    }

    async fn validate_cursor(
        &mut self,
        active_file: &mut ActiveWalFile,
        cursor: &WalConsumerCursor,
    ) -> WalResult<()> {
        // Validate sequence monotonicity
        if let Some(last_seq) = self.last_cursor_sequence {
            if cursor.safe_sequence < last_seq {
                return Err(WalError::InvalidConsumerCursor("safe sequence regressed"));
            }
        }

        // Validate WAL position monotonicity
        if cursor.safe_offset < self.cursor_state.wal_position {
            return Err(WalError::InvalidConsumerCursor("safe offset regressed"));
        }

        let active_start = self.active_wal_position_start();

        // If cursor is before or at the start of the active file, it's in a rotated file.
        // We trust that entry boundaries were validated when that file was active.
        if cursor.safe_offset <= active_start {
            return Ok(());
        }

        // Cursor is in the active file - validate it's within bounds and on an entry boundary
        let file_len = active_file.file_mut()?.metadata().await?.len();
        let data_within_active = cursor.safe_offset.saturating_sub(active_start);
        let file_offset = self.active_header_size.saturating_add(data_within_active);

        if file_offset > file_len {
            return Err(WalError::InvalidConsumerCursor(
                "safe offset beyond wal tail",
            ));
        }

        self.ensure_entry_boundary(active_file, file_offset)?;
        Ok(())
    }

    async fn write_cursor_sidecar(&mut self, cursor: &WalConsumerCursor) -> WalResult<()> {
        let wal_pos = cursor.safe_offset;
        self.record_wal_position(wal_pos).await?;

        // Only update active_file_validated_entry_boundary if the cursor is in the active file
        let active_start = self.active_wal_position_start();
        if wal_pos > active_start {
            let data_within_active = wal_pos.saturating_sub(active_start);
            let file_offset = self.active_header_size.saturating_add(data_within_active);
            self.active_file_validated_entry_boundary = file_offset;

            // Prune entry boundaries that are now behind the cursor.
            let keep_from = self
                .entry_boundaries
                .partition_point(|&offset| offset < file_offset);
            if keep_from > 0 {
                let _ = self.entry_boundaries.drain(..keep_from);
            }
        } else if wal_pos == active_start {
            self.active_file_validated_entry_boundary = self.active_header_size;
        }

        self.last_cursor_sequence = Some(cursor.safe_sequence);
        self.purge_rotated_files().await
    }

    async fn record_wal_position(&mut self, wal_pos: u64) -> WalResult<()> {
        if self.cursor_state.wal_position == wal_pos && self.sidecar_path.exists() {
            return Ok(());
        }
        self.cursor_state.wal_position = wal_pos;
        CursorSidecar::write_to(&self.sidecar_path, &self.cursor_state).await
    }

    async fn purge_rotated_files(&mut self) -> WalResult<()> {
        while let Some(front) = self.rotated_files.front() {
            if front.wal_position_end <= self.cursor_state.wal_position {
                // On non-Unix platforms, read-only files cannot be deleted directly.
                // Only attempt to clear the read-only flag when we actually set it
                // during rotation; otherwise we may hit errors on filesystems that
                // don't support permission changes (e.g. certain Kubernetes
                // volumeMounts, Azure Files SMB mounts).
                #[cfg(not(unix))]
                if self.options.enforce_file_readonly {
                    let mut permissions = std::fs::metadata(&front.path)?.permissions();
                    // The clippy warning about world-writable files only applies to Unix.
                    #[allow(clippy::permissions_set_readonly_false)]
                    permissions.set_readonly(false);
                    std::fs::set_permissions(&front.path, permissions)?;
                }

                let purged_bytes = front.total_bytes();
                tokio::fs::remove_file(&front.path).await?;
                self.aggregate_bytes = self.aggregate_bytes.saturating_sub(purged_bytes);

                if let Some(ref budget) = self.options.budget {
                    budget.remove(purged_bytes);
                }

                otel_debug!(
                    "quiver.wal.drop",
                    purged_bytes,
                    remaining_rotated_files = self.rotated_files.len().saturating_sub(1),
                    aggregate_bytes = self.aggregate_bytes,
                );

                self.purge_count = self.purge_count.saturating_add(1);
                let _ = self.rotated_files.pop_front();
            } else {
                break;
            }
        }
        Ok(())
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

        self.active_file.flush_now_sync();
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

async fn sync_file_data(file: &File) -> WalResult<()> {
    #[cfg(test)]
    test_support::record_sync_data();
    file.sync_data().await?;
    Ok(())
}

/// Syncs the parent directory to ensure rename durability.
///
/// On POSIX systems, `rename()` is atomic but not necessarily durable until the
/// parent directory is fsynced. This matters on filesystems without automatic
/// rename barriers (older ext3, NFS, non-default mount options).
///
/// On non-Unix platforms this is a no-op since directory sync semantics differ.
pub(super) async fn sync_parent_dir(path: &Path) -> WalResult<()> {
    #[cfg(unix)]
    {
        if let Some(parent) = path.parent() {
            let dir = File::open(parent).await?;
            dir.sync_all().await?;
        }
    }
    #[cfg(not(unix))]
    let _ = path; // silence unused warning
    Ok(())
}

async fn load_cursor_state(path: &Path) -> WalResult<CursorSidecar> {
    match CursorSidecar::read_from(path).await {
        Ok(state) => Ok(state),
        Err(WalError::InvalidCursorSidecar(_)) => Ok(default_cursor_state()),
        Err(WalError::Io(err))
            if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::UnexpectedEof) =>
        {
            Ok(default_cursor_state())
        }
        Err(err) => Err(err),
    }
}

const fn default_cursor_state() -> CursorSidecar {
    CursorSidecar::new(0)
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

async fn reopen_wal_file(
    path: &Path,
    segment_hash: [u8; 16],
    wal_position_start: u64,
) -> WalResult<File> {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(path)
        .await?;
    WalHeader::with_base_offset(segment_hash, wal_position_start)
        .write_to(&mut file)
        .await?;
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
    /// WAL stream position at the *end* of this file. When `wal_position >= wal_position_end`
    /// the file is fully consumed and can be deleted.
    wal_position_end: u64,
}

impl RotatedWalFile {
    const fn total_bytes(&self) -> u64 {
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

    /// Returns the current capacity of the writer's payload buffer.
    ///
    /// Used to test shrinking behavior.
    pub fn get_payload_buffer_capacity(writer: &super::WalWriter) -> usize {
        writer.active_file.payload_buffer.capacity()
    }
}
