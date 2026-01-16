// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Cross-cutting WAL tests live here so shared fixtures can touch writer, reader,
//! and helper plumbing without sprinkling large #[cfg(test)] blocks in each file.

use std::cmp;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use arrow_array::{Int64Array, RecordBatch, builder::StringBuilder};
use arrow_ipc::reader::StreamReader;
use arrow_schema::{DataType, Field, Schema};
use crc32fast::Hasher;
use tempfile::tempdir;

use crate::record_bundle::{
    BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
};

use super::cursor_sidecar::CursorSidecar;
use super::header::WalHeader;
use super::reader::test_support::{self, ReadFailure};
use super::writer::FlushPolicy;
use super::writer::test_support as writer_test_support;
use super::{
    ENTRY_HEADER_LEN, ENTRY_TYPE_RECORD_BUNDLE, SCHEMA_FINGERPRINT_LEN, WalConsumerCursor,
    WalError, WalReader, WalWriter, WalWriterOptions,
};

/// Helper to get the header size for a test WAL file.
fn test_header_size() -> u64 {
    WalHeader::new([0u8; 16]).encoded_len()
}

/// Convert WAL position to file offset (for tests that need to compare with file sizes).
fn wal_position_to_file_offset(wal_position: u64) -> u64 {
    wal_position + test_header_size()
}

struct FixtureSlot {
    id: SlotId,
    fingerprint: SchemaFingerprint,
    batch: RecordBatch,
}

impl FixtureSlot {
    fn new(id: SlotId, fingerprint_seed: u8, values: &[i64]) -> Self {
        let fingerprint = [fingerprint_seed; 32];
        let batch = build_batch(values);
        Self {
            id,
            fingerprint,
            batch,
        }
    }

    fn with_batch(id: SlotId, fingerprint_seed: u8, batch: RecordBatch) -> Self {
        let fingerprint = [fingerprint_seed; 32];
        Self {
            id,
            fingerprint,
            batch,
        }
    }
}

struct FixtureBundle {
    descriptor: BundleDescriptor,
    ingestion_time: SystemTime,
    slots: Vec<FixtureSlot>,
}

impl FixtureBundle {
    fn new(descriptor: BundleDescriptor, slots: Vec<FixtureSlot>) -> Self {
        Self {
            descriptor,
            ingestion_time: UNIX_EPOCH + Duration::from_secs(42),
            slots,
        }
    }

    fn with_ingestion_time(mut self, ts: SystemTime) -> Self {
        self.ingestion_time = ts;
        self
    }
}

impl RecordBundle for FixtureBundle {
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn ingestion_time(&self) -> SystemTime {
        self.ingestion_time
    }

    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
        self.slots
            .iter()
            .find(|s| s.id == slot)
            .map(|slot| PayloadRef {
                schema_fingerprint: slot.fingerprint,
                batch: &slot.batch,
            })
    }
}

fn build_batch(values: &[i64]) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "value",
        DataType::Int64,
        false,
    )]));
    let array = Int64Array::from(values.to_vec());
    RecordBatch::try_new(schema, vec![Arc::new(array)]).expect("valid batch")
}

fn read_batch(bytes: &[u8]) -> RecordBatch {
    let cursor = Cursor::new(bytes);
    let mut reader = StreamReader::try_new(cursor, None).expect("ipc reader");
    reader
        .next()
        .expect("one message")
        .expect("record batch present")
}

fn build_complex_batch(rows: usize, prefix: &str, payload_repeat: usize) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("value", DataType::Int64, false),
        Field::new("message", DataType::Utf8, false),
    ]));

    let values: Vec<i64> = (0..rows).map(|idx| idx as i64).collect();
    let mut builder = StringBuilder::new();
    let chunk = "x".repeat(payload_repeat);
    for idx in 0..rows {
        builder.append_value(format!("{prefix}-{idx:05}-{}", chunk));
    }

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(values)),
            Arc::new(builder.finish()),
        ],
    )
    .expect("complex batch")
}

fn slot_descriptor(id: u16, label: &'static str) -> SlotDescriptor {
    SlotDescriptor::new(SlotId::new(id), label)
}

fn descriptor_with_all_slots() -> BundleDescriptor {
    let slots = (0u16..64)
        .map(|id| {
            let leaked = Box::leak(format!("Slot{id}").into_boxed_str());
            let label: &'static str = leaked;
            SlotDescriptor::new(SlotId::new(id), label)
        })
        .collect();
    BundleDescriptor::new(slots)
}

fn encode_entry_header(entry_type: u8, slot_bitmap: u64) -> Vec<u8> {
    let mut buf = vec![0u8; ENTRY_HEADER_LEN];
    let mut cursor = 0;
    buf[cursor] = entry_type;
    cursor += 1;
    buf[cursor..cursor + 8].copy_from_slice(&42i64.to_le_bytes());
    cursor += 8;
    buf[cursor..cursor + 8].copy_from_slice(&7u64.to_le_bytes());
    cursor += 8;
    buf[cursor..cursor + 8].copy_from_slice(&slot_bitmap.to_le_bytes());
    buf
}

fn write_single_entry(body: &[u8]) -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("single.wal");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .expect("create wal file");
    let header = WalHeader::new([0xEE; 16]);
    header.write_to_sync(&mut file).expect("write header");
    let len = u32::try_from(body.len()).expect("body fits u32");
    let _ = file.seek(SeekFrom::End(0)).expect("seek end");
    file.write_all(&len.to_le_bytes()).expect("write len");
    file.write_all(body).expect("write body");
    let mut hasher = Hasher::new();
    hasher.update(body);
    let crc = hasher.finalize();
    file.write_all(&crc.to_le_bytes()).expect("write crc");
    (dir, path)
}

fn truncate_file_from_end(path: &Path, bytes: u64) {
    let metadata = std::fs::metadata(path).expect("metadata");
    assert!(
        metadata.len() > bytes,
        "file must be larger than truncation"
    );
    let new_len = metadata.len() - bytes;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open for truncate");
    file.set_len(new_len).expect("truncate file");
}

fn rotated_path_for(base: &Path, index: usize) -> PathBuf {
    let mut name = base.as_os_str().to_os_string();
    name.push(format!(".{index}"));
    PathBuf::from(name)
}

fn total_logical_bytes(path: &Path) -> u64 {
    let header = test_header_size();
    let mut total = std::fs::metadata(path)
        .expect("active metadata")
        .len()
        .saturating_sub(header);
    let mut index = 1;
    loop {
        let rotated = rotated_path_for(path, index);
        if !rotated.exists() {
            break;
        }
        let len = std::fs::metadata(&rotated)
            .expect("rotated metadata")
            .len()
            .saturating_sub(header);
        total = total.saturating_add(len);
        index += 1;
    }
    total
}

fn temp_wal(file_name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join(file_name);
    (dir, path)
}

fn logs_descriptor() -> BundleDescriptor {
    BundleDescriptor::new(vec![slot_descriptor(0, "Logs")])
}

fn single_slot_bundle(
    descriptor: &BundleDescriptor,
    fingerprint_seed: u8,
    values: &[i64],
) -> FixtureBundle {
    FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), fingerprint_seed, values)],
    )
}

/// Creates a WalWriter with default options for tests that just need basic functionality.
async fn open_test_writer(path: PathBuf, hash: [u8; 16]) -> WalWriter {
    WalWriter::open(WalWriterOptions::new(path, hash, FlushPolicy::Immediate))
        .await
        .expect("writer")
}

/// Creates a WalWriter with custom options builder.
async fn open_test_writer_with<F>(path: PathBuf, hash: [u8; 16], configure: F) -> WalWriter
where
    F: FnOnce(WalWriterOptions) -> WalWriterOptions,
{
    let options = WalWriterOptions::new(path, hash, FlushPolicy::Immediate);
    WalWriter::open(configure(options)).await.expect("writer")
}

/// Reads all entries from a WAL file starting at the header.
fn read_all_entries(path: &Path) -> Vec<super::WalRecordBundle> {
    let mut reader = WalReader::open(path).expect("reader");
    let iter = reader.iter_from(0).expect("iter");
    iter.map(|r| r.expect("entry")).collect()
}

/// Reads the first N entries from a WAL file.
fn read_entries(path: &Path, count: usize) -> Vec<super::WalRecordBundle> {
    let mut reader = WalReader::open(path).expect("reader");
    let iter = reader.iter_from(0).expect("iter");
    iter.take(count).map(|r| r.expect("entry")).collect()
}

async fn measure_bundle_data_bytes(mut build_bundle: impl FnMut() -> FixtureBundle) -> u64 {
    let (_dir, wal_path) = temp_wal("measure_bundle.wal");
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0xFE; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");
    let bundle = build_bundle();
    let _ = writer.append_bundle(&bundle).await.expect("append bundle");
    drop(writer);
    std::fs::metadata(&wal_path)
        .expect("metadata")
        .len()
        .saturating_sub(test_header_size())
}

struct FailureGuard;

impl FailureGuard {
    fn new() -> Self {
        test_support::reset_failures();
        Self
    }
}

impl Drop for FailureGuard {
    fn drop(&mut self) {
        test_support::reset_failures();
    }
}

#[tokio::test]
async fn wal_writer_reader_roundtrip_recovers_payloads() {
    let (_dir, wal_path) = temp_wal("roundtrip.wal");
    let hash = [0xAB; 16];

    let descriptor = BundleDescriptor::new(vec![
        slot_descriptor(0, "Logs"),
        slot_descriptor(1, "LogsAttrs"),
        slot_descriptor(2, "ScopeAttrs"),
    ]);

    let bundle = FixtureBundle::new(
        descriptor,
        vec![
            FixtureSlot::new(SlotId::new(0), 0x11, &[1, 2, 3]),
            FixtureSlot::new(SlotId::new(2), 0x33, &[99]),
        ],
    );

    let options = WalWriterOptions::new(wal_path.clone(), hash, FlushPolicy::Immediate);
    let mut writer = WalWriter::open(options).await.expect("writer");
    let offset = writer
        .append_bundle(&bundle)
        .await
        .expect("append succeeds");
    assert_eq!(offset.position, 0); // WAL position: first entry starts at 0
    assert_eq!(offset.sequence, 0);
    drop(writer);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    assert_eq!(reader.segment_cfg_hash(), hash);
    assert_eq!(reader.path(), wal_path.as_path());

    let mut iter = reader.iter_from(0).expect("iterator");
    let record = iter.next().expect("entry present").expect("entry ok");
    assert!(iter.next().is_none(), "only one entry expected");

    let expected_bitmap = (1u64 << 0) | (1u64 << 2);
    assert_eq!(record.slot_bitmap, expected_bitmap);
    assert_eq!(record.sequence, 0);
    assert_eq!(record.offset.position, 0); // WAL position: first entry starts at 0
    assert_eq!(record.slots.len(), 2);

    let slot0 = record
        .slots
        .iter()
        .find(|slot| slot.slot_id == SlotId::new(0))
        .expect("slot 0 present");
    assert_eq!(slot0.row_count, 3);
    assert_eq!(slot0.schema_fingerprint, [0x11; 32]);
    let decoded0 = read_batch(&slot0.payload);
    let values0 = decoded0
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .expect("int64 values");
    let collected0: Vec<i64> = values0
        .iter()
        .map(|value| value.expect("non-null"))
        .collect();
    assert_eq!(collected0, vec![1, 2, 3]);

    let slot2 = record
        .slots
        .iter()
        .find(|slot| slot.slot_id == SlotId::new(2))
        .expect("slot 2 present");
    assert_eq!(slot2.row_count, 1);
    assert_eq!(slot2.schema_fingerprint, [0x33; 32]);
    let decoded2 = read_batch(&slot2.payload);
    let values2 = decoded2
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .expect("int64 values");
    let collected2: Vec<i64> = values2
        .iter()
        .map(|value| value.expect("non-null"))
        .collect();
    assert_eq!(collected2, vec![99]);
}

