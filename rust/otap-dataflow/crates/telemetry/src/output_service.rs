// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Process-wide console output service.
//!
//! Engine cores format console output into complete, self-contained [`Frame`]s
//! and hand them to a bounded queue. One dedicated writer thread per stream
//! drains that queue and performs the blocking write while holding the
//! corresponding standard-stream lock for the whole frame. That gives two
//! guarantees per-core `tokio::io::stdout()` writes could not provide:
//!
//! * no blocking console I/O runs on an engine core thread, and
//! * concurrent producers can never interleave bytes inside one frame, even
//!   when the payload is larger than a single underlying write.
//!
//! Ordering is FIFO per producer. Global ordering across cores is not
//! guaranteed and is out of scope.
//!
//! The service is optional. When [`OutputService::init`] was never called the
//! stream handles fall back to direct, locked writes so unit tests and
//! standalone binaries keep their current behavior.
//!
//! The integrity guarantee covers frames submitted through this service only.
//! Writers that bypass it -- raw file descriptors, inherited child processes,
//! and separate binaries -- are excluded.

use crate::otel_error;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

/// Default bounded queue capacity, in frames, for the stdout writer.
pub const DEFAULT_STDOUT_QUEUE_CAPACITY: usize = 1024;

/// Default bounded queue capacity, in frames, for the stderr writer.
pub const DEFAULT_STDERR_QUEUE_CAPACITY: usize = 256;

/// Default upper bound on how long shutdown waits for queued frames to drain.
pub const DEFAULT_SHUTDOWN_DRAIN_DEADLINE: Duration = Duration::from_secs(5);

/// Bytes written since the last flush that force a safety flush.
const FLUSH_BYTES_THRESHOLD: usize = 256 * 1024;

/// Frames written since the last flush that force a safety flush.
const FLUSH_FRAMES_THRESHOLD: usize = 256;

/// Marks a stream closed in the packed admission state; the low bits count
/// enqueue attempts that were admitted but have not resolved yet.
const ADMISSION_CLOSED: u64 = 1 << 63;

/// Poll interval while shutdown waits for admitted enqueues to land.
const ENQUEUE_SETTLE_POLL: Duration = Duration::from_micros(200);

/// Set when stdout carries machine-readable records instead of prose.
static STRUCTURED_STDOUT: AtomicBool = AtomicBool::new(false);

/// The process-wide streams, present only after a successful init.
static SERVICE: OnceLock<GlobalStreams> = OnceLock::new();

/// Identifies which standard stream an [`OutputStream`] serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamId {
    /// Process standard output.
    Stdout,
    /// Process standard error.
    Stderr,
}

impl StreamId {
    /// Returns the stream name used in telemetry attributes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }

    /// Returns the writer thread name for this stream.
    const fn thread_name(self) -> &'static str {
        match self {
            Self::Stdout => "otap-stdout-writer",
            Self::Stderr => "otap-stderr-writer",
        }
    }

    /// Returns a sink writing to the real standard stream.
    fn std_sink(self) -> Box<dyn OutputSink> {
        match self {
            Self::Stdout => Box::new(StdoutSink),
            Self::Stderr => Box::new(StderrSink),
        }
    }
}

/// A complete, self-contained unit of console output.
///
/// A frame is written contiguously: the writer thread holds the stream lock for
/// the whole payload, so no other producer can interleave bytes inside it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frame {
    bytes: Vec<u8>,
}

impl Frame {
    /// Wraps already-formatted bytes.
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Wraps one or more complete newline-terminated JSON records.
    ///
    /// # Panics
    ///
    /// Panics in debug builds when the payload does not end with a newline,
    /// which would let a record straddle two frames.
    #[must_use]
    pub fn new_record_json(bytes: Vec<u8>) -> Self {
        debug_assert!(
            bytes.is_empty() || bytes.last() == Some(&b'\n'),
            "a record_json frame must end with a newline"
        );
        Self { bytes }
    }

    /// Builds a frame from a message, appending the terminating newline.
    #[must_use]
    pub fn line(message: &str) -> Self {
        let mut bytes = Vec::with_capacity(message.len() + 1);
        bytes.extend_from_slice(message.as_bytes());
        bytes.push(b'\n');
        Self { bytes }
    }

    /// Borrows the frame payload.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the payload length in bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns true when the frame carries no bytes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// Reasons a frame could not be accepted by an output stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SubmitError {
    /// The stream is shutting down or its writer thread has exited.
    #[error("console output queue is closed")]
    QueueClosed,
    /// The writer thread stopped after an unrecoverable write error.
    #[error("console output writer is unavailable")]
    WriterUnavailable,
    /// A non-blocking submit found the queue full.
    #[error("console output queue is full")]
    WouldBlock,
}

/// Destination for frames drained by a writer thread.
pub trait OutputSink: Send {
    /// Writes one complete frame.
    fn write_frame(&mut self, frame: &[u8]) -> io::Result<()>;

    /// Flushes buffered bytes to the operating system.
    fn flush(&mut self) -> io::Result<()>;
}

/// Sink writing complete frames to the process standard output.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdoutSink;

impl OutputSink for StdoutSink {
    fn write_frame(&mut self, frame: &[u8]) -> io::Result<()> {
        // The lock is held for the whole frame so partial underlying writes
        // cannot interleave with another producer.
        let stdout = io::stdout();
        let mut locked = stdout.lock();
        locked.write_all(frame)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

/// Sink writing complete frames to the process standard error.
#[derive(Debug, Default, Clone, Copy)]
pub struct StderrSink;

impl OutputSink for StderrSink {
    fn write_frame(&mut self, frame: &[u8]) -> io::Result<()> {
        let stderr = io::stderr();
        let mut locked = stderr.lock();
        locked.write_all(frame)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stderr().flush()
    }
}

/// Point-in-time snapshot of one stream's counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutputStats {
    /// Frames whose enqueue the producer observed succeeding.
    ///
    /// A submit that is cancelled after the queue took its frame is written
    /// without being counted here, so this is a lower bound on frames accepted.
    pub frames_submitted: u64,
    /// Frames rejected because the queue was closed, full, or unavailable.
    pub frames_enqueue_failed: u64,
    /// Frames written to the sink.
    pub frames_written: u64,
    /// Bytes written to the sink.
    pub bytes_written: u64,
    /// Failed sink writes. Any failure stops the writer.
    pub write_errors: u64,
    /// Best-effort diagnostics dropped because the queue was full.
    pub diagnostics_dropped: u64,
    /// Accepted frames still queued when the drain deadline expired.
    pub frames_dropped_shutdown: u64,
    /// Highest observed number of accepted-but-unwritten frames.
    pub queue_depth_high_water: u64,
}

/// Snapshot of both process-wide streams.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServiceStats {
    /// Counters for the stdout stream.
    pub stdout: OutputStats,
    /// Counters for the stderr stream.
    pub stderr: OutputStats,
}

