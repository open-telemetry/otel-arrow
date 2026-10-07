// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Single-file container conformance and section-isolation tests.

use otel_arrow_dfe_filelog_checkpoint::{
    CHECKPOINT_HEADER_BYTES, DecodeError, TransactionScan, crc32c, decode_checkpoint_header,
    decode_checkpoint_snapshot, encode_checkpoint, encode_transaction, namespace_digest,
    scan_next_transaction,
};

const EMPTY: &[u8] = include_bytes!("fixtures/checkpoint-empty.bin");
const ACTIVE: &[u8] = include_bytes!("fixtures/checkpoint-active.bin");
const WITH_WAL: &[u8] = include_bytes!("fixtures/checkpoint-with-wal.bin");
const SNAPSHOT: &[u8] = include_bytes!("fixtures/snapshot-active.bin");

fn set_offset(bytes: &mut [u8], offset: u64) {
    bytes[12..20].copy_from_slice(&offset.to_be_bytes());
    let crc = crc32c(&bytes[..20]);
    bytes[20..24].copy_from_slice(&crc.to_be_bytes());
}

/// Scenario: Independent empty and active containers carry snapshots and no WAL transactions.
/// Guarantees: Header layout, absolute offsets, snapshot bytes, generation, and encoding agree with the independent producer.
#[test]
fn checkpoint_fixtures_match_codec() {
    let namespace = namespace_digest("app-logs").unwrap();
    assert_eq!(CHECKPOINT_HEADER_BYTES, 24);
    assert_eq!(EMPTY.len(), 108);
    for (bytes, generation, count) in [(EMPTY, 0, 0), (ACTIVE, 7, 1)] {
        let header = decode_checkpoint_header(&bytes[..24], (bytes.len() - 24) as u64).unwrap();
        assert_eq!(header.wal_offset, bytes.len() as u64);
        let (snapshot, wal_offset) =
            decode_checkpoint_snapshot(bytes, &namespace, count, (bytes.len() - 24) as u64)
                .unwrap();
        assert_eq!(snapshot.generation, generation);
        assert_eq!(snapshot.records.len(), count as usize);
        assert_eq!(wal_offset, bytes.len());
        assert_eq!(
            encode_checkpoint(generation, "app-logs", &snapshot.records).unwrap(),
            bytes
        );
        assert_eq!(scan_next_transaction(&bytes[wal_offset..], 1), Ok(None));
    }
    assert_eq!(&ACTIVE[24..], SNAPSHOT);
}

/// Scenario: A complete container includes an independently encoded progress transaction after its snapshot.
/// Guarantees: Snapshot decoding stops exactly at the WAL boundary; transaction decoding and re-encoding preserve the remaining bytes.
#[test]
fn checkpoint_with_wal_preserves_section_boundary() {
    let namespace = namespace_digest("app-logs").unwrap();
    let (snapshot, offset) =
        decode_checkpoint_snapshot(WITH_WAL, &namespace, 1, SNAPSHOT.len() as u64).unwrap();
    assert_eq!(offset, ACTIVE.len());
    assert_eq!(&WITH_WAL[..offset], ACTIVE);
    let Some(TransactionScan::Complete {
        transaction,
        consumed,
    }) = scan_next_transaction(&WITH_WAL[offset..], 1).unwrap()
    else {
        panic!("complete transaction expected")
    };
    assert_eq!(offset + consumed, WITH_WAL.len());
    assert_eq!(transaction.sequence, 1);
    assert_eq!(transaction.operations.len(), 1);
    assert_eq!(
        transaction.operations[0].file_id(),
        snapshot.records[0].file_id
    );
    let mut encoded =
        encode_checkpoint(snapshot.generation, "app-logs", &snapshot.records).unwrap();
    encoded.extend(encode_transaction(&transaction).unwrap());
    assert_eq!(encoded, WITH_WAL);
    // A bounded store need only supply the immutable prefix for this stage.
    assert_eq!(
        decode_checkpoint_snapshot(&WITH_WAL[..offset], &namespace, 1, SNAPSHOT.len() as u64)
            .unwrap()
            .0,
        snapshot
    );
}

/// Scenario: Container headers are short, overlong, unsupported, reserved, or CRC-corrupt.
/// Guarantees: A damaged boundary never reaches snapshot parsing, and old separate-file magic is rejected.
#[test]
fn checkpoint_header_envelope_is_closed() {
    for len in 0..24 {
        assert!(matches!(
            decode_checkpoint_header(&EMPTY[..len], 84),
            Err(DecodeError::InvalidLength { .. })
        ));
    }
    assert!(matches!(
        decode_checkpoint_header(&EMPTY[..25], 84),
        Err(DecodeError::InvalidLength { .. })
    ));
    for (offset, kind) in [(0, 0), (8, 1), (10, 2), (12, 3), (23, 3)] {
        let mut header = EMPTY[..24].to_vec();
        header[offset] ^= 1;
        let error = decode_checkpoint_header(&header, 84).unwrap_err();
        match kind {
            0 => assert!(matches!(error, DecodeError::BadMagic { .. })),
            1 => assert!(matches!(error, DecodeError::UnsupportedVersion { .. })),
            2 => assert!(matches!(error, DecodeError::ReservedFieldNonZero { .. })),
            _ => assert!(matches!(error, DecodeError::ChecksumMismatch { .. })),
        }
    }
    for magic in [b"FLOGCUR\0", b"FLOGWAL\0", b"FLOGSNP\0"] {
        let mut header = EMPTY[..24].to_vec();
        header[..8].copy_from_slice(magic);
        assert!(matches!(
            decode_checkpoint_header(&header, 84),
            Err(DecodeError::BadMagic { .. })
        ));
    }
}

