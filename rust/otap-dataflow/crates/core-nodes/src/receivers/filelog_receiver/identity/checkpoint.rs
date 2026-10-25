// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Store delegation, bounded admission groups, and optional evidence maintenance.
use super::*;

impl IdentityOwner {
    /// Maximum plans per atomic admission group, accounting for three operations
    /// per plan and worst-case encoded operation size, not merely file count.
    #[must_use]
    pub fn admission_limit(&self) -> usize {
        self.admission_limit
    }

    /// Last applied WAL byte count; an unresolved attempt may have additional bytes.
    #[must_use]
    pub fn wal_bytes(&self) -> u64 {
        self.store.wal_bytes()
    }

    /// Last sequence certified durable by this store instance.
    #[must_use]
    pub fn synced_sequence(&self) -> u64 {
        self.store.synced_sequence()
    }

    /// Namespace exclusion identity, validated by the store before discovery starts.
    pub fn namespace_locator(&self) -> Result<cp::Locator, IdentityError> {
        self.running()?;
        Ok(self.store.namespace_locator()?)
    }

    /// Whether retry exhaustion or a terminal failure has stopped this owner.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// A retained append blocks new work, including attempts to re-prepare it.
    #[must_use]
    pub fn has_pending_write(&self) -> bool {
        self.pending.is_some()
    }

    fn next_revision(&mut self, operations: &[cp::Operation]) -> Result<u64, IdentityError> {
        if operations.iter().all(cp::Operation::is_ordinary_progress) {
            return Ok(self.identity_revision);
        }
        match self.identity_revision.checked_add(1) {
            Some(next) => Ok(next),
            None => {
                self.stopped = true;
                Err(IdentityError::RevisionExhausted)
            }
        }
    }

    fn pending_error(&mut self, error: StoreError) -> IdentityError {
        self.failures = self.failures.saturating_add(1);
        let recoverable = matches!(
            error,
            StoreError::Io { .. } | StoreError::AuthoritativeGenerationUnreadable(_)
        ) && self.store.can_resolve_pending();
        if !recoverable || self.failures >= self.failure_limit {
            self.stopped = true;
            IdentityError::Store(error)
        } else {
            IdentityError::PendingWrite(error)
        }
    }

    fn execute(
        &mut self,
        operations: Vec<cp::Operation>,
        durability: Durability,
        admissions: Vec<Completion>,
        purpose: WritePurpose,
    ) -> Result<(cp::store::AppendReceipt, Vec<Completion>), IdentityError> {
        self.running()?;
        let next_revision = self.next_revision(&operations)?;
        match self.store.append(operations, durability) {
            Ok(receipt) => {
                self.identity_revision = next_revision;
                self.failures = 0;
                Ok((receipt, admissions))
            }
            Err(error) => {
                if self.store.has_unresolved_append() {
                    self.pending = Some(PendingWrite {
                        next_revision,
                        admissions,
                        purpose,
                    });
                    Err(self.pending_error(error))
                } else {
                    let optional_capacity = purpose == WritePurpose::FingerprintGrowth
                        && matches!(error, StoreError::CompactionRequired);
                    self.stopped = !optional_capacity
                        && !matches!(error, StoreError::Encode(_) | StoreError::Replay(_));
                    Err(IdentityError::AppendRejected(error))
                }
            }
        }
    }

    /// Apply authorized progress or metadata. `PendingWrite` means the original
    /// attempt is retained: back off on the worker, then call resolve_pending.
    pub fn append_checkpoint(
        &mut self,
        operations: Vec<cp::Operation>,
        durability: Durability,
    ) -> Result<cp::store::AppendReceipt, IdentityError> {
        self.execute(operations, durability, Vec::new(), WritePurpose::Required)
            .map(|(receipt, _)| receipt)
    }

    /// Resolve precisely the retained append under the original namespace lock.
    /// Backoff belongs to the worker. The configured consecutive-failure budget
    /// includes the original failure; sync/authority/corruption failures are terminal.
    /// Success consumes the result once, including the original admission leases.
    pub fn resolve_pending(&mut self) -> Result<ResolvedWrite, IdentityError> {
        if self.stopped {
            return Err(IdentityError::Stopped);
        }
        if self.pending.is_none() {
            return Err(IdentityError::NoPendingWrite);
        }
        // Backoff separates this operation from initial commit. The original
        // outcomes/leases remain owned here if any registry check now fails.
        for admission in &self
            .pending
            .as_ref()
            .expect("checked pending outcome")
            .admissions
        {
            if let Err(error) = admission.seed.lease.validate() {
                self.stopped = true;
                return Err(error.into());
            }
        }
        match self.store.resolve_pending() {
            Ok(receipt) => {
                let pending = self.pending.take().expect("checked pending outcome");
                self.identity_revision = pending.next_revision;
                self.failures = 0;
                if pending.purpose == WritePurpose::FingerprintGrowth {
                    self.growth.clear();
                }
                let admissions = self.complete(pending.admissions);
                Ok(ResolvedWrite {
                    receipt,
                    admissions,
                })
            }
            Err(error) => Err(self.pending_error(error)),
        }
    }

