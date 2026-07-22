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
//! * Native OTAP metric-set representation

use crate::error::Error;
use crate::event::{ObservedEvent, ObservedEventReporter};
use crate::registry::TelemetryRegistryHandle;
use otap_df_config::observed_state::SendPolicy;
use otap_df_config::pipeline::telemetry::TelemetryConfig;
use otap_df_config::settings::telemetry::logs::{LogLevel, LoggingProviders, ProviderMode};
use self_tracing::LogContextFn;
use std::sync::Arc;
use tracing_init::ProviderSetup;

pub mod attributes;
pub mod collector;
/// Reusable enum attributes for internal telemetry.
pub mod common_attributes;
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
pub use otap_df_config::engine::INTERNAL_TELEMETRY_RECEIVER_URN;

/// Settings for internal telemetry consumption by the Internal Telemetry Receiver.
///
/// This bundles the receiver end of the logs channel, a pre-encoded OTLP
/// resource field, and a telemetry registry handle for exporting internal data.
#[derive(Clone)]
pub struct InternalTelemetrySettings {
    /// Receiver end of the logs channel for `ObservedEvent::Log` events.
    pub logs_receiver: flume::Receiver<ObservedEvent>,
    /// Pre-encoded OTLP `resource` field shared by logs and metrics.
    pub resource_field_bytes: bytes::Bytes,
    /// Handle to the telemetry registry for looking up entity attributes.
    pub registry: TelemetryRegistryHandle,
    /// Default registry drain interval. The internal telemetry receiver may
    /// override this cold-path drain and export interval in its node configuration.
    pub default_metric_drain_interval: std::time::Duration,
    /// Optional retained-log sink shared with admin consumers.
    pub log_tap: Option<log_tap::InternalLogTapHandle>,
}

impl std::fmt::Debug for InternalTelemetrySettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InternalTelemetrySettings")
            .field("resource_field_bytes", &self.resource_field_bytes)
            .field("registry", &self.registry)
            .field(
                "default_metric_drain_interval",
                &self.default_metric_drain_interval,
            )
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
/// - Tokio tracing subscriber configuration
pub struct InternalTelemetrySystem {
    // === Internal Metrics Subsystem ===
    /// The telemetry registry that holds all registered entities and metrics (data + metadata).
    registry: TelemetryRegistryHandle,

    /// The process collecting and processing internal signals.
    collector: Arc<collector::InternalCollector>,

    /// The process reporting metrics to an external system.
    metrics_reporter: reporter::MetricsReporter,

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
    its_reporter: ObservedEventReporter,

    /// Optional handle for querying retained internal logs.
    log_tap_handle: Option<log_tap::InternalLogTapHandle>,

    /// Backend settings consumed by the internal telemetry receiver.
    internal_telemetry_settings: InternalTelemetrySettings,
}

