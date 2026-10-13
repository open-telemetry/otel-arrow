// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded in-memory replay. Filesystem recovery and source proof belong to callers.

use std::collections::{HashMap, HashSet};

use crate::primitives::quarantine_reason_reserved;
use crate::{
    CommittedFrontierGuard, DecodeError, EncodeError, FRAMING_PROFILE_VERSION, FileId,
    FramingProfileParams, FramingResume, LifecycleState, Locator, Operation, QuarantineEvidence,
    RegisterFile, ResetQuarantineAction, SnapshotRecord, Transaction, TransactionScan,
    UpdateProgress, decode_checkpoint_snapshot, namespace_digest, scan_next_transaction,
};

/// Admission bounds and the selected identity/framing profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayConfig {
    /// Maximum number of tracked records, including finalized and quarantined records.
    pub max_tracked_files: u32,
    /// Maximum encoded snapshot section size, excluding the container header.
    pub max_snapshot_bytes: u64,
    /// Configured identity evidence window, in `16..=65535` bytes.
    fingerprint_bytes: u16,
    /// Selected version-one framing/identity profile digest.
    framing_profile_digest: [u8; 32],
}

impl ReplayConfig {
    /// Derives the fingerprint window and compatibility digest from one validated
    /// canonical profile. These coupled fields cannot be overridden independently.
    pub fn new(
        max_tracked_files: u32,
        max_snapshot_bytes: u64,
        profile: &FramingProfileParams,
    ) -> Result<Self, EncodeError> {
        Ok(Self {
            max_tracked_files,
            max_snapshot_bytes,
            fingerprint_bytes: profile.fingerprint_bytes,
            framing_profile_digest: profile.digest()?,
        })
    }
}

/// Structural or state-dependent rejection during replay.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReplayError {
    /// Encoded bytes failed codec validation before application.
    #[error(transparent)]
    Decode(#[from] DecodeError),
    /// Caller configuration cannot describe a valid replay context.
    #[error("invalid replay configuration: {0}")]
    InvalidConfig(&'static str),
    /// Administrative namespace differs from the selected raw checkpoint ID.
    #[error("administrative checkpoint namespace mismatch for {file_id:?}")]
    NamespaceMismatch {
        /// Target key, including when no such record exists.
        file_id: FileId,
    },
    /// An operation or stored record violates the semantic contract.
    #[error("checkpoint record {file_id:?}: {reason}")]
    InvalidState {
        /// Affected durable key.
        file_id: FileId,
        /// Violated invariant, without source or path content.
        reason: &'static str,
    },
    /// A keep-failed operation attempted to mutate immutable state.
    #[error("keep_failed would change checkpoint record {file_id:?}")]
    KeepFailedStateChange {
        /// Affected durable key.
        file_id: FileId,
    },
    /// The staged table would exceed the configured record bound.
    #[error("checkpoint table exceeds the configured {max} record limit")]
    RecordLimit {
        /// Maximum admitted records.
        max: u32,
    },
    /// All representable sequence numbers in this WAL generation were applied.
    #[error("WAL sequence exhausted; a new generation is required")]
    SequenceExhausted,
}

/// Result of scanning and applying at most one encoded WAL transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayStep {
    /// No input was supplied. This does not establish physical EOF.
    Empty,
    /// All operations were applied atomically.
    Applied {
        /// Sequence just applied.
        sequence: u64,
        /// Exact input bytes consumed; advance the caller's cursor by this amount.
        consumed: usize,
    },
    /// More bytes are needed; no state or sequence changed.
    /// Only the filesystem store can prove a final torn tail and authorize repair.
    Incomplete {
        /// Supplied suffix length.
        bytes: usize,
        /// Full frame length when a validated header is available.
        total_len: Option<usize>,
    },
}

/// A validated snapshot plus successfully applied WAL transactions.
///
/// The table is private and exposed only through immutable references. Each
/// progress transaction validates all updates before changing fixed-size fields.
/// Other transactions stage touched records and locator claims, never the entire table.
/// All returned errors leave authoritative records and the sequence unchanged.
/// An error must abort recovery; it does not authorize skipping corrupt bytes.
/// This type establishes no filesystem durability, source continuity, Ack proof,
/// or runtime lease. Callers must establish those independently.
#[derive(Debug)]
pub struct ReplayState {
    namespace: String,
    config: ReplayConfig,
    generation: u64,
    last_sequence: u64,
    records: HashMap<FileId, SnapshotRecord>,
    live_locators: HashMap<Locator, FileId>,
}

