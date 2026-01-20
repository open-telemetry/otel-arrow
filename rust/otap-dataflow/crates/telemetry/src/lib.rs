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
use crate::registry::TelemetryRegistryHandle;
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
/// - Tokio tracing subscriber configuration
pub struct InternalTelemetrySystem {
    // === Internal Metrics Subsystem ===
    /// The telemetry registry that holds all registered entities and metrics (data + metadata).
    registry: TelemetryRegistryHandle,

    /// The process collecting and processing internal signals.
    collector: Arc<collector::InternalCollector>,

    /// The process reporting metrics to an external system.
    reporter: reporter::MetricsReporter,

    /// The dispatcher that flushes internal telemetry metrics.
    dispatcher: Arc<metrics::dispatcher::MetricsDispatcher>,

    // === OTel SDK Subsystem ===
    /// OTel SDK meter provider for metrics export.
    meter_provider: SdkMeterProvider,

    /// OTel SDK logger provider for logs export.
    logger_provider: SdkLoggerProvider,

    /// Tokio runtime for OTLP exporters (kept alive).
    _otel_runtime: Option<tokio::runtime::Runtime>,
}

impl InternalTelemetrySystem {
    /// Creates a new [`InternalTelemetrySystem`] initialized with the given configuration.
    ///
    /// This creates:
    /// 1. The internal metrics subsystem (registry, collector, reporter, dispatcher)
    /// 2. The OpenTelemetry SDK providers (meter and logger)
    /// 3. Initializes the global tracing subscriber
    pub fn new(config: &TelemetryConfig) -> Result<Self, Error> {
        // 1. Create internal metrics subsystem
        let telemetry_registry = TelemetryRegistryHandle::new();
        let (collector, reporter) =
            collector::InternalCollector::new(config, telemetry_registry.clone());
        let dispatcher = Arc::new(metrics::dispatcher::MetricsDispatcher::new(
            telemetry_registry.clone(),
            config.reporting_interval,
        ));

        // 2. Create OTel SDK providers
        let otel_client = otel_sdk::OpentelemetryClient::new(config)?;
        let meter_provider = otel_client.meter_provider().clone();
        let logger_provider = otel_client.logger_provider().clone();
        let otel_runtime = otel_client.into_runtime();

        // 3. Initialize global tracing subscriber
        tracing_init::init_global_subscriber(config.logs.level, &logger_provider);

        Ok(Self {
            registry: telemetry_registry,
            collector: Arc::new(collector),
            reporter,
            dispatcher,
            meter_provider,
            logger_provider,
            _otel_runtime: otel_runtime,
        })
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
    pub fn reporter(&self) -> reporter::MetricsReporter {
        self.reporter.clone()
    }

    /// Returns a shareable handle to the metrics dispatcher.
    #[must_use]
    pub fn dispatcher(&self) -> Arc<metrics::dispatcher::MetricsDispatcher> {
        self.dispatcher.clone()
    }

    /// Shuts down the OpenTelemetry SDK providers.
    pub fn shutdown(self) -> Result<(), Error> {
        let meter_shutdown_result = self.meter_provider.shutdown();
        let logger_shutdown_result = self.logger_provider.shutdown();

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
        Self::new(&TelemetryConfig::default()).expect("default telemetry config should be valid")
    }
}
