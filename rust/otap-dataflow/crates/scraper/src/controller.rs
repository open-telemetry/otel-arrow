// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared ACK-driven composite watermark receiver core.
//!
//! Delivery is at least once. A page is emitted with a unique batch ID, and the
//! durable cursor advances only after a matching ACK is followed by a
//! successful checkpoint write. A retryable NACK replays from the durable cursor
//! after a fixed backoff; a permanent NACK pauses this source by default or
//! uses explicitly configured, capped retry without skipping rows.

use crate::checkpoint::{CheckpointState, CheckpointStore};
use crate::database::{
    CatchUpConfig, ColumnMetadata, CompiledQuery, CompositeCursor, DriverAdapter,
    DriverCancellation, EncodedPage, OnPermanentNack, OtlpPageEncoder, parse_utc_timestamp,
    validate_mapping,
};
use crate::partition::{LeaseError, SourceLease};
use crate::telemetry::DatabaseReceiverMetrics;
use async_trait::async_trait;
use otel_arrow_dfe_channel::error::SendError;
use otel_arrow_dfe_engine::control::{CallData, Context8u8, NodeControlMsg};
use otel_arrow_dfe_engine::error::{Error, ReceiverErrorKind, TypedError, format_error_sources};
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::memory_limiter::{LocalReceiverAdmissionState, MemoryPressureChanged};
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{Interests, ProducerEffectHandlerExtension};
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::{otel_debug, otel_info, otel_warn};
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{SyncSender, TrySendError, sync_channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

const WORKER_STOP_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_INITIAL: Duration = Duration::from_secs(1);
const RETRY_MAX: Duration = Duration::from_secs(30);

type ScraperJob = Box<dyn FnOnce() + Send>;

/// One receiver owns one thread and at most one active plus one queued job.
/// Filesystem calls can hang indefinitely: Tokio's blocking pool would make
/// runtime drop wait forever even after an async operation timeout. The detached
/// thread instead reports completion after dropping all job-loop resources.
struct ScraperWorker {
    jobs: Option<SyncSender<ScraperJob>>,
    exited: oneshot::Receiver<()>,
}

#[derive(Debug, thiserror::Error)]
enum ScraperWorkerError {
    #[error("scraper worker queue is full")]
    Busy,
    #[error("scraper worker stopped without confirming cleanup; source requires process restart")]
    Stopped,
    #[error("scraper worker missed its stop deadline; source requires process restart")]
    Deadline,
}

impl ScraperWorker {
    fn new() -> std::io::Result<Self> {
        // These channels cross only this receiver's async-core/worker boundary.
        // Nonblocking submission provides a hard bound without a shared pool.
        let (jobs, requests) = sync_channel::<ScraperJob>(1);
        let (exit, exited) = oneshot::channel();
        let _thread = std::thread::Builder::new()
            .name("database-scraper".to_owned())
            .spawn(move || {
                while let Ok(job) = requests.recv() {
                    job();
                }
                drop(requests);
                // Panic drops the sender instead; it must never look like success.
                let _ = exit.send(());
            })?;
        Ok(Self {
            jobs: Some(jobs),
            exited,
        })
    }

    fn run<T: Send + 'static>(
        &self,
        job: impl FnOnce() -> T + Send + 'static,
    ) -> Result<oneshot::Receiver<T>, ScraperWorkerError> {
        let (response, result) = oneshot::channel();
        self.jobs
            .as_ref()
            .ok_or(ScraperWorkerError::Stopped)?
            .try_send(Box::new(move || {
                let _ = response.send(job());
            }))
            .map_err(|error| match error {
                TrySendError::Full(_) => ScraperWorkerError::Busy,
                TrySendError::Disconnected(_) => ScraperWorkerError::Stopped,
            })?;
        Ok(result)
    }

    async fn stop(mut self, deadline: Instant) -> Result<(), ScraperWorkerError> {
        drop(self.jobs.take());
        tokio::time::timeout_at(worker_stop_deadline(deadline).into(), &mut self.exited)
            .await
            .map_err(|_| ScraperWorkerError::Deadline)?
            .map_err(|_| ScraperWorkerError::Stopped)
    }
}

fn worker_stop_deadline(deadline: Instant) -> Instant {
    deadline.min(Instant::now() + WORKER_STOP_TIMEOUT)
}

/// Shared only by this receiver's local control-wait phases, never by worker threads.
struct PollAdmission {
    state: LocalReceiverAdmissionState,
    cycle_interrupted: Cell<bool>,
}

impl PollAdmission {
    fn apply(&self, update: MemoryPressureChanged) {
        self.state.apply(update);
        if self.state.should_shed_ingress() {
            self.interrupt_cycle();
        }
    }

    fn interrupt_cycle(&self) {
        self.cycle_interrupted.set(true);
    }
}

#[derive(Debug, thiserror::Error)]
#[error("database worker missed its stop deadline; source is quarantined until process restart")]
struct WorkerStopDeadline;

#[derive(Debug, thiserror::Error)]
#[error("database adapter cleanup failed; source is quarantined until process restart: {0}")]
struct AdapterCleanupError<E: std::error::Error + 'static>(#[source] E);

#[derive(Clone)]
struct NonInterruptible;

#[async_trait(?Send)]
impl DriverCancellation for NonInterruptible {
    type Error = std::convert::Infallible;

