// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP export request assembly for saved log records.
//!
//! Unlike [`super::encoder`], which only writes into size-bounded buffers,
//! this module builds whole requests in a growable [`ProtoBuffer`]. It is
//! the only place in `self_tracing` that may name that type.

use super::encoder::DirectLogRecordEncoder;
use crate::attributes::AttributeValue;
use crate::event::LogEvent;
use crate::registry::{EntityKey, TelemetryRegistryHandle};
use bytes::Bytes;
use otel_arrow_dfe_config::pipeline::telemetry::{
    AttributeValue as ConfigAttributeValue, AttributeValueArray as ConfigAttributeValueArray,
};
use otel_arrow_dfe_pdata::otlp::common::{BoundedBuf, EncodeFailure, EncodeResult, ProtoBuffer};
use otel_arrow_dfe_pdata::proto::consts::{
    field_num::common::*, field_num::logs::*, field_num::resource::*, wire_types,
};
use slotmap::Key;
use std::collections::HashMap;

/// Encode configured resource attributes as the resource fragment shared by
/// OTLP `ResourceLogs` and `ResourceMetrics` messages.
///
/// The fragment contains only the encoded `resource` field. Both OTLP message
/// types assign that field the same number, so callers can reuse these bytes
/// without constructing an intermediate SDK resource.
#[must_use]
pub(crate) fn encode_config_resource_field(
    resource_attributes: &HashMap<String, ConfigAttributeValue>,
) -> Bytes {
    let mut buf = ProtoBuffer::with_capacity(256);
    let _: EncodeResult = buf.encode_len_delimited(RESOURCE_LOGS_RESOURCE, |buf| {
        for (key, value) in resource_attributes {
            encode_config_resource_attribute(buf, key, value)?;
        }
        Ok(())
    });
    buf.into_bytes()
}

/// Encode one configured resource attribute as an OTLP `KeyValue`.
fn encode_config_resource_attribute(
    buf: &mut ProtoBuffer,
    key: &str,
    value: &ConfigAttributeValue,
) -> EncodeResult {
    buf.encode_len_delimited(RESOURCE_ATTRIBUTES, |buf| {
        buf.encode_string(KEY_VALUE_KEY, key)?;
        buf.encode_len_delimited(KEY_VALUE_VALUE, |buf| {
            encode_config_resource_value(buf, value)
        })
    })
}

/// Encode one configured resource value as an OTLP `AnyValue` payload.
fn encode_config_resource_value(
    buf: &mut ProtoBuffer,
    value: &ConfigAttributeValue,
) -> EncodeResult {
    match value {
        ConfigAttributeValue::String(value) => {
            buf.encode_string(ANY_VALUE_STRING_VALUE, value)?;
        }
        ConfigAttributeValue::Bool(value) => {
            buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT)?;
            buf.encode_varint(u64::from(*value))?;
        }
        ConfigAttributeValue::I64(value) => {
            buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
            buf.encode_varint(*value as u64)?;
        }
        ConfigAttributeValue::F64(value) => {
            buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64)?;
            buf.extend_from_slice(&value.to_le_bytes())?;
        }
        ConfigAttributeValue::Array(values) => {
            buf.encode_len_delimited(ANY_VALUE_ARRAY_VALUE, |buf| match values {
                ConfigAttributeValueArray::Bool(values) => {
                    encode_config_resource_array(buf, values, |buf, value| {
                        buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT)?;
                        buf.encode_varint(u64::from(*value))
                    })
                }
                ConfigAttributeValueArray::I64(values) => {
                    encode_config_resource_array(buf, values, |buf, value| {
                        buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
                        buf.encode_varint(*value as u64)
                    })
                }
                ConfigAttributeValueArray::F64(values) => {
                    encode_config_resource_array(buf, values, |buf, value| {
                        buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64)?;
                        buf.extend_from_slice(&value.to_le_bytes())
                    })
                }
                ConfigAttributeValueArray::String(values) => {
                    encode_config_resource_array(buf, values, |buf, value| {
                        buf.encode_string(ANY_VALUE_STRING_VALUE, value)
                    })
                }
            })?;
        }
    }
    Ok(())
}

/// Encode a homogeneous configured array as repeated OTLP `AnyValue` entries.
fn encode_config_resource_array<T>(
    buf: &mut ProtoBuffer,
    values: &[T],
    mut encode_value: impl FnMut(&mut ProtoBuffer, &T) -> EncodeResult,
) -> EncodeResult {
    for value in values {
        buf.encode_len_delimited(ARRAY_VALUE_VALUES, |buf| encode_value(buf, value))?;
    }
    Ok(())
}

