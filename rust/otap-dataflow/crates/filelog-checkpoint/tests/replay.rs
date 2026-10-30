// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! State-machine and atomicity checks for checkpoint replay.

use otel_arrow_dfe_filelog_checkpoint::{
    AdvisoryPath, CommittedFrontierGuard, DecodeError, FileId, FramingEncoding,
    FramingOnDecodeError, FramingProfileParams, FramingResume, LifecycleState, Locator,
    MaxLogSizeBehavior, MultilineMode, Operation, QuarantineEvidence, QuarantineFile, RegisterFile,
    RemoveFile, ReplayConfig, ReplayError, ReplayState, ReplayStep, ResetAfterTruncate,
    ResetQuarantineAction, ResetQuarantinedFile, SnapshotRecord, Transaction, UpdateFingerprint,
    UpdateMetadata, UpdateProgress, crc32c, encode_checkpoint, encode_transaction,
};

const NAMESPACE: &str = "replay-test";

fn profile() -> FramingProfileParams {
    FramingProfileParams {
        fingerprint_profile_version: 1,
        fingerprint_bytes: 32,
        ignored_header_bytes: 0,
        encoding: FramingEncoding::Utf8,
        on_decode_error: FramingOnDecodeError::PreserveRaw,
        multiline_mode: MultilineMode::Newline,
        max_line_bytes: 1024,
        max_record_bytes: 1024,
        max_log_size_behavior: MaxLogSizeBehavior::Split,
        max_multiline_lines: 100,
        force_flush_period_millis: 0,
    }
}

fn config() -> ReplayConfig {
    ReplayConfig::new(100, 1024 * 1024, &profile()).expect("profile")
}

fn id(n: u8) -> FileId {
    FileId::from_bytes([n; 16])
}

fn guard(offset: u64) -> CommittedFrontierGuard {
    CommittedFrontierGuard::compute(offset, &vec![b'x'; offset.min(64) as usize]).expect("guard")
}

fn record(n: u8) -> SnapshotRecord {
    SnapshotRecord {
        file_id: id(n),
        file_epoch: 1,
        committed_offset: 10,
        committed_frontier_guard: guard(10),
        fingerprint: b"prefix".to_vec(),
        ignored_header_bytes: 0,
        locator: Locator::PosixDevIno {
            dev: 1,
            ino: u64::from(n),
        },
        framing_profile_version: 1,
        framing_profile_digest: profile().digest().expect("digest"),
        framing_resume: FramingResume::Clean,
        lifecycle_state: LifecycleState::Active,
        quarantine_evidence: None,
        last_seen_time_unix_nano: 11,
        advisory_path: AdvisoryPath::from_unix_bytes(b"/logs/example").expect("path"),
    }
}

fn state_with(
    records: &[SnapshotRecord],
    config: ReplayConfig,
) -> Result<ReplayState, ReplayError> {
    let bytes = encode_checkpoint(9, NAMESPACE, records).expect("snapshot");
    let (state, offset) = ReplayState::from_checkpoint(&bytes, NAMESPACE, config)?;
    assert_eq!(offset, bytes.len());
    Ok(state)
}

fn state(records: &[SnapshotRecord]) -> ReplayState {
    state_with(records, config()).expect("replay state")
}

fn tx(sequence: u64, operations: Vec<Operation>) -> Vec<u8> {
    encode_transaction(&Transaction {
        sequence,
        operations,
    })
    .expect("structurally valid transaction")
}

fn apply(state: &mut ReplayState, operations: Vec<Operation>) {
    let sequence = state.last_sequence() + 1;
    let bytes = tx(sequence, operations);
    assert_eq!(
        state.replay_next(&bytes),
        Ok(ReplayStep::Applied {
            sequence,
            consumed: bytes.len()
        })
    );
}

fn reject(state: &mut ReplayState, operations: Vec<Operation>) -> ReplayError {
    let before = contents(state);
    let seq = state.last_sequence();
    let error = state
        .replay_next(&tx(seq + 1, operations))
        .expect_err("must reject");
    assert_eq!(contents(state), before, "no partial record mutation");
    assert_eq!(state.last_sequence(), seq, "no sequence advancement");
    error
}

fn contents(state: &ReplayState) -> Vec<SnapshotRecord> {
    let mut records: Vec<_> = state.records().cloned().collect();
    records.sort_by_key(|r| r.file_id);
    records
}

fn registration(r: &SnapshotRecord) -> Operation {
    Operation::RegisterFile(RegisterFile {
        file_id: r.file_id,
        file_epoch: r.file_epoch,
        committed_offset: r.committed_offset,
        committed_frontier_guard: r.committed_frontier_guard,
        fingerprint: r.fingerprint.clone(),
        ignored_header_bytes: r.ignored_header_bytes,
        locator: r.locator,
        framing_profile_version: r.framing_profile_version,
        framing_profile_digest: r.framing_profile_digest,
        framing_resume: r.framing_resume,
        last_seen_time_unix_nano: r.last_seen_time_unix_nano,
        advisory_path: r.advisory_path.clone(),
    })
}

fn progress(r: &SnapshotRecord, end: u64) -> UpdateProgress {
    UpdateProgress {
        file_id: r.file_id,
        expected_file_epoch: r.file_epoch,
        expected_committed_offset: r.committed_offset,
        new_committed_offset: end,
        new_committed_frontier_guard: guard(end),
        new_framing_resume: FramingResume::Clean,
        new_last_seen_time_unix_nano: 99,
        finalize: false,
    }
}

