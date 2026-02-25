// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTel SDK-specific telemetry configuration. Note: this is presently
//! used only for internal metrics reporting, not for internal logging.

pub mod metrics;

use crate::settings::telemetry::logs::LogsConfig;
use metrics::MetricsConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, time::Duration};

/// Telemetry backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TelemetryConfig {
    /// The size of the reporting channel, measured in the number of internal metric events shared across all cores.
    #[serde(default = "default_reporting_channel_size")]
    pub reporting_channel_size: usize,
    /// The interval at which metrics are flushed and aggregated by the collector.
    #[serde(with = "humantime_serde", default = "default_reporting_interval")]
    #[schemars(with = "String")]
    pub reporting_interval: Duration,
    /// Metrics system configuration.
    #[serde(default)]
    pub metrics: MetricsConfig,
    /// Internal logs configuration.
    #[serde(default)]
    pub logs: LogsConfig,
    /// Resource attributes to associate with telemetry data.
    #[serde(default)]
    pub resource: HashMap<String, AttributeValue>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            metrics: MetricsConfig::default(),
            logs: LogsConfig::default(),
            resource: HashMap::default(),
            reporting_channel_size: default_reporting_channel_size(),
            reporting_interval: default_reporting_interval(),
        }
    }
}

const fn default_reporting_channel_size() -> usize {
    100
}

const fn default_reporting_interval() -> Duration {
    Duration::from_secs(1)
}

/// Attribute value types for telemetry resource attributes.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub enum AttributeValue {
    /// String type attribute value.
    String(String),
    /// Boolean type attribute value.
    Bool(bool),
    /// 64-bit integer type attribute value.
    I64(i64),
    /// 64-bit floating point type attribute value.
    F64(f64),
    /// Array type attribute value.
    Array(AttributeValueArray),
}

impl<'de> Deserialize<'de> for AttributeValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct AttributeValueVisitor;

        impl<'de> Visitor<'de> for AttributeValueVisitor {
            type Value = AttributeValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a string, boolean, number, or array")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(AttributeValue::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(AttributeValue::I64(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value > i64::MAX as u64 {
                    let message = format!("value {} out of range (max {})", value, i64::MAX);
                    return Err(de::Error::custom(message));
                }
                Ok(AttributeValue::I64(value as i64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(AttributeValue::F64(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(AttributeValue::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(AttributeValue::String(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                // Deserialize all elements as AttributeValue first
                let mut values: Vec<AttributeValue> = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }

                if values.is_empty() {
                    // Default to empty string array
                    return Ok(AttributeValue::Array(AttributeValueArray::String(
                        Vec::new(),
                    )));
                }

                // Determine type from first element and convert
                match &values[0] {
                    AttributeValue::Bool(_) => {
                        let bools: Result<Vec<bool>, _> = values
                            .into_iter()
                            .map(|v| match v {
                                AttributeValue::Bool(b) => Ok(b),
                                _ => Err(de::Error::custom("expected all elements to be booleans")),
                            })
                            .collect();
                        Ok(AttributeValue::Array(AttributeValueArray::Bool(bools?)))
                    }
                    AttributeValue::I64(_) => {
                        let ints: Result<Vec<i64>, _> = values
                            .into_iter()
                            .map(|v| match v {
                                AttributeValue::I64(i) => Ok(i),
                                _ => Err(de::Error::custom("expected all elements to be integers")),
                            })
                            .collect();
                        Ok(AttributeValue::Array(AttributeValueArray::I64(ints?)))
                    }
                    AttributeValue::F64(_) => {
                        let floats: Result<Vec<f64>, _> = values
                            .into_iter()
                            .map(|v| match v {
                                AttributeValue::F64(f) => Ok(f),
                                _ => Err(de::Error::custom("expected all elements to be floats")),
                            })
                            .collect();
                        Ok(AttributeValue::Array(AttributeValueArray::F64(floats?)))
                    }
                    AttributeValue::String(_) => {
                        let strings: Result<Vec<String>, _> = values
                            .into_iter()
                            .map(|v| match v {
                                AttributeValue::String(s) => Ok(s),
                                _ => Err(de::Error::custom("expected all elements to be strings")),
                            })
                            .collect();
                        Ok(AttributeValue::Array(AttributeValueArray::String(strings?)))
                    }
                    AttributeValue::Array(_) => {
                        Err(de::Error::custom("nested arrays are not supported"))
                    }
                }
            }
        }

        deserializer.deserialize_any(AttributeValueVisitor)
    }
}

/// Array attribute value types for telemetry resource attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum AttributeValueArray {
    /// Array of bools
    Bool(Vec<bool>),
    /// Array of integers
    I64(Vec<i64>),
    /// Array of floats
    F64(Vec<f64>),
    /// Array of strings
    String(Vec<String>),
}

/// A telemetry attribute with a value and an optional brief description.
///
/// Supports two YAML/JSON forms:
/// - **Bare value**: `attr_name: "value"` (brief defaults to `None`)
/// - **Extended form**: `attr_name: { value: "value", brief: "description" }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum TelemetryAttribute {
    /// Extended form: `{ value: <value>, brief: "..." }`
    Extended {
        /// The attribute value.
        value: AttributeValue,
        /// An optional short description of this attribute.
        brief: Option<String>,
    },
    /// Bare form: just a scalar or array value.
    Bare(AttributeValue),
}