// Field numbers for ExportLogsServiceRequest and related messages
const EXPORT_LOGS_REQUEST_RESOURCE_LOGS: u64 = 1;

/// Pre-encoded scope attribute bytes keyed by EntityKey.
///
/// The Internal Telemetry Receiver uses this to avoid re-encoding scope
/// attributes for each log event. Entity attributes are looked up once
/// from the registry and encoded as InstrumentationScope.attributes bytes.
/// Entity keys must remain registered until their first lookup populates this
/// cache; a key that is neither registered nor cached cannot be encoded.
///
/// TODO: ScopeToBytesMap grows without any attention to managing memory.
/// We will require a way to de-register entities that are no longer use
/// or to flush memory to maintain a fixed size.
#[derive(Debug)]
pub struct ScopeToBytesMap {
    cache: HashMap<EntityKey, Bytes>,
    registry: TelemetryRegistryHandle,
}

/// A dedicated error value indicating that an EntityKey was not
/// found.
pub enum MapError {
    /// EntityKey is not registered
    NotRegistered,
    /// Encoding the registered entity's attributes failed.
    Failed(EncodeFailure),
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

    /// Get or compute scope attributes for a registered or previously cached entity key.
    pub fn get_or_encode(&mut self, key: EntityKey) -> Result<Bytes, MapError> {
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.clone());
        }

        match self.registry.visit_entity(key, |attrs| {
            encode_scope_attributes(attrs.iter_attributes())
        }) {
            Some(Ok(bytes)) => {
                let _ = self.cache.insert(key, bytes.clone());
                Ok(bytes)
            }
            Some(Err(err)) => Err(MapError::Failed(err)),
            None => Err(MapError::NotRegistered),
        }
    }

    /// Clear the cache. Call this when entities may have been updated.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Encode an iterator of scope attributes.
fn encode_scope_attributes<'k, 'v>(
    attrs: impl IntoIterator<Item = (&'k str, &'v AttributeValue)>,
) -> Result<Bytes, EncodeFailure> {
    let mut buf = ProtoBuffer::with_capacity(128);
    for (attr_key, attr_value) in attrs {
        encode_scope_attribute(&mut buf, attr_key, attr_value)?;
    }
    Ok(buf.into_bytes())
}

/// Encode a single scope attribute as a KeyValue message for InstrumentationScope.attributes.
#[inline]
fn encode_scope_attribute(
    buf: &mut ProtoBuffer,
    key: &str,
    value: &AttributeValue,
) -> EncodeResult {
    encode_key_value(buf, INSTRUMENTATION_SCOPE_ATTRIBUTES, key, value)
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
    value: &AttributeValue,
) -> EncodeResult {
    buf.encode_len_delimited(outer_field, |buf| {
        buf.encode_string(KEY_VALUE_KEY, key)?;
        buf.encode_len_delimited(KEY_VALUE_VALUE, |buf| encode_any_value(buf, value))
    })
}

/// Encode an `AttributeValue` as an OTLP AnyValue (the inner value without key wrapping).
#[inline]
fn encode_any_value(buf: &mut ProtoBuffer, value: &AttributeValue) -> EncodeResult {
    match value {
        AttributeValue::String(s) => {
            buf.encode_string(ANY_VALUE_STRING_VALUE, s.as_str())?;
        }
        AttributeValue::Boolean(b) => {
            buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT)?;
            buf.encode_varint(u64::from(*b))?;
        }
        AttributeValue::Int(i) => {
            buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
            buf.encode_varint(*i as u64)?;
        }
        AttributeValue::UInt(u) => {
            buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
            buf.encode_varint(*u)?;
        }
        AttributeValue::Double(f) => {
            buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64)?;
            buf.extend_from_slice(&f.to_le_bytes())?;
        }
        AttributeValue::Map(m) => {
            // Encode as kvlist: AnyValue.kvlist_value (field 6) containing
            // KeyValueList with repeated KeyValue entries (field 1).
            buf.encode_len_delimited(ANY_VALUE_KVLIST_VALUE, |buf| {
                for (k, v) in m {
                    encode_key_value(buf, KEY_VALUE_LIST_VALUES, k, v)?;
                }
                Ok(())
            })?;
        }
    }

    Ok(())
}