#[tokio::test]
async fn wal_writer_rejects_slot_ids_outside_bitmap() {
    let (_dir, wal_path) = temp_wal("slot_range.wal");
    let mut writer = open_test_writer(wal_path, [0; 16]).await;

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(65, "Overflow")]);
    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(65), 0xAA, &[1])],
    );

    let err = writer
        .append_bundle(&bundle)
        .await
        .expect_err("slot validation");
    assert!(matches!(err, WalError::SlotOutOfRange(slot) if slot == SlotId::new(65)));
}

#[tokio::test]
async fn wal_writer_rejects_pre_epoch_timestamp() {
    let (_dir, wal_path) = temp_wal("pre_epoch.wal");
    let mut writer = open_test_writer(wal_path, [0; 16]).await;

    let descriptor = logs_descriptor();
    let bundle = FixtureBundle::new(descriptor, vec![])
        .with_ingestion_time(UNIX_EPOCH - Duration::from_secs(1));

    let err = writer
        .append_bundle(&bundle)
        .await
        .expect_err("timestamp validation");
    assert!(matches!(err, WalError::InvalidTimestamp));
}

#[tokio::test]
async fn wal_writer_rejects_truncated_existing_file() {
    use super::header::WAL_HEADER_MIN_LEN;

    let (_dir, wal_path) = temp_wal("truncated.wal");
    {
        let mut file = std::fs::File::create(&wal_path).expect("create file");
        // Write fewer bytes than the minimum header length (14 bytes)
        let truncated_len = WAL_HEADER_MIN_LEN - 1;
        file.write_all(&vec![0u8; truncated_len])
            .expect("truncate header");
    }

    let options = WalWriterOptions::new(wal_path, [0; 16], FlushPolicy::Immediate);
    let err = WalWriter::open(options)
        .await
        .expect_err("should reject truncated file");
    assert!(matches!(
        err,
        WalError::InvalidHeader("file smaller than minimum header")
    ));
}

#[tokio::test]
async fn wal_writer_reopens_with_matching_header() {
    let (_dir, wal_path) = temp_wal("existing.wal");
    let original_hash = [0xAA; 16];
    {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&wal_path)
            .expect("create file");
        WalHeader::new(original_hash)
            .write_to_sync(&mut file)
            .expect("write header");
        file.flush().expect("flush");
    }

    // Reopen with the same hash—should succeed and preserve the header.
    let options = WalWriterOptions::new(wal_path.clone(), original_hash, FlushPolicy::Immediate);
    let _writer = WalWriter::open(options).await.expect("open succeeds");
    drop(_writer);

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(&wal_path)
        .expect("open for read");
    let header = WalHeader::read_from_sync(&mut file).expect("read header");
    assert_eq!(header.segment_cfg_hash, original_hash);
}

#[tokio::test]
async fn wal_writer_flushes_after_interval_elapsed() {
    let (_dir, wal_path) = temp_wal("flush.wal");

    let descriptor = logs_descriptor();
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        FlushPolicy::EveryDuration(Duration::from_millis(10)),
    ))
    .await
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x42, &[1])],
    );

    let before = writer.test_last_flush();
    writer.test_set_last_flush(Instant::now() - Duration::from_secs(1));
    let _offset = writer
        .append_bundle(&bundle)
        .await
        .expect("append triggers flush");
    assert!(writer.test_last_flush() > before);
}

#[tokio::test]
async fn wal_writer_flush_syncs_file_data() {
    writer_test_support::reset_flush_notifications();

    let (_dir, wal_path) = temp_wal("flush_sync.wal");

    let descriptor = logs_descriptor();
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0xAA, &[7])],
    );

    assert!(!writer_test_support::take_sync_data_notification());
    let _offset = writer.append_bundle(&bundle).await.expect("append flush");
    assert!(writer_test_support::take_sync_data_notification());
}

#[tokio::test]
async fn wal_writer_records_cursor_without_truncating() {
    let (_dir, wal_path) = temp_wal("record_cursor.wal");

    let descriptor = logs_descriptor();
    let mut writer = open_test_writer(wal_path.clone(), [0x20; 16]).await;

    let _ = writer
        .append_bundle(&single_slot_bundle(&descriptor, 0x01, &[1, 2, 3]))
        .await
        .expect("first append");
    let _ = writer
        .append_bundle(&single_slot_bundle(&descriptor, 0x02, &[4, 5, 6]))
        .await
        .expect("second append");

    let len_before = std::fs::metadata(&wal_path).expect("metadata").len();

    let entries = read_entries(&wal_path, 1);
    let first_entry = &entries[0];

    let cursor = WalConsumerCursor {
        safe_offset: first_entry.next_offset,
        ..WalConsumerCursor::default()
    };
    writer.persist_cursor(&cursor).await.expect("record cursor");
    drop(writer);

    let len_after = std::fs::metadata(&wal_path).expect("metadata").len();
    assert_eq!(
        len_after, len_before,
        "recording a safe cursor no longer mutates the active wal immediately"
    );

    let sidecar_path = wal_path.parent().unwrap().join("quiver.wal.cursor");
    let sidecar = CursorSidecar::read_from_sync(&sidecar_path).expect("sidecar");
    assert_eq!(
        sidecar.wal_position,
        first_entry.next_offset // Already global coordinate
    );

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let entry_one = iter.next().expect("entry").expect("ok");
    let entry_two = iter.next().expect("entry").expect("ok");
    assert_eq!(entry_one.sequence, first_entry.sequence);
    assert_eq!(entry_two.sequence, first_entry.sequence + 1);
    assert!(iter.next().is_none());
}

#[tokio::test]
async fn wal_writer_enforces_safe_offset_boundaries() {
    let (_dir, wal_path) = temp_wal("safe_offset.wal");

    let descriptor = logs_descriptor();
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0x42; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[11, 12, 13])],
    );
    let _ = writer
        .append_bundle(&first_bundle)
        .await
        .expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[21, 22, 23])],
    );
    let _ = writer
        .append_bundle(&second_bundle)
        .await
        .expect("second append");

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let first_entry = iter.next().expect("entry").expect("ok");

    let mut cursor = WalConsumerCursor {
        safe_offset: first_entry.offset.position + 4,
        safe_sequence: first_entry.sequence,
    };

    match writer.persist_cursor(&cursor).await {
        Err(WalError::InvalidConsumerCursor(message)) => {
            assert_eq!(message, "safe offset splits entry boundary")
        }
        other => panic!("expected invalid cursor error, got {other:?}"),
    }

    cursor.increment(&first_entry);
    writer
        .persist_cursor(&cursor)
        .await
        .expect("record succeeds with aligned cursor");
    drop(writer);

    let sidecar_path = wal_path.parent().unwrap().join("quiver.wal.cursor");
    let sidecar = CursorSidecar::read_from_sync(&sidecar_path).expect("sidecar");
    assert_eq!(
        sidecar.wal_position,
        first_entry.next_offset // Already global coordinate
    );
}

#[tokio::test]
async fn wal_writer_persists_consumer_cursor_sidecar() {
    let (_dir, wal_path) = temp_wal("cursor_sidecar.wal");
    let descriptor = logs_descriptor();

    let mut writer = open_test_writer(wal_path.clone(), [0x99; 16]).await;

    let offset = writer
        .append_bundle(&single_slot_bundle(&descriptor, 0x01, &[1, 2]))
        .await
        .expect("append");

    // Use the WAL position from the append result, not file length
    let cursor = WalConsumerCursor {
        safe_offset: offset.next_offset,
        safe_sequence: offset.sequence,
    };
    writer.persist_cursor(&cursor).await.expect("record cursor");
    drop(writer);

    let sidecar_path = wal_path.parent().expect("dir").join("quiver.wal.cursor");
    let state = CursorSidecar::read_from_sync(&sidecar_path).expect("sidecar");
    assert_eq!(state.wal_position, offset.next_offset);
}

#[tokio::test]
async fn wal_writer_rotates_when_target_exceeded() {
    let (_dir, wal_path) = temp_wal("force_rotate.wal");

    let descriptor = logs_descriptor();
    let mut writer = open_test_writer_with(wal_path.clone(), [0x51; 16], |opts| {
        opts.with_rotation_target(1).with_max_rotated_files(4)
    })
    .await;

    let _ = writer
        .append_bundle(&single_slot_bundle(&descriptor, 0x01, &[1, 2, 3, 4]))
        .await
        .expect("append triggers rotation");
    drop(writer);

    let rotated_path = rotated_path_for(&wal_path, 1);
    assert!(
        rotated_path.exists(),
        "rotated file missing at {:?}",
        rotated_path
    );
    let rotated_len = std::fs::metadata(&rotated_path)
        .expect("rotated metadata")
        .len();
    assert!(rotated_len > test_header_size());

    let active_len = std::fs::metadata(&wal_path).expect("active metadata").len();
    assert_eq!(active_len, test_header_size());

    let sidecar_path = wal_path.parent().unwrap().join("quiver.wal.cursor");
    // Sidecar should exist after rotation (even if no cursor has been recorded yet)
    assert!(sidecar_path.exists(), "sidecar should exist after rotation");
    let sidecar = CursorSidecar::read_from_sync(&sidecar_path).expect("sidecar should be readable");
    // wal_position is 0 because no consumer cursor has been recorded yet
    assert_eq!(sidecar.wal_position, 0);
}

#[tokio::test]
async fn wal_writer_reloads_rotated_files_on_restart() {
    let (_dir, wal_path) = temp_wal("replay_rotations.wal");

    let descriptor = logs_descriptor();
    let options = WalWriterOptions::new(wal_path.clone(), [0x54; 16], FlushPolicy::Immediate)
        .with_rotation_target(1)
        .with_max_rotated_files(4);

    {
        let mut writer = WalWriter::open(options.clone())
            .await
            .expect("first writer");
        let bundle = single_slot_bundle(&descriptor, 0x01, &[1, 2, 3, 4]);
        let _ = writer
            .append_bundle(&bundle)
            .await
            .expect("first append triggers rotation");
    }

    assert!(
        rotated_path_for(&wal_path, 1).exists(),
        "expected initial rotation"
    );

    {
        let mut writer = WalWriter::open(options).await.expect("reopen writer");
        let bundle = single_slot_bundle(&descriptor, 0x02, &[5, 6, 7, 8]);
        let _ = writer
            .append_bundle(&bundle)
            .await
            .expect("rotation should succeed after restart");
    }

    assert!(
        rotated_path_for(&wal_path, 2).exists(),
        "existing rotation should be shifted during recovery"
    );
}

#[tokio::test]
async fn wal_writer_errors_when_rotated_file_cap_reached() {
    let (_dir, wal_path) = temp_wal("rotated_cap.wal");

    let descriptor = logs_descriptor();
    let mut writer = open_test_writer_with(wal_path.clone(), [0x52; 16], |opts| {
        opts.with_rotation_target(1).with_max_rotated_files(1)
    })
    .await;

    let payload = [10, 11, 12];
    let first_bundle = single_slot_bundle(&descriptor, 0x02, &payload);
    let _ = writer
        .append_bundle(&first_bundle)
        .await
        .expect("first append rotates");
    assert!(
        rotated_path_for(&wal_path, 1).exists(),
        "expected rotated file to exist",
    );

    let err = writer
        .append_bundle(&single_slot_bundle(&descriptor, 0x03, &payload))
        .await
        .expect_err("second rotation should hit rotated file cap");
    match err {
        WalError::WalAtCapacity(message) => {
            assert!(
                message.contains("rotated wal file cap"),
                "unexpected error message: {message}",
            );
        }
        other => panic!("expected WalAtCapacity, got {other:?}"),
    }
}

