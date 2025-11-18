// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value as OtelAnyValueEnum;
use serde_json::{Value, json};

use super::config::{Config, SchemaConfig};

const ATTRIBUTES_FIELD: &str = "attributes";

// TODO: Performance review and improvements
// TODO: Make sure mapping of all fields are covered
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
    // TODO: Remove print_stdout after logging is set up
    #[allow(clippy::print_stdout)]
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
                        Self::legacy_transform(&mut entry, log_record);
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
                        // TODO: a mechanism needed to handle logs that are dropped
                        if let Err(e) = self.transform_log_record(&mut entry, log_record) {
                            // TODO: log it appropriately (perhaps as part of monmon)
                            println!("Failed to transform log record: {e}");
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
        destination: &mut serde_json::Map<String, Value>,
        log_record: &opentelemetry_proto::tonic::logs::v1::LogRecord,
    ) {
        // Use timestamp or fallback to observed timestamp
        let timestamp = if log_record.time_unix_nano != 0 {
            Self::format_timestamp(log_record.time_unix_nano)
        } else {
            Self::format_timestamp(log_record.observed_time_unix_nano)
        };
        let _ = destination.insert("TimeGenerated".to_string(), json!(timestamp));

        // Add raw data as body string
        if let Some(ref body) = log_record.body {
            if let Some(ref value) = body.value {
                let body_str = Self::extract_string_value(value);
                let _ = destination.insert("RawData".to_string(), json!(body_str));
            }
        }
    }

    /// Apply resource mapping based on configuration  
    fn apply_resource_mapping(
        &self,
        resource: &Option<opentelemetry_proto::tonic::resource::v1::Resource>,
    ) -> serde_json::Map<String, Value> {
        let mut attrs = serde_json::Map::new();

        if let Some(resource) = resource {
            // Iterate over attributes once, lookup in HashMap for each
            for attr in &resource.attributes {
                if let Some(mapped_name) = self.schema.resource_mapping.get(&attr.key) {
                    if let Some(ref value) = attr.value {
                        if let Some(ref v) = value.value {
                            let _ = attrs.insert(mapped_name.clone(), Self::convert_any_value(v));
                        }
                    }
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
            // Iterate over attributes once, lookup in HashMap for each
            for attr in &scope.attributes {
                if let Some(mapped_name) = self.schema.scope_mapping.get(&attr.key) {
                    if let Some(ref value) = attr.value {
                        if let Some(ref v) = value.value {
                            let _ = attrs.insert(mapped_name.clone(), Self::convert_any_value(v));
                        }
                    }
                }
            }
        }

        attrs
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
                    // Iterate over log attributes once, lookup each in the mapping
                    for attr in &log_record.attributes {
                        if let Some(attr_value) = attr_mapping.get(&attr.key) {
                            if let Some(ref value) = attr.value {
                                if let Some(ref v) = value.value {
                                    let field_name = attr_value
                                        .as_str()
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| attr_value.to_string());
                                    let _ =
                                        destination.insert(field_name, Self::convert_any_value(v));
                                }
                            }
                        }
                    }
                }
            } else {
                // Handle direct log record field mapping - PROPAGATE ERRORS
                let log_record_value = Self::extract_value_from_log_record(key, log_record)?;
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
        key: &str,
        log_record: &opentelemetry_proto::tonic::logs::v1::LogRecord,
    ) -> Result<Value, String> {
        let key_lower = key.to_lowercase();
        match key_lower.as_str() {
            "time_unix_nano" => {
                let timestamp = Self::format_timestamp(log_record.time_unix_nano);
                Ok(json!(timestamp))
            }
            "observed_time_unix_nano" => {
                let timestamp = Self::format_timestamp(log_record.observed_time_unix_nano);
                Ok(json!(timestamp))
            }
            "trace_id" => {
                if log_record.trace_id.is_empty() {
                    Ok(json!(null))
                } else {
                    let trace_id = Self::bytes_to_hex(&log_record.trace_id);
                    Ok(json!(trace_id))
                }
            }
            "span_id" => {
                if log_record.span_id.is_empty() {
                    Ok(json!(null))
                } else {
                    let span_id = Self::bytes_to_hex(&log_record.span_id);
                    Ok(json!(span_id))
                }
            }
            "flags" => Ok(json!(log_record.flags)),
            "severity_number" => Ok(json!(log_record.severity_number as i64)),
            "severity_text" => Ok(json!(log_record.severity_text)),
            "body" => {
                if let Some(ref body) = log_record.body {
                    if let Some(ref value) = body.value {
                        let body_str = Self::extract_string_value(value);
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

    /// Convert AnyValue to JSON Value
    fn convert_any_value(value: &OtelAnyValueEnum) -> Value {
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
                    .map(Self::convert_any_value)
                    .collect();
                json!(values)
            }
            OtelAnyValueEnum::KvlistValue(kv) => {
                let mut map = serde_json::Map::new();
                for item in &kv.values {
                    if let Some(value) = &item.value {
                        if let Some(v) = &value.value {
                            let _ = map.insert(item.key.clone(), Self::convert_any_value(v));
                        }
                    }
                }
                Value::Object(map)
            }
            OtelAnyValueEnum::BytesValue(bytes) => json!(Self::bytes_to_hex(bytes)),
        }
    }

    /// Convert bytes to hex string
    fn bytes_to_hex(bytes: &[u8]) -> String {
        // Pre-allocate the exact size needed (2 hex chars per byte)
        // This avoids repeated allocations
        const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

        let mut hex = String::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            hex.push(HEX_CHARS[(byte >> 4) as usize] as char);
            hex.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
        }
        hex
    }

    /// Format timestamp from Unix nano to datetime string
    fn format_timestamp(time_unix_nano: u64) -> String {
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

    /// Extract string value from AnyValue (matches Go's AsString behavior)
    fn extract_string_value(value: &OtelAnyValueEnum) -> String {
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
                    .map(Self::extract_string_value)
                    .collect();
                format!("[{}]", values.join(", "))
            }
            OtelAnyValueEnum::KvlistValue(_) => {
                // Convert to JSON string for complex values
                let json_val = Self::convert_any_value(value);
                json_val.to_string()
            }
            OtelAnyValueEnum::BytesValue(bytes) => Self::bytes_to_hex(bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    };
    use std::collections::HashMap;

    fn create_test_config(disable_mapping: bool) -> Config {
        use super::super::config::{ApiConfig, AuthConfig, Config, SchemaConfig};

        Config {
            api: ApiConfig {
                dcr_endpoint: "https://test.com".to_string(),
                stream_name: "test-stream".to_string(),
                dcr: "test-dcr".to_string(),
                schema: SchemaConfig {
                    disable_schema_mapping: disable_mapping,
                    resource_mapping: HashMap::from([(
                        "service.name".to_string(),
                        "ServiceName".to_string(),
                    )]),
                    scope_mapping: HashMap::from([(
                        "scope.name".to_string(),
                        "ScopeName".to_string(),
                    )]),
                    log_record_mapping: HashMap::from([
                        ("body".to_string(), json!("Body")),
                        ("severity_text".to_string(), json!("Severity")),
                        (
                            "attributes".to_string(),
                            json!({
                                "test.attr": "TestAttr"
                            }),
                        ),
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
                            value: Some(OtelAnyValueEnum::StringValue("test body".to_string())),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let result = transformer.convert_to_log_analytics(&request);
        assert_eq!(result.len(), 1);
        assert!(result[0]["TimeGenerated"].as_str().is_some());
        assert_eq!(result[0]["RawData"], "test body");
    }

    #[test]
    fn test_schema_mapping() {
        let config = create_test_config(false);
        let transformer = Transformer::new(&config);

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::StringValue("my-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "test-scope".to_string(),
                        version: String::new(),
                        attributes: vec![KeyValue {
                            key: "scope.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(OtelAnyValueEnum::StringValue("my-scope".to_string())),
                            }),
                        }],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        body: Some(AnyValue {
                            value: Some(OtelAnyValueEnum::IntValue(42)),
                        }),
                        severity_text: "INFO".to_string(),
                        attributes: vec![KeyValue {
                            key: "test.attr".to_string(),
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

        let result = transformer.convert_to_log_analytics(&request);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["ServiceName"], "my-service");
        assert_eq!(result[0]["ScopeName"], "my-scope");
        assert_eq!(result[0]["Body"], "42");
        assert_eq!(result[0]["Severity"], "INFO");
        assert_eq!(result[0]["TestAttr"], true);
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
                        trace_id: vec![0xFF, 0x00],
                        span_id: vec![0xAB, 0xCD],
                        flags: 1,
                        severity_number: 9,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let result = transformer.convert_to_log_analytics(&request);
        assert!(result[0]["Time"].as_str().unwrap().contains("1970"));
        assert!(result[0]["ObservedTime"].as_str().unwrap().contains("1970"));
        assert_eq!(result[0]["TraceId"], "ff00");
        assert_eq!(result[0]["SpanId"], "abcd");
        assert_eq!(result[0]["Flags"], 1);
        assert_eq!(result[0]["SeverityNum"], 9);
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

        let result = transformer.convert_to_log_analytics(&request);
        assert_eq!(result[0]["Body"], "[4.14, dead]");
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
                                    key: "nested".to_string(),
                                    value: Some(AnyValue {
                                        value: Some(OtelAnyValueEnum::StringValue(
                                            "value".to_string(),
                                        )),
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

        let result = transformer.convert_to_log_analytics(&request);
        assert!(result[0]["Body"].as_str().unwrap().contains("nested"));
    }

    #[test]
    fn test_empty_values() {
        let mut config = create_test_config(false);
        let _ = config
            .api
            .schema
            .log_record_mapping
            .insert("trace_id".to_string(), json!("TraceId"));
        let _ = config
            .api
            .schema
            .log_record_mapping
            .insert("span_id".to_string(), json!("SpanId"));
        let _ = config
            .api
            .schema
            .log_record_mapping
            .insert("body".to_string(), json!("Body"));

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

        let result = transformer.convert_to_log_analytics(&request);
        assert_eq!(result[0]["TraceId"], json!(null));
        assert_eq!(result[0]["SpanId"], json!(null));
        assert_eq!(result[0]["Body"], json!(null));
    }

    #[test]
    fn test_invalid_mapping_error() {
        let mut config = create_test_config(false);
        let _ = config
            .api
            .schema
            .log_record_mapping
            .insert("invalid_field".to_string(), json!("Invalid"));

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

        let result = transformer.convert_to_log_analytics(&request);
        assert_eq!(result.len(), 0); // Record skipped due to error
    }

    #[test]
    fn test_zero_timestamp() {
        let timestamp = Transformer::format_timestamp(0);
        assert!(timestamp.contains('T')); // RFC3339 format
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

        let result = transformer.convert_to_log_analytics(&request);
        assert!(
            result[0]["TimeGenerated"]
                .as_str()
                .unwrap()
                .contains("1970")
        );
    }
}
