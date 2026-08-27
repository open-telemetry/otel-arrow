// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Async file lifecycle, tail recovery, and transactional frame writes.
//!
//! Each signal writer owns one file and a process-local path lease. Opening and rollback use
//! bounded blocking helpers where needed, while frame writes stay ordered on the local runtime.

use super::config::{Durability, FileExporterConfig, OpenMode, TailRecovery};
use super::metrics::FileOperation;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

static PATH_LEASES: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();

/// Tail bytes removed while opening an append-mode writer.
#[derive(Debug, Clone, Copy, Default)]
pub struct TailRecoveryResult {
    /// Number of bytes truncated from the incomplete final frame.
    pub recovered_bytes: u64,
}

/// Failure of one writer operation, optionally with a failed rollback.
#[derive(Debug)]
pub struct WriterFailure {
    /// Operation that first failed.
    pub operation: FileOperation,
    /// Human-readable error without a destination path.
    pub error: String,
    /// Rollback error that makes the file state indeterminate.
    pub rollback_error: Option<String>,
}

impl WriterFailure {
    fn new(operation: FileOperation, error: impl ToString) -> Self {
        Self {
            operation,
            error: error.to_string(),
            rollback_error: None,
        }
    }

    fn with_rollback(mut self, error: impl ToString) -> Self {
        self.rollback_error = Some(error.to_string());
        self
    }
}

/// Exclusively owned async writer for one resolved signal path.
pub struct SignalWriter {
    file: File,
    durability: Durability,
    _lease: PathLease,
}

impl SignalWriter {
    /// Opens and repairs a signal writer before returning it as ready.
    pub async fn open(
        path: &Path,
        config: &FileExporterConfig,
    ) -> Result<(Self, TailRecoveryResult), WriterFailure> {
        let lease = PathLease::acquire(path)
            .await
            .map_err(|error| WriterFailure::new(FileOperation::Open, error))?;
        if config.create_directories {
            create_parent_directories(path)
                .await
                .map_err(|error| WriterFailure::new(FileOperation::Open, error))?;
        }
        let mut options = OpenOptions::new();
        _ = options.read(true);
        match config.open_mode {
            OpenMode::Append => {
                #[cfg(windows)]
                {
                    // Windows append-only handles cannot call set_len(). Each frame write seeks
                    // to EOF first, so write access preserves exporter append behavior while
                    // allowing tail recovery and transactional rollback.
                    _ = options.write(true).create(true);
                }
                #[cfg(not(windows))]
                {
                    _ = options.append(true).create(true);
                }
            }
            OpenMode::Truncate => {
                _ = options.write(true).create(true).truncate(true);
            }
            OpenMode::CreateNew => {
                #[cfg(windows)]
                {
                    // Use the same writable Windows handle as append mode so rollback can
                    // truncate a partially written first frame.
                    _ = options.write(true).create_new(true);
                }
                #[cfg(not(windows))]
                {
                    _ = options.append(true).create_new(true);
                }
            }
        }
        #[cfg(unix)]
        {
            _ = options.mode(0o600);
        }
        let file = options
            .open(path)
            .await
            .map_err(|error| WriterFailure::new(FileOperation::Open, error))?;
        let mut writer = Self {
            file,
            durability: config.durability,
            _lease: lease,
        };
        let recovery = if let Some(policy) = config.effective_tail_recovery() {
            writer
                .recover_tail(policy, config.max_frame_bytes)
                .await
                .map_err(|error| WriterFailure::new(FileOperation::Open, error))?
        } else {
            TailRecoveryResult::default()
        };
        Ok((writer, recovery))
    }

    /// Writes one complete frame and rolls back to the prior length on failure.
    pub async fn write_frame(&mut self, frame: &[u8]) -> Result<(), WriterFailure> {
        let start_len = self
            .file
            .metadata()
            .await
            .map_err(|error| WriterFailure::new(FileOperation::Write, error))?
            .len();
        _ = self
            .file
            .seek(io::SeekFrom::End(0))
            .await
            .map_err(|error| WriterFailure::new(FileOperation::Write, error))?;
        if let Err(error) = async {
            self.file.write_all(frame).await?;
            self.file.flush().await
        }
        .await
        {
            let failure = WriterFailure::new(FileOperation::Write, error);
            return match self.rollback(start_len).await {
                Ok(()) => Err(failure),
                Err(rollback_error) => Err(failure.with_rollback(rollback_error)),
            };
        }
        if self.durability == Durability::SyncData
            && let Err(error) = self.file.sync_data().await
        {
            let failure = WriterFailure::new(FileOperation::Sync, error);
            return match self.rollback(start_len).await {
                Ok(()) => Err(failure),
                Err(rollback_error) => Err(failure.with_rollback(rollback_error)),
            };
        }
        Ok(())
    }

