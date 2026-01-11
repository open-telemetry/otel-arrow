// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal logs collection for OTAP-Dataflow.

use crate::error::Error;
use crate::self_tracing::{ConsoleWriter, LogRecord, SavedCallsite};
use std::cell::RefCell;
use tracing::{Event, Subscriber};
//use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{
    Context,
    Layer as TracingLayer, //, SubscriberExt
};
use tracing_subscriber::registry::LookupSpan;
//use tracing_subscriber::{EnvFilter, Registry};

/// A batch of log records from a pipeline thread.
pub struct LogBatch {
    /// The log records in this batch.
    pub records: Vec<LogRecord>,
    /// Number of records dropped in the same period.
    pub dropped_count: usize,
}

/// A payload of two kinds
pub enum LogPayload {
    /// A single record.
    Singleton(LogRecord),
    /// A batch.
    Batch(LogBatch),
}

impl LogBatch {
    /// The total number of dropped if you drop this batch.
    pub fn size_with_dropped(&self) -> usize {
        self.records.len() + self.dropped_count
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

    /// Try to send a payload, non-blocking.
    pub fn try_report(&self, payload: LogPayload) -> Result<(), Error> {
        self.sender
            .try_send(payload)
            .map_err(|e| Error::LogSendError(e.to_string()))
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
pub struct BufferedLayer {
    /// Reporter for flushing batches.
    reporter: LogsReporter,
}

impl BufferedLayer {
    /// Create a new buffered layer.
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

impl<S> TracingLayer<S> for BufferedLayer
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

// Note: Commented below because not use, not ready, slightly incorrect.

// /// Create a subscriber for engine threads that uses BufferedLayer.
// fn create_engine_thread_subscriber() -> impl Subscriber {
//     // Use the same filter as the global subscriber (INFO by default, RUST_LOG override)
//     let filter = EnvFilter::builder()
//         .with_default_directive(LevelFilter::INFO.into())
//         .from_env_lossy();
//     Registry::default()
//         .with(filter)
//         .with(BufferedLayer::default())
// }

// /// Run a closure with the engine thread subscriber as the default.
// pub fn with_engine_thread_subscriber<F, R>(f: F) -> R
// where
//     F: FnOnce() -> R,
// {
//     let subscriber = create_engine_thread_subscriber();
//     tracing::subscriber::with_default(subscriber, f)
// }