    async fn cancel(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Retains the actual storage lease if native or blocking work cannot be joined.
/// An externally dropped receiver future must fail closed just like a timeout.
struct HeldOwnership<L> {
    lease: Option<L>,
    abandoned: Cell<bool>,
    cleanup_joined: Cell<bool>,
}

impl HeldOwnership<SourceLease> {
    fn generation(&self) -> u64 {
        self.lease
            .as_ref()
            .expect("ownership retained until drop")
            .generation()
    }
}

impl<L> Drop for HeldOwnership<L> {
    fn drop(&mut self) {
        if self.abandoned.get() || !self.cleanup_joined.get() {
            // Native work and dedicated worker jobs cannot be aborted reliably.
            // Quarantine the real OS lock until process exit rather than allow
            // another receiver to overlap an operation whose join was lost.
            std::mem::forget(self.lease.take());
            otel_warn!(
                "database_receiver.worker_abandoned",
                message = "Worker cleanup was not joined; source ownership quarantined until process restart"
            );
        }
    }
}

/// A checkpoint store and the exclusive lease acquired for that exact store.
///
/// Construct this binding in the receiver factory, before entering the local
/// async polling runtime. The source ID is derived from the store, never supplied
/// independently. The binding is moved into the receiver and cannot be cloned.
///
/// Scenario: A factory attempts to combine a store with an independently acquired lease.
/// Guarantees: Private fields prevent bypassing the store-derived acquisition.
///
/// ```compile_fail,E0451
/// use otel_arrow_dfe_scraper::{CheckpointStore, SourceBinding, SourceLease};
///
/// fn bind_unchecked(checkpoint: CheckpointStore, lease: SourceLease) -> SourceBinding {
///     SourceBinding { checkpoint, lease }
/// }
/// ```
pub struct SourceBinding {
    checkpoint: CheckpointStore,
    lease: SourceLease,
}

impl SourceBinding {
    /// Acquires ownership using the checkpoint store's native filesystem key.
    ///
    /// This performs blocking filesystem work and must run during factory
    /// construction, off the pipeline's local async core. Acquisition failures
    /// are returned to the factory; no unprotected binding is constructed.
    pub fn acquire(checkpoint: CheckpointStore) -> Result<Self, LeaseError> {
        let lease = SourceLease::acquire(checkpoint.lease_key())?;
        Ok(Self { checkpoint, lease })
    }
}

/// Executes one compiled query through a database-specific adapter.
pub struct DatabaseReceiver<A> {
    adapter: A,
    query: CompiledQuery,
    source: SourceBinding,
    nack_backoff: Duration,
    max_consecutive_failures: u32,
    admission: LocalReceiverAdmissionState,
    metrics: Option<MetricSet<DatabaseReceiverMetrics>>,
}

impl<A> DatabaseReceiver<A>
where
    A: DriverAdapter,
{
    /// Creates a receiver bound to one durable checkpoint source.
    ///
    /// Acquire `source` in the factory with [`SourceBinding::acquire`]. This
    /// constructor performs no lease I/O and accepts no separate identity or lease.
    ///
    /// Bootstrap `admission` from the containing pipeline's process memory state
    /// so pre-existing hard pressure and observe-only mode are honored at startup.
    #[must_use]
    pub fn new(
        adapter: A,
        query: CompiledQuery,
        source: SourceBinding,
        nack_backoff: Duration,
        max_consecutive_failures: u32,
        admission: LocalReceiverAdmissionState,
        metrics: Option<MetricSet<DatabaseReceiverMetrics>>,
    ) -> Self {
        Self {
            adapter,
            query,
            source,
            nack_backoff,
            max_consecutive_failures,
            admission,
            metrics,
        }
    }
}

/// One page awaiting downstream ACK or NACK.
#[derive(Clone, Debug)]
struct PendingPage {
    id: u64,
    candidate: CompositeCursor,
}

#[derive(Clone, Debug)]
struct PollCycle {
    started: Instant,
    pages_started: usize,
}

#[derive(Clone, Debug)]
struct RejectionState {
    policy: OnPermanentNack,
    backoff: RetryBackoff,
}

/// Committed cursor plus in-flight and scheduling state.
#[derive(Clone, Debug)]
struct ReceiverState {
    committed: CompositeCursor,
    revision: u64,
    pending: Option<PendingPage>,
    next_batch_id: u64,
    next_poll: Instant,
    draining: bool,
    rejection: Option<RejectionState>,
    catch_up: CatchUpConfig,
    cycle: Option<PollCycle>,
}

#[derive(Debug, thiserror::Error)]
enum ProgressError {
    #[error("database query returned a page that did not advance the committed cursor")]
    NonAdvancingCursor,
    #[error("database cursor timestamp is not a supported UTC timestamp")]
    InvalidTimestamp,
}

fn ensure_cursor_advanced(
    committed: &CompositeCursor,
    candidate: &CompositeCursor,
) -> Result<(), ProgressError> {
    let candidate_time =
        parse_utc_timestamp(&candidate.timestamp).map_err(|_| ProgressError::InvalidTimestamp)?;
    let committed_time =
        parse_utc_timestamp(&committed.timestamp).map_err(|_| ProgressError::InvalidTimestamp)?;
    if (candidate_time, candidate.tie_breaker) <= (committed_time, committed.tie_breaker) {
        Err(ProgressError::NonAdvancingCursor)
    } else {
        Ok(())
    }
}

impl ReceiverState {
    fn new(checkpoint: CheckpointState, now: Instant, catch_up: CatchUpConfig) -> Self {
        Self {
            committed: checkpoint.cursor,
            revision: checkpoint.revision,
            pending: None,
            next_batch_id: 1,
            next_poll: now,
            draining: false,
            rejection: None,
            catch_up,
            cycle: None,
        }
    }

    /// Returns whether receiver state permits a new query; process admission is checked separately.
    ///
    /// At most one page is in flight per source, so a pending ACK/NACK blocks
    /// the next poll and prevents overlapping database work.
    fn can_poll(&self) -> bool {
        !self.draining && self.pending.is_none() && !self.rejection_paused()
    }

    fn rejection_paused(&self) -> bool {
        self.rejection
            .as_ref()
            .is_some_and(|rejection| rejection.policy == OnPermanentNack::Pause)
    }

    fn reject(&mut self, policy: OnPermanentNack, now: Instant) -> Option<Duration> {
        self.pending = None;
        self.cycle = None;
        let rejection = self.rejection.get_or_insert_with(|| RejectionState {
            policy,
            backoff: RetryBackoff::default(),
        });
        match rejection.policy {
            OnPermanentNack::Pause => None,
            OnPermanentNack::Retry => {
                let delay = rejection.backoff.next_delay();
                self.next_poll = now + delay;
                Some(delay)
            }
        }
    }

    fn schedule_after(&mut self, delay: Duration, now: Instant) {
        self.next_poll = now.checked_add(delay).unwrap_or(now);
    }

    fn begin_poll(&mut self, now: Instant) {
        let cycle = self.cycle.get_or_insert(PollCycle {
            started: now,
            pages_started: 0,
        });
        cycle.pages_started = cycle.pages_started.saturating_add(1);
    }

    fn can_continue_cycle(&self, now: Instant) -> bool {
        self.cycle.as_ref().is_some_and(|cycle| {
            cycle.pages_started < self.catch_up.max_pages
                && now.saturating_duration_since(cycle.started) < self.catch_up.max_duration
        })
    }

    fn finish_cycle(&mut self, interval: Duration, now: Instant) {
        self.cycle = None;
        self.schedule_after(interval, now);
    }

    fn schedule_after_commit(&mut self, interval: Duration, now: Instant, interrupted: bool) {
        if !interrupted && self.can_continue_cycle(now) {
            self.next_poll = now;
        } else {
            self.finish_cycle(interval, now);
        }
    }

    fn record_sent(&mut self, candidate: CompositeCursor) {
        debug_assert!(self.pending.is_none());
        let id = self.next_batch_id;
        self.next_batch_id = self.next_batch_id.saturating_add(1);
        self.pending = Some(PendingPage { id, candidate });
    }

    /// Returns the candidate only when the feedback matches the in-flight page.
    fn ack_candidate(&self, batch_id: u64) -> Option<CompositeCursor> {
        self.pending
            .as_ref()
            .filter(|pending| pending.id == batch_id)
            .map(|pending| pending.candidate.clone())
    }

    fn commit(&mut self, checkpoint: CheckpointState) {
        self.committed = checkpoint.cursor;
        self.revision = checkpoint.revision;
        self.pending = None;
        self.rejection = None;
    }

    /// Rewinds to the committed cursor when the feedback matches; else no-op.
    fn nack(&mut self, batch_id: u64, replay_at: Instant) -> bool {
        if self
            .pending
            .as_ref()
            .is_none_or(|pending| pending.id != batch_id)
        {
            return false;
        }
        self.pending = None;
        self.cycle = None;
        self.next_poll = replay_at;
        true
    }

    const fn begin_drain(&mut self) {
        self.draining = true;
    }
}

fn batch_id_from_call_data(call_data: &CallData, generation: u64) -> Option<u64> {
    if call_data.get(1).copied().map(u64::from) != Some(generation) {
        return None;
    }
    call_data.first().copied().map(u64::from)
}

fn pause_for_pressure(
    admission: &PollAdmission,
    state: &mut ReceiverState,
    encoder: &mut OtlpPageEncoder,
    interval: Duration,
    now: Instant,
) -> bool {
    if !admission.state.should_shed_ingress() {
        return false;
    }
    encoder.release_scratch();
    // Only an active cycle needs a new cooldown; pressure must not keep
    // postponing an already-scheduled idle poll.
    if state.cycle.is_some() {
        state.finish_cycle(interval, now);
    }
    true
}

#[async_trait(?Send)]
impl<A> local::Receiver<OtapPdata> for DatabaseReceiver<A>
where
    A: DriverAdapter + 'static,
{
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let Self {
            mut adapter,
            query,
            source,
            nack_backoff,
            max_consecutive_failures,
            admission,
            mut metrics,
        } = *self;
        let SourceBinding { checkpoint, lease } = source;
        let source_id = checkpoint.source_id().to_owned();
        let lease = HeldOwnership {
            lease: Some(lease),
            abandoned: Cell::new(false),
            cleanup_joined: Cell::new(false),
        };
        let admission = PollAdmission {
            state: admission,
            cycle_interrupted: Cell::new(false),
        };
        let stopping = StopState::default();
        let mut worker = None;
        // All normal and error exits confirm both workers stopped before releasing
        // ownership, including spawn/read failures. Drop quarantines the lease.
        let result = async {

        worker = Some(ScraperWorker::new().map_err(|error| {
            receiver_error(&effect_handler, ReceiverErrorKind::Other, error)
        })?);
        let worker = worker.as_ref().expect("worker just created");
        // Fail closed on unreadable durable state before any row is fetched.
        let loaded = match await_database_operation_or_stop(
            read_checkpoint(worker, &checkpoint, &effect_handler),
            NonInterruptible,
            &mut ctrl_msg_recv,
            &mut metrics,
            &lease.abandoned,
            &admission,
            &stopping,
        ).await? {
            OperationOutcome::Completed(result) => result?,
            OperationOutcome::Stopped(stop) => return Ok(stopped_state(stop, &metrics)),
        };
        let mut state = ReceiverState::new(
            loaded.unwrap_or_else(|| CheckpointState {
                revision: 0,
                cursor: query.watermark().initial.clone(),
            }),
            Instant::now(),
            query.catch_up(),
        );
        if let Some(metrics) = metrics.as_mut() {
            metrics.starts.add(1);
        }
        let _telemetry_timer = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;
        otel_info!(
            "database_receiver.start",
            source_id = source_id.as_str(),
            db_system = adapter.system().as_str(),
            checkpoint_revision = state.revision,
            ownership_generation = lease.generation()
        );

        let mut database_retry = DatabaseRetry::default();
        let columns = match prepare_database(
            &mut adapter,
            &query,
            &mut database_retry,
            &source_id,
            &mut ctrl_msg_recv,
            &effect_handler,
            &mut metrics,
            &lease.abandoned,
            &admission,
            &stopping,
        )
        .await?
        {
            OperationOutcome::Completed(columns) => columns,
            OperationOutcome::Stopped(stop) => {
                return Ok(stopped_state(stop, &metrics));
            }
        };
        let mut encoder = OtlpPageEncoder::new(
            adapter.system(), source_id.clone(), query.output().clone(), columns,
        ).map_err(|error| {
            receiver_error(&effect_handler, ReceiverErrorKind::Configuration, error)
        })?;

        let mut consecutive_checkpoint_failures = 0_u32;
        let mut deferred_feedback = None;

        loop {
            if let Some(metrics) = metrics.as_mut() {
                metrics.rejection_paused.set(u64::from(state.rejection_paused()));
            }
            // I/O helpers may have consumed pressure updates, including on an
            // empty query. Reconcile pressure whenever encoder ownership is local.
            let pressure_paused = pause_for_pressure(
                &admission, &mut state, &mut encoder, query.interval(), Instant::now(),
            );
            let can_poll = state.can_poll() && !pressure_paused;
            tokio::select! {
                biased;

                () = deadline_elapsed(stopping.wait_deadline.get()), if stopping.request.get().is_some() => {
                    let Some(stop) = stopping.request.get() else { continue };
                    if state.pending.is_some() {
                        otel_warn!(
                            "database_receiver.drain_wait_expired",
                            source_id = source_id.as_str(),
                            message = "Delivery wait ended to reserve worker cleanup time; the checkpoint was not advanced"
                        );
                    }
                    return Ok(stopped_state(stop, &metrics));
                }

                control = async {
                    match deferred_feedback.take() {
                        Some(control) => Ok(control),
                        None => ctrl_msg_recv.recv().await,
                    }
                } => {
                    let Some(control) = handle_common_control(
                        control.map_err(Error::ChannelRecvError)?, &mut metrics, &admission,
                    ) else { continue };
                    match control {
                        NodeControlMsg::Ack(ack) => {
                            let Some(batch_id) = batch_id_from_call_data(&ack.unwind.route.calldata, lease.generation())
                            else {
                                count_stale_feedback(&mut metrics);
                                continue;
                            };
                            let Some(candidate) = state.ack_candidate(batch_id) else {
                                // Late or duplicate feedback cannot advance state.
                                count_stale_feedback(&mut metrics);
                                continue;
                            };
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.acks.add(1);
                            }
                            let committed = commit_checkpoint(
                                worker,
                                &checkpoint,
                                state.revision,
                                &candidate,
                                max_consecutive_failures,
                                nack_backoff,
                                &mut consecutive_checkpoint_failures,
                                &source_id,
                                batch_id,
                                &effect_handler,
                                &mut metrics,
                                &lease.abandoned,
                                &mut ctrl_msg_recv,
                                &stopping,
                                &admission,
                            )
                            .await?;
                            let committed = match committed {
                                CommitOutcome::Committed(committed) => committed,
                                CommitOutcome::Stopped(stop) => {
                                    return Ok(stopped_state(stop, &metrics));
                                }
                            };
                            // In-memory state advances only after the durable write.
                            let rejection_recovered = state.rejection.is_some();
                            state.commit(committed);
                            if rejection_recovered {
                                otel_info!(
                                    "database_receiver.rejection_recovered",
                                    source_id = source_id.as_str(),
                                    message = "Downstream accepted the retried page; acknowledged progress committed"
                                );
                            }
                            if admission.state.should_shed_ingress() {
                                encoder.release_scratch();
                            }
                            state.schedule_after_commit(
                                query.interval(),
                                Instant::now(),
                                admission.cycle_interrupted.get(),
                            );
                            if let Some(stop) = stopping.request.get() {
                                return Ok(stopped_state(stop, &metrics));
                            }
                        }
                        NodeControlMsg::Nack(nack) => {
                            let Some(batch_id) =
                                batch_id_from_call_data(&nack.unwind.route.calldata, lease.generation())
                            else {
                                count_stale_feedback(&mut metrics);
                                continue;
                            };
                            if state.pending.as_ref().is_none_or(|pending| pending.id != batch_id) {
                                count_stale_feedback(&mut metrics);
                                continue;
                            }
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.nacks.add(1);
                            }
                            if nack.permanent || state.rejection.is_some() {
                                let delay = state.reject(query.on_permanent_nack(), Instant::now());
                                encoder.release_scratch();
                                admission.interrupt_cycle();
                                // Downstream reasons may contain customer data; report only fixed context.
                                match delay {
                                    None => otel_warn!(
                                        "database_receiver.source_paused",
                                        source_id = source_id.as_str(),
                                        batch_id = batch_id,
                                        message = "Downstream permanently rejected the page; source paused with checkpoint unchanged. Repair downstream and restart this source to resume"
                                    ),
                                    Some(delay) => {
                                        if let Some(metrics) = metrics.as_mut() {
                                            metrics.replays.add(1);
                                        }
                                        otel_warn!(
                                            "database_receiver.rejection_retry_scheduled",
                                            source_id = source_id.as_str(),
                                            batch_id = batch_id,
                                            backoff_millis = delay.as_millis() as u64,
                                            message = "Rejected page retained as uncommitted progress; explicit retry policy will re-query after backoff"
                                        );
                                    }
                                }
                                if let Some(stop) = stopping.request.get() {
                                    return Ok(stopped_state(stop, &metrics));
                                }
                                continue;
                            }
                            let replay_at = Instant::now()
                                .checked_add(nack_backoff)
                                .unwrap_or_else(Instant::now);
                            if state.nack(batch_id, replay_at) {
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.replays.add(1);
                                }
                                otel_warn!(
                                    "database_receiver.page_nacked",
                                    source_id = source_id.as_str(),
                                    batch_id = batch_id,
                                    backoff_millis = nack_backoff.as_millis() as u64,
                                    message = "Database receiver retained its checkpoint and will replay the page"
                                );
                                if let Some(stop) = stopping.request.get() {
                                    return Ok(stopped_state(stop, &metrics));
                                }
                            }
                        }
                        NodeControlMsg::DrainIngress { deadline, .. } => {
                            state.begin_drain();
                            let stop = stopping.record(StopRequest::Drain(deadline), Instant::now());
                            if state.pending.is_none() {
                                return Ok(stopped_state(stop, &metrics));
                            }
                        }
                        NodeControlMsg::Shutdown { deadline, .. } => {
                            let stop = stopping.record(StopRequest::Shutdown(deadline), Instant::now());
                            return Ok(stopped_state(stop, &metrics));
                        }
                        _ => {}
                    }
                }

                () = poll_due(state.next_poll, can_poll), if can_poll => {
                    let now = Instant::now();
                    // A ready timer does not authorize another page after the
                    // active cycle's budget or an intervening pressure event.
                    if state.cycle.is_some()
                        && (admission.cycle_interrupted.get() || !state.can_continue_cycle(now))
                    {
                        state.finish_cycle(query.interval(), now);
                        continue;
                    }
                    if state.cycle.is_none() {
                        admission.cycle_interrupted.set(false);
                    }
                    state.begin_poll(now);
                    let cursor = state.committed.clone();
                    let page = loop {
                        if database_retry.failures > 0
                            && let Some(stop) = wait_database_retry(
                                Instant::now(), &mut ctrl_msg_recv, &mut metrics,
                                &admission, &stopping,
                            ).await?
                        {
                            return Ok(stopped_state(stop, &metrics));
                        }
                        if let Some(metrics) = metrics.as_mut() {
                            metrics.polls.add(1);
                        }
                        let page = match adapter.begin_operation() {
                            Ok(cancellation) => match await_database_operation_or_stop(
                                adapter.execute(&query, &cursor),
                                cancellation,
                                &mut ctrl_msg_recv,
                                &mut metrics,
                                &lease.abandoned,
                                &admission,
                                &stopping,
                            ).await? {
                                OperationOutcome::Completed(result) => result,
                                OperationOutcome::Stopped(stop) => {
                                    return Ok(stopped_state(stop, &metrics));
                                }
                            },
                            Err(error) => Err(error),
                        };
                        match page {
                            Ok(page) => {
                                database_retry.recovered(&source_id);
                                break page;
                            }
                            Err(error) => {
                                if let Some(metrics) = metrics.as_mut() {
                                    metrics.query_failures.add(1);
                                }
                                database_retry.failed::<A>(
                                    error, "execute", &source_id, &effect_handler,
                                )?;
                                admission.interrupt_cycle();
                                encoder.release_scratch();
                                match prepare_database(
                                    &mut adapter, &query, &mut database_retry, &source_id,
                                    &mut ctrl_msg_recv, &effect_handler, &mut metrics,
                                    &lease.abandoned, &admission, &stopping,
                                ).await? {
                                    OperationOutcome::Completed(_) => {}
                                    OperationOutcome::Stopped(stop) => {
                                        return Ok(stopped_state(stop, &metrics));
                                    }
                                }
                            }
                        }
                    };
                    let observed_time = observed_time_unix_nano().map_err(|error| {
                        receiver_error(&effect_handler, ReceiverErrorKind::Other, error)
                    })?;
                    let limit = query.max_batch_bytes();
                    // One bounded page per receiver; encoding is CPU-intensive and
                    // must not monopolize the thread-per-core engine runtime.
                    // Move the single-owner cache into the job and back, avoiding
                    // per-poll configuration clones or a shared mutable cache.
                    let encoding = worker.run(move || {
                        let encoded = encoder.encode_page(page, observed_time, limit);
                        (encoder, encoded)
                    }).map_err(|error| receiver_error(
                        &effect_handler, ReceiverErrorKind::Other, error,
                    ))?;
                    let (returned_encoder, encoded) = match await_database_operation_or_stop(
                        encoding, NonInterruptible, &mut ctrl_msg_recv, &mut metrics, &lease.abandoned, &admission, &stopping,
                    ).await? {
                        OperationOutcome::Completed(result) => result,
                        OperationOutcome::Stopped(stop) => return Ok(stopped_state(stop, &metrics)),
                    }.map_err(|error| receiver_error(
                        &effect_handler, ReceiverErrorKind::Other, error,
                    ))?;
                    encoder = returned_encoder;
                    if admission.state.should_shed_ingress() {
                        encoder.release_scratch();
                    }
                    let encoded = match encoded {
                        Ok(Some(encoded)) => encoded,
                        Ok(None) => {
                            state.finish_cycle(query.interval(), Instant::now());
                            continue;
                        }
                        Err(error) => {
                            // An oversized first row or an invalid mapping
                            // cannot be skipped without losing data.
                            return Err(receiver_error(
                                &effect_handler,
                                ReceiverErrorKind::Configuration,
                                error,
                            ));
                        }
                    };
                    let EncodedPage {
                        mut pdata,
                        candidate,
                        row_count,
                        encoded_bytes,
                        deferred_rows,
                        event_time_fallbacks,
                    } = encoded;
                    ensure_cursor_advanced(&state.committed, &candidate).map_err(|error| {
                        receiver_error(&effect_handler, ReceiverErrorKind::Configuration, error)
                    })?;

                    let batch_id = state.next_batch_id;
                    let mut call_data = CallData::new();
                    call_data.push(Context8u8::from(batch_id));
                    call_data.push(Context8u8::from(lease.generation()));
                    effect_handler.subscribe_to(Interests::ACKS_OR_NACKS, call_data, &mut pdata);
                    match send_or_stop(
                        pdata,
                        &mut ctrl_msg_recv,
                        &effect_handler,
                        &mut state,
                        &mut metrics,
                        &stopping,
                        &mut deferred_feedback,
                        lease.generation(),
                        &admission,
                    )
                    .await?
                    {
                        SendOutcome::Sent => {}
                        SendOutcome::Stopped(stop) => {
                            return Ok(stopped_state(stop, &metrics));
                        }
                    }
                    // Recording after the successful send keeps the pending
                    // page and the emitted batch ID consistent on every path.
                    state.record_sent(candidate);
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.batches_sent.add(1);
                        metrics.rows_sent.add(row_count as u64);
                        metrics.encoded_bytes_sent.add(encoded_bytes as u64);
                        metrics
                            .event_time_fallbacks
                            .add(event_time_fallbacks as u64);
                    }
                    if event_time_fallbacks > 0 {
                        otel_warn!(
                            "database_receiver.event_time_fallback",
                            source_id = source_id.as_str(),
                            records = event_time_fallbacks as u64,
                            message = "Database records used observation time because source event time was outside the OTLP range"
                        );
                    }
                    otel_debug!(
                        "database_receiver.page_sent",
                        source_id = source_id.as_str(),
                        batch_id = batch_id,
                        rows = row_count as u64,
                        encoded_bytes = encoded_bytes as u64,
                        deferred_rows = deferred_rows as u64
                    );
                }
            }
        }
        }.await;
        let deadline = worker_stop_deadline(stopping.request.get().map_or_else(
            || Instant::now() + WORKER_STOP_TIMEOUT,
            StopRequest::deadline,
        ));
        // Both cleanup paths get the same budget and are polled even when it
        // has expired. Never block this core on a std::thread::JoinHandle.
        let (adapter_stopped, scraper_stopped) = tokio::join!(
            tokio::time::timeout_at(deadline.into(), adapter.shutdown()),
            async {
                match worker {
                    Some(worker) => worker.stop(deadline).await,
                    None => Ok(()),
                }
            }
        );
        if matches!(&adapter_stopped, Ok(Ok(()))) && scraper_stopped.is_ok() {
            // Confirmed cleanup supersedes provisional abandonment from an earlier wait.
            lease.abandoned.set(false);
            lease.cleanup_joined.set(true);
        } else {
            lease.abandoned.set(true);
        }
        if let Ok(Err(error)) = adapter_stopped {
            return Err(receiver_error(
                &effect_handler,
                A::classify_error(&error),
                AdapterCleanupError(error),
            ));
        }
        if let Err(error) = scraper_stopped {
            return Err(receiver_error(
                &effect_handler,
                ReceiverErrorKind::Shutdown,
                error,
            ));
        }
        if lease.abandoned.get() {
            return Err(receiver_error(
                &effect_handler,
                ReceiverErrorKind::Shutdown,
                WorkerStopDeadline,
            ));
        }
        // Notification can itself wait on the runtime control channel. Complete
        // cleanup first so a blocked notification cannot consume its reservation.
        if result.is_ok()
            && let Some(StopRequest::Drain(deadline)) = stopping.request.get()
        {
            tokio::time::timeout_at(deadline.into(), effect_handler.notify_receiver_drained())
                .await
                .map_err(|error| {
                    receiver_error(&effect_handler, ReceiverErrorKind::Shutdown, error)
                })??;
        }
        result
    }
}

