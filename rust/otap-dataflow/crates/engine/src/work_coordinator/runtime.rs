// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    authority::{AdmissionState, Lease},
    *,
};
use futures::{Future, future::LocalBoxFuture};
use parking_lot::Mutex;
use std::{
    any::Any,
    collections::{BTreeMap, HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc as std_mpsc,
    },
};
use tokio::sync::{mpsc, oneshot, watch};

pub(super) type PrepareFn = Box<
    dyn FnOnce(
            CoordinatorKey,
            Vec<ParticipantSession>,
            CoordinatorLimits,
            Arc<dyn Fn(String) + Send + Sync>,
        ) -> Result<(CoordinatorBinding, CoordinatorTask), CoordinatorError>
        + Send,
>;

/// A host task constructed by a declaration and run by the controller.
pub struct CoordinatorTask {
    run: Box<dyn FnOnce() -> LocalBoxFuture<'static, Result<(), CoordinatorError>> + Send>,
    /// Startup acknowledgement sent only after the provider calls `ready`.
    pub ready: std_mpsc::Receiver<Result<(), String>>,
}

impl CoordinatorTask {
    /// Consume the startup receiver and the factory that must run on the host thread.
    pub fn into_parts(
        self,
    ) -> (
        std_mpsc::Receiver<Result<(), String>>,
        impl FnOnce() -> LocalBoxFuture<'static, Result<(), CoordinatorError>> + Send,
    ) {
        (self.ready, self.run)
    }
}

struct Domain {
    status: Mutex<CoordinatorStatus>,
    admission: Arc<Mutex<AdmissionState>>,
    stopping: AtomicBool,
    stopped: watch::Sender<bool>,
    remaining: AtomicUsize,
    failure: Arc<dyn Fn(String) + Send + Sync>,
    bytes: Arc<AtomicUsize>,
    byte_limit: usize,
}

impl Domain {
    fn fail(&self, message: String) {
        let was_running = {
            let mut state = self.admission.lock();
            std::mem::replace(&mut state.running, false)
        };
        let _ = self.stopped.send_replace(true);
        if was_running && !self.stopping.load(Ordering::Acquire) {
            (self.failure)(message);
        }
    }

    fn begin_shutdown(&self) {
        self.stopping.store(true, Ordering::Release);
        for lease in self.admission.lock().grants.values_mut() {
            lease.accepting = false;
        }
    }

    fn stop(&self) {
        self.begin_shutdown();
        self.fail("coordinator stopped".into());
    }

    fn check(&self) -> Result<(), CoordinatorError> {
        if self.admission.lock().running {
            Ok(())
        } else {
            Err(CoordinatorError::Unavailable)
        }
    }
}

async fn available<T>(
    domain: &Domain,
    future: impl Future<Output = Result<T, CoordinatorError>>,
) -> Result<T, CoordinatorError> {
    let mut stopped = domain.stopped.subscribe();
    domain.check()?;
    tokio::select! {
        biased;
        _ = stopped.changed() => Err(CoordinatorError::Unavailable),
        result = future => result,
    }
}

fn loss_signal(domain: &Domain) -> watch::Receiver<bool> {
    let mut receiver = domain.stopped.subscribe();
    let already_stopped = *receiver.borrow();
    if already_stopped {
        receiver.mark_changed();
    }
    receiver
}

struct Charge {
    bytes: Arc<AtomicUsize>,
    size: usize,
}
impl Drop for Charge {
    fn drop(&mut self) {
        let _ = self.bytes.fetch_sub(self.size, Ordering::AcqRel);
    }
}
struct Envelope<T> {
    // Queue allocation stays independent of a protocol's inline payload size.
    value: Box<T>,
    _charge: Charge,
}
fn charged<T: RetainedBytes>(domain: &Domain, value: T) -> Result<Envelope<T>, CoordinatorError> {
    let size = value
        .retained_bytes()
        .saturating_add(size_of::<Envelope<T>>());
    let _ = domain
        .bytes
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            current
                .checked_add(size)
                .filter(|total| *total <= domain.byte_limit)
        })
        .map_err(|_| CoordinatorError::Capacity)?;
    Ok(Envelope {
        value: Box::new(value),
        _charge: Charge {
            bytes: Arc::clone(&domain.bytes),
            size,
        },
    })
}

trait ErasedBinding: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn domain(&self) -> &Arc<Domain>;
}