impl ReplayState {
    /// Validates the checkpoint header/snapshot and returns the WAL start offset.
    ///
    /// Trailing WAL bytes are not processed. Release the prefix input and call
    /// [`Self::replay_next`] on bounded WAL slices, one transaction at a time.
    /// The store must validate the fixed header before allocating its snapshot
    /// input and enforce the aggregate recovery, WAL-byte, and transaction-count
    /// budgets. This layer enforces snapshot, record, and per-transaction bounds.
    /// Incompatible framing profiles are preserved for per-file failure handling.
    pub fn from_checkpoint(
        bytes: &[u8],
        checkpoint_id: &str,
        config: ReplayConfig,
    ) -> Result<(Self, usize), ReplayError> {
        if config.max_tracked_files == 0 || config.fingerprint_bytes < 16 {
            return Err(ReplayError::InvalidConfig(
                "nonzero record limit and fingerprint window of at least 16 bytes required",
            ));
        }
        let digest = namespace_digest(checkpoint_id)
            .map_err(|_| ReplayError::InvalidConfig("invalid checkpoint namespace ID"))?;
        let (snapshot, wal_offset) = decode_checkpoint_snapshot(
            bytes,
            &digest,
            config.max_tracked_files,
            config.max_snapshot_bytes,
        )?;
        let mut state = Self {
            namespace: checkpoint_id.to_owned(),
            config,
            generation: snapshot.generation,
            last_sequence: 0,
            records: HashMap::new(),
            live_locators: HashMap::new(),
        };
        for record in snapshot.records {
            validate_record(&record, &config)?;
            if let Some(locator) = live_locator(&record) {
                let _ = state.live_locators.insert(locator, record.file_id);
            }
            let _ = state.records.insert(record.file_id, record);
        }
        Ok((state, wal_offset))
    }

    /// Snapshot generation; WAL replay does not change it.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Last successfully applied WAL sequence, or zero for the snapshot alone.
    #[must_use]
    pub const fn last_sequence(&self) -> u64 {
        self.last_sequence
    }

    /// Returns one immutable stored record.
    #[must_use]
    pub fn get(&self, file_id: &FileId) -> Option<&SnapshotRecord> {
        self.records.get(file_id)
    }

    /// Iterates the authoritative records in unspecified order.
    #[must_use]
    pub fn records(&self) -> impl ExactSizeIterator<Item = &SnapshotRecord> {
        self.records.values()
    }

    /// Whether a stored record matches the selected identity/framing profile.
    /// `None` means absent. A false result must block source resumption for that
    /// file; successful historical replay does not authorize a profile migration.
    #[must_use]
    pub fn profile_compatible(&self, file_id: &FileId) -> Option<bool> {
        self.get(file_id)
            .map(|r| profile_compatible(r, &self.config))
    }

    /// Validates and atomically applies the next transaction from encoded bytes.
    ///
    /// Scanner errors, semantic failures, and incomplete input publish no prefix.
    /// Complete frames are applied even if their original writer had not synced
    /// them; the format contains no durable-sequence cap. Input following the
    /// first complete transaction is not examined by this call.
    pub fn replay_next(&mut self, bytes: &[u8]) -> Result<ReplayStep, ReplayError> {
        if bytes.is_empty() {
            return Ok(ReplayStep::Empty);
        }
        let expected = self
            .last_sequence
            .checked_add(1)
            .ok_or(ReplayError::SequenceExhausted)?;
        match scan_next_transaction(bytes, expected)? {
            Some(TransactionScan::Complete {
                transaction,
                consumed,
            }) => {
                self.apply(transaction)?;
                Ok(ReplayStep::Applied {
                    sequence: expected,
                    consumed,
                })
            }
            Some(TransactionScan::Incomplete { bytes, total_len }) => {
                Ok(ReplayStep::Incomplete { bytes, total_len })
            }
            // The scanner returns None only for empty input, handled above.
            None => Ok(ReplayStep::Empty),
        }
    }

