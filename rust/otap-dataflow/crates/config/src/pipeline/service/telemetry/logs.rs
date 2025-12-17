// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logs level configurations.

pub mod processors;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Internal logs configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct LogsConfig {
    /// The log level for internal engine logs.
    #[serde(default)]
    pub level: LogLevel,

    /// The list of log processors to configure.
    #[serde(default)]
    pub processors: Vec<processors::LogProcessorConfig>,
}

/// Log level for internal engine logs.
///
/// TODO: Change default to `Info` once per-thread subscriber is implemented
/// to avoid contention from the global tracing subscriber.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
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
    fn test_logs_config_deserialize() {
        let yaml_str = r#"
            level: "info"
            processors:
              - batch:
                  exporter:
                    console:
            "#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.processors.len(), 1);
    }

    #[test]
    fn test_log_level_deserialize() {
        let yaml_str = r#"
            level: "info"
            "#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.level, LogLevel::Info);
    }

    #[test]
    fn test_logs_config_default_deserialize() -> Result<(), serde_yaml::Error> {
        let yaml_str = r#""#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str)?;
        assert_eq!(config.level, LogLevel::Off);
        assert!(config.processors.is_empty());
        Ok(())
    }
}
