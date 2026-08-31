// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Newline-delimited JSON formatting with one compact object per log record.

use super::{
    RecordJsonBodyField, RecordJsonConfig, RecordJsonInt64Format, RecordJsonTimestampFormat,
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, SecondsFormat, Utc};
use otap_df_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, ValueType,
};
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::borrow::Cow;

/// Formatter for one-record-per-line compact JSON output.
pub(super) struct RecordJsonFormatter {
    config: RecordJsonConfig,
}

impl RecordJsonFormatter {
    /// Create a record JSON formatter.
    pub(super) const fn new(config: RecordJsonConfig) -> Self {
        Self { config }
    }

    /// Format every log record as a compact JSON object followed by a newline.
    pub(super) fn format_logs_data_to<L: LogsDataView>(
        &self,
        logs_data: &L,
        output: &mut Vec<u8>,
    ) -> serde_json::Result<()> {
        for resource_logs in logs_data.resources() {
            let resource = resource_logs.resource();
            let resource_schema_url = resource_logs.schema_url();
            for scope_logs in resource_logs.scopes() {
                let scope = scope_logs.scope();
                let scope_schema_url = scope_logs.schema_url();
                for log_record in scope_logs.log_records() {
                    serde_json::to_writer(
                        &mut *output,
                        &RecordJson {
                            resource: resource.as_ref(),
                            scope: scope.as_ref(),
                            log_record: &log_record,
                            resource_schema_url,
                            scope_schema_url,
                            config: self.config,
                        },
                    )?;
                    output.push(b'\n');
                }
            }
        }
        Ok(())
    }
}

/// Serializable compact record envelope with optional inherited context.
struct RecordJson<'a, R, S, L> {
    resource: Option<&'a R>,
    scope: Option<&'a S>,
    log_record: &'a L,
    resource_schema_url: Option<&'a [u8]>,
    scope_schema_url: Option<&'a [u8]>,
    config: RecordJsonConfig,
}

impl<R, S, L> Serialize for RecordJson<'_, R, S, L>
where
    R: ResourceView,
    S: InstrumentationScopeView,
    L: LogRecordView,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let record = self.log_record;
        let mut map = serializer.serialize_map(None)?;

        if let Some(value) = record.time_unix_nano() {
            map.serialize_entry(
                "timestamp",
                &format_timestamp(value, self.config.timestamp_format),
            )?;
        }
        if let Some(value) = record.observed_time_unix_nano() {
            map.serialize_entry(
                "observed_timestamp",
                &format_timestamp(value, self.config.timestamp_format),
            )?;
        }
        if let Some(value) = record.severity_number() {
            map.serialize_entry("severity_number", &value)?;
        }
        if let Some(value) = record.severity_text() {
            map.serialize_entry("severity_text", &lossy_utf8(value))?;
        }
        if let Some(value) = record.body() {
            let field_name = match self.config.body_field {
                RecordJsonBodyField::Body => "body",
                RecordJsonBodyField::Message => "message",
            };
            map.serialize_entry(
                field_name,
                &CompactValueJson {
                    value,
                    int64_format: self.config.int64_format,
                },
            )?;
        }
        if let Some(value) = record.event_name() {
            map.serialize_entry("event_name", &lossy_utf8(value))?;
        }

        map.serialize_entry(
            "attributes",
            &LogAttributesJson {
                record,
                int64_format: self.config.int64_format,
            },
        )?;

        if self.config.resource {
            map.serialize_entry(
                "resource",
                &ResourceJson {
                    resource: self.resource,
                    int64_format: self.config.int64_format,
                },
            )?;
        }
        if self.config.scope {
            map.serialize_entry(
                "scope",
                &ScopeJson {
                    scope: self.scope,
                    int64_format: self.config.int64_format,
                },
            )?;
        }
        if let Some(value) = record.trace_id() {
            map.serialize_entry("trace_id", &hex::encode(value))?;
        }
        if let Some(value) = record.span_id() {
            map.serialize_entry("span_id", &hex::encode(value))?;
        }
        if let Some(value) = record.flags() {
            map.serialize_entry("trace_flags", &value)?;
        }
        if self.config.otel {
            map.serialize_entry(
                "otel",
                &OtelJson {
                    dropped_attributes_count: record.dropped_attributes_count(),
                    resource_schema_url: self.resource_schema_url,
                    scope_schema_url: self.scope_schema_url,
                },
            )?;
        }

        map.end()
    }
}

/// Serializable flattened resource attributes.
struct ResourceJson<'a, R> {
    resource: Option<&'a R>,
    int64_format: RecordJsonInt64Format,
}

impl<R: ResourceView> Serialize for ResourceJson<'_, R> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let attributes = self
            .resource
            .map(|resource| compact_attributes(resource.attributes(), self.int64_format))
            .unwrap_or_default();
        attributes.serialize(serializer)
    }
}

/// Serializable instrumentation scope context.
struct ScopeJson<'a, S> {
    scope: Option<&'a S>,
    int64_format: RecordJsonInt64Format,
}

