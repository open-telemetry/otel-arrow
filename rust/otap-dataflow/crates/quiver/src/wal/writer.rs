use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use crc32fast::Hasher;

use crate::record_bundle::{PayloadRef, RecordBundle, SchemaFingerprint, SlotId};

use super::header::{WalHeader, WAL_HEADER_LEN};
use super::{WalError, WalResult, ENTRY_TYPE_RECORD_BUNDLE};

const ENTRY_HEADER_LEN: usize = 1 + 8 + 8 + 8;
const SCHEMA_FINGERPRINT_LEN: usize = 32;
const SLOT_HEADER_LEN: usize = 2 + SCHEMA_FINGERPRINT_LEN + 4 + 4;

#[derive(Debug, Clone)]
pub(crate) struct WalWriterOptions {
    pub path: PathBuf,
    pub segment_cfg_hash: [u8; 16],
    pub flush_interval: Duration,
}

impl WalWriterOptions {
    pub fn new(path: PathBuf, segment_cfg_hash: [u8; 16], flush_interval: Duration) -> Self {
        Self {
            path,
            segment_cfg_hash,
            flush_interval,
        }
    }
}

pub(crate) struct WalWriter {
    file: File,
    payload_buffer: Vec<u8>,
    options: WalWriterOptions,
    next_sequence: u64,
    last_flush: Instant,
}

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
            .open(&options.path)?;

        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            let header = WalHeader::new(options.segment_cfg_hash);
            header.write_to(&mut file)?;
            file.flush()?;
        } else if metadata.len() < WAL_HEADER_LEN as u64 {
            return Err(WalError::InvalidHeader("file smaller than header"));
        } else {
            let _header = WalHeader::read_from(&mut file)?;
        }

        let _ = file.seek(SeekFrom::End(0))?;

        Ok(Self {
            file,
            payload_buffer: Vec::new(),
            options,
            next_sequence: 0,
            last_flush: Instant::now(),
        })
    }

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
        let entry_len = u32::try_from(entry_body_len)
            .map_err(|_| WalError::EntryTooLarge(entry_body_len))?;

        let mut hasher = Hasher::new();
        hasher.update(&entry_header);
        hasher.update(&self.payload_buffer);
        let crc = hasher.finalize();

        self.file.write_all(&entry_len.to_le_bytes())?;
        self.file.write_all(&entry_header)?;
        self.file.write_all(&self.payload_buffer)?;
        self.file.write_all(&crc.to_le_bytes())?;

        self.maybe_flush()?;

        Ok(WalOffset {
            position: entry_start,
            sequence,
        })
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

    fn maybe_flush(&mut self) -> WalResult<()> {
        if self.options.flush_interval.is_zero() {
            self.file.flush()?;
            return Ok(());
        }

        if self.last_flush.elapsed() >= self.options.flush_interval {
            self.file.flush()?;
            self.last_flush = Instant::now();
        }
        Ok(())
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
        let mut writer = StreamWriter::try_new(&mut buffer, &schema)
            .map_err(WalError::Arrow)?;
        writer.write(batch).map_err(WalError::Arrow)?;
        writer.finish().map_err(WalError::Arrow)?;
    }
    Ok(buffer)
}

struct EncodedSlot {
    slot_id_raw: u16,
    schema_fingerprint: SchemaFingerprint,
    row_count: u32,
    payload_len: u32,
    payload_bytes: Vec<u8>,
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
        buffer[cursor..cursor + SCHEMA_FINGERPRINT_LEN]
            .copy_from_slice(&self.schema_fingerprint);
        cursor += SCHEMA_FINGERPRINT_LEN;
        buffer[cursor..cursor + 4].copy_from_slice(&self.row_count.to_le_bytes());
        cursor += 4;
        buffer[cursor..cursor + 4].copy_from_slice(&self.payload_len.to_le_bytes());
        cursor += 4;
        buffer[cursor..cursor + self.payload_bytes.len()].copy_from_slice(&self.payload_bytes);
    }
}
