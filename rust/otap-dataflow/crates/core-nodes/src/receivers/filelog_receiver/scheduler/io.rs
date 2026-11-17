// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::source::{FileAccessError, FileLocator, SourceFile};
use super::*;

/// Source operations never decide identity, admission, epochs or loss policy.
/// Each method performs one bounded operation and checks cancellation. `open`
/// must apply the existing pin-bound path/target/namespace policy before read-open.
/// The caller holds a shared probe reservation and a reserved resident slot.
pub(crate) trait ReaderIo {
    type Handle;
    fn open(
        &mut self,
        key: ReaderKey,
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<Self::Handle, AccessFailure>;
    fn metadata(
        &mut self,
        handle: &Self::Handle,
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<(FileLocator, u64), AccessFailure>;
    fn read(
        &mut self,
        handle: &Self::Handle,
        offset: u64,
        bytes: &mut [u8],
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<usize, AccessFailure>;
}

/// Production I/O adapter. The opener is supplied by composition after it can
/// consume ReaderSeed; this module supplies no weaker default pathname opener.
pub(crate) struct LinuxIo<O>(pub O);

impl<O> ReaderIo for LinuxIo<O>
where
    O: FnMut(ReaderKey, &mut dyn FnMut() -> bool) -> Result<SourceFile, AccessFailure>,
{
    type Handle = SourceFile;
    fn open(
        &mut self,
        key: ReaderKey,
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<Self::Handle, AccessFailure> {
        (self.0)(key, cancel)
    }
    fn metadata(
        &mut self,
        handle: &SourceFile,
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<(FileLocator, u64), AccessFailure> {
        let m = handle.metadata(cancel).map_err(AccessFailure::from)?;
        Ok((FileLocator::from_metadata(&m), m.len()))
    }
    fn read(
        &mut self,
        handle: &SourceFile,
        offset: u64,
        bytes: &mut [u8],
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<usize, AccessFailure> {
        handle
            .read_at(offset, bytes, cancel)
            .map_err(AccessFailure::from)
    }
}

#[derive(Debug)]
pub(crate) enum AccessFailure {
    Cancelled,
    Environmental(std::io::Error),
    Transition(Transition),
    Terminal(FileAccessError),
}

impl From<FileAccessError> for AccessFailure {
    fn from(e: FileAccessError) -> Self {
        match e {
            FileAccessError::Cancelled => Self::Cancelled,
            FileAccessError::Io(e) | FileAccessError::PinnedReopen(e) => Self::Environmental(e),
            FileAccessError::LocatorChanged { .. } | FileAccessError::ReopenedMismatch { .. } => {
                Self::Transition(Transition::Replaced)
            }
            FileAccessError::NotRegular
            | FileAccessError::IsDirectory
            | FileAccessError::UnsupportedFilesystem { .. } => {
                Self::Transition(Transition::Ineligible)
            }
            e => Self::Terminal(e),
        }
    }
}

/// Existing identity evidence supplied by admission; never generated here.
pub(crate) struct Evidence {
    pub locator: FileLocator,
    pub fingerprint: Vec<u8>,
    pub fingerprint_offset: u64,
    pub applied_offset: u64,
    pub resume_end: u64,
    pub applied_guard: CommittedFrontierGuard,
}

pub(crate) struct Validation {
    phase: u8,
    cursor: usize,
    guard: [u8; 64],
}

impl Default for Validation {
    fn default() -> Self {
        Self {
            phase: 0,
            cursor: 0,
            guard: [0; 64],
        }
    }
}

impl Validation {
    /// One metadata/read operation, charged to the same turn budgets as content.
    pub(crate) fn step<I: ReaderIo>(
        &mut self,
        io: &mut I,
        handle: &I::Handle,
        evidence: &Evidence,
        provisional: &Window,
        scratch: &mut [u8],
        cancel: &mut dyn FnMut() -> bool,
    ) -> Result<(bool, usize), AccessFailure> {
        if self.phase == 0 {
            let (locator, size) = io.metadata(handle, cancel)?;
            if locator != evidence.locator {
                return Err(AccessFailure::Transition(Transition::Replaced));
            }
            if size
                < provisional
                    .end
                    .max(evidence.applied_offset)
                    .max(evidence.resume_end)
                    .max(if evidence.fingerprint.is_empty() {
                        0
                    } else {
                        evidence.fingerprint_offset + evidence.fingerprint.len() as u64
                    })
            {
                return Err(AccessFailure::Transition(Transition::Shrunk));
            }
            self.phase = 1;
            return Ok((false, 0));
        }
        let (start, len) = match self.phase {
            1 => (evidence.fingerprint_offset, evidence.fingerprint.len()),
            2 => (
                evidence.applied_offset.saturating_sub(64),
                evidence.applied_offset.min(64) as usize,
            ),
            3 => (
                provisional.end.saturating_sub(64),
                provisional.bytes().len(),
            ),
            _ => return Ok((true, 0)),
        };
        let n = (len - self.cursor).min(scratch.len());
        let read = if n == 0 {
            0
        } else {
            io.read(
                handle,
                start + self.cursor as u64,
                &mut scratch[..n],
                cancel,
            )?
        };
        if read == 0 && n != 0 {
            return Err(AccessFailure::Transition(Transition::Shrunk));
        }
        if read > n {
            return Err(AccessFailure::Terminal(FileAccessError::InvalidRange));
        }
        let matches = match self.phase {
            1 => scratch[..read] == evidence.fingerprint[self.cursor..self.cursor + read],
            2 => {
                self.guard[self.cursor..self.cursor + read].copy_from_slice(&scratch[..read]);
                true
            }
            3 => scratch[..read] == provisional.bytes()[self.cursor..self.cursor + read],
            _ => unreachable!(),
        };
        if !matches {
            return Err(AccessFailure::Transition(Transition::Changed));
        }
        self.cursor += read;
        if self.cursor == len {
            if self.phase == 2
                && CommittedFrontierGuard::compute(evidence.applied_offset, &self.guard[..len])
                    .expect("bounded guard")
                    != evidence.applied_guard
            {
                return Err(AccessFailure::Transition(Transition::Changed));
            }
            self.cursor = 0;
            self.phase += 1;
        }
        Ok((self.phase == 4, read))
    }
}
