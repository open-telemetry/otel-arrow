// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Cross-process checkpoint namespace exclusion.
//!
//! Keep process creation in its own test binary. A spawned child can briefly
//! retain other threads' open file descriptions before exec closes them, causing
//! unrelated drop/reopen tests in the same process to observe a held flock.
#![cfg(target_os = "linux")]

use otel_arrow_dfe_filelog_checkpoint::{
    FramingEncoding, FramingOnDecodeError, FramingProfileParams, MaxLogSizeBehavior, MultilineMode,
    ReplayConfig, WAL_MAX_TX_FRAME_BYTES,
    store::{CheckpointStore, StateRoot, StoreConfig, StoreError},
};
use std::{
    fs::{self, File},
    os::{fd::AsFd, unix::fs::PermissionsExt},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};

const ID: &str = "AppLogs";

fn config() -> StoreConfig {
    let profile = FramingProfileParams {
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
    };
    StoreConfig {
        replay: ReplayConfig::new(100, 1024 * 1024, &profile).expect("replay config"),
        max_wal_bytes: WAL_MAX_TX_FRAME_BYTES,
        max_wal_transactions: 100,
    }
}

struct ChildGuard(Option<Child>);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn wait_for_child(child: Child) -> Output {
    let mut guard = ChildGuard(Some(child));
    let deadline = Instant::now() + Duration::from_secs(30);
    // A real process cannot use a virtual clock. Poll its actual exit state;
    // the guard kills and joins it if any assertion or wait fails.
    while guard
        .0
        .as_mut()
        .expect("child")
        .try_wait()
        .expect("child status")
        .is_none()
    {
        assert!(Instant::now() < deadline, "lock-test child did not exit");
        std::thread::sleep(Duration::from_millis(10));
    }
    guard
        .0
        .take()
        .expect("child")
        .wait_with_output()
        .expect("child output")
}

/// Scenario: a child process opens the checkpoint namespace held by its parent.
/// Guarantees: the OS lock excludes another process without inheriting sibling tests' namespace locks.
#[test]
fn cross_process_lock() {
    let dir = tempfile::tempdir().expect("isolated namespace");
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).expect("private root");
    let file = File::open(dir.path()).expect("state root");
    let root = StateRoot::from_borrowed_fd(file.as_fd()).expect("root capability");
    let store = CheckpointStore::open(&root, ID, config()).expect("parent store");
    let child = Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            "lock_child",
            "--ignored",
            "--nocapture",
            "--format=pretty",
        ])
        .env("FILELOG_LOCK_TEST_ROOT", dir.path())
        .env("RUST_BACKTRACE", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("child starts");
    let output = wait_for_child(child);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("test output");
    assert!(stdout.contains("test lock_child ... ok"), "{stdout}");
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed;"),
        "{stdout}"
    );
    drop(store);
    let _reopened =
        CheckpointStore::open(&root, ID, config()).expect("parent reopens after child join");
}

/// Scenario: the explicitly invoked helper opens its parent's already owned namespace.
/// Guarantees: cross-process contention reports Locked without changing checkpoint authority.
#[test]
#[ignore = "invoked in a subprocess by cross_process_lock"]
fn lock_child() {
    let path = std::env::var_os("FILELOG_LOCK_TEST_ROOT").expect("child root passed by parent");
    let file = File::open(path).expect("state root");
    let root = StateRoot::from_borrowed_fd(file.as_fd()).expect("root capability");
    assert!(matches!(
        CheckpointStore::open(&root, ID, config()),
        Err(StoreError::Locked)
    ));
}