fn quarantine(r: &SnapshotRecord) -> Operation {
    Operation::QuarantineFile(QuarantineFile {
        file_id: r.file_id,
        expected_file_epoch: r.file_epoch,
        reason_code: 1,
        locator: r.locator,
        observed_size: 100,
        quarantine_epoch: r.file_epoch,
        quarantine_time_unix_nano: 50,
    })
}

fn metadata(r: &SnapshotRecord) -> Operation {
    Operation::UpdateMetadata(UpdateMetadata {
        file_id: r.file_id,
        expected_prior_state: r.lifecycle_state,
        expected_file_epoch: r.file_epoch,
        last_seen_time_unix_nano: 88,
        advisory_path: None,
    })
}

fn removal(r: &SnapshotRecord, administrative: bool) -> Operation {
    Operation::RemoveFile(RemoveFile {
        file_id: r.file_id,
        expected_file_epoch: r.file_epoch,
        expected_prior_state: r.lifecycle_state,
        removal_reason: 1,
        removal_time_unix_nano: 100,
        administrative,
        namespace_id: administrative.then(|| NAMESPACE.to_owned()),
        audit_reason: administrative.then(|| "operator approved".to_owned()),
    })
}

fn reset(r: &SnapshotRecord, action: ResetQuarantineAction) -> ResetQuarantinedFile {
    let (resulting_epoch, resulting_offset) = match action {
        ResetQuarantineAction::KeepFailed => (r.file_epoch, r.committed_offset),
        ResetQuarantineAction::ResetToBeginning => (r.file_epoch + 1, 0),
        ResetQuarantineAction::ResetToEnd => (r.file_epoch + 1, 100),
    };
    ResetQuarantinedFile {
        file_id: r.file_id,
        expected_quarantine_epoch: r.file_epoch,
        action,
        resulting_epoch,
        resulting_offset,
        new_committed_frontier_guard: guard(resulting_offset),
        new_framing_resume: r.framing_resume,
        new_fingerprint: r.fingerprint.clone(),
        action_time_unix_nano: 123,
        namespace_id: NAMESPACE.to_owned(),
        audit_reason: "verified source".to_owned(),
    }
}

fn continuation(start: u64, end: u64, index: u32) -> FramingResume {
    FramingResume::Continuation {
        record_start_offset: start,
        record_end_offset: end,
        next_fragment_index: index,
    }
}

// Modify a single-operation transaction while maintaining every checksum. This
// exercises semantic rules that the producer refuses to encode in the first place.
fn mutate_payload(bytes: &mut [u8], change: impl FnOnce(&mut [u8])) {
    let payload_len = u32::from_be_bytes(bytes[36..40].try_into().expect("length")) as usize;
    change(&mut bytes[40..40 + payload_len]);
    let crc_pos = 40 + payload_len;
    let op_crc = crc32c(&bytes[36..crc_pos]);
    bytes[crc_pos..crc_pos + 4].copy_from_slice(&op_crc.to_be_bytes());
    let tx_end = bytes.len() - 4;
    let tx_crc = crc32c(&bytes[..tx_end]);
    bytes[tx_end..].copy_from_slice(&tx_crc.to_be_bytes());
}

/// Scenario: A snapshot and successive WAL transactions form one recovery generation.
/// Guarantees: Replay starts at sequence one, consumes one frame, and preserves snapshot generation.
#[test]
fn snapshot_and_incremental_wal_recovery() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    let first = tx(1, vec![Operation::UpdateProgress(progress(&r, 20))]);
    let mut second_record = r.clone();
    second_record.committed_offset = 20;
    let second = tx(
        2,
        vec![Operation::UpdateProgress(progress(&second_record, 30))],
    );
    let bytes = [first.as_slice(), second.as_slice()].concat();
    assert_eq!(
        state.replay_next(&bytes),
        Ok(ReplayStep::Applied {
            sequence: 1,
            consumed: first.len()
        })
    );
    assert_eq!(state.get(&id(1)).expect("record").committed_offset, 20);
    let _ = state
        .replay_next(&bytes[first.len()..])
        .expect("second frame");
    assert_eq!(state.get(&id(1)).expect("record").committed_offset, 30);
    assert_eq!(state.last_sequence(), 2);
    assert_eq!(state.generation(), 9);
    assert_eq!(state.replay_next(&[]), Ok(ReplayStep::Empty));
}

/// Scenario: Every proper prefix of a valid transaction is offered before the full frame.
/// Guarantees: Incomplete input neither applies operations nor advances sequence, and can be retried intact.
#[test]
fn incomplete_prefixes_never_apply() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    let bytes = tx(1, vec![Operation::UpdateProgress(progress(&r, 20))]);
    for end in 1..bytes.len() {
        assert!(
            matches!(state.replay_next(&bytes[..end]), Ok(ReplayStep::Incomplete { bytes: n, .. }) if n == end)
        );
        assert_eq!(state.get(&id(1)), Some(&r));
        assert_eq!(state.last_sequence(), 0);
    }
    let _ = state.replay_next(&bytes).expect("complete frame");
}

/// Scenario: A complete frame is corrupt or has an unexpected sequence.
/// Guarantees: Corruption is not treated as a torn tail and repeated or skipped sequences cannot advance state.
#[test]
fn corruption_and_sequence_errors_fail_closed() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    let valid = tx(1, vec![metadata(&r)]);
    let mut corrupt = valid.clone();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 1;
    assert!(matches!(
        state.replay_next(&corrupt),
        Err(ReplayError::Decode(DecodeError::ChecksumMismatch { .. }))
    ));
    assert_eq!(state.get(&id(1)), Some(&r));
    assert!(matches!(
        state.replay_next(&tx(2, vec![metadata(&r)])),
        Err(ReplayError::Decode(DecodeError::SequenceOutOfOrder { .. }))
    ));
    let _ = state.replay_next(&valid).expect("valid");
    assert!(matches!(
        state.replay_next(&valid),
        Err(ReplayError::Decode(DecodeError::SequenceOutOfOrder { .. }))
    ));
    assert_eq!(state.last_sequence(), 1);
}

