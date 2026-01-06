// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Readers level configurations.

pub mod periodic;
pub mod pull;

use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::service::telemetry::metrics::readers::{
    periodic::MetricsPeriodicExporterConfig, pull::MetricsPullExporterConfig,
};

/// OpenTelemetry Metrics Reader configuration.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MetricsReaderConfig {
    /// Periodic reader that exports metrics at regular intervals.
    Periodic(MetricsReaderPeriodicConfig),
    /// Pull reader that allows on-demand metric collection.
    Pull(MetricsReaderPullConfig),
}

/// OpenTelemetry Metrics Periodic Reader configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricsReaderPeriodicConfig {
    /// The metrics exporter to use.
    pub exporter: MetricsPeriodicExporterConfig,
    /// The interval at which metrics are periodically exported.
    #[serde(with = "humantime_serde", default = "default_periodic_interval")]
    #[schemars(with = "String")]
    pub interval: Duration,
}

fn default_periodic_interval() -> Duration {
    Duration::from_secs(6)
}

impl<'de> Deserialize<'de> for MetricsReaderConfig {
    /// Custom deserialization to handle different reader types.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ReaderOptions {
            #[serde(rename = "periodic")]
            periodic: Option<MetricsReaderPeriodicConfig>,
            #[serde(rename = "pull")]
            pull: Option<MetricsReaderPullConfig>,
        }

        let reader_options_result = ReaderOptions::deserialize(deserializer);
        match reader_options_result {
            Ok(options) => {
                if let Some(config) = options.periodic {
                    Ok(MetricsReaderConfig::Periodic(config))
                } else if let Some(config) = options.pull {
                    Ok(MetricsReaderConfig::Pull(config))
                } else {
                    Err(serde::de::Error::custom(
                        "Expected either 'periodic' or 'pull' reader",
                    ))
                }
            }
            Err(err) => Err(serde::de::Error::custom(format!(
                "Expected a map with either 'periodic' or 'pull' reader: {}",
                err
            ))),
        }
    }
}

/// OpenTelemetry Metrics Pull Reader configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricsReaderPullConfig {
    /// The metrics exporter to use.
    pub exporter: MetricsPullExporterConfig,
}

/// The temporality of the metrics to be exported.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Temporality {
    /// Cumulative temporality means that the metric values are the sum of all values since the start of the process.
    #[default]
    Cumulative,
    /// Delta temporality means that the metric values are the difference between the current and previous values.
    Delta,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_reader_config_deserialize_periodic() {
        let yaml_str = r#"
            periodic:
                exporter:
                    console:
                interval: "10s"
            "#;
        let config: MetricsReaderConfig = serde_yaml::from_str(yaml_str).unwrap();

        if let MetricsReaderConfig::Periodic(periodic_config) = config {
            if MetricsPeriodicExporterConfig::Console != periodic_config.exporter {
                panic!("Expected console exporter");
            }
            assert_eq!(periodic_config.interval.as_secs(), 10);
        } else {
            panic!("Expected periodic reader");
        }
    }

    #[test]
    fn test_metrics_reader_config_deserialize_pull() {
        let yaml_str = r#"
            pull:
                exporter:
                    prometheus:
                        host: "0.0.0.0"
                        port: 9090
            "#;
        let config: MetricsReaderConfig = serde_yaml::from_str(yaml_str).unwrap();

        if let MetricsReaderConfig::Pull(pull_config) = config {
            let MetricsPullExporterConfig::Prometheus(prometheus_config) = pull_config.exporter;
            assert_eq!(prometheus_config.host, "0.0.0.0");
            assert_eq!(prometheus_config.port, 9090);
        } else {
            panic!("Expected pull reader");
        }
    }

    #[test]
    fn test_temporality_deserialize() {
        let yaml_str_cumulative = r#"
            cumulative
            "#;
        let temporality_cumulative: Temporality =
            serde_yaml::from_str(yaml_str_cumulative).unwrap();
        assert_eq!(temporality_cumulative, Temporality::Cumulative);

        let yaml_str_delta = r#"
            delta
            "#;
        let temporality_delta: Temporality = serde_yaml::from_str(yaml_str_delta).unwrap();
        assert_eq!(temporality_delta, Temporality::Delta);
    }
}
