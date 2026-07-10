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
use opentelemetry_sdk::metrics::SdkMeterProvider;
use otap_df_config::observed_state::SendPolicy;
use otap_df_config::pipeline::telemetry::TelemetryConfig;
use otap_df_config::settings::telemetry::logs::{LogLevel, LoggingProviders, ProviderMode};
use self_tracing::LogContextFn;
use std::sync::Arc;
use tracing_init::ProviderSetup;

pub mod attributes;
pub mod collector;
pub mod descriptor;
pub mod error;
/// Event types for lifecycle and log events.
pub mod event;
pub mod instrument;
/// Internal logs/events module for engine.
pub mod internal_events;
/// Internal log tap for admin-side log queries.
pub mod log_tap;
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

#[cfg(test)]
#[allow(unsafe_code)] // std::env mutation is synchronized and restored for test isolation.
/// Runs a test closure with `RUST_LOG` temporarily cleared so tracing setup tests
/// are not influenced by the developer's ambient shell configuration.
///
/// The helper serializes access because `RUST_LOG` is process-global and restores
/// the previous value even if the closure panics.
pub(crate) fn with_cleared_rust_log<F, R>(f: F) -> R
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    use std::env;
    use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
    use std::sync::{Mutex, OnceLock};

    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    let _guard = GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let previous = env::var_os("RUST_LOG");
    unsafe {
        env::remove_var("RUST_LOG");
    }

    let result = catch_unwind(AssertUnwindSafe(f));

    match previous {
        Some(value) => unsafe {
            env::set_var("RUST_LOG", value);
        },
        None => unsafe {
            env::remove_var("RUST_LOG");
        },
    }

    match result {
        Ok(value) => value,
        Err(payload) => resume_unwind(payload),
    }
}

#[cfg(test)]
pub(crate) fn ensure_test_crypto_provider() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

// Re-export _private module from internal_events for macro usage.
// This allows the otel_info!, otel_warn!, etc. macros to work in other crates
// without requiring them to add tracing as a direct dependency.
#[doc(hidden)]
pub use internal_events::_private;

// Re-export tracing::Level so callers can use otap_df_telemetry::Level without
// adding tracing as a direct dependency.
pub use tracing::Level;

// Re-export tracing span macros and types for crates that need span instrumentation.
// This allows dependent crates to use spans without adding tracing as a direct dependency.
// Re-exported with otel_ prefix for naming consistency with otel_info!, otel_warn!, etc.
pub use tracing::Span as OtelSpan;
pub use tracing::debug_span as otel_debug_span;
pub use tracing::error_span as otel_error_span;
pub use tracing::info_span as otel_info_span;
pub use tracing::trace_span as otel_trace_span;
pub use tracing::warn_span as otel_warn_span;

/// LogContext is a collection of entity keys.
pub use self_tracing::LogContext;

/// The URN for the internal telemetry receiver.
/// Defined here so it can be used by controller, engine, otap, and other crates.
pub const INTERNAL_TELEMETRY_RECEIVER_URN: &str = "urn:otel:receiver:internal_telemetry";

/// Settings for internal telemetry consumption by the Internal Telemetry Receiver.
///
/// This bundles the receiver end of the logs channel, pre-encoded resource bytes,
/// and a telemetry registry handle for resolving and exporting internal data.
#[derive(Clone)]
pub struct InternalTelemetrySettings {
    /// Receiver end of the logs channel for `ObservedEvent::Log` events.
    pub logs_receiver: flume::Receiver<ObservedEvent>,
    /// Pre-encoded OTLP resource and schema fields shared by logs and metrics.
    pub resource_bytes: bytes::Bytes,
    /// Handle to the telemetry registry for looking up entity attributes.
    pub registry: TelemetryRegistryHandle,
    /// Registry export interval when internal metrics are routed through ITS.
    /// `None` leaves metrics to the configured SDK provider or disables them.
    pub metrics_interval: Option<std::time::Duration>,
    /// Optional retained-log sink shared with admin consumers.
    pub log_tap: Option<log_tap::InternalLogTapHandle>,
}

