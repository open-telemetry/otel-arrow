// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    AdvisoryPath, CommittedFrontierGuard, FileId, FramingEncoding, FramingOnDecodeError,
    FramingProfileParams, FramingResume, Locator, MaxLogSizeBehavior, MultilineMode, RegisterFile,
    UpdateProgress,
};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};

const ID: &str = "AppLogs";

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

fn config() -> StoreConfig {
    StoreConfig {
        replay: ReplayConfig::new(100, 1024 * 1024, &profile()).unwrap(),
        max_wal_bytes: WAL_MAX_TX_FRAME_BYTES,
        max_wal_transactions: 100,
    }
}

fn root() -> (tempfile::TempDir, File) {
    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let file = File::open(dir.path()).unwrap();
    (dir, file)
}

fn namespace(root: &Path) -> PathBuf {
    root.join("filelog/@v1")
        .join(filesystem::namespace_component(ID).unwrap())
}

fn store(root: &File) -> CheckpointStore {
    CheckpointStore::open(root.as_fd(), ID, config()).unwrap()
}

fn registration() -> Operation {
    Operation::RegisterFile(RegisterFile {
        file_id: FileId::from_bytes([1; 16]),
        file_epoch: 1,
        committed_offset: 0,
        committed_frontier_guard: CommittedFrontierGuard::compute(0, &[]).unwrap(),
        fingerprint: b"prefix".to_vec(),
        ignored_header_bytes: 0,
        locator: Locator::PosixDevIno { dev: 1, ino: 1 },
        framing_profile_version: 1,
        framing_profile_digest: profile().digest().unwrap(),
        framing_resume: FramingResume::Clean,
        last_seen_time_unix_nano: 1,
        advisory_path: AdvisoryPath::from_unix_bytes(b"/log").unwrap(),
    })
}

fn progress(old: u64, new: u64) -> Operation {
    Operation::UpdateProgress(UpdateProgress {
        file_id: FileId::from_bytes([1; 16]),
        expected_file_epoch: 1,
        expected_committed_offset: old,
        new_committed_offset: new,
        new_committed_frontier_guard: CommittedFrontierGuard::compute(
            new,
            &vec![b'x'; new.min(64) as usize],
        )
        .unwrap(),
        new_framing_resume: FramingResume::Clean,
        new_last_seen_time_unix_nano: 2,
        finalize: false,
    })
}

fn artifact(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
}

fn append_bytes(path: &Path, bytes: &[u8]) {
    OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(bytes)
        .unwrap();
}

/// Scenario: Raw IDs include case differences, dots, invalid bytes, and the component limit.
/// Guarantees: Namespace encoding is exact and injective and rejects unsafe or oversized IDs.
#[test]
fn namespace_mapping() {
    assert_eq!(
        filesystem::namespace_component(ID).unwrap(),
        "4170704c6f6773"
    );
    assert_eq!(filesystem::namespace_component("..").unwrap(), "2e2e");
    assert_ne!(
        filesystem::namespace_component(ID).unwrap(),
        filesystem::namespace_component("applogs").unwrap()
    );
    assert_eq!(
        filesystem::namespace_component(&"a".repeat(127))
            .unwrap()
            .len(),
        254
    );
    for id in ["", "a/b", "a\\b", "\u{e9}", &"a".repeat(128)] {
        assert!(filesystem::namespace_component(id).is_err());
    }
}

/// Scenario: A fresh namespace registers a source and defers ordinary progress before reopening.
/// Guarantees: Registration is forced durable and all surviving valid progress replays on restart.
#[test]
fn publication_append_restart() {
    let (dir, root) = root();
    let mut store = store(&root);
    assert_eq!(store.recovered.end, 108);
    assert_eq!(store.wal_bytes(), 0);
    assert!(!store.recovery_report().interrupted_creation);
    assert!(
        store
            .append(vec![registration()], Durability::Deferred)
            .unwrap()
            .synced
    );
    assert!(
        !store
            .append(vec![progress(0, 10)], Durability::Deferred)
            .unwrap()
            .synced
    );
    assert_eq!(store.synced_sequence(), 1);
    let prefix = fs::read(namespace(dir.path()).join(DB)).unwrap()[..108].to_vec();
    drop(store);
    let store = self::store(&root);
    assert_eq!(store.state().unwrap().last_sequence(), 2);
    assert_eq!(store.synced_sequence(), 2);
    assert_eq!(
        fs::read(namespace(dir.path()).join(DB)).unwrap()[..108],
        prefix
    );
}

/// Scenario: A second independently opened store targets an already held namespace.
/// Guarantees: In-process contention fails immediately; separate IDs and released locks remain usable.
#[test]
fn same_process_lock_and_namespace_isolation() {
    let (_dir, root) = root();
    let first = store(&root);
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config()),
        Err(StoreError::Locked)
    ));
    let other = CheckpointStore::open(root.as_fd(), "applogs", config()).unwrap();
    drop(other);
    drop(first);
    let _reopened = store(&root);
}

/// Scenario: A child process attempts to open the namespace held by its parent.
/// Guarantees: The OS lock excludes distinct processes as well as independent in-process opens.
#[test]
fn cross_process_lock() {
    let (dir, root) = root();
    let _store = store(&root);
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "store::tests::lock_child",
            "--nocapture",
            "--format=pretty",
        ])
        .env("FILELOG_LOCK_TEST_ROOT", dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("test store::tests::lock_child ... ok"),
        "{stdout}"
    );
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed;"),
        "{stdout}"
    );
}

/// Scenario: The lock-test child opens its parent's checkpoint namespace.
/// Guarantees: Cross-process contention reports Locked without changing authority.
#[test]
fn lock_child() {
    let Some(path) = std::env::var_os("FILELOG_LOCK_TEST_ROOT") else {
        return;
    };
    let root = File::open(path).unwrap();
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config()),
        Err(StoreError::Locked)
    ));
}

