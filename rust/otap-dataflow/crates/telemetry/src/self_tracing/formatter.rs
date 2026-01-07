// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! An alternative to Tokio fmt::layer().

use super::{LogRecord, SavedCallsite};
use bytes::Bytes;
use otap_df_pdata::proto::consts::field_num::logs::{LOG_RECORD_ATTRIBUTES, LOG_RECORD_BODY};
use otap_df_pdata::proto::consts::wire_types;
use otap_df_pdata::views::common::{AnyValueView, AttributeView, ValueType};
use otap_df_pdata::views::otlp::bytes::common::{RawAnyValue, RawKeyValue};
use otap_df_pdata::views::otlp::bytes::decode::read_varint;
use std::io::Write;
use tracing::span::{Attributes, Record};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer as TracingLayer};
use tracing_subscriber::registry::LookupSpan;

/// Console formatter writes to stdout or stderr.
#[derive(Debug)]
pub struct ConsoleWriter {
    use_ansi: bool,
}

/// A minimal formatting layer that outputs log records to stdout/stderr.
///
/// This is a lightweight alternative to `tracing_subscriber::fmt::layer()`.
pub struct RawLayer {
    writer: ConsoleWriter,
}

// ANSI color codes
const ANSI_RESET: &str = "\x1b[0m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_BOLD: &str = "\x1b[1m";

impl RawLayer {
    /// Return a new fomatting layer with associated writer.
    pub fn new(writer: ConsoleWriter) -> Self {
        Self { writer }
    }
}

impl ConsoleWriter {
    /// Create a writer that outputs to stdout without ANSI colors.
    pub fn no_color() -> Self {
        Self { use_ansi: false }
    }

    /// Create a writer that outputs to stderr without ANSI colors.
    pub fn color() -> Self {
        Self { use_ansi: true }
    }

    /// Format a InternalLogRecord as a human-readable string.
    ///
    /// Output format: `2026-01-06T10:30:45.123Z  INFO target::name (file.rs:42): body [attr=value, ...]`
    pub fn format_log_record(&self, record: &LogRecord, callsite: &SavedCallsite) -> String {
        let event_name = Self::format_event_name(callsite);

        let body_attrs = Self::format_body_attrs(&record.body_attrs_bytes);

        if self.use_ansi {
            let level_color = Self::level_color(callsite.level);
            format!(
                "{}{}{}  {}{:5}{}  {}{}{}: {}\n",
                ANSI_DIM,
                Self::format_timestamp(record.timestamp_ns),
                ANSI_RESET,
                level_color,
                callsite.level.as_str(),
                ANSI_RESET,
                ANSI_BOLD,
                event_name,
                ANSI_RESET,
                body_attrs,
            )
        } else {
            format!(
                "{}  {:5}  {}: {}\n",
                Self::format_timestamp(record.timestamp_ns),
                callsite.level.as_str(),
                event_name,
                body_attrs,
            )
        }
    }

    /// Format callsite details as event_name string.
    ///
    /// Format: "target::name (file.rs:42)" or "target::name" if file/line unavailable.
    #[inline]
    fn format_event_name(callsite: &SavedCallsite) -> String {
        match (callsite.file, callsite.line) {
            (Some(file), Some(line)) => {
                format!("{}::{} ({}:{})", callsite.target, callsite.name, file, line)
            }
            _ => format!("{}::{}", callsite.target, callsite.name),
        }
    }

    /// Format nanosecond timestamp as ISO 8601 (UTC).
    fn format_timestamp(nanos: u64) -> String {
        let secs = nanos / 1_000_000_000;
        let subsec_millis = (nanos % 1_000_000_000) / 1_000_000;

        // Convert to datetime components
        // Days since Unix epoch
        let days = secs / 86400;
        let time_of_day = secs % 86400;

        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;

        // Calculate year/month/day from days since epoch (1970-01-01)
        let (year, month, day) = Self::days_to_ymd(days as i64);

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            year, month, day, hours, minutes, seconds, subsec_millis
        )
    }

    /// Convert days since Unix epoch to (year, month, day).
    fn days_to_ymd(days: i64) -> (i32, u32, u32) {
        // Algorithm from Howard Hinnant's date library
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = (z - era * 146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };

        (y as i32, m, d)
    }

    /// Format body+attrs bytes as readable string.
    ///
    /// Uses the pdata View types (`RawAnyValue`, `RawKeyValue`) for zero-copy
    /// parsing of the OTLP protobuf bytes. This is consistent with the decoding
    /// approach used in `otlp_bytes_formatter.rs`.
    fn format_body_attrs(bytes: &Bytes) -> String {
        if bytes.is_empty() {
            return String::new();
        }

        // The bytes contain LogRecord fields:
        // - field 5 (LOG_RECORD_BODY): AnyValue message
        // - field 6 (LOG_RECORD_ATTRIBUTES): repeated KeyValue messages

        let mut body_str = String::new();
        let mut attrs = Vec::new();
        let data = bytes.as_ref();
        let mut pos = 0;

        while pos < data.len() {
            // Read field tag
            let (tag, next_pos) = match read_varint(data, pos) {
                Some(v) => v,
                None => break,
            };
            pos = next_pos;

            let field_num = tag >> 3;
            let wire_type = tag & 0x7;

            if wire_type != wire_types::LEN {
                // Skip non-length-delimited fields (shouldn't happen for body/attrs)
                break;
            }

            // Read length-delimited content
            let (len, next_pos) = match read_varint(data, pos) {
                Some(v) => v,
                None => break,
            };
            pos = next_pos;
            let end = pos + len as usize;

            if end > data.len() {
                break;
            }

            let field_bytes = &data[pos..end];

            if field_num == LOG_RECORD_BODY {
                // Body: parse as AnyValue using pdata View
                let any_value = RawAnyValue::new(field_bytes);
                body_str = Self::format_any_value(&any_value);
            } else if field_num == LOG_RECORD_ATTRIBUTES {
                // Attribute: parse as KeyValue using pdata View
                let kv = RawKeyValue::new(field_bytes);
                let key = String::from_utf8_lossy(kv.key()).to_string();
                let value = match kv.value() {
                    Some(v) => Self::format_any_value(&v),
                    None => "<?>".to_string(),
                };
                attrs.push(format!("{}={}", key, value));
            }

            pos = end;
        }

        if !attrs.is_empty() {
            body_str.push_str(" [");
            body_str.push_str(&attrs.join(", "));
            body_str.push(']');
        }

        body_str
    }

    /// Format an AnyValue for display.
    fn format_any_value<'a>(value: &impl AnyValueView<'a>) -> String {
        match value.value_type() {
            ValueType::String => {
                if let Some(s) = value.as_string() {
                    String::from_utf8_lossy(s).to_string()
                } else {
                    String::new()
                }
            }
            ValueType::Int64 => {
                if let Some(i) = value.as_int64() {
                    i.to_string()
                } else {
                    String::new()
                }
            }
            ValueType::Bool => {
                if let Some(b) = value.as_bool() {
                    b.to_string()
                } else {
                    String::new()
                }
            }
            ValueType::Double => {
                if let Some(d) = value.as_double() {
                    format!("{:.6}", d)
                } else {
                    String::new()
                }
            }
            ValueType::Bytes => {
                if let Some(bytes) = value.as_bytes() {
                    format!("{:?}", bytes)
                } else {
                    String::new()
                }
            }
            ValueType::Array => {
                if let Some(array_iter) = value.as_array() {
                    let parts: Vec<_> = array_iter
                        .map(|item| Self::format_any_value(&item))
                        .collect();
                    format!("[{}]", parts.join(", "))
                } else {
                    "[]".to_string()
                }
            }
            ValueType::KeyValueList => {
                if let Some(kvlist_iter) = value.as_kvlist() {
                    let parts: Vec<_> = kvlist_iter
                        .map(|kv| {
                            let key_str = String::from_utf8_lossy(kv.key()).to_string();
                            match kv.value() {
                                Some(val) => {
                                    format!("{}={}", key_str, Self::format_any_value(&val))
                                }
                                None => key_str,
                            }
                        })
                        .collect();
                    format!("{{{}}}", parts.join(", "))
                } else {
                    "{}".to_string()
                }
            }
            ValueType::Empty => String::new(),
        }
    }

    /// Write a log line
    fn write_line(&self, level: &Level, line: &str) {
        let use_stderr = match *level {
            Level::ERROR => true,
            Level::WARN => true,
            _ => false,
        };
        // Ignore error from write()
        let _ = if use_stderr {
            std::io::stderr().write(line.as_bytes())
        } else {
            std::io::stdout().write(line.as_bytes())
        };
    }

    /// Get ANSI color code for a severity level.
    #[inline]
    fn level_color(level: &Level) -> &'static str {
        match *level {
            Level::ERROR => ANSI_RED,
            Level::WARN => ANSI_YELLOW,
            Level::INFO => ANSI_GREEN,
            Level::DEBUG => ANSI_BLUE,
            Level::TRACE => ANSI_MAGENTA,
        }
    }
}

