// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Resumable association with no source handles or borrowed checkpoint state.
use super::*;
use std::path::PathBuf;

/// Maximum record visits in one association turn, including inactive records.
pub const RECORDS_PER_STEP: usize = 64;
/// Maximum prefix bytes compared in one association turn.
pub const BYTES_PER_STEP: usize = 64 * 1024;

/// Work performed by one step. Limits are work bounds, not wall-clock promises.
#[derive(Debug, Default)]
pub struct AssociationWork {
    /// Record visits, including a partially compared record resumed this turn.
    pub records: usize,
    /// Prefix bytes compared, charged even when comparison exits early.
    pub bytes: usize,
    /// A result is available for finish_prepare; source evidence still needs revalidation.
    pub complete: bool,
}

/// One bounded candidate awaiting association. Drop on the worker to release its lease.
/// Counts against the same outstanding-group limit as PreparedAdmission.
#[derive(Debug)]
pub struct AssociationScan {
    path: PathBuf,
    advisory_path: cp::AdvisoryPath,
    size: u64,
    prefix: Vec<u8>,
    lease: LeaseGuard,
    generation: u64,
    revision: u64,
    recovery_revision: u64,
    record: usize,
    compared: usize,
    result: Option<Context>,
}

impl AssociationScan {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        path: &Path,
        size: u64,
        prefix: &[u8],
        lease: LeaseGuard,
        generation: u64,
        revision: u64,
        recovery_revision: u64,
    ) -> Result<Self, IdentityError> {
        Ok(Self {
            path: path.to_owned(),
            advisory_path: cp::AdvisoryPath::from_unix_bytes(path.as_os_str().as_bytes())?,
            size,
            prefix: prefix.to_vec(),
            lease,
            generation,
            revision,
            recovery_revision,
            record: 0,
            compared: 0,
            result: None,
        })
    }

    /// Whether association finished; no reader or durable mutation is authorized yet.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.result.is_some()
    }

    /// Complete candidate path for the policy-aware reopener supplied to finish_prepare.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl IdentityOwner {
    fn validate_scan(&self, scan: &AssociationScan) -> Result<(), IdentityError> {
        self.running()?;
        if !self.leases.owns(&scan.lease)
            || self.identity_revision != scan.revision
            || self.recovery_revision != scan.recovery_revision
            || self.store.state()?.generation() != scan.generation
        {
            return Err(IdentityError::StalePlan);
        }
        Ok(())
    }

    /// Compare at most 64 records and 64 KiB of prefix bytes, then return.
    /// Service checkpoint/control/read work between calls; do not loop to completion
    /// in a production worker turn. Ordinary progress may run between steps. Any
    /// identity mutation, finalization or generation change invalidates the scan.
    /// On error discard the scan on the worker and freshly prepare if appropriate.
    pub fn advance_association(
        &self,
        scan: &mut AssociationScan,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<AssociationWork, IdentityError> {
        self.validate_scan(scan)?;
        evidence::check_cancelled(&mut cancelled)?;
        let state = self.store.state()?;
        let mut work = AssociationWork::default();
        while scan.result.is_none()
            && work.records < RECORDS_PER_STEP
            && work.bytes < BYTES_PER_STEP
        {
            evidence::check_cancelled(&mut cancelled)?;
            let Some(record) = state.record_at(scan.record) else {
                scan.result = Some(Context::Unrelated);
                break;
            };
            work.records += 1;
            if record.lifecycle_state == cp::LifecycleState::Active {
                let path = &scan.advisory_path;
                if record.advisory_path.kind() == path.kind()
                    && record.advisory_path.full_path_len() == path.full_path_len()
                    && record.advisory_path.full_path_digest() == path.full_path_digest()
                {
                    scan.result = Some(Context::Associated);
                    break;
                }
                let common = scan.prefix.len().min(record.fingerprint.len());
                if record.ignored_header_bytes == self.ignored_header && common != 0 {
                    let end = common.min(scan.compared + BYTES_PER_STEP - work.bytes);
                    work.bytes += end - scan.compared;
                    if scan.prefix[scan.compared..end] == record.fingerprint[scan.compared..end] {
                        if end == common {
                            scan.result = Some(Context::Associated);
                            break;
                        }
                        scan.compared = end;
                        break;
                    }
                }
            }
            scan.record += 1;
            scan.compared = 0;
        }
        if scan.result.is_none() && scan.record == state.records().len() {
            scan.result = Some(Context::Unrelated);
        }
        work.complete = scan.is_complete();
        Ok(work)
    }

    /// Reopen under current policy and revalidate the scanned prefix before using
    /// the result. Changed evidence defers and releases the lease; prepare again.
    /// No source handle or probe reservation survives a yielded scan step.
    pub fn finish_prepare(
        &mut self,
        scan: AssociationScan,
        budget: &ProbeBudget,
        now: SystemTime,
        open: impl FnOnce() -> Result<SourceFile, FileAccessError>,
        cancelled: impl FnMut() -> bool,
    ) -> Result<Admission, IdentityError> {
        self.validate_scan(&scan)?;
        let context = scan.result.ok_or(IdentityError::InvalidCandidate)?;
        let mut cancelled = cancelled;
        evidence::check_cancelled(&mut cancelled)?;
        let Some(reservation) = budget.try_reserve() else {
            // Keep completed CPU work and its lease when discovery is probing.
            return Ok(Admission::Scanning(scan));
        };
        self.prepare_impl(
            Candidate {
                path: &scan.path,
                locator: scan.lease.locator(),
                size: scan.size,
                fingerprint: &scan.prefix,
                context,
            },
            budget,
            now,
            open,
            cancelled,
            Some((scan.lease, reservation)),
        )
    }
}