impl std::fmt::Debug for InternalTelemetrySettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternalTelemetrySettings")
            .field("resource_bytes", &self.resource_bytes)
            .field("registry", &self.registry)
            .field("metrics_interval", &self.metrics_interval)
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
    sdk_meter_provider: Option<SdkMeterProvider>,

    /// Tokio runtime for OTLP exporters (kept alive).
    _otel_runtime: Option<tokio::runtime::Runtime>,

    // === Logging Configuration ===
    /// Log level from config.
    log_level: LogLevel,

    /// The logging providers.
    provider_modes: LoggingProviders,

    /// Entity key providers for associating log events with their source entity context.
    context_fn: LogContextFn,

    /// Event reporter for asynchronous internal logging modes.
    console_async_reporter: Option<ObservedEventReporter>,

    /// Event reporter for ITS mode (Internal Telemetry System).
    its_reporter: Option<ObservedEventReporter>,

    /// Optional handle for querying retained internal logs.
    log_tap_handle: Option<log_tap::InternalLogTapHandle>,

    /// Internal telemetry pipeline setup.
    its_settings: Option<InternalTelemetrySettings>,
}

impl InternalTelemetrySystem {
    /// Creates a new [`InternalTelemetrySystem`] initialized with the given configuration.
    ///
    /// Depending on logging provider mode choices, multiple telemetry backends can be
    /// initialized:
    ///
    /// OpenTelemetry: the OTel metrics SDK is created only when selected by
    /// `engine.telemetry.metrics.provider`.
    ///
    /// ConsoleAsync: the ObservedEventReporer is passed in, having been created for
    /// use by the admin component unconditionally, to support the ConsoleAsync mode.
    ///
    /// ITS: if logs or metrics select the internal telemetry system, settings
    /// for an internal telemetry receiver are returned.
    ///
    /// **Note:** The global tracing subscriber is NOT initialized here. Call
    /// `init_global_subscriber()` when ready to start logging.
    pub fn new(
        config: &TelemetryConfig,
        telemetry_registry: TelemetryRegistryHandle,
        console_async_reporter: Option<ObservedEventReporter>,
        logging_send_policy: SendPolicy,
        context_fn: LogContextFn,
        log_tap_handle: Option<log_tap::InternalLogTapHandle>,
    ) -> Result<Self, Error> {
        // Validate signal providers and SDK-specific configuration.
        config
            .validate()
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;

        // 1. Create internal metrics subsystem
        let (collector, metrics_reporter) =
            collector::InternalCollector::new(config, telemetry_registry.clone());
        let dispatcher = Arc::new(metrics::dispatcher::MetricsDispatcher::new(
            telemetry_registry.clone(),
            config.reporting_interval,
        ));

        // 2. Create the OTel metrics SDK only when explicitly selected.
        let (sdk_meter_provider, otel_runtime) = if config.metrics.uses_opentelemetry_provider() {
            let otel_client = otel_sdk::OpentelemetryClient::new(config)?;
            let sdk_meter_provider = otel_client.meter_provider().clone();
            (Some(sdk_meter_provider), otel_client.into_runtime())
        } else {
            (None, None)
        };

        // 3. Create ITS transport when either logs or metrics use ITS. The
        // reporter is retained in metrics-only mode to keep the channel open.
        let (its_reporter, its_settings) = if config.uses_its_provider() {
            let (sender, logs_receiver) = flume::bounded(config.reporting_channel_size);
            let reporter = if let Some(log_tap) = &log_tap_handle {
                ObservedEventReporter::new(logging_send_policy.clone(), sender)
                    .with_drop_counter(log_tap.ingest_drop_counter())
            } else {
                ObservedEventReporter::new(logging_send_policy.clone(), sender)
            };
            let resource_bytes = otel_sdk::encode_resource_bytes(&config.resource);
            (
                Some(reporter),
                Some(InternalTelemetrySettings {
                    logs_receiver,
                    resource_bytes,
                    registry: telemetry_registry.clone(),
                    metrics_interval: config
                        .metrics
                        .uses_its_provider()
                        .then_some(config.reporting_interval),
                    log_tap: log_tap_handle.clone(),
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
            _otel_runtime: otel_runtime,
            log_level: config.logs.level.clone(),
            provider_modes: config.logs.providers.clone(),
            context_fn,
            console_async_reporter,
            its_reporter,
            log_tap_handle,
            its_settings,
        })
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
    fn tracing_setup_for(&self, mode: ProviderMode) -> TracingSetup {
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
        };

        TracingSetup::new(provider, self.log_level.clone(), self.context_fn)
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

    /// Returns a shareable handle to the internal log tap, if enabled.
    #[must_use]
    pub fn log_tap_handle(&self) -> Option<log_tap::InternalLogTapHandle> {
        self.log_tap_handle.clone()
    }

    /// Test-only accessor for the ITS reporter, used to verify that the
    /// configured logging send policy is threaded through during construction.
    #[cfg(test)]
    pub(crate) fn its_reporter(&self) -> Option<&ObservedEventReporter> {
        self.its_reporter.as_ref()
    }

    /// Returns the configured log level.
    #[must_use]
    pub const fn log_level(&self) -> &LogLevel {
        &self.log_level
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
        if let Some(meter_provider) = self.sdk_meter_provider
            && let Err(e) = meter_provider.shutdown()
        {
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

        Self::new(
            &config,
            TelemetryRegistryHandle::new(),
            Some(dummy_reporter),
            SendPolicy::default(),
            LogContext::new,
            None,
        )
        .expect("default telemetry config should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::pipeline::telemetry::{
        AttributeValue::I64 as OTelI64, AttributeValue::String as OTelString,
        metrics::MetricsProvider,
    };
    use otap_df_config::settings::telemetry::logs::{
        InternalLogTapConfig, LoggingProviders, LogsConfig, ProviderMode,
    };
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::{v1::LogsData, v1::ResourceLogs};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use prost::Message;
    use std::time::Duration;

    fn test_reporter() -> ObservedEventReporter {
        let (sender, _receiver) = flume::bounded(16);
        ObservedEventReporter::new(SendPolicy::default(), sender)
    }

    fn config_with_providers(providers: LoggingProviders) -> TelemetryConfig {
        TelemetryConfig {
            logs: LogsConfig {
                providers,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn test_system(config: &TelemetryConfig) -> InternalTelemetrySystem {
        InternalTelemetrySystem::new(
            config,
            TelemetryRegistryHandle::new(),
            Some(test_reporter()),
            SendPolicy::default(),
            LogContext::new,
            None,
        )
        .expect("should create internal telemetry system")
    }

    #[test]
    fn default_metrics_provider_uses_sdk_without_its_settings() {
        let its = test_system(&TelemetryConfig::default());

        assert!(its.sdk_meter_provider.is_some());
        assert!(its.internal_telemetry_settings().is_none());
        its.shutdown_otel().expect("SDK shutdown should succeed");
    }

    #[test]
    fn its_metrics_provider_uses_internal_settings_without_sdk() {
        let reporting_interval = Duration::from_millis(137);
        let mut config = TelemetryConfig {
            reporting_interval,
            ..TelemetryConfig::default()
        };
        config.metrics.provider = MetricsProvider::Its;

        let its = test_system(&config);
        let settings = its
            .internal_telemetry_settings()
            .expect("ITS metrics should provide receiver settings");

        assert!(its.sdk_meter_provider.is_none());
        assert_eq!(settings.metrics_interval, Some(reporting_interval));
        assert!(
            its.its_reporter().is_some(),
            "metrics-only ITS must retain its sender"
        );
        its.shutdown_otel()
            .expect("shutdown without an SDK should succeed");
    }

    #[test]
    fn its_logs_with_opentelemetry_metrics_keep_sdk_without_metric_ticks() {
        let providers = LoggingProviders {
            global: ProviderMode::Noop,
            engine: ProviderMode::ITS,
            internal: ProviderMode::Noop,
            admin: ProviderMode::Noop,
        };
        let its = test_system(&config_with_providers(providers));
        let settings = its
            .internal_telemetry_settings()
            .expect("ITS logs should provide receiver settings");

        assert!(its.sdk_meter_provider.is_some());
        assert_eq!(settings.metrics_interval, None);
        its.shutdown_otel().expect("SDK shutdown should succeed");
    }

    #[test]
    fn disabled_metrics_provider_has_no_sdk_or_its_settings() {
        let mut config = TelemetryConfig::default();
        config.metrics.provider = MetricsProvider::None;

        let its = test_system(&config);

        assert!(its.sdk_meter_provider.is_none());
        assert!(its.internal_telemetry_settings().is_none());
        its.shutdown_otel()
            .expect("shutdown without an SDK should succeed");
    }

    #[test]
    fn its_receiver_presence_depends_on_provider_mode() {
        with_cleared_rust_log(|| {
            // Default (no ITS) -> no receiver
            let its = InternalTelemetrySystem::new(
                &TelemetryConfig::default(),
                TelemetryRegistryHandle::new(),
                Some(test_reporter()),
                SendPolicy::default(),
                LogContext::new,
                None,
            )
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
            let its = InternalTelemetrySystem::new(
                &config_with_providers(providers),
                TelemetryRegistryHandle::new(),
                Some(test_reporter()),
                SendPolicy::default(),
                LogContext::new,
                None,
            )
            .expect("should create");
            let its_settings = its.internal_telemetry_settings();
            let rx = its_settings
                .expect("ITS mode should provide receiver")
                .logs_receiver;
            assert!(rx.is_empty(), "receiver starts empty");

            // Emit a log using the engine tracing setup (which uses ITS)
            its.engine_tracing_setup().with_subscriber_ignoring_env(|| {
                crate::otel_info!("telemetry.test_log", message = "test log message");
            });

            // Receiver should have the log
            let recv = rx
                .recv_timeout(Duration::from_secs(1))
                .expect("receiver should have log after emit");
            assert!(matches!(recv, ObservedEvent::Log(_)));
            let text = recv.to_string();
            assert!(text.contains("test log message"), "log message is {}", text);
        });
    }

    #[test]
    fn its_reporter_honors_configured_logging_send_policy() {
        // Verifies the fix that threads `engine.observed_state.logging_events`
        // through to the ITS reporter, covering both code paths:
        //   1. log_tap_handle: None
        //   2. log_tap_handle: Some(_)  (sets the drop counter on the reporter)
        //
        // Before the fix, both paths constructed the ITS reporter with
        // `SendPolicy::default()` (which has `console_fallback: true`),
        // so a user-provided `console_fallback: false` was silently ignored.
        with_cleared_rust_log(|| {
            let providers = LoggingProviders {
                global: ProviderMode::Noop,
                engine: ProviderMode::ITS,
                internal: ProviderMode::Noop,
                admin: ProviderMode::Noop,
            };
            let custom_policy = SendPolicy {
                blocking_timeout: Some(Duration::from_millis(7)),
                console_fallback: false,
            };

            // Case 1: no log tap handle.
            let its = InternalTelemetrySystem::new(
                &config_with_providers(providers.clone()),
                TelemetryRegistryHandle::new(),
                Some(test_reporter()),
                custom_policy.clone(),
                LogContext::new,
                None,
            )
            .expect("should create");
            let policy = its
                .its_reporter()
                .expect("ITS provider configured")
                .policy();
            assert_eq!(
                policy, &custom_policy,
                "ITS reporter (no log tap) must use configured logging_events policy"
            );

            // Case 2: with a log tap handle (reporter additionally gets the drop counter).
            let log_tap = log_tap::build(&InternalLogTapConfig {
                enabled: true,
                max_entries: 1,
                max_bytes: usize::MAX,
            });
            let its = InternalTelemetrySystem::new(
                &config_with_providers(providers),
                TelemetryRegistryHandle::new(),
                Some(test_reporter()),
                custom_policy.clone(),
                LogContext::new,
                Some(log_tap),
            )
            .expect("should create");
            let policy = its
                .its_reporter()
                .expect("ITS provider configured")
                .policy();
            assert_eq!(
                policy, &custom_policy,
                "ITS reporter (with log tap) must use configured logging_events policy"
            );
        });
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

        let its = InternalTelemetrySystem::new(
            &config,
            TelemetryRegistryHandle::new(),
            Some(test_reporter()),
            SendPolicy::default(),
            LogContext::new,
            None,
        )
        .expect("should create");

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
