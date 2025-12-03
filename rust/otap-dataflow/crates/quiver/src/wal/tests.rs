// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Cross-cutting WAL tests live here so shared fixtures can touch writer, reader,
//! and helper plumbing without sprinkling large #[cfg(test)] blocks in each file.

 use std::cmp;
 use std::io::{Cursor, Read, Seek, SeekFrom, Write};
 use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use arrow_array::{builder::StringBuilder, Int64Array, RecordBatch};
use arrow_ipc::reader::StreamReader;
use arrow_schema::{DataType, Field, Schema};
use crc32fast::Hasher;
use tempfile::tempdir;

use crate::record_bundle::{
    BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
};

use super::header::{WAL_HEADER_LEN, WalHeader};
use super::reader::test_support::{self, ReadFailure};
use super::writer::test_support as writer_test_support;
use super::{
    ENTRY_HEADER_LEN, ENTRY_TYPE_RECORD_BUNDLE, SCHEMA_FINGERPRINT_LEN, WalError, WalReader,
    WalTruncateCursor, WalWriter, WalWriterOptions,
};
use super::truncate_sidecar::{TruncateSidecar, TRUNCATE_SIDECAR_LEN};

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
        builder.append_value(&format!("{prefix}-{idx:05}-{}", chunk));
    }

    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(Int64Array::from(values)), Arc::new(builder.finish())],
    )
    .expect("complex batch");
    batch
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
    header.write_to(&mut file).expect("write header");
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

fn measure_bundle_data_bytes(mut build_bundle: impl FnMut() -> FixtureBundle) -> u64 {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("measure_bundle.wal");
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0xFE; 16],
        Duration::ZERO,
    ))
    .expect("writer");
    let bundle = build_bundle();
    let _ = writer.append_bundle(&bundle).expect("append bundle");
    drop(writer);
    std::fs::metadata(&wal_path)
        .expect("metadata")
        .len()
        .saturating_sub(WAL_HEADER_LEN as u64)
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

#[test]
fn wal_writer_reader_roundtrip_recovers_payloads() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("roundtrip.wal");
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

    let options = WalWriterOptions::new(wal_path.clone(), hash, Duration::ZERO);
    let mut writer = WalWriter::open(options).expect("writer");
    let offset = writer.append_bundle(&bundle).expect("append succeeds");
    assert_eq!(offset.position, WAL_HEADER_LEN as u64);
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
    assert_eq!(record.offset.position, WAL_HEADER_LEN as u64);
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

#[test]
fn wal_writer_rejects_slot_ids_outside_bitmap() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("slot_range.wal");
    let mut writer =
        WalWriter::open(WalWriterOptions::new(wal_path, [0; 16], Duration::ZERO)).expect("writer");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(65, "Overflow")]);
    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(65), 0xAA, &[1])],
    );

    let err = writer.append_bundle(&bundle).expect_err("slot validation");
    assert!(matches!(err, WalError::SlotOutOfRange(slot) if slot == SlotId::new(65)));
}

#[test]
fn wal_writer_rejects_pre_epoch_timestamp() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("pre_epoch.wal");
    let mut writer =
        WalWriter::open(WalWriterOptions::new(wal_path, [0; 16], Duration::ZERO)).expect("writer");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let bundle = FixtureBundle::new(descriptor, vec![])
        .with_ingestion_time(UNIX_EPOCH - Duration::from_secs(1));

    let err = writer
        .append_bundle(&bundle)
        .expect_err("timestamp validation");
    assert!(matches!(err, WalError::InvalidTimestamp));
}

#[test]
fn wal_writer_rejects_truncated_existing_file() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("truncated.wal");
    {
        let mut file = std::fs::File::create(&wal_path).expect("create file");
        file.write_all(&vec![0u8; WAL_HEADER_LEN - 1])
            .expect("truncate header");
    }

    let options = WalWriterOptions::new(wal_path, [0; 16], Duration::ZERO);
    let err = WalWriter::open(options).expect_err("should reject truncated file");
    assert!(matches!(err, WalError::InvalidHeader("file smaller than header")));
}