struct Binding<P: WorkProtocol> {
    participants: Mutex<HashMap<usize, WorkParticipant<P>>>,
    domain: Arc<Domain>,
}
impl<P: WorkProtocol> ErasedBinding for Binding<P> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn domain(&self) -> &Arc<Domain> {
        &self.domain
    }
}

/// Controller-owned, thread-safe binding for one node/role.
#[derive(Clone)]
pub struct CoordinatorBinding {
    key: CoordinatorKey,
    inner: Arc<dyn ErasedBinding>,
}
impl Debug for CoordinatorBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoordinatorBinding").finish_non_exhaustive()
    }
}
impl CoordinatorBinding {
    /// Read bounded diagnostics without exposing source identifiers.
    #[must_use]
    pub fn status(&self) -> CoordinatorStatus {
        let domain = self.inner.domain();
        let mut status = *domain.status.lock();
        status.queued_message_bytes = domain.bytes.load(Ordering::Acquire);
        status.stopped = !domain.admission.lock().running;
        status
    }

    /// Bind once during node initialization, checking protocol and exact generation.
    pub fn take<P: WorkProtocol>(
        &self,
        core: usize,
        generation: u64,
    ) -> Result<WorkParticipant<P>, CoordinatorError> {
        let binding = self
            .inner
            .as_any()
            .downcast_ref::<Binding<P>>()
            .ok_or(CoordinatorError::Invalid("coordinator protocol mismatch"))?;
        binding.domain.check()?;
        let mut participants = binding.participants.lock();
        let participant = participants.get(&core).ok_or(CoordinatorError::Invalid(
            "participant already bound or unknown core",
        ))?;
        if participant.session.generation != generation {
            return Err(CoordinatorError::Stale);
        }
        let mut participant = participants.remove(&core).expect("checked participant");
        participant.bound = true;
        Ok(participant)
    }

    /// Stop new admissions before normal pipeline drain; existing tickets can settle.
    pub fn begin_shutdown(&self) {
        self.inner.domain().begin_shutdown();
    }
    /// Invalidate all authority and stop the host, including on failed startup.
    pub fn stop(&self) {
        self.inner.domain().stop();
    }
    /// Subscribe to out-of-band authority loss; custom queue saturation cannot mask it.
    #[must_use]
    pub fn authority_loss(&self) -> watch::Receiver<bool> {
        loss_signal(self.inner.domain())
    }
}

/// Logical-pipeline bindings, injected by the controller and looked up only at setup.
#[derive(Clone, Debug, Default)]
pub struct CoordinatorBindings {
    entries: HashMap<(NodeId, String), CoordinatorBinding>,
}
impl CoordinatorBindings {
    /// Insert a prepared host binding; duplicate node/role pairs fail preparation.
    pub fn insert(
        &mut self,
        node: NodeId,
        role: String,
        binding: CoordinatorBinding,
    ) -> Result<(), CoordinatorError> {
        if binding.key.node_id != node || binding.key.role != role {
            return Err(CoordinatorError::Invalid(
                "coordinator binding identity mismatch",
            ));
        }
        if self.entries.contains_key(&(node.clone(), role.clone())) {
            return Err(CoordinatorError::Invalid("duplicate coordinator role"));
        }
        let _ = self.entries.insert((node, role), binding);
        Ok(())
    }
    /// Resolve this node's exact logical-pipeline binding and consume its session.
    pub fn take<P: WorkProtocol>(
        &self,
        group: &PipelineGroupId,
        pipeline: &PipelineId,
        node: &NodeId,
        role: &str,
        core: usize,
        generation: u64,
    ) -> Result<WorkParticipant<P>, CoordinatorError> {
        let binding = self
            .entries
            .get(&(node.clone(), role.into()))
            .ok_or(CoordinatorError::Invalid("undeclared coordinator role"))?;
        if binding.key.group_id != *group || binding.key.pipeline_id != *pipeline {
            return Err(CoordinatorError::Invalid(
                "coordinator binding pipeline mismatch",
            ));
        }
        binding.take::<P>(core, generation)
    }
    /// Stop admission before draining dependent pipelines.
    pub fn begin_shutdown(&self) {
        for binding in self.entries.values() {
            binding.begin_shutdown();
        }
    }
    /// Invalidate all hosts, for failed startup or final teardown.
    pub fn stop(&self) {
        for binding in self.entries.values() {
            binding.stop();
        }
    }
}

