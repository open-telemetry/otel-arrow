// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry system used to instrument the OTAP engine. This system currently focuses on metrics
//! but will be extended to cover events and traces.
//!
//! Our instrumentation framework follows a type-safe approach with the goals of being:
//!
//! * less error-prone: everything is encoded in the type system as structs, field names, and
//!   annotations to provide metadata (e.g. unit).
//! * more performant: a collection of metrics is encoded as a struct with fields of counter
//!   type (no hashmap or other dynamic data structures). Several metrics that share the same
//!   attributes don’t have to define those attributes multiple times.
//! * compatible with the semantic conventions: the definition of the metrics produced by the engine
//!   will be available in the semantic convention format.
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
use crate::registry::MetricsRegistryHandle;
use otap_df_config::pipeline::service::telemetry::TelemetryConfig;
use otap_df_config::pipeline::service::telemetry::logs::LogLevel;
use tokio_util::sync::CancellationToken;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub mod attributes;
pub mod collector;
pub mod descriptor;
pub mod error;
pub mod instrument;
/// Internal logs/events module for engine.
pub mod internal_events;
/// Internal logs collection and transport.
pub mod logs;
pub mod metrics;
pub mod opentelemetry_client;
pub mod registry;
pub mod reporter;
pub mod self_tracing;
pub mod semconv;

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

// Re-export commonly used logs types for convenience.
pub use logs::{
    EngineLogsSetup, LogBatch, LogPayload, LogsCollector, LogsFlusher, LogsReceiver, LogsReporter,
    ThreadBufferedLayer, UnbufferedLayer,
};

// TODO This should be #[cfg(test)], but something is preventing it from working.
// The #[cfg(test)]-labeled otap_batch_processor::test_helpers::from_config
// can't load this module unless I remove #[cfg(test)]! See #1304.
pub mod testing;

/// The main telemetry system that aggregates and reports metrics.
pub struct MetricsSystem {
    /// The metrics registry that holds all registered metrics (data + metadata).
    registry: MetricsRegistryHandle,

    /// The process collecting metrics from the pipelines and aggregating them into the registry.
    collector: collector::MetricsCollector,

    /// The process reporting metrics to an external system.
    reporter: reporter::MetricsReporter,

    /// The dispatcher that flushes internal telemetry metrics.
    dispatcher: Arc<metrics::dispatcher::MetricsDispatcher>,
}

impl MetricsSystem {
    /// Creates a new [`MetricsSystem`] initialized with the given configuration.
    #[must_use]
    pub fn new(config: &TelemetryConfig) -> Self {
        let metrics_registry = MetricsRegistryHandle::new();
        let (collector, reporter) =
            collector::MetricsCollector::new(config, metrics_registry.clone());
        let dispatcher = Arc::new(metrics::dispatcher::MetricsDispatcher::new(
            metrics_registry.clone(),
            config.reporting_interval,
        ));
        Self {
            registry: metrics_registry,
            collector,
            reporter,
            dispatcher,
        }
    }

    /// Returns a shareable/cloneable handle to the metrics registry.
    #[must_use]
    pub fn registry(&self) -> MetricsRegistryHandle {
        self.registry.clone()
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

    /// Starts the metrics collection loop and listens for a shutdown signal.
    /// This method returns when either the collection loop ends (Ok/Err) or the shutdown signal fires.
    pub async fn run(self, cancel: CancellationToken) -> Result<(), Error> {
        // Run the collector and race it against the shutdown signal.
        let collector = self.collector;

        tokio::select! {
            res = collector.run_collection_loop() => {
                res
            }
            _ = cancel.cancelled() => {
                // Shutdown requested; cancel the collection loop by dropping its future.
                Ok(())
            }
        }
    }

    /// Runs the metrics collection loop, which collects metrics from the reporting channel
    /// and aggregates them into the registry.
    ///
    /// This method runs indefinitely until the metrics channel is closed.
    /// Returns the pipeline instance when the loop ends (or None if no pipeline was configured).
    pub async fn run_collection_loop(self) -> Result<(), Error> {
        self.collector.run_collection_loop().await
    }
}

impl Default for MetricsSystem {
    fn default() -> Self {
        Self::new(&TelemetryConfig::default())
    }
}

// If RUST_LOG is set, use it for fine-grained control.
// Otherwise, fall back to the config level with some noisy dependencies silenced.
// Users can override by setting RUST_LOG explicitly.
pub(crate) fn get_env_filter(level: LogLevel) -> EnvFilter {
    let level = match level {
        LogLevel::Off => LevelFilter::OFF,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
    };

    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default filter: use config level, but silence known noisy HTTP dependencies
        EnvFilter::new(format!("{level},h2=off,hyper=off"))
    })
}
