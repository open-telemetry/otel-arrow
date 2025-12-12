// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Periodic reader level configurations.

pub mod otlp;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::IgnoredAny};

use crate::pipeline::service::telemetry::metrics::readers::periodic::otlp::OtlpExporterConfig;

/// Opentelemetry Metrics Perioidc Exporter configuration.
#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
pub enum MetricsPeriodicExporterConfig {
    /// Console exporter that writes metrics to the console.
    Console,
    /// OTLP exporter that sends metrics using the OpenTelemetry Protocol.
    Otlp(OtlpExporterConfig),
}

impl<'de> Deserialize<'de> for MetricsPeriodicExporterConfig {
    /// Custom deserialization to handle different exporter types.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;
        struct MetricsPeriodicExporterConfigVisitor;

        impl<'de> Visitor<'de> for MetricsPeriodicExporterConfigVisitor {
            type Value = MetricsPeriodicExporterConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map with either 'console' or 'otlp' key")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                if let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "console" => {
                            let _: IgnoredAny = map.next_value()?; // consume the value. Console does not have any attributes.
                            Ok(MetricsPeriodicExporterConfig::Console)
                        }
                        "otlp" => {
                            let otlp_config: OtlpExporterConfig = map.next_value()?;
                            Ok(MetricsPeriodicExporterConfig::Otlp(otlp_config))
                        }
                        _ => Err(serde::de::Error::unknown_field(&key, &["console", "otlp"])),
                    }
                } else {
                    Err(serde::de::Error::custom(
                        "Expected either 'console' or 'otlp' exporter",
                    ))
                }
            }
        }

        deserializer.deserialize_map(MetricsPeriodicExporterConfigVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline::service::telemetry::metrics::readers::{
        Temporality, periodic::otlp::OtlpProtocol,
    };

    use super::*;

    #[test]
    fn test_metrics_periodic_exporter_config_deserialize_console() {
        let yaml_str = r#"
            console:
            "#;

        let config: MetricsPeriodicExporterConfig = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(config, MetricsPeriodicExporterConfig::Console);
    }

    #[test]
    fn test_metrics_periodic_exporter_config_deserialize_otlp() {
        let yaml_str = r#"
            otlp:
                endpoint: "http://localhost:4317"
                protocol: "grpc/protobuf"
        "#;
        let config: MetricsPeriodicExporterConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(
            config,
            MetricsPeriodicExporterConfig::Otlp(OtlpExporterConfig {
                endpoint: "http://localhost:4317".to_string(),
                protocol: OtlpProtocol::Grpc,
                temporality: Temporality::Cumulative,
            })
        );
    }
}