/// Scenario: A multi-file progress transaction fails on its final operation.
/// Guarantees: Earlier valid progress and the sequence remain unchanged, so a corrected transaction can succeed.
#[test]
fn progress_transaction_rolls_back_all_files() {
    let a = record(1);
    let b = record(2);
    let mut state = state(&[a.clone(), b.clone()]);
    let mut stale = progress(&b, 30);
    stale.expected_committed_offset = 9;
    let _ = reject(
        &mut state,
        vec![
            Operation::UpdateProgress(progress(&a, 20)),
            Operation::UpdateProgress(stale),
        ],
    );
    apply(
        &mut state,
        vec![
            Operation::UpdateProgress(progress(&a, 20)),
            Operation::UpdateProgress(progress(&b, 30)),
        ],
    );
    assert_eq!(state.get(&id(1)).expect("a").committed_offset, 20);
    assert_eq!(state.get(&id(2)).expect("b").committed_offset, 30);
}

/// Scenario: A record is registered twice identically, then with changed evidence.
/// Guarantees: Only exact registration replay is idempotent; conflicting fields are rejected.
#[test]
fn registration_is_exactly_idempotent() {
    let r = record(1);
    let mut state = state(&[]);
    apply(&mut state, vec![registration(&r), registration(&r)]);
    let mut conflict = r.clone();
    conflict.last_seen_time_unix_nano += 1;
    let _ = reject(&mut state, vec![registration(&conflict)]);
    apply(&mut state, vec![quarantine(&r)]);
    let _ = reject(&mut state, vec![registration(&r)]);
}

/// Scenario: A registration transaction attempts two live records for one locator.
/// Guarantees: Both registrations and locator claims roll back, while a valid retry can claim that locator.
#[test]
fn locator_collision_rolls_back_index_and_records() {
    let a = record(1);
    let mut b = record(2);
    b.locator = a.locator;
    let mut state = state(&[]);
    let _ = reject(&mut state, vec![registration(&a), registration(&b)]);
    apply(&mut state, vec![registration(&b)]);
    assert_eq!(state.records().len(), 1);
    let _ = reject(&mut state, vec![registration(&a)]);
}

/// Scenario: A live locator is superseded through non-administrative removal.
/// Guarantees: Removal requires a different new file ID at that same locator in the same transaction.
#[test]
fn identity_replacement_is_atomic_and_locator_bound() {
    let a = record(1);
    let mut b = record(2);
    let mut state = state(std::slice::from_ref(&a));
    let _ = reject(&mut state, vec![removal(&a, false)]);
    let _ = reject(&mut state, vec![removal(&a, false), registration(&b)]);
    let _ = reject(&mut state, vec![removal(&a, false), registration(&a)]);
    b.locator = a.locator;
    let _ = reject(&mut state, vec![registration(&b), removal(&a, false)]);
    let _ = reject(
        &mut state,
        vec![removal(&a, false), registration(&b), removal(&b, true)],
    );
    apply(&mut state, vec![removal(&a, false), registration(&b)]);
    assert!(state.get(&id(1)).is_none());
    assert_eq!(state.get(&id(2)), Some(&b));
}

/// Scenario: A finalized record shares its old locator with a new live file.
/// Guarantees: Finalization releases the locator index, and removing the old finalized record cannot release the new claim.
#[test]
fn finalized_records_do_not_own_live_locators() {
    let a = record(1);
    let mut state = state(std::slice::from_ref(&a));
    let mut end = progress(&a, a.committed_offset);
    end.finalize = true;
    apply(&mut state, vec![Operation::UpdateProgress(end)]);
    let mut b = record(2);
    b.locator = a.locator;
    apply(&mut state, vec![registration(&b)]);
    let old = state.get(&id(1)).expect("finalized").clone();
    apply(&mut state, vec![removal(&old, true)]);
    let _ = reject(&mut state, vec![registration(&a)]);
}

/// Scenario: Administrative operations name a different raw namespace, including absent targets.
/// Guarantees: NamespaceMismatch precedes lookup and absent-file idempotency; valid absent removal is a no-op.
#[test]
fn administrative_namespace_precedes_lookup() {
    let r = record(1);
    for present in [false, true] {
        let mut state = state(if present {
            std::slice::from_ref(&r)
        } else {
            &[]
        });
        let Operation::RemoveFile(mut remove) = removal(&r, true) else {
            unreachable!()
        };
        remove.namespace_id = Some("other".to_owned());
        assert_eq!(
            reject(&mut state, vec![Operation::RemoveFile(remove)]),
            ReplayError::NamespaceMismatch { file_id: id(1) }
        );
        let mut reset = reset(&r, ResetQuarantineAction::ResetToBeginning);
        reset.namespace_id = "other".to_owned();
        assert_eq!(
            reject(&mut state, vec![Operation::ResetQuarantinedFile(reset)]),
            ReplayError::NamespaceMismatch { file_id: id(1) }
        );
    }
    let mut state = state(&[]);
    apply(&mut state, vec![removal(&r, true)]);
    let _ = reject(&mut state, vec![removal(&r, false)]);
}

