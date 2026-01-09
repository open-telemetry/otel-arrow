// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal logs collection for OTAP-Dataflow.
//!
//! Each pipeline thread has a single LogBuffer (via thread-local) that accumulates
//! log records. The pipeline runtime periodically flushes this buffer to the admin
//! via a channel. Components don't need to do anything special for logging.

use crate::error::Error;
use crate::registry::MetricsKey;
use crate::self_tracing::{ConsoleWriter, LogRecord, SavedCallsite};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{Event, Subscriber};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, Layer as TracingLayer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Registry};

/// A log entry with optional producer identification.
pub struct LogEntry {
    /// The log record (callsite, timestamp, encoded body/attrs).
    pub record: LogRecord,
    /// Optional key identifying the producing component (for first-party logs).
    /// None for third-party logs from libraries.
    pub producer_key: Option<MetricsKey>,
}

/// A batch of log entries from a pipeline thread.
pub struct LogBatch {
    /// The log entries in this batch.
    pub entries: Vec<LogEntry>,
}

/// Thread-local log buffer for a pipeline thread.
///
/// All components on this thread share the same buffer.
/// The pipeline runtime flushes it periodically on a timer.
/// If the buffer fills before flush, new events are dropped.
pub struct LogBuffer {
    entries: Vec<LogEntry>,
    capacity: usize,
    dropped_count: u64,
}

impl LogBuffer {
    /// Create a new log buffer with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity.min(256)),
            capacity,
            dropped_count: 0,
        }
    }

    /// Push a log entry. If at capacity, the new entry is dropped.
    ///
    /// Returns true if the entry was added, false if dropped.
    pub fn push(&mut self, entry: LogEntry) -> bool {
        if self.entries.len() >= self.capacity {
            self.dropped_count += 1;
            false
        } else {
            self.entries.push(entry);
            true
        }
    }

    /// Push just a LogRecord with no producer key (for third-party events).
    ///
    /// Returns true if the entry was added, false if dropped.
    pub fn push_record(&mut self, record: LogRecord) -> bool {
        self.push(LogEntry {
            record,
            producer_key: None,
        })
    }

    /// Check if the buffer has entries to flush.
    #[must_use]
    pub fn needs_flush(&self) -> bool {
        !self.entries.is_empty()
    }

    /// Drain all entries from the buffer, returning them as a batch.
    pub fn drain(&mut self) -> LogBatch {
        LogBatch {
            entries: std::mem::take(&mut self.entries),
        }
    }

    /// Returns the number of dropped entries since creation.
    #[must_use]
    pub fn dropped_count(&self) -> u64 {
        self.dropped_count
    }
}

// Thread-local log buffer for the current pipeline thread.
thread_local! {
    static CURRENT_LOG_BUFFER: RefCell<Option<LogBuffer>> = const { RefCell::new(None) };
}

// Thread-local current MetricsKey for third-party instrumentation.
// When a component is executing, this is set to that component's key so that
// any tracing::info!() calls from libraries can be attributed to the component.
thread_local! {
    static CURRENT_PRODUCER_KEY: RefCell<Option<MetricsKey>> = const { RefCell::new(None) };
}

/// Guard that sets the current producer key for the duration of a scope.
///
/// When dropped, restores the previous key (or None).
/// This allows nested scoping if needed.
pub struct ProducerKeyGuard {
    previous: Option<MetricsKey>,
}

impl ProducerKeyGuard {
    /// Enter a scope with the given producer key.
    ///
    /// Third-party log events will be attributed to this key until
    /// the guard is dropped.
    #[must_use]
    pub fn enter(key: MetricsKey) -> Self {
        let previous = CURRENT_PRODUCER_KEY.with(|cell| cell.borrow_mut().replace(key));
        Self { previous }
    }
}

impl Drop for ProducerKeyGuard {
    fn drop(&mut self) {
        CURRENT_PRODUCER_KEY.with(|cell| {
            *cell.borrow_mut() = self.previous;
        });
    }
}

/// Get the current producer key (if any component scope is active).
#[must_use]
pub fn current_producer_key() -> Option<MetricsKey> {
    CURRENT_PRODUCER_KEY.with(|cell| *cell.borrow())
}

