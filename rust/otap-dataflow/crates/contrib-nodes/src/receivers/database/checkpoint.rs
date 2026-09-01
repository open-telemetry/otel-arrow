// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Durable, revisioned filesystem checkpoints for database receivers.
//!
//! A checkpoint records the last cursor whose page was acknowledged
//! downstream. Reads and writes fail closed: corruption, an unsupported
//! version, a revision or source mismatch, or a configuration fingerprint
//! mismatch all abort startup rather than silently restarting from an
//! unrelated position.
//!
//! Every filesystem call in this module blocks. Callers must run it off the
//! local async engine core (see `DatabaseReceiver`).

use super::page::CompositeCursor;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex};

const ENVELOPE_VERSION: u8 = 1;
const MAX_CHECKPOINT_BYTES: u64 = 16 * 1024;
const RETAINED_REVISIONS: usize = 2;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CheckpointPayload {
    version: u8,
    revision: u64,
    source_id: String,
    config_fingerprint: String,
    cursor: CompositeCursor,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CheckpointEnvelope {
    payload: CheckpointPayload,
    checksum: String,
}

/// Durable checkpoint loaded from or installed on disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointState {
    /// Monotonic revision of the installed checkpoint file.
    pub revision: u64,
    /// Last durably acknowledged cursor.
    pub cursor: CompositeCursor,
}

/// Non-fatal outcome details of one durable checkpoint write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriteOutcome {
    /// Number of stale revision files that could not be removed.
    pub cleanup_failures: usize,
}

/// Stable checkpoint location plus the identity a checkpoint must match.
#[derive(Clone, Debug)]
pub struct CheckpointStore {
    prefix: PathBuf,
    source_id: String,
    config_fingerprint: String,
    // Test-only injection point for a post-install failure. `Arc` is required
    // because the store is cloned into a blocking worker for each write.
    #[cfg(test)]
    post_install_failures: Arc<AtomicUsize>,
}

