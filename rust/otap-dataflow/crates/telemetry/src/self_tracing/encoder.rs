// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct OTLP bytes encoder for tokio-tracing events.

use std::fmt::Write as FmtWrite;
use std::time::SystemTime;

use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_pdata::proto::consts::{
    field_num::common::*, field_num::logs::*, field_num::resource::*, wire_types,
};
use otap_df_pdata::proto_encode_len_delimited_unknown_size;
use tracing::Level;

use super::{LogRecord, SavedCallsite};
use crate::event::LogEvent;

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
    pub fn encode_log_record(&mut self, time: SystemTime, record: &LogRecord) -> usize {
        let start_len = self.buf.len();

        // Convert SystemTime to nanoseconds since UNIX epoch
        let timestamp_ns = time
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        // Encode time_unix_nano (field 1, fixed64)
        self.buf
            .encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
        self.buf.extend_from_slice(&timestamp_ns.to_le_bytes());

        // Encode severity_number (field 2, varint)
        let severity = level_to_severity_number(record.callsite().level());
        self.buf
            .encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
        self.buf.encode_varint(severity as u64);

        // Node we skip encoding severity_text (field 3, string)

        // Encode event_name (field 12, string) - format: "target::name (file:line)"
        encode_event_name(self.buf, record.callsite());

        self.buf.extend_from_slice(&record.body_attrs_bytes);

        self.buf.len() - start_len
    }
}

/// Encode the event name from callsite metadata.
/// Format: "target::name (file:line)" or "target::name" if no file/line.
fn encode_event_name(buf: &mut ProtoBuffer, callsite: SavedCallsite) {
    proto_encode_len_delimited_unknown_size!(
        LOG_RECORD_EVENT_NAME,
        {
            buf.extend_from_slice(callsite.target().as_bytes());
            buf.extend_from_slice(b"::");
            buf.extend_from_slice(callsite.name().as_bytes());
            if let (Some(file), Some(line)) = (callsite.file(), callsite.line()) {
                let _ = write!(ProtoWriter(buf), " ({}:{})", file, line);
            }
        },
        buf
    );
}

/// Wrapper that implements fmt::Write for a ProtoBuffer.
struct ProtoWriter<'a>(&'a mut ProtoBuffer);

impl FmtWrite for ProtoWriter<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
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
            let _ = write!(ProtoWriter(buf), "{:?}", value);
        },
        buf
    );
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
#[must_use]
pub fn level_to_severity_number(level: &Level) -> u8 {
    match *level {
        Level::TRACE => 1,
        Level::DEBUG => 5,
        Level::INFO => 9,
        Level::WARN => 13,
        Level::ERROR => 17,
    }
}

/// Encode an SDK Resource as OTLP Resource bytes (field 1 of ResourceLogs).
///
/// The buffer is NOT cleared; bytes are appended.
pub fn encode_resource<'a, I>(buf: &mut ProtoBuffer, attrs: I, schema_url: Option<&str>)
where
    I: Iterator<Item = (&'a opentelemetry::Key, &'a opentelemetry::Value)>,
{
    // ResourceLogs.resource (field 1, Resource message)
    proto_encode_len_delimited_unknown_size!(
        RESOURCE_LOGS_RESOURCE,
        {
            // Encode each attribute as a KeyValue
            for (key, value) in attrs {
                encode_resource_attribute(buf, key.as_str(), value);
            }
        },
        buf
    );

    // ResourceLogs.schema_url (field 3, string)
    if let Some(url) = schema_url {
        buf.encode_string(RESOURCE_LOGS_SCHEMA_URL, url);
    }
}

/// Encode an SDK Resource to bytes for later reuse.
#[must_use]
pub fn encode_resource_to_bytes(resource: &opentelemetry_sdk::Resource) -> bytes::Bytes {
    let mut buf = ProtoBuffer::with_capacity(256);
    encode_resource(&mut buf, resource.iter(), resource.schema_url());
    bytes::Bytes::copy_from_slice(buf.as_ref())
}

/// Encode a single resource attribute as a KeyValue message.
#[inline]
fn encode_resource_attribute(buf: &mut ProtoBuffer, key: &str, value: &opentelemetry::Value) {
    use opentelemetry::Value;

    proto_encode_len_delimited_unknown_size!(
        RESOURCE_ATTRIBUTES,
        {
            buf.encode_string(KEY_VALUE_KEY, key);
            proto_encode_len_delimited_unknown_size!(
                KEY_VALUE_VALUE,
                {
                    match value {
                        Value::String(s) => {
                            buf.encode_string(ANY_VALUE_STRING_VALUE, s.as_str());
                        }
                        Value::Bool(b) => {
                            buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
                            buf.encode_varint(u64::from(*b));
                        }
                        Value::I64(i) => {
                            buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
                            buf.encode_varint(*i as u64);
                        }
                        Value::F64(f) => {
                            buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
                            buf.extend_from_slice(&f.to_le_bytes());
                        }
                        _ => {
                            // TODO: share the encoding logic used somewhere else, somehow.
                            crate::raw_error!("cannot encode SDK resource value", value = ?value);
                        }
                    }
                },
                buf
            );
        },
        buf
    );
}

// Field numbers for ExportLogsServiceRequest and related messages
const EXPORT_LOGS_REQUEST_RESOURCE_LOGS: u64 = 1;

/// Encode a LogEvent as a complete ExportLogsServiceRequest.
pub fn encode_export_logs_request(
    buf: &mut ProtoBuffer,
    event: &LogEvent,
    resource_bytes: &bytes::Bytes,
) {
    buf.clear();

    // ExportLogsServiceRequest.resource_logs (field 1, repeated ResourceLogs)
    proto_encode_len_delimited_unknown_size!(
        EXPORT_LOGS_REQUEST_RESOURCE_LOGS,
        {
            // ResourceLogs.resource (field 1, Resource message)
            // Copy pre-encoded resource bytes directly
            buf.extend_from_slice(resource_bytes);

            // ResourceLogs.scope_logs (field 2, repeated ScopeLogs)
            proto_encode_len_delimited_unknown_size!(
                RESOURCE_LOGS_SCOPE_LOGS,
                {
                    // ScopeLogs.scope: TODO: add scope attributes referring to
                    // component identity or somehow produce instrumentation scope.

                    // ScopeLogs.log_records (field 2, repeated LogRecord)
                    proto_encode_len_delimited_unknown_size!(
                        SCOPE_LOGS_LOG_RECORDS,
                        {
                            // Encode the LogRecord fields
                            let mut encoder = DirectLogRecordEncoder::new(buf);
                            let _ = encoder.encode_log_record(event.time, &event.record);
                        },
                        buf
                    );
                },
                buf
            );
        },
        buf
    );
}