    /// Flushes and synchronizes a ready writer during graceful shutdown.
    pub async fn finalize(&mut self) -> Result<(), WriterFailure> {
        let flush_result = self.file.flush().await;
        let sync_result = self.file.sync_data().await;
        match (flush_result, sync_result) {
            (Err(error), _) => Err(WriterFailure::new(FileOperation::Write, error)),
            (Ok(()), Err(error)) => Err(WriterFailure::new(FileOperation::Sync, error)),
            (Ok(()), Ok(())) => Ok(()),
        }
    }

    async fn rollback(&mut self, length: u64) -> io::Result<()> {
        self.file.set_len(length).await?;
        _ = self.file.seek(io::SeekFrom::End(0)).await?;
        self.file.sync_data().await
    }

    async fn recover_tail(
        &mut self,
        policy: TailRecovery,
        max_frame_bytes: usize,
    ) -> io::Result<TailRecoveryResult> {
        let len = self.file.metadata().await?.len();
        if len == 0 {
            return Ok(TailRecoveryResult::default());
        }
        _ = self.file.seek(io::SeekFrom::End(-1)).await?;
        let mut final_byte = [0_u8; 1];
        _ = self.file.read_exact(&mut final_byte).await?;
        if final_byte[0] == b'\n' {
            return Ok(TailRecoveryResult::default());
        }
        if policy == TailRecovery::Fail {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "append target ends with an incomplete frame",
            ));
        }
        let scan_len = len.min(max_frame_bytes as u64);
        let scan_start = len - scan_len;
        _ = self.file.seek(io::SeekFrom::Start(scan_start)).await?;
        let mut tail = vec![0_u8; scan_len as usize];
        _ = self.file.read_exact(&mut tail).await?;
        let Some(boundary) = tail.iter().rposition(|byte| *byte == b'\n') else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "no complete frame boundary found within max_frame_bytes",
            ));
        };
        let recovered_len = scan_start + boundary as u64 + 1;
        self.file.set_len(recovered_len).await?;
        self.file.sync_data().await?;
        Ok(TailRecoveryResult {
            recovered_bytes: len - recovered_len,
        })
    }
}

struct PathLease {
    leased_path: PathBuf,
}

impl PathLease {
    async fn acquire(path: &Path) -> io::Result<Self> {
        let path = path.to_owned();
        let leased_path = tokio::task::spawn_blocking(move || resolve_for_lease(&path))
            .await
            .map_err(|error| io::Error::other(format!("path resolution task failed: {error}")))??;
        let leases = PATH_LEASES.get_or_init(|| Mutex::new(HashSet::new()));
        let mut leases = leases
            .lock()
            .map_err(|_| io::Error::other("file path lease registry is poisoned"))?;
        if !leases.insert(leased_path.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "resolved file path is already leased by another writer",
            ));
        }
        Ok(Self { leased_path })
    }
}

fn resolve_for_lease(path: &Path) -> io::Result<PathBuf> {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return Ok(canonical);
    }
    let mut ancestor = path;
    let mut suffix = Vec::new();
    let canonical_ancestor = loop {
        if let Ok(canonical) = std::fs::canonicalize(ancestor) {
            break canonical;
        }
        let Some(name) = ancestor.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "file path has no canonicalizable ancestor",
            ));
        };
        suffix.push(name.to_owned());
        ancestor = ancestor.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "file path has no canonicalizable ancestor",
            )
        })?;
    };
    Ok(suffix
        .into_iter()
        .rev()
        .fold(canonical_ancestor, |path, component| path.join(component)))
}

impl Drop for PathLease {
    fn drop(&mut self) {
        if let Some(leases) = PATH_LEASES.get()
            && let Ok(mut leases) = leases.lock()
        {
            let _ = leases.remove(&self.leased_path);
        }
    }
}

