// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{LifecycleState, QuarantineEvidence, SnapshotRecord};
use std::os::unix::fs::MetadataExt;

fn file_id(n: u128) -> FileId {
    FileId::from_bytes(n.to_be_bytes())
}

fn numbered_registration(n: u128) -> Operation {
    let Operation::RegisterFile(mut registration) = registration() else {
        unreachable!()
    };
    registration.file_id = file_id(n);
    registration.locator = Locator::PosixDevIno {
        dev: 1,
        ino: n as u64,
    };
    Operation::RegisterFile(registration)
}

fn records(store: &CheckpointStore) -> Vec<SnapshotRecord> {
    let mut records: Vec<_> = store.state().unwrap().records().cloned().collect();
    records.sort_unstable_by_key(|record| record.file_id);
    records
}

fn assert_blocked(store: &mut CheckpointStore) {
    assert!(matches!(
        store.state(),
        Err(StoreError::CompactionRecoveryRequired)
    ));
    assert!(matches!(
        store.append(vec![registration()], Durability::Sync),
        Err(StoreError::CompactionRecoveryRequired)
    ));
    assert!(matches!(
        store.sync(),
        Err(StoreError::CompactionRecoveryRequired)
    ));
    assert!(matches!(
        store.compact(),
        Err(StoreError::CompactionRecoveryRequired)
    ));
    assert!(matches!(
        store.resolve_pending(),
        Err(StoreError::CompactionRecoveryRequired)
    ));
}

/// Scenario: Compaction snapshots deferred progress before another append and restart.
/// Guarantees: Every saved field survives, WAL resets, and the new append reaches the replacement inode.
#[test]
fn compaction_append_restart() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let mut update = progress(0, 10);
    if let Operation::UpdateProgress(progress) = &mut update {
        progress.new_framing_resume = FramingResume::Continuation {
            record_start_offset: 0,
            record_end_offset: 0,
            next_fragment_index: 1,
        };
    }
    let _ = store.append(vec![update], Durability::Deferred).unwrap();
    let expected = records(&store);
    let path = namespace(dir.path()).join(DB);
    let old_inode = fs::metadata(&path).unwrap().ino();
    let start = store.io.events.len();
    let receipt = store.compact().unwrap();
    assert_eq!(
        receipt,
        CompactionReceipt {
            generation: 1,
            removed_records: 0
        }
    );
    assert_eq!(records(&store), expected);
    assert_eq!(store.wal_bytes(), 0);
    assert_eq!(store.synced_sequence(), 0);
    assert_eq!(store.state().unwrap().last_sequence(), 0);
    assert_ne!(fs::metadata(&path).unwrap().ino(), old_inode);
    let events = &store.io.events[start..];
    for (before, after) in [
        ("compaction file sync", "pre-compaction file sync"),
        ("pre-compaction file sync", "before checkpoint replacement"),
        ("after checkpoint replacement", "compaction directory sync"),
        ("compaction directory sync", "reopen compacted checkpoint"),
    ] {
        assert!(
            events.iter().position(|e| *e == before).unwrap()
                < events.iter().position(|e| *e == after).unwrap()
        );
    }
    assert_eq!(
        store
            .append(vec![progress(10, 20)], Durability::Sync)
            .unwrap()
            .sequence,
        1
    );
    let expected = records(&store);
    drop(store);
    let mut reopened = self::store(&root);
    assert_eq!(records(&reopened), expected);
    assert_eq!(reopened.state().unwrap().generation(), 1);
    assert_eq!(reopened.state().unwrap().last_sequence(), 1);
    assert_eq!(reopened.compact().unwrap().generation, 2);
}

/// Scenario: More than one non-progress transaction's worth of records is vetted for removal.
/// Guarantees: One generation removes the entire set and preserves remaining state across restart.
#[test]
fn compaction_retention_is_atomic_beyond_transaction_limit() {
    let (_dir, root) = root();
    let config = StoreConfig {
        replay: ReplayConfig::new(400, 1024 * 1024, &profile()).unwrap(),
        ..config()
    };
    let mut store = CheckpointStore::open(&root, ID, config).unwrap();
    for range in [1..=200, 201..=301] {
        let _ = store
            .append(range.map(numbered_registration).collect(), Durability::Sync)
            .unwrap();
    }
    let survivor = store.state().unwrap().get(&file_id(301)).unwrap().clone();
    let ids: Vec<_> = (1..=300).map(file_id).collect();
    let receipt = store.compact_with_retention(&ids).unwrap().unwrap();
    assert_eq!(receipt.removed_records, 300);
    assert_eq!(receipt.generation, 1);
    assert_eq!(store.state().unwrap().records().len(), 1);
    assert_eq!(store.wal_bytes(), 0);
    drop(store);
    let reopened = CheckpointStore::open(&root, ID, config).unwrap();
    assert_eq!(records(&reopened), vec![survivor]);
}

