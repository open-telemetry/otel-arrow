// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Otlp level configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::telemetry::metrics::readers::Temporality;

/// OpenTelemetry OTLP Exporter configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct OtlpExporterConfig {
    /// The Otlp communication protocol to use when exporting data.
    #[serde(default)]
    pub protocol: OtlpProtocol,
    /// The endpoint to which the Otlp exporter will send data.
    pub endpoint: String,

    /// The temporality of the metrics to be exported.
    #[serde(default)]
    pub temporality: Temporality,

    /// TLS configuration for secure communication.
    pub tls: Option<TlsConfig>,
}
impl OtlpExporterConfig {
    /// Create a new OTLP exporter configuration from a JSON value.
    ///
    /// If the value is `null`, it is treated as an empty object so that
    /// any fields with `#[serde(default)]` still receive their defaults.
    pub fn from_config(config: &serde_json::Value) -> Result<Self, crate::error::Error> {
        let effective = if config.is_null() {
            &serde_json::Value::Object(serde_json::Map::new())
        } else {
            config
        };
        serde_json::from_value(effective.clone()).map_err(|e| {
            crate::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })
    }
}

/// The Otlp communication protocol to use when exporting data.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
pub enum OtlpProtocol {
    /// Use gRPC for communication.
    #[default]
    #[serde(rename = "grpc/protobuf")]
    Grpc,

    #[serde(rename = "http/protobuf")]
    /// Use HTTP with binary encoding for communication.
    HttpBinary,

    #[serde(rename = "http/json")]
    /// Use HTTP with JSON encoding for communication.
    HttpJson,
}

/// TLS configuration for secure communication.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TlsConfig {
    /// Path to the CA certificate file.
    pub ca_file: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otlp_exporter_config_deserialize() {
        let yaml_str = r#"
            protocol: "http/json"
            endpoint: "http://localhost:4318/v1/metrics"
            temporality: "delta"
            "#;

        let config: OtlpExporterConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(config.protocol, OtlpProtocol::HttpJson);
        assert_eq!(config.endpoint, "http://localhost:4318/v1/metrics");
        assert_eq!(config.temporality, Temporality::Delta);
    }

    #[test]
    fn test_otlp_exporter_config_from_config_invalid() {
        let invalid = serde_json::json!({"protocol": "invalid_protocol"});
        let result = OtlpExporterConfig::from_config(&invalid);
        assert!(
            result.is_err(),
            "Expected error for invalid protocol, got: {result:?}"
        );
    }

    #[test]
    fn test_otlp_exporter_config_from_config_null() {
        // Null is treated as an empty object; endpoint is required so this must fail.
        let result = OtlpExporterConfig::from_config(&serde_json::Value::Null);
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("endpoint"),
            "Expected error about missing endpoint, got: {err_msg}"
        );
    }
}