impl<S> TracingLayer<S> for RawLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();

        // Build compact record
        let record = LogRecord::new(event);

        // Format and write immediately
        let callsite = SavedCallsite::new(metadata);
        let line = self.writer.format_log_record(&record, &callsite);
        self.writer.write_line(callsite.level, &line);
    }

    fn on_new_span(&self, _attrs: &Attributes<'_>, _id: &tracing::span::Id, _ctx: Context<'_, S>) {
        // Not handling spans
    }

    fn on_record(&self, _span: &tracing::span::Id, _values: &Record<'_>, _ctx: Context<'_, S>) {
        // Not handling spans
    }

    fn on_enter(&self, _id: &tracing::span::Id, _ctx: Context<'_, S>) {
        // Not handling spans
    }

    fn on_exit(&self, _id: &tracing::span::Id, _ctx: Context<'_, S>) {
        // Not handling spans
    }

    fn on_close(&self, _id: tracing::span::Id, _ctx: Context<'_, S>) {
        // Not handling spans
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;

    #[test]
    fn test_format_timestamp() {
        // 2026-01-06T10:30:45.123Z in nanoseconds
        // Let's use a known timestamp: 2024-01-01T00:00:00.000Z
        let nanos: u64 = 1704067200 * 1_000_000_000; // 2024-01-01 00:00:00 UTC
        let formatted = ConsoleWriter::format_timestamp(nanos);
        assert_eq!(formatted, "2024-01-01T00:00:00.000Z");

        // Test with milliseconds
        let nanos_with_ms: u64 = 1704067200 * 1_000_000_000 + 123_000_000;
        let formatted = ConsoleWriter::format_timestamp(nanos_with_ms);
        assert_eq!(formatted, "2024-01-01T00:00:00.123Z");
    }

    #[test]
    fn test_days_to_ymd() {
        // 1970-01-01 is day 0
        assert_eq!(ConsoleWriter::days_to_ymd(0), (1970, 1, 1));

        // 2024-01-01 is 19723 days after 1970-01-01
        assert_eq!(ConsoleWriter::days_to_ymd(19723), (2024, 1, 1));
    }

    #[test]
    fn test_simple_writer_creation() {
        let _stdout = ConsoleWriter::color();
        let _stderr = ConsoleWriter::no_color();
    }

    #[test]
    fn test_formatter_layer_creation() {
        let _color = RawLayer::new(ConsoleWriter::color());
        let _nocolor = RawLayer::new(ConsoleWriter::no_color());
    }

    #[test]
    fn test_layer_integration() {
        // Create the layer and subscriber
        let layer = RawLayer::new(ConsoleWriter::no_color());
        let subscriber = tracing_subscriber::registry().with(layer);

        // Set as default for this thread temporarily
        let dispatch = tracing::Dispatch::new(subscriber);
        let _guard = tracing::dispatcher::set_default(&dispatch);

        // Emit some events - these should be formatted and written to stderr
        tracing::info!("Test info message");
        tracing::warn!(count = 42, "Warning with attribute");
        tracing::error!(error = "something failed", "Error occurred");

        // The test verifies no panics occur; actual output goes to stderr
    }
}