/// Scenario: Empty retention maintenance runs on an old record without runtime absence evidence.
/// Guarantees: Timestamps grant no removal; empty work performs no I/O, while ordinary compaction works.
#[test]
fn compaction_empty_retention_does_nothing() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let before = fs::read(namespace(dir.path()).join(DB)).unwrap();
    let events = store.io.events.len();
    assert_eq!(store.compact_with_retention(&[]).unwrap(), None);
    assert_eq!(store.io.events.len(), events);
    assert_eq!(fs::read(namespace(dir.path()).join(DB)).unwrap(), before);
    assert_eq!(store.compact().unwrap().generation, 1);
    assert_eq!(store.state().unwrap().records().len(), 1);
}

/// Scenario: A caller proposes a set containing an unknown ID, duplicate, or quarantined record.
/// Guarantees: The whole set is rejected without writing, and ordinary compaction preserves quarantine.
#[test]
fn compaction_rejects_invalid_retention_sets() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(
            vec![numbered_registration(1), numbered_registration(2)],
            Durability::Sync,
        )
        .unwrap();
    let _ = store
        .append(
            vec![Operation::QuarantineFile(crate::QuarantineFile {
                file_id: file_id(2),
                expected_file_epoch: 1,
                reason_code: 1,
                locator: Locator::PosixDevIno { dev: 1, ino: 2 },
                observed_size: 42,
                quarantine_epoch: 1,
                quarantine_time_unix_nano: 100,
            })],
            Durability::Sync,
        )
        .unwrap();
    let before = fs::read(namespace(dir.path()).join(DB)).unwrap();
    let expected = records(&store);
    for ids in [
        vec![file_id(1), file_id(3)],
        vec![file_id(1), file_id(1)],
        vec![file_id(1), file_id(2)],
    ] {
        assert!(matches!(
            store.compact_with_retention(&ids),
            Err(StoreError::InvalidRetention(_))
        ));
        assert_eq!(records(&store), expected);
        assert_eq!(fs::read(namespace(dir.path()).join(DB)).unwrap(), before);
        assert!(!namespace(dir.path()).join(COMPACT).exists());
    }
    let _ = store.compact().unwrap();
    assert_eq!(records(&store), expected);
}

/// Scenario: Snapshot generation reaches u64::MAX before ordinary or retention compaction.
/// Guarantees: Overflow changes neither disk nor live state and creates no temporary.
#[test]
fn compaction_generation_overflow() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    fs::write(&path, encode_checkpoint(u64::MAX, ID, &[]).unwrap()).unwrap();
    let mut store = store(&root);
    assert!(matches!(
        store.compact(),
        Err(StoreError::GenerationExhausted)
    ));
    assert_eq!(store.state().unwrap().generation(), u64::MAX);
    assert!(!namespace(dir.path()).join(COMPACT).exists());
}

/// Scenario: WAL mutations grow the snapshot beyond its independently configured byte budget.
/// Guarantees: Compaction rejects before allocating the complete snapshot, writing, or advancing generation.
#[test]
fn compaction_snapshot_admission() {
    let (dir, root) = root();
    let config = StoreConfig {
        replay: ReplayConfig::new(100, 84, &profile()).unwrap(),
        ..config()
    };
    let mut store = CheckpointStore::open(&root, ID, config).unwrap();
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let before = fs::read(namespace(dir.path()).join(DB)).unwrap();
    assert!(matches!(
        store.compact(),
        Err(StoreError::RecoveryLimit("compaction snapshot bytes"))
    ));
    assert_eq!(store.state().unwrap().generation(), 0);
    assert_eq!(fs::read(namespace(dir.path()).join(DB)).unwrap(), before);
    assert!(!namespace(dir.path()).join(COMPACT).exists());
}

