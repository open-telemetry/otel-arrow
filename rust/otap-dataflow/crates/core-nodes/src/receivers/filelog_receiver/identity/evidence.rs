// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(super) use super::super::source::check_cancelled;
use super::*;
use rand::TryRng;
use std::{fs::Metadata, io::ErrorKind};

/// Private seam: production can only supply SourceFile; tests wrap a real handle.
pub(super) trait EvidenceSource {
    fn locator(&self) -> FileLocator;
    fn metadata(&self, cancelled: &mut impl FnMut() -> bool) -> Result<Metadata, FileAccessError>;
    fn read_at(
        &self,
        offset: u64,
        bytes: &mut [u8],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<usize, FileAccessError>;
}
impl EvidenceSource for SourceFile {
    fn locator(&self) -> FileLocator {
        self.locator()
    }
    fn metadata(&self, cancelled: &mut impl FnMut() -> bool) -> Result<Metadata, FileAccessError> {
        self.metadata(cancelled)
    }
    fn read_at(
        &self,
        offset: u64,
        bytes: &mut [u8],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<usize, FileAccessError> {
        self.read_at(offset, bytes, cancelled)
    }
}

pub(super) fn locator(locator: FileLocator) -> cp::Locator {
    cp::Locator::PosixDevIno {
        dev: locator.device,
        ino: locator.inode,
    }
}

// One bounded loop for prefix and frontier reads. Partial reads continue, EOF is
// explicit, and environmental errors preserve their original kind. A signal storm
// returns Interrupted after eight retries rather than monopolizing the worker.
fn read_fully(
    source: &impl EvidenceSource,
    offset: u64,
    buffer: &mut [u8],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<usize, FileAccessError> {
    let mut count = 0;
    let mut interrupted = 0;
    while count < buffer.len() {
        check_cancelled(cancelled)?;
        let position = offset
            .checked_add(count as u64)
            .ok_or(FileAccessError::InvalidRange)?;
        match source.read_at(position, &mut buffer[count..], cancelled) {
            Ok(0) => break,
            Ok(n) => count += n,
            Err(error)
                if error
                    .os_error()
                    .is_some_and(|e| e.kind() == ErrorKind::Interrupted)
                    && interrupted < 8 =>
            {
                interrupted += 1
            }
            Err(error) => return Err(error),
        }
    }
    Ok(count)
}

pub(super) fn prefix(
    source: &impl EvidenceSource,
    ignored: u32,
    buffer: &mut [u8],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<usize, FileAccessError> {
    read_fully(source, u64::from(ignored), buffer, cancelled)
}

pub(super) fn window(
    source: &impl EvidenceSource,
    offset: u64,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Option<cp::CommittedFrontierWindow>, FileAccessError> {
    let mut bytes = [0; 64];
    let len = offset.min(64) as usize;
    if read_fully(source, offset - len as u64, &mut bytes[..len], cancelled)? != len {
        return Ok(None);
    }
    Ok(Some(
        cp::CommittedFrontierWindow::new(offset, &bytes[..len]).expect("exact frontier length"),
    ))
}

pub(super) fn association(
    state: &cp::ReplayState,
    path: &cp::AdvisoryPath,
    prefix: &[u8],
    ignored_header: u32,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Context, FileAccessError> {
    for record in state.records() {
        check_cancelled(cancelled)?;
        if record.lifecycle_state != cp::LifecycleState::Active {
            continue;
        }
        let same_path = record.advisory_path.kind() == path.kind()
            && record.advisory_path.full_path_len() == path.full_path_len()
            && record.advisory_path.full_path_digest() == path.full_path_digest();
        // Association is conservative, not continuity: compare the common
        // nonempty raw prefix at the same source offset, including short files
        // and older evidence that has grown. Framing settings do not re-interpret
        // these raw bytes. No file ID, offset or epoch transfers across locators.
        let common = prefix.len().min(record.fingerprint.len());
        let same_prefix = record.ignored_header_bytes == ignored_header
            && common != 0
            && prefix[..common] == record.fingerprint[..common];
        if same_path || same_prefix {
            return Ok(Context::Associated);
        }
    }
    Ok(Context::Unrelated)
}

pub(super) fn new_id(state: &cp::ReplayState) -> Result<cp::FileId, IdentityError> {
    // OS randomness, never a path/content hash or a placement-derived key.
    // Bounded attempts also make a broken random provider fail closed.
    for _ in 0..8 {
        let mut bytes = [0; 16];
        rand::rngs::SysRng
            .try_fill_bytes(&mut bytes)
            .map_err(|_| IdentityError::Randomness)?;
        let id = cp::FileId::from_bytes(bytes);
        if state.get(&id).is_none() {
            return Ok(id);
        }
    }
    Err(IdentityError::Randomness)
}