/// Scenario: Matching and conflicting quarantine operations are replayed.
/// Guarantees: Quarantine preserves source progress and observation time, and exact evidence alone is idempotent.
#[test]
fn quarantine_freezes_evidence_without_observing_source() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    apply(&mut state, vec![quarantine(&r), quarantine(&r)]);
    let mut expected = r.clone();
    expected.lifecycle_state = LifecycleState::Quarantined;
    expected.quarantine_evidence = Some(QuarantineEvidence {
        reason_code: 1,
        observed_size: 100,
        quarantine_epoch: 1,
        quarantine_time_unix_nano: 50,
    });
    assert_eq!(state.get(&id(1)), Some(&expected));
    let Operation::QuarantineFile(mut conflict) = quarantine(&r) else {
        unreachable!()
    };
    conflict.observed_size += 1;
    let _ = reject(&mut state, vec![Operation::QuarantineFile(conflict)]);
    let _ = reject(
        &mut state,
        vec![Operation::UpdateProgress(progress(&r, 20))],
    );
}

/// Scenario: A keep-failed administrative operation carries attempted mutations.
/// Guarantees: Every operational field remains unchanged, including last-seen time, and mismatches have a distinct error.
#[test]
fn keep_failed_is_an_exact_noop() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    apply(&mut state, vec![quarantine(&r)]);
    let quarantined = state.get(&id(1)).expect("quarantine").clone();
    let keep = reset(&quarantined, ResetQuarantineAction::KeepFailed);
    apply(
        &mut state,
        vec![Operation::ResetQuarantinedFile(keep.clone())],
    );
    assert_eq!(state.get(&id(1)), Some(&quarantined));
    for variation in 0..4 {
        let mut changed = keep.clone();
        match variation {
            0 => {
                changed.resulting_offset += 1;
                changed.new_committed_frontier_guard = guard(changed.resulting_offset);
            }
            1 => changed.new_committed_frontier_guard.digest[0] ^= 1,
            2 => changed.new_fingerprint.push(b'x'),
            _ => changed.new_framing_resume = continuation(0, 0, 1),
        }
        assert_eq!(
            reject(&mut state, vec![Operation::ResetQuarantinedFile(changed)]),
            ReplayError::KeepFailedStateChange { file_id: id(1) }
        );
    }
}

/// Scenario: Quarantine is reset to the beginning or externally verified end.
/// Guarantees: Epoch, offset, guard, fingerprint and lifecycle change together while identity and profile remain fixed.
#[test]
fn quarantine_reset_rebases_source_evidence() {
    for action in [
        ResetQuarantineAction::ResetToBeginning,
        ResetQuarantineAction::ResetToEnd,
    ] {
        let r = record(1);
        let mut state = state(std::slice::from_ref(&r));
        apply(&mut state, vec![quarantine(&r)]);
        let mut reset = reset(state.get(&id(1)).expect("record"), action);
        reset.new_fingerprint = b"replacement".to_vec();
        let expected = SnapshotRecord {
            file_epoch: 2,
            committed_offset: reset.resulting_offset,
            committed_frontier_guard: reset.new_committed_frontier_guard,
            fingerprint: reset.new_fingerprint.clone(),
            last_seen_time_unix_nano: 123,
            ..r.clone()
        };
        apply(&mut state, vec![Operation::ResetQuarantinedFile(reset)]);
        assert_eq!(state.get(&id(1)), Some(&expected));
        let _ = reject(
            &mut state,
            vec![Operation::UpdateProgress(progress(&r, 20))],
        );
    }
}

/// Scenario: Truncation resets an active stream with new fingerprint evidence.
/// Guarantees: The new epoch starts at zero with an empty guard and stale old-epoch updates fail.
#[test]
fn truncate_reset_rebases_epoch_and_fingerprint() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    let reset = ResetAfterTruncate {
        file_id: id(1),
        expected_active_epoch: 1,
        observed_truncated_size: 3,
        resulting_epoch: 2,
        new_committed_offset: 0,
        new_framing_resume: FramingResume::Clean,
        new_fingerprint: b"new".to_vec(),
        reset_time_unix_nano: 77,
        reason_code: 1,
    };
    apply(&mut state, vec![Operation::ResetAfterTruncate(reset)]);
    let expected = SnapshotRecord {
        file_epoch: 2,
        committed_offset: 0,
        committed_frontier_guard: CommittedFrontierGuard::empty(),
        fingerprint: b"new".to_vec(),
        last_seen_time_unix_nano: 77,
        ..r.clone()
    };
    assert_eq!(state.get(&id(1)), Some(&expected));
    let _ = reject(&mut state, vec![metadata(&r)]);
    let _ = reject(&mut state, vec![quarantine(&r)]);
    let _ = reject(&mut state, vec![removal(&r, true)]);
}

/// Scenario: Fingerprint extension carries matching, conflicting, or oversized evidence.
/// Guarantees: Only strict growth of the stored prefix within the configured window can commit.
#[test]
fn fingerprint_growth_requires_stored_prefix_and_bound() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    let good = UpdateFingerprint {
        file_id: id(1),
        expected_file_epoch: 1,
        expected_fingerprint: r.fingerprint.clone(),
        new_fingerprint: b"prefix-more".to_vec(),
    };
    let mut mismatch = good.clone();
    mismatch.expected_fingerprint = b"other".to_vec();
    mismatch.new_fingerprint = b"other-more".to_vec();
    let _ = reject(&mut state, vec![Operation::UpdateFingerprint(mismatch)]);
    let mut oversized = good.clone();
    oversized.new_fingerprint.resize(33, b'x');
    let _ = reject(&mut state, vec![Operation::UpdateFingerprint(oversized)]);
    apply(&mut state, vec![Operation::UpdateFingerprint(good)]);
    let updated = state.get(&id(1)).expect("record");
    assert_eq!(updated.fingerprint, b"prefix-more");
    assert_eq!(updated.committed_frontier_guard, r.committed_frontier_guard);
    assert_eq!(updated.last_seen_time_unix_nano, r.last_seen_time_unix_nano);
}

