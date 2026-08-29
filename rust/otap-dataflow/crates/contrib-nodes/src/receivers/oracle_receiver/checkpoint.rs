// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Durable, revisioned Oracle watermark checkpoints.

use crate::receivers::sql_polling::CompoundWatermark;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const ENVELOPE_VERSION: u8 = 1;
const MAX_CHECKPOINT_BYTES: u64 = 16 * 1024;
const RETAINED_REVISIONS: usize = 2;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CheckpointPayload {
    version: u8,
    revision: u64,
    config_fingerprint: String,
    watermark: CompoundWatermark,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CheckpointEnvelope {
    payload: CheckpointPayload,
    checksum: String,
}

/// Loaded durable checkpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CheckpointState {
    pub(super) revision: u64,
    pub(super) watermark: CompoundWatermark,
}

/// Result of a durable checkpoint write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WriteOutcome {
    pub(super) cleanup_failures: usize,
}

/// Stable checkpoint location and expected configuration identity.
#[derive(Clone, Debug)]
pub(super) struct CheckpointStore {
    prefix: PathBuf,
    config_fingerprint: String,
    #[cfg(test)]
    post_install_failures: Arc<AtomicUsize>,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum CheckpointError {
    #[error("checkpoint path has no parent: {path}")]
    NoParent { path: PathBuf },
    #[error("failed to inspect checkpoint directory {path}: {source}")]
    Inspect { path: PathBuf, source: io::Error },
    #[error("failed to inspect checkpoint {path}: {source}")]
    Metadata { path: PathBuf, source: io::Error },
    #[error("checkpoint {path} exceeds {MAX_CHECKPOINT_BYTES} bytes")]
    TooLarge { path: PathBuf },
    #[error("failed to read checkpoint {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to parse checkpoint {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unsupported checkpoint version {version} in {path}")]
    UnsupportedVersion { path: PathBuf, version: u8 },
    #[error("checkpoint checksum mismatch in {path}")]
    ChecksumMismatch { path: PathBuf },
    #[error("checkpoint revision mismatch in {path}")]
    RevisionMismatch { path: PathBuf },
    #[error("checkpoint configuration fingerprint mismatch in {path}")]
    FingerprintMismatch { path: PathBuf },
    #[error("checkpoint revision overflow")]
    RevisionOverflow,
    #[error("failed to encode checkpoint: {source}")]
    Encode { source: serde_json::Error },
    #[error("failed to create checkpoint directory {path}: {source}")]
    CreateDirectory { path: PathBuf, source: io::Error },
    #[error("failed to create checkpoint temporary file {path}: {source}")]
    CreateTemporary { path: PathBuf, source: io::Error },
    #[error("failed to write checkpoint temporary file {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("failed to fsync checkpoint temporary file {path}: {source}")]
    FsyncFile { path: PathBuf, source: io::Error },
    #[error("checkpoint revision already exists at {path}")]
    RevisionExists { path: PathBuf },
    #[error("failed to install checkpoint {tmp} -> {path}: {source}")]
    Rename {
        tmp: PathBuf,
        path: PathBuf,
        source: io::Error,
    },
    #[cfg(unix)]
    #[error("failed to open checkpoint directory {path} for fsync: {source}")]
    OpenDirectory { path: PathBuf, source: io::Error },
    #[cfg(unix)]
    #[error("failed to fsync checkpoint directory {path}: {source}")]
    FsyncDirectory { path: PathBuf, source: io::Error },
    #[cfg(test)]
    #[error("injected failure after installing checkpoint {path}")]
    InjectedPostInstall { path: PathBuf },
}

impl CheckpointError {
    fn could_have_installed(&self) -> bool {
        match self {
            Self::RevisionExists { .. } | Self::Rename { .. } => true,
            #[cfg(unix)]
            Self::FsyncDirectory { .. } => true,
            #[cfg(test)]
            Self::InjectedPostInstall { .. } => true,
            _ => false,
        }
    }
}

impl CheckpointStore {
    pub(super) fn new(
        root: &Path,
        pipeline_group_id: &str,
        pipeline_id: &str,
        receiver_name: &str,
        source_id: &str,
        config_fingerprint: String,
    ) -> Self {
        let mut prefix = expand_state_dir(root);
        prefix.push(encode_path_segment(pipeline_group_id));
        prefix.push(encode_path_segment(pipeline_id));
        prefix.push(encode_path_segment(receiver_name));
        prefix.push(format!("{}.checkpoint", encode_path_segment(source_id)));
        Self {
            prefix,
            config_fingerprint,
            #[cfg(test)]
            post_install_failures: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(super) fn lease_key(&self) -> String {
        self.prefix.to_string_lossy().into_owned()
    }

    pub(super) fn read(&self) -> Result<Option<CheckpointState>, CheckpointError> {
        let Some(parent) = self.prefix.parent() else {
            return Err(CheckpointError::NoParent {
                path: self.prefix.clone(),
            });
        };
        let entries = match std::fs::read_dir(parent) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(CheckpointError::Inspect {
                    path: parent.to_path_buf(),
                    source,
                });
            }
        };
        let prefix_name = self
            .prefix
            .file_name()
            .and_then(OsStr::to_str)
            .expect("checkpoint prefix uses UTF-8 path segments");
        let mut revisions = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| CheckpointError::Inspect {
                path: parent.to_path_buf(),
                source,
            })?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if let Some(revision) = parse_revision(prefix_name, name) {
                revisions.push((revision, entry.path()));
            }
        }
        revisions.sort_unstable_by_key(|entry| std::cmp::Reverse(entry.0));
        let Some((filename_revision, path)) = revisions.into_iter().next() else {
            return Ok(None);
        };
        self.read_revision(&path, filename_revision).map(Some)
    }

    fn read_revision(
        &self,
        path: &Path,
        filename_revision: u64,
    ) -> Result<CheckpointState, CheckpointError> {
        let metadata = std::fs::metadata(path).map_err(|source| CheckpointError::Metadata {
            path: path.to_path_buf(),
            source,
        })?;
        if metadata.len() > MAX_CHECKPOINT_BYTES {
            return Err(CheckpointError::TooLarge {
                path: path.to_path_buf(),
            });
        }
        let bytes = std::fs::read(path).map_err(|source| CheckpointError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let envelope: CheckpointEnvelope =
            serde_json::from_slice(&bytes).map_err(|source| CheckpointError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        if envelope.payload.version != ENVELOPE_VERSION {
            return Err(CheckpointError::UnsupportedVersion {
                path: path.to_path_buf(),
                version: envelope.payload.version,
            });
        }
        if envelope.payload.revision != filename_revision {
            return Err(CheckpointError::RevisionMismatch {
                path: path.to_path_buf(),
            });
        }
        if envelope.payload.config_fingerprint != self.config_fingerprint {
            return Err(CheckpointError::FingerprintMismatch {
                path: path.to_path_buf(),
            });
        }
        let checksum = checksum(&envelope.payload)?;
        if envelope.checksum != checksum {
            return Err(CheckpointError::ChecksumMismatch {
                path: path.to_path_buf(),
            });
        }
        Ok(CheckpointState {
            revision: envelope.payload.revision,
            watermark: envelope.payload.watermark,
        })
    }

    pub(super) fn write(
        &self,
        current_revision: u64,
        watermark: &CompoundWatermark,
    ) -> Result<(CheckpointState, WriteOutcome), CheckpointError> {
        let revision = current_revision
            .checked_add(1)
            .ok_or(CheckpointError::RevisionOverflow)?;
        let payload = CheckpointPayload {
            version: ENVELOPE_VERSION,
            revision,
            config_fingerprint: self.config_fingerprint.clone(),
            watermark: watermark.clone(),
        };
        let envelope = CheckpointEnvelope {
            checksum: checksum(&payload)?,
            payload,
        };
        let bytes =
            serde_json::to_vec(&envelope).map_err(|source| CheckpointError::Encode { source })?;
        let parent = self
            .prefix
            .parent()
            .ok_or_else(|| CheckpointError::NoParent {
                path: self.prefix.clone(),
            })?;
        std::fs::create_dir_all(parent).map_err(|source| CheckpointError::CreateDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
        let final_path = revision_path(&self.prefix, revision);
        let install_result = (|| {
            if final_path.exists() {
                return Err(CheckpointError::RevisionExists {
                    path: final_path.clone(),
                });
            }
            let tmp = self
                .prefix
                .with_extension(format!("checkpoint.{revision:020}.tmp"));
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&tmp)
                .map_err(|source| CheckpointError::CreateTemporary {
                    path: tmp.clone(),
                    source,
                })?;
            file.write_all(&bytes)
                .map_err(|source| CheckpointError::Write {
                    path: tmp.clone(),
                    source,
                })?;
            file.sync_all()
                .map_err(|source| CheckpointError::FsyncFile {
                    path: tmp.clone(),
                    source,
                })?;
            drop(file);
            std::fs::rename(&tmp, &final_path).map_err(|source| CheckpointError::Rename {
                tmp,
                path: final_path.clone(),
                source,
            })?;
            self.sync_parent(parent, &final_path)
        })();
        if let Err(error) = install_result
            && (!error.could_have_installed()
                || !self.reconcile_installed(&final_path, revision, watermark, parent))
        {
            return Err(error);
        }
        let cleanup_failures = self.cleanup_old_revisions(revision);
        Ok((
            CheckpointState {
                revision,
                watermark: watermark.clone(),
            },
            WriteOutcome { cleanup_failures },
        ))
    }

    fn reconcile_installed(
        &self,
        path: &Path,
        revision: u64,
        watermark: &CompoundWatermark,
        parent: &Path,
    ) -> bool {
        let expected = CheckpointState {
            revision,
            watermark: watermark.clone(),
        };
        self.read_revision(path, revision)
            .is_ok_and(|installed| installed == expected)
            && self.sync_parent(parent, path).is_ok()
    }

    fn sync_parent(&self, parent: &Path, _installed: &Path) -> Result<(), CheckpointError> {
        #[cfg(test)]
        if self
            .post_install_failures
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok()
        {
            return Err(CheckpointError::InjectedPostInstall {
                path: _installed.to_path_buf(),
            });
        }
        sync_parent_directory(parent)
    }

    #[cfg(test)]
    fn inject_post_install_failures(&self, count: usize) {
        self.post_install_failures.store(count, Ordering::SeqCst);
    }

    fn cleanup_old_revisions(&self, newest: u64) -> usize {
        let Some(parent) = self.prefix.parent() else {
            return 1;
        };
        let Some(prefix_name) = self.prefix.file_name().and_then(OsStr::to_str) else {
            return 1;
        };
        let Ok(entries) = std::fs::read_dir(parent) else {
            return 1;
        };
        let mut revisions = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?;
                parse_revision(prefix_name, name).map(|revision| (revision, entry.path()))
            })
            .collect::<Vec<_>>();
        revisions.sort_unstable_by_key(|entry| std::cmp::Reverse(entry.0));
        revisions
            .into_iter()
            .filter(|(revision, _)| *revision <= newest)
            .skip(RETAINED_REVISIONS)
            .filter(|(_, path)| std::fs::remove_file(path).is_err())
            .count()
    }
}

