// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio tracing subscriber initialization.
//!
//! This module handles the setup of the global tokio tracing subscriber,
//! which is separate from OpenTelemetry SDK configuration. The tracing
//! subscriber determines how log events are captured and routed.

use crate::event::{LogEvent, ObservedEventReporter};
use crate::self_tracing::{ConsoleWriter, LogRecord, RawLoggingLayer};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use otap_df_config::pipeline::service::telemetry::logs::LogLevel;
use std::time::SystemTime;
use tracing::level_filters::LevelFilter;
use tracing::{Dispatch, Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer as TracingLayer};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

/// Creates an `EnvFilter` for the given log level.
///
/// If `RUST_LOG` is set in the environment, it takes precedence for fine-grained control.
/// Otherwise, falls back to the config level with known noisy dependencies (h2, hyper) silenced.
#[must_use]
pub fn create_env_filter(level: LogLevel) -> EnvFilter {
    let level_filter = match level {
        LogLevel::Off => LevelFilter::OFF,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
    };

    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default filter: use config level, but silence known noisy HTTP dependencies
        EnvFilter::new(format!("{level_filter},h2=off,hyper=off"))
    })
}

/// Combined tracing configuration for a thread.
///
/// This struct bundles the provider setup with the log level, allowing
/// the `InternalTelemetrySystem` to control all tracing configuration.
/// Future enhancements may include per-thread log level overrides.
#[derive(Clone)]
pub struct TracingSetup {
    /// The provider mode configuration.
    pub provider: ProviderSetup,
    /// The log level for filtering.
    pub log_level: LogLevel,
}

impl TracingSetup {
    /// Create a new tracing setup.
    #[must_use]
    pub fn new(provider: ProviderSetup, log_level: LogLevel) -> Self {
        Self {
            provider,
            log_level,
        }
    }

    /// Initialize this setup as the global tracing subscriber.
    pub fn try_init_global(&self) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        self.provider.try_init_global(self.log_level)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    pub fn with_subscriber<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.provider.with_subscriber(self.log_level, f)
    }
}

/// Provider configuration for setting up a tracing subscriber.
#[derive(Clone)]
pub enum ProviderSetup {
    /// Logs are silently dropped.
    Noop,

    /// Synchronous console logging via `RawLoggingLayer`.
    ConsoleDirect,

    /// Asynchronous console logging via an observed event reporter.
    ConsoleAsync {
        /// Reporter to send log events through.
        reporter: ObservedEventReporter,
    },

    /// OpenTelemetry SDK logging via `OpenTelemetryTracingBridge`.
    OpenTelemetry {
        /// OpenTelemetry SDK logger provider.
        logger_provider: SdkLoggerProvider,
    },
}

impl ProviderSetup {
    /// Build a `Dispatch` for this provider setup with the given log level.
    fn build_dispatch(&self, log_level: LogLevel) -> Dispatch {
        let filter = || create_env_filter(log_level);

        match self {
            ProviderSetup::Noop => Dispatch::new(tracing::subscriber::NoSubscriber::new()),

            ProviderSetup::ConsoleDirect => Dispatch::new(
                Registry::default()
                    .with(filter())
                    .with(RawLoggingLayer::new(ConsoleWriter::color())),
            ),

            ProviderSetup::ConsoleAsync { reporter } => {
                let layer = ConsoleAsyncLayer::new(reporter);
                Dispatch::new(Registry::default().with(filter()).with(layer))
            }

            ProviderSetup::OpenTelemetry { logger_provider } => {
                let sdk_layer = OpenTelemetryTracingBridge::new(logger_provider);
                Dispatch::new(Registry::default().with(filter()).with(sdk_layer))
            }
        }
    }

    /// Initialize this setup as the global tracing subscriber.
    pub fn try_init_global(
        &self,
        log_level: LogLevel,
    ) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        let dispatch = self.build_dispatch(log_level);
        tracing::dispatcher::set_global_default(dispatch)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    pub fn with_subscriber<F, R>(&self, log_level: LogLevel, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let dispatch = self.build_dispatch(log_level);
        tracing::dispatcher::with_default(&dispatch, f)
    }
}

/// A tracing layer that sends log records asynchronously via a channel.
pub struct ConsoleAsyncLayer {
    reporter: ObservedEventReporter,
}

