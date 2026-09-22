// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{LeaseError, SourceLease};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const PROCESS_ROOT: &str = "OTAP_SCRAPER_CHECKPOINT_TEST_ROOT";
const PROCESS_MODE: &str = "OTAP_SCRAPER_CHECKPOINT_TEST_MODE";
const PROCESS_TIMEOUT: Duration = Duration::from_secs(30);
const SHARED_STATE_ROOT: &str = "OTAP_SCRAPER_TEST_STATE_ROOT";
const SHARED_STATE_ALIAS: &str = "OTAP_SCRAPER_TEST_STATE_ALIAS";

struct CheckpointProcess(Child);

impl CheckpointProcess {
    fn start(root: &Path, mode: &str) -> Self {
        Self(
            Command::new(std::env::current_exe().expect("test executable"))
                .args([
                    "--exact",
                    "checkpoint::tests::checkpoint_process_worker",
                    "--quiet",
                ])
                .env(PROCESS_ROOT, root)
                .env(PROCESS_MODE, mode)
                .stdin(Stdio::null())
                .spawn()
                .expect("start checkpoint process"),
        )
    }

    fn wait_for_ready(&mut self, root: &Path) {
        let deadline = Instant::now() + PROCESS_TIMEOUT;
        while !root.join("ready").exists() {
            assert!(
                self.0.try_wait().expect("child status").is_none(),
                "child exited before opening its unfinished checkpoint"
            );
            assert!(
                Instant::now() < deadline,
                "checkpoint child did not become ready"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_success(&mut self) {
        let deadline = Instant::now() + PROCESS_TIMEOUT;
        loop {
            if let Some(status) = self.0.try_wait().expect("child status") {
                assert!(status.success(), "checkpoint child failed: {status}");
                return;
            }
            assert!(Instant::now() < deadline, "checkpoint child did not exit");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn terminate(&mut self) {
        self.0.kill().expect("terminate checkpoint child");
        assert!(!self.0.wait().expect("reap checkpoint child").success());
    }
}

impl Drop for CheckpointProcess {
    fn drop(&mut self) {
        // Reap only this test's child, including when a parent assertion fails.
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            _ = self.0.kill();
            _ = self.0.wait();
        }
    }
}

fn cursor(timestamp: &str, tie_breaker: i64) -> CompositeCursor {
    CompositeCursor::new(timestamp.to_owned(), tie_breaker)
}

fn store(root: &Path, fingerprint: &str) -> CheckpointStore {
    CheckpointStore::new(
        root,
        "group",
        "pipeline",
        "oracle-audit",
        "orders",
        fingerprint.to_owned(),
    )
}

fn write_legacy_checkpoint(
    store: &CheckpointStore,
    prefix: &Path,
    value: &CompositeCursor,
) -> CheckpointState {
    let mut legacy = store.clone();
    legacy.prefix = prefix.to_path_buf();
    legacy.directory_ready = Arc::new(AtomicBool::new(false));
    legacy.write(0, value).expect("legacy commit").0
}

/// Scenario: A source ID makes the serialized checkpoint exceed the read ceiling.
/// Guarantees: The write fails before creating directories or installing unreadable state.
#[test]
fn oversized_source_is_rejected_before_installation() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        &"s".repeat(MAX_CHECKPOINT_BYTES as usize),
        "fingerprint".to_owned(),
    );

    assert!(matches!(
        store.write(0, &cursor("2026-01-01 00:00:00", 1)),
        Err(CheckpointError::TooLarge { .. })
    ));
    assert!(!store.prefix.parent().expect("parent").exists());
    assert_eq!(store.read().expect("read absent state"), None);
}

/// Scenario: An oversized cursor is written after a readable checkpoint exists.
/// Guarantees: Rejection preserves the last committed revision and installs no new file.
#[test]
fn oversized_write_preserves_previous_checkpoint() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let (previous, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("initial commit");

    assert!(matches!(
        store.write(
            previous.revision,
            &cursor(&"x".repeat(MAX_CHECKPOINT_BYTES as usize), 2)
        ),
        Err(CheckpointError::TooLarge { .. })
    ));
    assert!(!revision_path(&store.prefix, previous.revision + 1).exists());
    assert_eq!(
        store.read().expect("previous state remains readable"),
        Some(previous)
    );
}

/// Scenario: A checkpoint envelope reaches the exact maximum readable size.
/// Guarantees: The shared read/write limit is inclusive and the installed cursor round-trips.
#[test]
fn checkpoint_at_size_limit_round_trips() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let payload = CheckpointPayload {
        version: ENVELOPE_VERSION,
        revision: 1,
        source_id: store.source_id.clone(),
        config_fingerprint: store.config_fingerprint.clone(),
        cursor: cursor("", 1),
    };
    let envelope = CheckpointEnvelope {
        checksum: checksum(&payload).expect("checksum"),
        payload,
    };
    let overhead = serde_json::to_vec(&envelope).expect("envelope").len();
    let candidate = cursor(&"x".repeat(MAX_CHECKPOINT_BYTES as usize - overhead), 1);
    let (committed, _) = store.write(0, &candidate).expect("exact-size commit");
    assert_eq!(
        std::fs::metadata(revision_path(&store.prefix, 1))
            .expect("revision")
            .len(),
        MAX_CHECKPOINT_BYTES
    );
    assert_eq!(store.read().expect("read"), Some(committed));
}

/// Scenario: Two source names share a checkpoint-like prefix in one directory.
/// Guarantees: Restart and revision cleanup never mistake another source's files for their own.
#[test]
fn dotted_sources_have_independent_revision_namespaces() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let orders = store(directory.path(), "fingerprint");
    let archive = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        "orders.checkpoint.archive",
        "fingerprint".to_owned(),
    );
    let (archived, _) = archive
        .write(0, &cursor("2026-01-01 00:00:00", 90))
        .expect("archive commit");
    assert_eq!(orders.read().expect("unwritten orders"), None);

    for revision in 0..4 {
        let (committed, _) = orders
            .write(revision, &cursor("2026-01-01 00:00:00", revision as i64))
            .expect("orders commit");
        assert_eq!(orders.read().expect("orders restart"), Some(committed));
        assert_eq!(
            archive.read().expect("archive restart"),
            Some(archived.clone())
        );
    }
    assert!(revision_path(&archive.prefix, archived.revision).exists());
}

