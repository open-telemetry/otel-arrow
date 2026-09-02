// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Version 1 snapshot artifact codec.

use std::collections::{HashMap, HashSet};

use crate::primitives::{
    FILELOG_FORMAT_VERSION, FINGERPRINT_MAX_BYTES, Reader, SNAPSHOT_FOOTER_MAGIC, SNAPSHOT_MAGIC,
    SNAPSHOT_MAX_RECORD_PAYLOAD_BYTES, Writer, crc32c, quarantine_reason_reserved,
};
use crate::{
    AdvisoryPath, CommittedFrontierGuard, DecodeError, EncodeError, FileId, FramingResume,
    LifecycleState, Locator,
};

/// Exact version 1 snapshot header width.
pub const SNAPSHOT_HEADER_BYTES: usize = 60;
/// Exact version 1 snapshot footer width.
pub const SNAPSHOT_FOOTER_BYTES: usize = 24;
const SNAPSHOT_RECORD_LENGTH_BYTES: u64 = size_of::<u32>() as u64;
const SNAPSHOT_RECORD_CRC_BYTES: u64 = size_of::<u32>() as u64;
/// Minimum version 1 payload: an Active record with empty bounded fields.
const SNAPSHOT_MIN_RECORD_PAYLOAD_BYTES: usize = 173;
/// Minimum complete version 1 record frame, including length and CRC.
const SNAPSHOT_MIN_RECORD_FRAME_BYTES: usize =
    size_of::<u32>() + SNAPSHOT_MIN_RECORD_PAYLOAD_BYTES + size_of::<u32>();
/// Maximum complete version 1 snapshot record frame.
pub const SNAPSHOT_MAX_RECORD_FRAME_BYTES: u64 =
    SNAPSHOT_RECORD_LENGTH_BYTES + SNAPSHOT_MAX_RECORD_PAYLOAD_BYTES + SNAPSHOT_RECORD_CRC_BYTES;

/// Immutable evidence attached to a quarantined record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineEvidence {
    /// Opaque diagnostic reason code.
    pub reason_code: u16,
    /// Observed file size at quarantine time.
    pub observed_size: u64,
    /// File epoch at quarantine time.
    pub quarantine_epoch: u32,
    /// Quarantine time as Unix nanoseconds.
    pub quarantine_time_unix_nano: u64,
}

/// One durable version 1 checkpoint record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRecord {
    /// Opaque durable record key.
    pub file_id: FileId,
    /// Current file epoch.
    pub file_epoch: u32,
    /// Acked source-byte frontier.
    pub committed_offset: u64,
    /// Evidence for bytes immediately before the frontier.
    pub committed_frontier_guard: CommittedFrontierGuard,
    /// Current bounded fingerprint evidence.
    pub fingerprint: Vec<u8>,
    /// Bytes excluded before fingerprinting.
    pub ignored_header_bytes: u32,
    /// Current or quarantined immutable locator.
    pub locator: Locator,
    /// Framing-profile structural version.
    pub framing_profile_version: u16,
    /// Framing-profile canonical digest.
    pub framing_profile_digest: [u8; 32],
    /// Durable record-framing resume state.
    pub framing_resume: FramingResume,
    /// Durable lifecycle state.
    pub lifecycle_state: LifecycleState,
    /// Evidence present exactly for quarantined records.
    pub quarantine_evidence: Option<QuarantineEvidence>,
    /// Last-observed time as Unix nanoseconds.
    pub last_seen_time_unix_nano: u64,
    /// Bounded advisory path evidence.
    pub advisory_path: AdvisoryPath,
}

