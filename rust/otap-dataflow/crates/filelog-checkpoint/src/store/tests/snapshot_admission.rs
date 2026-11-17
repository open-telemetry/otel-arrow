// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    LifecycleState, QuarantineFile, RemoveFile, ResetAfterTruncate, ResetQuarantineAction,
    ResetQuarantinedFile, UpdateFingerprint, UpdateMetadata,
};

fn id() -> FileId {
    FileId::from_bytes([1; 16])
}

fn snapshot_size(store: &CheckpointStore) -> u64 {
    let state = store.state().unwrap();
    let records: Vec<_> = state.records().cloned().collect();
    let encoded = encode_checkpoint(state.generation(), ID, &records).unwrap();
    let size = (encoded.len() - CHECKPOINT_HEADER_BYTES) as u64;
    assert_eq!(state.snapshot_bytes(), size);
    size
}

fn registration_size() -> u64 {
    let bytes = encode_checkpoint(0, ID, &[]).unwrap();
    let (mut state, _) = ReplayState::from_checkpoint(&bytes, ID, config().replay).unwrap();
    let tx = encode_transaction(&Transaction {
        sequence: 1,
        operations: vec![registration()],
    })
    .unwrap();
    let _ = state.replay_next(&tx).unwrap();
    state.snapshot_bytes()
}

fn bounded(root: &StateRoot, bytes: u64) -> CheckpointStore {
    CheckpointStore::open(
        root,
        ID,
        StoreConfig {
            replay: ReplayConfig::new(100, bytes, &profile()).unwrap(),
            ..config()
        },
    )
    .unwrap()
}

fn unchanged_rejection(store: &mut CheckpointStore, dir: &Path, ops: Vec<Operation>, size: u64) {
    let before = fs::read(namespace(dir).join(DB)).unwrap();
    let records: Vec<_> = store.state().unwrap().records().cloned().collect();
    let sequence = store.state().unwrap().last_sequence();
    let accounted = snapshot_size(store);
    let wal = store.wal_bytes();
    let synced = store.synced_sequence();
    let events = store.io.events.len();
    let error = store.append(ops, Durability::Sync).unwrap_err();
    assert!(
        matches!(error, StoreError::SnapshotLimit { size: actual, max }
        if actual == size && max == store.config.replay.max_snapshot_bytes),
        "{error:?}"
    );
    assert!(error.to_string().contains(&size.to_string()));
    assert!(
        error
            .to_string()
            .contains(&store.config.replay.max_snapshot_bytes.to_string())
    );
    assert_eq!(fs::read(namespace(dir).join(DB)).unwrap(), before);
    assert_eq!(
        store
            .state()
            .unwrap()
            .records()
            .cloned()
            .collect::<Vec<_>>(),
        records
    );
    assert_eq!(store.state().unwrap().last_sequence(), sequence);
    assert_eq!(snapshot_size(store), accounted);
    assert_eq!(store.wal_bytes(), wal);
    assert_eq!(store.synced_sequence(), synced);
    assert_eq!(store.io.events.len(), events);
    assert!(!store.has_unresolved_append());
}

fn metadata(path: &[u8]) -> Operation {
    Operation::UpdateMetadata(UpdateMetadata {
        file_id: id(),
        expected_prior_state: LifecycleState::Active,
        expected_file_epoch: 1,
        last_seen_time_unix_nano: 5,
        advisory_path: Some(AdvisoryPath::from_unix_bytes(path).unwrap()),
    })
}

fn quarantine() -> Operation {
    Operation::QuarantineFile(QuarantineFile {
        file_id: id(),
        expected_file_epoch: 1,
        reason_code: 1,
        locator: Locator::PosixDevIno { dev: 1, ino: 1 },
        observed_size: 10,
        quarantine_epoch: 1,
        quarantine_time_unix_nano: 5,
    })
}

