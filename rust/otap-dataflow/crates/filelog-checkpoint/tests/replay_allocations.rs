// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated allocation regression for the largest replay transaction class.

use otel_arrow_dfe_filelog_checkpoint::{
    AdvisoryPath, CommittedFrontierGuard, FileId, FramingEncoding, FramingOnDecodeError,
    FramingProfileParams, FramingResume, LifecycleState, Locator, MaxLogSizeBehavior,
    MultilineMode, Operation, ReplayConfig, ReplayState, SnapshotRecord, Transaction,
    UpdateProgress, encode_checkpoint, encode_transaction,
};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn measure(fingerprint_bytes: u16, path_bytes: usize) -> dhat::HeapStats {
    const COUNT: u32 = 128;
    let profile = FramingProfileParams {
        fingerprint_profile_version: 1,
        fingerprint_bytes,
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
    let digest = profile.digest().expect("profile");
    let path = AdvisoryPath::from_unix_bytes(&vec![b'x'; path_bytes]).expect("path");
    let records: Vec<_> = (0..COUNT)
        .map(|n| SnapshotRecord {
            file_id: FileId::from_bytes(u128::from(n).to_be_bytes()),
            file_epoch: 1,
            committed_offset: 0,
            committed_frontier_guard: CommittedFrontierGuard::empty(),
            fingerprint: vec![b'x'; usize::from(fingerprint_bytes)],
            ignored_header_bytes: 0,
            locator: Locator::PosixDevIno {
                dev: 1,
                ino: u64::from(n),
            },
            framing_profile_version: 1,
            framing_profile_digest: digest,
            framing_resume: FramingResume::Clean,
            lifecycle_state: LifecycleState::Active,
            quarantine_evidence: None,
            last_seen_time_unix_nano: 0,
            advisory_path: path.clone(),
        })
        .collect();
    let operations = records
        .iter()
        .map(|r| {
            Operation::UpdateProgress(UpdateProgress {
                file_id: r.file_id,
                expected_committed_offset: 0,
                expected_file_epoch: 1,
                new_committed_offset: 1,
                new_committed_frontier_guard: CommittedFrontierGuard::compute(1, b"x")
                    .expect("guard"),
                new_framing_resume: FramingResume::Clean,
                new_last_seen_time_unix_nano: 1,
                finalize: false,
            })
        })
        .collect();
    let bytes = encode_transaction(&Transaction {
        sequence: 1,
        operations,
    })
    .expect("transaction");
    let snapshot = encode_checkpoint(1, "allocation", &records).expect("snapshot");
    let config = ReplayConfig::new(COUNT, 16 * 1024 * 1024, &profile).expect("config");
    let (mut state, _) =
        ReplayState::from_checkpoint(&snapshot, "allocation", config).expect("state");
    // Snapshot decoding and input construction are outside the measured region.
    let profiler = dhat::Profiler::builder().testing().build();
    let _ = state.replay_next(&bytes).expect("replay");
    let stats = dhat::HeapStats::get();
    drop(profiler);
    assert_eq!(state.last_sequence(), 1);
    assert!(state.records().all(|r| r.committed_offset == 1));
    stats
}

/// Scenario: Progress replay touches files with minimal versus maximum fingerprint/path payloads.
/// Guarantees: Transaction allocation is independent of unchanged evidence size and stays below 128 KiB for 128 updates.
#[test]
fn progress_allocations_do_not_scale_with_stored_evidence() {
    let small = measure(16, 1);
    let large = measure(u16::MAX, 4096);
    assert_eq!(large.total_bytes, small.total_bytes);
    assert!(
        large.total_bytes < 128 * 1024,
        "allocated {} bytes",
        large.total_bytes
    );
    assert!(
        large.max_bytes < 128 * 1024,
        "retained {} bytes",
        large.max_bytes
    );
}
