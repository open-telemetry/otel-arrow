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

use crate::error::Error;
use crate::event::{ObservedEvent, ObservedEventReporter};
use crate::registry::TelemetryRegistryHandle;
use opentelemetry_sdk::{logs::SdkLoggerProvider, metrics::SdkMeterProvider};
use otap_df_config::observed_state::SendPolicy;
use otap_df_config::pipeline::service::telemetry::TelemetryConfig;
use otap_df_config::pipeline::service::telemetry::logs::{
    LogLevel, LoggingProviders, ProviderMode,
};
use std::sync::Arc;
use tracing_init::ProviderSetup;
use crate::tracing_init::empty_log_context;

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
pub use tracing_init::{TracingSetup, LogContextFn};

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
pub const INTERNAL_TELEMETRY_RECEIVER_URN: &str = "urn:otel:internal_telemetry:receiver";

/// Settings for internal telemetry consumption by the Internal Telemetry Receiver.
///
/// This bundles the receiver end of the logs channel and pre-encoded resource bytes
/// for injection into the ITR via the EffectHandler.
#[derive(Clone)]
pub struct InternalTelemetrySettings {
    /// Receiver end of the logs channel for `ObservedEvent::Log` events.
    pub logs_receiver: flume::Receiver<ObservedEvent>,
    /// Pre-encoded OTLP resource bytes (ResourceLogs.resource + schema_url fields).
    pub resource_bytes: bytes::Bytes,
}

impl std::fmt::Debug for InternalTelemetrySettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternalTelemetrySettings")
            .field("resource_bytes", &self.resource_bytes)
            .finish_non_exhaustive()
    }
}

// TODO This should be #[cfg(test)], but something is preventing it from working.
// The #[cfg(test)]-labeled otap_batch_processor::test_helpers::from_config
// can't load this module unless I remove #[cfg(test)]! See #1304.
pub mod entity;
pub mod testing;

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

    /// Event reporter for asynchronous internal logging modes.

    /// Entity key providers for associating events with context.
    context_fn: LogContextFn,

    /// Event reporter for ConsoleAsync mode (Internal Telemetry System).
    console_async_reporter: Option<ObservedEventReporter>,

    /// Event reporter for ITS mode (Internal Telemetry System).
    its_reporter: Option<ObservedEventReporter>,

    /// Internal telemetry pipeline setup.
    its_settings: Option<InternalTelemetrySettings>,
}

impl InternalTelemetrySystem {
    /// Creates a new [`InternalTelemetrySystem`] initialized with the given configuration.
    ///
    /// Depending on logging provider mode choices, multiple telemetry backends can be
    /// initialized:
    ///
    /// OpenTelemetry: the OTel logging provider is created if any
    /// service::telemetry::logs::providers uses this choice.  Note: the OTel meter
    /// provider is created unconditionally.
    ///
    /// ConsoleAsync: the ObservedEventReporer is passed in, having been created for
    /// use by the admin component unconditionally, to support the ConsoleAsync mode.
    ///
    /// ITS: if any logging provider is configured with for the internal telemetry system,
    /// an InternalReceiver will be returned.
    ///
    /// **Note:** The global tracing subscriber is NOT initialized here. Call
    /// `init_global_subscriber()` when ready to start logging.
    pub fn new(
        config: &TelemetryConfig,
        console_async_reporter: Option<ObservedEventReporter>,
        context_fn: LogContextFn,
    ) -> Result<Self, Error> {
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
        let (its_reporter, its_settings) = if config.logs.providers.uses_its_provider() {
            let (sender, logs_receiver) = flume::bounded(config.reporting_channel_size);
            let reporter = ObservedEventReporter::new(SendPolicy::default(), sender);
            let resource_bytes = otel_sdk::encode_resource_bytes(&config.resource);
            (
                Some(reporter),
                Some(InternalTelemetrySettings {
                    logs_receiver,
                    resource_bytes,
                }),
            )
        } else {
            (None, None)
        };

        Ok(Self {
            registry: telemetry_registry,
            collector: Arc::new(collector),
            metrics_reporter,
            dispatcher,
            sdk_meter_provider,
            sdk_logger_provider,
            _otel_runtime: otel_runtime,
            log_level: config.logs.level,
            provider_modes: config.logs.providers.clone(),
            context_fn,
            console_async_reporter,
            its_reporter,
            its_settings,
        })
    }

