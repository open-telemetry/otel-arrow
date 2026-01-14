// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Hierarchical formatter for OTLP data with tree-style output.
//!
//! Output format:
//! ```text
//! <timestamp> RESOURCE {service.name=my-service, host.name=localhost}
//! │ <timestamp> SCOPE {name=my-library, version=1.0.0}
//! │ ├─ INFO  event_name: message body [attr=value, ...]
//! │ ├─ WARN  event_name: warning message
//! │ └─ ERROR event_name: error message [code=500]
//! ```

use chrono::{DateTime, Datelike, Timelike, Utc};
use otap_df_pdata::views::common::{AnyValueView, AttributeView, InstrumentationScopeView};
use otap_df_pdata::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::views::resource::ResourceView;
use otap_df_pdata::OtlpProtoBytes;
use std::io::{Cursor, Write};

/// Buffer size for formatting output.
const OUTPUT_BUFFER_SIZE: usize = 8192;

/// Log level derived from OTLP severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

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

/// ANSI codes for colored output.
#[derive(Clone, Copy)]
#[repr(u8)]
enum AnsiCode {
    Reset = 0,
    Bold = 1,
    Dim = 2,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
}

/// Hierarchical formatter for OTLP data.
pub struct HierarchicalFormatter {
    use_color: bool,
    use_unicode: bool,
}

