// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logs level configurations.

pub mod processors;

use crate::error::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Internal logs configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogsConfig {
    /// The log level for internal engine logs.
    #[serde(default)]
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
    #[serde(default)]
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
    /// `tracing` subscriber. Default is Unbuffered. Note that Buffered
    /// requires opt-in thread-local setup.
    #[serde(default = "default_global_provider")]
    pub global: ProviderMode,

    /// Provider mod for engine/pipeline threads. This defines how the
    /// engine thread / core sets the Tokio `tracing`
    /// subscriber. Default is Buffered. Internal logs will be flushed
    /// by either the Internal Telemetry Receiver or the main pipeline
    /// controller.
    #[serde(default = "default_engine_provider")]
    pub engine: ProviderMode,

    /// Provider mode for nodes downstream of Internal Telemetry receiver.
    /// This defaults to Noop to avoid internal feedback.
    #[serde(default = "default_internal_provider")]
    pub internal: ProviderMode,
}

/// Logs producer: how log events are captured and routed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderMode {
    /// Log events are silently ignored.
    Noop,

    /// Immediate delivery to the internal telemetry pipeline.
    Immediate,

    /// Use OTel-Rust as the provider.
    OpenTelemetry,

    /// Synchronous console logging. Note! This can block the producing thread.
    Raw,
}

impl ProviderMode {
    /// Returns true if this requires a LogsReporter channel for
    /// asynchronous logging.
    #[must_use]
    pub fn needs_reporter(&self) -> bool {
        matches!(self, Self::Immediate)
    }
}

/// Output mode: what the recipient does with received events for
/// provider logging modes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Noop prevents the use of the Unbuffered mode. This output mode
    /// can be set when all providers are configured to avoid the
    /// internal output configuration through Noop, Raw, or
    /// OpenTelemetry settings.
    Noop,

    /// Direct console logging: format and print directly to console
    /// (stdout/stderr) from the logs collector thread, bypasses any
    /// internal use of the dataflow engine.  ERROR and WARN go to
    /// stderr, others to stdout.
    #[default]
    Direct,

    /// Route to the internal telemetry pipeline.
    Internal,
}

fn default_output() -> OutputMode {
    OutputMode::Direct
}

fn default_global_provider() -> ProviderMode {
    ProviderMode::Immediate
}

fn default_engine_provider() -> ProviderMode {
    ProviderMode::Immediate
}

fn default_internal_provider() -> ProviderMode {
    ProviderMode::Noop
}