async fn create_parent_directories(path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "file destination has no parent directory",
        ));
    };
    let parent = parent.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut builder = std::fs::DirBuilder::new();
        _ = builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            _ = builder.mode(0o700);
        }
        builder.create(parent)
    })
    .await
    .map_err(|error| io::Error::other(format!("directory creation task failed: {error}")))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn config(path: &Path, extra: serde_json::Value) -> FileExporterConfig {
        let mut value = json!({"path": path.to_string_lossy()});
        value.as_object_mut().unwrap().extend(
            extra
                .as_object()
                .expect("test config extras must be an object")
                .clone(),
        );
        FileExporterConfig::parse(&value).unwrap()
    }

    fn template_path(root: &Path) -> PathBuf {
        root.join("data-{signal}-{core_id}-{generation}.jsonl")
    }

    /// Scenario: Append mode opens a file ending in a complete JSON line and writes another frame.
    /// Guarantees: Existing complete frames are retained before the first exporter frame is written.
    #[tokio::test]
    async fn append_preserves_existing_frames() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data-logs-0-1.jsonl");
        tokio::fs::write(&path, b"{\"old\":true}\n").await.unwrap();
        let config = config(&template_path(dir.path()), json!({}));
        let (mut writer, recovery) = SignalWriter::open(&path, &config).await.unwrap();
        assert_eq!(recovery.recovered_bytes, 0);
        assert_eq!(tokio::fs::read(&path).await.unwrap(), b"{\"old\":true}\n");
        writer.write_frame(b"{\"new\":true}\n").await.unwrap();
        assert_eq!(
            tokio::fs::read(&path).await.unwrap(),
            b"{\"old\":true}\n{\"new\":true}\n"
        );
    }

    /// Scenario: Append mode opens a file with one complete frame and an incomplete crash tail.
    /// Guarantees: Only the incomplete final bytes are removed before the next frame is written.
    #[tokio::test]
    async fn append_truncates_one_bounded_partial_tail() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data-logs-0-1.jsonl");
        tokio::fs::write(&path, b"{\"ok\":true}\n{\"partial\"")
            .await
            .unwrap();
        let config = config(&template_path(dir.path()), json!({}));
        let (_writer, recovery) = SignalWriter::open(&path, &config).await.unwrap();
        assert_eq!(recovery.recovered_bytes, 10);
        assert_eq!(tokio::fs::read(&path).await.unwrap(), b"{\"ok\":true}\n");
    }

    /// Scenario: Append mode is configured to reject a destination with a partial final frame.
    /// Guarantees: Tail recovery failure leaves every existing byte unchanged.
    #[tokio::test]
    async fn append_fail_policy_rejects_and_preserves_partial_tail() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data-logs-0-1.jsonl");
        let original = b"{\"ok\":true}\n{\"partial\"";
        tokio::fs::write(&path, original).await.unwrap();
        let config = config(&template_path(dir.path()), json!({"tail_recovery": "fail"}));
        assert!(SignalWriter::open(&path, &config).await.is_err());
        assert_eq!(tokio::fs::read(&path).await.unwrap(), original);
    }

    /// Scenario: The nearest complete boundary lies outside the configured tail scan bound.
    /// Guarantees: Recovery refuses to guess a truncation point and preserves the destination.
    #[tokio::test]
    async fn append_recovery_rejects_a_tail_larger_than_the_frame_bound() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data-logs-0-1.jsonl");
        let original = b"{}\npartial";
        tokio::fs::write(&path, original).await.unwrap();
        let config = config(&template_path(dir.path()), json!({"max_frame_bytes": 4}));
        assert!(SignalWriter::open(&path, &config).await.is_err());
        assert_eq!(tokio::fs::read(&path).await.unwrap(), original);
    }

    /// Scenario: Truncate and create-new modes open existing and absent destinations.
    /// Guarantees: Truncate discards old bytes, while create-new rejects a second ownership file.
    #[tokio::test]
    async fn destructive_open_modes_have_explicit_first_open_behavior() {
        let dir = tempdir().unwrap();
        let truncate_path = dir.path().join("truncate-logs-0-1.jsonl");
        tokio::fs::write(&truncate_path, b"old\n").await.unwrap();
        let truncate_config = config(
            &dir.path()
                .join("truncate-{signal}-{core_id}-{generation}.jsonl"),
            json!({"open_mode": "truncate"}),
        );
        let (mut truncate_writer, _) = SignalWriter::open(&truncate_path, &truncate_config)
            .await
            .unwrap();
        assert!(tokio::fs::read(&truncate_path).await.unwrap().is_empty());
        truncate_writer.write_frame(b"{}\n").await.unwrap();
        assert_eq!(tokio::fs::read(&truncate_path).await.unwrap(), b"{}\n");
        drop(truncate_writer);

        let create_path = dir.path().join("new-logs-0-1.jsonl");
        let create_config = config(
            &dir.path().join("new-{signal}-{core_id}-{generation}.jsonl"),
            json!({"open_mode": "create_new"}),
        );
        let (create_writer, _) = SignalWriter::open(&create_path, &create_config)
            .await
            .unwrap();
        drop(create_writer);
        assert!(
            SignalWriter::open(&create_path, &create_config)
                .await
                .is_err()
        );
    }

    /// Scenario: Two live writer instances attempt to own the same normalized signal path.
    /// Guarantees: The second open fails until the first writer releases its process-local lease.
    #[tokio::test]
    async fn live_writers_cannot_share_a_resolved_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data-logs-0-1.jsonl");
        let config = config(&template_path(dir.path()), json!({}));
        let (writer, _) = SignalWriter::open(&path, &config).await.unwrap();
        assert!(SignalWriter::open(&path, &config).await.is_err());
        drop(writer);
        assert!(SignalWriter::open(&path, &config).await.is_ok());
    }

    #[cfg(unix)]
    /// Scenario: Two configured paths reach one file through real and symlinked parent directories.
    /// Guarantees: Canonical process-local leasing rejects filesystem aliases while either writer lives.
    #[tokio::test]
    async fn live_writers_cannot_bypass_leases_with_a_symlinked_parent() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target");
        let alias = dir.path().join("alias");
        std::fs::create_dir(&target).unwrap();
        std::os::unix::fs::symlink(&target, &alias).unwrap();
        let target_path = target.join("data-logs-0-1.jsonl");
        let alias_path = alias.join("data-logs-0-1.jsonl");
        let config = config(&template_path(&target), json!({}));
        let (writer, _) = SignalWriter::open(&target_path, &config).await.unwrap();
        assert!(SignalWriter::open(&alias_path, &config).await.is_err());
        drop(writer);
        assert!(SignalWriter::open(&alias_path, &config).await.is_ok());
    }

    #[cfg(unix)]
    /// Scenario: A symlink followed by `..` aliases the destination owned by another live writer.
    /// Guarantees: Lease resolution follows filesystem traversal order and rejects the alias.
    #[tokio::test]
    async fn live_writers_cannot_bypass_leases_with_symlinked_parent_traversal() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target");
        let child = target.join("child");
        let alias = dir.path().join("alias");
        std::fs::create_dir_all(&child).unwrap();
        std::os::unix::fs::symlink(&child, &alias).unwrap();
        let target_path = target.join("data-logs-0-1.jsonl");
        let alias_path = alias.join("../data-logs-0-1.jsonl");
        let config = config(&template_path(&target), json!({}));
        let (writer, _) = SignalWriter::open(&target_path, &config).await.unwrap();
        assert!(SignalWriter::open(&alias_path, &config).await.is_err());
        drop(writer);
        assert!(SignalWriter::open(&alias_path, &config).await.is_ok());
    }

    /// Scenario: Directory creation is enabled for a destination with missing nested parents.
    /// Guarantees: The writer creates parents lazily and writes a frame to the requested file.
    #[tokio::test]
    async fn creates_missing_parent_directories_when_enabled() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("one/two");
        let path = root.join("data-logs-0-1.jsonl");
        let config = config(&template_path(&root), json!({"create_directories": true}));
        let (mut writer, _) = SignalWriter::open(&path, &config).await.unwrap();
        writer.write_frame(b"{}\n").await.unwrap();
        assert_eq!(tokio::fs::read(path).await.unwrap(), b"{}\n");
    }
}
