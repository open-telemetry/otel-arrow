// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tokio tracing subscriber initialization.
//!
//! This module handles the setup of the global tokio tracing subscriber,
//! which is separate from OpenTelemetry SDK configuration. The tracing
//! subscriber determines how log events are captured and routed.
//!
//! # Provider Modes
//!
//! The logging infrastructure supports multiple provider modes for different use cases:
//!
//! - **Noop**: Logs are silently dropped. Useful for testing or when logging is disabled.
//! - **ConsoleDirect**: Synchronous console output. Simple but may block the producing thread.
//! - **ConsoleAsync**: Asynchronous console output via the `ObservedEventReporter` channel.
//!   Logs are sent as `EventMessage::Log` variants and printed by a background collector.
//! - **OpenTelemetry**: Routes logs through the OpenTelemetry SDK for export to backends.
//! - **ITS**: Routes logs through the Internal Telemetry System pipeline (not yet implemented).

use crate::event::LogEvent;
use crate::self_tracing::{ConsoleWriter, LogRecord, RawLoggingLayer};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use otap_df_config::pipeline::service::telemetry::logs::{LogLevel, ProviderMode};
use std::time::SystemTime;
use tracing::level_filters::LevelFilter;
use tracing::{Dispatch, Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer as TracingLayer};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

// Re-export for convenience
pub use crate::event::ObservedEventReporter;

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
    ///
    /// This should be called once during startup to set the global subscriber.
    /// Returns an error if a global subscriber has already been set.
    ///
    /// # Notes on Contention
    ///
    /// TODO: The engine uses a thread-per-core model and is NUMA aware.
    /// The global subscriber here is truly global, and hence this will be a source
    /// of contention. We need to evaluate alternatives:
    ///
    /// 1. Set up per thread subscriber using `tracing::subscriber::set_default`.
    /// 2. Use custom subscriber that batches logs in thread-local buffer.
    ///
    /// As of now, this causes contention which we accept temporarily.
    pub fn try_init_global(&self) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        self.provider.try_init_global(self.log_level)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    ///
    /// The closure runs with the configured logging layer active as a thread-local default.
    /// This is useful for per-thread subscriber configuration in the engine.
    pub fn with_subscriber<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.provider.with_subscriber(self.log_level, f)
    }
}

/// Provider configuration for setting up a tracing subscriber.
///
/// This enum captures all the resources needed for each provider mode,
/// allowing deferred initialization of the global subscriber.
#[derive(Clone)]
pub enum ProviderSetup {
    /// Logs are silently dropped.
    Noop,

    /// Synchronous console logging via `RawLoggingLayer`.
    ConsoleDirect,

    /// Asynchronous console logging via the observed event channel.
    /// Logs are sent as `EventMessage::Log` and printed by a background task.
    ConsoleAsync {
        /// Reporter to send log events through.
        reporter: ObservedEventReporter,
    },

    /// OpenTelemetry SDK logging via `OpenTelemetryTracingBridge`.
    OpenTelemetry {
        /// The OpenTelemetry SDK logger provider.
        logger_provider: SdkLoggerProvider,
    },
}

impl ProviderSetup {
    /// Build a `Dispatch` for this provider setup with the given log level.
    fn build_dispatch(&self, log_level: LogLevel) -> Dispatch {
        let filter = create_env_filter(log_level);

        match self {
            ProviderSetup::Noop => Dispatch::new(tracing::subscriber::NoSubscriber::new()),

            ProviderSetup::ConsoleDirect => Dispatch::new(
                Registry::default()
                    .with(filter)
                    .with(RawLoggingLayer::new(ConsoleWriter::color())),
            ),

            ProviderSetup::ConsoleAsync { reporter } => {
                let layer = ConsoleAsyncLayer::new(reporter);
                Dispatch::new(Registry::default().with(filter).with(layer))
            }

            ProviderSetup::OpenTelemetry { logger_provider } => {
                let sdk_layer = OpenTelemetryTracingBridge::new(logger_provider);
                let fmt_layer = tracing_subscriber::fmt::layer().with_thread_names(true);
                Dispatch::new(
                    Registry::default()
                        .with(filter)
                        .with(fmt_layer)
                        .with(sdk_layer),
                )
            }
        }
    }

    /// Initialize this setup as the global tracing subscriber.
    ///
    /// This should be called once during startup to set the global subscriber.
    /// Returns an error if a global subscriber has already been set.
    pub fn try_init_global(
        &self,
        log_level: LogLevel,
    ) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
        let dispatch = self.build_dispatch(log_level);
        tracing::dispatcher::set_global_default(dispatch)
    }

    /// Run a closure with the appropriate tracing subscriber for this setup.
    ///
    /// The closure runs with the configured logging layer active as a thread-local default.
    /// This is useful for per-thread subscriber configuration in the engine.
    pub fn with_subscriber<F, R>(&self, log_level: LogLevel, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let dispatch = self.build_dispatch(log_level);
        tracing::dispatcher::with_default(&dispatch, f)
    }
}

/// A tracing layer that sends log records asynchronously via a channel.
///
/// Each log event is converted to a `LogRecord` and sent through the
/// `ObservedEventReporter` channel as an `EventMessage::Log`. A background
/// task (the observed state store) receives these and prints them to console.
pub struct ConsoleAsyncLayer {
    /// Reporter for sending log events.
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

/// Initializes the global tracing subscriber based on provider mode.
///
/// This is a convenience function that creates a `TracingSetup` and initializes it.
/// For more control (e.g., deferred initialization), use `TracingSetup` directly.
pub fn init_global_subscriber_for_mode(
    log_level: LogLevel,
    mode: ProviderMode,
    logger_provider: Option<&SdkLoggerProvider>,
    event_reporter: Option<&ObservedEventReporter>,
) {
    let provider_setup = match mode {
        ProviderMode::Noop => ProviderSetup::Noop,

        ProviderMode::ConsoleDirect => ProviderSetup::ConsoleDirect,

        ProviderMode::ConsoleAsync => {
            let reporter = event_reporter.expect("ConsoleAsync requires event_reporter");
            ProviderSetup::ConsoleAsync {
                reporter: reporter.clone(),
            }
        }

        ProviderMode::OpenTelemetry => {
            let provider = logger_provider.expect("OpenTelemetry requires logger_provider");
            ProviderSetup::OpenTelemetry {
                logger_provider: provider.clone(),
            }
        }

        ProviderMode::ITS => {
            // ITS mode not yet implemented - fall back to Noop
            // TODO: Implement ITS mode with Internal Telemetry Receiver
            crate::raw_error!("ITS provider mode not yet implemented, falling back to Noop");
            ProviderSetup::Noop
        }
    };

    let setup = TracingSetup::new(provider_setup, log_level);
    if let Err(err) = setup.try_init_global() {
        crate::raw_error!("tracing.subscriber.init", error = err.to_string());
    }
}
