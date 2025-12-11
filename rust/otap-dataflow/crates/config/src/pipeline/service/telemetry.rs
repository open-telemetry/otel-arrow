// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry level configurations.

pub mod metrics;

use std::{collections::HashMap, time::Duration};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::service::telemetry::metrics::MetricsConfig;

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
    /// TODO: Support different types of attribute values.
    #[serde(default)]
    pub resource: HashMap<String, String>,
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

fn default_reporting_channel_size() -> usize {
    100
}

fn default_reporting_interval() -> Duration {
    Duration::from_secs(1)
}

/// Internal logs configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct LogsConfig {
    /// The log level for internal engine logs.
    #[serde(default)]
    pub level: LogLevel,
}

/// Log level for internal engine logs.
///
/// TODO: Change default to `Info` once per-thread subscriber is implemented
/// to avoid contention from the global tracing subscriber.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Logging is completely disabled.
    #[default]
    Off,
    /// Debug level logging.
    Debug,
    /// Info level logging.
    Info,
    /// Warn level logging.
    Warn,
    /// Error level logging.
    Error,
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
        assert_eq!(config.resource.get("service.version").unwrap(), "1.0.0");
        assert_eq!(config.metrics.readers.len(), 1);
    }
}
