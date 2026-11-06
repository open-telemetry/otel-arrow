// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg(all(target_os = "linux", feature = "sqlite"))]
//! Persistent transaction and namespace contracts, using isolated local databases.

use otel_arrow_dfe_work_coordinator_state::{
    Error, Grant, Limits, OperationId, Outcome, ProgressUpdate, Store,
};
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

fn fixture(limits: Limits) -> (TempDir, Store) {
    let root = tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o700))
        .tempdir()
        .expect("private state root");
    let store = Store::open(root.path(), "namespace", limits).expect("open authority");
    (root, store)
}
fn grant(store: &Store, work: &str, term: u64) -> Grant {
    Grant {
        epoch: store.epoch(),
        work: work.into(),
        term,
        token: term.to_le_bytes().to_vec(),
    }
}
fn op(store: &Store, sequence: u64) -> OperationId {
    OperationId {
        epoch: store.epoch(),
        sequence,
    }
}
fn update(grant: &Grant, source: &str, revision: u64, bytes: &[u8]) -> ProgressUpdate {
    ProgressUpdate {
        grant: grant.clone(),
        source: source.into(),
        expected_revision: revision,
        progress: bytes.into(),
    }
}

/// Scenario: a publication covers two source records under different current assignments.
/// Guarantees: one stale CAS rejects the whole publication; a valid commit advances both sources exactly once.
#[test]
fn multi_assignment_publication_is_atomic_and_idempotent() {
    let (_root, mut store) = fixture(Limits::default());
    let a = grant(&store, "partition-a", 1);
    let b = grant(&store, "partition-b", 2);
    assert_eq!(
        store.activate(op(&store, 1), &a).expect("activate a"),
        Outcome::Applied
    );
    assert_eq!(
        store.activate(op(&store, 2), &b).expect("activate b"),
        Outcome::Applied
    );
    let bad = [
        update(&a, "file-a", 0, b"100"),
        update(&b, "file-b", 1, b"200"),
    ];
    assert_eq!(
        store
            .commit(op(&store, 3), &bad)
            .expect("reject publication"),
        Outcome::Rejected
    );
    assert!(store.progress("file-a").expect("read").is_none());
    assert!(store.progress("file-b").expect("read").is_none());
    let publication = [
        update(&a, "file-a", 0, b"100"),
        update(&b, "file-b", 0, b"200"),
    ];
    let operation = op(&store, 4);
    assert_eq!(
        store.commit(operation, &publication).expect("commit"),
        Outcome::Applied
    );
    assert_eq!(
        store
            .commit(operation, &publication)
            .expect("replay identical intent"),
        Outcome::Applied
    );
    assert_eq!(
        store.progress("file-a").expect("read").expect("a").revision,
        1
    );
    assert_eq!(
        store.progress("file-b").expect("read").expect("b").bytes,
        b"200"
    );
    let changed = [update(&a, "file-a", 1, b"other")];
    assert_eq!(
        store
            .commit(operation, &changed)
            .expect("operation collision"),
        Outcome::Rejected
    );
    assert_eq!(
        store.resolve(operation).expect("original receipt"),
        Outcome::Applied
    );
    assert_eq!(
        store.progress("file-a").expect("read").expect("a").bytes,
        b"100"
    );
}

/// Scenario: ownership retires and a successor reuses the stable assignment identity.
/// Guarantees: stale grants and retired terms cannot mutate progress or reactivate; the new grant preserves the source frontier.
#[test]
fn retirement_fences_progress_and_preserves_source_state() {
    let (_root, mut store) = fixture(Limits::default());
    let old = grant(&store, "partition", 7);
    assert_eq!(
        store.activate(op(&store, 1), &old).expect("activate"),
        Outcome::Applied
    );
    assert_eq!(
        store
            .commit(op(&store, 2), &[update(&old, "file", 0, b"100")])
            .expect("commit"),
        Outcome::Applied
    );
    let mut impostor = old.clone();
    impostor.token.push(0);
    assert_eq!(
        store
            .retire(op(&store, 3), &impostor)
            .expect("reject wrong identity"),
        Outcome::Rejected
    );
    assert_eq!(
        store.retire(op(&store, 4), &old).expect("retire"),
        Outcome::Applied
    );
    assert_eq!(
        store
            .activate(op(&store, 5), &old)
            .expect("reject retired term"),
        Outcome::Rejected
    );
    assert_eq!(
        store
            .commit(op(&store, 6), &[update(&old, "file", 1, b"200")])
            .expect("stale commit"),
        Outcome::Rejected
    );
    let next = grant(&store, "partition", 8);
    assert_eq!(
        store.activate(op(&store, 7), &next).expect("successor"),
        Outcome::Applied
    );
    assert_eq!(
        store
            .commit(op(&store, 8), &[update(&next, "file", 1, b"200")])
            .expect("resume"),
        Outcome::Applied
    );
    assert_eq!(
        store
            .progress("file")
            .expect("read")
            .expect("progress")
            .revision,
        2
    );
}

