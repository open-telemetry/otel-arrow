// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! An alternative to Tokio fmt::layer().

use super::{LogRecord, SavedCallsite};
use bytes::Bytes;
use chrono::{DateTime, Datelike, Timelike, Utc};
use otap_df_pdata::views::common::{AnyValueView, AttributeView, ValueType};
use otap_df_pdata::views::logs::LogRecordView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogRecord;
use std::io::{Cursor, Write};
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
enum AnsiCode {
    Reset = 0,
    Bold = 1,
    Dim = 2,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
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

    /// Write level with color and padding.
    #[inline]
    fn write_level(self, w: &mut BufWriter<'_>, level: &Level) {
        self.write_ansi(w, Self::color(level));
        let _ = w.write_all(level.as_str().as_bytes());
        self.write_ansi(w, AnsiCode::Reset);
        let _ = w.write_all(b"  ");
    }

    /// Get ANSI color code for a severity level.
    #[inline]
    fn color(level: &Level) -> AnsiCode {
        match *level {
            Level::ERROR => AnsiCode::Red,
            Level::WARN => AnsiCode::Yellow,
            Level::INFO => AnsiCode::Green,
            Level::DEBUG => AnsiCode::Blue,
            Level::TRACE => AnsiCode::Magenta,
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
    pub fn format_log_record(&self, record: &LogRecord, callsite: &SavedCallsite) -> String {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let len = self.write_log_record(&mut buf, record, callsite);
        // The buffer contains valid UTF-8 since we only write ASCII and valid UTF-8 strings
        String::from_utf8_lossy(&buf[..len]).into_owned()
    }

    /// Write a LogRecord to stdout or stderr (based on level).
    ///
    /// ERROR and WARN go to stderr, others go to stdout.
    /// This is the same routing logic used by RawLoggingLayer.
    pub fn raw_print(&self, record: &LogRecord, callsite: &SavedCallsite) {
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let len = self.write_log_record(&mut buf, record, callsite);
        self.write_line(callsite.level(), &buf[..len]);
    }

    /// Write a LogRecord to a byte buffer. Returns the number of bytes written.
    pub(crate) fn write_log_record(
        &self,
        buf: &mut [u8],
        record: &LogRecord,
        callsite: &SavedCallsite,
    ) -> usize {
        let mut w = Cursor::new(buf);
        let cm = self.color_mode;

        cm.write_ansi(&mut w, AnsiCode::Dim);
        Self::write_timestamp(&mut w, record.timestamp_ns);
        cm.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b"  ");
        cm.write_level(&mut w, callsite.level());
        cm.write_ansi(&mut w, AnsiCode::Bold);
        Self::write_event_name(&mut w, callsite);
        cm.write_ansi(&mut w, AnsiCode::Reset);
        let _ = w.write_all(b": ");
        Self::write_body_attrs(&mut w, &record.body_attrs_bytes);
        let _ = w.write_all(b"\n");

        w.position() as usize
    }

    /// Write callsite details as event_name to buffer.
    #[inline]
    fn write_event_name(w: &mut BufWriter<'_>, callsite: &SavedCallsite) {
        let _ = w.write_all(callsite.target().as_bytes());
        let _ = w.write_all(b"::");
        let _ = w.write_all(callsite.name().as_bytes());
        if let (Some(file), Some(line)) = (callsite.file(), callsite.line()) {
            let _ = write!(w, " ({}:{})", file, line);
        }
    }

    /// Write nanosecond timestamp as ISO 8601 (UTC) to buffer.
    #[inline]
    fn write_timestamp(w: &mut BufWriter<'_>, nanos: u64) {
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

    /// Write body+attrs bytes to buffer using LogRecordView.
    fn write_body_attrs(w: &mut BufWriter<'_>, bytes: &Bytes) {
        if bytes.is_empty() {
            return;
        }

        // A partial protobuf message (just body + attributes) is still a valid message.
        // We can use the RawLogRecord view to access just the fields we encoded.
        let record = RawLogRecord::new(bytes.as_ref());

        // Write body if present
        if let Some(body) = record.body() {
            Self::write_any_value(w, &body);
        }

        // Write attributes if present
        let mut attrs = record.attributes().peekable();
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

    /// Write an AnyValue to buffer.
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

    /// Write a log line to stdout or stderr.
    pub(crate) fn write_line(&self, level: &Level, data: &[u8]) {
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
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // TODO: there are allocations implied here that we would prefer
        // to avoid, it will be an extensive change in the ProtoBuffer to
        // stack-allocate this temporary.
        // RawLoggingLayer is used before the logs infrastructure is set up,
        // so no producer_key context is available.
        let record = LogRecord::new(event, None);
        let callsite = SavedCallsite::new(event.metadata());
        self.writer.raw_print(&record, &callsite);
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
    use otap_df_pdata::prost::Message;
    use otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord as ProtoLogRecord;
    use std::sync::{Arc, Mutex};
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
            let record = LogRecord::new(event, None);
            let callsite = SavedCallsite::new(event.metadata());

            // Capture formatted output
            let writer = ConsoleWriter::no_color();
            *self.formatted.lock().unwrap() = writer.format_log_record(&record, &callsite);

            // Capture full OTLP encoding
            let mut buf = ProtoBuffer::with_capacity(512);
            let mut encoder = DirectLogRecordEncoder::new(&mut buf);
            let _ = encoder.encode_log_record(record, &callsite);
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

    fn format_timestamp(nanos: u64) -> String {
        let mut buf = [0u8; 32];
        let mut w = Cursor::new(buf.as_mut_slice());
        ConsoleWriter::write_timestamp(&mut w, nanos);
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
        assert_eq!(decoded.severity_text, sev_text, "severity_text mismatch");
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
        let record = LogRecord {
            callsite_id: tracing::callsite::Identifier(&TEST_CALLSITE),
            // 2024-01-15T12:30:45.678Z
            timestamp_ns: 1_705_321_845_678_000_000,
            body_attrs_bytes: Bytes::new(),
            producer_key: None,
        };

        let writer = ConsoleWriter::no_color();
        let output = writer.format_log_record(&record, &test_callsite());

        assert_eq!(
            output,
            "2024-01-15T12:30:45.678Z  INFO  test_module::submodule::test_event (src/test.rs:123): \n"
        );

        let writer = ConsoleWriter::color();
        let output = writer.format_log_record(&record, &test_callsite());

        // With ANSI codes: dim timestamp, green INFO, bold event name
        assert_eq!(
            output,
            "\x1b[2m2024-01-15T12:30:45.678Z\x1b[0m  \x1b[32mINFO\x1b[0m  \x1b[1mtest_module::submodule::test_event (src/test.rs:123)\x1b[0m: \n"
        );

        // Verify full OTLP encoding with known callsite
        let mut buf = ProtoBuffer::with_capacity(256);
        let mut encoder = DirectLogRecordEncoder::new(&mut buf);
        let _ = encoder.encode_log_record(record, &test_callsite());
        let decoded = ProtoLogRecord::decode(buf.into_bytes().as_ref()).expect("decode failed");

        assert_eq!(decoded.time_unix_nano, 1_705_321_845_678_000_000);
        assert_eq!(decoded.severity_number, 9); // INFO
        assert_eq!(decoded.severity_text, "INFO");
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

        let record = LogRecord {
            callsite_id: tracing::callsite::Identifier(&TEST_CALLSITE),
            timestamp_ns: 1_705_321_845_678_000_000,
            body_attrs_bytes: Bytes::from(encoded),
            producer_key: None,
        };

        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let writer = ConsoleWriter::no_color();
        let len = writer.write_log_record(&mut buf, &record, &test_callsite());

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

    fn test_callsite() -> SavedCallsite {
        SavedCallsite::new(&TEST_METADATA)
    }
}
