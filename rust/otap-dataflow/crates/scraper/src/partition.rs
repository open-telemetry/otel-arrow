// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exclusive ownership of checkpoint storage identities, not database queries.

use fs2::FileExt;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

const RETAINED_LEASE_GENERATIONS: usize = 2;

pub(crate) fn create_dir_all_durable(path: &Path) -> io::Result<()> {
    // mkdir only, matching journald. This does not fsync directory entries.
    // Durability of an installed file comes from file fsync plus fsync of
    // that file's parent after rename, not from this helper.
    std::fs::create_dir_all(path)
}

// Factory construction happens before pipeline cores are assigned, so this
// process-wide registry is touched only while constructing or dropping a
// receiver, never on the local async data path.
static SOURCE_LEASES: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Exclusive ownership of one checkpoint source identity.
///
/// The process-local registry covers platforms where advisory file locks are
/// process-scoped. The held file lock provides cross-process exclusion on
/// filesystems that honor the operating system's advisory locking semantics.
/// The generation advances on every successful acquisition in this lock
/// namespace. Persistence has
/// the platform-specific limits described by [`crate::CheckpointStore`].
///
/// The key identifies checkpoint storage. Separate pipeline/receiver names or
/// state directories may produce distinct keys for the same logical database
/// source. This guard does not detect equivalent or overlapping queries.
///
/// Dropping the guard unlocks and closes the lock file. The lock fd is opened
/// close-on-exec so a later `Command::spawn` cannot keep the lease alive.
/// The caller must retain the guard until source and checkpoint operations
/// have stopped; the guard cannot cancel or join outstanding workers.
#[derive(Debug)]
pub struct SourceLease {
    key: PathBuf,
    file: Option<File>,
    generation: u64,
}

impl SourceLease {
    /// Acquires the lease for one checkpoint storage identity.
    ///
    /// The path is canonicalized for lock identity; database/query equivalence
    /// is not evaluated.
    pub fn acquire(key: &Path) -> Result<Self, LeaseError> {
        let paths = LeasePaths::new(key)?;
        let key = paths.registry_key.clone();
        {
            let mut leases = SOURCE_LEASES.lock().map_err(|_| LeaseError::Unavailable)?;
            if !leases.insert(key.clone()) {
                return Err(LeaseError::AlreadyOwned);
            }
        }

        match acquire_file_lease(&paths) {
            Ok((file, generation)) => Ok(Self {
                key,
                file: Some(file),
                generation,
            }),
            Err(error) => {
                if let Ok(mut leases) = SOURCE_LEASES.lock() {
                    _ = leases.remove(&key);
                }
                Err(error)
            }
        }
    }

    /// Returns the durable ownership generation assigned to this acquisition.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
}

impl Drop for SourceLease {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            _ = FileExt::unlock(&file);
        }
        if let Ok(mut leases) = SOURCE_LEASES.lock() {
            _ = leases.remove(&self.key);
        }
    }
}

struct LeasePaths {
    registry_key: PathBuf,
    parent: PathBuf,
    lock: PathBuf,
    generation_prefix: String,
}

impl LeasePaths {
    fn new(source: &Path) -> Result<Self, LeaseError> {
        let parent = source.parent().ok_or_else(|| LeaseError::InvalidPath {
            path: source.to_path_buf(),
        })?;
        create_dir_all_durable(parent)
            .map_err(|source| LeaseError::io("create parent directory for", parent, source))?;
        let parent = std::fs::canonicalize(parent).map_err(|source| {
            LeaseError::io("canonicalize parent directory for", parent, source)
        })?;
        let file_name = source.file_name().ok_or_else(|| LeaseError::InvalidPath {
            path: source.to_path_buf(),
        })?;
        // The directory already namespaces the on-disk lock. Hash only the
        // filename so different mounts of the same directory lock the same file.
        let digest = checkpoint_name_digest(file_name).to_hex();
        Ok(Self {
            registry_key: parent.join(file_name),
            lock: parent.join(format!(".otel-arrow-source-{digest}.lock")),
            generation_prefix: format!(".otel-arrow-source-{digest}.generation."),
            parent,
        })
    }
}

pub(crate) fn checkpoint_name_digest(name: &OsStr) -> blake3::Hash {
    #[cfg(windows)]
    {
        // Preserve the existing case-folded lock namespace for Unicode names.
        if let Some(name) = name.to_str() {
            return blake3::hash(name.to_lowercase().as_bytes());
        }
        blake3::hash(&name.as_encoded_bytes().to_ascii_lowercase())
    }
    #[cfg(not(windows))]
    {
        blake3::hash(name.as_encoded_bytes())
    }
}