impl SnapshotRecord {
    fn validate(&self) -> Result<(), &'static str> {
        if self.file_epoch == 0 {
            return Err("file_epoch must be nonzero");
        }
        if !self
            .committed_frontier_guard
            .valid_for_offset(self.committed_offset)
        {
            return Err("frontier guard must have the required window and canonical empty digest");
        }
        if self.locator == Locator::Unspecified {
            return Err("locator must not be Unspecified");
        }
        if self.fingerprint.len() > FINGERPRINT_MAX_BYTES {
            return Err("fingerprint exceeds the version 1 maximum");
        }
        if u64::from(self.ignored_header_bytes)
            .checked_add(self.fingerprint.len() as u64)
            .is_none()
        {
            return Err("ignored_header_bytes plus fingerprint length overflows u64");
        }
        if self.framing_profile_version == 0 {
            return Err("framing_profile_version must be nonzero");
        }
        if !self.framing_resume.valid_for_offset(self.committed_offset) {
            return Err("framing resume is inconsistent with committed_offset");
        }
        match (self.lifecycle_state, &self.quarantine_evidence) {
            (LifecycleState::Active, None) => Ok(()),
            (LifecycleState::RotatedFinalized, None)
                if self.framing_resume == FramingResume::Clean =>
            {
                Ok(())
            }
            (LifecycleState::RotatedFinalized, None) => {
                Err("RotatedFinalized requires a clean framing resume")
            }
            (LifecycleState::Quarantined, Some(evidence))
                if evidence.reason_code != 0 && evidence.quarantine_epoch == self.file_epoch =>
            {
                Ok(())
            }
            (LifecycleState::Quarantined, Some(evidence)) if evidence.reason_code == 0 => {
                Err("Quarantined requires a nonzero reason code")
            }
            (LifecycleState::Quarantined, Some(_)) => Err("quarantine_epoch must equal file_epoch"),
            (LifecycleState::Quarantined, None) => Err("Quarantined requires quarantine evidence"),
            (LifecycleState::Active | LifecycleState::RotatedFinalized, Some(_)) => {
                Err("non-Quarantined must not carry quarantine evidence")
            }
        }
    }

    fn encode_payload(&self) -> Result<Vec<u8>, EncodeError> {
        if self.lifecycle_state == LifecycleState::Quarantined
            && let Some(evidence) = &self.quarantine_evidence
            && quarantine_reason_reserved(evidence.reason_code)
        {
            return Err(EncodeError::ReservedReasonCode {
                field: "snapshot_record.quarantine_reason_code",
                reason_code: evidence.reason_code,
            });
        }
        self.validate()
            .map_err(|reason| EncodeError::InvalidSnapshotState {
                file_id: self.file_id,
                reason,
            })?;

        let mut out = Writer::new();
        out.bytes(self.file_id.as_bytes());
        out.u32(self.file_epoch);
        out.u64(self.committed_offset);
        self.committed_frontier_guard.write(&mut out);
        out.var_bytes(
            "snapshot_record.fingerprint",
            &self.fingerprint,
            FINGERPRINT_MAX_BYTES,
        )?;
        out.u32(self.ignored_header_bytes);
        self.locator.write(&mut out);
        out.u16(self.framing_profile_version);
        out.bytes(&self.framing_profile_digest);
        self.framing_resume.write(&mut out);
        out.u8(self.lifecycle_state.to_wire());
        if let Some(evidence) = &self.quarantine_evidence {
            out.u16(evidence.reason_code);
            out.u64(evidence.observed_size);
            out.u32(evidence.quarantine_epoch);
            out.u64(evidence.quarantine_time_unix_nano);
        }
        out.u64(self.last_seen_time_unix_nano);
        self.advisory_path.write(&mut out);
        Ok(out.finish())
    }

    fn encode_frame(&self) -> Result<Vec<u8>, EncodeError> {
        let payload = self.encode_payload()?;
        let payload_len =
            u32::try_from(payload.len()).map_err(|_| EncodeError::ArithmeticOverflow {
                context: "snapshot record payload length",
            })?;
        let mut out = Writer::new();
        out.u32(payload_len);
        out.bytes(&payload);
        out.u32(crc32c(out.as_slice()));
        Ok(out.finish())
    }

    fn decode_payload(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut input = Reader::new(bytes);
        let file_id = FileId::from_bytes(input.array()?);
        let file_epoch = input.u32()?;
        let committed_offset = input.u64()?;
        let committed_frontier_guard = CommittedFrontierGuard::read(&mut input)?;
        let fingerprint = input
            .var_bytes("snapshot_record.fingerprint", FINGERPRINT_MAX_BYTES)?
            .to_vec();
        let ignored_header_bytes = input.u32()?;
        let locator = Locator::read(&mut input)?;
        let framing_profile_version = input.u16()?;
        let framing_profile_digest = input.array()?;
        let framing_resume = FramingResume::read(&mut input)?;
        let lifecycle_state =
            LifecycleState::from_wire(input.u8()?, "snapshot_record.lifecycle_state")?;
        let quarantine_evidence = if lifecycle_state == LifecycleState::Quarantined {
            Some(QuarantineEvidence {
                reason_code: input.u16()?,
                observed_size: input.u64()?,
                quarantine_epoch: input.u32()?,
                quarantine_time_unix_nano: input.u64()?,
            })
        } else {
            None
        };
        let last_seen_time_unix_nano = input.u64()?;
        let advisory_path = AdvisoryPath::read(&mut input)?;
        if input.remaining() != 0 {
            return Err(DecodeError::UnconsumedBytes {
                context: "snapshot record",
                declared: bytes.len(),
                consumed: input.position(),
            });
        }
        let record = Self {
            file_id,
            file_epoch,
            committed_offset,
            committed_frontier_guard,
            fingerprint,
            ignored_header_bytes,
            locator,
            framing_profile_version,
            framing_profile_digest,
            framing_resume,
            lifecycle_state,
            quarantine_evidence,
            last_seen_time_unix_nano,
            advisory_path,
        };
        record
            .validate()
            .map_err(|reason| DecodeError::InvalidSnapshotState { file_id, reason })?;
        Ok(record)
    }

    fn decode_frame(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let mut input = Reader::new(bytes);
        let declared = u64::from(input.u32()?);
        if declared > SNAPSHOT_MAX_RECORD_PAYLOAD_BYTES {
            return Err(DecodeError::LengthExceedsMaximum {
                field: "snapshot_record.record_len",
                declared,
                max: SNAPSHOT_MAX_RECORD_PAYLOAD_BYTES,
            });
        }
        let payload_len =
            usize::try_from(declared).map_err(|_| DecodeError::ArithmeticOverflow {
                context: "snapshot record length to usize",
            })?;
        let payload = input.exact(payload_len)?;
        let stored = input.u32()?;
        let consumed = input.position();
        let computed = crc32c(&bytes[..consumed - 4]);
        if stored != computed {
            return Err(DecodeError::ChecksumMismatch {
                context: "snapshot record",
                stored,
                computed,
            });
        }
        Ok((Self::decode_payload(payload)?, consumed))
    }
}

