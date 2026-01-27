// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct OTLP bytes encoder for tokio-tracing events.

use super::{LogRecord, SavedCallsite};
use crate::event::LogEvent;
use crate::registry::EntityKey;
use crate::registry::TelemetryRegistryHandle;
use bytes::Bytes;
use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_pdata::proto::consts::{
    field_num::common::*, field_num::logs::*, field_num::resource::*, wire_types,
};
use otap_df_pdata::proto_encode_len_delimited_unknown_size;
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::Level;

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
            super::formatter::write_event_name_to(buf, &callsite);
        },
        buf
    );
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
    use std::io::Write;
    proto_encode_len_delimited_unknown_size!(
        ANY_VALUE_STRING_VALUE,
        {
            let _ = write!(buf, "{:?}", value);
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
fn encode_resource<'a, I>(buf: &mut ProtoBuffer, attrs: I, schema_url: Option<&str>)
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
pub fn encode_resource_to_bytes(resource: &opentelemetry_sdk::Resource) -> Bytes {
    let mut buf = ProtoBuffer::with_capacity(256);
    encode_resource(&mut buf, resource.iter(), resource.schema_url());
    buf.into_bytes()
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

/// Pre-encoded scope attribute bytes keyed by EntityKey.
///
/// The Internal Telemetry Receiver uses this to avoid re-encoding scope
/// attributes for each log event. Entity attributes are looked up once
/// from the registry and encoded as InstrumentationScope.attributes bytes.
///
/// TODO: ScopeToBytesMap grows without any attention to managing memory.
/// We will require a way to de-register entities that are no longer use
/// or to flush memory to maintain a fixed size.
#[derive(Debug)]
pub struct ScopeToBytesMap {
    cache: HashMap<EntityKey, Bytes>,
    registry: TelemetryRegistryHandle,
}

impl ScopeToBytesMap {
    /// Create a new empty scope attribute cache.
    #[must_use]
    pub fn new(registry: TelemetryRegistryHandle) -> Self {
        Self {
            cache: HashMap::new(),
            registry,
        }
    }

    /// Get or compute the encoded scope attribute bytes for an entity key.
    pub fn get_or_encode(&mut self, key: EntityKey) -> Bytes {
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        // Encode the entity attributes as KeyValue messages
        let mut buf = ProtoBuffer::with_capacity(128);
        self.registry.visit_entity(key, |attrs| {
            for (attr_key, attr_value) in attrs.iter_attributes() {
                encode_scope_attribute(&mut buf, attr_key, attr_value);
            }
        });

        let bytes = buf.into_bytes();
        let _ = self.cache.insert(key, bytes.clone());
        bytes
    }

    /// Clear the cache. Call this when entities may have been updated.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Encode a single scope attribute as a KeyValue message for InstrumentationScope.attributes.
#[inline]
fn encode_scope_attribute(
    buf: &mut ProtoBuffer,
    key: &str,
    value: &crate::attributes::AttributeValue,
) {
    use crate::attributes::AttributeValue;

    proto_encode_len_delimited_unknown_size!(
        INSTRUMENTATION_SCOPE_ATTRIBUTES,
        {
            buf.encode_string(KEY_VALUE_KEY, key);
            proto_encode_len_delimited_unknown_size!(
                KEY_VALUE_VALUE,
                {
                    match value {
                        AttributeValue::String(s) => {
                            buf.encode_string(ANY_VALUE_STRING_VALUE, s.as_str());
                        }
                        AttributeValue::Boolean(b) => {
                            buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
                            buf.encode_varint(u64::from(*b));
                        }
                        AttributeValue::Int(i) => {
                            buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
                            buf.encode_varint(*i as u64);
                        }
                        AttributeValue::UInt(u) => {
                            buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
                            buf.encode_varint(*u);
                        }
                        AttributeValue::Double(f) => {
                            buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
                            buf.extend_from_slice(&f.to_le_bytes());
                        }
                    }
                },
                buf
            );
        },
        buf
    );
}

/// Encode a LogEvent as a complete ExportLogsServiceRequest.
///
/// This version resolves entity keys from the log record's context to populate
/// the InstrumentationScope.attributes field. The scope cache is used to avoid
/// re-encoding entity attributes for each log event.
pub fn encode_export_logs_request(
    buf: &mut ProtoBuffer,
    event: &LogEvent,
    resource_bytes: &Bytes,
    scope_cache: &mut ScopeToBytesMap,
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
                    // ScopeLogs.scope (field 1, InstrumentationScope message)
                    // Encode scope with attributes from entity context
                    proto_encode_len_delimited_unknown_size!(
                        SCOPE_LOG_SCOPE,
                        {
                            // For each entity key in the log context, append its pre-encoded attributes
                            for entity_key in event.record.context.iter() {
                                let scope_bytes = scope_cache.get_or_encode(*entity_key);
                                buf.extend_from_slice(&scope_bytes);
                            }
                        },
                        buf
                    );

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

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::Resource;

    #[test]
    fn encode_resource_to_bytes_encodes_attributes() {
        // Empty resource produces output
        let empty = encode_resource_to_bytes(&Resource::builder_empty().build());
        assert!(!empty.is_empty());

        // Resource with attributes contains encoded values
        let resource = Resource::builder_empty()
            .with_attributes([KeyValue::new("service.name", "test-svc")])
            .build();
        let bytes = encode_resource_to_bytes(&resource);
        assert!(bytes.windows(8).any(|w| w == b"test-svc"));
    }
}
