// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal logs collection for OTAP-Dataflow.

use crate::error::Error;
use crate::self_tracing::{
    ConsoleWriter, DirectLogRecordEncoder, LogRecord, RawLoggingLayer, SavedCallsite,
};
use bytes::Bytes;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use otap_df_config::pipeline::service::telemetry::logs::LogLevel;
use otap_df_config::pipeline::service::telemetry::AttributeValue;
use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_pdata::proto::consts::field_num::common::{
    ANY_VALUE_BOOL_VALUE, ANY_VALUE_DOUBLE_VALUE, ANY_VALUE_INT_VALUE, ANY_VALUE_STRING_VALUE,
    KEY_VALUE_KEY, KEY_VALUE_VALUE,
};
use otap_df_pdata::proto::consts::field_num::logs::{
    LOGS_DATA_RESOURCE, RESOURCE_LOGS_RESOURCE, RESOURCE_LOGS_SCOPE_LOGS, SCOPE_LOGS_LOG_RECORDS,
};
use otap_df_pdata::proto::consts::field_num::resource::RESOURCE_ATTRIBUTES;
use otap_df_pdata::proto::consts::wire_types;
use otap_df_pdata::proto_encode_len_delimited_unknown_size;
use std::collections::HashMap;
use tracing::{Event, Subscriber};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::{Context, Layer as TracingLayer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;

/// A batch of log records from a pipeline thread.
pub struct LogBatch {
    /// The log records in this batch.
    pub records: Vec<LogRecord>,
    /// Number of records dropped in the same period.
    pub dropped_count: usize,
}

/// Pre-encode resource bytes for use in OTLP log messages.
///
/// This encodes the resource attributes into a protobuf fragment that can be
/// inserted directly into ResourceLogs messages. The returned bytes include
/// the `RESOURCE_LOGS_RESOURCE` field tag and length-delimited Resource message.
///
/// This is a one-time operation at startup; the resulting bytes can be reused
/// for all subsequent log batches.
#[must_use]
pub fn encode_resource_bytes(resource_attributes: &HashMap<String, AttributeValue>) -> Bytes {
    if resource_attributes.is_empty() {
        return Bytes::new();
    }

    let mut buf = ProtoBuffer::with_capacity(resource_attributes.len() * 64);

    // Encode: field 1 (RESOURCE_LOGS_RESOURCE) -> Resource message
    proto_encode_len_delimited_unknown_size!(
        RESOURCE_LOGS_RESOURCE,
        {
            // Resource { attributes: [ KeyValue, ... ] }
            for (key, value) in resource_attributes {
                encode_resource_attribute(&mut buf, key, value);
            }
        },
        &mut buf
    );

    buf.into_bytes()
}

/// Encode a single resource attribute as a KeyValue message.
fn encode_resource_attribute(buf: &mut ProtoBuffer, key: &str, value: &AttributeValue) {
    proto_encode_len_delimited_unknown_size!(
        RESOURCE_ATTRIBUTES,
        {
            buf.encode_string(KEY_VALUE_KEY, key);
            proto_encode_len_delimited_unknown_size!(
                KEY_VALUE_VALUE,
                {
                    match value {
                        AttributeValue::String(s) => {
                            buf.encode_string(ANY_VALUE_STRING_VALUE, s);
                        }
                        AttributeValue::Bool(b) => {
                            buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
                            buf.encode_varint(u64::from(*b));
                        }
                        AttributeValue::I64(i) => {
                            buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
                            buf.encode_varint(*i as u64);
                        }
                        AttributeValue::F64(f) => {
                            buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
                            buf.extend_from_slice(&f.to_le_bytes());
                        }
                        AttributeValue::Array(_) => {
                            // Arrays not supported for resource attributes
                        }
                    }
                },
                buf
            );
        },
        buf
    );
}

impl LogBatch {
    /// The total size including dropped records.
    #[must_use]
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
        self.encode_export_logs_request_with_resource(&Bytes::new())
    }

    /// Encode this batch as an OTLP ExportLogsServiceRequest with pre-encoded resource.
    ///
    /// The `resource_bytes` should be pre-encoded using [`encode_resource_bytes`].
    /// This allows efficient reuse of the same resource for all log batches.
    #[must_use]
    pub fn encode_export_logs_request_with_resource(&self, resource_bytes: &Bytes) -> Bytes {
        let mut buf = ProtoBuffer::with_capacity(self.records.len() * 256 + resource_bytes.len());

        // ExportLogsServiceRequest { resource_logs: [ ResourceLogs { ... } ] }
        proto_encode_len_delimited_unknown_size!(
            LOGS_DATA_RESOURCE, // field 1: resource_logs (same field number)
            {
                // Insert pre-encoded resource (field 1: resource)
                buf.extend_from_slice(resource_bytes);

                // ResourceLogs { scope_logs: [ ScopeLogs { ... } ] }
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
    ///
    /// Returns:
    /// - `Ok(())` if the payload was sent
    /// - `Err` if the channel is full or disconnected
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

/// Type alias for the log payload receiver channel.
pub type LogsReceiver = flume::Receiver<LogPayload>;

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

    /// Create a reporter and receiver pair without the collector.
    ///
    /// Use this when the receiver will be consumed elsewhere (e.g., by the
    /// Internal Telemetry Receiver node).
    #[must_use]
    pub fn channel(channel_size: usize) -> (LogsReceiver, LogsReporter) {
        let (sender, receiver) = flume::bounded(channel_size);
        let reporter = LogsReporter::new(sender);
        (receiver, reporter)
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

        match self.reporter.try_report(LogPayload::Singleton(record)) {
            Ok(()) => {}
            Err(err) => {
                crate::raw_error!("failed to send log", err = %err);
            }
        }
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