/// Durable checkpoint read or write failure.
#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    /// The configured checkpoint prefix has no parent directory.
    #[error("checkpoint path has no parent: {path}")]
    NoParent {
        /// Configured checkpoint prefix.
        path: PathBuf,
    },
    /// The checkpoint directory could not be listed.
    #[error("failed to inspect checkpoint directory {path}")]
    Inspect {
        /// Checkpoint directory.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// A checkpoint file could not be inspected.
    #[error("failed to inspect checkpoint {path}")]
    Metadata {
        /// Checkpoint revision file.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// A checkpoint file exceeds the fixed bounded-read ceiling.
    #[error("checkpoint {path} exceeds {MAX_CHECKPOINT_BYTES} bytes")]
    TooLarge {
        /// Checkpoint revision file.
        path: PathBuf,
    },
    /// A checkpoint file could not be read.
    #[error("failed to read checkpoint {path}")]
    Read {
        /// Checkpoint revision file.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// A checkpoint file is not valid checkpoint JSON.
    #[error("failed to parse checkpoint {path}")]
    Parse {
        /// Checkpoint revision file.
        path: PathBuf,
        /// Underlying decoding error.
        #[source]
        source: serde_json::Error,
    },
    /// A checkpoint file uses an unknown schema version.
    #[error("unsupported checkpoint version {version} in {path}")]
    UnsupportedVersion {
        /// Checkpoint revision file.
        path: PathBuf,
        /// Version recorded in the file.
        version: u8,
    },
    /// A checkpoint file's content does not match its checksum.
    #[error("checkpoint checksum mismatch in {path}")]
    ChecksumMismatch {
        /// Checkpoint revision file.
        path: PathBuf,
    },
    /// A checkpoint file's recorded revision does not match its filename.
    #[error("checkpoint revision mismatch in {path}")]
    RevisionMismatch {
        /// Checkpoint revision file.
        path: PathBuf,
    },
    /// A checkpoint file belongs to a different configured source.
    #[error("checkpoint source identity mismatch in {path}")]
    SourceMismatch {
        /// Checkpoint revision file.
        path: PathBuf,
    },
    /// A checkpoint file belongs to a semantically different configuration.
    #[error("checkpoint configuration fingerprint mismatch in {path}")]
    FingerprintMismatch {
        /// Checkpoint revision file.
        path: PathBuf,
    },
    /// The revision counter cannot advance further.
    #[error("checkpoint revision overflow")]
    RevisionOverflow,
    /// The checkpoint envelope could not be encoded.
    #[error("failed to encode checkpoint")]
    Encode {
        /// Underlying encoding error.
        #[source]
        source: serde_json::Error,
    },
    /// The checkpoint directory could not be created.
    #[error("failed to create checkpoint directory {path}")]
    CreateDirectory {
        /// Checkpoint directory.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// The same-directory temporary file could not be created.
    #[error("failed to create checkpoint temporary file {path}")]
    CreateTemporary {
        /// Temporary file path.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// The temporary checkpoint file could not be written.
    #[error("failed to write checkpoint temporary file {path}")]
    Write {
        /// Temporary file path.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// The temporary checkpoint file could not be synced.
    #[error("failed to fsync checkpoint temporary file {path}")]
    FsyncFile {
        /// Temporary file path.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// The intended revision path already exists.
    #[error("checkpoint revision already exists at {path}")]
    RevisionExists {
        /// Intended revision file.
        path: PathBuf,
    },
    /// The atomic rename into place failed.
    #[error("failed to install checkpoint {tmp} into {path}")]
    Rename {
        /// Temporary file path.
        tmp: PathBuf,
        /// Intended revision file.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// The checkpoint directory could not be opened for syncing.
    #[cfg(unix)]
    #[error("failed to open checkpoint directory {path} for fsync")]
    OpenDirectory {
        /// Checkpoint directory.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// The checkpoint directory could not be synced.
    #[cfg(unix)]
    #[error("failed to fsync checkpoint directory {path}")]
    FsyncDirectory {
        /// Checkpoint directory.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// Test-only injected failure raised after the revision was installed.
    #[cfg(test)]
    #[error("injected failure after installing checkpoint {path}")]
    InjectedPostInstall {
        /// Installed revision file.
        path: PathBuf,
    },
}

impl CheckpointError {
    /// Returns whether the intended revision may already be on disk.
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
    /// Builds a store whose path encodes the full pipeline and source identity.
    #[must_use]
    pub fn new(
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
            source_id: source_id.to_owned(),
            config_fingerprint,
            #[cfg(test)]
            post_install_failures: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Returns the canonical identity used to lease this checkpoint source.
    #[must_use]
    pub fn lease_key(&self) -> String {
        self.prefix.to_string_lossy().into_owned()
    }

    /// Reads the newest installed revision, or `None` when no state exists.
    pub fn read(&self) -> Result<Option<CheckpointState>, CheckpointError> {
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
        let Some(prefix_name) = self.prefix.file_name().and_then(OsStr::to_str) else {
            return Err(CheckpointError::NoParent {
                path: self.prefix.clone(),
            });
        };
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
        // Bound the read before allocating so a corrupted or hostile file
        // cannot exhaust receiver memory during startup.
        if metadata.len() > MAX_CHECKPOINT_BYTES {
            return Err(CheckpointError::TooLarge {
                path: path.to_path_buf(),
            });
        }
        let bytes = std::fs::read(path).map_err(|source| CheckpointError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        if bytes.len() as u64 > MAX_CHECKPOINT_BYTES {
            return Err(CheckpointError::TooLarge {
                path: path.to_path_buf(),
            });
        }
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
        if envelope.payload.source_id != self.source_id {
            return Err(CheckpointError::SourceMismatch {
                path: path.to_path_buf(),
            });
        }
        if envelope.payload.config_fingerprint != self.config_fingerprint {
            return Err(CheckpointError::FingerprintMismatch {
                path: path.to_path_buf(),
            });
        }
        if envelope.checksum != checksum(&envelope.payload)? {
            return Err(CheckpointError::ChecksumMismatch {
                path: path.to_path_buf(),
            });
        }
        Ok(CheckpointState {
            revision: envelope.payload.revision,
            cursor: envelope.payload.cursor,
        })
    }

    /// Atomically installs the next revision for an acknowledged cursor.
    pub fn write(
        &self,
        current_revision: u64,
        cursor: &CompositeCursor,
    ) -> Result<(CheckpointState, WriteOutcome), CheckpointError> {
        let revision = current_revision
            .checked_add(1)
            .ok_or(CheckpointError::RevisionOverflow)?;
        let payload = CheckpointPayload {
            version: ENVELOPE_VERSION,
            revision,
            source_id: self.source_id.clone(),
            config_fingerprint: self.config_fingerprint.clone(),
            cursor: cursor.clone(),
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
        let install_result = self.install(&bytes, revision, &final_path, parent);
        if let Err(error) = install_result
            && (!error.could_have_installed()
                || !self.reconcile_installed(&final_path, revision, cursor, parent))
        {
            return Err(error);
        }
        let cleanup_failures = self.cleanup_old_revisions(revision);
        Ok((
            CheckpointState {
                revision,
                cursor: cursor.clone(),
            },
            WriteOutcome { cleanup_failures },
        ))
    }

    fn install(
        &self,
        bytes: &[u8],
        revision: u64,
        final_path: &Path,
        parent: &Path,
    ) -> Result<(), CheckpointError> {
        if final_path.exists() {
            return Err(CheckpointError::RevisionExists {
                path: final_path.to_path_buf(),
            });
        }
        // The temporary file lives in the same directory so the rename that
        // installs it is atomic on every supported filesystem.
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
        file.write_all(bytes)
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
        std::fs::rename(&tmp, final_path).map_err(|source| CheckpointError::Rename {
            tmp,
            path: final_path.to_path_buf(),
            source,
        })?;
        self.sync_parent(parent, final_path)
    }

    /// Confirms an uncertain install by reading back the exact intended state.
    fn reconcile_installed(
        &self,
        path: &Path,
        revision: u64,
        cursor: &CompositeCursor,
        parent: &Path,
    ) -> bool {
        let expected = CheckpointState {
            revision,
            cursor: cursor.clone(),
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

    /// Removes revisions older than the retained window, reporting failures.
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

const fn is_safe_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
}

const fn hex_digit(value: u8) -> u8 {
    match value {
        0..=9 => b'0' + value,
        10..=15 => b'A' + (value - 10),
        _ => unreachable!(),
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
    // Windows has no portable directory-fsync equivalent. The same-directory
    // temporary file plus atomic rename still guarantees a reader never sees a
    // partially written revision.
    Ok(())
}

// Process-local registry preventing two receivers in one process from
// advancing the same durable checkpoint. `Mutex` is required because factory
// construction happens before any pipeline core is assigned, so this registry
// is inherently cross-thread; it is touched only at construction and drop.
static SOURCE_LEASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Exclusive process-local ownership of one checkpoint source identity.
///
/// This complements, and does not replace, the deployment requirement that
/// exactly one collector replica owns a checkpoint source.
#[derive(Debug)]
pub struct SourceLease {
    key: String,
}

impl SourceLease {
    /// Acquires the lease for one canonical checkpoint source identity.
    pub fn acquire(key: &str) -> Result<Self, LeaseError> {
        let mut leases = SOURCE_LEASES.lock().map_err(|_| LeaseError::Unavailable)?;
        if !leases.insert(key.to_owned()) {
            return Err(LeaseError::AlreadyOwned);
        }
        Ok(Self {
            key: key.to_owned(),
        })
    }
}

impl Drop for SourceLease {
    fn drop(&mut self) {
        if let Ok(mut leases) = SOURCE_LEASES.lock() {
            _ = leases.remove(&self.key);
        }
    }
}

/// Failure while acquiring a process-local checkpoint source lease.
#[derive(Debug, thiserror::Error)]
pub enum LeaseError {
    /// Another receiver in this process already owns the source.
    #[error("another database receiver already owns this checkpoint source")]
    AlreadyOwned,
    /// The lease registry was poisoned by a panicking owner.
    #[error("database checkpoint source lease registry is unavailable")]
    Unavailable,
}

#[cfg(test)]
#[path = "checkpoint_tests.rs"]
mod tests;
