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
    /// `tracing` subscriber. Default is ConsoleAsync.
    #[serde(default = "default_global_provider")]
    pub global: ProviderMode,

    /// Provider mod for engine/pipeline threads. This defines how the
    /// engine thread / core sets the Tokio `tracing`
    /// subscriber. Default is ConsoleAsync. Internal logs will be flushed
    /// by either the Internal Telemetry Receiver or the main pipeline
    /// controller.
    #[serde(default = "default_engine_provider")]
    pub engine: ProviderMode,

    /// Provider mode for nodes downstream of Internal Telemetry receiver.
    /// This defaults to Noop to avoid internal feedback.
    #[serde(default = "default_internal_provider")]
    pub internal: ProviderMode,

    /// Provider mode for admin threads. Cannot be ConsoleAsync as
    /// that would create a feedback loop. Defaults to ConsoleDirect.
    #[serde(default = "default_admin_provider")]
    pub admin: ProviderMode,
}

/// Logs producer: how log events are captured and routed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderMode {
    /// Log events are silently ignored.
    Noop,

    /// Delivery using the internal telemetry system.
    ITS,

    /// Use OTel-Rust as the provider.
    OpenTelemetry,

    /// Asynchronous console logging. The caller writes to a channel
    /// the same as ITS delivery, but bypasses the internal pipeline
    /// with console logging.
    #[serde(rename = "console_async")]
    ConsoleAsync,

    /// Synchronous console logging. Note! This can block the
    /// producing thread.  The caller writes directly to the console.
    #[serde(rename = "console_direct")]
    ConsoleDirect,
}

impl ProviderMode {
    /// Returns true if this requires a LogsReporter channel for
    /// asynchronous logging.
    #[must_use]
    pub const fn needs_reporter(&self) -> bool {
        matches!(self, Self::ITS | Self::ConsoleAsync)
    }
}

const fn default_global_provider() -> ProviderMode {
    ProviderMode::ConsoleAsync
}

const fn default_engine_provider() -> ProviderMode {
    ProviderMode::ConsoleAsync
}

const fn default_internal_provider() -> ProviderMode {
    ProviderMode::Noop
}

const fn default_admin_provider() -> ProviderMode {
    ProviderMode::ConsoleDirect
}

fn default_providers() -> LoggingProviders {
    LoggingProviders {
        global: default_global_provider(),
        engine: default_engine_provider(),
        internal: default_internal_provider(),
        admin: default_admin_provider(),
    }
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            providers: default_providers(),
            processors: Vec::new(),
        }
    }
}

impl LogsConfig {
    /// Validate the logs configuration.
    ///
    /// Returns an error if:
    /// - `internal` is configured to use ITS, ConsoleAsync (needs_reporter())
    /// - `admin` is configured to use ConsoleAsync (would loop to itself)
    /// - `engine` is `OpenTelemetry` but `global` is not
    ///   (current implementation restriction).
    pub fn validate(&self) -> Result<(), Error> {
        if self.providers.internal.needs_reporter() {
            return Err(Error::InvalidUserConfig {
                error: format!(
                    "internal provider is invalid: {:?}",
                    self.providers.internal
                ),
            });
        }
        // Admin provider cannot use ConsoleAsync because the observed-state-store
        // runs in an admin thread and implements console_async logging. Using
        // ConsoleAsync for admin would create a feedback loop.
        if self.providers.admin == ProviderMode::ConsoleAsync {
            return Err(Error::InvalidUserConfig {
                error: "admin provider cannot be 'console_async' (would create feedback loop); \
                        use 'console_direct' or another mode instead"
                    .into(),
            });
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
        admin: ProviderMode,
    ) -> LoggingProviders {
        LoggingProviders {
            global,
            engine,
            internal,
            admin,
        }
    }

    /// Helper to create a config with custom providers.
    fn config_with(
        global: ProviderMode,
        engine: ProviderMode,
        internal: ProviderMode,
        admin: ProviderMode,
    ) -> LogsConfig {
        LogsConfig {
            providers: providers(global, engine, internal, admin),
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

    #[test]
    fn test_defaults() {
        // Manual Default impl matches serde defaults
        let config = LogsConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.providers.global, ProviderMode::ConsoleAsync);
        assert_eq!(config.providers.engine, ProviderMode::ConsoleAsync);
        assert_eq!(config.providers.internal, ProviderMode::Noop);
        assert_eq!(config.providers.admin, ProviderMode::ConsoleDirect);
        assert!(config.processors.is_empty());

        // Serde defaults should match Rust Default
        let parsed = parse("{}");
        assert_eq!(parsed.level, config.level);
        assert_eq!(parsed.providers.global, config.providers.global);
        assert_eq!(parsed.providers.engine, config.providers.engine);
        assert_eq!(parsed.providers.internal, config.providers.internal);
        assert_eq!(parsed.providers.admin, config.providers.admin);
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
    fn test_provider_mode_parsing() {
        let config = parse("providers: { global: noop, engine: its, internal: console_direct }");
        assert_eq!(config.providers.global, ProviderMode::Noop);
        assert_eq!(config.providers.engine, ProviderMode::ITS);
        assert_eq!(config.providers.internal, ProviderMode::ConsoleDirect);

        let config = parse("providers: { global: opentelemetry, engine: opentelemetry }");
        assert_eq!(config.providers.global, ProviderMode::OpenTelemetry);
        assert_eq!(config.providers.engine, ProviderMode::OpenTelemetry);
    }

    #[test]
    fn test_needs_reporter() {
        use ProviderMode::*;
        let cases = [
            (Noop, false),
            (ITS, true),
            (OpenTelemetry, false),
            (ConsoleDirect, false),
            (ConsoleAsync, true),
        ];
        for (mode, expected) in cases {
            assert_eq!(mode.needs_reporter(), expected, "{mode:?}");
        }
    }

    #[test]
    fn test_validate_default_succeeds() {
        assert!(LogsConfig::default().validate().is_ok());
    }

    #[test]
    fn test_validate_internal_cannot_use_reporter() {
        use ProviderMode::*;
        let config = config_with(Noop, Noop, ITS, Noop);
        assert_invalid(&config, "internal provider is invalid");

        let config = config_with(Noop, Noop, ConsoleAsync, Noop);
        assert_invalid(&config, "internal provider is invalid");
    }

    #[test]
    fn test_validate_engine_otel_requires_global_otel() {
        use ProviderMode::*;
        // Engine OpenTelemetry without global OpenTelemetry fails
        for global in [Noop, ITS, ConsoleDirect, ConsoleAsync] {
            let config = config_with(global, OpenTelemetry, Noop, Noop);
            assert_invalid(&config, "opentelemetry");
        }

        // Both OpenTelemetry succeeds
        let config = config_with(OpenTelemetry, OpenTelemetry, Noop, Noop);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_admin_cannot_use_console_async() {
        use ProviderMode::*;
        // Admin ConsoleAsync should fail validation
        let config = LogsConfig {
            providers: providers(ConsoleAsync, ConsoleAsync, Noop, ConsoleAsync),
            ..Default::default()
        };
        assert_invalid(&config, "admin provider cannot be 'console_async'");

        // Others should succeed
        for admin in [Noop, ITS, ConsoleDirect, OpenTelemetry] {
            let config = LogsConfig {
                providers: providers(ConsoleAsync, ConsoleAsync, Noop, admin),
                ..Default::default()
            };
            assert!(config.validate().is_ok());
        }
    }
}