fn default_providers() -> LoggingProviders {
    LoggingProviders {
        global: ProviderMode::Immediate,
        engine: ProviderMode::Immediate,
        internal: default_internal_provider(),
    }
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
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
    /// - `output` is `Noop` but a provider uses `Immediate`
    ///   (logs would be sent but discarded)
    /// - `engine` is `OpenTelemetry` but `global` is not
    ///   (current implementation restriction: the SDK logger provider is only
    ///   configured when global uses OpenTelemetry)
    pub fn validate(&self) -> Result<(), Error> {
        if self.providers.internal.needs_reporter() {
            return Err(Error::InvalidUserConfig {
                error: format!(
                    "internal provider is invalid: {:?}",
                    self.providers.internal
                ),
            });
        }
        if self.output == OutputMode::Noop {
            let global_reports = self.providers.global.needs_reporter();
            let engine_reports = self.providers.engine.needs_reporter();

            if global_reports || engine_reports {
                return Err(Error::InvalidUserConfig {
                    error: "output mode is 'noop' but a provider uses an internal reporter".into(),
                });
            }
        }

        // Current implementation restriction: engine OpenTelemetry requires global OpenTelemetry.
        // The SDK logger provider is only created when the global provider is OpenTelemetry.
        // This could be lifted in the future by creating the logger provider independently.
        if self.providers.engine == ProviderMode::OpenTelemetry
            && self.providers.global != ProviderMode::OpenTelemetry
        {
            return Err(Error::InvalidUserConfig {
                error: "engine provider 'opentelemetry' requires global provider to also be \
                        'opentelemetry' (current implementation restriction)"
                    .into(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to parse YAML into LogsConfig.
    fn parse(yaml: &str) -> LogsConfig {
        serde_yaml::from_str(yaml).unwrap()
    }

    /// Helper to create LoggingProviders with specified modes.
    fn providers(
        global: ProviderMode,
        engine: ProviderMode,
        internal: ProviderMode,
    ) -> LoggingProviders {
        LoggingProviders {
            global,
            engine,
            internal,
        }
    }

    /// Helper to create a config with custom providers and output.
    fn config_with(
        output: OutputMode,
        global: ProviderMode,
        engine: ProviderMode,
        internal: ProviderMode,
    ) -> LogsConfig {
        LogsConfig {
            output,
            providers: providers(global, engine, internal),
            ..Default::default()
        }
    }

    /// Asserts validation fails with expected substring in error message.
    fn assert_invalid(config: &LogsConfig, expected_msg: &str) {
        let err = config.validate().unwrap_err();
        assert!(matches!(err, Error::InvalidUserConfig { .. }));
        assert!(
            err.to_string().contains(expected_msg),
            "Expected '{}' in: {}",
            expected_msg,
            err
        );
    }

    // ==================== Defaults & Parsing ====================

    #[test]
    fn test_defaults() {
        // Manual Default impl matches serde defaults
        let config = LogsConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.output, OutputMode::Direct);
        assert_eq!(config.providers.global, ProviderMode::Immediate);
        assert_eq!(config.providers.engine, ProviderMode::Immediate);
        assert_eq!(config.providers.internal, ProviderMode::Noop);
        assert!(config.processors.is_empty());

        // Serde defaults should match Rust Default
        let parsed = parse("{}");
        assert_eq!(parsed.level, config.level);
        assert_eq!(parsed.output, config.output);
        assert_eq!(parsed.providers.global, config.providers.global);
        assert_eq!(parsed.providers.engine, config.providers.engine);
        assert_eq!(parsed.providers.internal, config.providers.internal);
    }

    #[test]
    fn test_log_level_parsing() {
        let cases = [
            ("off", LogLevel::Off),
            ("debug", LogLevel::Debug),
            ("info", LogLevel::Info),
            ("warn", LogLevel::Warn),
            ("error", LogLevel::Error),
        ];
        for (name, expected) in cases {
            assert_eq!(parse(&format!("level: {name}")).level, expected);
        }
    }

    #[test]
    fn test_output_mode_parsing() {
        let cases = [
            ("noop", OutputMode::Noop),
            ("direct", OutputMode::Direct),
            ("internal", OutputMode::Internal),
        ];
        for (name, expected) in cases {
            assert_eq!(parse(&format!("output: {name}")).output, expected);
        }
    }

    #[test]
    fn test_provider_mode_parsing() {
        let config = parse("providers: { global: noop, engine: immediate, internal: raw }");
        assert_eq!(config.providers.global, ProviderMode::Noop);
        assert_eq!(config.providers.engine, ProviderMode::Immediate);
        assert_eq!(config.providers.internal, ProviderMode::Raw);

        let config = parse("providers: { global: opentelemetry, engine: opentelemetry }");
        assert_eq!(config.providers.global, ProviderMode::OpenTelemetry);
        assert_eq!(config.providers.engine, ProviderMode::OpenTelemetry);
    }

    // ==================== ProviderMode::needs_reporter ====================

    #[test]
    fn test_needs_reporter() {
        use ProviderMode::*;
        let cases = [
            (Noop, false),
            (Immediate, true),
            (OpenTelemetry, false),
            (Raw, false),
        ];
        for (mode, expected) in cases {
            assert_eq!(mode.needs_reporter(), expected, "{mode:?}");
        }
    }

    // ==================== Validation ====================

    #[test]
    fn test_validate_default_succeeds() {
        assert!(LogsConfig::default().validate().is_ok());
    }

    #[test]
    fn test_validate_internal_cannot_use_reporter() {
        use ProviderMode::*;
        let config = config_with(OutputMode::Direct, Noop, Noop, Immediate);
        assert_invalid(&config, "internal provider is invalid");
    }

    #[test]
    fn test_validate_noop_output_rejects_reporter_providers() {
        use ProviderMode::*;
        // Global sends to reporter but output is Noop
        let config = config_with(OutputMode::Noop, Immediate, Noop, Noop);
        assert_invalid(&config, "output mode is 'noop'");

        // Engine sends to reporter but output is Noop
        let config = config_with(OutputMode::Noop, Noop, Immediate, Noop);
        assert_invalid(&config, "output mode is 'noop'");
    }

    #[test]
    fn test_validate_noop_output_allows_non_reporter_providers() {
        use ProviderMode::*;
        // All providers that don't need reporter are fine with Noop output
        for (global, engine) in [
            (Noop, Noop),
            (Noop, Raw),
            (Raw, Noop),
            (OpenTelemetry, OpenTelemetry),
        ] {
            let config = config_with(OutputMode::Noop, global, engine, Noop);
            assert!(
                config.validate().is_ok(),
                "Failed for global={global:?}, engine={engine:?}"
            );
        }
    }

    #[test]
    fn test_validate_engine_otel_requires_global_otel() {
        use ProviderMode::*;
        // Engine OpenTelemetry without global OpenTelemetry fails
        for global in [Noop, Immediate, Raw] {
            let config = config_with(OutputMode::Direct, global, OpenTelemetry, Noop);
            assert_invalid(&config, "opentelemetry");
        }

        // Both OpenTelemetry succeeds
        let config = config_with(OutputMode::Direct, OpenTelemetry, OpenTelemetry, Noop);
        assert!(config.validate().is_ok());
    }
}
