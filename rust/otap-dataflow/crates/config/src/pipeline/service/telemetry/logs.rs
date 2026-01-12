// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logs level configurations.

pub mod processors;

use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Internal Telemetry Receiver node URN for internal logging using OTLP bytes.
pub const INTERNAL_TELEMETRY_RECEIVER_URN: &str = "urn:otel:otlp:telemetry:receiver";

/// Internal logs configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogsConfig {
    /// The log level for internal logs.
    #[serde(default = "default_level")]
    pub level: LogLevel,

    /// Logging provider configuration.
    #[serde(default = "default_providers")]
    pub providers: LoggingProviders,

    /// What to do with collected log events. This applies when any ProviderMode
    /// in providers indicates Buffered or Unbuffered. Does not apply if all
    /// providers are in [Noop, Raw, OpenTelemetry].
    #[serde(default = "default_output")]
    pub output: OutputMode,

    /// OpenTelemetry SDK is configured via processors.
    pub processors: Vec<processors::LogProcessorConfig>,
}

/// Log level for dataflow engine logs.
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

/// Logging providers for different execution contexts.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggingProviders {
    /// Provider mode for non-engine threads. This defines the global Tokio
    /// `tracing` subsriber. Default is Unbuffered. Note that Buffered
    /// requires opt-in thread-local setup.
    pub global: ProviderMode,

    /// Provider mod for engine/pipeline threads. This defines how the
    /// engine thread / core sets the Tokio `tracing`
    /// subscriber. Default is Buffered. Internal logs will be flushed
    /// by either the Internal Telemetry Receiver or the main pipeline
    /// controller.
    pub engine: ProviderMode,

    /// Provider mode for nodes downstream of Internal Telemetry receiver.
    /// This defaults to Noop to avoid internal feedback.
    #[serde(default = "default_internal_provider")]
    pub internal: ProviderMode,
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

    /// Use synchronous logging. Note! This can block the producing thread.
    Raw,
}

/// Output mode: what the recipient does with received events for
/// Buffered and Unbuffered provider logging modes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Noop prevents the use of Buffered and Unbuffered modes. This
    /// output mode can be set when all providers are configured to
    /// avoid the internal output configuration through Noop, Raw, or
    /// OpenTelemetry settings.
    Noop,

    /// Raw logging: format and print directly to console
    /// (stdout/stderr) from the logs collector thread.  ERROR and
    /// WARN go to stderr, others to stdout.
    #[default]
    Raw,

    /// Route to Internal Telemetry Receiver node.  The pipeline must
    /// include a nod with INTERNAL_TELEMETRY_RECEIVER_URN.  The
    /// engine provider mode must be Buffered for internal output.
    Internal,
}

fn default_output() -> OutputMode {
    OutputMode::Raw
}

fn default_level() -> LogLevel {
    LogLevel::Info
}

fn default_internal_provider() -> ProviderMode {
    ProviderMode::Noop
}

fn default_providers() -> LoggingProviders {
    LoggingProviders {
        global: ProviderMode::Unbuffered,
        engine: ProviderMode::Buffered,
        internal: default_internal_provider(),
    }
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            providers: default_providers(),
            output: default_output(),
            processors: Vec::new(),
        }
    }
}

impl LogsConfig {
    /// Validate the logs configuration.
    ///
    /// Returns an error if:
    /// - `output` is `Noop` but a provider uses `Buffered` or `Unbuffered`
    ///   (logs would be sent but discarded)
    /// - `output` is `Internal` but engine provider is not `Buffered`
    pub fn validate(&self) -> Result<(), Error> {
        if self.output == OutputMode::Noop {
            let global_sends = matches!(
                self.providers.global,
                ProviderMode::Buffered | ProviderMode::Unbuffered
            );
            let engine_sends = matches!(
                self.providers.engine,
                ProviderMode::Buffered | ProviderMode::Unbuffered
            );

            if global_sends || engine_sends {
                return Err(Error::InvalidUserConfig {
                    error: "output mode is 'noop' but a provider uses buffered or unbuffered"
                        .into(),
                });
            }
        }

        if self.output == OutputMode::Internal {
            if self.providers.engine != ProviderMode::Buffered {
                return Err(Error::InvalidUserConfig {
                    error: "output mode is 'internal', engine must use buffered provider".into(),
                });
            }
        }

        Ok(())
    }
}
