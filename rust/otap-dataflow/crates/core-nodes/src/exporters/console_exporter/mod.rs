// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Console exporter that prints OTLP data in human-readable or structured formats.

mod metrics;
otap_df_telemetry::otel_component_scope!(
    urn = CONSOLE_EXPORTER_URN,
    target = "otel.exporter.console",
);

mod record_json;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::OtapPayload;
use otap_df_pdata::views::otap::OtapLogsView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata_views::views::common::InstrumentationScopeView;
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use otap_df_telemetry::self_tracing::{AnsiCode, ColorMode, LOG_BUFFER_SIZE, StyledBufWriter};
use otap_df_telemetry_macros::AttributeEnum;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use self::metrics::{ConsoleExportErrorType, ConsoleExporterMetrics};
use self::record_json::RecordJsonFormatter;

/// The URN for the console exporter
pub const CONSOLE_EXPORTER_URN: &str = "urn:otel:exporter:console";

/// Output formats supported by the console exporter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, AttributeEnum)]
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
pub struct ConsoleExporterConfig {
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

impl Default for ConsoleExporterConfig {
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
pub struct ConsoleExporter {
    formatter: ConsoleFormatter,
    metrics: ConsoleExporterMetrics,
}

impl ConsoleExporter {
    /// Create a new console exporter with the given configuration.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext, config: ConsoleExporterConfig) -> Self {
        let metrics = ConsoleExporterMetrics::register(pipeline_ctx, config.format);
        let formatter = match config.format {
            ConsoleOutputFormat::Pretty => {
                ConsoleFormatter::Pretty(HierarchicalFormatter::new(config.color, config.unicode))
            }
            ConsoleOutputFormat::RecordJson => {
                ConsoleFormatter::RecordJson(RecordJsonFormatter::new(config.record_json))
            }
        };
        Self { formatter, metrics }
    }

    fn terminal_state(&mut self, deadline: Instant) -> TerminalState {
        TerminalState::new(deadline, self.metrics.terminal_snapshots())
    }
}

/// Declare the Console Exporter as a local exporter factory
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static CONSOLE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: CONSOLE_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        let config: ConsoleExporterConfig = serde_json::from_value(node_config.config.clone())
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse console exporter config: {}", e),
            })?;
        Ok(ExporterWrapper::local(
            ConsoleExporter::new(&pipeline, config),
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<ConsoleExporterConfig>,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ConsoleExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = self.metrics.report(&mut metrics_reporter);
                }
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    return Ok(self.terminal_state(deadline));
                }
                Message::PData(data) => {
                    let export_start = Instant::now();
                    let signal = data.signal_type();
                    match self.export(data.payload_ref()).await {
                        Ok(()) => self.metrics.record_success(signal, export_start.elapsed()),
                        Err(error_type) => {
                            self.metrics
                                .record_failure(signal, error_type, export_start.elapsed())
                        }
                    }
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {
                    // do nothing
                }
            }
        }
    }
}

impl ConsoleExporter {
    async fn export(&self, payload: &OtapPayload) -> Result<(), ConsoleExportErrorType> {
        match payload.signal_type() {
            SignalType::Logs => self.export_logs(payload).await,
            SignalType::Traces => self.unsupported_signal("traces"),
            SignalType::Metrics => self.unsupported_signal("metrics"),
        }
    }

    async fn export_logs(&self, payload: &OtapPayload) -> Result<(), ConsoleExportErrorType> {
        match payload {
            OtapPayload::OtlpBytes(bytes) => match RawLogsData::try_from(bytes) {
                Ok(logs_view) => self.formatter.print_logs_data(&logs_view).await,
                Err(e) => {
                    otel_error!("console.logs_view.otlp_create_failed", error = ?e, message = "Failed to create OTLP logs view");
                    Err(ConsoleExportErrorType::OtlpViewCreation)
                }
            },
            OtapPayload::OtapArrowRecords(records) => match OtapLogsView::try_from(records) {
                Ok(logs_view) => self.formatter.print_logs_data(&logs_view).await,
                Err(e) => {
                    otel_error!("console.logs_view.otap_create_failed", error = ?e, message = "Failed to create OTAP logs view");
                    Err(ConsoleExportErrorType::OtapViewCreation)
                }
            },
        }
    }