/// Scenario: a host dies or closes while durable grants and source state remain.
/// Guarantees: reopen durably allocates a fresh epoch, rejects old authority, preserves progress, and resolves retained historical receipts.
#[test]
fn reopen_fences_previous_epoch_without_resetting_progress() {
    let (root, mut store) = fixture(Limits::default());
    let old = grant(&store, "partition", 1);
    let committed = op(&store, 2);
    assert_eq!(
        store.activate(op(&store, 1), &old).expect("activate"),
        Outcome::Applied
    );
    assert_eq!(
        store
            .commit(committed, &[update(&old, "file", 0, b"100")])
            .expect("commit"),
        Outcome::Applied
    );
    let epoch = store.epoch();
    drop(store);
    let mut reopened = Store::open(root.path(), "namespace", Limits::default()).expect("recover");
    assert!(reopened.epoch() > epoch);
    assert_eq!(
        reopened.resolve(committed).expect("resolve lost reply"),
        Outcome::Applied
    );
    assert_eq!(
        reopened
            .commit(op(&reopened, 1), &[update(&old, "file", 1, b"bad")])
            .expect("stale epoch"),
        Outcome::Rejected
    );
    let next = grant(&reopened, "partition", 1);
    assert_eq!(
        reopened
            .activate(op(&reopened, 2), &next)
            .expect("new epoch grant"),
        Outcome::Applied
    );
    assert_eq!(
        reopened
            .commit(op(&reopened, 3), &[update(&next, "file", 1, b"200")])
            .expect("resume"),
        Outcome::Applied
    );
    assert_eq!(
        reopened
            .progress("file")
            .expect("read")
            .expect("progress")
            .revision,
        2
    );
}

/// Scenario: progress is offered under a different assignment despite a matching checkpoint revision.
/// Guarantees: a source's binding cannot be stolen by another valid grant; duplicate updates reject before any mutation.
#[test]
fn source_binding_and_transaction_shape_are_enforced() {
    let (_root, mut store) = fixture(Limits::default());
    let a = grant(&store, "a", 1);
    let b = grant(&store, "b", 2);
    let _ = store.activate(op(&store, 1), &a).expect("activate");
    let _ = store.activate(op(&store, 2), &b).expect("activate");
    let _ = store
        .commit(op(&store, 3), &[update(&a, "file", 0, b"a")])
        .expect("register source");
    assert_eq!(
        store
            .commit(op(&store, 4), &[update(&b, "file", 1, b"b")])
            .expect("wrong binding"),
        Outcome::Rejected
    );
    let duplicate = [update(&a, "file", 1, b"b"), update(&a, "file", 1, b"c")];
    assert!(matches!(
        store.commit(op(&store, 5), &duplicate),
        Err(Error::Invalid(_))
    ));
    assert_eq!(
        store
            .progress("file")
            .expect("read")
            .expect("progress")
            .bytes,
        b"a"
    );
}

/// Scenario: queued requests arrive out of order and an older receipt falls outside retention.
/// Guarantees: bounded reordering works; expired intents remain unknown and cannot execute again under the old ID.
#[test]
fn receipt_window_never_reexecutes_expired_intents() {
    let (_root, mut store) = fixture(Limits {
        receipts: 3,
        ..Limits::default()
    });
    let a = grant(&store, "a", 1);
    assert_eq!(
        store
            .activate(op(&store, 2), &a)
            .expect("activate later sequence"),
        Outcome::Applied
    );
    let committed = op(&store, 1);
    assert_eq!(
        store
            .commit(committed, &[update(&a, "file", 0, b"100")])
            .expect("earlier queued progress"),
        Outcome::Applied
    );
    let _ = store
        .commit(op(&store, 3), &[update(&a, "file", 1, b"200")])
        .expect("advance");
    let _ = store
        .commit(op(&store, 4), &[update(&a, "file", 2, b"300")])
        .expect("advance");
    assert_eq!(store.resolve(committed).expect("expired"), Outcome::Unknown);
    assert_eq!(
        store
            .commit(committed, &[update(&a, "file", 3, b"bad")])
            .expect("expired replay"),
        Outcome::Unknown
    );
    assert_eq!(
        store
            .progress("file")
            .expect("read")
            .expect("progress")
            .bytes,
        b"300"
    );
}