/// Scenario: A checkpoint-shaped file has a malformed revision instead of a valid sequence.
/// Guarantees: Startup fails closed instead of silently replaying from the initial cursor.
#[test]
fn malformed_revision_name_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    std::fs::create_dir_all(store.prefix.parent().expect("parent")).expect("directory");
    std::fs::write(format!("{}.invalid.json", store.prefix.display()), b"{}").expect("file");
    assert!(matches!(
        store.read(),
        Err(CheckpointError::InvalidRevision { .. })
    ));
}

/// Scenario: two acknowledged pages commit successive durable checkpoints.
/// Guarantees: the newest cursor and a monotonically increasing revision are what a restarted
/// receiver reads back, so polling resumes after the last acknowledged row.
#[test]
fn writes_and_reads_latest_revision() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");

    let (first, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("first commit");
    let (second, _) = store
        .write(first.revision, &cursor("2026-01-01 00:00:01", 2))
        .expect("second commit");

    assert_eq!(second.revision, 2);
    assert_eq!(store.read().expect("read"), Some(second));
}

/// Scenario: the newest checkpoint file is corrupted after an earlier valid revision exists.
/// Guarantees: startup fails closed rather than silently falling back to the older revision and
/// re-emitting every row between the two positions.
#[test]
fn corrupt_latest_revision_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let (first, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("first commit");
    let (second, _) = store
        .write(first.revision, &cursor("2026-01-01 00:00:01", 2))
        .expect("second commit");
    std::fs::write(
        revision_path(&store.prefix, second.revision),
        b"not checkpoint json",
    )
    .expect("corrupt the newest revision");

    assert!(matches!(store.read(), Err(CheckpointError::Parse { .. })));
}

/// Scenario: a checkpoint file's recorded content no longer matches its stored checksum.
/// Guarantees: silent bit rot is detected and rejected instead of resuming from a cursor the
/// receiver never actually committed.
#[test]
fn checksum_mismatch_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let (state, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 7))
        .expect("commit");
    let path = revision_path(&store.prefix, state.revision);
    let bytes = std::fs::read(&path).expect("read revision");
    let mut envelope: serde_json::Value =
        serde_json::from_slice(&bytes).expect("revision should be JSON");
    envelope["payload"]["cursor"]["tie_breaker"] = serde_json::json!(9_999);
    std::fs::write(&path, serde_json::to_vec(&envelope).expect("encode")).expect("tamper");

    assert!(matches!(
        store.read(),
        Err(CheckpointError::ChecksumMismatch { .. })
    ));
}

