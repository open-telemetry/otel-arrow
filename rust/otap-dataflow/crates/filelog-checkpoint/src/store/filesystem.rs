// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Descriptor-relative Linux namespace operations. No diagnostic pathname is
//! reopened and no namespace artifact is followed through a symbolic link.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::BorrowedFd;

use nix::dir::Dir;
use nix::errno::Errno;
use nix::fcntl::{AtFlags, FcntlArg, OFlag, fcntl, openat};
use nix::sys::stat::{FileStat, Mode, SFlag, fstat, fstatat, mkdirat};
use nix::unistd::{UnlinkatFlags, geteuid, unlinkat};
// nix::renameat2 is GNU-only; rustix also supports Linux musl. Its flock API
// avoids File::try_lock, which is newer than the workspace's Rust 1.88 MSRV.
use rustix::fs::{FlockOperation, RenameFlags, flock, renameat_with};

use super::{COMPACT, CREATE, DB, StoreError, io};

const LOCK: &str = "ownership.lock";
const DIRECTORY: OFlag = OFlag::O_RDONLY
    .union(OFlag::O_DIRECTORY)
    .union(OFlag::O_NOFOLLOW)
    .union(OFlag::O_CLOEXEC);
const REGULAR: OFlag = OFlag::O_RDWR
    .union(OFlag::O_NOFOLLOW)
    .union(OFlag::O_CLOEXEC)
    .union(OFlag::O_NONBLOCK);

/// Encodes the exact configured ID, with no normalization or case folding.
pub(super) fn namespace_component(id: &str) -> Result<String, StoreError> {
    if id.is_empty()
        || id.len() > 127
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
    {
        return Err(StoreError::InvalidConfig(
            "checkpoint ID must be 1..=127 ASCII alphanumeric, underscore, hyphen, or dot bytes",
        ));
    }
    let mut component = String::with_capacity(id.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in id.bytes() {
        component.push(char::from(HEX[usize::from(byte >> 4)]));
        component.push(char::from(HEX[usize::from(byte & 15)]));
    }
    Ok(component)
}

/// Owned capability for an already provisioned checkpoint root.
///
/// Construction validates the directory but does not provision it or establish
/// durability of its ancestors. The caller must complete those steps first.
/// The handle is private: store operations never reopen a diagnostic pathname.
#[derive(Debug)]
pub struct StateRoot {
    directory: File,
}

impl StateRoot {
    /// Duplicates a borrowed Linux directory descriptor into an owned capability.
    ///
    /// The directory must belong to the effective UID with access mode 0700.
    /// Path-only (`O_PATH`) descriptors are rejected: namespace creation needs
    /// a root handle that supports synchronization, not just relative lookup.
    /// The original descriptor may be closed after this call. This performs no
    /// synchronization and does not certify the caller's provisioning history.
    pub fn from_borrowed_fd(root: BorrowedFd<'_>) -> Result<Self, StoreError> {
        let directory = File::from(
            root.try_clone_to_owned()
                .map_err(|e| io("duplicate root handle", e))?,
        );
        validate(&fstat(&directory).map_err(|e| io("stat root", e))?, true)?;
        let flags =
            fcntl(&directory, FcntlArg::F_GETFL).map_err(|e| io("read root handle flags", e))?;
        if OFlag::from_bits_truncate(flags).contains(OFlag::O_PATH) {
            return Err(StoreError::InvalidConfig(
                "state root must support synchronization; O_PATH is not supported",
            ));
        }
        Ok(Self { directory })
    }
}

/// Active checkpoint handle. Writes always append at physical EOF, regardless
/// of the read cursor used by recovery. Only Namespace can construct this type;
/// neither the raw file nor a general-purpose Write implementation is exposed.
pub(super) struct DataFile {
    file: File,
}

impl DataFile {
    pub fn len(&self) -> std::io::Result<u64> {
        self.file.metadata().map(|metadata| metadata.len())
    }

    pub fn truncate(&self, length: u64) -> std::io::Result<()> {
        self.file.set_len(length)
    }

    pub fn append(&mut self, io_ops: &mut Io, bytes: &[u8]) -> Result<(), StoreError> {
        io_ops.write_all(&mut self.file, bytes)
    }

    pub fn sync(&self, io_ops: &mut Io, operation: &'static str) -> Result<(), StoreError> {
        io_ops.sync(&self.file, operation)
    }

    #[cfg(test)]
    pub fn as_raw_fd(&self) -> std::os::fd::RawFd {
        std::os::fd::AsRawFd::as_raw_fd(&self.file)
    }
}

impl Read for DataFile {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(bytes)
    }
}