/// Result of draining an output stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownOutcome {
    /// Whether every accepted frame was written and flushed successfully.
    pub drained: bool,
    /// Whether a writer stopped on an I/O error rather than running out of time.
    pub writer_failed: bool,
    /// Accepted frames still unwritten when the operation returned.
    pub frames_pending: u64,
}

impl Default for ShutdownOutcome {
    fn default() -> Self {
        Self {
            drained: true,
            writer_failed: false,
            frames_pending: 0,
        }
    }
}

impl ShutdownOutcome {
    /// Folds another stream's outcome into this one.
    fn merge(&mut self, other: Self) {
        self.drained &= other.drained;
        self.writer_failed |= other.writer_failed;
        self.frames_pending = self.frames_pending.saturating_add(other.frames_pending);
    }
}

/// Configuration for the process-wide console output service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputServiceConfig {
    /// Bounded queue capacity, in frames, for the stdout writer.
    pub stdout_queue_capacity: usize,
    /// Bounded queue capacity, in frames, for the stderr writer.
    pub stderr_queue_capacity: usize,
    /// Flush whenever the queue is momentarily empty, bounding output latency.
    pub flush_on_idle: bool,
    /// Upper bound on how long shutdown waits for queued frames to drain.
    pub shutdown_drain_deadline: Duration,
}

impl Default for OutputServiceConfig {
    fn default() -> Self {
        Self {
            stdout_queue_capacity: DEFAULT_STDOUT_QUEUE_CAPACITY,
            stderr_queue_capacity: DEFAULT_STDERR_QUEUE_CAPACITY,
            flush_on_idle: true,
            shutdown_drain_deadline: DEFAULT_SHUTDOWN_DRAIN_DEADLINE,
        }
    }
}

/// Queue item. `Stop` ends the writer loop after everything ahead of it drains.
enum Command {
    Frame(Frame),
    /// Flush everything queued ahead of this barrier, then acknowledge.
    Barrier(flume::Sender<()>),
    Stop,
}

/// Why a writer thread stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriterExit {
    /// Every queued frame was written and the final flush succeeded.
    Drained,
    /// A write or flush failed, so queued frames were abandoned.
    Failed,
}

/// State shared between the producers and the writer thread of one stream.
#[derive(Debug, Default)]
struct StreamShared {
    /// Packed [`ADMISSION_CLOSED`] flag plus the count of unresolved enqueues.
    admission: AtomicU64,
    unavailable: AtomicBool,
    queue_depth_high_water: AtomicU64,
    frames_submitted: AtomicU64,
    frames_enqueue_failed: AtomicU64,
    frames_written: AtomicU64,
    bytes_written: AtomicU64,
    write_errors: AtomicU64,
    diagnostics_dropped: AtomicU64,
    frames_dropped_shutdown: AtomicU64,
}

impl StreamShared {
    /// Admits one enqueue attempt, or fails once the stream stopped accepting frames.
    ///
    /// Admission and the in-flight count move together, so a stream that observes
    /// zero in-flight attempts after closing knows no further frame can be queued.
    fn enter(&self) -> bool {
        let mut state = self.admission.load(Ordering::Acquire);
        loop {
            if state & ADMISSION_CLOSED != 0 {
                return false;
            }
            match self.admission.compare_exchange_weak(
                state,
                state + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(actual) => state = actual,
            }
        }
    }

    /// Marks one admitted enqueue attempt as resolved.
    fn leave(&self) {
        let _ = self.admission.fetch_sub(1, Ordering::AcqRel);
    }

    /// Stops admitting new enqueue attempts.
    fn close_admission(&self) {
        let _ = self.admission.fetch_or(ADMISSION_CLOSED, Ordering::AcqRel);
    }

    /// Returns the number of admitted enqueue attempts that have not resolved.
    fn enqueues_in_flight(&self) -> u64 {
        self.admission.load(Ordering::Acquire) & !ADMISSION_CLOSED
    }

    /// Returns the frames the writer accepted but has not written yet.
    ///
    /// Counted from the write side rather than the queue, so a frame the writer
    /// is still inside `write_frame` for is reported as pending.
    fn frames_pending(&self) -> u64 {
        self.frames_submitted
            .load(Ordering::Acquire)
            .saturating_sub(self.frames_written.load(Ordering::Acquire))
    }

    fn snapshot(&self) -> OutputStats {
        OutputStats {
            frames_submitted: self.frames_submitted.load(Ordering::Relaxed),
            frames_enqueue_failed: self.frames_enqueue_failed.load(Ordering::Relaxed),
            frames_written: self.frames_written.load(Ordering::Relaxed),
            bytes_written: self.bytes_written.load(Ordering::Relaxed),
            write_errors: self.write_errors.load(Ordering::Relaxed),
            diagnostics_dropped: self.diagnostics_dropped.load(Ordering::Relaxed),
            frames_dropped_shutdown: self.frames_dropped_shutdown.load(Ordering::Relaxed),
            queue_depth_high_water: self.queue_depth_high_water.load(Ordering::Relaxed),
        }
    }
}

/// Producer side of a running stream.
#[derive(Debug, Clone)]
struct QueuedHandle {
    sender: flume::Sender<Command>,
    shared: Arc<StreamShared>,
}

impl QueuedHandle {
    /// Admits one enqueue attempt, failing fast when the stream no longer accepts frames.
    fn admit(&self) -> Result<Admission<'_>, SubmitError> {
        if self.shared.unavailable.load(Ordering::Acquire) {
            return Err(SubmitError::WriterUnavailable);
        }
        if !self.shared.enter() {
            return Err(SubmitError::QueueClosed);
        }
        Ok(Admission {
            shared: &self.shared,
        })
    }

    fn rejected(&self, error: SubmitError) -> SubmitError {
        let _ = self
            .shared
            .frames_enqueue_failed
            .fetch_add(1, Ordering::Relaxed);
        error
    }
}

/// Cheap, clonable producer handle for one output stream.
#[derive(Debug, Clone)]
pub struct StreamHandle {
    id: StreamId,
    queued: Option<QueuedHandle>,
}
/// One admitted enqueue attempt, released when it resolves.
///
/// `submit` can be cancelled at its await point, so the release happens in
/// `Drop`. Without it a cancelled send would leave the stream permanently
/// believing an enqueue is still in flight, and shutdown could never settle.
struct Admission<'a> {
    shared: &'a StreamShared,
}

