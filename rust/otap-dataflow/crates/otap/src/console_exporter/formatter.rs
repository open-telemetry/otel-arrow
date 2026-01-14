// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Hierarchical formatter for OTLP data with tree-style output.
//!
//! This module uses the shared formatting primitives from `otap_df_telemetry::self_tracing`
//! to render OTLP log data in a hierarchical tree format:
//!
//! ```text
//! <timestamp> RESOURCE {service.name=my-service, host.name=localhost}
//! │ <timestamp> SCOPE {name=my-library, version=1.0.0}
//! │ ├─ INFO  event_name: message body [attr=value, ...]
//! │ ├─ WARN  event_name: warning message
//! │ └─ ERROR event_name: error message [code=500]
//! ```

use otap_df_pdata::views::common::{AttributeView, InstrumentationScopeView};
use otap_df_pdata::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::views::resource::ResourceView;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_telemetry::self_tracing::{AnsiCode, BufWriter, ConsoleWriter, LOG_BUFFER_SIZE};
use std::io::Write;

/// Tree drawing characters for Unicode mode.
mod unicode_tree {
    pub const VERTICAL: &str = "│";
    pub const TEE: &str = "├─";
    pub const CORNER: &str = "└─";
    pub const SPACE: &str = "  ";
}

/// Tree drawing characters for ASCII mode.
mod ascii_tree {
    pub const VERTICAL: &str = "|";
    pub const TEE: &str = "+-";
    pub const CORNER: &str = "\\-";
    pub const SPACE: &str = "  ";
}

/// Hierarchical formatter for OTLP data.
pub struct HierarchicalFormatter {
    writer: ConsoleWriter,
    use_unicode: bool,
}

impl HierarchicalFormatter {
    /// Create a new hierarchical formatter.
    #[must_use]
    pub fn new(use_color: bool, use_unicode: bool) -> Self {
        let writer = if use_color {
            ConsoleWriter::color()
        } else {
            ConsoleWriter::no_color()
        };
        Self { writer, use_unicode }
    }

    /// Format logs from OTLP bytes.
    pub fn format_logs_bytes(&self, bytes: &OtlpProtoBytes) {
        if let OtlpProtoBytes::ExportLogsRequest(data) = bytes {
            let logs_data = RawLogsData::new(data.as_ref());
            self.format_logs_data(&logs_data);
        }
    }

    /// Format logs from a LogsDataView.
    fn format_logs_data<L: LogsDataView>(&self, logs_data: &'_ L) {
        for resource_logs in logs_data.resources() {
            self.format_resource_logs(&resource_logs);
        }
    }

