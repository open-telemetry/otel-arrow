// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Positive conformance tests against independently generated fixtures.

use otel_arrow_dfe_filelog_checkpoint::{
    AdvisoryPath, CommittedFrontierGuard, DecodeError, EncodeError, FileId, FramingEncoding,
    FramingOnDecodeError, FramingProfileParams, LifecycleState, MaxLogSizeBehavior, MultilineMode,
    Operation, ResetQuarantineAction, TX_HEADER_BYTES, TX_MIN_BODY_BYTES, Transaction,
    TransactionScan, UpdateFingerprint, WAL_HEADER_BYTES, WAL_MAX_TX_BODY_BYTES, crc32c,
    decode_current, decode_operation, decode_snapshot, decode_wal_header, encode_current,
    encode_operation, encode_snapshot, encode_transaction, encode_wal_header, namespace_digest,
    scan_next_transaction,
};

const CURRENT: &[u8] = include_bytes!("fixtures/current-generation-42.bin");
const EMPTY_SNAPSHOT: &[u8] = include_bytes!("fixtures/snapshot-empty.bin");
const ACTIVE_SNAPSHOT: &[u8] = include_bytes!("fixtures/snapshot-active.bin");
const QUARANTINED_SNAPSHOT: &[u8] = include_bytes!("fixtures/snapshot-quarantined.bin");
const FINALIZED_SNAPSHOT: &[u8] = include_bytes!("fixtures/snapshot-rotated-finalized.bin");
const LONG_PATH_SNAPSHOT: &[u8] = include_bytes!("fixtures/snapshot-long-path.bin");
const WAL_HEADER: &[u8] = include_bytes!("fixtures/wal-header.bin");

struct TestScanResult {
    transactions: Vec<Transaction>,
    torn_tail_bytes: usize,
}

fn scan_all_for_test(bytes: &[u8]) -> Result<TestScanResult, DecodeError> {
    let mut suffix = bytes;
    let mut expected_sequence = 1u64;
    let mut transactions = Vec::new();
    let mut torn_tail_bytes = 0;
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
            TransactionScan::TornTail { bytes } => {
                torn_tail_bytes = bytes;
                break;
            }
        }
    }
    Ok(TestScanResult {
        transactions,
        torn_tail_bytes,
    })
}

fn expected(name: &str) -> Vec<u8> {
    include_str!("fixtures/expected-values.txt")
        .lines()
        .find_map(|line| line.split_once('=').filter(|(key, _)| *key == name))
        .map(|(_, value)| hex::decode(value).expect("fixture hex must be valid"))
        .expect("expected fixture key must exist")
}

fn default_profile(multiline: bool) -> FramingProfileParams {
    FramingProfileParams {
        fingerprint_profile_version: 1,
        fingerprint_bytes: 1000,
        ignored_header_bytes: 0,
        encoding: FramingEncoding::Utf8,
        on_decode_error: FramingOnDecodeError::PreserveRaw,
        multiline_mode: if multiline {
            MultilineMode::EndPattern {
                regex_profile_version: 1,
                pattern: "^END request$".to_owned(),
            }
        } else {
            MultilineMode::Newline
        },
        max_line_bytes: 1_048_576,
        max_record_bytes: 1_048_576,
        max_log_size_behavior: MaxLogSizeBehavior::Split,
        max_multiline_lines: 500,
        force_flush_period_millis: 500,
    }
}

/// Scenario: The independent CURRENT fixture selects generation 42.
/// Guarantees: The fixed width, CRC coverage, and big-endian generation agree with v1.
#[test]
fn current_fixture_matches_codec() {
    assert_eq!(decode_current(CURRENT), Ok(42));
    assert_eq!(encode_current(42), CURRENT);
}

