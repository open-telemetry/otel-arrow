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
//! local async engine core.

use crate::database::CompositeCursor;
use crate::partition::create_dir_all_durable;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

const ENVELOPE_VERSION: u8 = 1;
const MAX_CHECKPOINT_BYTES: u64 = 16 * 1024;
const MAX_READABLE_SOURCE_SEGMENT_BYTES: usize = 128;
const RETAINED_REVISIONS: usize = 2;
const TEMP_FILE_ATTEMPTS: usize = 16;
// Only allocates unique temporary filenames across blocking workers, not
// shared receiver data-path state.
static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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

/// Last acknowledged cursor and its durable checkpoint revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointState {
    /// Monotonic revision of the committed checkpoint.
    pub revision: u64,
    /// Last durably acknowledged cursor.
    pub cursor: CompositeCursor,
}

/// Non-fatal outcome details of one checkpoint write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriteOutcome {
    /// Number of stale revision files that could not be removed.
    pub cleanup_failures: usize,
}

// Test coordination crosses the local runtime/blocking-writer boundary.
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct WriteControl {
    pub(crate) delay: std::time::Duration,
    pub(crate) attempts: AtomicUsize,
    pub(crate) completed: AtomicUsize,
}

/// Stable checkpoint location plus the identity a checkpoint must match.
#[derive(Clone, Debug)]
pub struct CheckpointStore {
    prefix: PathBuf,
    legacy_prefix: Option<PathBuf>,
    source_id: String,
    config_fingerprint: String,
    // Clones move into successive blocking writers. Share the successful
    // directory-sync result so only initialization traverses the ancestors.
    directory_ready: Arc<AtomicBool>,
    // Test-only injection point for a post-install failure. `Arc` is required
    // because the store is cloned into a blocking worker for each write.
    #[cfg(test)]
    post_install_failures: Arc<AtomicUsize>,
    #[cfg(test)]
    pub(crate) write_control: Option<Arc<WriteControl>>,
}