#[test]
fn wal_writer_reopens_with_matching_header() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("existing.wal");
    let original_hash = [0xAA; 16];
    {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&wal_path)
            .expect("create file");
        WalHeader::new(original_hash)
            .write_to(&mut file)
            .expect("write header");
        file.flush().expect("flush");
    }

    // Reopen with the same hash—should succeed and preserve the header.
    let options = WalWriterOptions::new(wal_path.clone(), original_hash, Duration::ZERO);
    let _writer = WalWriter::open(options).expect("open succeeds");
    drop(_writer);

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(&wal_path)
        .expect("open for read");
    let header = WalHeader::read_from(&mut file).expect("read header");
    assert_eq!(header.segment_cfg_hash, original_hash);
}

#[test]
fn wal_writer_flushes_after_interval_elapsed() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("flush.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        Duration::from_millis(10),
    ))
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x42, &[1])],
    );

    let before = writer.test_last_flush();
    writer.test_set_last_flush(Instant::now() - Duration::from_secs(1));
    let _offset = writer
        .append_bundle(&bundle)
        .expect("append triggers flush");
    assert!(writer.test_last_flush() > before);
}

#[test]
fn wal_writer_flush_syncs_file_data() {
    writer_test_support::reset_flush_notifications();

    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("flush_sync.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path,
        [0; 16],
        Duration::ZERO,
    ))
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0xAA, &[7])],
    );

    assert!(!writer_test_support::take_sync_data_notification());
    let _offset = writer.append_bundle(&bundle).expect("append flush");
    assert!(writer_test_support::take_sync_data_notification());
}

#[test]
fn wal_writer_rewrite_compacts_prefix() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("rewrite_compact.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x20; 16], Duration::ZERO)
            .with_punch_capability(false),
    )
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1, 2, 3])],
    );
    let _ = writer.append_bundle(&first_bundle).expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[4, 5, 6])],
    );
    let _ = writer.append_bundle(&second_bundle).expect("second append");

    let len_before = std::fs::metadata(&wal_path).expect("metadata").len();

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let first_entry = iter.next().expect("entry").expect("ok");

    let mut cursor = WalTruncateCursor::default();
    cursor.safe_offset = first_entry.next_offset;
    writer.reclaim_prefix(&cursor).expect("reclaim prefix");
    drop(writer);

    let len_after = std::fs::metadata(&wal_path).expect("metadata").len();
    assert!(len_after < len_before);

    let sidecar_path = wal_path.parent().unwrap().join("truncate.offset");
    let sidecar = TruncateSidecar::read_from(&sidecar_path).expect("sidecar");
    assert_eq!(sidecar.truncate_offset, WAL_HEADER_LEN as u64);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let remaining = iter.next().expect("entry").expect("ok");
    assert_eq!(remaining.sequence, first_entry.sequence + 1);
    assert!(iter.next().is_none());
}

#[test]
fn wal_writer_enforces_safe_offset_boundaries() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("safe_offset.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x42; 16], Duration::ZERO)
            .with_punch_capability(false),
    )
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[11, 12, 13])],
    );
    let _ = writer.append_bundle(&first_bundle).expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[21, 22, 23])],
    );
    let _ = writer.append_bundle(&second_bundle).expect("second append");

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iter");
    let first_entry = iter.next().expect("entry").expect("ok");

    let mut cursor = WalTruncateCursor::default();
    cursor.safe_offset = first_entry.offset.position + 4;
    cursor.safe_sequence = first_entry.sequence;

    match writer.reclaim_prefix(&cursor) {
        Err(WalError::InvalidTruncateCursor(message)) => {
            assert_eq!(message, "safe offset splits entry boundary")
        }
        other => panic!("expected invalid cursor error, got {other:?}"),
    }

    cursor.advance(&first_entry);
    writer
        .reclaim_prefix(&cursor)
        .expect("reclaim succeeds with aligned cursor");
    drop(writer);

    let mut reader = WalReader::open(&wal_path).expect("reader after reclaim");
    let mut iter = reader.iter_from(0).expect("iter");
    let remaining = iter.next().expect("entry").expect("ok");
    assert_eq!(remaining.sequence, first_entry.sequence + 1);
    assert!(iter.next().is_none());
}

