// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logs level configurations.

pub mod processors;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Internal logs configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct LogsConfig {
    /// The log level for internal engine logs.
    #[serde(default)]
    pub level: LogLevel,

    /// The list of log processors to configure.
    #[serde(default)]
    pub processors: Vec<processors::LogProcessorConfig>,
    
    /// Internal collection for component-level logs.
    /// 
    /// When enabled, component logs (otel_info!, otel_warn!, etc.) are routed through
    /// an internal telemetry receiver in the OTAP pipeline, allowing use of built-in
    /// batch processors, retry, and exporters (console, OTLP, etc.).
    /// 
    /// When disabled (default), component logs are routed to the OpenTelemetry SDK,
    /// using the same export path as 3rd party logs from tokio-tracing-rs.
    #[serde(default)]
    pub internal_collection: InternalCollectionConfig,
}

/// Configuration for internal collection of component logs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InternalCollectionConfig {
    /// Enable internal collection for component logs.
    /// When false, component logs use the OpenTelemetry SDK (declarative config is honored).
    #[serde(default)]
    pub enabled: bool,
    
    /// Per-thread buffer size in bytes for accumulating component logs.
    /// This is a pre-allocated, fixed-size buffer. When full, logs are flushed to the channel.
    #[serde(default = "default_buffer_size_bytes")]
    pub buffer_size_bytes: usize,
    
    /// Maximum size in bytes for a single log record.
    /// Records exceeding this size are dropped with a counter increment.
    /// This limit enables encoder optimizations (2-byte length placeholders for 14-bit sizes).
    #[serde(default = "default_max_record_bytes")]
    pub max_record_bytes: usize,
    
    /// Maximum number of records in the bounded channel.
    /// When full, new records fall back to raw console logger.
    #[serde(default = "default_max_record_count")]
    pub max_record_count: usize,
    
    /// Flush interval for periodic flushing by the internal telemetry receiver.
    #[serde(with = "humantime_serde", default = "default_flush_interval")]
    #[schemars(with = "String")]
    pub flush_interval: Duration,
}

impl Default for InternalCollectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            buffer_size_bytes: default_buffer_size_bytes(),
            max_record_bytes: default_max_record_bytes(),
            max_record_count: default_max_record_count(),
            flush_interval: default_flush_interval(),
        }
    }
}

fn default_buffer_size_bytes() -> usize {
    64 * 1024  // 64 KiB - pre-allocated per thread
}

fn default_max_record_bytes() -> usize {
    16 * 1024  // 16 KiB - max single record (enables 2-byte length placeholders)
}

fn default_max_record_count() -> usize {
    1000  // messages (bounded channel)
}

fn default_flush_interval() -> Duration {
    Duration::from_secs(1)
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
        assert!(!config.internal_collection.enabled);
    }
    
    #[test]
    fn test_internal_collection_config_deserialize() {
        let yaml_str = r#"
            level: "info"
            internal_collection:
              enabled: true
              buffer_size_bytes: 131072
              max_record_bytes: 32768
              max_record_count: 2000
              flush_interval: "2s"
            "#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert!(config.internal_collection.enabled);
        assert_eq!(config.internal_collection.buffer_size_bytes, 131072);
        assert_eq!(config.internal_collection.max_record_bytes, 32768);
        assert_eq!(config.internal_collection.max_record_count, 2000);
        assert_eq!(config.internal_collection.flush_interval, Duration::from_secs(2));
    }
    
    #[test]
    fn test_internal_collection_config_defaults() {
        let yaml_str = r#"
            level: "info"
            internal_collection:
              enabled: true
            "#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert!(config.internal_collection.enabled);
        assert_eq!(config.internal_collection.buffer_size_bytes, 64 * 1024);
        assert_eq!(config.internal_collection.max_record_bytes, 16 * 1024);
        assert_eq!(config.internal_collection.max_record_count, 1000);
        assert_eq!(config.internal_collection.flush_interval, Duration::from_secs(1));
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
