// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::native_path::NativePath;
use super::roots::ExclusionRoot;
use super::*;
use crate::receivers::filelog_receiver::source::{FileAccessError, PinnedSource, SourceFile};
use rustix::fs::{AtFlags, CWD, Mode, OFlags, ResolveFlags, openat2, readlinkat_raw, statat};
use std::ffi::{CStr, OsStr};
use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;

pub(super) fn locator(stat: &rustix::fs::Stat) -> FileLocator {
    FileLocator {
        device: stat.st_dev,
        inode: stat.st_ino,
    }
}

/// Reads only the procfs link text, bounded independently of filesystem input.
pub(super) fn pinned_path<'a>(file: &File, scratch: &'a mut [u8]) -> io::Result<&'a Path> {
    let mut name = [0_u8; 32];
    let path = super::super::source::proc_fd_path(file.as_raw_fd() as u32, &mut name);
    let count = readlinkat_raw(CWD, path, &mut *scratch)?;
    if count == scratch.len() || count == 0 || scratch[0] != b'/' {
        return Err(io::Error::from_raw_os_error(libc::ENAMETOOLONG));
    }
    Ok(Path::new(OsStr::from_bytes(&scratch[..count])))
}

/// Verification must reject intermediate symlinks too. No unrestricted fallback.
pub(super) fn verify_path(path: &CStr, expected: FileLocator) -> io::Result<()> {
    let verified = File::from(openat2(
        CWD,
        path,
        OFlags::PATH | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS,
    )?);
    if FileLocator::from_metadata(&verified.metadata()?) != expected {
        return Err(io::Error::from_raw_os_error(libc::ESTALE));
    }
    Ok(())
}

pub(super) struct ProbeScratch {
    pub native: NativePath,
    target: Vec<u8>,
    first_target: Vec<u8>,
    first: Vec<u8>,
    second: Vec<u8>,
}

impl ProbeScratch {
    pub fn retained_bytes(&self) -> usize {
        self.native.heap_bytes()
            + self.target.capacity()
            + self.first_target.capacity()
            + self.first.capacity()
            + self.second.capacity()
    }
    pub fn new(limits: ScanLimits) -> Self {
        Self {
            native: NativePath::new(limits.path_bytes),
            target: vec![0; limits.path_bytes],
            first_target: vec![0; limits.path_bytes],
            first: vec![0; limits.fingerprint_bytes],
            second: vec![0; limits.fingerprint_bytes],
        }
    }
}

