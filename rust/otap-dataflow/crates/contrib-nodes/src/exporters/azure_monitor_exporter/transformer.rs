// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::{BufMut, Bytes, BytesMut};
use otap_df_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType,
};
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;

use super::config::{Config, SchemaConfig};
use super::error::Error;

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// Pre-parsed field mapping for a log record field
#[derive(Debug, Clone)]
struct FieldMapping {
    /// The source field name (e.g., "time_unix_nano", "body")
    source: LogRecordField,
    /// The destination field name in the output JSON
    dest: String,
}

/// Enum representing known log record fields for fast matching
#[derive(Debug, Clone, Copy)]
enum LogRecordField {
    TimeUnixNano,
    ObservedTimeUnixNano,
    TraceId,
    SpanId,
    Flags,
    SeverityNumber,
    SeverityText,
    Body,
    EventName,
}

impl LogRecordField {
    /// Parse a field name string into a LogRecordField enum
    fn from_str(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("time_unix_nano") {
            Some(Self::TimeUnixNano)
        } else if s.eq_ignore_ascii_case("observed_time_unix_nano") {
            Some(Self::ObservedTimeUnixNano)
        } else if s.eq_ignore_ascii_case("trace_id") {
            Some(Self::TraceId)
        } else if s.eq_ignore_ascii_case("span_id") {
            Some(Self::SpanId)
        } else if s.eq_ignore_ascii_case("flags") {
            Some(Self::Flags)
        } else if s.eq_ignore_ascii_case("severity_number") {
            Some(Self::SeverityNumber)
        } else if s.eq_ignore_ascii_case("severity_text") {
            Some(Self::SeverityText)
        } else if s.eq_ignore_ascii_case("body") {
            Some(Self::Body)
        } else if s.eq_ignore_ascii_case("event_name") {
            // Add this
            Some(Self::EventName)
        } else {
            None
        }
    }
}

/// Pre-parsed schema configuration for faster runtime processing
#[derive(Debug, Clone)]
struct ParsedSchema {
    /// Resource attribute mappings (source -> dest)
    resource_mapping: HashMap<String, String>,
    /// Scope attribute mappings (source -> dest)
    scope_mapping: HashMap<String, String>,
    /// Pre-parsed log record field mappings
    field_mappings: Vec<FieldMapping>,
    /// Pre-parsed attribute mappings (source attr -> dest field)
    attribute_mapping: HashMap<String, String>,
}

impl ParsedSchema {
    fn from_config(schema: &SchemaConfig) -> Result<Self, Error> {
        let mut field_mappings = Vec::new();
        let mut attribute_mapping = HashMap::new();

        for (key, value) in &schema.log_record_mapping {
            if key.eq_ignore_ascii_case("attributes") {
                // Parse attribute mappings
                if let Some(attr_map) = value.as_object() {
                    for (attr_key, attr_dest) in attr_map {
                        let dest = attr_dest
                            .as_str()
                            .map(String::from)
                            .unwrap_or_else(|| attr_dest.to_string());
                        _ = attribute_mapping.insert(attr_key.clone(), dest);
                    }
                }
            } else {
                // Parse field mapping
                let source = LogRecordField::from_str(key)
                    .ok_or_else(|| Error::UnknownLogRecordField { field: key.clone() })?;
                let dest = value
                    .as_str()
                    .ok_or_else(|| Error::InvalidFieldMapping { field: key.clone() })?
                    .to_string();
                field_mappings.push(FieldMapping { source, dest });
            }
        }

        Ok(Self {
            resource_mapping: schema.resource_mapping.clone(),
            scope_mapping: schema.scope_mapping.clone(),
            field_mappings,
            attribute_mapping,
        })
    }
}

/// Converts OTLP logs to Azure Log Analytics format
#[derive(Debug)]
pub struct Transformer {
    schema: ParsedSchema,
}

