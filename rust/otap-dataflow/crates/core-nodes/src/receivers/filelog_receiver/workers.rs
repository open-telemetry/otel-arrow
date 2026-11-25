// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Fixed discovery and read/checkpoint threads, without receiver registration.
//!
//! Discovery owns only observation production. The read/checkpoint closure owns
//! the namespace, identity resolver, readers and checkpoint store; observations
//! must pass through that resolver before becoming source work. Composition with
//! the scheduler and delivery state machine belongs to receiver wiring.
//!
//! These are deliberately cross-thread transports, unlike pipeline-local
//! channels. `Send + 'static` is required only when moving work onto an OS thread.
//! Message sizes must also be bounded by the composing scanner/batch builder.

use futures::{FutureExt, TryFutureExt, executor::block_on, future::Either};
use parking_lot::Mutex;
use std::{
    io,
    num::NonZeroUsize,
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tokio::sync::{Notify, mpsc, oneshot};
use tokio_util::sync::CancellationToken;

#[cfg(test)]
mod tests;

/// Shared only across the fixed workers and their async owner. The mutex makes
/// the first cancellation's deadline atomic with respect to peer failure.
struct Shutdown {
    token: CancellationToken,
    deadline: Mutex<Option<Instant>>,
    timeout: Duration,
    #[cfg(test)]
    test_now: Mutex<Option<Instant>>,
}

/// Out-of-band cancellation; never competes for bounded command capacity.
#[derive(Clone)]
pub(crate) struct Cancellation(Arc<Shutdown>);

impl Cancellation {
    fn new(timeout: Duration) -> io::Result<Self> {
        if timeout.is_zero() || Instant::now().checked_add(timeout).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "filelog worker shutdown timeout must be nonzero and representable",
            ));
        }
        Ok(Self(Arc::new(Shutdown {
            token: CancellationToken::new(),
            deadline: Mutex::new(None),
            timeout,
            #[cfg(test)]
            test_now: Mutex::new(None),
        })))
    }

    fn now(&self) -> Instant {
        #[cfg(test)]
        if let Some(now) = *self.0.test_now.lock() {
            return now;
        }
        Instant::now()
    }

    /// Starts one shared deadline. Subsequent requests return the original one.
    pub(crate) fn cancel(&self) -> Instant {
        let deadline = {
            let mut deadline = self.0.deadline.lock();
            *deadline.get_or_insert_with(|| {
                self.now()
                    .checked_add(self.0.timeout)
                    .expect("validated filelog worker shutdown timeout")
            })
        };
        self.0.token.cancel();
        deadline
    }

    /// Check between bounded scanner, source, and checkpoint work units.
    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.token.is_cancelled()
    }

    /// Interrupt async work when either worker or its owner requests shutdown.
    pub(crate) async fn cancelled(&self) {
        self.0.token.cancelled().await;
    }

    /// Interruptible delay between discovery passes. No Tokio runtime is needed
    /// on the OS thread; the timer is polled by its small blocking executor.
    pub(crate) fn wait(&self, duration: Duration) -> Result<(), ChannelError> {
        block_on(self.wait_async(duration))
    }

    async fn wait_async(&self, duration: Duration) -> Result<(), ChannelError> {
        tokio::select! {
            biased;
            _ = self.0.token.cancelled() => Err(ChannelError::Cancelled),
            _ = futures_timer::Delay::new(duration) => Ok(()),
        }
    }
}

/// Transport termination, never an acknowledgement or checkpoint success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ChannelError {
    /// Cooperative cancellation was requested.
    #[error("filelog workers cancelled")]
    Cancelled,
    /// Discovery's observation endpoint has gone away.
    #[error("filelog observation channel closed")]
    ObservationsClosed,
    /// The async owner's command endpoint has gone away.
    #[error("filelog command channel closed")]
    CommandsClosed,
    /// The async owner's batch endpoint has gone away.
    #[error("filelog batch channel closed")]
    BatchesClosed,
}

