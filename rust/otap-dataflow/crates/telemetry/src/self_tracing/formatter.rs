// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! An alternative to Tokio fmt::layer().

use super::encoder::level_to_severity_number;
use super::{LogRecord, SavedCallsite};
use chrono::{DateTime, Datelike, Timelike, Utc};
use otap_df_pdata::views::common::{AnyValueView, AttributeView, ValueType};
use otap_df_pdata::views::logs::LogRecordView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogRecord;
use std::io::{Cursor, Write};
use std::time::SystemTime;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer as TracingLayer};
use tracing_subscriber::registry::LookupSpan;

/// Default buffer size for log formatting.
///
/// TODO: Append a note to the log message when truncation occurs, otherwise
/// today the log record is silently truncated.
pub const LOG_BUFFER_SIZE: usize = 4096;

/// ANSI codes a.k.a. "Select Graphic Rendition" codes.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AnsiCode {
    /// Reset all attributes.
    Reset = 0,
    /// Bold text.
    Bold = 1,
    /// Dim text.
    Dim = 2,
    /// Red foreground.
    Red = 31,
    /// Green foreground.
    Green = 32,
    /// Yellow foreground.
    Yellow = 33,
    /// Blue foreground.
    Blue = 34,
    /// Magenta foreground.
    Magenta = 35,
    /// Cyan foreground.
    Cyan = 36,
}

/// Color mode for console output.
#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    /// Enable ANSI color codes.
    Color,
    /// Disable ANSI color codes.
    NoColor,
}

impl ColorMode {
    /// Write an ANSI escape sequence (no-op for NoColor).
    #[inline]
    fn write_ansi(self, w: &mut BufWriter<'_>, code: AnsiCode) {
        if let ColorMode::Color = self {
            let _ = write!(w, "\x1b[{}m", code as u8);
        }
    }
}

/// Console writes formatted text to stdout or stderr.
#[derive(Debug, Clone, Copy)]
pub struct ConsoleWriter {
    color_mode: ColorMode,
}

/// A minimal alternative to `tracing_subscriber::fmt::layer()`.
pub struct RawLoggingLayer {
    writer: ConsoleWriter,
}

impl RawLoggingLayer {
    /// Return a new formatting layer with associated writer.
    #[must_use]
    pub fn new(writer: ConsoleWriter) -> Self {
        Self { writer }
    }

    /// Process a tracing Event directly, bypassing the dispatcher.
    pub fn dispatch_event(&self, event: &Event<'_>) {
        let time = SystemTime::now();
        // TODO: there are allocations implied in LogRecord::new that we
        // would prefer to avoid; it will be an extensive change in the
        // ProtoBuffer impl to stack-allocate this as a temporary.
        let record = LogRecord::new(event);
        self.writer.print_log_record(time, &record);
    }
}

/// Type alias for a cursor over a byte buffer.
/// Uses `std::io::Cursor` for position tracking with `std::io::Write`.
pub type BufWriter<'a> = Cursor<&'a mut [u8]>;

impl ConsoleWriter {
    /// Create a writer that outputs to stdout without ANSI colors.
    #[must_use]
    pub fn no_color() -> Self {
        Self {
            color_mode: ColorMode::NoColor,
        }
    }

    /// Create a writer that outputs to stderr with ANSI colors.
    #[must_use]
    pub fn color() -> Self {
        Self {
            color_mode: ColorMode::Color,
        }
    }

    /// Format a LogRecord as a human-readable string (for testing/compatibility).
    ///
    /// Output format: `2026-01-06T10:30:45.123Z  INFO target::name (file.rs:42): body [attr=value, ...]`
    pub fn format_log_record(&self, time: Option<SystemTime>, record: &LogRecord) -> String {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let len = self.format_log_record_into(&mut buf, time, record);
        // The buffer contains valid UTF-8 since we only write ASCII and valid UTF-8 strings
        String::from_utf8_lossy(&buf[..len]).into_owned()
    }

    /// Print a LogRecord directly to stdout or stderr (based on level).
    pub fn print_log_record(&self, time: SystemTime, record: &LogRecord) {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let len = self.format_log_record_into(&mut buf, Some(time), record);
        self.write_line(record.callsite().level(), &buf[..len]);
    }

