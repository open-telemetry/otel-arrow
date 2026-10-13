// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine-owned, startup-provisioned directory capability.
//!
//! Provisioning is blocking and belongs outside per-core pipeline runtimes.
//! Handles prevent later pathname resolution from redirecting relative operations
//! through replaced ancestors. They do not freeze permissions or defend against
//! privileged actors or other processes using the trusted service identity.

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Failure to establish a configured state root. Available on every platform.
#[derive(Debug, thiserror::Error)]
pub enum StateDirectoryError {
    /// The configured path has an invalid lexical form.
    #[error("engine.state_dir {path:?}: {reason}")]
    InvalidPath {
        /// Native configured path, without lossy conversion.
        path: PathBuf,
        /// Why this path is invalid.
        reason: &'static str,
    },
    /// This platform has no qualified directory provisioning implementation.
    #[error(
        "engine.state_dir {path:?} is unsupported on this platform; provisioning requires Linux"
    )]
    UnsupportedPlatform {
        /// Configured path.
        path: PathBuf,
    },
    /// An opened directory violates the trust policy or changed identity.
    #[error("engine.state_dir: validate {path:?}: {reason}")]
    Untrusted {
        /// Directory being validated.
        path: PathBuf,
        /// Failed ownership, permission, type, or identity requirement.
        reason: &'static str,
    },
    /// A required filesystem operation failed, including a durability barrier.
    #[error("engine.state_dir: {operation} {path:?}: {source}")]
    Io {
        /// Required operation that failed.
        operation: &'static str,
        /// Directory being operated on, for diagnostics only.
        path: PathBuf,
        /// Original operating-system error.
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug)]
struct Inner {
    path: PathBuf,
    #[cfg(target_os = "linux")]
    directory: std::fs::File,
}

/// Cheaply cloned, immutable identity of the validated engine root.
///
/// There is no engine-root lock: processes with the same service identity may
/// share it. Consumers own their namespace locking, recovery, and durability.
/// Consumers that require state must reject an absent capability.
/// Never reopen [`Self::path`] for state operations; use the borrowed directory
/// descriptor with relative filesystem APIs. Consumers must constrain relative
/// names and apply their own no-follow/type checks for descendants.
#[derive(Clone, Debug)]
pub struct StateDirectory(Arc<Inner>);

impl StateDirectory {
    /// Provisions an absolute root at startup (blocking; Linux only).
    ///
    /// Validates ancestors, creates missing directories with mode 0700,
    /// and syncs all parents and the root, including existing entries.
    /// Leaves partial state on failure. Never call this for live recovery.
    pub fn provision(path: &Path) -> Result<Self, StateDirectoryError> {
        otel_arrow_dfe_config::engine::state_dir::validate_state_dir(path).map_err(|reason| {
            StateDirectoryError::InvalidPath {
                path: path.to_owned(),
                reason,
            }
        })?;
        #[cfg(target_os = "linux")]
        {
            linux::provision(path, &mut |directory, _| directory.sync_all())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(StateDirectoryError::UnsupportedPlatform {
                path: path.to_owned(),
            })
        }
    }

    /// Configured native path for diagnostics, not an authority for later opens.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0.path
    }
}

#[cfg(target_os = "linux")]
impl std::os::fd::AsFd for StateDirectory {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.0.directory.as_fd()
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use nix::errno::Errno;
    use nix::fcntl::{AtFlags, OFlag, open, openat};
    use nix::sys::stat::{FileStat, Mode, SFlag, fstat, fstatat, mkdirat};
    use nix::unistd::geteuid;
    use std::fs::File;
    use std::path::Component;

    const DIRECTORY_OPEN_FLAGS: OFlag = OFlag::O_RDONLY
        .union(OFlag::O_DIRECTORY)
        .union(OFlag::O_NOFOLLOW)
        .union(OFlag::O_CLOEXEC);

    fn io(
        operation: &'static str,
        path: &Path,
        source: impl Into<std::io::Error>,
    ) -> StateDirectoryError {
        StateDirectoryError::Io {
            operation,
            path: path.to_owned(),
            source: source.into(),
        }
    }

    fn untrusted(path: &Path, reason: &'static str) -> StateDirectoryError {
        StateDirectoryError::Untrusted {
            path: path.to_owned(),
            reason,
        }
    }

