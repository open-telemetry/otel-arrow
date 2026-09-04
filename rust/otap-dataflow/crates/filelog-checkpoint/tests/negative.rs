// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Structural corruption, bounds, invariant, and incomplete-suffix tests.

use otel_arrow_dfe_filelog_checkpoint::{
    AdvisoryPath, CommittedFrontierGuard, DecodeError, EncodeError, FRAMING_PROFILE_VERSION,
    FileId, FramingResume, LifecycleState, Locator, Operation, QuarantineEvidence,
    SNAPSHOT_FOOTER_BYTES, SNAPSHOT_HEADER_BYTES, SnapshotRecord, TX_MIN_BODY_BYTES, Transaction,
    TransactionScan, WAL_MAX_OPS_PER_TX, crc32c, decode_current, decode_operation, decode_snapshot,
    decode_wal_header, encode_operation, encode_snapshot, encode_transaction, namespace_digest,
    scan_next_transaction,
};

const CURRENT: &[u8] = include_bytes!("fixtures/current-generation-42.bin");
const ACTIVE_SNAPSHOT: &[u8] = include_bytes!("fixtures/snapshot-active.bin");
const QUARANTINED_SNAPSHOT: &[u8] = include_bytes!("fixtures/snapshot-quarantined.bin");
const MIN_TX: &[u8] = include_bytes!("fixtures/transaction-minimum.bin");
const PROGRESS_OP: &[u8] = include_bytes!("fixtures/operation-update-progress.bin");
const METADATA_OP: &[u8] = include_bytes!("fixtures/operation-update-metadata.bin");

struct TestScanResult {
    transactions: Vec<Transaction>,
    incomplete_bytes: usize,
}

fn scan_all_for_test(bytes: &[u8]) -> Result<TestScanResult, DecodeError> {
    let mut suffix = bytes;
    let mut expected_sequence = 1u64;
    let mut transactions = Vec::new();
    let mut incomplete_bytes = 0;
    while let Some(scan) = scan_next_transaction(suffix, expected_sequence)? {
        match scan {
            TransactionScan::Complete {
                transaction,
                consumed,
            } => {
                transactions.push(transaction);
                suffix = &suffix[consumed..];
                expected_sequence += 1;
            }
            TransactionScan::Incomplete { bytes } => {
                incomplete_bytes = bytes;
                break;
            }
        }
    }
    Ok(TestScanResult {
        transactions,
        incomplete_bytes,
    })
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
}

fn refresh_operation_crc(bytes: &mut [u8]) {
    let crc_offset = bytes.len() - 4;
    let checksum = crc32c(&bytes[..crc_offset]);
    put_u32(bytes, crc_offset, checksum);
}

fn operation_from_payload(payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(payload.len() + 8);
    bytes.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&crc32c(&bytes).to_be_bytes());
    bytes
}

fn refresh_transaction_crcs(bytes: &mut [u8]) {
    let header_checksum = crc32c(&bytes[..32]);
    put_u32(bytes, 32, header_checksum);
    let frame_crc_offset = bytes.len() - 4;
    let frame_checksum = crc32c(&bytes[..frame_crc_offset]);
    put_u32(bytes, frame_crc_offset, frame_checksum);
}

fn transaction_from_operations(operations: &[&[u8]]) -> Vec<u8> {
    let body: Vec<u8> = operations
        .iter()
        .flat_map(|operation| operation.iter().copied())
        .collect();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"FLOGTXN\0");
    bytes.extend_from_slice(&1u16.to_be_bytes());
    bytes.extend_from_slice(&0u16.to_be_bytes());
    bytes.extend_from_slice(&1u64.to_be_bytes());
    bytes.extend_from_slice(&(body.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&(!(body.len() as u32)).to_be_bytes());
    bytes.extend_from_slice(&(operations.len() as u16).to_be_bytes());
    bytes.extend_from_slice(&0u16.to_be_bytes());
    bytes.extend_from_slice(&crc32c(&bytes).to_be_bytes());
    bytes.extend_from_slice(&body);
    bytes.extend_from_slice(&crc32c(&bytes).to_be_bytes());
    bytes
}

fn active_record_frame() -> Vec<u8> {
    ACTIVE_SNAPSHOT[60..ACTIVE_SNAPSHOT.len() - 24].to_vec()
}

fn quarantined_record_frame() -> Vec<u8> {
    QUARANTINED_SNAPSHOT[60..QUARANTINED_SNAPSHOT.len() - 24].to_vec()
}

fn refresh_record_crc(frame: &mut [u8]) {
    let crc_offset = frame.len() - 4;
    let checksum = crc32c(&frame[..crc_offset]);
    put_u32(frame, crc_offset, checksum);
}

fn snapshot_from_frames(frames: &[Vec<u8>]) -> Vec<u8> {
    let mut header = ACTIVE_SNAPSHOT[..60].to_vec();
    put_u32(&mut header, 52, frames.len() as u32);
    let header_checksum = crc32c(&header[..56]);
    put_u32(&mut header, 56, header_checksum);
    let total: usize = frames.iter().map(Vec::len).sum();
    let mut bytes = header;
    for frame in frames {
        bytes.extend_from_slice(frame);
    }
    let mut footer = Vec::new();
    footer.extend_from_slice(b"FLOGSFT\0");
    footer.extend_from_slice(&(total as u64).to_be_bytes());
    footer.extend_from_slice(&(frames.len() as u32).to_be_bytes());
    footer.extend_from_slice(&crc32c(&footer).to_be_bytes());
    bytes.extend_from_slice(&footer);
    bytes
}

fn resize_record_payload(mut frame: Vec<u8>, payload: Vec<u8>) -> Vec<u8> {
    frame.clear();
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(&payload);
    frame.extend_from_slice(&crc32c(&frame).to_be_bytes());
    frame
}

fn snapshot_with_active_advisory(advisory: &[u8]) -> Vec<u8> {
    let frame = active_record_frame();
    let mut payload = frame[4..frame.len() - 4].to_vec();
    payload.truncate(145);
    payload.extend_from_slice(advisory);
    snapshot_from_frames(&[resize_record_payload(frame, payload)])
}

fn continuation_frame(start: u64, end: u64, index: u32, lifecycle: u8) -> Vec<u8> {
    let frame = active_record_frame();
    let mut payload = frame[4..frame.len() - 4].to_vec();
    let mut resume = Vec::with_capacity(21);
    resume.push(1);
    resume.extend_from_slice(&start.to_be_bytes());
    resume.extend_from_slice(&end.to_be_bytes());
    resume.extend_from_slice(&index.to_be_bytes());
    drop(payload.splice(135..136, resume));
    payload[156] = lifecycle;
    resize_record_payload(frame, payload)
}

/// Scenario: CURRENT is short, has bad magic/version/flags, or has a bad CRC.
/// Guarantees: Every fixed marker envelope failure is rejected rather than partially accepted.
#[test]
fn current_envelope_corruption_is_rejected() {
    assert!(matches!(
        decode_current(&CURRENT[..23]),
        Err(DecodeError::InvalidLength { .. })
    ));
    let mut bytes = CURRENT.to_vec();
    bytes[0] ^= 1;
    assert!(matches!(
        decode_current(&bytes),
        Err(DecodeError::BadMagic { .. })
    ));
    let mut bytes = CURRENT.to_vec();
    put_u16(&mut bytes, 8, 2);
    assert!(matches!(
        decode_current(&bytes),
        Err(DecodeError::UnsupportedVersion { .. })
    ));
    let mut bytes = CURRENT.to_vec();
    put_u16(&mut bytes, 10, 1);
    assert!(matches!(
        decode_current(&bytes),
        Err(DecodeError::ReservedFieldNonZero { .. })
    ));
    let mut bytes = CURRENT.to_vec();
    bytes[23] ^= 1;
    assert!(matches!(
        decode_current(&bytes),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
}

/// Scenario: Snapshot header CRC, record CRC, footer CRC, truncation, and trailing bytes are corrupted.
/// Guarantees: Snapshots fail closed with no WAL-style torn-tail salvage.
#[test]
fn snapshot_artifact_corruption_is_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let mut bytes = ACTIVE_SNAPSHOT.to_vec();
    bytes[59] ^= 1;
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
    let mut bytes = ACTIVE_SNAPSHOT.to_vec();
    bytes[20] ^= 1;
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
    let mut bytes = ACTIVE_SNAPSHOT.to_vec();
    bytes[ACTIVE_SNAPSHOT.len() - 25] ^= 1;
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
    let mut bytes = ACTIVE_SNAPSHOT.to_vec();
    let footer_crc = bytes.len() - 1;
    bytes[footer_crc] ^= 1;
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
    assert!(matches!(
        decode_snapshot(
            &ACTIVE_SNAPSHOT[..ACTIVE_SNAPSHOT.len() - 1],
            &namespace,
            u32::MAX
        ),
        Err(DecodeError::Truncated { .. })
    ));
    let mut bytes = ACTIVE_SNAPSHOT.to_vec();
    bytes.push(0);
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::TrailingBytes { .. })
    ));
}

