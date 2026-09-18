// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Descriptor-relative Linux source-file access.
//!
//! The caller selects and validates a directory handle, then supplies one native
//! filename. Keeping that handle across selection and open prevents an ancestor
//! path replacement from redirecting the open. Discovery owns root resolution,
//! descendant traversal, resolved-target exclusions, and directory validation.
//! Following a final symlink may leave the directory; this API is not a sandbox.
//!
//! An O_PATH probe pins the target without a device open or FIFO reader. Only a
//! verified regular file is reopened through trusted Linux `/proc/self/fd`;
//! there is no fallback to the original pathname. The pinned source filesystem
//! must not be procfs, sysfs, debugfs, tracefs, securityfs, or cgroup v1/v2. These
//! kernel-control filesystems may report regular files without ordinary log-file
//! semantics. This is not a filesystem allowlist or path confinement: regular
//! files on tmpfs remain eligible, and discovery still owns resolved-target policy.
//! Conversion briefly owns two descriptors. Callers must reserve that peak within
//! the receiver's shared source-open allowance, including for resident reopens.
//! Each successful return owns one read-only regular-file descriptor. Metadata and
//! content are obtained from that descriptor, including after rename or unlink.
//! A locator is current device/inode evidence, not a durable identity: inode reuse
//! and copytruncate still require caller-side fingerprint and continuity checks.
//! Some overlayfs configurations can change device/inode evidence during copy-up;
//! a mismatch remains a continuity failure, not permission to inherit progress.
//! Candidate admission requires two observations (closing the first probe before
//! the second), equal locators, nondecreasing size, and compatible fingerprints.
//! Opening a file or matching a locator alone does not establish admission.
//!
//! Run filesystem work on a blocking worker. Cancellation is cooperative between
//! operations; O_PATH avoids opening non-regular objects for I/O, but neither
//! O_PATH nor O_NONBLOCK makes filesystem calls interruptible. Fanotify permission
//! responses can also delay a read-open. NONBLOCK makes conflicting file leases
//! return WouldBlock rather than wait; callers may retry with bounded backoff.
//! Interrupted and other OS errors are returned without a retry loop, quarantine,
//! or progress advancement.

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

/// Handle-derived Linux locator; never a permanent identity or checkpoint key.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileLocator {
    /// Device containing the opened inode.
    pub device: u64,
    /// Inode number, which the filesystem may reuse after deletion.
    pub inode: u64,
}

impl FileLocator {
    /// Extracts locator evidence from a metadata observation.
    ///
    /// Callers must obtain acceptance evidence from an opened handle; path
    /// metadata may only supply an expectation checked against that handle.
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
    /// The read handle does not match the regular-file pin that was validated.
    /// This is distinct from an earlier selection mismatch; it grants no progress.
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
    /// Reopening the pinned regular file through `/proc/self/fd` failed.
    ///
    /// The source preserves the OS error: ENOENT may mean unavailable procfs;
    /// EACCES may mean denied read access to the pinned file. Neither permits
    /// a fallback to the original source path.
    #[error("failed to reopen pinned source through /proc/self/fd: {0}")]
    PinnedReopen(#[source] io::Error),
    /// Original OS error, preserved for caller-owned retry and reporting policy.
    #[error("source I/O failed: {0}")]
    Io(#[from] io::Error),
}

impl FileAccessError {
    /// Returns the underlying OS error, regardless of the failing I/O stage.
    ///
    /// Callers should use this to classify interruption, permission failures,
    /// WouldBlock and descriptor pressure consistently for both open stages.
    #[must_use]
    pub fn os_error(&self) -> Option<&io::Error> {
        match self {
            Self::Io(error) | Self::PinnedReopen(error) => Some(error),
            _ => None,
        }
    }
}

/// One owned read-only regular-file handle and its opening metadata observation.
///
/// The opening snapshot can become stale immediately. Use [`Self::metadata`]
/// when new evidence is required. This type retains no paths or source bytes.
/// It allocates no read buffers, and reads do not change the file cursor.
#[derive(Debug)]
pub struct SourceFile {
    file: File,
    opened_metadata: Metadata,
}

impl SourceFile {
    /// Opens one name relative to a caller-held directory and validates its handle.
    ///
    /// `name` is borrowed, already NUL-terminated native bytes, avoiding path
    /// construction on each probe. Slash-containing, empty, dot and parent names
    /// are rejected. `expected` checks an earlier selection/observation; `None`
    /// makes an initial observation, not a stable candidate admission.
    ///
    /// The O_PATH descriptor is checked before any read-open. Reopening uses
    /// `/proc/self/fd` while the pin is held, then validates the read handle and
    /// captures fresh metadata. Rejecting a symlink returns `NotRegular`.
    ///
    /// Cancellation is checked before each open, metadata and filesystem query. Every failure
    /// closes all acquired descriptors. Reserve two transient descriptor slots
    /// for the conversion; only the read handle survives success. The borrowed
    /// directory stays caller-owned. There are no retries or path fallbacks.
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

    /// Makes one positional read attempt into `buffer`, without changing a cursor.
    ///
    /// Work and requested bytes are bounded by the caller's buffer. Short reads
    /// are returned directly; zero on a nonempty buffer means EOF at this instant,
    /// not permanent EOF. An empty buffer returns zero without a read syscall.
    /// The range's exclusive end must fit a signed 64-bit offset.
    ///
    /// Cancellation is checked before the read, never after discarding a completed
    /// result. The caller always receives the byte count if the read succeeds,
    /// even if cancellation arrives in the meantime. No heap buffer or retry loop
    /// is created; interruption, permission and storage errors remain OS errors.
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
