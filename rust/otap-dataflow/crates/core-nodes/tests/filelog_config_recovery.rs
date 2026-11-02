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