#[test]
fn wal_writer_punch_failure_falls_back_to_rewrite() {
    writer_test_support::reset_flush_notifications();
    writer_test_support::set_force_punch_error(true);

    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("punch_fallback.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x30; 16], Duration::ZERO)
            .with_punch_capability(true),
    )
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[10, 11])],
    );
    let _ = writer.append_bundle(&bundle).expect("append");

    let mut cursor = WalTruncateCursor::default();
    cursor.safe_offset = std::fs::metadata(&wal_path)
        .expect("metadata")
        .len();

    writer.reclaim_prefix(&cursor).expect("reclaim prefix");
    assert!(writer_test_support::take_punch_failure_notification());
    writer_test_support::set_force_punch_error(false);
}

#[test]
fn wal_writer_persists_truncate_cursor_sidecar() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("truncate_sidecar.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0x99; 16],
        Duration::ZERO,
    ))
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1, 2])],
    );
    let _ = writer.append_bundle(&bundle).expect("append");

    let file_len = std::fs::metadata(&wal_path).expect("metadata").len();
    let mut cursor = WalTruncateCursor::default();
    cursor.safe_offset = file_len;
    writer
        .record_truncate_cursor(&cursor)
        .expect("record cursor");
    drop(writer);

    let sidecar_path = wal_path
        .parent()
        .expect("dir")
        .join("truncate.offset");
    let state = TruncateSidecar::read_from(&sidecar_path).expect("sidecar");
    assert_eq!(state.truncate_offset, file_len);
}

#[test]
fn wal_writer_rotates_when_target_exceeded() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("force_rotate.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x51; 16], Duration::ZERO)
            .with_rotation_target(1)
            .with_max_chunks(4),
    )
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1, 2, 3, 4])],
    );
    let _ = writer.append_bundle(&bundle).expect("append triggers rotation");
    drop(writer);

    let rotated_path = rotated_path_for(&wal_path, 1);
    assert!(rotated_path.exists(), "rotated chunk missing at {:?}", rotated_path);
    let rotated_len = std::fs::metadata(&rotated_path)
        .expect("rotated metadata")
        .len();
    assert!(rotated_len > WAL_HEADER_LEN as u64);

    let active_len = std::fs::metadata(&wal_path)
        .expect("active metadata")
        .len();
    assert_eq!(active_len, WAL_HEADER_LEN as u64);

    let sidecar_path = wal_path.parent().unwrap().join("truncate.offset");
    let sidecar = TruncateSidecar::read_from(&sidecar_path).expect("sidecar");
    assert_eq!(sidecar.rotation_generation, 1);
}

#[test]
fn wal_writer_errors_when_chunk_cap_reached() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("chunk_cap.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x52; 16], Duration::ZERO)
            .with_rotation_target(1)
            .with_max_chunks(1),
    )
    .expect("writer");

    let payload = [10, 11, 12];
    let first_bundle = single_slot_bundle(&descriptor, 0x02, &payload);
    let _ = writer
        .append_bundle(&first_bundle)
        .expect("first append rotates");
    assert!(
        rotated_path_for(&wal_path, 1).exists(),
        "expected rotated chunk to exist",
    );

    let err = writer
        .append_bundle(&single_slot_bundle(&descriptor, 0x03, &payload))
        .expect_err("second rotation should hit chunk cap");
    match err {
        WalError::WalAtCapacity(message) => {
            assert!(
                message.contains("chunk cap"),
                "unexpected error message: {message}",
            );
        }
        other => panic!("expected WalAtCapacity, got {other:?}"),
    }
}

