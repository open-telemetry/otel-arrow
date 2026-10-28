// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded Linux identity admission. See `../docs/identity.md` for integration.
//!
//! Prepare acquires a lease and validates evidence. Discovery then preflights its
//! transition, and commit establishes durability before publishing a reader.
//! Both phases run on the read/checkpoint worker, never on the async pipeline core.

mod checkpoint;
pub mod decision;
mod evidence;
pub mod lease;
#[cfg(test)]
mod tests;

use super::{
    config::ValidatedInputs,
    discovery::ProbeBudget,
    source::{FileAccessError, FileLocator, SourceFile},
};
use decision::{Anchor, Context, Decision, Existing, Policy, classify, source_age};
use lease::{LeaseError, LeaseGuard, LeaseOwner};
use otel_arrow_dfe_filelog_checkpoint::{
    self as cp,
    store::{CheckpointStore, Durability, StoreError},
};
use std::os::unix::ffi::OsStrExt;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Candidate borrowed from bounded discovery storage. Carries no descriptor.
#[derive(Debug)]
pub struct Candidate<'a> {
    /// Complete native path, never a truncated checkpoint suffix.
    pub path: &'a Path,
    /// Locator validated by discovery's stable observations.
    pub locator: FileLocator,
    /// Size of the last stable observation.
    pub size: u64,
    /// Raw prefix beginning at configured ignored-header offset.
    pub fingerprint: &'a [u8],
    /// Historical replacement/association information, not continuity authority.
    pub context: Context,
}