impl TelemetryAttribute {
    /// Create a new telemetry attribute with just a value (no brief).
    #[must_use]
    pub fn new(value: AttributeValue) -> Self {
        Self::Bare(value)
    }

    /// Create a new telemetry attribute with a value and brief description.
    pub fn with_brief(value: AttributeValue, brief: impl Into<String>) -> Self {
        Self::Extended {
            value,
            brief: Some(brief.into()),
        }
    }

    /// Returns a reference to the attribute value.
    #[must_use]
    pub fn value(&self) -> &AttributeValue {
        match self {
            Self::Extended { value, .. } => value,
            Self::Bare(value) => value,
        }
    }

    /// Returns the optional brief description.
    #[must_use]
    pub fn brief(&self) -> Option<&str> {
        match self {
            Self::Extended { brief, .. } => brief.as_deref(),
            Self::Bare(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_deserialize() {
        let yaml_str = r#"
            reporting_channel_size: 150
            reporting_interval: "3s"
            resource:
                service.version: "1.0.0"
            metrics:
                readers:
                    - periodic:
                        exporter:
                            console:
            "#;
        let config: TelemetryConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(config.reporting_channel_size, 150);
        assert_eq!(config.reporting_interval.as_secs(), 3);

        if let AttributeValue::String(version) = config.resource.get("service.version").unwrap() {
            assert_eq!(version, "1.0.0");
        } else {
            panic!("Expected service.version to be a String attribute value");
        }
        assert_eq!(config.metrics.readers.len(), 1);
    }

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert_eq!(config.reporting_channel_size, 100);
        assert_eq!(config.reporting_interval, Duration::from_secs(1));
        assert!(config.resource.is_empty());
        assert_eq!(config.metrics.readers.len(), 0);
    }

    #[test]
    fn test_attribute_value_deserialize_yaml() {
        let yaml_str = r#"
            string_attr: "example"
            bool_attr: true
            i64_attr: 42
            f64_attr: 1.23
            string_array: ["one", "two", "three"]
            bool_array: [true, false, true]
            i64_array: [1, 2, 3, 4]
            f64_array: [1.1, 2.2, 3.3]
            "#;

        let attrs: HashMap<String, AttributeValue> = serde_yaml::from_str(yaml_str).unwrap();

        match attrs.get("string_attr").unwrap() {
            AttributeValue::String(s) => assert_eq!(s, "example"),
            _ => panic!("Expected string_attr to be a String attribute value"),
        }

        match attrs.get("bool_attr").unwrap() {
            AttributeValue::Bool(b) => assert!(*b),
            _ => panic!("Expected bool_attr to be a Bool attribute value"),
        }

        match attrs.get("i64_attr").unwrap() {
            AttributeValue::I64(i) => assert_eq!(*i, 42),
            _ => panic!("Expected i64_attr to be an I64 attribute value"),
        }

        match attrs.get("f64_attr").unwrap() {
            AttributeValue::F64(f) => assert_eq!(*f, 1.23),
            _ => panic!("Expected f64_attr to be an F64 attribute value"),
        }

        match attrs.get("string_array").unwrap() {
            AttributeValue::Array(AttributeValueArray::String(arr)) => {
                assert_eq!(
                    arr,
                    &vec!["one".to_string(), "two".to_string(), "three".to_string()]
                )
            }
            _ => panic!("Expected string_array to be an Array of Strings"),
        }
    }

    #[test]
    fn test_attribute_value_deserialize_json() {
        let json_str = r#"
            {
                "string_attr": "example",
                "bool_attr": true,
                "i64_attr": 42,
                "f64_attr": 1.23,
                "string_array": ["one", "two", "three"],
                "bool_array": [true, false, true],
                "i64_array": [1, 2, 3, 4],
                "f64_array": [1.1, 2.2, 3.3]
            }
            "#;

        let attrs: HashMap<String, AttributeValue> = serde_json::from_str(json_str).unwrap();

        match attrs.get("string_attr").unwrap() {
            AttributeValue::String(s) => assert_eq!(s, "example"),
            _ => panic!("Expected string_attr to be a String attribute value"),
        }

        match attrs.get("bool_attr").unwrap() {
            AttributeValue::Bool(b) => assert!(*b),
            _ => panic!("Expected bool_attr to be a Bool attribute value"),
        }

        match attrs.get("i64_attr").unwrap() {
            AttributeValue::I64(i) => assert_eq!(*i, 42),
            _ => panic!("Expected i64_attr to be an I64 attribute value"),
        }

        match attrs.get("f64_attr").unwrap() {
            AttributeValue::F64(f) => assert_eq!(*f, 1.23),
            _ => panic!("Expected f64_attr to be an F64 attribute value"),
        }

        match attrs.get("string_array").unwrap() {
            AttributeValue::Array(AttributeValueArray::String(arr)) => {
                assert_eq!(
                    arr,
                    &vec!["one".to_string(), "two".to_string(), "three".to_string()]
                )
            }
            _ => panic!("Expected string_array to be an Array of Strings"),
        }
    }

    #[test]
    fn test_attribute_value_array_empty() {
        let yaml_str = r#"
            empty_array: []
            "#;
        let attrs: HashMap<String, AttributeValue> = serde_yaml::from_str(yaml_str).unwrap();

        match attrs.get("empty_array").unwrap() {
            AttributeValue::Array(arr) => match arr {
                AttributeValueArray::String(vec) => assert!(vec.is_empty()),
                AttributeValueArray::Bool(vec) => assert!(vec.is_empty()),
                AttributeValueArray::I64(vec) => assert!(vec.is_empty()),
                AttributeValueArray::F64(vec) => assert!(vec.is_empty()),
            },
            _ => panic!("Expected empty_array to be an Array attribute value"),
        }
    }

    #[test]
    fn test_attribute_value_array_mixed_types_error() {
        let yaml_str = r#"
            mixed_array: [1, "two", 3]
            "#;
        let result: Result<HashMap<String, AttributeValue>, _> = serde_yaml::from_str(yaml_str);
        if let Err(err) = &result {
            assert!(
                err.to_string()
                    .contains("expected all elements to be integers")
            );
        } else {
            panic!("Expected deserialization to fail for mixed type array");
        }
    }

    #[test]
    fn test_attribute_value_u64_out_of_range_error() {
        let yaml_str = r#"
            out_of_range_value: 9223372036854775808
            "#;
        let result: Result<HashMap<String, AttributeValue>, _> = serde_yaml::from_str(yaml_str);
        if let Err(err) = &result {
            assert!(err.to_string().contains("out of range"));
        } else {
            panic!("Expected deserialization to fail for out of range u64 value");
        }
    }

    #[test]
    fn test_telemetry_attribute_bare_value_yaml() {
        let yaml_str = r#"
            region: "us-west"
            count: 42
            enabled: true
            ratio: 1.5
        "#;
        let attrs: HashMap<String, TelemetryAttribute> = serde_yaml::from_str(yaml_str).unwrap();

        let region = attrs.get("region").unwrap();
        assert_eq!(
            *region.value(),
            AttributeValue::String("us-west".to_string())
        );
        assert_eq!(region.brief(), None);

        let count = attrs.get("count").unwrap();
        assert_eq!(*count.value(), AttributeValue::I64(42));
        assert_eq!(count.brief(), None);

        let enabled = attrs.get("enabled").unwrap();
        assert_eq!(*enabled.value(), AttributeValue::Bool(true));
        assert_eq!(enabled.brief(), None);

        let ratio = attrs.get("ratio").unwrap();
        assert_eq!(*ratio.value(), AttributeValue::F64(1.5));
        assert_eq!(ratio.brief(), None);
    }

    #[test]
    fn test_telemetry_attribute_extended_form_yaml() {
        let yaml_str = r#"
            region:
              value: "us-west"
              brief: "Deployment region"
            count:
              value: 42
        "#;
        let attrs: HashMap<String, TelemetryAttribute> = serde_yaml::from_str(yaml_str).unwrap();

        let region = attrs.get("region").unwrap();
        assert_eq!(
            *region.value(),
            AttributeValue::String("us-west".to_string())
        );
        assert_eq!(region.brief(), Some("Deployment region"));

        let count = attrs.get("count").unwrap();
        assert_eq!(*count.value(), AttributeValue::I64(42));
        assert_eq!(count.brief(), None);
    }

    #[test]
    fn test_telemetry_attribute_mixed_forms_yaml() {
        let yaml_str = r#"
            simple: "hello"
            detailed:
              value: "world"
              brief: "A detailed attribute"
        "#;
        let attrs: HashMap<String, TelemetryAttribute> = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(attrs.get("simple").unwrap().brief(), None);
        assert_eq!(
            attrs.get("detailed").unwrap().brief(),
            Some("A detailed attribute")
        );
    }

    #[test]
    fn test_telemetry_attribute_extended_form_json() {
        let json_str = r#"{
            "region": {"value": "us-west", "brief": "Deployment region"},
            "count": 42
        }"#;
        let attrs: HashMap<String, TelemetryAttribute> = serde_json::from_str(json_str).unwrap();

        let region = attrs.get("region").unwrap();
        assert_eq!(
            *region.value(),
            AttributeValue::String("us-west".to_string())
        );
        assert_eq!(region.brief(), Some("Deployment region"));

        let count = attrs.get("count").unwrap();
        assert_eq!(*count.value(), AttributeValue::I64(42));
        assert_eq!(count.brief(), None);
    }

    #[test]
    fn test_telemetry_attribute_serialize_bare() {
        let attr = TelemetryAttribute::new(AttributeValue::String("test".to_string()));
        let json = serde_json::to_string(&attr).unwrap();
        // Without brief, serializes as just the value (with enum tagging)
        assert_eq!(json, r#"{"String":"test"}"#);
    }

    #[test]
    fn test_telemetry_attribute_serialize_with_brief() {
        let attr = TelemetryAttribute::with_brief(
            AttributeValue::String("test".to_string()),
            "A test attribute",
        );
        let json = serde_json::to_string(&attr).unwrap();
        assert!(json.contains(r#""value":{"String":"test"}"#));
        assert!(json.contains(r#""brief":"A test attribute""#));
    }
}
