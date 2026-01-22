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

use crate::error::Error;
use crate::event::ObservedEventReporter;
use crate::registry::TelemetryRegistryHandle;
use otap_df_config::observed_state::SendPolicy;
use otap_df_config::pipeline::service::telemetry::TelemetryConfig;
use otap_df_config::pipeline::service::telemetry::logs::{
    LogLevel, LoggingProviders, ProviderMode,
};

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

// Re-export tracing setup types for per-thread subscriber configuration.
pub use tracing_init::TracingSetup;

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

/// The URN for the internal telemetry receiver.
/// Defined here so it can be used by controller, engine, otap, and other crates.
pub const INTERNAL_TELEMETRY_RECEIVER_URN: &str = "urn:otel:otap:internal_telemetry:receiver";

/// Settings for internal telemetry consumption by the Internal Telemetry Receiver.
///
/// This bundles the receiver end of the logs channel and pre-encoded resource bytes
/// for injection into the ITR via the EffectHandler.
#[derive(Clone)]
pub struct InternalTelemetrySettings {
    /// Receiver end of the logs channel for `ObservedEvent::Log` events.
    pub logs_receiver: flume::Receiver<event::ObservedEvent>,
    /// Pre-encoded OTLP resource bytes (ResourceLogs.resource + schema_url fields).
    pub resource_bytes: bytes::Bytes,
}

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
/// - Tokio tracing subscriber configuration
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
    sdk_meter_provider: SdkMeterProvider,

    /// OTel SDK logger provider for logs export (optional, only for OpenTelemetry mode).
    sdk_logger_provider: Option<SdkLoggerProvider>,

    /// Tokio runtime for OTLP exporters (kept alive).
    _otel_runtime: Option<tokio::runtime::Runtime>,

    // === Logging Configuration ===
    /// Log level from config.
    log_level: LogLevel,

    /// The logging providers.
    provider_modes: LoggingProviders,

    /// Event reporter for asynchronous internal logging modes (ConsoleAsync).
    admin_reporter: ObservedEventReporter,

    /// Event reporter for ITS mode (Internal Telemetry System).
    /// Only created when any provider uses ITS mode.
    its_reporter: Option<ObservedEventReporter>,

    /// Pre-encoded resource bytes for OTLP log encoding (for ITR).
    resource_bytes: bytes::Bytes,
}

impl InternalTelemetrySystem {
    /// Creates a new [`InternalTelemetrySystem`] initialized with the given configuration.
    ///
    /// This creates:
    /// 1. The internal metrics subsystem (registry, collector, reporter, dispatcher)
    /// 2. The OpenTelemetry SDK providers (meter, and optionally logger)
    /// 3. If any provider uses ITS mode, a dedicated logs channel for the ITR
    ///
    /// The `admin_reporter` is required for async logging modes (`ConsoleAsync`).
    /// Create the `ObservedStateStore` first and pass its reporter here.
    ///
    /// Returns the system and optionally the ITS logs receiver if any provider uses ITS mode.
    ///
    /// **Note:** The global tracing subscriber is NOT initialized here. Call
    /// `init_global_subscriber()` when ready to start logging.
    pub fn new(
        config: &TelemetryConfig,
        admin_reporter: ObservedEventReporter,
    ) -> Result<(Self, Option<flume::Receiver<event::ObservedEvent>>), Error> {
        // Validate logs config
        config
            .logs
            .validate()
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;

        // 1. Create internal metrics subsystem
        let telemetry_registry = TelemetryRegistryHandle::new();
        let (collector, metrics_reporter) =
            collector::InternalCollector::new(config, telemetry_registry.clone());
        let dispatcher = Arc::new(metrics::dispatcher::MetricsDispatcher::new(
            telemetry_registry.clone(),
            config.reporting_interval,
        ));

        // 2. Create OTel SDK providers
        // OTel Logger is only needed for OpenTelemetry mode
        let otel_client =
            otel_sdk::OpentelemetryClient::new(config, config.logs.providers.uses_otel_provider())?;
        let sdk_meter_provider = otel_client.meter_provider().clone();
        let sdk_logger_provider = otel_client.logger_provider().cloned();
        let otel_runtime = otel_client.into_runtime();

        // 3. Create ITS channel if any provider uses ITS mode
        let (its_reporter, its_receiver) = if config.logs.providers.uses_its_mode() {
            let (sender, receiver) = flume::bounded(config.reporting_channel_size);
            let reporter = ObservedEventReporter::new(SendPolicy::default(), sender);
            (Some(reporter), Some(receiver))
        } else {
            (None, None)
        };

        // 4. Pre-encode resource bytes for internal telemetry (ITR)
        let resource_bytes = otel_sdk::encode_resource_bytes(&config.resource);

        Ok((
            Self {
                registry: telemetry_registry,
                collector: Arc::new(collector),
                metrics_reporter,
                dispatcher,
                sdk_meter_provider,
                sdk_logger_provider,
                _otel_runtime: otel_runtime,
                log_level: config.logs.level,
                provider_modes: config.logs.providers.clone(),
                admin_reporter,
                its_reporter,
                resource_bytes,
            },
            its_receiver,
        ))
    }