impl Admission<'_> {
    /// Records a frame the writer now owns.
    fn commit(&self) {
        let submitted = self.shared.frames_submitted.fetch_add(1, Ordering::AcqRel) + 1;
        let written = self.shared.frames_written.load(Ordering::Acquire);
        let _ = self
            .shared
            .queue_depth_high_water
            .fetch_max(submitted.saturating_sub(written), Ordering::Relaxed);
    }
}

impl Drop for Admission<'_> {
    fn drop(&mut self) {
        self.shared.leave();
    }
}

impl StreamHandle {
    /// Returns a handle that writes directly to the standard stream.
    #[must_use]
    pub const fn direct(id: StreamId) -> Self {
        Self { id, queued: None }
    }

    /// Returns the stream this handle feeds.
    #[must_use]
    pub const fn stream_id(&self) -> StreamId {
        self.id
    }

    /// Returns true when this handle writes directly instead of through a writer thread.
    #[must_use]
    pub const fn is_direct(&self) -> bool {
        self.queued.is_none()
    }

    /// Submits a frame, awaiting queue capacity when the queue is full.
    ///
    /// This is the backpressure path: a full queue slows the producer down
    /// instead of dropping data.
    ///
    /// # Errors
    ///
    /// Returns [`SubmitError`] when the stream is closing, its writer stopped,
    /// or a direct fallback write failed.
    pub async fn submit(&self, frame: Frame) -> Result<(), SubmitError> {
        let Some(queued) = self.queued.as_ref() else {
            return write_direct(self.id, &frame);
        };
        let admission = queued.admit().map_err(|error| queued.rejected(error))?;
        match queued.sender.send_async(Command::Frame(frame)).await {
            Ok(()) => {
                admission.commit();
                Ok(())
            }
            Err(_) => Err(queued.rejected(SubmitError::QueueClosed)),
        }
    }

    /// Submits a frame without ever blocking the calling thread.
    ///
    /// Synchronous callers such as the self-tracing layer run on engine core
    /// threads and must not stall, so a full queue drops the frame and counts
    /// it in `diagnostics_dropped` instead.
    ///
    /// # Errors
    ///
    /// Returns [`SubmitError::WouldBlock`] when the queue is full, or the other
    /// [`SubmitError`] variants when the stream no longer accepts frames.
    pub fn try_submit(&self, frame: Frame) -> Result<(), SubmitError> {
        let Some(queued) = self.queued.as_ref() else {
            return write_direct(self.id, &frame);
        };
        let admission = queued.admit().map_err(|error| queued.rejected(error))?;
        match queued.sender.try_send(Command::Frame(frame)) {
            Ok(()) => {
                admission.commit();
                Ok(())
            }
            Err(flume::TrySendError::Full(_)) => {
                let _ = queued
                    .shared
                    .diagnostics_dropped
                    .fetch_add(1, Ordering::Relaxed);
                Err(queued.rejected(SubmitError::WouldBlock))
            }
            Err(flume::TrySendError::Disconnected(_)) => {
                Err(queued.rejected(SubmitError::QueueClosed))
            }
        }
    }

    /// Returns a snapshot of this stream's counters.
    #[must_use]
    pub fn stats(&self) -> OutputStats {
        self.queued
            .as_ref()
            .map(|queued| queued.shared.snapshot())
            .unwrap_or_default()
    }
}

/// Writes a frame straight to the standard stream, used when the service is not initialized.
fn write_direct(id: StreamId, frame: &Frame) -> Result<(), SubmitError> {
    let mut sink = id.std_sink();
    sink.write_frame(frame.as_bytes())
        .and_then(|()| sink.flush())
        .map_err(|_| SubmitError::WriterUnavailable)
}

/// Marks the stream unavailable once the writer thread exits, including on panic.
struct TeardownGuard {
    shared: Arc<StreamShared>,
}

impl Drop for TeardownGuard {
    fn drop(&mut self) {
        self.shared.unavailable.store(true, Ordering::Release);
    }
}

/// Owner of one stream's bounded queue and dedicated writer thread.
pub struct OutputStream {
    handle: StreamHandle,
    shared: Arc<StreamShared>,
    sender: flume::Sender<Command>,
    done: flume::Receiver<WriterExit>,
    worker: Option<thread::JoinHandle<()>>,
}

impl OutputStream {
    /// Starts a stream with a caller-supplied sink.
    ///
    /// Tests use this entry point so they never touch the real standard streams.
    ///
    /// # Errors
    ///
    /// Returns the [`io::Error`] produced when the writer thread cannot be spawned.
    pub fn start(
        id: StreamId,
        capacity: usize,
        flush_on_idle: bool,
        sink: Box<dyn OutputSink>,
    ) -> io::Result<Self> {
        let (sender, receiver) = flume::bounded::<Command>(capacity.max(1));
        let (done_tx, done) = flume::bounded::<WriterExit>(1);
        let shared = Arc::new(StreamShared::default());
        let worker_shared = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name(id.thread_name().to_owned())
            .spawn(move || {
                let _guard = TeardownGuard {
                    shared: Arc::clone(&worker_shared),
                };
                let exit = run_writer(id, receiver, sink, &worker_shared, flush_on_idle);
                // A panic instead drops this sender, which callers read as a failed drain.
                let _ = done_tx.send(exit);
            })?;

        Ok(Self {
            handle: StreamHandle {
                id,
                queued: Some(QueuedHandle {
                    sender: sender.clone(),
                    shared: Arc::clone(&shared),
                }),
            },
            shared,
            sender,
            done,
            worker: Some(worker),
        })
    }

    /// Returns a producer handle for this stream.
    #[must_use]
    pub fn handle(&self) -> StreamHandle {
        self.handle.clone()
    }

    /// Returns a snapshot of this stream's counters.
    #[must_use]
    pub fn stats(&self) -> OutputStats {
        self.shared.snapshot()
    }

    /// Writes and flushes everything accepted so far, leaving the writer running.
    ///
    /// A barrier queued behind the accepted frames is acknowledged only after they
    /// reach the stream, so this reports a completed drain without tearing the
    /// writer down. The stream keeps accepting frames afterwards.
    #[must_use]
    pub fn drain(&self, deadline: Duration) -> ShutdownOutcome {
        let started = Instant::now();
        let (ack_tx, ack_rx) = flume::bounded::<()>(1);
        if self
            .sender
            .send_timeout(Command::Barrier(ack_tx), deadline)
            .is_err()
        {
            return self.pending_outcome();
        }
        let remaining = deadline.saturating_sub(started.elapsed());
        match ack_rx.recv_timeout(remaining) {
            Ok(()) => ShutdownOutcome::default(),
            Err(_) => self.pending_outcome(),
        }
    }