fn encode_scope_logs(
    buf: &mut ProtoBuffer,
    target: &str,
    context: &[EntityKey],
    events: &[LogEvent],
    scope_cache: &mut ScopeToBytesMap,
) -> EncodeResult {
    buf.encode_len_delimited(RESOURCE_LOGS_SCOPE_LOGS, |buf| {
        buf.encode_len_delimited(SCOPE_LOG_SCOPE, |buf| {
            buf.encode_string(INSTRUMENTATION_SCOPE_NAME, target)?;
            for entity_key in context {
                match scope_cache.get_or_encode(*entity_key) {
                    Ok(b) => buf.extend_from_slice(&b)?,
                    Err(MapError::NotRegistered) => {
                        buf.extend_from_slice(&encode_scope_attributes(std::iter::once((
                            "unregistered.entity",
                            &AttributeValue::Int(entity_key.data().as_ffi() as i64),
                        )))?)?;
                    }
                    Err(MapError::Failed(err)) => return Err(err),
                }
            }
            Ok(())
        })?;

        for event in events {
            buf.encode_len_delimited(SCOPE_LOGS_LOG_RECORDS, |buf| {
                let mut encoder = DirectLogRecordEncoder::new(buf);
                let _ = encoder.encode_log_record(event.time, &event.record);
                Ok(())
            })?;
        }
        Ok(())
    })
}

