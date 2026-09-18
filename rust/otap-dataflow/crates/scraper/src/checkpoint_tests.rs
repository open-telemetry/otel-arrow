// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{LeaseError, SourceLease};

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
    let mut legacy_store = store.clone();
    legacy_store.prefix = store
        .legacy_prefix
        .clone()
        .expect("long source should have a legacy path");
    legacy_store.legacy_prefix = None;
    let (legacy, _) = legacy_store
        .write(0, &cursor("2026-01-01 00:00:00", 1))
        .expect("write legacy checkpoint");
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

/// Scenario: two receivers in one process target the same canonical checkpoint source.
/// Guarantees: only one owner holds the lease at a time, and releasing it makes the source
/// available with a higher durable ownership generation.
#[test]
fn source_lease_rejects_duplicate_owner() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let key = directory.path().join("source");
    let key = key.to_string_lossy();
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
    let _first = SourceLease::acquire(&first.lease_key()).expect("first lease");
    let _second = SourceLease::acquire(&second.lease_key()).expect("second lease");
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
