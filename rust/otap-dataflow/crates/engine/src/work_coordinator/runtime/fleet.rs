// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::collections::BTreeSet;

struct Session<P: WorkProtocol> {
    port: Port<P>,
    state: ParticipantState,
    eligible: bool,
}
struct Entry<P: WorkProtocol> {
    versions: BTreeMap<u64, Arc<WorkSpec<P>>>,
    phase: Option<AssignmentPhase>,
    target: Option<ParticipantSession>,
}
struct Stage {
    operation: OperationId,
    old_revision: u64,
    new_revision: u64,
    prior: BTreeMap<usize, (u64, u64)>,
    phase: ReconciliationPhase,
    prepare: Option<ReceiptCompleter>,
    finish: Option<ReceiptCompleter>,
    preparation_failed: bool,
    provider_done: bool,
}
struct Activation {
    receipt: ReceiptCompleter,
    retiring: BTreeSet<ParticipantSession>,
    required: BTreeSet<WorkId>,
    target: Option<ParticipantSession>,
}

pub(super) struct Fleet<P: WorkProtocol> {
    incarnation: uuid::Uuid,
    domain: Arc<Domain>,
    catalog: BTreeMap<WorkId, Entry<P>>,
    desired: BTreeMap<u64, BTreeSet<WorkId>>,
    sessions: BTreeMap<ParticipantSession, Session<P>>,
    registry: watch::Sender<Arc<ParticipantRegistry>>,
    lifecycle: mpsc::Sender<CoordinatorLifecycleEvent>,
    notices: Notices<P>,
    participants: Arc<Mutex<HashMap<(usize, u64), WorkParticipant<P>>>>,
    backend: InMemoryBackend,
    term: u64,
    operation: u64,
    revision: u64,
    initializing: bool,
    stage: Option<Stage>,
    activation: Option<Activation>,
    pub(super) provider_pending: Option<oneshot::Receiver<Result<(), CoordinatorError>>>,
}

fn identity(phase: &AssignmentPhase) -> &GrantIdentity {
    match phase {
        AssignmentPhase::Offered(id) => id,
        AssignmentPhase::Active(grant) | AssignmentPhase::Revoking(grant) => grant.identity(),
    }
}