    // Only the validating scanner creates inputs to this method. In particular,
    // class, duplicate progress IDs, operation sizes, and CRCs are already checked.
    fn apply(&mut self, transaction: Transaction) -> Result<(), ReplayError> {
        // The scanner guarantees a nonempty, homogeneous class and distinct IDs
        // for progress. No operation can affect another progress precondition.
        if matches!(
            transaction.operations.first(),
            Some(Operation::UpdateProgress(_))
        ) {
            self.apply_progress(&transaction.operations)?;
            self.last_sequence = transaction.sequence;
            return Ok(());
        }
        let mut staged = HashMap::<FileId, Option<SnapshotRecord>>::new();
        let mut claims = HashMap::<Locator, Option<FileId>>::new();
        let mut replacements = Vec::new();
        let mut registrations = HashSet::new();
        let mut count = self.records.len();
        for operation in transaction.operations {
            validate_namespace(&operation, &self.namespace)?;
            let id = operation.file_id();
            let record = staged
                .entry(id)
                .or_insert_with(|| self.records.get(&id).cloned());
            let old_locator = record.as_ref().and_then(live_locator);
            let was_present = record.is_some();
            if let Operation::RemoveFile(op) = &operation
                && !op.administrative
                && let Some(old) = record.as_ref()
            {
                replacements.push((id, old.locator));
            }
            if matches!(operation, Operation::RegisterFile(_)) && record.is_none() {
                let _ = registrations.insert(id);
            }
            transition(record, operation)?;
            if let Some(record) = record.as_ref() {
                validate_record(record, &self.config)?;
            }
            match (was_present, record.is_some()) {
                (false, true) => {
                    count = count.checked_add(1).ok_or(ReplayError::RecordLimit {
                        max: self.config.max_tracked_files,
                    })?;
                }
                (true, false) => count -= 1,
                _ => {}
            }
            let new_locator = record.as_ref().and_then(live_locator);
            if old_locator != new_locator {
                if let Some(locator) = old_locator {
                    let _ = claims.insert(locator, None);
                }
                if let Some(locator) = new_locator {
                    let owner = claims
                        .get(&locator)
                        .copied()
                        .unwrap_or_else(|| self.live_locators.get(&locator).copied());
                    if owner.is_some_and(|owner| owner != id) {
                        return Err(invalid(id, "locator already has a live owner"));
                    }
                    let _ = claims.insert(locator, Some(id));
                }
            }
        }
        // Admission constrains the resulting table. The bounded non-progress
        // overlay may temporarily hold registrations followed by removals.
        if count > self.config.max_tracked_files as usize {
            return Err(ReplayError::RecordLimit {
                max: self.config.max_tracked_files,
            });
        }
        for (removed, locator) in replacements {
            let replacement = claims.get(&locator).copied().flatten();
            if !replacement.is_some_and(|id| {
                id != removed && registrations.contains(&id) && !self.records.contains_key(&id)
            }) {
                return Err(invalid(
                    removed,
                    "non-administrative removal requires a new same-locator registration",
                ));
            }
        }
        // No fallible semantic work remains. Staging is proportional to touched
        // records; untouched records are neither scanned nor cloned.
        // Remove first so hash iteration order cannot temporarily exceed admission
        // when a full table replaces several identities in the same transaction.
        for (id, record) in &staged {
            if record.is_none() {
                let _ = self.records.remove(id);
            }
        }
        for (id, record) in staged {
            if let Some(record) = record {
                let _ = self.records.insert(id, record);
            }
        }
        for (locator, owner) in &claims {
            if owner.is_none() {
                let _ = self.live_locators.remove(locator);
            }
        }
        for (locator, owner) in claims {
            if let Some(owner) = owner {
                let _ = self.live_locators.insert(locator, owner);
            }
        }
        self.last_sequence = transaction.sequence;
        Ok(())
    }

    // Pure progress touches only fixed-size fields. Validate the complete set
    // against immutable records first, then commit without copying evidence.
    fn apply_progress(&mut self, operations: &[Operation]) -> Result<(), ReplayError> {
        for operation in operations {
            let Operation::UpdateProgress(op) = operation else {
                unreachable!("scanner validated progress-only transaction");
            };
            let record = self
                .records
                .get(&op.file_id)
                .ok_or_else(|| invalid(op.file_id, "target file is absent"))?;
            validate_progress(record, op)?;
        }
        for operation in operations {
            let Operation::UpdateProgress(op) = operation else {
                unreachable!("scanner validated progress-only transaction");
            };
            let record = self
                .records
                .get_mut(&op.file_id)
                .expect("validated target remains present during progress commit");
            apply_progress_fields(record, op);
            if op.finalize {
                let _ = self.live_locators.remove(&record.locator);
            }
        }
        Ok(())
    }
}