#[test]
fn wal_writer_enforces_size_cap_and_purges_rotations() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("size_cap.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let payload: Vec<i64> = (0..64).collect();
    let entry_bytes = measure_bundle_data_bytes(|| {
        single_slot_bundle(&descriptor, 0x07, payload.as_slice())
    });
    let header_len = WAL_HEADER_LEN as u64;
    let chunk_file_len = header_len + entry_bytes;
    let slack = cmp::max(1, entry_bytes / 2);
    let max_wal_size = chunk_file_len + header_len + slack;

    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path.clone(), [0x53; 16], Duration::ZERO)
            .with_rotation_target(1)
            .with_max_chunks(4)
            .with_max_wal_size(max_wal_size),
    )
    .expect("writer");

    let first_bundle = single_slot_bundle(&descriptor, 0x07, payload.as_slice());
    let _ = writer
        .append_bundle(&first_bundle)
        .expect("first append rotates under cap");
    assert!(rotated_path_for(&wal_path, 1).exists());

    let second_bundle = single_slot_bundle(&descriptor, 0x08, payload.as_slice());
    let err = writer
        .append_bundle(&second_bundle)
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

    let mut cursor = WalTruncateCursor::default();
    cursor.safe_offset = WAL_HEADER_LEN as u64;
    writer
        .record_truncate_cursor(&cursor)
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
        .expect("append succeeds once space is reclaimed");
}

#[test]
fn wal_writer_ignores_invalid_truncate_sidecar() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("bad_sidecar.wal");

    // Create the WAL header so the file exists.
    {
        let _writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0x11; 16],
            Duration::ZERO,
        ))
        .expect("writer");
    }

    let sidecar_path = wal_path
        .parent()
        .expect("dir")
        .join("truncate.offset");
    std::fs::write(&sidecar_path, vec![0u8; TRUNCATE_SIDECAR_LEN - 4]).expect("write corrupt");

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0x11; 16],
        Duration::ZERO,
    ))
    .expect("reopen");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[7])],
    );
    let _ = writer.append_bundle(&bundle).expect("append");
    let file_len = std::fs::metadata(&wal_path).expect("metadata").len();

    let mut cursor = WalTruncateCursor::default();
    cursor.safe_offset = file_len;
    writer
        .record_truncate_cursor(&cursor)
        .expect("record cursor");
    drop(writer);

    let state = TruncateSidecar::read_from(&sidecar_path).expect("sidecar");
    assert_eq!(state.truncate_offset, file_len);
}

#[test]
fn wal_writer_flushes_after_unflushed_byte_threshold() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("flush_bytes.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut writer = WalWriter::open(
        WalWriterOptions::new(wal_path, [0; 16], Duration::from_secs(60))
            .with_max_unflushed_bytes(1),
    )
    .expect("writer");

    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x99, &[1, 2, 3])],
    );

    writer.test_set_last_flush(Instant::now());
    let before = writer.test_last_flush();
    let _offset = writer.append_bundle(&bundle).expect("append triggers flush");
    assert!(writer.test_last_flush() > before);
}

#[test]
fn wal_writer_flushes_pending_bytes_on_drop() {
    writer_test_support::reset_flush_notifications();

    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("flush_drop.wal");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let writer = WalWriter::open(
        WalWriterOptions::new(wal_path, [0; 16], Duration::from_secs(60))
            .with_max_unflushed_bytes(0),
    )
    .expect("writer");

    {
        let mut writer = writer;
        let bundle = FixtureBundle::new(
            descriptor,
            vec![FixtureSlot::new(SlotId::new(0), 0x55, &[42])],
        );
        let _ = writer.append_bundle(&bundle).expect("append");
        assert!(!writer_test_support::take_drop_flush_notification());
    }

    assert!(writer_test_support::take_drop_flush_notification());
}

#[test]
fn wal_writer_rejects_truncate_beyond_file_end() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("truncate_oob.wal");
    let hash = [0xAB; 16];

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        hash,
        Duration::ZERO,
    ))
    .expect("writer");

    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );
    let _ = writer.append_bundle(&bundle).expect("append entry");

    let file_len = std::fs::metadata(&wal_path)
        .expect("metadata")
        .len()
        .saturating_add(8);
    let cursor = WalTruncateCursor {
        safe_offset: file_len,
        safe_sequence: 0,
    };

    match writer.truncate_to(&cursor) {
        Err(WalError::InvalidTruncateCursor("safe offset beyond wal tail")) => {}
        other => panic!("expected truncate bounds error, got {:?}", other),
    }
}