    /// Format a ResourceLogs with its nested scopes.
    fn format_resource_logs<R: ResourceLogsView>(&self, resource_logs: &'_ R) {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        // Get first timestamp from nested log records for the resource line
        let first_ts = self.get_first_log_timestamp(resource_logs);

        // Write resource header
        self.writer.write_ansi(&mut w, AnsiCode::Dim);
        ConsoleWriter::write_timestamp(&mut w, first_ts);
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b" ");
        self.writer.write_ansi(&mut w, AnsiCode::Cyan);
        self.writer.write_ansi(&mut w, AnsiCode::Bold);
        let _ = w.write_all(b"RESOURCE");
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b" ");

        // Write resource attributes
        if let Some(resource) = resource_logs.resource() {
            self.write_resource_attrs(&mut w, &resource);
        } else {
            let _ = w.write_all(b"{}");
        }
        let _ = w.write_all(b"\n");

        // Print resource line
        let len = w.position() as usize;
        let _ = std::io::stdout().write_all(&buf[..len]);

        // Format each scope
        let scopes: Vec<_> = resource_logs.scopes().collect();
        let scope_count = scopes.len();
        for (i, scope_logs) in scopes.into_iter().enumerate() {
            let is_last_scope = i == scope_count - 1;
            self.format_scope_logs(&scope_logs, is_last_scope);
        }
    }

    /// Get the first timestamp from log records in a ResourceLogs.
    fn get_first_log_timestamp<R: ResourceLogsView>(&self, resource_logs: &'_ R) -> u64 {
        for scope_logs in resource_logs.scopes() {
            for log_record in scope_logs.log_records() {
                if let Some(ts) = log_record.time_unix_nano() {
                    return ts;
                }
                if let Some(ts) = log_record.observed_time_unix_nano() {
                    return ts;
                }
            }
        }
        0
    }

    /// Format a ScopeLogs with its nested log records.
    fn format_scope_logs<S: ScopeLogsView>(&self, scope_logs: &'_ S, is_last_scope: bool) {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        let tree = self.tree_chars();

        // Get first timestamp from log records for the scope line
        let first_ts = scope_logs
            .log_records()
            .find_map(|lr| lr.time_unix_nano().or_else(|| lr.observed_time_unix_nano()))
            .unwrap_or(0);

        // Write scope header with tree prefix
        let _ = w.write_all(tree.vertical.as_bytes());
        let _ = w.write_all(b" ");
        self.writer.write_ansi(&mut w, AnsiCode::Dim);
        ConsoleWriter::write_timestamp(&mut w, first_ts);
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b" ");
        self.writer.write_ansi(&mut w, AnsiCode::Magenta);
        self.writer.write_ansi(&mut w, AnsiCode::Bold);
        let _ = w.write_all(b"SCOPE");
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b" ");

        // Write scope info
        if let Some(scope) = scope_logs.scope() {
            self.write_scope_info(&mut w, &scope);
        } else {
            let _ = w.write_all(b"{}");
        }
        let _ = w.write_all(b"\n");

        // Print scope line
        let len = w.position() as usize;
        let _ = std::io::stdout().write_all(&buf[..len]);

        // Format each log record
        let records: Vec<_> = scope_logs.log_records().collect();
        let record_count = records.len();
        for (i, log_record) in records.into_iter().enumerate() {
            let is_last_record = i == record_count - 1;
            self.format_log_record(&log_record, is_last_scope, is_last_record);
        }
    }

    /// Format a single log record.
    fn format_log_record<L: LogRecordView>(
        &self,
        log_record: &'_ L,
        is_last_scope: bool,
        is_last_record: bool,
    ) {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let mut w = std::io::Cursor::new(buf.as_mut_slice());

        let tree = self.tree_chars();
        let severity = log_record.severity_number();

        // Tree prefix: vertical line for scope continuation, then branch for record
        let _ = w.write_all(tree.vertical.as_bytes());
        let _ = w.write_all(b" ");
        if is_last_record && is_last_scope {
            let _ = w.write_all(tree.corner.as_bytes());
        } else {
            let _ = w.write_all(tree.tee.as_bytes());
        }
        let _ = w.write_all(b" ");

        // Timestamp
        let ts = log_record
            .time_unix_nano()
            .or_else(|| log_record.observed_time_unix_nano())
            .unwrap_or(0);
        self.writer.write_ansi(&mut w, AnsiCode::Dim);
        ConsoleWriter::write_timestamp(&mut w, ts);
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b" ");

        // Level (using shared severity formatting)
        self.writer.write_severity(&mut w, severity);

        // Event name
        self.writer.write_ansi(&mut w, AnsiCode::Bold);
        if let Some(name) = log_record.event_name() {
            let _ = w.write_all(name.as_ref());
        } else {
            let _ = w.write_all(b"event");
        }
        self.writer.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b": ");

        // Body (using shared AnyValue formatting)
        if let Some(body) = log_record.body() {
            ConsoleWriter::write_any_value(&mut w, &body);
        }

        // Attributes (using shared attribute formatting)
        ConsoleWriter::write_attrs(&mut w, log_record.attributes());

        let _ = w.write_all(b"\n");

        // Print to stdout or stderr based on severity
        let len = w.position() as usize;
        if ConsoleWriter::severity_is_error_or_warn(severity) {
            let _ = std::io::stderr().write_all(&buf[..len]);
        } else {
            let _ = std::io::stdout().write_all(&buf[..len]);
        }
    }

    /// Get tree drawing characters based on mode.
    fn tree_chars(&self) -> TreeChars {
        if self.use_unicode {
            TreeChars {
                vertical: unicode_tree::VERTICAL,
                tee: unicode_tree::TEE,
                corner: unicode_tree::CORNER,
                _space: unicode_tree::SPACE,
            }
        } else {
            TreeChars {
                vertical: ascii_tree::VERTICAL,
                tee: ascii_tree::TEE,
                corner: ascii_tree::CORNER,
                _space: ascii_tree::SPACE,
            }
        }
    }

    /// Write resource attributes in `{key=value, ...}` format.
    fn write_resource_attrs<R: ResourceView>(&self, w: &mut BufWriter<'_>, resource: &R) {
        let _ = w.write_all(b"{");
        let mut first = true;
        for attr in resource.attributes() {
            if !first {
                let _ = w.write_all(b", ");
            }
            first = false;
            let _ = w.write_all(attr.key());
            let _ = w.write_all(b"=");
            if let Some(v) = attr.value() {
                ConsoleWriter::write_any_value(w, &v);
            }
        }
        let _ = w.write_all(b"}");
    }

    /// Write scope information in `{name=..., version=..., ...}` format.
    fn write_scope_info<S: InstrumentationScopeView>(&self, w: &mut BufWriter<'_>, scope: &S) {
        let _ = w.write_all(b"{");
        let mut has_content = false;

        if let Some(name) = scope.name() {
            let _ = w.write_all(b"name=");
            let _ = w.write_all(name.as_ref());
            has_content = true;
        }

        if let Some(version) = scope.version() {
            if has_content {
                let _ = w.write_all(b", ");
            }
            let _ = w.write_all(b"version=");
            let _ = w.write_all(version.as_ref());
            has_content = true;
        }

        // Include scope attributes
        for attr in scope.attributes() {
            if has_content {
                let _ = w.write_all(b", ");
            }
            let _ = w.write_all(attr.key());
            let _ = w.write_all(b"=");
            if let Some(v) = attr.value() {
                ConsoleWriter::write_any_value(w, &v);
            }
            has_content = true;
        }

        let _ = w.write_all(b"}");
    }
}

/// Tree drawing characters.
struct TreeChars {
    vertical: &'static str,
    tee: &'static str,
    corner: &'static str,
    _space: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_chars() {
        let unicode = HierarchicalFormatter::new(false, true);
        let ascii = HierarchicalFormatter::new(false, false);

        assert_eq!(unicode.tree_chars().vertical, "│");
        assert_eq!(unicode.tree_chars().tee, "├─");

        assert_eq!(ascii.tree_chars().vertical, "|");
        assert_eq!(ascii.tree_chars().tee, "+-");
    }
}