/// Assignment state in a complete participant snapshot.
#[derive(Clone, Debug)]
pub enum AssignmentPhase {
    /// An offer is not executable authority.
    Offered(GrantIdentity),
    /// Backend-confirmed grant; acknowledge installation before admitting work.
    Active(AssignmentGrant),
    /// Admission stopped; settle tickets and join workers before releasing.
    Revoking(AssignmentGrant),
}

/// One entry in a participant's current assignment snapshot.
#[derive(Debug)]
pub struct WorkAssignment<P: WorkProtocol> {
    /// Immutable discovered execution metadata.
    pub work: Arc<WorkSpec<P>>,
    /// Offer, active grant or cooperative revocation.
    pub phase: AssignmentPhase,
}

/// Bounded full assignment view. Watch delivery coalesces views, never revocation.
/// Admission validity changes independently of snapshot delivery.
#[derive(Debug)]
pub struct AssignmentSnapshot<P: WorkProtocol> {
    /// Monotonic snapshot revision; consumers replace their entire local view.
    pub revision: u64,
    /// All responsibilities currently offered, active or revoking for this session.
    pub assignments: Vec<WorkAssignment<P>>,
}

/// A participant's engine event or custom protocol notice.
pub enum ParticipantEvent<P: WorkProtocol> {
    /// Complete assignment state, including revocations.
    Assignments(Arc<AssignmentSnapshot<P>>),
    /// Node-defined message; it never grants authority.
    CoordinatorNotice(P::CoordinatorNotice),
}

#[derive(Clone, Copy)]
enum AckKind {
    Ready,
    Accept,
    Activated,
    Released,
}
struct Control {
    session: ParticipantSession,
    kind: AckKind,
    grant: Option<GrantIdentity>,
    receipt: oneshot::Sender<Result<(), CoordinatorError>>,
}
struct Catalog<P: WorkProtocol> {
    works: Box<[WorkSpec<P>]>,
    receipt: oneshot::Sender<Result<(), CoordinatorError>>,
}

/// Typed, exact-session endpoint, consumed once by a node factory.
pub struct WorkParticipant<P: WorkProtocol> {
    session: ParticipantSession,
    domain: Arc<Domain>,
    controls: mpsc::Sender<Control>,
    requests: mpsc::Sender<(ParticipantSession, Envelope<P::ParticipantRequest>)>,
    notices: mpsc::Receiver<Envelope<P::CoordinatorNotice>>,
    assignments: watch::Receiver<Arc<AssignmentSnapshot<P>>>,
    stopped: watch::Receiver<bool>,
    bound: bool,
}

impl<P: WorkProtocol> WorkParticipant<P> {
    /// Out-of-band authority loss, for selecting alongside long-running I/O.
    #[must_use]
    pub fn authority_loss(&self) -> watch::Receiver<bool> {
        loss_signal(&self.domain)
    }

