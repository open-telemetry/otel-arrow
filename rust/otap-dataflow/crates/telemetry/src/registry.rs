// SPDX-License-Identifier: Apache-2.0

//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: This module will be entirely generated from a telemetry schema and Weaver in the future.

use crate::attributes::NodeStaticAttrs;
use crate::metrics::{
    MultivariateMetrics, OtlpExporterMetrics, OtlpReceiverMetrics, PerfExporterPdataMetrics,
};
use parking_lot::Mutex;
use slotmap::{SlotMap, new_key_type};
use std::fmt::Debug;
use std::sync::Arc;

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
pub struct MetricsRegistry {
    pub(crate) otlp_receiver_metrics: SlotMap<MetricsKey, (OtlpReceiverMetrics, NodeStaticAttrs)>,
    pub(crate) otlp_exporter_metrics: SlotMap<MetricsKey, (OtlpExporterMetrics, NodeStaticAttrs)>,
    pub(crate) perf_exporter_metrics:
        SlotMap<MetricsKey, (PerfExporterPdataMetrics, NodeStaticAttrs)>,
}

impl MetricsRegistry {
    fn register<T: MultivariateMetrics + Default + Debug + Send + Sync>(
        &mut self,
        metrics: &mut T,
        attrs: NodeStaticAttrs,
    ) {
        metrics.register_into(self, attrs);
    }

    fn add_otlp_receiver_metrics(
        &mut self,
        metrics_key: MetricsKey,
        metrics: &OtlpReceiverMetrics,
    ) {
        if let Some((existing_metrics, _)) = self.otlp_receiver_metrics.get_mut(metrics_key) {
            existing_metrics
                .bytes_received
                .add(metrics.bytes_received.get());
            existing_metrics
                .messages_received
                .add(metrics.messages_received.get());
        } else {
            // TODO: consider logging missing key
        }
    }

    fn add_otlp_exporter_metrics(
        &mut self,
        metrics_key: MetricsKey,
        metrics: &OtlpExporterMetrics,
    ) {
        if let Some((existing_metrics, _)) = self.otlp_exporter_metrics.get_mut(metrics_key) {
            existing_metrics
                .export_logs_request_received
                .add(metrics.export_logs_request_received.get());
            existing_metrics
                .export_logs_request_success
                .add(metrics.export_logs_request_success.get());
            existing_metrics
                .export_logs_request_failure
                .add(metrics.export_logs_request_failure.get());
            existing_metrics
                .export_traces_request_received
                .add(metrics.export_traces_request_received.get());
            existing_metrics
                .export_traces_request_success
                .add(metrics.export_traces_request_success.get());
            existing_metrics
                .export_traces_request_failure
                .add(metrics.export_traces_request_failure.get());
            existing_metrics
                .export_metrics_request_received
                .add(metrics.export_metrics_request_received.get());
            existing_metrics
                .export_metrics_request_success
                .add(metrics.export_metrics_request_success.get());
            existing_metrics
                .export_metrics_request_failure
                .add(metrics.export_metrics_request_failure.get());
        } else {
            // TODO: consider logging missing key
        }
    }

    // Perf exporter metrics operations
    fn add_perf_exporter_metrics(
        &mut self,
        metrics_key: MetricsKey,
        metrics: &PerfExporterPdataMetrics,
    ) {
        if let Some((existing_metrics, _)) = self.perf_exporter_metrics.get_mut(metrics_key) {
            existing_metrics.batches.add(metrics.batches.get());
            existing_metrics
                .invalid_batches
                .add(metrics.invalid_batches.get());
            existing_metrics
                .arrow_records
                .add(metrics.arrow_records.get());
            existing_metrics.logs.add(metrics.logs.get());
            existing_metrics.spans.add(metrics.spans.get());
            existing_metrics.metrics.add(metrics.metrics.get());
        } else {
            // TODO: consider logging missing key
        }
    }

    /// Iterate over all registered metrics (all concrete types) exposing them as trait objects with their static attrs.
    pub(crate) fn iter_metrics(
        &self,
    ) -> impl Iterator<Item = (&dyn MultivariateMetrics, &NodeStaticAttrs)> {
        let otlp_recv_iter = self
            .otlp_receiver_metrics
            .values()
            .map(|(m, attrs)| (m as &dyn MultivariateMetrics, attrs));
        let otlp_exporter_iter = self
            .otlp_exporter_metrics
            .values()
            .map(|(m, attrs)| (m as &dyn MultivariateMetrics, attrs));
        let perf_iter = self
            .perf_exporter_metrics
            .values()
            .map(|(m, attrs)| (m as &dyn MultivariateMetrics, attrs));
        otlp_recv_iter.chain(otlp_exporter_iter).chain(perf_iter)
    }