fn fingerprint(
    source: &SourceFile,
    offset: u64,
    bytes: &mut [u8],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<usize, FileAccessError> {
    let mut count = 0;
    while count < bytes.len() {
        let n = source.read_at(offset + count as u64, &mut bytes[count..], &mut *cancelled)?;
        if n == 0 {
            break;
        }
        count += n;
    }
    Ok(count)
}

fn policy(
    pin: &PinnedSource,
    matcher: &impl MatchPolicy,
    roots: &[ExclusionRoot],
    namespace: FileLocator,
    scratch: &mut [u8],
    native: &mut NativePath,
    cancelled: &mut impl FnMut() -> bool,
) -> io::Result<(bool, usize)> {
    let target = pinned_path(pin.file(), scratch)?;
    native.load(target)?;
    verify_path(native.as_c_str(), pin.locator())?;
    if matcher.excludes(target) {
        return Ok((false, target.as_os_str().len()));
    }
    for ancestor in target.ancestors().skip(1) {
        if cancelled() {
            return Err(io::Error::from_raw_os_error(libc::ECANCELED));
        }
        let identity = native.with_prefix(ancestor.as_os_str().len(), |path| {
            Ok(locator(&statat(CWD, path, AtFlags::SYMLINK_NOFOLLOW)?))
        })?;
        if identity == namespace {
            return Ok((false, target.as_os_str().len()));
        }
        for root in roots {
            if identity == root.locator
                && let Ok(suffix) = target
                    .strip_prefix(ancestor)
                    .expect("ancestor is a path prefix")
                    .strip_prefix(&root.missing_suffix)
                && matcher.excludes_mapped(root.index, suffix)
            {
                return Ok((false, target.as_os_str().len()));
            }
        }
    }
    // Detect substitution during ancestry inspection. This is not an atomic
    // ancestry snapshot; the specification explicitly bounds that guarantee.
    verify_path(native.as_c_str(), pin.locator())?;
    Ok((true, target.as_os_str().len()))
}

fn access_failure(out: &mut Observation, error: FileAccessError, reading: bool) {
    out.errno = error.os_error().and_then(io::Error::raw_os_error);
    out.reason = Some(match &error {
        FileAccessError::Cancelled => ScanReason::Cancelled,
        FileAccessError::NotRegular
        | FileAccessError::IsDirectory
        | FileAccessError::UnsupportedFilesystem { .. } => ScanReason::Unsupported,
        FileAccessError::LocatorChanged { .. } | FileAccessError::ReopenedMismatch { .. } => {
            out.locator = None;
            ScanReason::Unstable
        }
        _ if matches!(out.errno, Some(libc::EMFILE | libc::ENFILE)) => {
            ScanReason::DescriptorExhausted
        }
        _ if reading => ScanReason::Content,
        _ if out.errno == Some(libc::ENOENT) => ScanReason::Missing,
        _ if out.errno == Some(libc::ELOOP) => ScanReason::LeafLoop,
        _ => ScanReason::Coverage,
    });
}

pub(super) struct ProbeContext<'a, P> {
    pub config: &'a ScanConfig,
    pub matcher: &'a P,
    pub roots: &'a [ExclusionRoot],
    pub budget: &'a ProbeBudget,
    pub policy_available: bool,
}

pub(super) fn probe<P: MatchPolicy>(
    context: &ProbeContext<'_, P>,
    directory: &File,
    name: &CStr,
    observed_regular: bool,
    requires_subtree: bool,
    request: ProbeRequest,
    out: &mut Observation,
    scratch: &mut ProbeScratch,
    cancelled: &mut impl FnMut() -> bool,
) {
    probe_with_open(
        context,
        directory,
        name,
        observed_regular,
        requires_subtree,
        request,
        out,
        scratch,
        cancelled,
        |directory, name, symlinks, cancelled| {
            PinnedSource::open_at(directory, name, symlinks, None, cancelled)
        },
    );
}

