// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Direct OTLP bytes encoder for tokio-tracing events.

use super::{LogRecord, SavedCallsite};
use crate::event::LogEvent;
use crate::registry::EntityKey;
use crate::registry::TelemetryRegistryHandle;
use bytes::Bytes;
use otap_df_pdata::otlp::common::{BoundedBuf, Dropped, EncodeResult, ProtoBuffer};
use otap_df_pdata::proto::consts::{
    field_num::common::*, field_num::logs::*, field_num::resource::*, wire_types,
};
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::Level;

/// Direct encoder that writes a single LogRecord from a tracing Event.
pub struct DirectLogRecordEncoder<'buf, B: BoundedBuf> {
    buf: &'buf mut B,
}

impl<'buf, B: BoundedBuf> DirectLogRecordEncoder<'buf, B> {
    /// Create a new encoder that writes to the provided buffer.
    #[inline]
    pub const fn new(buf: &'buf mut B) -> Self {
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
        let _ = self
            .buf
            .encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
        let _ = self.buf.extend_from_slice(&timestamp_ns.to_le_bytes());

        // Encode severity_number (field 2, varint)
        let severity = level_to_severity_number(record.callsite().level());
        let _ = self
            .buf
            .encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
        let _ = self.buf.encode_varint(severity as u64);

        // Note we skip encoding severity_text (field 3, string)

        // Encode event_name (field 12, string) - format: "target::name (file:line)"
        let _ = encode_event_name(self.buf, record.callsite());

        // Pre-encoded body and attributes.
        let _ = self.buf.extend_from_slice(&record.body_attrs_bytes);

        // Encode dropped_attributes_count (field 7, varint) if any were dropped.
        if record.dropped_attributes_count > 0 {
            let _ = self
                .buf
                .encode_field_tag(LOG_RECORD_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT);
            let _ = self
                .buf
                .encode_varint(record.dropped_attributes_count as u64);
        }

        self.buf.len() - start_len
    }
}

/// Encode the event name from callsite metadata.
/// Format: "target::name (file:line)" or "target::name" if no file/line.
fn encode_event_name<B: BoundedBuf>(buf: &mut B, callsite: SavedCallsite) -> EncodeResult {
    buf.encode_len_delimited(LOG_RECORD_EVENT_NAME, |buf| {
        buf.try_extend(callsite.target().as_bytes())?;
        buf.try_extend(b"::")?;
        buf.try_extend(callsite.name().as_bytes())?;
        Ok(())
    })
}

/// Visitor that directly encodes tracing fields to protobuf.
pub struct DirectFieldVisitor<'buf, B: BoundedBuf> {
    buf: &'buf mut B,
    dropped_count: u32,
}

impl<'buf, B: BoundedBuf> DirectFieldVisitor<'buf, B> {
    /// Create a new DirectFieldVisitor that writes to the provided buffer.
    pub const fn new(buf: &'buf mut B) -> Self {
        Self {
            buf,
            dropped_count: 0,
        }
    }

    /// Returns the number of fields dropped due to size limits.
    #[must_use]
    pub const fn dropped_count(&self) -> u32 {
        self.dropped_count
    }

    /// Encode a string attribute, truncating the value if it doesn't fit.
    ///
    /// Returns `Ok(false)` if the full value was written, `Ok(true)` if the
    /// value was truncated (with a `[...]` suffix), or `Err(Dropped)` if even
    /// a truncated form would not fit. On `Err`, the buffer is left in a state
    /// that is invalid for OTLP (an unfinished partial KeyValue may have been
    /// written), so callers must invoke this within a [`BoundedBuf::try_encode`]
    /// transaction so the partial bytes are rolled back.
    #[inline]
    fn encode_string_attribute_truncating_to(
        buf: &mut B,
        key: &str,
        value: &str,
    ) -> Result<bool, Dropped> {
        let mut truncated = false;
        // Use _partial variants so the wrapper length placeholders are patched
        // even when the inner truncating encoder writes partial bytes (it
        // signals truncation to the caller via the captured `truncated` flag,
        // not via Err, so wrappers complete cleanly).
        buf.encode_len_delimited_partial(LOG_RECORD_ATTRIBUTES, |buf| {
            buf.encode_string(KEY_VALUE_KEY, key)?;
            buf.encode_len_delimited_partial(KEY_VALUE_VALUE, |buf| {
                match buf.encode_string_truncating(ANY_VALUE_STRING_VALUE, value) {
                    Ok(was_truncated) => {
                        truncated = was_truncated;
                        Ok(())
                    }
                    Err(Dropped) => Err(Dropped),
                }
            })
        })?;
        Ok(truncated)
    }