fn checksum(payload: &CheckpointPayload) -> Result<String, CheckpointError> {
    let bytes = serde_json::to_vec(payload).map_err(|source| CheckpointError::Encode { source })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn revision_path(prefix: &Path, revision: u64) -> PathBuf {
    let mut name = prefix.as_os_str().to_owned();
    name.push(format!(".{revision:020}.json"));
    PathBuf::from(name)
}

fn parse_revision(prefix: &str, name: &str) -> Option<u64> {
    let revision = name
        .strip_prefix(prefix)?
        .strip_prefix('.')?
        .strip_suffix(".json")?;
    if revision.len() != 20 || !revision.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    revision.parse().ok()
}

fn expand_state_dir(root: &Path) -> PathBuf {
    let text = root.to_string_lossy();
    if let Some(rest) = text.strip_prefix("${engine.state_dir}") {
        let base = std::env::var_os("OTAP_DF_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".otap-state"));
        return base.join(rest.trim_start_matches(['/', '\\']));
    }
    root.to_path_buf()
}

fn encode_path_segment(value: &str) -> String {
    if value.is_empty() {
        return "%".to_owned();
    }
    let encode_all = matches!(value, "." | "..");
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if !encode_all && is_safe_byte(byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(hex_digit(byte >> 4)));
            encoded.push(char::from(hex_digit(byte & 0x0f)));
        }
    }
    encoded
}