/// Scenario: A valid registration precedes a progress frame cut at every possible byte.
/// Guarantees: Only that incomplete suffix is removed and the entire accepted prefix is byte-identical.
#[test]
fn every_torn_frame_boundary_preserves_prefix() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let path = namespace(dir.path()).join(DB);
    let prefix = fs::read(&path).unwrap();
    drop(store);
    let frame = encode_transaction(&Transaction {
        sequence: 2,
        operations: vec![progress(0, 10)],
    })
    .unwrap();
    for cut in 1..frame.len() {
        append_bytes(&path, &frame[..cut]);
        let recovered = self::store(&root);
        assert_eq!(recovered.recovery_report().repaired_tail_bytes, cut as u64);
        assert_eq!(recovered.state().unwrap().last_sequence(), 1);
        assert_eq!(fs::read(&path).unwrap(), prefix);
        drop(recovered);
    }
}

/// Scenario: A checkpoint begins with a nonempty snapshot followed by a torn WAL header.
/// Guarantees: Repair uses the absolute WAL offset rather than truncating snapshot records.
#[test]
fn nonempty_snapshot_tail_repair() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let records: Vec<_> = store.state().unwrap().records().cloned().collect();
    drop(store);
    let prefix = encode_checkpoint(7, ID, &records).unwrap();
    let path = namespace(dir.path()).join(DB);
    fs::write(&path, &prefix).unwrap();
    append_bytes(&path, b"FLOG");
    let recovered = self::store(&root);
    assert_eq!(recovered.state().unwrap().generation(), 7);
    assert_eq!(recovered.state().unwrap().records().len(), 1);
    assert_eq!(fs::read(&path).unwrap(), prefix);
}

/// Scenario: A complete WAL frame has damaged header, body, or frame CRC bytes.
/// Guarantees: Complete corruption is never repaired or converted into an accepted prefix.
#[test]
fn complete_corruption_is_unchanged() {
    for index in [0, 20, 32, TX_HEADER_BYTES, 60] {
        let (dir, root) = root();
        drop(store(&root));
        let path = namespace(dir.path()).join(DB);
        let mut frame = encode_transaction(&Transaction {
            sequence: 1,
            operations: vec![registration()],
        })
        .unwrap();
        frame[index] ^= 1;
        append_bytes(&path, &frame);
        let before = fs::read(&path).unwrap();
        assert!(matches!(
            CheckpointStore::open(root.as_fd(), ID, config()),
            Err(StoreError::Decode(_) | StoreError::Replay(ReplayError::Decode(_)))
        ));
        assert_eq!(fs::read(&path).unwrap(), before);
    }
}

/// Scenario: The container or snapshot prefix is shorter than its declared immutable boundary.
/// Guarantees: Incomplete authority is distinguished from a repairable WAL suffix and left intact.
#[test]
fn incomplete_snapshot_is_not_tail_repair() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    let prefix = fs::read(&path).unwrap();
    for cut in [0, 23, 24, 80, 107] {
        fs::write(&path, &prefix[..cut]).unwrap();
        assert!(matches!(
            CheckpointStore::open(root.as_fd(), ID, config()),
            Err(StoreError::AuthoritativeGenerationIncomplete)
        ));
        assert_eq!(fs::read(&path).unwrap(), prefix[..cut]);
    }
}

/// Scenario: A recovery directory barrier fails while a torn tail and stale temporary exist.
/// Guarantees: Neither tail repair nor temporary deletion occurs before durable authority is established.
#[test]
fn recovery_barrier_precedes_mutations() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    append_bytes(&path, b"FLOG");
    let temp = namespace(dir.path()).join(COMPACT);
    artifact(&temp, b"unpublished");
    let before = fs::read(&path).unwrap();
    let fault = Io {
        fail_sync: Some("recovery directory sync"),
        ..Io::default()
    };
    assert!(matches!(
        CheckpointStore::open_with_io(root.as_fd(), ID, config(), fault),
        Err(StoreError::SyncFailed {
            operation: "recovery directory sync",
            ..
        })
    ));
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(temp.exists());
}

/// Scenario: A required initial-publication or recovery synchronization reports failure.
/// Guarantees: Open returns a terminal SyncFailed error instead of a usable store or retry instruction.
#[test]
fn publication_barrier_errors_are_terminal() {
    for operation in [
        "namespace parent sync",
        "initial recovery directory sync",
        "creation file sync",
        "publication directory sync",
        "recovery file sync",
    ] {
        let (_dir, root) = root();
        let fault = Io {
            fail_sync: Some(operation),
            ..Io::default()
        };
        assert!(matches!(
            CheckpointStore::open_with_io(root.as_fd(), ID, config(), fault),
            Err(StoreError::SyncFailed { operation: failed, .. }) if failed == operation
        ));
    }
}

/// Scenario: A valid namespace lacks its database but retains a compaction temporary.
/// Guarantees: Missing authority never promotes a temporary or silently creates empty state.
#[test]
fn missing_authority_with_compaction_fails_closed() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path());
    fs::remove_file(path.join(DB)).unwrap();
    artifact(&path.join(COMPACT), &encode_checkpoint(8, ID, &[]).unwrap());
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config()),
        Err(StoreError::AuthoritativeGenerationMissing)
    ));
    assert!(!path.join(DB).exists());
    assert!(path.join(COMPACT).exists());
}

/// Scenario: Initial creation leaves a partial temporary without authoritative data.
/// Guarantees: The recognized temporary is discarded and never interpreted as persisted progress.
#[test]
fn interrupted_creation_write() {
    let (_dir, root) = root();
    let fault = Io {
        fail_write_after: Some(40),
        ..Io::default()
    };
    assert!(CheckpointStore::open_with_io(root.as_fd(), ID, config(), fault).is_err());
    let recovered = store(&root);
    assert!(recovered.recovery_report().interrupted_creation);
    assert!(recovered.recovery_report().cleaned_temporary);
    assert_eq!(recovered.state().unwrap().last_sequence(), 0);
}

