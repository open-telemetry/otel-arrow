// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Descriptor-relative Linux source-file access.
//!
//! Pins candidates with O_PATH, validates them, then reopens through trusted
//! `/proc/self/fd` without a pathname fallback. The filesystem deny-list is not
//! an allowlist; following links can leave the supplied directory.
//!
//! Discovery owns directory validation, traversal, target policy and admission.
//! Callers reserve two shared transient descriptor slots; success retains one
//! read-only handle. A successful open alone does not establish safe admission.
//!
//! Run I/O on blocking workers. Cancellation is checked between operations and
//! cannot interrupt blocked filesystem calls. Errors are returned without retry.
//! See `docs/filelog-receiver-phase1-spec.md` for the full source contract.

use std::ffi::CStr;
use std::fs::{File, Metadata};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileExt, MetadataExt};

use nix::fcntl::{OFlag, open, openat};
use nix::sys::stat::Mode;
use nix::sys::statfs::{
    CGROUP_SUPER_MAGIC, CGROUP2_SUPER_MAGIC, DEBUGFS_MAGIC, FsType, PROC_SUPER_MAGIC,
    SECURITYFS_MAGIC, SYSFS_MAGIC, TRACEFS_MAGIC, fstatfs,
};

/// Policy for the final directory entry only; traversal policy is caller-owned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SymlinkPolicy {
    /// Reject a final symlink, including one substituted immediately before open.
    Reject,
    /// Follow the final symlink and validate the opened target.
    Follow,
}

/// Device/inode evidence, not a durable identity or checkpoint key.
///
/// Inode reuse and some overlayfs copy-ups can change this evidence. Matching
/// a locator alone does not prove source continuity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileLocator {
    /// Device containing the opened inode.
    pub device: u64,
    /// Inode number, which the filesystem may reuse after deletion.
    pub inode: u64,
}