/// Scenario: Metadata is updated while active and quarantined.
/// Guarantees: Optional paths preserve or replace advisory metadata without changing identity, progress, or quarantine evidence.
#[test]
fn metadata_preserves_operational_state() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    apply(&mut state, vec![metadata(&r)]);
    assert_eq!(
        state.get(&id(1)).expect("record").advisory_path,
        r.advisory_path
    );
    apply(&mut state, vec![quarantine(&r)]);
    let quarantined = state.get(&id(1)).expect("record").clone();
    let Operation::UpdateMetadata(mut op) = metadata(&quarantined) else {
        unreachable!()
    };
    op.advisory_path = Some(AdvisoryPath::unavailable());
    apply(&mut state, vec![Operation::UpdateMetadata(op)]);
    assert_eq!(
        state.get(&id(1)),
        Some(&SnapshotRecord {
            advisory_path: AdvisoryPath::unavailable(),
            ..quarantined
        })
    );
    let _ = reject(&mut state, vec![metadata(&r)]);
}

/// Scenario: Progress crosses known record ends, scan-to-LF boundaries, and clean boundaries.
/// Guarantees: Continuation origin/index cannot regress or disappear before a known record end.
#[test]
fn continuation_transition_matrix() {
    let cases = [
        (FramingResume::Clean, 20, continuation(10, 40, 1), true),
        (FramingResume::Clean, 20, continuation(9, 40, 1), false),
        (continuation(0, 30, 2), 20, continuation(0, 30, 3), true),
        (continuation(0, 30, 2), 20, continuation(0, 30, 2), false),
        (continuation(0, 30, 2), 20, continuation(1, 30, 3), false),
        (continuation(0, 30, 2), 20, continuation(0, 31, 3), false),
        (continuation(0, 30, 2), 20, FramingResume::Clean, false),
        (continuation(0, 30, 2), 30, FramingResume::Clean, true),
        (continuation(0, 30, 2), 40, continuation(30, 50, 1), true),
        (continuation(0, 30, 2), 40, continuation(29, 50, 3), false),
        (continuation(0, 0, 2), 20, continuation(0, 0, 3), true),
        (continuation(0, 0, 2), 20, continuation(0, 0, 2), false),
        (continuation(0, 0, 2), 20, continuation(0, 30, 3), false),
        (continuation(0, 0, 2), 20, continuation(10, 30, 1), true),
        (continuation(0, 0, 2), 20, FramingResume::Clean, true),
    ];
    for (before, end, after, valid) in cases {
        let r = SnapshotRecord {
            framing_resume: before,
            ..record(1)
        };
        let mut state = state(std::slice::from_ref(&r));
        let mut op = progress(&r, end);
        op.new_framing_resume = after;
        if valid {
            apply(&mut state, vec![Operation::UpdateProgress(op)]);
            assert_eq!(state.get(&id(1)).expect("record").framing_resume, after);
        } else {
            let _ = reject(&mut state, vec![Operation::UpdateProgress(op)]);
        }
    }
}

/// Scenario: Zero-delta progress attempts to change the guard or abandon continuation.
/// Guarantees: Zero-delta updates preserve source evidence exactly, including finalization attempts.
#[test]
fn zero_delta_cannot_replace_evidence() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    let mut op = progress(&r, 10);
    op.new_committed_frontier_guard.digest[0] ^= 1;
    let _ = reject(&mut state, vec![Operation::UpdateProgress(op)]);
    let r = SnapshotRecord {
        framing_resume: continuation(0, 0, 1),
        ..r
    };
    let mut state = state_with(std::slice::from_ref(&r), config()).expect("state");
    let mut op = progress(&r, 10);
    let _ = reject(&mut state, vec![Operation::UpdateProgress(op.clone())]);
    op.finalize = true;
    let _ = reject(&mut state, vec![Operation::UpdateProgress(op.clone())]);
    op.finalize = false;
    op.new_framing_resume = r.framing_resume;
    apply(&mut state, vec![Operation::UpdateProgress(op)]);
}

/// Scenario: Table admission fails after an earlier registration or is freed by removal.
/// Guarantees: Record limits include all lifecycle states and failed admission rolls back locator claims.
#[test]
fn record_limit_is_transactional() {
    let a = record(1);
    let b = record(2);
    let mut cfg = config();
    cfg.max_tracked_files = 1;
    let mut state = state_with(&[], cfg).expect("state");
    assert_eq!(
        reject(&mut state, vec![registration(&a), registration(&b)]),
        ReplayError::RecordLimit { max: 1 }
    );
    apply(&mut state, vec![registration(&a)]);
    apply(&mut state, vec![removal(&a, true), registration(&b)]);
    assert_eq!(state.get(&id(2)), Some(&b));
    assert!(matches!(
        state_with(&[a, b], cfg),
        Err(ReplayError::Decode(
            DecodeError::SnapshotRecordCountExceedsLimit { .. }
        ))
    ));
}

/// Scenario: A full table registers a different locator before an administrative removal.
/// Guarantees: Admission uses the final count, while a later failure rolls back both the new record and locator claim.
#[test]
fn full_table_accepts_registration_before_removal() {
    let a = record(1);
    let b = record(2);
    let mut cfg = config();
    cfg.max_tracked_files = 1;
    let mut state = state_with(std::slice::from_ref(&a), cfg).expect("state");
    let Operation::RemoveFile(mut stale) = removal(&a, true) else {
        unreachable!()
    };
    stale.expected_file_epoch += 1;
    let _ = reject(
        &mut state,
        vec![registration(&b), Operation::RemoveFile(stale)],
    );
    apply(&mut state, vec![registration(&b), removal(&a, true)]);
    assert_eq!(state.get(&id(2)), Some(&b));
    assert!(state.get(&id(1)).is_none());
    apply(&mut state, vec![registration(&a), removal(&b, true)]);
    assert_eq!(state.get(&id(1)), Some(&a));
}