    /// Refresh the full view after a stale offer/acknowledgement. No grant can
    /// become valid again merely because an older view is retained by the node.
    pub fn resynchronize(&mut self) -> Result<Arc<AssignmentSnapshot<P>>, CoordinatorError> {
        self.domain.check()?;
        Ok(self.assignments.borrow_and_update().clone())
    }
    /// This binding's fresh registration identity.
    #[must_use]
    pub fn session(&self) -> ParticipantSession {
        self.session
    }
    async fn acknowledge(
        &self,
        kind: AckKind,
        grant: Option<&GrantIdentity>,
    ) -> Result<(), CoordinatorError> {
        self.domain.check()?;
        if grant.is_some_and(|identity| {
            identity.session != self.session || identity.work_id.0.len() > 1024
        }) {
            return Err(CoordinatorError::Stale);
        }
        let (tx, rx) = oneshot::channel();
        available(&self.domain, async {
            self.controls
                .send(Control {
                    session: self.session,
                    kind,
                    grant: grant.cloned(),
                    receipt: tx,
                })
                .await
                .map_err(|_| CoordinatorError::Unavailable)?;
            rx.await.map_err(|_| CoordinatorError::Unavailable)?
        })
        .await
    }
    /// Signal readiness without waiting for an assignment or nonempty catalog.
    pub async fn ready_for_assignments(&self) -> Result<(), CoordinatorError> {
        self.acknowledge(AckKind::Ready, None).await
    }
    /// Accept an exact offer. This does not itself authorize execution.
    pub async fn accept(&self, offer: &GrantIdentity) -> Result<(), CoordinatorError> {
        self.acknowledge(AckKind::Accept, Some(offer)).await
    }
    /// Confirm local installation of an active grant, enabling admission.
    pub async fn activated(&self, grant: &AssignmentGrant) -> Result<(), CoordinatorError> {
        self.acknowledge(AckKind::Activated, Some(grant.identity()))
            .await
    }
    /// Prove workers stopped and tickets settled; active tickets reject release.
    pub async fn released(&self, grant: &AssignmentGrant) -> Result<(), CoordinatorError> {
        self.acknowledge(AckKind::Released, Some(grant.identity()))
            .await
    }
    /// Send bounded custom metadata without blocking lifecycle control.
    pub fn request(&self, request: P::ParticipantRequest) -> Result<(), CoordinatorError> {
        self.domain.check()?;
        let envelope = charged(&self.domain, request)?;
        self.requests
            .try_send((self.session, envelope))
            .map_err(|e| {
                if matches!(e, mpsc::error::TrySendError::Full(_)) {
                    CoordinatorError::Capacity
                } else {
                    CoordinatorError::Unavailable
                }
            })
    }
    /// Wait for assignment state or a custom notice. Authority loss has priority.
    pub async fn recv(&mut self) -> Result<ParticipantEvent<P>, CoordinatorError> {
        self.domain.check()?;
        tokio::select! {
            biased;
            _ = self.stopped.changed() => Err(CoordinatorError::Unavailable),
            changed = self.assignments.changed() => {
                changed.map_err(|_| CoordinatorError::Unavailable)?;
                Ok(ParticipantEvent::Assignments(self.assignments.borrow_and_update().clone()))
            }
            notice = self.notices.recv() => Ok(ParticipantEvent::CoordinatorNotice(*notice.ok_or(CoordinatorError::Unavailable)?.value)),
        }
    }
}

impl<P: WorkProtocol> Drop for WorkParticipant<P> {
    fn drop(&mut self) {
        if !self.bound {
            return;
        }
        if !self.domain.stopping.load(Ordering::Acquire) {
            self.domain
                .fail("work participant exited unexpectedly".into());
        }
        if self.domain.remaining.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.domain.stop();
        }
    }
}

/// Discovery/protocol endpoint owned by the provider on its independent runtime.
pub struct CoordinatorRuntime<P: WorkProtocol> {
    /// Stable configuration identity for diagnostics and discovery scoping.
    pub key: CoordinatorKey,
    domain: Arc<Domain>,
    ready: Option<std_mpsc::SyncSender<Result<(), String>>>,
    catalogs: mpsc::Sender<Catalog<P>>,
    requests: mpsc::Receiver<(ParticipantSession, Envelope<P::ParticipantRequest>)>,
    notices: HashMap<ParticipantSession, mpsc::Sender<Envelope<P::CoordinatorNotice>>>,
    registry: watch::Receiver<Arc<ParticipantRegistry>>,
    limits: CoordinatorLimits,
}