/// Scenario: Snapshot magic, format version, or reserved flags are invalid.
/// Guarantees: Snapshot headers use a closed versioned envelope before record decoding.
#[test]
fn snapshot_header_discriminants_are_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let mut magic = ACTIVE_SNAPSHOT.to_vec();
    magic[0] ^= 1;
    assert!(matches!(
        decode_snapshot(&magic, &namespace, u32::MAX),
        Err(DecodeError::BadMagic { .. })
    ));
    let mut version = ACTIVE_SNAPSHOT.to_vec();
    put_u16(&mut version, 8, 2);
    assert!(matches!(
        decode_snapshot(&version, &namespace, u32::MAX),
        Err(DecodeError::UnsupportedVersion { .. })
    ));
    let mut flags = ACTIVE_SNAPSHOT.to_vec();
    put_u16(&mut flags, 10, 1);
    assert!(matches!(
        decode_snapshot(&flags, &namespace, u32::MAX),
        Err(DecodeError::ReservedFieldNonZero { .. })
    ));
}

/// Scenario: A snapshot record declares an excessive or unavailable payload length.
/// Guarantees: Absolute bounds are checked before slicing and remaining-byte checks fail safely.
#[test]
fn snapshot_record_lengths_are_bounded_before_slicing() {
    let namespace = namespace_digest("app-logs").unwrap();
    let mut bytes = ACTIVE_SNAPSHOT.to_vec();
    put_u32(&mut bytes, 60, 69_855);
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::LengthExceedsMaximum { .. })
    ));
    let mut bytes = ACTIVE_SNAPSHOT.to_vec();
    put_u32(&mut bytes, 60, 500);
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::Truncated { .. })
    ));
}

/// Scenario: A one-record snapshot is opened with configured maxima of one and zero.
/// Guarantees: Equality is accepted and limit plus one is rejected before a corrupt record body is decoded.
#[test]
fn snapshot_record_count_is_checked_before_record_allocation_and_decode() {
    let namespace = namespace_digest("app-logs").unwrap();
    assert_eq!(
        decode_snapshot(ACTIVE_SNAPSHOT, &namespace, 1)
            .unwrap()
            .records
            .len(),
        1
    );

    let mut corrupt_record = ACTIVE_SNAPSHOT.to_vec();
    corrupt_record[ACTIVE_SNAPSHOT.len() - 25] ^= 1;
    assert_eq!(
        decode_snapshot(&corrupt_record, &namespace, 0),
        Err(DecodeError::SnapshotRecordCountExceedsLimit {
            declared: 1,
            max: 0,
        })
    );
}

/// Scenario: Authenticated snapshot headers declare exactly one minimum-width
/// record, one more record than those bytes can hold, and `u32::MAX` records
/// in a header-only slice.
/// Guarantees: Exact physical equality decodes, while impossible counts are
/// rejected before a deliberately corrupt record body or any allocation.
#[test]
fn snapshot_record_count_is_physically_bounded_before_decode() {
    let namespace = namespace_digest("app-logs").unwrap();
    let minimum_record = SnapshotRecord {
        file_id: FileId::from_bytes([0xA1; 16]),
        file_epoch: 1,
        committed_offset: 0,
        committed_frontier_guard: CommittedFrontierGuard::empty(),
        fingerprint: Vec::new(),
        ignored_header_bytes: 0,
        locator: Locator::PosixDevIno { dev: 1, ino: 1 },
        framing_profile_version: FRAMING_PROFILE_VERSION,
        framing_profile_digest: [0xB2; 32],
        framing_resume: FramingResume::Clean,
        lifecycle_state: LifecycleState::Active,
        quarantine_evidence: None,
        last_seen_time_unix_nano: 0,
        advisory_path: AdvisoryPath::unavailable(),
    };
    let exact = encode_snapshot(7, "app-logs", &[minimum_record]).unwrap();
    assert_eq!(
        exact.len(),
        SNAPSHOT_HEADER_BYTES + 181 + SNAPSHOT_FOOTER_BYTES
    );
    assert_eq!(
        decode_snapshot(&exact, &namespace, u32::MAX)
            .unwrap()
            .records
            .len(),
        1
    );

    let mut one_more = exact.clone();
    put_u32(&mut one_more, 52, 2);
    let header_crc = crc32c(&one_more[..56]);
    put_u32(&mut one_more, 56, header_crc);
    one_more[SNAPSHOT_HEADER_BYTES + 4] ^= 1;
    assert_eq!(
        decode_snapshot(&one_more, &namespace, u32::MAX),
        Err(DecodeError::SnapshotRecordCountExceedsPhysicalMaximum {
            declared: 2,
            max: 1,
            snapshot_bytes: exact.len(),
        })
    );

    let mut header_only = exact[..SNAPSHOT_HEADER_BYTES].to_vec();
    put_u32(&mut header_only, 52, u32::MAX);
    let header_crc = crc32c(&header_only[..56]);
    put_u32(&mut header_only, 56, header_crc);
    assert_eq!(
        decode_snapshot(&header_only, &namespace, u32::MAX),
        Err(DecodeError::SnapshotRecordCountExceedsPhysicalMaximum {
            declared: u32::MAX,
            max: 0,
            snapshot_bytes: SNAPSHOT_HEADER_BYTES,
        })
    );
}

/// Scenario: An offset-zero snapshot record carries a zero-length guard with a mutated digest.
/// Guarantees: Snapshot encoding and decoding require the canonical empty-frontier digest, not only its zero window length.
#[test]
fn snapshot_offset_zero_guard_requires_canonical_digest() {
    let fixture = include_bytes!("fixtures/snapshot-rotated-finalized.bin");
    let namespace = namespace_digest("app-logs").unwrap();
    let mut record = decode_snapshot(fixture, &namespace, 1)
        .unwrap()
        .records
        .remove(0);
    record.committed_frontier_guard.digest[0] ^= 1;
    assert!(matches!(
        encode_snapshot(7, "app-logs", &[record]),
        Err(EncodeError::InvalidSnapshotState { .. })
    ));

    let mut frame = fixture[60..fixture.len() - 24].to_vec();
    frame[34] ^= 1;
    refresh_record_crc(&mut frame);
    assert!(matches!(
        decode_snapshot(&snapshot_from_frames(&[frame]), &namespace, 1),
        Err(DecodeError::InvalidSnapshotState { .. })
    ));
}

/// Scenario: A snapshot repeats an exact record frame.
/// Guarantees: Duplicate file IDs are rejected globally instead of last-write-wins decoding.
#[test]
fn duplicate_snapshot_file_id_is_rejected() {
    let frame = active_record_frame();
    let bytes = snapshot_from_frames(&[frame.clone(), frame]);
    let namespace = namespace_digest("app-logs").unwrap();
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::DuplicateFileId { .. })
    ));
}

/// Scenario: Two live records have distinct file IDs but claim the same POSIX locator.
/// Guarantees: Snapshot-wide live locator uniqueness is enforced independently of record order.
#[test]
fn duplicate_live_snapshot_locator_is_rejected() {
    let first = active_record_frame();
    let mut second = first.clone();
    second[4..20].copy_from_slice(&[0xAA; 16]);
    refresh_record_crc(&mut second);
    let bytes = snapshot_from_frames(&[first, second]);
    let namespace = namespace_digest("app-logs").unwrap();
    assert!(matches!(
        decode_snapshot(&bytes, &namespace, u32::MAX),
        Err(DecodeError::DuplicateLiveLocator { .. })
    ));
}

/// Scenario: A snapshot record carries an unknown locator kind or durable Unspecified locator.
/// Guarantees: Locator discriminants are closed and reachable durable records require concrete identity.
#[test]
fn invalid_snapshot_locator_is_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let mut unknown = active_record_frame();
    unknown[88] = 9;
    refresh_record_crc(&mut unknown);
    assert!(matches!(
        decode_snapshot(&snapshot_from_frames(&[unknown]), &namespace, u32::MAX),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));

    let frame = active_record_frame();
    let mut payload = frame[4..frame.len() - 4].to_vec();
    drop(payload.drain(85..101));
    payload[84] = 0;
    let unspecified = resize_record_payload(frame, payload);
    assert!(matches!(
        decode_snapshot(&snapshot_from_frames(&[unspecified]), &namespace, u32::MAX),
        Err(DecodeError::InvalidSnapshotState { .. })
    ));
}