/// Scenario: An append returns an error before writing, within its header/body, or after the complete frame.
/// Guarantees: Resolution retains the lock, rejects new appends, and installs the transaction exactly once.
#[test]
fn failed_writes_resolve_without_duplicate_frames() {
    for count in [0, 1, TX_HEADER_BYTES, 70, usize::MAX] {
        let (_dir, root) = root();
        let mut store = store(&root);
        store.io.fail_write_after = Some(count);
        assert!(
            store
                .append(vec![registration()], Durability::Sync)
                .is_err()
        );
        assert!(matches!(store.state(), Err(StoreError::RecoveryRequired)));
        assert!(matches!(
            store.append(vec![registration()], Durability::Sync),
            Err(StoreError::RecoveryRequired)
        ));
        assert!(matches!(
            CheckpointStore::open(root.as_fd(), ID, config()),
            Err(StoreError::Locked)
        ));
        assert_eq!(
            store.resolve_pending().unwrap(),
            AppendReceipt {
                sequence: 1,
                synced: true
            }
        );
        assert_eq!(store.state().unwrap().records().len(), 1);
        assert_eq!(store.state().unwrap().last_sequence(), 1);
        drop(store);
        assert_eq!(self::store(&root).state().unwrap().last_sequence(), 1);
    }
}

fn assert_sync_poisoned(store: &mut CheckpointStore, synced_sequence: u64) {
    let barriers = store.io.events.len();
    assert!(
        store.io.fail_sync.is_none(),
        "the injected error was one-shot"
    );
    for _ in 0..2 {
        assert!(matches!(
            store.state(),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert!(matches!(store.sync(), Err(StoreError::SyncPoisoned { .. })));
        assert!(matches!(
            store.resolve_pending(),
            Err(StoreError::SyncPoisoned { .. })
        ));
        assert!(matches!(
            store.append(vec![registration()], Durability::Sync),
            Err(StoreError::SyncPoisoned { .. })
        ));
    }
    assert_eq!(
        store.io.events.len(),
        barriers,
        "never attempt another barrier"
    );
    assert_eq!(
        store.synced_sequence(),
        synced_sequence,
        "never certify failed writes"
    );
}

/// Scenario: A complete append encounters a one-shot synchronization error.
/// Guarantees: Neither reopening through resolution nor a later successful fsync can certify its bytes.
#[test]
fn append_sync_failure_permanently_poisons_store() {
    let (dir, root) = root();
    let mut store = store(&root);
    store.io.fail_sync = Some("append file sync");
    assert!(matches!(
        store.append(vec![registration()], Durability::Sync),
        Err(StoreError::SyncFailed { .. })
    ));
    let path = namespace(dir.path()).join(DB);
    let bytes = fs::read(&path).unwrap();
    assert_sync_poisoned(&mut store, 0);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config()),
        Err(StoreError::Locked)
    ));
}

/// Scenario: Periodic sync fails with more than one applied deferred transaction.
/// Guarantees: The entire unsynced suffix stays uncertified, with no automatic retry or partial release.
#[test]
fn deferred_sync_failure_permanently_poisons_store() {
    let (_dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let _ = store
        .append(vec![progress(0, 10)], Durability::Deferred)
        .unwrap();
    let _ = store
        .append(vec![progress(10, 20)], Durability::Deferred)
        .unwrap();
    store.io.fail_sync = Some("explicit file sync");
    assert!(matches!(store.sync(), Err(StoreError::SyncFailed { .. })));
    assert_sync_poisoned(&mut store, 1);
}

/// Scenario: Resolving a partial write fails a directory, repair, or final file barrier once.
/// Guarantees: Nested recovery failures poison the original store and cannot be cleared by reopening.
#[test]
fn resolution_barrier_failure_permanently_poisons_store() {
    for operation in [
        "pre-resolution file sync",
        "recovery directory sync",
        "recovery file sync",
        "resolved append sync",
    ] {
        let (_dir, root) = root();
        let mut store = store(&root);
        store.io.fail_write_after = Some(4);
        assert!(
            store
                .append(vec![registration()], Durability::Sync)
                .is_err()
        );
        store.io.fail_sync = Some(operation);
        assert!(matches!(
            store.resolve_pending(),
            Err(StoreError::SyncFailed { .. })
        ));
        assert_sync_poisoned(&mut store, 0);
    }
}

/// Scenario: After registration, a progress transaction names a stale expected offset.
/// Guarantees: Preflight rejection changes neither disk bytes nor live records or sequence.
#[test]
fn semantic_rejection_precedes_write() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let path = namespace(dir.path()).join(DB);
    let before = fs::read(&path).unwrap();
    assert!(
        store
            .append(vec![progress(5, 10)], Durability::Sync)
            .is_err()
    );
    assert_eq!(store.state().unwrap().last_sequence(), 1);
    assert_eq!(fs::read(&path).unwrap(), before);
    let _ = store
        .append(vec![progress(0, 10)], Durability::Sync)
        .unwrap();
}

/// Scenario: WAL transaction count reaches its configured limit exactly.
/// Guarantees: Equality is accepted and the next append requests compaction without overshoot.
#[test]
fn wal_count_admission() {
    let (dir, root) = root();
    let config = StoreConfig {
        max_wal_transactions: 1,
        ..config()
    };
    let mut store = CheckpointStore::open(root.as_fd(), ID, config).unwrap();
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let path = namespace(dir.path()).join(DB);
    let before = fs::read(&path).unwrap();
    assert!(matches!(
        store.append(vec![progress(0, 10)], Durability::Sync),
        Err(StoreError::CompactionRequired)
    ));
    assert_eq!(fs::read(&path).unwrap(), before);
    drop(store);
    append_bytes(
        &path,
        &encode_transaction(&Transaction {
            sequence: 2,
            operations: vec![progress(0, 10)],
        })
        .unwrap(),
    );
    let mut recovered = CheckpointStore::open(root.as_fd(), ID, config).unwrap();
    assert_eq!(recovered.state().unwrap().last_sequence(), 2);
    assert!(matches!(
        recovered.append(vec![progress(10, 20)], Durability::Sync),
        Err(StoreError::CompactionRequired)
    ));
}

