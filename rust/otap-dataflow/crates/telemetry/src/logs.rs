// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal logs collection for OTAP-Dataflow.

use bytes::Bytes;
use crate::error::Error;
use crate::self_tracing::{ConsoleWriter, DirectLogRecordEncoder, LogRecord, RawLoggingLayer, SavedCallsite};
use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_pdata::proto::consts::field_num::logs::{
    LOGS_DATA_RESOURCE, RESOURCE_LOGS_SCOPE_LOGS, SCOPE_LOGS_LOG_RECORDS,
};
use otap_df_pdata::proto_encode_len_delimited_unknown_size;
use std::cell::RefCell;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer as TracingLayer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Registry;

/// A batch of log records from a pipeline thread.
pub struct LogBatch {
    /// The log records in this batch.
    pub records: Vec<LogRecord>,
    /// Number of records dropped in the same period.
    pub dropped_count: usize,
}

impl LogBatch {
    /// The total size including dropped records.
    pub fn size_with_dropped(&self) -> usize {
        self.records.len() + self.dropped_count
    }

    /// Encode this batch as an OTLP ExportLogsServiceRequest.
    ///
    /// The batch is wrapped in a minimal structure:
    /// - One ResourceLogs with no resource attributes
    /// - One ScopeLogs with no scope
    /// - All log records from the batch
    #[must_use]
    pub fn encode_export_logs_request(&self) -> Bytes {
        let mut buf = ProtoBuffer::with_capacity(self.records.len() * 256);

        // ExportLogsServiceRequest { resource_logs: [ ResourceLogs { ... } ] }
        proto_encode_len_delimited_unknown_size!(
            LOGS_DATA_RESOURCE, // field 1: resource_logs (same field number)
            {
                // ResourceLogs { scope_logs: [ ScopeLogs { ... } ] }
                // Note: we skip resource (field 1) to use empty/default resource
                proto_encode_len_delimited_unknown_size!(
                    RESOURCE_LOGS_SCOPE_LOGS, // field 2: scope_logs
                    {
                        // ScopeLogs { log_records: [ ... ] }
                        // Note: we skip scope (field 1) to use empty/default scope
                        for record in &self.records {
                            self.encode_log_record(record, &mut buf);
                        }
                    },
                    &mut buf
                );
            },
            &mut buf
        );

        buf.into_bytes()
    }

    /// Encode a single log record into the buffer.
    fn encode_log_record(&self, record: &LogRecord, buf: &mut ProtoBuffer) {
        // Get the callsite metadata for encoding
        let metadata = record.callsite_id.0.metadata();
        let callsite = SavedCallsite::new(metadata);

        proto_encode_len_delimited_unknown_size!(
            SCOPE_LOGS_LOG_RECORDS, // field 2: log_records
            {
                let mut encoder = DirectLogRecordEncoder::new(buf);
                // Clone record since encode_log_record takes ownership
                let _ = encoder.encode_log_record(record.clone(), &callsite);
            },
            buf
        );
    }
}

/// A payload of two kinds
pub enum LogPayload {
    /// A single record.
    Singleton(LogRecord),
    /// A batch.
    Batch(LogBatch),
}

impl LogPayload {
    /// The total number of records (including dropped) in this payload.
    pub fn size_with_dropped(&self) -> usize {
        match self {
            Self::Singleton(_) => 1,
            Self::Batch(batch) => batch.size_with_dropped(),
        }
    }
}

/// Thread-local log buffer for a pipeline thread.
pub struct LogBuffer {
    batch: LogBatch,
}

impl LogBuffer {
    /// Create a new log buffer with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            batch: LogBatch {
                records: Vec::with_capacity(capacity),
                dropped_count: 0,
            },
        }
    }

    /// Push a log record. If at capacity, the record is dropped and counted.
    pub fn push(&mut self, record: LogRecord) {
        if self.batch.records.len() >= self.batch.records.capacity() {
            self.batch.dropped_count += 1;
        } else {
            self.batch.records.push(record);
        }
    }

    /// Drain all records from the buffer, returning them as a batch.
    pub fn drain(&mut self) -> LogBatch {
        LogBatch {
            records: self.batch.records.drain(..).collect(),
            dropped_count: std::mem::take(&mut self.batch.dropped_count),
        }
    }
}

// Thread-local log buffer for the current pipeline thread.
thread_local! {
    static CURRENT_LOG_BUFFER: RefCell<Option<LogBuffer>> = const { RefCell::new(None) };
}

/// Run a closure with a thread-local log buffer installed.
///
/// The buffer is automatically uninstalled when the closure returns (or panics).
pub fn with_thread_log_buffer<F, R>(capacity: usize, f: F) -> R
where
    F: FnOnce() -> R,
{
    CURRENT_LOG_BUFFER.with(|cell| {
        *cell.borrow_mut() = Some(LogBuffer::new(capacity));
    });

    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            CURRENT_LOG_BUFFER.with(|cell| {
                *cell.borrow_mut() = None;
            });
        }
    }
    let _guard = Guard;

    f()
}