/// One bounded observation queue, one batch slot, and one bounded command queue.
/// Standalone composition supplies the same cancellation for both workers;
/// coordinator-owned discovery may instead have its own lifetime.
pub(crate) fn channels<O, B, C>(
    observation_capacity: NonZeroUsize,
    command_capacity: NonZeroUsize,
    discovery_cancellation: Cancellation,
    read_cancellation: Cancellation,
) -> (Discovery<O>, ReadCheckpoint<O, B, C>, Async<B, C>) {
    let (observations_tx, observations_rx) = mpsc::channel(observation_capacity.get());
    let (batches_tx, batches_rx) = mpsc::channel(1);
    let (commands_tx, commands_rx) = mpsc::channel(command_capacity.get());
    (
        Discovery {
            observations: observations_tx,
            cancellation: discovery_cancellation,
        },
        ReadCheckpoint {
            observations: observations_rx,
            batches: batches_tx,
            commands: commands_rx,
            last_was_command: false,
            cancellation: read_cancellation.clone(),
        },
        Async {
            batches: batches_rx,
            commands: commands_tx,
            cancellation: read_cancellation,
        },
    )
}

/// Discovery endpoint, separable from the namespace/read-worker owner.
pub(crate) struct Discovery<O> {
    observations: mpsc::Sender<O>,
    cancellation: Cancellation,
}

impl<O> Discovery<O> {
    /// Stops traversal under backpressure without retaining filesystem handles.
    pub(crate) fn observe(&self, observation: O) -> Result<(), ChannelError> {
        block_on(self.observe_async(observation))
    }

    async fn observe_async(&self, observation: O) -> Result<(), ChannelError> {
        tokio::select! {
            biased;
            _ = self.cancellation.0.token.cancelled() => Err(ChannelError::Cancelled),
            result = self.observations.send(observation) =>
                result.map_err(|_| ChannelError::ObservationsClosed),
        }
    }
}

/// A read-worker turn. A due deadline takes precedence over a new observation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Input<O, C> {
    /// Resolve candidate evidence on this worker, not on discovery.
    Observation(O),
    /// Lifecycle, delivery completion, or other bounded owner command.
    Command(C),
    /// The caller must service due read, flush, retry, or checkpoint work.
    Deadline,
}

/// A command or deadline returns ownership of the batch so the worker can
/// service it, recompute its next deadline, and retry the same handoff.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Publish<B, C> {
    Sent,
    Command { batch: B, command: C },
    Deadline(B),
}

async fn deadline_timer(remaining: Option<Duration>) {
    match remaining {
        Some(Duration::ZERO) => {}
        Some(delay) => futures_timer::Delay::new(delay).await,
        None => std::future::pending().await,
    }
}

/// Prefer the first future when both are ready in the same poll.
async fn first_ready<T>(first: impl Future<Output = T>, second: impl Future<Output = T>) -> T {
    tokio::select! {
        biased;
        value = first => value,
        value = second => value,
    }
}

/// Commands get the first turn. After each command, ready work gets the next
/// turn even if the queue has already refilled. Cancellation always wins.
async fn command_or_work<C, W>(
    commands: &mut mpsc::Receiver<C>,
    cancellation: &Cancellation,
    last_was_command: &mut bool,
    work: impl Future<Output = Result<W, ChannelError>>,
) -> Result<Either<C, W>, ChannelError> {
    let command = async {
        commands
            .recv()
            .await
            .map(Either::Left)
            .ok_or(ChannelError::CommandsClosed)
    };
    let work = work.map_ok(Either::Right);
    let turn = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(ChannelError::Cancelled),
        result = async {
            if *last_was_command {
                first_ready(work, command).await
            } else {
                first_ready(command, work).await
            }
        } => result?,
    };
    *last_was_command = matches!(turn, Either::Left(_));
    Ok(turn)
}

/// Sole owner of read/checkpoint transport endpoints.
pub(crate) struct ReadCheckpoint<O, B, C> {
    observations: mpsc::Receiver<O>,
    batches: mpsc::Sender<B>,
    commands: mpsc::Receiver<C>,
    last_was_command: bool,
    cancellation: Cancellation,
}

impl<O, B, C> ReadCheckpoint<O, B, C> {
    /// Waits without polling, waking on input, channel closure, or cancellation.
    pub(crate) fn next(&mut self) -> Result<Input<O, C>, ChannelError> {
        self.next_until(true, None)
    }