/// Scenario: a checkpoint was written by a semantically different query or cursor definition.
/// Guarantees: a configuration fingerprint mismatch fails before polling, so a reused directory
/// cannot resume an unrelated stream's position.
#[test]
fn fingerprint_mismatch_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    _ = store(directory.path(), "first")
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("commit");

    assert!(matches!(
        store(directory.path(), "second").read(),
        Err(CheckpointError::FingerprintMismatch { .. })
    ));
}

/// Scenario: a checkpoint file records a different source identity than the configured receiver.
/// Guarantees: source identity is verified independently of the fingerprint, so a file copied
/// between sources cannot be adopted.
#[test]
fn source_mismatch_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let (state, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("commit");
    let path = revision_path(&store.prefix, state.revision);
    let bytes = std::fs::read(&path).expect("read revision");
    let mut envelope: serde_json::Value =
        serde_json::from_slice(&bytes).expect("revision should be JSON");
    envelope["payload"]["source_id"] = serde_json::json!("other-source");
    // Recompute the checksum so only the source identity check can reject it.
    let payload: CheckpointPayload =
        serde_json::from_value(envelope["payload"].clone()).expect("payload");
    envelope["checksum"] = serde_json::json!(checksum(&payload).expect("checksum"));
    std::fs::write(&path, serde_json::to_vec(&envelope).expect("encode")).expect("rewrite");

    assert!(matches!(
        store.read(),
        Err(CheckpointError::SourceMismatch { .. })
    ));
}

/// Scenario: a checkpoint file records a schema version this build does not understand.
/// Guarantees: an unsupported version aborts startup rather than being interpreted with the
/// wrong field semantics.
#[test]
fn unsupported_version_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let (state, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("commit");
    let path = revision_path(&store.prefix, state.revision);
    let bytes = std::fs::read(&path).expect("read revision");
    let mut envelope: serde_json::Value =
        serde_json::from_slice(&bytes).expect("revision should be JSON");
    envelope["payload"]["version"] = serde_json::json!(ENVELOPE_VERSION + 1);
    let payload: CheckpointPayload =
        serde_json::from_value(envelope["payload"].clone()).expect("payload");
    envelope["checksum"] = serde_json::json!(checksum(&payload).expect("checksum"));
    std::fs::write(&path, serde_json::to_vec(&envelope).expect("encode")).expect("rewrite");

    assert!(matches!(
        store.read(),
        Err(CheckpointError::UnsupportedVersion { .. })
    ));
}

/// Scenario: a checkpoint file's recorded revision disagrees with the revision in its filename.
/// Guarantees: a renamed or hand-edited revision file is rejected instead of installing an
/// out-of-order position that would break monotonic progression.
#[test]
fn revision_mismatch_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let (state, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("commit");
    let bytes = std::fs::read(revision_path(&store.prefix, state.revision)).expect("read");
    std::fs::write(revision_path(&store.prefix, 5), bytes).expect("copy under another revision");

    assert!(matches!(
        store.read(),
        Err(CheckpointError::RevisionMismatch { .. })
    ));
}

/// Scenario: a checkpoint file is far larger than any valid envelope.
/// Guarantees: the read is bounded before allocation, so a corrupted or hostile file cannot
/// exhaust receiver memory during startup.
#[test]
fn oversized_checkpoint_is_rejected_before_parsing() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let (state, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("commit");
    std::fs::write(
        revision_path(&store.prefix, state.revision),
        vec![b'x'; (MAX_CHECKPOINT_BYTES + 1) as usize],
    )
    .expect("write oversized revision");

    assert!(matches!(
        store.read(),
        Err(CheckpointError::TooLarge { .. })
    ));
}

/// Scenario: no checkpoint revision has ever been written for a configured source.
/// Guarantees: absent state is reported distinctly from invalid state, so the configured initial
/// cursor is used instead of failing startup.
#[test]
fn missing_checkpoint_is_distinct_from_invalid_state() {
    let directory = tempfile::tempdir().expect("temporary directory");

    assert_eq!(
        store(directory.path(), "fingerprint").read().expect("read"),
        None
    );
}

/// Scenario: the revision is installed on disk but the parent-directory sync then reports failure.
/// Guarantees: an exact read-back proving the intended checkpoint landed reconciles the commit,
/// so an ACK is not needlessly turned into a replay.
#[test]
fn post_install_failure_reconciles_exact_checkpoint() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let candidate = cursor("2026-01-01 00:00:00", 42);
    store.inject_post_install_failures(1);

    let (committed, _) = store.write(0, &candidate).expect("reconciled commit");

    assert_eq!(committed.revision, 1);
    assert_eq!(committed.cursor, candidate);
    assert_eq!(store.read().expect("read"), Some(committed));
}

