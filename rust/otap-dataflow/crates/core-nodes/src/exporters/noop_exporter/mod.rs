// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

mod record_json;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::SignalType;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata_views::views::common::InstrumentationScopeView;
use otap_df_pdata_views::views::logs::{ LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,};
use otap_df_pdata_views::views::resource::ResourceView;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::views::otap::OtapLogsView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_telemetry::otel_error;
use otap_df_telemetry::self_tracing::{AnsiCode, ColorMode, LOG_BUFFER_SIZE, StyledBufWriter};
use self::record_json::RecordJsonFormatter;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// The URN for the noop exporter.
pub const NOOP_EXPORTER_URN: &str = "urn:otel:exporter:noop";

/// Exporter that does nothing.
pub struct NoopExporterOld;

/// Declare the Noop Exporter as a local exporter factory.

// // TODO: is this called?
// #[allow(unsafe_code)]
// #[otap_df_engine::component_inventory(category = Exporter)]
// #[distributed_slice(OTAP_EXPORTER_FACTORIES)]
// pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
//     name: NOOP_EXPORTER_URN,
//     create: |_pipeline: PipelineContext,
//              node: NodeId,
//              node_config: Arc<NodeUserConfig>,
//              exporter_config: &ExporterConfig,
//              _capabilities: &otap_df_engine::capability::registry::Capabilities| {
//         Ok(ExporterWrapper::local(
//             NoopExporter {},
//             node,
//             node_config,
//             exporter_config,
//         ))
//     },
//     wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
//     validate_config: otap_df_config::validation::no_config,
// };

/*

#[async_trait(?Send)]
impl Exporter<OtapPdata> for NoopExporterOld {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {
                    // do nothing
                }
            }
        }

        Ok(TerminalState::default())
    }
}

 */

/// Output formats supported by the console exporter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsoleOutputFormat {
    /// Human-readable hierarchical output intended for interactive inspection.
    #[default]
    Pretty,
    /// One compact log record JSON object per line.
    RecordJson,
}

/// Timestamp encodings supported by `record_json`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordJsonTimestampFormat {
    /// UTC RFC 3339 with nanosecond precision.
    #[default]
    Rfc3339,
    /// Nanoseconds since the Unix epoch as a decimal string.
    UnixNano,
}

/// Field names supported for the `record_json` log body.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordJsonBodyField {
    /// Emit the log body under `body`.
    #[default]
    Body,
    /// Emit the log body under `message`.
    Message,
}

/// Int64 encodings supported by `record_json`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordJsonInt64Format {
    /// Emit int64 values as JSON integers.
    #[default]
    Number,
    /// Emit int64 values as decimal strings.
    String,
}

/// Format-specific configuration for `record_json`.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct RecordJsonConfig {
    /// Timestamp encoding (default: rfc3339).
    #[serde(default)]
    pub timestamp_format: RecordJsonTimestampFormat,
    /// Log body field name (default: body).
    #[serde(default)]
    pub body_field: RecordJsonBodyField,
    /// Int64 encoding (default: number).
    #[serde(default)]
    pub int64_format: RecordJsonInt64Format,
    /// Include resource attributes in every record (default: false).
    #[serde(default)]
    pub resource: bool,
    /// Include scope context in every record (default: true).
    #[serde(default = "default_record_json_scope")]
    pub scope: bool,
    /// Include OpenTelemetry bookkeeping fields (default: false).
    #[serde(default)]
    pub otel: bool,
}

impl Default for RecordJsonConfig {
    fn default() -> Self {
        Self {
            timestamp_format: RecordJsonTimestampFormat::default(),
            body_field: RecordJsonBodyField::default(),
            int64_format: RecordJsonInt64Format::default(),
            resource: false,
            scope: default_record_json_scope(),
            otel: false,
        }
    }
}