impl<P: WorkProtocol> CoordinatorRuntime<P> {
    /// Out-of-band final shutdown signal, suitable for selecting alongside `recv`.
    #[must_use]
    pub fn shutdown_signal(&self) -> watch::Receiver<bool> {
        loss_signal(&self.domain)
    }
    /// Wait for final host shutdown. Providers must stop and join their own
    /// discovery workers before returning; the host awaits that cleanup.
    pub async fn shutdown_requested(&self) {
        let mut stopped = self.domain.stopped.subscribe();
        if !*stopped.borrow_and_update() {
            let _ = stopped.changed().await;
        }
    }
    /// Acknowledge provider initialization; an empty catalog is ready.
    pub fn ready(&mut self) -> Result<(), CoordinatorError> {
        self.domain.check()?;
        if let Some(tx) = self.ready.take() {
            tx.send(Ok(())).map_err(|_| CoordinatorError::Unavailable)?;
        }
        Ok(())
    }
    /// Atomically propose a bounded catalog. Existing work metadata is immutable
    /// until removal/release completes. Receipt means accepted, not fully drained.
    pub async fn replace_catalog(&self, works: Vec<WorkSpec<P>>) -> Result<(), CoordinatorError> {
        self.domain.check()?;
        validate_catalog(&works, self.limits)?;
        let (tx, rx) = oneshot::channel();
        available(&self.domain, async {
            self.catalogs
                .send(Catalog {
                    works: works.into_boxed_slice(),
                    receipt: tx,
                })
                .await
                .map_err(|_| CoordinatorError::Unavailable)?;
            rx.await.map_err(|_| CoordinatorError::Unavailable)?
        })
        .await
    }
    /// Current complete participant registry, including initializing instances.
    #[must_use]
    pub fn participants(&self) -> Arc<ParticipantRegistry> {
        self.registry.borrow().clone()
    }
    /// Subscribe to registry changes independently of the custom request receiver.
    #[must_use]
    pub fn participant_registry(&self) -> watch::Receiver<Arc<ParticipantRegistry>> {
        self.registry.clone()
    }
    /// Wait for a readiness/registry update without polling observed admin state.
    pub async fn participants_changed(
        &mut self,
    ) -> Result<Arc<ParticipantRegistry>, CoordinatorError> {
        available(&self.domain, async {
            self.registry
                .changed()
                .await
                .map_err(|_| CoordinatorError::Unavailable)
        })
        .await?;
        Ok(self.registry.borrow_and_update().clone())
    }
    /// Receive a bounded, exact-session custom request.
    pub async fn recv(
        &mut self,
    ) -> Result<(ParticipantSession, P::ParticipantRequest), CoordinatorError> {
        let (session, envelope) = available(&self.domain, async {
            self.requests
                .recv()
                .await
                .ok_or(CoordinatorError::Unavailable)
        })
        .await?;
        Ok((session, *envelope.value))
    }
    /// Send a custom notice to an exact session. It cannot manufacture ownership.
    pub fn notice(
        &self,
        session: ParticipantSession,
        notice: P::CoordinatorNotice,
    ) -> Result<(), CoordinatorError> {
        self.domain.check()?;
        let tx = self.notices.get(&session).ok_or(CoordinatorError::Stale)?;
        tx.try_send(charged(&self.domain, notice)?).map_err(|e| {
            if matches!(e, mpsc::error::TrySendError::Full(_)) {
                CoordinatorError::Capacity
            } else {
                CoordinatorError::Unavailable
            }
        })
    }
}

fn work_bytes<P: WorkProtocol>(work: &WorkSpec<P>) -> usize {
    size_of::<WorkSpec<P>>()
        .saturating_add(work.id.0.capacity())
        .saturating_add(work.work.retained_bytes())
}
fn validate_catalog<P: WorkProtocol>(
    works: &[WorkSpec<P>],
    limits: CoordinatorLimits,
) -> Result<(), CoordinatorError> {
    if works.len() > limits.catalog_items {
        return Err(CoordinatorError::Capacity);
    }
    let mut ids = HashSet::new();
    let mut bytes = 0usize;
    for work in works {
        if work.id.0.is_empty() || work.id.0.len() > 1024 || !ids.insert(&work.id) {
            return Err(CoordinatorError::Invalid(
                "empty, oversized or duplicate work ID",
            ));
        }
        bytes = bytes
            .checked_add(work_bytes(work))
            .ok_or(CoordinatorError::Capacity)?;
        if bytes > limits.catalog_bytes {
            return Err(CoordinatorError::Capacity);
        }
    }
    Ok(())
}

struct Entry<P: WorkProtocol> {
    work: Arc<WorkSpec<P>>,
    desired: bool,
    phase: Option<AssignmentPhase>,
}
struct Fleet<P: WorkProtocol> {
    incarnation: uuid::Uuid,
    domain: Arc<Domain>,
    catalog: BTreeMap<WorkId, Entry<P>>,
    sessions:
        BTreeMap<ParticipantSession, (ParticipantState, watch::Sender<Arc<AssignmentSnapshot<P>>>)>,
    registry: watch::Sender<Arc<ParticipantRegistry>>,
    backend: InMemoryBackend,
    term: u64,
    operation: u64,
    revision: u64,
    limits: CoordinatorLimits,
}