/// Admission failure. Preserve source/store errors for lifecycle classification.
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    /// Process-wide conflict, capacity or registry integrity failure.
    #[error(transparent)]
    Lease(#[from] LeaseError),
    /// Source failure; scheduler owns environmental retries.
    #[error(transparent)]
    Source(#[from] FileAccessError),
    /// Durable operation failed. The owner remains stopped until teardown.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// This append did not write any bytes. Inspect the cause: capacity/encoding
    /// rejections can be corrected, but compaction and authority failures stop the owner.
    #[error("checkpoint append rejected before writing: {0}")]
    AppendRejected(#[source] StoreError),
    /// The process-local identity revision must never wrap and revive old plans.
    #[error("filelog identity revision exhausted")]
    RevisionExhausted,
    /// A retained append may be resolved through this owner; never resubmit it.
    #[error("checkpoint append pending resolution: {0}")]
    PendingWrite(#[source] StoreError),
    /// Only pending resolution is allowed until the retained attempt settles.
    #[error("resolve the pending checkpoint write before doing more work")]
    PendingResolutionRequired,
    /// There is no pending result to publish again.
    #[error("no pending checkpoint write")]
    NoPendingWrite,
    /// The requested admission group exceeds the configured bounded batch size.
    #[error("identity admission batch exceeds its limit")]
    BatchLimit,
    /// The preflighted group requests more durable slots than remain available.
    #[error("identity admission group exceeds remaining tracked capacity")]
    BatchCapacity,
    /// Internal evidence/operation failed checkpoint validation.
    #[error(transparent)]
    Encode(#[from] cp::EncodeError),
    /// The plan belongs to another owner or authoritative state changed.
    #[error("stale filelog admission plan")]
    StalePlan,
    /// A prior store error stopped this owner; no subsequent admission is allowed.
    #[error("filelog identity owner stopped after checkpoint failure")]
    Stopped,
    /// Input violates configured evidence bounds.
    #[error("invalid filelog candidate evidence")]
    InvalidCandidate,
    /// Store and admission settings must describe one namespace, profile and population.
    #[error("filelog identity configuration does not match its checkpoint store or lease limit")]
    ConfigurationMismatch,
    /// Randomness unavailable or bounded collision attempts exhausted.
    #[error("filelog file ID generation failed")]
    Randomness,
}

/// Prepare result. Only a committed Ready plan may become a logical reader.
// A bounded worker return, not an unbounded collection. Keep the prepared
// value inline instead of allocating a box for every admission.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Admission {
    /// Fresh evidence and a held lease; discovery preflight must precede commit.
    Ready(PreparedAdmission),
    /// Source changed since discovery or temporary probe capacity is busy.
    Deferred,
    /// Unrelated old file; no durable state or lease retained.
    Ignored,
    /// Checkpoint quarantine for the offered locator, not fresh path validation.
    Quarantined(cp::FileId),
    /// Checkpoint profile status for the offered locator, not fresh path validation.
    ProfileBlocked(cp::FileId),
    /// New identity needs a durable slot; recovery/supersede need no extra slot.
    NeedsCapacity,
}

/// Reader metadata and the logical lease. Does not retain an open source handle.
#[derive(Debug)]
pub struct ReaderSeed {
    /// Original admission decision, retained through pending-write recovery.
    pub decision: Decision,
    /// Durable logical key.
    pub file_id: cp::FileId,
    /// Epoch protecting progress against stream resets.
    pub epoch: u32,
    /// Validated starting offset from authoritative applied checkpoint state.
    pub offset: u64,
    /// Framing continuation copied from the recovered record.
    pub resume: cp::FramingResume,
    /// Raw trailing bytes seed subsequent guard construction.
    pub frontier: cp::CommittedFrontierWindow,
    /// Fresh bounded raw prefix for reopen validation and deferred growth updates.
    pub fingerprint: Vec<u8>,
    /// Own until logical reader retirement, including descriptor eviction.
    pub lease: LeaseGuard,
}

// Identity/evidence changes bump the namespace revision. This compact stamp also
// rejects progress changes to the exact record without copying its fingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RecordStamp {
    id: cp::FileId,
    locator: cp::Locator,
    lifecycle: cp::LifecycleState,
    epoch: u32,
    offset: u64,
    guard: cp::CommittedFrontierGuard,
    resume: cp::FramingResume,
}
impl From<&cp::SnapshotRecord> for RecordStamp {
    fn from(record: &cp::SnapshotRecord) -> Self {
        Self {
            id: record.file_id,
            locator: record.locator,
            lifecycle: record.lifecycle_state,
            epoch: record.file_epoch,
            offset: record.committed_offset,
            guard: record.committed_frontier_guard,
            resume: record.framing_resume,
        }
    }
}

/// Prepared bounded mutation. Dropping it before commit releases its lease.
#[derive(Debug)]
pub struct PreparedAdmission {
    generation: u64,
    identity_revision: u64,
    touched: Option<RecordStamp>,
    seed: ReaderSeed,
    operations: Vec<cp::Operation>,
    mtime_unavailable: bool,
}
impl PreparedAdmission {
    /// Stable proposed key for discovery preflight; not yet durable.
    #[must_use]
    pub fn file_id(&self) -> cp::FileId {
        self.seed.file_id
    }
    /// Selected policy, including superseded identity and explicit loss/quarantine.
    #[must_use]
    pub fn decision(&self) -> Decision {
        self.seed.decision
    }
    /// Reports inability to apply the age filter without silently excluding data.
    #[must_use]
    pub fn mtime_unavailable(&self) -> bool {
        self.mtime_unavailable
    }
}

/// Capacity-aware group formation, performed before discovery preflight.
#[derive(Debug)]
pub struct AdmissionSelection {
    /// Exactly these plans may be submitted together for discovery preflight.
    pub ready: Vec<PreparedAdmission>,
    /// Report capacity feedback for these candidates, then drop the plans on the
    /// worker. They must be freshly prepared when capacity becomes available.
    pub needs_capacity: Vec<PreparedAdmission>,
}

/// Settled durable result. Quarantine never hands a lease to a reader.
#[derive(Debug)]
pub enum CommittedAdmission {
    /// Registration/recovery is complete; scheduling must validate on reopen.
    Reader(ReaderSeed),
    /// Newly registered mismatch was quarantined atomically, even if cleanup failed.
    Quarantined {
        /// Original mismatch decision, including the superseded identity.
        decision: Decision,
        /// Durable quarantine identity, always reportable.
        file_id: cp::FileId,
        /// A cleanup failure has stopped the owner; never treat it as uncommitted.
        release_error: Option<LeaseError>,
    },
}

impl CommittedAdmission {
    /// Original decision for health reporting, on direct or recovered completion.
    #[must_use]
    pub fn decision(&self) -> Decision {
        match self {
            Self::Reader(seed) => seed.decision,
            Self::Quarantined { decision, .. } => *decision,
        }
    }
}

/// Required progress synchronization and optional maintenance are separate outcomes.
#[derive(Debug)]
#[must_use]
pub struct SyncOutcome {
    /// Sequence certified by the required barrier, even if maintenance later fails.
    pub synced_sequence: u64,
    /// Optional growth result. A pending/terminal error still requires resolution
    /// or shutdown, but does not undo the already-confirmed progress barrier.
    pub growth: Result<Option<cp::store::AppendReceipt>, IdentityError>,
}

/// Exactly one publication following successful pending-write resolution.
#[derive(Debug)]
pub struct ResolvedWrite {
    /// The original transaction, certified by the store's resolution protocol.
    pub receipt: cp::store::AppendReceipt,
    /// Empty for ordinary checkpoint/growth writes; otherwise the original readers/quarantines.
    pub admissions: Vec<CommittedAdmission>,
}

struct Completion {
    seed: ReaderSeed,
    quarantine: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum WritePurpose {
    Required,
    FingerprintGrowth,
}
struct PendingWrite {
    next_revision: u64,
    admissions: Vec<Completion>,
    purpose: WritePurpose,
}

/// One worker-owned admission executor, borrowing no discovery or runtime state.
///
/// All checkpoint mutation for this namespace is serialized through this owner.
/// Drop after worker access has ceased. Store drops before a failed-write lease.
pub struct IdentityOwner {
    store: CheckpointStore,
    leases: LeaseOwner,
    policy: Policy,
    profile: [u8; 32],
    ignored_header: u32,
    fingerprint_bytes: usize,
    tracked_limit: usize,
    scratch: Vec<u8>,
    stopped: bool,
    identity_revision: u64,
    pending: Option<PendingWrite>,
    failures: u64,
    failure_limit: u64,
    admission_limit: usize,
    growth_limit: usize,
    growth: Vec<cp::UpdateFingerprint>,
}
impl IdentityOwner {
    fn batch_capacity(inputs: &ValidatedInputs) -> usize {
        (inputs.settings().limits.max_open_files as usize)
            .min(usize::from(cp::WAL_MAX_NON_PROGRESS_OPS_PER_TX) / 3)
            .min((cp::WAL_MAX_TX_BODY_BYTES / (3 * (cp::MAX_OPERATION_PAYLOAD_BYTES + 8))) as usize)
    }

    /// Returns capacity for runtime readers plus one speculative admission group.
    ///
    /// Construct the lease owner with this capacity and keep at most
    /// `admission_limit()` prepared plans outstanding. Temporary new-file plans
    /// then cannot exhaust leases needed by existing identity recovery/replacement.
    pub fn required_lease_capacity(inputs: &ValidatedInputs) -> Result<usize, IdentityError> {
        (inputs.settings().limits.max_tracked_files as usize)
            .checked_add(Self::batch_capacity(inputs))
            .ok_or(IdentityError::ConfigurationMismatch)
    }

    /// Store and validated inputs must describe the same configured namespace/profile.
    pub fn new(
        store: CheckpointStore,
        leases: LeaseOwner,
        inputs: &ValidatedInputs,
    ) -> Result<Self, IdentityError> {
        let config = inputs.settings();
        let replay = inputs
            .store_config()
            .map_err(|_| IdentityError::ConfigurationMismatch)?
            .replay;
        if !store
            .state()?
            .matches_configuration(inputs.checkpoint_id(), replay)
            || leases.limit() != Self::required_lease_capacity(inputs)?
        {
            return Err(IdentityError::ConfigurationMismatch);
        }
        Ok(Self {
            store,
            leases,
            policy: Policy {
                start_at: config.start_at,
                mismatch: config.identity.on_recovery_mismatch,
                ignore_older_than: config.ignore_older_than,
            },
            profile: inputs.profile().digest()?,
            ignored_header: config.identity.ignored_header_bytes as u32,
            fingerprint_bytes: config.identity.fingerprint_bytes as usize,
            tracked_limit: config.limits.max_tracked_files as usize,
            scratch: vec![0; config.identity.fingerprint_bytes as usize],
            stopped: false,
            identity_revision: 0,
            pending: None,
            failures: 0,
            failure_limit: config.checkpoint.max_consecutive_failures,
            admission_limit: Self::batch_capacity(inputs),
            growth_limit: (config.limits.max_open_files as usize)
                .min(usize::from(cp::WAL_MAX_NON_PROGRESS_OPS_PER_TX))
                .min(
                    (cp::WAL_MAX_TX_BODY_BYTES
                        / (cp::MAX_VALID_UPDATE_FINGERPRINT_PAYLOAD_BYTES + 8))
                        as usize,
                ),
            growth: Vec::new(),
        })
    }

    /// Authoritative immutable checkpoint state, unavailable after owner failure.
    pub fn state(&self) -> Result<&cp::ReplayState, IdentityError> {
        self.running()?;
        Ok(self.store.state()?)
    }

    /// Reusable evidence-buffer capacity, excluding the authoritative store and leases.
    #[must_use]
    pub fn scratch_bytes(&self) -> usize {
        self.scratch.capacity()
    }

    fn running(&self) -> Result<(), IdentityError> {
        if self.stopped {
            Err(IdentityError::Stopped)
        } else if self.pending.is_some() {
            Err(IdentityError::PendingResolutionRequired)
        } else {
            Ok(())
        }
    }

    /// Prepare against fresh handle evidence after acquiring the process lease.
    ///
    /// `open` MUST enforce current selected-link, resolved-target and checkpoint
    /// namespace exclusion policy before returning the source. It is invoked only
    /// after acquiring the shared two-descriptor budget. The caller must not retain
    /// other transient probes. No handle escapes this method.
    ///
    /// The caller supplies one wall-clock observation; callbacks should observe
    /// cancellation during policy work. Failure/defer creates no checkpoint state.
    pub fn prepare(
        &mut self,
        candidate: Candidate<'_>,
        budget: &ProbeBudget,
        now: SystemTime,
        open: impl FnOnce() -> Result<SourceFile, FileAccessError>,
        cancelled: impl FnMut() -> bool,
    ) -> Result<Admission, IdentityError> {
        self.prepare_with_source(candidate, budget, now, open, cancelled)
    }

    fn prepare_with_source<S: evidence::EvidenceSource>(
        &mut self,
        candidate: Candidate<'_>,
        budget: &ProbeBudget,
        now: SystemTime,
        open: impl FnOnce() -> Result<S, FileAccessError>,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Admission, IdentityError> {
        self.running()?;
        if candidate.path.as_os_str().len() > 65536
            || !candidate.path.is_absolute()
            || candidate.path.as_os_str().as_bytes().contains(&0)
            || candidate.fingerprint.len() > self.fingerprint_bytes
            || candidate
                .size
                .saturating_sub(u64::from(self.ignored_header))
                < candidate.fingerprint.len() as u64
        {
            return Err(IdentityError::InvalidCandidate);
        }
        evidence::check_cancelled(&mut cancelled)?;
        let lease = self.leases.acquire(candidate.locator)?;
        let locator = evidence::locator(candidate.locator);
        let state = self.store.state()?;
        let exact = state.get_by_locator(&locator);
        if let Some(record) = exact
            && let Some(blocked) = decision::blocked(
                record,
                state.profile_compatible(&record.file_id) == Some(true),
            )
        {
            return Ok(match blocked {
                decision::Blocked::Quarantined(id) => Admission::Quarantined(id),
                decision::Blocked::Profile(id) => Admission::ProfileBlocked(id),
            });
        }
        // With age filtering enabled, resolve association and fresh handle mtime
        // before reporting capacity: an unrelated old file needs no durable slot.
        if exact.is_none()
            && self.policy.ignore_older_than.is_zero()
            && state.records().len() >= self.tracked_limit
        {
            return Ok(Admission::NeedsCapacity);
        }
        let Some(_reservation) = budget.try_reserve() else {
            return Ok(Admission::Deferred);
        };
        let source = match open() {
            Ok(source) => source,
            Err(FileAccessError::LocatorChanged { .. }) => return Ok(Admission::Deferred),
            Err(FileAccessError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Admission::Deferred);
            }
            Err(error) => return Err(error.into()),
        };
        if source.locator() != candidate.locator {
            return Ok(Admission::Deferred);
        }
        let metadata = source.metadata(&mut cancelled)?;
        if metadata.len() < candidate.size {
            return Ok(Admission::Deferred);
        }
        let count = evidence::prefix(
            &source,
            self.ignored_header,
            &mut self.scratch,
            &mut cancelled,
        )?;
        let prefix = &self.scratch[..count];
        if count
            < metadata
                .len()
                .saturating_sub(u64::from(self.ignored_header))
                .min(self.fingerprint_bytes as u64) as usize
            || !prefix.starts_with(candidate.fingerprint)
        {
            return Ok(Admission::Deferred);
        }
        let mut frontier = cp::CommittedFrontierWindow::empty();
        let continuous = if let Some(record) = exact {
            if metadata.len() < record.committed_offset || !prefix.starts_with(&record.fingerprint)
            {
                false
            } else {
                match evidence::window(&source, record.committed_offset, &mut cancelled)? {
                    Some(window) => {
                        let matches = window.guard()? == record.committed_frontier_guard;
                        frontier = window;
                        matches
                    }
                    None => false,
                }
            }
        } else {
            false
        };
        let path = cp::AdvisoryPath::from_unix_bytes(candidate.path.as_os_str().as_bytes())?;
        let context = if exact.is_none() && candidate.context == Context::Unrelated {
            evidence::association(state, &path, prefix, self.ignored_header, &mut cancelled)?
        } else {
            candidate.context
        };
        let modified = metadata.modified().ok();
        let age = modified.and_then(|mtime| source_age(now, mtime));
        let decision = classify(
            self.policy,
            exact.map(|record| Existing {
                record,
                profile_compatible: state.profile_compatible(&record.file_id) == Some(true),
                continuous,
            }),
            context,
            age,
        );
        let (file_id, epoch, offset, resume, operations) = match decision {
            Decision::Resume(id) => {
                let record = exact.expect("resume requires exact record");
                // Prefix growth is optional evidence maintenance. Admission must
                // not add an fsync/WAL transaction for every growing recovered file.
                let operations = Vec::new();
                (
                    id,
                    record.file_epoch,
                    record.committed_offset,
                    record.framing_resume,
                    operations,
                )
            }
            Decision::Register {
                anchor, superseded, ..
            } => {
                if superseded.is_none() && state.records().len() >= self.tracked_limit {
                    return Ok(Admission::NeedsCapacity);
                }
                let id = evidence::new_id(state)?;
                let offset = if anchor == Anchor::End {
                    metadata.len()
                } else {
                    0
                };
                frontier = match evidence::window(&source, offset, &mut cancelled)? {
                    Some(window) => window,
                    None => return Ok(Admission::Deferred),
                };
                let timestamp = now
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .and_then(|d| u64::try_from(d.as_nanos()).ok())
                    .unwrap_or(0);
                let mut operations = Vec::with_capacity(3);
                if let Some(old) = superseded {
                    operations.push(cp::Operation::RemoveFile(cp::RemoveFile {
                        file_id: old,
                        expected_file_epoch: exact
                            .expect("supersede requires exact record")
                            .file_epoch,
                        expected_prior_state: cp::LifecycleState::Active,
                        removal_reason: 1,
                        removal_time_unix_nano: timestamp,
                        administrative: false,
                        namespace_id: None,
                        audit_reason: None,
                    }));
                }
                operations.push(cp::Operation::RegisterFile(cp::RegisterFile {
                    file_id: id,
                    file_epoch: 1,
                    committed_offset: offset,
                    committed_frontier_guard: frontier.guard()?,
                    fingerprint: prefix.to_vec(),
                    ignored_header_bytes: self.ignored_header,
                    locator,
                    framing_profile_version: cp::FRAMING_PROFILE_VERSION,
                    framing_profile_digest: self.profile,
                    framing_resume: cp::FramingResume::Clean,
                    last_seen_time_unix_nano: timestamp,
                    advisory_path: path,
                }));
                if anchor == Anchor::Quarantine {
                    operations.push(cp::Operation::QuarantineFile(cp::QuarantineFile {
                        file_id: id,
                        expected_file_epoch: 1,
                        reason_code: 3,
                        locator,
                        observed_size: metadata.len(),
                        quarantine_epoch: 1,
                        quarantine_time_unix_nano: timestamp,
                    }));
                }
                (id, 1, offset, cp::FramingResume::Clean, operations)
            }
            Decision::Ignored => return Ok(Admission::Ignored),
            Decision::Quarantined(id) => return Ok(Admission::Quarantined(id)),
            Decision::ProfileBlocked(id) => return Ok(Admission::ProfileBlocked(id)),
        };
        // Detect shrink while evidence was collected. Later reader reopen must still
        // validate continuity; sampled evidence cannot prove an immutable source.
        let final_size = source.metadata(&mut cancelled)?.len();
        if final_size < metadata.len()
            || (count != 0 && final_size < u64::from(self.ignored_header) + count as u64)
        {
            return Ok(Admission::Deferred);
        }
        evidence::check_cancelled(&mut cancelled)?;
        Ok(Admission::Ready(PreparedAdmission {
            generation: state.generation(),
            identity_revision: self.identity_revision,
            touched: exact.map(RecordStamp::from),
            seed: ReaderSeed {
                decision,
                file_id,
                epoch,
                offset,
                resume,
                frontier,
                fingerprint: prefix.to_vec(),
                lease,
            },
            operations,
            mtime_unavailable: age.is_none(),
        }))
    }
}