enum OperationOutcome<T> {
    Completed(T),
    Stopped(StopRequest),
}

#[derive(Clone, Debug)]
struct RetryBackoff {
    next: Duration,
}

impl Default for RetryBackoff {
    fn default() -> Self {
        Self {
            next: RETRY_INITIAL,
        }
    }
}

impl RetryBackoff {
    fn next_delay(&mut self) -> Duration {
        let delay = self.next;
        self.next = delay.saturating_mul(2).min(RETRY_MAX);
        delay
    }
}

#[derive(Default)]
struct DatabaseRetry {
    backoff: RetryBackoff,
    retry_at: Option<Instant>,
    failures: u64,
}

impl DatabaseRetry {
    fn schedule(&mut self, now: Instant) -> Duration {
        let delay = self.backoff.next_delay();
        self.retry_at = Some(now + delay);
        self.failures = self.failures.saturating_add(1);
        delay
    }

    fn failed<A: DriverAdapter>(
        &mut self,
        error: A::Error,
        phase: &'static str,
        source_id: &str,
        effects: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        if !A::is_retryable(&error) {
            return Err(receiver_error(effects, A::classify_error(&error), error));
        }
        let delay = self.schedule(Instant::now());
        // Native errors can contain credentials or query values. Recovery logs
        // describe the phase and schedule without copying driver diagnostics.
        otel_warn!(
            "database_receiver.retry_scheduled",
            source_id = source_id,
            phase = phase,
            consecutive_failures = self.failures,
            backoff_millis = delay.as_millis() as u64,
            message = "Transient database failure; checkpoint retained while retrying locally"
        );
        Ok(())
    }

