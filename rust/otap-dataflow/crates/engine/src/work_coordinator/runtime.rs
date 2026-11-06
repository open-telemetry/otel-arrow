// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::lifecycle::ReceiptCompleter;
use super::{
    authority::{AdmissionState, Lease},
    *,
};
use futures::{Future, future::LocalBoxFuture};
use std::sync::atomic::AtomicU64;
use tokio::sync::Notify;
mod fleet;
use fleet::Fleet;
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
            CapabilityProfile,
            Arc<serde_json::Value>,
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

struct Member {
    session: ParticipantSession,
    revision: u64,
    alive: AtomicBool,
    bound: AtomicBool,
    owned: AtomicUsize,
}

struct Domain {
    profile: CapabilityProfile,
    members: Mutex<BTreeMap<ParticipantSession, Arc<Member>>>,
    nonce: AtomicU64,
    operation: AtomicU64,
    active_operation: AtomicU64,
    commit_decision: AtomicU64,
    current_revision: AtomicU64,
    revision_counter: AtomicU64,
    revisions: Mutex<BTreeMap<u64, Arc<serde_json::Value>>>,
    exits: Notify,
    limits: CoordinatorLimits,
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
    fn prepare(
        &self,
        settings: Arc<serde_json::Value>,
    ) -> Result<ReconciliationReceipt, CoordinatorError>;
    fn register(
        &self,
        spec: ParticipantSpec,
        revision: u64,
    ) -> Result<ReconciliationReceipt, CoordinatorError>;
    fn activate(
        &self,
        spec: ParticipantSpec,
        departing: bool,
    ) -> Result<ReconciliationReceipt, CoordinatorError>;
    fn finish(
        &self,
        operation: OperationId,
        commit: bool,
    ) -> Result<ReconciliationReceipt, CoordinatorError>;
    fn confirm_exit(&self, spec: ParticipantSpec) -> bool;
}

struct Binding<P: WorkProtocol> {
    participants: Arc<Mutex<HashMap<(usize, u64), WorkParticipant<P>>>>,
    domain: Arc<Domain>,
    controls: mpsc::Sender<HostControl<P>>,
    requests: mpsc::Sender<(ParticipantSession, Envelope<P::ParticipantRequest>)>,
    notices: Notices<P>,
    final_receipt: Mutex<Option<(OperationId, bool, ReconciliationReceipt)>>,
}
type Notices<P> = Arc<
    Mutex<
        HashMap<ParticipantSession, mpsc::Sender<Envelope<<P as WorkProtocol>::CoordinatorNotice>>>,
    >,
>;

struct Port<P: WorkProtocol> {
    member: Arc<Member>,
    assignments: watch::Sender<Arc<AssignmentSnapshot<P>>>,
}

enum HostControl<P: WorkProtocol> {
    Participant(Control),
    Register(Port<P>, ReceiptCompleter),
    Prepare(Arc<serde_json::Value>, ReceiptCompleter),
    Activate(ParticipantSpec, bool, ReceiptCompleter),
    Finish(OperationId, bool, ReceiptCompleter),
}

fn next_operation(domain: &Domain) -> Result<OperationId, CoordinatorError> {
    domain.check()?;
    domain
        .operation
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
            value.checked_add(1)
        })
        .map(|value| OperationId(value + 1))
        .map_err(|_| CoordinatorError::Failed("coordinator operation overflow".into()))
}