    /// Returns the total number of registered metrics sets.
    pub(crate) fn len(&self) -> usize {
        self.otlp_receiver_metrics.len()
            + self.otlp_exporter_metrics.len()
            + self.perf_exporter_metrics.len()
    }

    /// Iterates over all registered metrics that have at least one non-zero field, invoking the
    /// provided closure with the metric and its static attributes, then zeroes (flushes) them.
    ///
    /// This operation holds the registry mutex only for the duration of the iteration.
    pub(crate) fn for_each_changed_and_zero<F>(&mut self, mut f: F)
    where
        F: FnMut(&dyn MultivariateMetrics, &NodeStaticAttrs),
    {
        // OTLP Receiver metrics
        for (metrics, attrs) in self.otlp_receiver_metrics.values_mut() {
            if metrics.has_non_zero() {
                f(metrics as &dyn MultivariateMetrics, attrs);
                metrics.zero();
            }
        }
        // OTLP Exporter metrics
        for (metrics, attrs) in self.otlp_exporter_metrics.values_mut() {
            if metrics.has_non_zero() {
                f(metrics as &dyn MultivariateMetrics, attrs);
                metrics.zero();
            }
        }
        // Perf exporter metrics
        for (metrics, attrs) in self.perf_exporter_metrics.values_mut() {
            if metrics.has_non_zero() {
                f(metrics as &dyn MultivariateMetrics, attrs);
                metrics.zero();
            }
        }
    }
}

