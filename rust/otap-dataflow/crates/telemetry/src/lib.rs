// SPDX-License-Identifier: Apache-2.0

//! Type-safe API and NUMA-aware telemetry data structures and utilities.
//!
//! Note: The NUMA-aware design is not fully implemented yet.
//! ToDo: First aggregation pass per NUMA node, then global aggregation.

use crate::registry::MetricsRegistryHandle;

pub mod registry;
pub mod metrics;
pub mod collector;
pub mod counter;
mod descriptor;
pub mod attributes;
pub mod reporter;
mod error;
pub(crate) mod pipeline;

/// The main telemetry system that aggregates and reports metrics.
pub struct MetricsSystem {
    /// The metrics registry that holds all registered metrics.
    registry: MetricsRegistryHandle,

    /// The metrics collector that aggregates metrics.
    collector: collector::MetricsCollector,

    /// The reporter that sends metrics to the external system.
    reporter: reporter::MetricsReporter,
}

impl MetricsSystem {
    /// Creates a new `MetricsSystem`.
    pub fn new() -> Self {
        let metrics_registry = MetricsRegistryHandle::new();
        let (collector, reporter) = collector::MetricsCollector::new(metrics_registry.clone());
        Self {
            registry: metrics_registry,
            collector,
            reporter,
        }
    }

    /// Returns a handle to the metrics registry.
    pub fn registry(&self) -> MetricsRegistryHandle {
        self.registry.clone()
    }

    /// Returns a handle to the metrics reporter.
    pub fn reporter(&self) -> reporter::MetricsReporter {
        self.reporter.clone()
    }

    /// Returns a handle to the metrics collector.
    pub async fn run_collection_loop(self) -> Result<(), error::Error> {
        self.collector.run_collection_loop().await
    }
}