/// Scenario: the intended revision path already holds a different valid checkpoint.
/// Guarantees: reconciliation refuses a mismatched install, so the receiver never adopts a
/// cursor other than the one it intended to commit.
#[test]
fn existing_mismatched_revision_is_not_reconciled() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let (committed, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 41))
        .expect("initial commit");

    assert!(matches!(
        store.write(0, &cursor("2026-01-01 00:00:00", 42)),
        Err(CheckpointError::RevisionExists { .. })
    ));
    assert_eq!(store.read().expect("read"), Some(committed));
}

/// Scenario: several checkpoints are committed in sequence for one source.
/// Guarantees: exactly the two newest revisions are retained, bounding checkpoint disk usage
/// while keeping one prior revision available for inspection.
#[test]
fn retains_only_the_two_newest_revisions() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let mut revision = 0;
    for tie_breaker in 1..=5 {
        let (state, outcome) = store
            .write(revision, &cursor("2026-01-01 00:00:00", tie_breaker))
            .expect("commit");
        assert_eq!(outcome.cleanup_failures, 0);
        revision = state.revision;
    }

    let parent = store.prefix.parent().expect("checkpoint parent");
    let retained = std::fs::read_dir(parent)
        .expect("list checkpoint directory")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.ends_with(".json"))
        })
        .count();

    assert_eq!(retained, RETAINED_REVISIONS);
}

/// Scenario: a stale temporary file from an interrupted checkpoint write already exists.
/// Guarantees: the source lease allows a new checkpoint to prune its abandoned temporary file
/// before creating another, keeping failed attempts bounded on disk.
#[test]
fn stale_temporary_file_is_pruned() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let parent = store.prefix.parent().expect("checkpoint parent");
    let stale = parent.join(format!("{}stale.tmp", store.temporary_prefix()));
    std::fs::create_dir_all(parent).expect("create checkpoint directory");
    std::fs::write(&stale, b"unfinished checkpoint").expect("write stale temporary file");

    _ = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("commit through a unique temporary file");

    assert!(!stale.exists());
}

/// Scenario: a source identifier would exceed common filesystem component limits after encoding.
/// Guarantees: both temporary and final checkpoint names use fixed-length digests, while the
/// checkpoint payload retains the full source identity for compatibility validation.
#[test]
fn long_source_uses_bounded_checkpoint_filenames() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let source_id = "s".repeat(256);
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        &source_id,
        "fingerprint".to_owned(),
    );

    let (state, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("write checkpoint for long source");
    let final_path = revision_path(&store.prefix, state.revision);

    assert!(final_path.exists());
    assert!(
        final_path
            .file_name()
            .expect("checkpoint file name")
            .to_string_lossy()
            .len()
            <= 255
    );
    assert_eq!(
        store.read().expect("read checkpoint"),
        Some(state),
        "the full source identity in the payload must still validate"
    );
}

/// Scenario: an earlier build wrote a long source ID under the readable legacy filename.
/// Guarantees: startup validates and resumes that checkpoint, then the next acknowledged cursor
/// is installed under the bounded digest name without replaying from the initial cursor.
#[test]
fn long_source_resumes_legacy_checkpoint_before_migration() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let source_id = "s".repeat(150);
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        &source_id,
        "fingerprint".to_owned(),
    );
    let legacy = write_legacy_checkpoint(
        &store,
        store
            .older_legacy_prefix
            .as_ref()
            .expect("long source should have an older legacy path"),
        &cursor("2026-01-01 00:00:00", 1),
    );
    let legacy_revision = legacy.revision;

    assert_eq!(store.read().expect("read legacy checkpoint"), Some(legacy));

    let (migrated, _) = store
        .write(legacy_revision, &cursor("2026-01-01 00:00:01", 2))
        .expect("write bounded checkpoint");
    assert!(revision_path(&store.prefix, migrated.revision).exists());
    assert_eq!(
        store.read().expect("read bounded checkpoint"),
        Some(migrated)
    );
}