#[tokio::test]
async fn wal_writer_enforces_size_cap_and_purges_rotations() {
    let (_dir, wal_path) = temp_wal("size_cap.wal");

    let descriptor = logs_descriptor();
    let payload: Vec<i64> = (0..64).collect();
    let entry_bytes =
        measure_bundle_data_bytes(|| single_slot_bundle(&descriptor, 0x07, payload.as_slice()))
            .await;
    let header_len = test_header_size();
    let chunk_file_len = header_len + entry_bytes;
    let slack = cmp::max(1, entry_bytes / 2);
    let max_wal_size = chunk_file_len + header_len + slack;

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x53; 16], FlushPolicy::Immediate)
            .with_rotation_target(1)
            .with_max_rotated_files(4)
            .with_max_wal_size(max_wal_size),
    )
    .await
    .expect("writer");

    let first_bundle = single_slot_bundle(&descriptor, 0x07, payload.as_slice());
    let first_offset = writer
        .append_bundle(&first_bundle)
        .await
        .expect("first append rotates under cap");
    assert!(rotated_path_for(&wal_path, 1).exists());

    let second_bundle = single_slot_bundle(&descriptor, 0x08, payload.as_slice());
    let err = writer
        .append_bundle(&second_bundle)
        .await
        .expect_err("second rotation should exceed size cap");
    match err {
        WalError::WalAtCapacity(message) => {
            assert!(
                message.contains("size cap"),
                "unexpected error message: {message}",
            );
        }
        other => panic!("expected WalAtCapacity, got {other:?}"),
    }

    // Cursor past the first entry (using WAL position, not file offset)
    let cursor = WalConsumerCursor {
        safe_offset: first_offset.next_offset,
        safe_sequence: first_offset.sequence,
    };
    writer
        .persist_cursor(&cursor)
        .await
        .expect("record cursor purges rotated chunks");

    assert!(
        !rotated_path_for(&wal_path, 1).exists(),
        "all rotated chunks should be purged",
    );
    assert!(
        !rotated_path_for(&wal_path, 2).exists(),
        "purge removes even shifted chunks",
    );

    let third_bundle = single_slot_bundle(&descriptor, 0x09, payload.as_slice());
    let _ = writer
        .append_bundle(&third_bundle)
        .await
        .expect("append succeeds once space is reclaimed");
}

#[tokio::test]
async fn wal_writer_ignores_invalid_cursor_sidecar() {
    let (_dir, wal_path) = temp_wal("bad_sidecar.wal");

    // Create the WAL header so the file exists.
    {
        let _writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0x11; 16],
            FlushPolicy::Immediate,
        ))
        .await
        .expect("writer");
    }

    let sidecar_path = wal_path.parent().expect("dir").join("quiver.wal.cursor");
    // Write a truncated sidecar file (shorter than minimum length)
    std::fs::write(&sidecar_path, vec![0u8; 8]).expect("write corrupt");

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0x11; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("reopen");

    let descriptor = logs_descriptor();
    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[7])],
    );
    let offset = writer.append_bundle(&bundle).await.expect("append");

    // Use the WAL position from the append result, not file length
    let cursor = WalConsumerCursor {
        safe_offset: offset.next_offset,
        safe_sequence: offset.sequence,
    };
    writer.persist_cursor(&cursor).await.expect("record cursor");
    drop(writer);

    let state = CursorSidecar::read_from_sync(&sidecar_path).expect("sidecar");
    assert_eq!(state.wal_position, offset.next_offset);
}

#[tokio::test]
async fn wal_writer_flushes_after_unflushed_byte_threshold() {
    let (_dir, wal_path) = temp_wal("flush_bytes.wal");

    let descriptor = logs_descriptor();
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        FlushPolicy::EveryNBytes(1),
    ))
    .await
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x99, &[1, 2, 3])],
    );

    writer.test_set_last_flush(Instant::now());
    let before = writer.test_last_flush();
    let _offset = writer
        .append_bundle(&bundle)
        .await
        .expect("append triggers flush");
    assert!(writer.test_last_flush() > before);
}

#[tokio::test]
async fn wal_writer_flushes_pending_bytes_on_drop() {
    writer_test_support::reset_flush_notifications();

    let (_dir, wal_path) = temp_wal("flush_drop.wal");

    let descriptor = logs_descriptor();
    // Use a duration-based flush with a very long interval so we don't flush during the test
    let writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        FlushPolicy::EveryDuration(Duration::from_secs(3600)),
    ))
    .await
    .expect("writer");

    {
        let mut writer = writer;
        let bundle = FixtureBundle::new(
            descriptor,
            vec![FixtureSlot::new(SlotId::new(0), 0x55, &[42])],
        );
        let _ = writer.append_bundle(&bundle).await.expect("append");
        assert!(!writer_test_support::take_drop_flush_notification());
    }

    assert!(writer_test_support::take_drop_flush_notification());
}

#[tokio::test]
async fn wal_reader_rewind_allows_replay_from_start() {
    let (_dir, wal_path) = temp_wal("rewind.wal");
    let descriptor = logs_descriptor();

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0x10; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );
    let _ = writer
        .append_bundle(&first_bundle)
        .await
        .expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[2])],
    );
    let _ = writer
        .append_bundle(&second_bundle)
        .await
        .expect("second append");
    drop(writer);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    {
        let mut iter = reader.iter_from(0).expect("iterator");
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }

    reader.rewind().expect("rewind succeeds");

    let mut iter = reader.iter_from(0).expect("iterator after rewind");
    let entry = iter.next().expect("entry present").expect("entry ok");
    assert_eq!(entry.sequence, 0);
}

#[tokio::test]
async fn wal_reader_iterator_stays_finished_after_eof() {
    let (_dir, wal_path) = temp_wal("empty.wal");
    {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&wal_path)
            .expect("create wal");
        WalHeader::new([0x44; 16])
            .write_to_sync(&mut file)
            .expect("header");
    }

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    assert!(iter.next().is_none(), "no entries present");
    assert!(iter.next().is_none(), "iterator remains finished");
}

#[tokio::test]
async fn wal_writer_restores_sequence_after_restart() {
    let (_dir, wal_path) = temp_wal("sequence_resume.wal");
    let descriptor = logs_descriptor();
    let bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );

    {
        let mut writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0xAA; 16],
            FlushPolicy::Immediate,
        ))
        .await
        .expect("writer");

        let _ = writer.append_bundle(&bundle).await.expect("first append");
        let _ = writer.append_bundle(&bundle).await.expect("second append");
    }

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0xAA; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer reopen");

    let third = writer.append_bundle(&bundle).await.expect("third append");
    assert_eq!(third.sequence, 2, "sequence should continue across restart");
}

#[tokio::test]
async fn wal_writer_preflight_rejects_when_size_cap_hit() {
    let (_dir, wal_path) = temp_wal("size_cap.wal");
    let descriptor = logs_descriptor();
    let bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[4, 5, 6])],
    );

    let hash = [0x33; 16];
    {
        let mut writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            hash,
            FlushPolicy::Immediate,
        ))
        .await
        .expect("writer");
        let _ = writer.append_bundle(&bundle).await.expect("first append");
    }

    let wal_cap = std::fs::metadata(&wal_path).expect("metadata").len();

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), hash, FlushPolicy::Immediate)
            .with_max_wal_size(wal_cap),
    )
    .await
    .expect("writer with cap");
    let err = writer.append_bundle(&bundle).await.expect_err("cap hit");
    assert!(matches!(err, WalError::WalAtCapacity(_)));

    // Verify failed append did not persist
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let only = iter.next().expect("entry").expect("ok");
    assert!(iter.next().is_none(), "failed append must not persist");
    drop(iter);
    drop(reader);
    drop(writer);

    // Reopening with higher cap allows appending
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), hash, FlushPolicy::Immediate)
            .with_max_wal_size(u64::MAX),
    )
    .await
    .expect("writer after cap removed");
    let retry = writer.append_bundle(&bundle).await.expect("retry append");
    assert_eq!(retry.sequence, only.sequence + 1);
}

#[tokio::test]
async fn wal_writer_preflight_rejects_when_rotated_file_cap_hit() {
    let (_dir, wal_path) = temp_wal("rotated_file_cap.wal");
    let descriptor = logs_descriptor();
    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x03, &[7, 8])],
    );

    let hash = [0x77; 16];
    let constrained_opts = WalWriterOptions::new(wal_path.clone(), hash, FlushPolicy::Immediate)
        .with_rotation_target(1)
        .with_max_rotated_files(1);

    {
        let mut writer = WalWriter::open(constrained_opts.clone())
            .await
            .expect("writer");
        let first = writer.append_bundle(&bundle).await.expect("first append");
        assert_eq!(first.sequence, 0);
    }

    let rotated_path = rotated_path_for(&wal_path, 1);
    assert!(
        rotated_path.exists(),
        "rotation should have produced rotated file"
    );

    {
        let mut writer = WalWriter::open(constrained_opts)
            .await
            .expect("writer with cap");
        let err = writer
            .append_bundle(&bundle)
            .await
            .expect_err("rotated file cap hit");
        assert!(matches!(err, WalError::WalAtCapacity(_)));
    }

    let len = std::fs::metadata(&wal_path).expect("metadata").len();
    assert!(
        len <= test_header_size(),
        "active wal should contain at most header bytes"
    );

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), hash, FlushPolicy::Immediate)
            .with_rotation_target(1)
            .with_max_rotated_files(2),
    )
    .await
    .expect("writer with higher rotated file cap");
    let retry = writer
        .append_bundle(&bundle)
        .await
        .expect("retry append succeeds once cap raised");
    assert_eq!(retry.sequence, 1);
}