/// Scenario: A selected profile changes its evidence window after configuration construction.
/// Guarantees: Replay owns a consistent window/digest pair and preserves old-profile records as incompatible.
#[test]
fn config_derives_both_profile_inputs_together() {
    let mut selected = profile();
    let original_config = ReplayConfig::new(100, 1024 * 1024, &selected).expect("config");
    let mut r = record(1);
    r.fingerprint.resize(33, b'x');
    assert!(state_with(std::slice::from_ref(&r), original_config).is_err());
    selected.fingerprint_bytes = 64;
    let larger = ReplayConfig::new(100, 1024 * 1024, &selected).expect("config");
    let old_profile = state_with(std::slice::from_ref(&r), larger).expect("old profile preserved");
    assert_eq!(old_profile.profile_compatible(&r.file_id), Some(false));
    r.framing_profile_digest = selected.digest().expect("digest");
    let matching = state_with(std::slice::from_ref(&r), larger).expect("larger compatible window");
    assert_eq!(matching.profile_compatible(&r.file_id), Some(true));
    r.fingerprint.resize(65, b'x');
    assert!(state_with(std::slice::from_ref(&r), larger).is_err());
    selected.fingerprint_bytes = 15;
    assert!(ReplayConfig::new(100, 1024, &selected).is_err());
}

/// Scenario: A progress transaction would finalize one file but later targets a stale epoch.
/// Guarantees: Validation precedes every mutation, including release of the finalized file's locator claim.
#[test]
fn failed_progress_preserves_finalization_and_locator_claims() {
    let a = record(1);
    let b = record(2);
    let mut state = state(&[a.clone(), b.clone()]);
    let mut finalize = progress(&a, 10);
    finalize.finalize = true;
    let mut stale = progress(&b, 20);
    stale.expected_file_epoch += 1;
    let _ = reject(
        &mut state,
        vec![
            Operation::UpdateProgress(finalize.clone()),
            Operation::UpdateProgress(stale),
        ],
    );
    let mut c = record(3);
    c.locator = a.locator;
    let _ = reject(&mut state, vec![registration(&c)]);
    apply(&mut state, vec![Operation::UpdateProgress(finalize)]);
    apply(&mut state, vec![registration(&c)]);
}

/// Scenario: Stored framing profiles differ from the selected configuration.
/// Guarantees: Incompatible files remain identifiable and replayable without applying the new fingerprint window to old profiles.
#[test]
fn incompatible_profiles_remain_per_file() {
    let mut old = record(1);
    old.framing_profile_version = 2;
    old.fingerprint.resize(40, b'x');
    let good = record(2);
    let mut state = state(&[old.clone(), good]);
    assert_eq!(state.profile_compatible(&id(1)), Some(false));
    assert_eq!(state.profile_compatible(&id(2)), Some(true));
    assert_eq!(state.profile_compatible(&id(3)), None);
    apply(&mut state, vec![metadata(&old)]);
    old.framing_profile_version = 1;
    assert!(state_with(&[old.clone()], config()).is_err());
    old.framing_profile_digest[0] ^= 1;
    assert!(state_with(&[old], config()).is_ok());
}

/// Scenario: A checkpoint prefix has the wrong namespace or exceeds its admitted byte budget.
/// Guarantees: Snapshot loading fails before a replay table becomes available.
#[test]
fn snapshot_namespace_and_byte_budget_are_enforced() {
    let bytes = encode_checkpoint(1, NAMESPACE, &[record(1)]).expect("checkpoint");
    assert!(matches!(
        ReplayState::from_checkpoint(&bytes, "other", config()),
        Err(ReplayError::Decode(DecodeError::NamespaceMismatch { .. }))
    ));
    assert!(
        ReplayState::from_checkpoint(
            &bytes,
            NAMESPACE,
            ReplayConfig::new(100, 1, &profile()).expect("profile")
        )
        .is_err()
    );
    assert!(
        ReplayConfig::new(
            100,
            1024,
            &FramingProfileParams {
                fingerprint_bytes: 15,
                ..profile()
            }
        )
        .is_err()
    );
}

/// Scenario: Checksum-valid quarantine WAL entries carry reserved or extension reason codes.
/// Guarantees: Reserved reasons fail semantic replay while unknown nonreserved diagnostic reasons remain supported.
#[test]
fn quarantine_reason_codes_are_validated_at_replay() {
    let r = record(1);
    for reason in [0u16, 4, 5, 0x1234] {
        let mut state = state(std::slice::from_ref(&r));
        let mut bytes = tx(1, vec![quarantine(&r)]);
        mutate_payload(&mut bytes, |p| {
            p[21..23].copy_from_slice(&reason.to_be_bytes())
        });
        let result = state.replay_next(&bytes);
        if reason == 0 || reason == 4 {
            assert!(matches!(result, Err(ReplayError::InvalidState { .. })));
            assert_eq!(state.get(&id(1)), Some(&r));
        } else {
            assert!(result.is_ok());
        }
    }
}