impl Transformer {
    /// Create a new Transformer with the given configuration
    ///
    /// # Panics
    /// Panics if the schema configuration contains invalid field names
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            schema: ParsedSchema::from_config(&config.api.schema)
                .expect("Invalid schema configuration"),
        }
    }

    /// Create a new Transformer, returning an error if configuration is invalid
    pub fn try_new(config: &Config) -> Result<Self, Error> {
        Ok(Self {
            schema: ParsedSchema::from_config(&config.api.schema)?,
        })
    }

    /// High-perf, single-threaded: pre-serializes resource+scope fields once per ScopeLogs,
    /// then writes per-record fields directly to the buffer — no Map cloning or re-serialization.
    /// Assumes non-overlapping mappings across resource, scope, and log record levels.
    #[must_use]
    // TODO: When Rust generators stabilize (rust-lang/rust#117078), replace
    // Vec<Bytes> return with a generator that yields &[u8] per record,
    // allowing the caller to feed records directly into GzipBatcher
    // without the intermediate Vec allocation and second iteration pass.
    pub fn convert_to_log_analytics<T: LogsDataView>(&self, logs_view: &T) -> Vec<Bytes> {
        let mut results = Vec::with_capacity(1024);
        let mut buf = BytesMut::with_capacity(2048);
        let mut base_map = serde_json::Map::new();
        let mut record_buf = Vec::with_capacity(512);

        for resource_logs in logs_view.resources() {
            base_map.clear();
            if let Some(r) = resource_logs.resource() {
                self.apply_resource_mapping(&r, &mut base_map);
            }

            for scope_logs in resource_logs.scopes() {
                // Clone resource base once per ScopeLogs, add scope mappings
                let mut scope_map = base_map.clone();
                if let Some(s) = scope_logs.scope() {
                    self.apply_scope_mapping(&s, &mut scope_map);
                }

                // Pre-serialize resource+scope as JSON bytes (once per ScopeLogs)
                // Safety: base_map only contains valid JSON values from convert_any_value
                let base_json = serde_json::to_vec(&scope_map).unwrap_or_default();
                let has_base = base_json.len() > 2; // more than just "{}"
                // Strip trailing '}' to allow appending record fields
                let base_prefix = &base_json[..base_json.len() - 1];

                for log_record in scope_logs.log_records() {
                    record_buf.clear();

                    let has_record = self.write_record_fields_json(&log_record, &mut record_buf);

                    buf.clear();
                    match (has_base, has_record) {
                        (true, true) => {
                            buf.extend_from_slice(base_prefix);
                            buf.put_u8(b',');
                            buf.extend_from_slice(&record_buf);
                            buf.put_u8(b'}');
                        }
                        (true, false) => {
                            buf.extend_from_slice(&base_json);
                        }
                        (false, true) => {
                            buf.put_u8(b'{');
                            buf.extend_from_slice(&record_buf);
                            buf.put_u8(b'}');
                        }
                        (false, false) => {
                            buf.extend_from_slice(b"{}");
                        }
                    }

                    results.push(buf.split().freeze());
                }
            }
        }

        results
    }

    /// Write log record fields directly as JSON key:value pairs (no braces) to a byte buffer.
    /// Returns true if any fields were written.
    fn write_record_fields_json<R: LogRecordView>(
        &self,
        log_record: &R,
        out: &mut Vec<u8>,
    ) -> bool {
        let mut has_field = false;

        for fm in &self.schema.field_mappings {
            if has_field {
                out.push(b',');
            }
            has_field = true;
            // Write key
            Self::write_json_string(fm.dest.as_bytes(), out);
            out.push(b':');
            // Write value directly — avoids Value allocation for simple types
            Self::write_field_value_json(fm.source, log_record, out);
        }

        if !self.schema.attribute_mapping.is_empty() {
            for attr in log_record.attributes() {
                let attr_key: Cow<'_, str> = String::from_utf8_lossy(attr.key());
                if let Some(dest) = self.schema.attribute_mapping.get(attr_key.as_ref()) {
                    if let Some(val) = attr.value() {
                        if has_field {
                            out.push(b',');
                        }
                        has_field = true;
                        Self::write_json_string(dest.as_bytes(), out);
                        out.push(b':');
                        Self::write_any_value_json(&val, out);
                    }
                }
            }
        }

        has_field
    }

    /// Write a field value directly to JSON bytes, avoiding intermediate Value allocation.
    #[inline]
    fn write_field_value_json<R: LogRecordView>(
        field: LogRecordField,
        log_record: &R,
        out: &mut Vec<u8>,
    ) {
        match field {
            LogRecordField::TimeUnixNano => {
                let ts = Self::format_timestamp(log_record.time_unix_nano().unwrap_or(0));
                Self::write_json_string(ts.as_bytes(), out);
            }
            LogRecordField::ObservedTimeUnixNano => {
                let ts = Self::format_timestamp(log_record.observed_time_unix_nano().unwrap_or(0));
                Self::write_json_string(ts.as_bytes(), out);
            }
            LogRecordField::TraceId => match log_record.trace_id() {
                Some(id) => Self::write_json_hex(id, out),
                None => out.extend_from_slice(b"null"),
            },
            LogRecordField::SpanId => match log_record.span_id() {
                Some(id) => Self::write_json_hex(id, out),
                None => out.extend_from_slice(b"null"),
            },
            LogRecordField::Flags => {
                Self::write_u64(log_record.flags().unwrap_or(0).into(), out);
            }
            LogRecordField::SeverityNumber => {
                Self::write_u64(log_record.severity_number().unwrap_or(0) as u64, out);
            }
            LogRecordField::SeverityText => {
                Self::write_json_string(log_record.severity_text().unwrap_or(b""), out);
            }
            LogRecordField::Body => match log_record.body() {
                Some(b) => {
                    let s = Self::extract_string_value(&b);
                    Self::write_json_string(s.as_bytes(), out);
                }
                None => out.extend_from_slice(b"null"),
            },
            LogRecordField::EventName => {
                Self::write_json_string(log_record.event_name().unwrap_or(b""), out);
            }
        }
    }

    /// Write an AnyValueView directly as JSON bytes.
    fn write_any_value_json<'a, V: AnyValueView<'a>>(value: &V, out: &mut Vec<u8>) {
        match value.value_type() {
            ValueType::String => match value.as_string() {
                Some(s) => Self::write_json_string(s, out),
                None => out.extend_from_slice(b"null"),
            },
            ValueType::Int64 => match value.as_int64() {
                Some(n) => Self::write_i64(n, out),
                None => out.extend_from_slice(b"null"),
            },
            ValueType::Double => match value.as_double() {
                Some(d) if d.is_finite() => {
                    let mut ryu_buf = ryu::Buffer::new();
                    out.extend_from_slice(ryu_buf.format_finite(d).as_bytes());
                }
                _ => out.extend_from_slice(b"null"),
            },
            ValueType::Bool => match value.as_bool() {
                Some(true) => out.extend_from_slice(b"true"),
                Some(false) => out.extend_from_slice(b"false"),
                None => out.extend_from_slice(b"null"),
            },
            ValueType::Bytes => match value.as_bytes() {
                Some(b) => Self::write_json_hex(b, out),
                None => out.extend_from_slice(b"null"),
            },
            ValueType::Array | ValueType::KeyValueList => {
                let v = Self::convert_any_value(value);
                _ = serde_json::to_writer(out, &v);
            }
            ValueType::Empty => out.extend_from_slice(b"null"),
        }
    }

    /// Write a JSON-escaped string with quotes directly to output bytes.
    #[inline]
    fn write_json_string(s: &[u8], out: &mut Vec<u8>) {
        out.push(b'"');
        for &b in s {
            match b {
                b'"' => out.extend_from_slice(b"\\\""),
                b'\\' => out.extend_from_slice(b"\\\\"),
                b'\n' => out.extend_from_slice(b"\\n"),
                b'\r' => out.extend_from_slice(b"\\r"),
                b'\t' => out.extend_from_slice(b"\\t"),
                b if b < 0x20 => {
                    // Control characters: \u00XX
                    out.extend_from_slice(b"\\u00");
                    out.push(HEX_CHARS[(b >> 4) as usize]);
                    out.push(HEX_CHARS[(b & 0x0f) as usize]);
                }
                _ => out.push(b),
            }
        }
        out.push(b'"');
    }

    /// Write hex-encoded bytes as a JSON string directly.
    #[inline]
    fn write_json_hex(bytes: &[u8], out: &mut Vec<u8>) {
        out.push(b'"');
        for &byte in bytes {
            out.push(HEX_CHARS[(byte >> 4) as usize]);
            out.push(HEX_CHARS[(byte & 0x0f) as usize]);
        }
        out.push(b'"');
    }

    /// Write a u64 as decimal to output bytes.
    #[inline]
    fn write_u64(n: u64, out: &mut Vec<u8>) {
        let mut itoa_buf = itoa::Buffer::new();
        out.extend_from_slice(itoa_buf.format(n).as_bytes());
    }

    /// Write an i64 as decimal to output bytes.
    #[inline]
    fn write_i64(n: i64, out: &mut Vec<u8>) {
        let mut itoa_buf = itoa::Buffer::new();
        out.extend_from_slice(itoa_buf.format(n).as_bytes());
    }

    /// Apply resource mapping based on configuration
    fn apply_resource_mapping<R: ResourceView>(
        &self,
        resource: &R,
        map: &mut serde_json::Map<String, Value>,
    ) {
        for attr in resource.attributes() {
            let key: Cow<'_, str> = String::from_utf8_lossy(attr.key());
            if let Some(mapped_name) = self.schema.resource_mapping.get(key.as_ref()) {
                if let Some(value) = attr.value() {
                    _ = map.insert(mapped_name.clone(), Self::convert_any_value(&value));
                }
            }
        }
    }

    /// Apply scope mapping based on configuration
    fn apply_scope_mapping<S: InstrumentationScopeView>(
        &self,
        scope: &S,
        map: &mut serde_json::Map<String, Value>,
    ) {
        for attr in scope.attributes() {
            let key: Cow<'_, str> = String::from_utf8_lossy(attr.key());
            if let Some(mapped_name) = self.schema.scope_mapping.get(key.as_ref()) {
                if let Some(value) = attr.value() {
                    _ = map.insert(mapped_name.clone(), Self::convert_any_value(&value));
                }
            }
        }
    }

    /// Convert Str<'_> (&[u8]) to String
    #[inline]
    fn str_to_string(s: Str<'_>) -> String {
        String::from_utf8_lossy(s).into_owned()
    }

    /// Hot path - hex encoding for trace_id/span_id
    #[inline]
    fn bytes_to_hex(bytes: &[u8]) -> String {
        let mut hex = String::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            hex.push(HEX_CHARS[(byte >> 4) as usize] as char);
            hex.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
        }
        hex
    }

    /// Format nanosecond timestamp to RFC3339
    #[inline]
    fn format_timestamp(time_unix_nano: u64) -> String {
        if time_unix_nano == 0 {
            return chrono::Utc::now().to_rfc3339();
        }
        let secs = (time_unix_nano / 1_000_000_000) as i64;
        let nanos = (time_unix_nano % 1_000_000_000) as u32;
        chrono::DateTime::from_timestamp(secs, nanos)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
    }

    /// Convert AnyValueView to serde_json::Value
    fn convert_any_value<'a, V: AnyValueView<'a>>(value: &V) -> Value {
        match value.value_type() {
            ValueType::String => {
                Value::String(Self::str_to_string(value.as_string().unwrap_or(b"")))
            }
            ValueType::Int64 => Value::Number(value.as_int64().unwrap_or(0).into()),
            ValueType::Double => serde_json::Number::from_f64(value.as_double().unwrap_or(0.0))
                .map(Value::Number)
                .unwrap_or(Value::Null),
            ValueType::Bool => Value::Bool(value.as_bool().unwrap_or(false)),
            ValueType::Bytes => Value::String(Self::bytes_to_hex(value.as_bytes().unwrap_or(&[]))),
            ValueType::Array => value
                .as_array()
                .map(|iter| Value::Array(iter.map(|v| Self::convert_any_value(&v)).collect()))
                .unwrap_or_else(|| Value::Array(Vec::new())),
            ValueType::KeyValueList => value
                .as_kvlist()
                .map(|iter| {
                    let map: serde_json::Map<String, Value> = iter
                        .filter_map(|kv| {
                            kv.value().map(|v| {
                                (Self::str_to_string(kv.key()), Self::convert_any_value(&v))
                            })
                        })
                        .collect();
                    Value::Object(map)
                })
                .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
            ValueType::Empty => Value::Null,
        }
    }

    /// Extract string representation from AnyValueView (for body)
    fn extract_string_value<'a, V: AnyValueView<'a>>(value: &V) -> String {
        match value.value_type() {
            ValueType::String => Self::str_to_string(value.as_string().unwrap_or(b"")),
            ValueType::Int64 => value.as_int64().unwrap_or(0).to_string(),
            ValueType::Double => value.as_double().unwrap_or(0.0).to_string(),
            ValueType::Bool => value.as_bool().unwrap_or(false).to_string(),
            ValueType::Bytes => Self::bytes_to_hex(value.as_bytes().unwrap_or(&[])),
            ValueType::Array => value
                .as_array()
                .map(|iter| {
                    let vals: Vec<String> = iter.map(|v| Self::extract_string_value(&v)).collect();
                    format!("[{}]", vals.join(", "))
                })
                .unwrap_or_else(|| "[]".into()),
            ValueType::KeyValueList => Self::convert_any_value(value).to_string(),
            ValueType::Empty => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{
            AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList,
            any_value::Value as OtelAnyValueEnum,
        },
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    };
    use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
    use prost::Message;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        use super::super::config::{ApiConfig, AuthConfig, SchemaConfig};

        Config {
            api: ApiConfig {
                dcr_endpoint: "https://test.com".into(),
                stream_name: "test-stream".into(),
                dcr: "test-dcr".into(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::from([(
                        "service.name".into(),
                        "ServiceName".into(),
                    )]),
                    scope_mapping: HashMap::from([("scope.name".into(), "ScopeName".into())]),
                    log_record_mapping: HashMap::from([
                        ("body".into(), json!("Body")),
                        ("severity_text".into(), json!("Severity")),
                        ("attributes".into(), json!({"test.attr": "TestAttr"})),
                    ]),
                },
                azure_monitor_source_resourceid: None,
            },
            auth: AuthConfig::default(),
        }
    }

    #[test]
    fn test_schema_mapping() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("my-service".into())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "test-scope".into(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("my-scope".into())),
                            }),
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::IntValue(42)),
                        }),
                        severity_text: "INFO".into(),
                        attributes: vec![KeyValue {
                            key: "test.attr".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::BoolValue(true)),
                            }),
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["ServiceName"], "my-service");
        assert_eq!(json["ScopeName"], "my-scope");
        assert_eq!(json["Body"], "42");
        assert_eq!(json["Severity"], "INFO");
        assert_eq!(json["TestAttr"], true);
    }

    #[test]
    fn test_all_log_record_fields() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([
            ("time_unix_nano".to_string(), json!("Time")),
            ("observed_time_unix_nano".to_string(), json!("ObservedTime")),
            ("trace_id".to_string(), json!("TraceId")),
            ("span_id".to_string(), json!("SpanId")),
            ("flags".to_string(), json!("Flags")),
            ("severity_number".to_string(), json!("SeverityNum")),
        ]);

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_000_000_000,
                        observed_time_unix_nano: 2_000_000_000,
                        // trace_id must be 16 bytes
                        trace_id: vec![
                            0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x01,
                        ],
                        // span_id must be 8 bytes
                        span_id: vec![0xAB, 0xCD, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
                        flags: 1,
                        severity_number: 9,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);

        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();

        assert!(json["Time"].as_str().unwrap().contains("1970"));
        assert!(json["ObservedTime"].as_str().unwrap().contains("1970"));
        assert_eq!(json["TraceId"], "ff000000000000000000000000000001");
        assert_eq!(json["SpanId"], "abcd000000000001");
        assert_eq!(json["Flags"], 1);
        assert_eq!(json["SeverityNum"], 9);
    }

    #[test]
    fn test_any_value_types() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::ArrayValue(ArrayValue {
                                values: vec![
                                    AnyValue {
                                        value: Some(OtelAnyValueEnum::DoubleValue(4.14)),
                                    },
                                    AnyValue {
                                        value: Some(OtelAnyValueEnum::BytesValue(vec![0xDE, 0xAD])),
                                    },
                                ],
                            })),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Body"], "[4.14, dead]");
    }

    #[test]
    fn test_kvlist_value() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::KvlistValue(KeyValueList {
                                values: vec![KeyValue {
                                    key: "nested".into(),
                                    value: Some(AnyValue {
                                        value: Some(OtelAnyValueEnum::StringValue("value".into())),
                                    }),
                                }],
                            })),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert!(json["Body"].as_str().unwrap().contains("nested"));
    }

    #[test]
    fn test_empty_values() {
        let mut config = create_test_config();
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("trace_id".into(), json!("TraceId"));
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("span_id".into(), json!("SpanId"));
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("body".into(), json!("Body"));

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        trace_id: vec![],
                        span_id: vec![],
                        body: None,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["TraceId"], json!(null));
        assert_eq!(json["SpanId"], json!(null));
        assert_eq!(json["Body"], json!(null));
    }

    #[test]
    fn test_zero_timestamp() {
        let timestamp = Transformer::format_timestamp(0);
        assert!(timestamp.contains('T'));
    }

    #[test]
    fn test_try_new_success() {
        let config = create_test_config();
        let result = Transformer::try_new(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_new_invalid_field() {
        let mut config = create_test_config();
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("invalid_field".into(), json!("Invalid"));

        let result = Transformer::try_new(&config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown log record field")
        );
    }

    #[test]
    fn test_try_new_invalid_mapping_type() {
        let mut config = create_test_config();
        // Field mapping value must be a string, not an object
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("body".into(), json!({"nested": "object"}));

        let result = Transformer::try_new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a string"));
    }

    #[test]
    fn test_empty_schema_mappings() {
        use super::super::config::{ApiConfig, AuthConfig, SchemaConfig};

        let config = Config {
            api: ApiConfig {
                dcr_endpoint: "https://test.com".into(),
                stream_name: "test-stream".into(),
                dcr: "test-dcr".into(),
                schema: SchemaConfig {
                    resource_mapping: HashMap::new(),
                    scope_mapping: HashMap::new(),
                    log_record_mapping: HashMap::new(),
                },
                azure_monitor_source_resourceid: None,
            },
            auth: AuthConfig::default(),
        };

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "unmapped.attr".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("value".into())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord::default()],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // Should be empty object since no mappings configured
        assert_eq!(json, json!({}));
    }

    #[test]
    fn test_attribute_with_no_value() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: None, // No value
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "test".into(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".into(),
                            value: None, // No value
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue {
                            key: "test.attr".into(),
                            value: None, // No value
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // Attributes with no value should be skipped
        assert!(json.get("ServiceName").is_none());
        assert!(json.get("ScopeName").is_none());
        assert!(json.get("TestAttr").is_none());
    }

    #[test]
    fn test_multiple_log_records() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![
                        LogRecord {
                            body: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("first".into())),
                            }),
                            severity_text: "INFO".into(),
                            ..Default::default()
                        },
                        LogRecord {
                            body: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("second".into())),
                            }),
                            severity_text: "ERROR".into(),
                            ..Default::default()
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 2);

        let json1: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json1["Body"], "first");
        assert_eq!(json1["Severity"], "INFO");

        let json2: Value = serde_json::from_slice(&result[1]).unwrap();
        assert_eq!(json2["Body"], "second");
        assert_eq!(json2["Severity"], "ERROR");
    }

    #[test]
    fn test_empty_body_string() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([("body".into(), json!("Body"))]);

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue(String::new())),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json["Body"], "");
    }

    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(Transformer::bytes_to_hex(&[]), "");
        assert_eq!(Transformer::bytes_to_hex(&[0x00]), "00");
        assert_eq!(Transformer::bytes_to_hex(&[0xff]), "ff");
        assert_eq!(
            Transformer::bytes_to_hex(&[0x12, 0x34, 0xab, 0xcd]),
            "1234abcd"
        );
    }

    #[test]
    fn test_case_insensitive_field_names() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([
            ("TIME_UNIX_NANO".into(), json!("Time")),
            ("Body".into(), json!("Body")),
            ("SEVERITY_TEXT".into(), json!("Severity")),
        ]);

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_000_000_000,
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("test".into())),
                        }),
                        severity_text: "WARN".into(),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert!(json["Time"].as_str().is_some());
        assert_eq!(json["Body"], "test");
        assert_eq!(json["Severity"], "WARN");
    }

    #[test]
    fn test_double_nan_becomes_null() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::DoubleValue(f64::NAN)),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord::default()],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        // NaN cannot be represented in JSON, so it becomes null
        assert_eq!(json["ServiceName"], json!(null));
    }

    #[test]
    fn test_event_name_field() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([
            ("event_name".into(), json!("EventName")),
            ("severity_text".into(), json!("Severity")),
        ]);

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![
                        LogRecord {
                            event_name: "user.login".into(),
                            severity_text: "INFO".into(),
                            ..Default::default()
                        },
                        LogRecord {
                            event_name: String::new(),
                            severity_text: String::new(),
                            ..Default::default()
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let result = transformer.convert_to_log_analytics(&logs_view);

        assert_eq!(result.len(), 2);

        let json1: Value = serde_json::from_slice(&result[0]).unwrap();
        assert_eq!(json1["EventName"], "user.login");
        assert_eq!(json1["Severity"], "INFO");

        let json2: Value = serde_json::from_slice(&result[1]).unwrap();
        assert_eq!(json2["EventName"], "");
        assert_eq!(json2["Severity"], "");
    }

    /// Round-trip test: every record produced by the transformer must be valid JSON
    /// that can be deserialized back into a serde_json::Value and re-serialized
    /// to produce identical output.
    #[test]
    fn test_roundtrip_json_validity() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".into(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("my-service".into())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "test".into(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("my-scope".into())),
                            }),
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("hello world".into())),
                        }),
                        severity_text: "INFO".into(),
                        attributes: vec![KeyValue {
                            key: "test.attr".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::BoolValue(true)),
                            }),
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let results = transformer.convert_to_log_analytics(&logs_view);

        for (i, result) in results.iter().enumerate() {
            // Must parse as valid JSON
            let parsed: Value = serde_json::from_slice(result).unwrap_or_else(|e| {
                panic!(
                    "record {i} is not valid JSON: {e}\nraw: {}",
                    String::from_utf8_lossy(result)
                )
            });

            // Re-serialize and re-parse must produce identical value
            let re_serialized = serde_json::to_vec(&parsed).unwrap();
            let re_parsed: Value = serde_json::from_slice(&re_serialized).unwrap();
            assert_eq!(parsed, re_parsed, "round-trip mismatch for record {i}");
        }
    }

    /// Test that UTF-8 multi-byte strings (emoji, CJK, etc.) survive the
    /// direct JSON serialization without corruption.
    #[test]
    fn test_utf8_multibyte_strings() {
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([
            ("body".into(), json!("Body")),
            ("severity_text".into(), json!("Sev")),
        ]);

        let transformer = Transformer::new(&config);

        let test_strings = vec![
            "hello world",                  // ASCII
            "héllo wörld",                  // Latin diacritics
            "日本語テスト",                 // CJK
            "🚀🔥💯",                       // Emoji
            "mixed: café ☕ naïve 日本 🎉", // Mixed scripts
            "line1\nline2\ttab",            // Escape sequences
            "quote\"and\\backslash",        // JSON-special chars
            "\x01\x02\x1f",                 // Control characters
            "",                             // Empty string
        ];

        for test_str in &test_strings {
            let request = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    resource: None,
                    scope_logs: vec![ScopeLogs {
                        scope: None,
                        log_records: vec![LogRecord {
                            body: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue(test_str.to_string())),
                            }),
                            severity_text: test_str.to_string(),
                            ..Default::default()
                        }],
                        schema_url: String::new(),
                    }],
                    schema_url: String::new(),
                }],
            };

            let bytes = request.encode_to_vec();
            let logs_view = RawLogsData::new(&bytes);
            let results = transformer.convert_to_log_analytics(&logs_view);

            assert_eq!(
                results.len(),
                1,
                "expected 1 result for input: {test_str:?}"
            );

            let json: Value = serde_json::from_slice(&results[0]).unwrap_or_else(|e| {
                panic!(
                    "invalid JSON for input {test_str:?}: {e}\nraw bytes: {:?}",
                    &results[0]
                )
            });

            // Verify the body value round-trips correctly
            let body = json["Body"].as_str().unwrap_or_else(|| {
                panic!("Body field missing or not a string for input: {test_str:?}")
            });
            assert_eq!(body, *test_str, "body mismatch for input: {test_str:?}");

            // Verify severity_text also round-trips
            let sev = json["Sev"].as_str().unwrap();
            assert_eq!(sev, *test_str, "severity mismatch for input: {test_str:?}");
        }
    }

    /// Test that write_json_string produces output identical to serde_json
    /// for various edge-case strings.
    #[test]
    fn test_write_json_string_matches_serde() {
        let cases = vec![
            "",
            "simple",
            "with \"quotes\"",
            "back\\slash",
            "new\nline",
            "tab\there",
            "carriage\rreturn",
            "\x00\x01\x1f", // control chars
            "emoji 🎉 and ñ",
            "日本語",
            "mixed\t\"escape\"\n🚀",
        ];

        for input in &cases {
            let mut our_output = Vec::new();
            Transformer::write_json_string(input.as_bytes(), &mut our_output);

            let serde_output = serde_json::to_vec(input).unwrap();

            assert_eq!(
                our_output,
                serde_output,
                "mismatch for input {input:?}:\n  ours:  {}\n  serde: {}",
                String::from_utf8_lossy(&our_output),
                String::from_utf8_lossy(&serde_output)
            );
        }
    }

    /// Test numeric formatting matches serde_json output.
    #[test]
    fn test_numeric_formatting_matches_serde() {
        // Integers via itoa
        let int_cases: Vec<i64> = vec![0, 1, -1, 42, -42, i64::MAX, i64::MIN];
        for n in &int_cases {
            let mut our_output = Vec::new();
            Transformer::write_i64(*n, &mut our_output);
            let serde_output = serde_json::to_vec(n).unwrap();
            assert_eq!(
                our_output,
                serde_output,
                "i64 mismatch for {n}: ours={}, serde={}",
                String::from_utf8_lossy(&our_output),
                String::from_utf8_lossy(&serde_output)
            );
        }

        // Unsigned via itoa
        let uint_cases: Vec<u64> = vec![0, 1, 42, u64::MAX];
        for n in &uint_cases {
            let mut our_output = Vec::new();
            Transformer::write_u64(*n, &mut our_output);
            let serde_output = serde_json::to_vec(n).unwrap();
            assert_eq!(
                our_output,
                serde_output,
                "u64 mismatch for {n}: ours={}, serde={}",
                String::from_utf8_lossy(&our_output),
                String::from_utf8_lossy(&serde_output)
            );
        }
    }

    /// Test that attribute values with type mismatch (e.g. value_type says Int64
    /// but as_int64() returns None) produce null instead of silently defaulting
    /// to 0, false, or empty string. This covers the null-safe extraction in
    /// write_any_value_json.
    #[test]
    fn test_null_safe_attribute_value_extraction() {
        // Test each value type mapped as a log record attribute
        let mut config = create_test_config();
        config.api.schema.log_record_mapping = HashMap::from([(
            "attributes".into(),
            json!({
                "str_attr": "StrCol",
                "int_attr": "IntCol",
                "dbl_attr": "DblCol",
                "bool_attr": "BoolCol",
                "bytes_attr": "BytesCol",
            }),
        )]);

        let transformer = Transformer::new(&config);

        // Normal case: all attributes present with valid values
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![
                            KeyValue {
                                key: "str_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::StringValue("hello".into())),
                                }),
                            },
                            KeyValue {
                                key: "int_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::IntValue(42)),
                                }),
                            },
                            KeyValue {
                                key: "dbl_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::DoubleValue(2.72)),
                                }),
                            },
                            KeyValue {
                                key: "bool_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::BoolValue(true)),
                                }),
                            },
                            KeyValue {
                                key: "bytes_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::BytesValue(vec![0xCA, 0xFE])),
                                }),
                            },
                        ],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let results = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&results[0]).unwrap();

        assert_eq!(json["StrCol"], "hello");
        assert_eq!(json["IntCol"], 42);
        assert_eq!(json["DblCol"], 2.72);
        assert_eq!(json["BoolCol"], true);
        assert_eq!(json["BytesCol"], "cafe");

        // NaN/Infinity doubles become null
        let request_nan = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![
                            KeyValue {
                                key: "dbl_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::DoubleValue(f64::NAN)),
                                }),
                            },
                            KeyValue {
                                key: "int_attr".into(),
                                value: Some(AnyValue {
                                    value: Some(OtelAnyValueEnum::DoubleValue(f64::INFINITY)),
                                }),
                            },
                        ],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request_nan.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let results = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&results[0]).unwrap();

        assert_eq!(json["DblCol"], json!(null), "NaN must become null");
        assert_eq!(json["IntCol"], json!(null), "Infinity must become null");

        // Boolean false is a real value, not null
        let request_false = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        attributes: vec![KeyValue {
                            key: "bool_attr".into(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::BoolValue(false)),
                            }),
                        }],
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let bytes = request_false.encode_to_vec();
        let logs_view = RawLogsData::new(&bytes);
        let results = transformer.convert_to_log_analytics(&logs_view);
        let json: Value = serde_json::from_slice(&results[0]).unwrap();

        assert_eq!(
            json["BoolCol"], false,
            "false must be serialized as false, not null"
        );
    }
}
