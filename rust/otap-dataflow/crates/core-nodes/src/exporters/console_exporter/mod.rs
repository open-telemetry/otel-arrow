// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Console exporter that prints OTLP data in human-readable or structured formats.

mod metrics;
mod pretty_metrics;
mod pretty_writer;
otap_df_telemetry::otel_component_scope!(
    urn = CONSOLE_EXPORTER_URN,
    target = "otel.exporter.console",
);

mod record_json;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::engine::OtelDataflowSpec;
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
use otap_df_pdata::views::otap::{OtapLogsView, OtapMetricsView};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::views::otlp::bytes::metrics::RawMetricsData;
use otap_df_pdata::{OtapPayload, PayloadData};
use otap_df_pdata_views::views::common::InstrumentationScopeView;
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otap_df_pdata_views::views::metrics::MetricsView;
use otap_df_pdata_views::views::resource::ResourceView;
use otap_df_telemetry::output_service::{Frame, OutputService, StreamHandle};
use otap_df_telemetry::self_tracing::{AnsiCode, ColorMode, LOG_BUFFER_SIZE, StyledBufWriter};
use otap_df_telemetry_macros::AttributeEnum;
use std::io::Write;
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
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

/// Histogram detail levels supported by `pretty`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrettyHistogramMode {
    /// Render compact distribution statistics without bucket details.
    #[default]
    Compact,
    /// Render the complete histogram representation, including buckets.
    Raw,
}

/// Format-specific configuration for `pretty`.
#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
pub struct PrettyConfig {
    /// Histogram detail level (default: compact).
    #[serde(default)]
    pub histogram: PrettyHistogramMode,
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
    /// Format-specific options for `pretty`.
    #[serde(default)]
    pub pretty: PrettyConfig,
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
            pretty: PrettyConfig::default(),
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
    /// Overrides the resolved stream; only tests supply one.
    #[cfg(test)]
    output: Option<TestOutput>,
}

/// Test-only stream override that also counts how often it was resolved.
#[cfg(test)]
#[derive(Clone)]
struct TestOutput {
    handle: StreamHandle,
    resolutions: Arc<AtomicUsize>,
}