    /// Initialize the global tracing subscriber.
    ///
    /// This sets up the global subscriber based on the configured `global` provider mode.
    /// The event reporter passed to `new()` is used internally for async modes.
    pub fn init_global_subscriber(&self) {
        let setup = self.tracing_setup_for(self.provider_modes.global);

        if let Err(err) = setup.try_init_global() {
            raw_error!("tracing.subscriber.init", error = err.to_string());
        }
    }

    /// Returns a `TracingSetup` for the given provider mode.
    ///
    /// This is useful for per-thread subscriber configuration in the engine.
    /// The event reporter is taken from the internal state.
    #[must_use]
    pub fn tracing_setup_for(&self, mode: ProviderMode) -> TracingSetup {
        use tracing_init::ProviderSetup;

        let provider = match mode {
            ProviderMode::Noop => ProviderSetup::Noop,

            ProviderMode::ConsoleDirect => ProviderSetup::ConsoleDirect,

            ProviderMode::ConsoleAsync => ProviderSetup::ConsoleAsync {
                reporter: self.admin_reporter.clone(),
            },

            ProviderMode::OpenTelemetry => {
                let logger = self
                    .sdk_logger_provider
                    .as_ref()
                    .expect("OpenTelemetry mode requires logger_provider");
                ProviderSetup::OpenTelemetry {
                    logger_provider: logger.clone(),
                }
            }

            ProviderMode::ITS => {
                match &self.its_reporter {
                    Some(reporter) => ProviderSetup::ITS {
                        reporter: reporter.clone(),
                    },
                    None => {
                        // ITS mode configured but no reporter available - this shouldn't happen
                        // if uses_its_mode() returned true during construction
                        raw_error!("ITS provider mode configured but no ITS reporter available, falling back to Noop");
                        ProviderSetup::Noop
                    }
                }
            }
        };

        TracingSetup::new(provider, self.log_level)
    }

    /// Returns a `TracingSetup` for engine threads.
    #[must_use]
    pub fn engine_tracing_setup(&self) -> TracingSetup {
        self.tracing_setup_for(self.provider_modes.engine)
    }

    /// Returns a `TracingSetup` for admin threads.
    #[must_use]
    pub fn admin_tracing_setup(&self) -> TracingSetup {
        self.tracing_setup_for(self.provider_modes.admin)
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
    /// TODO: Rename metrics_reporter.
    #[must_use]
    pub fn reporter(&self) -> reporter::MetricsReporter {
        self.metrics_reporter.clone()
    }

    /// Returns a shareable handle to the metrics dispatcher.
    #[must_use]
    pub fn dispatcher(&self) -> Arc<metrics::dispatcher::MetricsDispatcher> {
        self.dispatcher.clone()
    }

    /// Returns the pre-encoded resource bytes for internal telemetry.
    #[must_use]
    pub fn resource_bytes(&self) -> bytes::Bytes {
        self.resource_bytes.clone()
    }

    /// Creates InternalTelemetrySettings from the ITS receiver and resource bytes.
    #[must_use]
    pub fn make_internal_telemetry_settings(
        &self,
        logs_receiver: flume::Receiver<event::ObservedEvent>,
    ) -> InternalTelemetrySettings {
        InternalTelemetrySettings {
            logs_receiver,
            resource_bytes: self.resource_bytes.clone(),
        }
    }

    /// Shuts down the OpenTelemetry SDK providers.
    pub fn shutdown_otel(self) -> Result<(), Error> {
        let meter_shutdown_result = self.sdk_meter_provider.shutdown();
        let logger_shutdown_result = self.sdk_logger_provider.map(|p| p.shutdown()).transpose();

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
        // Dummy channel for testing. Events will be dropped.
        let (sender, _receiver) = flume::bounded(1);
        let config = TelemetryConfig::default();
        let dummy_reporter = ObservedEventReporter::new(SendPolicy::default(), sender);

        let (system, _its_receiver) =
            Self::new(&config, dummy_reporter).expect("default telemetry config should be valid");
        system
    }
}