/// Drain the thread-local log buffer and return the batch.
///
/// Returns `None` if no buffer is installed (e.g., not in an engine thread).
/// This is for use by the internal telemetry receiver node.
#[must_use]
pub fn drain_thread_log_buffer() -> Option<LogBatch> {
    CURRENT_LOG_BUFFER.with(|cell| cell.borrow_mut().as_mut().map(|buffer| buffer.drain()))
}

/// Reporter for sending log batches through a channel.
#[derive(Clone)]
pub struct LogsReporter {
    sender: flume::Sender<LogPayload>,
}

impl LogsReporter {
    /// Create a new LogsReporter with the given sender.
    #[must_use]
    pub fn new(sender: flume::Sender<LogPayload>) -> Self {
        Self { sender }
    }

    /// Create a null reporter that discards all payloads.
    ///
    /// Used for internal telemetry mode where the buffer is drained directly
    /// rather than sent through a channel.
    #[must_use]
    pub fn null() -> Self {
        // Create a bounded channel of size 0 - sends will always fail
        // but we never actually call try_report on a null reporter
        let (sender, _receiver) = flume::bounded(0);
        Self { sender }
    }

    /// Try to send a payload, non-blocking.
    pub fn try_report(&self, payload: LogPayload) -> Result<(), Error> {
        self.sender
            .try_send(payload)
            .map_err(|e| Error::LogSendError {
                message: e.to_string(),
                dropped: e.into_inner().size_with_dropped(),
            })
    }
}

/// Collector that receives log batches and writes them to console.
pub struct LogsCollector {
    receiver: flume::Receiver<LogPayload>,
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
                Ok(payload) => {
                    self.write_batch(payload);
                }
                Err(err) => {
                    crate::raw_error!("log collector error:", err = err.to_string());
                    return Ok(());
                }
            }
        }
    }

    /// Write a batch of log records to console.
    fn write_batch(&self, payload: LogPayload) {
        // TODO: Print dropped count as a formatted warning before the batch
        match payload {
            LogPayload::Singleton(record) => self.write_record(record),
            LogPayload::Batch(batch) => {
                for record in batch.records {
                    self.write_record(record);
                }
            }
        }
    }

    /// Write one record.
    fn write_record(&self, record: LogRecord) {
        // Identifier.0 is the &'static dyn Callsite
        let metadata = record.callsite_id.0.metadata();
        let saved = SavedCallsite::new(metadata);
        // Use ConsoleWriter's routing: ERROR/WARN to stderr, others to stdout
        self.writer.raw_print(&record, &saved);
    }
}

/// A tracing Layer that buffers records in thread-local storage.
///
/// For engine threads that control their own flush timing.
pub struct ThreadBufferedLayer {
    /// Reporter for flushing batches.
    reporter: LogsReporter,
}

impl ThreadBufferedLayer {
    /// Create a new thread-buffered layer.
    #[must_use]
    pub fn new(reporter: LogsReporter) -> Self {
        Self { reporter }
    }

    /// Flush the current thread's log buffer and send via the channel.
    pub fn flush(&self) -> Result<(), Error> {
        if let Some(batch) =
            CURRENT_LOG_BUFFER.with(|cell| cell.borrow_mut().as_mut().map(|buffer| buffer.drain()))
        {
            let _ = self.reporter.try_report(LogPayload::Batch(batch))?;
        }
        Ok(())
    }
}

impl<S> TracingLayer<S> for ThreadBufferedLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let record = LogRecord::new(event);

        CURRENT_LOG_BUFFER.with(|cell| {
            if let Some(ref mut buffer) = *cell.borrow_mut() {
                buffer.push(record);
            }
            // TODO: Fallback consideration.
        });
    }
}

/// A tracing Layer that sends each record immediately.
pub struct UnbufferedLayer {
    /// Reporter for sending to the channel.
    reporter: LogsReporter,
}

impl UnbufferedLayer {
    /// Create a new unbuffered layer.
    #[must_use]
    pub fn new(reporter: LogsReporter) -> Self {
        Self { reporter }
    }
}

impl<S> TracingLayer<S> for UnbufferedLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let record = LogRecord::new(event);

        match self.reporter.try_report(LogPayload::Singleton(record)) {
            Ok(()) => {}
            Err(err) => {
                crate::raw_error!("failed to send log", err = err.to_string());
            }
        }
    }
}