impl Seek for DataFile {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(position)
    }
}

pub(super) struct Namespace {
    directory: File,
    // Keep ancestors open so namespace identity can be rechecked before writes.
    root: File,
    filelog: File,
    version: File,
    component: String,
    lock: File,
    pub created: bool,
}

#[derive(Default)]
pub(super) struct Inventory {
    pub database: bool,
    pub create: bool,
    pub compact: bool,
}

impl Namespace {
    /// Opens and locks the namespace, creating missing directories as needed.
    /// Each child directory is validated and its parent synchronized, including
    /// existing entries. These ancestor barriers precede all artifact access;
    /// the protocol still orders publication, recovery and cleanup barriers.
    pub fn open(root: &StateRoot, id: &str, io_ops: &mut Io) -> Result<Self, StoreError> {
        let component = namespace_component(id)?;
        // Each namespace owns its root handle independently of the caller.
        let root = root
            .directory
            .try_clone()
            .map_err(|e| io("duplicate root handle", e))?;
        validate(&fstat(&root).map_err(|e| io("stat root", e))?, true)?;
        let (filelog, _) = child_directory(&root, "filelog", io_ops)?;
        let (version, _) = child_directory(&filelog, "@v1", io_ops)?;
        let (directory, created) = child_directory(&version, &component, io_ops)?;
        let lock = open_regular(&directory, LOCK, REGULAR | OFlag::O_CREAT)?;
        if lock.metadata().map_err(|e| io("stat lock", e))?.len() != 0 {
            return Err(StoreError::Untrusted("lock file must be empty"));
        }
        flock(&lock, FlockOperation::NonBlockingLockExclusive).map_err(|error| {
            if error == rustix::io::Errno::WOULDBLOCK {
                StoreError::Locked
            } else {
                io("lock namespace", error)
            }
        })?;
        verify_entry(&directory, LOCK, &lock)?;
        let namespace = Self {
            directory,
            root,
            filelog,
            version,
            component,
            lock,
            created,
        };
        namespace.verify_ancestry()?;
        Ok(namespace)
    }

    fn verify_ancestry(&self) -> Result<(), StoreError> {
        verify_entry(&self.root, "filelog", &self.filelog)?;
        verify_entry(&self.filelog, "@v1", &self.version)?;
        verify_entry(&self.version, &self.component, &self.directory)?;
        verify_entry(&self.directory, LOCK, &self.lock)
    }

    pub fn verify(&self, name: &str, file: &File) -> Result<(), StoreError> {
        self.verify_ancestry()?;
        verify_entry(&self.directory, name, file)
    }

    pub fn inventory(&self) -> Result<Inventory, StoreError> {
        self.verify_ancestry()?;
        // Reopen '.' to give iteration its own directory offset. Never allocate
        // a list proportional to unknown directory contents: reject immediately.
        let fd = openat(&self.directory, ".", DIRECTORY, Mode::empty())
            .map_err(|e| io("open inventory", e))?;
        let mut directory = Dir::from_fd(fd).map_err(|e| io("read inventory", e))?;
        let mut inventory = Inventory::default();
        for entry in directory.iter() {
            let entry = entry.map_err(|e| io("read inventory entry", e))?;
            let name = entry.file_name().to_bytes();
            let name = match name {
                b"." | b".." => continue,
                b"ownership.lock" => LOCK,
                b"checkpoint.db" => {
                    inventory.database = true;
                    DB
                }
                b"checkpoint.db.create.tmp" => {
                    inventory.create = true;
                    CREATE
                }
                b"checkpoint.db.compact.tmp" => {
                    inventory.compact = true;
                    COMPACT
                }
                _ => return Err(StoreError::Untrusted("unexpected namespace artifact")),
            };
            let stat = fstatat(&self.directory, name, AtFlags::AT_SYMLINK_NOFOLLOW)
                .map_err(|e| io("stat inventory artifact", e))?;
            validate(&stat, false)?;
        }
        if inventory.create && inventory.compact {
            return Err(StoreError::Untrusted("multiple checkpoint temporaries"));
        }
        Ok(inventory)
    }