/// Encode logs as one request, grouping by scope.
///
/// Every entity key referenced by an event must remain registered until its
/// scope attributes have been cached.
pub fn encode_export_logs_request(
    buf: &mut ProtoBuffer,
    events: &mut [LogEvent],
    resource_bytes: &Bytes,
    scope_cache: &mut ScopeToBytesMap,
) -> EncodeResult {
    buf.clear();
    if events.is_empty() {
        return Ok(());
    }

    // Sort the events in scope order.
    events.sort_by(|a, b| {
        a.record
            .callsite()
            .target()
            .cmp(b.record.callsite().target())
            .then_with(|| {
                a.record
                    .context
                    .iter()
                    .map(|key| key.data().as_ffi())
                    .cmp(b.record.context.iter().map(|key| key.data().as_ffi()))
            })
    });

    buf.encode_len_delimited(EXPORT_LOGS_REQUEST_RESOURCE_LOGS, |buf| {
        buf.extend_from_slice(resource_bytes)?;

        for events in events.chunk_by(|a, b| {
            a.record.callsite().target() == b.record.callsite().target()
                && a.record.context == b.record.context
        }) {
            let first = &events[0];

            encode_scope_logs(
                buf,
                first.record.callsite().target(),
                first.record.context.as_slice(),
                events,
                scope_cache,
            )?;
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::__log_record_impl;
    use crate::LogContext;
    use crate::attributes::AttributeSetHandler;
    use crate::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};
    use crate::self_tracing::format_log_record_to_string;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
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

    /// Scenario: configured scalar and array resource attributes are encoded directly.
    /// Guarantees: the reusable OTLP resource fragment preserves every supported value type.
    #[test]
    fn encode_config_resource_field_preserves_supported_values() {
        let empty = ResourceLogs::decode(encode_config_resource_field(&HashMap::new()))
            .expect("empty resource fragment should decode");
        assert_eq!(
            empty
                .resource
                .expect("empty resource should still be present")
                .attributes,
            []
        );

        let resource = HashMap::from([
            (
                "service.name".to_owned(),
                ConfigAttributeValue::String("test-svc".to_owned()),
            ),
            ("enabled".to_owned(), ConfigAttributeValue::Bool(true)),
            ("replicas".to_owned(), ConfigAttributeValue::I64(3)),
            ("ratio".to_owned(), ConfigAttributeValue::F64(0.5)),
            (
                "zones".to_owned(),
                ConfigAttributeValue::Array(ConfigAttributeValueArray::String(vec![
                    "west".to_owned(),
                    "east".to_owned(),
                ])),
            ),
            (
                "flags".to_owned(),
                ConfigAttributeValue::Array(ConfigAttributeValueArray::Bool(vec![true, false])),
            ),
            (
                "ports".to_owned(),
                ConfigAttributeValue::Array(ConfigAttributeValueArray::I64(vec![4317, 4318])),
            ),
            (
                "weights".to_owned(),
                ConfigAttributeValue::Array(ConfigAttributeValueArray::F64(vec![0.25, 0.75])),
            ),
        ]);
        let bytes = encode_config_resource_field(&resource);
        let decoded = ResourceLogs::decode(bytes).expect("resource fragment should decode");
        let actual = decoded
            .resource
            .expect("resource should exist")
            .attributes
            .into_iter()
            .map(|attribute| {
                (
                    attribute.key,
                    attribute.value.expect("attribute value should be present"),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let expected = BTreeMap::from([
            ("enabled".to_owned(), AnyValue::new_bool(true)),
            (
                "flags".to_owned(),
                AnyValue::new_array(vec![AnyValue::new_bool(true), AnyValue::new_bool(false)]),
            ),
            (
                "ports".to_owned(),
                AnyValue::new_array(vec![AnyValue::new_int(4317), AnyValue::new_int(4318)]),
            ),
            ("ratio".to_owned(), AnyValue::new_double(0.5)),
            ("replicas".to_owned(), AnyValue::new_int(3)),
            ("service.name".to_owned(), AnyValue::new_string("test-svc")),
            (
                "weights".to_owned(),
                AnyValue::new_array(vec![AnyValue::new_double(0.25), AnyValue::new_double(0.75)]),
            ),
            (
                "zones".to_owned(),
                AnyValue::new_array(vec![
                    AnyValue::new_string("west"),
                    AnyValue::new_string("east"),
                ]),
            ),
        ]);

        assert_eq!(actual, expected);
        assert!(decoded.scope_logs.is_empty());
    }

    #[test]
    fn encode_export_logs_request_with_scope_attributes() {
        let registry = TelemetryRegistryHandle::new();
        let entity_key = registry.register_entity(TestScopeAttributes::new("my-pipeline", 3));
        let mut scope_cache = ScopeToBytesMap::new(registry.clone());

        // Use the macro to create a LogRecord, then override context with entity
        let record = __log_record_impl!(Level::INFO, "test.scope.encoding")
            .into_record(LogContext::from_buf([entity_key]));

        // Create a LogEvent with known timestamp, empty resource.
        let timestamp_ns: u64 = 1_705_321_845_000_000_000;
        let time = SystemTime::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);
        let log_event = LogEvent { time, record };

        let resource_bytes = encode_config_resource_field(&HashMap::new());

        let mut buf = ProtoBuffer::default();
        encode_export_logs_request(
            &mut buf,
            &mut [log_event.clone()],
            &resource_bytes,
            &mut scope_cache,
        )
        .unwrap();

        let decoded = ExportLogsServiceRequest::decode(buf.into_bytes().as_ref()).unwrap();
        let scope_logs = &decoded.resource_logs.first().unwrap().scope_logs;
        let scope = scope_logs
            .first()
            .unwrap()
            .scope
            .as_ref()
            .expect("scope present");
        let event_name = &scope_logs
            .first()
            .unwrap()
            .log_records
            .first()
            .unwrap()
            .event_name;

        // event_name is the bare semconv identifier; the emitting crate
        // is conveyed via InstrumentationScope.name. See
        // encode_event_name + encode_export_logs_request.
        assert_eq!(event_name, "test.scope.encoding");
        assert_eq!(scope.name, "otel-arrow-dfe-telemetry");

        let expected = ExportLogsServiceRequest::new([ResourceLogs::new(
            Resource::build().finish(),
            [ScopeLogs::new(
                InstrumentationScope::build()
                    .name("otel-arrow-dfe-telemetry")
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

        // Inspect the printed format. Console output still uses the
        // `target::name` style for human readability; only OTLP changes.
        assert_eq!(
            format_log_record_to_string(None, &log_event.record),
            format!(
                "INFO  otel-arrow-dfe-telemetry::{event_name} entity={:?}\n",
                entity_key
            ),
        );
        assert_eq!(expected, decoded);
    }

    /// Scenario: a log references an entity key that has been unregistered.
    /// Guarantees: the scope records the unregistered entity's FFI key as an int64 attribute.
    #[test]
    fn encode_export_logs_request_with_unregistered_entity() {
        let registry = TelemetryRegistryHandle::new();
        let entity_key = registry.register_entity(TestScopeAttributes::new("removed", 1));
        assert!(registry.unregister_entity(entity_key));
        let mut scope_cache = ScopeToBytesMap::new(registry);
        let record = __log_record_impl!(Level::INFO, "test.unregistered.entity")
            .into_record(LogContext::from_buf([entity_key]));
        let mut events = [LogEvent {
            time: SystemTime::UNIX_EPOCH,
            record,
        }];
        let mut buf = ProtoBuffer::default();

        encode_export_logs_request(&mut buf, &mut events, &Bytes::new(), &mut scope_cache).unwrap();

        let decoded =
            ExportLogsServiceRequest::decode(buf.into_bytes()).expect("valid logs request");
        let scope = decoded.resource_logs[0].scope_logs[0]
            .scope
            .as_ref()
            .expect("scope present");
        assert_eq!(
            scope.attributes,
            [KeyValue::new(
                "unregistered.entity",
                AnyValue::new_int(entity_key.data().as_ffi() as i64),
            )]
        );
    }

    /// Scenario: a batch interleaves records from two entity contexts.
    /// Guarantees: equal contexts share one scope regardless of entity-key sort order.
    #[test]
    fn encode_export_logs_request_groups_equal_scopes() {
        fn assert_scope<const N: usize>(
            scope_logs: &[ScopeLogs],
            pipeline_name: &'static str,
            cpu_id: i64,
            event_names: [&'static str; N],
        ) {
            let expected_attributes = [
                KeyValue::new("pipeline.name", AnyValue::new_string(pipeline_name)),
                KeyValue::new("cpu.id", AnyValue::new_int(cpu_id)),
            ];
            let scope_logs = scope_logs
                .iter()
                .find(|scope_logs| {
                    scope_logs
                        .scope
                        .as_ref()
                        .is_some_and(|scope| scope.attributes == expected_attributes)
                })
                .expect("expected scope attributes");

            assert_eq!(
                scope_logs
                    .log_records
                    .iter()
                    .map(|record| record.event_name.as_str())
                    .collect::<Vec<_>>(),
                event_names
            );
        }

        let registry = TelemetryRegistryHandle::new();
        let key_a = registry.register_entity(TestScopeAttributes::new("pipeline-a", 1));
        let key_b = registry.register_entity(TestScopeAttributes::new("pipeline-b", 2));
        let mut scope_cache = ScopeToBytesMap::new(registry);
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
        let mut events = vec![
            LogEvent {
                time,
                record: __log_record_impl!(Level::INFO, "test.batch.b-first")
                    .into_record(LogContext::from_buf([key_b])),
            },
            LogEvent {
                time,
                record: __log_record_impl!(Level::INFO, "test.batch.a")
                    .into_record(LogContext::from_buf([key_a])),
            },
            LogEvent {
                time,
                record: __log_record_impl!(Level::INFO, "test.batch.b-second")
                    .into_record(LogContext::from_buf([key_b])),
            },
        ];

        let mut buf = ProtoBuffer::default();
        encode_export_logs_request(
            &mut buf,
            events.as_mut_slice(),
            &Bytes::new(),
            &mut scope_cache,
        )
        .unwrap();
        let decoded =
            ExportLogsServiceRequest::decode(buf.into_bytes()).expect("valid logs request");
        let scope_logs = &decoded.resource_logs[0].scope_logs;

        assert_eq!(scope_logs.len(), 2);
        assert_scope(
            scope_logs,
            "pipeline-b",
            2,
            ["test.batch.b-first", "test.batch.b-second"],
        );
        assert_scope(scope_logs, "pipeline-a", 1, ["test.batch.a"]);
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

        let record = __log_record_impl!(Level::INFO, "test.map.encoding")
            .into_record(LogContext::from_buf([entity_key]));

        let timestamp_ns: u64 = 1_705_321_845_000_000_000;
        let time = SystemTime::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);
        let log_event = LogEvent { time, record };

        let resource_bytes = encode_config_resource_field(&HashMap::new());
        let mut buf = ProtoBuffer::default();
        encode_export_logs_request(
            &mut buf,
            &mut [log_event],
            &resource_bytes,
            &mut scope_cache,
        )
        .unwrap();

        let decoded = ExportLogsServiceRequest::decode(buf.into_bytes().as_ref()).unwrap();
        let scope = decoded.resource_logs[0].scope_logs[0]
            .scope
            .as_ref()
            .expect("scope present");
        let event_name = &decoded.resource_logs[0].scope_logs[0].log_records[0].event_name;
        assert_eq!(event_name, "test.map.encoding");
        assert_eq!(scope.name, "otel-arrow-dfe-telemetry");

        // BTreeMap iterates in sorted key order: "priority" before "region".
        let expected = ExportLogsServiceRequest::new([ResourceLogs::new(
            Resource::build().finish(),
            [ScopeLogs::new(
                InstrumentationScope::build()
                    .name("otel-arrow-dfe-telemetry")
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
