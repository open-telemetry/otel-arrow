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
use otap_df_config::pipeline::TelemetryConfig;
use tokio_util::sync::CancellationToken;

pub mod attributes;
pub mod collector;
pub mod descriptor;
pub mod error;
pub mod instrument;
/// Internal logs/events module for engine.
pub mod internal_events;
pub mod metrics;
pub mod opentelemetry_client;
pub mod registry;
pub mod reporter;
pub mod semconv;

// Re-export _private module from internal_events for macro usage.
// This allows the otel_info!, otel_warn!, etc. macros to work in other crates
// without requiring them to add tracing as a direct dependency.
#[doc(hidden)]
pub use internal_events::_private;

/// Initializes internal logging for the OTAP engine.
///
/// This should be called once at application startup before any logging occurs.
///
/// TODO: The engine uses a thread-per-core model.
/// The fmt::init() here is truly global, and hence
/// this will be a source of contention.
/// We need to evaluate alternatives:
/// 1. Set up per thread subscriber.
/// //start of thread
/// let _guard = tracing::subscriber::set_default(subscriber);
/// now, with this thread, all tracing calls will go to this subscriber
/// eliminating contention.
/// //end of thread
/// 2. Use custom subscriber that batches logs in thread-local buffer, and
/// flushes them periodically.
/// The TODO here is to evaluate these options and implement one of them.
/// As of now, this causes contention, and we just need to accept temporarily.
pub fn init_logging() {
    tracing_subscriber::fmt::init();
}

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
