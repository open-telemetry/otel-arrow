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

    /// Logging strategy configuration for different thread contexts.
    #[serde(default)]
    pub strategies: LoggingStrategies,

    /// How the admin thread handles received log events.
    #[serde(default)]
    pub output: LogOutputConfig,

    /// The list of log processors to configure (for OpenTelemetry SDK output mode).
    /// Only used when `output.mode` is set to `opentelemetry`.
    #[serde(default)]
    pub processors: Vec<processors::LogProcessorConfig>,
}

/// Logging strategies for different execution contexts.
///
/// Controls how log events are captured and routed to the admin thread.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggingStrategies {
    /// Strategy for non-engine threads (admin, metrics aggregator, etc.).
    /// These threads don't have an EffectHandler and use the global tracing subscriber.
    /// Default: `global` (send to admin channel).
    #[serde(default = "default_global_strategy")]
    pub global: ProducerStrategy,

    /// Strategy for engine/pipeline threads.
    /// These threads have an EffectHandler and use buffered logging.
    /// Default: `buffered` (thread-local buffer, batch flush on timer).
    #[serde(default = "default_engine_strategy")]
    pub engine: ProducerStrategy,
}

impl Default for LoggingStrategies {
    fn default() -> Self {
        Self {
            global: default_global_strategy(),
            engine: default_engine_strategy(),
        }
    }
}

fn default_global_strategy() -> ProducerStrategy {
    ProducerStrategy::Global
}

fn default_engine_strategy() -> ProducerStrategy {
    ProducerStrategy::Buffered
}

/// Producer strategy: how log events are captured and routed to the admin thread.
///
/// Used to configure logging behavior for different thread types:
/// - Global subscriber for non-engine threads
/// - Engine threads with EffectHandler
/// - Per-component (future: for ITR downstream to prevent feedback)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProducerStrategy {
    /// No-op: log events are silently dropped.
    /// Use for ITR-downstream components to prevent feedback loops.
    Noop,

    /// Global channel: send individual events to the admin collector thread.
    /// Non-blocking (drops if channel full). Default for non-engine threads.
    Global,

    /// Buffered: accumulate events in thread-local buffer, flush on timer.
    /// Default for engine threads. Events are batched before sending to admin.
    Buffered,
}

/// Configuration for how the admin thread outputs received log events.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogOutputConfig {
    /// The output mode for log events received by the admin thread.
    #[serde(default = "default_output_mode")]
    pub mode: OutputMode,

    /// Ring buffer capacity for `memory` mode (number of log entries).
    /// Also used for the `/logs` HTTP endpoint regardless of mode.
    #[serde(default = "default_ring_buffer_capacity")]
    pub ring_buffer_capacity: usize,
}

impl Default for LogOutputConfig {
    fn default() -> Self {
        Self {
            mode: default_output_mode(),
            ring_buffer_capacity: default_ring_buffer_capacity(),
        }
    }
}

fn default_output_mode() -> OutputMode {
    OutputMode::Raw
}

fn default_ring_buffer_capacity() -> usize {
    1000
}

/// Output mode: what the admin thread does with received log events.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Raw logging: format and print directly to console (stdout/stderr).
    /// ERROR/WARN go to stderr, others to stdout.
    Raw,

    /// Memory only: store in ring buffer for `/logs` HTTP endpoint.
    /// No console output. Useful for headless/production deployments.
    Memory,

    /// OpenTelemetry SDK: forward to OTel logging SDK with configured processors.
    /// Events are sent through the OTel appender bridge for OTLP export.
    Opentelemetry,
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

    #[test]
    fn test_logging_strategies_deserialize() {
        let yaml_str = r#"
            level: "info"
            strategies:
              global: global
              engine: buffered
            "#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.strategies.global, ProducerStrategy::Global);
        assert_eq!(config.strategies.engine, ProducerStrategy::Buffered);
    }

    #[test]
    fn test_logging_strategies_default() {
        let config = LogsConfig::default();
        assert_eq!(config.strategies.global, ProducerStrategy::Global);
        assert_eq!(config.strategies.engine, ProducerStrategy::Buffered);
        assert_eq!(config.output.mode, OutputMode::Raw);
    }

    #[test]
    fn test_output_modes() {
        let yaml_str = r#"
            level: "info"
            output:
              mode: memory
              ring_buffer_capacity: 5000
            "#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.output.mode, OutputMode::Memory);
        assert_eq!(config.output.ring_buffer_capacity, 5000);
    }

    #[test]
    fn test_opentelemetry_output() {
        let yaml_str = r#"
            level: "info"
            output:
              mode: opentelemetry
            processors:
              - batch:
                  exporter:
                    console:
            "#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.output.mode, OutputMode::Opentelemetry);
        assert_eq!(config.processors.len(), 1);
    }

    #[test]
    fn test_noop_strategy_for_itr() {
        let yaml_str = r#"
            level: "info"
            strategies:
              global: noop
              engine: noop
            "#;
        let config: LogsConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config.strategies.global, ProducerStrategy::Noop);
        assert_eq!(config.strategies.engine, ProducerStrategy::Noop);
    }
}
