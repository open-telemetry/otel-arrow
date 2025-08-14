// SPDX-License-Identifier: Apache-2.0

//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: This module will be entirely generated from a telemetry schema and Weaver in the future.

use std::fmt::Debug;
use std::sync::Arc;
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};
use crate::attributes::NodeStaticAttrs;
use crate::metrics::{MultivariateMetrics, PerfExporterMetrics, ReceiverMetrics};

new_key_type! {
    /// A unique key for identifying a set of metrics in the registry.
    pub struct MetricsKey;
}

/// A sendable and cloneable handle on a metrics registry.
///
/// # Performance Note
///
/// The mutexes used here ARE NOT on the hot path of metrics reporting. They are only used:
/// - when registering new metrics (which is a rare operation compared to reporting metrics),
/// - or when the consumer of the MPSC channel aggregates the received metrics into the registry
///   (which is not on the hot path either).
#[derive(Debug, Clone)]
pub struct MetricsRegistryHandle {
    metric_registry: Arc<Mutex<MetricsRegistry>>,
}

/// A metrics registry that maintains aggregated metrics for different set of static attributes.
///
#[derive(Debug)]
pub(crate) struct MetricsRegistry {
    pub(crate) receiver_metrics: SlotMap<MetricsKey,(ReceiverMetrics, NodeStaticAttrs)>,
    pub(crate) perf_exporter_metrics: SlotMap<MetricsKey,(PerfExporterMetrics, NodeStaticAttrs)>,
}

impl MetricsRegistry {
    fn register<T: MultivariateMetrics + Default + Debug + Send + Sync>(
        &mut self,
        metrics: &mut T,
        attrs: NodeStaticAttrs,
    ) {
        metrics.register_into(self, attrs);
    }

    fn add_receiver_metrics(&mut self, metrics_key: MetricsKey, metrics: &ReceiverMetrics) {
        if let Some((existing_metrics, _)) = self.receiver_metrics.get_mut(metrics_key) {
            existing_metrics.bytes_received.add(metrics.bytes_received.get());
            existing_metrics.messages_received.add(metrics.messages_received.get());
        } else {
            // TODO: consider logging missing key
        }
    }
    fn get_receiver_metrics(&self, metrics_key: MetricsKey) -> Option<(ReceiverMetrics, NodeStaticAttrs)> {
        self.receiver_metrics.get(metrics_key).cloned()
    }

    // Perf exporter metrics operations
    fn add_perf_exporter_metrics(&mut self, metrics_key: MetricsKey, metrics: &PerfExporterMetrics) {
        if let Some((existing_metrics, _)) = self.perf_exporter_metrics.get_mut(metrics_key) {
            existing_metrics.bytes_total.add(metrics.bytes_total.get());
            existing_metrics.pdata_msgs.add(metrics.pdata_msgs.get());
            existing_metrics.invalid_pdata_msgs.add(metrics.invalid_pdata_msgs.get());
            existing_metrics.logs.add(metrics.logs.get());
            existing_metrics.spans.add(metrics.spans.get());
            existing_metrics.metrics.add(metrics.metrics.get());
        } else {
            // TODO: consider logging missing key
        }
    }
    fn get_perf_exporter_metrics(&self, metrics_key: MetricsKey) -> Option<(PerfExporterMetrics, NodeStaticAttrs)> {
        self.perf_exporter_metrics.get(metrics_key).cloned()
    }
}

impl MetricsRegistryHandle {
    /// Creates a new `MetricsRegistry`.
    pub fn new() -> Self {
        Self {
            metric_registry: Arc::new(Mutex::new(MetricsRegistry {
                receiver_metrics: SlotMap::default(),
                perf_exporter_metrics: SlotMap::default(),
            })),
        }
    }

    /// Registers a new multivariate metrics instance with the given static attributes.
    pub fn register<T: MultivariateMetrics + Default + Debug + Send + Sync>(
        &self,
        metrics: &mut T,
        attrs: NodeStaticAttrs,
    ) {
        self.metric_registry.lock().register(metrics, attrs);
    }
    
    /// Adds a new set of receiver metrics to the aggregator.
    pub fn add_receiver_metrics(&self, metrics_key: MetricsKey, metrics: &ReceiverMetrics) {
        self.metric_registry.lock().add_receiver_metrics(metrics_key, metrics);
    }

    /// Adds a new set of perf exporter metrics to the aggregator.
    pub fn add_perf_exporter_metrics(&self, metrics_key: MetricsKey, metrics: &PerfExporterMetrics) {
        self.metric_registry.lock().add_perf_exporter_metrics(metrics_key, metrics);
    }
}

#[cfg(test)]
mod tests {
    use crate::attributes::NodeStaticAttrs;
    use crate::metrics::{MultivariateMetrics, ReceiverMetrics};

    #[test]
    fn test_multivariate_metrics_aggregator() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = super::MetricsRegistryHandle::new();

        // Declare 2 receivers and 1 perf exporter multivariate metrics
        let mut otlp_receiver_mvm = ReceiverMetrics::default();
        registry.register(&mut otlp_receiver_mvm, NodeStaticAttrs {
            node_id: "otlp_receiver".into(),
            node_type: "receiver".into(),
            pipeline_id: "pipeline1".into(),
            core_id: 0,
            numa_node_id: 0,
            process_id: 0,
        });

        let mut otap_receiver_mvm = ReceiverMetrics::default();
        registry.register(&mut otap_receiver_mvm, NodeStaticAttrs {
            node_id: "otap_receiver".into(),
            node_type: "receiver".into(),
            pipeline_id: "pipeline1".into(),
            core_id: 0,
            numa_node_id: 0,
            process_id: 0,
        });

        let mut perf_exporter_mvm = crate::metrics::PerfExporterMetrics::default();
        registry.register(&mut perf_exporter_mvm, NodeStaticAttrs {
            node_id: "perf_exporter".into(),
            node_type: "exporter".into(),
            pipeline_id: "pipeline1".into(),
            core_id: 0,
            numa_node_id: 0,
            process_id: 0,
        });

        otlp_receiver_mvm.bytes_received += 100;
        otlp_receiver_mvm.messages_received += 10;
        otlp_receiver_mvm.bytes_received += 40;
        otlp_receiver_mvm.messages_received += 20;
        otlp_receiver_mvm.aggregate_into(&mut registry)?;

        otap_receiver_mvm.bytes_received += 200;
        otap_receiver_mvm.messages_received += 20;
        otap_receiver_mvm.aggregate_into(&mut registry)?;

        otlp_receiver_mvm.bytes_received += 10;
        otlp_receiver_mvm.messages_received += 50;
        otlp_receiver_mvm.bytes_received += 70;
        otlp_receiver_mvm.messages_received += 90;
        otlp_receiver_mvm.aggregate_into(&mut registry)?;

        perf_exporter_mvm.bytes_total += 500;
        perf_exporter_mvm.pdata_msgs += 50;
        perf_exporter_mvm.invalid_pdata_msgs += 40;
        perf_exporter_mvm.logs += 60;
        perf_exporter_mvm.spans += 70;
        perf_exporter_mvm.metrics += 80;
        perf_exporter_mvm.aggregate_into(&mut registry)?;

        perf_exporter_mvm.bytes_total += 600;
        perf_exporter_mvm.pdata_msgs += 70;
        perf_exporter_mvm.invalid_pdata_msgs += 60;
        perf_exporter_mvm.logs += 80;
        perf_exporter_mvm.spans += 90;
        perf_exporter_mvm.metrics += 100;
        perf_exporter_mvm.aggregate_into(&mut registry)?;

        dbg!(registry);
        
        Ok(())
    }
}