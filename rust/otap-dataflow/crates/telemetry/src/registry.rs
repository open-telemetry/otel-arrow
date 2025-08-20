// SPDX-License-Identifier: Apache-2.0

//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: Concrete metrics live in their respective crates; this registry aggregates them via
//! dynamic dispatch.

use crate::attributes::NodeStaticAttrs;
use crate::metrics::{MetricSetHandler, MetricSet};
use parking_lot::Mutex;
use slotmap::{SlotMap, new_key_type};
use std::fmt::Debug;
use std::sync::Arc;
use crate::descriptor::MetricsField;
use crate::descriptor::MetricsDescriptor;

new_key_type! {
    /// A unique key type for metrics in the registry.
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
    pub(crate) metrics: SlotMap<MetricsKey, (Vec<u64>, &'static MetricsDescriptor, NodeStaticAttrs)>,
}

impl Debug for MetricsRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsRegistry")
            .field("metrics_len", &self.metrics.len())
            .finish()
    }
}

impl MetricsRegistry {
    fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(&mut self, attrs: NodeStaticAttrs) -> MetricSet<T> {
        let metrics = T::default();
        let descriptor = metrics.descriptor();
        let metrics_key = self.metrics.insert((metrics.snapshot_values(), descriptor, attrs));

        MetricSet { key: metrics_key, metrics }
    }

    /// Generic add method: merges the provided metrics into the registered instance keyed by `metrics_key`.
    pub fn add_metrics(&mut self, metrics_key: MetricsKey, metrics_values: &[u64]) {
        if let Some((existing, _descriptor, _attrs)) = self.metrics.get_mut(metrics_key) {
            existing.iter_mut().zip(metrics_values).for_each(|(e, v)| *e += v);
        } else {
            // TODO: consider logging missing key
        }
    }

    /// Returns the total number of registered metrics sets.
    pub(crate) fn len(&self) -> usize {
    self.metrics.len()
    }

    /// Iterates over all registered metrics which have at least one non-zero value.
    /// For each such metrics instance it constructs an iterator of (&MetricsField, u64)
    /// for only the non-zero entries, invokes `f`, then zeroes the metrics.
    pub(crate) fn for_each_changed_field_iter_and_zero<F>(&mut self, mut f: F)
    where
        F: for<'a> FnMut(&'static str, Box<dyn Iterator<Item = (&'a MetricsField, u64)> + 'a>, &NodeStaticAttrs),
    {
        for (values, descriptor, attrs) in self.metrics.values_mut() {
            if values.iter().any(|&v| v != 0) {
                let iter = descriptor
                    .fields
                    .iter()
                    .zip(values.iter())
                    .filter(|(_, v)| **v != 0)
                    .map(|(field, &v)| (field, v));
                f(descriptor.name, Box::new(iter), attrs);
                // zero after reporting
                values.iter_mut().for_each(|v| *v = 0);
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
    pub fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        attrs: NodeStaticAttrs,
    ) -> MetricSet<T> {
        self.metric_registry.lock().register(attrs)
    }

    /// Adds a new metrics snapshot to the aggregator for the given key.
    pub fn add_metrics(&self, metrics_key: MetricsKey, metrics: &[u64]) {
        self.metric_registry.lock().add_metrics(metrics_key, metrics);
    }

    /// Returns the total number of registered metrics sets.
    pub fn len(&self) -> usize {
        self.metric_registry.lock().len()
    }

    /// Handle wrapper for `MetricsRegistry::for_each_changed_field_iter_and_zero`.
    pub fn for_each_changed_field_iter_and_zero<F>(&self, f: F)
    where
        F: for<'a> FnMut(&'static str, Box<dyn Iterator<Item = (&'a MetricsField, u64)> + 'a>, &NodeStaticAttrs),
    {
        let mut reg = self.metric_registry.lock();
        reg.for_each_changed_field_iter_and_zero(f);
    }
}