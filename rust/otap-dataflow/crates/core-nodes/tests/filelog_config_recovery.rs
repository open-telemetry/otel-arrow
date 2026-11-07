// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Keep checkpoint lock/reopen coverage separate from tests that spawn children:
//! a concurrent fork can temporarily inherit a lock even with close-on-exec.

#![cfg(target_os = "linux")]

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::config::Config;

/// Scenario: A maximum-width fingerprint reopens and compacts under a smaller selected window.
/// Guarantees: Snapshot/WAL recovery, compaction, and restart preserve old evidence as profile-incompatible.
#[test]
fn reduced_fingerprint_window_preserves_checkpoint() {
    use otel_arrow_dfe_filelog_checkpoint::{
        self as checkpoint,
        store::{CheckpointStore, Durability, StateRoot},
    };
    use std::{
        fs,
        os::{fd::AsFd, unix::fs::PermissionsExt},
    };

    let settings = "limits: {max_tracked_files: 1, max_pending_candidates: 1, max_open_files: 1}";
    let compiled = |window| {
        let config: Config = serde_yaml::from_str(&format!(
            "include: ['/logs/*.log']\n{settings}\nidentity: {{fingerprint_bytes: {window}}}"
        ))
        .unwrap();
        config
            .compile("default", "main", "filelog", std::path::Path::new("/work"))
            .unwrap()
    };
    let old = compiled(65535);
    let new = compiled(16);
    assert_eq!(old.inputs().checkpoint_id(), new.inputs().checkpoint_id());
    assert_eq!(
        old.inputs().resources().snapshot_bytes,
        new.inputs().resources().snapshot_bytes
    );
    assert_eq!(
        old.inputs().resources().reader_table_bytes,
        new.inputs().resources().reader_table_bytes
    );
    let id = checkpoint::FileId::from_bytes([1; 16]);
    for snapshot in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let directory = fs::File::open(dir.path()).unwrap();
        let root = StateRoot::from_borrowed_fd(directory.as_fd()).unwrap();
        let mut store = CheckpointStore::open(
            &root,
            old.inputs().checkpoint_id(),
            old.inputs().store_config().unwrap(),
        )
        .unwrap();
        let _ = store
            .append(
                vec![checkpoint::Operation::RegisterFile(
                    checkpoint::RegisterFile {
                        file_id: id,
                        file_epoch: 1,
                        committed_offset: 64,
                        committed_frontier_guard: checkpoint::CommittedFrontierGuard::compute(
                            64,
                            &[b'x'; 64],
                        )
                        .unwrap(),
                        fingerprint: vec![b'x'; 65535],
                        ignored_header_bytes: 0,
                        locator: checkpoint::Locator::PosixDevIno { dev: 1, ino: 1 },
                        framing_profile_version: checkpoint::FRAMING_PROFILE_VERSION,
                        framing_profile_digest: old.inputs().profile().digest().unwrap(),
                        framing_resume: checkpoint::FramingResume::Clean,
                        last_seen_time_unix_nano: 1,
                        advisory_path: checkpoint::AdvisoryPath::from_unix_bytes(b"/logs/app.log")
                            .unwrap(),
                    },
                )],
                Durability::Sync,
            )
            .unwrap();
        let record = store.state().unwrap().get(&id).unwrap().clone();
        drop(store);
        let database = dir
            .path()
            .join("filelog/@v1")
            .join(old.inputs().namespace_component())
            .join("checkpoint.db");
        if snapshot {
            // Construct the durable snapshot fixture while no store holds the namespace.
            let bytes = checkpoint::encode_checkpoint(
                2,
                old.inputs().checkpoint_id(),
                std::slice::from_ref(&record),
            )
            .unwrap();
            fs::write(&database, bytes).unwrap();
        }
        let before = fs::read(&database).unwrap();
        let mut reopened = CheckpointStore::open(
            &root,
            new.inputs().checkpoint_id(),
            new.inputs().store_config().unwrap(),
        )
        .unwrap();
        assert_eq!(reopened.state().unwrap().get(&id), Some(&record));
        assert_eq!(
            reopened.state().unwrap().profile_compatible(&id),
            Some(false)
        );
        assert_eq!(fs::read(&database).unwrap(), before);
        let generation = reopened.state().unwrap().generation();
        let receipt = reopened.compact().unwrap();
        assert_eq!(receipt.generation, generation + 1);
        assert_eq!(receipt.removed_records, 0);
        assert_eq!(reopened.state().unwrap().get(&id), Some(&record));
        assert_eq!(
            reopened.state().unwrap().profile_compatible(&id),
            Some(false)
        );
        assert_eq!(reopened.wal_bytes(), 0);
        assert_eq!(reopened.synced_sequence(), 0);
        drop(reopened);
        let restarted = CheckpointStore::open(
            &root,
            new.inputs().checkpoint_id(),
            new.inputs().store_config().unwrap(),
        )
        .unwrap();
        assert_eq!(restarted.state().unwrap().generation(), generation + 1);
        assert_eq!(restarted.state().unwrap().last_sequence(), 0);
        assert_eq!(restarted.state().unwrap().get(&id), Some(&record));
        assert_eq!(
            restarted.state().unwrap().profile_compatible(&id),
            Some(false)
        );
    }
}