fn invalid(file_id: FileId, reason: &'static str) -> ReplayError {
    ReplayError::InvalidState { file_id, reason }
}

fn live_locator(record: &SnapshotRecord) -> Option<Locator> {
    (record.lifecycle_state != LifecycleState::RotatedFinalized).then_some(record.locator)
}

fn profile_compatible(record: &SnapshotRecord, config: &ReplayConfig) -> bool {
    record.framing_profile_version == FRAMING_PROFILE_VERSION
        && record.framing_profile_digest == config.framing_profile_digest
}

fn validate_record(record: &SnapshotRecord, config: &ReplayConfig) -> Result<(), ReplayError> {
    record
        .validate()
        .map_err(|reason| invalid(record.file_id, reason))?;
    if record
        .quarantine_evidence
        .as_ref()
        .is_some_and(|e| quarantine_reason_reserved(e.reason_code))
    {
        return Err(invalid(record.file_id, "reserved quarantine reason"));
    }
    if profile_compatible(record, config)
        && record.fingerprint.len() > usize::from(config.fingerprint_bytes)
    {
        return Err(invalid(
            record.file_id,
            "fingerprint exceeds the compatible configured window",
        ));
    }
    Ok(())
}

fn validate_namespace(operation: &Operation, namespace: &str) -> Result<(), ReplayError> {
    let carried = match operation {
        Operation::ResetQuarantinedFile(op) => Some(op.namespace_id.as_str()),
        Operation::RemoveFile(op) if op.administrative => op.namespace_id.as_deref(),
        _ => return Ok(()),
    };
    if carried != Some(namespace) {
        return Err(ReplayError::NamespaceMismatch {
            file_id: operation.file_id(),
        });
    }
    Ok(())
}

fn registered(op: RegisterFile) -> SnapshotRecord {
    SnapshotRecord {
        file_id: op.file_id,
        file_epoch: op.file_epoch,
        committed_offset: op.committed_offset,
        committed_frontier_guard: op.committed_frontier_guard,
        fingerprint: op.fingerprint,
        ignored_header_bytes: op.ignored_header_bytes,
        locator: op.locator,
        framing_profile_version: op.framing_profile_version,
        framing_profile_digest: op.framing_profile_digest,
        framing_resume: op.framing_resume,
        lifecycle_state: LifecycleState::Active,
        quarantine_evidence: None,
        last_seen_time_unix_nano: op.last_seen_time_unix_nano,
        advisory_path: op.advisory_path,
    }
}