    pub fn create(&self, name: &str) -> Result<File, StoreError> {
        self.verify_ancestry()?;
        open_regular(
            &self.directory,
            name,
            REGULAR | OFlag::O_CREAT | OFlag::O_EXCL,
        )
    }

    pub fn open_database(&self) -> Result<DataFile, StoreError> {
        self.verify_ancestry()?;
        // O_APPEND ensures writes after reads/recovery do not depend on a cursor.
        match open_regular(&self.directory, DB, REGULAR | OFlag::O_APPEND) {
            Err(StoreError::Io { source, .. }) => {
                Err(StoreError::AuthoritativeGenerationUnreadable(source))
            }
            result => result.map(|file| DataFile { file }),
        }
    }

    pub fn verify_database(&self, file: &DataFile) -> Result<(), StoreError> {
        self.verify(DB, &file.file)
    }

    /// Synchronizes namespace entries. The protocol chooses when this barrier
    /// is required; the same fault-injection and poison latch covers every sync.
    pub fn sync_directory(
        &self,
        io_ops: &mut Io,
        operation: &'static str,
    ) -> Result<(), StoreError> {
        io_ops.sync(&self.directory, operation)
    }

    /// Atomically publishes the creation temporary without replacing authority.
    /// This does not sync the directory; publication is durable only after the
    /// protocol's subsequent sync_directory call succeeds. No fallback is used.
    pub fn publish_no_replace(&self) -> Result<(), StoreError> {
        self.verify_ancestry()?;
        renameat_with(
            &self.directory,
            CREATE,
            &self.directory,
            DB,
            RenameFlags::NOREPLACE,
        )
        .map_err(|e| io("publish initial checkpoint without replacement", e))
    }