#[tokio::test]
async fn wal_reader_errors_on_truncated_entry_length() {
    let (_dir, wal_path) = temp_wal("length_trunc.wal");
    {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&wal_path)
            .expect("create wal");
        WalHeader::new([0x55; 16])
            .write_to_sync(&mut file)
            .expect("header");
        file.write_all(&[0xAA, 0xBB])
            .expect("write partial entry length");
    }

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::UnexpectedEof("entry length"))) => {}
        other => panic!("expected entry length eof, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_reports_crc_mismatch() {
    let (_dir, wal_path) = temp_wal("crc.wal");
    let descriptor = logs_descriptor();
    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x55, &[7, 8])],
    );

    let options = WalWriterOptions::new(wal_path.clone(), [1; 16], FlushPolicy::Immediate);
    let mut writer = WalWriter::open(options).await.expect("writer");
    let _offset = writer.append_bundle(&bundle).await.expect("append");
    drop(writer);

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&wal_path)
        .expect("open file");
    let _ = file.seek(SeekFrom::End(-1)).expect("seek to crc");
    let mut byte = [0u8; 1];
    file.read_exact(&mut byte).expect("read crc");
    byte[0] ^= 0xFF;
    let _ = file.seek(SeekFrom::End(-1)).expect("rewind");
    file.write_all(&byte).expect("overwrite crc");

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    match iter.next() {
        Some(Err(WalError::CrcMismatch { .. })) => {}
        other => panic!("expected crc mismatch, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_rejects_unsupported_entry_type() {
    let body = encode_entry_header(0xAA, 0);
    let (_dir, wal_path) = write_single_entry(&body);
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");

    match iter.next() {
        Some(Err(WalError::UnsupportedEntry(ty))) => assert_eq!(ty, 0xAA),
        other => panic!("expected unsupported entry, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_errors_on_truncated_slot_header() {
    let mut body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 1);
    body.extend_from_slice(&SlotId::new(0).0.to_le_bytes());
    let (_dir, wal_path) = write_single_entry(&body);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::InvalidEntry(message))) => {
            assert_eq!(message, "slot header truncated")
        }
        other => panic!("expected truncated slot header error, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_errors_on_truncated_slot_payload() {
    let mut body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 1);
    body.extend_from_slice(&SlotId::new(0).0.to_le_bytes());
    body.extend_from_slice(&[0x7Au8; SCHEMA_FINGERPRINT_LEN]);
    body.extend_from_slice(&1u32.to_le_bytes());
    body.extend_from_slice(&4u32.to_le_bytes());
    body.extend_from_slice(&[0x01, 0x02]);

    let (_dir, wal_path) = write_single_entry(&body);
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::InvalidEntry(message))) => assert_eq!(message, "slot payload"),
        other => panic!("expected slot payload error, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_errors_on_entry_header_underflow() {
    let body = vec![0u8; ENTRY_HEADER_LEN - 1];
    let (_dir, wal_path) = write_single_entry(&body);
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::InvalidEntry(message))) => {
            assert_eq!(message, "body shorter than header")
        }
        other => panic!("expected header underflow error, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_errors_on_unexpected_trailing_bytes() {
    let mut body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    body.push(0xFF);
    let (_dir, wal_path) = write_single_entry(&body);
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::InvalidEntry(message))) => {
            assert_eq!(message, "unexpected trailing bytes")
        }
        other => panic!("expected trailing bytes error, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_errors_on_truncated_entry_body() {
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);
    truncate_file_from_end(&wal_path, 6);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    // The reader rejects entries whose declared length exceeds remaining file bytes.
    // This guards against corrupted/malicious length values causing excessive allocation.
    match iter.next() {
        Some(Err(WalError::InvalidEntry("entry length exceeds remaining file size"))) => {}
        other => panic!("expected entry length overflow error, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_errors_on_truncated_entry_crc() {
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);
    truncate_file_from_end(&wal_path, 2);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::UnexpectedEof("entry crc"))) => {}
        other => panic!("expected entry crc EOF, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_reports_io_error_during_entry_length_read() {
    let _guard = FailureGuard::new();
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);

    test_support::fail_next_read(ReadFailure::Length);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::Io(_))) => {}
        other => panic!("expected io error for entry length read, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_reports_io_error_during_entry_body_read() {
    let _guard = FailureGuard::new();
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);

    test_support::fail_next_read(ReadFailure::Body);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::Io(_))) => {}
        other => panic!("expected io error for entry body read, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_reports_io_error_during_entry_crc_read() {
    let _guard = FailureGuard::new();
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);

    test_support::fail_next_read(ReadFailure::Crc);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::Io(_))) => {}
        other => panic!("expected io error for entry crc read, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_iter_from_respects_offsets() {
    let (_dir, wal_path) = temp_wal("offsets.wal");
    let descriptor = logs_descriptor();

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0xCC; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );
    let first_offset = writer
        .append_bundle(&first_bundle)
        .await
        .expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[2])],
    );
    let second_offset = writer
        .append_bundle(&second_bundle)
        .await
        .expect("second append");
    drop(writer);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    let entry_one = iter.next().expect("first entry").expect("ok");
    let entry_two = iter.next().expect("second entry").expect("ok");
    assert_eq!(entry_one.sequence, 0);
    assert_eq!(entry_two.sequence, 1);
    assert_eq!(entry_one.next_offset, entry_two.offset.position);

    let mut cursor = WalConsumerCursor::default();
    cursor.increment(&entry_one);
    assert_eq!(cursor.safe_offset, entry_one.next_offset);
    assert_eq!(cursor.safe_sequence, entry_one.sequence);

    let mut reader_from_offset = WalReader::open(&wal_path).expect("reader");
    let mut iter_from_second = reader_from_offset
        .iter_from(second_offset.position)
        .expect("iter from offset");
    let entry = iter_from_second.next().expect("entry").expect("ok");
    assert_eq!(entry.sequence, 1);
    assert!(iter_from_second.next().is_none());
    assert_eq!(first_offset.sequence, 0);
    assert_eq!(second_offset.sequence, 1);
}

#[tokio::test]
async fn wal_reader_iter_from_partial_length_reports_error() {
    let (_dir, wal_path) = temp_wal("partial_offset.wal");
    let descriptor = logs_descriptor();

    {
        let mut writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0x44; 16],
            FlushPolicy::Immediate,
        ))
        .await
        .expect("writer");

        let bundle = FixtureBundle::new(
            descriptor.clone(),
            vec![FixtureSlot::new(SlotId::new(0), 0xAA, &[1, 2])],
        );
        let _ = writer.append_bundle(&bundle).await.expect("append");
    }

    let metadata_len = std::fs::metadata(&wal_path).expect("metadata").len();
    // Convert file position to WAL position for iter_from
    // We want to start reading 2 bytes before the end of data
    let file_offset = metadata_len.saturating_sub(2);
    let wal_position = file_offset.saturating_sub(test_header_size());

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader
        .iter_from(wal_position)
        .expect("iterator from misaligned offset");
    match iter.next() {
        Some(Err(WalError::UnexpectedEof("entry length"))) => {}
        other => panic!("expected entry length eof, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_iter_from_offset_past_file_returns_none() {
    let (_dir, wal_path) = temp_wal("past_end_offset.wal");
    let descriptor = logs_descriptor();

    {
        let mut writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0x55; 16],
            FlushPolicy::Immediate,
        ))
        .await
        .expect("writer");

        let bundle = FixtureBundle::new(
            descriptor,
            vec![FixtureSlot::new(SlotId::new(0), 0xCC, &[3, 4])],
        );
        let _ = writer.append_bundle(&bundle).await.expect("append");
    }

    let metadata_len = std::fs::metadata(&wal_path).expect("metadata").len();
    let offset_beyond_file = metadata_len + 128;

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader
        .iter_from(offset_beyond_file)
        .expect("iterator past end");
    assert!(iter.next().is_none());
}

#[tokio::test]
async fn wal_writer_reader_handles_all_bitmap_slots() {
    let (_dir, wal_path) = temp_wal("all_slots.wal");
    let descriptor = descriptor_with_all_slots();

    let slots: Vec<_> = (0u16..64)
        .map(|id| {
            let values = [id as i64, (id as i64) * 2 + 1];
            FixtureSlot::new(SlotId::new(id), id as u8, &values)
        })
        .collect();
    let bundle = FixtureBundle::new(descriptor.clone(), slots);

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0xAA; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");
    let _ = writer.append_bundle(&bundle).await.expect("append");
    drop(writer);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let entry = iter.next().expect("entry present").expect("ok");
    assert_eq!(entry.slot_bitmap, u64::MAX, "all 64 slots set");
    assert_eq!(entry.slots.len(), 64);

    for slot in entry.slots {
        assert_eq!(slot.payload_len, slot.payload.len() as u32);
        assert_eq!(slot.row_count, 2);
    }

    assert!(iter.next().is_none());
}

#[tokio::test]
async fn wal_writer_handles_large_payload_batches() {
    let (_dir, wal_path) = temp_wal("large_payload.wal");
    let descriptor = BundleDescriptor::new(vec![
        slot_descriptor(0, "Logs"),
        slot_descriptor(1, "LogsAttrs"),
        slot_descriptor(2, "ScopeAttrs"),
    ]);

    let slot_specs = [
        (SlotId::new(0), 0x10, 6_000usize, "alpha", 256usize),
        (SlotId::new(1), 0x20, 5_000usize, "beta", 512usize),
        (SlotId::new(2), 0x30, 4_000usize, "gamma", 768usize),
    ];

    let slots: Vec<_> = slot_specs
        .iter()
        .map(|(id, seed, rows, prefix, repeat)| {
            FixtureSlot::with_batch(*id, *seed, build_complex_batch(*rows, prefix, *repeat))
        })
        .collect();

    let bundle = FixtureBundle::new(descriptor.clone(), slots);

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0xBB; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    let _ = writer
        .append_bundle(&bundle)
        .await
        .expect("append large payload");
    drop(writer);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let entry = iter.next().expect("entry present").expect("ok");

    let expected_bitmap = (1u64 << 0) | (1u64 << 1) | (1u64 << 2);
    assert_eq!(entry.slot_bitmap, expected_bitmap);
    assert_eq!(entry.slots.len(), 3);

    for (slot, (_, _, expected_rows, _, repeat)) in entry.slots.iter().zip(slot_specs.iter()) {
        assert!(slot.payload_len as usize >= expected_rows * repeat);
        let decoded = read_batch(&slot.payload);
        assert_eq!(decoded.num_rows(), *expected_rows);
        assert_eq!(slot.row_count as usize, *expected_rows);
    }

    assert!(iter.next().is_none());
}

/// Verifies that `WalWriter::open` automatically truncates trailing garbage
/// when the file has been truncated mid-entry (simulating a crash where bytes
/// were lost). This complements `wal_writer_auto_truncates_trailing_garbage_on_open`
/// which tests garbage appended at the end.
#[tokio::test]
async fn wal_writer_auto_truncates_after_mid_entry_truncation() {
    let (_dir, wal_path) = temp_wal("recovery.wal");
    let descriptor = logs_descriptor();
    let hash = [0x99; 16];

    // Write two valid entries
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        hash,
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );
    let _ = writer
        .append_bundle(&first_bundle)
        .await
        .expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[2])],
    );
    let _ = writer
        .append_bundle(&second_bundle)
        .await
        .expect("second append");
    drop(writer);

    // Record valid file length after first entry for later comparison
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    let first_entry = iter.next().expect("first entry").expect("ok");
    let first_entry_end = first_entry.next_offset;
    drop(reader);

    // Simulate a crash that truncates the file in the middle of the second
    // entry, losing bytes (not appending garbage)
    let first_entry_file_end = wal_position_to_file_offset(first_entry_end);
    {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&wal_path)
            .expect("open for truncation");
        // Truncate one byte into the second entry
        file.set_len(first_entry_file_end + 1)
            .expect("truncate inside entry");
    }

    // Reopen the writer - it should auto-truncate to end of first entry
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        hash,
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer reopens and auto-truncates");

    // Verify file was truncated to first entry boundary
    let file_len = std::fs::metadata(&wal_path).expect("metadata").len();
    assert_eq!(
        file_len, first_entry_file_end,
        "file should be truncated to end of first valid entry"
    );

    // Append a recovery entry - should continue from sequence 1
    let recovery_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x03, &[3])],
    );
    let recovery_offset = writer
        .append_bundle(&recovery_bundle)
        .await
        .expect("append recovery entry");
    assert_eq!(recovery_offset.position, first_entry_end); // WAL position
    assert_eq!(
        recovery_offset.sequence, 1,
        "sequence continues from last valid"
    );
    drop(writer);

    // Verify the WAL now has exactly two entries: first + recovery
    let mut reader = WalReader::open(&wal_path).expect("reader after recovery");
    let mut iter = reader.iter_from(0).expect("iterator");
    let first = iter.next().expect("first entry").expect("ok");
    assert_eq!(first.sequence, 0);
    let recovery = iter.next().expect("recovery entry").expect("ok");
    assert_eq!(recovery.sequence, 1);
    assert_eq!(
        recovery.slots.first().expect("slot").schema_fingerprint,
        [0x03; 32]
    );
    assert!(iter.next().is_none(), "only two entries should exist");
}