#[test]
fn wal_reader_rewind_allows_replay_from_start() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("rewind.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0x10; 16],
        Duration::ZERO,
    ))
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );
    let _ = writer.append_bundle(&first_bundle).expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[2])],
    );
    let _ = writer.append_bundle(&second_bundle).expect("second append");
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

#[test]
fn wal_reader_iterator_stays_finished_after_eof() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("empty.wal");
    {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&wal_path)
            .expect("create wal");
        WalHeader::new([0x44; 16])
            .write_to(&mut file)
            .expect("header");
    }

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    assert!(iter.next().is_none(), "no entries present");
    assert!(iter.next().is_none(), "iterator remains finished");
}

#[test]
fn wal_reader_errors_on_truncated_entry_length() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("length_trunc.wal");
    {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&wal_path)
            .expect("create wal");
        WalHeader::new([0x55; 16])
            .write_to(&mut file)
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

#[test]
fn wal_reader_reports_crc_mismatch() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("crc.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x55, &[7, 8])],
    );

    let options = WalWriterOptions::new(wal_path.clone(), [1; 16], Duration::ZERO);
    let mut writer = WalWriter::open(options).expect("writer");
    let _offset = writer.append_bundle(&bundle).expect("append");
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

#[test]
fn wal_reader_rejects_unsupported_entry_type() {
    let body = encode_entry_header(0xAA, 0);
    let (_dir, wal_path) = write_single_entry(&body);
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");

    match iter.next() {
        Some(Err(WalError::UnsupportedEntry(ty))) => assert_eq!(ty, 0xAA),
        other => panic!("expected unsupported entry, got {:?}", other),
    }
}

#[test]
fn wal_reader_errors_on_truncated_slot_header() {
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

#[test]
fn wal_reader_errors_on_truncated_slot_payload() {
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

#[test]
fn wal_reader_errors_on_entry_header_underflow() {
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

#[test]
fn wal_reader_errors_on_unexpected_trailing_bytes() {
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

#[test]
fn wal_reader_errors_on_truncated_entry_body() {
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);
    truncate_file_from_end(&wal_path, 6);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::UnexpectedEof("entry body"))) => {}
        other => panic!("expected entry body EOF, got {:?}", other),
    }
}

#[test]
fn wal_reader_errors_on_truncated_entry_crc() {
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

#[test]
fn wal_reader_reports_io_error_during_entry_length_read() {
    let _guard = FailureGuard::new();
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);

    test_support::fail_next_read(ReadFailure::EntryLength);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::Io(_))) => {}
        other => panic!("expected io error for entry length read, got {:?}", other),
    }
}

#[test]
fn wal_reader_reports_io_error_during_entry_body_read() {
    let _guard = FailureGuard::new();
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);

    test_support::fail_next_read(ReadFailure::EntryBody);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::Io(_))) => {}
        other => panic!("expected io error for entry body read, got {:?}", other),
    }
}

#[test]
fn wal_reader_reports_io_error_during_entry_crc_read() {
    let _guard = FailureGuard::new();
    let body = encode_entry_header(ENTRY_TYPE_RECORD_BUNDLE, 0);
    let (_dir, wal_path) = write_single_entry(&body);

    test_support::fail_next_read(ReadFailure::EntryCrc);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    match iter.next() {
        Some(Err(WalError::Io(_))) => {}
        other => panic!("expected io error for entry crc read, got {:?}", other),
    }
}

#[test]
fn wal_reader_iter_from_respects_offsets() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("offsets.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);

    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        [0xCC; 16],
        Duration::ZERO,
    ))
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );
    let first_offset = writer.append_bundle(&first_bundle).expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[2])],
    );
    let second_offset = writer.append_bundle(&second_bundle).expect("second append");
    drop(writer);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    let entry_one = iter.next().expect("first entry").expect("ok");
    let entry_two = iter.next().expect("second entry").expect("ok");
    assert_eq!(entry_one.sequence, 0);
    assert_eq!(entry_two.sequence, 1);
    assert_eq!(entry_one.next_offset, entry_two.offset.position);

    let mut cursor = WalTruncateCursor::default();
    cursor.advance(&entry_one);
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

