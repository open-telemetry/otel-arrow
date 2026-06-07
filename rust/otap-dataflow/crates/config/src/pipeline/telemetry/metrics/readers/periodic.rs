// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Periodic reader level configurations.

pub mod otlp;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The type of metrics periodic exporter to use.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MetricsPeriodicExporterType {
    /// Console exporter that writes metrics to stdout.
    Console,
    /// OTLP exporter that sends metrics using the OpenTelemetry Protocol.
    Otlp,
}

/// OpenTelemetry Metrics Periodic Exporter configuration.
///
/// The `type` field selects the exporter, and `config` holds the
/// exporter-specific configuration as a free-form JSON value.
/// For `console`, `config` can be omitted or left empty.
/// For `otlp`, `config` must contain a valid `OtlpExporterConfig`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricsPeriodicExporterConfig {
    /// The type of exporter to use.
    #[serde(rename = "type")]
    pub exporter_type: MetricsPeriodicExporterType,

    /// Exporter-specific configuration as a free-form JSON value.
    /// The shape of this field depends on the selected `type`.
    #[serde(default)]
    // serde_json::Value produces a schema that the Kubernetes API server rejects,
    // since it cannot be represented as a structural schema. The
    // x-kubernetes-preserve-unknown-fields extension tells the API server to
    // accept arbitrary JSON here instead of requiring a fully-specified schema.
    #[schemars(extend("x-kubernetes-preserve-unknown-fields" = true))]
    pub config: Value,
}

impl MetricsPeriodicExporterConfig {
    /// Validates that the `config` field is compatible with the selected exporter `type`.
    ///
    /// For `Console`, no config is required.
    /// For `Otlp`, the config must deserialize into a valid [`otlp::OtlpExporterConfig`].
    pub fn validate(&self) -> Result<(), crate::error::Error> {
        match self.exporter_type {
            MetricsPeriodicExporterType::Console => Ok(()),
            MetricsPeriodicExporterType::Otlp => {
                let _ = otlp::OtlpExporterConfig::from_config(&self.config)?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline::telemetry::metrics::readers::{
        Temporality,
        periodic::otlp::{OtlpExporterConfig, OtlpProtocol},
    };

    use super::*;

    #[test]
    fn test_metrics_periodic_exporter_config_deserialize_console() {
        let yaml_str = r#"
            type: console
            "#;

        let config: MetricsPeriodicExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.exporter_type, MetricsPeriodicExporterType::Console);
    }

    #[test]
    fn test_metrics_periodic_exporter_config_deserialize_otlp() {
        let yaml_str = r#"
            type: otlp
            config:
                endpoint: "http://localhost:4317"
                protocol: "grpc/protobuf"
        "#;
        let config: MetricsPeriodicExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.exporter_type, MetricsPeriodicExporterType::Otlp);
        assert_eq!(
            OtlpExporterConfig::from_config(&config.config).expect("valid otlp configuration"),
            OtlpExporterConfig {
                endpoint: "http://localhost:4317".to_string(),
                protocol: OtlpProtocol::Grpc,
                temporality: Temporality::Cumulative,
                tls: None,
            }
        );
    }

    #[test]
    fn test_metrics_periodic_exporter_config_otlp_missing_config() {
        let yaml_str = r#"
            type: otlp
        "#;
        let config: MetricsPeriodicExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.exporter_type, MetricsPeriodicExporterType::Otlp);
        // Deserialization succeeds, but validation should fail because
        // OtlpExporterConfig requires at minimum an `endpoint` field.
        let err = config.validate().unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("endpoint"),
            "Expected error about missing endpoint, got: {err_msg}"
        );
    }

    #[test]
    fn test_metrics_periodic_exporter_config_console_validates() {
        let config = MetricsPeriodicExporterConfig {
            exporter_type: MetricsPeriodicExporterType::Console,
            config: Value::Null,
        };
        config
            .validate()
            .expect("Console exporter should validate without config");
    }

    #[test]
    fn test_metrics_periodic_exporter_config_roundtrip() {
        let yaml_str = r#"
            type: otlp
            config:
                endpoint: "http://localhost:4317"
                protocol: "grpc/protobuf"
        "#;
        let config: MetricsPeriodicExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        let json = serde_json::to_string(&config).unwrap();
        let roundtripped: MetricsPeriodicExporterConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, roundtripped);
    }

    #[test]
    fn test_metrics_pull_exporter_config_extra_fields() {
        let yaml_str = r#"
        type: otlp
        extra_field: bar
        "#;
        let result: Result<MetricsPeriodicExporterConfig, _> = serde_yaml::from_str(yaml_str);
        match result {
            Ok(_) => panic!("Deserialization should have failed for extra fields present"),
            Err(err) => {
                let err_msg = err.to_string();
                assert!(err_msg.contains("unknown field `extra_field`"));
            }
        }
    }

    #[test]
    fn test_metrics_periodic_exporter_config_schema_has_preserve_unknown_fields() {
        let schema = schemars::schema_for!(MetricsPeriodicExporterConfig);
        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(
            json.contains("x-kubernetes-preserve-unknown-fields"),
            "Schema must contain x-kubernetes-preserve-unknown-fields extension"
        );
    }
}