/// Scenario: 3,340 historical maximum-window records coexist with registrations under a 1,024-byte profile.
/// Guarantees: Receiver-derived admission is valid, but the 268,476,824-byte prospective snapshot is rejected atomically before exhausting the WAL.
#[test]
fn mixed_fingerprint_snapshot_admission() {
    use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::identity::{
        IdentityError, IdentityOwner, lease::LeaseRegistry,
    };
    use otel_arrow_dfe_filelog_checkpoint::{
        self as cp,
        store::{CheckpointStore, Durability, StateRoot, StoreError},
    };
    use std::{
        fs,
        os::{fd::AsFd, unix::fs::PermissionsExt},
    };

    let compile = |tracked, window| {
        let config: Config = serde_yaml::from_str(&format!(
            "include: ['/logs/*.log']\nlimits: {{max_tracked_files: {tracked}, max_pending_candidates: 1, max_open_files: 1}}\nidentity: {{fingerprint_bytes: {window}}}\ncheckpoint: {{compact_after_bytes: 67108864}}"
        )).unwrap();
        config
            .compile("default", "main", "filelog", std::path::Path::new("/work"))
            .unwrap()
    };
    let old = compile(3340, 65535);
    let current = compile(10000, 1024);
    let inputs = current.inputs();
    let config = inputs.store_config().unwrap();
    assert_eq!(config.replay.max_snapshot_bytes, 268_435_450);
    assert_eq!(config.max_wal_bytes, 64 * 1024 * 1024);
    assert_eq!(inputs.checkpoint_id(), old.inputs().checkpoint_id());
    let registration = |n: u64, historical: bool| cp::RegisterFile {
        file_id: cp::FileId::from_bytes(u128::from(n).to_be_bytes()),
        file_epoch: 1,
        committed_offset: 0,
        committed_frontier_guard: cp::CommittedFrontierGuard::empty(),
        fingerprint: vec![b'x'; if historical { 65535 } else { 1024 }],
        ignored_header_bytes: 0,
        locator: cp::Locator::PosixDevIno { dev: 1, ino: n },
        framing_profile_version: cp::FRAMING_PROFILE_VERSION,
        framing_profile_digest: if historical {
            old.inputs().profile()
        } else {
            inputs.profile()
        }
        .digest()
        .unwrap(),
        framing_resume: cp::FramingResume::Clean,
        last_seen_time_unix_nano: 1,
        advisory_path: cp::AdvisoryPath::from_unix_bytes(&vec![b'x'; 4096]).unwrap(),
    };
    let historical: Vec<_> = (1..=3340)
        .map(|n| {
            let r = registration(n, true);
            cp::SnapshotRecord {
                file_id: r.file_id,
                file_epoch: r.file_epoch,
                committed_offset: r.committed_offset,
                committed_frontier_guard: r.committed_frontier_guard,
                fingerprint: r.fingerprint,
                ignored_header_bytes: r.ignored_header_bytes,
                locator: r.locator,
                framing_profile_version: r.framing_profile_version,
                framing_profile_digest: r.framing_profile_digest,
                framing_resume: r.framing_resume,
                lifecycle_state: cp::LifecycleState::Active,
                quarantine_evidence: None,
                last_seen_time_unix_nano: 1,
                advisory_path: r.advisory_path,
            }
        })
        .collect();
    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let directory = fs::File::open(dir.path()).unwrap();
    let root = StateRoot::from_borrowed_fd(directory.as_fd()).unwrap();
    drop(CheckpointStore::open(&root, inputs.checkpoint_id(), config).unwrap());
    let path = dir
        .path()
        .join("filelog/@v1")
        .join(inputs.namespace_component())
        .join("checkpoint.db");
    fs::write(
        &path,
        cp::encode_checkpoint(1, inputs.checkpoint_id(), &historical).unwrap(),
    )
    .unwrap();
    drop(historical);
    let mut store = CheckpointStore::open(&root, inputs.checkpoint_id(), config).unwrap();
    assert_eq!(
        store
            .state()
            .unwrap()
            .profile_compatible(&registration(1, true).file_id),
        Some(false)
    );
    // Stop one transaction before filling the population; its final size is the reported defect.
    let final_start = 3341 + 51 * 128;
    for start in (3341..final_start).step_by(128) {
        let ops = (start..start + 128)
            .map(|n| cp::Operation::RegisterFile(registration(n, false)))
            .collect();
        let _ = store.append(ops, Durability::Sync).unwrap();
    }
    let before_size = store.state().unwrap().snapshot_bytes();
    let before_sequence = store.state().unwrap().last_sequence();
    let before_records = store.state().unwrap().records().len();
    let before_wal = store.wal_bytes();
    let before = fs::read(&path).unwrap();
    let registry = LeaseRegistry::process();
    let leases = registry
        .owner(
            "snapshot-admission",
            IdentityOwner::required_lease_capacity(inputs).unwrap(),
        )
        .unwrap();
    let mut owner = IdentityOwner::new(store, leases, inputs).unwrap();
    let ops = (final_start..=10000)
        .map(|n| cp::Operation::RegisterFile(registration(n, false)))
        .collect();
    let error = owner.append_checkpoint(ops, Durability::Sync).unwrap_err();
    assert!(
        matches!(
            error,
            IdentityError::AppendRejected(StoreError::SnapshotLimit {
                size: 268_476_824,
                max: 268_435_450,
            })
        ),
        "{error:?}"
    );
    // Required admission is receiver-owner terminal, not an implicit per-source failure.
    assert!(owner.is_stopped());
    assert!(!owner.has_pending_write());
    assert_eq!(owner.wal_bytes(), before_wal);
    assert_eq!(owner.synced_sequence(), before_sequence);
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(before);
    drop(owner);
    let mut reopened = CheckpointStore::open(&root, inputs.checkpoint_id(), config).unwrap();
    assert_eq!(reopened.state().unwrap().snapshot_bytes(), before_size);
    assert_eq!(reopened.state().unwrap().last_sequence(), before_sequence);
    assert_eq!(reopened.state().unwrap().records().len(), before_records);
    let _ = reopened.compact().unwrap();
    assert_eq!(reopened.state().unwrap().snapshot_bytes(), before_size);
}