/// Verifies that `WalWriter::open` automatically truncates trailing garbage
/// left by a crash that interrupted a write. This test simulates the scenario
/// where a process crashes mid-write, leaving a partial entry at the end of
/// the WAL file. On reopen, the writer should automatically detect and remove
/// this garbage so that subsequent appends resume from a clean boundary.
#[tokio::test]
async fn wal_writer_auto_truncates_trailing_garbage_on_open() {
    let (_dir, wal_path) = temp_wal("auto_truncate.wal");
    let descriptor = logs_descriptor();
    let hash = [0xAA; 16];

    // Write two valid entries to establish a baseline.
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        hash,
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    let bundle1 = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );
    let _ = writer.append_bundle(&bundle1).await.expect("first append");

    let bundle2 = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[2])],
    );
    let _ = writer.append_bundle(&bundle2).await.expect("second append");
    drop(writer);

    // Get the file length before corruption
    let valid_len = std::fs::metadata(&wal_path).expect("metadata").len();

    // Simulate a crash mid-write by appending garbage bytes to the WAL file
    // (this simulates a partial entry header or body that was interrupted).
    {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .expect("open for corruption");
        // Write partial "entry length" field plus some garbage
        file.write_all(&[0xFF, 0x00, 0x10, 0x00, 0xDE, 0xAD, 0xBE, 0xEF])
            .expect("append garbage");
    }

    let corrupted_len = std::fs::metadata(&wal_path).expect("metadata").len();
    assert!(corrupted_len > valid_len, "garbage should extend file");

    // Reopen the writer - it should automatically truncate the garbage
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        hash,
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer reopens and truncates garbage");

    // File should be truncated back to valid length
    let after_open_len = std::fs::metadata(&wal_path).expect("metadata").len();
    assert_eq!(
        after_open_len, valid_len,
        "garbage should be truncated on open"
    );

    // Append a third entry - it should continue from the correct position
    let bundle3 = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x03, &[3])],
    );
    let offset3 = writer.append_bundle(&bundle3).await.expect("third append");

    // The third entry should have sequence 2 (continuing from 0, 1)
    assert_eq!(offset3.sequence, 2, "sequence should continue correctly");
    drop(writer);

    // Verify all three entries are readable and valid
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");

    let e1 = iter.next().expect("first entry").expect("ok");
    assert_eq!(e1.sequence, 0);
    assert_eq!(e1.slots.first().unwrap().schema_fingerprint, [0x01; 32]);

    let e2 = iter.next().expect("second entry").expect("ok");
    assert_eq!(e2.sequence, 1);
    assert_eq!(e2.slots.first().unwrap().schema_fingerprint, [0x02; 32]);

    let e3 = iter.next().expect("third entry").expect("ok");
    assert_eq!(e3.sequence, 2);
    assert_eq!(e3.slots.first().unwrap().schema_fingerprint, [0x03; 32]);

    assert!(iter.next().is_none(), "only three entries should exist");
}

#[tokio::test]
async fn wal_writer_rejects_segment_config_mismatch() {
    let (_dir, wal_path) = temp_wal("mismatch.wal");
    let original_hash = [0xAA; 16];

    // Create a WAL with one config hash.
    {
        let options =
            WalWriterOptions::new(wal_path.clone(), original_hash, FlushPolicy::Immediate);
        let _writer = WalWriter::open(options).await.expect("initial open");
    }

    // Attempt to reopen with a different hash.
    let different_hash = [0xBB; 16];
    let options = WalWriterOptions::new(wal_path, different_hash, FlushPolicy::Immediate);
    match WalWriter::open(options).await {
        Err(WalError::SegmentConfigMismatch { expected, found }) => {
            assert_eq!(expected, different_hash);
            assert_eq!(found, original_hash);
        }
        other => panic!("expected segment config mismatch, got {:?}", other),
    }
}

#[tokio::test]
async fn wal_reader_detects_unexpected_segment_config() {
    let (_dir, wal_path) = temp_wal("reader_mismatch.wal");
    let stored_hash = [0xDD; 16];

    // Write a WAL with a known config hash.
    {
        let options = WalWriterOptions::new(wal_path.clone(), stored_hash, FlushPolicy::Immediate);
        let _writer = WalWriter::open(options).await.expect("writer");
    }

    // Reader opens successfully and exposes the stored hash for the caller to verify.
    let reader = WalReader::open(&wal_path).expect("reader opens");
    assert_eq!(reader.segment_cfg_hash(), stored_hash);

    // Caller can decide what to do if it doesn't match expectations.
    let expected_hash = [0xEE; 16];
    assert_ne!(
        reader.segment_cfg_hash(),
        expected_hash,
        "caller detects mismatch"
    );
}

#[tokio::test]
async fn wal_reader_fails_on_corrupt_header_version() {
    let (_dir, wal_path) = temp_wal("bad_version.wal");

    // Create a valid WAL first.
    {
        let options = WalWriterOptions::new(wal_path.clone(), [0x11; 16], FlushPolicy::Immediate);
        let _writer = WalWriter::open(options).await.expect("writer");
    }

    // Corrupt the version field in the header.
    {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&wal_path)
            .expect("open");
        // Version is at offset WAL_MAGIC.len() (10 bytes).
        let _ = file.seek(SeekFrom::Start(10)).expect("seek");
        file.write_all(&99u16.to_le_bytes())
            .expect("corrupt version");
    }

    match WalReader::open(&wal_path) {
        Err(WalError::InvalidHeader("unsupported version")) => {}
        other => panic!("expected unsupported version error, got {:?}", other),
    }
}

#[derive(Clone, Copy)]
struct CrashCase {
    name: &'static str,
    injection: writer_test_support::CrashInjection,
}

#[tokio::test]
async fn wal_writer_recovers_from_crash_resilience_scenarios() {
    let cases = [CrashCase {
        name: "sidecar_pre_rename",
        injection: writer_test_support::CrashInjection::BeforeSidecarRename,
    }];

    for case in cases {
        run_crash_case(case).await;
    }
}

async fn run_crash_case(case: CrashCase) {
    writer_test_support::reset_flush_notifications();

    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join(format!("crash_{}.wal", case.name));
    let descriptor = logs_descriptor();
    let options = WalWriterOptions::new(wal_path.clone(), [0xC7; 16], FlushPolicy::Immediate)
        .with_rotation_target(32 * 1024)
        .with_max_rotated_files(4);

    let mut writer = WalWriter::open(options.clone()).await.expect("writer");
    for value in 0..4 {
        let bundle = FixtureBundle::new(
            descriptor.clone(),
            vec![FixtureSlot::with_batch(
                SlotId::new(0),
                0x60 + value as u8,
                build_complex_batch(256, "crash", 1024),
            )],
        );
        let _ = writer.append_bundle(&bundle).await.expect("append bundle");
    }

    for value in 0..4 {
        let bundle = FixtureBundle::new(
            descriptor.clone(),
            vec![FixtureSlot::new(
                SlotId::new(0),
                0x40 + value as u8,
                &[(value + 1) as i64],
            )],
        );
        let _ = writer.append_bundle(&bundle).await.expect("append bundle");
    }

    let cursor = wal_cursor_after_entries(&wal_path, 2);
    assert!(
        cursor.safe_offset > test_header_size(),
        "{}: cursor safe offset must exceed header",
        case.name
    );
    writer_test_support::inject_crash(case.injection);
    let err = match writer.persist_cursor(&cursor).await {
        Ok(_) => panic!("{}: crash injection did not trigger", case.name),
        Err(err) => err,
    };
    assert!(
        matches!(err, WalError::InjectedCrash(_)),
        "unexpected error: {:?}",
        err
    );

    writer.test_force_crash();

    assert_crash_recovery(&options, &descriptor, case.name, &cursor).await;
    writer_test_support::reset_flush_notifications();
}

fn wal_cursor_after_entries(path: &Path, entry_count: usize) -> WalConsumerCursor {
    let mut reader = WalReader::open(path).expect("reader for cursor");
    let mut iter = reader.iter_from(0).expect("cursor iterator");
    let mut cursor = WalConsumerCursor::default();
    for idx in 0..entry_count {
        let bundle = iter
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "not enough entries for cursor (wanted {}, stopped at {})",
                    entry_count, idx
                )
            })
            .expect("entry ok while building cursor");
        cursor.increment(&bundle);
    }
    cursor
}

async fn assert_crash_recovery(
    options: &WalWriterOptions,
    descriptor: &BundleDescriptor,
    case_name: &str,
    cursor: &WalConsumerCursor,
) {
    assert_reader_clean(&options.path, cursor.safe_offset, case_name);

    let mut writer = WalWriter::open(options.clone())
        .await
        .expect("writer reopen");
    let repair_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0xF0, &[99])],
    );
    let _ = writer
        .append_bundle(&repair_bundle)
        .await
        .expect("append after crash");
    drop(writer);

    assert_reader_clean(&options.path, cursor.safe_offset, case_name);

    let sidecar_path = options
        .path
        .parent()
        .expect("wal dir")
        .join("quiver.wal.cursor");
    if sidecar_path.exists() {
        let sidecar = CursorSidecar::read_from_sync(&sidecar_path).expect("sidecar readable");
        let total_logical = total_logical_bytes(&options.path);
        assert!(
            sidecar.wal_position <= total_logical,
            "{case_name}: logical cursor must stay within logical stream"
        );
    }
}

fn assert_reader_clean(path: &Path, offset: u64, case_name: &str) {
    let mut reader = WalReader::open(path)
        .unwrap_or_else(|err| panic!("{}: reader open failed: {:?}", case_name, err));
    let iter = reader
        .iter_from(offset)
        .unwrap_or_else(|err| panic!("{}: iterator init failed: {:?}", case_name, err));
    for entry in iter {
        let _ = entry.unwrap_or_else(|err| panic!("{}: wal entry error {:?}", case_name, err));
    }
}

#[tokio::test]
async fn wal_writer_flushes_with_bytes_or_duration_policy_on_bytes() {
    // Test that BytesOrDuration flushes when byte threshold is exceeded
    let (_dir, wal_path) = temp_wal("flush_bytes_or_duration_bytes.wal");

    let descriptor = logs_descriptor();
    // Set a very small byte threshold and very long duration
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        FlushPolicy::BytesOrDuration {
            bytes: 1,
            duration: Duration::from_secs(3600),
        },
    ))
    .await
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x99, &[1, 2, 3])],
    );

    let before = writer.test_last_flush();
    let _offset = writer
        .append_bundle(&bundle)
        .await
        .expect("append triggers flush via bytes threshold");
    assert!(
        writer.test_last_flush() > before,
        "flush should occur when bytes threshold exceeded"
    );
}

#[tokio::test]
async fn wal_writer_flushes_with_bytes_or_duration_policy_on_duration() {
    // Test that BytesOrDuration flushes when duration threshold is exceeded
    let (_dir, wal_path) = temp_wal("flush_bytes_or_duration_time.wal");

    let descriptor = logs_descriptor();
    // Set a very large byte threshold and very short duration
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        FlushPolicy::BytesOrDuration {
            bytes: u64::MAX,
            duration: Duration::from_millis(1),
        },
    ))
    .await
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x99, &[1, 2, 3])],
    );

    // Force the last_flush to be in the past
    writer.test_set_last_flush(Instant::now() - Duration::from_secs(1));
    let before = writer.test_last_flush();

    let _offset = writer
        .append_bundle(&bundle)
        .await
        .expect("append triggers flush via duration threshold");
    assert!(
        writer.test_last_flush() > before,
        "flush should occur when duration threshold exceeded"
    );
}