    /// Stops accepting frames, drains what was accepted, and flushes.
    ///
    /// Returns once the writer finishes or the deadline expires, whichever comes
    /// first, so a stalled console pipe cannot block the caller indefinitely.
    pub fn shutdown(&mut self, deadline: Duration) -> ShutdownOutcome {
        let started = Instant::now();
        self.shared.close_admission();
        // Stop must be the last command queued, otherwise a frame that already
        // reported a successful enqueue could land behind it and be discarded.
        while self.shared.enqueues_in_flight() > 0 && started.elapsed() < deadline {
            thread::sleep(ENQUEUE_SETTLE_POLL);
        }
        let remaining = deadline.saturating_sub(started.elapsed());
        let _ = self.sender.send_timeout(Command::Stop, remaining);
        let remaining = deadline.saturating_sub(started.elapsed());
        let exit = self.done.recv_timeout(remaining);

        // A timed-out writer is still running, so it cannot be joined here.
        if !matches!(exit, Err(flume::RecvTimeoutError::Timeout))
            && let Some(worker) = self.worker.take()
        {
            let _ = worker.join();
        }

        if exit == Ok(WriterExit::Drained) {
            return ShutdownOutcome::default();
        }

        let outcome = self.pending_outcome();
        let _ = self
            .shared
            .frames_dropped_shutdown
            .fetch_add(outcome.frames_pending, Ordering::Relaxed);
        outcome
    }

    /// Reports the frames that were accepted but are not on the stream yet.
    fn pending_outcome(&self) -> ShutdownOutcome {
        ShutdownOutcome {
            drained: false,
            // The writer latches this before it exits, including on panic, so it
            // separates an I/O failure from a deadline that simply expired.
            writer_failed: self.shared.unavailable.load(Ordering::Acquire),
            frames_pending: self.shared.frames_pending(),
        }
    }
}

/// Drains the queue and writes each frame contiguously.
fn run_writer(
    id: StreamId,
    receiver: flume::Receiver<Command>,
    mut sink: Box<dyn OutputSink>,
    shared: &StreamShared,
    flush_on_idle: bool,
) -> WriterExit {
    let mut pending_bytes = 0usize;
    let mut pending_frames = 0usize;

    'writer: while let Ok(first) = receiver.recv() {
        let mut next = Some(first);
        while let Some(command) = next.take() {
            match command {
                Command::Frame(frame) => {
                    if !write_one(id, sink.as_mut(), &frame, shared) {
                        return WriterExit::Failed;
                    }
                    pending_bytes = pending_bytes.saturating_add(frame.len());
                    pending_frames += 1;
                    // A saturated queue never goes idle, so flush on volume too.
                    if pending_bytes >= FLUSH_BYTES_THRESHOLD
                        || pending_frames >= FLUSH_FRAMES_THRESHOLD
                    {
                        if !flush_sink(id, sink.as_mut(), shared) {
                            return WriterExit::Failed;
                        }
                        pending_bytes = 0;
                        pending_frames = 0;
                    }
                }
                Command::Barrier(ack) => {
                    if !flush_sink(id, sink.as_mut(), shared) {
                        return WriterExit::Failed;
                    }
                    pending_bytes = 0;
                    pending_frames = 0;
                    let _ = ack.send(());
                }
                Command::Stop => break 'writer,
            }
            next = receiver.try_recv().ok();
        }
        if flush_on_idle && pending_frames > 0 {
            if !flush_sink(id, sink.as_mut(), shared) {
                return WriterExit::Failed;
            }
            pending_bytes = 0;
            pending_frames = 0;
        }
    }

    if flush_sink(id, sink.as_mut(), shared) {
        WriterExit::Drained
    } else {
        WriterExit::Failed
    }
}

/// Writes one frame, returning false when the writer must stop.
fn write_one(
    id: StreamId,
    sink: &mut dyn OutputSink,
    frame: &Frame,
    shared: &StreamShared,
) -> bool {
    match sink.write_frame(frame.as_bytes()) {
        Ok(()) => {
            let _ = shared.frames_written.fetch_add(1, Ordering::Relaxed);
            let _ = shared
                .bytes_written
                .fetch_add(frame.len() as u64, Ordering::Relaxed);
            true
        }
        Err(error) => {
            report_failure(id, shared, &error);
            false
        }
    }
}

/// Flushes the sink, returning false when the writer must stop.
///
/// A failed flush can lose bytes the writer already accepted, so it ends the
/// writer exactly like a failed write instead of being silently discarded.
fn flush_sink(id: StreamId, sink: &mut dyn OutputSink, shared: &StreamShared) -> bool {
    match sink.flush() {
        Ok(()) => true,
        Err(error) => {
            report_failure(id, shared, &error);
            false
        }
    }
}

/// Latches the stream as unavailable and reports the failure off the failed stream.
fn report_failure(id: StreamId, shared: &StreamShared, error: &io::Error) {
    let _ = shared.write_errors.fetch_add(1, Ordering::Relaxed);
    shared.unavailable.store(true, Ordering::Release);
    match id {
        // The self-tracing layer routes errors to stderr, so this cannot
        // recurse into the stream that just failed.
        StreamId::Stdout => otel_error!(
            "output_service.write_failed",
            stream = id.as_str(),
            error = ?error,
            message = "Console writer stopped after a write or flush error"
        ),
        // Diagnostics normally go to stderr, which is the stream that just died.
        StreamId::Stderr => report_dead_stderr_on_stdout(error),
    }
}

/// Last-resort notice that the stderr writer stopped, emitted on stdout.
///
/// Skipped when stdout carries records, because a prose line there would break
/// the guarantee this service exists to provide. The failure stays visible in
/// `write_errors` and in [`ShutdownOutcome::writer_failed`].
fn report_dead_stderr_on_stdout(error: &io::Error) {
    if OutputService::structured_stdout() {
        return;
    }
    let stdout = OutputService::stdout();
    // Without a running service the failed stream is a caller-owned one, so the
    // process stdout is not ours to write to.
    if stdout.is_direct() {
        return;
    }
    let _ = stdout.try_submit(Frame::line(&format!(
        "otap: stderr console writer stopped after a write or flush error: {error}"
    )));
}

/// The process-wide streams held by [`SERVICE`].
struct GlobalStreams {
    stdout: OutputStream,
    stderr: OutputStream,
}

/// Process-wide console output facade.
#[derive(Debug, Clone, Copy)]
pub struct OutputService;