    fn validate_plan(&self, prepared: &PreparedAdmission) -> Result<(), IdentityError> {
        let state = self.store.state()?;
        if !self.leases.owns(&prepared.seed.lease)
            || state.generation() != prepared.generation
            || self.identity_revision != prepared.identity_revision
            || state
                .get_by_locator(&evidence::locator(prepared.seed.lease.locator()))
                .map(RecordStamp::from)
                != prepared.touched
        {
            return Err(IdentityError::StalePlan);
        }
        Ok(())
    }

    /// Select the fitting group before discovery preflight. Input is bounded by
    /// admission_limit(), which already accounts for worst-case operation/byte
    /// limits. Recovery and same-locator supersession consume no additional slot.
    /// Input order is preserved; capacity-blocked new files do not suppress later
    /// net-zero work. This changes neither store nor discovery state.
    pub fn select_batch(
        &self,
        plans: Vec<PreparedAdmission>,
    ) -> Result<AdmissionSelection, IdentityError> {
        self.running()?;
        if plans.len() > self.admission_limit {
            return Err(IdentityError::BatchLimit);
        }
        let mut remaining = self
            .tracked_limit
            .checked_sub(self.store.state()?.records().len())
            .ok_or(IdentityError::BatchCapacity)?;
        let mut selection = AdmissionSelection {
            ready: Vec::with_capacity(plans.len()),
            needs_capacity: Vec::new(),
        };
        for plan in plans {
            self.validate_plan(&plan)?;
            if matches!(
                plan.decision,
                Decision::Register {
                    superseded: None,
                    ..
                }
            ) {
                if remaining == 0 {
                    selection.needs_capacity.push(plan);
                    continue;
                }
                remaining -= 1;
            }
            selection.ready.push(plan);
        }
        Ok(selection)
    }

    /// Commit one preflighted admission. On `PendingWrite` the owner, not the caller,
    /// retains its outcome; resolution returns it in `ResolvedWrite.admissions`.
    pub fn commit(
        &mut self,
        prepared: PreparedAdmission,
    ) -> Result<CommittedAdmission, IdentityError> {
        let mut completed = self.commit_batch(vec![prepared])?;
        Ok(completed.pop().expect("one plan produces one outcome"))
    }

    /// Commit a bounded group atomically after discovery has preflighted the entire
    /// group. Validate all plans against one revision before writing anything.
    /// Do not publish any discovery member before this succeeds or resolves.
    pub fn commit_batch(
        &mut self,
        plans: Vec<PreparedAdmission>,
    ) -> Result<Vec<CommittedAdmission>, IdentityError> {
        self.running()?;
        if plans.is_empty() || plans.len() > self.admission_limit {
            return Err(IdentityError::BatchLimit);
        }
        for plan in &plans {
            self.validate_plan(plan)?;
            if let Err(error) = plan.seed.lease.validate() {
                self.stopped = true;
                return Err(error.into());
            }
        }
        // A caller that skipped selection still cannot partially publish an
        // over-capacity preflighted group. Never trim members after preflight.
        let additions = plans
            .iter()
            .filter(|plan| {
                matches!(
                    plan.decision,
                    Decision::Register {
                        superseded: None,
                        ..
                    }
                )
            })
            .count();
        if self
            .store
            .state()?
            .records()
            .len()
            .checked_add(additions)
            .is_none_or(|count| count > self.tracked_limit)
        {
            return Err(IdentityError::BatchCapacity);
        }
        let mut operations = Vec::new();
        let mut admissions = Vec::with_capacity(plans.len());
        for plan in plans {
            operations.extend(plan.operations);
            admissions.push(Completion {
                seed: plan.seed,
                quarantine: matches!(
                    plan.decision,
                    Decision::Register {
                        anchor: Anchor::Quarantine,
                        ..
                    }
                ),
            });
        }
        if operations.is_empty() {
            return Ok(self.complete(admissions));
        }
        let (_, admissions) = self.execute(
            operations,
            Durability::Sync,
            admissions,
            WritePurpose::Required,
        )?;
        Ok(self.complete(admissions))
    }

    fn complete(&mut self, admissions: Vec<Completion>) -> Vec<CommittedAdmission> {
        admissions
            .into_iter()
            .map(|done| {
                if done.quarantine {
                    self.complete_quarantine(done.seed)
                } else {
                    CommittedAdmission::Reader(done.seed)
                }
            })
            .collect()
    }