/// Scenario: Snapshot lifecycle or framing-resume discriminants and shapes are invalid.
/// Guarantees: Unknown lifecycle values and locally unreachable continuation state fail closed.
#[test]
fn invalid_snapshot_lifecycle_and_resume_are_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let mut lifecycle = active_record_frame();
    lifecycle[140] = 9;
    refresh_record_crc(&mut lifecycle);
    assert!(matches!(
        decode_snapshot(&snapshot_from_frames(&[lifecycle]), &namespace, u32::MAX),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));

    let frame = active_record_frame();
    let mut payload = frame[4..frame.len() - 4].to_vec();
    payload[135] = 1;
    drop(payload.splice(136..136, [0u8; 20]));
    let invalid_resume = resize_record_payload(frame, payload);
    assert!(matches!(
        decode_snapshot(
            &snapshot_from_frames(&[invalid_resume]),
            &namespace,
            u32::MAX
        ),
        Err(DecodeError::InvalidSnapshotState { .. })
    ));
}

/// Scenario: CRC-valid snapshot records carry zero epoch or zero framing-profile version fields.
/// Guarantees: Both scalar reachable-state minima are rejected by snapshot decoding while unknown nonzero profile versions remain supported.
#[test]
fn snapshot_zero_epoch_and_profile_version_are_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    for (offset, width) in [(20, 4), (105, 2)] {
        let mut frame = active_record_frame();
        frame[offset..offset + width].fill(0);
        refresh_record_crc(&mut frame);
        assert!(matches!(
            decode_snapshot(&snapshot_from_frames(&[frame]), &namespace, 1),
            Err(DecodeError::InvalidSnapshotState { .. })
        ));
    }
}

/// Scenario: CRC-valid snapshot continuations violate each offset relationship or appear on a finalized record.
/// Guarantees: Continuation start/end reachability and the finalized-clean requirement are enforced before WAL replay.
#[test]
fn snapshot_continuation_reachability_is_enforced() {
    let namespace = namespace_digest("app-logs").unwrap();
    for frame in [
        continuation_frame(4, 0, 1, 1),
        continuation_frame(0, 4, 1, 1),
        continuation_frame(0, 0, 1, 2),
    ] {
        assert!(matches!(
            decode_snapshot(&snapshot_from_frames(&[frame]), &namespace, 1),
            Err(DecodeError::InvalidSnapshotState { .. })
        ));
    }
}

/// Scenario: CRC-valid quarantined snapshot records carry a zero reason or a quarantine epoch different from the file epoch.
/// Guarantees: Quarantine evidence is locally reachable only with a nonzero reason and matching epoch.
#[test]
fn snapshot_invalid_quarantine_evidence_values_are_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();

    let mut zero_reason = quarantined_record_frame();
    put_u16(&mut zero_reason, 149, 0);
    refresh_record_crc(&mut zero_reason);
    assert!(matches!(
        decode_snapshot(&snapshot_from_frames(&[zero_reason]), &namespace, 1),
        Err(DecodeError::InvalidSnapshotState { .. })
    ));

    let mut mismatched_epoch = quarantined_record_frame();
    put_u32(&mut mismatched_epoch, 159, 5);
    refresh_record_crc(&mut mismatched_epoch);
    assert!(matches!(
        decode_snapshot(&snapshot_from_frames(&[mismatched_epoch]), &namespace, 1),
        Err(DecodeError::InvalidSnapshotState { .. })
    ));
}

/// Scenario: CRC-valid snapshot payloads omit required quarantine evidence or append evidence to an Active shape.
/// Guarantees: Lifecycle-determined evidence layouts cannot be silently reinterpreted as reachable records.
#[test]
fn snapshot_quarantine_evidence_shape_is_enforced_during_decode() {
    let namespace = namespace_digest("app-logs").unwrap();

    let frame = quarantined_record_frame();
    let mut payload = frame[4..frame.len() - 4].to_vec();
    drop(payload.drain(145..167));
    let missing = resize_record_payload(frame, payload);
    assert!(decode_snapshot(&snapshot_from_frames(&[missing]), &namespace, 1).is_err());

    let frame = active_record_frame();
    let mut payload = frame[4..frame.len() - 4].to_vec();
    let mut evidence = Vec::with_capacity(22);
    evidence.extend_from_slice(&1u16.to_be_bytes());
    evidence.extend_from_slice(&88u64.to_be_bytes());
    evidence.extend_from_slice(&2u32.to_be_bytes());
    evidence.extend_from_slice(&99u64.to_be_bytes());
    drop(payload.splice(137..137, evidence));
    let unexpected = resize_record_payload(frame, payload);
    assert!(decode_snapshot(&snapshot_from_frames(&[unexpected]), &namespace, 1).is_err());
}

/// Scenario: An in-memory Active record incorrectly carries quarantine evidence.
/// Guarantees: The encoder rejects the impossible presence shape rather than emitting ambiguous bytes.
#[test]
fn invalid_quarantine_presence_shape_is_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let mut record = decode_snapshot(ACTIVE_SNAPSHOT, &namespace, u32::MAX)
        .unwrap()
        .records
        .remove(0);
    record.quarantine_evidence = Some(QuarantineEvidence {
        reason_code: 1,
        observed_size: 0,
        quarantine_epoch: record.file_epoch,
        quarantine_time_unix_nano: 0,
    });
    assert!(matches!(
        encode_snapshot(1, "app-logs", &[record]),
        Err(EncodeError::InvalidSnapshotState { .. })
    ));
}

/// Scenario: Current-version encoders receive reserved quarantine or removal reason codes.
/// Guarantees: Encoders never produce reserved diagnostic values even though decoders keep reason fields opaque.
#[test]
fn encoder_rejects_reserved_reason_codes() {
    let namespace = namespace_digest("app-logs").unwrap();
    let fixture = include_bytes!("fixtures/snapshot-quarantined.bin");
    let mut record = decode_snapshot(fixture, &namespace, u32::MAX)
        .unwrap()
        .records
        .remove(0);
    record.quarantine_evidence.as_mut().unwrap().reason_code = 0;
    assert!(matches!(
        encode_snapshot(7, "app-logs", &[record]),
        Err(EncodeError::ReservedReasonCode { .. })
    ));

    let mut quarantine = decode_operation(include_bytes!("fixtures/operation-quarantine-file.bin"))
        .unwrap()
        .0;
    let Operation::QuarantineFile(operation) = &mut quarantine else {
        panic!("fixture must be quarantine_file");
    };
    operation.reason_code = 4;
    assert!(matches!(
        encode_operation(&quarantine),
        Err(EncodeError::ReservedReasonCode { .. })
    ));

    let mut remove = decode_operation(include_bytes!("fixtures/operation-remove-file.bin"))
        .unwrap()
        .0;
    let Operation::RemoveFile(operation) = &mut remove else {
        panic!("fixture must be remove_file");
    };
    operation.removal_reason = 0;
    assert!(matches!(
        encode_operation(&remove),
        Err(EncodeError::ReservedReasonCode { .. })
    ));
}

fn assert_operation_encode_rejected(operation: Operation) {
    assert!(matches!(
        encode_operation(&operation),
        Err(EncodeError::InvalidFieldValue { .. })
            | Err(EncodeError::ReservedReasonCode { .. })
            | Err(EncodeError::RequiredFieldEmpty { .. })
    ));
}