    /// Wait for the earliest deadline supplied by the composing worker. Pausing
    /// intake leaves observations in the bounded queue while commands progress.
    /// Keep passing the deadline across command turns until its work is serviced.
    pub(crate) fn next_until(
        &mut self,
        intake_enabled: bool,
        deadline: Option<Instant>,
    ) -> Result<Input<O, C>, ChannelError> {
        if let Some(end) = deadline
            && end > self.cancellation.now()
            && let Some(input) = self
                .next_async(intake_enabled, std::future::pending())
                .now_or_never()
        {
            // A ready input needs no native timer or timer-thread wakeup.
            return input;
        }
        let remaining = deadline.map(|end| end.saturating_duration_since(self.cancellation.now()));
        block_on(self.next_async(intake_enabled, deadline_timer(remaining)))
    }

    async fn next_async(
        &mut self,
        intake_enabled: bool,
        timer: impl Future<Output = ()>,
    ) -> Result<Input<O, C>, ChannelError> {
        let work = async {
            tokio::select! {
                biased;
                _ = timer => Ok(Input::Deadline),
                observation = self.observations.recv(), if intake_enabled =>
                    observation.map(Input::Observation).ok_or(ChannelError::ObservationsClosed),
            }
        };
        match command_or_work(
            &mut self.commands,
            &self.cancellation,
            &mut self.last_was_command,
            work,
        )
        .await?
        {
            Either::Left(command) => Ok(Input::Command(command)),
            Either::Right(input) => Ok(input),
        }
    }

    /// Untimed handoff. Use `publish_until` when checkpoint or delivery work has deadlines.
    /// Retains at most this pending batch in addition to the single handoff slot.
    /// No more observations/source reads occur here. Commands remain serviceable
    /// even while downstream is stalled; callbacks must perform bounded work.
    pub(crate) fn publish(&mut self, batch: B, command: impl FnMut(C)) -> Result<(), ChannelError> {
        block_on(self.publish_async(batch, command))
    }

    async fn publish_async(
        &mut self,
        batch: B,
        mut command: impl FnMut(C),
    ) -> Result<(), ChannelError> {
        let mut pending = batch;
        loop {
            match self.publish_step(pending, std::future::pending()).await? {
                Publish::Sent => return Ok(()),
                Publish::Command {
                    batch,
                    command: next,
                } => {
                    command(next);
                    pending = batch;
                }
                Publish::Deadline(_) => unreachable!("untimed publish has no deadline"),
            }
        }
    }

    /// Wait for one handoff, command, or deadline without taking more input.
    /// A ready handoff wins over a due deadline; the caller retains that deadline
    /// for its next wait. Handle a returned command or due work before retrying.
    pub(crate) fn publish_until(
        &mut self,
        batch: B,
        deadline: Option<Instant>,
    ) -> Result<Publish<B, C>, ChannelError> {
        let remaining = deadline.map(|end| end.saturating_duration_since(self.cancellation.now()));
        block_on(self.publish_step(batch, deadline_timer(remaining)))
    }

    async fn publish_step(
        &mut self,
        batch: B,
        timer: impl Future<Output = ()>,
    ) -> Result<Publish<B, C>, ChannelError> {
        let handoff = async {
            tokio::select! {
                biased;
                permit = self.batches.reserve() =>
                    permit.map(Some).map_err(|_| ChannelError::BatchesClosed),
                _ = timer => Ok(None),
            }
        };
        match command_or_work(
            &mut self.commands,
            &self.cancellation,
            &mut self.last_was_command,
            handoff,
        )
        .await?
        {
            Either::Left(command) => Ok(Publish::Command { batch, command }),
            Either::Right(Some(permit)) => {
                permit.send(batch);
                Ok(Publish::Sent)
            }
            Either::Right(None) => Ok(Publish::Deadline(batch)),
        }
    }
}

/// Pipeline-local async endpoint. Its futures do not impose `Send` bounds.
pub(crate) struct Async<B, C> {
    batches: mpsc::Receiver<B>,
    commands: mpsc::Sender<C>,
    cancellation: Cancellation,
}

