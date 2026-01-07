// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct OTLP bytes encoder for tokio-tracing events.

use std::fmt::Write as FmtWrite;

use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_pdata::proto::consts::{field_num::common::*, field_num::logs::*, wire_types};
use otap_df_pdata::proto_encode_len_delimited_unknown_size;
use tracing::Level;

use super::{LogRecord, SavedCallsite};

/// Direct encoder that writes a single LogRecord from a tracing Event.
pub struct DirectLogRecordEncoder<'buf> {
    buf: &'buf mut ProtoBuffer,
}

impl<'buf> DirectLogRecordEncoder<'buf> {
    /// Create a new encoder that writes to the provided buffer.
    #[inline]
    pub fn new(buf: &'buf mut ProtoBuffer) -> Self {
        Self { buf }
    }

    /// Reset the underlying buffer.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Encode a tracing Event as a complete LogRecord message.
    ///
    /// Returns the number of bytes written.
    pub fn encode_log_record(&mut self, record: LogRecord, callsite: &SavedCallsite) -> usize {
        let start_len = self.buf.len();

        // Encode time_unix_nano (field 1, fixed64)
        self.buf
            .encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
        self.buf
            .extend_from_slice(&record.timestamp_ns.to_le_bytes());

        // Encode severity_number (field 2, varint)
        let severity = level_to_severity_number(&callsite.level);
        self.buf
            .encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
        self.buf.encode_varint(severity as u64);

        // Encode severity_text (field 3, string)
        self.buf
            .encode_string(LOG_RECORD_SEVERITY_TEXT, callsite.level.as_str());

        self.buf.extend_from_slice(&record.body_attrs_bytes);

        self.buf.len() - start_len
    }
}

/// Visitor that directly encodes tracing fields to protobuf.
pub struct DirectFieldVisitor<'buf> {
    buf: &'buf mut ProtoBuffer,
}

impl<'buf> DirectFieldVisitor<'buf> {
    /// Create a new DirectFieldVisitor that writes to the provided buffer.
    pub fn new(buf: &'buf mut ProtoBuffer) -> Self {
        Self { buf }
    }

    /// Encode an attribute (KeyValue message) with a string value.
    #[inline]
    pub fn encode_string_attribute(&mut self, key: &str, value: &str) {
        proto_encode_len_delimited_unknown_size!(
            LOG_RECORD_ATTRIBUTES,
            {
                self.buf.encode_string(KEY_VALUE_KEY, key);
                proto_encode_len_delimited_unknown_size!(
                    KEY_VALUE_VALUE,
                    {
                        self.buf.encode_string(ANY_VALUE_STRING_VALUE, value);
                    },
                    self.buf
                );
            },
            self.buf
        );
    }

    /// Encode an attribute with an i64 value.
    #[inline]
    pub fn encode_int_attribute(&mut self, key: &str, value: i64) {
        proto_encode_len_delimited_unknown_size!(
            LOG_RECORD_ATTRIBUTES,
            {
                self.buf.encode_string(KEY_VALUE_KEY, key);
                proto_encode_len_delimited_unknown_size!(
                    KEY_VALUE_VALUE,
                    {
                        self.buf
                            .encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
                        self.buf.encode_varint(value as u64);
                    },
                    self.buf
                );
            },
            self.buf
        );
    }

    /// Encode an attribute with a bool value.
    #[inline]
    pub fn encode_bool_attribute(&mut self, key: &str, value: bool) {
        proto_encode_len_delimited_unknown_size!(
            LOG_RECORD_ATTRIBUTES,
            {
                self.buf.encode_string(KEY_VALUE_KEY, key);
                proto_encode_len_delimited_unknown_size!(
                    KEY_VALUE_VALUE,
                    {
                        self.buf
                            .encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
                        self.buf.encode_varint(u64::from(value));
                    },
                    self.buf
                );
            },
            self.buf
        );
    }

    /// Encode an attribute with a double value.
    #[inline]
    pub fn encode_double_attribute(&mut self, key: &str, value: f64) {
        proto_encode_len_delimited_unknown_size!(
            LOG_RECORD_ATTRIBUTES,
            {
                self.buf.encode_string(KEY_VALUE_KEY, key);
                proto_encode_len_delimited_unknown_size!(
                    KEY_VALUE_VALUE,
                    {
                        self.buf
                            .encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
                        self.buf.extend_from_slice(&value.to_le_bytes());
                    },
                    self.buf
                );
            },
            self.buf
        );
    }

    /// Encode the body (AnyValue message) as a string.
    #[inline]
    pub fn encode_body_string(&mut self, value: &str) {
        proto_encode_len_delimited_unknown_size!(
            LOG_RECORD_BODY,
            {
                self.buf.encode_string(ANY_VALUE_STRING_VALUE, value);
            },
            self.buf
        );
    }

    /// Encode the body (AnyValue message) from a Debug value without allocation.
    #[inline]
    pub fn encode_body_debug(&mut self, value: &dyn std::fmt::Debug) {
        proto_encode_len_delimited_unknown_size!(
            LOG_RECORD_BODY,
            {
                encode_debug_string(self.buf, value);
            },
            self.buf
        );
    }

    /// Encode an attribute with a Debug value without allocation.
    #[inline]
    pub fn encode_debug_attribute(&mut self, key: &str, value: &dyn std::fmt::Debug) {
        proto_encode_len_delimited_unknown_size!(
            LOG_RECORD_ATTRIBUTES,
            {
                self.buf.encode_string(KEY_VALUE_KEY, key);
                proto_encode_len_delimited_unknown_size!(
                    KEY_VALUE_VALUE,
                    {
                        encode_debug_string(self.buf, value);
                    },
                    self.buf
                );
            },
            self.buf
        );
    }
}

/// Helper to encode a Debug value as a protobuf string field.
/// This is separate from DirectFieldVisitor to avoid borrow conflicts with the macro.
#[inline]
fn encode_debug_string(buf: &mut ProtoBuffer, value: &dyn std::fmt::Debug) {
    proto_encode_len_delimited_unknown_size!(
        ANY_VALUE_STRING_VALUE,
        {
            let _ = write!(DebugWriter(buf), "{:?}", value);
        },
        buf
    );
}

/// Wrapper that implements fmt::Write for a ProtoBuffer.
struct DebugWriter<'a>(&'a mut ProtoBuffer);

impl FmtWrite for DebugWriter<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl tracing::field::Visit for DirectFieldVisitor<'_> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if field.name() == "message" {
            // TODO: encode f64 body
            return;
        }
        self.encode_double_attribute(field.name(), value);
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "message" {
            // TODO: encode i64 body
            return;
        }
        self.encode_int_attribute(field.name(), value);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "message" {
            // TODO: encode u64 body
            return;
        }
        self.encode_int_attribute(field.name(), value as i64);
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if field.name() == "message" {
            // TODO: encode bool body
            return;
        }
        self.encode_bool_attribute(field.name(), value);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.encode_body_string(value);
            return;
        }
        self.encode_string_attribute(field.name(), value);
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // The Rust Debug type cannot be destructured, only formatted.
        if field.name() == "message" {
            self.encode_body_debug(value);
        } else {
            self.encode_debug_attribute(field.name(), value);
        }
    }
}

/// Convert tracing Level to OTLP severity number.
///
/// See: https://opentelemetry.io/docs/specs/otel/logs/data-model/#field-severitynumber
#[inline]
pub fn level_to_severity_number(level: &Level) -> u8 {
    match *level {
        Level::TRACE => 1,
        Level::DEBUG => 5,
        Level::INFO => 9,
        Level::WARN => 13,
        Level::ERROR => 17,
    }
}
