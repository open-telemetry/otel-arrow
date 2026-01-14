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
pub mod registry;
pub mod reporter;
pub mod self_tracing;
pub mod semconv;
pub mod telemetry_runtime;

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
    ImmediateLayer, LogBatch, LogPayload, LogsCollector, LogsReceiver, LogsReporter, TelemetrySetup,
};

// TODO This should be #[cfg(test)], but something is preventing it from working.
// The #[cfg(test)]-labeled otap_batch_processor::test_helpers::from_config
// can't load this module unless I remove #[cfg(test)]! See #1304.
pub mod entity;
pub mod testing;

/// The internal telemetry system that registers, collects, and reports internal signals.
pub struct InternalTelemetrySystem {
    /// The telemetry registry that holds all registered entities and metrics (data + metadata).
    registry: TelemetryRegistryHandle,

    /// The process collecting and processing internal signals.
    collector: collector::InternalCollector,

    /// The process reporting metrics to an external system.
    reporter: reporter::MetricsReporter,

    /// The dispatcher that flushes internal telemetry metrics.
    dispatcher: Arc<metrics::dispatcher::MetricsDispatcher>,
}

impl InternalTelemetrySystem {
    /// Creates a new [`InternalTelemetrySystem`] initialized with the given configuration.
    #[must_use]
    pub fn new(config: &TelemetryConfig) -> Self {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let (collector, reporter) =
            collector::InternalCollector::new(config, telemetry_registry.clone());
        let dispatcher = Arc::new(metrics::dispatcher::MetricsDispatcher::new(
            telemetry_registry.clone(),
            config.reporting_interval,
        ));
        Self {
            registry: telemetry_registry,
            collector,
            reporter,
            dispatcher,
        }
    }

    /// Returns a shareable/cloneable handle to the telemetry registry.
    #[must_use]
    pub fn registry(&self) -> TelemetryRegistryHandle {
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

    /// Starts the internal signal collection loop and listens for a shutdown signal.
    /// This method returns when either the collection loop ends (Ok/Err) or the shutdown signal
    /// fires.
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

    /// Runs the internal signal collection loop, which collects signals from the reporting channel
    /// and aggregates them into the registry.
    ///
    /// This method runs indefinitely until the signals channel is closed.
    /// Returns the pipeline instance when the loop ends (or None if no pipeline was configured).
    pub async fn run_collection_loop(self) -> Result<(), Error> {
        self.collector.run_collection_loop().await
    }
}

impl Default for InternalTelemetrySystem {
    fn default() -> Self {
        Self::new(&TelemetryConfig::default())
    }
}

/// Creates an `EnvFilter` for the given log level.
///
/// If `RUST_LOG` is set in the environment, it takes precedence for fine-grained control.
/// Otherwise, falls back to the config level with known noisy dependencies (h2, hyper) silenced.
#[must_use]
pub fn get_env_filter(level: LogLevel) -> EnvFilter {
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