impl FileLocator {
    /// Extracts a locator. Admission evidence must come from handle metadata;
    /// path metadata can only supply an expected locator.
    #[must_use]
    pub fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

/// Failure to open or read one source file; no error authorizes source progress.
#[derive(Debug, thiserror::Error)]
pub enum FileAccessError {
    /// Cancellation was observed before the next filesystem operation.
    #[error("source operation cancelled")]
    Cancelled,
    /// The supplied name is not a single non-dot directory entry.
    #[error("source name must be one nonempty directory entry, excluding . and ..")]
    InvalidName,
    /// The opened object was not a regular file; no content was read.
    #[error("opened source is not a regular file")]
    NotRegular,
    /// The selected locator changed before open completed.
    #[error("opened source locator does not match the selection: {actual:?}")]
    LocatorChanged {
        /// Locator obtained from the newly opened handle.
        actual: FileLocator,
    },
    /// The reopened handle's type or locator differs from the validated pin.
    #[error("reopened source differs from its pinned object: {actual:?}")]
    ReopenedMismatch {
        /// Locator observed on the unexpected read handle.
        actual: FileLocator,
    },
    /// A kernel-control filesystem is ineligible as a source of ordinary logs.
    #[error("source filesystem is not eligible for filelog: {filesystem}")]
    UnsupportedFilesystem {
        /// Bounded, static name of the rejected filesystem family.
        filesystem: &'static str,
    },
    /// The requested range cannot be represented by Linux signed file offsets.
    #[error("source read range exceeds the signed 64-bit file-offset domain")]
    InvalidRange,
    /// Procfs reopening failed, preserving the OS error. ENOENT may mean
    /// unavailable procfs; EACCES may mean denied source read access.
    #[error("failed to reopen pinned source through /proc/self/fd: {0}")]
    PinnedReopen(#[source] io::Error),
    /// Original OS error, preserved for caller-owned retry and reporting policy.
    #[error("source I/O failed: {0}")]
    Io(#[from] io::Error),
}

impl FileAccessError {
    /// Returns the OS error from any I/O stage for common retry classification.
    #[must_use]
    pub fn os_error(&self) -> Option<&io::Error> {
        match self {
            Self::Io(error) | Self::PinnedReopen(error) => Some(error),
            _ => None,
        }
    }
}

/// One read-only regular-file handle and its opening metadata snapshot.
///
/// The snapshot may already be stale; use [`Self::metadata`] to refresh it.
#[derive(Debug)]
pub struct SourceFile {
    file: File,
    opened_metadata: Metadata,
}

impl SourceFile {
    /// Opens one native filename under the caller-validated `directory`.
    ///
    /// Rejects empty, dot, parent and slash-containing names. `expected`, when
    /// present, must match the pin's locator; `None` makes an initial observation.
    ///
    /// Reserve two shared transient descriptor slots. Success returns one read
    /// handle with fresh metadata; errors and cancellation close acquired handles.
    /// The directory stays caller-owned. A rejected symlink returns `NotRegular`.
    pub fn open_at(
        directory: &File,
        name: &CStr,
        symlinks: SymlinkPolicy,
        expected: Option<FileLocator>,
        cancelled: impl FnMut() -> bool,
    ) -> Result<Self, FileAccessError> {
        Self::open_at_with_reopen(
            directory,
            name,
            symlinks,
            expected,
            cancelled,
            reopen_pinned,
        )
    }

    fn open_at_with_reopen(
        directory: &File,
        name: &CStr,
        symlinks: SymlinkPolicy,
        expected: Option<FileLocator>,
        mut cancelled: impl FnMut() -> bool,
        reopen: impl FnOnce(&File) -> io::Result<File>,
    ) -> Result<Self, FileAccessError> {
        check_cancelled(&mut cancelled)?;
        let bytes = name.to_bytes();
        if bytes.is_empty() || bytes == b"." || bytes == b".." || bytes.contains(&b'/') {
            return Err(FileAccessError::InvalidName);
        }

        let mut flags = OFlag::O_PATH | OFlag::O_CLOEXEC;
        if symlinks == SymlinkPolicy::Reject {
            flags |= OFlag::O_NOFOLLOW;
        }
        let pinned =
            File::from(openat(directory, name, flags, Mode::empty()).map_err(io::Error::from)?);
        check_cancelled(&mut cancelled)?;
        let pinned_metadata = pinned.metadata()?;
        if !pinned_metadata.is_file() {
            return Err(FileAccessError::NotRegular);
        }
        let pinned_locator = FileLocator::from_metadata(&pinned_metadata);
        if let Some(expected) = expected
            && expected != pinned_locator
        {
            return Err(FileAccessError::LocatorChanged {
                actual: pinned_locator,
            });
        }

        check_cancelled(&mut cancelled)?;
        reject_kernel_control_filesystem(
            fstatfs(&pinned).map_err(io::Error::from)?.filesystem_type(),
        )?;
        check_cancelled(&mut cancelled)?;
        let file = reopen(&pinned).map_err(FileAccessError::PinnedReopen)?;
        check_cancelled(&mut cancelled)?;
        let opened_metadata = file.metadata()?;
        let actual = FileLocator::from_metadata(&opened_metadata);
        if !opened_metadata.is_file() || actual != pinned_locator {
            return Err(FileAccessError::ReopenedMismatch { actual });
        }
        Ok(Self {
            file,
            opened_metadata,
        })
    }

    /// Returns the locator observed from the opened descriptor.
    #[must_use]
    pub fn locator(&self) -> FileLocator {
        FileLocator::from_metadata(&self.opened_metadata)
    }

    /// Returns the metadata snapshot captured during open, without another syscall.
    #[must_use]
    pub fn opened_metadata(&self) -> &Metadata {
        &self.opened_metadata
    }

    /// Observes current metadata from the same handle, with no path lookup.
    pub fn metadata(
        &self,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Metadata, FileAccessError> {
        check_cancelled(&mut cancelled)?;
        Ok(self.file.metadata()?)
    }

    /// Reads once at `offset` into `buffer` without changing the file cursor.
    ///
    /// Returns short reads and temporary EOF directly. Empty buffers perform no
    /// read; the range's exclusive end must fit a signed 64-bit offset.
    /// Cancellation is checked before I/O; completed reads are always reported.
    pub fn read_at(
        &self,
        offset: u64,
        buffer: &mut [u8],
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<usize, FileAccessError> {
        check_cancelled(&mut cancelled)?;
        let len = u64::try_from(buffer.len()).map_err(|_| FileAccessError::InvalidRange)?;
        if offset
            .checked_add(len)
            .is_none_or(|end| end > i64::MAX as u64)
        {
            return Err(FileAccessError::InvalidRange);
        }
        if buffer.is_empty() {
            return Ok(0);
        }
        Ok(self.file.read_at(buffer, offset)?)
    }
}

fn reject_kernel_control_filesystem(kind: FsType) -> Result<(), FileAccessError> {
    let filesystem = match kind {
        PROC_SUPER_MAGIC => "procfs",
        SYSFS_MAGIC => "sysfs",
        DEBUGFS_MAGIC => "debugfs",
        TRACEFS_MAGIC => "tracefs",
        SECURITYFS_MAGIC => "securityfs",
        CGROUP_SUPER_MAGIC => "cgroup",
        CGROUP2_SUPER_MAGIC => "cgroup2",
        _ => return Ok(()),
    };
    Err(FileAccessError::UnsupportedFilesystem { filesystem })
}

fn reopen_pinned(pinned: &File) -> io::Result<File> {
    let mut buffer = [0_u8; 32];
    let fd = u32::try_from(pinned.as_raw_fd()).expect("an owned descriptor is nonnegative");
    let path = proc_fd_path(fd, &mut buffer);
    // NONBLOCK prevents a conflicting write lease from delaying this read-open:
    // WouldBlock is returned for caller-owned bounded retry. It does not avoid
    // waits for fanotify permission responses or uninterruptible filesystem I/O.
    // O_NOFOLLOW is omitted for this trusted procfs link, not a candidate name.
    let flags = OFlag::O_RDONLY
        | OFlag::O_CLOEXEC
        | OFlag::O_NONBLOCK
        | OFlag::O_NOCTTY
        | OFlag::O_LARGEFILE;
    Ok(File::from(
        open(path, flags, Mode::empty()).map_err(io::Error::from)?,
    ))
}

// Ten decimal digits cover every u32. Prefix, digits and NUL fit in 32 bytes;
// only the borrowed suffix is used, so the buffer can be reused without clearing.
fn proc_fd_path(mut fd: u32, buffer: &mut [u8; 32]) -> &CStr {
    const PREFIX: &[u8] = b"/proc/self/fd/";
    let mut start = buffer.len() - 1;
    buffer[start] = 0;
    loop {
        start -= 1;
        buffer[start] = b'0' + (fd % 10) as u8;
        fd /= 10;
        if fd == 0 {
            break;
        }
    }
    start -= PREFIX.len();
    buffer[start..start + PREFIX.len()].copy_from_slice(PREFIX);
    CStr::from_bytes_with_nul(&buffer[start..])
        .expect("static prefix and descriptor digits contain no NUL")
}

fn check_cancelled(cancelled: &mut impl FnMut() -> bool) -> Result<(), FileAccessError> {
    if cancelled() {
        Err(FileAccessError::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