    fn unsupported_signal(&self, signal: &'static str) -> Result<(), ConsoleExportErrorType> {
        otel_warn!(
            "console.message.unsupported_signal",
            signal = signal,
            message = "Console exporter supports logs only; use processor:debug followed by exporter:noop to inspect metrics or traces"
        );
        Err(ConsoleExportErrorType::UnsupportedSignal)
    }
}

/// Runtime-selected console formatter.
enum ConsoleFormatter {
    Pretty(HierarchicalFormatter),
    RecordJson(RecordJsonFormatter),
}

impl ConsoleFormatter {
    /// Format logs and write the complete payload to stdout.
    async fn print_logs_data<L: LogsDataView>(
        &self,
        logs_data: &L,
    ) -> Result<(), ConsoleExportErrorType> {
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
            return Err(ConsoleExportErrorType::Formatting);
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
            return Err(ConsoleExportErrorType::Write);
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::encode::encode_logs_otap_batch;
    use otap_df_pdata::proto::opentelemetry::{
        common::v1::{
            AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList, any_value,
        },
        logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use otap_df_pdata::testing::fixtures::logs_with_full_resource_and_scope;
    use otap_df_pdata::views::otap::OtapLogsView;
    use prost::Message;
    use serde_json::{Value, json};

    /// Format proto logs through the raw OTLP view and parse the resulting JSON lines.
    fn format_record_json(logs_data: &LogsData, formatter: &RecordJsonFormatter) -> Vec<Value> {
        let bytes = OtlpProtoBytes::ExportLogsRequest(logs_data.encode_to_vec().into());
        let logs_view = RawLogsData::try_from(&bytes).expect("logs");
        let mut output = Vec::new();
        formatter
            .format_logs_data_to(&logs_view, &mut output)
            .expect("format record JSON");
        parse_json_lines(&output)
    }

    /// Parse a complete NDJSON buffer into individual values.
    fn parse_json_lines(output: &[u8]) -> Vec<Value> {
        std::str::from_utf8(output)
            .expect("JSON output is UTF-8")
            .lines()
            .map(|line| serde_json::from_str(line).expect("line is valid JSON"))
            .collect()
    }

    /// Build an AnyValue containing the supplied protobuf oneof value.
    fn any_value(value: any_value::Value) -> AnyValue {
        AnyValue { value: Some(value) }
    }

    /// Build a KeyValue containing the supplied protobuf oneof value.
    fn attribute(key: &str, value: any_value::Value) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(any_value(value)),
        }
    }

    /// Scenario: the text formatter receives a fixture with multiple scopes and records.
    /// Guarantees: the existing hierarchical text output remains byte-for-byte unchanged.
    #[test]
    fn text_formatter_preserves_hierarchical_output() {
        let logs_data = logs_with_full_resource_and_scope();
        let bytes = OtlpProtoBytes::ExportLogsRequest(logs_data.encode_to_vec().into());
        let formatter = HierarchicalFormatter::new(false, true);

        let mut output = Vec::new();
        let logs_view = RawLogsData::try_from(&bytes).expect("logs");
        formatter.format_logs_data_to(&logs_view, &mut output);

        let text = String::from_utf8_lossy(&output);

        // The fixture creates two scopes with two logs each:
        // - scope-alpha/1.0.0: INFO + WARN
        // - scope-beta/2.0.0: ERROR + DEBUG
        let expected = "\
2025-01-15T10:30:00.000Z  RESOURCE   v1.Resource [res.id=self]
2025-01-15T10:30:00.000Z  \u{2502} SCOPE    scope-alpha/1.0.0 [scopekey=scopeval]
2025-01-15T10:30:00.000Z  \u{2502} \u{251C}\u{2500} INFO  event_1: first log in alpha
2025-01-15T10:30:01.000Z  \u{2502} \u{251C}\u{2500} WARN  second log in alpha
2025-01-15T10:30:02.000Z  \u{2502} SCOPE    scope-beta/2.0.0
2025-01-15T10:30:02.000Z  \u{2502} \u{251C}\u{2500} HOTHOT first log in beta
2025-01-15T10:30:03.000Z  \u{2502} \u{2514}\u{2500} DEBUG event_2: [detail=no body here]
";
        assert_eq!(text, expected);
    }