/// Configuration for the console exporter
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NoopExporterConfig {
    /// Output format (default: pretty).
    #[serde(default)]
    pub format: ConsoleOutputFormat,
    /// Whether to use ANSI colors in output (default: true)
    #[serde(default = "default_color")]
    pub color: bool,
    /// Whether to use Unicode box-drawing characters (default: true)
    #[serde(default = "default_unicode")]
    pub unicode: bool,
    /// Format-specific options for `record_json`.
    #[serde(default)]
    pub record_json: RecordJsonConfig,
}

impl Default for NoopExporterConfig {
    fn default() -> Self {
        Self {
            format: ConsoleOutputFormat::default(),
            color: default_color(),
            unicode: default_unicode(),
            record_json: RecordJsonConfig::default(),
        }
    }
}

const fn default_color() -> bool {
    true
}

const fn default_unicode() -> bool {
    true
}

const fn default_record_json_scope() -> bool {
    true
}

/// Console exporter that prints OTLP data to stdout.
pub struct NoopExporter {
    formatter: ConsoleFormatter,
}

impl NoopExporter {
    /// Create a new console exporter with the given configuration.
    #[must_use]
    pub const fn new(config: NoopExporterConfig) -> Self {
        let formatter = match config.format {
            ConsoleOutputFormat::Pretty => {
                ConsoleFormatter::Pretty(HierarchicalFormatter::new(config.color, config.unicode))
            }
            ConsoleOutputFormat::RecordJson => {
                ConsoleFormatter::RecordJson(RecordJsonFormatter::new(config.record_json))
            }
        };
        Self { formatter }
    }
}

/// Declare the Console Exporter as a local exporter factory
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: NOOP_EXPORTER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        let config: NoopExporterConfig = serde_json::from_value(node_config.config.clone())
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse console exporter config: {}", e),
            })?;
        Ok(ExporterWrapper::local(
            NoopExporter::new(config),
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<NoopExporterConfig>,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for NoopExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    self.export(data.payload_ref()).await;
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {
                    // do nothing
                }
            }
        }

        Ok(TerminalState::default())
    }
}

impl NoopExporter {
    async fn export(&self, payload: &OtapPayload) {
        match payload.signal_type() {
            SignalType::Logs => self.export_logs(payload).await,
            SignalType::Traces => self.export_traces(payload).await,
            SignalType::Metrics => self.export_metrics(payload).await,
        }
    }

    async fn export_logs(&self, payload: &OtapPayload) {
        match payload {
            OtapPayload::OtlpBytes(bytes) => match RawLogsData::try_from(bytes) {
                Ok(logs_view) => {
                    self.formatter.print_logs_data(&logs_view).await;
                }
                Err(e) => {
                    otel_error!("console.logs_view.otlp_create_failed", error = ?e, message = "Failed to create OTLP logs view");
                }
            },
            OtapPayload::OtapArrowRecords(records) => match OtapLogsView::try_from(records) {
                Ok(logs_view) => {
                    self.formatter.print_logs_data(&logs_view).await;
                }
                Err(e) => {
                    otel_error!("console.logs_view.otap_create_failed", error = ?e, message = "Failed to create OTAP logs view");
                }
            },
        }
    }

    async fn export_traces(&self, _payload: &OtapPayload) {
        // TODO: Implement traces formatting.
        otel_error!(
            "console.traces.not_implemented",
            message = "Traces formatting not yet implemented"
        );
    }

    async fn export_metrics(&self, _payload: &OtapPayload) {
        // TODO: Implement metrics formatting.
        otel_error!(
            "console.metrics.not_implemented",
            message = "Metrics formatting not yet implemented"
        );
    }
}

/// Runtime-selected console formatter.
enum ConsoleFormatter {
    Pretty(HierarchicalFormatter),
    RecordJson(RecordJsonFormatter),
}