/// Scenario: Snapshot/WAL admission overflows or would exceed the fixed recovery ceiling.
/// Guarantees: Invalid configuration is rejected before creating any namespace artifacts.
#[test]
fn checked_resource_admission() {
    let (dir, root) = root();
    let mut config = config();
    config.replay.max_snapshot_bytes = u64::MAX;
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config),
        Err(StoreError::InvalidLimits {
            max_snapshot_bytes: u64::MAX,
            ..
        })
    ));
    assert!(!dir.path().join("filelog").exists());
}

/// Scenario: A namespace contains unexpected names, links, or insecure artifact permissions.
/// Guarantees: Opening fails closed without deleting or following the untrusted artifact.
#[test]
fn untrusted_artifacts_fail_closed() {
    for kind in ["unknown", "symlink", "hardlink", "permissions", "fifo"] {
        let (dir, root) = root();
        drop(store(&root));
        let path = namespace(dir.path());
        match kind {
            "unknown" => artifact(&path.join("CURRENT"), b"legacy"),
            "symlink" => symlink(path.join(DB), path.join(CREATE)).unwrap(),
            "hardlink" => fs::hard_link(path.join(DB), path.join(CREATE)).unwrap(),
            "permissions" => {
                fs::set_permissions(path.join(DB), fs::Permissions::from_mode(0o666)).unwrap()
            }
            "fifo" => nix::unistd::mkfifo(
                &path.join(CREATE),
                nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
            )
            .unwrap(),
            _ => unreachable!(),
        }
        assert!(
            CheckpointStore::open(root.as_fd(), ID, config()).is_err(),
            "{kind}"
        );
        assert!(path.join(DB).exists());
    }
}

/// Scenario: The namespace hierarchy contains a symlink instead of a directory.
/// Guarantees: Descendant traversal refuses the link and never writes through it.
#[test]
fn symlink_ancestor_rejected() {
    let (dir, root) = root();
    let other = tempfile::tempdir().unwrap();
    symlink(other.path(), dir.path().join("filelog")).unwrap();
    assert!(CheckpointStore::open(root.as_fd(), ID, config()).is_err());
    assert_eq!(fs::read_dir(other.path()).unwrap().count(), 0);
}

/// Scenario: A published data file is replaced behind a still-open store.
/// Guarantees: Appends cannot silently write through the stale handle to an unlinked inode.
#[test]
fn replacement_substitution_rejected() {
    let (dir, root) = root();
    let mut store = store(&root);
    let path = namespace(dir.path()).join(DB);
    fs::remove_file(&path).unwrap();
    artifact(&path, &encode_checkpoint(0, ID, &[]).unwrap());
    assert!(
        store
            .append(vec![registration()], Durability::Sync)
            .is_err()
    );
    assert_eq!(fs::metadata(path).unwrap().len(), 108);
    assert!(matches!(store.state(), Err(StoreError::AuthorityChanged)));
    assert!(matches!(store.sync(), Err(StoreError::AuthorityChanged)));
    assert!(matches!(
        store.resolve_pending(),
        Err(StoreError::AuthorityChanged)
    ));
}

/// Scenario: A valid checkpoint is copied into a different raw-ID namespace.
/// Guarantees: Namespace digest mismatch blocks recovery and never resets the misplaced checkpoint.
#[test]
fn namespace_digest_mismatch_is_preserved() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    let wrong = encode_checkpoint(0, "other", &[]).unwrap();
    fs::write(&path, &wrong).unwrap();
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config()),
        Err(StoreError::Replay(ReplayError::Decode(
            DecodeError::NamespaceMismatch { .. }
        )))
    ));
    assert_eq!(fs::read(path).unwrap(), wrong);
}

/// Scenario: Recovery of a failed append finds a same-length but different valid transaction.
/// Guarantees: Resolution rejects unexpected authority instead of reporting the retained operation applied.
#[test]
fn pending_must_match_exact_frame() {
    let (dir, root) = root();
    let mut store = store(&root);
    store.io.fail_write_after = Some(usize::MAX);
    assert!(
        store
            .append(vec![registration()], Durability::Sync)
            .is_err()
    );
    let mut other = registration();
    if let Operation::RegisterFile(op) = &mut other {
        op.last_seen_time_unix_nano = 2;
    }
    let frame = encode_transaction(&Transaction {
        sequence: 1,
        operations: vec![other],
    })
    .unwrap();
    let path = namespace(dir.path()).join(DB);
    let mut bytes = fs::read(&path).unwrap();
    bytes[108..].copy_from_slice(&frame);
    fs::write(&path, &bytes).unwrap();
    assert!(matches!(
        store.resolve_pending(),
        Err(StoreError::AuthorityChanged)
    ));
    assert!(store.state().is_err());
    assert_eq!(fs::read(path).unwrap(), bytes);
}

/// Scenario: First publication encounters an already authoritative database.
/// Guarantees: No-replace publication preserves the existing file and leaves its temporary for recovery.
#[test]
fn initial_publication_never_replaces_authority() {
    let (dir, root) = root();
    let store = store(&root);
    let path = namespace(dir.path());
    let original = fs::read(path.join(DB)).unwrap();
    artifact(&path.join(CREATE), &encode_checkpoint(99, ID, &[]).unwrap());
    assert!(store.namespace.publish_initial().is_err());
    assert_eq!(fs::read(path.join(DB)).unwrap(), original);
    assert!(path.join(CREATE).exists());
}