/// Durable checkpoint read or write failure.
#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    /// Test-only failure before installing a revision.
    #[cfg(test)]
    #[error("injected checkpoint write failure")]
    InjectedWrite,
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
    /// A checkpoint-shaped filename contains an invalid revision.
    #[error("invalid checkpoint revision filename: {path}")]
    InvalidRevision {
        /// Invalid checkpoint filename.
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
    /// A stale or failed temporary checkpoint file could not be removed.
    #[error("failed to remove checkpoint temporary file {path}")]
    RemoveTemporary {
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
        let (source_name, legacy_source_name) = source_checkpoint_names(source_id);
        let legacy_prefix = legacy_source_name.map(|name| prefix.join(name));
        prefix.push(source_name);
        Self {
            prefix,
            legacy_prefix,
            source_id: source_id.to_owned(),
            config_fingerprint,
            directory_ready: Arc::new(AtomicBool::new(false)),
            #[cfg(test)]
            post_install_failures: Arc::new(AtomicUsize::new(0)),
            #[cfg(test)]
            write_control: None,
        }
    }

    /// Returns the canonical identity used to lease this checkpoint source.
    #[must_use]
    pub fn lease_key(&self) -> String {
        self.prefix.to_string_lossy().into_owned()
    }

    /// Reads the newest installed revision, or `None` when no state exists.
    pub fn read(&self) -> Result<Option<CheckpointState>, CheckpointError> {
        if let Some(checkpoint) = self.read_from_prefix(&self.prefix)? {
            return Ok(Some(checkpoint));
        }
        if let Some(legacy_prefix) = self.legacy_prefix.as_ref() {
            return self.read_from_prefix(legacy_prefix);
        }
        Ok(None)
    }

    fn read_from_prefix(&self, prefix: &Path) -> Result<Option<CheckpointState>, CheckpointError> {
        let Some(parent) = prefix.parent() else {
            return Err(CheckpointError::NoParent {
                path: prefix.to_path_buf(),
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
        let Some(prefix_name) = prefix.file_name().and_then(OsStr::to_str) else {
            return Err(CheckpointError::NoParent {
                path: prefix.to_path_buf(),
            });
        };
        let mut newest = None;
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
                if newest
                    .as_ref()
                    .is_none_or(|(current, _)| revision > *current)
                {
                    newest = Some((revision, entry.path()));
                }
            } else if revision_suffix(prefix_name, name).is_some() {
                return Err(CheckpointError::InvalidRevision { path: entry.path() });
            }
        }
        let Some((filename_revision, path)) = newest else {
            return Ok(None);
        };
        self.read_revision(&path, filename_revision).map(Some)
    }

    fn read_revision(
        &self,
        path: &Path,
        filename_revision: u64,
    ) -> Result<CheckpointState, CheckpointError> {
        let file = std::fs::File::open(path).map_err(|source| CheckpointError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        // Limit the open handle itself: metadata checks race file growth.
        let mut bytes = Vec::new();
        _ = file
            .take(MAX_CHECKPOINT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|source| CheckpointError::Read {
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
        #[cfg(test)]
        if let Some(control) = &self.write_control {
            _ = control.attempts.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(control.delay);
            _ = control.completed.fetch_add(1, Ordering::SeqCst);
            return Err(CheckpointError::InjectedWrite);
        }
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
        let final_path = revision_path(&self.prefix, revision);
        if bytes.len() as u64 > MAX_CHECKPOINT_BYTES {
            return Err(CheckpointError::TooLarge { path: final_path });
        }
        let parent = self
            .prefix
            .parent()
            .ok_or_else(|| CheckpointError::NoParent {
                path: self.prefix.clone(),
            })?;
        if !self.directory_ready.load(Ordering::Acquire) {
            create_dir_all_durable(parent).map_err(|source| CheckpointError::CreateDirectory {
                path: parent.to_path_buf(),
                source,
            })?;
            self.directory_ready.store(true, Ordering::Release);
        }
        let install_result = self.install(&bytes, &final_path, parent);
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
        final_path: &Path,
        parent: &Path,
    ) -> Result<(), CheckpointError> {
        self.cleanup_temporary_files(parent)?;
        if final_path.exists() {
            return Err(CheckpointError::RevisionExists {
                path: final_path.to_path_buf(),
            });
        }
        let (tmp, mut file) = self.create_temporary(parent)?;
        if let Err(source) = file.write_all(bytes) {
            drop(file);
            _ = std::fs::remove_file(&tmp);
            return Err(CheckpointError::Write { path: tmp, source });
        }
        if let Err(source) = file.sync_all() {
            drop(file);
            _ = std::fs::remove_file(&tmp);
            return Err(CheckpointError::FsyncFile { path: tmp, source });
        }
        drop(file);
        if let Err(source) = std::fs::rename(&tmp, final_path) {
            _ = std::fs::remove_file(&tmp);
            return Err(CheckpointError::Rename {
                tmp,
                path: final_path.to_path_buf(),
                source,
            });
        }
        self.sync_parent(parent, final_path)
    }

    fn create_temporary(&self, parent: &Path) -> Result<(PathBuf, std::fs::File), CheckpointError> {
        let process_id = std::process::id();
        let prefix = self.temporary_prefix();
        let mut last_error = None;
        for _ in 0..TEMP_FILE_ATTEMPTS {
            let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let tmp = parent.join(format!("{prefix}{process_id}.{sequence}.tmp"));
            match std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&tmp)
            {
                Ok(file) => return Ok((tmp, file)),
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                    last_error = Some((tmp, source));
                }
                Err(source) => {
                    return Err(CheckpointError::CreateTemporary { path: tmp, source });
                }
            }
        }
        let (path, source) = last_error.expect("temporary file attempts must be nonzero");
        Err(CheckpointError::CreateTemporary { path, source })
    }

    fn temporary_prefix(&self) -> String {
        let identity = self.prefix.to_string_lossy();
        let digest = blake3::hash(identity.as_bytes()).to_hex();
        format!(".otel-arrow-checkpoint-{digest}.")
    }

    fn cleanup_temporary_files(&self, parent: &Path) -> Result<(), CheckpointError> {
        let prefix = self.temporary_prefix();
        for entry in
            std::fs::read_dir(parent).map_err(|source| CheckpointError::CreateTemporary {
                path: parent.to_path_buf(),
                source,
            })?
        {
            let entry = entry.map_err(|source| CheckpointError::CreateTemporary {
                path: parent.to_path_buf(),
                source,
            })?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if name.starts_with(&prefix) && name.ends_with(".tmp") {
                let path = entry.path();
                std::fs::remove_file(&path)
                    .map_err(|source| CheckpointError::RemoveTemporary { path, source })?;
            }
        }
        Ok(())
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

fn source_checkpoint_names(source_id: &str) -> (String, Option<String>) {
    let encoded = encode_path_segment(source_id);
    let legacy = format!("{encoded}.checkpoint");
    if encoded.len() <= MAX_READABLE_SOURCE_SEGMENT_BYTES {
        return (legacy, None);
    }
    let digest = blake3::hash(source_id.as_bytes()).to_hex();
    (format!("source-{digest}.checkpoint"), Some(legacy))
}

fn revision_suffix<'a>(prefix: &str, name: &'a str) -> Option<&'a str> {
    // Source names may themselves contain ".checkpoint.". The final marker
    // separates the complete source identity from its revision.
    let (source, revision) = name.strip_suffix(".json")?.rsplit_once(".checkpoint.")?;
    (source == prefix.strip_suffix(".checkpoint")?).then_some(revision)
}

fn parse_revision(prefix: &str, name: &str) -> Option<u64> {
    let revision = revision_suffix(prefix, name)?;
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

#[cfg(test)]
#[path = "checkpoint_tests.rs"]
mod tests;