impl<P: WorkProtocol> ErasedBinding for Binding<P> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn domain(&self) -> &Arc<Domain> {
        &self.domain
    }
    fn prepare(
        &self,
        settings: Arc<serde_json::Value>,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.domain.check()?;
        let _ = lifecycle::settings_bytes(&settings)?;
        let permit = self
            .controls
            .try_reserve()
            .map_err(|_| CoordinatorError::Capacity)?;
        let mut status = self.domain.status.lock();
        if self.domain.profile != CapabilityProfile::Cooperative
            || status.reconciliation.is_some_and(|(_, phase)| {
                !matches!(
                    phase,
                    ReconciliationPhase::Committed | ReconciliationPhase::Aborted
                )
            })
        {
            return Err(CoordinatorError::Invalid(
                "coordinator reconciliation already pending or unsupported",
            ));
        }
        let operation = next_operation(&self.domain)?;
        self.domain
            .active_operation
            .store(operation.0, Ordering::Release);
        self.domain.commit_decision.store(0, Ordering::Release);
        status.reconciliation = Some((operation, ReconciliationPhase::Preparing));
        let (receipt, completer) = ReconciliationReceipt::pair(operation);
        permit.send(HostControl::Prepare(settings, completer));
        Ok(receipt)
    }
    fn register(
        &self,
        spec: ParticipantSpec,
        revision: u64,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.domain.check()?;
        if self.domain.profile != CapabilityProfile::Cooperative
            || self.domain.stopping.load(Ordering::Acquire)
            || !self.domain.revisions.lock().contains_key(&revision)
        {
            return Err(CoordinatorError::Invalid(
                "cannot register this coordinator revision",
            ));
        }
        let permit = self
            .controls
            .try_reserve()
            .map_err(|_| CoordinatorError::Capacity)?;
        let mut members = self.domain.members.lock();
        let mut participants = self.participants.lock();
        if members.values().any(|member| {
            member.session.core_id == spec.core_id
                && member.session.generation == spec.generation
                && member.alive.load(Ordering::Acquire)
        }) {
            return Err(CoordinatorError::Invalid(
                "participant instance already registered",
            ));
        }
        if members.len() >= self.domain.limits.participants * 2 {
            return Err(CoordinatorError::Capacity);
        }
        let nonce = self
            .domain
            .nonce
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(1)
            })
            .map_err(|_| CoordinatorError::Failed("participant nonce overflow".into()))?
            + 1;
        let session = ParticipantSession {
            core_id: spec.core_id,
            generation: spec.generation,
            nonce,
        };
        let (participant, port) = endpoint(
            session,
            revision,
            &self.domain,
            &self.controls,
            &self.requests,
            &self.notices,
        );
        let _ = members.insert(session, Arc::clone(&port.member));
        let _ = participants.insert((spec.core_id, spec.generation), participant);
        let (receipt, completer) = ReconciliationReceipt::pair(next_operation(&self.domain)?);
        permit.send(HostControl::Register(port, completer));
        Ok(receipt)
    }
    fn activate(
        &self,
        spec: ParticipantSpec,
        departing: bool,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.domain.check()?;
        let permit = self
            .controls
            .try_reserve()
            .map_err(|_| CoordinatorError::Capacity)?;
        let (receipt, completer) = ReconciliationReceipt::pair(next_operation(&self.domain)?);
        permit.send(HostControl::Activate(spec, departing, completer));
        Ok(receipt)
    }
    fn finish(
        &self,
        operation: OperationId,
        commit: bool,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.domain.check()?;
        let mut cached = self.final_receipt.lock();
        if let Some((id, kind, receipt)) = &*cached
            && *id == operation
            && *kind == commit
        {
            return Ok(receipt.clone());
        }
        let permit = self
            .controls
            .try_reserve()
            .map_err(|_| CoordinatorError::Capacity)?;
        let mut status = self.domain.status.lock();
        let Some((id, phase)) = status.reconciliation else {
            return Err(CoordinatorError::Stale);
        };
        if id != operation {
            return Err(CoordinatorError::Stale);
        }
        if commit {
            if !matches!(
                phase,
                ReconciliationPhase::Activated
                    | ReconciliationPhase::CommitDecided
                    | ReconciliationPhase::Committed
            ) {
                return Err(CoordinatorError::Invalid(
                    "commit requires acknowledged activation",
                ));
            }
            self.domain
                .commit_decision
                .store(operation.0, Ordering::Release);
            status.reconciliation = Some((id, ReconciliationPhase::CommitDecided));
        } else if self.domain.commit_decision.load(Ordering::Acquire) == operation.0 {
            return Err(CoordinatorError::Invalid(
                "committed reconciliation cannot abort",
            ));
        }
        let (receipt, completer) = ReconciliationReceipt::pair(operation);
        *cached = Some((operation, commit, receipt.clone()));
        permit.send(HostControl::Finish(operation, commit, completer));
        Ok(receipt)
    }
    fn confirm_exit(&self, spec: ParticipantSpec) -> bool {
        let mut members = self.domain.members.lock();
        let sessions: Vec<_> = members
            .iter()
            .filter(|(session, _)| {
                session.core_id == spec.core_id && session.generation == spec.generation
            })
            .map(|(session, _)| *session)
            .collect();
        if sessions.is_empty() {
            return false;
        }
        let clean = sessions.iter().all(|session| {
            let member = &members[session];
            member.owned.load(Ordering::Acquire) == 0
                && (!member.bound.load(Ordering::Acquire) || !member.alive.load(Ordering::Acquire))
        });
        if clean {
            for session in sessions {
                if let Some(member) = members.remove(&session) {
                    member.alive.store(false, Ordering::Release);
                }
            }
            let _ = self
                .participants
                .lock()
                .remove(&(spec.core_id, spec.generation));
            self.domain.exits.notify_one();
        }
        clean
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
    /// Declared lifecycle profile.
    #[must_use]
    pub fn profile(&self) -> CapabilityProfile {
        self.inner.domain().profile
    }
    /// Latest committed settings revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.inner.domain().current_revision.load(Ordering::Acquire)
    }
    /// Stage compatible settings without changing committed participants.
    pub fn prepare_reconciliation(
        &self,
        settings: Arc<serde_json::Value>,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.inner.prepare(settings)
    }
    /// Register an exact, initially ineligible instance before node construction.
    pub fn register_instance(
        &self,
        spec: ParticipantSpec,
        revision: u64,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.inner.register(spec, revision)
    }
    /// Transfer work to a ready instance, retiring its predecessor on the same core.
    pub fn activate_instance(
        &self,
        spec: ParticipantSpec,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.inner.activate(spec, false)
    }
    /// Release a departing instance before draining its downstream completion path.
    pub fn retire_instance(
        &self,
        spec: ParticipantSpec,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.inner.activate(spec, true)
    }
    /// Latch the irreversible commit decision, then finalize provider cleanup.
    pub fn commit(
        &self,
        operation: OperationId,
    ) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.inner.finish(operation, true)
    }
    /// Restore old eligibility/settings using fresh grants after candidate cleanup.
    pub fn abort(&self, operation: OperationId) -> Result<ReconciliationReceipt, CoordinatorError> {
        self.inner.finish(operation, false)
    }
    /// Called only after the runtime physically exited; proves all owned work was released.
    #[must_use]
    pub fn confirm_instance_exit(&self, spec: ParticipantSpec) -> bool {
        self.inner.confirm_exit(spec)
    }
    /// Whether an unconsumed endpoint is already prepared for this instance.
    #[must_use]
    pub fn has_pending_instance(&self, spec: ParticipantSpec) -> bool {
        self.inner.domain().members.lock().values().any(|member| {
            member.session.core_id == spec.core_id
                && member.session.generation == spec.generation
                && member.alive.load(Ordering::Acquire)
                && !member.bound.load(Ordering::Acquire)
        })
    }
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
        if !participants.contains_key(&(core, generation))
            && participants
                .keys()
                .any(|(known_core, _)| *known_core == core)
        {
            return Err(CoordinatorError::Stale);
        }
        let participant =
            participants
                .get(&(core, generation))
                .ok_or(CoordinatorError::Invalid(
                    "participant already bound or unknown core",
                ))?;
        if participant.session.generation != generation {
            return Err(CoordinatorError::Stale);
        }
        let mut participant = participants
            .remove(&(core, generation))
            .expect("checked participant");
        participant.bound = true;
        participant.member.bound.store(true, Ordering::Release);
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
    /// Iterate declared bindings during controller lifecycle operations.
    pub fn iter(&self) -> impl Iterator<Item = (&(NodeId, String), &CoordinatorBinding)> {
        self.entries.iter()
    }

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
    revision: u64,
    works: Box<[WorkSpec<P>]>,
    receipt: oneshot::Sender<Result<(), CoordinatorError>>,
}