/// Scenario: Rotation finalization requests deferred durability.
/// Guarantees: Finalization is synchronized before the receipt permits descriptor or lease release.
#[test]
fn finalization_forces_sync() {
    let (_dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let mut finalization = progress(0, 10);
    if let Operation::UpdateProgress(op) = &mut finalization {
        op.finalize = true;
    }
    assert!(
        store
            .append(vec![finalization], Durability::Deferred)
            .unwrap()
            .synced
    );
    assert_eq!(store.synced_sequence(), 2);
}

/// Scenario: An existing namespace reopens after all ancestors are already visible.
/// Guarantees: Every immediate parent is still synced; visible directories do not bypass durability.
#[test]
fn existing_ancestors_repeat_parent_barriers() {
    let (_dir, root) = root();
    drop(store(&root));
    let store = store(&root);
    assert_eq!(
        store
            .io
            .events
            .iter()
            .filter(|event| **event == "namespace parent sync")
            .count(),
        3
    );
}

/// Scenario: The snapshot generation changes while an append remains pending.
/// Guarantees: Resolution rejects the changed generation without repairing or rewriting the file.
#[test]
fn pending_recovery_rejects_changed_generation() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    store.io.fail_write_after = Some(4);
    assert!(
        store
            .append(vec![progress(0, 10)], Durability::Sync)
            .is_err()
    );
    let path = namespace(dir.path()).join(DB);
    // Keep the observed file length unchanged, but change the snapshot generation
    // using a complete checksummed prefix. Repair must not touch the torn suffix.
    let mut bytes = fs::read(&path).unwrap();
    bytes[..108].copy_from_slice(&encode_checkpoint(1, ID, &[]).unwrap());
    fs::write(&path, &bytes).unwrap();
    assert!(matches!(
        store.resolve_pending(),
        Err(StoreError::AuthorityChanged)
    ));
    assert_eq!(fs::read(path).unwrap(), bytes);
}

/// Scenario: A checksummed container header declares a snapshot larger than the configured budget.
/// Guarantees: Recovery rejects the header before attempting that allocation or treating it as a torn tail.
#[test]
fn snapshot_header_budget_precedes_allocation() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    let mut bytes = fs::read(&path).unwrap();
    bytes[12..20].copy_from_slice(&u64::MAX.to_be_bytes());
    let crc = crate::crc32c(&bytes[..20]);
    bytes[20..24].copy_from_slice(&crc.to_be_bytes());
    fs::write(&path, &bytes).unwrap();
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config()),
        Err(StoreError::Decode(DecodeError::LengthExceedsMaximum { .. }))
    ));
    assert_eq!(fs::read(path).unwrap(), bytes);
}

/// Scenario: Cleanup sync fails after a recognized compaction temporary has been removed.
/// Guarantees: Cleanup returns terminal SyncFailed without making recovered state available.
#[test]
fn cleanup_sync_failure_is_terminal() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path());
    artifact(&path.join(COMPACT), b"unpublished");
    let fault = Io {
        fail_sync: Some("cleanup directory sync"),
        ..Io::default()
    };
    let before = fs::read(path.join(DB)).unwrap();
    assert!(matches!(
        CheckpointStore::open_with_io(root.as_fd(), ID, config(), fault),
        Err(StoreError::SyncFailed {
            operation: "cleanup directory sync",
            ..
        })
    ));
    assert_eq!(fs::read(path.join(DB)).unwrap(), before);
    assert!(!path.join(COMPACT).exists());
}

/// Scenario: The byte compaction threshold is lowered below an older valid WAL.
/// Guarantees: Streaming recovery accepts it and the next append requests compaction without changing disk.
#[test]
fn lower_byte_threshold_allows_recovery() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    let mut op = registration();
    if let Operation::RegisterFile(record) = &mut op {
        record.advisory_path = AdvisoryPath::from_unix_bytes(&vec![b'x'; 4096]).unwrap();
    }
    let mut file = OpenOptions::new().append(true).open(&path).unwrap();
    let mut sequence = 0;
    while file.metadata().unwrap().len() - 108 <= config().max_wal_bytes {
        sequence += 1;
        let frame = encode_transaction(&Transaction {
            sequence,
            operations: vec![op.clone(); 256],
        })
        .unwrap();
        file.write_all(&frame).unwrap();
    }
    let length = file.metadata().unwrap().len();
    drop(file);
    let mut recovered = store(&root);
    assert_eq!(recovered.state().unwrap().last_sequence(), sequence);
    assert!(matches!(
        recovered.append(vec![progress(0, 10)], Durability::Sync),
        Err(StoreError::CompactionRequired)
    ));
    assert_eq!(fs::metadata(path).unwrap().len(), length);
}

/// Scenario: An authoritative file exceeds the fixed ceiling even with valid prefix bytes.
/// Guarantees: Recovery rejects it before reading WAL data or modifying the oversized file.
#[test]
fn recovery_enforces_hard_file_ceiling() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(MAX_BYTES + 1)
        .unwrap();
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config()),
        Err(StoreError::RecoveryLimit(_))
    ));
    assert_eq!(fs::metadata(path).unwrap().len(), MAX_BYTES + 1);
}

/// Scenario: A pending append's short on-disk suffix is changed into different incomplete bytes.
/// Guarantees: Mechanical incompleteness alone cannot authorize truncating a mismatching live attempt.
#[test]
fn torn_pending_suffix_must_match_retained_frame() {
    let (dir, root) = root();
    let mut store = store(&root);
    store.io.fail_write_after = Some(4);
    assert!(
        store
            .append(vec![registration()], Durability::Sync)
            .is_err()
    );
    let path = namespace(dir.path()).join(DB);
    let mut bytes = fs::read(&path).unwrap();
    bytes[108] ^= 1;
    fs::write(&path, &bytes).unwrap();
    assert!(matches!(
        store.resolve_pending(),
        Err(StoreError::AuthorityChanged)
    ));
    assert!(matches!(store.state(), Err(StoreError::AuthorityChanged)));
    assert_eq!(fs::read(path).unwrap(), bytes);
}

/// Scenario: The data file grows behind a store without replacing its inode.
/// Guarantees: Unexpected length permanently blocks stale state and further mutations.
#[test]
fn changed_file_length_latches_authority_failure() {
    let (dir, root) = root();
    let mut store = store(&root);
    append_bytes(&namespace(dir.path()).join(DB), b"unexpected");
    assert!(matches!(
        store.append(vec![registration()], Durability::Sync),
        Err(StoreError::AuthorityChanged)
    ));
    assert!(matches!(store.state(), Err(StoreError::AuthorityChanged)));
    assert!(matches!(store.sync(), Err(StoreError::AuthorityChanged)));
}

