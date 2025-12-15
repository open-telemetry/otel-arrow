// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::{BufMut, Bytes, BytesMut};
use otap_df_pdata::views::{
    common::{AnyValueView, AttributeView, InstrumentationScopeView, Str, ValueType},
    logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView},
    resource::ResourceView,
};
use serde::Serialize;
use serde_json::{Value, json};

use super::config::{Config, SchemaConfig};

const ATTRIBUTES_FIELD: &str = "attributes";
const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

/// Flat view of final log line: resource + scope + record
#[derive(Serialize)]
struct LogEntry<'a> {
    #[serde(flatten)]
    resource: &'a serde_json::Map<String, Value>,

    #[serde(flatten)]
    scope: &'a serde_json::Map<String, Value>,

    #[serde(flatten)]
    record: serde_json::Map<String, Value>,
}

// TODO: Performance review and improvements
// TODO: Make sure mapping of all fields are covered
/// Converts OTLP logs to Azure Log Analytics format
pub struct Transformer {
    schema: SchemaConfig,
}

#[allow(clippy::print_stdout)]
impl Transformer {
    /// Create a new Transformer with the given configuration
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            schema: config.api.schema.clone(),
        }
    }

    /// High-perf, single-threaded: one reusable BytesMut, grows to max size, no extra copies.
    /// Now accepts any type implementing `LogsDataView` (both `OtapLogsView` and `RawLogsData`).
    #[must_use]
    pub fn convert_to_log_analytics<T: LogsDataView>(&self, logs_view: &T) -> Vec<Bytes> {
        let mut results = Vec::new();
        let mut buf = BytesMut::with_capacity(512);

        for resource_logs in logs_view.resources() {
            let resource_attrs = if self.schema.disable_schema_mapping {
                serde_json::Map::new()
            } else {
                resource_logs
                    .resource()
                    .map(|r| self.apply_resource_mapping(&r))
                    .unwrap_or_default()
            };

            for scope_logs in resource_logs.scopes() {
                let scope_attrs = if self.schema.disable_schema_mapping {
                    serde_json::Map::new()
                } else {
                    scope_logs
                        .scope()
                        .map(|s| self.apply_scope_mapping(&s))
                        .unwrap_or_default()
                };

                for log_record in scope_logs.log_records() {
                    let record_map = if self.schema.disable_schema_mapping {
                        Self::legacy_transform_to_map(&log_record)
                    } else {
                        match self.transform_log_record_to_map(&log_record) {
                            Ok(m) => m,
                            Err(e) => {
                                println!(
                                    "[AzureMonitorExporter] Skipping log record due to transformation error: {e}"
                                );
                                continue;
                            }
                        }
                    };

                    let entry = LogEntry {
                        resource: &resource_attrs,
                        scope: &scope_attrs,
                        record: record_map,
                    };

                    buf.clear();
                    {
                        let writer = (&mut buf).writer();
                        let mut ser = serde_json::Serializer::new(writer);
                        if let Err(e) = entry.serialize(&mut ser) {
                            println!("Failed to serialize log entry: {e}");
                            continue;
                        }
                    }
                    results.push(buf.clone().freeze());
                }
            }
        }

        results
    }

    /// Legacy transform when schema mapping is disabled
    fn legacy_transform_to_map<R: LogRecordView>(log_record: &R) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();

        let timestamp = log_record
            .time_unix_nano()
            .filter(|&ts| ts != 0)
            .or_else(|| log_record.observed_time_unix_nano())
            .unwrap_or(0);

        _ = map.insert(
            "TimeGenerated".into(),
            json!(Self::format_timestamp(timestamp)),
        );

        if let Some(body) = log_record.body() {
            _ = map.insert("RawData".into(), json!(Self::extract_string_value(&body)));
        }

        map
    }

    /// Apply resource mapping based on configuration
    fn apply_resource_mapping<R: ResourceView>(
        &self,
        resource: &R,
    ) -> serde_json::Map<String, Value> {
        let mut attrs = serde_json::Map::new();

        for attr in resource.attributes() {
            let key = Self::str_to_string(attr.key());
            if let Some(mapped_name) = self.schema.resource_mapping.get(&key) {
                if let Some(value) = attr.value() {
                    _ = attrs.insert(mapped_name.clone(), Self::convert_any_value(&value));
                }
            }
        }

        attrs
    }

    /// Apply scope mapping based on configuration
    fn apply_scope_mapping<S: InstrumentationScopeView>(
        &self,
        scope: &S,
    ) -> serde_json::Map<String, Value> {
        let mut attrs = serde_json::Map::new();

        for attr in scope.attributes() {
            let key = Self::str_to_string(attr.key());
            if let Some(mapped_name) = self.schema.scope_mapping.get(&key) {
                if let Some(value) = attr.value() {
                    _ = attrs.insert(mapped_name.clone(), Self::convert_any_value(&value));
                }
            }
        }

        attrs
    }

    /// Transform log record fields into a Map
    fn transform_log_record_to_map<R: LogRecordView>(
        &self,
        log_record: &R,
    ) -> Result<serde_json::Map<String, Value>, String> {
        let mut map = serde_json::Map::new();

        for (key, value) in &self.schema.log_record_mapping {
            if key == ATTRIBUTES_FIELD {
                self.map_attributes(&mut map, log_record, value)?;
            } else {
                let field_name = value
                    .as_str()
                    .ok_or("Field mapping value must be a string")?;
                let extracted = Self::extract_field_value(key, log_record)?;
                _ = map.insert(field_name.into(), extracted);
            }
        }

        Ok(map)
    }

    /// Map log record attributes based on attr_mapping config
    fn map_attributes<R: LogRecordView>(
        &self,
        dest: &mut serde_json::Map<String, Value>,
        log_record: &R,
        attr_mapping_value: &Value,
    ) -> Result<(), String> {
        let Some(attr_mapping) = attr_mapping_value.as_object() else {
            return Ok(());
        };

        for attr in log_record.attributes() {
            let attr_key = Self::str_to_string(attr.key());
            if let Some(mapped_field) = attr_mapping.get(&attr_key) {
                if let Some(val) = attr.value() {
                    let field_name = mapped_field
                        .as_str()
                        .map(String::from)
                        .unwrap_or_else(|| mapped_field.to_string());
                    _ = dest.insert(field_name, Self::convert_any_value(&val));
                }
            }
        }

        Ok(())
    }

    /// Extract value from log record by field name
    fn extract_field_value<R: LogRecordView>(key: &str, log_record: &R) -> Result<Value, String> {
        match key.to_lowercase().as_str() {
            "time_unix_nano" => Ok(json!(Self::format_timestamp(
                log_record.time_unix_nano().unwrap_or(0)
            ))),
            "observed_time_unix_nano" => Ok(json!(Self::format_timestamp(
                log_record.observed_time_unix_nano().unwrap_or(0)
            ))),
            "trace_id" => Ok(log_record
                .trace_id()
                .map(|id| json!(Self::bytes_to_hex(id)))
                .unwrap_or(Value::Null)),
            "span_id" => Ok(log_record
                .span_id()
                .map(|id| json!(Self::bytes_to_hex(id)))
                .unwrap_or(Value::Null)),
            "flags" => Ok(json!(log_record.flags().unwrap_or(0))),
            "severity_number" => Ok(json!(log_record.severity_number().unwrap_or(0) as i64)),
            "severity_text" => Ok(json!(Self::str_to_string(
                log_record.severity_text().unwrap_or(b"")
            ))),
            "body" => Ok(log_record
                .body()
                .map(|b| json!(Self::extract_string_value(&b)))
                .unwrap_or(Value::Null)),
            _ => Err(format!("Unknown field name: {key}")),
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
            ValueType::String => json!(Self::str_to_string(value.as_string().unwrap_or(b""))),
            ValueType::Int64 => json!(value.as_int64().unwrap_or(0)),
            ValueType::Double => json!(value.as_double().unwrap_or(0.0)),
            ValueType::Bool => json!(value.as_bool().unwrap_or(false)),
            ValueType::Bytes => json!(Self::bytes_to_hex(value.as_bytes().unwrap_or(&[]))),
            ValueType::Array => value
                .as_array()
                .map(|iter| {
                    json!(
                        iter.map(|v| Self::convert_any_value(&v))
                            .collect::<Vec<_>>()
                    )
                })
                .unwrap_or(json!([])),
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
                .unwrap_or(json!({})),
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
    use std::collections::HashMap;

    fn create_test_config(disable_mapping: bool) -> Config {
        use super::super::config::{ApiConfig, AuthConfig, SchemaConfig};

        Config {
            api: ApiConfig {
                dcr_endpoint: "https://test.com".into(),
                stream_name: "test-stream".into(),
                dcr: "test-dcr".into(),
                schema: SchemaConfig {
                    disable_schema_mapping: disable_mapping,
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
    fn test_legacy_transform() {
        let config = create_test_config(true);
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_000_000_000,
                        observed_time_unix_nano: 2_000_000_000,
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("test body".into())),
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

        assert_eq!(result.len(), 1);
        let json: Value = serde_json::from_slice(&result[0]).unwrap();
        assert!(json["TimeGenerated"].as_str().is_some());
        assert_eq!(json["RawData"], "test body");
    }

    #[test]
    fn test_schema_mapping() {
        let config = create_test_config(false);
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
        let mut config = create_test_config(false);
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
        let config = create_test_config(false);
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
        let config = create_test_config(false);
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
        let mut config = create_test_config(false);
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
    fn test_invalid_mapping_error() {
        let mut config = create_test_config(false);
        _ = config
            .api
            .schema
            .log_record_mapping
            .insert("invalid_field".into(), json!("Invalid"));

        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
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
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_zero_timestamp() {
        let timestamp = Transformer::format_timestamp(0);
        assert!(timestamp.contains('T'));
    }

    #[test]
    fn test_observed_time_fallback() {
        let config = create_test_config(true);
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 0,
                        observed_time_unix_nano: 3_000_000_000,
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
        assert!(json["TimeGenerated"].as_str().unwrap().contains("1970"));
    }
}