impl<P: WorkProtocol> Fleet<P> {
    fn next_operation(&mut self) -> Result<OperationId, CoordinatorError> {
        self.operation = self
            .operation
            .checked_add(1)
            .ok_or(CoordinatorError::Invalid("operation ID overflow"))?;
        Ok(OperationId(self.operation))
    }
    fn publish(&mut self) -> Result<(), CoordinatorError> {
        *self.domain.status.lock() = CoordinatorStatus {
            participants: self.sessions.len(),
            ready_participants: self
                .sessions
                .values()
                .filter(|(state, _)| *state == ParticipantState::ReadyForAssignments)
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
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(CoordinatorError::Invalid("assignment revision overflow"))?;
        let mut snapshots: BTreeMap<_, Vec<_>> = self
            .sessions
            .keys()
            .map(|session| (*session, Vec::new()))
            .collect();
        for entry in self.catalog.values() {
            if let Some(phase) = &entry.phase {
                let session = match phase {
                    AssignmentPhase::Offered(id) => id.session,
                    AssignmentPhase::Active(grant) | AssignmentPhase::Revoking(grant) => {
                        grant.identity().session
                    }
                };
                snapshots
                    .get_mut(&session)
                    .ok_or(CoordinatorError::Stale)?
                    .push(WorkAssignment {
                        work: Arc::clone(&entry.work),
                        phase: phase.clone(),
                    });
            }
        }
        for (session, (_, tx)) in &self.sessions {
            let _ = tx.send_replace(Arc::new(AssignmentSnapshot {
                revision: self.revision,
                assignments: snapshots.remove(session).expect("session snapshot"),
            }));
        }
        let _ = self.registry.send_replace(Arc::new(ParticipantRegistry {
            participants: self
                .sessions
                .iter()
                .map(|(session, (state, _))| (*session, *state))
                .collect(),
        }));
        Ok(())
    }
    fn allocate(&mut self) -> Result<(), CoordinatorError> {
        if self.domain.stopping.load(Ordering::Acquire)
            || self
                .sessions
                .values()
                .any(|(state, _)| *state != ParticipantState::ReadyForAssignments)
        {
            return self.publish();
        }
        let mut loads: BTreeMap<_, usize> =
            self.sessions.keys().map(|session| (*session, 0)).collect();
        for entry in self.catalog.values() {
            if let Some(phase) = &entry.phase {
                let session = match phase {
                    AssignmentPhase::Offered(id) => id.session,
                    AssignmentPhase::Active(grant) | AssignmentPhase::Revoking(grant) => {
                        grant.identity().session
                    }
                };
                *loads.get_mut(&session).ok_or(CoordinatorError::Stale)? += 1;
            }
        }
        for entry in self
            .catalog
            .values_mut()
            .filter(|entry| entry.desired && entry.phase.is_none())
        {
            let (session, load) = loads
                .iter_mut()
                .min_by_key(|(session, load)| (**load, **session))
                .ok_or(CoordinatorError::Unavailable)?;
            self.term = self
                .term
                .checked_add(1)
                .ok_or(CoordinatorError::Invalid("assignment term overflow"))?;
            entry.phase = Some(AssignmentPhase::Offered(GrantIdentity {
                incarnation: self.incarnation,
                work_id: entry.work.id.clone(),
                session: *session,
                revision: 0,
                term: self.term,
            }));
            *load += 1;
        }
        self.publish()
    }
    fn catalog(&mut self, works: Vec<WorkSpec<P>>) -> Result<(), CoordinatorError> {
        validate_catalog(&works, self.limits)?;
        // Validate the union before mutating: removed active entries must remain
        // retained until cooperative release, even under catalog churn.
        let mut items = self.catalog.len();
        let mut bytes = self
            .catalog
            .values()
            .map(|entry| work_bytes(&entry.work))
            .sum::<usize>();
        for work in &works {
            if let Some(current) = self.catalog.get(&work.id) {
                if current.work.work != work.work {
                    return Err(CoordinatorError::Invalid(
                        "work metadata changed before retirement",
                    ));
                }
            } else {
                items += 1;
                bytes = bytes.saturating_add(work_bytes(work));
            }
        }
        if items > self.limits.catalog_items || bytes > self.limits.catalog_bytes {
            return Err(CoordinatorError::Capacity);
        }
        let desired: HashSet<_> = works.iter().map(|work| work.id.clone()).collect();
        for entry in self.catalog.values_mut() {
            entry.desired = desired.contains(&entry.work.id);
            if !entry.desired {
                match entry.phase.take() {
                    Some(AssignmentPhase::Offered(_)) => {}
                    Some(AssignmentPhase::Active(grant) | AssignmentPhase::Revoking(grant)) => {
                        if let Some(lease) = self
                            .domain
                            .admission
                            .lock()
                            .grants
                            .get_mut(&grant.identity.term)
                        {
                            lease.accepting = false;
                        }
                        entry.phase = Some(AssignmentPhase::Revoking(grant));
                    }
                    None => {}
                }
            }
        }
        self.catalog
            .retain(|_, entry| entry.desired || entry.phase.is_some());
        for work in works {
            let _ = self
                .catalog
                .entry(work.id.clone())
                .or_insert_with(|| Entry {
                    work: Arc::new(work),
                    desired: true,
                    phase: None,
                });
        }
        self.allocate()
    }
    async fn control(&mut self, control: &Control) -> Result<(), CoordinatorError> {
        self.domain.check()?;
        let (state, _) = self
            .sessions
            .get_mut(&control.session)
            .ok_or(CoordinatorError::Stale)?;
        if matches!(control.kind, AckKind::Ready) {
            *state = ParticipantState::ReadyForAssignments;
            return self.allocate();
        }
        let identity = control.grant.as_ref().ok_or(CoordinatorError::Stale)?;
        if identity.session != control.session || identity.incarnation != self.incarnation {
            return Err(CoordinatorError::Stale);
        }
        let phase = self
            .catalog
            .get(&identity.work_id)
            .and_then(|entry| entry.phase.clone())
            .ok_or(CoordinatorError::Stale)?;
        match (control.kind, phase) {
            (AckKind::Accept, AssignmentPhase::Offered(offer)) if offer == *identity => {
                if self.domain.stopping.load(Ordering::Acquire) {
                    return Err(CoordinatorError::Unavailable);
                }
                let operation = self.next_operation()?;
                if self.backend.activate(operation, identity).await? != BackendOutcome::Confirmed {
                    return Err(CoordinatorError::Failed(
                        "activation was not confirmed".into(),
                    ));
                }
                let _ = self
                    .domain
                    .admission
                    .lock()
                    .grants
                    .insert(identity.term, Lease::default());
                self.catalog
                    .get_mut(&identity.work_id)
                    .expect("checked entry")
                    .phase = Some(AssignmentPhase::Active(AssignmentGrant {
                    identity: identity.clone(),
                    admission: Arc::clone(&self.domain.admission),
                }));
            }
            (AckKind::Activated, AssignmentPhase::Active(grant))
                if grant.identity() == identity =>
            {
                let mut state = self.domain.admission.lock();
                if self.domain.stopping.load(Ordering::Acquire) {
                    return Err(CoordinatorError::Unavailable);
                }
                let lease = state
                    .grants
                    .get_mut(&identity.term)
                    .ok_or(CoordinatorError::Stale)?;
                if lease.accepting {
                    return Ok(());
                }
                lease.accepting = true;
            }
            (AckKind::Released, AssignmentPhase::Revoking(grant))
                if grant.identity() == identity =>
            {
                if self
                    .domain
                    .admission
                    .lock()
                    .grants
                    .get(&identity.term)
                    .is_none_or(|lease| lease.tickets != 0)
                {
                    return Err(CoordinatorError::Invalid(
                        "release requires settled tickets and joined workers",
                    ));
                }
                let operation = self.next_operation()?;
                if self.backend.retire(operation, identity).await? != BackendOutcome::Confirmed {
                    return Err(CoordinatorError::Failed(
                        "retirement was not confirmed".into(),
                    ));
                }
                let _ = self.domain.admission.lock().grants.remove(&identity.term);
                self.catalog
                    .get_mut(&identity.work_id)
                    .expect("checked entry")
                    .phase = None;
                self.catalog
                    .retain(|_, entry| entry.desired || entry.phase.is_some());
            }
            _ => return Err(CoordinatorError::Stale),
        }
        self.allocate()
    }
}

struct HostGuard {
    domain: Arc<Domain>,
}
impl Drop for HostGuard {
    fn drop(&mut self) {
        self.domain.fail("work coordinator host exited".into());
    }
}

pub(super) fn prepare<P, F>(
    key: CoordinatorKey,
    sessions: Vec<ParticipantSession>,
    limits: CoordinatorLimits,
    factory: F,
    failure: Arc<dyn Fn(String) + Send + Sync>,
) -> Result<(CoordinatorBinding, CoordinatorTask), CoordinatorError>
where
    P: WorkProtocol,
    F: FnOnce() -> Box<dyn WorkCoordinator<P>> + Send + 'static,
{
    if sessions.is_empty()
        || sessions.len() > limits.participants
        || sessions
            .iter()
            .map(|s| s.core_id)
            .collect::<HashSet<_>>()
            .len()
            != sessions.len()
    {
        return Err(CoordinatorError::Invalid(
            "invalid initial coordinator fleet",
        ));
    }
    let admission = Arc::new(Mutex::new(AdmissionState {
        running: true,
        grants: HashMap::new(),
    }));
    let (stopped, mut stop_rx) = watch::channel(false);
    let domain = Arc::new(Domain {
        status: Mutex::new(CoordinatorStatus {
            participants: sessions.len(),
            ..CoordinatorStatus::default()
        }),
        admission,
        stopping: AtomicBool::new(false),
        stopped,
        remaining: AtomicUsize::new(sessions.len()),
        failure,
        bytes: Arc::new(AtomicUsize::new(0)),
        byte_limit: limits.message_bytes,
    });
    let (control_tx, mut controls) = mpsc::channel(sessions.len() * 2);
    let (request_tx, requests) = mpsc::channel(limits.message_items);
    let (catalog_tx, mut catalogs) = mpsc::channel::<Catalog<P>>(1);
    let (ready_tx, ready_rx) = std_mpsc::sync_channel(1);
    let (registry, registry_rx) = watch::channel(Arc::new(ParticipantRegistry {
        participants: sessions
            .iter()
            .map(|s| (*s, ParticipantState::Initializing))
            .collect(),
    }));
    let mut participants = HashMap::new();
    let mut fleet_sessions = BTreeMap::new();
    let mut notices = HashMap::new();
    for session in sessions {
        let (assign_tx, assignments) = watch::channel(Arc::new(AssignmentSnapshot {
            revision: 0,
            assignments: Vec::new(),
        }));
        let (notice_tx, notice_rx) = mpsc::channel(limits.message_items);
        let _ = notices.insert(session, notice_tx);
        let _ = fleet_sessions.insert(session, (ParticipantState::Initializing, assign_tx));
        let _ = participants.insert(
            session.core_id,
            WorkParticipant {
                session,
                domain: Arc::clone(&domain),
                controls: control_tx.clone(),
                requests: request_tx.clone(),
                notices: notice_rx,
                assignments,
                stopped: domain.stopped.subscribe(),
                bound: false,
            },
        );
    }
    let binding = CoordinatorBinding {
        key: key.clone(),
        inner: Arc::new(Binding::<P> {
            participants: Mutex::new(participants),
            domain: Arc::clone(&domain),
        }),
    };
    let guard = HostGuard {
        domain: Arc::clone(&domain),
    };
    let task = CoordinatorTask {
        ready: ready_rx,
        run: Box::new(move || {
            Box::pin(async move {
                let _guard = guard;
                let runtime = CoordinatorRuntime {
                    key,
                    domain: Arc::clone(&domain),
                    ready: Some(ready_tx),
                    catalogs: catalog_tx,
                    requests,
                    notices,
                    registry: registry_rx,
                    limits,
                };
                let mut fleet = Fleet::<P> {
                    incarnation: uuid::Uuid::new_v4(),
                    domain: Arc::clone(&domain),
                    catalog: BTreeMap::new(),
                    sessions: fleet_sessions,
                    registry,
                    backend: InMemoryBackend::new(limits.catalog_items),
                    term: 0,
                    operation: 0,
                    revision: 0,
                    limits,
                };
                let provider = factory().run(runtime);
                tokio::pin!(provider);
                loop {
                    tokio::select! {
                        biased;
                        _ = stop_rx.changed() => return match provider.await {
                            Err(CoordinatorError::Unavailable) if domain.stopping.load(Ordering::Acquire) => Ok(()),
                            result => result,
                        },
                        result = &mut provider => return match result {
                            Ok(()) | Err(CoordinatorError::Unavailable)
                                if domain.stopping.load(Ordering::Acquire) && *stop_rx.borrow() => Ok(()),
                            Ok(()) => Err(CoordinatorError::Failed("provider returned before shutdown".into())),
                            Err(error) => Err(error),
                        },
                        control = controls.recv() => {
                            let control = control.ok_or(CoordinatorError::Unavailable)?;
                            let result = fleet.control(&control).await;
                            if matches!(result, Err(CoordinatorError::Failed(_))) { return result; }
                            let _ = control.receipt.send(result);
                        }
                        catalog = catalogs.recv() => {
                            let catalog = catalog.ok_or(CoordinatorError::Unavailable)?;
                            let result = fleet.catalog(catalog.works.into_vec());
                            let _ = catalog.receipt.send(result);
                        }
                    }
                    // Bounded control/catalog processing must leave discovery and I/O runnable.
                    tokio::task::yield_now().await;
                }
            })
        }),
    };
    Ok((binding, task))
}
