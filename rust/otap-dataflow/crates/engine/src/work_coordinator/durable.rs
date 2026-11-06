// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded, independent durable authority and publication-progress writer.

use super::runtime::Domain;
use super::{
    AdmissionTicket, AssignmentBackend, BackendOutcome, CoordinatorError, GrantIdentity,
    OperationId,
};
use async_trait::async_trait;
use otel_arrow_dfe_work_coordinator_state as state;
use parking_lot::{Condvar, Mutex};
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
    thread::JoinHandle,
};
use tokio::sync::oneshot;

/// Storage admission bounds, independent of catalog and custom-message budgets.
pub use state::Limits as DurableLimits;
/// Durable progress receipt identity, retained independently of caller futures.
pub use state::OperationId as ProgressOperationId;
/// Definite or unresolved result of one atomic publication.
pub use state::Outcome as ProgressOutcome;
/// Committed, opaque source progress.
pub use state::Progress as SourceProgress;

/// Source execution policy; durable checkpoints alone never authorize takeover.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TakeoverPolicy {
    /// Require physical worker release before a successor can execute.
    #[default]
    CooperativeOnly,
    /// Reserved until the source publication/state adapter has been qualified.
    AllowFencedTakeover,
}

/// Node-declared local storage. The root must be provisioned privately before startup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurableOptions {
    /// Existing absolute 0700 root; its ancestors must be protected by the provisioner.
    pub root: PathBuf,
    /// Stable source namespace, independent of participant placement and settings revision.
    pub namespace: String,
    /// Persistent state, transaction and receipt bounds.
    pub limits: DurableLimits,
    /// Queued publication/read/resolve slots. Authority has a separate reserved queue.
    pub queue_items: usize,
    /// Shared retained-byte budget, including the in-flight request.
    pub queue_bytes: usize,
    /// Physical exclusion remains the default even with durable authority.
    pub takeover: TakeoverPolicy,
}
impl DurableOptions {
    /// Conservative defaults for a dedicated local writer.
    #[must_use]
    pub fn new(root: PathBuf, namespace: String) -> Self {
        Self {
            root,
            namespace,
            limits: DurableLimits::default(),
            queue_items: 32,
            queue_bytes: 4 * 1024 * 1024,
            takeover: TakeoverPolicy::CooperativeOnly,
        }
    }
    /// Side-effect-free capability and bounds validation.
    pub fn validate(&self) -> Result<(), CoordinatorError> {
        if !state::AVAILABLE {
            return Err(CoordinatorError::Invalid(
                "durable authority requires the durable-authority feature on Linux",
            ));
        }
        let _ = self.limits.validate().map_err(storage_error)?;
        if !self.root.is_absolute()
            || self.root.as_os_str().len() > 4096
            || self.namespace.is_empty()
            || self.namespace.len() > 1024
            || self.namespace.contains('\0')
            || self.queue_items == 0
            || self.queue_items > 1024
            || self.queue_bytes == 0
            || self.queue_bytes > 64 * 1024 * 1024
            || self.limits.receipts < self.queue_items * 2 + 8
            || self.takeover != TakeoverPolicy::CooperativeOnly
        {
            return Err(CoordinatorError::Invalid(
                "unsupported durable authority capabilities or limits",
            ));
        }
        Ok(())
    }
}

/// One source update, retaining its admitted operation through the storage outcome.
/// The source adapter must establish Ack/loss authorization before constructing it.
pub struct ProgressMutation {
    ticket: Arc<AdmissionTicket>,
    source: String,
    expected_revision: u64,
    bytes: Vec<u8>,
}
impl ProgressMutation {
    /// Reuse an admitted publication ticket for several source records via `Arc`.
    #[must_use]
    pub fn new(
        ticket: Arc<AdmissionTicket>,
        source: String,
        expected_revision: u64,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            ticket,
            source,
            expected_revision,
            bytes,
        }
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.source.capacity())
            .saturating_add(self.bytes.capacity())
            .saturating_add(self.ticket.grant().work_id.0.capacity())
            .saturating_add(size_of::<AdmissionTicket>())
            .saturating_add(size_of::<state::ProgressUpdate>() + 80)
    }
}

