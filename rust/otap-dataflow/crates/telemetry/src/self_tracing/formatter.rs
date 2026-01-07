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
pub const LOG_BUFFER_SIZE: usize = 4096;

/// Console formatter writes to stdout or stderr.
#[derive(Debug, Clone, Copy)]
pub struct ConsoleWriter {
    use_ansi: bool,
}

/// A minimal formatting layer that outputs log records to stdout/stderr.
///
/// This is a lightweight alternative to `tracing_subscriber::fmt::layer()`.
pub struct RawLayer {
    writer: ConsoleWriter,
}

// ANSI SGR (Select Graphic Rendition) codes
const ANSI_RESET: u8 = 0;
const ANSI_BOLD: u8 = 1;
const ANSI_DIM: u8 = 2;
const ANSI_RED: u8 = 31;
const ANSI_GREEN: u8 = 32;
const ANSI_YELLOW: u8 = 33;
const ANSI_BLUE: u8 = 34;
const ANSI_MAGENTA: u8 = 35;

impl RawLayer {
    /// Return a new formatting layer with associated writer.
    pub fn new(writer: ConsoleWriter) -> Self {
        Self { writer }
    }
}

/// Type alias for a cursor over a byte buffer.
/// Uses `std::io::Cursor` for position tracking with `std::io::Write`.
pub type BufWriter<'a> = Cursor<&'a mut [u8]>;

impl ConsoleWriter {
    /// Create a writer that outputs to stdout without ANSI colors.
    pub fn no_color() -> Self {
        Self { use_ansi: false }
    }