/// Scenario: Constructed register operations violate each self-contained initial-record rule.
/// Guarantees: The encoder cannot emit epoch zero/noninitial epoch, Unspecified identity, continuation state, bad guard shape, or profile version zero.
#[test]
fn encoder_rejects_locally_invalid_register_operations() {
    let base = decode_operation(include_bytes!("fixtures/operation-register-file.bin"))
        .unwrap()
        .0;

    for mutate in [
        |op: &mut otel_arrow_dfe_filelog_checkpoint::RegisterFile| op.file_epoch = 0,
        |op: &mut otel_arrow_dfe_filelog_checkpoint::RegisterFile| op.file_epoch = 2,
        |op: &mut otel_arrow_dfe_filelog_checkpoint::RegisterFile| {
            op.locator = Locator::Unspecified;
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::RegisterFile| {
            op.framing_resume = FramingResume::Continuation {
                record_start_offset: 0,
                record_end_offset: 2,
                next_fragment_index: 1,
            };
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::RegisterFile| {
            op.committed_frontier_guard.window_len = 1;
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::RegisterFile| {
            op.framing_profile_version = 0;
        },
    ] {
        let mut operation = base.clone();
        let Operation::RegisterFile(register) = &mut operation else {
            panic!("register fixture expected");
        };
        mutate(register);
        assert_operation_encode_rejected(operation);
    }
}

/// Scenario: Constructed progress operations move backward or carry inconsistent frontier/finalization state.
/// Guarantees: All progress rules decidable without stored checkpoint state are enforced before bytes are written.
#[test]
fn encoder_rejects_locally_invalid_progress_operations() {
    let base = decode_operation(PROGRESS_OP).unwrap().0;

    let mut operation = base.clone();
    let Operation::UpdateProgress(progress) = &mut operation else {
        panic!("progress fixture expected");
    };
    progress.expected_file_epoch = 0;
    assert_operation_encode_rejected(operation);

    let mut operation = base.clone();
    let Operation::UpdateProgress(progress) = &mut operation else {
        panic!("progress fixture expected");
    };
    progress.expected_committed_offset = progress.new_committed_offset + 1;
    assert_operation_encode_rejected(operation);

    let mut operation = base.clone();
    let Operation::UpdateProgress(progress) = &mut operation else {
        panic!("progress fixture expected");
    };
    progress.new_committed_frontier_guard.window_len -= 1;
    assert_operation_encode_rejected(operation);

    let mut operation = base;
    let Operation::UpdateProgress(progress) = &mut operation else {
        panic!("progress fixture expected");
    };
    progress.new_committed_offset = 2;
    progress.new_committed_frontier_guard = CommittedFrontierGuard::compute(2, b"ab").unwrap();
    progress.new_framing_resume = FramingResume::Continuation {
        record_start_offset: 0,
        record_end_offset: 3,
        next_fragment_index: 1,
    };
    progress.finalize = true;
    assert_operation_encode_rejected(operation);
}

/// Scenario: Constructed truncate-reset and quarantine operations violate locally decidable transition shapes.
/// Guarantees: Nonzero reset offsets, dirty reset resumes, wrong epoch/reason transitions, and invalid quarantine identity/epoch are not encodable.
#[test]
fn encoder_rejects_locally_invalid_reset_and_quarantine_operations() {
    let reset = decode_operation(include_bytes!(
        "fixtures/operation-reset-after-truncate.bin"
    ))
    .unwrap()
    .0;
    for mutate in [
        |op: &mut otel_arrow_dfe_filelog_checkpoint::ResetAfterTruncate| {
            op.expected_active_epoch = 0;
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::ResetAfterTruncate| {
            op.resulting_epoch = 3;
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::ResetAfterTruncate| {
            op.new_committed_offset = 1;
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::ResetAfterTruncate| {
            op.new_framing_resume = FramingResume::Continuation {
                record_start_offset: 0,
                record_end_offset: 2,
                next_fragment_index: 1,
            };
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::ResetAfterTruncate| op.reason_code = 2,
    ] {
        let mut operation = reset.clone();
        let Operation::ResetAfterTruncate(reset) = &mut operation else {
            panic!("truncate reset fixture expected");
        };
        mutate(reset);
        assert_operation_encode_rejected(operation);
    }

    let quarantine = decode_operation(include_bytes!("fixtures/operation-quarantine-file.bin"))
        .unwrap()
        .0;
    for mutate in [
        |op: &mut otel_arrow_dfe_filelog_checkpoint::QuarantineFile| {
            op.expected_file_epoch = 0;
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::QuarantineFile| {
            op.locator = Locator::Unspecified;
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::QuarantineFile| {
            op.quarantine_epoch += 1;
        },
    ] {
        let mut operation = quarantine.clone();
        let Operation::QuarantineFile(quarantine) = &mut operation else {
            panic!("quarantine fixture expected");
        };
        mutate(quarantine);
        assert_operation_encode_rejected(operation);
    }
}

/// Scenario: Constructed reset-quarantined actions violate action-local epoch, offset, guard, or resume rules.
/// Guarantees: Reset-to-beginning/end are locally valid before encoding while keep_failed remains reserved for stored-state comparison in PR1B.
#[test]
fn encoder_rejects_locally_invalid_quarantine_reset_actions() {
    for fixture in [
        include_bytes!("fixtures/operation-reset-quarantined-beginning.bin").as_slice(),
        include_bytes!("fixtures/operation-reset-quarantined-end.bin").as_slice(),
    ] {
        let base = decode_operation(fixture).unwrap().0;

        let mut operation = base.clone();
        let Operation::ResetQuarantinedFile(reset) = &mut operation else {
            panic!("quarantine reset fixture expected");
        };
        reset.expected_quarantine_epoch = 0;
        assert_operation_encode_rejected(operation);

        let mut operation = base.clone();
        let Operation::ResetQuarantinedFile(reset) = &mut operation else {
            panic!("quarantine reset fixture expected");
        };
        reset.resulting_epoch += 1;
        assert_operation_encode_rejected(operation);

        let mut operation = base.clone();
        let Operation::ResetQuarantinedFile(reset) = &mut operation else {
            panic!("quarantine reset fixture expected");
        };
        reset.new_committed_frontier_guard.window_len ^= 1;
        assert_operation_encode_rejected(operation);

        let mut operation = base;
        let Operation::ResetQuarantinedFile(reset) = &mut operation else {
            panic!("quarantine reset fixture expected");
        };
        reset.new_framing_resume = FramingResume::Continuation {
            record_start_offset: 0,
            record_end_offset: reset.resulting_offset.saturating_add(1),
            next_fragment_index: 1,
        };
        assert_operation_encode_rejected(operation);
    }

    let mut beginning = decode_operation(include_bytes!(
        "fixtures/operation-reset-quarantined-beginning.bin"
    ))
    .unwrap()
    .0;
    let Operation::ResetQuarantinedFile(reset) = &mut beginning else {
        panic!("quarantine reset fixture expected");
    };
    reset.resulting_offset = 1;
    assert_operation_encode_rejected(beginning);
}

/// Scenario: Constructed fingerprint, metadata, and removal operations violate local epoch and shape rules.
/// Guarantees: Every non-table-dependent invariant on the remaining operation types is checked by the shared encode path.
#[test]
fn encoder_rejects_other_locally_invalid_operations() {
    let mut fingerprint =
        decode_operation(include_bytes!("fixtures/operation-update-fingerprint.bin"))
            .unwrap()
            .0;
    let Operation::UpdateFingerprint(update) = &mut fingerprint else {
        panic!("fingerprint fixture expected");
    };
    update.expected_file_epoch = 0;
    assert_operation_encode_rejected(fingerprint);

    let mut fingerprint =
        decode_operation(include_bytes!("fixtures/operation-update-fingerprint.bin"))
            .unwrap()
            .0;
    let Operation::UpdateFingerprint(update) = &mut fingerprint else {
        panic!("fingerprint fixture expected");
    };
    update.new_fingerprint = update.expected_fingerprint.clone();
    assert_operation_encode_rejected(fingerprint);

    let metadata = decode_operation(METADATA_OP).unwrap().0;
    for mutate in [
        |op: &mut otel_arrow_dfe_filelog_checkpoint::UpdateMetadata| {
            op.expected_file_epoch = 0;
        },
        |op: &mut otel_arrow_dfe_filelog_checkpoint::UpdateMetadata| {
            op.expected_prior_state = LifecycleState::RotatedFinalized;
        },
    ] {
        let mut operation = metadata.clone();
        let Operation::UpdateMetadata(metadata) = &mut operation else {
            panic!("metadata fixture expected");
        };
        mutate(metadata);
        assert_operation_encode_rejected(operation);
    }

    let mut remove = decode_operation(include_bytes!("fixtures/operation-remove-file.bin"))
        .unwrap()
        .0;
    let Operation::RemoveFile(operation) = &mut remove else {
        panic!("remove fixture expected");
    };
    operation.expected_file_epoch = 0;
    assert_operation_encode_rejected(remove);

    let mut remove = decode_operation(include_bytes!("fixtures/operation-remove-file.bin"))
        .unwrap()
        .0;
    let Operation::RemoveFile(operation) = &mut remove else {
        panic!("remove fixture expected");
    };
    operation.administrative = false;
    assert_operation_encode_rejected(remove);

    let mut non_administrative = decode_operation(include_bytes!(
        "fixtures/operation-remove-file-non-administrative.bin"
    ))
    .unwrap()
    .0;
    let Operation::RemoveFile(operation) = &mut non_administrative else {
        panic!("remove fixture expected");
    };
    operation.expected_prior_state = LifecycleState::Quarantined;
    assert_operation_encode_rejected(non_administrative);
}

/// Scenario: WAL operations carry offset-zero guards whose digest differs from the canonical empty guard.
/// Guarantees: Registration, progress, and quarantine-reset encoding reject the malformed guard while valid keep_failed remains representable.
#[test]
fn wal_encoder_requires_canonical_offset_zero_guards() {
    let mut register = decode_operation(include_bytes!("fixtures/operation-register-file.bin"))
        .unwrap()
        .0;
    let Operation::RegisterFile(operation) = &mut register else {
        panic!("register fixture expected");
    };
    operation.committed_frontier_guard.digest[0] ^= 1;
    assert_operation_encode_rejected(register);

    let mut progress = decode_operation(PROGRESS_OP).unwrap().0;
    let Operation::UpdateProgress(operation) = &mut progress else {
        panic!("progress fixture expected");
    };
    operation.expected_committed_offset = 0;
    operation.new_committed_offset = 0;
    operation.new_committed_frontier_guard = CommittedFrontierGuard::empty();
    operation.new_committed_frontier_guard.digest[0] ^= 1;
    assert_operation_encode_rejected(progress);

    let mut reset = decode_operation(include_bytes!(
        "fixtures/operation-reset-quarantined-beginning.bin"
    ))
    .unwrap()
    .0;
    let Operation::ResetQuarantinedFile(operation) = &mut reset else {
        panic!("quarantine reset fixture expected");
    };
    operation.new_committed_frontier_guard.digest[0] ^= 1;
    assert_operation_encode_rejected(reset);

    let keep_failed = decode_operation(include_bytes!(
        "fixtures/operation-reset-quarantined-keep-failed.bin"
    ))
    .unwrap()
    .0;
    assert!(encode_operation(&keep_failed).is_ok());

    let mut malformed_keep_failed = keep_failed;
    let Operation::ResetQuarantinedFile(operation) = &mut malformed_keep_failed else {
        panic!("quarantine reset fixture expected");
    };
    operation.resulting_offset = 0;
    operation.new_committed_frontier_guard = CommittedFrontierGuard::empty();
    operation.new_committed_frontier_guard.digest[0] ^= 1;
    assert_operation_encode_rejected(malformed_keep_failed);
}

/// Scenario: CRC-valid WAL operation frames carry malformed offset-zero guard digests.
/// Guarantees: Standalone operation decoding and transaction scanning reject noncanonical empty guards before replay.
#[test]
fn wal_decoder_requires_canonical_offset_zero_guards() {
    let mut register = include_bytes!("fixtures/operation-register-file.bin").to_vec();
    register[35] ^= 1;
    refresh_operation_crc(&mut register);
    assert!(matches!(
        decode_operation(&register),
        Err(DecodeError::InvalidCommittedFrontierGuard { .. })
    ));
    let transaction = transaction_from_operations(&[&register]);
    assert!(matches!(
        scan_all_for_test(&transaction),
        Err(DecodeError::InvalidCommittedFrontierGuard { .. })
    ));

    let mut progress = PROGRESS_OP.to_vec();
    put_u64(&mut progress, 33, 0);
    put_u16(&mut progress, 41, 0);
    progress[43] ^= 1;
    refresh_operation_crc(&mut progress);
    assert!(matches!(
        decode_operation(&progress),
        Err(DecodeError::InvalidCommittedFrontierGuard { .. })
    ));

    let mut reset = include_bytes!("fixtures/operation-reset-quarantined-beginning.bin").to_vec();
    reset[40] ^= 1;
    refresh_operation_crc(&mut reset);
    assert!(matches!(
        decode_operation(&reset),
        Err(DecodeError::InvalidCommittedFrontierGuard { .. })
    ));

    let mut keep_failed =
        include_bytes!("fixtures/operation-reset-quarantined-keep-failed.bin").to_vec();
    put_u64(&mut keep_failed, 30, 0);
    put_u16(&mut keep_failed, 38, 0);
    keep_failed[40] ^= 1;
    refresh_operation_crc(&mut keep_failed);
    assert!(matches!(
        decode_operation(&keep_failed),
        Err(DecodeError::InvalidCommittedFrontierGuard { .. })
    ));
}

/// Scenario: Advisory-path bytes contain an unknown kind inside an otherwise CRC-valid record.
/// Guarantees: Malformed path representations are rejected after bounded record parsing.
#[test]
fn malformed_advisory_path_is_rejected() {
    let mut frame = active_record_frame();
    frame[149] = 9;
    refresh_record_crc(&mut frame);
    let namespace = namespace_digest("app-logs").unwrap();
    assert!(matches!(
        decode_snapshot(&snapshot_from_frames(&[frame]), &namespace, u32::MAX),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));
}

/// Scenario: CRC-valid snapshot advisory paths use reserved flags or a noncanonical Unavailable shape.
/// Guarantees: Reserved bits and both nonzero Unavailable flag/length forms fail in the actual record decoder.
#[test]
fn advisory_path_flags_and_unavailable_shape_are_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let unix = include_bytes!("fixtures/advisory-unix.bin");

    let mut reserved = unix.to_vec();
    reserved[1] = 0x02;
    assert!(matches!(
        decode_snapshot(&snapshot_with_active_advisory(&reserved), &namespace, 1),
        Err(DecodeError::ReservedFieldNonZero { .. })
    ));

    let mut unavailable_flags = vec![0, 1];
    unavailable_flags.extend_from_slice(&0u64.to_be_bytes());
    unavailable_flags.extend_from_slice(&0u16.to_be_bytes());
    unavailable_flags.extend_from_slice(&[0; 32]);
    assert!(matches!(
        decode_snapshot(
            &snapshot_with_active_advisory(&unavailable_flags),
            &namespace,
            1
        ),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));

    let mut unavailable_length = unavailable_flags;
    unavailable_length[1] = 0;
    put_u64(&mut unavailable_length, 2, 1);
    assert!(matches!(
        decode_snapshot(
            &snapshot_with_active_advisory(&unavailable_length),
            &namespace,
            1
        ),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));
}

/// Scenario: CRC-valid Unix advisory paths carry inconsistent complete/truncated lengths or a wrong complete digest.
/// Guarantees: Every decoder-only length-selection arm and recomputable digest check fails closed inside a snapshot record.
#[test]
fn advisory_path_unix_length_and_digest_rules_are_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let unix = include_bytes!("fixtures/advisory-unix.bin");

    let mut empty_present = unix.to_vec();
    put_u64(&mut empty_present, 2, 0);
    assert!(matches!(
        decode_snapshot(
            &snapshot_with_active_advisory(&empty_present),
            &namespace,
            1
        ),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));

    let mut incomplete = unix.to_vec();
    put_u16(&mut incomplete, 10, 15);
    assert_eq!(incomplete.remove(12), b'/');
    assert!(matches!(
        decode_snapshot(&snapshot_with_active_advisory(&incomplete), &namespace, 1),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));

    let mut short_truncated = include_bytes!("fixtures/advisory-long-truncated.bin").to_vec();
    put_u64(&mut short_truncated, 2, 4096);
    assert!(matches!(
        decode_snapshot(
            &snapshot_with_active_advisory(&short_truncated),
            &namespace,
            1
        ),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));

    let mut wrong_suffix_len = include_bytes!("fixtures/advisory-long-truncated.bin").to_vec();
    put_u16(&mut wrong_suffix_len, 10, 4095);
    assert_eq!(wrong_suffix_len.remove(12), b'x');
    assert!(matches!(
        decode_snapshot(
            &snapshot_with_active_advisory(&wrong_suffix_len),
            &namespace,
            1
        ),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));

    let mut wrong_digest = unix.to_vec();
    let last = wrong_digest.len() - 1;
    wrong_digest[last] ^= 1;
    assert!(matches!(
        decode_snapshot(&snapshot_with_active_advisory(&wrong_digest), &namespace, 1),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));
}

/// Scenario: CRC-valid Windows advisory paths carry odd complete or stored native-byte lengths.
/// Guarantees: UTF-16LE alignment is validated independently for both length fields by the containing snapshot decoder.
#[test]
fn advisory_path_windows_alignment_is_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let windows = include_bytes!("fixtures/advisory-windows-utf16le.bin");

    let mut odd_full = windows.to_vec();
    let full = u64::from_be_bytes(odd_full[2..10].try_into().unwrap());
    put_u64(&mut odd_full, 2, full - 1);
    assert!(matches!(
        decode_snapshot(&snapshot_with_active_advisory(&odd_full), &namespace, 1),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));

    let mut odd_stored = windows.to_vec();
    let stored = u16::from_be_bytes(odd_stored[10..12].try_into().unwrap());
    put_u16(&mut odd_stored, 10, stored - 1);
    assert_eq!(odd_stored.remove(12), b'C');
    assert!(matches!(
        decode_snapshot(&snapshot_with_active_advisory(&odd_stored), &namespace, 1),
        Err(DecodeError::InvalidAdvisoryPath { .. })
    ));
}

/// Scenario: Snapshot record/footer lengths or count echoes disagree under valid local CRCs.
/// Guarantees: Self-delimiting records consume exactly their payload and footer summaries match the file.
#[test]
fn snapshot_unconsumed_record_and_footer_mismatches_are_rejected() {
    let namespace = namespace_digest("app-logs").unwrap();
    let frame = active_record_frame();
    let mut payload = frame[4..frame.len() - 4].to_vec();
    payload.push(0);
    let extended = resize_record_payload(frame, payload);
    assert!(matches!(
        decode_snapshot(&snapshot_from_frames(&[extended]), &namespace, u32::MAX),
        Err(DecodeError::UnconsumedBytes { .. })
    ));

    for (offset, value) in [(281, 0u64), (289, 2u64)] {
        let mut bytes = ACTIVE_SNAPSHOT.to_vec();
        if offset == 281 {
            put_u64(&mut bytes, offset, value);
        } else {
            put_u32(&mut bytes, offset, value as u32);
        }
        let footer_start = bytes.len() - 24;
        let checksum = crc32c(&bytes[footer_start..footer_start + 20]);
        let crc_offset = bytes.len() - 4;
        put_u32(&mut bytes, crc_offset, checksum);
        assert!(matches!(
            decode_snapshot(&bytes, &namespace, u32::MAX),
            Err(DecodeError::UnconsumedBytes { .. })
        ));
    }
}

/// Scenario: A snapshot stores a nonzero future framing-profile structural version.
/// Guarantees: PR1A preserves opaque nonzero versions for later compatibility comparison instead of guessing support.
#[test]
fn future_framing_profile_version_is_structurally_preserved() {
    let mut frame = active_record_frame();
    put_u16(&mut frame, 105, 2);
    refresh_record_crc(&mut frame);
    let namespace = namespace_digest("app-logs").unwrap();
    let snapshot = decode_snapshot(&snapshot_from_frames(&[frame]), &namespace, u32::MAX).unwrap();
    assert_eq!(snapshot.records[0].framing_profile_version, 2);
}

/// Scenario: WAL header magic, version, flags, and checksum are corrupted.
/// Guarantees: Every fixed WAL header field is validated before transaction scanning.
#[test]
fn wal_header_corruption_is_rejected() {
    let fixture = include_bytes!("fixtures/wal-header.bin");
    assert!(matches!(
        decode_wal_header(&fixture[..55]),
        Err(DecodeError::InvalidLength { .. })
    ));
    for (offset, expected) in [(0, "magic"), (8, "version"), (10, "flags"), (55, "crc")] {
        let mut bytes = fixture.to_vec();
        bytes[offset] ^= 1;
        let error = decode_wal_header(&bytes).unwrap_err();
        match expected {
            "magic" => assert!(matches!(error, DecodeError::BadMagic { .. })),
            "version" => assert!(matches!(error, DecodeError::UnsupportedVersion { .. })),
            "flags" => assert!(matches!(error, DecodeError::ReservedFieldNonZero { .. })),
            _ => assert!(matches!(error, DecodeError::ChecksumMismatch { .. })),
        }
    }
}

/// Scenario: Operation framing has an excessive length, missing declared bytes, or bad CRC.
/// Guarantees: Operation bounds precede slicing and complete corrupt frames are never salvaged.
#[test]
fn operation_length_and_crc_corruption_is_rejected() {
    let mut excessive = Vec::from((131_096u32).to_be_bytes());
    assert!(matches!(
        decode_operation(&excessive),
        Err(DecodeError::LengthExceedsMaximum { .. })
    ));
    put_u32(&mut excessive, 0, 100);
    assert!(matches!(
        decode_operation(&excessive),
        Err(DecodeError::Truncated { .. })
    ));
    let mut bad_crc = PROGRESS_OP.to_vec();
    let last = bad_crc.len() - 1;
    bad_crc[last] ^= 1;
    assert!(matches!(
        decode_operation(&bad_crc),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
}

/// Scenario: An operation has an unknown code or invalid boolean discriminant under a valid CRC.
/// Guarantees: Operation code and boolean spaces are closed rather than interpreted by truthiness.
#[test]
fn operation_discriminants_are_rejected() {
    let mut unknown = PROGRESS_OP.to_vec();
    unknown[4] = 0xFF;
    refresh_operation_crc(&mut unknown);
    assert!(matches!(
        decode_operation(&unknown),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));
    let mut invalid_bool = PROGRESS_OP.to_vec();
    let bool_offset = invalid_bool.len() - 5;
    invalid_bool[bool_offset] = 2;
    refresh_operation_crc(&mut invalid_bool);
    assert!(matches!(
        decode_operation(&invalid_bool),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));
}

/// Scenario: Nested resume, action, lifecycle, and presence discriminants are outside v1.
/// Guarantees: Every operation-local tagged value and reserved bit is closed under valid frame CRCs.
#[test]
fn nested_operation_discriminants_are_rejected() {
    let mut resume = PROGRESS_OP.to_vec();
    resume[75] = 9;
    refresh_operation_crc(&mut resume);
    assert!(matches!(
        decode_operation(&resume),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));

    let mut action =
        include_bytes!("fixtures/operation-reset-quarantined-keep-failed.bin").to_vec();
    action[25] = 9;
    refresh_operation_crc(&mut action);
    assert!(matches!(
        decode_operation(&action),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));

    let mut lifecycle = include_bytes!("fixtures/operation-remove-file.bin").to_vec();
    lifecycle[25] = 0;
    refresh_operation_crc(&mut lifecycle);
    assert!(matches!(
        decode_operation(&lifecycle),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));

    let mut presence = METADATA_OP.to_vec();
    presence[26] = 2;
    refresh_operation_crc(&mut presence);
    assert!(matches!(
        decode_operation(&presence),
        Err(DecodeError::ReservedFieldNonZero { .. })
    ));

    let mut metadata_state = METADATA_OP.to_vec();
    metadata_state[21] = 0;
    refresh_operation_crc(&mut metadata_state);
    assert!(matches!(
        decode_operation(&metadata_state),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));
}

/// Scenario: An operation payload has a valid CRC but carries one undefined extension byte.
/// Guarantees: Version 1 consumes every length-delimited payload byte and cannot silently ignore extensions.
#[test]
fn operation_unconsumed_extension_is_rejected() {
    let payload = &PROGRESS_OP[4..PROGRESS_OP.len() - 4];
    let mut extended = payload.to_vec();
    extended.push(0);
    assert!(matches!(
        decode_operation(&operation_from_payload(&extended)),
        Err(DecodeError::UnconsumedBytes { .. })
    ));
}

/// Scenario: Both fingerprint length fields use their full u16 structural range.
/// Guarantees: The 131,095-byte structural operation maximum is accepted without confusing it with the 131,094-byte semantic maximum.
#[test]
fn structural_maximum_fingerprint_operation_is_bounded_and_decodable() {
    let mut payload = Vec::with_capacity(131_095);
    payload.push(4);
    payload.extend_from_slice(&[0x22; 16]);
    payload.extend_from_slice(&1u32.to_be_bytes());
    payload.extend_from_slice(&u16::MAX.to_be_bytes());
    payload.extend_from_slice(&vec![0xAA; usize::from(u16::MAX)]);
    payload.extend_from_slice(&u16::MAX.to_be_bytes());
    payload.extend_from_slice(&vec![0xBB; usize::from(u16::MAX)]);
    assert_eq!(payload.len(), 131_095);
    let bytes = operation_from_payload(&payload);
    assert!(matches!(
        decode_operation(&bytes),
        Ok((Operation::UpdateFingerprint(_), _))
    ));
}

/// Scenario: Transaction headers carry bad complements, flags, versions, CRCs, or body bounds.
/// Guarantees: A complete header is fully validated before its declared body can be called incomplete.
#[test]
fn transaction_header_corruption_is_rejected_before_incomplete() {
    let mut bytes = MIN_TX.to_vec();
    bytes[24] ^= 1;
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::LengthComplementMismatch { .. })
    ));
    let mut bytes = MIN_TX.to_vec();
    put_u16(&mut bytes, 10, 1);
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::ReservedFieldNonZero { .. })
    ));
    let mut bytes = MIN_TX.to_vec();
    put_u16(&mut bytes, 30, 1);
    refresh_transaction_crcs(&mut bytes);
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::ReservedFieldNonZero { .. })
    ));
    let mut bytes = MIN_TX.to_vec();
    put_u16(&mut bytes, 8, 2);
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::UnsupportedVersion { .. })
    ));
    let mut bytes = MIN_TX.to_vec();
    bytes[35] ^= 1;
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
    for body_len in [33u32, 16 * 1024 * 1024 + 1] {
        let mut bytes = MIN_TX.to_vec();
        put_u32(&mut bytes, 20, body_len);
        put_u32(&mut bytes, 24, !body_len);
        refresh_transaction_crcs(&mut bytes);
        assert!(matches!(
            scan_all_for_test(&bytes),
            Err(DecodeError::TransactionBodyOutOfBounds { .. })
        ));
    }
}

