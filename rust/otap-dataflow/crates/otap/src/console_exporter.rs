// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Console exporter that prints OTLP data with hierarchical formatting.

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;
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
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otap_df_pdata::OtapPayload;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::views::resource::ResourceView;
use otap_df_telemetry::raw_error;
use otap_df_telemetry::self_tracing::{AnsiCode, BufWriter, ConsoleWriter, LOG_BUFFER_SIZE};
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// The URN for the console exporter
pub const CONSOLE_EXPORTER_URN: &str = "urn:otel:console:exporter";

/// Configuration for the console exporter
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ConsoleExporterConfig {
    /// Whether to use ANSI colors in output (default: true)
    #[serde(default = "default_color")]
    pub color: bool,
    /// Whether to use Unicode box-drawing characters (default: true)
    #[serde(default = "default_unicode")]
    pub unicode: bool,
}

fn default_color() -> bool {
    true
}

fn default_unicode() -> bool {
    true
}

/// Console exporter that prints OTLP data with hierarchical formatting
pub struct ConsoleExporter {
    formatter: HierarchicalFormatter,
}

impl ConsoleExporter {
    /// Create a new console exporter with the given configuration.
    #[must_use]
    pub fn new(config: ConsoleExporterConfig) -> Self {
        Self {
            formatter: HierarchicalFormatter::new(config.color, config.unicode),
        }
    }
}

/// Declare the Console Exporter as a local exporter factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static CONSOLE_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: CONSOLE_EXPORTER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        let config: ConsoleExporterConfig = serde_json::from_value(node_config.config.clone())
            .map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("Failed to parse console exporter config: {}", e),
            })?;
        Ok(ExporterWrapper::local(
            ConsoleExporter::new(config),
            node,
            node_config,
            exporter_config,
        ))
    },
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for ConsoleExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    self.export(&data);
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

impl ConsoleExporter {
    fn export(&self, data: &OtapPdata) {
        let (_, payload) = data.clone().into_parts();
        match payload.signal_type() {
            SignalType::Logs => self.export_logs(&payload),
            SignalType::Traces => self.export_traces(&payload),
            SignalType::Metrics => self.export_metrics(&payload),
        }
    }

    fn export_logs(&self, payload: &OtapPayload) {
        match payload {
            OtapPayload::OtlpBytes(bytes) => {
                self.formatter.format_logs_bytes(bytes);
            }
            OtapPayload::OtapArrowRecords(_records) => {
                // TODO: Support Arrow format using the Arrow view
                raw_error!("Console exporter: Arrow format not yet supported for logs");
            }
        }
    }

    fn export_traces(&self, _payload: &OtapPayload) {
        // TODO: Implement traces formatting.
        raw_error!("Console exporter: Traces formatting not yet implemented");
    }

    fn export_metrics(&self, _payload: &OtapPayload) {
        // TODO: Implement metrics formatting.
        raw_error!("Console exporter: Metrics formatting not yet implemented");
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
        vertical: "│",
        tee: "├─",
        corner: "└─",
    };
    const ASCII: Self = Self {
        vertical: "|",
        tee: "+-",
        corner: "\\-",
    };
}

/// Hierarchical formatter for OTLP data.
pub struct HierarchicalFormatter {
    writer: ConsoleWriter,
    tree: TreeChars,
}

impl HierarchicalFormatter {
    /// Create a new hierarchical formatter.
    #[must_use]
    pub fn new(use_color: bool, use_unicode: bool) -> Self {
        Self {
            writer: if use_color {
                ConsoleWriter::color()
            } else {
                ConsoleWriter::no_color()
            },
            tree: if use_unicode {
                TreeChars::UNICODE
            } else {
                TreeChars::ASCII
            },
        }
    }

    /// Format logs from OTLP bytes.
    pub fn format_logs_bytes(&self, bytes: &OtlpProtoBytes) {
        if let OtlpProtoBytes::ExportLogsRequest(data) = bytes {
            let logs_data = RawLogsData::new(data.as_ref());
            self.format_logs_data(&logs_data);
        }
    }

    /// Format logs from a LogsDataView.
    fn format_logs_data<L: LogsDataView>(&self, logs_data: &L) {
        for resource_logs in logs_data.resources() {
            self.format_resource_logs(&resource_logs);
        }
    }

    /// Format a ResourceLogs with its nested scopes.
    fn format_resource_logs<R: ResourceLogsView>(&self, resource_logs: &R) {
        // Get first timestamp from nested log records
        let first_ts = self.get_first_log_timestamp(resource_logs);

        // Format resource line with its attributes
        self.print_resource_header(first_ts, resource_logs.resource());

        // Format each scope
        let scopes: Vec<_> = resource_logs.scopes().collect();
        let scope_count = scopes.len();
        for (i, scope_logs) in scopes.into_iter().enumerate() {
            let is_last_scope = i == scope_count - 1;
            self.format_scope_logs(&scope_logs, is_last_scope);
        }
    }