    pub fn remove(&self, name: &str) -> Result<(), StoreError> {
        debug_assert!(matches!(name, CREATE | COMPACT));
        self.verify_ancestry()?;
        // Opening and validating rejects symlinks, hard links, special files,
        // permission changes, and replacement before removal.
        match open_regular(&self.directory, name, REGULAR) {
            Ok(file) => self.verify(name, &file)?,
            Err(StoreError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(error),
        }
        match unlinkat(&self.directory, name, UnlinkatFlags::NoRemoveDir) {
            Ok(()) | Err(Errno::ENOENT) => Ok(()),
            Err(error) => Err(io("remove checkpoint temporary", error)),
        }
    }
}

fn child_directory(parent: &File, name: &str, io_ops: &mut Io) -> Result<(File, bool), StoreError> {
    let mut created = false;
    let fd = match openat(parent, name, DIRECTORY, Mode::empty()) {
        Ok(fd) => fd,
        Err(Errno::ENOENT) => {
            match mkdirat(parent, name, Mode::S_IRWXU) {
                Ok(()) => created = true,
                Err(Errno::EEXIST) => (),
                Err(error) => return Err(io("create namespace directory", error)),
            }
            openat(parent, name, DIRECTORY, Mode::empty())
                .map_err(|e| io("open created namespace directory", e))?
        }
        Err(error) => return Err(io("open namespace directory", error)),
    };
    let file = File::from(fd);
    validate(
        &fstat(&file).map_err(|e| io("stat namespace directory", e))?,
        true,
    )?;
    io_ops.sync(parent, "namespace parent sync")?;
    verify_entry(parent, name, &file)?;
    Ok((file, created))
}

fn open_regular(parent: &File, name: &str, flags: OFlag) -> Result<File, StoreError> {
    let file = File::from(
        openat(parent, name, flags, Mode::S_IRUSR | Mode::S_IWUSR)
            .map_err(|e| io("open namespace artifact", e))?,
    );
    validate(
        &fstat(&file).map_err(|e| io("stat namespace artifact", e))?,
        false,
    )?;
    verify_entry(parent, name, &file)?;
    Ok(file)
}

fn validate(stat: &FileStat, directory: bool) -> Result<(), StoreError> {
    let kind = if directory {
        SFlag::S_IFDIR
    } else {
        SFlag::S_IFREG
    };
    let mode = if directory { 0o700 } else { 0o600 };
    // Match engine provisioning: directory access is governed by the low nine
    // permission bits. Inherited setgid/sticky bits do not grant extra access.
    // Regular artifacts still reject all special mode bits.
    let permission_mask = if directory { 0o777 } else { 0o7777 };
    if SFlag::from_bits_truncate(stat.st_mode) != kind
        || stat.st_uid != geteuid().as_raw()
        || stat.st_mode & permission_mask != mode
        || (!directory && stat.st_nlink != 1)
    {
        return Err(StoreError::Untrusted(
            "wrong type, owner, permissions, or hard-link count",
        ));
    }
    Ok(())
}

fn verify_entry(parent: &File, name: &str, file: &File) -> Result<(), StoreError> {
    let held = fstat(file).map_err(|e| io("stat held artifact", e))?;
    let entry = fstatat(parent, name, AtFlags::AT_SYMLINK_NOFOLLOW)
        .map_err(|e| io("recheck artifact name", e))?;
    let is_directory = SFlag::from_bits_truncate(held.st_mode) == SFlag::S_IFDIR;
    validate(&held, is_directory)?;
    validate(&entry, is_directory)?;
    if held.st_dev != entry.st_dev || held.st_ino != entry.st_ino {
        return Err(StoreError::AuthorityChanged);
    }
    Ok(())
}

// Unit tests can fail
// one named barrier or a write after an exact byte count, using real files.
#[derive(Default)]
pub(super) struct Io {
    pub(super) sync_failed: Option<&'static str>,
    #[cfg(test)]
    pub fail_sync: Option<&'static str>,
    #[cfg(test)]
    pub fail_write_after: Option<usize>,
    #[cfg(test)]
    pub events: Vec<&'static str>,
    #[cfg(test)]
    pub sync_fds: Vec<std::os::fd::RawFd>,
}

impl Io {
    pub fn check_sync_health(&self) -> Result<(), StoreError> {
        match self.sync_failed {
            Some(operation) => Err(StoreError::SyncPoisoned { operation }),
            None => Ok(()),
        }
    }

    pub fn sync(&mut self, file: &File, operation: &'static str) -> Result<(), StoreError> {
        self.check_sync_health()?;
        // Remains latched on every returned failure, including nested recovery
        // barriers. No future call can mistake a one-shot error for durability.
        self.sync_failed = Some(operation);
        #[cfg(test)]
        {
            self.events.push(operation);
            self.sync_fds.push(std::os::fd::AsRawFd::as_raw_fd(file));
            if self.fail_sync == Some(operation) {
                self.fail_sync = None;
                return Err(StoreError::SyncFailed {
                    operation,
                    source: std::io::Error::other("injected sync failure"),
                });
            }
        }
        file.sync_all()
            .map_err(|source| StoreError::SyncFailed { operation, source })?;
        self.sync_failed = None;
        Ok(())
    }

    pub fn write_all(&mut self, file: &mut File, bytes: &[u8]) -> Result<(), StoreError> {
        #[cfg(test)]
        if let Some(count) = self.fail_write_after.take() {
            file.write_all(&bytes[..count.min(bytes.len())])
                .map_err(|e| io("write checkpoint", e))?;
            return Err(io(
                "write checkpoint",
                std::io::Error::other("injected write failure"),
            ));
        }
        file.write_all(bytes).map_err(|e| io("write checkpoint", e))
    }
}