/// Scenario: One header carries multiple faults that are repaired in the
/// normative validation order.
/// Guarantees: Reserved fields precede complement/CRC checks, body bounds
/// precede operation counts, and operation counts precede sequence ordering.
#[test]
fn transaction_header_validation_precedence_matches_the_format() {
    let mut bytes = MIN_TX.to_vec();
    put_u64(&mut bytes, 12, 2);
    put_u32(&mut bytes, 20, TX_MIN_BODY_BYTES as u32 - 1);
    put_u32(&mut bytes, 24, 0);
    put_u16(&mut bytes, 28, 0);
    put_u16(&mut bytes, 30, 1);
    assert_eq!(
        scan_next_transaction(&bytes, 1),
        Err(DecodeError::ReservedFieldNonZero {
            field: "wal_transaction.reserved",
            value: 1,
        })
    );

    put_u16(&mut bytes, 30, 0);
    put_u32(&mut bytes, 24, !(TX_MIN_BODY_BYTES as u32 - 1));
    refresh_transaction_crcs(&mut bytes);
    assert!(matches!(
        scan_next_transaction(&bytes, 1),
        Err(DecodeError::TransactionBodyOutOfBounds { .. })
    ));

    put_u32(&mut bytes, 20, TX_MIN_BODY_BYTES as u32);
    put_u32(&mut bytes, 24, !(TX_MIN_BODY_BYTES as u32));
    refresh_transaction_crcs(&mut bytes);
    assert_eq!(
        scan_next_transaction(&bytes, 1),
        Err(DecodeError::EmptyTransaction { sequence: 2 })
    );

    put_u16(&mut bytes, 28, 1);
    refresh_transaction_crcs(&mut bytes);
    assert_eq!(
        scan_next_transaction(&bytes, 1),
        Err(DecodeError::SequenceOutOfOrder {
            expected: 1,
            found: 2,
        })
    );
}