/// Scenario: a previous build wrote a short source ID under its readable filename.
/// Guarantees: upgrade reads that checkpoint and writes subsequent progress to the case-safe layout.
#[test]
fn short_source_resumes_legacy_checkpoint_before_migration() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = store(directory.path(), "fingerprint");
    let legacy = write_legacy_checkpoint(
        &store,
        &store.legacy_prefix,
        &cursor("2026-01-01 00:00:00", 1),
    );

    assert_eq!(
        store.read().expect("read old checkpoint"),
        Some(legacy.clone())
    );
    let (migrated, _) = store
        .write(legacy.revision, &cursor("2026-01-01 00:00:01", 2))
        .expect("write to new namespace");
    assert!(revision_path(&store.prefix, migrated.revision).exists());
    assert_eq!(store.read().expect("read new checkpoint"), Some(migrated));
}

/// Scenario: a prior build wrote a long source ID under its bounded digest name.
/// Guarantees: the versioned namespace can resume the current legacy format before migration.
#[test]
fn long_source_resumes_digest_legacy_checkpoint() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let source_id = "s".repeat(150);
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        &source_id,
        "fingerprint".to_owned(),
    );
    let legacy = write_legacy_checkpoint(
        &store,
        &store.legacy_prefix,
        &cursor("2026-01-01 00:00:00", 1),
    );

    assert_eq!(
        store.read().expect("read digest checkpoint"),
        Some(legacy.clone())
    );
    let (migrated, _) = store
        .write(legacy.revision, &cursor("2026-01-01 00:00:01", 2))
        .expect("write to versioned namespace");
    assert_eq!(store.read().expect("read new checkpoint"), Some(migrated));
}

/// Scenario: identity segments differ only by ASCII case on a case-insensitive filesystem.
/// Guarantees: sources and pipeline paths have different on-disk names and can hold separate
/// leases and checkpoints without sharing a revision file.
#[test]
fn case_variants_use_distinct_checkpoint_and_lease_names() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let first = CheckpointStore::new(
        directory.path(),
        "Group",
        "Pipeline",
        "Receiver",
        "Orders",
        "fingerprint".to_owned(),
    );
    let variants = [
        ("group", "Pipeline", "Receiver", "Orders"),
        ("Group", "pipeline", "Receiver", "Orders"),
        ("Group", "Pipeline", "receiver", "Orders"),
        ("Group", "Pipeline", "Receiver", "orders"),
    ];
    let first_lease = SourceLease::acquire(first.lease_key()).expect("first lease");
    let (first_state, _) = first
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("first write");
    for (group, pipeline, receiver, source) in variants {
        let other = CheckpointStore::new(
            directory.path(),
            group,
            pipeline,
            receiver,
            source,
            "fingerprint".to_owned(),
        );
        assert_ne!(
            first.prefix.to_string_lossy().to_lowercase(),
            other.prefix.to_string_lossy().to_lowercase()
        );
        let _other_lease = SourceLease::acquire(other.lease_key()).expect("independent lease");
        let (other_state, _) = other
            .write(0, &cursor("2026-01-01 00:00:00", 2))
            .expect("independent write");
        assert_eq!(other.read().expect("read other"), Some(other_state));
        assert_eq!(first.read().expect("read first"), Some(first_state.clone()));
    }
    drop(first_lease);
}

/// Scenario: old source filenames differ only by case on a case-insensitive filesystem.
/// Guarantees: an unrelated source cannot mistake legacy progress for absent state or adopt it.
#[test]
fn case_variant_legacy_checkpoint_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let first = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "receiver",
        "Orders",
        "fingerprint".to_owned(),
    );
    _ = write_legacy_checkpoint(
        &first,
        &first.legacy_prefix,
        &cursor("2026-01-01 00:00:00", 1),
    );
    let other = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "receiver",
        "orders",
        "fingerprint".to_owned(),
    );
    assert!(matches!(
        other.read(),
        Err(CheckpointError::LegacyNamespaceCollision { .. })
    ));
}

/// Scenario: old pipeline directories differ only by case and cannot identify their owner.
/// Guarantees: legacy recovery refuses to adopt a different pipeline's cursor.
#[test]
fn case_variant_legacy_directory_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let first = CheckpointStore::new(
        directory.path(),
        "Group",
        "pipeline",
        "receiver",
        "orders",
        "fingerprint".to_owned(),
    );
    _ = write_legacy_checkpoint(
        &first,
        &first.legacy_prefix,
        &cursor("2026-01-01 00:00:00", 1),
    );
    let other = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "receiver",
        "orders",
        "fingerprint".to_owned(),
    );
    assert!(matches!(
        other.read(),
        Err(CheckpointError::LegacyNamespaceCollision { .. })
    ));
}