fn acquire_file_lease(paths: &LeasePaths) -> Result<(File, u64), LeaseError> {
    let mut options = OpenOptions::new();
    // Child processes must not inherit the flock. Parallel tests spawn
    // subprocesses from this binary; an inherited lock fd stays held after
    // Drop in the parent and makes reacquisition fail with AlreadyOwned.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        _ = options
            .create(true)
            .read(true)
            .truncate(false)
            .write(true)
            .custom_flags(libc::O_CLOEXEC);
    }
    #[cfg(not(unix))]
    {
        _ = options.create(true).read(true).truncate(false).write(true);
    }
    let file = options
        .open(&paths.lock)
        .map_err(|source| LeaseError::io("open", &paths.lock, source))?;
    match FileExt::try_lock_exclusive(&file) {
        Ok(()) => {}
        Err(source)
            if source.kind() == io::ErrorKind::WouldBlock
                || source.raw_os_error() == fs2::lock_contended_error().raw_os_error() =>
        {
            return Err(LeaseError::AlreadyOwned);
        }
        Err(source) => return Err(LeaseError::io("lock", &paths.lock, source)),
    }

    let generation = install_next_generation(paths)?;
    Ok((file, generation))
}

fn install_next_generation(paths: &LeasePaths) -> Result<u64, LeaseError> {
    let mut generations = read_generations(paths)?;
    if generations.len() >= RETAINED_LEASE_GENERATIONS {
        let remove_count = generations.len() - (RETAINED_LEASE_GENERATIONS - 1);
        for generation in generations.drain(..remove_count) {
            let path = generation_path(paths, generation);
            std::fs::remove_file(&path)
                .map_err(|source| LeaseError::io("remove stale generation", &path, source))?;
        }
        sync_parent_directory(&paths.parent, &paths.lock)?;
    }

    let generation = generations
        .last()
        .copied()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| LeaseError::GenerationOverflow {
            path: paths.lock.clone(),
        })?;
    let path = generation_path(paths, generation);
    let mut marker = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|source| LeaseError::io("create ownership generation", &path, source))?;
    if let Err(source) = writeln!(marker, "{generation}") {
        _ = std::fs::remove_file(&path);
        return Err(LeaseError::io("write ownership generation", &path, source));
    }
    if let Err(source) = marker.sync_all() {
        _ = std::fs::remove_file(&path);
        return Err(LeaseError::io("sync ownership generation", &path, source));
    }
    sync_parent_directory(&paths.parent, &path)?;
    Ok(generation)
}

fn read_generations(paths: &LeasePaths) -> Result<Vec<u64>, LeaseError> {
    let mut generations = Vec::new();
    for entry in std::fs::read_dir(&paths.parent)
        .map_err(|source| LeaseError::io("list ownership generations for", &paths.lock, source))?
    {
        let entry = entry.map_err(|source| {
            LeaseError::io("read ownership generation for", &paths.lock, source)
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(suffix) = name.strip_prefix(&paths.generation_prefix) else {
            continue;
        };
        if suffix.len() != 20 || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(LeaseError::InvalidGeneration { path: entry.path() });
        }
        generations.push(
            suffix
                .parse()
                .map_err(|_| LeaseError::InvalidGeneration { path: entry.path() })?,
        );
    }
    generations.sort_unstable();
    Ok(generations)
}

fn generation_path(paths: &LeasePaths, generation: u64) -> PathBuf {
    paths
        .parent
        .join(format!("{}{generation:020}", paths.generation_prefix))
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path, lease_path: &Path) -> Result<(), LeaseError> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| LeaseError::io("sync parent directory for", lease_path, source))
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path, _lease_path: &Path) -> Result<(), LeaseError> {
    Ok(())
}

/// Failure while acquiring a checkpoint source lease.
#[derive(Debug, thiserror::Error)]
pub enum LeaseError {
    /// Another receiver already owns the source.
    #[error("another database receiver already owns this checkpoint source")]
    AlreadyOwned,
    /// The lease registry was poisoned by a panicking owner.
    #[error("database checkpoint source lease registry is unavailable")]
    Unavailable,
    /// The derived lease path has no parent directory.
    #[error("database checkpoint source lease path has no parent: {path}")]
    InvalidPath {
        /// Invalid lease path.
        path: PathBuf,
    },
    /// The durable ownership generation is malformed.
    #[error("database checkpoint source lease has an invalid ownership generation: {path}")]
    InvalidGeneration {
        /// Lease file containing the malformed generation.
        path: PathBuf,
    },
    /// The durable ownership generation cannot be incremented.
    #[error("database checkpoint source lease ownership generation overflowed: {path}")]
    GenerationOverflow {
        /// Lease file whose generation reached `u64::MAX`.
        path: PathBuf,
    },
    /// A filesystem operation failed.
    #[error("failed to {operation} database checkpoint source lease {path}")]
    Io {
        /// Operation that failed.
        operation: &'static str,
        /// Lease file involved in the failure.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
}

impl LeaseError {
    fn io(operation: &'static str, path: &Path, source: io::Error) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEASE_CHILD_KEY: &str = "OTEL_ARROW_SCRAPER_LEASE_CHILD_KEY";

    /// Scenario: The same checkpoint basename is accessed under different mount-path spellings.
    /// Guarantees: On-disk lock and generation names are independent of the absolute parent path.
    #[test]
    fn lock_namespace_is_independent_of_mount_path() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let first = LeasePaths::new(&directory.path().join("first").join("orders.checkpoint"))
            .expect("first paths");
        let second = LeasePaths::new(&directory.path().join("second").join("orders.checkpoint"))
            .expect("second paths");
        assert_ne!(first.registry_key, second.registry_key);
        assert_eq!(first.lock.file_name(), second.lock.file_name());
        assert_eq!(first.generation_prefix, second.generation_prefix);
    }

