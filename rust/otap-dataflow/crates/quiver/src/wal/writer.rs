// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Write-ahead-log (WAL) writer for Quiver.
//!
//! The writer is responsible for appending Arrow payloads, rotating chunk
//! files when size thresholds are exceeded, and reclaiming durable space once
//! downstream consumers advance the truncation cursor. Safe offsets are always
//! validated against real entry boundaries so we never expose partially written
//! frames to readers. This module currently powers tests while the runtime
//! integration is still being wired up.
#![allow(dead_code)]

use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::VecDeque;

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use crc32fast::Hasher;

#[cfg(unix)]
use std::os::fd::AsFd;

use crate::record_bundle::{PayloadRef, RecordBundle, SchemaFingerprint, SlotId};

use super::header::{WAL_HEADER_LEN, WalHeader};
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
    #[cfg(test)]
    force_punch_capability: Option<bool>,
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
            #[cfg(test)]
            force_punch_capability: None,
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

    #[cfg(test)]
    pub fn with_punch_capability(mut self, punch_capable: bool) -> Self {
        self.force_punch_capability = Some(punch_capable);
        self
    }
}

/// Stateful writer that maintains append position, rotation metadata, and
/// persisted truncate checkpoints. It tracks both the *current* WAL file and
/// any rotated chunks so the total on-disk footprint (`aggregate_bytes`) can be
/// compared against caps before admitting new entries.
#[derive(Debug)]
pub(crate) struct WalWriter {
    file: File,
    payload_buffer: Vec<u8>,
    options: WalWriterOptions,
    next_sequence: u64,
    last_flush: Instant,
    unflushed_bytes: u64,
    sidecar_path: PathBuf,
    truncate_state: TruncateSidecar,
    punch_capable: bool,
    current_len: u64,
    aggregate_bytes: u64,
    rotated_chunks: VecDeque<RotatedChunk>,
    data_offset_base: u64,
    last_global_safe_offset: u64,
    validated_safe_offset: u64,
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
        let truncate_state = load_truncate_state(&sidecar_path, metadata_len)?;

        let punch_capable = {
            #[cfg(test)]
            if let Some(force) = options.force_punch_capability {
                force
            } else {
                detect_hole_punch(&options.path)
            }
            #[cfg(not(test))]
            {
                detect_hole_punch(&options.path)
            }
        };

        let current_len = metadata_len;

        let mut writer = Self {
            file,
            payload_buffer: Vec::new(),
            options,
            next_sequence: 0,
            last_flush: Instant::now(),
            unflushed_bytes: 0,
            sidecar_path,
            truncate_state,
            punch_capable,
            current_len,
            aggregate_bytes: current_len,
            rotated_chunks: VecDeque::new(),
            data_offset_base: 0,
            last_global_safe_offset: 0,
            validated_safe_offset: truncate_state.truncate_offset,
            last_safe_sequence: None,
        };
        writer.last_global_safe_offset = writer.global_safe_offset(writer.truncate_state.truncate_offset);
        Ok(writer)
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

        self.payload_buffer.clear();