impl OutputService {
    /// Starts the process-wide writer threads.
    ///
    /// Initialization is guarded so the first successful call wins. Returns
    /// `false` when the service was already running, leaving it untouched.
    ///
    /// # Errors
    ///
    /// Returns the [`io::Error`] produced when a writer thread cannot be spawned.
    pub fn init(config: OutputServiceConfig) -> io::Result<bool> {
        if SERVICE.get().is_some() {
            return Ok(false);
        }
        let stdout = OutputStream::start(
            StreamId::Stdout,
            config.stdout_queue_capacity,
            config.flush_on_idle,
            StreamId::Stdout.std_sink(),
        )?;
        let stderr = OutputStream::start(
            StreamId::Stderr,
            config.stderr_queue_capacity,
            config.flush_on_idle,
            StreamId::Stderr.std_sink(),
        )?;
        // A losing racer drops its streams here, which stops its writer threads.
        Ok(SERVICE.set(GlobalStreams { stdout, stderr }).is_ok())
    }

    /// Returns a producer handle for the process standard output.
    #[must_use]
    pub fn stdout() -> StreamHandle {
        SERVICE.get().map_or_else(
            || StreamHandle::direct(StreamId::Stdout),
            |service| service.stdout.handle(),
        )
    }

    /// Returns a producer handle for the process standard error.
    #[must_use]
    pub fn stderr() -> StreamHandle {
        SERVICE.get().map_or_else(
            || StreamHandle::direct(StreamId::Stderr),
            |service| service.stderr.handle(),
        )
    }

    /// Returns the handle human-readable diagnostics must use.
    ///
    /// While stdout carries machine-readable records, prose moves to stderr so a
    /// structured stream never gains a line that does not parse.
    #[must_use]
    pub fn diagnostics() -> StreamHandle {
        if Self::structured_stdout() {
            Self::stderr()
        } else {
            Self::stdout()
        }
    }

    /// Writes and flushes everything accepted so far on both streams.
    ///
    /// The writer threads keep running afterwards, so a process that hosts more
    /// than one engine run in sequence or in parallel keeps its console output.
    /// The deadline bounds the total wait across both streams.
    pub fn drain(deadline: Duration) -> ShutdownOutcome {
        let Some(service) = SERVICE.get() else {
            return ShutdownOutcome::default();
        };
        let started = Instant::now();
        let mut outcome = service.stdout.drain(deadline);
        let remaining = deadline.saturating_sub(started.elapsed());
        outcome.merge(service.stderr.drain(remaining));
        outcome
    }

    /// Returns a snapshot of both streams' counters.
    #[must_use]
    pub fn stats() -> ServiceStats {
        SERVICE
            .get()
            .map_or_else(ServiceStats::default, |service| ServiceStats {
                stdout: service.stdout.stats(),
                stderr: service.stderr.stats(),
            })
    }

    /// Records that stdout carries machine-readable records.
    ///
    /// This latches for the life of the process: once a stream has carried
    /// records, keeping prose off it stays correct even when a later engine run
    /// in the same process emits only human-readable output.
    pub fn mark_structured_stdout() {
        STRUCTURED_STDOUT.store(true, Ordering::Release);
    }