/// Scenario: The supplied suffix is shorter than a header or shorter than a
/// valid header's declared frame.
/// Guarantees: The codec reports exact incomplete bytes without claiming the
/// slice reaches physical EOF or authorizing truncation.
#[test]
fn incomplete_transaction_suffix_is_reported_without_eof_claim() {
    assert_eq!(
        scan_next_transaction(&MIN_TX[..35], 1),
        Ok(Some(TransactionScan::Incomplete { bytes: 35 }))
    );
    let missing_frame_crc = scan_all_for_test(&MIN_TX[..MIN_TX.len() - 1]).unwrap();
    assert_eq!(missing_frame_crc.incomplete_bytes, MIN_TX.len() - 1);
    assert!(missing_frame_crc.transactions.is_empty());
}

/// Scenario: A valid sequence-1 transaction precedes an incomplete or
/// complete-corrupt sequence-2 frame in the supplied slice.
/// Guarantees: Incremental scanning returns the first transaction, reports the
/// exact remaining incomplete bytes, and still fails on complete corruption.
#[test]
fn incomplete_suffix_after_valid_transaction_is_reported_incrementally() {
    let mut second = MIN_TX.to_vec();
    put_u64(&mut second, 12, 2);
    refresh_transaction_crcs(&mut second);

    let mut wal = MIN_TX.to_vec();
    wal.extend_from_slice(&second[..second.len() - 1]);
    let Some(TransactionScan::Complete {
        transaction,
        consumed,
    }) = scan_next_transaction(&wal, 1).unwrap()
    else {
        panic!("first transaction must be complete");
    };
    assert_eq!(transaction.sequence, 1);
    let suffix = &wal[consumed..];
    drop(transaction);
    let Some(TransactionScan::Incomplete { bytes }) = scan_next_transaction(suffix, 2).unwrap()
    else {
        panic!("second transaction must be incomplete");
    };
    assert_eq!(bytes, suffix.len());
    assert_eq!(bytes, second.len() - 1);

    let mut corrupt_second = second;
    let last = corrupt_second.len() - 1;
    corrupt_second[last] ^= 1;
    let mut corrupt_wal = MIN_TX.to_vec();
    corrupt_wal.extend_from_slice(&corrupt_second);
    let Some(TransactionScan::Complete { consumed, .. }) =
        scan_next_transaction(&corrupt_wal, 1).unwrap()
    else {
        panic!("first transaction must remain complete");
    };
    assert!(matches!(
        scan_next_transaction(&corrupt_wal[consumed..], 2),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
}

/// Scenario: A CRC-valid out-of-sequence header is followed by an incomplete body.
/// Guarantees: Sequence corruption is reported before incomplete-suffix classification.
#[test]
fn wrong_sequence_header_with_incomplete_body_is_corruption() {
    let mut bytes = MIN_TX.to_vec();
    put_u64(&mut bytes, 12, 2);
    refresh_transaction_crcs(&mut bytes);
    bytes.truncate(40);
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::SequenceOutOfOrder { .. })
    ));
}