fn is_safe_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
}

fn hex_digit(value: u8) -> u8 {
    match value {
        0..=9 => b'0' + value,
        10..=15 => b'A' + (value - 10),
        _ => unreachable!("nibble must be in range"),
    }
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> Result<(), CheckpointError> {
    let directory =
        std::fs::File::open(parent).map_err(|source| CheckpointError::OpenDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    directory
        .sync_all()
        .map_err(|source| CheckpointError::FsyncDirectory {
            path: parent.to_path_buf(),
            source,
        })
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> Result<(), CheckpointError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn watermark(timestamp: &str, tie_breaker: i64) -> CompoundWatermark {
        CompoundWatermark {
            timestamp: timestamp.to_owned(),
            tie_breaker,
        }
    }

    fn store(root: &Path, fingerprint: &str) -> CheckpointStore {
        CheckpointStore::new(
            root,
            "group",
            "pipeline",
            "oracle",
            "orders",
            fingerprint.into(),
        )
    }

    /// Scenario: two acknowledged batches advance a revisioned checkpoint.
    /// Guarantees: the latest durable tuple and monotonic revision survive restart.
    #[test]
    fn writes_and_reads_latest_revision() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = store(dir.path(), "fingerprint");
        let (first, _) = store
            .write(0, &watermark("2026-01-01 00:00:00", 1))
            .unwrap();
        let (second, _) = store
            .write(first.revision, &watermark("2026-01-01 00:00:01", 2))
            .unwrap();

        assert_eq!(second.revision, 2);
        assert_eq!(store.read().unwrap(), Some(second));
    }

    /// Scenario: the newest checkpoint file is corrupted after an earlier valid revision.
    /// Guarantees: restart fails closed instead of silently falling back and duplicating data.
    #[test]
    fn corrupt_latest_revision_fails_closed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = store(dir.path(), "fingerprint");
        let (first, _) = store
            .write(0, &watermark("2026-01-01 00:00:00", 1))
            .unwrap();
        let (second, _) = store
            .write(first.revision, &watermark("2026-01-01 00:00:01", 2))
            .unwrap();
        std::fs::write(revision_path(&store.prefix, second.revision), b"invalid").unwrap();

        assert!(store.read().is_err());
    }

    /// Scenario: a checkpoint was created by a semantically different source query.
    /// Guarantees: configuration fingerprint mismatch fails closed before polling.
    #[test]
    fn fingerprint_mismatch_fails_closed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let original = store(dir.path(), "first");
        let _ = original
            .write(0, &watermark("2026-01-01 00:00:00", 1))
            .unwrap();

        assert!(store(dir.path(), "second").read().is_err());
    }

    /// Scenario: no checkpoint revision exists for a configured source.
    /// Guarantees: startup reports missing state so the explicit initial tuple is used.
    #[test]
    fn missing_checkpoint_is_distinct_from_invalid_state() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(store(dir.path(), "fingerprint").read().unwrap(), None);
    }

    /// Scenario: installing a revision succeeds but the first parent-directory sync reports failure.
    /// Guarantees: exact read-back plus a successful retry sync reconciles the intended commit.
    #[test]
    fn post_install_failure_reconciles_exact_checkpoint() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = store(dir.path(), "fingerprint");
        let candidate = watermark("2026-01-01 00:00:00", 42);
        store.inject_post_install_failures(1);

        let (committed, _) = store.write(0, &candidate).expect("reconciled commit");

        assert_eq!(committed.revision, 1);
        assert_eq!(committed.watermark, candidate);
        assert_eq!(store.read().expect("read"), Some(committed));
    }

    /// Scenario: the intended revision path already contains a different valid candidate.
    /// Guarantees: RevisionExists is retained and reconciliation never advances to mismatched state.
    #[test]
    fn existing_mismatched_revision_is_not_reconciled() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = store(dir.path(), "fingerprint");
        let installed = watermark("2026-01-01 00:00:00", 41);
        let intended = watermark("2026-01-01 00:00:00", 42);
        let (committed, _) = store.write(0, &installed).expect("initial commit");

        assert!(matches!(
            store.write(0, &intended),
            Err(CheckpointError::RevisionExists { .. })
        ));
        assert_eq!(store.read().expect("read"), Some(committed));
    }
}