/// Scenario: CRC-valid offsets point before the minimum snapshot end or exceed the caller's byte budget.
/// Guarantees: Header validation rejects undersized and excessive sections before reading or allocating a snapshot.
#[test]
fn checkpoint_offsets_are_bounded_before_snapshot_reads() {
    let mut header = EMPTY[..24].to_vec();
    for offset in [0, 23, 24, 107] {
        set_offset(&mut header, offset);
        assert_eq!(
            decode_checkpoint_header(&header, u64::MAX),
            Err(DecodeError::InvalidWalOffset { found: offset })
        );
    }
    set_offset(&mut header, 108);
    assert_eq!(
        decode_checkpoint_header(&header, 84).unwrap().wal_offset,
        108
    );
    assert!(matches!(
        decode_checkpoint_header(&header, 83),
        Err(DecodeError::LengthExceedsMaximum {
            declared: 84,
            max: 83,
            ..
        })
    ));
    set_offset(&mut header, u64::MAX);
    assert!(matches!(
        decode_checkpoint_header(&header, 84),
        Err(DecodeError::LengthExceedsMaximum { .. })
    ));
}

/// Scenario: The supplied input ends anywhere inside a valid checkpoint header or snapshot section.
/// Guarantees: No incomplete immutable prefix is mistaken for a discardable WAL tail.
#[test]
fn checkpoint_prefix_truncation_is_never_a_wal_tail() {
    let namespace = namespace_digest("app-logs").unwrap();
    for len in 0..ACTIVE.len() {
        assert!(matches!(
            decode_checkpoint_snapshot(&ACTIVE[..len], &namespace, 1, SNAPSHOT.len() as u64),
            Err(DecodeError::Truncated { .. })
        ));
    }
}

/// Scenario: A valid container offset is moved one byte before or after the actual snapshot end.
/// Guarantees: Exact snapshot framing rejects both cutting the footer and including a WAL byte in the snapshot section.
#[test]
fn checkpoint_offset_must_match_complete_snapshot_end() {
    let namespace = namespace_digest("app-logs").unwrap();
    for offset in [ACTIVE.len() - 1, ACTIVE.len() + 1] {
        let mut bytes = WITH_WAL.to_vec();
        set_offset(&mut bytes, offset as u64);
        assert!(decode_checkpoint_snapshot(&bytes, &namespace, 1, u64::MAX).is_err());
    }
}

/// Scenario: A complete container has the wrong namespace, an exceeded record budget, or corrupt snapshot bytes.
/// Guarantees: Combining files preserves namespace, allocation, and immutable snapshot integrity checks.
#[test]
fn checkpoint_preserves_snapshot_validation() {
    let namespace = namespace_digest("app-logs").unwrap();
    let other = namespace_digest("other").unwrap();
    assert!(matches!(
        decode_checkpoint_snapshot(ACTIVE, &other, 1, u64::MAX),
        Err(DecodeError::NamespaceMismatch { .. })
    ));
    assert!(matches!(
        decode_checkpoint_snapshot(ACTIVE, &namespace, 0, u64::MAX),
        Err(DecodeError::SnapshotRecordCountExceedsLimit { .. })
    ));
    let mut corrupt = ACTIVE.to_vec();
    corrupt[24 + 60 + 4] ^= 1;
    assert!(matches!(
        decode_checkpoint_snapshot(&corrupt, &namespace, 1, u64::MAX),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
}

/// Scenario: Every partial prefix of a final WAL transaction follows a complete snapshot; the complete transaction is then corrupted.
/// Guarantees: Only WAL scanning reports incompleteness, complete corruption stays an error, and the snapshot remains independently decodable.
#[test]
fn checkpoint_wal_tail_rules_remain_incremental() {
    let namespace = namespace_digest("app-logs").unwrap();
    let offset = ACTIVE.len();
    for end in offset + 1..WITH_WAL.len() {
        let (_, actual) =
            decode_checkpoint_snapshot(&WITH_WAL[..end], &namespace, 1, u64::MAX).unwrap();
        assert_eq!(actual, offset);
        assert!(matches!(
            scan_next_transaction(&WITH_WAL[offset..end], 1),
            Ok(Some(TransactionScan::Incomplete { .. }))
        ));
    }
    let mut corrupt = WITH_WAL.to_vec();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(decode_checkpoint_snapshot(&corrupt, &namespace, 1, u64::MAX).is_ok());
    assert!(matches!(
        scan_next_transaction(&corrupt[offset..], 1),
        Err(DecodeError::ChecksumMismatch { .. })
    ));
    assert!(matches!(
        scan_next_transaction(&WITH_WAL[offset..], 2),
        Err(DecodeError::SequenceOutOfOrder { .. })
    ));
}
