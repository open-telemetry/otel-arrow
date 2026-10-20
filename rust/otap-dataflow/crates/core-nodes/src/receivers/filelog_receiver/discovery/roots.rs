// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::native_path::NativePath;
use super::probe::{pinned_path, verify_path};
use super::*;
use rustix::fs::{AtFlags, CWD, Mode, OFlags, openat, statat};
use std::{fs::File, io};

pub(super) struct ExclusionRoot {
    pub locator: FileLocator,
    pub index: usize,
    // When the literal prefix is absent, map through its verified ancestor.
    pub missing_suffix: PathBuf,
}

pub(super) fn resolve_include_root<'a>(
    root: &Path,
    scratch: &'a mut [u8],
    native: &mut NativePath,
    baseline: &mut Option<FileLocator>,
    checkpoint_namespace: FileLocator,
) -> Result<(&'a Path, FileLocator, bool), (ScanReason, io::Error)> {
    native
        .load(root)
        .map_err(|e| (ScanReason::RootUnavailable, e))?;
    let file = File::from(
        openat(
            CWD,
            native.as_c_str(),
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|e| (ScanReason::RootUnavailable, io::Error::from(e)))?,
    );
    let (path, locator) =
        verify_directory(file, scratch, native).map_err(|e| (ScanReason::RootUnavailable, e))?;
    if locator == checkpoint_namespace {
        return Err((
            ScanReason::RootUnavailable,
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "include root resolves to the checkpoint namespace",
            ),
        ));
    }
    let changed = baseline.is_some_and(|old| old != locator);
    if baseline.is_none() {
        *baseline = Some(locator);
    }
    // Positive observations may use the current directory without adopting it
    // as the expected environment or authorizing negative conclusions.
    Ok((path, locator, changed))
}

fn verify_directory<'a>(
    file: File,
    scratch: &'a mut [u8],
    native: &mut NativePath,
) -> io::Result<(&'a Path, FileLocator)> {
    let locator = FileLocator::from_metadata(&file.metadata()?);
    let path = pinned_path(&file, scratch)?;
    drop(file); // One directory handle, including verification.
    native.load(path)?;
    verify_path(native.as_c_str(), locator)?;
    Ok((path, locator))
}

// Only an initial path lookup returning ENOENT can shorten a literal prefix.
// A dangling prefix alias or a failure after opening a root remains a failure.
pub(super) fn resolve_exclusion(
    root: &Path,
    scratch: &mut [u8],
    native: &mut NativePath,
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
        native
            .load(ancestor)
            .map_err(|e| (ScanReason::ExclusionUnavailable, e))?;
        match openat(
            CWD,
            native.as_c_str(),
            OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(fd) => {
                let (_, identity) = verify_directory(File::from(fd), scratch, native)
                    .map_err(|e| (ScanReason::ExclusionUnavailable, e))?;
                mapping.locator = identity;
                mapping.missing_suffix.clear();
                mapping
                    .missing_suffix
                    .push(root.strip_prefix(ancestor).expect("literal ancestor"));
                return Ok(());
            }
            Err(error) if error == rustix::io::Errno::NOENT => {
                // Inspect the link itself without an allocating path conversion.
                match statat(CWD, native.as_c_str(), AtFlags::SYMLINK_NOFOLLOW) {
                    Err(rustix::io::Errno::NOENT) => (),
                    Err(e) => return Err((ScanReason::ExclusionUnavailable, e.into())),
                    Ok(_) => return Err((ScanReason::ExclusionUnavailable, error.into())),
                }
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| (ScanReason::ExclusionUnavailable, io::Error::from(error)))?;
            }
            Err(error) => return Err((ScanReason::ExclusionUnavailable, error.into())),
        }
    }
}