/// Scenario: A bound calculation rejects a configuration with several contributing values.
/// Guarantees: Diagnostics report the exact configured limits rather than only a generic reason.
#[test]
fn rejected_limits_report_configuration() {
    let mut limits = config();
    limits.max_wal_bytes = u64::MAX;
    let error = limits.validate().unwrap_err().to_string();
    for field in [
        format!("max_tracked_files={}", limits.replay.max_tracked_files),
        format!("max_snapshot_bytes={}", limits.replay.max_snapshot_bytes),
        format!("max_wal_bytes={}", limits.max_wal_bytes),
        format!("max_wal_transactions={}", limits.max_wal_transactions),
    ] {
        assert!(error.contains(&field), "{error}");
    }
}

/// Scenario: Engine-accepted private roots carry setgid and/or sticky directory bits.
/// Guarantees: Inherited directory bits remain compatible while access stays private and files stay 0600.
#[test]
fn special_directory_bits_match_engine_provisioning() {
    for mode in [0o2700, 0o1700, 0o3700] {
        let (dir, root) = root();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(mode)).unwrap();
        let held = store(&root);
        for path in [
            dir.path().join("filelog"),
            dir.path().join("filelog/@v1"),
            namespace(dir.path()),
        ] {
            let actual = fs::metadata(path).unwrap().permissions().mode();
            assert_eq!(actual & 0o777, 0o700);
            assert_eq!(actual & 0o2000, mode & 0o2000, "setgid inheritance");
        }
        for name in [DB, "ownership.lock"] {
            assert_eq!(
                fs::metadata(namespace(dir.path()).join(name))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o7777,
                0o600
            );
        }
        drop(held);
        let _reopened = store(&root);
    }
}

/// Scenario: An existing regular checkpoint artifact acquires a special mode bit.
/// Guarantees: Accepting directory setgid/sticky does not relax the 0600 regular-file contract.
#[test]
fn regular_artifact_special_bits_still_rejected() {
    let (dir, root) = root();
    drop(store(&root));
    fs::set_permissions(
        namespace(dir.path()).join(DB),
        fs::Permissions::from_mode(0o2600),
    )
    .unwrap();
    assert!(matches!(
        CheckpointStore::open(root.as_fd(), ID, config()),
        Err(StoreError::Untrusted(_))
    ));
}

/// Scenario: The first writer is interrupted before creation-file sync or before publication-directory sync.
/// Guarantees: Recognized interruption states recover when no synchronization error was observed.
#[test]
fn interrupted_publication_without_sync_error_recovers() {
    for published in [false, true] {
        let (_dir, root) = root();
        let mut io = Io::default();
        let namespace = Namespace::open(root.as_fd(), ID, &mut io).unwrap();
        let prefix = encode_checkpoint(0, ID, &[]).unwrap();
        let mut file = namespace.create(CREATE).unwrap();
        file.write_all(&prefix).unwrap();
        if published {
            file.sync_all().unwrap();
            drop(file);
            namespace.publish_initial().unwrap();
            // Stop before the directory barrier, without manufacturing SyncFailed.
        } else {
            // Stop before the creation-file barrier, also without a reported error.
            drop(file);
        }
        drop(namespace);
        let recovered = store(&root);
        assert_eq!(recovered.state().unwrap().generation(), 0);
        assert_eq!(recovered.state().unwrap().records().len(), 0);
        assert!(recovered.io.events.contains(&"recovery directory sync"));
    }
}

/// Scenario: A previous writer stopped with a torn tail and an unpublished compaction temporary, without a sync error.
/// Guarantees: Ordinary recovery repairs only the tail and removes the temporary after its barriers succeed.
#[test]
fn interrupted_tail_and_temporary_recover_without_sync_error() {
    let (dir, root) = root();
    drop(store(&root));
    let path = namespace(dir.path()).join(DB);
    let before = fs::read(&path).unwrap();
    append_bytes(&path, b"FLOG");
    let temp = namespace(dir.path()).join(COMPACT);
    artifact(&temp, b"unpublished");
    let recovered = store(&root);
    assert_eq!(recovered.recovery_report().repaired_tail_bytes, 4);
    assert!(recovered.recovery_report().cleaned_temporary);
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!temp.exists());
}

/// Scenario: Deferred progress precedes a partial or ambiguously complete failed append.
/// Guarantees: Resolution checks the original handle's error history before recovery and certifies exactly sequence three.
#[test]
fn original_handle_sync_precedes_pending_recovery() {
    for written in [4, usize::MAX] {
        let (_dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(vec![registration()], Durability::Sync)
            .unwrap();
        let _ = store
            .append(vec![progress(0, 10)], Durability::Deferred)
            .unwrap();
        assert_eq!(store.synced_sequence(), 1);
        let original_fd = store.recovered.file.as_raw_fd();
        store.io.fail_write_after = Some(written);
        assert!(
            store
                .append(vec![progress(10, 20)], Durability::Sync)
                .is_err()
        );
        let start = store.io.events.len();
        let receipt = store.resolve_pending().unwrap();
        let events = &store.io.events[start..];
        assert_eq!(events[0], "pre-resolution file sync");
        assert_eq!(store.io.sync_fds[start], original_fd);
        let directory_barrier = events
            .iter()
            .position(|event| *event == "recovery directory sync")
            .unwrap();
        assert!(directory_barrier > 0);
        let recovered_file_barrier = events
            .iter()
            .position(|event| *event == "recovery file sync")
            .unwrap();
        assert_ne!(
            store.io.sync_fds[start + recovered_file_barrier],
            original_fd
        );
        assert_eq!(
            receipt,
            AppendReceipt {
                sequence: 3,
                synced: true
            }
        );
        assert_eq!(store.synced_sequence(), 3);
        assert_eq!(store.state().unwrap().last_sequence(), 3);
    }
}

/// Scenario: Original-handle sync reports a one-shot error after deferred progress and a torn append.
/// Guarantees: The store is poisoned at sequence one and neither recovery, repair, nor re-append starts.
#[test]
fn original_handle_sync_failure_blocks_pending_recovery() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let _ = store
        .append(vec![progress(0, 10)], Durability::Deferred)
        .unwrap();
    store.io.fail_write_after = Some(4);
    assert!(
        store
            .append(vec![progress(10, 20)], Durability::Sync)
            .is_err()
    );
    let path = namespace(dir.path()).join(DB);
    let before = fs::read(&path).unwrap();
    let original_fd = store.recovered.file.as_raw_fd();
    let start = store.io.events.len();
    store.io.fail_sync = Some("pre-resolution file sync");
    assert!(matches!(
        store.resolve_pending(),
        Err(StoreError::SyncFailed {
            operation: "pre-resolution file sync",
            ..
        })
    ));
    assert_eq!(&store.io.events[start..], &["pre-resolution file sync"]);
    assert_eq!(store.io.sync_fds[start], original_fd);
    assert_eq!(fs::read(path).unwrap(), before);
    assert_sync_poisoned(&mut store, 1);
}

