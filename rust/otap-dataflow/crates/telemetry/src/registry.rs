// SPDX-License-Identifier: Apache-2.0

//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: Concrete metrics live in their respective crates; this registry aggregates them via
//! dynamic dispatch.

use crate::attributes::NodeStaticAttrs;
use crate::metrics::MultivariateMetrics;
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
pub struct MetricsRegistry {
    /// All registered metrics keyed by MetricsKey with their static attrs.
    pub(crate) metrics: SlotMap<MetricsKey, (Box<dyn MultivariateMetrics + Send + Sync>, NodeStaticAttrs)>,
}

impl Debug for MetricsRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsRegistry")
            .field("metrics_len", &self.metrics.len())
            .finish()
    }
}

impl MetricsRegistry {
    fn register<T: MultivariateMetrics + Default + Debug + Send + Sync>(&mut self, metrics: &mut T, attrs: NodeStaticAttrs) { metrics.register_into(self, attrs); }

    /// Insert a default instance of a metrics type with attributes, returning its key.
    pub fn insert_default<T: MultivariateMetrics + Default + Send + Sync + 'static>(&mut self, attrs: NodeStaticAttrs) -> MetricsKey {
        self.metrics.insert((Box::new(T::default()), attrs))
    }

    /// Generic add method: merges the provided metrics into the registered instance keyed by `metrics_key`.
    pub fn add_metrics(&mut self, metrics_key: MetricsKey, metrics: &dyn MultivariateMetrics) {
        if let Some((existing, _attrs)) = self.metrics.get_mut(metrics_key) {
            existing.merge_from_same_kind(metrics);
        } else {
            // TODO: consider logging missing key
        }
    }

    /// Iterate over all registered metrics (all concrete types) exposing them as trait objects with their static attrs.
    pub(crate) fn iter_metrics(
        &self,
    ) -> impl Iterator<Item = (&dyn MultivariateMetrics, &NodeStaticAttrs)> {
        self.metrics
            .values()
            .map(|(m, attrs)| (m.as_ref() as &dyn MultivariateMetrics, attrs))
    }

    /// Returns the total number of registered metrics sets.
    pub(crate) fn len(&self) -> usize {
    self.metrics.len()
    }

    /// Iterates over all registered metrics that have at least one non-zero field, invoking the
    /// provided closure with the metric and its static attributes, then zeroes (flushes) them.
    ///
    /// This operation holds the registry mutex only for the duration of the iteration.
    pub(crate) fn for_each_changed_and_zero<F>(&mut self, mut f: F)
    where
        F: FnMut(&dyn MultivariateMetrics, &NodeStaticAttrs),
    {
        for (metrics, attrs) in self.metrics.values_mut() {
            if metrics.has_non_zero() {
                f(metrics.as_ref(), attrs);
                metrics.zero();
            }
        }
    }
}

