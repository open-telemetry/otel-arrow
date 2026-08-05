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
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
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
    /// Frames accepted into the queue.
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

/// Result of draining an output stream at shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownOutcome {
    /// Whether every accepted frame was written and flushed.
    pub drained: bool,
    /// Accepted frames still queued when the drain deadline expired.
    pub frames_dropped: u64,
}

impl Default for ShutdownOutcome {
    fn default() -> Self {
        Self {
            drained: true,
            frames_dropped: 0,
        }
    }
}

impl ShutdownOutcome {
    /// Folds another stream's outcome into this one.
    fn merge(&mut self, other: Self) {
        self.drained &= other.drained;
        self.frames_dropped = self.frames_dropped.saturating_add(other.frames_dropped);
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
    Stop,
}

/// State shared between the producers and the writer thread of one stream.
#[derive(Debug, Default)]
struct StreamShared {
    closing: AtomicBool,
    unavailable: AtomicBool,
    queue_depth: AtomicUsize,
    queue_depth_high_water: AtomicUsize,
    frames_submitted: AtomicU64,
    frames_enqueue_failed: AtomicU64,
    frames_written: AtomicU64,
    bytes_written: AtomicU64,
    write_errors: AtomicU64,
    diagnostics_dropped: AtomicU64,
    frames_dropped_shutdown: AtomicU64,
}

impl StreamShared {
    fn snapshot(&self) -> OutputStats {
        OutputStats {
            frames_submitted: self.frames_submitted.load(Ordering::Relaxed),
            frames_enqueue_failed: self.frames_enqueue_failed.load(Ordering::Relaxed),
            frames_written: self.frames_written.load(Ordering::Relaxed),
            bytes_written: self.bytes_written.load(Ordering::Relaxed),
            write_errors: self.write_errors.load(Ordering::Relaxed),
            diagnostics_dropped: self.diagnostics_dropped.load(Ordering::Relaxed),
            frames_dropped_shutdown: self.frames_dropped_shutdown.load(Ordering::Relaxed),
            queue_depth_high_water: self.queue_depth_high_water.load(Ordering::Relaxed) as u64,
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
    /// Reserves a queue slot, failing fast when the stream no longer accepts frames.
    fn reserve(&self) -> Result<(), SubmitError> {
        if self.shared.closing.load(Ordering::Acquire) {
            return Err(SubmitError::QueueClosed);
        }
        if self.shared.unavailable.load(Ordering::Acquire) {
            return Err(SubmitError::WriterUnavailable);
        }
        let depth = self.shared.queue_depth.fetch_add(1, Ordering::AcqRel) + 1;
        let _ = self
            .shared
            .queue_depth_high_water
            .fetch_max(depth, Ordering::Relaxed);
        Ok(())
    }

    fn release(&self) {
        let _ = self.shared.queue_depth.fetch_sub(1, Ordering::AcqRel);
    }

    fn accepted(&self) {
        let _ = self.shared.frames_submitted.fetch_add(1, Ordering::Relaxed);
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
        queued.reserve().map_err(|error| queued.rejected(error))?;
        match queued.sender.send_async(Command::Frame(frame)).await {
            Ok(()) => {
                queued.accepted();
                Ok(())
            }
            Err(_) => {
                queued.release();
                Err(queued.rejected(SubmitError::QueueClosed))
            }
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
        queued.reserve().map_err(|error| queued.rejected(error))?;
        match queued.sender.try_send(Command::Frame(frame)) {
            Ok(()) => {
                queued.accepted();
                Ok(())
            }
            Err(flume::TrySendError::Full(_)) => {
                queued.release();
                let _ = queued
                    .shared
                    .diagnostics_dropped
                    .fetch_add(1, Ordering::Relaxed);
                Err(queued.rejected(SubmitError::WouldBlock))
            }
            Err(flume::TrySendError::Disconnected(_)) => {
                queued.release();
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
    done: flume::Receiver<()>,
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
        let (done_tx, done) = flume::bounded::<()>(1);
        let shared = Arc::new(StreamShared::default());
        let worker_shared = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name(id.thread_name().to_owned())
            .spawn(move || {
                // Dropped last, so a completed drain is only signalled after the
                // receiver is gone and the unavailable flag is set.
                let _done = done_tx;
                let _guard = TeardownGuard {
                    shared: Arc::clone(&worker_shared),
                };
                run_writer(id, receiver, sink, &worker_shared, flush_on_idle);
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

    /// Stops accepting frames, drains what was accepted, and flushes.
    ///
    /// Returns once the writer finishes or the deadline expires, whichever
    /// comes first, so a stalled console pipe can never block process exit.
    pub fn shutdown(&mut self, deadline: Duration) -> ShutdownOutcome {
        self.shared.closing.store(true, Ordering::Release);
        let started = Instant::now();
        // FIFO means Stop is only observed after every frame already queued.
        let _ = self.sender.send_timeout(Command::Stop, deadline);
        let remaining = deadline.saturating_sub(started.elapsed());
        // Nothing is ever sent on `done`; the writer exiting drops its sender.
        let finished = matches!(
            self.done.recv_timeout(remaining),
            Err(flume::RecvTimeoutError::Disconnected)
        );

        if finished {
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
            return ShutdownOutcome {
                drained: true,
                frames_dropped: 0,
            };
        }

        let undrained = self.shared.queue_depth.load(Ordering::Acquire) as u64;
        let _ = self
            .shared
            .frames_dropped_shutdown
            .fetch_add(undrained, Ordering::Relaxed);
        ShutdownOutcome {
            drained: false,
            frames_dropped: undrained,
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
) {
    let mut pending_bytes = 0usize;
    let mut pending_frames = 0usize;

    'writer: while let Ok(first) = receiver.recv() {
        let mut next = Some(first);
        while let Some(command) = next.take() {
            let Command::Frame(frame) = command else {
                break 'writer;
            };
            let _ = shared.queue_depth.fetch_sub(1, Ordering::AcqRel);
            if !write_one(id, sink.as_mut(), &frame, shared) {
                break 'writer;
            }
            pending_bytes = pending_bytes.saturating_add(frame.len());
            pending_frames += 1;
            // A saturated queue never goes idle, so flush on volume too.
            if pending_bytes >= FLUSH_BYTES_THRESHOLD || pending_frames >= FLUSH_FRAMES_THRESHOLD {
                let _ = sink.flush();
                pending_bytes = 0;
                pending_frames = 0;
            }
            next = receiver.try_recv().ok();
        }
        if flush_on_idle && pending_frames > 0 {
            let _ = sink.flush();
            pending_bytes = 0;
            pending_frames = 0;
        }
    }

    let _ = sink.flush();
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
            let _ = shared.write_errors.fetch_add(1, Ordering::Relaxed);
            shared.unavailable.store(true, Ordering::Release);
            if id == StreamId::Stdout {
                // The self-tracing layer routes errors to stderr, so this cannot
                // recurse into the stream that just failed.
                otel_error!(
                    "output_service.write_failed",
                    stream = id.as_str(),
                    error = ?error,
                    message = "Console writer stopped after a write error"
                );
            }
            false
        }
    }
}

/// The process-wide streams held by [`SERVICE`].
struct GlobalStreams {
    stdout_handle: StreamHandle,
    stderr_handle: StreamHandle,
    stdout: Mutex<Option<OutputStream>>,
    stderr: Mutex<Option<OutputStream>>,
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
        let streams = GlobalStreams {
            stdout_handle: stdout.handle(),
            stderr_handle: stderr.handle(),
            stdout: Mutex::new(Some(stdout)),
            stderr: Mutex::new(Some(stderr)),
        };
        // A losing racer drops its streams here, which stops its writer threads.
        Ok(SERVICE.set(streams).is_ok())
    }

    /// Returns a producer handle for the process standard output.
    #[must_use]
    pub fn stdout() -> StreamHandle {
        SERVICE.get().map_or_else(
            || StreamHandle::direct(StreamId::Stdout),
            |service| service.stdout_handle.clone(),
        )
    }

    /// Returns a producer handle for the process standard error.
    #[must_use]
    pub fn stderr() -> StreamHandle {
        SERVICE.get().map_or_else(
            || StreamHandle::direct(StreamId::Stderr),
            |service| service.stderr_handle.clone(),
        )
    }

    /// Stops accepting frames, drains both streams, and flushes.
    ///
    /// The deadline bounds the total wait across both streams.
    pub fn shutdown(deadline: Duration) -> ShutdownOutcome {
        let Some(service) = SERVICE.get() else {
            return ShutdownOutcome::default();
        };
        let started = Instant::now();
        // stdout drains first so late diagnostics still reach stderr.
        let mut outcome = shutdown_stream(&service.stdout, deadline);
        let remaining = deadline.saturating_sub(started.elapsed());
        outcome.merge(shutdown_stream(&service.stderr, remaining));
        outcome
    }

    /// Returns a snapshot of both streams' counters.
    #[must_use]
    pub fn stats() -> ServiceStats {
        ServiceStats {
            stdout: Self::stdout().stats(),
            stderr: Self::stderr().stats(),
        }
    }

    /// Records whether stdout currently carries machine-readable records.
    ///
    /// While set, human-readable diagnostics are routed to stderr so they
    /// cannot corrupt a structured stdout stream.
    pub fn set_structured_stdout(enabled: bool) {
        STRUCTURED_STDOUT.store(enabled, Ordering::Release);
    }

    /// Returns whether stdout currently carries machine-readable records.
    #[must_use]
    pub fn structured_stdout() -> bool {
        STRUCTURED_STDOUT.load(Ordering::Acquire)
    }
}

/// Shuts one global stream down and releases it.
fn shutdown_stream(slot: &Mutex<Option<OutputStream>>, deadline: Duration) -> ShutdownOutcome {
    let mut guard = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    match guard.as_mut() {
        Some(stream) => {
            let outcome = stream.shutdown(deadline);
            *guard = None;
            outcome
        }
        None => ShutdownOutcome::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        stalled: Option<Arc<AtomicBool>>,
    }

    impl TestSink {
        fn new() -> Self {
            Self {
                buffer: Arc::new(Mutex::new(Vec::new())),
                flushes: Arc::new(AtomicU32::new(0)),
                delay: None,
                fail_after: None,
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
    /// number of accepted-but-unwritten frames never exceeds the queue capacity.
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
        // One extra slot is in flight in the writer while the queue refills.
        assert!(stats.queue_depth_high_water <= CAPACITY as u64 + 1);
    }

    /// Scenario: the sink starts returning io::Error partway through a run.
    /// Guarantees: the writer records the error, marks itself unavailable, closes
    /// the queue, and later submits fail fast instead of blocking.
    #[tokio::test]
    async fn write_error_makes_the_stream_unavailable() {
        // Uses the stderr stream so the failure report cannot reach the stream
        // under test; stdout failures are reported through otel_error! to stderr.
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
        assert!(outcome.drained);
        let stats = stream.stats();
        assert_eq!(stats.write_errors, 1);
        assert_eq!(sink.contents(), b"first\n");
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
        assert_eq!(outcome.frames_dropped, 0);
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
        assert!(outcome.frames_dropped > 0);
        assert!(elapsed < Duration::from_secs(5), "shutdown must be bounded");
        assert_eq!(
            stream.stats().frames_dropped_shutdown,
            outcome.frames_dropped
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