    pub(super) fn complete_quarantine(&mut self, seed: ReaderSeed) -> CommittedAdmission {
        let release_error = seed.lease.release().err();
        if release_error.is_some() {
            self.stopped = true;
        }
        CommittedAdmission::Quarantined {
            file_id: seed.file_id,
            release_error,
        }
    }

    /// Queue a bounded raw-prefix extension observed by this live reader. False
    /// means the bounded queue is full: retain the bytes on the reader and retry
    /// after flush. This performs no I/O and cannot shrink/rebase another epoch.
    pub fn queue_fingerprint(
        &mut self,
        reader: &ReaderSeed,
        prefix: &[u8],
    ) -> Result<bool, IdentityError> {
        self.running()?;
        if !self.leases.owns(&reader.lease) || prefix.len() > self.fingerprint_bytes {
            return Err(IdentityError::InvalidCandidate);
        }
        let state = self.store.state()?;
        let record = state.get(&reader.file_id).ok_or(IdentityError::StalePlan)?;
        if record.lifecycle_state != cp::LifecycleState::Active
            || record.file_epoch != reader.epoch
            || record.locator != evidence::locator(reader.lease.locator())
        {
            return Err(IdentityError::StalePlan);
        }
        if record.fingerprint.starts_with(prefix) {
            return Ok(true);
        }
        if !prefix.starts_with(&record.fingerprint) {
            return Err(IdentityError::InvalidCandidate);
        }
        let update = cp::UpdateFingerprint {
            file_id: reader.file_id,
            expected_file_epoch: reader.epoch,
            expected_fingerprint: record.fingerprint.clone(),
            new_fingerprint: prefix.to_vec(),
        };
        if let Some(slot) = self
            .growth
            .iter_mut()
            .find(|op| op.file_id == reader.file_id)
        {
            if slot.expected_file_epoch == reader.epoch {
                if slot.new_fingerprint.starts_with(prefix) {
                    return Ok(true);
                }
                if !prefix.starts_with(&slot.new_fingerprint) {
                    return Err(IdentityError::InvalidCandidate);
                }
            }
            *slot = update;
        } else {
            if self.growth.len() >= self.growth_limit {
                return Ok(false);
            }
            self.growth.push(update);
        }
        Ok(true)
    }

    /// Persist optional growth as a separate transaction. Known pre-write WAL
    /// capacity rejection retains the queue and returns None, without stopping
    /// the owner. This does not certify progress; use sync_checkpoint for that.
    /// Encoding/replay failures remain visible; write, sync and authority failures
    /// keep the normal containment and pending-resolution rules.
    pub fn flush_fingerprints(
        &mut self,
    ) -> Result<Option<cp::store::AppendReceipt>, IdentityError> {
        self.running()?;
        let state = self.store.state()?;
        self.growth.retain(|update| {
            state.get(&update.file_id).is_some_and(|record| {
                record.lifecycle_state == cp::LifecycleState::Active
                    && record.file_epoch == update.expected_file_epoch
                    && record.fingerprint.len() < update.new_fingerprint.len()
                    && update.new_fingerprint.starts_with(&record.fingerprint)
            })
        });
        let operations: Vec<_> = self
            .growth
            .iter()
            .map(|update| {
                let record = state.get(&update.file_id).expect("retained growth record");
                cp::Operation::UpdateFingerprint(cp::UpdateFingerprint {
                    file_id: update.file_id,
                    expected_file_epoch: update.expected_file_epoch,
                    expected_fingerprint: record.fingerprint.clone(),
                    new_fingerprint: update.new_fingerprint.clone(),
                })
            })
            .collect();
        if operations.is_empty() {
            return Ok(None);
        }
        match self.execute(
            operations,
            Durability::Sync,
            Vec::new(),
            WritePurpose::FingerprintGrowth,
        ) {
            Ok((receipt, _)) => {
                self.growth.clear();
                Ok(Some(receipt))
            }
            Err(IdentityError::AppendRejected(StoreError::CompactionRequired)) => Ok(None),
            Err(
                error
                @ IdentityError::AppendRejected(StoreError::Encode(_) | StoreError::Replay(_)),
            ) => {
                // Report invalid optional work once rather than retrying it at
                // every barrier. Verified reader evidence can be queued anew.
                self.growth.clear();
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    /// Certify required progress first. Optional growth must never consume the
    /// interval/drain barrier or prevent it through a pre-write capacity rejection.
    pub fn sync_checkpoint(&mut self) -> Result<SyncOutcome, IdentityError> {
        self.running()?;
        if let Err(error) = self.store.sync() {
            self.stopped = true;
            return Err(error.into());
        }
        let synced_sequence = self.store.synced_sequence();
        let growth = self.flush_fingerprints();
        Ok(SyncOutcome {
            synced_sequence,
            growth,
        })
    }
}