    /// Encode an i64 attribute into a buffer.
    #[inline]
    fn encode_int_attribute_to(buf: &mut B, key: &str, value: i64) -> EncodeResult {
        buf.encode_len_delimited(LOG_RECORD_ATTRIBUTES, |buf| {
            buf.encode_string(KEY_VALUE_KEY, key)?;
            buf.encode_len_delimited(KEY_VALUE_VALUE, |buf| {
                buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
                buf.encode_varint(value as u64)
            })
        })
    }

    /// Encode a bool attribute into a buffer.
    #[inline]
    fn encode_bool_attribute_to(buf: &mut B, key: &str, value: bool) -> EncodeResult {
        buf.encode_len_delimited(LOG_RECORD_ATTRIBUTES, |buf| {
            buf.encode_string(KEY_VALUE_KEY, key)?;
            buf.encode_len_delimited(KEY_VALUE_VALUE, |buf| {
                buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT)?;
                buf.encode_varint(u64::from(value))
            })
        })
    }

    /// Encode a double attribute into a buffer.
    #[inline]
    fn encode_double_attribute_to(buf: &mut B, key: &str, value: f64) -> EncodeResult {
        buf.encode_len_delimited(LOG_RECORD_ATTRIBUTES, |buf| {
            buf.encode_string(KEY_VALUE_KEY, key)?;
            buf.encode_len_delimited(KEY_VALUE_VALUE, |buf| {
                buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64)?;
                buf.extend_from_slice(&value.to_le_bytes())
            })
        })
    }

    /// Encode a Debug attribute into a buffer.
    #[inline]
    fn encode_debug_attribute_to(
        buf: &mut B,
        key: &str,
        value: &dyn std::fmt::Debug,
    ) -> EncodeResult {
        buf.encode_len_delimited(LOG_RECORD_ATTRIBUTES, |buf| {
            buf.encode_string(KEY_VALUE_KEY, key)?;
            buf.encode_len_delimited(KEY_VALUE_VALUE, |buf| encode_debug_string(buf, value))
        })
    }

    /// Encode the body as a string. Empty strings are skipped.
    ///
    /// Wrapped in `try_encode` so that any partial wire bytes written before
    /// hitting the buffer's limit are rolled back. Otherwise an unpatched
    /// length placeholder + leftover content bytes would corrupt subsequent
    /// fields (e.g. `dropped_attributes_count`) appended to the buffer later.
    #[inline]
    pub fn encode_body_string(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }
        let _ = self.buf.try_encode(|buf| {
            buf.encode_len_delimited(LOG_RECORD_BODY, |buf| {
                buf.encode_string(ANY_VALUE_STRING_VALUE, value)
            })
        });
    }

    /// Encode the body from a Debug value without allocation.
    ///
    /// Wrapped in `try_encode` so partial bytes are rolled back on overflow;
    /// see [`Self::encode_body_string`] for rationale.
    #[inline]
    pub fn encode_body_debug(&mut self, value: &dyn std::fmt::Debug) {
        let _ = self.buf.try_encode(|buf| {
            buf.encode_len_delimited(LOG_RECORD_BODY, |buf| encode_debug_string(buf, value))
        });
    }
}

/// Adapter that lets `write!` format directly into a `BoundedBuf` without
/// an intermediate `String`. `try_extend` returns `Err(Dropped)` on
/// overflow; we map that to `fmt::Error` so `write!` short-circuits and
/// the caller learns truncation occurred.
struct BoundedBufFmt<'a, B: BoundedBuf>(&'a mut B);

impl<B: BoundedBuf> std::fmt::Write for BoundedBufFmt<'_, B> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.try_extend(s.as_bytes()).map_err(|_| std::fmt::Error)
    }
}

/// Helper to encode a Debug value as a protobuf string field.
/// This is separate from DirectFieldVisitor to avoid borrow conflicts with the macro.
#[inline]
fn encode_debug_string<B: BoundedBuf>(buf: &mut B, value: &dyn std::fmt::Debug) -> EncodeResult {
    buf.encode_len_delimited(ANY_VALUE_STRING_VALUE, |buf| {
        // Wrap in a local fmt::Write adapter so the formatter machinery
        // writes directly into the buffer (no intermediate String). If the
        // buffer fills up, `write!` returns `Err(fmt::Error)` and we
        // propagate as `Dropped` so the surrounding `try_encode` rolls
        // back partial bytes and the caller bumps `dropped_count`.
        use std::fmt::Write as _;
        let mut adapter = BoundedBufFmt(buf);
        write!(adapter, "{:?}", value).map_err(|_| Dropped)
    })
}

