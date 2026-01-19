// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal Telemetry System (ITS) used to instrument the OTAP Dataflow Engine. This system
//! currently focuses on entities metrics but will be extended to cover events and traces.
//!
//! Our instrumentation framework follows a type-safe approach with the goals of being:
//!
//! * entity-centric: metrics are defined and collected per entity (e.g. pipeline, node, channel)
//! * less error-prone: everything is encoded in the type system as structs, field names, and
//!   annotations to provide metadata (e.g. unit).
//! * more performant: a collection of metrics is encoded as a struct with fields of counter
//!   type (no hashmap or other dynamic data structures). Several metrics that share the same
//!   entity attributes don’t have to define those attributes multiple times.
//! * compatible with the semantic conventions: the definition of the entities and metrics produced
//!   by the engine will be available in the semantic convention format.
//!
//! More details on the approach in the telemetry [guides](../../docs/telemetry/README.md).
//!
//! Future directions:
//!
//! * NUMA-aware architecture (soon)
//! * Native support for events
//! * Native support for traces
//! * Export of a registry compatible with the semantic registry format
//! * Client SDK generation with Weaver

use std::sync::Arc;

use crate::event::ObservedEventReporter;
use crate::error::Error;
use crate::registry::TelemetryRegistryHandle;
use crate::tracing_init::TracingSetup;
use otap_df_config::pipeline::service::telemetry::logs::{LogLevel, ProviderMode};
use otap_df_config::pipeline::service::telemetry::TelemetryConfig;

pub mod attributes;
pub mod collector;
pub mod descriptor;
pub mod error;
/// Event types for lifecycle and log events.
pub mod event;
pub mod instrument;
/// Internal logs/events module for engine.
pub mod internal_events;
pub mod metrics;
/// OpenTelemetry SDK provider configuration.
pub mod otel_sdk;
pub mod registry;
pub mod reporter;
pub mod self_tracing;
pub mod semconv;
/// Tokio tracing subscriber initialization.
pub mod tracing_init;

// Re-export _private module from internal_events for macro usage.
// This allows the otel_info!, otel_warn!, etc. macros to work in other crates
// without requiring them to add tracing as a direct dependency.
#[doc(hidden)]
pub use internal_events::_private;

// Re-export tracing span macros and types for crates that need span instrumentation.
// This allows dependent crates to use spans without adding tracing as a direct dependency.
// Re-exported with otel_ prefix for naming consistency with otel_info!, otel_warn!, etc.
pub use tracing::Span as OtelSpan;
pub use tracing::debug_span as otel_debug_span;
pub use tracing::error_span as otel_error_span;
pub use tracing::info_span as otel_info_span;
pub use tracing::trace_span as otel_trace_span;
pub use tracing::warn_span as otel_warn_span;

// TODO This should be #[cfg(test)], but something is preventing it from working.
// The #[cfg(test)]-labeled otap_batch_processor::test_helpers::from_config
// can't load this module unless I remove #[cfg(test)]! See #1304.
pub mod entity;
pub mod testing;

use opentelemetry_sdk::{logs::SdkLoggerProvider, metrics::SdkMeterProvider};

/// The internal telemetry system - unified entry point for all telemetry.
///
/// This system manages:
/// - Internal multivariate metrics (registry, collection, reporting)
/// - OpenTelemetry SDK providers (metrics and logs export)
/// - Tokio tracing subscriber configuration (deferred until `init_global_subscriber` is called)
///
/// # Initialization Pattern
///
/// Create the `ObservedStateStore` first, then pass its reporter to the telemetry system.
/// The global tracing subscriber is NOT initialized during `new()` - call
/// `init_global_subscriber()` when ready.
///
/// ```ignore
/// let obs_store = ObservedStateStore::new(...);
/// let telemetry = InternalTelemetrySystem::new(config, obs_store.reporter())?;
/// 
/// // Now initialize global logging (uses the reporter internally)
/// telemetry.init_global_subscriber();
/// ```
pub struct InternalTelemetrySystem {
    // === Internal Metrics Subsystem ===
    /// The telemetry registry that holds all registered entities and metrics (data + metadata).
    registry: TelemetryRegistryHandle,

    /// The process collecting and processing internal signals.
    collector: Arc<collector::InternalCollector>,

    /// The process reporting metrics to an external system.
    metrics_reporter: reporter::MetricsReporter,

    /// The dispatcher that flushes internal telemetry metrics.
    dispatcher: Arc<metrics::dispatcher::MetricsDispatcher>,

    // === OTel SDK Subsystem ===
    /// OTel SDK meter provider for metrics export.
    meter_provider: SdkMeterProvider,

    /// OTel SDK logger provider for logs export (optional, only for OpenTelemetry mode).
    logger_provider: Option<SdkLoggerProvider>,

    /// Tokio runtime for OTLP exporters (kept alive).
    _otel_runtime: Option<tokio::runtime::Runtime>,

    // === Logging Configuration ===
    /// Log level from config.
    log_level: LogLevel,

    /// Global provider mode from config.
    global_provider_mode: ProviderMode,

    /// Engine provider mode from config.
    engine_provider_mode: ProviderMode,

    /// Event reporter for async logging modes (ConsoleAsync, future ITS).
    /// Held internally so that both global and per-thread subscribers can use it.
    event_reporter: ObservedEventReporter,
}