    /// Encode a LogRecord to a byte buffer. Returns the number of bytes written.
    #[allow(dead_code)]
    fn format_log_record_into(
        &self,
        buf: &mut [u8],
        time: Option<SystemTime>,
        record: &LogRecord,
    ) -> usize {
        let mut w = Cursor::new(buf);

        // Create a view over the pre-encoded body+attrs bytes
        let view = RawLogRecord::new(record.body_attrs_bytes.as_ref());
        let level = *record.callsite().level();

        self.format_log_line(
            &mut w,
            time,
            &view,
            |w, cw| cw.write_level(w, &level),
            |w, cw| {
                cw.write_styled(w, AnsiCode::Bold, |w| Self::write_event_name(w, record));
            },
        );

        // Add scope continuation line if entity context is present (Inline mode only)
        if record.context.len() != 0 {
            self.format_scope_continuation_into(&mut w, record);
        }

        w.position() as usize
    }

    /// Format a scope continuation line showing entity context.
    /// This prints on a new line without timestamp/level/name prefix.
    /// Format: `  └─ scope [pipeline=<key>, node=<key>]`
    fn format_scope_continuation_into(&self, w: &mut BufWriter<'_>, record: &LogRecord) {
        let _ = w.write_all(b"    ");
        self.write_styled(w, AnsiCode::Dim, |w| {
            let _ = w.write_all(b"scope");
        });
        let _ = w.write_all(b" [");

        let mut first = true;
        for key in record.context.iter() {
            if !first {
                let _ = w.write_all(b", ");
            }
            let _ = write!(w, "{:?}", key);
            first = false;
        }
        let _ = w.write_all(b"]\n");
    }

    /// Write callsite details as event_name to buffer.
    #[inline]
    pub(crate) fn write_event_name(w: &mut BufWriter<'_>, record: &LogRecord) {
        write_event_name_to(w, &record.callsite());
    }
}

/// Write callsite details as event_name to any `io::Write` target.
///
/// Format: `target::name (file:line)` or `target::name` if no file/line.
/// This is used by both the text formatter and the OTLP encoder.
#[inline]
pub fn write_event_name_to<W: Write>(w: &mut W, callsite: &SavedCallsite) {
    let _ = w.write_all(callsite.target().as_bytes());
    let _ = w.write_all(b"::");
    let _ = w.write_all(callsite.name().as_bytes());
    if let (Some(file), Some(line)) = (callsite.file(), callsite.line()) {
        let _ = write!(w, " ({}:{})", file, line);
    }
}

impl ConsoleWriter {
    /// Write a SystemTime timestamp as ISO 8601 (UTC) to buffer.
    #[inline]
    pub fn write_timestamp(w: &mut BufWriter<'_>, time: SystemTime) {
        let dt: DateTime<Utc> = time.into();
        let millis = dt.timestamp_subsec_millis();

        let _ = write!(
            w,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
            millis
        );
    }

    /// Write body and attributes from a LogRecordView to buffer.
    /// - If has_event_name and body present: print ": " then body
    /// - If has_event_name and no body but attrs present: print ":" (attrs add " [")
    /// - Body prints directly
    /// - Attributes print " [...]" before themselves
    fn write_body_and_attrs<V: LogRecordView>(
        w: &mut BufWriter<'_>,
        record: &V,
        has_event_name: bool,
    ) {
        let body = record.body();
        let mut attrs = record.attributes().peekable();
        let has_body = body.is_some();
        let has_attrs = attrs.peek().is_some();

        // Print separator after event_name if there's content following
        if has_event_name && (has_body || has_attrs) {
            if has_body {
                let _ = w.write_all(b": ");
            } else {
                // No body, attrs will add " [" so just print ":"
                let _ = w.write_all(b":");
            }
        }

        // Write body if present
        if let Some(body) = body {
            Self::write_any_value(w, &body);
        }

        // Write attributes if present (with leading " [")
        Self::write_attrs(w, attrs);
    }