impl ConsoleFormatter {
    /// Format logs and write the complete payload to stdout.
    async fn print_logs_data<L: LogsDataView>(&self, logs_data: &L) {
        let mut output = Vec::new();
        let format_result = match self {
            Self::Pretty(formatter) => {
                formatter.format_logs_data_to(logs_data, &mut output);
                Ok(())
            }
            Self::RecordJson(formatter) => formatter.format_logs_data_to(logs_data, &mut output),
        };

        if let Err(err) = format_result {
            otel_error!(
                "console.format_failed",
                error = ?err,
                message = "Could not format console output"
            );
            return;
        }

        // Note: each per-core exporter currently creates a new Tokio stdout handle for every
        // payload. Because stdout is a process-global serialized sink, concurrent handles still
        // contend, and large writes can be reordered or interleaved. A future implementation
        // could move each core's complete formatted buffers through a bounded channel to one
        // dedicated process-wide writer thread, preserving backpressure while keeping blocking
        // I/O off the core threads. A filelog exporter could avoid this serialization by letting
        // each core write its logs to a separate file in parallel.
        use tokio::io::AsyncWriteExt;
        if let Err(err) = tokio::io::stdout().write_all(&output).await {
            otel_error!("console.write_failed", error = ?err, message = "Could not write to console");
        }
    }
}

/// Tree drawing characters (Unicode or ASCII).
#[derive(Clone, Copy)]
struct TreeChars {
    vertical: &'static str,
    tee: &'static str,
    corner: &'static str,
}

impl TreeChars {
    const UNICODE: Self = Self {
        vertical: "\u{2502}",
        tee: "\u{251C}\u{2500}",
        corner: "\u{2514}\u{2500}",
    };
    const ASCII: Self = Self {
        vertical: "|",
        tee: "+-",
        corner: "\\-",
    };
}

/// Hierarchical formatter for OTLP data.
pub struct HierarchicalFormatter {
    color: ColorMode,
    tree: TreeChars,
}

impl HierarchicalFormatter {
    /// Create a new hierarchical formatter.
    #[must_use]
    pub const fn new(use_color: bool, use_unicode: bool) -> Self {
        Self {
            color: if use_color {
                ColorMode::Color
            } else {
                ColorMode::NoColor
            },
            tree: if use_unicode {
                TreeChars::UNICODE
            } else {
                TreeChars::ASCII
            },
        }
    }

    /// Format logs from a LogsDataView to a writer.
    fn format_logs_data_to<L: LogsDataView>(&self, logs_data: &L, output: &mut Vec<u8>) {
        for resource_logs in logs_data.resources() {
            self.format_resource_logs_to(&resource_logs, output);
        }
    }

    /// Format a ResourceLogs with its nested scopes.
    fn format_resource_logs_to<R: ResourceLogsView>(
        &self,
        resource_logs: &R,
        output: &mut Vec<u8>,
    ) {
        let first_ts = self.get_first_log_timestamp(resource_logs);

        // Format resource header
        self.format_line(output, |w| {
            w.format_header_line(
                Some(first_ts),
                resource_logs.resource().iter().flat_map(|r| r.attributes()),
                |w| {
                    w.write_styled(AnsiCode::Cyan, |w| {
                        let _ = w.write_all(b"RESOURCE");
                    });
                    let _ = w.write_all(b"   ");
                },
                |w| {
                    let _ = w.write_all(b"v1.Resource");
                },
                |_| {}, // No line suffix.
            );
        });

        // Format each scope
        let mut scopes = resource_logs.scopes().peekable();
        while let Some(scope_logs) = scopes.next() {
            let is_last_scope = scopes.peek().is_none();
            self.format_scope_logs_to(&scope_logs, is_last_scope, output);
        }
    }

    /// Get the first timestamp from log records in a ResourceLogs.
    fn get_first_log_timestamp<R: ResourceLogsView>(&self, resource_logs: &R) -> SystemTime {
        for scope_logs in resource_logs.scopes() {
            for log_record in scope_logs.log_records() {
                if let Some(ts) = log_record.time_unix_nano() {
                    return nanos_to_time(ts);
                }
                if let Some(ts) = log_record.observed_time_unix_nano() {
                    return nanos_to_time(ts);
                }
            }
        }
        SystemTime::UNIX_EPOCH
    }

