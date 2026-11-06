// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![cfg(all(target_os = "linux", feature = "test-utils"))]
//! Real process interruption at SQLite transaction boundaries. This separate
//! binary keeps process spawning away from concurrent lock-owning unit tests.

use otel_arrow_dfe_work_coordinator_state::{
    FaultPoint, Grant, Limits, OperationId, Outcome, ProgressUpdate, Store,
};
use std::os::unix::fs::PermissionsExt;
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const BYTES: usize = 1024 * 1024;
fn limits() -> Limits {
    Limits {
        source_bytes: BYTES * 2,
        transaction_bytes: BYTES * 8,
        ..Limits::default()
    }
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
fn update(grant: &Grant, source: &str, expected_revision: u64, byte: u8) -> ProgressUpdate {
    ProgressUpdate {
        grant: grant.clone(),
        source: source.into(),
        expected_revision,
        progress: vec![byte; BYTES],
    }
}
struct ChildGuard(Child);
impl ChildGuard {
    fn interrupted(&mut self) {
        // A real child and filesystem cannot use Tokio's virtual clock. Poll
        // only this process, within a generous deadline, and always reap it.
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = self.0.try_wait().expect("child status") {
                assert_eq!(
                    status.code(),
                    Some(77),
                    "the requested crash boundary must execute"
                );
                return;
            }
            assert!(
                Instant::now() < deadline,
                "storage child interruption deadline"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Scenario: a process dies during activation, retirement, or a multi-source commit, including after commit but before its reply.
/// Guarantees: reopen preserves all-or-none source state and exact receipts; old process authority cannot write into the new epoch.
#[test]
fn interrupted_mutations_recover_atomically_and_fence_old_process() {
    for (kind, point) in [
        ("activate", "before"),
        ("activate", "after"),
        ("retire", "before"),
        ("retire", "after"),
        ("progress", "first"),
        ("progress", "before"),
        ("progress", "after"),
    ] {
        let root = tempfile::Builder::new()
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir()
            .expect("private crash fixture");
        let mut store = Store::open(root.path(), "crash", limits()).expect("initial store");
        let a = grant(&store, "a", 1);
        let b = grant(&store, "b", 2);
        let _ = store.activate(op(&store, 1), &a).expect("activate");
        let _ = store.activate(op(&store, 2), &b).expect("activate");
        assert_eq!(
            store
                .commit(
                    op(&store, 3),
                    &[update(&a, "file-a", 0, b'o'), update(&b, "file-b", 0, b'p')]
                )
                .expect("baseline"),
            Outcome::Applied
        );
        let child_epoch = store.epoch() + 1;
        drop(store);
        let mut child = ChildGuard(
            Command::new(std::env::current_exe().expect("test executable"))
                .args(["--ignored", "--exact", "crash_child"])
                .env("DFE_COORD_CRASH_ROOT", root.path())
                .env("DFE_COORD_CRASH_KIND", kind)
                .env("DFE_COORD_CRASH_POINT", point)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("crash child"),
        );
        child.interrupted();
        let mut recovered = Store::open(root.path(), "crash", limits()).expect("SQLite recovery");
        assert_eq!(recovered.epoch(), child_epoch + 1);
        let committed = kind == "progress" && point == "after";
        for (source, old, new) in [("file-a", b'o', b'a'), ("file-b", b'p', b'b')] {
            let progress = recovered
                .progress(source)
                .expect("read")
                .expect("source retained");
            assert_eq!(
                progress.revision,
                if committed { 2 } else { 1 },
                "{kind}/{point}"
            );
            assert_eq!(progress.bytes.len(), BYTES);
            assert!(
                progress
                    .bytes
                    .iter()
                    .all(|b| *b == if committed { new } else { old }),
                "{kind}/{point}: no torn or partial source state"
            );
        }
        let sequence = if kind == "activate" { 1 } else { 3 };
        let receipt = OperationId {
            epoch: child_epoch,
            sequence,
        };
        assert_eq!(
            recovered
                .resolve(receipt)
                .expect("resolve interrupted reply"),
            if point == "after" {
                Outcome::Applied
            } else {
                Outcome::Unknown
            },
            "{kind}/{point}"
        );
        let obsolete = Grant {
            epoch: child_epoch,
            work: "a".into(),
            term: 1,
            token: 1u64.to_le_bytes().to_vec(),
        };
        assert_eq!(
            recovered
                .commit(
                    op(&recovered, 1),
                    &[update(
                        &obsolete,
                        "file-a",
                        if committed { 2 } else { 1 },
                        b'x'
                    )]
                )
                .expect("reject stale process"),
            Outcome::Rejected
        );
    }
}

/// Scenario: the parent invokes this isolated helper with one exact interruption boundary.
/// Guarantees: exit happens without unwinding storage, after real SQLite journal writes for interrupted progress.
#[test]
#[ignore = "invoked only by interrupted_mutations_recover_atomically_and_fence_old_process"]
fn crash_child() {
    let root = std::path::PathBuf::from(std::env::var_os("DFE_COORD_CRASH_ROOT").expect("root"));
    let kind = std::env::var("DFE_COORD_CRASH_KIND").expect("kind");
    let point = match std::env::var("DFE_COORD_CRASH_POINT")
        .expect("point")
        .as_str()
    {
        "first" => FaultPoint::AfterFirstUpdate,
        "before" => FaultPoint::BeforeCommit,
        "after" => FaultPoint::AfterCommit,
        _ => panic!("unknown crash boundary"),
    };
    let mut store = Store::open(&root, "crash", limits()).expect("child ownership");
    let a = grant(&store, "a", 1);
    let b = grant(&store, "b", 2);
    if kind != "activate" {
        let _ = store.activate(op(&store, 1), &a).expect("activate");
        let _ = store.activate(op(&store, 2), &b).expect("activate");
    }
    let require_journal = kind == "progress" && point != FaultPoint::AfterCommit;
    store.set_fault_hook(move |at| {
        if at == point {
            if require_journal {
                let namespace = std::fs::read_dir(&root)
                    .expect("namespace")
                    .next()
                    .expect("entry")
                    .expect("entry")
                    .path();
                assert!(
                    std::fs::metadata(namespace.join("state.sqlite-journal"))
                        .expect("real rollback journal")
                        .len()
                        > 0
                );
            }
            std::process::exit(77);
        }
    });
    match kind.as_str() {
        "activate" => {
            let _ = store.activate(op(&store, 1), &a).expect("activate");
        }
        "retire" => {
            let _ = store.retire(op(&store, 3), &a).expect("retire");
        }
        "progress" => {
            let _ = store
                .commit(
                    op(&store, 3),
                    &[update(&a, "file-a", 1, b'a'), update(&b, "file-b", 1, b'b')],
                )
                .expect("commit");
        }
        _ => panic!("unknown mutation"),
    }
    panic!("requested crash boundary was not reached");
}