/// Scenario: two receivers in one process target the same canonical checkpoint source.
/// Guarantees: only one owner holds the lease at a time, and releasing it makes the source
/// available with a higher durable ownership generation.
#[test]
fn source_lease_rejects_duplicate_owner() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let key = directory.path().join("source");
    let first = SourceLease::acquire(&key).expect("first lease");
    assert_eq!(first.generation(), 1);

    assert!(matches!(
        SourceLease::acquire(&key),
        Err(LeaseError::AlreadyOwned)
    ));

    drop(first);
    let restarted = SourceLease::acquire(&key).expect("restarted lease");
    assert_eq!(restarted.generation(), 2);
}

/// Scenario: two different checkpoint sources are configured in one process.
/// Guarantees: leasing is keyed by the canonical checkpoint identity, so unrelated sources do
/// not block one another.
#[test]
fn source_lease_is_keyed_by_checkpoint_identity() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let first = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        "orders",
        "fingerprint".to_owned(),
    );
    let second = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        "shipments",
        "fingerprint".to_owned(),
    );

    assert_ne!(first.lease_key(), second.lease_key());
    let _first = SourceLease::acquire(first.lease_key()).expect("first lease");
    let _second = SourceLease::acquire(second.lease_key()).expect("second lease");
}

/// Scenario: A production-like nested state path is leased, a cursor is committed, then the
/// owner restarts and reads it back.
/// Guarantees: Nested mkdir plus file and parent-directory fsync can start, persist, and
/// resume without fsyncing the filesystem root.
#[test]
fn nested_state_path_lease_checkpoint_and_restart() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let root = directory
        .path()
        .join("var")
        .join("lib")
        .join("otap")
        .join("prod");
    let store = store(&root, "fingerprint");
    let lease = SourceLease::acquire(store.lease_key()).expect("startup lease");
    let (committed, _) = store
        .write(0, &cursor("2026-01-01 00:00:00", 42))
        .expect("first checkpoint");
    drop(lease);

    let restarted = SourceLease::acquire(store.lease_key()).expect("restart lease");
    assert_eq!(restarted.generation(), 2);
    assert_eq!(store.read().expect("resume from disk"), Some(committed));
}

/// Scenario: Two Unix state roots have different non-UTF-8 bytes that both display as U+FFFD.
/// Guarantees: The leases, generation markers, and checkpoints remain in their respective
/// directories, and neither source identity is lost through string conversion.
#[cfg(unix)]
#[test]
fn non_utf8_state_roots_keep_lease_and_checkpoint_together() {
    use std::os::unix::ffi::OsStringExt;

    let directory = tempfile::tempdir().expect("temporary directory");
    let root = |byte| {
        directory.path().join(std::ffi::OsString::from_vec(vec![
            b's', b't', b'a', b't', b'e', b'-', byte,
        ]))
    };
    let first = store(&root(0xff), "fingerprint");
    let second = store(&root(0xfe), "fingerprint");
    assert_ne!(first.lease_key(), second.lease_key());
    assert_eq!(
        first.lease_key().to_string_lossy(),
        second.lease_key().to_string_lossy(),
        "lossy keys would collapse distinct directories"
    );

    let first_lease = SourceLease::acquire(first.lease_key()).expect("first lease");
    let second_lease = SourceLease::acquire(second.lease_key()).expect("second lease");
    assert!(matches!(
        SourceLease::acquire(first.lease_key()),
        Err(LeaseError::AlreadyOwned)
    ));
    CheckpointProcess::start(&root(0xff), "contend").wait_success();

    for (store, position) in [(&first, 1), (&second, 2)] {
        let (committed, _) = store
            .write(0, &cursor("2026-01-01 00:00:00", position))
            .expect("checkpoint write");
        assert_eq!(store.read().expect("checkpoint read"), Some(committed));

        let parent = store.lease_key().parent().expect("checkpoint parent");
        let names: Vec<_> = std::fs::read_dir(parent)
            .expect("lease and checkpoint directory")
            .map(|entry| entry.expect("directory entry").file_name())
            .collect();
        assert!(
            names
                .iter()
                .any(|name| name.to_string_lossy().ends_with(".lock"))
        );
        assert!(
            names
                .iter()
                .any(|name| name.to_string_lossy().contains(".generation."))
        );
        assert!(revision_path(store.lease_key(), 1).exists());
    }
    drop(first_lease);
    drop(second_lease);
}