impl InternalTelemetrySystem {
    /// Creates a new [`InternalTelemetrySystem`] initialized with the given configuration.
    ///
    /// Depending on logging provider mode choices, multiple telemetry backends can be
    /// initialized:
    ///
    /// ConsoleAsync: the `ObservedEventReporter` is passed in, having been created for
    /// use by the admin component unconditionally, to support the ConsoleAsync mode.
    ///
    /// ITS: settings for the always-running internal telemetry receiver are
    /// returned. Log providers independently select whether logs use ITS.
    ///
    /// **Note:** The global tracing subscriber is NOT initialized here. Call
    /// `init_global_subscriber()` when ready to start logging.
    pub fn new(
        config: &TelemetryConfig,
        default_metric_drain_interval: std::time::Duration,
        telemetry_registry: TelemetryRegistryHandle,
        console_async_reporter: Option<ObservedEventReporter>,
        logging_send_policy: SendPolicy,
        context_fn: LogContextFn,
        log_tap_handle: Option<log_tap::InternalLogTapHandle>,
    ) -> Result<Self, Error> {
        // Validate internal telemetry configuration.
        config
            .validate()
            .map_err(|e| Error::ConfigurationError(e.to_string()))?;

        // 1. Create internal metrics subsystem
        let (collector, metrics_reporter) =
            collector::InternalCollector::new(config, telemetry_registry.clone());
        let collector = Arc::new(collector);

        // 2. Always create the ITS transport. Metrics use it unconditionally;
        // configured log providers decide whether they send into the log side.
        let (its_reporter, internal_telemetry_settings) = {
            let (sender, logs_receiver) = flume::bounded(config.reporting_channel_size);
            let reporter = if let Some(log_tap) = &log_tap_handle {
                ObservedEventReporter::new(logging_send_policy.clone(), sender)
                    .with_drop_counter(log_tap.ingest_drop_counter())
            } else {
                ObservedEventReporter::new(logging_send_policy.clone(), sender)
            };
            let resource_field_bytes =
                self_tracing::encoder::encode_config_resource_field(&config.resource);
            (
                reporter,
                InternalTelemetrySettings {
                    logs_receiver,
                    resource_field_bytes,
                    registry: telemetry_registry.clone(),
                    default_metric_drain_interval,
                    log_tap: log_tap_handle.clone(),
                },
            )
        };

        Ok(Self {
            registry: telemetry_registry,
            collector,
            metrics_reporter,
            log_level: config.logs.level.clone(),
            provider_modes: config.logs.providers.clone(),
            context_fn,
            console_async_reporter,
            its_reporter,
            log_tap_handle,
            internal_telemetry_settings,
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
                reporter: self.its_reporter.clone(),
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

    /// Returns a `TracingSetup` for engine observability pipeline threads.
    ///
    /// This defaults to `Noop` to avoid feedback loops where logs from
    /// the engine observability pipeline would be sent back to itself.
    #[must_use]
    pub fn internal_tracing_setup(&self) -> TracingSetup {
        self.tracing_setup_for(self.provider_modes.internal)
    }

    /// Returns the internal telemetry receiver's backend settings.
    #[must_use]
    pub fn internal_telemetry_settings(&self) -> InternalTelemetrySettings {
        self.internal_telemetry_settings.clone()
    }

    /// Returns a shareable handle to the internal log tap, if enabled.
    #[must_use]
    pub fn log_tap_handle(&self) -> Option<log_tap::InternalLogTapHandle> {
        self.log_tap_handle.clone()
    }

    /// Test-only accessor for the ITS reporter, used to verify that the
    /// configured logging send policy is threaded through during construction.
    #[cfg(test)]
    pub(crate) const fn its_reporter(&self) -> &ObservedEventReporter {
        &self.its_reporter
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
}

impl Default for InternalTelemetrySystem {
    fn default() -> Self {
        // Dummy channel for testing. Events will be dropped.
        let (sender, _receiver) = flume::bounded(1);
        let config = TelemetryConfig::default();
        let dummy_reporter = ObservedEventReporter::new(SendPolicy::default(), sender);

        Self::new(
            &config,
            config.reporting_interval,
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
    use otap_df_config::pipeline::telemetry::AttributeValue as ConfigAttributeValue;
    use otap_df_config::settings::telemetry::logs::{
        InternalLogTapConfig, LoggingProviders, LogsConfig, ProviderMode,
    };
    use otap_df_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
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
            config.reporting_interval,
            TelemetryRegistryHandle::new(),
            Some(test_reporter()),
            SendPolicy::default(),
            LogContext::new,
            None,
        )
        .expect("should create internal telemetry system")
    }

    /// Scenario: all log providers explicitly disable ITS delivery.
    /// Guarantees: the receiver transport and configured metric drain interval remain available.
    #[test]
    fn metrics_transport_does_not_depend_on_log_provider_modes() {
        let providers = LoggingProviders {
            global: ProviderMode::Noop,
            engine: ProviderMode::Noop,
            internal: ProviderMode::Noop,
            admin: ProviderMode::Noop,
        };
        let mut config = config_with_providers(providers);
        config.reporting_interval = Duration::from_millis(137);
        let its = test_system(&config);
        let settings = its.internal_telemetry_settings();

        assert_eq!(
            settings.default_metric_drain_interval,
            config.reporting_interval
        );
        assert!(
            !settings.logs_receiver.is_disconnected(),
            "the receiver transport must remain connected"
        );
    }

    /// Scenario: engine logging switches between noop and ITS provider modes.
    /// Guarantees: ITS log delivery follows the provider while metrics transport stays present.
    #[test]
    fn its_log_delivery_depends_on_provider_mode() {
        with_cleared_rust_log(|| {
            // Explicitly disabling ITS logs still retains metrics settings.
            let no_its_providers = LoggingProviders {
                global: ProviderMode::Noop,
                engine: ProviderMode::Noop,
                internal: ProviderMode::Noop,
                admin: ProviderMode::Noop,
            };
            let its = InternalTelemetrySystem::new(
                &config_with_providers(no_its_providers),
                Duration::from_secs(1),
                TelemetryRegistryHandle::new(),
                Some(test_reporter()),
                SendPolicy::default(),
                LogContext::new,
                None,
            )
            .expect("should create");
            let rx = its.internal_telemetry_settings().logs_receiver;
            its.engine_tracing_setup().with_subscriber_ignoring_env(|| {
                crate::otel_info!("telemetry.noop_log", message = "noop log message");
            });
            assert!(rx.is_empty(), "noop provider must not deliver logs");

            // ITS mode on engine -> receiver present and receives logs
            let providers = LoggingProviders {
                global: ProviderMode::Noop,
                engine: ProviderMode::ITS,
                internal: ProviderMode::Noop,
                admin: ProviderMode::Noop,
            };
            let its = InternalTelemetrySystem::new(
                &config_with_providers(providers),
                Duration::from_secs(1),
                TelemetryRegistryHandle::new(),
                Some(test_reporter()),
                SendPolicy::default(),
                LogContext::new,
                None,
            )
            .expect("should create");
            let settings = its.internal_telemetry_settings();
            let rx = settings.logs_receiver;
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
                Duration::from_secs(1),
                TelemetryRegistryHandle::new(),
                Some(test_reporter()),
                custom_policy.clone(),
                LogContext::new,
                None,
            )
            .expect("should create");
            let policy = its.its_reporter().policy();
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
                Duration::from_secs(1),
                TelemetryRegistryHandle::new(),
                Some(test_reporter()),
                custom_policy.clone(),
                LogContext::new,
                Some(log_tap),
            )
            .expect("should create");
            let policy = its.its_reporter().policy();
            assert_eq!(
                policy, &custom_policy,
                "ITS reporter (with log tap) must use configured logging_events policy"
            );
        });
    }

    /// Scenario: telemetry construction receives configured resource attributes.
    /// Guarantees: settings contain a reusable OTLP resource-field fragment.
    #[test]
    fn internal_telemetry_settings_include_resource_field() {
        let mut config = TelemetryConfig::default();
        let _ = config.resource.insert(
            "service.name".to_string(),
            ConfigAttributeValue::String("my-test-service".into()),
        );

        let its = InternalTelemetrySystem::new(
            &config,
            config.reporting_interval,
            TelemetryRegistryHandle::new(),
            Some(test_reporter()),
            SendPolicy::default(),
            LogContext::new,
            None,
        )
        .expect("should create");

        let settings = its.internal_telemetry_settings();
        let resource_logs = ResourceLogs::decode(settings.resource_field_bytes)
            .expect("decode OTLP resource field");
        let attributes = resource_logs
            .resource
            .expect("resource field should be present")
            .attributes;

        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key, "service.name");
        assert_eq!(
            attributes[0]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(
                &otap_df_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                    "my-test-service".to_owned()
                )
            )
        );
        assert!(resource_logs.scope_logs.is_empty());
    }
}