#[tokio::test]
async fn wal_writer_skips_flush_when_neither_threshold_met() {
    // Test that BytesOrDuration does NOT flush when neither threshold is met
    writer_test_support::reset_flush_notifications();

    let (_dir, wal_path) = temp_wal("flush_bytes_or_duration_skip.wal");

    let descriptor = logs_descriptor();
    // Set very large byte threshold and very long duration
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        FlushPolicy::BytesOrDuration {
            bytes: u64::MAX,
            duration: Duration::from_secs(3600),
        },
    ))
    .await
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x99, &[1, 2, 3])],
    );

    // Ensure last_flush is recent
    writer.test_set_last_flush(Instant::now());
    let before = writer.test_last_flush();

    let _offset = writer
        .append_bundle(&bundle)
        .await
        .expect("append without flush");

    // last_flush should not have changed (no flush occurred during append)
    assert_eq!(
        writer.test_last_flush(),
        before,
        "flush should NOT occur when neither threshold is met"
    );
}

#[tokio::test]
async fn wal_writer_appends_empty_bundle_with_no_slots() {
    // Test that a bundle with zero populated slots can be appended
    let (_dir, wal_path) = temp_wal("empty_bundle.wal");

    let descriptor = logs_descriptor();
    let mut writer = open_test_writer(wal_path.clone(), [0x11; 16]).await;

    // Create a bundle with no slots populated (empty slots vec)
    let empty_bundle = FixtureBundle::new(descriptor.clone(), vec![]);
    let offset = writer
        .append_bundle(&empty_bundle)
        .await
        .expect("empty bundle should append successfully");
    assert_eq!(offset.sequence, 0);

    // Append a normal bundle after to verify the writer is still functional
    let offset2 = writer
        .append_bundle(&single_slot_bundle(&descriptor, 0x22, &[1, 2, 3]))
        .await
        .expect("normal bundle after empty");
    assert_eq!(offset2.sequence, 1);
    drop(writer);

    // Verify both entries can be read back
    let entries = read_all_entries(&wal_path);
    assert_eq!(entries.len(), 2);

    assert_eq!(entries[0].sequence, 0);
    assert_eq!(
        entries[0].slot_bitmap, 0,
        "empty bundle should have zero bitmap"
    );
    assert!(
        entries[0].slots.is_empty(),
        "empty bundle should have no slots"
    );

    assert_eq!(entries[1].sequence, 1);
    assert_eq!(entries[1].slots.len(), 1);
}

#[tokio::test]
async fn wal_writer_rejects_cursor_sequence_regression() {
    // Test that advancing cursor with a lower sequence number fails
    let (_dir, wal_path) = temp_wal("sequence_regression.wal");

    let descriptor = logs_descriptor();
    let mut writer = open_test_writer(wal_path.clone(), [0x33; 16]).await;

    // Append three bundles
    for i in 0..3 {
        let _ = writer
            .append_bundle(&single_slot_bundle(&descriptor, i, &[i as i64]))
            .await
            .expect("append");
    }

    // Read entries to get offsets
    let entries = read_entries(&wal_path, 2);

    // First, advance to the second entry (sequence=1)
    let cursor_at_second = WalConsumerCursor {
        safe_offset: entries[1].next_offset,
        safe_sequence: entries[1].sequence,
    };
    writer
        .persist_cursor(&cursor_at_second)
        .await
        .expect("advance to second entry");

    // Now try to regress to the first entry (sequence=0)
    let cursor_at_first = WalConsumerCursor {
        safe_offset: entries[0].next_offset,
        safe_sequence: entries[0].sequence, // sequence=0, which is less than 1
    };

    match writer.persist_cursor(&cursor_at_first).await {
        Err(WalError::InvalidConsumerCursor(msg)) => {
            assert!(
                msg.contains("regressed"),
                "expected regression error, got: {msg}"
            );
        }
        other => panic!("expected sequence regression error, got {other:?}"),
    }
}

#[tokio::test]
async fn wal_writer_rejects_cursor_offset_regression() {
    // Test that advancing cursor with a lower offset fails
    let (_dir, wal_path) = temp_wal("offset_regression.wal");

    let descriptor = logs_descriptor();
    let mut writer = open_test_writer(wal_path.clone(), [0x44; 16]).await;

    // Append two bundles
    for i in 0..2 {
        let _ = writer
            .append_bundle(&single_slot_bundle(&descriptor, i, &[i as i64]))
            .await
            .expect("append");
    }

    // Read entries to get offsets
    let entries = read_entries(&wal_path, 2);

    // Advance to the second entry
    let cursor_at_second = WalConsumerCursor {
        safe_offset: entries[1].next_offset,
        safe_sequence: entries[1].sequence,
    };
    writer
        .persist_cursor(&cursor_at_second)
        .await
        .expect("advance to second entry");

    // Now try to advance with a higher sequence but lower offset
    // This simulates a malformed cursor
    let bad_cursor = WalConsumerCursor {
        safe_offset: entries[0].next_offset, // lower offset than before
        safe_sequence: entries[1].sequence + 1, // higher sequence to pass that check
    };

    match writer.persist_cursor(&bad_cursor).await {
        Err(WalError::InvalidConsumerCursor(msg)) => {
            assert!(
                msg.contains("regressed"),
                "expected offset regression error, got: {msg}"
            );
        }
        other => panic!("expected offset regression error, got {other:?}"),
    }
}

#[tokio::test]
async fn wal_writer_handles_bundle_with_unpopulated_descriptor_slots() {
    // Test a bundle where the descriptor has slots but payload() returns None for some
    let (_dir, wal_path) = temp_wal("sparse_bundle.wal");

    // Descriptor declares 3 slots
    let descriptor = BundleDescriptor::new(vec![
        slot_descriptor(0, "Logs"),
        slot_descriptor(1, "LogAttrs"),
        slot_descriptor(2, "ScopeAttrs"),
    ]);

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0x55; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    // Only populate slot 1 (middle slot), leaving 0 and 2 empty
    let sparse_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(1), 0xBB, &[100, 200])],
    );

    let offset = writer
        .append_bundle(&sparse_bundle)
        .await
        .expect("sparse bundle appends");
    assert_eq!(offset.sequence, 0);
    drop(writer);

    // Verify the entry
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let entry = iter.next().expect("entry").expect("ok");

    // Only bit 1 should be set in the bitmap
    assert_eq!(entry.slot_bitmap, 1u64 << 1);
    assert_eq!(entry.slots.len(), 1);
    assert_eq!(entry.slots[0].slot_id, SlotId::new(1));
    assert_eq!(entry.slots[0].row_count, 2);
}

#[tokio::test]
async fn wal_recovery_clamps_stale_sidecar_offset() {
    // Test that recovery handles a sidecar with wal_position beyond actual WAL data
    let (_dir, wal_path) = temp_wal("stale_sidecar.wal");
    let sidecar_path = wal_path.parent().unwrap().join("quiver.wal.cursor");

    let descriptor = logs_descriptor();

    // First, create a WAL with some data
    {
        let mut writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0x66; 16],
            FlushPolicy::Immediate,
        ))
        .await
        .expect("writer");

        let bundle = single_slot_bundle(&descriptor, 0x01, &[1, 2, 3]);
        let _ = writer.append_bundle(&bundle).await.expect("append");
    }

    // Now write a sidecar with an absurdly large offset
    let stale_sidecar = CursorSidecar::new(u64::MAX / 2);
    CursorSidecar::write_to_sync(&sidecar_path, &stale_sidecar).expect("write stale sidecar");

    // Reopen the writer - it should clamp the offset internally and not panic
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0x66; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer should recover from stale sidecar");

    // Writer should still be functional
    let bundle = single_slot_bundle(&descriptor, 0x02, &[4, 5, 6]);
    let offset = writer
        .append_bundle(&bundle)
        .await
        .expect("append after recovery");
    assert_eq!(offset.sequence, 1, "sequence should continue from WAL scan");

    // Advance the cursor to the END of the WAL (after the second entry)
    // Use the offset returned by append_bundle which is in global coordinates.
    let cursor = WalConsumerCursor {
        safe_offset: offset.next_offset,
        safe_sequence: offset.sequence,
    };
    writer
        .persist_cursor(&cursor)
        .await
        .expect("advance cursor to end of WAL");
    drop(writer);

    // Verify the sidecar now has a valid offset
    let wal_len = std::fs::metadata(&wal_path).expect("metadata").len();
    let recovered_sidecar = CursorSidecar::read_from_sync(&sidecar_path).expect("read sidecar");
    let max_logical = wal_len.saturating_sub(test_header_size());
    assert!(
        recovered_sidecar.wal_position <= max_logical,
        "sidecar offset {} should be within actual WAL data {}",
        recovered_sidecar.wal_position,
        max_logical
    );
}

#[tokio::test]
async fn wal_recovery_handles_rotated_files_with_gaps_in_ids() {
    // Test that recovery handles rotated files with non-contiguous IDs (e.g., wal.1, wal.5, wal.12)
    let (_dir, wal_path) = temp_wal("rotation_gaps.wal");

    let descriptor = logs_descriptor();

    // Manually create rotated files with gaps in their IDs
    // We'll create wal.2 and wal.7 (skipping 1, 3-6)
    for rotation_id in [2u64, 7u64] {
        let rotated_path = rotated_path_for(&wal_path, rotation_id as usize);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&rotated_path)
            .expect("create rotated file");
        WalHeader::new([0x77; 16])
            .write_to_sync(&mut file)
            .expect("write header");
        file.flush().expect("flush");
    }

    // Create the active WAL file
    {
        let mut writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0x77; 16],
            FlushPolicy::Immediate,
        ))
        .await
        .expect("writer");

        let bundle = single_slot_bundle(&descriptor, 0x01, &[1]);
        let _ = writer.append_bundle(&bundle).await.expect("append");
    }

    // Reopen and verify the writer picks up where it left off
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x77; 16], FlushPolicy::Immediate)
            .with_rotation_target(1)
            .with_max_rotated_files(10),
    )
    .await
    .expect("writer with gap-id rotations");

    // Append and trigger a rotation
    let bundle = single_slot_bundle(&descriptor, 0x02, &[2, 3, 4, 5]);
    let _ = writer
        .append_bundle(&bundle)
        .await
        .expect("append triggers rotation");
    drop(writer);

    // The new rotation should use ID 8 (max existing + 1 = 7 + 1)
    let expected_new_rotation = rotated_path_for(&wal_path, 8);
    assert!(
        expected_new_rotation.exists(),
        "new rotation should use ID 8 (after existing max of 7)"
    );

    // Verify wal.2 and wal.7 still exist
    assert!(rotated_path_for(&wal_path, 2).exists());
    assert!(rotated_path_for(&wal_path, 7).exists());
}