    /// Create a writer that outputs to stderr with ANSI colors.
    pub fn color() -> Self {
        Self { use_ansi: true }
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

    /// Write a LogRecord to a byte buffer. Returns the number of bytes written.
    ///
    /// This is the efficient path - no heap allocation, writes directly to the buffer.
    pub fn write_log_record(
        &self,
        buf: &mut [u8],
        record: &LogRecord,
        callsite: &SavedCallsite,
    ) -> usize {
        let mut w = Cursor::new(buf);

        if self.use_ansi {
            Self::write_ansi(&mut w, ANSI_DIM);
            Self::write_timestamp(&mut w, record.timestamp_ns);
            Self::write_ansi(&mut w, ANSI_RESET);
            let _ = w.write_all(b"  ");
            Self::write_ansi(&mut w, Self::level_color(callsite.level));
            Self::write_level(&mut w, callsite.level);
            Self::write_ansi(&mut w, ANSI_RESET);
            let _ = w.write_all(b"  ");
            Self::write_ansi(&mut w, ANSI_BOLD);
            Self::write_event_name(&mut w, callsite);
            Self::write_ansi(&mut w, ANSI_RESET);
            let _ = w.write_all(b": ");
        } else {
            Self::write_timestamp(&mut w, record.timestamp_ns);
            let _ = w.write_all(b"  ");
            Self::write_level(&mut w, callsite.level);
            let _ = w.write_all(b"  ");
            Self::write_event_name(&mut w, callsite);
            let _ = w.write_all(b": ");
        }

        Self::write_body_attrs(&mut w, &record.body_attrs_bytes);
        let _ = w.write_all(b"\n");

        w.position() as usize
    }

    /// Write level with padding.
    #[inline]
    fn write_level(w: &mut BufWriter<'_>, level: &Level) {
        let _ = match *level {
            Level::TRACE => w.write_all(b"TRACE"),
            Level::DEBUG => w.write_all(b"DEBUG"),
            Level::INFO => w.write_all(b"INFO "),
            Level::WARN => w.write_all(b"WARN "),
            Level::ERROR => w.write_all(b"ERROR"),
        };
    }

    /// Write callsite details as event_name to buffer.
    #[inline]
    fn write_event_name(w: &mut BufWriter<'_>, callsite: &SavedCallsite) {
        let _ = w.write_all(callsite.target.as_bytes());
        let _ = w.write_all(b"::");
        let _ = w.write_all(callsite.name.as_bytes());
        if let (Some(file), Some(line)) = (callsite.file, callsite.line) {
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
    fn write_line(&self, level: &Level, data: &[u8]) {
        let use_stderr = matches!(*level, Level::ERROR | Level::WARN);
        let _ = if use_stderr {
            std::io::stderr().write_all(data)
        } else {
            std::io::stdout().write_all(data)
        };
    }

    /// Write an ANSI escape sequence.
    #[inline]
    fn write_ansi(w: &mut BufWriter<'_>, code: u8) {
        let _ = write!(w, "\x1b[{}m", code);
    }

    /// Get ANSI color code for a severity level.
    #[inline]
    fn level_color(level: &Level) -> u8 {
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

        // Allocate buffer on stack and format directly
        let callsite = SavedCallsite::new(metadata);
        let mut buf = [0u8; LOG_BUFFER_SIZE];
        let len = self.writer.write_log_record(&mut buf, &record, &callsite);
        self.writer.write_line(callsite.level, &buf[..len]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;

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

    #[test]
    fn test_log_format() {
        // Test the formatter by capturing output through our layer
        use std::sync::{Arc, Mutex};

        let output: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let output_clone = output.clone();

        struct CaptureLayer {
            output: Arc<Mutex<String>>,
        }
        impl<S> TracingLayer<S> for CaptureLayer
        where
            S: Subscriber + for<'a> LookupSpan<'a>,
        {
            fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
                let record = LogRecord::new(event);
                let callsite = SavedCallsite::new(event.metadata());
                let writer = ConsoleWriter::no_color();
                *self.output.lock().unwrap() = writer.format_log_record(&record, &callsite);
            }
        }

        // Helper to strip timestamp (24 char timestamp + 2 spaces)
        fn strip_ts(s: &str) -> &str {
            s[26..].trim_end()
        }

        let layer = CaptureLayer { output: output_clone };
        let subscriber = tracing_subscriber::registry().with(layer);
        let dispatch = tracing::Dispatch::new(subscriber);
        let _guard = tracing::dispatcher::set_default(&dispatch);

        // Test info
        tracing::info!("hello world");
        let binding = output.lock().unwrap();
        let result = strip_ts(&binding);
        assert!(result.starts_with("INFO "), "got: {}", result);
        assert!(result.ends_with(": hello world"), "got: {}", result);
        drop(binding);

        // Test warn with attribute
        tracing::warn!(count = 42, "warning");
        let binding = output.lock().unwrap();
        let result = strip_ts(&binding);
        assert!(result.starts_with("WARN "), "got: {}", result);
        assert!(result.ends_with(": warning [count=42]"), "got: {}", result);
        drop(binding);

        // Test error with string attribute
        tracing::error!(msg = "oops", "failed");
        let binding = output.lock().unwrap();
        let result = strip_ts(&binding);
        assert!(result.starts_with("ERROR"), "got: {}", result);
        assert!(result.ends_with(": failed [msg=oops]"), "got: {}", result);
    }

    #[test]
    fn test_full_format_with_known_timestamp() {
        use bytes::Bytes;

        let record = LogRecord {
            callsite_id: tracing::callsite::Identifier(&TEST_CALLSITE),
            // 2024-01-15T12:30:45.678Z
            timestamp_ns: 1705321845_678_000_000,
            body_attrs_bytes: Bytes::new(),
        };

        let callsite = SavedCallsite {
            target: "my_crate::module",
            name: "event",
            file: Some("src/lib.rs"),
            line: Some(42),
            level: &Level::INFO,
        };

        let writer = ConsoleWriter::no_color();
        let output = writer.format_log_record(&record, &callsite);

        assert_eq!(
            output,
            "2024-01-15T12:30:45.678Z  INFO   my_crate::module::event (src/lib.rs:42): \n"
        );
    }

    static TEST_CALLSITE: TestCallsite = TestCallsite;
    struct TestCallsite;
    impl tracing::Callsite for TestCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            unimplemented!()
        }
    }
}