    /// Initialize the global tracing subscriber.
    ///
    /// This sets up the global subscriber based on the configured `global` provider mode.
    /// The event reporter passed to `new()` is used internally for async modes.
    /// If entity providers have been set, they will be used to associate logs with context.
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
        let provider = match mode {
            ProviderMode::Noop => ProviderSetup::Noop,

            ProviderMode::ConsoleDirect => ProviderSetup::ConsoleDirect,

            ProviderMode::ConsoleAsync => ProviderSetup::InternalAsync {
                reporter: self
                    .console_async_reporter
                    .as_ref()
                    .expect("has provider")
                    .clone(),
            },

            ProviderMode::ITS => ProviderSetup::InternalAsync {
                reporter: self.its_reporter.as_ref().expect("has provider").clone(),
            },

            ProviderMode::OpenTelemetry => {
                let logger = self.sdk_logger_provider.as_ref().expect("has provider");
                ProviderSetup::OpenTelemetry {
                    logger_provider: logger.clone(),
                }
            }
        };

        TracingSetup::new(provider, self.log_level, self.context_fn)
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

    /// Returns a `TracingSetup` for internal telemetry pipeline threads.
    ///
    /// This defaults to `Noop` to avoid feedback loops where logs from
    /// the internal pipeline would be sent back to itself.
    #[must_use]
    pub fn internal_tracing_setup(&self) -> TracingSetup {
        self.tracing_setup_for(self.provider_modes.internal)
    }

    /// Ihe internal telemetry pipeline backend setup.
    #[must_use]
    pub fn internal_telemetry_settings(&self) -> Option<InternalTelemetrySettings> {
        self.its_settings.clone()
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

        Self::new(&config, Some(dummy_reporter), empty_log_context).expect("default telemetry config should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::pipeline::service::telemetry::{
        AttributeValue::I64 as OTelI64,
        AttributeValue::String as OTelString,
        logs::{LoggingProviders, ProviderMode},
    };
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::{v1::LogsData, v1::ResourceLogs};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use prost::Message;

    fn test_reporter() -> ObservedEventReporter {
        let (sender, _receiver) = flume::bounded(16);
        ObservedEventReporter::new(SendPolicy::default(), sender)
    }

    fn config_with_providers(providers: LoggingProviders) -> TelemetryConfig {
        TelemetryConfig {
            logs: otap_df_config::pipeline::service::telemetry::logs::LogsConfig {
                providers,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn its_receiver_presence_depends_on_provider_mode() {
        // Default (no ITS) -> no receiver
        let its = InternalTelemetrySystem::new(&TelemetryConfig::default(), Some(test_reporter()))
            .expect("should create");
        assert!(
            its.internal_telemetry_settings().is_none(),
            "no ITS mode -> no receiver"
        );

        // ITS mode on engine -> receiver present and receives logs
        let providers = LoggingProviders {
            global: ProviderMode::Noop,
            engine: ProviderMode::ITS,
            internal: ProviderMode::Noop,
            admin: ProviderMode::Noop,
        };
        let its =
            InternalTelemetrySystem::new(&config_with_providers(providers), Some(test_reporter()))
                .expect("should create");
        let its_settings = its.internal_telemetry_settings();
        let rx = its_settings
            .expect("ITS mode should provide receiver")
            .logs_receiver;
        assert!(rx.is_empty(), "receiver starts empty");

        // Emit a log using the engine tracing setup (which uses ITS)
        its.engine_tracing_setup().with_subscriber(|| {
            crate::otel_info!("test log message");
        });

        // Receiver should have the log
        let recv = rx.recv().expect("receiver should have log after emit");
        assert!(matches!(recv, ObservedEvent::Log(_)));
        let text = recv.to_string();
        assert!(text.contains("test log message"), "log message is {}", text);
    }

    #[test]
    fn resource_bytes() {
        let mut config = TelemetryConfig::default();
        let _ = config.resource.insert(
            "service.name".to_string(),
            OTelString("my-test-service".into()),
        );
        let _ = config
            .resource
            .insert("service.id".to_string(), OTelI64(1234));
        config.logs.providers.global = ProviderMode::ITS;

        let its =
            InternalTelemetrySystem::new(&config, Some(test_reporter())).expect("should create");

        let settings = its.internal_telemetry_settings().expect("has ITS");
        let parse =
            ResourceLogs::decode(settings.resource_bytes).expect("decode OTLP resource bytes");

        // The encoding is a fragment of ResourceLogs with just the Resource field set
        assert_equivalent(
            &[OtlpProtoMessage::Logs(LogsData::new([parse]))],
            &[OtlpProtoMessage::Logs(LogsData::new([ResourceLogs::new(
                Resource::build()
                    .attributes([
                        KeyValue::new("service.name", AnyValue::new_string("my-test-service")),
                        KeyValue::new("service.id", AnyValue::new_int(1234)),
                    ])
                    .finish(),
                [],
            )]))],
        );
    }
}
