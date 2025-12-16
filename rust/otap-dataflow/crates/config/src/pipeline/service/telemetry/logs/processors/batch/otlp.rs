// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP batch exporter level configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::service::telemetry::metrics::readers::periodic::otlp::OtlpProtocol;

/// OpenTelemetry OTLP Exporter configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct OtlpExporterConfig {
    /// The Otlp communication protocol to use when exporting data.
    #[serde(default)]
    pub protocol: OtlpProtocol,
    /// The endpoint to which the Otlp exporter will send data.
    pub endpoint: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otlp_exporter_config_grpc_deserialize() {
        let yaml_str = r#"
            protocol: "grpc/protobuf"
            endpoint: "http://localhost:4317"
            "#;
        let config: OtlpExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.protocol, OtlpProtocol::Grpc);
        assert_eq!(config.endpoint, "http://localhost:4317");
    }

    #[test]
    fn test_otlp_exporter_config_http_deserialize() {
        let yaml_str = r#"
            protocol: "http/protobuf"
            endpoint: "http://localhost:4318/v1/logs"
            "#;
        let config: OtlpExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.protocol, OtlpProtocol::HttpBinary);
        assert_eq!(config.endpoint, "http://localhost:4318/v1/logs");
    }

    #[test]
    fn test_otlp_exporter_config_json_deserialize() {
        let yaml_str = r#"
            protocol: "http/json"
            endpoint: "http://localhost:4318/v1/logs"
            "#;
        let config: OtlpExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.protocol, OtlpProtocol::HttpJson);
        assert_eq!(config.endpoint, "http://localhost:4318/v1/logs");
    }
}