enum Payload {
    Activate(state::Grant),
    Retire(state::Grant),
    Commit(Vec<ProgressMutation>),
    Resolve(ProgressOperationId),
    Read(String),
}
impl Payload {
    fn bytes(&self) -> usize {
        size_of::<Job>()
            .saturating_add(128)
            .saturating_add(match self {
                Self::Activate(g) | Self::Retire(g) => {
                    g.work.capacity().saturating_add(g.token.capacity())
                }
                Self::Commit(updates) => updates
                    .capacity()
                    .saturating_mul(size_of::<ProgressMutation>())
                    .saturating_add(
                        updates
                            .iter()
                            .map(ProgressMutation::retained_bytes)
                            .sum::<usize>(),
                    ),
                Self::Read(key) => key.capacity(),
                Self::Resolve(_) => 0,
            })
    }
}
enum Response {
    Outcome(ProgressOutcome),
    Progress(Option<SourceProgress>),
}
struct Job {
    operation: ProgressOperationId,
    payload: Payload,
    result: oneshot::Sender<Result<Response, CoordinatorError>>,
    bytes: usize,
    authority: bool,
}
struct Queues {
    authority: VecDeque<Job>,
    progress: VecDeque<Job>,
    progress_bytes: usize,
    authority_bytes: usize,
    sequence: u64,
    closing: bool,
}
struct Shared {
    queues: Mutex<Queues>,
    available: Condvar,
    options: DurableOptions,
    epoch: AtomicU64,
    domain: Weak<Domain>,
    #[cfg(all(test, target_os = "linux"))]
    pause: Mutex<Option<Arc<TestPause>>>,
    #[cfg(all(test, target_os = "linux"))]
    drop_reply: Mutex<Option<TestKind>>,
    #[cfg(all(test, target_os = "linux"))]
    changed: tokio::sync::Notify,
}
impl Shared {
    fn submit(
        &self,
        payload: Payload,
        authority: bool,
    ) -> Result<
        (
            ProgressOperationId,
            oneshot::Receiver<Result<Response, CoordinatorError>>,
        ),
        CoordinatorError,
    > {
        self.domain
            .upgrade()
            .ok_or(CoordinatorError::Unavailable)?
            .check()?;
        let bytes = payload.bytes();
        let mut queues = self.queues.lock();
        if queues.closing {
            return Err(CoordinatorError::Unavailable);
        }
        let full = if authority {
            queues.authority.len() >= 2
                || bytes > (16usize * 1024).saturating_sub(queues.authority_bytes)
        } else {
            queues.progress.len() >= self.options.queue_items
                || bytes
                    > self
                        .options
                        .queue_bytes
                        .saturating_sub(queues.progress_bytes)
        };
        if full {
            return Err(CoordinatorError::Capacity);
        }
        queues.sequence = queues
            .sequence
            .checked_add(1)
            .filter(|n| *n <= i64::MAX as u64)
            .ok_or(CoordinatorError::Capacity)?;
        let operation = ProgressOperationId {
            epoch: self.epoch.load(Ordering::Acquire),
            sequence: queues.sequence,
        };
        let (tx, rx) = oneshot::channel();
        if authority {
            queues.authority_bytes += bytes;
        } else {
            queues.progress_bytes += bytes;
        }
        let job = Job {
            operation,
            payload,
            result: tx,
            bytes,
            authority,
        };
        if authority {
            queues.authority.push_back(job);
        } else {
            queues.progress.push_back(job);
        }
        #[cfg(all(test, target_os = "linux"))]
        self.changed.notify_waiters();
        let _ = self.available.notify_one();
        Ok((operation, rx))
    }
    fn close(&self) {
        let mut queues = self.queues.lock();
        queues.closing = true;
        queues.authority_bytes -= queues.authority.iter().map(|job| job.bytes).sum::<usize>();
        queues.progress_bytes -= queues.progress.iter().map(|job| job.bytes).sum::<usize>();
        queues.authority.clear();
        queues.progress.clear();
        #[cfg(all(test, target_os = "linux"))]
        if let Some(pause) = self.pause.lock().clone() {
            pause.release();
        }
        let _ = self.available.notify_all();
    }
}