/// Fully decoded version 1 snapshot contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Generation declared by the snapshot.
    pub generation: u64,
    /// Records in exact on-disk order.
    pub records: Vec<SnapshotRecord>,
}

fn validate_uniqueness(records: &[SnapshotRecord]) -> Result<(), EncodeError> {
    let mut ids = HashSet::with_capacity(records.len());
    let mut locators = HashMap::with_capacity(records.len());
    for record in records {
        if !ids.insert(record.file_id) {
            return Err(EncodeError::DuplicateFileId {
                file_id: record.file_id,
            });
        }
        if record.lifecycle_state != LifecycleState::RotatedFinalized
            && let Some(first) = locators.insert(record.locator, record.file_id)
        {
            return Err(EncodeError::DuplicateLiveLocator {
                first,
                second: record.file_id,
            });
        }
    }
    Ok(())
}

/// Encodes a complete version 1 snapshot artifact.
pub fn encode_snapshot(
    generation: u64,
    checkpoint_id: &str,
    records: &[SnapshotRecord],
) -> Result<Vec<u8>, EncodeError> {
    validate_uniqueness(records)?;
    let record_count =
        u32::try_from(records.len()).map_err(|_| EncodeError::ArithmeticOverflow {
            context: "snapshot record count",
        })?;
    let namespace = crate::namespace_digest(checkpoint_id)?;
    let mut out = Writer::new();
    out.bytes(SNAPSHOT_MAGIC);
    out.u16(FILELOG_FORMAT_VERSION);
    out.u16(0);
    out.u64(generation);
    out.bytes(&namespace);
    out.u32(record_count);
    out.u32(crc32c(out.as_slice()));

    let mut total_record_bytes = 0u64;
    for record in records {
        let frame = record.encode_frame()?;
        total_record_bytes = total_record_bytes.checked_add(frame.len() as u64).ok_or(
            EncodeError::ArithmeticOverflow {
                context: "snapshot total record bytes",
            },
        )?;
        out.bytes(&frame);
    }

    let footer_start = out.as_slice().len();
    out.bytes(SNAPSHOT_FOOTER_MAGIC);
    out.u64(total_record_bytes);
    out.u32(record_count);
    let footer_crc = crc32c(&out.as_slice()[footer_start..]);
    out.u32(footer_crc);
    Ok(out.finish())
}