/// Scenario: A snapshot contains the reserved quarantine reason and a following removal could hide it.
/// Guarantees: Snapshot recovery rejects the unsupported evidence before any WAL operation is applied.
#[test]
fn reserved_snapshot_evidence_cannot_be_removed_by_replay() {
    let r = record(1);
    let mut initial = state(std::slice::from_ref(&r));
    apply(&mut initial, vec![quarantine(&r)]);
    let mut bytes = encode_checkpoint(1, NAMESPACE, &contents(&initial)).expect("snapshot");
    // Locate the one immutable reason field from the fixed record layout and bounded fields.
    let payload_start = 24 + 60 + 4;
    let reason =
        payload_start + 16 + 4 + 8 + 34 + 2 + r.fingerprint.len() + 4 + 17 + 2 + 32 + 1 + 1;
    bytes[reason..reason + 2].copy_from_slice(&4u16.to_be_bytes());
    let footer = bytes.len() - 24;
    let record_crc = crc32c(&bytes[84..footer - 4]);
    bytes[footer - 4..footer].copy_from_slice(&record_crc.to_be_bytes());
    // The unchanged footer has its own checksum; it does not cover record bytes.
    bytes.extend_from_slice(&tx(
        1,
        vec![removal(initial.get(&id(1)).expect("record"), true)],
    ));
    assert!(matches!(
        ReplayState::from_checkpoint(&bytes, NAMESPACE, config()),
        Err(ReplayError::InvalidState { .. })
    ));
}

/// Scenario: Checksummed WAL operations carry impossible registration or truncate epochs.
/// Guarantees: Replay independently validates producer-only rules and rejects epoch overflow without mutation.
#[test]
fn decoded_epoch_and_reset_rules_are_not_trusted() {
    let r = record(1);
    let mut empty = state(&[]);
    let mut register = tx(1, vec![registration(&r)]);
    mutate_payload(&mut register, |p| {
        p[17..21].copy_from_slice(&2u32.to_be_bytes())
    });
    assert!(matches!(
        empty.replay_next(&register),
        Err(ReplayError::InvalidState { .. })
    ));
    assert_eq!(empty.records().len(), 0);

    let reset = ResetAfterTruncate {
        file_id: id(1),
        expected_active_epoch: 1,
        observed_truncated_size: 0,
        resulting_epoch: 2,
        new_committed_offset: 0,
        new_framing_resume: FramingResume::Clean,
        new_fingerprint: vec![],
        reset_time_unix_nano: 1,
        reason_code: 1,
    };
    for variation in 0..3 {
        let mut bytes = tx(1, vec![Operation::ResetAfterTruncate(reset.clone())]);
        let mut stored = r.clone();
        mutate_payload(&mut bytes, |p| match variation {
            0 => p[29..33].copy_from_slice(&3u32.to_be_bytes()),
            1 => {
                let end = p.len();
                p[end - 2..].fill(0);
            }
            _ => {
                stored.file_epoch = u32::MAX;
                p[17..21].copy_from_slice(&u32::MAX.to_be_bytes());
                p[29..33].fill(0);
            }
        });
        let mut state = state(std::slice::from_ref(&stored));
        assert!(matches!(
            state.replay_next(&bytes),
            Err(ReplayError::InvalidState { .. })
        ));
        assert_eq!(state.get(&id(1)), Some(&stored));
        assert_eq!(state.last_sequence(), 0);
    }
}

/// Scenario: A producer-invalid finalization retains a continuation in a checksum-valid frame.
/// Guarantees: Replay enforces reachable finalized state even when the wire structure is valid.
#[test]
fn decoded_finalization_cannot_retain_continuation() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    let mut op = progress(&r, 20);
    op.new_framing_resume = continuation(10, 30, 1);
    let mut bytes = tx(1, vec![Operation::UpdateProgress(op)]);
    mutate_payload(&mut bytes, |p| {
        let last = p.len() - 1;
        p[last] = 1;
    });
    assert!(matches!(
        state.replay_next(&bytes),
        Err(ReplayError::InvalidState { .. })
    ));
    assert_eq!(state.get(&id(1)), Some(&r));
}

/// Scenario: Every mutation other than registration targets absent or finalized state.
/// Guarantees: Operations cannot implicitly create records, and finalized state accepts only matching administrative removal.
#[test]
fn absent_and_finalized_transition_matrix() {
    let r = record(1);
    let finalized = SnapshotRecord {
        lifecycle_state: LifecycleState::RotatedFinalized,
        ..r.clone()
    };
    let operations = vec![
        Operation::UpdateProgress(progress(&r, 20)),
        metadata(&r),
        quarantine(&r),
        Operation::UpdateFingerprint(UpdateFingerprint {
            file_id: id(1),
            expected_file_epoch: 1,
            expected_fingerprint: r.fingerprint.clone(),
            new_fingerprint: b"prefix-long".to_vec(),
        }),
        Operation::ResetAfterTruncate(ResetAfterTruncate {
            file_id: id(1),
            expected_active_epoch: 1,
            observed_truncated_size: 0,
            resulting_epoch: 2,
            new_committed_offset: 0,
            new_framing_resume: FramingResume::Clean,
            new_fingerprint: vec![],
            reset_time_unix_nano: 1,
            reason_code: 1,
        }),
        Operation::ResetQuarantinedFile(reset(&r, ResetQuarantineAction::ResetToBeginning)),
        removal(&r, false),
    ];
    for records in [&[][..], std::slice::from_ref(&finalized)] {
        let mut state = state(records);
        for operation in &operations {
            let _ = reject(&mut state, vec![operation.clone()]);
        }
    }
    let mut state = state(std::slice::from_ref(&finalized));
    apply(&mut state, vec![removal(&finalized, true)]);
    assert_eq!(state.records().len(), 0);
}

