// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::probe::{pinned_path, verify_path};
use super::*;
use rustix::fs::{CWD, Mode, OFlags, openat};
use std::{fs::File, io};

pub(super) struct ExclusionRoot {
    pub locator: FileLocator,
    pub index: usize,
    // When the literal prefix is absent, map through its verified ancestor.
    pub missing_suffix: PathBuf,
}

pub(super) fn resolve_include_root(
    root: &Path,
    scratch: &mut [u8],
    baseline: &mut Option<FileLocator>,
) -> Result<(PathBuf, FileLocator, bool), (ScanReason, io::Error)> {
    let mut resolve = || -> io::Result<(PathBuf, FileLocator)> {
        let file = File::from(openat(
            CWD,
            root,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )?);
        let locator = FileLocator::from_metadata(&file.metadata()?);
        let path = pinned_path(&file, scratch)?.to_owned();
        drop(file); // One directory handle, including root verification.
        verify_path(&path, locator)?;
        Ok((path, locator))
    };
    let (path, locator) = resolve().map_err(|error| (ScanReason::RootUnavailable, error))?;
    let changed = baseline.is_some_and(|old| old != locator);
    if baseline.is_none() {
        *baseline = Some(locator);
    }
    // Positive observations may use the current directory without adopting it
    // as the expected environment or authorizing negative conclusions.
    Ok((path, locator, changed))
}

// Only an initial path lookup returning ENOENT can shorten a literal prefix.
// A dangling prefix alias or a failure after opening a root remains a failure.
pub(super) fn resolve_exclusion(
    root: &Path,
    scratch: &mut [u8],
    mapping: &mut ExclusionRoot,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), (ScanReason, io::Error)> {
    let mut ancestor = root;
    loop {
        if cancelled() {
            return Err((
                ScanReason::Cancelled,
                io::Error::from_raw_os_error(libc::ECANCELED),
            ));
        }
        match openat(
            CWD,
            ancestor,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(fd) => {
                let file = File::from(fd);
                let identity = file
                    .metadata()
                    .map(|m| FileLocator::from_metadata(&m))
                    .map_err(|e| (ScanReason::Policy, e))?;
                let canonical = pinned_path(&file, scratch).map_err(|e| (ScanReason::Policy, e))?;
                drop(file);
                verify_path(canonical, identity).map_err(|e| (ScanReason::Policy, e))?;
                mapping.locator = identity;
                mapping.missing_suffix.clear();
                mapping
                    .missing_suffix
                    .push(root.strip_prefix(ancestor).expect("literal ancestor"));
                return Ok(());
            }
            Err(error) if error == rustix::io::Errno::NOENT => {
                // Unlike metadata(), this exposes a dangling final symlink.
                match std::fs::symlink_metadata(ancestor) {
                    Err(e) if e.raw_os_error() == Some(libc::ENOENT) => (),
                    Err(e) => return Err((ScanReason::Policy, e)),
                    Ok(_) => return Err((ScanReason::Policy, error.into())),
                }
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| (ScanReason::Policy, io::Error::from(error)))?;
            }
            Err(error) => return Err((ScanReason::Policy, error.into())),
        }
    }
}