/// Scenario: The independent empty snapshot carries generation zero and no records.
/// Guarantees: Header/footer widths, echoes, namespace binding, and CRCs agree with v1.
#[test]
fn empty_snapshot_fixture_matches_codec() {
    let namespace = namespace_digest("app-logs").unwrap();
    let snapshot = decode_snapshot(EMPTY_SNAPSHOT, &namespace, u32::MAX).unwrap();
    assert_eq!(snapshot.generation, 0);
    assert!(snapshot.records.is_empty());
    assert_eq!(encode_snapshot(0, "app-logs", &[]).unwrap(), EMPTY_SNAPSHOT);
}

/// Scenario: Independently encoded snapshots cover every reachable v1 lifecycle.
/// Guarantees: Active, Quarantined, and RotatedFinalized records decode and re-encode exactly.
#[test]
fn lifecycle_snapshot_fixtures_match_codec() {
    let namespace = namespace_digest("app-logs").unwrap();
    for (bytes, expected_state) in [
        (ACTIVE_SNAPSHOT, LifecycleState::Active),
        (QUARANTINED_SNAPSHOT, LifecycleState::Quarantined),
        (FINALIZED_SNAPSHOT, LifecycleState::RotatedFinalized),
    ] {
        let snapshot = decode_snapshot(bytes, &namespace, u32::MAX).unwrap();
        assert_eq!(snapshot.records.len(), 1);
        assert_eq!(snapshot.records[0].lifecycle_state, expected_state);
        assert_eq!(
            encode_snapshot(snapshot.generation, "app-logs", &snapshot.records).unwrap(),
            bytes
        );
    }
}

/// Scenario: An independent snapshot carries a 5,000-byte Unix path as a bounded suffix.
/// Guarantees: Decoding retains the full length/digest while storing exactly the final 4,096 bytes.
#[test]
fn long_advisory_path_snapshot_is_bounded() {
    let namespace = namespace_digest("app-logs").unwrap();
    let snapshot = decode_snapshot(LONG_PATH_SNAPSHOT, &namespace, u32::MAX).unwrap();
    let path = &snapshot.records[0].advisory_path;
    assert!(path.is_truncated());
    assert_eq!(path.full_path_len(), 5000);
    assert_eq!(path.stored_path_bytes(), vec![b'x'; 4096]);
    assert_eq!(
        path.full_path_digest(),
        expected("advisory_long").as_slice()
    );
}

/// Scenario: The independent WAL header fixture declares generation seven.
/// Guarantees: WAL magic, namespace digest, version fields, and CRC agree with v1.
#[test]
fn wal_header_fixture_matches_codec() {
    assert_eq!(WAL_HEADER.len(), WAL_HEADER_BYTES);
    let header = decode_wal_header(WAL_HEADER).unwrap();
    assert_eq!(header.generation, 7);
    assert_eq!(
        header.namespace_digest,
        namespace_digest("app-logs").unwrap()
    );
    assert_eq!(encode_wal_header(7, "app-logs").unwrap(), WAL_HEADER);
}