/// Scenario: a publication exceeds aggregate source bounds and the assignment tombstone budget fills.
/// Guarantees: quota pressure changes no source state and never evicts a retired term to admit stale ownership.
#[test]
fn capacity_rejection_is_atomic_and_does_not_evict_fences() {
    let (_root, mut store) = fixture(Limits {
        assignments: 1,
        sources: 1,
        progress_bytes: 4,
        source_bytes: 4,
        ..Limits::default()
    });
    let a = grant(&store, "a", 1);
    let _ = store.activate(op(&store, 1), &a).expect("activate");
    let batch = [update(&a, "one", 0, b"a"), update(&a, "two", 0, b"b")];
    assert_eq!(
        store.commit(op(&store, 2), &batch).expect("quota"),
        Outcome::Rejected
    );
    assert!(store.progress("one").expect("read").is_none());
    let _ = store.retire(op(&store, 3), &a).expect("retire");
    let b = grant(&store, "b", 2);
    assert_eq!(
        store.activate(op(&store, 4), &b).expect("tombstone quota"),
        Outcome::Rejected
    );
    assert_eq!(
        store
            .activate(op(&store, 5), &a)
            .expect("old term remains fenced"),
        Outcome::Rejected
    );
}

/// Scenario: a publication grows one source while shrinking another within the same aggregate byte quota.
/// Guarantees: quota admission uses the atomic final state rather than the order in which updates were supplied.
#[test]
fn aggregate_byte_quota_is_independent_of_update_order() {
    let (_root, mut store) = fixture(Limits {
        sources: 2,
        progress_bytes: 4,
        source_bytes: 4,
        ..Limits::default()
    });
    let a = grant(&store, "a", 1);
    let _ = store.activate(op(&store, 1), &a).expect("activate");
    assert_eq!(
        store
            .commit(
                op(&store, 2),
                &[
                    update(&a, "first", 0, b"a"),
                    update(&a, "second", 0, b"bbb")
                ]
            )
            .expect("initial progress"),
        Outcome::Applied
    );
    assert_eq!(
        store
            .commit(
                op(&store, 3),
                &[
                    update(&a, "first", 1, b"aaa"),
                    update(&a, "second", 1, b"b")
                ]
            )
            .expect("grow before shrink"),
        Outcome::Applied
    );
    assert_eq!(
        store.progress("first").expect("read").expect("first").bytes,
        b"aaa"
    );
    assert_eq!(
        store
            .progress("second")
            .expect("read")
            .expect("second")
            .bytes,
        b"b"
    );
}

/// Scenario: the same namespace is opened twice and its authoritative database later disappears.
/// Guarantees: independent local owners are excluded and missing committed state is never recreated empty.
#[test]
fn exclusive_namespace_and_missing_authority_fail_closed() {
    let (root, store) = fixture(Limits::default());
    assert!(matches!(
        Store::open(root.path(), "namespace", Limits::default()),
        Err(Error::Locked)
    ));
    drop(store);
    let namespace = std::fs::read_dir(root.path())
        .expect("namespace")
        .next()
        .expect("entry")
        .expect("entry")
        .path();
    std::fs::remove_file(namespace.join("state.sqlite")).expect("simulate missing authority");
    assert!(matches!(
        Store::open(root.path(), "namespace", Limits::default()),
        Err(Error::Untrusted(_))
    ));
}

/// Scenario: SQLite cannot fit a transaction within the configured physical database bound.
/// Guarantees: the instance latches unknown I/O outcome and rejects further writes and receipt claims until recovery.
#[test]
fn storage_failure_poisons_the_writer_instead_of_retrying_sync() {
    let (root, mut store) = fixture(Limits {
        database_bytes: 1024 * 1024,
        source_bytes: 2 * 1024 * 1024,
        transaction_bytes: 4 * 1024 * 1024,
        ..Limits::default()
    });
    let a = grant(&store, "a", 1);
    let _ = store.activate(op(&store, 1), &a).expect("activate");
    let transaction = op(&store, 2);
    assert!(matches!(
        store.commit(
            transaction,
            &[update(&a, "large", 0, &vec![0; 2 * 1024 * 1024])]
        ),
        Err(Error::Storage(_))
    ));
    assert!(matches!(store.resolve(transaction), Err(Error::Poisoned)));
    assert!(matches!(
        store.retire(op(&store, 3), &a),
        Err(Error::Poisoned)
    ));
    drop(store);
    let mut reopened = Store::open(
        root.path(),
        "namespace",
        Limits {
            database_bytes: 1024 * 1024,
            ..Limits::default()
        },
    )
    .expect("recover rolled-back transaction");
    assert!(
        reopened
            .progress("large")
            .expect("read after recovery")
            .is_none()
    );
}