/// Compute the per-attribute budget: at most half of remaining space, but
/// at least 1 byte (to ensure a chance of a tag byte being written and
/// triggering an atomic drop). Intentionally halves for every attribute,
/// including the last — this keeps the policy single-pass without requiring
/// pre-counting the field set, while still preventing one large value from
/// consuming the entire remaining buffer.
#[inline]
fn attr_budget<B: BoundedBuf>(buf: &B) -> usize {
    buf.remaining().div_ceil(2)
}

impl<B: BoundedBuf> tracing::field::Visit for DirectFieldVisitor<'_, B> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if field.name() == "message" {
            return;
        }
        let budget = attr_budget(self.buf);
        let fit = self.buf.with_max_remaining(budget, |buf| {
            buf.try_encode(|b| {
                DirectFieldVisitor::encode_double_attribute_to(b, field.name(), value)
            })
        });
        if fit.is_err() {
            self.dropped_count += 1;
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "message" {
            return;
        }
        let budget = attr_budget(self.buf);
        let fit = self.buf.with_max_remaining(budget, |buf| {
            buf.try_encode(|b| DirectFieldVisitor::encode_int_attribute_to(b, field.name(), value))
        });
        if fit.is_err() {
            self.dropped_count += 1;
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "message" {
            return;
        }
        let budget = attr_budget(self.buf);
        let fit = self.buf.with_max_remaining(budget, |buf| {
            buf.try_encode(|b| {
                DirectFieldVisitor::encode_int_attribute_to(b, field.name(), value as i64)
            })
        });
        if fit.is_err() {
            self.dropped_count += 1;
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        if field.name() == "message" {
            return;
        }
        let budget = attr_budget(self.buf);
        let fit = self.buf.with_max_remaining(budget, |buf| {
            buf.try_encode(|b| DirectFieldVisitor::encode_bool_attribute_to(b, field.name(), value))
        });
        if fit.is_err() {
            self.dropped_count += 1;
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.encode_body_string(value);
            return;
        }
        let budget = attr_budget(self.buf);
        let key = field.name();
        let mut truncated = false;
        // Attempt the encoding (with truncation as needed) within the scoped
        // budget, transactionally so a hard failure rolls back any partial
        // KeyValue bytes. Truncation status is reported via the captured
        // `truncated` flag rather than via Err, so the transaction commits.
        let fit = self.buf.with_max_remaining(budget, |buf| {
            buf.try_encode(|b| {
                truncated =
                    DirectFieldVisitor::encode_string_attribute_truncating_to(b, key, value)?;
                Ok::<(), Dropped>(())
            })
        });
        match (fit, truncated) {
            (Ok(()), false) => {} // Fully encoded.
            (Ok(()), true) => {
                // Truncated; bytes were preserved.
                self.dropped_count += 1;
            }
            (Err(Dropped), _) => {
                // Hard failure; everything rolled back.
                self.dropped_count += 1;
            }
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.encode_body_debug(value);
            return;
        }
        let budget = attr_budget(self.buf);
        let fit = self.buf.with_max_remaining(budget, |buf| {
            buf.try_encode(|b| {
                DirectFieldVisitor::encode_debug_attribute_to(b, field.name(), value)
            })
        });
        if fit.is_err() {
            self.dropped_count += 1;
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
    let _: EncodeResult = buf.encode_len_delimited(RESOURCE_LOGS_RESOURCE, |buf| {
        // Encode each attribute as a KeyValue
        for (key, value) in attrs {
            encode_resource_attribute(buf, key.as_str(), value)?;
        }
        Ok(())
    });

    // ResourceLogs.schema_url (field 3, string)
    if let Some(url) = schema_url {
        let _ = buf.encode_string(RESOURCE_LOGS_SCHEMA_URL, url);
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
fn encode_resource_attribute(
    buf: &mut ProtoBuffer,
    key: &str,
    value: &opentelemetry::Value,
) -> EncodeResult {
    use opentelemetry::Value;

    buf.encode_len_delimited(RESOURCE_ATTRIBUTES, |buf| {
        buf.encode_string(KEY_VALUE_KEY, key)?;
        buf.encode_len_delimited(KEY_VALUE_VALUE, |buf| {
            match value {
                Value::String(s) => {
                    buf.encode_string(ANY_VALUE_STRING_VALUE, s.as_str())?;
                }
                Value::Bool(b) => {
                    buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT)?;
                    buf.encode_varint(u64::from(*b))?;
                }
                Value::I64(i) => {
                    buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT)?;
                    buf.encode_varint(*i as u64)?;
                }
                Value::F64(f) => {
                    buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64)?;
                    buf.extend_from_slice(&f.to_le_bytes())?;
                }
                _ => {
                    // TODO: share the encoding logic used somewhere else, somehow.
                    crate::raw_error!("cannot encode SDK resource value", value = ?value);
                }
            }
            Ok(())
        })
    })
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
    let _ = encode_key_value(buf, INSTRUMENTATION_SCOPE_ATTRIBUTES, key, value);
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
) -> EncodeResult {
    buf.encode_len_delimited(outer_field, |buf| {
        buf.encode_string(KEY_VALUE_KEY, key)?;
        buf.encode_len_delimited(KEY_VALUE_VALUE, |buf| encode_any_value(buf, value))
    })
}

/// Encode an `AttributeValue` as an OTLP AnyValue (the inner value without key wrapping).
#[inline]
fn encode_any_value(
    buf: &mut ProtoBuffer,
    value: &crate::attributes::AttributeValue,
) -> EncodeResult {
    use crate::attributes::AttributeValue;

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
    let _: EncodeResult = buf.encode_len_delimited(EXPORT_LOGS_REQUEST_RESOURCE_LOGS, |buf| {
        // ResourceLogs.resource (field 1, Resource message)
        // Copy pre-encoded resource bytes directly
        buf.extend_from_slice(resource_bytes)?;

        // ResourceLogs.scope_logs (field 2, repeated ScopeLogs)
        buf.encode_len_delimited(RESOURCE_LOGS_SCOPE_LOGS, |buf| {
            // ScopeLogs.scope (field 1, InstrumentationScope message)
            buf.encode_len_delimited(SCOPE_LOG_SCOPE, |buf| {
                for entity_key in event.record.context.iter() {
                    let scope_bytes = scope_cache.get_or_encode(*entity_key);
                    buf.extend_from_slice(&scope_bytes)?;
                }
                Ok(())
            })?;

            // ScopeLogs.log_records (field 2, repeated LogRecord)
            buf.encode_len_delimited(SCOPE_LOGS_LOG_RECORDS, |buf| {
                let mut encoder = DirectLogRecordEncoder::new(buf);
                let _ = encoder.encode_log_record(event.time, &event.record);
                Ok(())
            })
        })
    });
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
        let record = __log_record_impl!(Level::INFO, "test.scope.encoding")
            .into_record(LogContext::from_buf([entity_key]));

        // Create a LogEvent with known timestamp, empty resource.
        let timestamp_ns: u64 = 1_705_321_845_000_000_000;
        let time = SystemTime::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);
        let log_event = LogEvent { time, record };

        let resource_bytes = encode_resource_to_bytes(&OTelResource::builder_empty().build());

        let mut buf = ProtoBuffer::default();
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

        let record = __log_record_impl!(Level::INFO, "test.map.encoding")
            .into_record(LogContext::from_buf([entity_key]));

        let timestamp_ns: u64 = 1_705_321_845_000_000_000;
        let time = SystemTime::UNIX_EPOCH + Duration::from_nanos(timestamp_ns);
        let log_event = LogEvent { time, record };

        let resource_bytes = encode_resource_to_bytes(&OTelResource::builder_empty().build());
        let mut buf = ProtoBuffer::default();
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

    /// Verify the fair-budget halving policy for string attributes.
    ///
    /// With a 256-byte inline buffer and 4 attributes whose values are 1000
    /// bytes each, every attribute should consume roughly half of whatever
    /// remained, leaving each successive value smaller than the previous.
    /// All four values are truncated, so `dropped_attributes_count` is 4.
    #[test]
    fn record_str_halves_remaining_budget_across_attributes() {
        use crate::self_tracing::encoder::DirectFieldVisitor;
        use otap_df_pdata::otlp::common::TRUNCATION_SUFFIX;
        use otap_df_pdata::otlp::common::{BoundedBuf, StackProtoBuffer};
        use otap_df_pdata::proto::opentelemetry::common::v1::KeyValue as ProtoKeyValue;
        use prost::Message;
        use tracing::field::Visit;

        const INLINE: usize = 256;
        let big = "x".repeat(1000);

        let meta = &BUDGET_TEST_METADATA;
        let fields = meta.fields();
        let a = fields.field("a").unwrap();
        let b = fields.field("b").unwrap();
        let c = fields.field("c").unwrap();
        let d = fields.field("d").unwrap();

        // Drive the visitor directly with each (field, value) pair, which is
        // what `event.record(visitor)` does internally.
        let mut buf = StackProtoBuffer::<INLINE>::default();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_str(&a, big.as_str());
        visitor.record_str(&b, big.as_str());
        visitor.record_str(&c, big.as_str());
        visitor.record_str(&d, big.as_str());
        let dropped = visitor.dropped_count();

        // All four values should be truncated.
        assert_eq!(dropped, 4, "expected 4 truncated attributes");

        // Buffer should not exceed the inline limit.
        assert!(
            buf.len() <= INLINE,
            "buffer len {} exceeds INLINE {}",
            buf.len(),
            INLINE
        );

        // Decode each attribute as a KeyValue (field tag = LOG_RECORD_ATTRIBUTES = 6,
        // wire type LEN). Walk the buffer manually.
        let bytes = buf.as_ref().to_vec();
        let mut cursor = bytes.as_slice();
        let mut decoded: Vec<(String, String)> = Vec::new();
        while !cursor.is_empty() {
            let (tag, n) = read_varint(cursor);
            cursor = &cursor[n..];
            let field_num = tag >> 3;
            let wire_type = tag & 0x7;
            assert_eq!(wire_type, wire_types::LEN, "expected LEN wire type");
            assert_eq!(field_num, LOG_RECORD_ATTRIBUTES, "expected ATTRIBUTES tag");
            let (len, n) = read_varint(cursor);
            cursor = &cursor[n..];
            let len = len as usize;
            let kv_bytes = &cursor[..len];
            cursor = &cursor[len..];
            let kv = ProtoKeyValue::decode(kv_bytes).unwrap();
            let val = kv
                .value
                .as_ref()
                .and_then(|v| v.value.as_ref())
                .map(|v| match v {
                    otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => s.clone(),
                    _ => panic!("expected string value"),
                })
                .unwrap();
            decoded.push((kv.key, val));
        }

        assert_eq!(decoded.len(), 4, "expected 4 decoded attributes");

        // Each value should end with the truncation suffix.
        let suffix = std::str::from_utf8(TRUNCATION_SUFFIX).unwrap();
        for (k, v) in &decoded {
            assert!(
                v.ends_with(suffix),
                "attribute {k} value should end with {suffix}, got len {}",
                v.len()
            );
        }

        // Each successive value should be smaller (halving budget). We compare
        // value lengths rather than exact sizes to avoid brittleness.
        let lens: Vec<usize> = decoded.iter().map(|(_, v)| v.len()).collect();
        assert!(
            lens.windows(2).all(|w| w[0] > w[1]),
            "expected strictly decreasing value lengths, got {lens:?}"
        );

        // Sanity-check the rough magnitudes from the user's spec
        // (~100 / ~50 / ~25 / ~10), accommodating overhead variation.
        // Observed: ~[118, 55, 23, 7].
        assert!(
            lens[0] > 60 && lens[0] < 140,
            "1st value len {} out of expected range",
            lens[0]
        );
        assert!(
            lens[1] > 25 && lens[1] < 80,
            "2nd value len {} out of expected range",
            lens[1]
        );
        assert!(
            lens[2] > 10 && lens[2] < 50,
            "3rd value len {} out of expected range",
            lens[2]
        );
        // Last value may be just the suffix (~5 bytes) up to ~25 bytes.
        assert!(
            lens[3] >= suffix.len() && lens[3] < 30,
            "4th value len {} out of expected range",
            lens[3]
        );
    }

    /// Verify `record_debug` formats a Debug value directly into the buffer
    /// (via the `BoundedBufFmt` adapter) and produces a decodable KeyValue
    /// whose string matches `format!("{:?}", value)` — i.e. no truncation
    /// when the value comfortably fits.
    #[test]
    fn record_debug_writes_fmt_output_without_intermediate_string() {
        use otap_df_pdata::otlp::common::StackProtoBuffer;
        use otap_df_pdata::proto::opentelemetry::common::v1::KeyValue as ProtoKeyValue;
        use prost::Message;
        use tracing::field::Visit;

        #[derive(Debug)]
        #[allow(dead_code)]
        struct Payload {
            id: u32,
            label: &'static str,
        }
        let value = Payload {
            id: 7,
            label: "hello",
        };
        let expected_repr = format!("{:?}", value);

        let meta = &DEBUG_TEST_METADATA;
        let fields = meta.fields();
        let dbg = fields.field("dbg").unwrap();

        let mut buf = StackProtoBuffer::<256>::default();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_debug(&dbg, &value);
        assert_eq!(visitor.dropped_count(), 0, "no fields should be dropped");

        // Decode the single KeyValue from the buffer.
        let bytes = buf.as_ref().to_vec();
        let mut cursor = bytes.as_slice();
        let (tag, n) = read_varint(cursor);
        cursor = &cursor[n..];
        assert_eq!(tag >> 3, LOG_RECORD_ATTRIBUTES);
        assert_eq!(tag & 0x7, wire_types::LEN);
        let (len, n) = read_varint(cursor);
        cursor = &cursor[n..];
        let kv = ProtoKeyValue::decode(&cursor[..len as usize]).unwrap();
        assert_eq!(kv.key, "dbg");
        let s = match kv.value.as_ref().unwrap().value.as_ref().unwrap() {
            otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(s) => {
                s.clone()
            }
            other => panic!("expected StringValue, got {other:?}"),
        };
        assert_eq!(s, expected_repr);
    }

    /// When a Debug value overflows the available buffer, `BoundedBufFmt`
    /// returns `fmt::Error` from the formatter, `encode_debug_string`
    /// propagates `Dropped`, and the surrounding `try_encode` rolls back
    /// any partial KeyValue bytes — leaving the buffer unchanged and
    /// incrementing `dropped_count`.
    #[test]
    fn record_debug_overflow_rolls_back_and_increments_dropped() {
        use otap_df_pdata::otlp::common::{BoundedBuf, StackProtoBuffer};
        use tracing::field::Visit;

        // A Debug impl that writes far more than the buffer can hold.
        struct Huge;
        impl std::fmt::Debug for Huge {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // 4 KiB of output into a 64-byte buffer.
                for _ in 0..512 {
                    f.write_str("xxxxxxxx")?;
                }
                Ok(())
            }
        }

        let meta = &DEBUG_TEST_METADATA;
        let fields = meta.fields();
        let dbg = fields.field("dbg").unwrap();

        let mut buf = StackProtoBuffer::<64>::default();
        let before_len = buf.len();
        let mut visitor = DirectFieldVisitor::new(&mut buf);
        visitor.record_debug(&dbg, &Huge);
        assert_eq!(
            visitor.dropped_count(),
            1,
            "the single oversized debug field should be counted as dropped"
        );

        // No partial KeyValue bytes survive — the transaction was rolled back.
        assert_eq!(
            buf.len(),
            before_len,
            "buffer must be rolled back to its pre-call length"
        );
    }

    static DEBUG_TEST_CALLSITE: DebugTestCallsite = DebugTestCallsite;

    struct DebugTestCallsite;

    impl tracing::Callsite for DebugTestCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            &DEBUG_TEST_METADATA
        }
    }

    static DEBUG_TEST_METADATA: tracing::Metadata<'static> = tracing::Metadata::new(
        "debug_test",
        "otap-df-telemetry",
        Level::INFO,
        Some(file!()),
        Some(line!()),
        Some(module_path!()),
        tracing::field::FieldSet::new(
            &["dbg"],
            tracing::callsite::Identifier(&DEBUG_TEST_CALLSITE),
        ),
        tracing::metadata::Kind::EVENT,
    );

    /// Read an unsigned varint, returning (value, bytes_consumed).
    fn read_varint(bytes: &[u8]) -> (u64, usize) {
        let mut value = 0u64;
        let mut shift = 0;
        for (i, &b) in bytes.iter().enumerate() {
            value |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 {
                return (value, i + 1);
            }
            shift += 7;
        }
        panic!("truncated varint");
    }

    static BUDGET_TEST_CALLSITE: BudgetTestCallsite = BudgetTestCallsite;

    struct BudgetTestCallsite;

    impl tracing::Callsite for BudgetTestCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            &BUDGET_TEST_METADATA
        }
    }

    static BUDGET_TEST_METADATA: tracing::Metadata<'static> = tracing::Metadata::new(
        "budget_test",
        "otap-df-telemetry",
        Level::INFO,
        Some(file!()),
        Some(line!()),
        Some(module_path!()),
        tracing::field::FieldSet::new(
            &["a", "b", "c", "d"],
            tracing::callsite::Identifier(&BUDGET_TEST_CALLSITE),
        ),
        tracing::metadata::Kind::EVENT,
    );
}