/// Decodes one complete version 1 snapshot for `expected_namespace_digest`.
///
/// The authenticated header count is checked against both `max_records` and
/// the maximum number of minimum-width frames physically possible in `bytes`
/// before any record storage is allocated or any record body is decoded.
pub fn decode_snapshot(
    bytes: &[u8],
    expected_namespace_digest: &[u8; 32],
    max_records: u32,
) -> Result<Snapshot, DecodeError> {
    if bytes.len() < SNAPSHOT_HEADER_BYTES {
        return Err(DecodeError::Truncated {
            needed: SNAPSHOT_HEADER_BYTES,
            available: bytes.len(),
        });
    }
    let mut header = Reader::new(bytes);
    if header.exact(8)? != SNAPSHOT_MAGIC {
        return Err(DecodeError::BadMagic {
            context: "snapshot header",
        });
    }
    let version = header.u16()?;
    if version != FILELOG_FORMAT_VERSION {
        return Err(DecodeError::UnsupportedVersion {
            context: "snapshot header",
            found: version,
        });
    }
    let flags = header.u16()?;
    if flags != 0 {
        return Err(DecodeError::ReservedFieldNonZero {
            field: "snapshot_header.flags",
            value: u64::from(flags),
        });
    }
    let generation = header.u64()?;
    let namespace = header.array()?;
    let record_count = header.u32()?;
    let stored_header_crc = header.u32()?;
    let computed_header_crc = crc32c(&bytes[..56]);
    if stored_header_crc != computed_header_crc {
        return Err(DecodeError::ChecksumMismatch {
            context: "snapshot header",
            stored: stored_header_crc,
            computed: computed_header_crc,
        });
    }
    if &namespace != expected_namespace_digest {
        return Err(DecodeError::NamespaceMismatch {
            context: "snapshot",
        });
    }
    if record_count > max_records {
        return Err(DecodeError::SnapshotRecordCountExceedsLimit {
            declared: record_count,
            max: max_records,
        });
    }
    let available_record_region = bytes
        .len()
        .saturating_sub(SNAPSHOT_HEADER_BYTES.saturating_add(SNAPSHOT_FOOTER_BYTES));
    let maximum_physical_records =
        u64::try_from(available_record_region / SNAPSHOT_MIN_RECORD_FRAME_BYTES)
            .unwrap_or(u64::MAX);
    if u64::from(record_count) > maximum_physical_records {
        return Err(DecodeError::SnapshotRecordCountExceedsPhysicalMaximum {
            declared: record_count,
            max: maximum_physical_records,
            snapshot_bytes: bytes.len(),
        });
    }

    let capacity = usize::try_from(record_count).map_err(|_| DecodeError::ArithmeticOverflow {
        context: "snapshot record capacity",
    })?;
    let mut records = Vec::with_capacity(capacity);
    let mut ids = HashSet::with_capacity(capacity);
    let mut locators = HashMap::with_capacity(capacity);
    let mut cursor = SNAPSHOT_HEADER_BYTES;
    let mut total_record_bytes = 0u64;
    for _ in 0..record_count {
        let remaining = bytes.get(cursor..).ok_or(DecodeError::Truncated {
            needed: 4,
            available: 0,
        })?;
        let (record, consumed) = SnapshotRecord::decode_frame(remaining)?;
        if !ids.insert(record.file_id) {
            return Err(DecodeError::DuplicateFileId {
                file_id: record.file_id,
            });
        }
        if record.lifecycle_state != LifecycleState::RotatedFinalized
            && let Some(first) = locators.insert(record.locator, record.file_id)
        {
            return Err(DecodeError::DuplicateLiveLocator {
                first,
                second: record.file_id,
            });
        }
        cursor = cursor
            .checked_add(consumed)
            .ok_or(DecodeError::ArithmeticOverflow {
                context: "snapshot cursor",
            })?;
        total_record_bytes = total_record_bytes.checked_add(consumed as u64).ok_or(
            DecodeError::ArithmeticOverflow {
                context: "snapshot total record bytes",
            },
        )?;
        records.push(record);
    }

    let footer_end =
        cursor
            .checked_add(SNAPSHOT_FOOTER_BYTES)
            .ok_or(DecodeError::ArithmeticOverflow {
                context: "snapshot footer end",
            })?;
    if footer_end > bytes.len() {
        return Err(DecodeError::Truncated {
            needed: SNAPSHOT_FOOTER_BYTES,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    let footer_bytes = &bytes[cursor..footer_end];
    let mut footer = Reader::new(footer_bytes);
    if footer.exact(8)? != SNAPSHOT_FOOTER_MAGIC {
        return Err(DecodeError::BadMagic {
            context: "snapshot footer",
        });
    }
    let recorded_total = footer.u64()?;
    let echoed_count = footer.u32()?;
    let stored_footer_crc = footer.u32()?;
    let computed_footer_crc = crc32c(&footer_bytes[..20]);
    if stored_footer_crc != computed_footer_crc {
        return Err(DecodeError::ChecksumMismatch {
            context: "snapshot footer",
            stored: stored_footer_crc,
            computed: computed_footer_crc,
        });
    }
    if recorded_total != total_record_bytes {
        return Err(DecodeError::UnconsumedBytes {
            context: "snapshot footer total_record_bytes",
            declared: usize::try_from(recorded_total).unwrap_or(usize::MAX),
            consumed: usize::try_from(total_record_bytes).unwrap_or(usize::MAX),
        });
    }
    if echoed_count != record_count {
        return Err(DecodeError::UnconsumedBytes {
            context: "snapshot footer record_count_echo",
            declared: record_count as usize,
            consumed: echoed_count as usize,
        });
    }
    if footer_end != bytes.len() {
        return Err(DecodeError::TrailingBytes {
            context: "snapshot file",
            remaining: bytes.len() - footer_end,
        });
    }
    Ok(Snapshot {
        generation,
        records,
    })
}
