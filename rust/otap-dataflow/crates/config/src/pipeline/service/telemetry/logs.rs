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

    /// The list of log processors to configure (for OpenTelemetry SDK output mode).
    /// Only used when `output.mode` is set to `opentelemetry`.
    #[serde(default)]
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
    #[serde(default = "default_global_strategy")]
    pub global: ProviderMode,

    /// Strategy for engine/pipeline threads.
    #[serde(default = "default_engine_strategy")]
    pub engine: ProviderMode,

    /// Default for internal telemetry-reporting components.
    #[serde(default = "default_internal_strategy")]
    pub internal: ProviderMode,
}

/// Logs producer: how log events are captured and routed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderMode {
    /// No-op: log events are silently dropped.
    /// Use for ITR-downstream components to prevent feedback loops.
    Noop,

    /// Regional channel: send individual events to a regional thread.
    /// Drop events when full.
    Regional,

    /// Use OTel as the first class provider.
    OpenTelemetry,
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

impl Default for LoggingStrategies {
    fn default() -> Self {
        Self {
            global: default_global_strategy(),
            engine: default_engine_strategy(),
            internal: default_internal_strategy(),
        }
    }
}

fn default_global_strategy() -> ProviderMode {
    ProviderMode::Regional
}

fn default_engine_strategy() -> ProviderMode {
    ProviderMode::Regional
}

fn default_internal_strategy() -> ProviderMode {
    ProviderMode::Noop
}
