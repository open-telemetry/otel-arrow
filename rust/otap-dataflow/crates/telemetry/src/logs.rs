// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal logs collection for OTAP-Dataflow.

use crate::self_tracing::{ConsoleWriter, LogRecord, RawLoggingLayer, SavedCallsite};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use otap_df_config::pipeline::service::telemetry::logs::LogLevel;
use tracing::{Event, Subscriber};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::{Context, Layer as TracingLayer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;

/// A payload of log data
/// TODO: merge with Event in crates/state
pub enum LogPayload {
    /// A single record.
    Singleton(LogRecord),
}

/// Reporter for sending log batches through a channel.
pub type LogsReporter = flume::Sender<LogPayload>;

/// Type alias for the log payload receiver channel.
pub type LogsReceiver = flume::Receiver<LogPayload>;

/// Create a reporter and receiver pair without the collector.
///
/// Use this when the receiver will be consumed elsewhere (e.g., by the
/// Internal Telemetry Receiver node).
#[must_use]
pub fn channel(channel_size: usize) -> (LogsReporter, LogsReceiver) {
    flume::bounded(channel_size)
}

/// Direct logs collector
pub struct DirectCollector {
    writer: ConsoleWriter,
    receiver: LogsReceiver,
}

impl DirectCollector {
    /// New collector with writer.
    pub fn new(writer: ConsoleWriter, receiver: LogsReceiver) -> Self {
        Self { writer, receiver }
    }

    /// Run the collection loop until the channel is closed.
    pub async fn run(self) -> Result<(), crate::Error> {
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
        match payload {
            LogPayload::Singleton(record) => self.write_record(record),
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

/// A tracing Layer that sends each record immediately.
pub struct ImmediateLayer {
    /// Reporter for sending to the channel.
    reporter: LogsReporter,
}

impl ImmediateLayer {
    /// Create a new unbuffered layer.
    #[must_use]
    pub fn new(reporter: LogsReporter) -> Self {
        Self { reporter }
    }
}

impl<S> TracingLayer<S> for ImmediateLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let record = LogRecord::new(event);

        match self.reporter.try_send(LogPayload::Singleton(record)) {
            Ok(()) => {}
            Err(err) => {
                crate::raw_error!("failed to send log", err = %err);
            }
        };
    }
}

/// Telemetry setup for pipeline threads, carrying the data needed for each mode.
///
/// This enum is constructed based on `config.logs.providers.engine` (for main pipelines)
/// or `config.logs.providers.internal` (for the internal telemetry pipeline).
/// Pipeline threads use `with_subscriber()` to run with the appropriate logging layer.
#[derive(Clone)]
pub enum TelemetrySetup {
    /// Logs are silently dropped.
    Noop,
    /// Synchronous raw logging to console.
    Raw,
    /// Immediate: each log is sent immediately.
    Immediate {
        /// Reporter to send singletons through.
        reporter: LogsReporter,
    },
    /// OpenTelemetry SDK: logs go through the OpenTelemetry logging pipeline.
    OpenTelemetry {
        /// The OpenTelemetry SDK logger provider.
        logger_provider: SdkLoggerProvider,
    },
}

impl TelemetrySetup {
    /// Initialize this setup as the global tracing subscriber.
    ///
    /// This is used during startup to set the global subscriber. Returns an error
    /// if a global subscriber has already been set.
    pub fn try_init_global(
        &self,
        log_level: LogLevel,
    ) -> Result<(), tracing_subscriber::util::TryInitError> {
        use tracing_subscriber::util::SubscriberInitExt;

        let filter = crate::get_env_filter(log_level);

        match self {
            TelemetrySetup::Noop => tracing::subscriber::NoSubscriber::new().try_init(),
            TelemetrySetup::Raw => Registry::default()
                .with(filter)
                .with(RawLoggingLayer::new(ConsoleWriter::default()))
                .try_init(),
            TelemetrySetup::Immediate { reporter } => {
                let layer = ImmediateLayer::new(reporter.clone());
                Registry::default().with(filter).with(layer).try_init()
            }
            TelemetrySetup::OpenTelemetry { logger_provider } => {
                let sdk_layer = OpenTelemetryTracingBridge::new(logger_provider);
                Registry::default().with(filter).with(sdk_layer).try_init()
            }
        }
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    ///
    /// The closure runs with the configured logging layer active.
    pub fn with_subscriber<F, R>(&self, log_level: LogLevel, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let filter = crate::get_env_filter(log_level);

        match self {
            TelemetrySetup::Noop => {
                let subscriber = tracing::subscriber::NoSubscriber::new();
                tracing::subscriber::with_default(subscriber, f)
            }
            TelemetrySetup::Raw => {
                let subscriber = Registry::default()
                    .with(filter)
                    .with(RawLoggingLayer::new(ConsoleWriter::default()));
                tracing::subscriber::with_default(subscriber, f)
            }
            TelemetrySetup::Immediate { reporter } => {
                let layer = ImmediateLayer::new(reporter.clone());
                let subscriber = Registry::default().with(filter).with(layer);
                tracing::subscriber::with_default(subscriber, f)
            }
            TelemetrySetup::OpenTelemetry { logger_provider } => {
                let sdk_layer = OpenTelemetryTracingBridge::new(logger_provider);
                let subscriber = Registry::default().with(filter).with(sdk_layer);
                tracing::subscriber::with_default(subscriber, f)
            }
        }
    }
}