/// Scenario: Independent standalone frames exercise every version 1 operation code.
/// Guarantees: Each operation is recognized, consumes its exact frame, and re-encodes byte-for-byte.
#[test]
fn every_operation_fixture_matches_codec() {
    let fixtures: &[&[u8]] = &[
        include_bytes!("fixtures/operation-register-file.bin"),
        include_bytes!("fixtures/operation-update-progress.bin"),
        include_bytes!("fixtures/operation-reset-after-truncate.bin"),
        include_bytes!("fixtures/operation-update-fingerprint.bin"),
        include_bytes!("fixtures/operation-update-metadata.bin"),
        include_bytes!("fixtures/operation-quarantine-file.bin"),
        include_bytes!("fixtures/operation-reset-quarantined-keep-failed.bin"),
        include_bytes!("fixtures/operation-remove-file.bin"),
    ];
    for bytes in fixtures {
        let (operation, consumed) = decode_operation(bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(encode_operation(&operation).unwrap(), *bytes);
    }
    assert!(matches!(
        decode_operation(fixtures[0]).unwrap().0,
        Operation::RegisterFile(_)
    ));
    assert!(matches!(
        decode_operation(fixtures[7]).unwrap().0,
        Operation::RemoveFile(_)
    ));
}

/// Scenario: A keep_failed operation contains values that may conflict with stored state.
/// Guarantees: PR1A preserves the structural PR1B test case without attempting table-dependent rejection.
#[test]
fn keep_failed_mutation_remains_structurally_valid() {
    let bytes = include_bytes!("fixtures/operation-keep-failed-mutation.bin");
    let (operation, consumed) = decode_operation(bytes).unwrap();
    assert_eq!(consumed, bytes.len());
    let Operation::ResetQuarantinedFile(operation) = &operation else {
        panic!("fixture must carry reset_quarantined_file");
    };
    assert_eq!(operation.action, ResetQuarantineAction::KeepFailed);
    assert_eq!(operation.resulting_epoch, 99);
    assert_eq!(
        encode_operation(&Operation::ResetQuarantinedFile(operation.clone())).unwrap(),
        bytes
    );
}

/// Scenario: Every operation is independently wrapped in a complete one-operation transaction.
/// Guarantees: Transaction envelope and operation CRC coverage agree independently for all operation codes.
#[test]
fn every_operation_transaction_fixture_matches_codec() {
    let fixtures: &[&[u8]] = &[
        include_bytes!("fixtures/transaction-register-file.bin"),
        include_bytes!("fixtures/transaction-update-progress.bin"),
        include_bytes!("fixtures/transaction-reset-after-truncate.bin"),
        include_bytes!("fixtures/transaction-update-fingerprint.bin"),
        include_bytes!("fixtures/transaction-update-metadata.bin"),
        include_bytes!("fixtures/transaction-quarantine-file.bin"),
        include_bytes!("fixtures/transaction-reset-quarantined-keep-failed.bin"),
        include_bytes!("fixtures/transaction-keep-failed-mutation.bin"),
        include_bytes!("fixtures/transaction-remove-file.bin"),
        include_bytes!("fixtures/transaction-reset-quarantined-beginning.bin"),
        include_bytes!("fixtures/transaction-reset-quarantined-end.bin"),
        include_bytes!("fixtures/transaction-update-metadata-without-path.bin"),
        include_bytes!("fixtures/transaction-remove-file-non-administrative.bin"),
    ];
    for bytes in fixtures {
        let scan = scan_all_for_test(bytes).unwrap();
        assert_eq!(scan.torn_tail_bytes, 0);
        assert_eq!(scan.transactions.len(), 1);
        assert_eq!(encode_transaction(&scan.transactions[0]).unwrap(), *bytes);
    }
}

/// Scenario: The smallest semantically valid transaction uses an empty-to-one-byte fingerprint extension.
/// Guarantees: The published minimum body arithmetic is exact and accepted by the scanner.
#[test]
fn minimum_transaction_fixture_has_exact_boundary() {
    let bytes = include_bytes!("fixtures/transaction-minimum.bin");
    assert_eq!(
        bytes.len(),
        TX_HEADER_BYTES + TX_MIN_BODY_BYTES as usize + 4
    );
    assert_eq!(scan_all_for_test(bytes).unwrap().transactions.len(), 1);
}

/// Scenario: A progress transaction contains the maximum 4,096 unique operations at maximum progress width.
/// Guarantees: The progress count/body boundary is accepted without exceeding the published bounded size.
#[test]
fn maximum_progress_transaction_fixture_is_accepted() {
    let bytes = include_bytes!("fixtures/transaction-max-progress.bin");
    let scan = scan_all_for_test(bytes).unwrap();
    assert_eq!(scan.transactions[0].operations.len(), 4096);
    assert_eq!(bytes.len(), 446_504);
}

/// Scenario: Independent fixtures contain progress-only and non-progress transaction classes.
/// Guarantees: Both legal classes scan cleanly and retain their exact operation counts.
#[test]
fn transaction_class_fixtures_are_accepted() {
    for bytes in [
        include_bytes!("fixtures/transaction-progress-class.bin").as_slice(),
        include_bytes!("fixtures/transaction-non-progress-class.bin").as_slice(),
    ] {
        let scan = scan_all_for_test(bytes).unwrap();
        assert_eq!(scan.transactions[0].operations.len(), 2);
        assert_eq!(scan.torn_tail_bytes, 0);
    }
}

/// Scenario: Published namespace, advisory-path, frontier, and CRC vectors are recomputed.
/// Guarantees: Every domain separator and native path representation matches the normative values.
#[test]
fn published_digest_and_crc_vectors_match() {
    assert_eq!(
        crc32c(b"123456789").to_be_bytes(),
        expected("crc32c_123456789")[..]
    );
    assert_eq!(
        namespace_digest("app-logs").unwrap(),
        expected("namespace_app_logs")[..]
    );

    let unix = AdvisoryPath::from_unix_bytes(b"/var/log/app.log").unwrap();
    assert_eq!(
        unix.full_path_digest(),
        expected("advisory_unix").as_slice()
    );
    let windows_units: Vec<u16> = "C:\\logs\\app.log".encode_utf16().collect();
    let windows = AdvisoryPath::from_windows_utf16_units(&windows_units).unwrap();
    assert_eq!(
        windows.full_path_digest(),
        expected("advisory_windows_utf16le").as_slice()
    );
    let long = AdvisoryPath::from_unix_bytes(&vec![b'x'; 5000]).unwrap();
    assert!(long.is_truncated());
    assert_eq!(long.stored_path_bytes().len(), 4096);
    assert_eq!(
        long.full_path_digest(),
        expected("advisory_long").as_slice()
    );

    let empty = CommittedFrontierGuard::empty();
    assert_eq!(empty.digest, expected("frontier_empty")[..]);
    let nonempty = CommittedFrontierGuard::compute(4, b"abc\n").unwrap();
    assert_eq!(nonempty.digest, expected("frontier_nonempty")[..]);
}

/// Scenario: Default and end-pattern framing profiles are serialized independently.
/// Guarantees: Canonical field order, widths, pattern bytes, and SHA-256 digests match the published vectors.
#[test]
fn framing_profile_fixtures_match_codec() {
    for (multiline, canonical_fixture, digest_key) in [
        (
            false,
            include_bytes!("fixtures/framing-profile-default.bin").as_slice(),
            "framing_profile_default",
        ),
        (
            true,
            include_bytes!("fixtures/framing-profile-multiline.bin").as_slice(),
            "framing_profile_multiline",
        ),
    ] {
        let profile = default_profile(multiline);
        assert_eq!(profile.canonical_bytes().unwrap(), canonical_fixture);
        assert_eq!(profile.digest().unwrap(), expected(digest_key)[..]);
    }
}

/// Scenario: Recovery scans one hundred correctly sequenced transactions and then an empty suffix.
/// Guarantees: The public scanner returns only one transaction at a time and clean EOF is distinct from a torn tail.
#[test]
fn transaction_scanning_is_incremental_and_accepts_clean_eof() {
    let operation = decode_operation(include_bytes!("fixtures/operation-update-metadata.bin"))
        .unwrap()
        .0;
    let mut wal = Vec::new();
    for sequence in 1..=100 {
        wal.extend_from_slice(
            &encode_transaction(&Transaction {
                sequence,
                operations: vec![operation.clone()],
            })
            .unwrap(),
        );
    }

    let mut suffix = wal.as_slice();
    for expected_sequence in 1..=100 {
        let Some(TransactionScan::Complete {
            transaction,
            consumed,
        }) = scan_next_transaction(suffix, expected_sequence).unwrap()
        else {
            panic!("complete transaction expected");
        };
        assert_eq!(transaction.sequence, expected_sequence);
        suffix = &suffix[consumed..];
        drop(transaction);
    }
    assert!(suffix.is_empty());
    assert_eq!(scan_next_transaction(suffix, 101).unwrap(), None);
}

/// Scenario: A non-progress transaction contains exactly the class maximum of 256 operations.
/// Guarantees: The inclusive operation-count boundary is accepted by both encoder and scanner.
#[test]
fn maximum_non_progress_operation_count_is_accepted() {
    let operation = decode_operation(include_bytes!("fixtures/operation-update-metadata.bin"))
        .unwrap()
        .0;
    let bytes = encode_transaction(&Transaction {
        sequence: 1,
        operations: vec![operation; 256],
    })
    .unwrap();
    let Some(TransactionScan::Complete { transaction, .. }) =
        scan_next_transaction(&bytes, 1).unwrap()
    else {
        panic!("complete transaction expected");
    };
    assert_eq!(transaction.operations.len(), 256);
}

fn fingerprint_extension(file_id: FileId, old_len: usize, new_len: usize) -> Operation {
    let expected_fingerprint = vec![0xA5; old_len];
    let mut new_fingerprint = expected_fingerprint.clone();
    new_fingerprint.resize(new_len, 0x5A);
    Operation::UpdateFingerprint(UpdateFingerprint {
        file_id,
        expected_file_epoch: 1,
        expected_fingerprint,
        new_fingerprint,
    })
}

/// Scenario: Non-progress transaction bodies are exactly 16 MiB and one byte larger.
/// Guarantees: The exact hard boundary encodes and scans, while the first excessive body emits no frame.
#[test]
fn transaction_body_limit_is_exact() {
    let full = fingerprint_extension(FileId::from_bytes([0x31; 16]), 65_534, 65_535);
    let final_exact = fingerprint_extension(FileId::from_bytes([0x32; 16]), 63_614, 63_615);
    let mut transaction = Transaction {
        sequence: 1,
        operations: vec![full; 127],
    };
    transaction.operations.push(final_exact);

    let bytes = encode_transaction(&transaction).unwrap();
    assert_eq!(bytes.len(), TX_HEADER_BYTES + 16 * 1024 * 1024 + 4);
    let Some(TransactionScan::Complete { consumed, .. }) =
        scan_next_transaction(&bytes, 1).unwrap()
    else {
        panic!("complete transaction expected");
    };
    assert_eq!(consumed, bytes.len());
    drop(bytes);

    let Operation::UpdateFingerprint(last) = transaction.operations.last_mut().unwrap() else {
        panic!("fingerprint operation expected");
    };
    last.new_fingerprint.push(0x5A);
    assert!(matches!(
        encode_transaction(&transaction),
        Err(EncodeError::TransactionBodyTooLarge {
            len,
            max: WAL_MAX_TX_BODY_BYTES,
            ..
        }) if len == WAL_MAX_TX_BODY_BYTES + 1
    ));
}

/// Scenario: Independent fixtures exercise both reset actions, optional metadata, and non-administrative removal.
/// Guarantees: All three quarantine actions and both optional-field absences round-trip byte-for-byte.
#[test]
fn independent_optional_and_action_fixtures_match_codec() {
    let fixtures: &[&[u8]] = &[
        include_bytes!("fixtures/operation-reset-quarantined-beginning.bin"),
        include_bytes!("fixtures/operation-reset-quarantined-end.bin"),
        include_bytes!("fixtures/operation-update-metadata-without-path.bin"),
        include_bytes!("fixtures/operation-remove-file-non-administrative.bin"),
    ];
    for bytes in fixtures {
        let (operation, consumed) = decode_operation(bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(encode_operation(&operation).unwrap(), *bytes);
    }

    let Operation::ResetQuarantinedFile(beginning) = decode_operation(fixtures[0]).unwrap().0
    else {
        panic!("reset operation expected");
    };
    let Operation::ResetQuarantinedFile(end) = decode_operation(fixtures[1]).unwrap().0 else {
        panic!("reset operation expected");
    };
    assert_eq!(beginning.action, ResetQuarantineAction::ResetToBeginning);
    assert_eq!(end.action, ResetQuarantineAction::ResetToEnd);
}

/// Scenario: Unix advisory paths contain exactly 4,096 and 4,097 native bytes.
/// Guarantees: Truncation begins only above the v1 stored-path limit and retains the exact final 4,096 bytes.
#[test]
fn advisory_path_truncation_boundary_is_exact() {
    let exact_bytes = vec![b'a'; 4096];
    let exact = AdvisoryPath::from_unix_bytes(&exact_bytes).unwrap();
    assert!(!exact.is_truncated());
    assert_eq!(exact.full_path_len(), 4096);
    assert_eq!(exact.stored_path_bytes(), exact_bytes);

    let mut long_bytes = vec![b'b'; 4097];
    long_bytes[0] = b'a';
    let truncated = AdvisoryPath::from_unix_bytes(&long_bytes).unwrap();
    assert!(truncated.is_truncated());
    assert_eq!(truncated.full_path_len(), 4097);
    assert_eq!(truncated.stored_path_bytes(), &long_bytes[1..]);
}

/// Scenario: Frontier guards are computed immediately below, at, and above the 64-byte window cap.
/// Guarantees: Required window lengths are 63, 64, and 64 for offsets 63, 64, and 65 respectively.
#[test]
fn frontier_guard_window_boundary_is_exact() {
    for (offset, window_len) in [(63, 63), (64, 64), (65, 64)] {
        let guard = CommittedFrontierGuard::compute(offset, &vec![0xA5; window_len]).unwrap();
        assert_eq!(guard.window_len, window_len as u16);
    }
    assert!(CommittedFrontierGuard::compute(65, &[0xA5; 65]).is_err());
}

/// Scenario: Framing profiles exercise exact pattern length and invalid version boundaries.
/// Guarantees: A 4,096-byte pattern is accepted; 4,097 bytes, zero versions, empty patterns, and subminimum fingerprints are rejected.
#[test]
fn framing_profile_pattern_and_version_boundaries_are_enforced() {
    let mut profile = default_profile(false);
    profile.multiline_mode = MultilineMode::StartPattern {
        regex_profile_version: 1,
        pattern: "x".repeat(4096),
    };
    assert!(profile.canonical_bytes().is_ok());

    let mut too_long = profile.clone();
    let MultilineMode::StartPattern { pattern, .. } = &mut too_long.multiline_mode else {
        unreachable!();
    };
    pattern.push('x');
    assert!(matches!(
        too_long.canonical_bytes(),
        Err(EncodeError::FieldTooLong { max: 4096, .. })
    ));

    let mut zero_regex = profile.clone();
    let MultilineMode::StartPattern {
        regex_profile_version,
        ..
    } = &mut zero_regex.multiline_mode
    else {
        unreachable!();
    };
    *regex_profile_version = 0;
    assert!(matches!(
        zero_regex.canonical_bytes(),
        Err(EncodeError::InvalidFieldValue { .. })
    ));

    let mut empty_pattern = profile.clone();
    let MultilineMode::StartPattern { pattern, .. } = &mut empty_pattern.multiline_mode else {
        unreachable!();
    };
    pattern.clear();
    assert!(matches!(
        empty_pattern.canonical_bytes(),
        Err(EncodeError::RequiredFieldEmpty { .. })
    ));

    let mut zero_fingerprint_version = default_profile(false);
    zero_fingerprint_version.fingerprint_profile_version = 0;
    assert!(matches!(
        zero_fingerprint_version.canonical_bytes(),
        Err(EncodeError::InvalidFieldValue { .. })
    ));
    let mut short_fingerprint = default_profile(false);
    short_fingerprint.fingerprint_bytes = 15;
    assert!(matches!(
        short_fingerprint.canonical_bytes(),
        Err(EncodeError::InvalidFieldValue { .. })
    ));
}
