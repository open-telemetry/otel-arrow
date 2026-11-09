// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Keep process creation outside the parallel store unit-test executable.
//! A fork can temporarily retain unrelated tests' flock descriptions even with
//! CLOEXEC, making their drop-and-reopen checks spuriously report Locked.

#![cfg(target_os = "linux")]

use otel_arrow_dfe_filelog_checkpoint::{
    FramingEncoding, FramingOnDecodeError, FramingProfileParams, MaxLogSizeBehavior, MultilineMode,
    ReplayConfig, WAL_MAX_TX_FRAME_BYTES,
    store::{CheckpointStore, StateRoot, StoreConfig, StoreError},
};
use std::fs::{self, File};
use std::os::fd::AsFd;
use std::os::unix::fs::PermissionsExt;

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
        replay: ReplayConfig::new(100, 1024 * 1024, &profile).expect("valid test configuration"),
        max_wal_bytes: WAL_MAX_TX_FRAME_BYTES,
        max_wal_transactions: 100,
    }
}

/// Scenario: A child process opens the exact namespace held by its parent.
/// Guarantees: Cross-process acquisition reports Locked without interfering with parallel store fixtures.
#[test]
fn cross_process_lock() {
    const CHILD_ROOT: &str = "FILELOG_LOCK_TEST_ROOT";
    if let Some(path) = std::env::var_os(CHILD_ROOT) {
        let file = File::open(path).unwrap();
        let root = StateRoot::from_borrowed_fd(file.as_fd()).unwrap();
        assert!(matches!(
            CheckpointStore::open(&root, "AppLogs", config()),
            Err(StoreError::Locked)
        ));
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let file = File::open(dir.path()).unwrap();
    let root = StateRoot::from_borrowed_fd(file.as_fd()).unwrap();
    let _store = CheckpointStore::open(&root, "AppLogs", config()).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cross_process_lock",
            "--nocapture",
            "--format=pretty",
        ])
        .env(CHILD_ROOT, dir.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("test cross_process_lock ... ok"),
        "{stdout}"
    );
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed;"),
        "{stdout}"
    );
}
