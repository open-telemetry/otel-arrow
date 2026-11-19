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

use futures::executor::block_on;
use parking_lot::Mutex;
use std::{
    io,
    num::NonZeroUsize,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};
use tokio::{sync::mpsc, time::Instant};
use tokio_util::sync::CancellationToken;

#[cfg(test)]
mod tests;

/// Shared only across the fixed workers and their async owner. The mutex makes
/// the first cancellation's deadline atomic with respect to peer failure.
struct Shutdown {
    token: CancellationToken,
    deadline: Mutex<Option<Instant>>,
    timeout: Duration,
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
        })))
    }

    /// Starts one shared deadline. Subsequent requests return the original one.
    pub(crate) fn cancel(&self) -> Instant {
        let deadline = {
            let mut deadline = self.0.deadline.lock();
            *deadline.get_or_insert_with(|| {
                Instant::now()
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

/// Input to the read/checkpoint worker; commands have priority over observations.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Input<O, C> {
    /// Resolve candidate evidence on this worker, not on discovery.
    Observation(O),
    /// Lifecycle, delivery completion, or other bounded owner command.
    Command(C),
}

/// Sole owner of read/checkpoint transport endpoints.
pub(crate) struct ReadCheckpoint<O, B, C> {
    observations: mpsc::Receiver<O>,
    batches: mpsc::Sender<B>,
    commands: mpsc::Receiver<C>,
    cancellation: Cancellation,
}

impl<O, B, C> ReadCheckpoint<O, B, C> {
    /// Waits without polling, waking on input, channel closure, or cancellation.
    pub(crate) fn next(&mut self) -> Result<Input<O, C>, ChannelError> {
        block_on(self.next_async())
    }

    async fn next_async(&mut self) -> Result<Input<O, C>, ChannelError> {
        tokio::select! {
            biased;
            _ = self.cancellation.0.token.cancelled() => Err(ChannelError::Cancelled),
            command = self.commands.recv() =>
                command.map(Input::Command).ok_or(ChannelError::CommandsClosed),
            observation = self.observations.recv() =>
                observation.map(Input::Observation).ok_or(ChannelError::ObservationsClosed),
        }
    }

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
        loop {
            tokio::select! {
                biased;
                _ = self.cancellation.0.token.cancelled() => return Err(ChannelError::Cancelled),
                next = self.commands.recv() => {
                    command(next.ok_or(ChannelError::CommandsClosed)?);
                }
                permit = self.batches.reserve() => {
                    permit.map_err(|_| ChannelError::BatchesClosed)?.send(batch);
                    return Ok(());
                }
            }
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
    /// A blocked command send can itself be cancelled independently of capacity.
    pub(crate) async fn command(&self, command: C) -> Result<(), ChannelError> {
        tokio::select! {
            biased;
            _ = self.cancellation.0.token.cancelled() => Err(ChannelError::Cancelled),
            result = self.commands.send(command) =>
                result.map_err(|_| ChannelError::CommandsClosed),
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
    /// The OS thread terminated and its join succeeded.
    Joined,
    /// The OS thread terminated with a panic, including its diagnostic.
    Panicked(String),
    /// The thread has not terminated; its handle and resources remain owned.
    StillRunning,
}

/// A missing entry means this owner never started that worker.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct JoinReport {
    pub(crate) discovery: Option<WorkerOutcome>,
    pub(crate) read_checkpoint: Option<WorkerOutcome>,
}

struct Worker {
    handle: Option<JoinHandle<()>>,
    outcome: Option<WorkerOutcome>,
}

impl Worker {
    async fn join(&mut self, deadline: Instant) -> WorkerOutcome {
        if let Some(outcome) = &self.outcome {
            return outcome.clone();
        }
        let handle = self.handle.as_ref().expect("unjoined worker handle");
        while !handle.is_finished() {
            if Instant::now() >= deadline {
                return WorkerOutcome::StillRunning;
            }
            // Only observe on the async thread; never send an unbounded join to
            // Tokio's blocking pool or block the pipeline waiting for kernel I/O.
            tokio::time::sleep_until(deadline.min(Instant::now() + Duration::from_millis(1))).await;
        }
        let outcome = match self.handle.take().expect("finished worker handle").join() {
            Ok(()) => WorkerOutcome::Joined,
            Err(payload) => WorkerOutcome::Panicked(
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                    .unwrap_or_else(|| "non-string panic payload".to_owned()),
            ),
        };
        self.outcome = Some(outcome.clone());
        outcome
    }
}

/// Standalone owner: either thread's return or panic cancels its peer.
///
/// Start methods are separate so a future coordinator can own discovery in its
/// own worker owner. Dropping this owner requests cancellation, not successful
/// cleanup. Timed-out handles stay here; neither timeout nor drop releases any
/// resource still held by a running thread. Process-fatal escalation is an engine
/// integration requirement, not implemented by this transport.
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
        let cancellation = cancellation.clone();
        let worker_cancellation = cancellation.clone();
        match thread::Builder::new().name(name.to_owned()).spawn(move || {
            // Runs on normal return and unwind, including partial startup.
            struct CancelOnExit(Cancellation);
            impl Drop for CancelOnExit {
                fn drop(&mut self) {
                    _ = self.0.cancel();
                }
            }
            let _on_exit = CancelOnExit(worker_cancellation.clone());
            work(worker_cancellation);
        }) {
            Ok(handle) => {
                *slot = Some(Worker {
                    handle: Some(handle),
                    outcome: None,
                });
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
            Some(worker) => Some(worker.join(deadline).await),
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