/// Completion of a submitted publication. Dropping it does not cancel storage.
pub struct ProgressReceipt {
    operation: ProgressOperationId,
    result: oneshot::Receiver<Result<Response, CoordinatorError>>,
}
impl ProgressReceipt {
    /// Save this ID before awaiting, so a lost reply can be resolved later.
    #[must_use]
    pub fn operation(&self) -> ProgressOperationId {
        self.operation
    }
    /// Wait for the definite storage outcome; cancellation leaves the outcome unknown.
    pub async fn wait(self) -> Result<ProgressOutcome, CoordinatorError> {
        outcome(self.result).await
    }
}
async fn outcome(
    result: oneshot::Receiver<Result<Response, CoordinatorError>>,
) -> Result<ProgressOutcome, CoordinatorError> {
    match result.await {
        Err(_) => Ok(ProgressOutcome::Unknown),
        Ok(result) => match result? {
            Response::Outcome(value) => Ok(value),
            _ => Err(CoordinatorError::Failed(
                "invalid progress writer receipt".into(),
            )),
        },
    }
}

/// Cloneable progress path. Publications never pass through the coordinator loop.
#[derive(Clone)]
pub struct ProgressWriter {
    shared: Arc<Shared>,
}
/// Bounded writer diagnostics, without namespace names or source identifiers.
#[derive(Clone, Copy, Debug)]
pub struct WriterStatus {
    /// Queued authority commands, excluding the current transaction.
    pub authority_items: usize,
    /// Queued source progress/read/resolve commands, excluding the current transaction.
    pub progress_items: usize,
    /// Authority queue and in-flight byte charge, in a reserved 16 KiB budget.
    pub authority_bytes: usize,
    /// Publication queue and in-flight retained byte charge.
    pub progress_bytes: usize,
    /// This writer has entered final shutdown.
    pub closed: bool,
}
impl ProgressWriter {
    /// Inspect independent item/byte budgets, including in-flight byte charges.
    #[must_use]
    pub fn status(&self) -> WriterStatus {
        let q = self.shared.queues.lock();
        WriterStatus {
            authority_items: q.authority.len(),
            progress_items: q.progress.len(),
            authority_bytes: q.authority_bytes,
            progress_bytes: q.progress_bytes,
            closed: q.closing,
        }
    }
    /// Submit one bounded all-or-none publication while retaining all contributing tickets.
    pub fn commit(
        &self,
        updates: Vec<ProgressMutation>,
    ) -> Result<ProgressReceipt, CoordinatorError> {
        let domain = self
            .shared
            .domain
            .upgrade()
            .ok_or(CoordinatorError::Unavailable)?;
        if updates.is_empty() || updates.len() > self.shared.options.limits.transaction_items {
            return Err(CoordinatorError::Capacity);
        }
        for update in &updates {
            if !Arc::ptr_eq(&update.ticket.grant.admission, &domain.admission)
                || !update.ticket.is_valid()
            {
                return Err(CoordinatorError::Stale);
            }
            if update.source.is_empty()
                || update.source.len() > 1024
                || update.source.contains('\0')
                || update.bytes.len() > self.shared.options.limits.source_bytes
            {
                return Err(CoordinatorError::Invalid("invalid source progress"));
            }
        }
        let payload = Payload::Commit(updates);
        if payload.bytes() > self.shared.options.limits.transaction_bytes {
            return Err(CoordinatorError::Capacity);
        }
        let (operation, result) = self.shared.submit(payload, false)?;
        Ok(ProgressReceipt { operation, result })
    }
    /// Query an unknown outcome. A missing/expired receipt stays `Unknown`.
    pub async fn resolve(
        &self,
        operation: ProgressOperationId,
    ) -> Result<ProgressOutcome, CoordinatorError> {
        let (_, result) = self.shared.submit(Payload::Resolve(operation), false)?;
        outcome(result).await
    }
    /// Read bounded opaque progress before resuming a source. Reading grants no execution permission.
    pub async fn read(&self, source: String) -> Result<Option<SourceProgress>, CoordinatorError> {
        if source.is_empty() || source.len() > 1024 || source.contains('\0') {
            return Err(CoordinatorError::Invalid("invalid source identity"));
        }
        let (_, result) = self.shared.submit(Payload::Read(source), false)?;
        match result.await.map_err(|_| CoordinatorError::Unavailable)?? {
            Response::Progress(value) => Ok(value),
            _ => Err(CoordinatorError::Failed(
                "invalid progress read receipt".into(),
            )),
        }
    }
}