    /// Scenario: console configuration selects pretty output and every record JSON override.
    /// Guarantees: pretty and record JSON defaults remain stable and invalid selectors are rejected.
    #[test]
    fn console_config_supports_pretty_and_record_json_options() {
        let config: ConsoleExporterConfig =
            serde_json::from_value(json!({})).expect("default config");
        assert_eq!(config.format, ConsoleOutputFormat::Pretty);
        assert!(config.color);
        assert!(config.unicode);
        assert_eq!(
            config.record_json.timestamp_format,
            RecordJsonTimestampFormat::Rfc3339
        );
        assert_eq!(config.record_json.body_field, RecordJsonBodyField::Body);
        assert_eq!(
            config.record_json.int64_format,
            RecordJsonInt64Format::Number
        );
        assert!(!config.record_json.resource);
        assert!(config.record_json.scope);
        assert!(!config.record_json.otel);

        let config: ConsoleExporterConfig =
            serde_json::from_value(json!({"format": "pretty"})).expect("pretty config");
        assert_eq!(config.format, ConsoleOutputFormat::Pretty);

        let config: ConsoleExporterConfig = serde_json::from_value(json!({
            "format": "record_json",
            "color": false,
            "unicode": false,
            "record_json": {
                "timestamp_format": "unix_nano",
                "body_field": "message",
                "int64_format": "string",
                "resource": true,
                "scope": false,
                "otel": true
            }
        }))
        .expect("record JSON config");
        assert_eq!(config.format, ConsoleOutputFormat::RecordJson);
        assert_eq!(
            config.record_json.timestamp_format,
            RecordJsonTimestampFormat::UnixNano
        );
        assert_eq!(config.record_json.body_field, RecordJsonBodyField::Message);
        assert_eq!(
            config.record_json.int64_format,
            RecordJsonInt64Format::String
        );
        assert!(config.record_json.resource);
        assert!(!config.record_json.scope);
        assert!(config.record_json.otel);

        for unsupported in ["text", "json", "otlp_json", "logfmt"] {
            let result = serde_json::from_value::<ConsoleExporterConfig>(json!({
                "format": unsupported
            }));
            assert!(result.is_err(), "{unsupported} should be rejected");
        }
        for (field, unsupported) in [
            ("timestamp_format", "epoch"),
            ("body_field", "log"),
            ("int64_format", "float"),
        ] {
            let result = serde_json::from_value::<ConsoleExporterConfig>(json!({
                "format": "record_json",
                "record_json": {(field): unsupported}
            }));
            assert!(
                result.is_err(),
                "{field} value {unsupported} should be rejected"
            );
        }
    }

    /// Scenario: record JSON formats a hierarchy containing four log records.
    /// Guarantees: each record is one valid line with compact fields and default scope context.
    #[test]
    fn record_json_emits_one_line_per_record_with_default_scope() {
        let logs_data = logs_with_full_resource_and_scope();
        let formatter = RecordJsonFormatter::new(RecordJsonConfig::default());
        let values = format_record_json(&logs_data, &formatter);

        assert_eq!(values.len(), 4);
        assert!(values.iter().all(|value| value.get("resource").is_none()));
        assert_eq!(
            values[0],
            json!({
                "timestamp": "2025-01-15T10:30:00.000000000Z",
                "observed_timestamp": "2025-01-15T10:30:00.100000000Z",
                "severity_number": 9,
                "body": "first log in alpha",
                "event_name": "event_1",
                "attributes": {},
                "scope": {
                    "name": "scope-alpha",
                    "version": "1.0.0",
                    "attributes": {"scopekey": "scopeval"}
                }
            })
        );
        assert_eq!(values[2]["severity_text"], "HOTHOT");
        assert_eq!(values[3]["attributes"], json!({"detail": "no body here"}));
    }