#[test]
fn wal_reader_iter_from_partial_length_reports_error() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("partial_offset.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);

    {
        let mut writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0x44; 16],
            Duration::ZERO,
        ))
        .expect("writer");

        let bundle = FixtureBundle::new(
            descriptor.clone(),
            vec![FixtureSlot::new(SlotId::new(0), 0xAA, &[1, 2])],
        );
        let _ = writer.append_bundle(&bundle).expect("append");
    }

    let metadata_len = std::fs::metadata(&wal_path)
        .expect("metadata")
        .len();
    let misaligned_offset = metadata_len.saturating_sub(2);

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader
        .iter_from(misaligned_offset)
        .expect("iterator from misaligned offset");
    match iter.next() {
        Some(Err(WalError::UnexpectedEof("entry length"))) => {}
        other => panic!("expected entry length eof, got {:?}", other),
    }
}

#[test]
fn wal_reader_iter_from_offset_past_file_returns_none() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("past_end_offset.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);

    {
        let mut writer = WalWriter::open(WalWriterOptions::new(
            wal_path.clone(),
            [0x55; 16],
            Duration::ZERO,
        ))
        .expect("writer");

        let bundle = FixtureBundle::new(
            descriptor,
            vec![FixtureSlot::new(SlotId::new(0), 0xCC, &[3, 4])],
        );
        let _ = writer.append_bundle(&bundle).expect("append");
    }

    let metadata_len = std::fs::metadata(&wal_path)
        .expect("metadata")
        .len();
    let offset_beyond_file = metadata_len + 128;

    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader
        .iter_from(offset_beyond_file)
        .expect("iterator past end");
    assert!(iter.next().is_none());
}

#[test]
fn wal_writer_reader_handles_all_bitmap_slots() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("all_slots.wal");
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
        Duration::ZERO,
    ))
    .expect("writer");
    let _ = writer.append_bundle(&bundle).expect("append");
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

#[test]
fn wal_writer_handles_large_payload_batches() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("large_payload.wal");
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
        Duration::ZERO,
    ))
    .expect("writer");

    let _ = writer.append_bundle(&bundle).expect("append large payload");
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

#[test]
fn wal_truncate_cursor_recovers_after_partial_entry() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("recovery.wal");
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let hash = [0x99; 16];

    // Start with a WAL containing two valid entries so the cursor has
    // something to mark as safe.
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        hash,
        Duration::ZERO,
    ))
    .expect("writer");

    let first_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x01, &[1])],
    );
    let _ = writer.append_bundle(&first_bundle).expect("first append");

    let second_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0x02, &[2])],
    );
    let _ = writer.append_bundle(&second_bundle).expect("second append");
    drop(writer);

    // Replay the WAL and advance the truncate cursor to the end of the first
    // entry—everything before this offset is known to be durable.
    let mut reader = WalReader::open(&wal_path).expect("reader");
    let mut iter = reader.iter_from(0).expect("iterator");
    let first_entry = iter.next().expect("first entry").expect("ok");
    let mut cursor = WalTruncateCursor::default();
    cursor.advance(&first_entry);
    drop(reader);

    // Simulate a crash that truncates the file in the middle of the second
    // entry body, making it unreadable.
    {
        let metadata = std::fs::metadata(&wal_path).expect("metadata");
        assert!(metadata.len() > cursor.safe_offset + 1);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&wal_path)
            .expect("open for corruption");
        file.set_len(cursor.safe_offset + 1)
            .expect("truncate inside entry");
    }

    // Reader now sees the first entry but errors on the partial second entry.
    let mut reader = WalReader::open(&wal_path).expect("reader after corruption");
    let mut iter = reader.iter_from(0).expect("iterator");
    let _ = iter.next().expect("first entry remains").expect("ok");
    match iter.next() {
        Some(Err(WalError::UnexpectedEof("entry length"))) => {}
        other => panic!("expected entry length eof, got {:?}", other),
    }
    drop(reader);

    // Truncate the file back to the cursor's safe offset so future appends
    // resume from a clean boundary.
    let mut writer = WalWriter::open(WalWriterOptions::new(
        wal_path.clone(),
        hash,
        Duration::ZERO,
    ))
    .expect("writer reopens");
    writer
        .truncate_to(&cursor)
        .expect("truncate back to safe offset");

    let recovery_bundle = FixtureBundle::new(
        descriptor,
        vec![FixtureSlot::new(SlotId::new(0), 0x03, &[3])],
    );
    let recovery_offset = writer
        .append_bundle(&recovery_bundle)
        .expect("append recovery entry");
    assert_eq!(recovery_offset.position, cursor.safe_offset);
    drop(writer);

    // Opening again now yields the original first entry and the repaired
    // entry, with no lingering corruption.
    let mut reader = WalReader::open(&wal_path).expect("reader after recovery");
    let mut iter = reader.iter_from(0).expect("iterator");
    let first = iter.next().expect("first entry").expect("ok");
    assert_eq!(first.sequence, 0);
    let repaired = iter.next().expect("repaired entry").expect("ok");
    assert!(iter.next().is_none(), "only two entries remain");

    assert_eq!(
        repaired
            .slots
            .first()
            .expect("slot present")
            .schema_fingerprint,
        [0x03; 32]
    );
}

