//! Cross-cutting WAL tests live here so shared fixtures can touch writer, reader,
//! and helper plumbing without sprinkling large #[cfg(test)] blocks in each file.

use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arrow_array::{Int64Array, RecordBatch};
use arrow_ipc::reader::StreamReader;
use arrow_schema::{DataType, Field, Schema};
use crc32fast::Hasher;
use tempfile::tempdir;

use crate::record_bundle::{
    BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
};

use super::header::{WAL_HEADER_LEN, WalHeader};
use super::{
    ENTRY_HEADER_LEN, ENTRY_TYPE_RECORD_BUNDLE, SCHEMA_FINGERPRINT_LEN, WalError, WalReader,
    WalTruncateCursor, WalWriter, WalWriterOptions,
};

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

fn slot_descriptor(id: u16, label: &'static str) -> SlotDescriptor {
    SlotDescriptor::new(SlotId::new(id), label)
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

fn write_single_entry(body: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
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

#[test]
fn wal_writer_reader_roundtrip_recovers_payloads() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("roundtrip.wal");
    let hash = [0xAB; 16];

    let descriptor = BundleDescriptor::new(vec![
        slot_descriptor(0, "Logs"),
        slot_descriptor(1, "Metrics"),
        slot_descriptor(2, "Traces"),
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