impl<S: InstrumentationScopeView> Serialize for ScopeJson<'_, S> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(scope) = self.scope {
            if let Some(value) = scope.name() {
                map.serialize_entry("name", &lossy_utf8(value))?;
            }
            if let Some(value) = scope.version() {
                map.serialize_entry("version", &lossy_utf8(value))?;
            }
            map.serialize_entry(
                "attributes",
                &ScopeAttributesJson {
                    scope,
                    int64_format: self.int64_format,
                },
            )?;
        } else {
            map.serialize_entry("attributes", &JsonMap::<String, JsonValue>::new())?;
        }
        map.end()
    }
}

/// Serializable OpenTelemetry bookkeeping fields.
struct OtelJson<'a> {
    dropped_attributes_count: u32,
    resource_schema_url: Option<&'a [u8]>,
    scope_schema_url: Option<&'a [u8]>,
}

impl Serialize for OtelJson<'_> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("dropped_attributes_count", &self.dropped_attributes_count)?;
        if let Some(value) = self.resource_schema_url {
            map.serialize_entry("resource_schema_url", &lossy_utf8(value))?;
        }
        if let Some(value) = self.scope_schema_url {
            map.serialize_entry("scope_schema_url", &lossy_utf8(value))?;
        }
        map.end()
    }
}

/// Serializable log-record attributes.
struct LogAttributesJson<'a, L> {
    record: &'a L,
    int64_format: RecordJsonInt64Format,
}

impl<L: LogRecordView> Serialize for LogAttributesJson<'_, L> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        compact_attributes(self.record.attributes(), self.int64_format).serialize(serializer)
    }
}

/// Serializable instrumentation-scope attributes.
struct ScopeAttributesJson<'a, S> {
    scope: &'a S,
    int64_format: RecordJsonInt64Format,
}

impl<S: InstrumentationScopeView> Serialize for ScopeAttributesJson<'_, S> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        compact_attributes(self.scope.attributes(), self.int64_format).serialize(serializer)
    }
}

/// Serializable compact AnyValue.
struct CompactValueJson<T> {
    value: T,
    int64_format: RecordJsonInt64Format,
}

impl<'value, T> Serialize for CompactValueJson<T>
where
    T: AnyValueView<'value> + 'value,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        compact_value(&self.value, self.int64_format).serialize(serializer)
    }
}

/// Convert an attribute iterator to a compact JSON object.
fn compact_attributes<I, A>(
    attributes: I,
    int64_format: RecordJsonInt64Format,
) -> JsonMap<String, JsonValue>
where
    I: Iterator<Item = A>,
    A: AttributeView,
{
    let mut map = JsonMap::new();
    for attribute in attributes {
        let key = lossy_utf8(attribute.key()).into_owned();
        if let Some(value) = attribute.value() {
            _ = map.insert(key, compact_value(&value, int64_format));
        } else {
            _ = map.remove(&key);
        }
    }
    map
}

/// Convert an AnyValue to its compact JSON representation.
fn compact_value<'value, T>(value: &T, int64_format: RecordJsonInt64Format) -> JsonValue
where
    T: AnyValueView<'value> + 'value,
{
    match value.value_type() {
        ValueType::Empty => JsonValue::Null,
        ValueType::String => JsonValue::String(
            lossy_utf8(value.as_string().expect("value type is string")).into_owned(),
        ),
        ValueType::Bool => JsonValue::Bool(value.as_bool().expect("value type is bool")),
        ValueType::Int64 => {
            let integer = value.as_int64().expect("value type is int64");
            match int64_format {
                RecordJsonInt64Format::Number => JsonValue::Number(integer.into()),
                RecordJsonInt64Format::String => JsonValue::String(integer.to_string()),
            }
        }
        ValueType::Double => JsonNumber::from_f64(value.as_double().expect("value type is double"))
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        ValueType::Array => JsonValue::Array(
            value
                .as_array()
                .expect("value type is array")
                .map(|child| compact_value(&child, int64_format))
                .collect(),
        ),
        ValueType::KeyValueList => JsonValue::Object(compact_attributes(
            value.as_kvlist().expect("value type is key-value list"),
            int64_format,
        )),
        ValueType::Bytes => JsonValue::String(
            BASE64_STANDARD.encode(value.as_bytes().expect("value type is bytes")),
        ),
    }
}

/// Format a nanosecond Unix timestamp according to the configured representation.
fn format_timestamp(value: u64, format: RecordJsonTimestampFormat) -> String {
    match format {
        RecordJsonTimestampFormat::Rfc3339 => {
            let seconds = (value / 1_000_000_000) as i64;
            let nanoseconds = (value % 1_000_000_000) as u32;
            DateTime::<Utc>::from_timestamp(seconds, nanoseconds)
                .expect("u64 nanosecond timestamp is within the chrono range")
                .to_rfc3339_opts(SecondsFormat::Nanos, true)
        }
        RecordJsonTimestampFormat::UnixNano => value.to_string(),
    }
}

/// Convert a potentially invalid protobuf string to valid JSON text.
fn lossy_utf8(value: &[u8]) -> Cow<'_, str> {
    String::from_utf8_lossy(value)
}