/// Install a log buffer for the current thread.
///
/// Called by the pipeline runtime when the thread starts.
pub fn install_thread_log_buffer(capacity: usize) {
    CURRENT_LOG_BUFFER.with(|cell| {
        *cell.borrow_mut() = Some(LogBuffer::new(capacity));
    });
}

/// Uninstall the log buffer for the current thread.
///
/// Called by the pipeline runtime when the thread shuts down.
pub fn uninstall_thread_log_buffer() {
    CURRENT_LOG_BUFFER.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Push a log record to the current thread's buffer (if installed).
///
/// If `producer_key` is None, uses the current thread-local producer key
/// (set via `ProducerKeyGuard::enter()`). This allows third-party instrumentation
/// to be attributed to the currently-executing component.
///
/// Returns false if no buffer is installed or buffer is full (event dropped).
pub fn push_to_thread_buffer(record: LogRecord, producer_key: Option<MetricsKey>) -> bool {
    CURRENT_LOG_BUFFER.with(|cell| {
        if let Some(ref mut buffer) = *cell.borrow_mut() {
            // Use explicit key if provided, otherwise use thread-current key
            let key = producer_key.or_else(current_producer_key);
            buffer.push(LogEntry { record, producer_key: key })
        } else {
            false
        }
    })
}

/// Flush the current thread's log buffer, returning the batch.
///
/// Called by the pipeline runtime on a timer.
pub fn flush_thread_log_buffer() -> Option<LogBatch> {
    CURRENT_LOG_BUFFER.with(|cell| {
        cell.borrow_mut().as_mut().and_then(|buffer| {
            if buffer.needs_flush() {
                Some(buffer.drain())
            } else {
                None
            }
        })
    })
}

/// Reporter for sending log batches through a channel.
#[derive(Clone)]
pub struct LogsReporter {
    sender: flume::Sender<LogBatch>,
}

impl LogsReporter {
    /// Create a new LogsReporter with the given sender.
    #[must_use]
    pub fn new(sender: flume::Sender<LogBatch>) -> Self {
        Self { sender }
    }

    /// Try to send a batch, non-blocking.
    ///
    /// If the channel is full, the batch is dropped (returns Ok).
    /// Only returns Err if the channel is disconnected.
    pub fn try_report(&self, batch: LogBatch) -> Result<(), Error> {
        match self.sender.try_send(batch) {
            Ok(()) => Ok(()),
            Err(flume::TrySendError::Full(_)) => Ok(()),
            Err(flume::TrySendError::Disconnected(_)) => Err(Error::LogsChannelClosed),
        }
    }
}

/// Collector that receives log batches and writes them to console.
pub struct LogsCollector {
    receiver: flume::Receiver<LogBatch>,
    writer: ConsoleWriter,
}

impl LogsCollector {
    /// Create a new collector and reporter pair.
    #[must_use]
    pub fn new(channel_size: usize) -> (Self, LogsReporter) {
        let (sender, receiver) = flume::bounded(channel_size);
        let collector = Self {
            receiver,
            writer: ConsoleWriter::color(),
        };
        let reporter = LogsReporter::new(sender);
        (collector, reporter)
    }

    /// Run the collection loop until the channel is closed.
    pub async fn run(self) -> Result<(), Error> {
        loop {
            match self.receiver.recv_async().await {
                Ok(batch) => {
                    self.write_batch(batch);
                }
                Err(_) => {
                    return Ok(());
                }
            }
        }
    }

    /// Write a batch of log entries to console.
    fn write_batch(&self, batch: LogBatch) {
        for entry in batch.entries {
            // Identifier.0 is the &'static dyn Callsite
            let metadata = entry.record.callsite_id.0.metadata();
            let saved = SavedCallsite::new(metadata);
            // Use ConsoleWriter's routing: ERROR/WARN to stderr, others to stdout
            self.writer.print_log_record(&entry.record, &saved);
            // TODO: include producer_key in output when present
        }
    }
}

// ============================================================================
// BufferWriterLayer - For engine threads with thread-local buffer
// ============================================================================

/// A tracing Layer for engine threads that writes to thread-local LogBuffer.
///
/// This layer is installed via `with_default()` on each engine thread.
/// Events are accumulated in the thread-local buffer and flushed on a timer.
pub struct BufferWriterLayer {
    /// Count of events successfully captured to the buffer.
    events_captured: AtomicU64,
    /// Count of events dropped because the buffer was full.
    events_dropped: AtomicU64,
}

impl BufferWriterLayer {
    /// Create a new BufferWriterLayer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            events_captured: AtomicU64::new(0),
            events_dropped: AtomicU64::new(0),
        }
    }

    /// Get the number of events successfully captured.
    #[must_use]
    pub fn events_captured(&self) -> u64 {
        self.events_captured.load(Ordering::Relaxed)
    }

    /// Get the number of events dropped because buffer was full.
    #[must_use]
    pub fn events_dropped(&self) -> u64 {
        self.events_dropped.load(Ordering::Relaxed)
    }
}