/// Engine logging configuration, carrying the data needed for each mode.
///
/// This enum is constructed based on `config.logs.strategies.engine` and passed
/// to each engine thread. The engine thread uses `with_engine_subscriber()` to
/// run its work with the appropriate logging layer.
#[derive(Clone)]
pub enum EngineLogsSetup {
    /// Logs are silently dropped.
    Noop,
    /// Synchronous raw logging to console.
    Raw,
    /// Buffered: accumulates in thread-local buffer, flushed periodically.
    Buffered {
        /// Reporter to send batches through.
        reporter: LogsReporter,
        /// Buffer capacity per thread.
        capacity: usize,
    },
    /// Unbuffered: each log is sent immediately.
    Unbuffered {
        /// Reporter to send singletons through.
        reporter: LogsReporter,
    },
    /// Internal: accumulates in thread-local buffer, drained by internal telemetry receiver.
    Internal {
        /// Buffer capacity per thread.
        capacity: usize,
    },
}

/// Handle for flushing buffered logs from the engine thread.
///
/// For non-buffered modes, flush is a no-op.
#[derive(Clone)]
pub enum LogsFlusher {
    /// No-op flusher for modes that don't buffer.
    Noop,
    /// Flusher that drains the thread-local buffer and sends via the reporter.
    Buffered(LogsReporter),
    /// Flusher for internal telemetry mode - drain returns batch directly.
    /// Used by internal telemetry receiver node.
    InternalDrain,
}

impl LogsFlusher {
    /// Flush any buffered logs by sending to the reporter.
    ///
    /// For `Noop` and `InternalDrain`, this does nothing.
    /// For `Buffered`, this drains the thread-local buffer and sends as a batch.
    pub fn flush(&self) -> Result<(), Error> {
        match self {
            LogsFlusher::Noop | LogsFlusher::InternalDrain => Ok(()),
            LogsFlusher::Buffered(reporter) => {
                if let Some(batch) = CURRENT_LOG_BUFFER
                    .with(|cell| cell.borrow_mut().as_mut().map(|buffer| buffer.drain()))
                {
                    reporter.try_report(LogPayload::Batch(batch))?;
                }
                Ok(())
            }
        }
    }

    /// Drain the thread-local buffer and return the batch directly.
    ///
    /// For use by internal telemetry receiver only.
    /// Returns `None` if no buffer is installed or if this is not `InternalDrain` mode.
    pub fn drain(&self) -> Option<LogBatch> {
        match self {
            LogsFlusher::InternalDrain => {
                CURRENT_LOG_BUFFER.with(|cell| cell.borrow_mut().as_mut().map(|buffer| buffer.drain()))
            }
            _ => None,
        }
    }
}

impl EngineLogsSetup {
    /// Run a closure with the engine-appropriate tracing subscriber.
    ///
    /// Returns a `LogsFlusher` that can be used to periodically flush buffered logs.
    /// For non-buffered modes, the flusher is a no-op.
    pub fn with_engine_subscriber<F, R>(
        &self,
        log_level: otap_df_config::pipeline::service::telemetry::logs::LogLevel,
        f: F,
    ) -> R
    where
        F: FnOnce(LogsFlusher) -> R,
    {
        let filter = crate::get_env_filter(log_level);

        match self {
            EngineLogsSetup::Noop => {
                // Use NoSubscriber - events are dropped
                let subscriber = tracing::subscriber::NoSubscriber::new();
                tracing::subscriber::with_default(subscriber, || f(LogsFlusher::Noop))
            }
            EngineLogsSetup::Raw => {
                let subscriber = Registry::default()
                    .with(filter)
                    .with(RawLoggingLayer::new(ConsoleWriter::default()));
                tracing::subscriber::with_default(subscriber, || f(LogsFlusher::Noop))
            }
            EngineLogsSetup::Buffered { reporter, capacity } => {
                let layer = ThreadBufferedLayer::new(reporter.clone());
                let subscriber = Registry::default().with(filter).with(layer);
                let flusher = LogsFlusher::Buffered(reporter.clone());

                // Install the thread-local buffer
                with_thread_log_buffer(*capacity, || {
                    tracing::subscriber::with_default(subscriber, || f(flusher))
                })
            }
            EngineLogsSetup::Unbuffered { reporter } => {
                let layer = UnbufferedLayer::new(reporter.clone());
                let subscriber = Registry::default().with(filter).with(layer);
                tracing::subscriber::with_default(subscriber, || f(LogsFlusher::Noop))
            }
            EngineLogsSetup::Internal { capacity } => {
                // For internal mode, we use a "null" reporter that doesn't send anywhere.
                // The internal telemetry receiver will drain the buffer directly.
                let null_reporter = LogsReporter::null();
                let layer = ThreadBufferedLayer::new(null_reporter);
                let subscriber = Registry::default().with(filter).with(layer);
                let flusher = LogsFlusher::InternalDrain;

                // Install the thread-local buffer
                with_thread_log_buffer(*capacity, || {
                    tracing::subscriber::with_default(subscriber, || f(flusher))
                })
            }
        }
    }
}
