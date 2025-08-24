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

use crate::config::Config;
use crate::registry::MetricsRegistryHandle;
use tokio_util::sync::CancellationToken;
use crate::error::Error;

pub mod attributes;
pub mod collector;
mod config;
pub mod descriptor;
pub mod error;
pub mod instrument;
pub mod metrics;
pub mod registry;
pub mod reporter;
pub mod semconv;

/// The main telemetry system that aggregates and reports metrics.
pub struct MetricsSystem {
    /// The metrics registry that holds all registered metrics (data + metadata).
    registry: MetricsRegistryHandle,

    /// The process collecting metrics from the pipelines and aggregating them into the registry.
    collector: collector::MetricsCollector,

    /// The process reporting metrics to an external system.
    reporter: reporter::MetricsReporter,
}

impl MetricsSystem {
    /// Creates a new [`MetricsSystem`] initialized with the given configuration.
    pub fn new(config: Config) -> Self {
        let metrics_registry = MetricsRegistryHandle::new();
        let (collector, reporter) = collector::MetricsCollector::new(
            config.clone(),
            metrics_registry.clone(),
        );
        Self {
            registry: metrics_registry,
            collector,
            reporter,
        }
    }

    /// Returns a shareable/cloneable handle to the metrics registry.
    pub fn registry(&self) -> MetricsRegistryHandle {
        self.registry.clone()
    }

    /// Returns a shareable/cloneable handle to the metrics reporter.
    pub fn reporter(&self) -> reporter::MetricsReporter {
        self.reporter.clone()
    }
    
    /// Starts the metrics collection loop and listens for a shutdown signal.
    /// This method returns when either the collection loop ends (Ok/Err) or the shutdown signal fires.
    pub async fn run(
        self,
        cancel: CancellationToken,
     ) -> Result<(), Error> {
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
        Self::new(Config::default())
    }
}