/// Scenario: A complete final transaction has a bad frame CRC or a bad enclosed operation CRC.
/// Guarantees: Complete corruption is never reported as an incomplete slice.
#[test]
fn complete_bad_crc_transaction_is_corruption() {
    let mut frame_crc = MIN_TX.to_vec();
    let last = frame_crc.len() - 1;
    frame_crc[last] ^= 1;
    assert!(matches!(
        scan_all_for_test(&frame_crc),
        Err(DecodeError::ChecksumMismatch { .. })
    ));

    let mut operation_crc = MIN_TX.to_vec();
    operation_crc[MIN_TX.len() - 5] ^= 1;
    let frame_crc_offset = operation_crc.len() - 4;
    let checksum = crc32c(&operation_crc[..frame_crc_offset]);
    put_u32(&mut operation_crc, frame_crc_offset, checksum);
    assert!(matches!(
        scan_all_for_test(&operation_crc),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
}

/// Scenario: First or later transaction sequence is zero, gapped, or repeated.
/// Guarantees: The scanner owns strict one-based sequence structure across the complete suffix.
#[test]
fn transaction_sequence_zero_gap_and_repeat_are_rejected() {
    for sequence in [0u64, 2] {
        let mut bytes = MIN_TX.to_vec();
        put_u64(&mut bytes, 12, sequence);
        refresh_transaction_crcs(&mut bytes);
        assert!(matches!(
            scan_all_for_test(&bytes),
            Err(DecodeError::SequenceOutOfOrder { .. })
        ));
    }
    let mut repeated = MIN_TX.to_vec();
    repeated.extend_from_slice(MIN_TX);
    assert!(matches!(
        scan_all_for_test(&repeated),
        Err(DecodeError::SequenceOutOfOrder { .. })
    ));
}

/// Scenario: A CRC-valid sequence-zero transaction is scanned with caller-expected sequence zero.
/// Guarantees: Neither a caller-provided zero nor a stored zero can bypass the one-based WAL sequence contract.
#[test]
fn direct_scanner_rejects_zero_expected_and_stored_sequence() {
    let mut bytes = MIN_TX.to_vec();
    put_u64(&mut bytes, 12, 0);
    refresh_transaction_crcs(&mut bytes);
    assert_eq!(
        scan_next_transaction(&bytes, 0),
        Err(DecodeError::ExpectedSequenceZero)
    );
    assert_eq!(
        scan_next_transaction(&[], 0),
        Err(DecodeError::ExpectedSequenceZero)
    );
    assert!(matches!(
        scan_next_transaction(&bytes, 1),
        Err(DecodeError::SequenceOutOfOrder {
            expected: 1,
            found: 0,
        })
    ));
}

/// Scenario: A transaction declares zero operations or mixes progress and non-progress operations.
/// Guarantees: Atomic transaction classes cannot be empty or cross the progress/non-progress boundary.
#[test]
fn empty_and_mixed_transactions_are_rejected() {
    let mut empty = MIN_TX.to_vec();
    put_u16(&mut empty, 28, 0);
    refresh_transaction_crcs(&mut empty);
    assert!(matches!(
        scan_all_for_test(&empty),
        Err(DecodeError::EmptyTransaction { .. })
    ));
    let mixed = transaction_from_operations(&[PROGRESS_OP, METADATA_OP]);
    assert!(matches!(
        scan_all_for_test(&mixed),
        Err(DecodeError::MixedTransactionClass { .. })
    ));
}

/// Scenario: A progress-only transaction repeats one file key.
/// Guarantees: Duplicate progress keys are rejected before a consumer can apply ambiguous updates.
#[test]
fn duplicate_progress_file_id_is_rejected() {
    let duplicate = transaction_from_operations(&[PROGRESS_OP, PROGRESS_OP]);
    assert!(matches!(
        scan_all_for_test(&duplicate),
        Err(DecodeError::DuplicateProgressFileId { .. })
    ));
}

/// Scenario: A non-progress transaction contains 257 individually valid operations.
/// Guarantees: The tighter non-progress count limit is enforced after class determination.
#[test]
fn non_progress_operation_count_boundary_is_rejected() {
    let operations: Vec<&[u8]> = (0..257).map(|_| METADATA_OP).collect();
    let bytes = transaction_from_operations(&operations);
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::TooManyOperations { max: 256, .. })
    ));
}