pub(super) fn probe_with_open<P: MatchPolicy, F: FnMut() -> bool>(
    context: &ProbeContext<'_, P>,
    directory: &File,
    name: &CStr,
    observed_regular: bool,
    requires_subtree: bool,
    request: ProbeRequest,
    out: &mut Observation,
    scratch: &mut ProbeScratch,
    cancelled: &mut F,
    mut open: impl FnMut(&File, &CStr, SymlinkPolicy, &mut F) -> Result<PinnedSource, FileAccessError>,
) {
    let reservation = context.budget.try_reserve().or_else(|| {
        // One scheduler yield, not a spin loop or an unbounded retry queue.
        std::thread::yield_now();
        if cancelled() {
            out.reason = Some(ScanReason::Cancelled);
            None
        } else {
            context.budget.try_reserve()
        }
    });
    let Some(_reservation) = reservation else {
        return;
    };
    let mut prior = None;
    let mut first_size = 0;
    let mut first_len = 0;
    let mut first_target_len = 0;
    for pass in 0..2 {
        let pin = match open(directory, name, context.config.symlinks, &mut *cancelled) {
            Ok(pin) => pin,
            Err(error) => {
                // An unsuccessful second open cannot certify that the first
                // locator remained consistent. Do not export stale metadata.
                if pass == 1 {
                    out.locator = None;
                }
                if (observed_regular
                    && matches!(
                        error,
                        FileAccessError::NotRegular | FileAccessError::IsDirectory
                    ))
                    || (requires_subtree && matches!(error, FileAccessError::IsDirectory))
                {
                    out.reason = Some(ScanReason::Unstable);
                    out.locator = None;
                    return;
                }
                access_failure(out, error, false);
                // Classification may have succeeded before the target changed.
                // A failed pin cannot turn required traversal into a leaf-only
                // miss/rejection. Content failures after pinning remain separate.
                if requires_subtree
                    && matches!(out.reason, Some(ScanReason::Missing | ScanReason::LeafLoop))
                {
                    out.reason = Some(ScanReason::Coverage);
                }
                if pass == 1
                    && matches!(
                        out.reason,
                        Some(ScanReason::Missing | ScanReason::LeafLoop | ScanReason::Unsupported)
                    )
                {
                    out.reason = Some(ScanReason::Unstable);
                    out.locator = None;
                }
                return;
            }
        };
        let current = pin.locator();
        if prior.is_some_and(|previous| previous != current) {
            out.reason = Some(ScanReason::Unstable);
            out.locator = None;
            return;
        }
        out.locator = Some(current);
        if !context.policy_available {
            out.reason = Some(ScanReason::Policy);
            return;
        }
        match policy(
            &pin,
            context.matcher,
            context.roots,
            context.config.checkpoint_namespace,
            &mut scratch.target,
            &mut scratch.native,
            cancelled,
        ) {
            Ok((true, len)) => {
                if pass == 0 {
                    first_target_len = len;
                    scratch.first_target[..len].copy_from_slice(&scratch.target[..len]);
                } else if len != first_target_len
                    || scratch.first_target[..len] != scratch.target[..len]
                {
                    out.reason = Some(ScanReason::Unstable);
                    return;
                }
            }
            Ok((false, _)) => {
                out.reason = Some(ScanReason::Excluded);
                return;
            }
            Err(error) => {
                out.reason = Some(if error.raw_os_error() == Some(libc::ECANCELED) {
                    ScanReason::Cancelled
                } else {
                    ScanReason::Policy
                });
                out.errno = error.raw_os_error();
                return;
            }
        }
        if pass == 0 && request == (ProbeRequest::PresenceOnly { expected: current }) {
            out.reason = Some(ScanReason::PresenceOnly);
            return;
        }
        let source = match pin.into_source(&mut *cancelled) {
            Ok(source) => source,
            Err(error) => {
                access_failure(out, error, true);
                return;
            }
        };
        let opened_size = source.opened_metadata().len();
        let buffer = if pass == 0 {
            &mut scratch.first
        } else {
            &mut scratch.second
        };
        let len = match fingerprint(
            &source,
            context.config.limits.ignored_header_bytes,
            buffer,
            cancelled,
        ) {
            Ok(len) => len,
            Err(error) => {
                access_failure(out, error, true);
                return;
            }
        };
        let metadata = match source.metadata(&mut *cancelled) {
            Ok(metadata) => metadata,
            Err(error) => {
                access_failure(out, error, true);
                return;
            }
        };
        let size = metadata.len();
        let modified = metadata.modified().ok();
        if size < opened_size
            || (len != 0 && size < context.config.limits.ignored_header_bytes + len as u64)
        {
            out.reason = Some(ScanReason::Unstable);
            return;
        }
        // Source closes here before the next path-only pin is acquired.
        drop(source);
        if pass == 0 {
            first_len = len;
            prior = Some(current);
            first_size = size;
        } else {
            if size < first_size
                || len < first_len
                || scratch.first[..first_len] != scratch.second[..first_len]
            {
                out.reason = Some(ScanReason::Unstable);
                return;
            }
            out.size = Some(size);
            out.modified = modified;
            out.fingerprint.extend_from_slice(&scratch.second[..len]);
            out.reason = None;
        }
    }
}