impl<P: WorkProtocol> Fleet<P> {
    pub fn new(
        domain: Arc<Domain>,
        ports: Vec<Port<P>>,
        registry: watch::Sender<Arc<ParticipantRegistry>>,
        lifecycle: mpsc::Sender<CoordinatorLifecycleEvent>,
        notices: Notices<P>,
        participants: Arc<Mutex<HashMap<(usize, u64), WorkParticipant<P>>>>,
    ) -> Self {
        let capacity = domain.limits.catalog_items;
        Self {
            incarnation: uuid::Uuid::new_v4(),
            domain,
            catalog: BTreeMap::new(),
            desired: BTreeMap::from([(0, BTreeSet::new())]),
            sessions: ports
                .into_iter()
                .map(|port| {
                    (
                        port.member.session,
                        Session {
                            port,
                            state: ParticipantState::Initializing,
                            eligible: true,
                        },
                    )
                })
                .collect(),
            registry,
            lifecycle,
            notices,
            participants,
            backend: InMemoryBackend::new(capacity),
            term: 0,
            operation: 0,
            revision: 0,
            initializing: true,
            stage: None,
            activation: None,
            provider_pending: None,
        }
    }
    fn next_operation(&mut self) -> Result<OperationId, CoordinatorError> {
        self.operation = self
            .operation
            .checked_add(1)
            .ok_or_else(|| CoordinatorError::Failed("backend operation overflow".into()))?;
        Ok(OperationId(self.operation))
    }
    fn phase(&mut self, phase: ReconciliationPhase) {
        if let Some(stage) = &mut self.stage {
            stage.phase = phase;
            self.domain.status.lock().reconciliation = Some((stage.operation, phase));
        }
    }
    fn live(&self, session: ParticipantSession) -> bool {
        self.sessions
            .get(&session)
            .is_some_and(|session| session.port.member.alive.load(Ordering::Acquire))
    }
    fn eligible(&self, session: ParticipantSession, work: &WorkId) -> bool {
        self.sessions.get(&session).is_some_and(|session| {
            session.eligible
                && session.state == ParticipantState::ReadyForAssignments
                && session.port.member.alive.load(Ordering::Acquire)
                && self
                    .desired
                    .get(&session.port.member.revision)
                    .is_some_and(|desired| desired.contains(work))
        })
    }
    fn wanted(&self, work: &WorkId) -> bool {
        self.sessions
            .keys()
            .any(|session| self.eligible(*session, work))
    }
    fn garbage_collect(&mut self) {
        for (id, entry) in &mut self.catalog {
            let owned_revision = entry.phase.as_ref().map(|phase| identity(phase).revision);
            entry.versions.retain(|revision, _| {
                owned_revision == Some(*revision)
                    || self
                        .desired
                        .get(revision)
                        .is_some_and(|desired| desired.contains(id))
            });
        }
        self.catalog
            .retain(|_, entry| !entry.versions.is_empty() || entry.phase.is_some());
    }
    fn publish(&mut self) -> Result<(), CoordinatorError> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| CoordinatorError::Failed("registry revision overflow".into()))?;
        let mut snapshots: BTreeMap<_, Vec<_>> = self
            .sessions
            .keys()
            .map(|session| (*session, Vec::new()))
            .collect();
        for entry in self.catalog.values() {
            if let Some(phase) = &entry.phase {
                let grant = identity(phase);
                let work = entry
                    .versions
                    .get(&grant.revision)
                    .ok_or(CoordinatorError::Stale)?;
                snapshots
                    .get_mut(&grant.session)
                    .ok_or(CoordinatorError::Stale)?
                    .push(WorkAssignment {
                        work: Arc::clone(work),
                        phase: phase.clone(),
                    });
            }
        }
        for (id, session) in &self.sessions {
            let _ = session
                .port
                .assignments
                .send_replace(Arc::new(AssignmentSnapshot {
                    revision: self.revision,
                    assignments: snapshots.remove(id).expect("session snapshot"),
                }));
        }
        let _ = self.registry.send_replace(Arc::new(ParticipantRegistry {
            revision: self.revision,
            participants: self
                .sessions
                .iter()
                .map(|(session, state)| (*session, state.state))
                .collect(),
        }));
        let mut status = self.domain.status.lock();
        let reconciliation = status.reconciliation;
        *status = CoordinatorStatus {
            reconciliation,
            participants: self.sessions.len(),
            ready_participants: self
                .sessions
                .values()
                .filter(|session| session.state == ParticipantState::ReadyForAssignments)
                .count(),
            catalog_items: self.catalog.len(),
            active_assignments: self
                .catalog
                .values()
                .filter(|entry| matches!(entry.phase, Some(AssignmentPhase::Active(_))))
                .count(),
            revoking_assignments: self
                .catalog
                .values()
                .filter(|entry| matches!(entry.phase, Some(AssignmentPhase::Revoking(_))))
                .count(),
            ..CoordinatorStatus::default()
        };
        Ok(())
    }
    fn revoke(&mut self, work: &WorkId) {
        if let Some(entry) = self.catalog.get_mut(work) {
            entry.phase = match entry.phase.take() {
                Some(AssignmentPhase::Active(grant) | AssignmentPhase::Revoking(grant)) => {
                    if let Some(lease) = self
                        .domain
                        .admission
                        .lock()
                        .grants
                        .get_mut(&grant.identity().term)
                    {
                        lease.accepting = false;
                    }
                    Some(AssignmentPhase::Revoking(grant))
                }
                _ => None,
            };
        }
    }
    fn allocate(&mut self) -> Result<(), CoordinatorError> {
        if self.domain.stopping.load(Ordering::Acquire) {
            return self.publish();
        }
        if self.initializing {
            if self
                .sessions
                .values()
                .any(|session| session.state != ParticipantState::ReadyForAssignments)
            {
                return self.publish();
            }
            self.initializing = false;
        }
        let mut loads: BTreeMap<_, usize> =
            self.sessions.keys().map(|session| (*session, 0)).collect();
        for entry in self.catalog.values() {
            if let Some(phase) = &entry.phase {
                *loads
                    .get_mut(&identity(phase).session)
                    .ok_or(CoordinatorError::Stale)? += 1;
            }
        }
        let ids: Vec<_> = self
            .catalog
            .iter()
            .filter(|(_, entry)| entry.phase.is_none())
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            let entry = &self.catalog[&id];
            let target = entry
                .target
                .filter(|target| self.eligible(*target, &id))
                .or_else(|| {
                    loads
                        .iter()
                        .filter(|(session, _)| self.eligible(**session, &id))
                        .min_by_key(|(session, load)| (**load, **session))
                        .map(|(session, _)| *session)
                });
            if let Some(target) = target {
                self.term = self
                    .term
                    .checked_add(1)
                    .ok_or_else(|| CoordinatorError::Failed("assignment term overflow".into()))?;
                let revision = self.sessions[&target].port.member.revision;
                let entry = self.catalog.get_mut(&id).expect("catalog entry");
                entry.phase = Some(AssignmentPhase::Offered(GrantIdentity {
                    incarnation: self.incarnation,
                    work_id: id,
                    session: target,
                    revision,
                    term: self.term,
                }));
                entry.target = None;
                *loads.get_mut(&target).expect("target load") += 1;
            }
        }
        self.publish()
    }
    pub fn catalog(
        &mut self,
        revision: u64,
        works: Vec<WorkSpec<P>>,
    ) -> Result<(), CoordinatorError> {
        validate_catalog(&works, self.domain.limits)?;
        if !self.domain.revisions.lock().contains_key(&revision) {
            return Err(CoordinatorError::Stale);
        }
        if let Some(stage) = &self.stage
            && stage.old_revision != stage.new_revision
            && ((self.domain.commit_decision.load(Ordering::Acquire) == stage.operation.0
                && revision == stage.old_revision)
                || (stage.phase == ReconciliationPhase::Aborting && revision == stage.new_revision))
        {
            return Err(CoordinatorError::Stale);
        }
        let mut ids: BTreeSet<_> = self.catalog.keys().cloned().collect();
        let mut bytes: usize = self
            .catalog
            .values()
            .flat_map(|entry| entry.versions.values())
            .map(|work| work_bytes(work))
            .sum();
        for work in &works {
            if let Some(current) = self
                .catalog
                .get(&work.id)
                .and_then(|entry| entry.versions.get(&revision))
            {
                if current.work != work.work {
                    return Err(CoordinatorError::Invalid(
                        "work metadata changed before retirement",
                    ));
                }
            } else {
                bytes = bytes.saturating_add(work_bytes(work));
            }
            let _ = ids.insert(work.id.clone());
        }
        if ids.len() > self.domain.limits.catalog_items || bytes > self.domain.limits.catalog_bytes
        {
            return Err(CoordinatorError::Capacity);
        }
        let desired: BTreeSet<_> = works.iter().map(|work| work.id.clone()).collect();
        let _ = self.desired.insert(revision, desired);
        for work in works {
            let entry = self
                .catalog
                .entry(work.id.clone())
                .or_insert_with(|| Entry {
                    versions: BTreeMap::new(),
                    phase: None,
                    target: None,
                });
            let _ = entry
                .versions
                .entry(revision)
                .or_insert_with(|| Arc::new(work));
        }
        let revoke: Vec<_> = self
            .catalog
            .iter()
            .filter_map(|(id, entry)| {
                entry
                    .phase
                    .as_ref()
                    .filter(|phase| {
                        identity(phase).revision == revision
                            && !self.desired[&revision].contains(id)
                    })
                    .map(|_| id.clone())
            })
            .collect();
        for id in revoke {
            self.revoke(&id);
        }
        self.garbage_collect();
        self.allocate()
    }
    async fn participant(&mut self, control: &Control) -> Result<(), CoordinatorError> {
        self.domain.check()?;
        if !self.live(control.session) {
            return Err(CoordinatorError::Stale);
        }
        if matches!(control.kind, AckKind::Ready) {
            self.sessions
                .get_mut(&control.session)
                .ok_or(CoordinatorError::Stale)?
                .state = ParticipantState::ReadyForAssignments;
            return self.allocate();
        }
        let grant = control.grant.as_ref().ok_or(CoordinatorError::Stale)?;
        if grant.session != control.session || grant.incarnation != self.incarnation {
            return Err(CoordinatorError::Stale);
        }
        let phase = self
            .catalog
            .get(&grant.work_id)
            .and_then(|entry| entry.phase.clone())
            .ok_or(CoordinatorError::Stale)?;
        match (control.kind, phase) {
            (AckKind::Accept, AssignmentPhase::Offered(offer)) if offer == *grant => {
                if self.domain.stopping.load(Ordering::Acquire)
                    || !self.eligible(control.session, &grant.work_id)
                {
                    return Err(CoordinatorError::Unavailable);
                }
                let operation = self.next_operation()?;
                if self.backend.activate(operation, grant).await? != BackendOutcome::Confirmed {
                    return Err(CoordinatorError::Failed(
                        "activation was not confirmed".into(),
                    ));
                }
                let _ = self
                    .domain
                    .admission
                    .lock()
                    .grants
                    .insert(grant.term, Lease::default());
                let _ = self.sessions[&control.session]
                    .port
                    .member
                    .owned
                    .fetch_add(1, Ordering::AcqRel);
                self.catalog
                    .get_mut(&grant.work_id)
                    .expect("catalog entry")
                    .phase = Some(AssignmentPhase::Active(AssignmentGrant {
                    identity: grant.clone(),
                    admission: Arc::clone(&self.domain.admission),
                }));
            }
            (AckKind::Activated, AssignmentPhase::Active(active)) if active.identity() == grant => {
                let mut admission = self.domain.admission.lock();
                if self.domain.stopping.load(Ordering::Acquire) {
                    return Err(CoordinatorError::Unavailable);
                }
                let lease = admission
                    .grants
                    .get_mut(&grant.term)
                    .ok_or(CoordinatorError::Stale)?;
                if lease.accepting {
                    return Ok(());
                }
                lease.accepting = true;
            }
            (AckKind::Released, AssignmentPhase::Revoking(active))
                if active.identity() == grant =>
            {
                if self
                    .domain
                    .admission
                    .lock()
                    .grants
                    .get(&grant.term)
                    .is_none_or(|lease| lease.tickets != 0)
                {
                    return Err(CoordinatorError::Invalid(
                        "release requires settled tickets and joined workers",
                    ));
                }
                let operation = self.next_operation()?;
                if self.backend.retire(operation, grant).await? != BackendOutcome::Confirmed {
                    return Err(CoordinatorError::Failed(
                        "retirement was not confirmed".into(),
                    ));
                }
                let _ = self.domain.admission.lock().grants.remove(&grant.term);
                let _ = self.sessions[&control.session]
                    .port
                    .member
                    .owned
                    .fetch_sub(1, Ordering::AcqRel);
                self.catalog
                    .get_mut(&grant.work_id)
                    .expect("catalog entry")
                    .phase = None;
                self.garbage_collect();
            }
            _ => return Err(CoordinatorError::Stale),
        }
        self.allocate()
    }
    pub async fn command(&mut self, command: HostControl<P>) -> Result<(), CoordinatorError> {
        match command {
            HostControl::Participant(control) => {
                let result = self.participant(&control).await;
                if matches!(result, Err(CoordinatorError::Failed(_))) {
                    return result;
                }
                let _ = control.receipt.send(result);
            }
            HostControl::Register(port, receipt) => {
                let session = port.member.session;
                let valid = self
                    .domain
                    .revisions
                    .lock()
                    .contains_key(&port.member.revision)
                    && self.sessions.len() < self.domain.limits.participants * 2;
                if valid {
                    let _ = self.sessions.insert(
                        session,
                        Session {
                            port,
                            state: ParticipantState::Initializing,
                            eligible: false,
                        },
                    );
                    receipt.finish(Ok(ReconciliationOutcome::Registered(session)));
                    self.publish()?;
                } else {
                    port.member.alive.store(false, Ordering::Release);
                    let _ = self
                        .participants
                        .lock()
                        .remove(&(session.core_id, session.generation));
                    let _ = self.domain.members.lock().remove(&session);
                    let _ = self.notices.lock().remove(&session);
                    receipt.finish(Err(CoordinatorError::Capacity));
                }
            }
            HostControl::Prepare(settings, receipt) => {
                self.prepare(settings, receipt)?;
            }
            HostControl::Activate(spec, departing, receipt) => {
                if self.activation.is_some() {
                    receipt.finish(Err(CoordinatorError::Invalid(
                        "ownership transition already pending",
                    )));
                } else {
                    self.activate(spec, departing, receipt)?;
                }
            }
            HostControl::Finish(operation, commit, receipt) => {
                self.finish(operation, commit, receipt)?;
            }
        }
        Ok(())
    }
    fn prepare(
        &mut self,
        settings: Arc<serde_json::Value>,
        receipt: ReceiptCompleter,
    ) -> Result<(), CoordinatorError> {
        let operation = receipt.operation();
        let old_revision = self.domain.current_revision.load(Ordering::Acquire);
        let old_settings = self
            .domain
            .revisions
            .lock()
            .get(&old_revision)
            .cloned()
            .ok_or(CoordinatorError::Stale)?;
        let changed = *old_settings != *settings;
        let new_revision = if changed {
            self.domain
                .revision_counter
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                    value.checked_add(1)
                })
                .map_err(|_| CoordinatorError::Failed("settings revision overflow".into()))?
                + 1
        } else {
            old_revision
        };
        let prior = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                session.eligible && session.port.member.alive.load(Ordering::Acquire)
            })
            .map(|(id, session)| (id.core_id, (id.generation, session.port.member.revision)))
            .collect();
        self.stage = Some(Stage {
            operation,
            old_revision,
            new_revision,
            prior,
            phase: ReconciliationPhase::Preparing,
            prepare: Some(receipt),
            finish: None,
            preparation_failed: false,
            provider_done: !changed,
        });
        if changed {
            if self.domain.revisions.lock().len() >= 2 {
                return Err(CoordinatorError::Failed(
                    "retained coordinator revision limit exceeded".into(),
                ));
            }
            let _ = self
                .domain
                .revisions
                .lock()
                .insert(new_revision, Arc::clone(&settings));
            let _ = self.desired.insert(new_revision, BTreeSet::new());
            let (receipt, receiver) = ProviderReceipt::channel();
            self.provider_pending = Some(receiver);
            if self
                .lifecycle
                .try_send(CoordinatorLifecycleEvent::Prepare {
                    operation,
                    revision: new_revision,
                    settings,
                    receipt,
                })
                .is_err()
            {
                self.provider_completed(Ok(Err(CoordinatorError::Capacity)))?;
            }
        } else {
            self.prepared(Ok(()));
        }
        Ok(())
    }
    fn prepared(&mut self, result: Result<(), CoordinatorError>) {
        if let Some(stage) = &mut self.stage {
            stage.preparation_failed = result.is_err();
            stage.provider_done = true;
            if let Some(receipt) = stage.prepare.take() {
                receipt
                    .finish(result.map(|()| ReconciliationOutcome::Prepared(stage.new_revision)));
            }
            if !stage.preparation_failed {
                self.phase(ReconciliationPhase::Prepared);
            }
        }
    }
    fn rebalance(&mut self) {
        let mut loads: BTreeMap<_, usize> = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                session.eligible
                    && session.state == ParticipantState::ReadyForAssignments
                    && session.port.member.alive.load(Ordering::Acquire)
            })
            .map(|(id, _)| (*id, 0))
            .collect();
        for entry in self.catalog.values() {
            if let Some(phase) = &entry.phase
                && let Some(load) = loads.get_mut(&identity(phase).session)
            {
                *load += 1;
            }
        }
        let ids: Vec<_> = self.catalog.keys().cloned().collect();
        for id in ids {
            let owner = self.catalog[&id]
                .phase
                .as_ref()
                .map(|phase| identity(phase).session);
            let Some(owner) = owner else {
                continue;
            };
            let target = loads
                .iter()
                .filter(|(session, _)| self.eligible(**session, &id))
                .min_by_key(|(session, load)| (**load, **session))
                .map(|(session, load)| (*session, *load));
            if let Some((target, load)) = target
                && loads
                    .get(&owner)
                    .is_some_and(|owner_load| *owner_load > load + 1)
            {
                *loads.get_mut(&owner).expect("owner load") -= 1;
                *loads.get_mut(&target).expect("target load") += 1;
                self.catalog.get_mut(&id).expect("entry").target = Some(target);
                self.revoke(&id);
            }
        }
    }
    fn activate(
        &mut self,
        spec: ParticipantSpec,
        departing: bool,
        receipt: ReceiptCompleter,
    ) -> Result<(), CoordinatorError> {
        if self.domain.stopping.load(Ordering::Acquire) {
            receipt.finish(Err(CoordinatorError::Unavailable));
            return Ok(());
        }
        let target = self
            .sessions
            .iter()
            .find(|(id, session)| {
                id.core_id == spec.core_id
                    && id.generation == spec.generation
                    && session.port.member.alive.load(Ordering::Acquire)
            })
            .map(|(id, _)| *id);
        if !departing
            && target
                .is_none_or(|id| self.sessions[&id].state != ParticipantState::ReadyForAssignments)
        {
            receipt.finish(Err(CoordinatorError::Stale));
            return Ok(());
        }
        let retiring: BTreeSet<_> = self
            .sessions
            .keys()
            .copied()
            .filter(|id| {
                id.core_id == spec.core_id
                    && if departing {
                        id.generation == spec.generation
                    } else {
                        Some(*id) != target
                    }
            })
            .collect();
        for (id, session) in &mut self.sessions {
            if retiring.contains(id) {
                session.eligible = false;
            }
            if !departing && Some(*id) == target {
                session.eligible = true;
            }
        }
        let revoke: Vec<_> = self
            .catalog
            .iter()
            .filter_map(|(id, entry)| {
                entry
                    .phase
                    .as_ref()
                    .filter(|phase| retiring.contains(&identity(phase).session))
                    .map(|_| id.clone())
            })
            .collect();
        for id in revoke {
            self.revoke(&id);
        }
        self.rebalance();
        let required = self
            .catalog
            .keys()
            .filter(|id| self.wanted(id))
            .cloned()
            .collect();
        self.activation = Some(Activation {
            receipt,
            retiring,
            required,
            target: if departing { None } else { target },
        });
        self.phase(ReconciliationPhase::Activating);
        self.allocate()
    }
    fn finish(
        &mut self,
        operation: OperationId,
        commit: bool,
        receipt: ReceiptCompleter,
    ) -> Result<(), CoordinatorError> {
        let Some(stage) = &self.stage else {
            receipt.finish(Err(CoordinatorError::Stale));
            return Ok(());
        };
        if stage.operation != operation {
            receipt.finish(Err(CoordinatorError::Stale));
            return Ok(());
        }
        if commit && stage.phase == ReconciliationPhase::Committed {
            receipt.finish(Ok(ReconciliationOutcome::Committed));
            return Ok(());
        }
        if !commit && self.domain.commit_decision.load(Ordering::Acquire) == operation.0 {
            receipt.finish(Err(CoordinatorError::Invalid(
                "committed reconciliation cannot abort",
            )));
            return Ok(());
        }
        if let Some(activation) = self.activation.take() {
            activation.receipt.finish(Err(CoordinatorError::Stale));
        }
        self.provider_pending = None;
        if let Some(stage) = &mut self.stage {
            if let Some(prepare) = stage.prepare.take() {
                prepare.finish(Err(CoordinatorError::Stale));
            }
            stage.finish = Some(receipt);
            stage.provider_done =
                stage.old_revision == stage.new_revision || stage.preparation_failed;
        }
        self.phase(if commit {
            ReconciliationPhase::CommitDecided
        } else {
            ReconciliationPhase::Aborting
        });
        let stage = self.stage.as_ref().expect("stage");
        let new_revision = stage.new_revision;
        let changed = stage.old_revision != stage.new_revision && !stage.preparation_failed;
        if commit {
            self.domain
                .current_revision
                .store(new_revision, Ordering::Release);
        } else {
            for (id, session) in &mut self.sessions {
                session.eligible =
                    stage
                        .prior
                        .get(&id.core_id)
                        .is_some_and(|(generation, revision)| {
                            *generation == id.generation
                                && *revision == session.port.member.revision
                        })
                        && session.port.member.alive.load(Ordering::Acquire);
            }
            let revoke: Vec<_> = self
                .catalog
                .iter()
                .filter_map(|(id, entry)| {
                    entry
                        .phase
                        .as_ref()
                        .filter(|phase| !self.sessions[&identity(phase).session].eligible)
                        .map(|_| id.clone())
                })
                .collect();
            for id in revoke {
                self.revoke(&id);
            }
        }
        if changed {
            let (receipt, receiver) = ProviderReceipt::channel();
            self.provider_pending = Some(receiver);
            let event = if commit {
                CoordinatorLifecycleEvent::Commit {
                    operation,
                    revision: new_revision,
                    receipt,
                }
            } else {
                CoordinatorLifecycleEvent::Abort {
                    operation,
                    revision: new_revision,
                    receipt,
                }
            };
            if self.lifecycle.try_send(event).is_err() {
                self.provider_completed(Ok(Err(CoordinatorError::Capacity)))?;
            }
        }
        self.allocate()
    }
    pub fn provider_completed(
        &mut self,
        result: Result<Result<(), CoordinatorError>, oneshot::error::RecvError>,
    ) -> Result<(), CoordinatorError> {
        self.provider_pending = None;
        let result = result.unwrap_or_else(|_| {
            Err(CoordinatorError::Failed(
                "provider receipt lost; cleanup outcome unknown".into(),
            ))
        });
        if self
            .stage
            .as_ref()
            .is_some_and(|stage| stage.phase == ReconciliationPhase::Preparing)
        {
            self.prepared(result);
        } else if let Some(stage) = &mut self.stage {
            match result {
                Ok(()) => stage.provider_done = true,
                Err(error) => {
                    if let Some(receipt) = stage.finish.take() {
                        receipt.finish(Err(error));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn sweep_exits(&mut self) -> Result<(), CoordinatorError> {
        let dead: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                !session.port.member.alive.load(Ordering::Acquire)
                    && session.port.member.owned.load(Ordering::Acquire) == 0
            })
            .map(|(id, _)| *id)
            .collect();
        for id in dead {
            let offered: Vec<_> = self
                .catalog
                .iter()
                .filter_map(|(work, entry)| {
                    entry
                        .phase
                        .as_ref()
                        .filter(|phase| identity(phase).session == id)
                        .map(|_| work.clone())
                })
                .collect();
            for work in offered {
                self.revoke(&work);
            }
            let _ = self.sessions.remove(&id);
            let _ = self.notices.lock().remove(&id);
        }
        self.garbage_collect();
        self.allocate()
    }
    fn settled(
        &self,
        required: &BTreeSet<WorkId>,
        retiring: &BTreeSet<ParticipantSession>,
    ) -> bool {
        if self.catalog.values().any(|entry| {
            entry
                .phase
                .as_ref()
                .is_some_and(|phase| retiring.contains(&identity(phase).session))
        }) {
            return false;
        }
        required.iter().all(|id| {
            !self.wanted(id)
                || self
                    .catalog
                    .get(id)
                    .and_then(|entry| entry.phase.as_ref())
                    .is_some_and(|phase| match phase {
                        AssignmentPhase::Active(grant) => self
                            .domain
                            .admission
                            .lock()
                            .grants
                            .get(&grant.identity().term)
                            .is_some_and(|lease| lease.accepting),
                        _ => false,
                    })
        })
    }
    pub fn progress(&mut self) -> Result<(), CoordinatorError> {
        if let Some(activation) = &self.activation {
            if activation.target.is_some_and(|target| !self.live(target)) {
                self.activation
                    .take()
                    .expect("activation")
                    .receipt
                    .finish(Err(CoordinatorError::Unavailable));
            } else if self.settled(&activation.required, &activation.retiring) {
                self.activation
                    .take()
                    .expect("activation")
                    .receipt
                    .finish(Ok(ReconciliationOutcome::Activated));
                self.phase(ReconciliationPhase::Activated);
            }
        }
        let Some(stage) = &self.stage else {
            return Ok(());
        };
        if !stage.provider_done
            || !matches!(
                stage.phase,
                ReconciliationPhase::CommitDecided | ReconciliationPhase::Aborting
            )
        {
            return Ok(());
        }
        let commit = stage.phase == ReconciliationPhase::CommitDecided;
        if self
            .catalog
            .values()
            .any(|entry| matches!(entry.phase, Some(AssignmentPhase::Revoking(_))))
        {
            return Ok(());
        }
        if !commit {
            let required: BTreeSet<_> = self
                .catalog
                .keys()
                .filter(|id| self.wanted(id))
                .cloned()
                .collect();
            if !self.settled(&required, &BTreeSet::new()) {
                return Ok(());
            }
        }
        let discard = if commit {
            stage.old_revision
        } else {
            stage.new_revision
        };
        let keep = if commit {
            stage.new_revision
        } else {
            stage.old_revision
        };
        if discard != keep {
            if self.catalog.values().any(|entry| {
                entry
                    .phase
                    .as_ref()
                    .is_some_and(|phase| identity(phase).revision == discard)
            }) {
                return Ok(());
            }
            let _ = self.domain.revisions.lock().remove(&discard);
            let _ = self.desired.remove(&discard);
        }
        self.garbage_collect();
        let outcome = if commit {
            ReconciliationOutcome::Committed
        } else {
            ReconciliationOutcome::Aborted
        };
        if let Some(receipt) = self.stage.as_mut().expect("stage").finish.take() {
            receipt.finish(Ok(outcome));
        }
        self.phase(if commit {
            ReconciliationPhase::Committed
        } else {
            ReconciliationPhase::Aborted
        });
        self.publish()
    }
}