/// Typed, exact-session endpoint, consumed once by a node factory.
pub struct WorkParticipant<P: WorkProtocol> {
    session: ParticipantSession,
    domain: Arc<Domain>,
    controls: mpsc::Sender<HostControl<P>>,
    member: Arc<Member>,
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
        if !self.member.alive.load(Ordering::Acquire) {
            return Err(CoordinatorError::Stale);
        }
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
        if !self.member.alive.load(Ordering::Acquire) {
            return Err(CoordinatorError::Stale);
        }
        if grant.is_some_and(|identity| {
            identity.session != self.session || identity.work_id.0.len() > 1024
        }) {
            return Err(CoordinatorError::Stale);
        }
        let (tx, rx) = oneshot::channel();
        available(&self.domain, async {
            self.controls
                .send(HostControl::Participant(Control {
                    session: self.session,
                    kind,
                    grant: grant.cloned(),
                    receipt: tx,
                }))
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
        if !self.member.alive.load(Ordering::Acquire) {
            return Err(CoordinatorError::Stale);
        }
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
        if !self.member.alive.load(Ordering::Acquire) {
            return Err(CoordinatorError::Stale);
        }
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
        self.member.alive.store(false, Ordering::Release);
        if !self.domain.stopping.load(Ordering::Acquire)
            && (self.domain.profile == CapabilityProfile::Basic
                || self.member.owned.load(Ordering::Acquire) != 0)
        {
            self.domain
                .fail("work participant exited without acknowledged release".into());
        }
        self.domain.exits.notify_one();
        if self.domain.profile == CapabilityProfile::Basic
            && self.domain.remaining.fetch_sub(1, Ordering::AcqRel) == 1
        {
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
    notices: Notices<P>,
    lifecycle: Option<mpsc::Receiver<CoordinatorLifecycleEvent>>,
    registry: watch::Receiver<Arc<ParticipantRegistry>>,
    limits: CoordinatorLimits,
}

impl<P: WorkProtocol> CoordinatorRuntime<P> {
    /// Take the dedicated lifecycle stream once, independently of custom requests.
    pub fn lifecycle_events(
        &mut self,
    ) -> Result<mpsc::Receiver<CoordinatorLifecycleEvent>, CoordinatorError> {
        self.lifecycle
            .take()
            .ok_or(CoordinatorError::Invalid("lifecycle stream already taken"))
    }
    /// Read immutable settings for a live or staged revision.
    pub fn settings(&self, revision: u64) -> Result<Arc<serde_json::Value>, CoordinatorError> {
        self.domain
            .revisions
            .lock()
            .get(&revision)
            .cloned()
            .ok_or(CoordinatorError::Stale)
    }
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
        self.replace_catalog_for(0, works).await
    }
    /// Publish a catalog for one immutable settings revision. Old and candidate
    /// catalogs coexist until commit or abort is acknowledged.
    pub async fn replace_catalog_for(
        &self,
        revision: u64,
        works: Vec<WorkSpec<P>>,
    ) -> Result<(), CoordinatorError> {
        self.domain.check()?;
        validate_catalog(&works, self.limits)?;
        let (tx, rx) = oneshot::channel();
        available(&self.domain, async {
            self.catalogs
                .send(Catalog {
                    revision,
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
        let notices = self.notices.lock();
        let tx = notices.get(&session).ok_or(CoordinatorError::Stale)?;
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

struct HostGuard {
    domain: Arc<Domain>,
}
impl Drop for HostGuard {
    fn drop(&mut self) {
        self.domain.fail("work coordinator host exited".into());
    }
}

fn endpoint<P: WorkProtocol>(
    session: ParticipantSession,
    revision: u64,
    domain: &Arc<Domain>,
    controls: &mpsc::Sender<HostControl<P>>,
    requests: &mpsc::Sender<(ParticipantSession, Envelope<P::ParticipantRequest>)>,
    notices: &Notices<P>,
) -> (WorkParticipant<P>, Port<P>) {
    let member = Arc::new(Member {
        session,
        revision,
        alive: AtomicBool::new(true),
        bound: AtomicBool::new(false),
        owned: AtomicUsize::new(0),
    });
    let (assignments_tx, assignments) = watch::channel(Arc::new(AssignmentSnapshot {
        revision: 0,
        assignments: Vec::new(),
    }));
    let (notice_tx, notice_rx) = mpsc::channel(domain.limits.message_items);
    let _ = notices.lock().insert(session, notice_tx);
    let participant = WorkParticipant {
        session,
        member: Arc::clone(&member),
        domain: Arc::clone(domain),
        controls: controls.clone(),
        requests: requests.clone(),
        notices: notice_rx,
        assignments,
        stopped: domain.stopped.subscribe(),
        bound: false,
    };
    (
        participant,
        Port {
            member,
            assignments: assignments_tx,
        },
    )
}

pub(super) fn prepare<P, F>(
    key: CoordinatorKey,
    sessions: Vec<ParticipantSession>,
    limits: CoordinatorLimits,
    profile: CapabilityProfile,
    settings: Arc<serde_json::Value>,
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
    let (stopped, mut stop_rx) = watch::channel(false);
    let domain = Arc::new(Domain {
        profile,
        members: Mutex::new(BTreeMap::new()),
        nonce: AtomicU64::new(sessions.iter().map(|s| s.nonce).max().unwrap_or(0)),
        operation: AtomicU64::new(0),
        active_operation: AtomicU64::new(0),
        commit_decision: AtomicU64::new(0),
        current_revision: AtomicU64::new(0),
        revision_counter: AtomicU64::new(0),
        revisions: Mutex::new(BTreeMap::from([(0, settings)])),
        exits: Notify::new(),
        limits,
        status: Mutex::new(CoordinatorStatus {
            participants: sessions.len(),
            ..CoordinatorStatus::default()
        }),
        admission: Arc::new(Mutex::new(AdmissionState {
            running: true,
            grants: HashMap::new(),
        })),
        stopping: AtomicBool::new(false),
        stopped,
        remaining: AtomicUsize::new(sessions.len()),
        failure,
        bytes: Arc::new(AtomicUsize::new(0)),
        byte_limit: limits.message_bytes,
    });
    let (control_tx, mut controls) = mpsc::channel(limits.participants * 4);
    let (request_tx, requests) = mpsc::channel(limits.message_items);
    let (catalog_tx, mut catalogs) = mpsc::channel::<Catalog<P>>(1);
    let (ready_tx, ready_rx) = std_mpsc::sync_channel(1);
    let (lifecycle_tx, lifecycle_rx) = mpsc::channel(2);
    let (registry, registry_rx) = watch::channel(Arc::new(ParticipantRegistry {
        revision: 0,
        participants: sessions
            .iter()
            .map(|s| (*s, ParticipantState::Initializing))
            .collect(),
    }));
    let participants = Arc::new(Mutex::new(HashMap::new()));
    let notices = Arc::new(Mutex::new(HashMap::new()));
    let mut ports = Vec::new();
    for session in sessions {
        let (participant, port) = endpoint(session, 0, &domain, &control_tx, &request_tx, &notices);
        let _ = domain
            .members
            .lock()
            .insert(session, Arc::clone(&port.member));
        let _ = participants
            .lock()
            .insert((session.core_id, session.generation), participant);
        ports.push(port);
    }
    let binding = CoordinatorBinding {
        key: key.clone(),
        inner: Arc::new(Binding::<P> {
            participants: Arc::clone(&participants),
            domain: Arc::clone(&domain),
            controls: control_tx,
            requests: request_tx,
            notices: Arc::clone(&notices),
            final_receipt: Mutex::new(None),
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
                    notices: Arc::clone(&notices),
                    lifecycle: Some(lifecycle_rx),
                    registry: registry_rx,
                    limits,
                };
                let mut fleet = Fleet::new(
                    Arc::clone(&domain),
                    ports,
                    registry,
                    lifecycle_tx,
                    notices,
                    participants,
                );
                let provider = factory().run(runtime);
                tokio::pin!(provider);
                loop {
                    tokio::select! {
                        biased;
                        _ = stop_rx.changed() => return match provider.await { Err(CoordinatorError::Unavailable) if domain.stopping.load(Ordering::Acquire) => Ok(()), result => result },
                        _ = domain.exits.notified() => { fleet.sweep_exits()?; }
                        control = controls.recv() => { fleet.command(control.ok_or(CoordinatorError::Unavailable)?).await?; }
                        result = async { match &mut fleet.provider_pending { Some(pending) => pending.await, None => std::future::pending().await } } => { fleet.provider_completed(result)?; }
                        result = &mut provider => return match result {
                            Ok(()) | Err(CoordinatorError::Unavailable)
                                if domain.stopping.load(Ordering::Acquire) && *stop_rx.borrow() => Ok(()),
                            Ok(()) => Err(CoordinatorError::Failed("provider returned before shutdown".into())),
                            Err(error) => Err(error),
                        },
                        catalog = catalogs.recv() => {
                            let catalog = catalog.ok_or(CoordinatorError::Unavailable)?;
                            let result = fleet.catalog(catalog.revision, catalog.works.into_vec());
                            let _ = catalog.receipt.send(result);
                        }
                    }
                    fleet.progress()?;
                    tokio::task::yield_now().await;
                }
            })
        }),
    };
    Ok((binding, task))
}