/// Scenario: An engine state-root placeholder has a non-UTF-8 trailing component on Unix.
/// Guarantees: Placeholder expansion preserves that component's exact OS-native bytes.
#[cfg(unix)]
#[test]
fn state_root_expansion_preserves_non_utf8_suffix() {
    use std::os::unix::ffi::OsStringExt;

    let suffix = std::ffi::OsString::from_vec(vec![b'r', b'o', b'o', b't', b'-', 0xff]);
    let root = Path::new("${engine.state_dir}").join(&suffix);
    assert_eq!(
        expand_state_dir(&root).file_name(),
        Some(suffix.as_os_str())
    );
}

/// Scenario: identity segments contain path separators or traversal components.
/// Guarantees: encoded segments keep every checkpoint inside its configured root, so a crafted
/// source identifier cannot write outside the state directory.
#[test]
fn identity_segments_cannot_escape_the_checkpoint_root() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = CheckpointStore::new(
        directory.path(),
        "..",
        "pipe/line",
        "oracle",
        "../../escape",
        "fingerprint".to_owned(),
    );

    assert!(store.prefix.starts_with(directory.path()));
    assert!(
        !store
            .prefix
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    );
}

/// Scenario: Independent processes commit, reopen, and advance one source checkpoint.
/// Guarantees: Persisted cursors and ownership generations survive process boundaries without shared memory.
#[test]
fn checkpoint_survives_independent_process_restarts() {
    let directory = tempfile::tempdir().expect("temporary directory");
    CheckpointProcess::start(directory.path(), "commit").wait_success();
    CheckpointProcess::start(directory.path(), "resume").wait_success();

    let reopened = store(directory.path(), "fingerprint");
    let lease = SourceLease::acquire(reopened.lease_key()).expect("third owner");
    assert_eq!(lease.generation(), 3);
    let committed = reopened.read().expect("read").expect("committed state");
    assert_eq!(committed.revision, 2);
    assert_eq!(committed.cursor, cursor("2026-01-01 00:00:01", 42));
}

/// Scenario: A process is killed while holding a lease and a partially written temporary checkpoint.
/// Guarantees: The OS releases ownership, restart keeps only committed progress, and the next write removes the orphan.
#[test]
fn forced_process_exit_preserves_progress_and_releases_lease() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let mut child = CheckpointProcess::start(directory.path(), "hold-unfinished");
    child.wait_for_ready(directory.path());
    let reopened = store(directory.path(), "fingerprint");
    assert!(matches!(
        SourceLease::acquire(reopened.lease_key()),
        Err(LeaseError::AlreadyOwned)
    ));

    child.terminate();
    let lease = SourceLease::acquire(reopened.lease_key()).expect("owner after forced exit");
    assert_eq!(lease.generation(), 2);
    let committed = reopened
        .read()
        .expect("restart read")
        .expect("committed state");
    assert_eq!(committed.revision, 1);
    assert_eq!(committed.cursor, cursor("2026-01-01 00:00:00", 41));
    let parent = reopened.prefix.parent().expect("checkpoint directory");
    let orphan_count = || {
        std::fs::read_dir(parent)
            .expect("checkpoint directory")
            .map(|entry| entry.expect("directory entry").file_name())
            .filter(|name| {
                name.to_string_lossy()
                    .starts_with(&reopened.temporary_prefix())
            })
            .count()
    };
    assert_eq!(orphan_count(), 1);
    let (next, _) = reopened
        .write(committed.revision, &cursor("2026-01-01 00:00:01", 42))
        .expect("commit after restart");
    assert_eq!(orphan_count(), 0);
    assert_eq!(reopened.read().expect("read new commit"), Some(next));
}

/// Scenario: Restart uses incompatible configuration or encounters a corrupt newest revision.
/// Guarantees: Fresh processes fail closed rather than using an older checkpoint or treating state as absent.
#[test]
fn process_restart_rejects_incompatible_and_corrupt_state() {
    let directory = tempfile::tempdir().expect("temporary directory");
    CheckpointProcess::start(directory.path(), "commit").wait_success();
    CheckpointProcess::start(directory.path(), "incompatible").wait_success();
    CheckpointProcess::start(directory.path(), "resume").wait_success();
    let saved = store(directory.path(), "fingerprint");
    std::fs::write(
        revision_path(&saved.prefix, 2),
        b"corrupt newest checkpoint",
    )
    .expect("corrupt newest revision");
    CheckpointProcess::start(directory.path(), "corrupt").wait_success();
}