/// Scenario: A temporary write stops at zero, partial-header, or partial-snapshot bytes.
/// Guarantees: Old authority and the complete removal set remain unchanged until recovery cleans the temporary.
#[test]
fn compaction_interrupted_write_recovers_old_state() {
    for count in [0, 10, 100] {
        let (dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(
                vec![numbered_registration(1), numbered_registration(2)],
                Durability::Sync,
            )
            .unwrap();
        let before = fs::read(namespace(dir.path()).join(DB)).unwrap();
        store.io.fail_write_after = Some(count);
        assert!(matches!(
            store.compact_with_retention(&[file_id(1), file_id(2)]),
            Err(StoreError::Io { .. })
        ));
        assert_eq!(fs::read(namespace(dir.path()).join(DB)).unwrap(), before);
        assert_eq!(store.recovered.state.records().len(), 2);
        assert_blocked(&mut store);
        assert!(matches!(
            CheckpointStore::open(&root, ID, config()),
            Err(StoreError::Locked)
        ));
        assert_eq!(store.recover_compaction().unwrap(), None);
        assert_eq!(store.state().unwrap().records().len(), 2);
        assert!(!namespace(dir.path()).join(COMPACT).exists());
        assert!(
            store
                .compact_with_retention(&[file_id(1), file_id(2)])
                .unwrap()
                .is_some()
        );
        assert_eq!(store.state().unwrap().records().len(), 0);
    }
}

/// Scenario: Replacement reports failure before or after rename, or reopening fails after directory sync.
/// Guarantees: No old handle survives replacement attempts; recovery adopts exactly the old or full new set.
#[test]
fn compaction_uncertain_replacement_and_reopen() {
    for (point, published) in [
        ("before checkpoint replacement", false),
        ("after checkpoint replacement", true),
        ("reopen compacted checkpoint", true),
    ] {
        let (dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(
                vec![numbered_registration(1), numbered_registration(2)],
                Durability::Sync,
            )
            .unwrap();
        store.io.fail_step = Some(point);
        assert!(matches!(
            store.compact_with_retention(&[file_id(1), file_id(2)]),
            Err(StoreError::Io { .. })
        ));
        assert!(store.recovered.file.is_none());
        assert_eq!(store.recovered.state.generation(), 0);
        assert_eq!(store.recovered.state.records().len(), 2);
        assert_blocked(&mut store);
        assert!(matches!(
            CheckpointStore::open(&root, ID, config()),
            Err(StoreError::Locked)
        ));
        let receipt = store.recover_compaction().unwrap();
        assert_eq!(receipt.is_some(), published);
        assert_eq!(
            store.state().unwrap().records().len(),
            if published { 0 } else { 2 }
        );
        assert!(!namespace(dir.path()).join(COMPACT).exists());
        drop(store);
        let reopened = self::store(&root);
        assert_eq!(reopened.state().unwrap().generation(), u64::from(published));
    }
}

/// Scenario: The process drops its store after a pre/post-rename interruption without live resolution.
/// Guarantees: Startup selects only complete authority and performs the recovery barrier before cleanup.
#[test]
fn compaction_restart_at_publication_boundaries() {
    for (point, generation) in [
        ("validate compaction temporary", 0),
        ("before checkpoint replacement", 0),
        ("after checkpoint replacement", 1),
    ] {
        let (dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(
                vec![numbered_registration(1), numbered_registration(2)],
                Durability::Sync,
            )
            .unwrap();
        store.io.fail_step = Some(point);
        assert!(
            store
                .compact_with_retention(&[file_id(1), file_id(2)])
                .is_err()
        );
        drop(store);
        let reopened = self::store(&root);
        assert_eq!(reopened.state().unwrap().generation(), generation);
        assert_eq!(
            reopened.state().unwrap().records().len(),
            if generation == 1 { 0 } else { 2 }
        );
        assert!(!namespace(dir.path()).join(COMPACT).exists());
        let events = &reopened.io.events;
        if let Some(cleanup) = events.iter().position(|e| *e == "before temporary removal") {
            assert!(
                events
                    .iter()
                    .position(|e| *e == "recovery directory sync")
                    .unwrap()
                    < cleanup
            );
        }
    }
}

/// Scenario: A compaction file, old-handle, directory, or reopened-file synchronization fails once.
/// Guarantees: Every protected operation stays poisoned without another barrier or premature live publication.
#[test]
fn compaction_sync_failures_are_terminal() {
    for barrier in [
        "compaction file sync",
        "pre-compaction file sync",
        "compaction directory sync",
        "recovery directory sync",
        "recovery file sync",
    ] {
        let (_dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(vec![registration()], Durability::Sync)
            .unwrap();
        let _ = store
            .append(vec![progress(0, 10)], Durability::Deferred)
            .unwrap();
        let before = records(&store);
        store.io.fail_sync = Some(barrier);
        assert!(
            matches!(store.compact(), Err(StoreError::SyncFailed { operation, .. }) if operation == barrier)
        );
        assert_eq!(store.recovered.state.generation(), 0);
        assert_eq!(store.recovered.state.records().next().unwrap(), &before[0]);
        let events = store.io.events.len();
        assert!(matches!(
            store.state(),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert!(matches!(store.sync(), Err(StoreError::SyncPoisoned { .. })));
        assert!(matches!(
            store.compact(),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert!(matches!(
            store.compact_with_retention(&[]),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert!(matches!(
            store.recover_compaction(),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert!(matches!(
            store.append(vec![progress(10, 20)], Durability::Sync),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert_eq!(store.io.events.len(), events);
    }
}

/// Scenario: Recovery cleanup fails before or after removing an interrupted temporary.
/// Guarantees: Publication stays blocked, the lock stays held, and recovery safely resumes deletion/barriers.
#[test]
fn compaction_interrupted_cleanup() {
    for point in ["before temporary removal", "after temporary removal"] {
        let (dir, root) = root();
        let mut store = store(&root);
        store.io.fail_write_after = Some(1);
        assert!(store.compact().is_err());
        store.io.fail_step = Some(point);
        assert!(matches!(
            store.recover_compaction(),
            Err(StoreError::Io { .. })
        ));
        assert_blocked(&mut store);
        assert!(matches!(
            CheckpointStore::open(&root, ID, config()),
            Err(StoreError::Locked)
        ));
        assert_eq!(store.recover_compaction().unwrap(), None);
        assert!(!namespace(dir.path()).join(COMPACT).exists());
        assert!(namespace(dir.path()).join(DB).exists());
        assert!(namespace(dir.path()).join("ownership.lock").exists());
        assert_eq!(store.compact().unwrap().generation, 1);
    }
}

/// Scenario: Recovery of an interrupted compaction fails its old-file, publication, or cleanup barrier.
/// Guarantees: Even after the temporary disappears, no subsequent operation can retry a failed sync.
#[test]
fn compaction_recovery_sync_failure_is_terminal() {
    for barrier in [
        "pre-compaction recovery file sync",
        "recovery directory sync",
        "recovery file sync",
        "cleanup directory sync",
    ] {
        let (_dir, root) = root();
        let mut store = store(&root);
        store.io.fail_write_after = Some(1);
        assert!(store.compact().is_err());
        store.io.fail_sync = Some(barrier);
        assert!(matches!(
            store.recover_compaction(),
            Err(StoreError::SyncFailed { .. })
        ));
        let events = store.io.events.len();
        assert!(matches!(
            store.recover_compaction(),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert!(matches!(
            store.compact(),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert_eq!(events, store.io.events.len());
    }
}

/// Scenario: A recognized compaction temporary already exists when a new attempt starts.
/// Guarantees: Exclusive creation never overwrites it; recovery cleans it before the generation is retried.
#[test]
fn compaction_temporary_collision() {
    let (dir, root) = root();
    let mut store = store(&root);
    let path = namespace(dir.path()).join(COMPACT);
    artifact(&path, b"leftover");
    assert!(matches!(store.compact(), Err(StoreError::Io { .. })));
    assert_eq!(fs::read(&path).unwrap(), b"leftover");
    assert_blocked(&mut store);
    assert_eq!(store.recover_compaction().unwrap(), None);
    assert_eq!(store.compact().unwrap().generation, 1);
}

/// Scenario: A failed WAL append awaits exact-attempt resolution when compaction is requested.
/// Guarantees: Compaction cannot drop or incorporate an unresolved transaction; resolution precedes publication.
#[test]
fn compaction_waits_for_pending_append() {
    let (_dir, root) = root();
    let mut store = store(&root);
    store.io.fail_write_after = Some(0);
    assert!(
        store
            .append(vec![registration()], Durability::Sync)
            .is_err()
    );
    assert!(matches!(store.compact(), Err(StoreError::RecoveryRequired)));
    let _ = store.resolve_pending().unwrap();
    let _ = store.compact().unwrap();
    assert_eq!(store.state().unwrap().records().len(), 1);
}

/// Scenario: Finalized, quarantined, and incompatible-profile records coexist in recovered state.
/// Guarantees: Compaction copies all evidence exactly without changing framing compatibility or lifecycle.
#[test]
fn compaction_preserves_all_lifecycles_and_profiles() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(
            vec![
                numbered_registration(1),
                numbered_registration(2),
                numbered_registration(3),
            ],
            Durability::Sync,
        )
        .unwrap();
    let mut expected = records(&store);
    drop(store);
    expected[0].lifecycle_state = LifecycleState::RotatedFinalized;
    expected[1].lifecycle_state = LifecycleState::Quarantined;
    expected[1].quarantine_evidence = Some(QuarantineEvidence {
        reason_code: 1,
        observed_size: 99,
        quarantine_epoch: 1,
        quarantine_time_unix_nano: 42,
    });
    expected[2].framing_profile_digest = [9; 32];
    fs::write(
        namespace(dir.path()).join(DB),
        encode_checkpoint(7, ID, &expected).unwrap(),
    )
    .unwrap();
    let mut store = self::store(&root);
    assert_eq!(
        store.state().unwrap().profile_compatible(&file_id(3)),
        Some(false)
    );
    assert_eq!(store.compact().unwrap().generation, 8);
    assert_eq!(records(&store), expected);
    assert_eq!(
        store.state().unwrap().profile_compatible(&file_id(3)),
        Some(false)
    );
}

/// Scenario: Recovery encounters a valid but unrelated snapshot after ambiguous replacement.
/// Guarantees: It rejects the state before barriers or cleanup and latches authority failure.
#[test]
fn compaction_recovery_rejects_changed_authority() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    store.io.fail_step = Some("after checkpoint replacement");
    assert!(store.compact().is_err());
    fs::write(
        namespace(dir.path()).join(DB),
        encode_checkpoint(1, ID, &[]).unwrap(),
    )
    .unwrap();
    let events = store.io.events.len();
    assert!(matches!(
        store.recover_compaction(),
        Err(StoreError::AuthorityChanged)
    ));
    assert!(!store.can_recover_compaction());
    assert_eq!(store.io.events.len(), events);
    assert!(matches!(store.state(), Err(StoreError::AuthorityChanged)));
}

/// Scenario: A transaction exactly fills the byte threshold after recovery of a large valid WAL.
/// Guarantees: Equality succeeds, the next append cannot overshoot, and compaction restores admission.
#[test]
fn compaction_byte_threshold_equality() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    let mut op = registration();
    if let Operation::RegisterFile(record) = &mut op {
        record.advisory_path = AdvisoryPath::from_unix_bytes(&vec![b'x'; 4096]).unwrap();
    }
    let mut file = OpenOptions::new().append(true).open(&path).unwrap();
    let mut sequence = 0;
    while file.metadata().unwrap().len() - 108 < WAL_MAX_TX_FRAME_BYTES {
        sequence += 1;
        file.write_all(
            &encode_transaction(&Transaction {
                sequence,
                operations: vec![op.clone(); 256],
            })
            .unwrap(),
        )
        .unwrap();
    }
    let wal_bytes = file.metadata().unwrap().len() - 108;
    drop(file);
    let frame_len = encode_transaction(&Transaction {
        sequence: sequence + 1,
        operations: vec![progress(0, 10)],
    })
    .unwrap()
    .len() as u64;
    let config = StoreConfig {
        max_wal_bytes: wal_bytes + frame_len,
        ..config()
    };
    let mut store = CheckpointStore::open(&root, ID, config).unwrap();
    let _ = store
        .append(vec![progress(0, 10)], Durability::Sync)
        .unwrap();
    assert_eq!(store.wal_bytes(), config.max_wal_bytes);
    assert!(matches!(
        store.append(vec![progress(10, 20)], Durability::Sync),
        Err(StoreError::CompactionRequired)
    ));
    let _ = store.compact().unwrap();
    assert_eq!(
        store
            .append(vec![progress(10, 20)], Durability::Sync)
            .unwrap()
            .sequence,
        1
    );
}
