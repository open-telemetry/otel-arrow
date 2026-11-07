// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared setup for complete-compaction measurements; setup is outside the measured call.

use otel_arrow_dfe_filelog_checkpoint::{
    self as cp,
    store::{CheckpointStore, Durability, StateRoot, StoreConfig},
};
use std::{
    fs,
    os::{fd::AsFd, unix::fs::PermissionsExt},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const ID: &str = "compaction-measurement";

pub struct Fixture {
    _directory: tempfile::TempDir,
    root: StateRoot,
    config: StoreConfig,
    pub count: u32,
    pub fingerprint: u16,
    pub path_bytes: usize,
}

impl Fixture {
    pub fn from_args() -> Result<Self> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        if args.len() != 3 {
            return Err("expected: record-count fingerprint-bytes path-bytes".into());
        }
        let count: u32 = args[0].parse()?;
        let fingerprint: u16 = args[1].parse()?;
        let path_bytes: usize = args[2].parse()?;
        if count == 0 || path_bytes > 4096 {
            return Err("require nonzero population and path length <= 4096".into());
        }
        let profile = cp::FramingProfileParams {
            fingerprint_profile_version: 1,
            fingerprint_bytes: fingerprint,
            ignored_header_bytes: 0,
            encoding: cp::FramingEncoding::Utf8,
            on_decode_error: cp::FramingOnDecodeError::PreserveRaw,
            multiline_mode: cp::MultilineMode::Newline,
            max_line_bytes: 1024,
            max_record_bytes: 1024,
            max_log_size_behavior: cp::MaxLogSizeBehavior::Split,
            max_multiline_lines: 100,
            force_flush_period_millis: 0,
        };
        let bounds = cp::CheckpointBounds::for_population(
            count,
            fingerprint,
            cp::WAL_MAX_TX_FRAME_BYTES,
            10_000,
        )?;
        let config = StoreConfig {
            replay: cp::ReplayConfig::new(count, bounds.snapshot_bytes, &profile)?,
            max_wal_bytes: cp::WAL_MAX_TX_FRAME_BYTES,
            max_wal_transactions: 10_000,
        };
        let directory = tempfile::tempdir()?;
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))?;
        let root = StateRoot::from_borrowed_fd(fs::File::open(directory.path())?.as_fd())?;
        drop(CheckpointStore::open(&root, ID, config)?);
        let digest = profile.digest()?;
        let path = cp::AdvisoryPath::from_unix_bytes(&vec![b'p'; path_bytes])?;
        let records: Vec<_> = (0..count)
            .map(|n| cp::SnapshotRecord {
                file_id: cp::FileId::from_bytes(u128::from(n).to_be_bytes()),
                file_epoch: 1,
                committed_offset: 0,
                committed_frontier_guard: cp::CommittedFrontierGuard::empty(),
                fingerprint: vec![b'x'; usize::from(fingerprint)],
                ignored_header_bytes: 0,
                locator: cp::Locator::PosixDevIno {
                    dev: 1,
                    ino: u64::from(n) + 1,
                },
                framing_profile_version: 1,
                framing_profile_digest: digest,
                framing_resume: cp::FramingResume::Clean,
                lifecycle_state: cp::LifecycleState::Active,
                quarantine_evidence: None,
                last_seen_time_unix_nano: 1,
                advisory_path: path.clone(),
            })
            .collect();
        let database = directory
            .path()
            .join("filelog/@v1")
            .join(cp::namespace_component(ID)?)
            .join("checkpoint.db");
        fs::write(&database, cp::encode_checkpoint(7, ID, &records)?)?;
        fs::File::open(database)?.sync_all()?;
        Ok(Self {
            _directory: directory,
            root,
            config,
            count,
            fingerprint,
            path_bytes,
        })
    }

    pub fn open(&self) -> Result<CheckpointStore> {
        let mut store = CheckpointStore::open(&self.root, ID, self.config)?;
        let _ = store.append(
            vec![cp::Operation::UpdateProgress(cp::UpdateProgress {
                file_id: cp::FileId::from_bytes([0; 16]),
                expected_file_epoch: 1,
                expected_committed_offset: 0,
                new_committed_offset: 1,
                new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(1, b"x")?,
                new_framing_resume: cp::FramingResume::Clean,
                new_last_seen_time_unix_nano: 2,
                finalize: false,
            })],
            Durability::Deferred,
        )?;
        Ok(store)
    }

    pub fn verify(&self, store: &CheckpointStore) -> Result<()> {
        assert_eq!(store.state()?.generation(), 8);
        assert_eq!(store.state()?.records().len(), self.count as usize);
        assert_eq!(
            store
                .state()?
                .get(&cp::FileId::from_bytes([0; 16]))
                .expect("retained record")
                .committed_offset,
            1
        );
        assert_eq!(store.wal_bytes(), 0);
        Ok(())
    }
}