fn continuation() -> Operation {
    let mut op = progress(0, 10);
    if let Operation::UpdateProgress(p) = &mut op {
        p.new_framing_resume = FramingResume::Continuation {
            record_start_offset: 0,
            record_end_offset: 0,
            next_fragment_index: 1,
        };
    }
    op
}

fn remove(file_id: FileId, epoch: u32) -> Operation {
    Operation::RemoveFile(RemoveFile {
        file_id,
        expected_file_epoch: epoch,
        expected_prior_state: LifecycleState::Active,
        removal_reason: 1,
        removal_time_unix_nano: 10,
        administrative: true,
        namespace_id: Some(ID.into()),
        audit_reason: Some("approved".into()),
    })
}

/// Scenario: Registration, fingerprint/path growth, continuation and quarantine reach exact limits.
/// Guarantees: Equality is admitted; one byte over rejects without disk, record, sequence or accounting changes.
#[test]
fn snapshot_admission_exact_boundaries() {
    let base = registration_size();
    for (op, extra) in [
        (metadata(b"/logs"), 1),
        (
            Operation::UpdateFingerprint(UpdateFingerprint {
                file_id: id(),
                expected_file_epoch: 1,
                expected_fingerprint: b"prefix".to_vec(),
                new_fingerprint: b"prefix!".to_vec(),
            }),
            1,
        ),
        (continuation(), 20),
        (quarantine(), 22),
    ] {
        for room in [extra - 1, extra] {
            let (dir, root) = root();
            let mut store = bounded(&root, base + room);
            let _ = store
                .append(vec![registration()], Durability::Sync)
                .unwrap();
            if room == extra {
                let _ = store.append(vec![op.clone()], Durability::Sync).unwrap();
                assert_eq!(snapshot_size(&store), base + extra);
                let _ = store.compact().unwrap();
                assert_eq!(snapshot_size(&store), base + extra);
            } else {
                unchanged_rejection(&mut store, dir.path(), vec![op.clone()], base + extra);
            }
        }
    }
    for budget in [base - 1, base] {
        let (dir, root) = root();
        let mut store = bounded(&root, budget);
        if budget < base {
            unchanged_rejection(&mut store, dir.path(), vec![registration()], base);
        } else {
            let _ = store
                .append(vec![registration()], Durability::Sync)
                .unwrap();
            assert_eq!(snapshot_size(&store), base);
        }
    }
}