#[tokio::test]
#[ignore] // Run manually: cargo test wal_recovery_scan_benchmark --release -- --ignored --nocapture
async fn wal_recovery_scan_benchmark() {
    use std::time::Instant;

    let (_dir, wal_path) = temp_wal("benchmark.wal");
    let descriptor = logs_descriptor();
    let hash = [0xBE; 16];

    // Create entries with realistic payload sizes
    // Typical telemetry batch might be ~1-4 KB
    let payload: Vec<i64> = (0..256).collect(); // ~2KB payload

    // Test 1: Single file (no rotation)
    println!("\n=== WAL Recovery Benchmark ===\n");
    println!("--- Test 1: Single file (64 MB, no rotation) ---");

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), hash, FlushPolicy::Immediate)
            .with_rotation_target(512 * 1024 * 1024) // 512 MB - no rotation
            .with_max_wal_size(512 * 1024 * 1024),
    )
    .await
    .expect("writer");

    let target_bytes = 64 * 1024 * 1024u64;
    let mut total_bytes = 0u64;
    let mut entry_count = 0u64;

    while total_bytes < target_bytes {
        let bundle = single_slot_bundle(&descriptor, 0x01, payload.as_slice());
        let offset = writer.append_bundle(&bundle).await.expect("append");
        total_bytes = offset.position;
        entry_count += 1;
    }
    drop(writer);

    println!(
        "Total data: {} MB, {} entries",
        total_bytes / 1024 / 1024,
        entry_count
    );

    let recovery_start = Instant::now();
    let _writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        hash,
        FlushPolicy::Immediate,
    ))
    .await
    .expect("reopen");
    let recovery_elapsed = recovery_start.elapsed();

    println!("Recovery time: {:?}", recovery_elapsed);
    println!(
        "Scan rate: {:.2} MB/s",
        (total_bytes as f64 / 1024.0 / 1024.0) / recovery_elapsed.as_secs_f64()
    );

    // Test 2: Multiple rotated files
    let (_dir2, wal_path2) = temp_wal("benchmark_rotated.wal");
    println!("\n--- Test 2: With rotation (8 MB per file, ~8 rotated files) ---");

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path2.clone(), hash, FlushPolicy::Immediate)
            .with_rotation_target(8 * 1024 * 1024) // 8 MB per file
            .with_max_rotated_files(16) // Allow enough rotated files
            .with_max_wal_size(512 * 1024 * 1024),
    )
    .await
    .expect("writer");

    entry_count = 0;
    // Write a fixed number of entries to get ~64 MB total
    let entries_for_64mb = 26000u64;
    for _ in 0..entries_for_64mb {
        let bundle = single_slot_bundle(&descriptor, 0x01, payload.as_slice());
        let _ = writer.append_bundle(&bundle).await.expect("append");
        entry_count += 1;
    }
    drop(writer);

    // Count rotated files
    let rotated_count = std::fs::read_dir(wal_path2.parent().unwrap())
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .contains(".wal.")
        })
        .count();
    println!(
        "Total entries: {}, {} rotated files",
        entry_count, rotated_count
    );

    // Calculate total size from all files
    let total_size: u64 = std::fs::read_dir(wal_path2.parent().unwrap())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .unwrap()
                .contains("benchmark_rotated")
        })
        .map(|e| e.metadata().unwrap().len())
        .sum();
    println!(
        "Total data across all files: {} MB",
        total_size / 1024 / 1024
    );

    let recovery_start = Instant::now();
    let _writer = WalWriter::open(
        WalWriterOptions::new(wal_path2.clone(), hash, FlushPolicy::Immediate)
            .with_rotation_target(8 * 1024 * 1024)
            .with_max_rotated_files(16),
    )
    .await
    .expect("reopen");
    let recovery_elapsed = recovery_start.elapsed();

    println!("Recovery time: {:?}", recovery_elapsed);
    println!(
        "Note: Only scans active file (~8 MB), not {} rotated files",
        rotated_count
    );
    println!("\n===============================\n");
}

/// Manual test to observe RSS behavior after a large bundle spike.
///
/// Run with: cargo test wal_memory_after_large_bundle -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn wal_memory_after_large_bundle_spike() {
    fn get_rss_kb() -> Option<u64> {
        // Read RSS from /proc/self/statm (Linux-specific)
        // statm format: size resident shared text lib data dt (all in pages)
        let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
        let resident_pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
        let page_size = 4096u64; // Standard page size on Linux
        Some(resident_pages * page_size / 1024)
    }

    fn print_rss(label: &str) {
        if let Some(rss) = get_rss_kb() {
            println!(
                "  RSS {}: {} KB ({:.1} MB)",
                label,
                rss,
                rss as f64 / 1024.0
            );
        }
    }

    let (_dir, wal_path) = temp_wal("memory_spike.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Data")]);

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0xEE; 16],
        FlushPolicy::Immediate,
    ))
    .await
    .expect("writer");

    println!("=== Memory Spike Test ===");
    println!();

    // Phase 1: Write a few small bundles as baseline
    println!("Phase 1: Writing 100 small bundles (baseline)...");
    for _ in 0..100 {
        let small_slot = FixtureSlot::new(SlotId::new(0), 0x01, &[1, 2, 3]);
        let small_bundle = FixtureBundle::new(descriptor.clone(), vec![small_slot]);
        let _ = writer
            .append_bundle(&small_bundle)
            .await
            .expect("append small");
    }
    print_rss("after baseline");

    // Phase 2: Write a very large bundle (~1000 MB)
    println!("Phase 2: Writing 1 large bundle (~1000 MB)...");
    {
        let large_slot = FixtureSlot::with_batch(
            SlotId::new(0),
            0x02,
            build_complex_batch(1_000_000, "large", 1024), // ~1000 MB
        );
        let large_bundle = FixtureBundle::new(descriptor.clone(), vec![large_slot]);
        let _ = writer
            .append_bundle(&large_bundle)
            .await
            .expect("append large");
        // print_rss("after large bundle (before drop)");
    }
    // large_slot and large_bundle are now dropped
    print_rss("after large bundle dropped");

    // Phase 3: Write many small bundles
    println!(
        "Phase 3: Writing 100 small bundles...should observe RSS shrinking back toward baseline."
    );
    for i in 0..100 {
        let small_slot = FixtureSlot::new(SlotId::new(0), 0x03, &[4, 5, 6]);
        let small_bundle = FixtureBundle::new(descriptor.clone(), vec![small_slot]);
        let _ = writer
            .append_bundle(&small_bundle)
            .await
            .expect("append small");
        drop(small_bundle);
        if (i + 1) % 10 == 0 {
            print_rss(&format!("after {} small bundles", i + 1));
        }
    }
}

#[tokio::test]
async fn wal_buffer_decay_rate_rejects_zero_denominator() {
    let (_dir, wal_path) = temp_wal("decay_rate_zero_denom.wal");
    let options = WalWriterOptions::new(wal_path, [0xAA; 16], FlushPolicy::Immediate)
        .with_buffer_decay_rate(15, 0); // Invalid: denominator is zero

    let result = WalWriter::open(options).await;
    let err = result.expect_err("should reject zero denominator");
    assert!(
        matches!(err, WalError::InvalidConfig(msg) if msg.contains("denominator")),
        "unexpected error: {:?}",
        err
    );
}

#[tokio::test]
async fn wal_buffer_decay_rate_rejects_numerator_gte_denominator() {
    let (_dir, wal_path) = temp_wal("decay_rate_bad_ratio.wal");
    let options = WalWriterOptions::new(wal_path, [0xAA; 16], FlushPolicy::Immediate)
        .with_buffer_decay_rate(16, 16); // Invalid: numerator >= denominator (no decay)

    let result = WalWriter::open(options).await;
    let err = result.expect_err("should reject numerator >= denominator");
    assert!(
        matches!(err, WalError::InvalidConfig(msg) if msg.contains("numerator")),
        "unexpected error: {:?}",
        err
    );
}

#[tokio::test]
async fn wal_buffer_decay_rate_accepts_valid_values() {
    let (_dir, wal_path) = temp_wal("decay_rate_valid.wal");
    // These should all succeed - validation happens at open() time
    let _ = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0xAA; 16], FlushPolicy::Immediate)
            .with_buffer_decay_rate(0, 1), // Aggressive: decay to zero immediately
    )
    .await
    .expect("(0, 1) should be valid");

    let _ = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0xAA; 16], FlushPolicy::Immediate)
            .with_buffer_decay_rate(1, 2), // 50% decay per append
    )
    .await
    .expect("(1, 2) should be valid");

    let _ = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0xAA; 16], FlushPolicy::Immediate)
            .with_buffer_decay_rate(31, 32), // ~3% decay per append (conservative)
    )
    .await
    .expect("(31, 32) should be valid");

    let _ = WalWriter::open(
        WalWriterOptions::new(wal_path, [0xAA; 16], FlushPolicy::Immediate)
            .with_buffer_decay_rate(999, 1000), // ~0.1% decay per append (very conservative)
    )
    .await
    .expect("(999, 1000) should be valid");
}

#[tokio::test]
async fn wal_buffer_decay_rate_affects_shrinking_behavior() {
    // Test that a faster decay rate causes faster shrinking.
    // We use a small threshold and aggressive decay to observe the effect.
    use super::writer::test_support::get_payload_buffer_capacity;

    let (_dir, wal_path) = temp_wal("decay_rate_behavior.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Data")]);

    // Use aggressive decay (1/2 = 50% per append) to see faster shrinking
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path, [0xEE; 16], FlushPolicy::Immediate)
            .with_buffer_decay_rate(1, 2),
    )
    .await
    .expect("writer");

    // Write a moderately large bundle to grow the buffer
    let large_slot = FixtureSlot::with_batch(
        SlotId::new(0),
        0x01,
        build_complex_batch(1000, "medium", 256), // ~256 KB payload
    );
    let bundle = FixtureBundle::new(descriptor.clone(), vec![large_slot]);
    let _ = writer.append_bundle(&bundle).await.expect("append");
    drop(bundle);

    let capacity_after_large = get_payload_buffer_capacity(&writer);
    assert!(
        capacity_after_large >= 100 * 1024,
        "buffer should have grown: {}",
        capacity_after_large
    );

    // Write small bundles; with 50% decay the high-water mark drops fast
    // After ~10 appends: high_water ≈ initial * (1/2)^10 ≈ 0.1% of initial
    for _ in 0..20 {
        let small_slot = FixtureSlot::new(SlotId::new(0), 0x02, &[1, 2, 3]);
        let small_bundle = FixtureBundle::new(descriptor.clone(), vec![small_slot]);
        let _ = writer.append_bundle(&small_bundle).await.expect("append");
    }

    let capacity_after_small = get_payload_buffer_capacity(&writer);
    // With aggressive decay, buffer should have shrunk significantly
    // (exact threshold depends on SHRINK_THRESHOLD constant, but it should be smaller)
    assert!(
        capacity_after_small < capacity_after_large,
        "buffer should have shrunk: before={}, after={}",
        capacity_after_large,
        capacity_after_small
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// WAL Position Coordinate System Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Verifies that WAL positions remain stable across WAL file rotations.
/// After a rotation, new entries should receive positions that continue from
/// the previous entry's `next_offset`, not restart at the active file position.
#[tokio::test]
async fn wal_positions_stable_across_rotation() {
    let (_dir, wal_path) = temp_wal("wal_positions_rotation.wal");
    let descriptor = logs_descriptor();

    // Configure for immediate rotation after each entry
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x77; 16], FlushPolicy::Immediate)
            .with_rotation_target(1)
            .with_max_rotated_files(10),
    )
    .await
    .expect("writer");

    // Append three entries, each triggering a rotation
    let bundle1 = single_slot_bundle(&descriptor, 0x01, &[1, 2, 3, 4]);
    let offset1 = writer.append_bundle(&bundle1).await.expect("first append");

    let bundle2 = single_slot_bundle(&descriptor, 0x02, &[5, 6, 7, 8]);
    let offset2 = writer.append_bundle(&bundle2).await.expect("second append");

    let bundle3 = single_slot_bundle(&descriptor, 0x03, &[9, 10, 11, 12]);
    let offset3 = writer.append_bundle(&bundle3).await.expect("third append");

    // Verify WAL positions are monotonically increasing and contiguous
    assert_eq!(
        offset1.next_offset, offset2.position,
        "second entry should start where first ended"
    );
    assert_eq!(
        offset2.next_offset, offset3.position,
        "third entry should start where second ended"
    );

    // Verify rotations happened
    assert_eq!(writer.rotation_count(), 3, "expected 3 rotations");
    assert!(rotated_path_for(&wal_path, 1).exists(), "first rotation");
    assert!(rotated_path_for(&wal_path, 2).exists(), "second rotation");
    assert!(rotated_path_for(&wal_path, 3).exists(), "third rotation");
}