impl ConsoleExporter {
    /// Create a new console exporter with the given configuration.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext, config: ConsoleExporterConfig) -> Self {
        let metrics = ConsoleExporterMetrics::register(pipeline_ctx, config.format);
        let formatter = match config.format {
            ConsoleOutputFormat::Pretty => ConsoleFormatter::Pretty(HierarchicalFormatter::new(
                config.color,
                config.unicode,
                config.pretty.histogram,
            )),
            ConsoleOutputFormat::RecordJson => {
                ConsoleFormatter::RecordJson(RecordJsonFormatter::new(config.record_json))
            }
        };
        Self {
            formatter,
            metrics,
            #[cfg(test)]
            output: None,
        }
    }

    fn terminal_state(&mut self, deadline: Instant) -> TerminalState {
        TerminalState::new(deadline, self.metrics.terminal_snapshots())
    }

    /// Creates an exporter bound to a caller-supplied output stream.
    #[cfg(test)]
    #[must_use]
    fn with_output(
        pipeline_ctx: &PipelineContext,
        config: ConsoleExporterConfig,
        output: StreamHandle,
    ) -> Self {
        Self::with_counted_output(pipeline_ctx, config, output).0
    }

    /// Creates an exporter bound to a test stream, returning the resolution counter.
    #[cfg(test)]
    #[must_use]
    fn with_counted_output(
        pipeline_ctx: &PipelineContext,
        config: ConsoleExporterConfig,
        output: StreamHandle,
    ) -> (Self, Arc<AtomicUsize>) {
        let resolutions = Arc::new(AtomicUsize::new(0));
        let exporter = Self {
            output: Some(TestOutput {
                handle: output,
                resolutions: Arc::clone(&resolutions),
            }),
            ..Self::new(pipeline_ctx, config)
        };
        (exporter, resolutions)
    }

    /// Returns the stream this exporter writes to.
    ///
    /// Pretty output is this exporter's product rather than engine prose, so it
    /// uses stdout until another exporter claims stdout for machine-readable
    /// records.
    fn output_handle(&self) -> StreamHandle {
        #[cfg(test)]
        if let Some(output) = self.output.as_ref() {
            let _ = output.resolutions.fetch_add(1, Ordering::Relaxed);
            return output.handle.clone();
        }
        match self.formatter {
            ConsoleFormatter::Pretty(_) if OutputService::structured_stdout() => {
                OutputService::stderr()
            }
            ConsoleFormatter::Pretty(_) | ConsoleFormatter::RecordJson(_) => {
                OutputService::stdout()
            }
        }
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
        require_structured_stdout_claim(config.format, OutputService::structured_stdout())?;
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

/// Claims stdout for records when `engine_cfg` deploys a `record_json` console exporter.
///
/// Pipelines are built on their own threads, so an exporter created late cannot
/// stop an earlier `pretty` exporter from having already written to stdout. The
/// process host calls this once a configuration is accepted and before any
/// pipeline starts, which keeps `pretty` output on stderr from its first
/// payload. Validation stays free of side effects so a rejected candidate
/// configuration never reroutes a running exporter.
pub fn claim_structured_stdout(engine_cfg: &OtelDataflowSpec) {
    if config_emits_records(engine_cfg) {
        OutputService::mark_structured_stdout();
    }
}

/// Returns true when any console exporter in `engine_cfg` emits record JSON.
///
/// The engine's own observability pipeline carries a console exporter too, so it
/// is scanned alongside the configured groups.
fn config_emits_records(engine_cfg: &OtelDataflowSpec) -> bool {
    let observability = engine_cfg
        .engine
        .observability
        .pipeline
        .clone()
        .into_pipeline_config();
    engine_cfg
        .groups
        .values()
        .flat_map(|group| group.pipelines.values())
        .chain(std::iter::once(&observability))
        .flat_map(|pipeline| pipeline.node_iter())
        .any(|(_, node)| {
            node.r#type.as_str() == CONSOLE_EXPORTER_URN
                && match serde_json::from_value::<ConsoleExporterConfig>(node.config.clone()) {
                    Ok(config) => config.format == ConsoleOutputFormat::RecordJson,
                    // The host normally validates first. An embedder that does not
                    // gets the safe failure direction: prose moves off stdout.
                    Err(_) => true,
                }
        })
}

/// Rejects a record exporter that was not claimed before pipelines started.
fn require_structured_stdout_claim(
    format: ConsoleOutputFormat,
    structured_stdout: bool,
) -> Result<(), ConfigError> {
    if format == ConsoleOutputFormat::RecordJson && !structured_stdout {
        return Err(ConfigError::InvalidUserConfig {
            error: "record_json console output was not claimed before pipeline startup; call claim_structured_stdout on the accepted engine configuration".to_owned(),
        });
    }
    Ok(())
}

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
                    // Resolved per payload: another pipeline can claim stdout for
                    // records after this exporter has already started.
                    let output = self.output_handle();
                    let export_start = Instant::now();
                    let signal = data.signal_type();
                    match self.export(data.payload_ref(), &output).await {
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
    async fn export(
        &self,
        payload: &OtapPayload,
        output: &StreamHandle,
    ) -> Result<(), ConsoleExportErrorType> {
        match payload.signal_type() {
            SignalType::Logs => self.export_logs(payload, output).await,
            SignalType::Traces => self.unsupported_signal("traces"),
            SignalType::Metrics => self.export_metrics(payload, output).await,
        }
    }

    async fn export_logs(
        &self,
        payload: &OtapPayload,
        output: &StreamHandle,
    ) -> Result<(), ConsoleExportErrorType> {
        match payload.data() {
            PayloadData::OtlpBytes(bytes) => match RawLogsData::try_from(bytes) {
                Ok(logs_view) => self.formatter.print_logs_data(&logs_view, output).await,
                Err(e) => {
                    otel_error!("console.logs_view.otlp_create_failed", error = ?e, message = "Failed to create OTLP logs view");
                    Err(ConsoleExportErrorType::OtlpViewCreation)
                }
            },
            PayloadData::OtapArrowRecords(records) => match OtapLogsView::try_from(records) {
                Ok(logs_view) => self.formatter.print_logs_data(&logs_view, output).await,
                Err(e) => {
                    otel_error!("console.logs_view.otap_create_failed", error = ?e, message = "Failed to create OTAP logs view");
                    Err(ConsoleExportErrorType::OtapViewCreation)
                }
            },
        }
    }

    async fn export_metrics(
        &self,
        payload: &OtapPayload,
        output: &StreamHandle,
    ) -> Result<(), ConsoleExportErrorType> {
        if !self.formatter.supports_metrics() {
            return self.unsupported_signal("metrics");
        }

        match payload.data() {
            PayloadData::OtlpBytes(bytes) => {
                let metrics_bytes = match bytes {
                    otap_df_pdata::OtlpProtoBytes::ExportMetricsRequest(bytes) => bytes,
                    _ => unreachable!("metrics payload must contain metrics OTLP bytes"),
                };
                match RawMetricsData::try_new(metrics_bytes) {
                    Ok(metrics_view) => {
                        self.formatter
                            .print_metrics_data(&metrics_view, output)
                            .await
                    }
                    Err(e) => {
                        otel_warn!("console.metrics_view.otlp_create_failed", error = ?e);
                        Err(ConsoleExportErrorType::OtlpViewCreation)
                    }
                }
            }
            PayloadData::OtapArrowRecords(records) => match OtapMetricsView::try_from(records) {
                Ok(metrics_view) => {
                    self.formatter
                        .print_metrics_data(&metrics_view, output)
                        .await
                }
                Err(e) => {
                    otel_warn!("console.metrics_view.otap_create_failed", error = ?e);
                    Err(ConsoleExportErrorType::OtapViewCreation)
                }
            },
        }
    }

    fn unsupported_signal(&self, signal: &'static str) -> Result<(), ConsoleExportErrorType> {
        let message = match (signal, &self.formatter) {
            ("metrics", ConsoleFormatter::RecordJson(_)) => {
                "Console exporter record_json format supports logs only; select pretty to inspect metrics"
            }
            _ => {
                "Console exporter does not support this signal in the selected format; use processor:debug followed by exporter:noop to inspect it"
            }
        };
        otel_warn!(
            "console.message.unsupported_signal",
            signal = signal,
            message = message
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
    const fn supports_metrics(&self) -> bool {
        matches!(self, Self::Pretty(_))
    }

    /// Format logs and hand the complete payload to the process-wide writer.
    async fn print_logs_data<L: LogsDataView>(
        &self,
        logs_data: &L,
        output: &StreamHandle,
    ) -> Result<(), ConsoleExportErrorType> {
        let mut buffer = Vec::new();
        let format_result = match self {
            Self::Pretty(formatter) => {
                formatter.format_logs_data_to(logs_data, &mut buffer);
                Ok(())
            }
            Self::RecordJson(formatter) => formatter.format_logs_data_to(logs_data, &mut buffer),
        };

        if let Err(err) = format_result {
            otel_error!(
                "console.format_failed",
                error = ?err,
                message = "Could not format console output"
            );
            return Err(ConsoleExportErrorType::Formatting);
        }

        // One frame per payload: the writer holds the stdout lock for the whole
        // buffer, so concurrent exporters can never split a record.
        let frame = match self {
            Self::Pretty(_) => Frame::new(buffer),
            Self::RecordJson(_) => Frame::new_record_json(buffer),
        };
        if let Err(err) = output.submit(frame).await {
            otel_error!("console.write_failed", error = ?err, message = "Could not write to console");
            return Err(ConsoleExportErrorType::Write);
        }

        Ok(())
    }

    /// Format metrics and hand the complete payload to the process-wide writer.
    async fn print_metrics_data<M: MetricsView>(
        &self,
        metrics_data: &M,
        output: &StreamHandle,
    ) -> Result<(), ConsoleExportErrorType> {
        let mut buffer = Vec::new();
        let format_result = match self {
            Self::Pretty(formatter) => formatter.format_metrics_data_to(metrics_data, &mut buffer),
            Self::RecordJson(_) => {
                unreachable!("record_json metrics are rejected before formatting")
            }
        };

        if let Err(err) = format_result {
            otel_error!(
                "console.format_failed",
                error = ?err,
                message = "Could not format console output"
            );
            return Err(ConsoleExportErrorType::Formatting);
        }

        if let Err(err) = output.submit(Frame::new(buffer)).await {
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
    histogram_mode: PrettyHistogramMode,
}

impl HierarchicalFormatter {
    /// Create a new hierarchical formatter.
    #[must_use]
    pub const fn new(
        use_color: bool,
        use_unicode: bool,
        histogram_mode: PrettyHistogramMode,
    ) -> Self {
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
            histogram_mode,
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
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::Interests;
    use otap_df_engine::control::PipelineCompletionMsg;
    use otap_df_engine::testing::exporter::{
        TestRuntime, create_exporter_from_factory, create_test_pipeline_context,
    };
    use otap_df_engine::testing::test_node;
    use otap_df_otap::testing::{TestCallData, create_test_pdata, next_ack};
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::encode::{encode_logs_otap_batch, encode_metrics_otap_batch};
    use otap_df_pdata::proto::opentelemetry::{
        common::v1::{
            AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList, any_value,
        },
        logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
        metrics::v1::{
            AggregationTemporality as ProtoAggregationTemporality, Exemplar, ExponentialHistogram,
            ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint, Metric,
            MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary,
            SummaryDataPoint, exemplar, exponential_histogram_data_point, metric,
            number_data_point, summary_data_point,
        },
        resource::v1::Resource,
    };
    use otap_df_pdata::testing::fixtures::logs_with_full_resource_and_scope;
    use otap_df_pdata::views::otap::OtapLogsView;
    use otap_df_telemetry::output_service::{
        DEFAULT_STDOUT_BYTE_CAPACITY, OutputSink, OutputStream, StreamId,
    };
    use prost::Message;
    use serde_json::{Value, json};
    use std::time::Instant;

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

    fn metrics_with_all_data_types() -> MetricsData {
        let point_attribute = || {
            vec![attribute(
                "series",
                any_value::Value::StringValue("blue".to_string()),
            )]
        };
        let exemplar = || Exemplar {
            filtered_attributes: vec![attribute("sampled", any_value::Value::BoolValue(true))],
            time_unix_nano: 150,
            span_id: (1u8..=8).collect(),
            trace_id: (1u8..=16).collect(),
            value: Some(exemplar::Value::AsDouble(1.25)),
        };
        let number_point = |value| NumberDataPoint {
            attributes: point_attribute(),
            start_time_unix_nano: 100,
            time_unix_nano: 200,
            exemplars: vec![exemplar()],
            flags: 1,
            value: Some(value),
        };

        MetricsData {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![attribute(
                        "service.name",
                        any_value::Value::StringValue("metrics-test".to_string()),
                    )],
                    ..Resource::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "metrics-scope".to_string(),
                        version: "1.0.0".to_string(),
                        attributes: vec![attribute("scope.attr", any_value::Value::IntValue(7))],
                        dropped_attributes_count: 0,
                    }),
                    metrics: vec![
                        Metric {
                            name: "temperature".to_string(),
                            description: "Current temperature".to_string(),
                            unit: "Cel".to_string(),
                            metadata: vec![attribute(
                                "metadata.attr",
                                any_value::Value::StringValue("value".to_string()),
                            )],
                            data: Some(metric::Data::Gauge(Gauge {
                                data_points: vec![number_point(
                                    number_data_point::Value::AsDouble(21.5),
                                )],
                            })),
                        },
                        Metric {
                            name: "requests".to_string(),
                            description: "Request count".to_string(),
                            unit: "{request}".to_string(),
                            metadata: vec![],
                            data: Some(metric::Data::Sum(Sum {
                                data_points: vec![number_point(number_data_point::Value::AsInt(
                                    42,
                                ))],
                                aggregation_temporality: ProtoAggregationTemporality::Cumulative
                                    as i32,
                                is_monotonic: true,
                            })),
                        },
                        Metric {
                            name: "latency".to_string(),
                            description: String::new(),
                            unit: "ms".to_string(),
                            metadata: vec![],
                            data: Some(metric::Data::Histogram(Histogram {
                                data_points: vec![HistogramDataPoint {
                                    attributes: point_attribute(),
                                    start_time_unix_nano: 100,
                                    time_unix_nano: 200,
                                    count: 4,
                                    sum: Some(20.0),
                                    bucket_counts: vec![1, 2, 1],
                                    explicit_bounds: vec![1.0, 10.0],
                                    exemplars: vec![exemplar()],
                                    flags: 1,
                                    min: Some(0.5),
                                    max: Some(12.0),
                                }],
                                aggregation_temporality: ProtoAggregationTemporality::Delta as i32,
                            })),
                        },
                        Metric {
                            name: "size_distribution".to_string(),
                            description: String::new(),
                            unit: "By".to_string(),
                            metadata: vec![],
                            data: Some(metric::Data::ExponentialHistogram(ExponentialHistogram {
                                data_points: vec![ExponentialHistogramDataPoint {
                                    attributes: point_attribute(),
                                    start_time_unix_nano: 100,
                                    time_unix_nano: 200,
                                    count: 6,
                                    sum: Some(30.0),
                                    scale: -1,
                                    zero_count: 1,
                                    positive: Some(exponential_histogram_data_point::Buckets {
                                        offset: 2,
                                        bucket_counts: vec![2, 1],
                                    }),
                                    negative: Some(exponential_histogram_data_point::Buckets {
                                        offset: -2,
                                        bucket_counts: vec![1, 1],
                                    }),
                                    flags: 1,
                                    exemplars: vec![exemplar()],
                                    min: Some(-4.0),
                                    max: Some(16.0),
                                    zero_threshold: 0.01,
                                }],
                                aggregation_temporality: ProtoAggregationTemporality::Cumulative
                                    as i32,
                            })),
                        },
                        Metric {
                            name: "request_summary".to_string(),
                            description: String::new(),
                            unit: "ms".to_string(),
                            metadata: vec![],
                            data: Some(metric::Data::Summary(Summary {
                                data_points: vec![SummaryDataPoint {
                                    attributes: point_attribute(),
                                    start_time_unix_nano: 100,
                                    time_unix_nano: 200,
                                    count: 5,
                                    sum: 25.0,
                                    quantile_values: vec![
                                        summary_data_point::ValueAtQuantile {
                                            quantile: 0.5,
                                            value: 4.0,
                                        },
                                        summary_data_point::ValueAtQuantile {
                                            quantile: 0.99,
                                            value: 9.0,
                                        },
                                    ],
                                    flags: 1,
                                }],
                            })),
                        },
                    ],
                    schema_url: "https://opentelemetry.io/schemas/scope".to_string(),
                }],
                schema_url: "https://opentelemetry.io/schemas/resource".to_string(),
            }],
        }
    }

    fn format_pretty_metrics_with_histogram_mode(
        metrics_data: &MetricsData,
        histogram_mode: PrettyHistogramMode,
    ) -> String {
        let bytes = metrics_data.encode_to_vec();
        let metrics_view = RawMetricsData::try_new(&bytes).expect("metrics");
        let formatter = HierarchicalFormatter::new(false, false, histogram_mode);
        let mut output = Vec::new();
        formatter
            .format_metrics_data_to(&metrics_view, &mut output)
            .expect("format pretty metrics");
        String::from_utf8(output).expect("pretty output is UTF-8")
    }

    fn format_pretty_metrics(metrics_data: &MetricsData) -> String {
        format_pretty_metrics_with_histogram_mode(metrics_data, PrettyHistogramMode::Compact)
    }

    /// Scenario: the text formatter receives a fixture with multiple scopes and records.
    /// Guarantees: the existing hierarchical text output remains byte-for-byte unchanged.
    #[test]
    fn text_formatter_preserves_hierarchical_output() {
        let logs_data = logs_with_full_resource_and_scope();
        let bytes = OtlpProtoBytes::ExportLogsRequest(logs_data.encode_to_vec().into());
        let formatter = HierarchicalFormatter::new(false, true, PrettyHistogramMode::Compact);

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

    /// Scenario: Pretty output receives one batch containing every OTLP metric data type.
    /// Guarantees: The compact metrics hierarchy, field ordering, and tree prefixes remain
    /// byte-for-byte stable while raw histogram details are omitted.
    #[test]
    fn pretty_metrics_render_all_data_types_and_semantics() {
        let text = format_pretty_metrics(&metrics_with_all_data_types());

        let expected = concat!(
            "RESOURCE schema_url=https://opentelemetry.io/schemas/resource [service.name=metrics-test]\n",
            "| +- SCOPE name=metrics-scope version=1.0.0 schema_url=https://opentelemetry.io/schemas/scope [scope.attr=7]\n",
            "| | +- METRIC name=temperature description=Current temperature unit=Cel [metadata.attr=value]\n",
            "| | | +- GAUGE\n",
            "| | | | +- DATA_POINT start_time_unix_nano=100 time_unix_nano=200 value_double=21.5 flags=1 [series=blue]\n",
            "| | | | | +- EXEMPLAR time_unix_nano=150 value_double=1.25 span_id=0102030405060708 trace_id=0102030405060708090a0b0c0d0e0f10 [sampled=true]\n",
            "| | +- METRIC name=requests description=Request count unit={request}\n",
            "| | | +- SUM temporality=cumulative monotonic=true\n",
            "| | | | +- DATA_POINT start_time_unix_nano=100 time_unix_nano=200 value_int=42 flags=1 [series=blue]\n",
            "| | | | | +- EXEMPLAR time_unix_nano=150 value_double=1.25 span_id=0102030405060708 trace_id=0102030405060708090a0b0c0d0e0f10 [sampled=true]\n",
            "| | +- METRIC name=latency unit=ms\n",
            "| | | +- HISTOGRAM temporality=delta\n",
            "| | | | +- DATA_POINT start_time_unix_nano=100 time_unix_nano=200 count=4 sum=20 avg=5 min=0.5 max=12 flags=1 [series=blue]\n",
            "| | | | | +- EXEMPLAR time_unix_nano=150 value_double=1.25 span_id=0102030405060708 trace_id=0102030405060708090a0b0c0d0e0f10 [sampled=true]\n",
            "| | +- METRIC name=size_distribution unit=By\n",
            "| | | +- EXPONENTIAL_HISTOGRAM temporality=cumulative\n",
            "| | | | +- DATA_POINT start_time_unix_nano=100 time_unix_nano=200 count=6 sum=30 avg=5 min=-4 max=16 flags=1 [series=blue]\n",
            "| | | | | +- EXEMPLAR time_unix_nano=150 value_double=1.25 span_id=0102030405060708 trace_id=0102030405060708090a0b0c0d0e0f10 [sampled=true]\n",
            "| | +- METRIC name=request_summary unit=ms\n",
            "| | | +- SUMMARY\n",
            "| | | | +- DATA_POINT start_time_unix_nano=100 time_unix_nano=200 count=5 sum=25 flags=1 [series=blue]\n",
            "| | | | | +- QUANTILE quantile=0.5 value=4\n",
            "| | | | | +- QUANTILE quantile=0.99 value=9\n",
        );
        assert_eq!(text, expected);
    }

    /// Scenario: Compact histograms have no usable sum or contain no observations.
    /// Guarantees: Average is omitted instead of emitting a fabricated or undefined value.
    #[test]
    fn pretty_metrics_compact_histograms_omit_unavailable_average() {
        let mut metrics_data = metrics_with_all_data_types();
        let metrics = &mut metrics_data.resource_metrics[0].scope_metrics[0].metrics;
        let Some(metric::Data::Histogram(histogram)) = metrics[2].data.as_mut() else {
            panic!("expected explicit histogram");
        };
        histogram.data_points[0].count = 0;
        histogram.data_points[0].sum = Some(0.0);
        let Some(metric::Data::ExponentialHistogram(histogram)) = metrics[3].data.as_mut() else {
            panic!("expected exponential histogram");
        };
        histogram.data_points[0].sum = None;

        let text = format_pretty_metrics(&metrics_data);

        assert!(!text.contains(" avg="));
    }

    /// Scenario: Pretty metrics select raw histogram rendering.
    /// Guarantees: Exact explicit and exponential bucket details remain available on request.
    #[test]
    fn pretty_metrics_raw_histograms_preserve_bucket_details() {
        let text = format_pretty_metrics_with_histogram_mode(
            &metrics_with_all_data_types(),
            PrettyHistogramMode::Raw,
        );

        assert!(text.contains(
            "count=4 sum=20 min=0.5 max=12 flags=1 [series=blue]\n\
             | | | | | +- EXPLICIT_BOUND index=0 value=1\n\
             | | | | | +- EXPLICIT_BOUND index=1 value=10\n\
             | | | | | +- BUCKET_COUNT index=0 count=1\n\
             | | | | | +- BUCKET_COUNT index=1 count=2\n\
             | | | | | +- BUCKET_COUNT index=2 count=1\n"
        ));
        assert!(text.contains(
            "count=6 scale=-1 zero_count=1 zero_threshold=0.01 sum=30 min=-4 max=16 flags=1 [series=blue]\n\
             | | | | | +- POS_BUCKET offset=2 bucket_index=2 count=2\n\
             | | | | | +- POS_BUCKET offset=2 bucket_index=3 count=1\n\
             | | | | | +- NEG_BUCKET offset=-2 bucket_index=-2 count=1\n\
             | | | | | +- NEG_BUCKET offset=-2 bucket_index=-1 count=1\n"
        ));
    }

    /// Scenario: Equivalent metrics use compact and raw formatting through both payload models.
    /// Guarantees: OTLP bytes and OTAP Arrow records render identically in each histogram mode.
    #[test]
    fn pretty_metrics_support_otlp_and_otap_views() {
        let metrics_data = metrics_with_all_data_types();
        let records = encode_metrics_otap_batch(&metrics_data).expect("encode OTAP metrics");
        let otap_view = OtapMetricsView::try_from(&records).expect("OTAP metrics view");

        for histogram_mode in [PrettyHistogramMode::Compact, PrettyHistogramMode::Raw] {
            let otlp_text =
                format_pretty_metrics_with_histogram_mode(&metrics_data, histogram_mode);
            let formatter = HierarchicalFormatter::new(false, false, histogram_mode);
            let mut output = Vec::new();
            formatter
                .format_metrics_data_to(&otap_view, &mut output)
                .expect("format OTAP metrics");
            let otap_text = String::from_utf8(output).expect("pretty output is UTF-8");

            assert_eq!(otap_text, otlp_text);
        }
    }

    /// Scenario: A metrics field exceeds the fixed line capacity used by internal telemetry.
    /// Guarantees: Data-plane pretty output preserves the complete field without truncation.
    #[test]
    fn pretty_metrics_preserve_long_lines() {
        let mut metrics_data = metrics_with_all_data_types();
        let long_name = "x".repeat(LOG_BUFFER_SIZE * 2);
        metrics_data.resource_metrics[0].scope_metrics[0].metrics[0].name = long_name.clone();

        let bytes = metrics_data.encode_to_vec();
        let metrics_view = RawMetricsData::try_new(&bytes).expect("metrics");
        let formatter = HierarchicalFormatter::new(true, false, PrettyHistogramMode::Compact);
        let mut output = Vec::new();
        formatter
            .format_metrics_data_to(&metrics_view, &mut output)
            .expect("format long metrics line");
        let text = String::from_utf8(output).expect("UTF-8");
        let metric_line = text
            .lines()
            .find(|line| line.contains(&long_name))
            .expect("metric line");

        assert!(metric_line.len() > LOG_BUFFER_SIZE);
        assert!(metric_line.contains("\x1b[32mMETRIC\x1b[0m"));
        assert!(metric_line.contains(&format!("name={long_name}")));
        assert!(metric_line.ends_with("[metadata.attr=value]"));
    }

    /// Scenario: The destination writer rejects pretty metrics output.
    /// Guarantees: The formatter returns the destination error instead of reporting success.
    #[test]
    fn pretty_metrics_propagate_output_errors() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let metrics_data = metrics_with_all_data_types();
        let bytes = metrics_data.encode_to_vec();
        let metrics_view = RawMetricsData::try_new(&bytes).expect("metrics");
        let formatter = HierarchicalFormatter::new(false, false, PrettyHistogramMode::Compact);

        let err = formatter
            .format_metrics_data_to(&metrics_view, &mut FailingWriter)
            .expect_err("writer failure must propagate");

        assert_eq!(err.kind(), std::io::ErrorKind::BrokenPipe);
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
        assert_eq!(config.pretty.histogram, PrettyHistogramMode::Compact);
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

        let config: ConsoleExporterConfig = serde_json::from_value(json!({
            "format": "pretty",
            "pretty": {
                "histogram": "raw"
            }
        }))
        .expect("pretty config");
        assert_eq!(config.format, ConsoleOutputFormat::Pretty);
        assert_eq!(config.pretty.histogram, PrettyHistogramMode::Raw);
        assert!(
            ConsoleFormatter::Pretty(HierarchicalFormatter::new(
                false,
                false,
                config.pretty.histogram
            ))
            .supports_metrics()
        );

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
        assert!(
            !ConsoleFormatter::RecordJson(RecordJsonFormatter::new(config.record_json))
                .supports_metrics()
        );

        for unsupported in ["text", "json", "otlp_json", "logfmt"] {
            let result = serde_json::from_value::<ConsoleExporterConfig>(json!({
                "format": unsupported
            }));
            assert!(result.is_err(), "{unsupported} should be rejected");
        }
        let result = serde_json::from_value::<ConsoleExporterConfig>(json!({
            "format": "pretty",
            "pretty": {
                "histogram": "summary"
            }
        }));
        assert!(
            result.is_err(),
            "unsupported histogram mode should be rejected"
        );
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

    /// Sink that records the frames written by a test-owned writer thread.
    #[derive(Clone)]
    struct RecordingSink {
        buffer: Arc<std::sync::Mutex<Vec<u8>>>,
    }

    impl RecordingSink {
        fn new() -> Self {
            Self {
                buffer: Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn contents(&self) -> Vec<u8> {
            self.buffer.lock().expect("sink buffer").clone()
        }
    }

    impl OutputSink for RecordingSink {
        fn write_frame(&mut self, frame: &[u8]) -> std::io::Result<()> {
            let mut buffer = self.buffer.lock().expect("sink buffer");
            // Chunked appends mimic an operating system that accepts partial writes.
            for chunk in frame.chunks(4096) {
                buffer.extend_from_slice(chunk);
                std::thread::yield_now();
            }
            Ok(())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Scenario: several console exporters concurrently emit record JSON through one writer.
    /// Guarantees: every emitted line remains an independently parseable JSON record,
    /// so concurrent producers never interleave bytes inside a record.
    #[test]
    fn record_json_lines_stay_parseable_under_concurrent_exporters() {
        // The fixture carries four log records per payload.
        const RECORDS_PER_PAYLOAD: usize = 4;
        const EXPORTERS: usize = 4;
        const PAYLOADS_PER_EXPORTER: usize = 25;

        let encoded = logs_with_full_resource_and_scope().encode_to_vec();
        let sink = RecordingSink::new();
        let stream = OutputStream::start(
            StreamId::Stdout,
            8,
            DEFAULT_STDOUT_BYTE_CAPACITY,
            true,
            Box::new(sink.clone()),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();

        let workers: Vec<_> = (0..EXPORTERS)
            .map(|_| {
                let handle = handle.clone();
                let encoded = encoded.clone();
                std::thread::spawn(move || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .build()
                        .expect("current-thread runtime");
                    let formatter = ConsoleFormatter::RecordJson(RecordJsonFormatter::new(
                        RecordJsonConfig::default(),
                    ));
                    let bytes = OtlpProtoBytes::ExportLogsRequest(encoded.into());
                    runtime.block_on(async {
                        for _ in 0..PAYLOADS_PER_EXPORTER {
                            let logs_view = RawLogsData::try_from(&bytes).expect("logs");
                            formatter
                                .print_logs_data(&logs_view, &handle)
                                .await
                                .expect("frame is accepted");
                        }
                    });
                })
            })
            .collect();
        for worker in workers {
            worker.join().expect("exporter thread finishes");
        }

        let outcome = stream.shutdown(Duration::from_secs(30));
        assert!(outcome.drained);

        let records = parse_json_lines(&sink.contents());
        assert_eq!(
            records.len(),
            EXPORTERS * PAYLOADS_PER_EXPORTER * RECORDS_PER_PAYLOAD
        );
    }

    /// Scenario: the console writer is already closed when the exporter formats a payload.
    /// Guarantees: the handoff resolves as a counted enqueue failure instead of
    /// blocking, which is what lets the exporter's message loop continue to its ACK.
    #[tokio::test]
    async fn console_handoff_resolves_when_the_writer_is_gone() {
        let encoded = logs_with_full_resource_and_scope().encode_to_vec();
        let bytes = OtlpProtoBytes::ExportLogsRequest(encoded.into());
        let sink = RecordingSink::new();
        let stream = OutputStream::start(
            StreamId::Stdout,
            1,
            DEFAULT_STDOUT_BYTE_CAPACITY,
            true,
            Box::new(sink.clone()),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();
        assert!(stream.shutdown(Duration::from_secs(5)).drained);

        let formatter =
            ConsoleFormatter::RecordJson(RecordJsonFormatter::new(RecordJsonConfig::default()));
        let logs_view = RawLogsData::try_from(&bytes).expect("logs");
        let handoff = formatter.print_logs_data(&logs_view, &handle).await;
        assert!(
            handoff.is_err(),
            "a closed writer must report the failed handoff"
        );

        assert!(sink.contents().is_empty());
        assert_eq!(handle.stats().frames_enqueue_failed, 1);
    }

    /// Scenario: a payload reaches the exporter's message loop while the console writer
    /// is already closed, so the handoff fails.
    /// Guarantees: the exporter still ACKs that message exactly once and terminates
    /// cleanly, so a dead console writer never strands the upstream pipeline.
    #[test]
    fn enqueue_failure_still_acks_exactly_once() {
        const SUBSCRIBER_NODE_ID: usize = 4242;

        let sink = RecordingSink::new();
        let stream = OutputStream::start(
            StreamId::Stdout,
            1,
            DEFAULT_STDOUT_BYTE_CAPACITY,
            true,
            Box::new(sink.clone()),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();
        assert!(stream.shutdown(Duration::from_secs(5)).drained);

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(CONSOLE_EXPORTER_URN));
        let exporter = ExporterWrapper::local(
            ConsoleExporter::with_output(
                &create_test_pipeline_context(),
                ConsoleExporterConfig {
                    format: ConsoleOutputFormat::RecordJson,
                    ..ConsoleExporterConfig::default()
                },
                handle.clone(),
            ),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let pdata = create_test_pdata().test_subscribe_to(
                    Interests::ACKS,
                    TestCallData::default().into(),
                    SUBSCRIBER_NODE_ID,
                );
                ctx.send_pdata(pdata)
                    .await
                    .expect("exporter accepts the payload");
                ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test complete")
                    .await
                    .expect("exporter accepts shutdown");
            })
            .run_validation(move |mut ctx, result| async move {
                result.expect("exporter terminates cleanly after a failed console handoff");

                let mut completion_rx = ctx
                    .take_pipeline_completion_receiver()
                    .expect("pipeline completion receiver");
                match completion_rx.recv().await {
                    Ok(PipelineCompletionMsg::DeliverAck { ack }) => {
                        let (node_id, _) = next_ack(ack).expect("an ACKS subscriber");
                        assert_eq!(node_id, SUBSCRIBER_NODE_ID);
                    }
                    other => panic!("expected exactly one ACK, got {other:?}"),
                }
                assert!(
                    completion_rx.recv().await.is_err(),
                    "the failed handoff must not produce a second completion message"
                );

                assert_eq!(handle.stats().frames_enqueue_failed, 1);
                assert!(sink.contents().is_empty());
            });
    }

    /// Scenario: an exporter's message loop handles several payloads in one run.
    /// Guarantees: the target stream is resolved once per payload, so a stdout claim
    /// that lands mid-run is honored instead of being fixed when the loop starts.
    #[test]
    fn output_stream_is_resolved_for_every_payload() {
        const PAYLOADS: usize = 3;

        let sink = RecordingSink::new();
        let stream = OutputStream::start(
            StreamId::Stdout,
            8,
            DEFAULT_STDOUT_BYTE_CAPACITY,
            true,
            Box::new(sink.clone()),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();

        let (exporter, resolutions) = ConsoleExporter::with_counted_output(
            &create_test_pipeline_context(),
            ConsoleExporterConfig {
                format: ConsoleOutputFormat::RecordJson,
                ..ConsoleExporterConfig::default()
            },
            handle,
        );
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(CONSOLE_EXPORTER_URN));
        let wrapper = ExporterWrapper::local(
            exporter,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let observed = Arc::clone(&resolutions);
        test_runtime
            .set_exporter(wrapper)
            .run_test(|ctx| async move {
                for _ in 0..PAYLOADS {
                    ctx.send_pdata(create_test_pdata())
                        .await
                        .expect("exporter accepts the payload");
                }
                ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test complete")
                    .await
                    .expect("exporter accepts shutdown");
            })
            .run_validation(move |_ctx, result| async move {
                result.expect("exporter terminates cleanly");
                assert_eq!(
                    observed.load(Ordering::Relaxed),
                    PAYLOADS,
                    "the stream must be resolved once per payload, not once per run"
                );
            });
    }

    /// Scenario: an accepted or malformed configuration is scanned for record exporters.
    /// Guarantees: record JSON and malformed console configs claim stdout conservatively,
    /// while a configuration whose only console exporter is pretty does not.
    #[test]
    fn config_scan_claims_only_safe_cases() {
        let records = engine_config_with_console(r#"{ "format": "record_json" }"#);
        let pretty_only = engine_config_with_console(r#"{ "format": "pretty" }"#);
        let malformed = engine_config_with_console(r#"{ "format": "invalid" }"#);

        assert!(config_emits_records(&records));
        assert!(!config_emits_records(&pretty_only));
        assert!(config_emits_records(&malformed));
    }

    /// Scenario: a record exporter is created without the process host preclaiming stdout.
    /// Guarantees: unsafe startup and live-control transitions are rejected before the
    /// exporter can emit JSON into a stdout that may already contain pretty output.
    #[test]
    fn record_json_requires_a_prestartup_claim() {
        assert!(require_structured_stdout_claim(ConsoleOutputFormat::Pretty, false).is_ok());
        assert!(require_structured_stdout_claim(ConsoleOutputFormat::RecordJson, true).is_ok());
        let error = require_structured_stdout_claim(ConsoleOutputFormat::RecordJson, false)
            .expect_err("an unclaimed record exporter must be rejected");
        assert!(error.to_string().contains("before pipeline startup"));
    }

    /// Scenario: the registered factory creates a record exporter in a fresh process
    /// where stdout has not been preclaimed.
    /// Guarantees: the factory call site rejects the exporter before it can emit JSON
    /// into a stdout that may already contain pretty output.
    #[test]
    fn record_json_factory_rejects_unclaimed_stdout() {
        const CHILD_ENV: &str = "OTAP_TEST_UNCLAIMED_RECORD_FACTORY";

        if std::env::var_os(CHILD_ENV).is_some() {
            let result =
                create_exporter_from_factory(&CONSOLE_EXPORTER, json!({ "format": "record_json" }));
            let error = match result {
                Ok(_) => panic!("an unclaimed record exporter must be rejected"),
                Err(error) => error,
            };
            assert!(error.to_string().contains("before pipeline startup"));
            return;
        }

        let output = std::process::Command::new(
            std::env::current_exe().expect("the test executable path is available"),
        )
        .arg("record_json_factory_rejects_unclaimed_stdout")
        .arg("--nocapture")
        .env(CHILD_ENV, "1")
        .output()
        .expect("the isolated factory test starts");

        assert!(
            output.status.success(),
            "isolated factory test failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn engine_config_with_console(config: &str) -> OtelDataflowSpec {
        OtelDataflowSpec::from_json(&format!(
            r#"{{
                "version": "otel_dataflow/v1",
                "groups": {{
                    "g1": {{
                        "pipelines": {{
                            "p1": {{
                                "nodes": {{
                                    "console": {{
                                        "type": "urn:otel:exporter:console",
                                        "config": {config}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }}"#
        ))
        .expect("engine config parses")
    }

    /// Scenario: a pretty exporter already exists when another exporter claims stdout
    /// for machine-readable records.
    /// Guarantees: the pretty exporter re-resolves its stream and moves to stderr, so a
    /// stream cached at construction cannot keep prose on a structured stdout.
    #[test]
    fn pretty_output_steps_aside_once_stdout_carries_records() {
        // Built before the claim: a construction-time binding would keep stdout.
        let pipeline_ctx = create_test_pipeline_context();
        let pretty = ConsoleExporter::new(&pipeline_ctx, ConsoleExporterConfig::default());
        let records = ConsoleExporter::new(
            &pipeline_ctx,
            ConsoleExporterConfig {
                format: ConsoleOutputFormat::RecordJson,
                ..ConsoleExporterConfig::default()
            },
        );

        // The latch is process-wide and monotonic, matching a record JSON exporter
        // having been created anywhere in this process.
        OutputService::mark_structured_stdout();

        assert_eq!(pretty.output_handle().stream_id(), StreamId::Stderr);
        assert_eq!(records.output_handle().stream_id(), StreamId::Stdout);
    }
}