impl InternalTelemetrySystem {
    /// Creates a new [`InternalTelemetrySystem`] initialized with the given configuration.
    ///
    /// This creates:
    /// 1. The internal metrics subsystem (registry, collector, reporter, dispatcher)
    /// 2. The OpenTelemetry SDK providers (meter, and optionally logger)
    ///
    /// The `event_reporter` is required for async logging modes (`ConsoleAsync`, future `ITS`).
    /// Create the `ObservedStateStore` first and pass its reporter here.
    ///
    /// **Note:** The global tracing subscriber is NOT initialized here. Call
    /// `init_global_subscriber()` when ready to start logging.
    pub fn new(
        config: &TelemetryConfig,
        event_reporter: ObservedEventReporter,
    ) -> Result<Self, Error> {
        // Validate logs config
        config.logs.validate().map_err(|e| Error::ConfigurationError(e.to_string()))?;

        // 1. Create internal metrics subsystem
        let telemetry_registry = TelemetryRegistryHandle::new();
        let (collector, metrics_reporter) =
            collector::InternalCollector::new(config, telemetry_registry.clone());
        let dispatcher = Arc::new(metrics::dispatcher::MetricsDispatcher::new(
            telemetry_registry.clone(),
            config.reporting_interval,
        ));

        // 2. Create OTel SDK providers
        // Logger provider is only needed for OpenTelemetry mode
        let needs_logger_provider = config.logs.providers.global == ProviderMode::OpenTelemetry
            || config.logs.providers.engine == ProviderMode::OpenTelemetry;

        let otel_client = otel_sdk::OpentelemetryClient::new(config, needs_logger_provider)?;
        let meter_provider = otel_client.meter_provider().clone();
        let logger_provider = otel_client.logger_provider().cloned();
        let otel_runtime = otel_client.into_runtime();

        Ok(Self {
            registry: telemetry_registry,
            collector: Arc::new(collector),
            metrics_reporter,
            dispatcher,
            meter_provider,
            logger_provider,
            _otel_runtime: otel_runtime,
            log_level: config.logs.level,
            global_provider_mode: config.logs.providers.global,
            engine_provider_mode: config.logs.providers.engine,
            event_reporter,
        })
    }

    /// Initialize the global tracing subscriber.
    ///
    /// This sets up the global subscriber based on the configured `global` provider mode.
    /// The event reporter passed to `new()` is used internally for async modes.
    pub fn init_global_subscriber(&self) {
        let setup = self.tracing_setup_for(self.global_provider_mode);

        if let Err(err) = setup.try_init_global(self.log_level) {
            raw_error!("tracing.subscriber.init", error = err.to_string());
        }
    }

    /// Returns a `TracingSetup` for the given provider mode.
    ///
    /// This is useful for per-thread subscriber configuration in the engine.
    /// The event reporter is taken from the internal state.
    #[must_use]
    pub fn tracing_setup_for(&self, mode: ProviderMode) -> TracingSetup {
        match mode {
            ProviderMode::Noop => TracingSetup::Noop,

            ProviderMode::ConsoleDirect => TracingSetup::ConsoleDirect,

            ProviderMode::ConsoleAsync => TracingSetup::ConsoleAsync {
                reporter: self.event_reporter.clone(),
            },

            ProviderMode::OpenTelemetry => {
                let provider = self.logger_provider.as_ref().expect(
                    "OpenTelemetry mode requires logger_provider",
                );
                TracingSetup::OpenTelemetry {
                    logger_provider: provider.clone(),
                }
            }

            ProviderMode::ITS => {
                // ITS mode not yet implemented - fall back to Noop
                raw_error!("ITS provider mode not yet implemented, falling back to Noop");
                TracingSetup::Noop
            }
        }
    }

    /// Returns a `TracingSetup` for engine threads.
    ///
    /// This uses the configured `engine` provider mode from the config.
    #[must_use]
    pub fn engine_tracing_setup(&self) -> TracingSetup {
        self.tracing_setup_for(self.engine_provider_mode)
    }

    /// Returns the configured log level.
    #[must_use]
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Returns a shareable/cloneable handle to the telemetry registry.
    #[must_use]
    pub fn registry(&self) -> TelemetryRegistryHandle {
        self.registry.clone()
    }

    /// Returns a shareable/cloneable handle to the internal metrics collector.
    #[must_use]
    pub fn collector(&self) -> Arc<collector::InternalCollector> {
        self.collector.clone()
    }

    /// Returns a shareable/cloneable handle to the metrics reporter.
    #[must_use]
    pub fn metrics_reporter(&self) -> reporter::MetricsReporter {
        self.metrics_reporter.clone()
    }

    /// Returns a clone of the event reporter for observed events.
    #[must_use]
    pub fn event_reporter(&self) -> ObservedEventReporter {
        self.event_reporter.clone()
    }

    /// Returns a shareable handle to the metrics dispatcher.
    #[must_use]
    pub fn dispatcher(&self) -> Arc<metrics::dispatcher::MetricsDispatcher> {
        self.dispatcher.clone()
    }

    /// Shuts down the OpenTelemetry SDK providers.
    pub fn shutdown(self) -> Result<(), Error> {
        let meter_shutdown_result = self.meter_provider.shutdown();
        let logger_shutdown_result = self
            .logger_provider
            .map(|p| p.shutdown())
            .transpose();

        if let Err(e) = meter_shutdown_result {
            return Err(Error::ShutdownError(e.to_string()));
        }

        if let Err(e) = logger_shutdown_result {
            return Err(Error::ShutdownError(e.to_string()));
        }
        Ok(())
    }
}

impl Default for InternalTelemetrySystem {
    fn default() -> Self {
        // Create a dummy channel for testing - events will be dropped
        let (sender, _receiver) = flume::bounded(1);
        let dummy_reporter = ObservedEventReporter::new(
            std::time::Duration::from_millis(1),
            sender,
        );

        Self::new(&TelemetryConfig::default(), dummy_reporter)
            .expect("default telemetry config should be valid")
    }
}