impl<B, C> Async<B, C> {
    /// Dropping this future while it waits for capacity drops the command.
    /// Use `reserve_command` when interruptible forwarding must retain an Ack.
    pub(crate) async fn command(&self, command: C) -> Result<(), ChannelError> {
        self.reserve_command().await?.send(command);
        Ok(())
    }

    /// Reserve a slot before moving an Ack. Control handling can interrupt this
    /// wait and retry with the same Ack while the receiver continues running.
    pub(crate) async fn reserve_command(&self) -> Result<mpsc::Permit<'_, C>, ChannelError> {
        tokio::select! {
            biased;
            _ = self.cancellation.0.token.cancelled() => Err(ChannelError::Cancelled),
            result = self.commands.reserve() => result.map_err(|_| ChannelError::CommandsClosed),
        }
    }

    /// Receives the single batch handoff without blocking the pipeline thread.
    pub(crate) async fn batch(&mut self) -> Result<B, ChannelError> {
        tokio::select! {
            biased;
            _ = self.cancellation.0.token.cancelled() => Err(ChannelError::Cancelled),
            batch = self.batches.recv() => batch.ok_or(ChannelError::BatchesClosed),
        }
    }
}

/// One started worker's termination observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkerOutcome {
    /// Native join completed, including thread-local cleanup.
    Joined {
        /// Publication was late or an earlier join already reported `StillRunning`.
        deadline_missed: bool,
    },
    /// The OS thread terminated with a panic, including its diagnostic.
    Panicked {
        /// The worker's panic diagnostic.
        message: String,
        /// Publication was late or an earlier join already reported `StillRunning`.
        deadline_missed: bool,
    },
    /// Native join is incomplete; its observer retains the handle and resources.
    StillRunning,
}

/// A missing entry means this owner never started that worker.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct JoinReport {
    pub(crate) discovery: Option<WorkerOutcome>,
    pub(crate) read_checkpoint: Option<WorkerOutcome>,
}

struct Worker {
    // Fixed per-worker observers avoid both pipeline blocking and dependence on
    // Tokio's shared blocking pool, which a stuck native join could exhaust.
    _observer: JoinHandle<()>,
    reported_timeout: bool,
    completion: Arc<JoinCompletion>,
}

struct Completed {
    at: Instant,
    result: Result<(), String>,
}

struct JoinCompletion {
    completed: Mutex<Option<Completed>>,
    ready: Notify,
}

impl JoinCompletion {
    /// The clock runs under the publication lock and must be cheap and nonblocking.
    fn publish(&self, result: Result<(), String>, now: impl FnOnce() -> Instant) {
        {
            let mut completed = self.completed.lock();
            *completed = Some(Completed { at: now(), result });
        }
        self.ready.notify_one();
    }
}

impl Worker {
    fn start(
        name: &str,
        cancellation: &Cancellation,
        work: impl FnOnce(Cancellation) + Send + 'static,
    ) -> io::Result<Self> {
        let completion = Arc::new(JoinCompletion {
            completed: Mutex::new(None),
            ready: Notify::new(),
        });
        let observed_completion = completion.clone();
        let observer_cancellation = cancellation.clone();
        let (handle_tx, handle_rx) = oneshot::channel::<JoinHandle<()>>();
        // Start the observer first, so failure to create it cannot orphan a
        // running worker. Failed worker startup closes the empty handle handoff.
        let observer = thread::Builder::new()
            .name(format!("{name}-join"))
            .spawn(move || {
                if let Ok(handle) = block_on(handle_rx) {
                    let result = handle.join();
                    let result = result.map_err(|payload| {
                        payload
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                            .unwrap_or_else(|| "non-string panic payload".to_owned())
                    });
                    observed_completion.publish(result, || observer_cancellation.now());
                }
            })?;
        let worker_cancellation = cancellation.clone();
        let handle = thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || {
                // Runs on normal return and unwind, including partial startup.
                struct CancelOnExit(Cancellation);
                impl Drop for CancelOnExit {
                    fn drop(&mut self) {
                        _ = self.0.cancel();
                    }
                }
                let _on_exit = CancelOnExit(worker_cancellation.clone());
                work(worker_cancellation);
            })?;
        handle_tx
            .send(handle)
            .expect("started filelog join observer owns the handle receiver");
        Ok(Self {
            _observer: observer,
            reported_timeout: false,
            completion,
        })
    }

    async fn join(&mut self, deadline: Instant, now: impl Fn() -> Instant) -> WorkerOutcome {
        loop {
            // notify_one retains a permit if completion races this observation.
            let ready = self.completion.ready.notified();
            let remaining = {
                // Keep the slot check and clock read under the same lock. Otherwise
                // an on-time publication between them could be reported as a timeout.
                let published = self.completion.completed.lock();
                if let Some(completed) = published.as_ref() {
                    let deadline_missed = self.reported_timeout || completed.at > deadline;
                    return match &completed.result {
                        Ok(()) => WorkerOutcome::Joined { deadline_missed },
                        Err(message) => WorkerOutcome::Panicked {
                            message: message.clone(),
                            deadline_missed,
                        },
                    };
                }
                let remaining = deadline.saturating_duration_since(now());
                if remaining.is_zero() {
                    self.reported_timeout = true;
                    return WorkerOutcome::StillRunning;
                }
                remaining
            };
            tokio::select! {
                _ = ready => {}
                _ = tokio::time::sleep(remaining) => {}
            }
        }
    }
}

