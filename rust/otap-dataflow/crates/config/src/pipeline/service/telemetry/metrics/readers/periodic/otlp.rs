// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Otlp level configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::service::telemetry::metrics::readers::Temporality;

/// Opentelemetry OTLP Exporter configuration.
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
}