/// Scenario: A stale administrative removal follows an earlier staged metadata change.
/// Guarantees: Removal checks the stored epoch and state and rolls back preceding work on failure.
#[test]
fn removal_preconditions_protect_stored_state() {
    let r = record(1);
    let mut state = state(std::slice::from_ref(&r));
    let Operation::RemoveFile(mut remove) = removal(&r, true) else {
        unreachable!()
    };
    remove.expected_file_epoch = 2;
    let _ = reject(
        &mut state,
        vec![metadata(&r), Operation::RemoveFile(remove.clone())],
    );
    remove.expected_file_epoch = 1;
    remove.expected_prior_state = LifecycleState::Quarantined;
    let _ = reject(
        &mut state,
        vec![metadata(&r), Operation::RemoveFile(remove)],
    );
}

/// Scenario: A maximum-size progress transaction updates 4096 separate files.
/// Guarantees: The format's full atomic progress width is supported without losing records or admitting duplicate updates.
#[test]
fn maximum_progress_width_is_atomic() {
    let records: Vec<_> = (0..4096u64)
        .map(|n| {
            let mut bytes = [0; 16];
            bytes[..8].copy_from_slice(&n.to_be_bytes());
            SnapshotRecord {
                file_id: FileId::from_bytes(bytes),
                locator: Locator::PosixDevIno { dev: 1, ino: n },
                ..record(1)
            }
        })
        .collect();
    let cfg = ReplayConfig::new(4096, 2 * 1024 * 1024, &profile()).expect("profile");
    let mut state = state_with(&records, cfg).expect("state");
    let operations: Vec<_> = records
        .iter()
        .map(|r| Operation::UpdateProgress(progress(r, 20)))
        .collect();
    let mut invalid = operations.clone();
    if let Some(Operation::UpdateProgress(last)) = invalid.last_mut() {
        last.expected_file_epoch = 2;
    }
    let _ = reject(&mut state, invalid);
    apply(&mut state, operations);
    assert_eq!(state.records().len(), 4096);
    assert!(state.records().all(|r| r.committed_offset == 20));
}

/// Scenario: A valid earlier transaction is followed by a semantically invalid transaction.
/// Guarantees: The failing transaction rolls back alone; earlier validated WAL history remains intact.
#[test]
fn later_failure_preserves_prior_transactions() {
    let a = record(1);
    let mut state = state(&[]);
    apply(&mut state, vec![registration(&a)]);
    let mut stale = progress(&a, 20);
    stale.expected_file_epoch = 2;
    let _ = reject(&mut state, vec![Operation::UpdateProgress(stale)]);
    assert_eq!(state.last_sequence(), 1);
    assert_eq!(state.get(&id(1)), Some(&a));
}

/// Scenario: Independent Python fixtures include both valid registration and stale stored progress.
/// Guarantees: Replay accepts externally generated valid bytes but rejects structurally valid WAL whose preconditions do not match its snapshot.
#[test]
fn independent_fixtures_exercise_semantic_boundary() {
    let (mut empty, _) = ReplayState::from_checkpoint(
        include_bytes!("fixtures/checkpoint-empty.bin"),
        "app-logs",
        config(),
    )
    .expect("empty fixture");
    let _ = empty
        .replay_next(include_bytes!("fixtures/transaction-register-file.bin"))
        .expect("registration fixture");
    let registered_id = FileId::from_bytes(11u128.to_be_bytes());
    let registered = empty.get(&registered_id).expect("fixture identity");
    assert_eq!(registered.file_epoch, 1);
    assert_eq!(registered.committed_offset, 0);
    assert_eq!(registered.fingerprint, b"0123456789abcdef");

    // This codec fixture deliberately combines an epoch-two/offset-four snapshot
    // with an epoch-one/offset-zero operation. Structural validity is insufficient.
    let bytes = include_bytes!("fixtures/checkpoint-with-wal.bin");
    let (mut state, wal_offset) =
        ReplayState::from_checkpoint(bytes, "app-logs", config()).expect("prefix");
    let before = contents(&state);
    assert!(matches!(
        state.replay_next(&bytes[wal_offset..]),
        Err(ReplayError::InvalidState { .. })
    ));
    assert_eq!(contents(&state), before);
    assert_eq!(state.last_sequence(), 0);
}

/// Scenario: An independently generated keep-failed operation carries unequal epochs and changed evidence.
/// Guarantees: Replay rejects the existing codec fixture with KeepFailedStateChange and preserves the quarantined record.
#[test]
fn independent_keep_failed_mutation_is_rejected() {
    let stored = SnapshotRecord {
        file_id: FileId::from_bytes(17u128.to_be_bytes()),
        committed_offset: 4,
        committed_frontier_guard: CommittedFrontierGuard::compute(4, b"abc\n").expect("guard"),
        fingerprint: b"0123456789abcdef".to_vec(),
        lifecycle_state: LifecycleState::Quarantined,
        quarantine_evidence: Some(QuarantineEvidence {
            reason_code: 1,
            observed_size: 88,
            quarantine_epoch: 1,
            quarantine_time_unix_nano: 50,
        }),
        ..record(1)
    };
    let bytes = encode_checkpoint(1, "app-logs", std::slice::from_ref(&stored)).expect("snapshot");
    let (mut state, _) =
        ReplayState::from_checkpoint(&bytes, "app-logs", config()).expect("prefix");
    assert_eq!(
        state.replay_next(include_bytes!(
            "fixtures/transaction-keep-failed-mutation.bin"
        )),
        Err(ReplayError::KeepFailedStateChange {
            file_id: stored.file_id
        })
    );
    assert_eq!(state.get(&stored.file_id), Some(&stored));
    let _ = state
        .replay_next(include_bytes!(
            "fixtures/transaction-reset-quarantined-keep-failed.bin"
        ))
        .expect("exact keep-failed");
    assert_eq!(state.get(&stored.file_id), Some(&stored));
}