    /// Returns whether stdout currently carries machine-readable records.
    #[must_use]
    pub fn structured_stdout() -> bool {
        STRUCTURED_STDOUT.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicU32;

    /// Chunk size used to simulate an operating system that accepts partial writes.
    const PARTIAL_WRITE_CHUNK: usize = 64 * 1024;

    /// Recording sink that mimics a real stream: writes land in chunks, and the
    /// thread yields between chunks so any interleaving would become visible.
    #[derive(Clone)]
    struct TestSink {
        buffer: Arc<Mutex<Vec<u8>>>,
        flushes: Arc<AtomicU32>,
        delay: Option<Duration>,
        fail_after: Option<Arc<AtomicU32>>,
        flush_fails: bool,
        stalled: Option<Arc<AtomicBool>>,
    }

    impl TestSink {
        fn new() -> Self {
            Self {
                buffer: Arc::new(Mutex::new(Vec::new())),
                flushes: Arc::new(AtomicU32::new(0)),
                delay: None,
                fail_after: None,
                flush_fails: false,
                stalled: None,
            }
        }

        fn with_delay(mut self, delay: Duration) -> Self {
            self.delay = Some(delay);
            self
        }

        fn failing_after(mut self, writes: u32) -> Self {
            self.fail_after = Some(Arc::new(AtomicU32::new(writes)));
            self
        }

        fn failing_flush(mut self) -> Self {
            self.flush_fails = true;
            self
        }

        fn stalling(mut self, stalled: Arc<AtomicBool>) -> Self {
            self.stalled = Some(stalled);
            self
        }

        fn contents(&self) -> Vec<u8> {
            self.buffer
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
        }

        fn boxed(&self) -> Box<dyn OutputSink> {
            Box::new(self.clone())
        }
    }

    impl OutputSink for TestSink {
        fn write_frame(&mut self, frame: &[u8]) -> io::Result<()> {
            if let Some(stalled) = self.stalled.as_ref() {
                while stalled.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(5));
                }
            }
            if let Some(remaining) = self.fail_after.as_ref()
                && remaining
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                        value.checked_sub(1)
                    })
                    .is_err()
            {
                return Err(io::Error::other("simulated console failure"));
            }
            if let Some(delay) = self.delay {
                thread::sleep(delay);
            }
            let mut buffer = self
                .buffer
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            for chunk in frame.chunks(PARTIAL_WRITE_CHUNK) {
                buffer.extend_from_slice(chunk);
                thread::yield_now();
            }
            Ok(())
        }

        fn flush(&mut self) -> io::Result<()> {
            let _ = self.flushes.fetch_add(1, Ordering::Relaxed);
            if self.flush_fails {
                return Err(io::Error::other("simulated console flush failure"));
            }
            Ok(())
        }
    }

    fn start(sink: &TestSink, capacity: usize) -> OutputStream {
        OutputStream::start(StreamId::Stdout, capacity, true, sink.boxed())
            .expect("writer thread spawns")
    }

    /// Scenario: many threads concurrently submit distinguishable frames to one stream.
    /// Guarantees: every frame is written whole, so no frame's bytes are ever
    /// interleaved with another frame's bytes.
    #[test]
    fn concurrent_producers_never_interleave_frames() {
        const PRODUCERS: usize = 8;
        const FRAMES_PER_PRODUCER: usize = 64;

        let sink = TestSink::new();
        let mut stream = start(&sink, 16);
        let handle = stream.handle();

        let workers: Vec<_> = (0..PRODUCERS)
            .map(|producer| {
                let handle = handle.clone();
                thread::spawn(move || {
                    for frame in 0..FRAMES_PER_PRODUCER {
                        let line = format!("producer-{producer}-frame-{frame}");
                        // A repeated marker makes any split inside the frame visible.
                        let payload = format!("{}{}{}\n", line, "x".repeat(4096), line);
                        while handle
                            .try_submit(Frame::new(payload.clone().into_bytes()))
                            .is_err()
                        {
                            thread::yield_now();
                        }
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().expect("producer thread finishes");
        }

        let outcome = stream.shutdown(Duration::from_secs(10));
        assert!(outcome.drained);

        let contents = String::from_utf8(sink.contents()).expect("utf8 output");
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), PRODUCERS * FRAMES_PER_PRODUCER);
        for line in lines {
            let marker = line.split('x').next().expect("frame prefix");
            assert!(line.starts_with(marker));
            assert!(line.ends_with(marker));
        }
    }

    /// Scenario: a single record larger than Tokio's 2 MiB blocking-write chunk is submitted.
    /// Guarantees: the record is emitted contiguously despite multiple underlying partial writes.
    #[tokio::test]
    async fn oversized_record_is_written_contiguously() {
        let sink = TestSink::new();
        let mut stream = start(&sink, 4);
        let handle = stream.handle();

        let body = "a".repeat(3 * 1024 * 1024);
        let payload = format!("{{\"body\":\"{body}\"}}\n");
        handle
            .submit(Frame::new_record_json(payload.clone().into_bytes()))
            .await
            .expect("oversized frame is accepted");
        let outcome = stream.shutdown(Duration::from_secs(10));

        assert!(outcome.drained);
        assert_eq!(sink.contents(), payload.as_bytes());
    }

    /// Scenario: several records are submitted around the 2 MiB blocking-write boundary.
    /// Guarantees: each line parses independently, so a split always lands between
    /// frames and never inside a JSON record.
    #[tokio::test]
    async fn records_straddling_the_write_boundary_stay_intact() {
        const BOUNDARY: usize = 2 * 1024 * 1024;

        let sink = TestSink::new();
        let mut stream = start(&sink, 8);
        let handle = stream.handle();

        for offset in [BOUNDARY - 1, BOUNDARY, BOUNDARY + 1] {
            let body = "b".repeat(offset);
            let payload = format!("{{\"v\":\"{body}\"}}\n");
            handle
                .submit(Frame::new_record_json(payload.into_bytes()))
                .await
                .expect("boundary frame is accepted");
        }
        let outcome = stream.shutdown(Duration::from_secs(10));

        assert!(outcome.drained);
        let contents = String::from_utf8(sink.contents()).expect("utf8 output");
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 3);
        for (line, offset) in lines
            .iter()
            .zip([BOUNDARY - 1, BOUNDARY, BOUNDARY + 1].into_iter())
        {
            assert!(line.starts_with("{\"v\":\""));
            assert!(line.ends_with("\"}"));
            assert_eq!(line.matches('b').count(), offset);
        }
    }

    /// Scenario: one frame carries many newline-terminated records.
    /// Guarantees: all records arrive in submission order and none is split.
    #[tokio::test]
    async fn multi_record_frame_keeps_record_order() {
        let sink = TestSink::new();
        let mut stream = start(&sink, 4);
        let handle = stream.handle();

        let mut payload = String::new();
        for index in 0..1000 {
            payload.push_str(&format!("{{\"index\":{index}}}\n"));
        }
        handle
            .submit(Frame::new_record_json(payload.clone().into_bytes()))
            .await
            .expect("multi-record frame is accepted");
        let outcome = stream.shutdown(Duration::from_secs(10));

        assert!(outcome.drained);
        let contents = String::from_utf8(sink.contents()).expect("utf8 output");
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 1000);
        for (index, line) in lines.iter().enumerate() {
            assert_eq!(*line, format!("{{\"index\":{index}}}"));
        }
    }

    /// Scenario: a slow sink cannot keep up with a producer submitting more frames than the queue holds.
    /// Guarantees: the producer waits for capacity and still makes progress, and the
    /// number of accepted-but-unwritten frames stays bounded by the queue capacity
    /// rather than growing with the number of frames submitted.
    #[tokio::test]
    async fn slow_sink_applies_bounded_backpressure() {
        const CAPACITY: usize = 4;
        const FRAMES: usize = 40;

        let sink = TestSink::new().with_delay(Duration::from_millis(1));
        let mut stream = OutputStream::start(
            StreamId::Stdout,
            CAPACITY,
            true,
            Box::new(sink.clone()) as Box<dyn OutputSink>,
        )
        .expect("writer thread spawns");
        let handle = stream.handle();

        for index in 0..FRAMES {
            handle
                .submit(Frame::line(&format!("line-{index}")))
                .await
                .expect("frame is accepted after waiting for capacity");
        }
        let outcome = stream.shutdown(Duration::from_secs(10));

        assert!(outcome.drained);
        let stats = stream.stats();
        assert_eq!(stats.frames_submitted, FRAMES as u64);
        assert_eq!(stats.frames_written, FRAMES as u64);
        assert_eq!(stats.frames_enqueue_failed, 0);
        // Two slots sit outside the queue: one frame the writer has taken but not
        // yet accounted for, and one the producer reserved but has not sent yet.
        assert!(stats.queue_depth_high_water <= CAPACITY as u64 + 2);
    }

    /// Scenario: the sink starts returning io::Error partway through a run.
    /// Guarantees: the writer records the error, marks itself unavailable, closes
    /// the queue, later submits fail fast instead of blocking, and shutdown reports
    /// a failed drain because accepted frames were abandoned.
    #[tokio::test]
    async fn write_error_makes_the_stream_unavailable() {
        // Uses the stderr stream so the failure report cannot reach the stream
        // under test: a dead stderr writer is reported on stdout, and a dead
        // stdout writer is reported through otel_error! to stderr.
        let sink = TestSink::new().failing_after(1);
        let mut stream =
            OutputStream::start(StreamId::Stderr, 4, true, Box::new(sink.clone())).expect("spawn");
        let handle = stream.handle();

        handle
            .submit(Frame::line("first"))
            .await
            .expect("first frame is accepted");
        // The second write fails and stops the writer; retry until the failure is visible.
        let mut failed = false;
        for _ in 0..1000 {
            if handle.submit(Frame::line("next")).await.is_err() {
                failed = true;
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert!(failed, "submits must start failing after a write error");

        let outcome = stream.shutdown(Duration::from_secs(5));
        assert!(
            !outcome.drained,
            "a write failure is not a successful drain"
        );
        let stats = stream.stats();
        assert_eq!(stats.write_errors, 1);
        assert_eq!(sink.contents(), b"first\n");
    }

    /// Scenario: the sink accepts every write but fails to flush.
    /// Guarantees: shutdown reports a failed drain, because bytes the writer already
    /// accepted may never have reached the operating system.
    #[tokio::test]
    async fn flush_error_reports_a_failed_drain() {
        let sink = TestSink::new().failing_flush();
        let mut stream = OutputStream::start(StreamId::Stderr, 4, false, Box::new(sink.clone()))
            .expect("writer thread spawns");
        let handle = stream.handle();

        handle
            .submit(Frame::line("first"))
            .await
            .expect("frame is accepted");
        let outcome = stream.shutdown(Duration::from_secs(5));

        assert!(
            !outcome.drained,
            "a flush failure is not a successful drain"
        );
        assert!(stream.stats().write_errors > 0);
    }

    /// Scenario: producers keep submitting while shutdown closes the stream.
    /// Guarantees: every submit that reported success is written before the writer
    /// stops, so an accepted frame is never discarded behind the stop marker.
    #[test]
    fn shutdown_never_drops_an_accepted_frame() {
        // Enough concurrent producers that at least one is reliably mid-enqueue
        // when shutdown closes the stream.
        const PRODUCERS: usize = 16;

        let sink = TestSink::new();
        let mut stream = start(&sink, 4);
        let handle = stream.handle();
        let keep_going = Arc::new(AtomicBool::new(true));

        let workers: Vec<_> = (0..PRODUCERS)
            .map(|_| {
                let handle = handle.clone();
                let keep_going = Arc::clone(&keep_going);
                thread::spawn(move || {
                    while keep_going.load(Ordering::Acquire) {
                        if handle.try_submit(Frame::line("frame")) == Err(SubmitError::QueueClosed)
                        {
                            return;
                        }
                    }
                })
            })
            .collect();

        thread::sleep(Duration::from_millis(20));
        let outcome = stream.shutdown(Duration::from_secs(10));
        keep_going.store(false, Ordering::Release);
        for worker in workers {
            worker.join().expect("producer thread finishes");
        }

        assert!(outcome.drained);
        assert_eq!(outcome.frames_pending, 0);
        let stats = stream.stats();
        assert!(
            stats.frames_submitted > 0,
            "the race window must be exercised"
        );
        assert_eq!(
            stats.frames_written, stats.frames_submitted,
            "every accepted frame must reach the sink"
        );
        assert_eq!(stats.frames_dropped_shutdown, 0);
        let contents = String::from_utf8(sink.contents()).expect("utf8 output");
        assert_eq!(contents.lines().count() as u64, stats.frames_submitted);
    }

    /// Scenario: a pending `submit` is cancelled while it waits for queue capacity.
    /// Guarantees: the cancelled attempt releases its admission slot and is not counted
    /// as accepted, so a later shutdown settles instead of waiting out its deadline.
    #[tokio::test]
    async fn cancelled_submit_releases_its_admission_slot() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let mut stream = start(&sink, 1);
        let handle = stream.handle();

        // One frame reaches the stalled writer, the next fills the single queue slot.
        handle
            .submit(Frame::line("in-writer"))
            .await
            .expect("frame is accepted");
        handle
            .submit(Frame::line("queued"))
            .await
            .expect("frame is accepted");

        // This one cannot be enqueued, so the timeout drops it mid-send.
        let cancelled = tokio::time::timeout(
            Duration::from_millis(100),
            handle.submit(Frame::line("cancelled")),
        )
        .await;
        assert!(
            cancelled.is_err(),
            "the submit must still be pending when it is cancelled"
        );

        stalled.store(false, Ordering::Release);
        let started = Instant::now();
        let outcome = stream.shutdown(Duration::from_secs(5));
        let elapsed = started.elapsed();

        assert!(outcome.drained);
        assert!(
            elapsed < Duration::from_secs(2),
            "shutdown must not wait on a cancelled submit"
        );
        let stats = stream.stats();
        assert_eq!(
            stats.frames_submitted, 2,
            "a cancelled submit must not count as accepted"
        );
        assert_eq!(stats.frames_written, stats.frames_submitted);
    }

    /// Scenario: a drain completes and the same stream is used again afterwards.
    /// Guarantees: the drain confirms everything queued before it was written and
    /// flushed, and the writer keeps accepting frames, so one engine run in a
    /// process cannot silence the console for a later one.
    #[tokio::test]
    async fn drain_flushes_without_stopping_the_writer() {
        const BATCH: usize = 8;

        let sink = TestSink::new().with_delay(Duration::from_millis(1));
        let stream = start(&sink, 4);
        let handle = stream.handle();

        for index in 0..BATCH {
            handle
                .submit(Frame::line(&format!("first-{index}")))
                .await
                .expect("frame is accepted");
        }
        assert!(stream.drain(Duration::from_secs(10)).drained);
        let contents = String::from_utf8(sink.contents()).expect("utf8 output");
        assert_eq!(contents.lines().count(), BATCH);
        assert!(sink.flushes.load(Ordering::Relaxed) > 0);

        for index in 0..BATCH {
            handle
                .submit(Frame::line(&format!("second-{index}")))
                .await
                .expect("the stream still accepts frames after a drain");
        }
        assert!(stream.drain(Duration::from_secs(10)).drained);
        let contents = String::from_utf8(sink.contents()).expect("utf8 output");
        assert_eq!(contents.lines().count(), BATCH * 2);
    }

    /// Scenario: the writer thread panics while a producer is still submitting.
    /// Guarantees: submits fail fast with a closed queue instead of blocking forever
    /// on a queue nobody drains.
    #[tokio::test]
    async fn writer_panic_does_not_block_producers() {
        struct PanickingSink;

        impl OutputSink for PanickingSink {
            fn write_frame(&mut self, _frame: &[u8]) -> io::Result<()> {
                panic!("simulated writer panic");
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let stream = OutputStream::start(StreamId::Stderr, 1, true, Box::new(PanickingSink))
            .expect("writer thread spawns");
        let handle = stream.handle();

        let mut error = None;
        for _ in 0..1000 {
            match handle.submit(Frame::line("boom")).await {
                Ok(()) => thread::sleep(Duration::from_millis(1)),
                Err(err) => {
                    error = Some(err);
                    break;
                }
            }
        }
        assert!(
            matches!(
                error,
                Some(SubmitError::QueueClosed | SubmitError::WriterUnavailable)
            ),
            "expected a fail-fast submit error, got {error:?}"
        );
    }

    /// Scenario: shutdown is requested while frames are still queued behind a slow sink.
    /// Guarantees: every accepted frame is written and the sink is flushed before
    /// shutdown reports a completed drain.
    #[tokio::test]
    async fn shutdown_drains_and_flushes_queued_frames() {
        const FRAMES: usize = 16;

        let sink = TestSink::new().with_delay(Duration::from_millis(2));
        let mut stream = start(&sink, FRAMES);
        let handle = stream.handle();

        for index in 0..FRAMES {
            handle
                .submit(Frame::line(&format!("queued-{index}")))
                .await
                .expect("frame is accepted");
        }
        let outcome = stream.shutdown(Duration::from_secs(10));

        assert!(outcome.drained);
        assert_eq!(outcome.frames_pending, 0);
        assert_eq!(stream.stats().frames_written, FRAMES as u64);
        assert!(sink.flushes.load(Ordering::Relaxed) > 0);
        let contents = String::from_utf8(sink.contents()).expect("utf8 output");
        assert_eq!(contents.lines().count(), FRAMES);
    }

    /// Scenario: shutdown runs against a sink that never completes a write.
    /// Guarantees: shutdown returns within its deadline and reports the frames it
    /// could not drain instead of blocking process exit.
    #[tokio::test]
    async fn shutdown_gives_up_on_a_stalled_sink() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let mut stream = start(&sink, 8);
        let handle = stream.handle();

        for index in 0..4 {
            handle
                .submit(Frame::line(&format!("stuck-{index}")))
                .await
                .expect("frame is accepted");
        }

        let started = Instant::now();
        let outcome = stream.shutdown(Duration::from_millis(200));
        let elapsed = started.elapsed();

        assert!(!outcome.drained);
        assert!(outcome.frames_pending > 0);
        assert!(elapsed < Duration::from_secs(5), "shutdown must be bounded");
        assert_eq!(
            stream.stats().frames_dropped_shutdown,
            outcome.frames_pending
        );
        stalled.store(false, Ordering::Release);
    }

    /// Scenario: a handle is requested while the process-wide service was never initialized.
    /// Guarantees: the handle falls back to direct writes so existing behavior is
    /// preserved rather than lost. No test in this module calls
    /// `OutputService::init`, so the global service stays uninitialized.
    #[tokio::test]
    async fn uninitialized_service_falls_back_to_direct_writes() {
        let handle = OutputService::stdout();
        assert!(handle.is_direct());
        assert_eq!(handle.stream_id(), StreamId::Stdout);
        assert!(OutputService::stderr().is_direct());
        // An empty frame exercises the direct path without emitting output.
        handle
            .submit(Frame::new(Vec::new()))
            .await
            .expect("direct write succeeds");
        assert_eq!(handle.stats(), OutputStats::default());
    }

    /// Scenario: a synchronous caller uses try_submit while the queue is full.
    /// Guarantees: the call returns WouldBlock immediately and the dropped frame is
    /// counted, so a tracing callback can never stall an engine core thread.
    #[test]
    fn try_submit_drops_instead_of_blocking_when_full() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = start(&sink, 1);
        let handle = stream.handle();

        let mut dropped = false;
        for _ in 0..100 {
            if handle.try_submit(Frame::line("diagnostic")) == Err(SubmitError::WouldBlock) {
                dropped = true;
                break;
            }
        }

        assert!(dropped, "a full queue must reject non-blocking submits");
        let stats = stream.stats();
        assert!(stats.diagnostics_dropped > 0);
        assert_eq!(stats.frames_enqueue_failed, stats.diagnostics_dropped);
        stalled.store(false, Ordering::Release);
    }

    /// Scenario: a drain runs after the writer already stopped on an I/O error.
    /// Guarantees: the outcome separates a failed writer from an expired deadline, so
    /// an operator is not told output merely ran out of time.
    #[tokio::test]
    async fn drain_reports_a_failed_writer_separately_from_a_timeout() {
        let failing = TestSink::new().failing_after(0);
        let stream = OutputStream::start(StreamId::Stderr, 4, true, Box::new(failing.clone()))
            .expect("writer thread spawns");
        let handle = stream.handle();
        // Drive one write so the sink fails and the writer latches unavailable.
        let _ = handle.submit(Frame::line("doomed")).await;
        for _ in 0..1000 {
            if stream.stats().write_errors > 0 {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }

        let failed = stream.drain(Duration::from_millis(200));
        assert!(!failed.drained);
        assert!(
            failed.writer_failed,
            "an I/O failure must be distinguishable"
        );

        // A stalled but healthy writer is a timeout, not a failure.
        let stalled = Arc::new(AtomicBool::new(true));
        let slow = TestSink::new().stalling(Arc::clone(&stalled));
        let stalled_stream = start(&slow, 4);
        // The writer blocks inside this frame's write, so the barrier queues behind it.
        stalled_stream
            .handle()
            .submit(Frame::line("stuck"))
            .await
            .expect("frame is accepted");
        let timed_out = stalled_stream.drain(Duration::from_millis(200));
        stalled.store(false, Ordering::Release);

        assert!(!timed_out.drained);
        assert!(!timed_out.writer_failed);
    }

    /// Scenario: engine prose is emitted while stdout carries machine-readable records.
    /// Guarantees: the diagnostics handle moves to stderr, so `record_json` stdout
    /// never gains a human-readable line that does not parse.
    #[test]
    fn diagnostics_move_to_stderr_while_stdout_is_structured() {
        // The latch is process-wide and monotonic, so this test owns the transition.
        assert_eq!(OutputService::diagnostics().stream_id(), StreamId::Stdout);

        OutputService::mark_structured_stdout();

        assert_eq!(OutputService::diagnostics().stream_id(), StreamId::Stderr);
    }

    /// Scenario: a frame is built from a plain message, as `EffectHandler::info` does.
    /// Guarantees: the frame carries the message plus exactly one trailing newline, so
    /// one call still produces exactly one console line.
    #[test]
    fn line_frame_carries_exactly_one_trailing_newline() {
        let frame = Frame::line("pipeline started");
        assert_eq!(frame.as_bytes(), b"pipeline started\n");
        assert_eq!(frame.as_bytes().iter().filter(|b| **b == b'\n').count(), 1);
        assert!(!frame.is_empty());
    }

    /// Scenario: a record_json frame is built from bytes that do not end with a newline.
    /// Guarantees: debug builds fail loudly rather than emitting a frame that could
    /// split a JSON record across two writes.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "must end with a newline")]
    fn record_json_frame_requires_a_trailing_newline() {
        let _ = Frame::new_record_json(b"{\"body\":\"no newline\"}".to_vec());
    }
}
