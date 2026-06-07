// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pull reader level configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The type of metrics pull exporter to use.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MetricsPullExporterType {
    /// Prometheus exporter that exposes metrics for scraping.
    Prometheus,
}

/// OpenTelemetry Metrics Pull Exporter configuration.
///
/// The `type` field selects the exporter, and `config` holds the
/// exporter-specific configuration as a free-form JSON value.
/// For `prometheus`, `config` should contain a valid `PrometheusExporterConfig`.
/// If `config` is omitted, Prometheus defaults (host=0.0.0.0, port=9090,
/// path=/metrics) are used.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricsPullExporterConfig {
    /// The type of exporter to use.
    #[serde(rename = "type")]
    pub exporter_type: MetricsPullExporterType,

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

/// Prometheus Exporter configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PrometheusExporterConfig {
    /// The host address where the Prometheus exporter will expose metrics.
    #[serde(default = "default_host")]
    pub host: String,

    /// The port on which the Prometheus exporter will listen for scrape requests.
    #[serde(default = "default_port")]
    pub port: u16,

    /// The HTTP path where metrics will be exposed.
    #[serde(default = "default_metrics_path")]
    pub path: String,
}
impl MetricsPullExporterConfig {
    /// Validates that the `config` field is compatible with the selected exporter `type`.
    ///
    /// For `Prometheus`, the config must deserialize into a valid [`PrometheusExporterConfig`].
    /// If the config is omitted (null), Prometheus defaults are used.
    pub fn validate(&self) -> Result<(), crate::error::Error> {
        match self.exporter_type {
            MetricsPullExporterType::Prometheus => {
                let _ = PrometheusExporterConfig::from_config(&self.config)?;
                Ok(())
            }
        }
    }
}

impl PrometheusExporterConfig {
    /// Create a new prometheus exporter configuration from a JSON value.
    ///
    /// If the value is `null`, it is treated as an empty object so that
    /// all fields receive their defaults (host=0.0.0.0, port=9090,
    /// path=/metrics).
    pub fn from_config(config: &Value) -> Result<Self, crate::error::Error> {
        let effective = if config.is_null() {
            &Value::Object(serde_json::Map::new())
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

fn default_host() -> String {
    "0.0.0.0".to_string()
}

const fn default_port() -> u16 {
    9090
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_pull_exporter_config_deserialize() {
        let yaml_str = r#"
        type: prometheus
        config:
            host: "127.0.0.1"
            port: 9090
            path: "/"
        "#;
        let config: MetricsPullExporterConfig = serde_yaml::from_str(yaml_str).unwrap();

        let prometheus_config = PrometheusExporterConfig::from_config(&config.config)
            .expect("Valid prometheus exporter config");
        assert_eq!(prometheus_config.host, "127.0.0.1");
        assert_eq!(prometheus_config.port, 9090);
        assert_eq!(prometheus_config.path, "/");
    }

    #[test]
    fn test_metrics_pull_exporter_invalid_config_deserialize() {
        let yaml_str = r#"
        type: unknown_exporter
        config:
            some_field: "value"
        "#;
        let result: Result<MetricsPullExporterConfig, _> = serde_yaml::from_str(yaml_str);
        match result {
            Ok(_) => panic!("Deserialization should have failed for unknown exporter"),
            Err(err) => {
                let err_msg = err.to_string();
                assert!(err_msg.contains("unknown variant"));
                assert!(err_msg.contains("prometheus"));
            }
        }
    }

    #[test]
    fn test_prometheus_exporter_config_deserialize() {
        let yaml_str = r#"
        host: "127.0.0.1"
        port: 9090
        path: "/custom_metrics"
        "#;
        let config: PrometheusExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 9090);
        assert_eq!(config.path, "/custom_metrics");
    }

    #[test]
    fn test_prometheus_exporter_config_default_path_deserialize() {
        let yaml_str = r#"
        host: "127.0.0.1"
        port: 9090
        "#;
        let config: PrometheusExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 9090);
        assert_eq!(config.path, "/metrics");
    }

    #[test]
    fn test_prometheus_exporter_unknown_field_config_deserialize() {
        let yaml_str = r#"
        host: "0.0.0.0"
        port: 8080
        extra_field: "unexpected"
        "#;
        let result: Result<PrometheusExporterConfig, _> = serde_yaml::from_str(yaml_str);
        match result {
            Ok(_) => panic!("Deserialization should have failed for unknown field"),
            Err(err) => {
                let err_msg = err.to_string();
                assert!(err_msg.contains("unknown field `extra_field`"));
            }
        }
    }

    #[test]
    fn test_prometheus_exporter_config_defaults() {
        let yaml_str = r#""#;
        let config: PrometheusExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9090);
        assert_eq!(config.path, "/metrics");
    }

    #[test]
    fn test_metrics_pull_exporter_config_prometheus_missing_config() {
        let yaml_str = r#"
        type: prometheus
        "#;
        let config: MetricsPullExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.exporter_type, MetricsPullExporterType::Prometheus);
        // Should succeed — PrometheusExporterConfig has defaults for all fields,
        // and from_config treats null as an empty object.
        config
            .validate()
            .expect("Prometheus with defaults should be valid");
    }

    #[test]
    fn test_metrics_pull_exporter_config_extra_fields() {
        let yaml_str = r#"
        type: prometheus
        extra_field: bar
        "#;
        let result: Result<MetricsPullExporterConfig, _> = serde_yaml::from_str(yaml_str);
        match result {
            Ok(_) => panic!("Deserialization should have failed for extra fields present"),
            Err(err) => {
                let err_msg = err.to_string();
                assert!(err_msg.contains("unknown field `extra_field`"));
            }
        }
    }

    #[test]
    fn test_prometheus_exporter_config_from_config_invalid() {
        let invalid = serde_json::json!({"port": "not_a_number"});
        let result = PrometheusExporterConfig::from_config(&invalid);
        assert!(
            result.is_err(),
            "Expected error for invalid port type, got: {result:?}"
        );
    }

    #[test]
    fn test_metrics_pull_exporter_config_roundtrip() {
        let yaml_str = r#"
        type: prometheus
        config:
            host: "127.0.0.1"
            port: 9090
            path: "/metrics"
        "#;
        let config: MetricsPullExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        let json = serde_json::to_string(&config).unwrap();
        let roundtripped: MetricsPullExporterConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, roundtripped);
    }

    #[test]
    fn test_metrics_pull_exporter_config_schema_has_preserve_unknown_fields() {
        let schema = schemars::schema_for!(MetricsPullExporterConfig);
        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(
            json.contains("x-kubernetes-preserve-unknown-fields"),
            "Schema must contain x-kubernetes-preserve-unknown-fields extension"
        );
    }
}
