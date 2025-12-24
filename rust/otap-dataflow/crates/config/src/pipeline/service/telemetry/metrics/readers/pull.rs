// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pull reader level configurations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// OpenTelemetry Metrics Pull Exporter configuration.
#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MetricsPullExporterConfig {
    /// Prometheus exporter that exposes metrics for scraping.
    Prometheus(PrometheusExporterConfig),
}

impl<'de> Deserialize<'de> for MetricsPullExporterConfig {
    /// Custom deserialization to handle different exporter types.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;
        struct MetricsPullExporterConfigVisitor;

        impl<'de> Visitor<'de> for MetricsPullExporterConfigVisitor {
            type Value = MetricsPullExporterConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map with 'prometheus' key")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                if let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "prometheus" => {
                            let prometheus_config: PrometheusExporterConfig = map.next_value()?;
                            Ok(MetricsPullExporterConfig::Prometheus(prometheus_config))
                        }
                        _ => Err(serde::de::Error::unknown_field(&key, &["prometheus"])),
                    }
                } else {
                    Err(serde::de::Error::custom("Expected 'prometheus' exporter"))
                }
            }
        }

        deserializer.deserialize_map(MetricsPullExporterConfigVisitor)
    }
}

/// Prometheus Exporter configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PrometheusExporterConfig {
    /// The host address where the Prometheus exporter will expose metrics.
    pub host: String,
    /// The port on which the Prometheus exporter will listen for scrape requests.
    pub port: u16,
    /// The HTTP path where metrics will be exposed.
    #[serde(default = "default_metrics_path")]
    pub path: String,
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
        prometheus:
            host: "127.0.0.1"
            port: 9090
            path: "/"
        "#;
        let config: MetricsPullExporterConfig = serde_yaml::from_str(yaml_str).unwrap();

        let MetricsPullExporterConfig::Prometheus(prometheus_config) = config;
        assert_eq!(prometheus_config.host, "127.0.0.1");
        assert_eq!(prometheus_config.port, 9090);
        assert_eq!(prometheus_config.path, "/");
    }

    #[test]
    fn test_metrics_pull_exporter_invalid_config_deserialize() {
        let yaml_str = r#"
        unknown_exporter:
            some_field: "value"
        "#;
        let result: Result<MetricsPullExporterConfig, _> = serde_yaml::from_str(yaml_str);
        match result {
            Ok(_) => panic!("Deserialization should have failed for unknown exporter"),
            Err(err) => {
                let err_msg = err.to_string();
                assert!(err_msg.contains("unknown field"));
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
    fn test_prometheus_exporter_missing_field_config_deserialize() {
        let yaml_str = r#"
        host: "0.0.0.0"
        "#;
        let result: Result<PrometheusExporterConfig, _> = serde_yaml::from_str(yaml_str);
        match result {
            Ok(_) => panic!("Deserialization should have failed for missing port"),
            Err(err) => {
                let err_msg = err.to_string();
                assert!(err_msg.contains("missing field `port`"));
            }
        }
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
}