/// Scenario: A file is truncated or extended in place after deferred progress, before any further append.
/// Guarantees: Ordinary sync detects the changed length before a barrier and never advances the durable sequence.
#[test]
fn sync_directly_rejects_changed_file_length() {
    for extend in [false, true] {
        let (dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(vec![registration()], Durability::Sync)
            .unwrap();
        let _ = store
            .append(vec![progress(0, 10)], Durability::Deferred)
            .unwrap();
        let path = namespace(dir.path()).join(DB);
        let length = store.recovered.end;
        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(if extend { length + 1 } else { length - 1 })
            .unwrap();
        let before = fs::read(&path).unwrap();
        let barriers = store.io.events.len();
        assert!(matches!(store.sync(), Err(StoreError::AuthorityChanged)));
        assert_eq!(store.synced_sequence(), 1);
        assert_eq!(store.io.events.len(), barriers);
        assert_eq!(fs::read(path).unwrap(), before);
        assert!(matches!(store.state(), Err(StoreError::AuthorityChanged)));
        assert!(matches!(store.sync(), Err(StoreError::AuthorityChanged)));
    }
}

/// Scenario: A one-transaction threshold bounds a WAL whose configured byte threshold alone exceeds admission.
/// Guarantees: The reachable WAL bound permits startup while the original transaction threshold still blocks overshoot.
#[test]
fn transaction_threshold_bounds_admitted_wal_bytes() {
    let (_dir, root) = root();
    let limits = StoreConfig {
        max_wal_bytes: MAX_BYTES,
        max_wal_transactions: 1,
        ..config()
    };
    let mut store = CheckpointStore::open(root.as_fd(), ID, limits).unwrap();
    assert_eq!(store.config.max_wal_bytes, MAX_BYTES);
    assert_eq!(store.config.max_wal_transactions, 1);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    assert!(matches!(
        store.append(vec![progress(0, 10)], Durability::Sync),
        Err(StoreError::CompactionRequired)
    ));
}

/// Scenario: The byte threshold binds with either a representable or overflowing maximum-frame product.
/// Guarantees: A fitting configuration is admitted without rejecting u64::MAX transaction counts or changing raw values.
#[test]
fn byte_threshold_bounds_large_transaction_counts() {
    for count in [2, u64::MAX] {
        let limits = StoreConfig {
            max_wal_transactions: count,
            ..config()
        };
        let admitted = limits.validate().unwrap();
        assert_eq!(admitted.max_wal_transactions, count);
        assert_eq!(admitted.max_wal_bytes, limits.max_wal_bytes);
    }
}

/// Scenario: Saturation bounds the intermediate WAL product but combined file or recovery budgets remain too large.
/// Guarantees: Saturation does not weaken checked admission of the actual resource sums.
#[test]
fn interacting_wal_bound_preserves_combined_limits() {
    let mut file_limit = config();
    file_limit.max_wal_transactions = u64::MAX;
    file_limit.max_wal_bytes = MAX_BYTES;
    let mut snapshot_limit = config();
    snapshot_limit.max_wal_transactions = u64::MAX;
    snapshot_limit.replay.max_snapshot_bytes = MAX_BYTES / 4;
    let mut replay_limit = config();
    replay_limit.max_wal_transactions = u64::MAX;
    replay_limit.max_wal_bytes = 400 * 1024 * 1024;
    replay_limit.replay.max_snapshot_bytes = 200 * 1024 * 1024;
    for limits in [file_limit, snapshot_limit, replay_limit] {
        assert!(matches!(
            limits.validate(),
            Err(StoreError::InvalidLimits { .. })
        ));
    }
    let too_small = StoreConfig {
        max_wal_bytes: WAL_MAX_TX_FRAME_BYTES - 1,
        ..config()
    };
    assert!(matches!(
        too_small.validate(),
        Err(StoreError::InvalidLimits { .. })
    ));
}

/// Scenario: A pending append's file shrinks below the accepted boundary or grows beyond its retained frame.
/// Guarantees: Resolution latches authority failure before any barrier, repair, or durable-sequence advancement.
#[test]
fn pending_recovery_rejects_out_of_range_file_size() {
    for grow in [false, true] {
        let (dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(vec![registration()], Durability::Sync)
            .unwrap();
        store.io.fail_write_after = Some(4);
        assert!(matches!(
            store.append(vec![progress(0, 10)], Durability::Sync),
            Err(StoreError::Io { .. })
        ));
        let pending = store.pending.as_ref().unwrap();
        let length = if grow {
            pending.boundary + pending.frame.len() as u64 + 1
        } else {
            pending.boundary - 1
        };
        let path = namespace(dir.path()).join(DB);
        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(length)
            .unwrap();
        let before = fs::read(&path).unwrap();
        let barriers = store.io.events.len();
        assert!(matches!(
            store.resolve_pending(),
            Err(StoreError::AuthorityChanged)
        ));
        assert_eq!(store.io.events.len(), barriers);
        assert_eq!(store.synced_sequence(), 1);
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(matches!(store.state(), Err(StoreError::AuthorityChanged)));
        assert!(matches!(
            store.resolve_pending(),
            Err(StoreError::AuthorityChanged)
        ));
        assert_eq!(store.io.events.len(), barriers);
    }
}

/// Scenario: Re-appending a retained attempt fails again with no bytes, a partial frame, or a complete frame written.
/// Guarantees: The attempt and lock survive the second error; another resolution commits exactly once and survives restart.
#[test]
fn repeated_pending_write_failures_retain_attempt_and_lock() {
    for second_write in [0, 4, TX_HEADER_BYTES, usize::MAX] {
        let (dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(vec![registration()], Durability::Sync)
            .unwrap();
        let path = namespace(dir.path()).join(DB);
        let prefix = fs::read(&path).unwrap();
        store.io.fail_write_after = Some(4);
        assert!(matches!(
            store.append(vec![progress(0, 10)], Durability::Sync),
            Err(StoreError::Io { .. })
        ));
        let frame = store.pending.as_ref().unwrap().frame.clone();
        store.io.fail_write_after = Some(second_write);
        assert!(matches!(
            store.resolve_pending(),
            Err(StoreError::Io {
                operation: "write checkpoint",
                ..
            })
        ));
        assert!(
            store.io.fail_write_after.is_none(),
            "the second write was attempted"
        );
        let retained = store.pending.as_ref().unwrap();
        assert_eq!(retained.frame, frame);
        assert_eq!(retained.boundary, prefix.len() as u64);
        assert_eq!(retained.sequence, 2);
        assert_eq!(store.synced_sequence(), 1);
        assert!(matches!(store.state(), Err(StoreError::RecoveryRequired)));
        assert!(matches!(
            store.append(vec![progress(0, 10)], Durability::Sync),
            Err(StoreError::RecoveryRequired)
        ));
        assert!(matches!(store.sync(), Err(StoreError::RecoveryRequired)));
        assert!(matches!(
            CheckpointStore::open(root.as_fd(), ID, config()),
            Err(StoreError::Locked)
        ));
        let mut partial = prefix.clone();
        partial.extend_from_slice(&frame[..second_write.min(frame.len())]);
        assert_eq!(fs::read(&path).unwrap(), partial);
        assert_eq!(
            store.resolve_pending().unwrap(),
            AppendReceipt {
                sequence: 2,
                synced: true
            }
        );
        let mut complete = prefix;
        complete.extend_from_slice(&frame);
        assert_eq!(fs::read(&path).unwrap(), complete);
        assert_eq!(store.state().unwrap().last_sequence(), 2);
        assert_eq!(store.synced_sequence(), 2);
        drop(store);
        let recovered = self::store(&root);
        assert_eq!(recovered.state().unwrap().last_sequence(), 2);
        assert_eq!(
            recovered
                .state()
                .unwrap()
                .get(&FileId::from_bytes([1; 16]))
                .unwrap()
                .committed_offset,
            10
        );
    }
}

/// Scenario: An opened namespace's filelog or layout-version parent is renamed and replaced with a private directory.
/// Guarantees: Pending recovery rejects the substituted ancestor before synchronization and leaves both trees untouched.
#[test]
fn pending_recovery_rejects_parent_directory_replacement() {
    for parent_name in ["filelog", "filelog/@v1"] {
        let (dir, root) = root();
        let mut store = store(&root);
        let _ = store
            .append(vec![registration()], Durability::Sync)
            .unwrap();
        store.io.fail_write_after = Some(4);
        assert!(matches!(
            store.append(vec![progress(0, 10)], Durability::Sync),
            Err(StoreError::Io { .. })
        ));
        let path = namespace(dir.path()).join(DB);
        let before = fs::read(&path).unwrap();
        let parent = dir.path().join(parent_name);
        let relative_file = path.strip_prefix(&parent).unwrap().to_owned();
        let displaced = dir.path().join("displaced");
        fs::rename(&parent, &displaced).unwrap();
        fs::create_dir(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let barriers = store.io.events.len();
        assert!(matches!(
            store.resolve_pending(),
            Err(StoreError::AuthorityChanged)
        ));
        assert_eq!(store.io.events.len(), barriers);
        assert_eq!(store.synced_sequence(), 1);
        assert_eq!(fs::read(displaced.join(relative_file)).unwrap(), before);
        assert_eq!(fs::read_dir(parent).unwrap().count(), 0);
        assert!(matches!(store.state(), Err(StoreError::AuthorityChanged)));
        assert!(matches!(
            store.append(vec![progress(0, 10)], Durability::Sync),
            Err(StoreError::AuthorityChanged)
        ));
    }
}

/// Scenario: An accepted WAL frame's header is changed to declare a body beyond EOF before pending recovery.
/// Guarantees: A mechanically incomplete frame before the saved boundary never authorizes rolling back accepted progress.
#[test]
fn pending_recovery_rejects_incomplete_accepted_prefix() {
    let (dir, root) = root();
    let mut store = store(&root);
    let _ = store
        .append(vec![registration()], Durability::Sync)
        .unwrap();
    let second_frame = usize::try_from(store.recovered.end).unwrap();
    let _ = store
        .append(vec![progress(0, 10)], Durability::Sync)
        .unwrap();
    store.io.fail_write_after = Some(4);
    assert!(matches!(
        store.append(vec![progress(10, 20)], Durability::Sync),
        Err(StoreError::Io { .. })
    ));
    let path = namespace(dir.path()).join(DB);
    let mut bytes = fs::read(&path).unwrap();
    // Keep the file length in the pending range and preserve a valid envelope.
    // Its longer body is incomplete, so replay stops before the saved boundary.
    let body_len = u32::try_from(bytes.len() - second_frame).unwrap();
    let header = &mut bytes[second_frame..second_frame + TX_HEADER_BYTES];
    header[20..24].copy_from_slice(&body_len.to_be_bytes());
    header[24..28].copy_from_slice(&(!body_len).to_be_bytes());
    let crc = crate::crc32c(&header[..32]);
    header[32..36].copy_from_slice(&crc.to_be_bytes());
    fs::write(&path, &bytes).unwrap();
    let barriers = store.io.events.len();
    assert!(matches!(
        store.resolve_pending(),
        Err(StoreError::AuthorityChanged)
    ));
    assert_eq!(&store.io.events[barriers..], &["pre-resolution file sync"]);
    assert_eq!(store.synced_sequence(), 2);
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert!(matches!(store.state(), Err(StoreError::AuthorityChanged)));
}