impl ConsoleAsyncLayer {
    /// Create a new async logging layer.
    #[must_use]
    pub fn new(reporter: &ObservedEventReporter) -> Self {
        Self {
            reporter: reporter.clone(),
        }
    }
}

impl<S> TracingLayer<S> for ConsoleAsyncLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let time = SystemTime::now();
        let record = LogRecord::new(event);
        self.reporter.log(LogEvent { time, record });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ObservedEvent;
    use crate::{otel_debug, otel_error, otel_info, otel_warn};
    use opentelemetry_sdk::logs::SdkLoggerProvider;
    use otap_df_config::observed_state::SendPolicy;

    fn test_reporter() -> (ObservedEventReporter, flume::Receiver<ObservedEvent>) {
        let (tx, rx) = flume::bounded(16);
        let reporter = ObservedEventReporter::new(SendPolicy::default(), tx);
        (reporter, rx)
    }

    #[test]
    fn tracing_setup_new() {
        let _ = TracingSetup::new(ProviderSetup::Noop, LogLevel::Info);
        let _ = TracingSetup::new(ProviderSetup::ConsoleDirect, LogLevel::Debug);

        let (reporter, _rx) = test_reporter();
        let _ = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, LogLevel::Warn);
    }

    #[test]
    fn noop_provider_runs() {
        let setup = TracingSetup::new(ProviderSetup::Noop, LogLevel::Info);
        setup.with_subscriber(|| {
            otel_info!("log_dropped");
        });
    }

    #[test]
    fn noop_provider_all_levels() {
        for level in [
            LogLevel::Off,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            let setup = TracingSetup::new(ProviderSetup::Noop, level);
            setup.with_subscriber(|| {
                otel_debug!("debug");
                otel_info!("info");
                otel_warn!("warn");
                otel_error!("error");
            });
        }
    }

    #[test]
    fn console_direct_provider_runs() {
        let setup = TracingSetup::new(ProviderSetup::ConsoleDirect, LogLevel::Info);
        setup.with_subscriber(|| {
            otel_info!("console_log");
        });
    }

    #[test]
    fn console_direct_all_levels() {
        for level in [
            LogLevel::Off,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            let setup = TracingSetup::new(ProviderSetup::ConsoleDirect, level);
            setup.with_subscriber(|| {
                otel_debug!("debug");
                otel_info!("info");
                otel_warn!("warn");
                otel_error!("error");
            });
        }
    }

    #[test]
    fn console_async_provider_sends_logs() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, LogLevel::Info);

        setup.with_subscriber(|| {
            otel_info!("async_log");
        });

        // Verify the log was sent through the channel
        let event = receiver.try_recv().expect("should receive log event");
        assert!(
            matches!(event, ObservedEvent::Log(_)),
            "event should be a log"
        );
    }

    #[test]
    fn console_async_all_levels() {
        for level in [
            LogLevel::Off,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            let (reporter, _receiver) = test_reporter();
            let setup = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, level);
            setup.with_subscriber(|| {
                otel_debug!("debug");
                otel_info!("info");
                otel_warn!("warn");
                otel_error!("error");
            });
        }
    }

    #[test]
    fn opentelemetry_provider_runs() {
        let logger_provider = SdkLoggerProvider::builder().build();
        let setup = TracingSetup::new(
            ProviderSetup::OpenTelemetry { logger_provider },
            LogLevel::Info,
        );

        setup.with_subscriber(|| {
            otel_info!("otel_log");
        });
    }

    #[test]
    fn opentelemetry_provider_all_levels() {
        for level in [
            LogLevel::Off,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            let logger_provider = SdkLoggerProvider::builder().build();
            let setup = TracingSetup::new(ProviderSetup::OpenTelemetry { logger_provider }, level);
            setup.with_subscriber(|| {
                otel_debug!("debug");
                otel_info!("info");
                otel_warn!("warn");
                otel_error!("error");
            });
        }
    }

    #[test]
    fn log_level_filters_debug() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, LogLevel::Info);

        setup.with_subscriber(|| {
            otel_debug!("filtered");
        });

        assert!(
            receiver.try_recv().is_err(),
            "debug log should not be received at Info level"
        );
    }

    #[test]
    fn log_level_warn_filters_lower() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, LogLevel::Warn);

        setup.with_subscriber(|| {
            otel_debug!("filtered");
            otel_info!("filtered");
            otel_warn!("not_filtered");
        });

        // Should only receive the warn
        let event = receiver.try_recv().expect("should receive warn");
        assert!(matches!(event, ObservedEvent::Log(_)));
        assert!(receiver.try_recv().is_err(), "should only have one event");
    }

    #[test]
    fn log_level_error_filters_lower() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, LogLevel::Error);

        setup.with_subscriber(|| {
            otel_debug!("filtered");
            otel_info!("filtered");
            otel_warn!("filtered");
            otel_error!("not_filtered");
        });

        let event = receiver.try_recv().expect("should receive error");
        assert!(matches!(event, ObservedEvent::Log(_)));
        assert!(receiver.try_recv().is_err(), "should only have one event");
    }

    #[test]
    fn log_level_off_filters_all() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, LogLevel::Off);

        setup.with_subscriber(|| {
            otel_debug!("filtered");
            otel_info!("filtered");
            otel_warn!("filtered");
            otel_error!("filtered");
        });

        assert!(receiver.try_recv().is_err(), "all logs should be filtered");
    }

    #[test]
    fn log_level_debug_allows_all() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, LogLevel::Debug);

        setup.with_subscriber(|| {
            otel_debug!("d");
            otel_info!("i");
            otel_warn!("w");
            otel_error!("e");
        });

        // Should receive all 4
        for _ in 0..4 {
            let _ = receiver.try_recv().expect("should receive log");
        }
    }

    #[test]
    fn console_async_layer_new() {
        let (reporter, _rx) = test_reporter();
        let _layer = ConsoleAsyncLayer::new(&reporter);
    }

    #[test]
    fn console_async_layer_with_fields() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(ProviderSetup::ConsoleAsync { reporter }, LogLevel::Info);

        setup.with_subscriber(|| {
            otel_info!("structured", key = "value", number = 42);
        });

        let event = receiver.try_recv().expect("should receive log");
        assert!(matches!(event, ObservedEvent::Log(_)));
    }

    #[test]
    fn provider_setup_with_subscriber_all_variants() {
        ProviderSetup::Noop.with_subscriber(LogLevel::Info, || {
            otel_info!("noop");
        });

        ProviderSetup::ConsoleDirect.with_subscriber(LogLevel::Info, || {
            otel_info!("console_direct");
        });

        let (reporter, _rx) = test_reporter();
        ProviderSetup::ConsoleAsync { reporter }.with_subscriber(LogLevel::Info, || {
            otel_info!("console_async");
        });

        let logger_provider = SdkLoggerProvider::builder().build();
        ProviderSetup::OpenTelemetry { logger_provider }.with_subscriber(LogLevel::Info, || {
            otel_info!("otel");
        });
    }

    #[test]
    fn provider_setup_clone() {
        let _ = ProviderSetup::Noop.clone();
        let _ = ProviderSetup::ConsoleDirect.clone();

        let (reporter, _rx) = test_reporter();
        let _ = ProviderSetup::ConsoleAsync { reporter }.clone();

        let logger_provider = SdkLoggerProvider::builder().build();
        let provider_setup = ProviderSetup::OpenTelemetry { logger_provider }.clone();

        let setup = TracingSetup::new(provider_setup, LogLevel::Info);
        let _cloned = setup.clone();
    }

    #[test]
    fn nested_with_subscriber() {
        let (reporter1, receiver1) = test_reporter();
        let (reporter2, receiver2) = test_reporter();

        let setup1 = TracingSetup::new(
            ProviderSetup::ConsoleAsync {
                reporter: reporter1,
            },
            LogLevel::Info,
        );
        let setup2 = TracingSetup::new(
            ProviderSetup::ConsoleAsync {
                reporter: reporter2,
            },
            LogLevel::Info,
        );

        let result = setup1.with_subscriber(|| {
            otel_info!("outer");
            setup2.with_subscriber(|| {
                otel_info!("inner");
            });
            otel_info!("outer_again");
            100
        });

        assert_eq!(result, 100);

        // Outer should receive 2, inner should receive 1 and no more.
        assert!(receiver1.try_recv().is_ok());
        assert!(receiver2.try_recv().is_ok());
        assert!(receiver1.try_recv().is_ok());

        assert!(receiver1.try_recv().is_err());
        assert!(receiver2.try_recv().is_err());
    }
}
