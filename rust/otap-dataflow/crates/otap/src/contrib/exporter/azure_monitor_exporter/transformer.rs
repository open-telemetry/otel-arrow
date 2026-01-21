// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::{BufMut, Bytes, BytesMut};
use otap_df_pdata::views::{
    common::{AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType},
    logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView},
    resource::ResourceView,
};
use serde::Serialize;
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

#[allow(clippy::print_stdout)]
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

    /// High-perf, single-threaded: one reusable BytesMut, grows to max size, no extra copies.
    /// Now accepts any type implementing `LogsDataView` (both `OtapLogsView` and `RawLogsData`).
    #[must_use]
    pub fn convert_to_log_analytics<T: LogsDataView>(&self, logs_view: &T) -> Vec<Bytes> {
        let mut results = Vec::with_capacity(1024);
        let mut buf = BytesMut::with_capacity(2048);
        let mut record_map = serde_json::Map::new();

        for resource_logs in logs_view.resources() {
            for scope_logs in resource_logs.scopes() {
                for log_record in scope_logs.log_records() {
                    record_map.clear();

                    // normally config should be validated to avoid duplicate keys, but if that
                    // ever happens for any reason such as a bug, then the logic below ensures that
                    // the lowest level fields override the higher level ones.

                    // apply resource mapping first to allow scope to override if needed
                    if let Some(r) = resource_logs.resource() {
                        self.apply_resource_mapping(&r, &mut record_map);
                    }

                    // apply scope mapping next to allow log record to override if needed
                    if let Some(s) = scope_logs.scope() {
                        self.apply_scope_mapping(&s, &mut record_map);
                    }

                    self.transform_log_record_to_map(&log_record, &mut record_map);

                    buf.clear();
                    {
                        let writer = (&mut buf).writer();
                        let mut ser = serde_json::Serializer::new(writer);
                        if let Err(e) = record_map.serialize(&mut ser) {
                            println!("Failed to serialize log entry: {e}");
                            continue;
                        }
                    }

                    results.push(buf.split().freeze());
                }
            }
        }

        results
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

    /// Transform log record fields into a Map (no longer returns Result - validation done at construction)
    fn transform_log_record_to_map<R: LogRecordView>(
        &self,
        log_record: &R,
        map: &mut serde_json::Map<String, Value>,
    ) {
        // Process pre-parsed field mappings
        for field_mapping in &self.schema.field_mappings {
            let value = Self::extract_field_value(field_mapping.source, log_record);
            _ = map.insert(field_mapping.dest.clone(), value);
        }

        // Process attribute mappings
        if !self.schema.attribute_mapping.is_empty() {
            for attr in log_record.attributes() {
                let attr_key: Cow<'_, str> = String::from_utf8_lossy(attr.key());
                if let Some(dest_field) = self.schema.attribute_mapping.get(attr_key.as_ref()) {
                    if let Some(val) = attr.value() {
                        _ = map.insert(dest_field.clone(), Self::convert_any_value(&val));
                    }
                }
            }
        }
    }

    /// Extract value from log record by pre-parsed field enum (no string comparison needed)
    #[inline]
    fn extract_field_value<R: LogRecordView>(field: LogRecordField, log_record: &R) -> Value {
        match field {
            LogRecordField::TimeUnixNano => Value::String(Self::format_timestamp(
                log_record.time_unix_nano().unwrap_or(0),
            )),
            LogRecordField::ObservedTimeUnixNano => Value::String(Self::format_timestamp(
                log_record.observed_time_unix_nano().unwrap_or(0),
            )),
            LogRecordField::TraceId => log_record
                .trace_id()
                .map(|id| Value::String(Self::bytes_to_hex(id)))
                .unwrap_or(Value::Null),
            LogRecordField::SpanId => log_record
                .span_id()
                .map(|id| Value::String(Self::bytes_to_hex(id)))
                .unwrap_or(Value::Null),
            LogRecordField::Flags => Value::Number(log_record.flags().unwrap_or(0).into()),
            LogRecordField::SeverityNumber => {
                Value::Number((log_record.severity_number().unwrap_or(0) as i64).into())
            }
            LogRecordField::SeverityText => Value::String(Self::str_to_string(
                log_record.severity_text().unwrap_or(b""),
            )),
            LogRecordField::Body => log_record
                .body()
                .map(|b| Value::String(Self::extract_string_value(&b)))
                .unwrap_or(Value::Null),
            LogRecordField::EventName => {
                Value::String(Self::str_to_string(log_record.event_name().unwrap_or(b"")))
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

        let result: Vec<Bytes> = transformer.convert_to_log_analytics(&logs_view);
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
}