impl MetricsRegistryHandle {
    /// Creates a new `MetricsRegistry`.
    pub fn new() -> Self {
        Self {
            metric_registry: Arc::new(Mutex::new(MetricsRegistry {
                otlp_receiver_metrics: SlotMap::default(),
                otlp_exporter_metrics: SlotMap::default(),
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

    /// Adds a new set of OTLP receiver metrics to the aggregator.
    pub fn add_otlp_receiver_metrics(
        &self,
        metrics_key: MetricsKey,
        metrics: &OtlpReceiverMetrics,
    ) {
        self.metric_registry
            .lock()
            .add_otlp_receiver_metrics(metrics_key, metrics);
    }

    /// Adds a new set of OTLP exporter metrics to the aggregator.
    pub fn add_otlp_exporter_metrics(
        &self,
        metrics_key: MetricsKey,
        metrics: &OtlpExporterMetrics,
    ) {
        self.metric_registry
            .lock()
            .add_otlp_exporter_metrics(metrics_key, metrics);
    }

    /// Adds a new set of perf exporter metrics to the aggregator.
    pub fn add_perf_exporter_metrics(
        &self,
        metrics_key: MetricsKey,
        metrics: &PerfExporterPdataMetrics,
    ) {
        self.metric_registry
            .lock()
            .add_perf_exporter_metrics(metrics_key, metrics);
    }

    /// Iterate over all registered metrics invoking the provided closure.
    /// The closure receives a `&dyn MultivariateMetrics` and a `&NodeStaticAttrs`.
    /// Note: The closure must not attempt to register new metrics to avoid deadlocks.
    pub fn for_each_metrics<F>(&self, mut f: F)
    where
        F: FnMut(&dyn MultivariateMetrics, &NodeStaticAttrs),
    {
        let reg = self.metric_registry.lock();
        for (m, attrs) in reg.iter_metrics() {
            f(m, attrs);
        }
    }

    /// Returns the total number of registered metrics sets.
    pub fn len(&self) -> usize {
        self.metric_registry.lock().len()
    }

    /// Iterates over all multivariate metrics that have at least one non-zero counter/value.
    /// The closure is invoked with the current (pre-zero) metrics followed by the metrics being
    /// zeroed (flushed) before proceeding. Metrics that are all zero are skipped.
    pub fn for_each_changed_and_zero<F>(&self, f: F)
    where
        F: FnMut(&dyn MultivariateMetrics, &NodeStaticAttrs),
    {
        let mut reg = self.metric_registry.lock();
        reg.for_each_changed_and_zero(f);
    }
}

#[cfg(test)]
mod tests {
    use crate::attributes::NodeStaticAttrs;
    use crate::metrics::{MultivariateMetrics, OtlpReceiverMetrics};

    #[test]
    fn test_multivariate_metrics_aggregator() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = super::MetricsRegistryHandle::new();

        // Declare 2 receivers and 1 perf exporter multivariate metrics
        let mut otlp_receiver_mvm = OtlpReceiverMetrics::default();
        registry.register(
            &mut otlp_receiver_mvm,
            NodeStaticAttrs {
                node_id: "otlp_receiver".into(),
                node_type: "receiver".into(),
                pipeline_id: "pipeline1".into(),
                core_id: 0,
                numa_node_id: 0,
                process_id: 0,
            },
        );

        let mut otap_receiver_mvm = OtlpReceiverMetrics::default();
        registry.register(
            &mut otap_receiver_mvm,
            NodeStaticAttrs {
                node_id: "otap_receiver".into(),
                node_type: "receiver".into(),
                pipeline_id: "pipeline1".into(),
                core_id: 0,
                numa_node_id: 0,
                process_id: 0,
            },
        );

        let mut perf_exporter_mvm = crate::metrics::PerfExporterPdataMetrics::default();
        registry.register(
            &mut perf_exporter_mvm,
            NodeStaticAttrs {
                node_id: "perf_exporter".into(),
                node_type: "exporter".into(),
                pipeline_id: "pipeline1".into(),
                core_id: 0,
                numa_node_id: 0,
                process_id: 0,
            },
        );

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

        perf_exporter_mvm.batches += 50;
        perf_exporter_mvm.invalid_batches += 40;
        perf_exporter_mvm.logs += 60;
        perf_exporter_mvm.spans += 70;
        perf_exporter_mvm.metrics += 80;
        perf_exporter_mvm.aggregate_into(&mut registry)?;

        perf_exporter_mvm.batches += 70;
        perf_exporter_mvm.invalid_batches += 60;
        perf_exporter_mvm.logs += 80;
        perf_exporter_mvm.spans += 90;
        perf_exporter_mvm.metrics += 100;
        perf_exporter_mvm.aggregate_into(&mut registry)?;

        // Previous dbg!(registry) moved the handle; borrow instead.
        dbg!(&registry);

        // Validate iterator exposes all 3 metrics sets.
        let inner = registry.metric_registry.lock();
        let count = inner.iter_metrics().count();
        assert_eq!(count, 3);
        Ok(())
    }

    #[test]
    fn test_for_each_changed_and_zero() -> Result<(), Box<dyn std::error::Error>> {
        use crate::metrics::PerfExporterPdataMetrics;
        let mut registry = super::MetricsRegistryHandle::new();

        let mut recv = OtlpReceiverMetrics::default();
        registry.register(
            &mut recv,
            NodeStaticAttrs {
                node_id: "r1".into(),
                node_type: "receiver".into(),
                pipeline_id: "p".into(),
                core_id: 0,
                numa_node_id: 0,
                process_id: 0,
            },
        );
        let mut perf = PerfExporterPdataMetrics::default();
        registry.register(
            &mut perf,
            NodeStaticAttrs {
                node_id: "perf".into(),
                node_type: "exporter".into(),
                pipeline_id: "p".into(),
                core_id: 0,
                numa_node_id: 0,
                process_id: 0,
            },
        );

        // Initially all zero: expect no invocation.
        let mut calls = 0usize;
        registry.for_each_changed_and_zero(|_, _| calls += 1);
        assert_eq!(calls, 0, "No metrics should be flushed when all are zero");

        // Add some values and aggregate.
        recv.bytes_received += 10;
        recv.messages_received += 2;
        recv.aggregate_into(&mut registry)?;
        perf.invalid_batches += 3; // only this and one other counter changed
        perf.aggregate_into(&mut registry)?;

        // Expect two invocations (receiver + perf exporter)
        let mut seen = Vec::new();
        registry.for_each_changed_and_zero(|m, attrs| {
            seen.push(attrs.node_id.clone());
            // ensure reported values are non-zero
            assert!(m.has_non_zero());
        });
        seen.sort();
        assert_eq!(seen, vec!["perf", "r1"]);

        // A second pass should yield nothing (they were zeroed)
        let mut calls_after = 0usize;
        registry.for_each_changed_and_zero(|_, _| calls_after += 1);
        assert_eq!(
            calls_after, 0,
            "Metrics should have been zeroed after flush"
        );

        // Change only one perf exporter field and ensure it's still flushed.
        perf.invalid_batches += 7;
        perf.aggregate_into(&mut registry)?;
        let mut flushed = Vec::new();
        registry.for_each_changed_and_zero(|_, attrs| flushed.push(attrs.node_id.clone()));
        assert_eq!(
            flushed,
            vec!["perf"],
            "Only perf metrics changed so only it should flush"
        );

        Ok(())
    }
}