fn transition(slot: &mut Option<SnapshotRecord>, operation: Operation) -> Result<(), ReplayError> {
    let id = operation.file_id();
    if let Operation::RegisterFile(op) = operation {
        if op.file_epoch != 1 || op.framing_resume != FramingResume::Clean {
            return Err(invalid(
                id,
                "registration requires epoch one and clean framing",
            ));
        }
        let new = registered(op);
        if let Some(old) = slot.as_ref() {
            if old != &new {
                return Err(invalid(id, "conflicting registration"));
            }
        } else {
            *slot = Some(new);
        }
        return Ok(());
    }
    // Namespace validation precedes this absent-target idempotency.
    if slot.is_none()
        && matches!(&operation, Operation::RemoveFile(op) if op.administrative && op.removal_reason != 0)
    {
        return Ok(());
    }
    let record = slot
        .as_mut()
        .ok_or_else(|| invalid(id, "target file is absent"))?;
    match operation {
        Operation::RegisterFile(_) => unreachable!("registration handled above"),
        Operation::UpdateProgress(op) => {
            validate_progress(record, &op)?;
            apply_progress_fields(record, &op);
        }
        Operation::ResetAfterTruncate(op) => {
            require(record, LifecycleState::Active, op.expected_active_epoch)?;
            if record.file_epoch.checked_add(1) != Some(op.resulting_epoch)
                || op.new_committed_offset != 0
                || op.new_framing_resume != FramingResume::Clean
                || op.reason_code != 1
            {
                return Err(invalid(id, "invalid truncate reset"));
            }
            record.file_epoch = op.resulting_epoch;
            record.committed_offset = 0;
            record.committed_frontier_guard = CommittedFrontierGuard::empty();
            record.framing_resume = FramingResume::Clean;
            record.fingerprint = op.new_fingerprint;
            record.last_seen_time_unix_nano = op.reset_time_unix_nano;
        }
        Operation::UpdateFingerprint(op) => {
            require(record, LifecycleState::Active, op.expected_file_epoch)?;
            if record.fingerprint != op.expected_fingerprint
                || op.new_fingerprint.len() <= op.expected_fingerprint.len()
                || !op.new_fingerprint.starts_with(&op.expected_fingerprint)
            {
                return Err(invalid(
                    id,
                    "fingerprint must strictly extend matching stored evidence",
                ));
            }
            record.fingerprint = op.new_fingerprint;
        }
        Operation::UpdateMetadata(op) => {
            if op.expected_prior_state == LifecycleState::RotatedFinalized {
                return Err(invalid(id, "finalized metadata is immutable"));
            }
            require(record, op.expected_prior_state, op.expected_file_epoch)?;
            if let Some(path) = op.advisory_path {
                record.advisory_path = path;
            }
            record.last_seen_time_unix_nano = op.last_seen_time_unix_nano;
        }
        Operation::QuarantineFile(op) => {
            if op.expected_file_epoch != record.file_epoch
                || op.quarantine_epoch != record.file_epoch
                || op.locator != record.locator
                || quarantine_reason_reserved(op.reason_code)
            {
                return Err(invalid(id, "invalid quarantine evidence"));
            }
            let evidence = QuarantineEvidence {
                reason_code: op.reason_code,
                observed_size: op.observed_size,
                quarantine_epoch: op.quarantine_epoch,
                quarantine_time_unix_nano: op.quarantine_time_unix_nano,
            };
            match record.lifecycle_state {
                LifecycleState::Active => {
                    record.lifecycle_state = LifecycleState::Quarantined;
                    record.quarantine_evidence = Some(evidence);
                }
                LifecycleState::Quarantined
                    if record.quarantine_evidence.as_ref() == Some(&evidence) => {}
                _ => return Err(invalid(id, "conflicting quarantine or finalized record")),
            }
        }
        Operation::ResetQuarantinedFile(op) => {
            require(
                record,
                LifecycleState::Quarantined,
                op.expected_quarantine_epoch,
            )?;
            if op.action == ResetQuarantineAction::KeepFailed {
                if op.resulting_epoch != record.file_epoch
                    || op.resulting_offset != record.committed_offset
                    || op.new_committed_frontier_guard != record.committed_frontier_guard
                    || op.new_framing_resume != record.framing_resume
                    || op.new_fingerprint != record.fingerprint
                {
                    return Err(ReplayError::KeepFailedStateChange { file_id: id });
                }
                return Ok(());
            }
            if record.file_epoch.checked_add(1) != Some(op.resulting_epoch)
                || op.new_framing_resume != FramingResume::Clean
                || (op.action == ResetQuarantineAction::ResetToBeginning
                    && op.resulting_offset != 0)
            {
                return Err(invalid(id, "invalid quarantine reset"));
            }
            record.lifecycle_state = LifecycleState::Active;
            record.quarantine_evidence = None;
            record.file_epoch = op.resulting_epoch;
            record.committed_offset = op.resulting_offset;
            record.committed_frontier_guard = op.new_committed_frontier_guard;
            record.framing_resume = op.new_framing_resume;
            record.fingerprint = op.new_fingerprint;
            record.last_seen_time_unix_nano = op.action_time_unix_nano;
        }
        Operation::RemoveFile(op) => {
            require(record, op.expected_prior_state, op.expected_file_epoch)?;
            if op.removal_reason == 0
                || (!op.administrative && record.lifecycle_state != LifecycleState::Active)
            {
                return Err(invalid(
                    id,
                    "invalid removal reason or non-administrative lifecycle",
                ));
            }
            *slot = None;
        }
    }
    Ok(())
}

fn require(record: &SnapshotRecord, state: LifecycleState, epoch: u32) -> Result<(), ReplayError> {
    if record.lifecycle_state != state || record.file_epoch != epoch {
        return Err(invalid(
            record.file_id,
            "stored lifecycle or epoch does not match",
        ));
    }
    Ok(())
}