    /// Format a ScopeLogs with its nested log records.
    fn format_scope_logs_to<S: ScopeLogsView>(
        &self,
        scope_logs: &S,
        is_last_scope: bool,
        output: &mut Vec<u8>,
    ) {
        let first_ts = scope_logs
            .log_records()
            .find_map(|lr| lr.time_unix_nano().or_else(|| lr.observed_time_unix_nano()))
            .map(nanos_to_time)
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let prefix = self.tree.vertical;
        let scope = scope_logs.scope();

        // Extract name/version for inline display (keep as raw bytes to avoid allocation)
        let name = scope.as_ref().and_then(|s| s.name());
        let version = scope.as_ref().and_then(|s| s.version());

        self.format_line(output, |w| {
            w.format_header_line(
                Some(first_ts),
                scope.iter().flat_map(|s| s.attributes()),
                |w| {
                    let _ = w.write_all(prefix.as_bytes());
                    let _ = w.write_all(b" ");
                    w.write_styled(AnsiCode::Magenta, |w| {
                        let _ = w.write_all(b"SCOPE");
                    });
                    let _ = w.write_all(b"    ");
                },
                |w| match (name, version) {
                    (Some(n), Some(v)) => {
                        let _ = w.write_all(n);
                        let _ = w.write_all(b"/");
                        let _ = w.write_all(v);
                    }
                    (Some(n), None) => {
                        let _ = w.write_all(n);
                    }
                    _ => {
                        let _ = w.write_all(b"v1.InstrumentationScope");
                    }
                },
                |_| {}, // No line suffix.
            );
        });

        // Format each log record
        let mut records = scope_logs.log_records().peekable();
        while let Some(log_record) = records.next() {
            let is_last_record = records.peek().is_none();
            self.format_log_record_to(&log_record, is_last_scope, is_last_record, output);
        }
    }

    /// Format a single log record.
    fn format_log_record_to<L: LogRecordView>(
        &self,
        log_record: &L,
        is_last_scope: bool,
        is_last_record: bool,
        output: &mut Vec<u8>,
    ) {
        let time = log_record
            .time_unix_nano()
            .or_else(|| log_record.observed_time_unix_nano())
            .map(nanos_to_time)
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let event_name = log_record
            .event_name()
            .map(|s| String::from_utf8_lossy(s).into_owned());

        let severity = log_record.severity_number();
        let severity_text = log_record.severity_text();
        let tree = self.tree;

        self.format_line(output, |w| {
            w.format_log_line(
                Some(time),
                log_record,
                |w| {
                    let _ = w.write_all(tree.vertical.as_bytes());
                    let _ = w.write_all(b" ");
                    if is_last_record && is_last_scope {
                        let _ = w.write_all(tree.corner.as_bytes());
                    } else {
                        let _ = w.write_all(tree.tee.as_bytes());
                    }
                    let _ = w.write_all(b" ");
                    w.write_severity(severity, severity_text.as_ref().map(|s| s.as_ref()));
                },
                |w| {
                    if let Some(name) = event_name {
                        let _ = w.write_all(name.as_bytes());
                    }
                },
                |_| {}, // No line suffix (scope printed above).
            );
        });
    }

    /// Format a line to the output buffer.
    fn format_line<F>(&self, output: &mut Vec<u8>, f: F)
    where
        F: FnOnce(&mut StyledBufWriter<'_>),
    {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = StyledBufWriter::new(&mut buf, self.color);
        f(&mut w);
        let len = w.position();
        output.extend_from_slice(&buf[..len]);
    }
}

/// Convert nanoseconds since UNIX_EPOCH to SystemTime.
#[inline]
fn nanos_to_time(nanos: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_nanos(nanos)
}