impl MetricsRegistryHandle {
    /// Creates a new `MetricsRegistry`.
    pub fn new() -> Self {
        Self {
            metric_registry: Arc::new(Mutex::new(MetricsRegistry { metrics: SlotMap::default() })),
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

    /// Adds a new metrics snapshot to the aggregator for the given key.
    pub fn add_metrics(&self, metrics_key: MetricsKey, metrics: &dyn MultivariateMetrics) {
        self.metric_registry.lock().add_metrics(metrics_key, metrics);
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
    use crate::metrics::MultivariateMetrics;
    use crate::registry::MetricsKey;
    use crate::descriptor::{MetricsDescriptor, MetricsField, MetricsKind};

    #[test]
    fn test_multivariate_metrics_aggregator() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = super::MetricsRegistryHandle::new();
    #[derive(Clone, Default, Debug)]
        struct M { key: Option<MetricsKey>, v: u64 }
        const DESC: MetricsDescriptor = MetricsDescriptor { name: "m", fields: &[MetricsField { name: "v", unit: "{u}", kind: MetricsKind::Counter }] };
        impl MultivariateMetrics for M {
            fn register_into(&mut self, registry: &mut super::MetricsRegistry, attrs: NodeStaticAttrs) { self.key = Some(registry.insert_default::<Self>(attrs)); }
            fn descriptor(&self) -> &'static MetricsDescriptor { &DESC }
            fn field_values(&self) -> Box<dyn Iterator<Item=(&'static MetricsField, u64)> + '_> { Box::new(DESC.fields.iter().zip([self.v].into_iter()).map(|(f,v)| (f,v))) }
            fn merge_from_same_kind(&mut self, other: &dyn MultivariateMetrics) { let o = other.as_any().downcast_ref::<Self>().unwrap(); self.v += o.v; }
            fn aggregate_into(&self, registry: &mut super::MetricsRegistryHandle) -> Result<(), crate::error::Error> { if let Some(k)=self.key { registry.add_metrics(k, self); Ok(()) } else { Err(crate::error::Error::MetricsNotRegistered { descriptor: self.descriptor() }) } }
            fn zero(&mut self) { self.v = 0; }
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        }

        let mut m1 = M::default();
        registry.register(&mut m1, NodeStaticAttrs { node_id: "n1".into(), node_type: "t".into(), pipeline_id: "p".into(), core_id: 0, numa_node_id: 0, process_id: 0 });
        m1.v = 10; m1.aggregate_into(&mut registry)?;
        let mut m2 = M::default();
        registry.register(&mut m2, NodeStaticAttrs { node_id: "n2".into(), node_type: "t".into(), pipeline_id: "p".into(), core_id: 0, numa_node_id: 0, process_id: 0 });
        m2.v = 5; m2.aggregate_into(&mut registry)?;
        let inner = registry.metric_registry.lock();
        assert_eq!(inner.len(), 2);
        Ok(())
    }

    #[test]
    fn test_for_each_changed_and_zero() -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = super::MetricsRegistryHandle::new();

    #[derive(Clone, Default, Debug)]
        struct M { key: Option<MetricsKey>, v: u64 }
        const DESC: MetricsDescriptor = MetricsDescriptor { name: "m", fields: &[MetricsField { name: "v", unit: "{u}", kind: MetricsKind::Counter }] };
        impl MultivariateMetrics for M {
            fn register_into(&mut self, registry: &mut super::MetricsRegistry, attrs: NodeStaticAttrs) { self.key = Some(registry.insert_default::<Self>(attrs)); }
            fn descriptor(&self) -> &'static MetricsDescriptor { &DESC }
            fn field_values(&self) -> Box<dyn Iterator<Item=(&'static MetricsField, u64)> + '_> { Box::new(DESC.fields.iter().zip([self.v].into_iter()).map(|(f,v)| (f,v))) }
            fn merge_from_same_kind(&mut self, other: &dyn MultivariateMetrics) { let o = other.as_any().downcast_ref::<Self>().unwrap(); self.v += o.v; }
            fn aggregate_into(&self, registry: &mut super::MetricsRegistryHandle) -> Result<(), crate::error::Error> { if let Some(k)=self.key { registry.add_metrics(k, self); Ok(()) } else { Err(crate::error::Error::MetricsNotRegistered { descriptor: self.descriptor() }) } }
            fn zero(&mut self) { self.v = 0; }
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        }

        let mut recv = M::default();
        registry.register(&mut recv, NodeStaticAttrs { node_id: "r1".into(), node_type: "receiver".into(), pipeline_id: "p".into(), core_id: 0, numa_node_id: 0, process_id: 0 });
        let mut perf = M::default();
        registry.register(&mut perf, NodeStaticAttrs { node_id: "perf".into(), node_type: "exporter".into(), pipeline_id: "p".into(), core_id: 0, numa_node_id: 0, process_id: 0 });

        // Initially all zero: expect no invocation.
        let mut calls = 0usize;
        registry.for_each_changed_and_zero(|_, _| calls += 1);
        assert_eq!(calls, 0, "No metrics should be flushed when all are zero");

        // Add some values and aggregate.
        recv.v += 10;
        recv.aggregate_into(&mut registry)?;
    perf.v += 3; // changed
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
