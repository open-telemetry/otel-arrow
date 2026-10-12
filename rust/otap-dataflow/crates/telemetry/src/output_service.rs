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
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};

/// Default bounded queue capacity, in frames, for the stdout writer.
pub const DEFAULT_STDOUT_QUEUE_CAPACITY: usize = 1024;

/// Default bounded queue capacity, in frames, for the stderr writer.
pub const DEFAULT_STDERR_QUEUE_CAPACITY: usize = 256;

/// Default bounded queue capacity, in bytes, for the stdout writer.
///
/// The frame count alone does not bound memory, because a frame owns its
/// payload and payloads are variable sized.
pub const DEFAULT_STDOUT_BYTE_CAPACITY: usize = 64 * 1024 * 1024;

/// Default bounded queue capacity, in bytes, for the stderr writer.
pub const DEFAULT_STDERR_BYTE_CAPACITY: usize = 16 * 1024 * 1024;

/// Default upper bound on how long shutdown waits for queued frames to drain.
pub const DEFAULT_SHUTDOWN_DRAIN_DEADLINE: Duration = Duration::from_secs(5);

/// Bytes written since the last flush that force a safety flush.
const FLUSH_BYTES_THRESHOLD: usize = 256 * 1024;

/// Frames written since the last flush that force a safety flush.
const FLUSH_FRAMES_THRESHOLD: usize = 256;

/// Poll interval while a drain waits for a free control slot.
const CONTROL_SLOT_POLL: Duration = Duration::from_micros(200);

/// Queue slots reserved for drain barriers, so a queued frame always finds room.
const CONTROL_SLOTS: usize = 2;

/// Pause before retrying a write that a nonblocking stream refused.
const WOULD_BLOCK_RETRY_DELAY: Duration = Duration::from_millis(1);

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
    /// The frame is larger than the stream's whole byte budget, so no amount of
    /// draining could ever admit it.
    #[error("console output frame is larger than the stream byte budget")]
    FrameTooLarge,
}

/// Destination for frames drained by a writer thread.
pub trait OutputSink: Send {
    /// Writes one complete frame.
    ///
    /// Any error stops the writer, so a sink retries a transient `WouldBlock`
    /// itself: only the sink knows how much of the frame already went out.
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
        write_all_retrying(&mut locked, frame)
    }

    fn flush(&mut self) -> io::Result<()> {
        flush_retrying(&mut io::stdout().lock())
    }
}

/// Sink writing complete frames to the process standard error.
#[derive(Debug, Default, Clone, Copy)]
pub struct StderrSink;

impl OutputSink for StderrSink {
    fn write_frame(&mut self, frame: &[u8]) -> io::Result<()> {
        let stderr = io::stderr();
        let mut locked = stderr.lock();
        write_all_retrying(&mut locked, frame)
    }

    fn flush(&mut self) -> io::Result<()> {
        flush_retrying(&mut io::stderr().lock())
    }
}