/// Scenario: Multiple transitions touch the same record, reset evidence, and remove/re-register it atomically.
/// Guarantees: Incremental accounting matches wire encoding after every operation, including shrinking framing and quarantine.
#[test]
fn snapshot_accounting_all_transitions() {
    let (_dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let base = snapshot_size(&store);
    let _ = store
        .append(vec![continuation()], Durability::Deferred)
        .unwrap();
    assert_eq!(snapshot_size(&store), base + 20);
    let _ = store
        .append(vec![progress(10, 20)], Durability::Deferred)
        .unwrap();
    assert_eq!(snapshot_size(&store), base);
    let _ = store
        .append(
            vec![metadata(b"/long-path"), metadata(b"/x")],
            Durability::Sync,
        )
        .unwrap();
    assert_eq!(snapshot_size(&store), base - 2);
    let _ = store.append(vec![quarantine()], Durability::Sync).unwrap();
    assert_eq!(snapshot_size(&store), base - 2 + 22);
    let _ = store.append(vec![quarantine()], Durability::Sync).unwrap();
    assert_eq!(snapshot_size(&store), base - 2 + 22);
    let _ = store
        .append(
            vec![Operation::ResetQuarantinedFile(ResetQuarantinedFile {
                file_id: id(),
                expected_quarantine_epoch: 1,
                action: ResetQuarantineAction::ResetToBeginning,
                resulting_epoch: 2,
                resulting_offset: 0,
                new_committed_frontier_guard: CommittedFrontierGuard::empty(),
                new_framing_resume: FramingResume::Clean,
                new_fingerprint: b"p".to_vec(),
                action_time_unix_nano: 6,
                namespace_id: ID.into(),
                audit_reason: "approved".into(),
            })],
            Durability::Sync,
        )
        .unwrap();
    assert_eq!(snapshot_size(&store), base - 7);
    let _ = store
        .append(
            vec![Operation::ResetAfterTruncate(ResetAfterTruncate {
                file_id: id(),
                expected_active_epoch: 2,
                observed_truncated_size: 0,
                resulting_epoch: 3,
                new_committed_offset: 0,
                new_framing_resume: FramingResume::Clean,
                new_fingerprint: vec![],
                reset_time_unix_nano: 7,
                reason_code: 1,
            })],
            Durability::Sync,
        )
        .unwrap();
    assert_eq!(snapshot_size(&store), base - 8);
    let _ = store
        .append(vec![remove(id(), 3), registration()], Durability::Sync)
        .unwrap();
    assert_eq!(snapshot_size(&store), base);
    let _ = store.compact_with_retention(&[id()]).unwrap();
    assert_eq!(
        snapshot_size(&store),
        (SNAPSHOT_HEADER_BYTES + SNAPSHOT_FOOTER_BYTES) as u64
    );
}

/// Scenario: An older writer's WAL reconstructs a table above today's snapshot budget.
/// Guarantees: Startup remains compatible; neutral/reducing writes work, growth rejects, and vetted retention restores compaction.
#[test]
fn snapshot_legacy_over_budget_wal() {
    let (dir, root) = root();
    drop(store(&root));
    let mut second = registration();
    let second_id = FileId::from_bytes([2; 16]);
    if let Operation::RegisterFile(r) = &mut second {
        r.file_id = second_id;
        r.locator = Locator::PosixDevIno { dev: 1, ino: 2 };
    }
    let tx = encode_transaction(&Transaction {
        sequence: 1,
        operations: vec![registration(), second],
    })
    .unwrap();
    append_bytes(&namespace(dir.path()).join(DB), &tx);
    let base = registration_size();
    let mut store = bounded(&root, base);
    let over = snapshot_size(&store);
    assert!(over > base);
    assert!(
        matches!(store.compact(), Err(StoreError::SnapshotLimit { size, max }) if size == over && max == base)
    );
    unchanged_rejection(&mut store, dir.path(), vec![continuation()], over + 20);
    let _ = store
        .append(vec![progress(0, 10)], Durability::Deferred)
        .unwrap();
    assert_eq!(snapshot_size(&store), over);
    let _ = store
        .append(vec![metadata(b"/x")], Durability::Sync)
        .unwrap();
    assert_eq!(snapshot_size(&store), over - 2);
    drop(store);
    let mut store = bounded(&root, base);
    assert_eq!(snapshot_size(&store), over - 2);
    let _ = store.compact_with_retention(&[second_id]).unwrap();
    assert_eq!(snapshot_size(&store), base - 2);
    let _ = store
        .append(vec![metadata(b"/log")], Durability::Sync)
        .unwrap();
    assert_eq!(snapshot_size(&store), base);
    let _ = store
        .append(vec![remove(id(), 1)], Durability::Sync)
        .unwrap();
    assert_eq!(snapshot_size(&store), 84);
}

/// Scenario: An admitted size-changing append fails with either a torn or complete WAL frame.
/// Guarantees: Resolution and restart reconstruct the same exact size once, without double charging.
#[test]
fn snapshot_accounting_pending_recovery() {
    for written in [0, 17, usize::MAX] {
        let (_dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(vec![registration()], Durability::Sync)
            .unwrap();
        let base = snapshot_size(&store);
        store.inject_write_failure_after(written);
        assert!(
            store
                .append(vec![continuation()], Durability::Sync)
                .is_err()
        );
        let _ = store.resolve_pending().unwrap();
        assert_eq!(snapshot_size(&store), base + 20);
        drop(store);
        assert_eq!(snapshot_size(&self::store(&root)), base + 20);
    }
}

/// Scenario: Truncate and quarantine resets exchange framing/evidence bytes for a larger fingerprint.
/// Guarantees: Resets cannot bypass size admission, and accepted shrinking resets release their exact wire charge.
#[test]
fn snapshot_reset_growth_and_shrinkage() {
    let (dir, root) = root();
    let base = registration_size();
    let mut store = bounded(&root, base + 20);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let _ = store
        .append(vec![continuation()], Durability::Sync)
        .unwrap();
    let truncate = |fingerprint| {
        Operation::ResetAfterTruncate(ResetAfterTruncate {
            file_id: id(),
            expected_active_epoch: 1,
            observed_truncated_size: 0,
            resulting_epoch: 2,
            new_committed_offset: 0,
            new_framing_resume: FramingResume::Clean,
            new_fingerprint: fingerprint,
            reset_time_unix_nano: 7,
            reason_code: 1,
        })
    };
    unchanged_rejection(
        &mut store,
        dir.path(),
        vec![truncate(vec![b'x'; 32])],
        base + 26,
    );
    let _ = store
        .append(vec![truncate(b"p".to_vec())], Durability::Sync)
        .unwrap();
    assert_eq!(snapshot_size(&store), base - 5);
    let mut q = quarantine();
    if let Operation::QuarantineFile(q) = &mut q {
        q.expected_file_epoch = 2;
        q.quarantine_epoch = 2;
    }
    let _ = store.append(vec![q], Durability::Sync).unwrap();
    assert_eq!(snapshot_size(&store), base + 17);
    let reset = |fingerprint| {
        Operation::ResetQuarantinedFile(ResetQuarantinedFile {
            file_id: id(),
            expected_quarantine_epoch: 2,
            action: ResetQuarantineAction::ResetToBeginning,
            resulting_epoch: 3,
            resulting_offset: 0,
            new_committed_frontier_guard: CommittedFrontierGuard::empty(),
            new_framing_resume: FramingResume::Clean,
            new_fingerprint: fingerprint,
            action_time_unix_nano: 8,
            namespace_id: ID.into(),
            audit_reason: "approved".into(),
        })
    };
    unchanged_rejection(
        &mut store,
        dir.path(),
        vec![reset(vec![b'x'; 32])],
        base + 26,
    );
    let _ = store
        .append(vec![reset(vec![b'x'; 20])], Durability::Sync)
        .unwrap();
    assert_eq!(snapshot_size(&store), base + 14);
}

/// Scenario: A bounded atomic group temporarily enlarges a record and then returns to its admitted size.
/// Guarantees: Snapshot admission charges the resulting table once, without rejecting or double-counting intermediate state.
#[test]
fn snapshot_admission_uses_final_group_size() {
    let (_dir, root) = root();
    let base = registration_size();
    let mut store = bounded(&root, base);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let _ = store
        .append(
            vec![metadata(b"/a-longer-path"), metadata(b"/log")],
            Durability::Sync,
        )
        .unwrap();
    assert_eq!(snapshot_size(&store), base);
    let mut replacement = registration();
    let next_id = FileId::from_bytes([2; 16]);
    if let Operation::RegisterFile(r) = &mut replacement {
        r.file_id = next_id;
    }
    let mut removal = remove(id(), 1);
    if let Operation::RemoveFile(r) = &mut removal {
        r.administrative = false;
        r.namespace_id = None;
        r.audit_reason = None;
    }
    let _ = store
        .append(vec![removal, replacement], Durability::Sync)
        .unwrap();
    assert_eq!(snapshot_size(&store), base);
    assert!(store.state().unwrap().get(&id()).is_none());
    assert!(store.state().unwrap().get(&next_id).is_some());
}