    /// Scenario: record JSON changes inherited context and receives absent context views.
    /// Guarantees: enabled context has stable empty objects and disabled context is omitted.
    #[test]
    fn record_json_honors_context_controls_and_stable_empty_objects() {
        let logs_data = logs_with_full_resource_and_scope();
        let formatter = RecordJsonFormatter::new(RecordJsonConfig {
            resource: true,
            scope: false,
            ..RecordJsonConfig::default()
        });
        let values = format_record_json(&logs_data, &formatter);

        assert_eq!(values.len(), 4);
        for value in values {
            assert_eq!(value["resource"], json!({"res.id": "self"}));
            assert!(value.get("scope").is_none());
        }

        let empty_context = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord::default()],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let formatter = RecordJsonFormatter::new(RecordJsonConfig {
            resource: true,
            ..RecordJsonConfig::default()
        });
        assert_eq!(
            format_record_json(&empty_context, &formatter),
            vec![json!({
                "attributes": {},
                "resource": {},
                "scope": {"attributes": {}}
            })]
        );
    }

    /// Scenario: a log record contains every AnyValue variant and duplicate attributes.
    /// Guarantees: compact values, null handling, base64 bytes, and last-key-wins remain stable.
    #[test]
    fn record_json_uses_compact_value_encodings() {
        let attributes = vec![
            attribute("string", any_value::Value::StringValue("value".to_string())),
            attribute("bool", any_value::Value::BoolValue(true)),
            attribute("int", any_value::Value::IntValue(-7)),
            attribute("double", any_value::Value::DoubleValue(1.5)),
            attribute("nan", any_value::Value::DoubleValue(f64::NAN)),
            attribute(
                "positive_infinity",
                any_value::Value::DoubleValue(f64::INFINITY),
            ),
            attribute(
                "negative_infinity",
                any_value::Value::DoubleValue(f64::NEG_INFINITY),
            ),
            attribute("bytes", any_value::Value::BytesValue(vec![0, 1, 255])),
            attribute(
                "array",
                any_value::Value::ArrayValue(ArrayValue {
                    values: vec![
                        any_value(any_value::Value::BoolValue(false)),
                        any_value(any_value::Value::IntValue(9)),
                    ],
                }),
            ),
            attribute(
                "kvlist",
                any_value::Value::KvlistValue(KeyValueList {
                    values: vec![
                        attribute(
                            "nested",
                            any_value::Value::StringValue("inside".to_string()),
                        ),
                        attribute(
                            "duplicate",
                            any_value::Value::StringValue("first".to_string()),
                        ),
                        attribute(
                            "duplicate",
                            any_value::Value::StringValue("last".to_string()),
                        ),
                        attribute(
                            "removed",
                            any_value::Value::StringValue("present".to_string()),
                        ),
                        KeyValue {
                            key: "removed".to_string(),
                            value: None,
                        },
                    ],
                }),
            ),
            KeyValue {
                key: "empty".to_string(),
                value: Some(AnyValue { value: None }),
            },
            KeyValue {
                key: "missing".to_string(),
                value: None,
            },
            attribute(
                "duplicate",
                any_value::Value::StringValue("first".to_string()),
            ),
            attribute(
                "duplicate",
                any_value::Value::StringValue("last".to_string()),
            ),
            attribute(
                "removed",
                any_value::Value::StringValue("present".to_string()),
            ),
            KeyValue {
                key: "removed".to_string(),
                value: None,
            },
        ];
        let logs_data = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        time_unix_nano: 42,
                        observed_time_unix_nano: 43,
                        severity_number: SeverityNumber::Info as i32,
                        severity_text: "INFO".to_string(),
                        body: Some(any_value(any_value::Value::StringValue(
                            "message\ncontinued".to_string(),
                        ))),
                        attributes,
                        dropped_attributes_count: 2,
                        flags: 1,
                        trace_id: (0u8..16).collect(),
                        span_id: (16u8..24).collect(),
                        event_name: "event".to_string(),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let values = format_record_json(
            &logs_data,
            &RecordJsonFormatter::new(RecordJsonConfig::default()),
        );
        assert_eq!(values.len(), 1);
        let value = &values[0];
        assert_eq!(value["timestamp"], "1970-01-01T00:00:00.000000042Z");
        assert_eq!(
            value["observed_timestamp"],
            "1970-01-01T00:00:00.000000043Z"
        );
        assert_eq!(value["severity_number"], 9);
        assert_eq!(value["severity_text"], "INFO");
        assert_eq!(value["body"], "message\ncontinued");
        assert_eq!(value["trace_flags"], 1);
        assert_eq!(value["trace_id"], "000102030405060708090a0b0c0d0e0f");
        assert_eq!(value["span_id"], "1011121314151617");
        assert_eq!(value["event_name"], "event");
        assert!(value.get("otel").is_none());
        assert_eq!(
            value["attributes"],
            json!({
                "string": "value",
                "bool": true,
                "int": -7,
                "double": 1.5,
                "nan": null,
                "positive_infinity": null,
                "negative_infinity": null,
                "bytes": "AAH/",
                "array": [false, 9],
                "kvlist": {
                    "nested": "inside",
                    "duplicate": "last"
                },
                "empty": null,
                "duplicate": "last"
            })
        );
    }

    /// Scenario: record JSON selects Unix timestamps, message, string int64, resource, and OTel.
    /// Guarantees: every format option changes only its documented field representation.
    #[test]
    fn record_json_honors_all_format_options() {
        let logs_data = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource::default()),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        time_unix_nano: 42,
                        observed_time_unix_nano: 43,
                        severity_number: SeverityNumber::Info as i32,
                        body: Some(any_value(any_value::Value::IntValue(i64::MIN))),
                        attributes: vec![attribute(
                            "maximum",
                            any_value::Value::IntValue(i64::MAX),
                        )],
                        dropped_attributes_count: 2,
                        flags: 1,
                        ..LogRecord::default()
                    }],
                    schema_url: "https://opentelemetry.io/schemas/scope".to_string(),
                }],
                schema_url: "https://opentelemetry.io/schemas/resource".to_string(),
            }],
        };

        let default_value = &format_record_json(
            &logs_data,
            &RecordJsonFormatter::new(RecordJsonConfig::default()),
        )[0];
        assert_eq!(default_value["body"], json!(i64::MIN));
        assert_eq!(default_value["attributes"]["maximum"], json!(i64::MAX));
        assert!(default_value.get("message").is_none());

        let formatter = RecordJsonFormatter::new(RecordJsonConfig {
            timestamp_format: RecordJsonTimestampFormat::UnixNano,
            body_field: RecordJsonBodyField::Message,
            int64_format: RecordJsonInt64Format::String,
            resource: true,
            scope: false,
            otel: true,
        });
        let values = format_record_json(&logs_data, &formatter);
        assert_eq!(
            values,
            vec![json!({
                "timestamp": "42",
                "observed_timestamp": "43",
                "severity_number": 9,
                "message": i64::MIN.to_string(),
                "attributes": {"maximum": i64::MAX.to_string()},
                "resource": {},
                "trace_flags": 1,
                "otel": {
                    "dropped_attributes_count": 2,
                    "resource_schema_url": "https://opentelemetry.io/schemas/resource",
                    "scope_schema_url": "https://opentelemetry.io/schemas/scope"
                }
            })]
        );
        assert!(values[0].get("body").is_none());
        assert!(values[0].get("scope").is_none());
    }

    /// Scenario: equivalent logs are viewed from OTLP bytes and OTAP Arrow records.
    /// Guarantees: record JSON matches for record and attribute data preserved by both backends.
    #[test]
    fn record_json_matches_otlp_and_otap_views() {
        let logs_data = logs_with_full_resource_and_scope();
        let formatter = RecordJsonFormatter::new(RecordJsonConfig {
            resource: true,
            ..RecordJsonConfig::default()
        });
        let mut otlp_values = format_record_json(&logs_data, &formatter);

        // OTAP scope views do not currently expose scope name or version.
        for value in &mut otlp_values {
            let scope = value["scope"].as_object_mut().expect("scope object");
            _ = scope.remove("name");
            _ = scope.remove("version");
        }

        let otap_records = encode_logs_otap_batch(&logs_data).expect("encode OTAP logs");
        let otap_view = OtapLogsView::try_from(&otap_records).expect("OTAP logs view");
        let mut output = Vec::new();
        formatter
            .format_logs_data_to(&otap_view, &mut output)
            .expect("format OTAP record JSON");
        let mut otap_values = parse_json_lines(&output);

        // OTAP views currently represent an absent body as an empty AnyValue.
        for value in &mut otap_values {
            if value.get("body") == Some(&Value::Null) {
                _ = value.as_object_mut().expect("record object").remove("body");
            }
        }

        assert_eq!(otap_values, otlp_values);
    }
}