    /// Print resource header with attributes.
    fn print_resource_header<R: ResourceView>(&self, time: SystemTime, resource: Option<R>) {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        self.writer.format_header_line(
            &mut w,
            Some(time),
            resource.iter().flat_map(|r| r.attributes()),
            |w, cw| {
                cw.write_styled(w, AnsiCode::Cyan, |w| {
                    let _ = w.write_all(b"RESOURCE");
                });
                let _ = w.write_all(b"   ");
            },
            |w, _| {
                let _ = w.write_all(b"v1.Resource");
            },
        );

        let len = w.position() as usize;
        let _ = std::io::stdout().write_all(&buf[..len]);
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
    fn format_scope_logs<S: ScopeLogsView>(&self, scope_logs: &S, is_last_scope: bool) {
        // Get first timestamp from log records
        let first_ts = scope_logs
            .log_records()
            .find_map(|lr| lr.time_unix_nano().or_else(|| lr.observed_time_unix_nano()))
            .map(nanos_to_time)
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let prefix = format!("{} ", self.tree.vertical);

        // Format scope line: show name/version inline, then attributes
        self.print_scope_header(first_ts, &prefix, scope_logs.scope());

        // Format each log record
        let records: Vec<_> = scope_logs.log_records().collect();
        let record_count = records.len();
        for (i, log_record) in records.into_iter().enumerate() {
            let is_last_record = i == record_count - 1;
            self.print_log_record(&log_record, is_last_scope, is_last_record);
        }
    }

    /// Print scope header: "SCOPE name/version [attributes]"
    fn print_scope_header<S: otap_df_pdata::views::common::InstrumentationScopeView>(
        &self,
        time: SystemTime,
        prefix: &str,
        scope: Option<S>,
    ) {
        let prefix = prefix.to_string();
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        // Extract name/version for inline display
        let name = scope
            .as_ref()
            .and_then(|s| s.name())
            .map(|n| String::from_utf8_lossy(n).into_owned());
        let version = scope
            .as_ref()
            .and_then(|s| s.version())
            .map(|v| String::from_utf8_lossy(v).into_owned());

        self.writer.format_header_line(
            &mut w,
            Some(time),
            scope.iter().flat_map(|s| s.attributes()),
            |w, cw| {
                let _ = w.write_all(prefix.as_bytes());
                cw.write_styled(w, AnsiCode::Magenta, |w| {
                    let _ = w.write_all(b"SCOPE");
                });
                let _ = w.write_all(b"    ");
            },
            |w, _| {
                // Format as "name/version" or just "name" or "v1.InstrumentationScope"
                match (&name, &version) {
                    (Some(n), Some(v)) => {
                        let _ = write!(w, "{}/{}", n, v);
                    }
                    (Some(n), None) => {
                        let _ = w.write_all(n.as_bytes());
                    }
                    _ => {
                        let _ = w.write_all(b"v1.InstrumentationScope");
                    }
                }
            },
        );

        let len = w.position() as usize;
        let _ = std::io::stdout().write_all(&buf[..len]);
    }

    /// Print a single log record using format_log_line.
    fn print_log_record<L: LogRecordView>(
        &self,
        log_record: &L,
        is_last_scope: bool,
        is_last_record: bool,
    ) {
        let time = log_record
            .time_unix_nano()
            .or_else(|| log_record.observed_time_unix_nano())
            .map(nanos_to_time)
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let event_name = log_record
            .event_name()
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .unwrap_or_else(|| "event".to_string());

        let severity = log_record.severity_number();
        let tree = self.tree;

        self.print_line(
            time,
            log_record,
            |w, cw| {
                // Tree prefix
                let _ = w.write_all(tree.vertical.as_bytes());
                let _ = w.write_all(b" ");
                if is_last_record && is_last_scope {
                    let _ = w.write_all(tree.corner.as_bytes());
                } else {
                    let _ = w.write_all(tree.tee.as_bytes());
                }
                let _ = w.write_all(b" ");
                // Severity with color
                cw.write_severity(w, severity);
            },
            |w, _| {
                let _ = w.write_all(event_name.as_bytes());
            },
        );
    }

    /// Print a line using the shared format_log_line.
    fn print_line<V, L, E>(
        &self,
        time: SystemTime,
        record: &V,
        format_level: L,
        format_event_name: E,
    ) where
        V: LogRecordView,
        L: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
        E: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
    {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        self.writer
            .format_log_line(&mut w, Some(time), record, format_level, format_event_name);

        let len = w.position() as usize;
        let _ = std::io::stdout().write_all(&buf[..len]);
    }
}

/// Convert nanoseconds since UNIX_EPOCH to SystemTime.
#[inline]
fn nanos_to_time(nanos: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_nanos(nanos)
}
