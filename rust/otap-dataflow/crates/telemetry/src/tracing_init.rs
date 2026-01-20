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
    use std::time::Duration;

    fn test_reporter() -> (ObservedEventReporter, flume::Receiver<ObservedEvent>) {
        let (tx, rx) = flume::bounded(16);
        let reporter = ObservedEventReporter::new(
            Duration::from_millis(100),
            otap_df_config::pipeline::service::telemetry::logs::ProviderMode::Noop,
            tx,
        );
        (reporter, rx)
    }

    /// Test that Noop provider runs without panicking.
    #[test]
    fn noop_provider_runs() {
        let setup = TracingSetup::new(ProviderSetup::Noop, LogLevel::Info);
        setup.with_subscriber(|| {
            tracing::info!("this log is silently dropped");
        });
    }

    /// Test that ConsoleDirect provider runs without panicking.
    #[test]
    fn console_direct_provider_runs() {
        let setup = TracingSetup::new(ProviderSetup::ConsoleDirect, LogLevel::Info);
        setup.with_subscriber(|| {
            tracing::info!("this log goes to console");
        });
    }

    /// Test that ConsoleAsync provider sends logs through the channel.
    #[test]
    fn console_async_provider_sends_logs() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(
            ProviderSetup::ConsoleAsync { reporter },
            LogLevel::Info,
        );

        setup.with_subscriber(|| {
            tracing::info!("async log message");
        });

        // Verify the log was sent through the channel
        let event = receiver.try_recv().expect("should receive log event");
        assert!(
            matches!(event, ObservedEvent::Log(_)),
            "event should be a log"
        );
    }

    /// Test that debug logs are filtered out when log level is Info.
    #[test]
    fn log_level_filters_debug() {
        let (reporter, receiver) = test_reporter();
        let setup = TracingSetup::new(
            ProviderSetup::ConsoleAsync { reporter },
            LogLevel::Info,
        );

        setup.with_subscriber(|| {
            tracing::debug!("this should be filtered out");
        });

        assert!(
            receiver.try_recv().is_err(),
            "debug log should not be received at Info level"
        );
    }
}