    fn recovered(&mut self, source_id: &str) {
        if self.failures > 0 {
            otel_info!(
                "database_receiver.recovered",
                source_id = source_id,
                consecutive_failures = self.failures,
                message = "Database polling recovered without restarting the receiver"
            );
        }
        *self = Self::default();
    }
}

async fn wait_database_retry(
    retry_at: Instant,
    controls: &mut local::ControlChannel<OtapPdata>,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
    admission: &PollAdmission,
    stopping: &StopState,
) -> Result<Option<StopRequest>, Error> {
    loop {
        tokio::select! {
            biased;
            control = controls.recv() => {
                let Some(control) = handle_common_control(
                    control.map_err(Error::ChannelRecvError)?, metrics, admission,
                ) else { continue };
                count_discarded_feedback(&control, metrics);
                if let Some(stop) = stop_request(&control) {
                    return Ok(Some(stopping.record(stop, Instant::now())));
                }
            }
            () = poll_due(retry_at, !admission.state.should_shed_ingress()) => {
                return Ok(None);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn prepare_database<A: DriverAdapter>(
    adapter: &mut A,
    query: &CompiledQuery,
    retry: &mut DatabaseRetry,
    source_id: &str,
    controls: &mut local::ControlChannel<OtapPdata>,
    effects: &local::EffectHandler<OtapPdata>,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
    abandoned: &Cell<bool>,
    admission: &PollAdmission,
    stopping: &StopState,
) -> Result<OperationOutcome<Vec<ColumnMetadata>>, Error> {
    loop {
        if let Some(stop) = wait_database_retry(
            retry.retry_at.unwrap_or_else(Instant::now),
            controls,
            metrics,
            admission,
            stopping,
        )
        .await?
        {
            return Ok(OperationOutcome::Stopped(stop));
        }
        if retry.retry_at.is_some() {
            let result = match adapter.begin_operation() {
                Ok(cancellation) => {
                    if let Some(metrics) = metrics.as_mut() {
                        metrics.reconnects.add(1);
                    }
                    match await_database_operation_or_stop(
                        adapter.reconnect(query),
                        cancellation,
                        controls,
                        metrics,
                        abandoned,
                        admission,
                        stopping,
                    )
                    .await?
                    {
                        OperationOutcome::Completed(result) => result,
                        OperationOutcome::Stopped(stop) => {
                            return Ok(OperationOutcome::Stopped(stop));
                        }
                    }
                }
                Err(error) => Err(error),
            };
            if let Err(error) = result {
                retry.failed::<A>(error, "reconnect", source_id, effects)?;
                continue;
            }
            retry.retry_at = None;
            if let Some(stop) =
                wait_database_retry(Instant::now(), controls, metrics, admission, stopping).await?
            {
                return Ok(OperationOutcome::Stopped(stop));
            }
        }
        let result = match adapter.begin_operation() {
            Ok(cancellation) => match await_database_operation_or_stop(
                adapter.validate_query(query),
                cancellation,
                controls,
                metrics,
                abandoned,
                admission,
                stopping,
            )
            .await?
            {
                OperationOutcome::Completed(result) => result,
                OperationOutcome::Stopped(stop) => return Ok(OperationOutcome::Stopped(stop)),
            },
            Err(error) => Err(error),
        };
        match result {
            Ok(columns) => {
                validate_mapping(&columns, query.output()).map_err(|error| {
                    receiver_error(effects, ReceiverErrorKind::Configuration, error)
                })?;
                return Ok(OperationOutcome::Completed(columns));
            }
            Err(error) => retry.failed::<A>(error, "validate", source_id, effects)?,
        }
    }
}

enum SendOutcome {
    Sent,
    Stopped(StopRequest),
}

#[derive(Clone, Copy)]
enum StopRequest {
    Drain(Instant),
    Shutdown(Instant),
}

impl StopRequest {
    fn deadline(self) -> Instant {
        match self {
            Self::Drain(deadline) | Self::Shutdown(deadline) => deadline,
        }
    }
}

/// One fixed wait cutoff shared by every local stop phase, including error exits.
#[derive(Default)]
struct StopState {
    request: Cell<Option<StopRequest>>,
    wait_deadline: Cell<Option<Instant>>,
}

impl StopState {
    fn record(&self, request: StopRequest, now: Instant) -> StopRequest {
        let deadline = request.deadline();
        // Keep short deadlines useful for both delivery and cleanup, while
        // reserving no more than the existing five-second worker stop allowance.
        let reserve = WORKER_STOP_TIMEOUT.min(deadline.saturating_duration_since(now) / 2);
        let wait_deadline = deadline - reserve;
        self.wait_deadline.set(Some(
            self.wait_deadline
                .get()
                .map_or(wait_deadline, |current| current.min(wait_deadline)),
        ));
        let request = match self.request.get() {
            Some(current) => {
                let deadline = deadline.min(current.deadline());
                if matches!(current, StopRequest::Shutdown(_))
                    || matches!(request, StopRequest::Shutdown(_))
                {
                    StopRequest::Shutdown(deadline)
                } else {
                    StopRequest::Drain(deadline)
                }
            }
            None => request,
        };
        self.request.set(Some(request));
        request
    }
}

/// Waits until the phase cutoff, or forever when no stop is pending.
async fn deadline_elapsed(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => {
            tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
        }
        None => std::future::pending::<()>().await,
    }
}

/// Waits until the next poll is due, or forever when polling is blocked.
async fn poll_due(next_poll: Instant, can_poll: bool) {
    if can_poll {
        if next_poll > Instant::now() {
            tokio::time::sleep_until(tokio::time::Instant::from_std(next_poll)).await;
        }
    } else {
        std::future::pending::<()>().await;
    }
}

async fn read_checkpoint(
    worker: &ScraperWorker,
    store: &CheckpointStore,
    effect_handler: &local::EffectHandler<OtapPdata>,
) -> Result<Option<CheckpointState>, Error> {
    // Checkpoint filesystem work blocks, so it must not run on the local
    // async engine core.
    let store = store.clone();
    worker
        .run(move || store.read())
        .map_err(|error| receiver_error(effect_handler, ReceiverErrorKind::Other, error))?
        .await
        .map_err(|error| receiver_error(effect_handler, ReceiverErrorKind::Other, error))?
        .map_err(|error| receiver_error(effect_handler, ReceiverErrorKind::Configuration, error))
}

#[allow(clippy::too_many_arguments)]
async fn commit_checkpoint(
    worker: &ScraperWorker,
    store: &CheckpointStore,
    revision: u64,
    candidate: &CompositeCursor,
    max_failures: u32,
    retry_backoff: Duration,
    consecutive_failures: &mut u32,
    source_id: &str,
    batch_id: u64,
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
    abandoned: &Cell<bool>,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    stopping: &StopState,
    admission: &PollAdmission,
) -> Result<CommitOutcome, Error> {
    // Cap checkpoint stop waits without resetting an inherited cleanup reservation.
    let mut wait_deadline = stopping.wait_deadline.get().map(worker_stop_deadline);
    loop {
        if wait_deadline.is_some_and(|deadline| Instant::now() >= deadline)
            && let Some(stop) = stopping.request.get()
        {
            return Ok(CommitOutcome::Stopped(stop));
        }
        let store = store.clone();
        let cursor = candidate.clone();
        // Only one bounded write is outstanding. Keep processing controls
        // while filesystem work runs off-core; never release ownership if
        // the worker outlives the stop deadline.
        let mut write = worker
            .run(move || store.write(revision, &cursor))
            .map_err(|error| receiver_error(effect_handler, ReceiverErrorKind::Other, error))?;
        let result = loop {
            tokio::select! {
                biased;
                result = &mut write => {
                    break result.map_err(|error| receiver_error(effect_handler, ReceiverErrorKind::Other, error))?;
                }
                () = deadline_elapsed(wait_deadline), if wait_deadline.is_some() => {
                    abandoned.set(true);
                    if let Some(stop) = stopping.request.get() { return Ok(CommitOutcome::Stopped(stop)); }
                }
                control = ctrl_msg_recv.recv() => {
                    match control {
                        Ok(control) => {
                            let Some(control) = handle_common_control(control, metrics, admission)
                            else { continue };
                            count_discarded_feedback(&control, metrics);
                            if let Some(request) = stop_request(&control) {
                                _ = stopping.record(request, Instant::now());
                                let deadline = stopping.wait_deadline.get().map(worker_stop_deadline);
                                wait_deadline = match (wait_deadline, deadline) {
                                    (Some(current), Some(deadline)) => Some(current.min(deadline)),
                                    (_, deadline) => deadline,
                                };
                            }
                        }
                        Err(error) => {
                            abandoned.set(true);
                            return Err(Error::ChannelRecvError(error));
                        }
                    }
                }
            }
        };
        match result {
            Ok((checkpoint, outcome)) => {
                *consecutive_failures = 0;
                if let Some(metrics) = metrics.as_mut() {
                    metrics.checkpoint_commits.add(1);
                    metrics
                        .checkpoint_cleanup_failures
                        .add(outcome.cleanup_failures as u64);
                }
                if outcome.cleanup_failures > 0 {
                    otel_warn!(
                        "database_receiver.checkpoint_cleanup_failed",
                        source_id = source_id,
                        failures = outcome.cleanup_failures as u64,
                        message = "Database receiver could not remove stale checkpoint revisions"
                    );
                }
                otel_debug!(
                    "database_receiver.checkpoint_committed",
                    source_id = source_id,
                    batch_id = batch_id,
                    revision = checkpoint.revision
                );
                if let Some(stop) = stopping.request.get() {
                    return Ok(CommitOutcome::Stopped(stop));
                }
                return Ok(CommitOutcome::Committed(checkpoint));
            }
            Err(error) => {
                admission.interrupt_cycle();
                if let Some(stop @ StopRequest::Shutdown(_)) = stopping.request.get() {
                    return Ok(CommitOutcome::Stopped(stop));
                }
                if let Some(stop) = stopping.request.get()
                    && wait_deadline.is_some_and(|deadline| Instant::now() >= deadline)
                {
                    return Ok(CommitOutcome::Stopped(stop));
                }
                // The in-memory cursor is never advanced on a failed write, so
                // a later retry or restart replays the same page.
                *consecutive_failures = consecutive_failures.saturating_add(1);
                if let Some(metrics) = metrics.as_mut() {
                    metrics.checkpoint_failures.add(1);
                }
                otel_warn!(
                    "database_receiver.checkpoint_failed",
                    source_id = source_id,
                    batch_id = batch_id,
                    attempt = u64::from(*consecutive_failures),
                    error = %error
                );
                if *consecutive_failures >= max_failures {
                    return Err(receiver_error(
                        effect_handler,
                        ReceiverErrorKind::Other,
                        error,
                    ));
                }
                let retry = tokio::time::sleep(retry_backoff);
                tokio::pin!(retry);
                loop {
                    tokio::select! {
                        biased;

                        () = deadline_elapsed(wait_deadline), if wait_deadline.is_some() => {
                            if let Some(stop) = stopping.request.get() {
                                return Ok(CommitOutcome::Stopped(stop));
                            }
                        }
                        control = ctrl_msg_recv.recv() => {
                            let Some(control) = handle_common_control(
                                control.map_err(Error::ChannelRecvError)?, metrics, admission,
                            ) else { continue };
                            count_discarded_feedback(&control, metrics);
                            if let Some(request) = stop_request(&control) {
                                let stop = stopping.record(request, Instant::now());
                                if matches!(stop, StopRequest::Shutdown(_)) {
                                    return Ok(CommitOutcome::Stopped(stop));
                                }
                                let deadline = stopping.wait_deadline.get().map(worker_stop_deadline);
                                wait_deadline = match (wait_deadline, deadline) {
                                    (Some(current), Some(deadline)) => Some(current.min(deadline)),
                                    (_, deadline) => deadline,
                                };
                            }
                        }
                        () = &mut retry => break,
                    }
                }
            }
        }
    }
}

enum CommitOutcome {
    Committed(CheckpointState),
    Stopped(StopRequest),
}

async fn await_database_operation_or_stop<F, T, C>(
    operation: F,
    cancellation: C,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
    abandoned: &Cell<bool>,
    admission: &PollAdmission,
    stopping: &StopState,
) -> Result<OperationOutcome<T>, Error>
where
    F: Future<Output = T>,
    C: DriverCancellation,
{
    tokio::pin!(operation);
    loop {
        tokio::select! {
            biased;

            control = ctrl_msg_recv.recv() => {
                match control {
                    Ok(control) => {
                        let Some(control) = handle_common_control(control, metrics, admission)
                        else { continue };
                        count_discarded_feedback(&control, metrics);
                        if let Some(stop) = stop_request(&control) {
                            let stop = stopping.record(stop, Instant::now());
                            if let Some(metrics) = metrics.as_mut() {
                                metrics.cancellations.add(1);
                            }
                            let deadline = stopping.wait_deadline.get().expect("stop recorded");
                            if !cancel_and_join(operation.as_mut(), &cancellation, deadline).await {
                                abandoned.set(true);
                            }
                            return Ok(OperationOutcome::Stopped(stop));
                        }
                    }
                    Err(error) => {
                        if let Some(metrics) = metrics.as_mut() {
                            metrics.cancellations.add(1);
                        }
                        if !cancel_and_join(operation.as_mut(), &cancellation, Instant::now() + WORKER_STOP_TIMEOUT).await {
                            abandoned.set(true);
                        }
                        return Err(Error::ChannelRecvError(error));
                    }
                }
            }
            result = &mut operation => return Ok(OperationOutcome::Completed(result)),
        }
    }
}

async fn cancel_and_join<F, C>(
    mut operation: Pin<&mut F>,
    cancellation: &C,
    deadline: Instant,
) -> bool
where
    F: Future,
    C: DriverCancellation,
{
    let deadline = worker_stop_deadline(deadline);
    tokio::time::timeout_at(deadline.into(), async {
        if let Err(error) = cancellation.cancel().await {
            otel_warn!(
                "database_receiver.cancellation_failed",
                error = %error,
                message = "Database receiver could not interrupt its active operation"
            );
        }
        // Join blocking work even when cancellation itself fails, so a
        // replacement receiver cannot overlap this operation.
        _ = operation.as_mut().await;
    })
    .await
    .is_ok()
}

#[allow(clippy::too_many_arguments)]
async fn send_or_stop(
    pdata: OtapPdata,
    ctrl_msg_recv: &mut local::ControlChannel<OtapPdata>,
    effect_handler: &local::EffectHandler<OtapPdata>,
    state: &mut ReceiverState,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
    stopping: &StopState,
    deferred_feedback: &mut Option<NodeControlMsg<OtapPdata>>,
    generation: u64,
    admission: &PollAdmission,
) -> Result<SendOutcome, Error> {
    let pdata = match effect_handler.try_send_message(pdata) {
        Ok(()) => return Ok(SendOutcome::Sent),
        Err(TypedError::ChannelSendError(SendError::Full(pdata))) => {
            admission.interrupt_cycle();
            pdata
        }
        Err(error) => return Err(error.into()),
    };
    // Backpressure: block on the downstream send while remaining drainable.
    let send = effect_handler.send_message(pdata);
    tokio::pin!(send);

    loop {
        tokio::select! {
            biased;

            () = deadline_elapsed(stopping.wait_deadline.get()), if stopping.request.get().is_some() => {
                let Some(stop) = stopping.request.get() else { continue };
                otel_warn!(
                    "database_receiver.drain_wait_expired",
                    message = "Downstream send wait ended to reserve worker cleanup time"
                );
                return Ok(SendOutcome::Stopped(stop));
            }
            control = ctrl_msg_recv.recv() => {
                let Some(control) = handle_common_control(
                    control.map_err(Error::ChannelRecvError)?, metrics, admission,
                ) else { continue };
                match control {
                    NodeControlMsg::DrainIngress { deadline, .. } => {
                        state.begin_drain();
                        _ = stopping.record(StopRequest::Drain(deadline), Instant::now());
                    }
                    NodeControlMsg::Shutdown { deadline, .. } => {
                        let stop = stopping.record(StopRequest::Shutdown(deadline), Instant::now());
                        return Ok(SendOutcome::Stopped(stop));
                    }
                    control @ (NodeControlMsg::Ack(_) | NodeControlMsg::Nack(_)) => {
                        let calldata = match &control {
                            NodeControlMsg::Ack(ack) => &ack.unwind.route.calldata,
                            NodeControlMsg::Nack(nack) => &nack.unwind.route.calldata,
                            _ => unreachable!(),
                        };
                        // Only one page can be in flight. Retain its first feedback
                        // without allowing stale or duplicate messages to grow a queue.
                        if deferred_feedback.is_none()
                            && batch_id_from_call_data(calldata, generation) == Some(state.next_batch_id)
                        {
                            *deferred_feedback = Some(control);
                        } else {
                            count_stale_feedback(metrics);
                        }
                    }
                    _ => {}
                }
            }
            result = &mut send => {
                result.map_err(Error::from)?;
                return Ok(SendOutcome::Sent);
            }
        }
    }
}

fn count_stale_feedback(metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>) {
    if let Some(metrics) = metrics.as_mut() {
        metrics.stale_feedback.add(1);
    }
}

/// During query/encoding and checkpoint waits, no new feedback can be accepted.
fn count_discarded_feedback(
    control: &NodeControlMsg<OtapPdata>,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
) {
    if matches!(control, NodeControlMsg::Ack(_) | NodeControlMsg::Nack(_)) {
        count_stale_feedback(metrics);
    }
}

/// Handles phase-independent controls and counts each newly received stop message.
/// The caller retains phase-specific ACK, drain and shutdown behavior.
fn handle_common_control(
    control: NodeControlMsg<OtapPdata>,
    metrics: &mut Option<MetricSet<DatabaseReceiverMetrics>>,
    admission: &PollAdmission,
) -> Option<NodeControlMsg<OtapPdata>> {
    match control {
        NodeControlMsg::CollectTelemetry {
            mut metrics_reporter,
        } => {
            if let Some(metrics) = metrics.as_mut() {
                _ = metrics_reporter.report(metrics);
            }
            None
        }
        NodeControlMsg::MemoryPressureChanged { update } => {
            admission.apply(update);
            None
        }
        control => {
            if let (Some(stop), Some(metrics)) = (stop_request(&control), metrics.as_mut()) {
                match stop {
                    StopRequest::Drain(_) => metrics.drains.add(1),
                    StopRequest::Shutdown(_) => metrics.shutdowns.add(1),
                }
            }
            Some(control)
        }
    }
}

fn stop_request(control: &NodeControlMsg<OtapPdata>) -> Option<StopRequest> {
    match control {
        NodeControlMsg::DrainIngress { deadline, .. } => Some(StopRequest::Drain(*deadline)),
        NodeControlMsg::Shutdown { deadline, .. } => Some(StopRequest::Shutdown(*deadline)),
        _ => None,
    }
}

fn stopped_state(
    stop: StopRequest,
    metrics: &Option<MetricSet<DatabaseReceiverMetrics>>,
) -> TerminalState {
    let deadline = stop.deadline();
    match metrics {
        Some(metrics) => TerminalState::new(deadline, [metrics.snapshot()]),
        None => TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []),
    }
}

fn observed_time_unix_nano() -> Result<u64, ObservationTimeError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    // OTLP uses u64 nanoseconds. Reject an unrepresentable clock value rather
    // than silently saturating and emitting a misleading timestamp.
    Ok(duration.as_nanos().try_into()?)
}

#[derive(Debug, thiserror::Error)]
enum ObservationTimeError {
    /// The system clock is earlier than the Unix epoch.
    #[error("system clock is earlier than the Unix epoch")]
    BeforeUnixEpoch(#[from] std::time::SystemTimeError),
    /// Nanoseconds since the epoch do not fit OTLP's unsigned 64-bit field.
    #[error("observation time is outside the supported OTLP range")]
    OutOfRange(#[from] std::num::TryFromIntError),
}

fn receiver_error(
    effect_handler: &local::EffectHandler<OtapPdata>,
    kind: ReceiverErrorKind,
    error: impl std::error::Error + 'static,
) -> Error {
    let source_detail = format_error_sources(&error);
    Error::ReceiverError {
        receiver: effect_handler.receiver_id(),
        kind,
        error: error.to_string(),
        source_detail,
    }
}

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;