/// Verifies that WAL positions remain stable after rotated files are purged.
/// The `wal_position_start` field in the WAL header preserves the coordinate system.
#[tokio::test]
async fn wal_positions_stable_after_purge() {
    let (_dir, wal_path) = temp_wal("wal_positions_purge.wal");
    let descriptor = logs_descriptor();

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x88; 16], FlushPolicy::Immediate)
            .with_rotation_target(1)
            .with_max_rotated_files(10),
    )
    .await
    .expect("writer");

    // Append several entries to trigger rotations
    let bundle1 = single_slot_bundle(&descriptor, 0x01, &[1, 2, 3, 4]);
    let offset1 = writer.append_bundle(&bundle1).await.expect("first append");

    let bundle2 = single_slot_bundle(&descriptor, 0x02, &[5, 6, 7, 8]);
    let offset2 = writer.append_bundle(&bundle2).await.expect("second append");

    let bundle3 = single_slot_bundle(&descriptor, 0x03, &[9, 10, 11, 12]);
    let offset3 = writer.append_bundle(&bundle3).await.expect("third append");

    // Verify rotations occurred
    assert_eq!(writer.rotation_count(), 3, "three rotations");

    // Persist cursor - all rotated files should be purged
    let cursor = WalConsumerCursor {
        safe_offset: offset3.next_offset,
        safe_sequence: offset3.sequence,
    };
    writer
        .persist_cursor(&cursor)
        .await
        .expect("persist cursor");

    // After persisting cursor for all data, all rotated files should be purged
    let purge_count = writer.purge_count();
    assert_eq!(purge_count, 3, "all three files should be purged");

    // Append another entry after all purges - its offset should continue correctly
    let bundle4 = single_slot_bundle(&descriptor, 0x04, &[13, 14, 15, 16]);
    let offset4 = writer
        .append_bundle(&bundle4)
        .await
        .expect("fourth append after purge");

    // The fourth entry should start where the third ended
    assert_eq!(
        offset3.next_offset, offset4.position,
        "WAL position should continue after purge: expected {}, got {}",
        offset3.next_offset, offset4.position
    );

    // Verify offsets are strictly increasing throughout the sequence
    assert!(offset1.position < offset2.position, "offset1 < offset2");
    assert!(offset2.position < offset3.position, "offset2 < offset3");
    assert!(offset3.position < offset4.position, "offset3 < offset4");
}

/// Verifies that rotation and purge counters are tracked correctly.
#[tokio::test]
async fn rotation_and_purge_counters() {
    let (_dir, wal_path) = temp_wal("rotation_purge_counters.wal");
    let descriptor = logs_descriptor();

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x99; 16], FlushPolicy::Immediate)
            .with_rotation_target(1)
            .with_max_rotated_files(10),
    )
    .await
    .expect("writer");

    assert_eq!(writer.rotation_count(), 0, "no rotations yet");
    assert_eq!(writer.purge_count(), 0, "no purges yet");

    // Append entries to trigger rotations
    let mut last_offset = None;
    for i in 0..5i64 {
        let bundle = single_slot_bundle(&descriptor, i as u8, &[i, i + 1, i + 2]);
        last_offset = Some(writer.append_bundle(&bundle).await.expect("append"));
    }

    assert_eq!(writer.rotation_count(), 5, "5 rotations after 5 appends");
    assert_eq!(writer.purge_count(), 0, "no purges without cursor");

    // Persist cursor to purge all rotated files
    let final_offset = last_offset.unwrap();
    let cursor = WalConsumerCursor {
        safe_offset: final_offset.next_offset,
        safe_sequence: final_offset.sequence,
    };
    writer
        .persist_cursor(&cursor)
        .await
        .expect("persist cursor");

    assert_eq!(writer.purge_count(), 5, "all 5 rotated files purged");
    assert_eq!(writer.rotation_count(), 5, "rotation count unchanged");
}

/// Verifies that recovery after restart with purged files correctly infers
/// the purged cumulative offset from the cursor.
#[tokio::test]
async fn recovery_infers_purged_offset_from_cursor() {
    let (_dir, wal_path) = temp_wal("recovery_purged_offset.wal");
    let descriptor = logs_descriptor();

    let options = WalWriterOptions::new(wal_path.clone(), [0xAA; 16], FlushPolicy::Immediate)
        .with_rotation_target(1)
        .with_max_rotated_files(10);

    // Phase 1: Write entries, persist cursor, and purge
    let final_offset;
    {
        let mut writer = WalWriter::open(options.clone())
            .await
            .expect("first writer");

        // Append entries
        for i in 0..3i64 {
            let bundle = single_slot_bundle(&descriptor, i as u8, &[i * 10, i * 10 + 1]);
            let _ = writer.append_bundle(&bundle).await.expect("append");
        }

        // Persist cursor
        let bundle = single_slot_bundle(&descriptor, 0x99, &[99]);
        final_offset = writer.append_bundle(&bundle).await.expect("final append");

        let cursor = WalConsumerCursor {
            safe_offset: final_offset.next_offset,
            safe_sequence: final_offset.sequence,
        };
        writer
            .persist_cursor(&cursor)
            .await
            .expect("persist cursor");

        // All rotated files should be purged
        assert!(!rotated_path_for(&wal_path, 1).exists());
        assert!(!rotated_path_for(&wal_path, 2).exists());
        assert!(!rotated_path_for(&wal_path, 3).exists());
    }

    // Phase 2: Restart and append new entries
    {
        let mut writer = WalWriter::open(options).await.expect("reopen writer");

        // Append a new entry - it should get a correct WAL position
        let bundle = single_slot_bundle(&descriptor, 0xBB, &[0xBB]);
        let new_offset = writer
            .append_bundle(&bundle)
            .await
            .expect("append after restart");

        // The new entry should continue from where the last one ended,
        // even though the rotated files were purged before restart
        assert_eq!(
            new_offset.position, final_offset.next_offset,
            "offset should continue from cursor: expected {}, got {}",
            final_offset.next_offset, new_offset.position
        );
    }
}

/// Verifies that multiple rotation-cursor-purge cycles maintain
/// correct WAL positions throughout.
#[tokio::test]
async fn multiple_purge_cycles_maintain_wal_positions() {
    let (_dir, wal_path) = temp_wal("multiple_purge_cycles.wal");
    let descriptor = logs_descriptor();

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0xBB; 16], FlushPolicy::Immediate)
            .with_rotation_target(1)
            .with_max_rotated_files(10),
    )
    .await
    .expect("writer");

    let mut expected_next_position = 0u64;

    // Run multiple cycles of: append -> persist cursor -> purge
    for cycle in 0..5i64 {
        let bundle = single_slot_bundle(&descriptor, cycle as u8, &[cycle, cycle + 1]);
        let offset = writer.append_bundle(&bundle).await.expect("append");

        if expected_next_position > 0 {
            assert_eq!(
                offset.position, expected_next_position,
                "cycle {}: offset should continue from previous: expected {}, got {}",
                cycle, expected_next_position, offset.position
            );
        }
        expected_next_position = offset.next_offset;

        // Persist cursor and purge
        let cursor = WalConsumerCursor {
            safe_offset: offset.next_offset,
            safe_sequence: offset.sequence,
        };
        writer
            .persist_cursor(&cursor)
            .await
            .expect("persist cursor");
    }

    assert_eq!(writer.rotation_count(), 5, "5 rotations");
    assert_eq!(writer.purge_count(), 5, "5 purges");
}

/// Verifies that persisting cursor works correctly when the cursor position
/// is within a rotated file, not the active file.
///
/// This tests a scenario where:
/// 1. Multiple entries are appended, triggering rotations
/// 2. A cursor is set to a position within an older rotated file
/// 3. The cursor should be accepted and rotated files up to that point purged
#[tokio::test]
async fn cursor_in_rotated_file_is_valid() {
    let (_dir, wal_path) = temp_wal("cursor_in_rotated.wal");
    let descriptor = logs_descriptor();

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0xCC; 16], FlushPolicy::Immediate)
            .with_rotation_target(1) // Rotate after each entry
            .with_max_rotated_files(10),
    )
    .await
    .expect("writer");

    // Append three entries, each triggering a rotation
    let bundle1 = single_slot_bundle(&descriptor, 0x01, &[1, 2, 3, 4]);
    let offset1 = writer.append_bundle(&bundle1).await.expect("first append");

    let bundle2 = single_slot_bundle(&descriptor, 0x02, &[5, 6, 7, 8]);
    let offset2 = writer.append_bundle(&bundle2).await.expect("second append");

    let bundle3 = single_slot_bundle(&descriptor, 0x03, &[9, 10, 11, 12]);
    let _offset3 = writer.append_bundle(&bundle3).await.expect("third append");

    // Verify rotations occurred
    assert_eq!(writer.rotation_count(), 3, "three rotations");
    assert_eq!(writer.purge_count(), 0, "no purges yet");

    // Persist cursor to the END of the FIRST entry (which is in a rotated file)
    // This should be valid - the cursor is at an entry boundary in a rotated file
    let cursor = WalConsumerCursor {
        safe_offset: offset1.next_offset,
        safe_sequence: offset1.sequence,
    };
    writer
        .persist_cursor(&cursor)
        .await
        .expect("cursor in rotated file should succeed");

    // The first rotated file should be purged (it's fully consumed)
    assert_eq!(
        writer.purge_count(),
        1,
        "first rotated file should be purged"
    );

    // Persist cursor to the end of the second entry (also in a rotated file)
    let cursor2 = WalConsumerCursor {
        safe_offset: offset2.next_offset,
        safe_sequence: offset2.sequence,
    };
    writer
        .persist_cursor(&cursor2)
        .await
        .expect("cursor in second rotated file should succeed");

    // Now two rotated files should be purged
    assert_eq!(
        writer.purge_count(),
        2,
        "two rotated files should be purged"
    );
}

#[tokio::test]
async fn wal_error_is_at_capacity_returns_true_for_capacity_errors() {
    let capacity_error = WalError::WalAtCapacity("test capacity message");
    assert!(
        capacity_error.is_at_capacity(),
        "WalAtCapacity should return true for is_at_capacity()"
    );
}

#[tokio::test]
async fn wal_error_is_at_capacity_returns_false_for_other_errors() {
    let io_error = WalError::Io(std::io::Error::other("test"));
    assert!(
        !io_error.is_at_capacity(),
        "Io error should return false for is_at_capacity()"
    );

    let invalid_config = WalError::InvalidConfig("test config");
    assert!(
        !invalid_config.is_at_capacity(),
        "InvalidConfig should return false for is_at_capacity()"
    );

    let invalid_entry = WalError::InvalidEntry("test entry");
    assert!(
        !invalid_entry.is_at_capacity(),
        "InvalidEntry should return false for is_at_capacity()"
    );
}