        let entry_start = self.file.seek(SeekFrom::End(0))?;
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);

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

        self.payload_buffer.reserve(total_payload_bytes);
        for slot in encoded_slots {
            slot.write_into(&mut self.payload_buffer);
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

        let entry_body_len = ENTRY_HEADER_LEN + self.payload_buffer.len();
        let entry_len =
            u32::try_from(entry_body_len).map_err(|_| WalError::EntryTooLarge(entry_body_len))?;

        let mut hasher = Hasher::new();
        hasher.update(&entry_header);
        hasher.update(&self.payload_buffer);
        let crc = hasher.finalize();

        self.file.write_all(&entry_len.to_le_bytes())?;
        self.file.write_all(&entry_header)?;
        self.file.write_all(&self.payload_buffer)?;
        self.file.write_all(&crc.to_le_bytes())?;

        let entry_total_bytes = 4u64 + entry_len as u64 + 4;
        self.current_len = self.current_len.saturating_add(entry_total_bytes);
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(entry_total_bytes);
        self.maybe_flush(entry_total_bytes)?;
        self.maybe_rotate_after_append()?;
        self.enforce_size_cap()?;

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
        let safe_offset = self.resolve_truncate_cursor(cursor)?;
        self.file.set_len(safe_offset)?;
        let _ = self.file.seek(SeekFrom::Start(safe_offset))?;
        self.current_len = safe_offset;
        self.recalculate_aggregate_bytes();
        self.commit_truncate_state(cursor, safe_offset)
    }

    /// Updates the truncate sidecar without touching the underlying WAL bytes.
    /// This is used when we merely want to record a new safe cursor that will
    /// later be applied via punching or rewrite.
    pub(crate) fn record_truncate_cursor(&mut self, cursor: &WalTruncateCursor) -> WalResult<()> {
        let safe_offset = self.resolve_truncate_cursor(cursor)?;
        self.commit_truncate_state(cursor, safe_offset)
    }

    /// Reclaims bytes before the provided cursor using hole punching when the
    /// platform supports it, otherwise by rewriting the file in-place. Either
    /// path guarantees that the WAL header is preserved so readers can still
    /// validate future appends.
    pub fn reclaim_prefix(&mut self, cursor: &WalTruncateCursor) -> WalResult<()> {
        let safe_offset = self.resolve_truncate_cursor(cursor)?;
        if safe_offset <= WAL_HEADER_LEN as u64 {
            return self.commit_truncate_state(cursor, WAL_HEADER_LEN as u64);
        }

        if self.try_punch_prefix(safe_offset)? {
            return self.commit_truncate_state(cursor, safe_offset);
        }
        self.rewrite_prefix_in_place(safe_offset)?;
        self.commit_truncate_state(cursor, WAL_HEADER_LEN as u64)
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

    fn maybe_flush(&mut self, bytes_written: u64) -> WalResult<()> {
        self.unflushed_bytes = self.unflushed_bytes.saturating_add(bytes_written);

        if self.options.flush_interval.is_zero() {
            return self.flush_now();
        }

        if self.options.max_unflushed_bytes > 0
            && self.unflushed_bytes >= self.options.max_unflushed_bytes
        {
            return self.flush_now();
        }

        if self.last_flush.elapsed() >= self.options.flush_interval {
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

impl Drop for WalWriter {
    fn drop(&mut self) {
        if self.unflushed_bytes == 0 {
            return;
        }

        let _ = self.flush_now();
        #[cfg(test)]
        test_support::record_drop_flush();
    }
}

#[cfg(test)]
impl WalWriter {
    pub(crate) fn test_set_last_flush(&mut self, instant: Instant) {
        self.last_flush = instant;
    }

    pub(crate) fn test_last_flush(&self) -> Instant {
        self.last_flush
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

fn load_truncate_state(path: &Path, file_len: u64) -> WalResult<TruncateSidecar> {
    match TruncateSidecar::read_from(path) {
        Ok(mut state) => {
            state.truncate_offset = clamp_offset(state.truncate_offset, file_len);
            Ok(state)
        }
        Err(WalError::InvalidTruncateSidecar(_)) => Ok(default_truncate_state(file_len)),
        Err(WalError::Io(err))
            if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::UnexpectedEof) =>
        {
            Ok(default_truncate_state(file_len))
        }
        Err(err) => Err(err),
    }
}

fn default_truncate_state(file_len: u64) -> TruncateSidecar {
    let min_offset = WAL_HEADER_LEN as u64;
    let clamped = min_offset.min(file_len);
    TruncateSidecar::new(clamped, 0)
}

fn clamp_offset(offset: u64, file_len: u64) -> u64 {
    let min_offset = WAL_HEADER_LEN as u64;
    let upper = file_len.max(min_offset);
    offset
        .max(min_offset)
        .min(upper)
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

impl WalWriter {
    /// Validates that the caller-provided cursor never regresses and lands on a
    /// concrete entry boundary. The returned offset can be safely truncated or
    /// reclaimed without risking mid-entry corruption.
    fn resolve_truncate_cursor(&mut self, cursor: &WalTruncateCursor) -> WalResult<u64> {
        let requested_offset = cursor.safe_offset.max(WAL_HEADER_LEN as u64);
        let file_len = self.file.metadata()?.len();
        if requested_offset > file_len {
            return Err(WalError::InvalidTruncateCursor("safe offset beyond wal tail"));
        }
        if requested_offset < self.truncate_state.truncate_offset {
            return Err(WalError::InvalidTruncateCursor("safe offset regressed"));
        }
        if let Some(last_seq) = self.last_safe_sequence {
            if cursor.safe_sequence < last_seq {
                return Err(WalError::InvalidTruncateCursor("safe sequence regressed"));
            }
        }
        self.ensure_entry_boundary(requested_offset)?;
        Ok(requested_offset)
    }

    /// Walks forward from the last validated safe offset until the requested
    /// position is reached, ensuring the cursor never bisects an entry.
    fn ensure_entry_boundary(&mut self, target: u64) -> WalResult<()> {
        if target == self.validated_safe_offset {
            return Ok(());
        }
        if target < self.validated_safe_offset {
            return Err(WalError::InvalidTruncateCursor("safe offset regressed"));
        }

        let original_pos = self.file.seek(SeekFrom::Current(0))?;
        let mut cursor = self.validated_safe_offset;
        let _ = self.file.seek(SeekFrom::Start(cursor))?;
        while cursor < target {
            let mut len_buf = [0u8; 4];
            self.file.read_exact(&mut len_buf)?;
            let entry_len = u32::from_le_bytes(len_buf) as u64;
            let entry_total = 4u64
                .checked_add(entry_len)
                .and_then(|val| val.checked_add(4))
                .ok_or(WalError::InvalidTruncateCursor("entry length overflow"))?;
            cursor = cursor
                .checked_add(entry_total)
                .ok_or(WalError::InvalidTruncateCursor("safe offset overflow"))?;
            if cursor > target {
                let _ = self.file.seek(SeekFrom::Start(original_pos))?;
                return Err(WalError::InvalidTruncateCursor(
                    "safe offset splits entry boundary",
                ));
            }
            let _ = self
                .file
                .seek(SeekFrom::Current(entry_len as i64 + 4))?;
        }
        let _ = self.file.seek(SeekFrom::Start(original_pos))?;
        Ok(())
    }

    /// Persists the truncate metadata to the sidecar and updates book-keeping
    /// such as the global safe offset used to retire rotated chunks.
    fn commit_truncate_state(
        &mut self,
        cursor: &WalTruncateCursor,
        recorded_offset: u64,
    ) -> WalResult<()> {
        self.record_truncate_offset(recorded_offset)?;
        self.last_global_safe_offset = self.global_safe_offset(recorded_offset);
        self.validated_safe_offset = recorded_offset;
        self.last_safe_sequence = Some(cursor.safe_sequence);
        self.purge_rotated_chunks()
    }

    /// Writes the truncate sidecar if the offset changed. This lets restarts
    /// resume from the most recent safe cursor even if the process crashes
    /// before punching/rewriting the WAL file.
    fn record_truncate_offset(&mut self, requested_offset: u64) -> WalResult<()> {
        let safe_offset = requested_offset.max(WAL_HEADER_LEN as u64);
        if self.truncate_state.truncate_offset == safe_offset
            && self.sidecar_path.exists()
        {
            return Ok(());
        }
        self.truncate_state.truncate_offset = safe_offset;
        TruncateSidecar::write_to(&self.sidecar_path, &self.truncate_state)
    }

    /// Attempts to hole-punch the WAL prefix up to `safe_offset`. Returns
    /// `false` if the filesystem declines support so the caller can fall back
    /// to a copy-based rewrite.
    fn try_punch_prefix(&mut self, safe_offset: u64) -> WalResult<bool> {
        if !self.punch_capable {
            return Ok(false);
        }
        if safe_offset <= WAL_HEADER_LEN as u64 {
            return Ok(true);
        }
        match punch_wal_prefix(&self.file, safe_offset) {
            Ok(()) => Ok(true),
            Err(err) => {
                self.punch_capable = false;
                #[cfg(test)]
                test_support::record_punch_failure(&err);
                Ok(false)
            }
        }
    }

    /// Compactly rewrites the WAL in-place by copying bytes after
    /// `safe_offset` down to the header. This guarantees compatibility on
    /// filesystems that cannot punch holes.
    fn rewrite_prefix_in_place(&mut self, safe_offset: u64) -> WalResult<()> {
        let (new_len, reclaimed) = rewrite_wal_prefix_in_place(&mut self.file, safe_offset)?;
        self.current_len = new_len;
        self.recalculate_aggregate_bytes();
        self.data_offset_base = self.data_offset_base.saturating_add(reclaimed);
        Ok(())
    }

    /// Recomputes the total on-disk footprint (active file + rotated chunks)
    /// based on current bookkeeping structures.
    fn recalculate_aggregate_bytes(&mut self) {
        let rotated_total: u64 = self
            .rotated_chunks
            .iter()
            .map(|chunk| chunk.total_bytes())
            .sum();
        self.aggregate_bytes = rotated_total.saturating_add(self.current_len);
    }

    /// Checks whether the active file crossed the rotation target after an
    /// append and, if so, moves it aside so a fresh WAL can start at the same
    /// path while keeping the header intact.
    fn maybe_rotate_after_append(&mut self) -> WalResult<()> {
        let active_data_bytes = self
            .current_len
            .saturating_sub(WAL_HEADER_LEN as u64);
        if active_data_bytes <= self.options.rotation_target_bytes {
            return Ok(());
        }
        self.rotate_active_file()
    }

    /// Performs the actual rotation: flushes the current file, renames it to a
    /// numbered chunk, re-seeds the active WAL, and records the new generation
    /// in the truncate sidecar.
    fn rotate_active_file(&mut self) -> WalResult<()> {
        self.purge_rotated_chunks()?;
        if self.rotated_chunks.len() >= self.options.max_chunks {
            return Err(WalError::WalAtCapacity(
                "wal chunk cap reached; truncate more data before rotating",
            ));
        }

        self.file.flush()?;
        sync_file_data(&self.file)?;
        let old_len = self.current_len;
        if old_len <= WAL_HEADER_LEN as u64 {
            return Ok(());
        }
        self.aggregate_bytes = self.aggregate_bytes.saturating_sub(old_len);

        shift_rotated_files(&self.options.path, self.rotated_chunks.len())?;
        self.update_rotated_chunk_paths();

        let new_chunk_path = rotated_path(&self.options.path, 1);
        std::fs::rename(&self.options.path, &new_chunk_path)?;

        let data_bytes = old_len.saturating_sub(WAL_HEADER_LEN as u64);
        let end_data_offset = self.data_offset_base.saturating_add(data_bytes);
        self.rotated_chunks.push_back(RotatedChunk {
            path: new_chunk_path,
            file_bytes: old_len,
            end_data_offset,
        });
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(old_len);
        self.data_offset_base = end_data_offset;

        self.file = reopen_wal_file(&self.options.path, self.options.segment_cfg_hash)?;
        self.current_len = WAL_HEADER_LEN as u64;
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(self.current_len);
        self.validated_safe_offset = WAL_HEADER_LEN as u64;
        self.truncate_state.truncate_offset = WAL_HEADER_LEN as u64;

        self.truncate_state.rotation_generation = self
            .truncate_state
            .rotation_generation
            .saturating_add(1);
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

    /// Ensures the total bytes consumed by active + rotated chunks never
    /// exceed `max_wal_size`. Hitting the cap indicates that downstream
    /// consumers must advance the truncation cursor before more data can be
    /// ingested.
    fn enforce_size_cap(&mut self) -> WalResult<()> {
        if self.aggregate_bytes <= self.options.max_wal_size {
            return Ok(());
        }
        self.purge_rotated_chunks()?;
        if self.aggregate_bytes <= self.options.max_wal_size {
            return Ok(());
        }
        Err(WalError::WalAtCapacity(
            "wal size cap exceeded; truncate finalized segments",
        ))
    }

    /// Deletes rotated chunks whose logical data range now falls entirely
    /// before the global safe offset. This is triggered after every truncate
    /// commit and before rotations to avoid unbounded disk usage.
    fn purge_rotated_chunks(&mut self) -> WalResult<()> {
        while let Some(front) = self.rotated_chunks.front() {
            if front.end_data_offset <= self.last_global_safe_offset_data() {
                std::fs::remove_file(&front.path)?;
                self.aggregate_bytes = self.aggregate_bytes.saturating_sub(front.total_bytes());
                let _ = self.rotated_chunks.pop_front();
            } else {
                break;
            }
        }
        Ok(())
    }

    fn last_global_safe_offset_data(&self) -> u64 {
        self.last_global_safe_offset
    }

    /// Converts a cursor expressed in absolute WAL bytes into the logical data
    /// offset that spans both active and rotated chunks.
    fn global_safe_offset(&self, cursor_offset: u64) -> u64 {
        let data_offset = cursor_offset.saturating_sub(WAL_HEADER_LEN as u64);
        self.data_offset_base.saturating_add(data_offset)
    }
}

fn detect_hole_punch(path: &Path) -> bool {
    #[cfg(windows)]
    {
        let _ = path;
        false
    }
    #[cfg(not(windows))]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        const PROBE_LEN: i64 = 4096;
        let dir = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let probe_path = dir.join("wal_punch_probe.tmp");
        let result = (|| {
            let file = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .mode(0o600)
                .open(&probe_path)?;
            file.set_len(PROBE_LEN as u64)?;
            punch_file_region(&file, WAL_HEADER_LEN as u64, PROBE_LEN as u64)
        })();
        let _ = std::fs::remove_file(&probe_path);
        result.is_ok()
    }
}

#[cfg(unix)]
fn punch_wal_prefix(file: &File, safe_offset: u64) -> io::Result<()> {
    if safe_offset <= WAL_HEADER_LEN as u64 {
        return Ok(());
    }
#[cfg(test)]
    if test_support::should_inject_punch_error() {
        return Err(io::Error::new(
            ErrorKind::Other,
            "injected punch failure",
        ));
    }
    let punch_len = safe_offset - WAL_HEADER_LEN as u64;
    punch_file_region(file, WAL_HEADER_LEN as u64, punch_len)
}

#[cfg(unix)]
fn punch_file_region(file: &File, start: u64, len: u64) -> io::Result<()> {
    use nix::fcntl::{fallocate, FallocateFlags};

    if len == 0 {
        return Ok(());
    }
    let fd = file.as_fd();
    let flags = FallocateFlags::FALLOC_FL_PUNCH_HOLE | FallocateFlags::FALLOC_FL_KEEP_SIZE;
    fallocate(fd, flags, start as i64, len as i64).map_err(|err| io::Error::from_raw_os_error(err as i32))
}

#[cfg(windows)]
fn punch_wal_prefix(_file: &File, _safe_offset: u64) -> io::Result<()> {
    Err(io::Error::new(
        ErrorKind::Unsupported,
        "hole punch unsupported on windows",
    ))
}

fn rewrite_wal_prefix_in_place(file: &mut File, safe_offset: u64) -> WalResult<(u64, u64)> {
    let header_len = WAL_HEADER_LEN as u64;
    if safe_offset <= header_len {
        return Ok((file.metadata()?.len(), 0));
    }

    let file_len = file.metadata()?.len();
    if safe_offset >= file_len {
        file.set_len(header_len)?;
        let _ = file.seek(SeekFrom::End(0))?;
        file.flush()?;
        sync_file_data(file)?;
        return Ok((header_len, file_len.saturating_sub(header_len)));
    }

    let mut read_pos = safe_offset;
    let mut write_pos = header_len;
    let mut buffer = vec![0u8; 64 * 1024];

    loop {
        let _ = file.seek(SeekFrom::Start(read_pos))?;
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let _ = file.seek(SeekFrom::Start(write_pos))?;
        file.write_all(&buffer[..read])?;
        read_pos += read as u64;
        write_pos += read as u64;
    }

    file.set_len(write_pos)?;
    let _ = file.seek(SeekFrom::End(0))?;
    file.flush()?;
    sync_file_data(file)?;
    let reclaimed = safe_offset.saturating_sub(header_len);
    Ok((write_pos, reclaimed))
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

#[cfg(test)]
pub(super) mod test_support {
    use std::cell::Cell;
    use std::io::Error;

    thread_local! {
        static FLUSH_NOTIFIED: Cell<bool> = Cell::new(false);
        static DROP_FLUSH_NOTIFIED: Cell<bool> = Cell::new(false);
        static SYNC_DATA_NOTIFIED: Cell<bool> = Cell::new(false);
        static FORCE_PUNCH_ERROR: Cell<bool> = Cell::new(false);
        static PUNCH_FAILURE_NOTIFIED: Cell<bool> = Cell::new(false);
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
        FORCE_PUNCH_ERROR.with(|cell| cell.set(false));
        PUNCH_FAILURE_NOTIFIED.with(|cell| cell.set(false));
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

    pub fn set_force_punch_error(flag: bool) {
        FORCE_PUNCH_ERROR.with(|cell| cell.set(flag));
    }

    pub fn should_inject_punch_error() -> bool {
        FORCE_PUNCH_ERROR.with(|cell| cell.get())
    }

    pub fn record_punch_failure(_err: &Error) {
        PUNCH_FAILURE_NOTIFIED.with(|cell| cell.set(true));
    }

    pub fn take_punch_failure_notification() -> bool {
        PUNCH_FAILURE_NOTIFIED.with(|cell| {
            let notified = cell.get();
            cell.set(false);
            notified
        })
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
