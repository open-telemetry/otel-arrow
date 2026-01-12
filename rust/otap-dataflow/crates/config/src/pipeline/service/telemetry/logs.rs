// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logs level configurations.

pub mod processors;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Internal logs configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogsConfig {
    /// The log level for internal engine logs.
    pub level: LogLevel,

    /// Logging strategy configuration for different thread contexts.
    pub strategies: LoggingStrategies,

    /// The list of log processors to configure (for OpenTelemetry SDK output mode).
    /// Only used when `output.mode` is set to `opentelemetry`.
    pub processors: Vec<processors::LogProcessorConfig>,
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

/// Logging strategies for different execution contexts.
///
/// Controls how log events are captured and routed to the admin thread.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggingStrategies {
    /// Strategy for non-engine threads.
    pub global: ProviderMode,

    /// Strategy for engine/pipeline threads.
    pub engine: ProviderMode,
}

/// Logs producer: how log events are captured and routed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderMode {
    /// Log events are silently ignored.
    Noop,

    /// Place into a thread-local buffer.
    Buffered,

    /// Non-blocking, immediate delivery.
    Unbuffered,

    /// Use OTel-Rust as the provider.
    OpenTelemetry,

    /// Use synchronous logging.
    Raw,
}

/// Output mode: what the recipient does with received log events.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Disable output.
    Noop,

    /// Raw logging: format and print directly to console (stdout/stderr).
    /// ERROR/WARN go to stderr, others to stdout.
    Raw,

    /// [Demonstrated]: Deliver to a dedicated telemetry pipeline.
    Pipeline,

    /// [Hypothetical]: Store in a memory ring buffer for `/logs` HTTP endpoint.
    Memory,

    /// [Hypothetical]: Forward OTLP bytes into the OTel SDK pipeline (requires
    /// OTLP-bytes-to-SDK-event).
    OpenTelemetry,
}

fn default_level() -> LogLevel {
    LogLevel::Off
}

fn default_strategies() -> LoggingStrategies {
    LoggingStrategies {
        global: ProviderMode::Unbuffered,
        engine: ProviderMode::Buffered,
    }
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            strategies: default_strategies(),
            processors: Vec::new(),
        }
    }
}