#[test]
fn wal_writer_rejects_segment_config_mismatch() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("mismatch.wal");
    let original_hash = [0xAA; 16];

    // Create a WAL with one config hash.
    {
        let options = WalWriterOptions::new(wal_path.clone(), original_hash, Duration::ZERO);
        let _writer = WalWriter::open(options).expect("initial open");
    }

    // Attempt to reopen with a different hash.
    let different_hash = [0xBB; 16];
    let options = WalWriterOptions::new(wal_path, different_hash, Duration::ZERO);
    match WalWriter::open(options) {
        Err(WalError::SegmentConfigMismatch { expected, found }) => {
            assert_eq!(expected, different_hash);
            assert_eq!(found, original_hash);
        }
        other => panic!("expected segment config mismatch, got {:?}", other),
    }
}

#[test]
fn wal_reader_detects_unexpected_segment_config() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("reader_mismatch.wal");
    let stored_hash = [0xDD; 16];

    // Write a WAL with a known config hash.
    {
        let options = WalWriterOptions::new(wal_path.clone(), stored_hash, Duration::ZERO);
        let _writer = WalWriter::open(options).expect("writer");
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

#[test]
fn wal_reader_fails_on_corrupt_header_version() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("bad_version.wal");

    // Create a valid WAL first.
    {
        let options = WalWriterOptions::new(wal_path.clone(), [0x11; 16], Duration::ZERO);
        let _writer = WalWriter::open(options).expect("writer");
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
        file.write_all(&99u16.to_le_bytes()).expect("corrupt version");
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
    punch_capable: bool,
}

#[test]
fn wal_writer_recovers_from_crash_resilience_scenarios() {
    let cases = [
        CrashCase {
            name: "sidecar_pre_rename",
            injection: writer_test_support::CrashInjection::BeforeSidecarRename,
            punch_capable: true,
        },
        CrashCase {
            name: "rewrite_mid_copy",
            injection: writer_test_support::CrashInjection::DuringRewriteCopy,
            punch_capable: false,
        },
        CrashCase {
            name: "post_punch",
            injection: writer_test_support::CrashInjection::AfterPunch,
            punch_capable: true,
        },
    ];

    for case in cases {
        run_crash_case(case);
    }
}

fn run_crash_case(case: CrashCase) {
    writer_test_support::reset_flush_notifications();

    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join(format!("crash_{}.wal", case.name));
    let descriptor = BundleDescriptor::new(vec![slot_descriptor(0, "Logs")]);
    let mut options = WalWriterOptions::new(wal_path.clone(), [0xC7; 16], Duration::ZERO)
        .with_rotation_target(32 * 1024)
        .with_max_chunks(4);
    options = if case.punch_capable {
        options.with_punch_capability(true)
    } else {
        options.with_punch_capability(false)
    };

    let mut writer = WalWriter::open(options.clone()).expect("writer");
    for value in 0..4 {
        let bundle = FixtureBundle::new(
            descriptor.clone(),
            vec![FixtureSlot::with_batch(
                SlotId::new(0),
                0x60 + value as u8,
                build_complex_batch(256, "crash", 1024),
            )],
        );
        let _ = writer.append_bundle(&bundle).expect("append bundle");
    }

    for value in 0..4 {
        let bundle = FixtureBundle::new(
            descriptor.clone(),
            vec![FixtureSlot::new(SlotId::new(0), 0x40 + value as u8, &[(value + 1) as i64])],
        );
        let _ = writer.append_bundle(&bundle).expect("append bundle");
    }

    let cursor = wal_cursor_after_entries(&wal_path, 2);
    assert!(
        cursor.safe_offset > WAL_HEADER_LEN as u64,
        "{}: cursor safe offset must exceed header",
        case.name
    );
    if !case.punch_capable {
        writer_test_support::set_force_punch_error(true);
    }
    writer_test_support::inject_crash(case.injection);
    let err = match writer.reclaim_prefix(&cursor) {
        Ok(_) => panic!("{}: crash injection did not trigger", case.name),
        Err(err) => err,
    };
    assert!(
        matches!(err, WalError::InjectedCrash(_)),
        "unexpected error: {:?}",
        err
    );

    writer.test_force_crash();

    assert_crash_recovery(&options, &descriptor, case.name, &cursor);
    writer_test_support::reset_flush_notifications();
}

fn wal_cursor_after_entries(path: &Path, entry_count: usize) -> WalTruncateCursor {
    let mut reader = WalReader::open(path).expect("reader for cursor");
    let mut iter = reader.iter_from(0).expect("cursor iterator");
    let mut cursor = WalTruncateCursor::default();
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
        cursor.advance(&bundle);
    }
    cursor
}