    fn validate_directory(
        stat: &FileStat,
        path: &Path,
        uid: u32,
        require_private_permissions: bool,
    ) -> Result<(), StateDirectoryError> {
        if SFlag::from_bits_truncate(stat.st_mode) != SFlag::S_IFDIR {
            return Err(untrusted(path, "not a directory"));
        }
        if require_private_permissions {
            if stat.st_uid != uid || stat.st_mode & 0o777 != 0o700 {
                return Err(untrusted(
                    path,
                    concat!(
                        "directory must be owned by the collector effective UID with mode 0700; ",
                        "check ownership, process umask, and inherited ACLs; ",
                        "permissions are not repaired automatically",
                    ),
                ));
            }
        } else if (stat.st_uid != 0 && stat.st_uid != uid) || stat.st_mode & 0o022 != 0 {
            return Err(untrusted(
                path,
                concat!(
                    "ancestor must be owned by root or the collector effective UID, ",
                    "without group/other write permission; ",
                    "check ownership and access permissions/ACLs",
                ),
            ));
        }
        // Linux POSIX ACLs expose ACL_MASK through the group mode bits, so
        // these checks constrain effective named-user and group grants.
        // Default ACLs affect creation; validate the resulting permissions.
        Ok(())
    }

    pub(super) fn provision(
        path: &Path,
        sync: &mut impl FnMut(&File, &Path) -> std::io::Result<()>,
    ) -> Result<StateDirectory, StateDirectoryError> {
        let uid = geteuid().as_raw();
        let mut parent_path = PathBuf::from("/");
        let mut parent = File::from(
            open(Path::new("/"), DIRECTORY_OPEN_FLAGS, Mode::empty())
                .map_err(|e| io("open trusted anchor", &parent_path, e))?,
        );
        validate_directory(
            &fstat(&parent).map_err(|e| io("stat anchor", &parent_path, e))?,
            &parent_path,
            uid,
            false,
        )?;
        // Establish the anchor barrier before descending. Unsupported fsync is
        // an error, never a reason to silently weaken the durability contract.
        sync(&parent, &parent_path).map_err(|e| io("sync anchor", &parent_path, e))?;
        let mut components = path.components().skip(1).peekable();
        loop {
            let name = match components.next() {
                Some(Component::Normal(name)) => name,
                None => break,
                Some(_) => {
                    return Err(StateDirectoryError::InvalidPath {
                        path: path.to_owned(),
                        reason: "unexpected component during state-directory traversal",
                    });
                }
            };
            let child_path = parent_path.join(name);
            let mut created = false;
            let child = match openat(&parent, name, DIRECTORY_OPEN_FLAGS, Mode::empty()) {
                Ok(fd) => fd,
                Err(Errno::ENOENT) => {
                    match mkdirat(&parent, name, Mode::S_IRWXU) {
                        Ok(()) => created = true,
                        Err(Errno::EEXIST) => (),
                        Err(e) => return Err(io("create directory (0700)", &child_path, e)),
                    }
                    // A concurrent creator may have won. Always inspect the
                    // object actually opened, without following a symlink.
                    openat(&parent, name, DIRECTORY_OPEN_FLAGS, Mode::empty())
                        .map_err(|e| io("open created directory", &child_path, e))?
                }
                Err(e) => return Err(io("open directory", &child_path, e)),
            };
            let child = File::from(child);
            let stat = fstat(&child).map_err(|e| io("stat directory", &child_path, e))?;
            let final_root = components.peek().is_none();
            let require_private_permissions = final_root || created;
            validate_directory(&stat, &child_path, uid, require_private_permissions)?;
            sync(&parent, &parent_path)
                .map_err(|e| io("sync parent directory", &parent_path, e))?;
            if final_root {
                sync(&child, path).map_err(|e| io("sync state root", path, e))?;
            }
            let entry = fstatat(&parent, name, AtFlags::AT_SYMLINK_NOFOLLOW)
                .map_err(|e| io("recheck directory entry", &child_path, e))?;
            if entry.st_dev != stat.st_dev || entry.st_ino != stat.st_ino {
                return Err(untrusted(
                    &child_path,
                    "directory entry changed during provisioning",
                ));
            }
            validate_directory(&entry, &child_path, uid, require_private_permissions)?;
            // At most two directory descriptors are live, independent of depth.
            parent = child;
            parent_path = child_path;
        }
        Ok(StateDirectory(Arc::new(Inner {
            path: path.to_owned(),
            directory: parent,
        })))
    }

    #[cfg(test)]
    mod tests;
}

#[cfg(all(test, not(target_os = "linux")))]
mod tests {
    use super::*;

    /// Scenario: a root is configured on an unsupported platform.
    /// Guarantees: provisioning returns a portable error without creating state.
    #[test]
    fn unsupported_platform() {
        let path = if cfg!(windows) {
            Path::new(r"C:\otel\state")
        } else {
            Path::new("/var/lib/otel/state")
        };
        assert!(matches!(
            StateDirectory::provision(path),
            Err(StateDirectoryError::UnsupportedPlatform { .. })
        ));
    }
}