impl HierarchicalFormatter {
    /// Create a new hierarchical formatter.
    #[must_use]
    pub fn new(use_color: bool, use_unicode: bool) -> Self {
        Self {
            use_color,
            use_unicode,
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
    fn format_logs_data<'a, L: LogsDataView>(&self, logs_data: &'a L) {
        for resource_logs in logs_data.resources() {
            self.format_resource_logs(&resource_logs);
        }
    }

    /// Format a ResourceLogs with its nested scopes.
    fn format_resource_logs<'a, R: ResourceLogsView>(&self, resource_logs: &'a R) {
        let mut buf = [0u8; OUTPUT_BUFFER_SIZE];
        let mut w = Cursor::new(buf.as_mut_slice());

        // Get first timestamp from nested log records for the resource line
        let first_ts = self.get_first_log_timestamp(resource_logs);

        // Write resource header
        self.write_ansi(&mut w, AnsiCode::Dim);
        Self::write_timestamp(&mut w, first_ts);
        self.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b" ");
        self.write_ansi(&mut w, AnsiCode::Cyan);
        self.write_ansi(&mut w, AnsiCode::Bold);
        let _ = w.write_all(b"RESOURCE");
        self.write_ansi(&mut w, AnsiCode::Reset);
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
    fn get_first_log_timestamp<'a, R: ResourceLogsView>(&self, resource_logs: &'a R) -> u64 {
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
    fn format_scope_logs<'a, S: ScopeLogsView>(&self, scope_logs: &'a S, is_last_scope: bool) {
        let mut buf = [0u8; OUTPUT_BUFFER_SIZE];
        let mut w = Cursor::new(buf.as_mut_slice());

        let tree = self.tree_chars();

        // Get first timestamp from log records for the scope line
        let first_ts = scope_logs
            .log_records()
            .find_map(|lr| lr.time_unix_nano().or_else(|| lr.observed_time_unix_nano()))
            .unwrap_or(0);

        // Write scope header with tree prefix
        let _ = w.write_all(tree.vertical.as_bytes());
        let _ = w.write_all(b" ");
        self.write_ansi(&mut w, AnsiCode::Dim);
        Self::write_timestamp(&mut w, first_ts);
        self.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b" ");
        self.write_ansi(&mut w, AnsiCode::Magenta);
        self.write_ansi(&mut w, AnsiCode::Bold);
        let _ = w.write_all(b"SCOPE");
        self.write_ansi(&mut w, AnsiCode::Reset);
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
    fn format_log_record<'a, L: LogRecordView>(
        &self,
        log_record: &'a L,
        is_last_scope: bool,
        is_last_record: bool,
    ) {
        let mut buf = [0u8; OUTPUT_BUFFER_SIZE];
        let mut w = Cursor::new(buf.as_mut_slice());

        let tree = self.tree_chars();

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
        self.write_ansi(&mut w, AnsiCode::Dim);
        Self::write_timestamp(&mut w, ts);
        self.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b" ");

        // Level
        let level = self.severity_to_level(log_record.severity_number());
        self.write_level(&mut w, level);
        let _ = w.write_all(b" ");

        // Event name
        self.write_ansi(&mut w, AnsiCode::Bold);
        if let Some(name) = log_record.event_name() {
            let _ = w.write_all(name.as_ref());
        } else {
            let _ = w.write_all(b"event");
        }
        self.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b": ");

        // Body
        if let Some(body) = log_record.body() {
            self.write_any_value(&mut w, &body);
        }

        // Attributes
        let mut attrs = log_record.attributes().peekable();
        if attrs.peek().is_some() {
            let _ = w.write_all(b" [");
            let mut first = true;
            for attr in attrs {
                if !first {
                    let _ = w.write_all(b", ");
                }
                first = false;
                let _ = w.write_all(attr.key());
                let _ = w.write_all(b"=");
                if let Some(v) = attr.value() {
                    self.write_any_value(&mut w, &v);
                }
            }
        let _ = w.write_all(b"]");
        }

        let _ = w.write_all(b"\n");

        // Print to stdout or stderr based on level
        let len = w.position() as usize;
        if matches!(level, Level::Error | Level::Warn) {
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

    /// Write an ANSI escape code.
    #[inline]
    fn write_ansi(&self, w: &mut Cursor<&mut [u8]>, code: AnsiCode) {
        if self.use_color {
            let _ = write!(w, "\x1b[{}m", code as u8);
        }
    }

    /// Write a colored level indicator.
    fn write_level(&self, w: &mut Cursor<&mut [u8]>, level: Level) {
        let color = match level {
            Level::Error => AnsiCode::Red,
            Level::Warn => AnsiCode::Yellow,
            Level::Info => AnsiCode::Green,
            Level::Debug => AnsiCode::Blue,
            Level::Trace => AnsiCode::Magenta,
        };
        self.write_ansi(w, color);
        let _ = w.write_all(level.as_str().as_bytes());
        self.write_ansi(w, AnsiCode::Reset);
        // Pad to 5 chars
        let padding = 5 - level.as_str().len();
        for _ in 0..padding {
            let _ = w.write_all(b" ");
        }
    }

    /// Convert OTLP severity number to Level.
    fn severity_to_level(&self, severity: Option<i32>) -> Level {
        match severity {
            Some(n) if n >= 17 => Level::Error, // FATAL, ERROR
            Some(n) if n >= 13 => Level::Warn,  // WARN
            Some(n) if n >= 9 => Level::Info,   // INFO
            Some(n) if n >= 5 => Level::Debug,  // DEBUG
            Some(_) => Level::Trace,            // TRACE
            None => Level::Info,                // Default to INFO
        }
    }

    /// Write timestamp in ISO 8601 format.
    fn write_timestamp(w: &mut Cursor<&mut [u8]>, nanos: u64) {
        let secs = (nanos / 1_000_000_000) as i64;
        let subsec_nanos = (nanos % 1_000_000_000) as u32;

        if let Some(dt) = DateTime::<Utc>::from_timestamp(secs, subsec_nanos) {
            let date = dt.date_naive();
            let time = dt.time();
            let millis = subsec_nanos / 1_000_000;

            let _ = write!(
                w,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                date.year(),
                date.month(),
                date.day(),
                time.hour(),
                time.minute(),
                time.second(),
                millis
            );
        } else {
            let _ = w.write_all(b"<invalid>");
        }
    }

    /// Write resource attributes.
    fn write_resource_attrs<R: ResourceView>(&self, w: &mut Cursor<&mut [u8]>, resource: &R) {
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
                self.write_any_value(w, &v);
            }
        }
        let _ = w.write_all(b"}");
    }

    /// Write scope information.
    fn write_scope_info<S: InstrumentationScopeView>(
        &self,
        w: &mut Cursor<&mut [u8]>,
        scope: &S,
    ) {
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
                self.write_any_value(w, &v);
            }
            has_content = true;
        }

        let _ = w.write_all(b"}");
    }

    /// Write an AnyValue.
    fn write_any_value<'a>(&self, w: &mut Cursor<&mut [u8]>, value: &impl AnyValueView<'a>) {
        use otap_df_pdata::views::common::ValueType;

        match value.value_type() {
            ValueType::String => {
                if let Some(s) = value.as_string() {
                    let _ = w.write_all(s);
                }
            }
            ValueType::Int64 => {
                if let Some(i) = value.as_int64() {
                    let _ = write!(w, "{}", i);
                }
            }
            ValueType::Bool => {
                if let Some(b) = value.as_bool() {
                    let _ = w.write_all(if b { b"true" } else { b"false" });
                }
            }
            ValueType::Double => {
                if let Some(d) = value.as_double() {
                    let _ = write!(w, "{:.6}", d);
                }
            }
            ValueType::Bytes => {
                if let Some(bytes) = value.as_bytes() {
                    let _ = write!(w, "<{} bytes>", bytes.len());
                }
            }
            ValueType::Array => {
                let _ = w.write_all(b"[");
                if let Some(array_iter) = value.as_array() {
                    let mut first = true;
                    for item in array_iter {
                        if !first {
                            let _ = w.write_all(b", ");
                        }
                        first = false;
                        self.write_any_value(w, &item);
                    }
                }
                let _ = w.write_all(b"]");
            }
            ValueType::KeyValueList => {
                let _ = w.write_all(b"{");
                if let Some(kvlist_iter) = value.as_kvlist() {
                    let mut first = true;
                    for kv in kvlist_iter {
                        if !first {
                            let _ = w.write_all(b", ");
                        }
                        first = false;
                        let _ = w.write_all(kv.key());
                        if let Some(val) = kv.value() {
                            let _ = w.write_all(b"=");
                            self.write_any_value(w, &val);
                        }
                    }
                }
                let _ = w.write_all(b"}");
            }
            ValueType::Empty => {}
        }
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
    fn test_severity_to_level() {
        let formatter = HierarchicalFormatter::new(false, true);

        assert_eq!(formatter.severity_to_level(Some(21)), Level::Error); // FATAL
        assert_eq!(formatter.severity_to_level(Some(17)), Level::Error); // ERROR
        assert_eq!(formatter.severity_to_level(Some(13)), Level::Warn); // WARN
        assert_eq!(formatter.severity_to_level(Some(9)), Level::Info); // INFO
        assert_eq!(formatter.severity_to_level(Some(5)), Level::Debug); // DEBUG
        assert_eq!(formatter.severity_to_level(Some(1)), Level::Trace); // TRACE
        assert_eq!(formatter.severity_to_level(None), Level::Info); // Default
    }

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
