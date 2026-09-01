// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

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

/// Scenario: two receivers in one process target the same canonical checkpoint source.
/// Guarantees: only one owner holds the lease at a time, and releasing it makes the source
/// available again so a restarted receiver is not permanently locked out.
#[test]
fn source_lease_rejects_duplicate_owner() {
    let key = "database-checkpoint-test-source-lease";
    let first = SourceLease::acquire(key).expect("first lease");

    assert!(matches!(
        SourceLease::acquire(key),
        Err(LeaseError::AlreadyOwned)
    ));

    drop(first);
    assert!(SourceLease::acquire(key).is_ok());
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