    /// Write attributes from any AttributeView iterator to buffer.
    pub fn write_attrs<A: AttributeView>(w: &mut BufWriter<'_>, attrs: impl Iterator<Item = A>) {
        let mut attrs = attrs.peekable();
        if attrs.peek().is_some() {
            let _ = w.write_all(b" [");
            let mut first = true;
            for attr in attrs {
                if Self::is_full(w) {
                    break;
                }
                if !first {
                    let _ = w.write_all(b", ");
                }
                first = false;
                let _ = w.write_all(attr.key());
                let _ = w.write_all(b"=");
                match attr.value() {
                    Some(v) => Self::write_any_value(w, &v),
                    None => {
                        let _ = w.write_all(b"<?>");
                    }
                }
            }
            let _ = w.write_all(b"]");
        }
    }

    /// Check if the buffer is full (position >= capacity).
    #[inline]
    fn is_full(w: &BufWriter<'_>) -> bool {
        w.position() as usize >= w.get_ref().len()
    }

    /// Write an AnyValue to buffer (strings unquoted).
    fn write_any_value<'a>(w: &mut BufWriter<'_>, value: &impl AnyValueView<'a>) {
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
                    let _ = w.write_all(b"[");
                    for (i, b) in bytes.iter().enumerate() {
                        if i > 0 {
                            let _ = w.write_all(b", ");
                        }
                        let _ = write!(w, "{}", b);
                    }
                    let _ = w.write_all(b"]");
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
                        Self::write_any_value(w, &item);
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
                            Self::write_any_value(w, &val);
                        }
                    }
                }
                let _ = w.write_all(b"}");
            }
            ValueType::Empty => {}
        }
    }

    /// Write content with ANSI styling, automatically resetting after.
    #[inline]
    pub fn write_styled<F>(&self, w: &mut BufWriter<'_>, code: AnsiCode, f: F)
    where
        F: FnOnce(&mut BufWriter<'_>),
    {
        self.color_mode.write_ansi(w, code);
        f(w);
        self.color_mode.write_ansi(w, AnsiCode::Reset);
    }

    /// Write a tracing Level with appropriate color and padding.
    #[inline]
    pub fn write_level(&self, w: &mut BufWriter<'_>, level: &Level) {
        self.write_severity(w, Some(level_to_severity_number(level) as i32), None);
    }

    /// Write severity with appropriate color and padding.
    /// Severity numbers follow OTLP conventions (1-24, where INFO=9).
    /// If severity_text is provided, it overrides the default text derived from the number.
    #[inline]
    pub fn write_severity(
        &self,
        w: &mut BufWriter<'_>,
        severity: Option<i32>,
        severity_text: Option<&[u8]>,
    ) {
        // Determine text: use provided text if non-empty, otherwise derive from number
        let (text, code): (&[u8], _) = match severity_text.filter(|t| !t.is_empty()) {
            Some(t) => (t, Self::severity_to_color(severity)),
            None => match severity {
                Some(s) if s >= 17 => (b"ERROR", AnsiCode::Red),
                Some(s) if s >= 13 => (b"WARN ", AnsiCode::Yellow),
                Some(s) if s >= 9 => (b"INFO ", AnsiCode::Green),
                Some(s) if s >= 5 => (b"DEBUG", AnsiCode::Blue),
                Some(s) if s >= 1 => (b"TRACE", AnsiCode::Magenta),
                _ => (b"     ", AnsiCode::Reset),
            },
        };
        self.write_styled(w, code, |w| {
            let _ = w.write_all(text);
        });
        let _ = w.write_all(b" ");
    }

    /// Map severity number to ANSI color code.
    #[inline]
    fn severity_to_color(severity: Option<i32>) -> AnsiCode {
        match severity {
            Some(s) if s >= 17 => AnsiCode::Red,    // FATAL/ERROR
            Some(s) if s >= 13 => AnsiCode::Yellow, // WARN
            Some(s) if s >= 9 => AnsiCode::Green,   // INFO
            Some(s) if s >= 5 => AnsiCode::Blue,    // DEBUG
            Some(s) if s >= 1 => AnsiCode::Magenta, // TRACE
            _ => AnsiCode::Reset,
        }
    }

    /// Format a log line from a LogRecordView with custom formatters.
    ///
    /// Spacing convention: each section adds its own leading separator.
    /// - Level ends with a space
    /// - Event name (if any) prints space before itself
    /// - Body (if any) prints ": " before itself
    /// - Attributes (if any) print " [...]" before themselves
    pub fn format_log_line<V, L, E>(
        &self,
        w: &mut BufWriter<'_>,
        time: Option<SystemTime>,
        record: &V,
        format_level: L,
        format_event_name: E,
    ) where
        V: LogRecordView,
        L: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
        E: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
    {
        self.format_line_impl(
            w,
            time,
            format_level,
            format_event_name,
            |w, has_event_name| {
                Self::write_body_and_attrs(w, record, has_event_name);
            },
        );
    }

    /// Format a header line (RESOURCE, SCOPE) with attributes and custom formatters.
    ///
    /// Unlike `format_log_line`, this takes raw attributes instead of a LogRecordView,
    /// and doesn't print a body - just the header name and attributes.
    pub fn format_header_line<A, L, E>(
        &self,
        w: &mut BufWriter<'_>,
        time: Option<SystemTime>,
        attrs: impl Iterator<Item = A>,
        format_level: L,
        format_event_name: E,
    ) where
        A: AttributeView,
        L: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
        E: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
    {
        self.format_line_impl(
            w,
            time,
            format_level,
            format_event_name,
            |w, _has_event_name| {
                Self::write_attrs(w, attrs);
            },
        );
    }

    /// Common implementation for formatting a line with timestamp, level, event name, and content.
    fn format_line_impl<L, E, C>(
        &self,
        w: &mut BufWriter<'_>,
        time: Option<SystemTime>,
        format_level: L,
        format_event_name: E,
        write_content: C,
    ) where
        L: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
        E: FnOnce(&mut BufWriter<'_>, &ConsoleWriter),
        C: FnOnce(&mut BufWriter<'_>, bool),
    {
        // Timestamp (optional)
        if let Some(time) = time {
            self.write_styled(w, AnsiCode::Dim, |w| Self::write_timestamp(w, time));
            let _ = w.write_all(b"  ");
        }

        // Custom level/prefix formatting (tree structure, severity, etc.)
        format_level(w, self);

        // Track position to detect if event_name was written
        let pos_before = w.position();
        format_event_name(w, self);
        let has_event_name = w.position() > pos_before;

        // Write content (body+attrs or just attrs)
        write_content(w, has_event_name);

        let _ = w.write_all(b"\n");
    }

    /// Write a log line to stdout or stderr.
    fn write_line(&self, level: &Level, data: &[u8]) {
        let use_stderr = matches!(*level, Level::ERROR | Level::WARN);
        let _ = if use_stderr {
            std::io::stderr().write_all(data)
        } else {
            std::io::stdout().write_all(data)
        };
    }
}

impl<S> TracingLayer<S> for RawLoggingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // Allocates a buffer on the stack, formats the event to a LogRecord
    // with partial OTLP bytes.
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        self.dispatch_event(event);
    }

    // Note! This tracing layer does not implement Span-related features
    // available through LookupSpan. This is important future work and will
    // require introducing a notion of context. Presently, the Tokio tracing
    // Context does not pass through the OTAP dataflow engine.
    //
    // We are likely to issue span events as events, meaning not to build
    // Span objects at runtime.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_tracing::DirectLogRecordEncoder;
    use crate::self_tracing::encoder::level_to_severity_number;
    use bytes::Bytes;
    use otap_df_pdata::otlp::ProtoBuffer;
    use otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord as ProtoLogRecord;
    use prost::Message;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tracing_subscriber::prelude::*;

    struct CaptureLayer {
        formatted: Arc<Mutex<String>>,
        encoded: Arc<Mutex<Bytes>>,
    }

    impl<S> TracingLayer<S> for CaptureLayer
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            let time = SystemTime::now();
            let record = LogRecord::new(event);

            // Capture formatted output
            let writer = ConsoleWriter::no_color();
            *self.formatted.lock().unwrap() = writer.format_log_record(Some(time), &record);

            // Capture full OTLP encoding
            let mut buf = ProtoBuffer::with_capacity(512);
            let mut encoder = DirectLogRecordEncoder::new(&mut buf);
            let _ = encoder.encode_log_record(time, &record);
            *self.encoded.lock().unwrap() = buf.into_bytes();
        }
    }

    fn new_capture_layer() -> (CaptureLayer, Arc<Mutex<String>>, Arc<Mutex<Bytes>>) {
        let formatted = Arc::new(Mutex::new(String::new()));
        let encoded = Arc::new(Mutex::new(Bytes::new()));
        let layer = CaptureLayer {
            formatted: formatted.clone(),
            encoded: encoded.clone(),
        };
        (layer, formatted, encoded)
    }

    // strip timestamp and newline
    fn strip_ts(s: &str) -> (&str, &str) {
        // timestamp is 24 bytes, see assertion below.
        (&s[..24], s[26..].trim_end())
    }

    // helps test that a timestamp formats to text and from proto timestamp the same.
    fn format_timestamp(nanos: u64) -> String {
        let mut buf = [0u8; 32];
        let mut w = Cursor::new(buf.as_mut_slice());
        let time = std::time::UNIX_EPOCH + Duration::from_nanos(nanos);
        ConsoleWriter::write_timestamp(&mut w, time);
        let len = w.position() as usize;
        assert_eq!(len, 24);
        String::from_utf8_lossy(&buf[..len]).into_owned()
    }

    fn format_attrs(attrs: &[KeyValue]) -> String {
        if attrs.is_empty() {
            return String::new();
        }
        let mut result = String::from(" [");
        for (i, kv) in attrs.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&kv.key);
            result.push('=');
            if let Some(ref v) = kv.value {
                if let Some(ref val) = v.value {
                    match val {
                        Value::StringValue(s) => result.push_str(s),
                        Value::IntValue(i) => result.push_str(&i.to_string()),
                        Value::BoolValue(b) => result.push_str(if *b { "true" } else { "false" }),
                        Value::DoubleValue(d) => result.push_str(&format!("{:.6}", d)),
                        _ => unreachable!(),
                    }
                }
            }
        }
        result.push(']');
        result
    }

    fn assert_log_record(
        formatted: &Arc<Mutex<String>>,
        encoded: &Arc<Mutex<Bytes>>,
        expected_level: Level,
        expected_body: &str,
        expected_attrs: Vec<KeyValue>,
    ) {
        // Decode the OTLP bytes
        let bytes = encoded.lock().unwrap();
        let decoded = ProtoLogRecord::decode(bytes.as_ref()).expect("decode failed");

        // Verify OTLP encoding
        let sev_text = expected_level.as_str();
        assert_eq!(
            decoded.severity_number,
            level_to_severity_number(&expected_level) as i32,
            "severity_number mismatch"
        );
        // Severity text not coded in OTLP bytes form.
        assert!(decoded.severity_text.is_empty());
        assert_eq!(
            decoded.body,
            Some(AnyValue::new_string(expected_body)),
            "body mismatch"
        );
        assert_eq!(decoded.attributes, expected_attrs, "attributes mismatch");

        // Build expected text suffix
        let attrs_text = format_attrs(&expected_attrs);
        let expected_suffix = format!(": {}{}", expected_body, attrs_text);

        // Verify event_name has correct prefix. Note: file:line are not always available, not tested.
        let expected_prefix = "otap_df_telemetry::self_tracing::formatter::tests::event";
        assert!(
            decoded.event_name.starts_with(expected_prefix),
            "event_name should start with '{}', got: {}",
            expected_prefix,
            decoded.event_name
        );

        // Verify text formatting
        let binding = formatted.lock().unwrap();
        let (ts_str, rest) = strip_ts(&binding);

        // Verify timestamp matches OTLP value
        let expected_ts = format_timestamp(decoded.time_unix_nano);
        assert_eq!(ts_str, expected_ts, "timestamp mismatch");

        assert!(
            rest.starts_with(sev_text),
            "expected level '{}', got: {}",
            sev_text,
            rest
        );
        assert!(
            rest.ends_with(&expected_suffix),
            "expected suffix '{}', got: {}",
            expected_suffix,
            rest
        );
    }

    #[test]
    fn test_log_format() {
        let (layer, formatted, encoded) = new_capture_layer();
        let subscriber = tracing_subscriber::registry().with(layer);
        let dispatch = tracing::Dispatch::new(subscriber);
        let _guard = tracing::dispatcher::set_default(&dispatch);

        tracing::info!("hello world");
        assert_log_record(&formatted, &encoded, Level::INFO, "hello world", vec![]);

        tracing::warn!(count = 42i64, "something odd");
        assert_log_record(
            &formatted,
            &encoded,
            Level::WARN,
            "something odd",
            vec![KeyValue::new("count", AnyValue::new_int(42))],
        );

        tracing::error!(msg = "oops", "we failed");
        assert_log_record(
            &formatted,
            &encoded,
            Level::ERROR,
            "we failed",
            vec![KeyValue::new("msg", AnyValue::new_string("oops"))],
        );
    }

    #[test]
    fn test_timestamp_format() {
        // Create a specific timestamp: 2024-01-15T12:30:45.678Z
        let timestamp_ns: u64 = 1_705_321_845_678_000_000;
        let time = std::time::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);

        let record = LogRecord {
            callsite_id: tracing::callsite::Identifier(&TEST_CALLSITE),
            body_attrs_bytes: Bytes::new(),
            pipeline_entity_key: None,
            node_entity_key: None,
        };

        let writer = ConsoleWriter::no_color();
        let output = writer.format_log_record(Some(time), &record);

        // Note that the severity text is formatted using the Metadata::Level
        // so the text appears, unlike the protobuf case.
        assert_eq!(
            output,
            "2024-01-15T12:30:45.678Z  INFO  test_module::submodule::test_event (src/test.rs:123)\n"
        );

        let writer = ConsoleWriter::color();
        let output = writer.format_log_record(Some(time), &record);

        // With ANSI codes: dim timestamp, green INFO (padded to 5 chars), bold event name
        assert_eq!(
            output,
            "\x1b[2m2024-01-15T12:30:45.678Z\x1b[0m  \x1b[32mINFO \x1b[0m \x1b[1mtest_module::submodule::test_event (src/test.rs:123)\x1b[0m\n"
        );

        // Verify full OTLP encoding with known callsite
        let mut buf = ProtoBuffer::with_capacity(256);
        let mut encoder = DirectLogRecordEncoder::new(&mut buf);
        let _ = encoder.encode_log_record(time, &record);
        let decoded = ProtoLogRecord::decode(buf.into_bytes().as_ref()).expect("decode failed");

        assert_eq!(decoded.time_unix_nano, 1_705_321_845_678_000_000);
        assert_eq!(decoded.severity_number, 9); // INFO
        assert!(decoded.severity_text.is_empty()); // Not coded
        assert_eq!(
            decoded.event_name,
            "test_module::submodule::test_event (src/test.rs:123)"
        );
    }

    #[test]
    fn test_buffer_overflow() {
        let mut attrs = Vec::new();
        for i in 0..200 {
            attrs.push(KeyValue::new(
                format!("attribute_key_{:03}", i),
                AnyValue::new_string(format!("value_{:03}", i)),
            ));
        }

        let proto_record = ProtoLogRecord {
            body: Some(AnyValue::new_string("This is the log message body")),
            attributes: attrs,
            ..Default::default()
        };

        let mut encoded = Vec::new();
        proto_record.encode(&mut encoded).unwrap();

        // Create a specific timestamp: 2024-01-15T12:30:45.678Z
        let timestamp_ns: u64 = 1_705_321_845_678_000_000;
        let time = std::time::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);

        let record = LogRecord {
            callsite_id: tracing::callsite::Identifier(&TEST_CALLSITE),
            body_attrs_bytes: Bytes::from(encoded),
            pipeline_entity_key: None,
            node_entity_key: None,
        };

        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let writer = ConsoleWriter::no_color();
        let len = writer.format_log_record_into(&mut buf, Some(time), &record);

        // Fills exactly to capacity due to overflow.
        // Note! we could append a ... or some other indicator.
        assert_eq!(len, LOG_BUFFER_SIZE);

        // Verify the output starts correctly with timestamp and body
        let output = std::str::from_utf8(&buf[..len]).unwrap();
        assert!(
            output.starts_with("2024-01-15T12:30:45.678Z"),
            "got: {}",
            output
        );
        assert!(
            output.contains("This is the log message body"),
            "got: {}",
            output
        );
        assert!(
            output.contains("attribute_key_000=value_000"),
            "got: {}",
            output
        );
        assert!(
            output.contains("attribute_key_010=value_010"),
            "got: {}",
            output
        );
    }

    #[test]
    fn test_scope_continuation_line() {
        use crate::registry::EntityKey;
        use slotmap::SlotMap;

        // Create entity keys for testing
        let mut slot_map: SlotMap<EntityKey, ()> = SlotMap::with_key();
        let pipeline_key = slot_map.insert(());
        let node_key = slot_map.insert(());

        // Create a specific timestamp: 2024-01-15T12:30:45.678Z
        let timestamp_ns: u64 = 1_705_321_845_678_000_000;
        let time = std::time::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);

        // Test with both entity keys
        let record = LogRecord {
            callsite_id: tracing::callsite::Identifier(&TEST_CALLSITE),
            body_attrs_bytes: Bytes::new(),
            pipeline_entity_key: Some(pipeline_key),
            node_entity_key: Some(node_key),
        };

        let writer = ConsoleWriter::no_color();
        let output = writer.format_log_record(Some(time), &record);

        // Should have two lines
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2, "expected 2 lines, got: {:?}", lines);

        // First line is the normal log line
        assert!(
            lines[0].starts_with("2024-01-15T12:30:45.678Z"),
            "got: {}",
            lines[0]
        );

        // Second line is the scope continuation with entity keys
        assert!(
            lines[1].contains("scope"),
            "second line should contain 'scope', got: {}",
            lines[1]
        );
        assert!(
            lines[1].contains("pipeline="),
            "should contain pipeline key, got: {}",
            lines[1]
        );
        assert!(
            lines[1].contains("node="),
            "should contain node key, got: {}",
            lines[1]
        );

        // Test with only pipeline key
        let record_pipeline_only = LogRecord {
            callsite_id: tracing::callsite::Identifier(&TEST_CALLSITE),
            body_attrs_bytes: Bytes::new(),
            pipeline_entity_key: Some(pipeline_key),
            node_entity_key: None,
        };

        let output = writer.format_log_record(Some(time), &record_pipeline_only);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2, "expected 2 lines with pipeline only");
        assert!(
            lines[1].contains("pipeline=") && !lines[1].contains("node="),
            "should only have pipeline, got: {}",
            lines[1]
        );

        // Test with no entity keys (should be single line)
        let record_no_entity = LogRecord {
            callsite_id: tracing::callsite::Identifier(&TEST_CALLSITE),
            body_attrs_bytes: Bytes::new(),
            pipeline_entity_key: None,
            node_entity_key: None,
        };

        let output = writer.format_log_record(Some(time), &record_no_entity);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines.len(),
            1,
            "expected 1 line without entity context, got: {:?}",
            lines
        );

        // Test Grouped mode - scope continuation should be suppressed
        let output = writer.format_log_record(Some(time), &record);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines.len(),
            1,
            "Grouped mode should produce 1 line even with entity context, got: {:?}",
            lines
        );
        assert!(
            !output.contains("scope"),
            "Grouped mode should not contain scope line, got: {}",
            output
        );
    }

    static TEST_CALLSITE: TestCallsite = TestCallsite;
    struct TestCallsite;

    impl tracing::Callsite for TestCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            &TEST_METADATA
        }
    }

    static TEST_METADATA: tracing::Metadata<'static> = tracing::Metadata::new(
        "test_event",
        "test_module::submodule",
        Level::INFO,
        Some("src/test.rs"),
        Some(123),
        Some("test_module::submodule"),
        tracing::field::FieldSet::new(&[], tracing::callsite::Identifier(&TEST_CALLSITE)),
        tracing::metadata::Kind::EVENT,
    );
}
