// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value as OtelAnyValueEnum;
use serde_json::{Value, json};

use super::config::{Config, SchemaConfig};

const ATTRIBUTES_FIELD: &str = "attributes";

/// Converts OTLP logs to Azure Log Analytics format
pub struct Transformer {
    schema: SchemaConfig,
}

impl Transformer {
    /// Create a new transformer with the given configuration.
    /// Must be used; constructing and discarding would be a no-op.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            schema: config.api.schema.clone(),
        }
    }

    /// Convert OTLP logs to flat JSON objects for Log Analytics.
    /// Must be used; otherwise transformed entries are lost.
    #[must_use]
    pub fn convert_to_log_analytics(&self, request: &ExportLogsServiceRequest) -> Vec<Value> {
        let mut entries = Vec::new();

        for resource_logs in &request.resource_logs {
            let resource_attrs = if !self.schema.disable_schema_mapping {
                // Schema mapping enabled: only add mapped resource attributes
                self.apply_resource_mapping(&resource_logs.resource)
            } else {
                // Schema mapping disabled: no resource attributes in legacy format
                serde_json::Map::new()
            };

            for scope_logs in &resource_logs.scope_logs {
                let scope_attrs = if !self.schema.disable_schema_mapping {
                    // Schema mapping enabled: only add mapped scope attributes
                    self.apply_scope_mapping(&scope_logs.scope)
                } else {
                    // Schema mapping disabled: no scope attributes in legacy format
                    serde_json::Map::new()
                };

                for log_record in &scope_logs.log_records {
                    let mut entry = serde_json::Map::new();

                    if self.schema.disable_schema_mapping {
                        // Legacy transform when schema mapping is disabled
                        self.legacy_transform(&mut entry, log_record);
                    } else {
                        // Apply configured mappings when schema mapping is enabled

                        // Add resource and scope attributes first
                        for (k, v) in &resource_attrs {
                            let _ = entry.insert(k.clone(), v.clone());
                        }
                        for (k, v) in &scope_attrs {
                            let _ = entry.insert(k.clone(), v.clone());
                        }

                        // Transform log record based on mapping
                        if let Err(e) = self.transform_log_record(&mut entry, log_record) {
                            log::warn!("Failed to transform log record: {e}");
                            continue;
                        }
                    }

                    entries.push(Value::Object(entry));
                }
            }
        }

        entries
    }

    /// Legacy transform when schema mapping is disabled (matches Go implementation)
    fn legacy_transform(
        &self,
        destination: &mut serde_json::Map<String, Value>,
        log_record: &opentelemetry_proto::tonic::logs::v1::LogRecord,
    ) {
        // Use timestamp or fallback to observed timestamp
        let timestamp = if log_record.time_unix_nano != 0 {
            self.format_timestamp(log_record.time_unix_nano)
        } else {
            self.format_timestamp(log_record.observed_time_unix_nano)
        };
        let _ = destination.insert("TimeGenerated".to_string(), json!(timestamp));

        // Add raw data as body string
        if let Some(ref body) = log_record.body {
            if let Some(ref value) = body.value {
                let body_str = self.extract_string_value(value);
                let _ = destination.insert("RawData".to_string(), json!(body_str));
            }
        }
    }

    /// Transform log record fields based on the log_record_mapping configuration
    fn transform_log_record(
        &self,
        destination: &mut serde_json::Map<String, Value>,
        log_record: &opentelemetry_proto::tonic::logs::v1::LogRecord,
    ) -> Result<(), String> {
        // Process each mapping in log_record_mapping
        for (key, value) in &self.schema.log_record_mapping {
            if key == ATTRIBUTES_FIELD {
                // Handle nested attribute mapping
                if let Some(attr_mapping) = value.as_object() {
                    for (attr_key, attr_value) in attr_mapping {
                        if let Some(actual_value) =
                            self.extract_attribute(&log_record.attributes, attr_key)
                        {
                            let field_name = attr_value
                                .as_str()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| attr_value.to_string());
                            let _ = destination.insert(field_name, actual_value);
                        }
                    }
                }
            } else {
                // Handle direct log record field mapping - PROPAGATE ERRORS
                let log_record_value = self.extract_value_from_log_record(key, log_record)?;
                let field_name = value
                    .as_str()
                    .ok_or_else(|| "Field mapping value must be a string".to_string())?;
                let _ = destination.insert(field_name.to_string(), log_record_value);
            }
        }

        Ok(())
    }

    /// Extract value from log record properties by field name
    fn extract_value_from_log_record(
        &self,
        key: &str,
        log_record: &opentelemetry_proto::tonic::logs::v1::LogRecord,
    ) -> Result<Value, String> {
        let key_lower = key.to_lowercase();
        match key_lower.as_str() {
            "time_unix_nano" => {
                let timestamp = self.format_timestamp(log_record.time_unix_nano);
                Ok(json!(timestamp))
            }
            "observed_time_unix_nano" => {
                let timestamp = self.format_timestamp(log_record.observed_time_unix_nano);
                Ok(json!(timestamp))
            }
            "trace_id" => {
                if log_record.trace_id.is_empty() {
                    Ok(json!(null))
                } else {
                    let trace_id = self.bytes_to_hex(&log_record.trace_id);
                    Ok(json!(trace_id))
                }
            }
            "span_id" => {
                if log_record.span_id.is_empty() {
                    Ok(json!(null))
                } else {
                    let span_id = self.bytes_to_hex(&log_record.span_id);
                    Ok(json!(span_id))
                }
            }
            "flags" => Ok(json!(log_record.flags)),
            "severity_number" => Ok(json!(log_record.severity_number as i64)),
            "severity_text" => Ok(json!(log_record.severity_text)),
            "body" => {
                if let Some(ref body) = log_record.body {
                    if let Some(ref value) = body.value {
                        let body_str = self.extract_string_value(value);
                        Ok(json!(body_str))
                    } else {
                        Ok(json!(null))
                    }
                } else {
                    Ok(json!(null))
                }
            }
            _ => Err(format!("Unknown field name: {key}")),
        }
    }

    /// Extract attribute value by key from the attributes list
    fn extract_attribute(
        &self,
        attributes: &[opentelemetry_proto::tonic::common::v1::KeyValue],
        key: &str,
    ) -> Option<Value> {
        for attr in attributes {
            if attr.key == key {
                if let Some(ref value) = attr.value {
                    if let Some(ref v) = value.value {
                        return Some(self.convert_any_value(v));
                    }
                }
            }
        }
        None
    }

    /// Apply resource mapping based on configuration
    fn apply_resource_mapping(
        &self,
        resource: &Option<opentelemetry_proto::tonic::resource::v1::Resource>,
    ) -> serde_json::Map<String, Value> {
        let mut attrs = serde_json::Map::new();

        if let Some(resource) = resource {
            for (key, mapped_name) in &self.schema.resource_mapping {
                if let Some(actual_value) = self.extract_attribute(&resource.attributes, key) {
                    let _ = attrs.insert(mapped_name.clone(), actual_value);
                }
            }
        }

        attrs
    }

    /// Apply scope mapping based on configuration
    fn apply_scope_mapping(
        &self,
        scope: &Option<opentelemetry_proto::tonic::common::v1::InstrumentationScope>,
    ) -> serde_json::Map<String, Value> {
        let mut attrs = serde_json::Map::new();

        if let Some(scope) = scope {
            for (key, mapped_name) in &self.schema.scope_mapping {
                if let Some(actual_value) = self.extract_attribute(&scope.attributes, key) {
                    let _ = attrs.insert(mapped_name.clone(), actual_value);
                }
            }
        }

        attrs
    }

    /// Extract string value from AnyValue (matches Go's AsString behavior)
    fn extract_string_value(&self, value: &OtelAnyValueEnum) -> String {
        match value {
            OtelAnyValueEnum::StringValue(s) => s.clone(),
            OtelAnyValueEnum::IntValue(i) => i.to_string(),
            OtelAnyValueEnum::DoubleValue(d) => d.to_string(),
            OtelAnyValueEnum::BoolValue(b) => b.to_string(),
            OtelAnyValueEnum::ArrayValue(arr) => {
                let values: Vec<String> = arr
                    .values
                    .iter()
                    .filter_map(|v| v.value.as_ref())
                    .map(|v| self.extract_string_value(v))
                    .collect();
                format!("[{}]", values.join(", "))
            }
            OtelAnyValueEnum::KvlistValue(_) => {
                // Convert to JSON string for complex values
                let json_val = self.convert_any_value(value);
                json_val.to_string()
            }
            OtelAnyValueEnum::BytesValue(bytes) => self.bytes_to_hex(bytes),
        }
    }

    /// Convert AnyValue to JSON Value
    fn convert_any_value(&self, value: &OtelAnyValueEnum) -> Value {
        match value {
            OtelAnyValueEnum::StringValue(s) => json!(s),
            OtelAnyValueEnum::IntValue(i) => json!(i),
            OtelAnyValueEnum::DoubleValue(d) => json!(d),
            OtelAnyValueEnum::BoolValue(b) => json!(b),
            OtelAnyValueEnum::ArrayValue(arr) => {
                let values: Vec<Value> = arr
                    .values
                    .iter()
                    .filter_map(|v| v.value.as_ref())
                    .map(|v| self.convert_any_value(v))
                    .collect();
                json!(values)
            }
            OtelAnyValueEnum::KvlistValue(kv) => {
                let mut map = serde_json::Map::new();
                for item in &kv.values {
                    if let Some(value) = &item.value {
                        if let Some(v) = &value.value {
                            let _ = map.insert(item.key.clone(), self.convert_any_value(v));
                        }
                    }
                }
                Value::Object(map)
            }
            OtelAnyValueEnum::BytesValue(bytes) => json!(self.bytes_to_hex(bytes)),
        }
    }

    /// Convert bytes to hex string
    fn bytes_to_hex(&self, bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    }

    /// Format timestamp from Unix nano to datetime string
    fn format_timestamp(&self, time_unix_nano: u64) -> String {
        if time_unix_nano > 0 {
            let secs = (time_unix_nano / 1_000_000_000) as i64;
            let nanos = (time_unix_nano % 1_000_000_000) as u32;
            if let Some(dt) = chrono::DateTime::from_timestamp(secs, nanos) {
                // Use RFC3339 format to match Go's time.Time format
                dt.to_rfc3339()
            } else {
                chrono::Utc::now().to_rfc3339()
            }
        } else {
            chrono::Utc::now().to_rfc3339()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use opentelemetry_proto::tonic::logs::v1::LogRecord;
    use opentelemetry_proto::tonic::logs::v1::ResourceLogs;
    use opentelemetry_proto::tonic::logs::v1::ScopeLogs;
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        Config {
            client_config: HashMap::new(),
            api: super::super::config::ApiConfig {
                dcr_endpoint: "https://example.com".to_string(),
                stream_name: "test_stream".to_string(),
                dcr: "test_dcr".to_string(),
                schema: SchemaConfig::default(),
            },
            auth: super::super::config::AuthConfig::default(),
        }
    }

    fn create_test_config_with_schema(
        resource_mapping: HashMap<String, String>,
        scope_mapping: HashMap<String, String>,
        log_record_mapping: HashMap<String, Value>,
        disable_schema_mapping: bool,
    ) -> Config {
        let mut config = create_test_config();
        config.api.schema = SchemaConfig {
            resource_mapping,
            scope_mapping,
            log_record_mapping,
            disable_schema_mapping,
        };
        config
    }

    // ===================== Helper Functions =====================

    #[test]
    fn test_bytes_to_hex() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        assert_eq!(transformer.bytes_to_hex(&[]), "".to_string());
        assert_eq!(transformer.bytes_to_hex(&[0]), "00".to_string());
        assert_eq!(transformer.bytes_to_hex(&[255]), "ff".to_string());
        assert_eq!(transformer.bytes_to_hex(&[1, 2, 3]), "010203".to_string());
        assert_eq!(
            transformer.bytes_to_hex(&[0xab, 0xcd, 0xef]),
            "abcdef".to_string()
        );
    }

    #[test]
    fn test_format_timestamp_with_valid_time() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        // 1609459200000000000 is 2021-01-01T00:00:00Z in nanoseconds
        let timestamp = transformer.format_timestamp(1609459200000000000);
        assert!(timestamp.contains("2021-01-01"));
    }

    #[test]
    fn test_format_timestamp_with_zero() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let timestamp = transformer.format_timestamp(0);
        // Should return current time
        assert!(!timestamp.is_empty());
        assert!(timestamp.contains('T')); // ISO format
    }

    #[test]
    fn test_format_timestamp_with_nanos() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        // Test with specific nanoseconds
        let time_nano = 1000000000000000001u64; // 1 second + 1 nano
        let timestamp = transformer.format_timestamp(time_nano);
        assert!(!timestamp.is_empty());
    }

    // ===================== Value Conversion Tests =====================

    #[test]
    fn test_extract_string_value_string() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::StringValue("hello".to_string());
        assert_eq!(transformer.extract_string_value(&value), "hello");
    }

    #[test]
    fn test_extract_string_value_int() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::IntValue(42);
        assert_eq!(transformer.extract_string_value(&value), "42");
    }

    #[test]
    fn test_extract_string_value_double() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::DoubleValue(3.14);
        assert!(transformer.extract_string_value(&value).contains("3.14"));
    }

    #[test]
    fn test_extract_string_value_bool() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value_true = OtelAnyValueEnum::BoolValue(true);
        let value_false = OtelAnyValueEnum::BoolValue(false);
        assert_eq!(transformer.extract_string_value(&value_true), "true");
        assert_eq!(transformer.extract_string_value(&value_false), "false");
    }

    #[test]
    fn test_extract_string_value_bytes() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::BytesValue(vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(transformer.extract_string_value(&value), "deadbeef");
    }

    #[test]
    fn test_extract_string_value_array() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut array_value = opentelemetry_proto::tonic::common::v1::ArrayValue::default();
        array_value.values = vec![
            AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("a".to_string())),
            },
            AnyValue {
                value: Some(OtelAnyValueEnum::IntValue(1)),
            },
        ];

        let value = OtelAnyValueEnum::ArrayValue(array_value);
        let result = transformer.extract_string_value(&value);
        assert_eq!(result, "[a, 1]");
    }

    #[test]
    fn test_extract_string_value_kvlist() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut kvlist = opentelemetry_proto::tonic::common::v1::KeyValueList::default();
        kvlist.values = vec![KeyValue {
            key: "key1".to_string(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("val1".to_string())),
            }),
        }];

        let value = OtelAnyValueEnum::KvlistValue(kvlist);
        let result = transformer.extract_string_value(&value);
        assert!(result.contains("key1"));
        assert!(result.contains("val1"));
    }

    #[test]
    fn test_convert_any_value_string() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::StringValue("test".to_string());
        let result = transformer.convert_any_value(&value);
        assert_eq!(result, json!("test"));
    }

    #[test]
    fn test_convert_any_value_int() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::IntValue(123);
        let result = transformer.convert_any_value(&value);
        assert_eq!(result, json!(123));
    }

    #[test]
    fn test_convert_any_value_double() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::DoubleValue(2.71);
        let result = transformer.convert_any_value(&value);
        assert_eq!(result, json!(2.71));
    }

    #[test]
    fn test_convert_any_value_bool() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::BoolValue(true);
        let result = transformer.convert_any_value(&value);
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_convert_any_value_bytes() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let value = OtelAnyValueEnum::BytesValue(vec![1, 2, 3]);
        let result = transformer.convert_any_value(&value);
        assert_eq!(result, json!("010203"));
    }

    #[test]
    fn test_convert_any_value_array() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut array_value = opentelemetry_proto::tonic::common::v1::ArrayValue::default();
        array_value.values = vec![
            AnyValue {
                value: Some(OtelAnyValueEnum::IntValue(1)),
            },
            AnyValue {
                value: Some(OtelAnyValueEnum::IntValue(2)),
            },
        ];

        let value = OtelAnyValueEnum::ArrayValue(array_value);
        let result = transformer.convert_any_value(&value);
        assert_eq!(result, json!([1, 2]));
    }

    #[test]
    fn test_convert_any_value_kvlist() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut kvlist = opentelemetry_proto::tonic::common::v1::KeyValueList::default();
        kvlist.values = vec![
            KeyValue {
                key: "name".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue("Alice".to_string())),
                }),
            },
            KeyValue {
                key: "age".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::IntValue(30)),
                }),
            },
        ];

        let value = OtelAnyValueEnum::KvlistValue(kvlist);
        let result = transformer.convert_any_value(&value);

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap(), "Alice");
        assert_eq!(obj.get("age").unwrap(), 30);
    }

    // ===================== Attribute Extraction Tests =====================

    #[test]
    fn test_extract_attribute_found() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let attributes = vec![KeyValue {
            key: "service.name".to_string(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("my-service".to_string())),
            }),
        }];

        let result = transformer.extract_attribute(&attributes, "service.name");
        assert_eq!(result, Some(json!("my-service")));
    }

    #[test]
    fn test_extract_attribute_not_found() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let attributes = vec![KeyValue {
            key: "service.name".to_string(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("my-service".to_string())),
            }),
        }];

        let result = transformer.extract_attribute(&attributes, "service.version");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_attribute_empty_list() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let attributes: Vec<KeyValue> = vec![];
        let result = transformer.extract_attribute(&attributes, "key");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_attribute_multiple_values() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let attributes = vec![
            KeyValue {
                key: "key1".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue("value1".to_string())),
                }),
            },
            KeyValue {
                key: "key2".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::IntValue(42)),
                }),
            },
            KeyValue {
                key: "key3".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::BoolValue(true)),
                }),
            },
        ];

        assert_eq!(
            transformer.extract_attribute(&attributes, "key1"),
            Some(json!("value1"))
        );
        assert_eq!(
            transformer.extract_attribute(&attributes, "key2"),
            Some(json!(42))
        );
        assert_eq!(
            transformer.extract_attribute(&attributes, "key3"),
            Some(json!(true))
        );
    }

    // ===================== Extract Value from Log Record Tests =====================

    #[test]
    fn test_extract_value_time_unix_nano() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.time_unix_nano = 1609459200000000000;

        let result = transformer.extract_value_from_log_record("time_unix_nano", &log_record);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_string());
        assert!(value.as_str().unwrap().contains("2021-01-01"));
    }

    #[test]
    fn test_extract_value_observed_time_unix_nano() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.observed_time_unix_nano = 1609459200000000000;

        let result =
            transformer.extract_value_from_log_record("observed_time_unix_nano", &log_record);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_string());
    }

    #[test]
    fn test_extract_value_trace_id_non_empty() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.trace_id = vec![0x01, 0x02, 0x03, 0x04];

        let result = transformer.extract_value_from_log_record("trace_id", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("01020304"));
    }

    #[test]
    fn test_extract_value_trace_id_empty() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let log_record = LogRecord::default();

        let result = transformer.extract_value_from_log_record("trace_id", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(null));
    }

    #[test]
    fn test_extract_value_span_id_non_empty() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.span_id = vec![0xaa, 0xbb];

        let result = transformer.extract_value_from_log_record("span_id", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("aabb"));
    }

    #[test]
    fn test_extract_value_span_id_empty() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let log_record = LogRecord::default();

        let result = transformer.extract_value_from_log_record("span_id", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(null));
    }

    #[test]
    fn test_extract_value_flags() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.flags = 0x01;

        let result = transformer.extract_value_from_log_record("flags", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(1));
    }

    #[test]
    fn test_extract_value_severity_number() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.severity_number = 17;

        let result = transformer.extract_value_from_log_record("severity_number", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(17));
    }

    #[test]
    fn test_extract_value_severity_text() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.severity_text = "ERROR".to_string();

        let result = transformer.extract_value_from_log_record("severity_text", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("ERROR"));
    }

    #[test]
    fn test_extract_value_body_with_string() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("Log message".to_string())),
        });

        let result = transformer.extract_value_from_log_record("body", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!("Log message"));
    }

    #[test]
    fn test_extract_value_body_none() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let log_record = LogRecord::default();

        let result = transformer.extract_value_from_log_record("body", &log_record);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(null));
    }

    #[test]
    fn test_extract_value_unknown_field() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let log_record = LogRecord::default();

        let result = transformer.extract_value_from_log_record("unknown_field", &log_record);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown field name"));
    }

    #[test]
    fn test_extract_value_case_insensitive() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.flags = 5;

        // Test various case combinations
        let result1 = transformer.extract_value_from_log_record("FLAGS", &log_record);
        let result2 = transformer.extract_value_from_log_record("Flags", &log_record);
        let result3 = transformer.extract_value_from_log_record("flags", &log_record);

        assert_eq!(result1.unwrap(), json!(5));
        assert_eq!(result2.unwrap(), json!(5));
        assert_eq!(result3.unwrap(), json!(5));
    }

    // ===================== Resource and Scope Mapping Tests =====================

    #[test]
    fn test_apply_resource_mapping_empty() {
        let config =
            create_test_config_with_schema(HashMap::new(), HashMap::new(), HashMap::new(), false);
        let transformer = Transformer::new(&config);

        let result = transformer.apply_resource_mapping(&None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_resource_mapping_with_attributes() {
        let mut resource_mapping = HashMap::new();
        let _ = resource_mapping.insert("service.name".to_string(), "ServiceName".to_string());
        let _ = resource_mapping.insert("host.name".to_string(), "HostName".to_string());

        let config =
            create_test_config_with_schema(resource_mapping, HashMap::new(), HashMap::new(), false);
        let transformer = Transformer::new(&config);

        let mut resource = Resource::default();
        resource.attributes = vec![
            KeyValue {
                key: "service.name".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue("payment-api".to_string())),
                }),
            },
            KeyValue {
                key: "host.name".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue("server-01".to_string())),
                }),
            },
        ];

        let result = transformer.apply_resource_mapping(&Some(resource));
        assert_eq!(result.get("ServiceName").unwrap(), "payment-api");
        assert_eq!(result.get("HostName").unwrap(), "server-01");
    }

    #[test]
    fn test_apply_resource_mapping_missing_attributes() {
        let mut resource_mapping = HashMap::new();
        let _ = resource_mapping.insert("service.name".to_string(), "ServiceName".to_string());
        let _ = resource_mapping.insert("missing.attr".to_string(), "MissingAttr".to_string());

        let config =
            create_test_config_with_schema(resource_mapping, HashMap::new(), HashMap::new(), false);
        let transformer = Transformer::new(&config);

        let mut resource = Resource::default();
        resource.attributes = vec![KeyValue {
            key: "service.name".to_string(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("api".to_string())),
            }),
        }];

        let result = transformer.apply_resource_mapping(&Some(resource));
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("ServiceName").unwrap(), "api");
    }

    #[test]
    fn test_apply_scope_mapping_empty() {
        let config =
            create_test_config_with_schema(HashMap::new(), HashMap::new(), HashMap::new(), false);
        let transformer = Transformer::new(&config);

        let result = transformer.apply_scope_mapping(&None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_scope_mapping_with_attributes() {
        let mut scope_mapping = HashMap::new();
        let _ = scope_mapping.insert(
            "instrumentation.name".to_string(),
            "InstrumentationName".to_string(),
        );

        let config =
            create_test_config_with_schema(HashMap::new(), scope_mapping, HashMap::new(), false);
        let transformer = Transformer::new(&config);

        let mut scope = InstrumentationScope::default();
        scope.name = "tracer-provider".to_string();
        scope.attributes = vec![KeyValue {
            key: "instrumentation.name".to_string(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("otel-js".to_string())),
            }),
        }];

        let result = transformer.apply_scope_mapping(&Some(scope));
        assert_eq!(result.get("InstrumentationName").unwrap(), "otel-js");
    }

    #[test]
    fn test_apply_scope_mapping_with_multiple_attributes() {
        let mut scope_mapping = HashMap::new();
        let _ = scope_mapping.insert(
            "otel.library.name".to_string(),
            "InstrumentationLibrary".to_string(),
        );
        let _ = scope_mapping.insert(
            "otel.library.version".to_string(),
            "InstrumentationVersion".to_string(),
        );

        let config =
            create_test_config_with_schema(HashMap::new(), scope_mapping, HashMap::new(), false);
        let transformer = Transformer::new(&config);

        let mut scope = InstrumentationScope::default();
        scope.name = "my-logger".to_string();
        scope.version = "1.0.0".to_string();
        scope.attributes = vec![
            KeyValue {
                key: "otel.library.name".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue(
                        "my-instrumentation".to_string(),
                    )),
                }),
            },
            KeyValue {
                key: "otel.library.version".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue("1.2.3".to_string())),
                }),
            },
        ];

        let result = transformer.apply_scope_mapping(&Some(scope));
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get("InstrumentationLibrary").unwrap(),
            "my-instrumentation"
        );
        assert_eq!(result.get("InstrumentationVersion").unwrap(), "1.2.3");
    }

    // ===================== Legacy Transform Tests =====================

    #[test]
    fn test_legacy_transform_with_time_unix_nano() {
        let config =
            create_test_config_with_schema(HashMap::new(), HashMap::new(), HashMap::new(), true);
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.time_unix_nano = 1609459200000000000;
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("test log".to_string())),
        });

        let mut entry = serde_json::Map::new();
        transformer.legacy_transform(&mut entry, &log_record);

        assert!(entry.contains_key("TimeGenerated"));
        assert!(entry.contains_key("RawData"));
        assert_eq!(entry.get("RawData").unwrap(), "test log");
    }

    #[test]
    fn test_legacy_transform_fallback_to_observed_time() {
        let config =
            create_test_config_with_schema(HashMap::new(), HashMap::new(), HashMap::new(), true);
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.time_unix_nano = 0; // Will fallback to observed_time_unix_nano
        log_record.observed_time_unix_nano = 1609459200000000000;
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("observed log".to_string())),
        });

        let mut entry = serde_json::Map::new();
        transformer.legacy_transform(&mut entry, &log_record);

        assert!(entry.contains_key("TimeGenerated"));
        assert!(entry.get("RawData").unwrap() == "observed log");
    }

    #[test]
    fn test_legacy_transform_no_body() {
        let config =
            create_test_config_with_schema(HashMap::new(), HashMap::new(), HashMap::new(), true);
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.time_unix_nano = 1609459200000000000;
        log_record.body = None;

        let mut entry = serde_json::Map::new();
        transformer.legacy_transform(&mut entry, &log_record);

        assert!(entry.contains_key("TimeGenerated"));
        assert!(!entry.contains_key("RawData"));
    }

    // ===================== Full Transformation Tests =====================

    #[test]
    fn test_transform_log_record_with_mappings() {
        let mut log_record_mapping = HashMap::new();
        let _ = log_record_mapping.insert("body".to_string(), json!("Message"));
        let _ = log_record_mapping.insert("severity_text".to_string(), json!("Level"));

        let config = create_test_config_with_schema(
            HashMap::new(),
            HashMap::new(),
            log_record_mapping,
            false,
        );
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("error occurred".to_string())),
        });
        log_record.severity_text = "ERROR".to_string();

        let mut entry = serde_json::Map::new();
        let result = transformer.transform_log_record(&mut entry, &log_record);

        assert!(result.is_ok());
        assert_eq!(entry.get("Message").unwrap(), "error occurred");
        assert_eq!(entry.get("Level").unwrap(), "ERROR");
    }

    #[test]
    fn test_transform_log_record_with_attribute_mapping() {
        let mut attr_mapping = serde_json::Map::new();
        let _ = attr_mapping.insert("request.id".to_string(), json!("RequestId"));
        let _ = attr_mapping.insert("user.id".to_string(), json!("UserId"));

        let mut log_record_mapping = HashMap::new();
        let _ = log_record_mapping.insert("attributes".to_string(), Value::Object(attr_mapping));

        let config = create_test_config_with_schema(
            HashMap::new(),
            HashMap::new(),
            log_record_mapping,
            false,
        );
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.attributes = vec![
            KeyValue {
                key: "request.id".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::StringValue("req-123".to_string())),
                }),
            },
            KeyValue {
                key: "user.id".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::IntValue(42)),
                }),
            },
        ];

        let mut entry = serde_json::Map::new();
        let result = transformer.transform_log_record(&mut entry, &log_record);

        assert!(result.is_ok());
        assert_eq!(entry.get("RequestId").unwrap(), "req-123");
        assert_eq!(entry.get("UserId").unwrap(), 42);
    }

    #[test]
    fn test_transform_log_record_with_non_string_attribute_mapping_value() {
        let mut attr_mapping = serde_json::Map::new();
        let _ = attr_mapping.insert("count".to_string(), json!(123)); // Non-string value
        let _ = attr_mapping.insert("active".to_string(), json!(true)); // Boolean value

        let mut log_record_mapping = HashMap::new();
        let _ = log_record_mapping.insert("attributes".to_string(), Value::Object(attr_mapping));

        let config = create_test_config_with_schema(
            HashMap::new(),
            HashMap::new(),
            log_record_mapping,
            false,
        );
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.attributes = vec![
            KeyValue {
                key: "count".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::IntValue(42)),
                }),
            },
            KeyValue {
                key: "active".to_string(),
                value: Some(AnyValue {
                    value: Some(OtelAnyValueEnum::BoolValue(true)),
                }),
            },
        ];

        let mut entry = serde_json::Map::new();
        let result = transformer.transform_log_record(&mut entry, &log_record);

        assert!(result.is_ok());
        // Non-string mapping values should be converted to their JSON string representation
        assert_eq!(entry.get("123").unwrap(), 42);
        assert_eq!(entry.get("true").unwrap(), true);
    }

    #[test]
    fn test_transform_log_record_with_direct_field_non_string_mapping_fails() {
        let mut log_record_mapping = HashMap::new();
        let _ = log_record_mapping.insert("body".to_string(), json!(123)); // Non-string field name mapping

        let config = create_test_config_with_schema(
            HashMap::new(),
            HashMap::new(),
            log_record_mapping,
            false,
        );
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("test".to_string())),
        });

        let mut entry = serde_json::Map::new();
        let result = transformer.transform_log_record(&mut entry, &log_record);

        // Direct field mapping with non-string value should fail
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Field mapping value must be a string")
        );
    }

    #[test]
    fn test_convert_to_log_analytics_legacy_mode() {
        let config =
            create_test_config_with_schema(HashMap::new(), HashMap::new(), HashMap::new(), true);
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.time_unix_nano = 1609459200000000000;
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("test message".to_string())),
        });

        let mut scope_logs = ScopeLogs::default();
        scope_logs.log_records = vec![log_record];

        let mut resource_logs = ResourceLogs::default();
        resource_logs.scope_logs = vec![scope_logs];

        let mut request = ExportLogsServiceRequest::default();
        request.resource_logs = vec![resource_logs];

        let results = transformer.convert_to_log_analytics(&request);

        assert_eq!(results.len(), 1);
        let entry = results[0].as_object().unwrap();
        assert!(entry.contains_key("TimeGenerated"));
        assert!(entry.contains_key("RawData"));
    }

    #[test]
    fn test_convert_to_log_analytics_with_schema_mapping() {
        let mut log_record_mapping = HashMap::new();
        let _ = log_record_mapping.insert("body".to_string(), json!("LogMessage"));
        let _ = log_record_mapping.insert("trace_id".to_string(), json!("TraceId"));

        let config = create_test_config_with_schema(
            HashMap::new(),
            HashMap::new(),
            log_record_mapping,
            false,
        );
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue(
                "application error".to_string(),
            )),
        });
        log_record.trace_id = vec![0x01, 0x02];

        let mut scope_logs = ScopeLogs::default();
        scope_logs.log_records = vec![log_record];

        let mut resource_logs = ResourceLogs::default();
        resource_logs.scope_logs = vec![scope_logs];

        let mut request = ExportLogsServiceRequest::default();
        request.resource_logs = vec![resource_logs];

        let results = transformer.convert_to_log_analytics(&request);

        assert_eq!(results.len(), 1);
        let entry = results[0].as_object().unwrap();
        assert_eq!(entry.get("LogMessage").unwrap(), "application error");
        assert_eq!(entry.get("TraceId").unwrap(), "0102");
    }

    #[test]
    fn test_convert_to_log_analytics_multiple_records() {
        let mut log_record_mapping = HashMap::new();
        let _ = log_record_mapping.insert("body".to_string(), json!("Message"));

        let config = create_test_config_with_schema(
            HashMap::new(),
            HashMap::new(),
            log_record_mapping,
            false,
        );
        let transformer = Transformer::new(&config);

        let mut log_record1 = LogRecord::default();
        log_record1.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("message 1".to_string())),
        });

        let mut log_record2 = LogRecord::default();
        log_record2.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("message 2".to_string())),
        });

        let mut scope_logs = ScopeLogs::default();
        scope_logs.log_records = vec![log_record1, log_record2];

        let mut resource_logs = ResourceLogs::default();
        resource_logs.scope_logs = vec![scope_logs];

        let mut request = ExportLogsServiceRequest::default();
        request.resource_logs = vec![resource_logs];

        let results = transformer.convert_to_log_analytics(&request);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_convert_to_log_analytics_empty_request() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest::default();
        let results = transformer.convert_to_log_analytics(&request);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_convert_to_log_analytics_with_resource_and_scope_mapping() {
        let mut resource_mapping = HashMap::new();
        let _ = resource_mapping.insert("service.name".to_string(), "Service".to_string());

        let mut scope_mapping = HashMap::new();
        let _ = scope_mapping.insert(
            "instrumentation.name".to_string(),
            "Instrumentation".to_string(),
        );

        let mut log_record_mapping = HashMap::new();
        let _ = log_record_mapping.insert("body".to_string(), json!("Message"));

        let config = create_test_config_with_schema(
            resource_mapping,
            scope_mapping,
            log_record_mapping,
            false,
        );
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("test".to_string())),
        });

        let mut scope = InstrumentationScope::default();
        scope.attributes = vec![KeyValue {
            key: "instrumentation.name".to_string(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("otel-rust".to_string())),
            }),
        }];

        let mut scope_logs = ScopeLogs::default();
        scope_logs.scope = Some(scope);
        scope_logs.log_records = vec![log_record];

        let mut resource = Resource::default();
        resource.attributes = vec![KeyValue {
            key: "service.name".to_string(),
            value: Some(AnyValue {
                value: Some(OtelAnyValueEnum::StringValue("my-service".to_string())),
            }),
        }];

        let mut resource_logs = ResourceLogs::default();
        resource_logs.resource = Some(resource);
        resource_logs.scope_logs = vec![scope_logs];

        let mut request = ExportLogsServiceRequest::default();
        request.resource_logs = vec![resource_logs];

        let results = transformer.convert_to_log_analytics(&request);

        assert_eq!(results.len(), 1);
        let entry = results[0].as_object().unwrap();
        assert_eq!(entry.get("Service").unwrap(), "my-service");
        assert_eq!(entry.get("Instrumentation").unwrap(), "otel-rust");
        assert_eq!(entry.get("Message").unwrap(), "test");
    }

    #[test]
    fn test_convert_to_log_analytics_with_invalid_field_mapping() {
        let mut log_record_mapping = HashMap::new();
        let _ = log_record_mapping.insert("invalid_field".to_string(), json!("Field"));
        let _ = log_record_mapping.insert("body".to_string(), json!("Message"));

        let config = create_test_config_with_schema(
            HashMap::new(),
            HashMap::new(),
            log_record_mapping,
            false,
        );
        let transformer = Transformer::new(&config);

        let mut log_record = LogRecord::default();
        log_record.body = Some(AnyValue {
            value: Some(OtelAnyValueEnum::StringValue("test".to_string())),
        });

        let mut scope_logs = ScopeLogs::default();
        scope_logs.log_records = vec![log_record];

        let mut resource_logs = ResourceLogs::default();
        resource_logs.scope_logs = vec![scope_logs];

        let mut request = ExportLogsServiceRequest::default();
        request.resource_logs = vec![resource_logs];

        let results = transformer.convert_to_log_analytics(&request);

        // Invalid field causes entire record to be skipped
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_transformer_creation() {
        let config = create_test_config();
        let transformer = Transformer::new(&config);

        assert_eq!(transformer.schema.disable_schema_mapping, false);
    }
}