/// Scenario: Two processes access one backing state directory through distinct container bind mounts.
/// Guarantees: Their checkpoint lock, ownership generation, and orphan-cleanup namespace agree across mounts.
#[test]
fn same_storage_mounted_at_different_paths_has_one_owner() {
    let Some(root) = std::env::var_os(SHARED_STATE_ROOT) else {
        return;
    };
    let alias = PathBuf::from(std::env::var_os(SHARED_STATE_ALIAS).expect("second state mount"));
    let directory = tempfile::tempdir_in(root).expect("shared state directory");
    let alias = alias.join(directory.path().file_name().expect("state directory name"));
    std::fs::write(
        directory.path().join("mount-probe"),
        b"shared backing directory",
    )
    .expect("mount probe");
    assert_eq!(
        std::fs::read(alias.join("mount-probe")).expect("read through second mount"),
        b"shared backing directory"
    );

    let primary_store = store(directory.path(), "fingerprint");
    let alias_store = store(&alias, "fingerprint");
    assert_ne!(primary_store.lease_key(), alias_store.lease_key());
    assert_eq!(
        primary_store.temporary_prefix(),
        alias_store.temporary_prefix()
    );
    let owner = SourceLease::acquire(primary_store.lease_key()).expect("primary owner");
    _ = primary_store
        .write(0, &cursor("2026-01-01 00:00:00", 41))
        .expect("primary checkpoint");
    let (orphan, file) = primary_store
        .create_temporary(primary_store.prefix.parent().expect("parent"))
        .expect("orphan temporary file");
    drop(file);
    CheckpointProcess::start(&alias, "contend").wait_success();

    drop(owner);
    CheckpointProcess::start(&alias, "resume").wait_success();
    assert!(
        !orphan.exists(),
        "alias owner must remove the old mount's orphan"
    );
    let next_owner = SourceLease::acquire(primary_store.lease_key()).expect("next primary owner");
    assert_eq!(next_owner.generation(), 3);
    assert_eq!(
        primary_store.read().expect("read").expect("state").revision,
        2
    );
}

/// Scenario: A parent test starts a fresh executable for a checkpoint lifecycle operation.
/// Guarantees: Each operation exercises production file I/O and OS locking in an independent process.
#[test]
fn checkpoint_process_worker() {
    let Some(root) = std::env::var_os(PROCESS_ROOT) else {
        return;
    };
    let root = PathBuf::from(root);
    let mode = std::env::var(PROCESS_MODE).expect("checkpoint process mode");
    let fingerprint = if mode == "incompatible" {
        "different-fingerprint"
    } else {
        "fingerprint"
    };
    let store = store(&root, fingerprint);
    if mode == "contend" {
        assert!(matches!(
            SourceLease::acquire(store.lease_key()),
            Err(LeaseError::AlreadyOwned)
        ));
        return;
    }
    let lease = SourceLease::acquire(store.lease_key()).expect("child lease");
    match mode.as_str() {
        "commit" | "hold-unfinished" => {
            assert_eq!(lease.generation(), 1);
            assert_eq!(store.read().expect("new source"), None);
            _ = store
                .write(0, &cursor("2026-01-01 00:00:00", 41))
                .expect("child commit");
            if mode == "hold-unfinished" {
                let (_, mut unfinished) = store
                    .create_temporary(store.prefix.parent().expect("checkpoint directory"))
                    .expect("next revision temporary file");
                unfinished
                    .write_all(b"{\"unfinished\":")
                    .expect("partial write");
                unfinished.sync_all().expect("sync partial file");
                std::fs::write(root.join("ready"), b"ready").expect("signal parent");
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
        "resume" => {
            let previous = store
                .read()
                .expect("child restart")
                .expect("saved checkpoint");
            assert_eq!(previous.revision, 1);
            assert_eq!(previous.cursor, cursor("2026-01-01 00:00:00", 41));
            let (next, _) = store
                .write(previous.revision, &cursor("2026-01-01 00:00:01", 42))
                .expect("child resumed commit");
            assert_eq!(next.revision, 2);
            assert_eq!(store.read().expect("child readback"), Some(next));
        }
        "incompatible" => {
            assert!(matches!(
                store.read(),
                Err(CheckpointError::FingerprintMismatch { .. })
            ));
        }
        "corrupt" => {
            assert!(matches!(store.read(), Err(CheckpointError::Parse { .. })));
        }
        _ => panic!("unknown checkpoint process mode"),
    }
}