    /// Scenario: Nested checkpoint directories are created under an existing state root.
    /// Guarantees: mkdir succeeds without fsyncing ancestors up to the filesystem root.
    #[test]
    fn directory_creation_does_not_fsync_ancestors() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("new").join("nested").join("state");
        create_dir_all_durable(&target).expect("create nested state directory");
        assert!(target.is_dir());
    }

    /// Scenario: A filesystem entry blocks the requested directory hierarchy.
    /// Guarantees: Creation fails before lease installation can succeed.
    #[test]
    fn directory_creation_failure_is_not_hidden() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let file = directory.path().join("file");
        std::fs::write(&file, b"not a directory").expect("blocking file");
        assert!(create_dir_all_durable(&file.join("state")).is_err());
    }

    /// Scenario: receiver processes contend for one source lease.
    /// Guarantees: the operating-system lock rejects the second process, durable generations
    /// advance after release, and only the bounded recovery window remains on disk.
    #[test]
    fn file_lease_excludes_another_process() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let key = directory.path().join("source");
        let first = SourceLease::acquire(&key).expect("first lease");

        assert_eq!(first.generation(), 1);
        let output =
            std::process::Command::new(std::env::current_exe().expect("current test executable"))
                .args([
                    "--exact",
                    "partition::tests::file_lease_child_process",
                    "--nocapture",
                ])
                .env(LEASE_CHILD_KEY, key.as_os_str())
                .output()
                .expect("run lease contender");
        assert!(
            output.status.success(),
            "lease contender failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        drop(first);
        let second = SourceLease::acquire(&key).expect("second lease");
        assert_eq!(second.generation(), 2);
        drop(second);
        let third = SourceLease::acquire(&key).expect("third lease");
        assert_eq!(third.generation(), 3);

        let paths = LeasePaths::new(&key).expect("lease paths");
        let retained = std::fs::read_dir(&paths.parent)
            .expect("list lease directory")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(&paths.generation_prefix))
            })
            .count();
        assert_eq!(retained, RETAINED_LEASE_GENERATIONS);
    }

    /// Scenario: a child process attempts to acquire the lease held by its parent test.
    /// Guarantees: cross-process file locking reports the source as already owned.
    #[test]
    fn file_lease_child_process() {
        let Some(key) = std::env::var_os(LEASE_CHILD_KEY) else {
            return;
        };
        assert!(matches!(
            SourceLease::acquire(Path::new(&key)),
            Err(LeaseError::AlreadyOwned)
        ));
    }

    /// Scenario: a process crashes after creating an ownership marker but before writing it.
    /// Guarantees: the marker's generation remains consumed and the next owner advances beyond it.
    #[test]
    fn incomplete_generation_marker_is_consumed() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let key = directory.path().join("source");
        let paths = LeasePaths::new(&key).expect("lease paths");
        std::fs::write(generation_path(&paths, 7), b"").expect("write incomplete marker");

        let (_lease, generation) = acquire_file_lease(&paths).expect("acquire after crash");

        assert_eq!(generation, 8);
    }

    /// Scenario: a source's ownership-generation filename is malformed.
    /// Guarantees: acquisition fails closed instead of guessing fencing state.
    #[test]
    fn malformed_generation_fails_closed() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let key = directory.path().join("source");
        let paths = LeasePaths::new(&key).expect("lease paths");
        let malformed = paths
            .parent
            .join(format!("{}invalid", paths.generation_prefix));
        std::fs::write(&malformed, b"").expect("write malformed marker");

        assert!(matches!(
            acquire_file_lease(&paths),
            Err(LeaseError::InvalidGeneration { path }) if path == malformed
        ));
    }
}
