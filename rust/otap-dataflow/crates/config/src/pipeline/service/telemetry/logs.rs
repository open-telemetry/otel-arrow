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

    /// Internal log configuration options
    #[serde(default = "default_internal")]
    pub internal: LogsInternalConfig,

    /// The list of log processors to configure.
    #[serde(default)]
    pub processors: Vec<processors::LogProcessorConfig>,
}

fn default_internal() -> LogsInternalConfig {
    LogsInternalConfig {
        enabled: true,
    }
}

/// Log level for internal engine logs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Logging is completely disabled.
    Off,
    /// Debug level logging.
    Debug,
    /// Info level logging.
    #[default]
    Info,
    /// Warn level logging.
    Warn,
    /// Error level logging.
    Error,
}

/// Log internal configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct LogsInternalConfig {
    /// Is internal logging in use?
    pub enabled: bool,
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
        assert_eq!(config.level, LogLevel::Info);
        assert!(config.processors.is_empty());
        Ok(())
    }
}