fn assert_crash_recovery(
    options: &WalWriterOptions,
    descriptor: &BundleDescriptor,
    case_name: &str,
    cursor: &WalTruncateCursor,
) {
    assert_reader_clean(&options.path, cursor.safe_offset, case_name);

    let mut writer = WalWriter::open(options.clone()).expect("writer reopen");
    let repair_bundle = FixtureBundle::new(
        descriptor.clone(),
        vec![FixtureSlot::new(SlotId::new(0), 0xF0, &[99])],
    );
    let _ = writer
        .append_bundle(&repair_bundle)
        .expect("append after crash");
    drop(writer);

    assert_reader_clean(&options.path, cursor.safe_offset, case_name);

    let sidecar_path = options
        .path
        .parent()
        .expect("wal dir")
        .join("truncate.offset");
    if sidecar_path.exists() {
        let sidecar = TruncateSidecar::read_from(&sidecar_path).expect("sidecar readable");
        let wal_len = std::fs::metadata(&options.path).expect("metadata").len();
        assert!(
            sidecar.truncate_offset >= WAL_HEADER_LEN as u64,
            "{case_name}: truncate offset must include header"
        );
        assert!(
            sidecar.truncate_offset <= wal_len,
            "{case_name}: truncate offset must not exceed file len"
        );
    }
}

fn assert_reader_clean(path: &Path, offset: u64, case_name: &str) {
    let mut reader = WalReader::open(path)
        .unwrap_or_else(|err| panic!("{}: reader open failed: {:?}", case_name, err));
    let mut iter = reader
        .iter_from(offset)
        .unwrap_or_else(|err| panic!("{}: iterator init failed: {:?}", case_name, err));
    while let Some(entry) = iter.next() {
        let _ = entry.unwrap_or_else(|err| panic!("{}: wal entry error {:?}", case_name, err));
    }
}