fn validate_progress(record: &SnapshotRecord, op: &UpdateProgress) -> Result<(), ReplayError> {
    require(record, LifecycleState::Active, op.expected_file_epoch)?;
    let reject = || {
        invalid(
            record.file_id,
            "progress does not preserve the stored frontier and continuation contract",
        )
    };
    if !op
        .new_committed_frontier_guard
        .valid_for_offset(op.new_committed_offset)
        || !op
            .new_framing_resume
            .valid_for_offset(op.new_committed_offset)
        || (op.finalize && op.new_framing_resume != FramingResume::Clean)
    {
        return Err(reject());
    }
    if record.committed_offset != op.expected_committed_offset
        || op.new_committed_offset < record.committed_offset
    {
        return Err(reject());
    }
    if op.new_committed_offset == record.committed_offset {
        if op.new_committed_frontier_guard != record.committed_frontier_guard
            || op.new_framing_resume != record.framing_resume
        {
            return Err(reject());
        }
        return Ok(());
    }
    let next_start = match op.new_framing_resume {
        FramingResume::Clean => None,
        FramingResume::Continuation {
            record_start_offset,
            ..
        } => Some(record_start_offset),
    };
    match record.framing_resume {
        FramingResume::Clean => {
            if next_start.is_some_and(|start| start < record.committed_offset) {
                return Err(reject());
            }
        }
        FramingResume::Continuation {
            record_start_offset: start,
            record_end_offset: end,
            next_fragment_index: index,
        } => {
            let same = matches!(op.new_framing_resume,
                FramingResume::Continuation { record_start_offset, record_end_offset, next_fragment_index }
                if record_start_offset == start && record_end_offset == end && next_fragment_index > index);
            if end != 0 && op.new_committed_offset < end {
                if !same {
                    return Err(reject());
                }
            } else if end != 0 {
                if next_start.is_some_and(|start| start < end) {
                    return Err(reject());
                }
            } else if !same && next_start.is_some_and(|start| start < record.committed_offset) {
                return Err(reject());
            }
            // For scan-to-LF state, only the caller can prove the LF boundary.
            // A clean or later continuation is legal after that source proof.
        }
    }
    Ok(())
}

fn apply_progress_fields(record: &mut SnapshotRecord, op: &UpdateProgress) {
    record.committed_offset = op.new_committed_offset;
    record.committed_frontier_guard = op.new_committed_frontier_guard;
    record.framing_resume = op.new_framing_resume;
    record.last_seen_time_unix_nano = op.new_last_seen_time_unix_nano;
    if op.finalize {
        record.lifecycle_state = LifecycleState::RotatedFinalized;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RemoveFile, encode_checkpoint, encode_transaction};

    /// Scenario: Replay reaches the final representable sequence in one WAL generation.
    /// Guarantees: The final transaction applies once, empty input remains valid, and further input cannot wrap the sequence.
    #[test]
    fn sequence_exhaustion_does_not_wrap() {
        let config = ReplayConfig {
            max_tracked_files: 1,
            max_snapshot_bytes: 1024,
            fingerprint_bytes: 16,
            framing_profile_digest: [0; 32],
        };
        let prefix = encode_checkpoint(1, "test", &[]).expect("empty checkpoint");
        let (mut state, _) = ReplayState::from_checkpoint(&prefix, "test", config).expect("state");
        state.last_sequence = u64::MAX - 1;
        let transaction = Transaction {
            sequence: u64::MAX,
            operations: vec![Operation::RemoveFile(RemoveFile {
                file_id: FileId::from_bytes([0; 16]),
                expected_file_epoch: 1,
                expected_prior_state: LifecycleState::Active,
                removal_reason: 1,
                removal_time_unix_nano: 0,
                administrative: true,
                namespace_id: Some("test".to_owned()),
                audit_reason: Some("approved".to_owned()),
            })],
        };
        let bytes = encode_transaction(&transaction).expect("frame");
        let _ = state.replay_next(&bytes).expect("last transaction");
        assert_eq!(state.last_sequence(), u64::MAX);
        assert_eq!(state.replay_next(&[]), Ok(ReplayStep::Empty));
        assert_eq!(
            state.replay_next(&bytes),
            Err(ReplayError::SequenceExhausted)
        );
        assert_eq!(state.records().len(), 0);
    }
}