fn storage_error(error: state::Error) -> CoordinatorError {
    CoordinatorError::Failed(error.to_string())
}
fn persistent_grant(epoch: u64, grant: &GrantIdentity) -> state::Grant {
    let mut token = Vec::with_capacity(56);
    token.extend_from_slice(grant.incarnation.as_bytes());
    for value in [
        grant.session.core_id as u64,
        grant.session.generation,
        grant.session.nonce,
        grant.revision,
        grant.term,
    ] {
        token.extend_from_slice(&value.to_le_bytes());
    }
    state::Grant {
        epoch,
        work: grant.work_id.0.clone(),
        term: grant.term,
        token,
    }
}

struct DurableBackend {
    shared: Arc<Shared>,
    last: Option<(OperationId, bool, GrantIdentity, ProgressOperationId)>,
}
impl DurableBackend {
    async fn mutate(
        &mut self,
        operation: OperationId,
        grant: &GrantIdentity,
        activate: bool,
    ) -> Result<BackendOutcome, CoordinatorError> {
        if let Some((prior, kind, identity, stored)) = &self.last {
            if operation.0 < prior.0
                || (operation == *prior && (activate != *kind || grant != identity))
            {
                return Ok(BackendOutcome::Rejected);
            }
            if operation == *prior {
                let (_, result) = self.shared.submit(Payload::Resolve(*stored), true)?;
                return outcome(result).await.map(backend_outcome);
            }
        }
        let persistent = persistent_grant(self.shared.epoch.load(Ordering::Acquire), grant);
        let payload = if activate {
            Payload::Activate(persistent)
        } else {
            Payload::Retire(persistent)
        };
        let (stored, result) = self.shared.submit(payload, true)?;
        self.last = Some((operation, activate, grant.clone(), stored));
        outcome(result).await.map(backend_outcome)
    }
}
fn backend_outcome(outcome: ProgressOutcome) -> BackendOutcome {
    match outcome {
        ProgressOutcome::Applied => BackendOutcome::Confirmed,
        ProgressOutcome::Rejected => BackendOutcome::Rejected,
        ProgressOutcome::Unknown => BackendOutcome::Unknown,
    }
}
#[async_trait(?Send)]
impl AssignmentBackend for DurableBackend {
    async fn activate(
        &mut self,
        operation: OperationId,
        grant: &GrantIdentity,
    ) -> Result<BackendOutcome, CoordinatorError> {
        self.mutate(operation, grant, true).await
    }
    async fn retire(
        &mut self,
        operation: OperationId,
        grant: &GrantIdentity,
    ) -> Result<BackendOutcome, CoordinatorError> {
        self.mutate(operation, grant, false).await
    }
    async fn resolve(
        &mut self,
        operation: OperationId,
    ) -> Result<BackendOutcome, CoordinatorError> {
        let Some((prior, _, _, stored)) = self.last.as_ref() else {
            return Ok(BackendOutcome::Unknown);
        };
        if *prior != operation {
            return Ok(BackendOutcome::Unknown);
        }
        let (_, result) = self.shared.submit(Payload::Resolve(*stored), true)?;
        outcome(result).await.map(backend_outcome)
    }
}