impl Default for BufferWriterLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> TracingLayer<S> for BufferWriterLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let record = LogRecord::new(event);
        let producer_key = current_producer_key();

        CURRENT_LOG_BUFFER.with(|cell| {
            if let Some(ref mut buffer) = *cell.borrow_mut() {
                if buffer.push(LogEntry { record, producer_key }) {
                    let _ = self.events_captured.fetch_add(1, Ordering::Relaxed);
                } else {
                    let _ = self.events_dropped.fetch_add(1, Ordering::Relaxed);
                }
            }
            // No buffer = programming error on engine thread, silently drop
        });
    }
}

// ============================================================================
// DirectChannelLayer - Global fallback for non-engine threads
// ============================================================================

/// A tracing Layer for non-engine threads that sends directly to channel.
///
/// This is installed as the global subscriber. Events are sent immediately
/// to the LogsCollector (non-blocking, dropped if channel is full).
pub struct DirectChannelLayer {
    /// Reporter for sending to the channel.
    reporter: LogsReporter,
    /// Count of events successfully sent.
    events_captured: AtomicU64,
    /// Count of events dropped because channel was full.
    events_dropped: AtomicU64,
}

impl DirectChannelLayer {
    /// Create a new DirectChannelLayer with the given reporter.
    #[must_use]
    pub fn new(reporter: LogsReporter) -> Self {
        Self {
            reporter,
            events_captured: AtomicU64::new(0),
            events_dropped: AtomicU64::new(0),
        }
    }

    /// Get the number of events successfully sent.
    #[must_use]
    pub fn events_captured(&self) -> u64 {
        self.events_captured.load(Ordering::Relaxed)
    }

    /// Get the number of events dropped because channel was full.
    #[must_use]
    pub fn events_dropped(&self) -> u64 {
        self.events_dropped.load(Ordering::Relaxed)
    }
}

impl<S> TracingLayer<S> for DirectChannelLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let record = LogRecord::new(event);
        // Non-engine threads don't have producer_key context
        let batch = LogBatch {
            entries: vec![LogEntry {
                record,
                producer_key: None,
            }],
        };

        match self.reporter.sender.try_send(batch) {
            Ok(()) => {
                let _ = self.events_captured.fetch_add(1, Ordering::Relaxed);
            }
            Err(flume::TrySendError::Full(_)) => {
                let _ = self.events_dropped.fetch_add(1, Ordering::Relaxed);
            }
            Err(flume::TrySendError::Disconnected(_)) => {
                // Channel closed, nothing we can do
            }
        }
    }
}

// ============================================================================
// Engine Thread Subscriber Setup
// ============================================================================

/// Create a subscriber for engine threads that uses BufferWriterLayer.
///
/// This subscriber captures events to the thread-local buffer instead of
/// sending them to the channel directly.
fn create_engine_thread_subscriber() -> impl Subscriber {
    // Use the same filter as the global subscriber (INFO by default, RUST_LOG override)
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    
    Registry::default()
        .with(filter)
        .with(BufferWriterLayer::new())
}

/// Run a closure with the engine thread subscriber as the default.
///
/// This should be called at the top of each engine thread to ensure all
/// logging on that thread goes to the thread-local buffer.
pub fn with_engine_thread_subscriber<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let subscriber = create_engine_thread_subscriber();
    tracing::subscriber::with_default(subscriber, f)
}