/// Writes the whole frame, resuming after `WouldBlock` at the byte it stopped at.
///
/// A nonblocking stream refuses writes while its reader catches up. `write_all`
/// would report that as a failure without saying how much of the frame went out,
/// so the frame could neither be abandoned safely nor retried without a duplicate.
fn write_all_retrying(writer: &mut impl Write, frame: &[u8]) -> io::Result<()> {
    let mut offset = 0;
    while offset < frame.len() {
        match writer.write(&frame[offset..]) {
            Ok(0) => return Err(io::Error::from(io::ErrorKind::WriteZero)),
            Ok(written) => offset += written,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(WOULD_BLOCK_RETRY_DELAY);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

/// Flushes, retrying while a nonblocking stream refuses the buffered bytes.
///
/// A buffered writer keeps whatever the stream refused, so retrying never repeats bytes.
fn flush_retrying(writer: &mut impl Write) -> io::Result<()> {
    loop {
        match writer.flush() {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(WOULD_BLOCK_RETRY_DELAY);
            }
            result => return result,
        }
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
    /// Failed sink writes. Any failure stops the writer; a transient `WouldBlock`
    /// is retried instead of counted.
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
    /// Whether the deadline elapsed while a writer could still be running.
    pub deadline_expired: bool,
    /// Accepted frames still unwritten when the operation returned.
    pub frames_pending: u64,
}

impl Default for ShutdownOutcome {
    fn default() -> Self {
        Self {
            drained: true,
            writer_failed: false,
            deadline_expired: false,
            frames_pending: 0,
        }
    }
}

impl ShutdownOutcome {
    /// Folds another stream's outcome into this one.
    fn merge(&mut self, other: Self) {
        self.drained &= other.drained;
        self.writer_failed |= other.writer_failed;
        self.deadline_expired |= other.deadline_expired;
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
    /// Bounded queue capacity, in bytes, for the stdout writer.
    pub stdout_byte_capacity: usize,
    /// Bounded queue capacity, in bytes, for the stderr writer.
    pub stderr_byte_capacity: usize,
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
            stdout_byte_capacity: DEFAULT_STDOUT_BYTE_CAPACITY,
            stderr_byte_capacity: DEFAULT_STDERR_BYTE_CAPACITY,
            flush_on_idle: true,
            shutdown_drain_deadline: DEFAULT_SHUTDOWN_DRAIN_DEADLINE,
        }
    }
}

/// Queue item.
///
/// A barrier holds one of the [`CONTROL_SLOTS`] until the writer takes it, so
/// barriers can never fill the room a frame's slot reserved.
enum Command {
    Frame(QueuedFrame),
    /// Flush everything queued ahead of this barrier, then acknowledge.
    Barrier(flume::Sender<()>, OwnedSemaphorePermit),
}

/// A queued frame and the reservations it holds.
///
/// The reservations are pure accounting: they never copy the payload. They
/// travel with the frame into the queue. The queue slot returns when the writer
/// takes the frame, and the bytes return once it has been written or abandoned,
/// rather than when the producer finished enqueuing it.
struct QueuedFrame {
    frame: Frame,
    bytes: OwnedSemaphorePermit,
    slot: OwnedSemaphorePermit,
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
    /// Set when the writer stopped on an I/O error or a panic, never on a clean exit.
    writer_failed: AtomicBool,
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
    /// Classifies a stream that no longer accepts frames.
    ///
    /// A failure is recorded before the queue closes, so a caller that saw the
    /// queue closed also sees why.
    fn closed_error(&self) -> SubmitError {
        if self.writer_failed.load(Ordering::Acquire) {
            SubmitError::WriterUnavailable
        } else {
            SubmitError::QueueClosed
        }
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
    sender: async_channel::Sender<Command>,
    shared: Arc<StreamShared>,
    /// One permit per byte currently queued but not yet written.
    bytes: Arc<Semaphore>,
    byte_capacity: usize,
    /// One permit per queue slot a frame may occupy, shared with the writer thread.
    frame_slots: Arc<Semaphore>,
}

impl QueuedHandle {
    /// Fails fast when the stream no longer accepts frames.
    fn ensure_open(&self) -> Result<(), SubmitError> {
        if self.shared.writer_failed.load(Ordering::Acquire) || self.sender.is_closed() {
            return Err(self.shared.closed_error());
        }
        Ok(())
    }

    /// Counts the frame and hands it to the writer with no await in between.
    ///
    /// A cancelled submit therefore either queued and counted its frame, or did
    /// neither. The frame's slot guarantees room, so only a closed queue refuses it.
    fn enqueue(&self, frame: QueuedFrame) -> Result<(), SubmitError> {
        // Counted before the send, so frames written never run ahead of frames submitted.
        let submitted = self.shared.frames_submitted.fetch_add(1, Ordering::AcqRel) + 1;
        let written = self.shared.frames_written.load(Ordering::Acquire);
        let _ = self
            .shared
            .queue_depth_high_water
            .fetch_max(submitted.saturating_sub(written), Ordering::Relaxed);
        if self.sender.try_send(Command::Frame(frame)).is_ok() {
            return Ok(());
        }
        let _ = self.shared.frames_submitted.fetch_sub(1, Ordering::AcqRel);
        Err(self.rejected(SubmitError::QueueClosed))
    }

    /// Converts a frame length into a byte reservation size.
    ///
    /// A frame that cannot fit even in an empty queue is rejected here instead
    /// of waiting for capacity that can never arrive.
    fn reservation(&self, len: usize) -> Result<u32, SubmitError> {
        if len > self.byte_capacity {
            return Err(SubmitError::FrameTooLarge);
        }
        u32::try_from(len).map_err(|_| SubmitError::FrameTooLarge)
    }

    fn rejected(&self, error: SubmitError) -> SubmitError {
        let _ = self
            .shared
            .frames_enqueue_failed
            .fetch_add(1, Ordering::Relaxed);
        error
    }

    /// Counts a frame the non-blocking path had to drop rather than queue.
    fn dropped_diagnostic(&self, error: SubmitError) -> SubmitError {
        let _ = self
            .shared
            .diagnostics_dropped
            .fetch_add(1, Ordering::Relaxed);
        self.rejected(error)
    }

    /// Classifies a permit the non-blocking path could not take.
    ///
    /// Only a full queue drops a diagnostic. A permit closed by shutdown or a
    /// retired writer means the stream is closed, even when the open check passed.
    fn permit_error(&self, error: TryAcquireError) -> SubmitError {
        match error {
            TryAcquireError::NoPermits => self.dropped_diagnostic(SubmitError::WouldBlock),
            TryAcquireError::Closed => self.rejected(self.shared.closed_error()),
        }
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
    /// This is the backpressure path: a queue that is full by frame count or by
    /// queued bytes slows the producer down instead of dropping data.
    ///
    /// # Errors
    ///
    /// Returns [`SubmitError`] when the stream is closing, its writer stopped,
    /// the frame exceeds the stream byte budget, or a direct fallback write
    /// failed.
    pub async fn submit(&self, frame: Frame) -> Result<(), SubmitError> {
        let Some(queued) = self.queued.as_ref() else {
            return write_direct(self.id, &frame);
        };
        let reservation = queued
            .reservation(frame.len())
            .map_err(|error| queued.rejected(error))?;
        queued
            .ensure_open()
            .map_err(|error| queued.rejected(error))?;
        let Ok(bytes) = Arc::clone(&queued.bytes)
            .acquire_many_owned(reservation)
            .await
        else {
            return Err(queued.rejected(SubmitError::QueueClosed));
        };
        let Ok(slot) = Arc::clone(&queued.frame_slots).acquire_owned().await else {
            return Err(queued.rejected(SubmitError::QueueClosed));
        };
        queued.enqueue(QueuedFrame { frame, bytes, slot })
    }

    /// Submits a frame without ever blocking the calling thread.
    ///
    /// Synchronous callers such as the self-tracing layer run on engine core
    /// threads and must not stall, so a full queue drops the frame and counts
    /// it in `diagnostics_dropped` instead.
    ///
    /// # Errors
    ///
    /// Returns [`SubmitError::WouldBlock`] when the queue is full by frame count
    /// or by queued bytes, [`SubmitError::FrameTooLarge`] when the frame can
    /// never fit, or the other [`SubmitError`] variants when the stream no
    /// longer accepts frames.
    pub fn try_submit(&self, frame: Frame) -> Result<(), SubmitError> {
        let Some(queued) = self.queued.as_ref() else {
            return write_direct(self.id, &frame);
        };
        let reservation = queued
            .reservation(frame.len())
            .map_err(|error| queued.dropped_diagnostic(error))?;
        queued
            .ensure_open()
            .map_err(|error| queued.rejected(error))?;
        let bytes = Arc::clone(&queued.bytes)
            .try_acquire_many_owned(reservation)
            .map_err(|error| queued.permit_error(error))?;
        let slot = Arc::clone(&queued.frame_slots)
            .try_acquire_owned()
            .map_err(|error| queued.permit_error(error))?;
        queued.enqueue(QueuedFrame { frame, bytes, slot })
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

/// Retires the stream once its writer thread exits, including on panic.
///
/// The queue closes, byte and queue-slot waiters fail, and the frames left in
/// the queue are dropped, returning their bytes.
struct TeardownGuard {
    shared: Arc<StreamShared>,
    bytes: Arc<Semaphore>,
    frame_slots: Arc<Semaphore>,
    receiver: async_channel::Receiver<Command>,
    /// Set only after the writer drained cleanly, so a panic counts as a failure.
    drained: bool,
}

impl Drop for TeardownGuard {
    fn drop(&mut self) {
        // Recorded before the queue closes; see `StreamShared::closed_error`.
        if !self.drained {
            self.shared.writer_failed.store(true, Ordering::Release);
        }
        let _ = self.receiver.close();
        self.bytes.close();
        self.frame_slots.close();
        // Nothing can be queued after the close, so this empties the queue for good. Each
        // dropped frame returns its bytes, and a dropped barrier fails its drain.
        while self.receiver.try_recv().is_ok() {}
    }
}

/// Why a control command could not be queued.
enum ControlSendError {
    /// Every control slot stayed taken until the deadline.
    Timeout,
    /// The queue is closed, because the stream shut down or its writer retired.
    Disconnected,
}

/// Owner of one stream's bounded queue and dedicated writer thread.
pub struct OutputStream {
    handle: StreamHandle,
    shared: Arc<StreamShared>,
    sender: async_channel::Sender<Command>,
    /// Byte budget shared with producers, closed at shutdown to fail waiting submits.
    bytes: Arc<Semaphore>,
    /// Frame slots shared with producers, closed at shutdown to fail waiting submits.
    frame_slots: Arc<Semaphore>,
    /// Queue slots for drain barriers; drain runs on any thread.
    control_slots: Arc<Semaphore>,
    done: flume::Receiver<WriterExit>,
    // Behind a lock so the process-wide service, which only ever holds a shared
    // reference, can still join the writer during terminal shutdown.
    worker: Mutex<Option<thread::JoinHandle<()>>>,
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
        byte_capacity: usize,
        flush_on_idle: bool,
        sink: Box<dyn OutputSink>,
    ) -> io::Result<Self> {
        let capacity = capacity.max(1);
        // Frames and control commands each hold a slot, so the queue can never be full for either.
        let (sender, receiver) = async_channel::bounded::<Command>(capacity + CONTROL_SLOTS);
        let (done_tx, done) = flume::bounded::<WriterExit>(1);
        // Permits are counted in bytes, and one acquire is capped at u32.
        let byte_capacity = byte_capacity.clamp(1, u32::MAX as usize);
        let bytes = Arc::new(Semaphore::new(byte_capacity));
        let frame_slots = Arc::new(Semaphore::new(capacity));
        let shared = Arc::new(StreamShared::default());
        let worker_shared = Arc::clone(&shared);
        let worker_bytes = Arc::clone(&bytes);
        let worker_frame_slots = Arc::clone(&frame_slots);
        let worker = thread::Builder::new()
            .name(id.thread_name().to_owned())
            .spawn(move || {
                let mut guard = TeardownGuard {
                    shared: Arc::clone(&worker_shared),
                    bytes: worker_bytes,
                    frame_slots: worker_frame_slots,
                    receiver,
                    drained: false,
                };
                let exit = run_writer(id, &guard.receiver, sink, &worker_shared, flush_on_idle);
                guard.drained = exit == WriterExit::Drained;
                // Retire first, so shutdown never joins a writer still discarding frames.
                drop(guard);
                // A panic instead drops this sender, which callers read as a failed drain.
                let _ = done_tx.send(exit);
            })?;

        Ok(Self {
            handle: StreamHandle {
                id,
                queued: Some(QueuedHandle {
                    sender: sender.clone(),
                    shared: Arc::clone(&shared),
                    bytes: Arc::clone(&bytes),
                    byte_capacity,
                    frame_slots: Arc::clone(&frame_slots),
                }),
            },
            shared,
            sender,
            bytes,
            frame_slots,
            control_slots: Arc::new(Semaphore::new(CONTROL_SLOTS)),
            done,
            worker: Mutex::new(Some(worker)),
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
        match self.send_control(started, deadline, |slot| Command::Barrier(ack_tx, slot)) {
            Ok(()) => {}
            Err(ControlSendError::Timeout) => return self.pending_outcome(true),
            Err(ControlSendError::Disconnected) => return self.pending_outcome(false),
        }
        let remaining = deadline.saturating_sub(started.elapsed());
        match ack_rx.recv_timeout(remaining) {
            Ok(()) => ShutdownOutcome::default(),
            Err(flume::RecvTimeoutError::Timeout) => self.pending_outcome(true),
            Err(flume::RecvTimeoutError::Disconnected) => self.pending_outcome(false),
        }
    }

    /// Stops accepting frames, drains what was accepted, and flushes.
    ///
    /// Returns once the writer finishes or the deadline expires, whichever comes
    /// first, so a stalled console pipe cannot block the caller indefinitely. A
    /// submit that has not reached the queue when shutdown starts, including one
    /// still waiting for capacity, fails with [`SubmitError::QueueClosed`].
    pub fn shutdown(&self, deadline: Duration) -> ShutdownOutcome {
        // Closed on this thread, so producers are refused at once even behind a stalled
        // writer, which still receives everything queued before the close.
        let _ = self.sender.close();
        self.bytes.close();
        self.frame_slots.close();
        let exit = self.done.recv_timeout(deadline);
        let deadline_expired = matches!(exit, Err(flume::RecvTimeoutError::Timeout));

        // A timed-out writer is still running, so it cannot be joined here.
        if !deadline_expired
            && let Some(worker) = self
                .worker
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take()
        {
            let _ = worker.join();
        }

        if exit == Ok(WriterExit::Drained) {
            return ShutdownOutcome::default();
        }

        let outcome = self.pending_outcome(deadline_expired);
        let _ = self
            .shared
            .frames_dropped_shutdown
            .fetch_add(outcome.frames_pending, Ordering::Relaxed);
        outcome
    }

    /// Reports the frames that were accepted but are not on the stream yet.
    fn pending_outcome(&self, deadline_expired: bool) -> ShutdownOutcome {
        ShutdownOutcome {
            drained: false,
            writer_failed: self.shared.writer_failed.load(Ordering::Acquire),
            deadline_expired,
            frames_pending: self.shared.frames_pending(),
        }
    }

    /// Queues a control command once a control slot frees up, giving up at the deadline.
    ///
    /// A barrier left behind by a timed-out drain keeps its slot until the writer
    /// takes it, so repeated drains behind a stalled writer stay bounded.
    fn send_control(
        &self,
        started: Instant,
        deadline: Duration,
        command: impl FnOnce(OwnedSemaphorePermit) -> Command,
    ) -> Result<(), ControlSendError> {
        let slot = loop {
            match Arc::clone(&self.control_slots).try_acquire_owned() {
                Ok(slot) => break slot,
                Err(TryAcquireError::Closed) => return Err(ControlSendError::Disconnected),
                Err(TryAcquireError::NoPermits) if started.elapsed() >= deadline => {
                    return Err(ControlSendError::Timeout);
                }
                Err(TryAcquireError::NoPermits) => thread::sleep(CONTROL_SLOT_POLL),
            }
        };
        // The slot guarantees room, so only a closed queue refuses the command.
        self.sender
            .try_send(command(slot))
            .map_err(|_| ControlSendError::Disconnected)
    }
}

/// Applies one deadline across both streams and preserves each outcome for tests.
fn shutdown_streams(
    stdout: &OutputStream,
    stderr: &OutputStream,
    deadline: Duration,
) -> (ShutdownOutcome, ShutdownOutcome) {
    let started = Instant::now();
    let stdout_outcome = stdout.shutdown(deadline);
    let remaining = deadline.saturating_sub(started.elapsed());
    let stderr_outcome = stderr.shutdown(remaining);
    (stdout_outcome, stderr_outcome)
}

/// Drains the queue and writes each frame contiguously.
///
/// Runs until the queue is closed and empty, so every frame queued before
/// shutdown closed it is written.
fn run_writer(
    id: StreamId,
    receiver: &async_channel::Receiver<Command>,
    mut sink: Box<dyn OutputSink>,
    shared: &StreamShared,
    flush_on_idle: bool,
) -> WriterExit {
    let mut pending_bytes = 0usize;
    let mut pending_frames = 0usize;

    while let Ok(first) = receiver.recv_blocking() {
        let mut next = Some(first);
        while let Some(command) = next.take() {
            match command {
                Command::Frame(QueuedFrame { frame, bytes, slot }) => {
                    // The frame has left the queue, so its slot can admit the next one.
                    drop(slot);
                    if !write_one(id, sink.as_mut(), &frame, shared) {
                        return WriterExit::Failed;
                    }
                    pending_bytes = pending_bytes.saturating_add(frame.len());
                    pending_frames += 1;
                    // Dropping the reservation returns the frame's bytes to the budget.
                    drop(bytes);
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
                Command::Barrier(ack, slot) => {
                    drop(slot);
                    if !flush_sink(id, sink.as_mut(), shared) {
                        return WriterExit::Failed;
                    }
                    pending_bytes = 0;
                    pending_frames = 0;
                    let _ = ack.send(());
                }
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

/// Latches the stream as failed and reports the failure off the failed stream.
fn report_failure(id: StreamId, shared: &StreamShared, error: &io::Error) {
    let _ = shared.write_errors.fetch_add(1, Ordering::Relaxed);
    shared.writer_failed.store(true, Ordering::Release);
    match id {
        // The self-tracing layer routes errors to stderr, so this cannot
        // recurse into the stream that just failed.
        StreamId::Stdout => otel_error!(
            "output_service.write_failed",
            stream = id.as_str(),
            error = ?error,
            message = "Console writer stopped after a write or flush error"
        ),
        // Diagnostics go to stderr, which is the stream that just died.
        StreamId::Stderr => report_dead_stderr_on_stdout(error),
    }
}

/// Last-resort notice that the stderr writer stopped, emitted on stdout.
///
/// Skipped when stdout carries records, because a prose line there would break
/// the guarantee this service exists to provide. The failure stays visible in
/// `write_errors`, in [`ShutdownOutcome::writer_failed`], and in the run result
/// the controller returns.
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
    write_stderr_failure_notice(&stdout, error);
}

/// Submits the dead-stderr notice to an already-vetted fallback stream.
fn write_stderr_failure_notice(fallback: &StreamHandle, error: &io::Error) {
    let _ = fallback.try_submit(Frame::line(&format!(
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
            config.stdout_byte_capacity,
            config.flush_on_idle,
            StreamId::Stdout.std_sink(),
        )?;
        let stderr = OutputStream::start(
            StreamId::Stderr,
            config.stderr_queue_capacity,
            config.stderr_byte_capacity,
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

    /// Returns the handle human-readable engine diagnostics must use.
    ///
    /// Diagnostics always use stderr. Exporters are built while pipelines start,
    /// so a stdout claim can arrive after the first diagnostics are emitted;
    /// keeping prose off stdout unconditionally is what makes stdout parseable
    /// no matter when that claim lands.
    #[must_use]
    pub fn diagnostics() -> StreamHandle {
        Self::stderr()
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

    /// Stops both writers, draining and flushing what they already accepted.
    ///
    /// This is the terminal operation: it closes both queues, waits for the queued
    /// frames, and joins the writer threads. Only the process host may call it,
    /// because the streams do not accept frames afterwards. An engine run that
    /// shares the process with later runs uses [`OutputService::drain`] instead.
    pub fn shutdown(deadline: Duration) -> ShutdownOutcome {
        let Some(service) = SERVICE.get() else {
            return ShutdownOutcome::default();
        };
        let (stdout, stderr) = shutdown_streams(&service.stdout, &service.stderr, deadline);
        let mut outcome = stdout;
        outcome.merge(stderr);
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
    use std::collections::VecDeque;
    use std::future::Future;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicU32;
    use std::task::{Context, Waker};

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
        fail_on: Option<Vec<u8>>,
        flush_fails: bool,
        stalled: Option<Arc<AtomicBool>>,
        panics: bool,
    }

    impl TestSink {
        fn new() -> Self {
            Self {
                buffer: Arc::new(Mutex::new(Vec::new())),
                flushes: Arc::new(AtomicU32::new(0)),
                delay: None,
                fail_after: None,
                fail_on: None,
                flush_fails: false,
                stalled: None,
                panics: false,
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

        fn failing_on(mut self, frame: &[u8]) -> Self {
            self.fail_on = Some(frame.to_vec());
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

        fn panicking(mut self) -> Self {
            self.panics = true;
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
            if self.panics {
                panic!("simulated writer panic");
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
            if self.fail_on.as_deref() == Some(frame) {
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

    /// A byte budget large enough that count-based tests never hit it.
    const TEST_BYTE_CAPACITY: usize = DEFAULT_STDOUT_BYTE_CAPACITY;

    fn start(sink: &TestSink, capacity: usize) -> OutputStream {
        OutputStream::start(
            StreamId::Stdout,
            capacity,
            TEST_BYTE_CAPACITY,
            true,
            sink.boxed(),
        )
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
        let stream = start(&sink, 16);
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
        let stream = start(&sink, 4);
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
        let stream = start(&sink, 8);
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
        let stream = start(&sink, 4);
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
        let stream = OutputStream::start(
            StreamId::Stdout,
            CAPACITY,
            TEST_BYTE_CAPACITY,
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
        let stream = OutputStream::start(
            StreamId::Stderr,
            4,
            TEST_BYTE_CAPACITY,
            true,
            Box::new(sink.clone()),
        )
        .expect("spawn");
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
        let stream = OutputStream::start(
            StreamId::Stderr,
            4,
            TEST_BYTE_CAPACITY,
            false,
            Box::new(sink.clone()),
        )
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
    /// stops, so an accepted frame is never discarded when the queue closes.
    #[test]
    fn shutdown_never_drops_an_accepted_frame() {
        // Enough concurrent producers that at least one is reliably mid-enqueue
        // when shutdown closes the stream.
        const PRODUCERS: usize = 16;

        let sink = TestSink::new();
        let stream = start(&sink, 4);
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

    /// Scenario: shutdown's deadline expires behind a stalled writer while a submit waits for
    /// byte capacity.
    /// Guarantees: shutdown closes the stream before it waits, so the parked submit fails with
    /// `QueueClosed` without the writer's help and is counted as a failed enqueue, while every
    /// accepted frame is still written once the writer resumes.
    #[tokio::test]
    async fn submit_parked_on_bytes_is_rejected_at_shutdown() {
        const BYTE_CAPACITY: usize = 1024;

        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = OutputStream::start(StreamId::Stdout, 8, BYTE_CAPACITY, true, sink.boxed())
            .expect("writer thread spawns");
        let handle = stream.handle();

        // The stalled writer holds the whole budget, so the next submit waits for bytes.
        handle
            .submit(Frame::new(vec![b'a'; BYTE_CAPACITY]))
            .await
            .expect("frame is accepted");
        let mut late = Box::pin(handle.submit(Frame::line("late")));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(late.as_mut().poll(&mut cx).is_pending());

        let outcome = stream.shutdown(Duration::from_millis(100));
        assert!(outcome.deadline_expired);
        // The writer is still stalled here, so only shutdown's own close can end the wait.
        let late = tokio::time::timeout(Duration::from_secs(5), late)
            .await
            .expect("the parked submit resolves while the writer is stalled");
        assert_eq!(late, Err(SubmitError::QueueClosed));

        stalled.store(false, Ordering::Release);
        assert!(wait_until(|| stream.stats().frames_written == 1));
        let stats = stream.stats();
        assert_eq!(stats.frames_submitted, 1);
        assert_eq!(stats.frames_written, stats.frames_submitted);
        assert_eq!(stats.frames_enqueue_failed, 1);
    }

    /// Scenario: a pending `submit` is cancelled while it waits for queue capacity.
    /// Guarantees: the cancelled attempt is not counted as accepted and leaves nothing for
    /// shutdown to wait on, so a later shutdown drains promptly instead of waiting out its
    /// deadline.
    #[tokio::test]
    async fn cancelled_submit_is_not_counted_and_does_not_delay_shutdown() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = start(&sink, 1);
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

        let stream = OutputStream::start(
            StreamId::Stderr,
            1,
            TEST_BYTE_CAPACITY,
            true,
            Box::new(PanickingSink),
        )
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

    /// Parks a submit on byte capacity behind two frames, then lets `sink` end the writer.
    async fn assert_writer_exit_releases_the_byte_budget(sink: TestSink, stalled: Arc<AtomicBool>) {
        const BYTE_CAPACITY: usize = 1024;
        const HALF: usize = BYTE_CAPACITY / 2;

        // stderr keeps the failure report away from the stream under test.
        let stream = OutputStream::start(StreamId::Stderr, 8, BYTE_CAPACITY, true, sink.boxed())
            .expect("writer thread spawns");
        let handle = stream.handle();

        // One frame reaches the stalled writer and one stays queued, filling the budget.
        handle
            .submit(Frame::new(vec![b'a'; HALF]))
            .await
            .expect("frame is accepted");
        handle
            .submit(Frame::new(vec![b'c'; HALF]))
            .await
            .expect("frame is accepted");
        // Needs more than one frame returns, so only retiring the stream ends its wait.
        let mut waiter = Box::pin(handle.submit(Frame::new(vec![b'd'; HALF + 1])));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(waiter.as_mut().poll(&mut cx).is_pending());

        stalled.store(false, Ordering::Release);
        let result = tokio::time::timeout(Duration::from_secs(5), waiter)
            .await
            .expect("a byte waiter must not hang after the writer exits");
        assert_eq!(result, Err(SubmitError::QueueClosed));

        let bytes = &handle.queued.as_ref().expect("queued handle").bytes;
        let started = Instant::now();
        while bytes.available_permits() < BYTE_CAPACITY
            && started.elapsed() < Duration::from_secs(5)
        {
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            bytes.available_permits(),
            BYTE_CAPACITY,
            "the abandoned frame must return its bytes"
        );

        let outcome = stream.shutdown(Duration::from_secs(5));
        assert!(outcome.writer_failed);
        assert!(!outcome.deadline_expired);
        assert_eq!(outcome.frames_pending, 2);
    }

    /// Scenario: the sink fails while one frame is queued and another submit waits for bytes.
    /// Guarantees: the waiting submit fails with `QueueClosed` instead of hanging, the
    /// abandoned frame is dropped so the whole byte budget returns, and shutdown settles.
    #[tokio::test]
    async fn write_error_releases_byte_waiters_and_abandoned_frames() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new()
            .stalling(Arc::clone(&stalled))
            .failing_after(0);
        assert_writer_exit_releases_the_byte_budget(sink, stalled).await;
    }

    /// Scenario: the writer thread panics while one frame is queued and another submit waits
    /// for bytes.
    /// Guarantees: the teardown still runs while the thread unwinds, so the waiting submit
    /// fails with `QueueClosed` and the abandoned frame returns its bytes.
    #[tokio::test]
    async fn writer_panic_releases_byte_waiters_and_abandoned_frames() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled)).panicking();
        assert_writer_exit_releases_the_byte_budget(sink, stalled).await;
    }

    /// Scenario: the writer fails while a submit parked on bytes is never polled again.
    /// Guarantees: the retiring writer does not wait for that submit, so shutdown returns
    /// within its deadline and reports the writer failure rather than a timeout.
    #[tokio::test]
    async fn failed_writer_does_not_wait_on_a_stuck_submit() {
        const BYTE_CAPACITY: usize = 1024;

        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new()
            .stalling(Arc::clone(&stalled))
            .failing_after(0);
        let stream = OutputStream::start(StreamId::Stderr, 8, BYTE_CAPACITY, true, sink.boxed())
            .expect("writer thread spawns");
        let handle = stream.handle();

        handle
            .submit(Frame::new(vec![b'a'; BYTE_CAPACITY]))
            .await
            .expect("frame is accepted");
        // Parked on bytes, then never polled again while shutdown runs.
        let mut stuck = Box::pin(handle.submit(Frame::line("stuck")));
        let parked = tokio::time::timeout(Duration::from_millis(20), stuck.as_mut()).await;
        assert!(parked.is_err(), "the submit must be waiting for bytes");

        stalled.store(false, Ordering::Release);
        let (outcome_tx, outcome_rx) = std::sync::mpsc::channel();
        let shutdown = thread::spawn(move || {
            let _ = outcome_tx.send(stream.shutdown(Duration::from_secs(5)));
        });
        let outcome = outcome_rx.recv_timeout(Duration::from_secs(10));
        drop(stuck);
        shutdown.join().expect("shutdown thread finishes");

        let outcome = outcome.expect("shutdown must return");
        assert!(outcome.writer_failed);
        assert!(
            !outcome.deadline_expired,
            "the retiring writer must not wait for a submit nobody polls"
        );
    }

    /// Scenario: shutdown is requested while frames are still queued behind a slow sink.
    /// Guarantees: every accepted frame is written and the sink is flushed before
    /// shutdown reports a completed drain.
    #[tokio::test]
    async fn shutdown_drains_and_flushes_queued_frames() {
        const FRAMES: usize = 16;

        let sink = TestSink::new().with_delay(Duration::from_millis(2));
        let stream = start(&sink, FRAMES);
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

    /// Scenario: queued frames reach the stream byte budget while the writer is stalled.
    /// Guarantees: the queue stops accepting bytes well before its frame count is
    /// reached, so a few large frames cannot retain unbounded memory.
    #[tokio::test]
    async fn queued_bytes_are_bounded_independently_of_frame_count() {
        const BYTE_CAPACITY: usize = 4096;
        const FRAME_BYTES: usize = 1024;

        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        // The frame count alone would admit 64 frames, the byte budget admits far fewer.
        let stream = OutputStream::start(
            StreamId::Stdout,
            64,
            BYTE_CAPACITY,
            true,
            Box::new(sink.clone()),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();

        let payload = "a".repeat(FRAME_BYTES - 1);
        let mut accepted = 0usize;
        for _ in 0..64 {
            match tokio::time::timeout(
                Duration::from_millis(50),
                handle.submit(Frame::line(&payload)),
            )
            .await
            {
                Ok(result) => {
                    result.expect("frame is accepted");
                    accepted += 1;
                }
                // The byte budget is exhausted, so this submit is applying backpressure.
                Err(_) => break,
            }
        }

        assert!(accepted > 0, "the budget must admit at least one frame");
        assert!(
            accepted <= BYTE_CAPACITY / FRAME_BYTES + 1,
            "queued bytes must be bounded by the budget, accepted {accepted} frames"
        );

        stalled.store(false, Ordering::Release);
        let _ = stream.shutdown(Duration::from_secs(5));
    }

    /// Scenario: a frame larger than the whole stream byte budget is submitted.
    /// Guarantees: both submit paths reject it immediately instead of waiting for
    /// capacity that draining could never free.
    #[tokio::test]
    async fn frame_larger_than_the_budget_is_rejected() {
        const BYTE_CAPACITY: usize = 1024;

        let sink = TestSink::new();
        let stream = OutputStream::start(
            StreamId::Stdout,
            8,
            BYTE_CAPACITY,
            true,
            Box::new(sink.clone()),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();

        let oversized = Frame::new(vec![b'a'; BYTE_CAPACITY + 1]);
        assert_eq!(
            handle.try_submit(oversized.clone()),
            Err(SubmitError::FrameTooLarge)
        );
        let awaited = tokio::time::timeout(Duration::from_secs(1), handle.submit(oversized)).await;
        assert_eq!(
            awaited.expect("an oversized submit must not wait"),
            Err(SubmitError::FrameTooLarge)
        );

        // A frame that exactly fills the budget is still accepted.
        handle
            .submit(Frame::new(vec![b'a'; BYTE_CAPACITY]))
            .await
            .expect("a frame the size of the budget is accepted");

        assert!(stream.shutdown(Duration::from_secs(5)).drained);
        assert_eq!(stream.stats().frames_written, 1);
    }

    /// Scenario: a submit waiting for byte capacity is cancelled.
    /// Guarantees: the cancelled attempt returns its reserved bytes, so a later
    /// submit of the same size is not blocked by a reservation nobody holds.
    #[tokio::test]
    async fn cancelled_submit_releases_its_reserved_bytes() {
        const BYTE_CAPACITY: usize = 1024;

        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = OutputStream::start(
            StreamId::Stdout,
            8,
            BYTE_CAPACITY,
            true,
            Box::new(sink.clone()),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();

        // Reserves the whole budget, so the next submit must wait for bytes.
        handle
            .submit(Frame::new(vec![b'a'; BYTE_CAPACITY]))
            .await
            .expect("the first frame is accepted");

        let cancelled = tokio::time::timeout(
            Duration::from_millis(100),
            handle.submit(Frame::new(vec![b'b'; BYTE_CAPACITY])),
        )
        .await;
        assert!(
            cancelled.is_err(),
            "the second submit must still be waiting"
        );

        stalled.store(false, Ordering::Release);
        assert!(stream.drain(Duration::from_secs(5)).drained);

        // Only possible if the cancelled attempt released its reservation.
        tokio::time::timeout(
            Duration::from_secs(5),
            handle.submit(Frame::new(vec![b'c'; BYTE_CAPACITY])),
        )
        .await
        .expect("a later submit must not inherit a cancelled reservation")
        .expect("frame is accepted");

        assert!(stream.shutdown(Duration::from_secs(5)).drained);
    }

    /// Scenario: shutdown runs against a sink that never completes a write.
    /// Guarantees: shutdown returns within its deadline and reports the frames it
    /// could not drain instead of blocking process exit.
    #[tokio::test]
    async fn shutdown_gives_up_on_a_stalled_sink() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = start(&sink, 8);
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

    /// Scenario: a writer has failed but still misses the shutdown join deadline.
    /// Guarantees: failure and timeout remain independent, so callers do not write
    /// directly to a stream whose writer thread may still hold its lock.
    #[tokio::test]
    async fn shutdown_reports_failure_and_timeout_independently() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = start(&sink, 1);
        stream
            .handle()
            .submit(Frame::line("stuck"))
            .await
            .expect("frame is accepted");
        stream.shared.writer_failed.store(true, Ordering::Release);

        let outcome = stream.shutdown(Duration::from_millis(200));

        assert!(outcome.writer_failed);
        assert!(outcome.deadline_expired);
        stalled.store(false, Ordering::Release);
        let _ = stream.shutdown(Duration::from_secs(5));
    }

    /// Scenario: both standard streams are stalled under one terminal deadline.
    /// Guarantees: stdout cannot consume more than the shared budget, stderr sees
    /// the exhausted budget, and both outcomes identify writers that may still run.
    #[tokio::test]
    async fn shutdown_splits_one_deadline_across_both_streams() {
        let stdout_stalled = Arc::new(AtomicBool::new(true));
        let stderr_stalled = Arc::new(AtomicBool::new(true));
        let stdout_sink = TestSink::new().stalling(Arc::clone(&stdout_stalled));
        let stderr_sink = TestSink::new().stalling(Arc::clone(&stderr_stalled));
        let stdout = OutputStream::start(
            StreamId::Stdout,
            4,
            TEST_BYTE_CAPACITY,
            true,
            Box::new(stdout_sink.clone()),
        )
        .expect("stdout writer spawns");
        let stderr = OutputStream::start(
            StreamId::Stderr,
            4,
            TEST_BYTE_CAPACITY,
            true,
            Box::new(stderr_sink.clone()),
        )
        .expect("stderr writer spawns");

        stdout
            .handle()
            .submit(Frame::line("stdout stuck"))
            .await
            .expect("stdout frame accepted");
        stderr
            .handle()
            .submit(Frame::line("stderr stuck"))
            .await
            .expect("stderr frame accepted");

        let started = Instant::now();
        let (stdout_outcome, stderr_outcome) =
            shutdown_streams(&stdout, &stderr, Duration::from_millis(200));

        assert!(stdout_outcome.deadline_expired);
        assert!(stderr_outcome.deadline_expired);
        assert!(started.elapsed() < Duration::from_secs(2));

        stdout_stalled.store(false, Ordering::Release);
        stderr_stalled.store(false, Ordering::Release);
        let _ = stdout.shutdown(Duration::from_secs(5));
        let _ = stderr.shutdown(Duration::from_secs(5));
    }

    /// Scenario: the process host shuts down before the global service was initialized.
    /// Guarantees: the no-service path is a completed no-op rather than a timeout or
    /// writer failure, so a CLI without initialized writers does not fail spuriously.
    #[test]
    fn uninitialized_service_shutdown_is_a_completed_no_op() {
        let outcome = OutputService::shutdown(Duration::from_millis(1));

        assert!(outcome.drained);
        assert!(!outcome.writer_failed);
        assert!(!outcome.deadline_expired);
        assert_eq!(outcome.frames_pending, 0);
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

    /// Scenario: `try_submit` is called after shutdown closed a healthy stream, and after
    /// shutdown of a stream whose writer failed.
    /// Guarantees: the frame is refused with `QueueClosed` or `WriterUnavailable` and counted
    /// as a failed enqueue, never as a dropped diagnostic, so a closed stream is not reported
    /// as a full one.
    #[test]
    fn try_submit_after_shutdown_counts_a_failed_enqueue() {
        let sink = TestSink::new();
        let stream = start(&sink, 4);
        assert!(stream.shutdown(Duration::from_secs(5)).drained);
        assert_eq!(
            stream.handle().try_submit(Frame::line("late")),
            Err(SubmitError::QueueClosed)
        );
        let stats = stream.stats();
        assert_eq!(stats.frames_enqueue_failed, 1);
        assert_eq!(stats.diagnostics_dropped, 0);

        // stderr keeps the failure report away from the stream under test.
        let failing = TestSink::new().failing_after(0);
        let failed = OutputStream::start(
            StreamId::Stderr,
            4,
            TEST_BYTE_CAPACITY,
            true,
            failing.boxed(),
        )
        .expect("writer thread spawns");
        let handle = failed.handle();
        handle
            .try_submit(Frame::line("doomed"))
            .expect("the frame is accepted before its write fails");
        assert!(failed.shutdown(Duration::from_secs(5)).writer_failed);
        let before = failed.stats();
        assert_eq!(
            handle.try_submit(Frame::line("late")),
            Err(SubmitError::WriterUnavailable)
        );
        let after = failed.stats();
        assert_eq!(
            after.frames_enqueue_failed,
            before.frames_enqueue_failed + 1
        );
        assert_eq!(after.diagnostics_dropped, 0);
    }

    /// Scenario: `try_submit` finds its byte or frame-slot permit closed, as when shutdown
    /// lands between the open check and the permit.
    /// Guarantees: the frame is refused with `QueueClosed` and counted as a failed enqueue, not
    /// as a dropped diagnostic, so a closing stream is never reported as a full one.
    #[test]
    fn try_submit_reports_a_closed_permit_as_a_closed_queue() {
        for close_bytes in [true, false] {
            let sink = TestSink::new();
            let stream = start(&sink, 4);
            let handle = stream.handle();
            let queued = handle.queued.as_ref().expect("queued handle");
            if close_bytes {
                queued.bytes.close();
            } else {
                queued.frame_slots.close();
            }

            assert_eq!(
                handle.try_submit(Frame::line("late")),
                Err(SubmitError::QueueClosed)
            );
            let stats = stream.stats();
            assert_eq!(stats.frames_submitted, 0);
            assert_eq!(stats.frames_enqueue_failed, 1);
            assert_eq!(stats.diagnostics_dropped, 0);
            assert!(stream.shutdown(Duration::from_secs(5)).drained);
        }
    }

    /// Scenario: a drain runs after the writer already stopped on an I/O error.
    /// Guarantees: the outcome separates a failed writer from an expired deadline, so
    /// an operator is not told output merely ran out of time.
    #[tokio::test]
    async fn drain_reports_a_failed_writer_separately_from_a_timeout() {
        let failing = TestSink::new().failing_after(0);
        let stream = OutputStream::start(
            StreamId::Stderr,
            4,
            TEST_BYTE_CAPACITY,
            true,
            Box::new(failing.clone()),
        )
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

    /// Scenario: engine prose is emitted before and after stdout starts carrying records.
    /// Guarantees: diagnostics always resolve to stderr, so a stdout claim that lands
    /// after startup cannot leave earlier prose on a machine-readable stdout.
    #[test]
    fn diagnostics_always_use_stderr() {
        assert_eq!(OutputService::diagnostics().stream_id(), StreamId::Stderr);

        OutputService::mark_structured_stdout();

        assert_eq!(OutputService::diagnostics().stream_id(), StreamId::Stderr);
    }

    /// Scenario: the stderr writer dies and the notice is routed to a live stdout stream.
    /// Guarantees: the fallback actually submits the failure line, so a dead stderr
    /// writer is reported somewhere a reader can see it.
    #[test]
    fn stderr_failure_notice_reaches_the_fallback_stream() {
        let sink = TestSink::new();
        let fallback = start(&sink, 4);

        write_stderr_failure_notice(
            &fallback.handle(),
            &io::Error::other("simulated console failure"),
        );
        assert!(fallback.shutdown(Duration::from_secs(5)).drained);

        let contents = String::from_utf8(sink.contents()).expect("utf8 output");
        assert_eq!(contents.lines().count(), 1);
        assert!(
            contents.starts_with("otap: stderr console writer stopped"),
            "unexpected notice: {contents}"
        );
        assert!(contents.ends_with("simulated console failure\n"));
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

    /// One scripted outcome of a [`ScriptedWriter`] write call.
    enum Step {
        /// Accepts at most this many bytes.
        Accept(u8),
        /// Fails the call with this error kind.
        Refuse(io::ErrorKind),
    }

    /// Writer that plays scripted write and flush outcomes, then accepts everything.
    struct ScriptedWriter {
        writes: VecDeque<Step>,
        flushes: VecDeque<io::ErrorKind>,
        written: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for ScriptedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let accepted = match self.writes.pop_front() {
                Some(Step::Refuse(kind)) => return Err(io::Error::from(kind)),
                Some(Step::Accept(limit)) => usize::from(limit).min(buf.len()),
                None => buf.len(),
            };
            self.written
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .extend_from_slice(&buf[..accepted]);
            Ok(accepted)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes
                .pop_front()
                .map_or(Ok(()), |kind| Err(io::Error::from(kind)))
        }
    }

    /// Sink that writes through the same retry helpers as the standard-stream sinks.
    struct RetryingSink(ScriptedWriter);

    impl OutputSink for RetryingSink {
        fn write_frame(&mut self, frame: &[u8]) -> io::Result<()> {
            write_all_retrying(&mut self.0, frame)
        }

        fn flush(&mut self) -> io::Result<()> {
            flush_retrying(&mut self.0)
        }
    }

    /// Scenario: a nonblocking stream takes part of a frame, then refuses writes and a flush with
    /// `WouldBlock`, and interrupts one write, before its reader catches up.
    /// Guarantees: the writer resumes the frame at the byte it stopped at, so every frame is
    /// written exactly once, later frames still go out, and the stream is not marked failed.
    #[tokio::test]
    async fn would_block_resumes_the_frame_where_it_stopped() {
        let written = Arc::new(Mutex::new(Vec::new()));
        let writer = ScriptedWriter {
            writes: VecDeque::from([
                Step::Accept(3),
                Step::Refuse(io::ErrorKind::WouldBlock),
                Step::Refuse(io::ErrorKind::Interrupted),
                Step::Accept(2),
                Step::Refuse(io::ErrorKind::WouldBlock),
            ]),
            flushes: VecDeque::from([io::ErrorKind::WouldBlock]),
            written: Arc::clone(&written),
        };
        let stream = OutputStream::start(
            StreamId::Stdout,
            4,
            TEST_BYTE_CAPACITY,
            true,
            Box::new(RetryingSink(writer)),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();

        handle
            .submit(Frame::line("first frame"))
            .await
            .expect("frame is accepted");
        handle
            .submit(Frame::line("second frame"))
            .await
            .expect("frame is accepted");
        let outcome = stream.shutdown(Duration::from_secs(5));

        assert!(outcome.drained);
        assert!(!outcome.writer_failed);
        assert_eq!(stream.stats().write_errors, 0);
        let contents = written
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        assert_eq!(contents, b"first frame\nsecond frame\n");
    }

    /// Scenario: a stream fails a write, then a flush, with an error other than `WouldBlock`
    /// or `Interrupted`.
    /// Guarantees: the retry helpers return that error without retrying, so a real console
    /// failure still stops the writer.
    #[test]
    fn errors_other_than_would_block_are_not_retried() {
        let written = Arc::new(Mutex::new(Vec::new()));
        let mut writer = ScriptedWriter {
            writes: VecDeque::from([Step::Accept(2), Step::Refuse(io::ErrorKind::BrokenPipe)]),
            flushes: VecDeque::from([io::ErrorKind::BrokenPipe]),
            written: Arc::clone(&written),
        };

        let error = write_all_retrying(&mut writer, b"frame\n").expect_err("the write fails");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(
            written
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_slice(),
            b"fr"
        );
        let error = flush_retrying(&mut writer).expect_err("the flush fails");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }

    /// Polls `done` every millisecond for up to five seconds.
    fn wait_until(mut done: impl FnMut() -> bool) -> bool {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(5) {
            if done() {
                return true;
            }
            thread::sleep(Duration::from_millis(1));
        }
        false
    }

    /// Scenario: a submit parked on a full one-frame queue is cancelled after the writer frees
    /// the slot it waited for, then a later accepted frame fails to write.
    /// Guarantees: the cancelled submit neither queues nor counts its frame, so the frame
    /// whose write failed is still reported as pending instead of being hidden by a count
    /// that fell behind the frames actually written.
    #[tokio::test]
    async fn cancelled_parked_submit_never_hides_a_later_lost_frame() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new()
            .stalling(Arc::clone(&stalled))
            .failing_on(b"lost\n");
        // stderr keeps the failure report away from the stream under test.
        let stream =
            OutputStream::start(StreamId::Stderr, 1, TEST_BYTE_CAPACITY, true, sink.boxed())
                .expect("writer thread spawns");
        let handle = stream.handle();

        // One frame reaches the stalled writer, the next fills the only queue slot.
        handle
            .submit(Frame::line("in-writer"))
            .await
            .expect("frame is accepted");
        handle
            .submit(Frame::line("queued"))
            .await
            .expect("frame is accepted");
        // Parked on the full queue, and never polled again once the writer makes room.
        let mut parked = Box::pin(handle.submit(Frame::line("cancelled")));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(parked.as_mut().poll(&mut cx).is_pending());

        stalled.store(false, Ordering::Release);
        assert!(wait_until(|| stream.stats().frames_written >= 2));
        // Leaves time for any frame the parked submit handed over to be written as well.
        thread::sleep(Duration::from_millis(50));
        drop(parked);
        let after_cancel = stream.stats();
        assert_eq!(
            after_cancel.frames_written, after_cancel.frames_submitted,
            "a cancelled submit must not leave an uncounted frame behind"
        );

        handle
            .submit(Frame::line("lost"))
            .await
            .expect("the frame is accepted before its write fails");
        assert!(wait_until(|| stream.stats().write_errors == 1));
        let outcome = stream.shutdown(Duration::from_secs(5));

        assert!(outcome.writer_failed);
        assert_eq!(
            outcome.frames_pending, 1,
            "the frame whose write failed must be reported"
        );
    }

    /// Scenario: a shutdown deadline expires while one frame is inside a stalled write, one
    /// is queued behind it, and one submit is parked on the full queue.
    /// Guarantees: the outcome reports the two accepted frames as pending but not the parked
    /// submit, whose frame never reached the queue, and that submit is refused with
    /// `QueueClosed` and counted instead of landing after shutdown.
    #[tokio::test]
    async fn deadline_snapshot_counts_queued_frames_but_not_parked_submits() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = start(&sink, 1);
        let handle = stream.handle();

        handle
            .submit(Frame::line("in-writer"))
            .await
            .expect("frame is accepted");
        handle
            .submit(Frame::line("queued"))
            .await
            .expect("frame is accepted");
        let mut parked = Box::pin(handle.submit(Frame::line("parked")));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(parked.as_mut().poll(&mut cx).is_pending());

        let outcome = stream.shutdown(Duration::from_millis(100));
        assert!(outcome.deadline_expired);
        assert_eq!(outcome.frames_pending, 2);
        let parked = tokio::time::timeout(Duration::from_secs(5), parked)
            .await
            .expect("the parked submit resolves once shutdown closes the stream");
        assert_eq!(parked, Err(SubmitError::QueueClosed));

        stalled.store(false, Ordering::Release);
        assert!(wait_until(|| stream.stats().frames_written == 2));
        let stats = stream.stats();
        assert_eq!(stats.frames_written, stats.frames_submitted);
        assert_eq!(stats.frames_enqueue_failed, 1);
    }

    /// Scenario: drains keep timing out behind a stalled writer, and then a producer submits
    /// a frame.
    /// Guarantees: barriers left behind by timed-out drains never take more than the control
    /// slots, so the frame still finds room in the queue instead of waiting behind them.
    #[tokio::test]
    async fn timed_out_drains_cannot_fill_the_queue_for_frames() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = start(&sink, 1);
        let handle = stream.handle();

        handle
            .submit(Frame::line("in-writer"))
            .await
            .expect("frame is accepted");
        let frame_slots = &handle.queued.as_ref().expect("queued handle").frame_slots;
        // The writer took the frame, so the queue holds only what the drains leave behind.
        assert!(wait_until(|| frame_slots.available_permits() == 1));
        for _ in 0..CONTROL_SLOTS + 2 {
            let outcome = stream.drain(Duration::from_millis(20));
            assert!(outcome.deadline_expired);
        }
        assert_eq!(
            stream.sender.len(),
            CONTROL_SLOTS,
            "only the control slots may hold stale barriers"
        );

        tokio::time::timeout(
            Duration::from_secs(1),
            handle.submit(Frame::line("after-drains")),
        )
        .await
        .expect("a frame must not wait behind stale barriers")
        .expect("frame is accepted");

        stalled.store(false, Ordering::Release);
        assert!(stream.shutdown(Duration::from_secs(5)).drained);
        assert_eq!(stream.stats().frames_written, 2);
    }

    /// Scenario: drains time out behind a stalled writer until they hold every control slot,
    /// shutdown then times out too, and a producer submits fresh frames through both paths.
    /// Guarantees: shutdown closes the stream on the calling thread before it waits for the
    /// writer, so the fresh frames fail with `QueueClosed` and are never counted as accepted,
    /// while the frame accepted earlier is still written once the writer resumes.
    #[tokio::test]
    async fn shutdown_refuses_new_frames_while_the_writer_is_stalled() {
        let stalled = Arc::new(AtomicBool::new(true));
        let sink = TestSink::new().stalling(Arc::clone(&stalled));
        let stream = start(&sink, 4);
        let handle = stream.handle();

        handle
            .submit(Frame::line("in-writer"))
            .await
            .expect("frame is accepted");
        for _ in 0..CONTROL_SLOTS {
            assert!(stream.drain(Duration::from_millis(20)).deadline_expired);
        }
        assert_eq!(stream.control_slots.available_permits(), 0);

        assert!(stream.shutdown(Duration::from_millis(100)).deadline_expired);
        assert_eq!(
            handle.submit(Frame::line("fresh")).await,
            Err(SubmitError::QueueClosed)
        );
        assert_eq!(
            handle.try_submit(Frame::line("fresh")),
            Err(SubmitError::QueueClosed)
        );
        let stats = stream.stats();
        assert_eq!(stats.frames_submitted, 1);
        assert_eq!(stats.frames_enqueue_failed, 2);
        assert_eq!(stats.diagnostics_dropped, 0);

        stalled.store(false, Ordering::Release);
        assert!(stream.shutdown(Duration::from_secs(5)).drained);
        assert_eq!(stream.stats().frames_written, 1);
    }

    /// Scenario: a stream shuts down cleanly, then is drained, shut down again, and sent a
    /// late frame through both submit paths.
    /// Guarantees: none of the later outcomes reports a writer failure and the late frame is
    /// refused as a closed queue, so a clean exit is never mistaken for a failed writer.
    #[tokio::test]
    async fn clean_shutdown_is_not_reported_as_a_writer_failure() {
        let sink = TestSink::new();
        let stream = start(&sink, 4);
        let handle = stream.handle();

        handle
            .submit(Frame::line("before shutdown"))
            .await
            .expect("frame is accepted");
        assert!(stream.shutdown(Duration::from_secs(5)).drained);

        assert!(!stream.drain(Duration::from_secs(1)).writer_failed);
        assert!(!stream.shutdown(Duration::from_secs(1)).writer_failed);
        assert_eq!(
            handle.submit(Frame::line("late")).await,
            Err(SubmitError::QueueClosed)
        );
        assert_eq!(
            handle.try_submit(Frame::line("late")),
            Err(SubmitError::QueueClosed)
        );
        assert_eq!(stream.stats().write_errors, 0);
    }

    /// Scenario: a writer stops on a write error, then the stream is shut down, drained, shut
    /// down again, and sent a late frame.
    /// Guarantees: every later outcome still reports the failure and the late frame is refused
    /// as an unavailable writer, so separating closure from failure hides no real failure.
    #[tokio::test]
    async fn failed_writer_stays_reported_after_shutdown() {
        let sink = TestSink::new().failing_after(0);
        // stderr keeps the failure report away from the stream under test.
        let stream =
            OutputStream::start(StreamId::Stderr, 4, TEST_BYTE_CAPACITY, true, sink.boxed())
                .expect("writer thread spawns");
        let handle = stream.handle();

        let _ = handle.submit(Frame::line("doomed")).await;
        assert!(stream.shutdown(Duration::from_secs(5)).writer_failed);

        assert!(stream.drain(Duration::from_secs(1)).writer_failed);
        assert!(stream.shutdown(Duration::from_secs(1)).writer_failed);
        assert_eq!(
            handle.submit(Frame::line("late")).await,
            Err(SubmitError::WriterUnavailable)
        );
    }
}