/// Standalone owner: either thread's return or panic cancels its peer.
///
/// Start methods are separate so a future coordinator can own discovery in its
/// own worker owner. Dropping this owner requests cancellation, not successful
/// cleanup. Fixed join observers retain timed-out native handles; neither timeout
/// nor drop releases resources still held by a worker, including thread-local
/// cleanup. Process-fatal escalation is an engine integration requirement, not
/// implemented by this transport.
pub(crate) struct Workers {
    cancellation: Cancellation,
    discovery: Option<Worker>,
    read_checkpoint: Option<Worker>,
}

impl Workers {
    pub(crate) fn new(shutdown_timeout: Duration) -> io::Result<Self> {
        Ok(Self {
            cancellation: Cancellation::new(shutdown_timeout)?,
            discovery: None,
            read_checkpoint: None,
        })
    }

    pub(crate) fn cancellation(&self) -> Cancellation {
        self.cancellation.clone()
    }

    pub(crate) fn start_discovery(
        &mut self,
        work: impl FnOnce(Cancellation) + Send + 'static,
    ) -> io::Result<()> {
        Self::start(
            &mut self.discovery,
            "filelog-discovery",
            &self.cancellation,
            work,
        )
    }

    pub(crate) fn start_read_checkpoint(
        &mut self,
        work: impl FnOnce(Cancellation) + Send + 'static,
    ) -> io::Result<()> {
        Self::start(
            &mut self.read_checkpoint,
            "filelog-read-checkpoint",
            &self.cancellation,
            work,
        )
    }

    fn start(
        slot: &mut Option<Worker>,
        name: &str,
        cancellation: &Cancellation,
        work: impl FnOnce(Cancellation) + Send + 'static,
    ) -> io::Result<()> {
        if slot.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "filelog worker already started",
            ));
        }
        if cancellation.is_cancelled() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "filelog worker owner already cancelled",
            ));
        }
        match Worker::start(name, cancellation, work) {
            Ok(worker) => {
                *slot = Some(worker);
                Ok(())
            }
            Err(error) => {
                _ = cancellation.cancel();
                Err(error)
            }
        }
    }

    /// Requests cancellation and observes both joins against the same deadline.
    /// This also works for partially started sets. A later call can reap a worker
    /// that outlived the deadline, but cannot extend the deadline or undo timeout.
    pub(crate) async fn join(&mut self) -> JoinReport {
        let deadline = self.cancellation.cancel();
        let join = async |worker: &mut Option<Worker>| match worker {
            Some(worker) => Some(worker.join(deadline, || self.cancellation.now()).await),
            None => None,
        };
        let (discovery, read_checkpoint) =
            tokio::join!(join(&mut self.discovery), join(&mut self.read_checkpoint));
        JoinReport {
            discovery,
            read_checkpoint,
        }
    }
}

impl Drop for Workers {
    fn drop(&mut self) {
        _ = self.cancellation.cancel();
    }
}