pub(super) struct Worker {
    shared: Arc<Shared>,
    join: Option<JoinHandle<()>>,
    done: Option<oneshot::Receiver<()>>,
}
impl Worker {
    pub async fn shutdown(mut self) -> Result<(), CoordinatorError> {
        if let Some(domain) = self.shared.domain.upgrade() {
            domain.fail("durable writer stopped".into());
        }
        self.shared.close();
        if let Some(done) = self.done.take() {
            let _ = done.await;
        }
        if let Some(join) = self.join.take() {
            tokio::task::spawn_blocking(move || join.join())
                .await
                .map_err(|e| CoordinatorError::Failed(e.to_string()))?
                .map_err(|_| CoordinatorError::Failed("durable writer panicked".into()))?;
        }
        Ok(())
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(domain) = self.shared.domain.upgrade() {
            domain.fail("durable writer owner exited".into());
        }
        self.shared.close();
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

struct RunGuard(Arc<Shared>);
impl Drop for RunGuard {
    fn drop(&mut self) {
        let expected = self.0.queues.lock().closing;
        if !expected && let Some(domain) = self.0.domain.upgrade() {
            domain.fail("durable writer exited unexpectedly".into());
        }
        self.0.close();
        let mut queues = self.0.queues.lock();
        queues.authority_bytes = 0;
        queues.progress_bytes = 0;
    }
}

pub(super) async fn start(
    options: DurableOptions,
    domain: Weak<Domain>,
) -> Result<(Box<dyn AssignmentBackend>, ProgressWriter, Worker), CoordinatorError> {
    options.validate()?;
    let shared = Arc::new(Shared {
        queues: Mutex::new(Queues {
            authority: VecDeque::new(),
            progress: VecDeque::new(),
            progress_bytes: 0,
            authority_bytes: 0,
            sequence: 0,
            closing: false,
        }),
        available: Condvar::new(),
        options,
        epoch: AtomicU64::new(0),
        domain,
        #[cfg(all(test, target_os = "linux"))]
        pause: Mutex::new(None),
        #[cfg(all(test, target_os = "linux"))]
        drop_reply: Mutex::new(None),
        #[cfg(all(test, target_os = "linux"))]
        changed: tokio::sync::Notify::new(),
    });
    let (initialized, ready) = oneshot::channel::<Result<(), CoordinatorError>>();
    let (finished, done) = oneshot::channel();
    let thread_shared = Arc::clone(&shared);
    let join = std::thread::Builder::new()
        .name("work-coordinator-writer".into())
        .spawn(move || {
            let options = &thread_shared.options;
            let store = state::Store::open(&options.root, &options.namespace, options.limits);
            match store {
                Ok(store) => {
                    thread_shared.epoch.store(store.epoch(), Ordering::Release);
                    let _ = initialized.send(Ok(()));
                    run(store, thread_shared);
                }
                Err(error) => {
                    let _ = initialized.send(Err(storage_error(error)));
                }
            }
            let _ = finished.send(());
        })
        .map_err(|e| CoordinatorError::Failed(e.to_string()))?;
    let worker = Worker {
        shared: Arc::clone(&shared),
        join: Some(join),
        done: Some(done),
    };
    ready.await.map_err(|_| CoordinatorError::Unavailable)??;
    Ok((
        Box::new(DurableBackend {
            shared: Arc::clone(&shared),
            last: None,
        }),
        ProgressWriter { shared },
        worker,
    ))
}

fn run(mut store: state::Store, shared: Arc<Shared>) {
    // Function locals drop before the Store parameter: invalidate admission
    // before releasing namespace exclusion, including during panic unwinding.
    let _guard = RunGuard(Arc::clone(&shared));
    let mut progress_turn = false;
    loop {
        let job = {
            let mut queues = shared.queues.lock();
            loop {
                if queues.closing {
                    return;
                }
                // Prefer authority, but service one publication between control
                // commands so receipt-window admission bounds reordering.
                let next = if progress_turn {
                    queues
                        .progress
                        .pop_front()
                        .or_else(|| queues.authority.pop_front())
                } else {
                    queues
                        .authority
                        .pop_front()
                        .or_else(|| queues.progress.pop_front())
                };
                if let Some(job) = next {
                    progress_turn = !progress_turn;
                    break job;
                }
                shared.available.wait(&mut queues);
            }
        };
        #[cfg(all(test, target_os = "linux"))]
        pause_job(&shared, &job.payload);
        #[cfg(all(test, target_os = "linux"))]
        let drop_reply = {
            let mut drop = shared.drop_reply.lock();
            if drop.is_some_and(|kind| kind.matches(&job.payload)) {
                let _ = drop.take();
                true
            } else {
                false
            }
        };
        let Job {
            operation,
            payload,
            result: sender,
            bytes,
            authority,
        } = job;
        let result = match payload {
            Payload::Activate(grant) => store.activate(operation, &grant).map(Response::Outcome),
            Payload::Retire(grant) => store.retire(operation, &grant).map(Response::Outcome),
            Payload::Resolve(operation) => store.resolve(operation).map(Response::Outcome),
            Payload::Read(source) => store.progress(&source).map(Response::Progress),
            Payload::Commit(updates) => {
                if updates.iter().any(|u| !u.ticket.is_valid()) {
                    let _ = sender.send(Err(CoordinatorError::Unavailable));
                    shared.queues.lock().progress_bytes -= bytes;
                    continue;
                }
                let mut tickets = Vec::with_capacity(updates.len());
                let stored: Vec<_> = updates
                    .into_iter()
                    .map(|u| {
                        let grant = persistent_grant(
                            shared.epoch.load(Ordering::Acquire),
                            u.ticket.grant(),
                        );
                        tickets.push(u.ticket);
                        state::ProgressUpdate {
                            grant,
                            source: u.source,
                            expected_revision: u.expected_revision,
                            progress: u.bytes,
                        }
                    })
                    .collect();
                store.commit(operation, &stored).map(Response::Outcome)
            }
        };
        if matches!(
            result,
            Err(state::Error::Storage(_) | state::Error::Poisoned)
        ) && let Some(domain) = shared.domain.upgrade()
        {
            domain.fail("durable authority lost its storage outcome".into());
        }
        #[cfg(all(test, target_os = "linux"))]
        if drop_reply {
            drop(sender);
        } else {
            let _ = sender.send(result.map_err(storage_error));
        }
        #[cfg(not(all(test, target_os = "linux")))]
        let _ = sender.send(result.map_err(storage_error));
        let mut queues = shared.queues.lock();
        if authority {
            queues.authority_bytes -= bytes;
        } else {
            queues.progress_bytes -= bytes;
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
#[derive(Clone, Copy)]
pub(super) enum TestKind {
    Activate,
    Commit,
}
#[cfg(all(test, target_os = "linux"))]
impl TestKind {
    fn matches(self, payload: &Payload) -> bool {
        matches!(
            (self, payload),
            (Self::Activate, Payload::Activate(_)) | (Self::Commit, Payload::Commit(_))
        )
    }
}
#[cfg(all(test, target_os = "linux"))]
struct TestPause {
    kind: TestKind,
    started: Mutex<Option<oneshot::Sender<()>>>,
    released: Mutex<bool>,
    changed: Condvar,
}
#[cfg(all(test, target_os = "linux"))]
impl TestPause {
    fn release(&self) {
        *self.released.lock() = true;
        let _ = self.changed.notify_all();
    }
}
#[cfg(all(test, target_os = "linux"))]
pub(super) struct PauseGuard {
    state: Arc<TestPause>,
    started: Option<oneshot::Receiver<()>>,
}
#[cfg(all(test, target_os = "linux"))]
impl PauseGuard {
    pub async fn entered(&mut self) {
        self.started
            .take()
            .expect("one gate waiter")
            .await
            .expect("writer reached controlled barrier");
    }
    pub fn release(&self) {
        self.state.release();
    }
}
#[cfg(all(test, target_os = "linux"))]
impl Drop for PauseGuard {
    fn drop(&mut self) {
        self.state.release();
    }
}
#[cfg(all(test, target_os = "linux"))]
impl ProgressWriter {
    pub(super) fn pause_next(&self, kind: TestKind) -> PauseGuard {
        let (tx, rx) = oneshot::channel();
        let state = Arc::new(TestPause {
            kind,
            started: Mutex::new(Some(tx)),
            released: Mutex::new(false),
            changed: Condvar::new(),
        });
        assert!(
            self.shared
                .pause
                .lock()
                .replace(Arc::clone(&state))
                .is_none()
        );
        PauseGuard {
            state,
            started: Some(rx),
        }
    }
    pub(super) fn lose_next_reply(&self, kind: TestKind) {
        *self.shared.drop_reply.lock() = Some(kind);
    }
    pub(super) async fn authority_queued(&self) {
        loop {
            let changed = self.shared.changed.notified();
            tokio::pin!(changed);
            let _ = changed.as_mut().enable();
            if !self.shared.queues.lock().authority.is_empty() {
                return;
            }
            changed.await;
        }
    }
}
#[cfg(all(test, target_os = "linux"))]
fn pause_job(shared: &Shared, payload: &Payload) {
    let pause = shared
        .pause
        .lock()
        .clone()
        .filter(|pause| pause.kind.matches(payload));
    if let Some(pause) = pause {
        if let Some(started) = pause.started.lock().take() {
            let _ = started.send(());
        }
        let mut released = pause.released.lock();
        while !*released {
            pause.changed.wait(&mut released);
        }
        drop(released);
        let _ = shared.pause.lock().take();
    }
}
