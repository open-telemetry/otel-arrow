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
    pub const fn new(buf: &'buf mut ProtoBuffer) -> Self {
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
    pub const fn new(buf: &'buf mut ProtoBuffer) -> Self {
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
pub const fn level_to_severity_number(level: &Level) -> u8 {
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

        let visited = self.registry.visit_entity(key, |attrs| {
            attrs
                .iter_attributes()
                .map(|(a, b)| (a, b.clone()))
                .collect::<Vec<_>>()
        });
        visited
            .map(|attrs| {
                let mut buf = ProtoBuffer::with_capacity(128);
                for (attr_key, attr_value) in attrs {
                    encode_scope_attribute(&mut buf, attr_key, &attr_value);
                }
                let bytes = buf.into_bytes();
                let _ = self.cache.insert(key, bytes.clone());
                bytes
            })
            .unwrap_or_default()
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
    encode_key_value(buf, INSTRUMENTATION_SCOPE_ATTRIBUTES, key, value);
}

/// Encode a KeyValue message wrapped in the given outer field tag.
///
/// The outer field tag determines context:
/// - `INSTRUMENTATION_SCOPE_ATTRIBUTES` (field 3) for scope attributes
/// - `KEY_VALUE_LIST_VALUES` (field 1) for entries inside a kvlist
#[inline]
fn encode_key_value(
    buf: &mut ProtoBuffer,
    outer_field: u64,
    key: &str,
    value: &crate::attributes::AttributeValue,
) {
    proto_encode_len_delimited_unknown_size!(
        outer_field,
        {
            buf.encode_string(KEY_VALUE_KEY, key);
            proto_encode_len_delimited_unknown_size!(
                KEY_VALUE_VALUE,
                {
                    encode_any_value(buf, value);
                },
                buf
            );
        },
        buf
    );
}

/// Encode an `AttributeValue` as an OTLP AnyValue (the inner value without key wrapping).
#[inline]
fn encode_any_value(buf: &mut ProtoBuffer, value: &crate::attributes::AttributeValue) {
    use crate::attributes::AttributeValue;

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
        AttributeValue::Map(m) => {
            // Encode as kvlist: AnyValue.kvlist_value (field 6) containing
            // KeyValueList with repeated KeyValue entries (field 1).
            proto_encode_len_delimited_unknown_size!(
                ANY_VALUE_KVLIST_VALUE,
                {
                    for (k, v) in m {
                        encode_key_value(buf, KEY_VALUE_LIST_VALUES, k, v);
                    }
                },
                buf
            );
        }
    }
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
    use crate::__log_record_impl;
    use crate::LogContext;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};
    use crate::event::LogEvent;
    use crate::self_tracing::formatter::format_log_record_to_string;
    use opentelemetry::KeyValue as OTelKeyValue;
    use opentelemetry_sdk::Resource as OTelResource;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use otap_df_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
    use otap_df_pdata::proto::opentelemetry::logs::v1::ScopeLogs;
    use otap_df_pdata::proto::opentelemetry::logs::v1::SeverityNumber;
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use prost::Message;
    use std::collections::BTreeMap;
    use std::time::{Duration, SystemTime};
    use tracing::Level;

    static TEST_SCOPE_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "TestScope",
        fields: &[
            AttributeField {
                key: "pipeline.name",
                r#type: AttributeValueType::String,
                brief: "Pipeline name",
            },
            AttributeField {
                key: "cpu.id",
                r#type: AttributeValueType::Int,
                brief: "CPU ID",
            },
        ],
    };

    /// Mock attribute set for testing scope attributes.
    #[derive(Debug)]
    struct TestScopeAttributes {
        values: Vec<AttributeValue>,
    }

    impl TestScopeAttributes {
        fn new(name: &str, id: i64) -> Self {
            Self {
                // Note: order matches the AttributeFields.
                values: vec![AttributeValue::String(name.into()), AttributeValue::Int(id)],
            }
        }
    }

    impl AttributeSetHandler for TestScopeAttributes {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &TEST_SCOPE_ATTRIBUTES_DESCRIPTOR
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    #[test]
    fn encode_resource_to_bytes_encodes_attributes() {
        // Empty resource produces output
        let empty = encode_resource_to_bytes(&OTelResource::builder_empty().build());
        assert!(!empty.is_empty());

        // Resource with attributes contains encoded values
        let resource = OTelResource::builder_empty()
            .with_attributes([OTelKeyValue::new("service.name", "test-svc")])
            .build();
        let bytes = encode_resource_to_bytes(&resource);
        assert!(bytes.windows(8).any(|w| w == b"test-svc"));
    }

    #[test]
    fn encode_export_logs_request_with_scope_attributes() {
        let registry = TelemetryRegistryHandle::new();
        let entity_key = registry.register_entity(TestScopeAttributes::new("my-pipeline", 3));
        let mut scope_cache = ScopeToBytesMap::new(registry.clone());

        // Use the macro to create a LogRecord, then override context with entity
        let mut record = __log_record_impl!(Level::INFO, "test.scope.encoding");
        record.context = LogContext::from_buf([entity_key]);

        // Create a LogEvent with known timestamp, empty resource.
        let timestamp_ns: u64 = 1_705_321_845_000_000_000;
        let time = SystemTime::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);
        let log_event = LogEvent { time, record };

        let resource_bytes = encode_resource_to_bytes(&OTelResource::builder_empty().build());

        let mut buf = ProtoBuffer::with_capacity(512);
        encode_export_logs_request(&mut buf, &log_event, &resource_bytes, &mut scope_cache);

        let decoded = ExportLogsServiceRequest::decode(buf.into_bytes().as_ref()).unwrap();
        let event_name = &decoded
            .resource_logs
            .first()
            .unwrap()
            .scope_logs
            .first()
            .unwrap()
            .log_records
            .first()
            .unwrap()
            .event_name;

        // Test for the event name prefix to avoid a hard-coded line number.
        assert!(event_name.starts_with("otap-df-telemetry::test.scope.encoding"));

        let expected = ExportLogsServiceRequest::new([ResourceLogs::new(
            Resource::build().finish(),
            [ScopeLogs::new(
                InstrumentationScope::build()
                    .attributes([
                        KeyValue::new("pipeline.name", AnyValue::new_string("my-pipeline")),
                        KeyValue::new("cpu.id", AnyValue::new_int(3)),
                    ])
                    .finish(),
                [LogRecord::build()
                    .event_name(event_name) // from the decoded value
                    .time_unix_nano(timestamp_ns)
                    .severity_number(SeverityNumber::Info)
                    .finish()],
            )],
        )]);

        // Inspect the printed format. Entity name is appended.
        assert_eq!(
            format_log_record_to_string(None, &log_event.record),
            format!("INFO  {event_name} entity={:?}\n", entity_key),
        );
        assert_eq!(expected, decoded);
    }

    // --- Test infrastructure for Map (kvlist) scope attributes ---

    static TEST_MAP_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "TestMap",
        fields: &[AttributeField {
            key: "custom",
            r#type: AttributeValueType::Map,
            brief: "Custom user-defined attributes",
        }],
    };

    /// Mirrors engine::CustomAttributeSet: a single "custom" field of type Map.
    #[derive(Debug)]
    struct TestMapAttributes {
        values: Vec<AttributeValue>,
    }

    impl TestMapAttributes {
        fn new(map: BTreeMap<String, AttributeValue>) -> Self {
            Self {
                values: vec![AttributeValue::Map(map)],
            }
        }
    }

    impl AttributeSetHandler for TestMapAttributes {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &TEST_MAP_ATTRIBUTES_DESCRIPTOR
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    /// Validate that Map attributes encode as OTLP kvlist values in
    /// InstrumentationScope.attributes. This mirrors what CustomAttributeSet
    /// does for user-defined node/pipeline attributes.
    #[test]
    fn encode_export_logs_request_with_map_scope_attribute() {
        let registry = TelemetryRegistryHandle::new();

        let mut custom_map = BTreeMap::new();
        let _ = custom_map.insert("priority".to_string(), AttributeValue::Int(5));
        let _ = custom_map.insert(
            "region".to_string(),
            AttributeValue::String("us-east-1".into()),
        );

        let entity_key = registry.register_entity(TestMapAttributes::new(custom_map));
        let mut scope_cache = ScopeToBytesMap::new(registry.clone());

        let mut record = __log_record_impl!(Level::INFO, "test.map.encoding");
        record.context = LogContext::from_buf([entity_key]);

        let timestamp_ns: u64 = 1_705_321_845_000_000_000;
        let time = SystemTime::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);
        let log_event = LogEvent { time, record };

        let resource_bytes = encode_resource_to_bytes(&OTelResource::builder_empty().build());
        let mut buf = ProtoBuffer::with_capacity(512);
        encode_export_logs_request(&mut buf, &log_event, &resource_bytes, &mut scope_cache);

        let decoded = ExportLogsServiceRequest::decode(buf.into_bytes().as_ref()).unwrap();
        let event_name = &decoded.resource_logs[0].scope_logs[0].log_records[0].event_name;
        assert!(event_name.starts_with("otap-df-telemetry::test.map.encoding"));

        // BTreeMap iterates in sorted key order: "priority" before "region".
        let expected = ExportLogsServiceRequest::new([ResourceLogs::new(
            Resource::build().finish(),
            [ScopeLogs::new(
                InstrumentationScope::build()
                    .attributes([KeyValue::new(
                        "custom",
                        AnyValue::new_kvlist(vec![
                            KeyValue::new("priority", AnyValue::new_int(5)),
                            KeyValue::new("region", AnyValue::new_string("us-east-1")),
                        ]),
                    )])
                    .finish(),
                [LogRecord::build()
                    .event_name(event_name)
                    .time_unix_nano(timestamp_ns)
                    .severity_number(SeverityNumber::Info)
                    .finish()],
            )],
        )]);

        assert_eq!(expected, decoded);
    }
}