/// Scenario: A transaction body contains two operations while op_count declares one.
/// Guarantees: Transaction bodies must be consumed exactly and cannot hide uncounted operations.
#[test]
fn transaction_body_and_operation_count_must_agree() {
    let mut bytes = transaction_from_operations(&[METADATA_OP, METADATA_OP]);
    put_u16(&mut bytes, 28, 1);
    refresh_transaction_crcs(&mut bytes);
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::UnconsumedBytes { .. })
    ));
}

/// Scenario: A transaction declares or attempts to encode a 4,097th progress operation.
/// Guarantees: Both decoder and encoder reject the operation-count boundary before application.
#[test]
fn progress_operation_count_boundary_is_rejected() {
    let max_fixture = include_bytes!("fixtures/transaction-max-progress.bin");
    let mut declared = max_fixture.to_vec();
    put_u16(&mut declared, 28, WAL_MAX_OPS_PER_TX + 1);
    refresh_transaction_crcs(&mut declared);
    assert!(matches!(
        scan_all_for_test(&declared),
        Err(DecodeError::TooManyOperations { .. })
    ));

    let mut transaction = scan_all_for_test(max_fixture)
        .unwrap()
        .transactions
        .remove(0);
    let mut extra = transaction.operations[0].clone();
    let Operation::UpdateProgress(progress) = &mut extra else {
        panic!("boundary fixture must be progress-only");
    };
    progress.file_id = FileId::from_bytes([0xFF; 16]);
    transaction.operations.push(extra);
    assert!(matches!(
        encode_transaction(&transaction),
        Err(EncodeError::TooManyOperations { .. })
    ));
}

/// Scenario: A complete transaction encloses an unknown operation with recomputed outer CRC.
/// Guarantees: Outer integrity cannot make an unsupported operation code acceptable.
#[test]
fn unknown_operation_in_complete_transaction_is_rejected() {
    let mut operation = PROGRESS_OP.to_vec();
    operation[4] = 0xFE;
    refresh_operation_crc(&mut operation);
    let bytes = transaction_from_operations(&[&operation]);
    assert!(matches!(
        scan_all_for_test(&bytes),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));
}

/// Scenario: Constructed transactions have no operations or mix transaction classes.
/// Guarantees: Encoder-side validation mirrors structural class rules before writing bytes.
#[test]
fn encoder_rejects_empty_and_mixed_transactions() {
    let empty = Transaction {
        sequence: 1,
        operations: Vec::new(),
    };
    assert!(matches!(
        encode_transaction(&empty),
        Err(EncodeError::EmptyTransaction { .. })
    ));
    let progress = decode_operation(PROGRESS_OP).unwrap().0;
    let metadata = decode_operation(METADATA_OP).unwrap().0;
    let mixed = Transaction {
        sequence: 1,
        operations: vec![progress, metadata],
    };
    assert!(matches!(
        encode_transaction(&mixed),
        Err(EncodeError::MixedTransactionClass { .. })
    ));
}

/// Scenario: A malformed metadata operation carries an unknown advisory-path kind under a valid CRC.
/// Guarantees: Nested variable structures retain closed discriminants within operation framing.
#[test]
fn malformed_operation_advisory_path_is_rejected() {
    let mut operation = METADATA_OP.to_vec();
    operation[35] = 9;
    refresh_operation_crc(&mut operation);
    assert!(matches!(
        decode_operation(&operation),
        Err(DecodeError::UnknownDiscriminant { .. })
    ));
}

/// Scenario: Administrative string fields are forbidden, missing, or invalid UTF-8.
/// Guarantees: Presence coupling, required nonempty audit data, and UTF-8 validation are structural.
#[test]
fn administrative_string_shapes_are_rejected() {
    let remove = include_bytes!("fixtures/operation-remove-file.bin");
    let mut forbidden = remove.to_vec();
    forbidden[36] = 0;
    refresh_operation_crc(&mut forbidden);
    assert!(matches!(
        decode_operation(&forbidden),
        Err(DecodeError::UnexpectedPresentField { .. })
    ));

    let mut invalid_utf8 = remove.to_vec();
    invalid_utf8[39] = 0xFF;
    refresh_operation_crc(&mut invalid_utf8);
    assert!(matches!(
        decode_operation(&invalid_utf8),
        Err(DecodeError::InvalidUtf8 { .. })
    ));

    let keep = include_bytes!("fixtures/operation-reset-quarantined-keep-failed.bin");
    let payload = &keep[4..keep.len() - 4];
    let mut empty_audit = payload[..105].to_vec();
    empty_audit.extend_from_slice(&0u16.to_be_bytes());
    assert!(matches!(
        decode_operation(&operation_from_payload(&empty_audit)),
        Err(DecodeError::EmptyRequiredField { .. })
    ));
}

/// Scenario: A finalized snapshot record is changed to a continuation resume in memory.
/// Guarantees: Finalized records remain clean even when all fields are otherwise representable.
#[test]
fn finalized_snapshot_requires_clean_resume() {
    let fixture = include_bytes!("fixtures/snapshot-rotated-finalized.bin");
    let namespace = namespace_digest("app-logs").unwrap();
    let mut record: SnapshotRecord = decode_snapshot(fixture, &namespace, u32::MAX)
        .unwrap()
        .records
        .remove(0);
    record.framing_resume = FramingResume::Continuation {
        record_start_offset: 0,
        record_end_offset: 2,
        next_fragment_index: 1,
    };
    record.committed_offset = 1;
    record.committed_frontier_guard = CommittedFrontierGuard::compute(1, b"x").unwrap();
    assert!(matches!(
        encode_snapshot(7, "app-logs", &[record]),
        Err(EncodeError::InvalidSnapshotState { .. })
    ));
}

/// Scenario: A decoded active fixture is assigned Unspecified in memory.
/// Guarantees: Snapshot encoder and decoder apply the same non-Unspecified durable locator invariant.
#[test]
fn snapshot_encoder_rejects_unspecified_locator() {
    let namespace = namespace_digest("app-logs").unwrap();
    let mut record = decode_snapshot(ACTIVE_SNAPSHOT, &namespace, u32::MAX)
        .unwrap()
        .records
        .remove(0);
    record.locator = Locator::Unspecified;
    assert!(matches!(
        encode_snapshot(7, "app-logs", &[record]),
        Err(EncodeError::InvalidSnapshotState { .. })
    ));
}

/// Scenario: Snapshot bytes are valid but belong to a different namespace.
/// Guarantees: Namespace binding fails before any record can be consumed by the caller.
#[test]
fn snapshot_namespace_mismatch_is_rejected() {
    let other = namespace_digest("other").unwrap();
    assert!(matches!(
        decode_snapshot(ACTIVE_SNAPSHOT, &other, u32::MAX),
        Err(DecodeError::NamespaceMismatch { .. })
    ));
}
