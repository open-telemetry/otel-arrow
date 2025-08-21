// SPDX-License-Identifier: Apache-2.0

//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: Concrete metrics live in their respective crates; this registry aggregates them via
//! dynamic dispatch.

use crate::attributes::AttributeSetHandler;
use crate::metrics::{MetricSetHandler, MetricSet};
use crate::semconv::SemConvRegistry;
use parking_lot::Mutex;
use slotmap::{SlotMap, new_key_type};
use std::fmt::Debug;
use std::sync::Arc;
use std::collections::HashSet;
use crate::descriptor::MetricsField;
use crate::descriptor::MetricsDescriptor;

new_key_type! {
    /// This key is used to identify a specific metrics entry in the registry (slotmap index).
    pub struct MetricsKey;
}

/// A registered metrics entry containing all necessary information for metrics aggregation.
pub struct MetricsEntry {
    /// The static descriptor describing the metrics structure
    pub metrics_descriptor: &'static MetricsDescriptor,
    /// The static descriptor describing the attributes structure
    pub attributes_descriptor: &'static crate::descriptor::AttributesDescriptor,

    /// Current metric values stored as a vector
    pub metric_values: Vec<u64>,

    /// Handler for the associated attribute set
    pub attribute_values: Box<dyn AttributeSetHandler + Send + Sync>,
}

impl Debug for MetricsEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsEntry")
            .field("metrics_descriptor", &self.metrics_descriptor)
            .field("attributes_descriptor", &self.attributes_descriptor)
            .field("metric_values", &self.metric_values)
            .field("attribute_values", &"<AttributeSetHandler>")
            .finish()
    }
}

impl MetricsEntry {
    /// Creates a new metrics entry
    pub fn new(
        metrics_descriptor: &'static MetricsDescriptor,
        attributes_descriptor: &'static crate::descriptor::AttributesDescriptor,
        metric_values: Vec<u64>,
        attribute_values: Box<dyn AttributeSetHandler + Send + Sync>,
    ) -> Self {
        Self {
            metrics_descriptor,
            attributes_descriptor,
            metric_values,
            attribute_values,
        }
    }
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
    pub(crate) metrics: SlotMap<MetricsKey, MetricsEntry>,
}

impl Debug for MetricsRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsRegistry")
            .field("metrics_len", &self.metrics.len())
            .finish()
    }
}

impl MetricsRegistry {
    fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(&mut self, static_attrs: impl AttributeSetHandler + Send + Sync + 'static) -> MetricSet<T> {
        let metrics = T::default();
        let descriptor = metrics.descriptor();

        let metrics_key = self.metrics.insert(MetricsEntry::new(descriptor, static_attrs.descriptor(), metrics.snapshot_values(), Box::new(static_attrs)));

        // ToDo remove this debug print in production code
        println!("{}", self.generate_semconv_registry().to_yaml());

        MetricSet { key: metrics_key, metrics }
    }

    /// Generic add method: merges the provided metrics into the registered instance keyed by `metrics_key`.
    pub fn add_metrics(&mut self, metrics_key: MetricsKey, metrics_values: &[u64]) {
        if let Some(entry) = self.metrics.get_mut(metrics_key) {
            entry.metric_values.iter_mut().zip(metrics_values).for_each(|(e, v)| *e += v);
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
        F: for<'a> FnMut(&'static str, Box<dyn Iterator<Item = (&'a MetricsField, u64)> + 'a>, &dyn AttributeSetHandler),
    {
        for entry in self.metrics.values_mut() {
            let values = &mut entry.metric_values;
            let descriptor = entry.metrics_descriptor;
            let attrs = entry.attribute_values.as_ref();

            if values.iter().any(|&v| v != 0) {
                let iter = descriptor
                    .metrics
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

    /// Generates a SemConvRegistry from the current MetricsRegistry.
    /// AttributeFields are deduplicated based on their key.
    pub fn generate_semconv_registry(&self) -> SemConvRegistry {
        let mut unique_attributes = HashSet::new();
        let mut attributes = Vec::new();
        let mut metric_sets = Vec::new();

        // Collect all unique metric descriptors
        let mut unique_metrics = HashSet::new();
        for entry in self.metrics.values() {
            // Add metrics descriptor if not already seen
            if unique_metrics.insert(entry.metrics_descriptor as *const _) {
                metric_sets.push(entry.metrics_descriptor);
            }

            // Add attribute fields, deduplicating by key
            for field in entry.attributes_descriptor.fields {
                if unique_attributes.insert(field.key) {
                    attributes.push(field);
                }
            }
        }

        SemConvRegistry {
            version: "2".into(),
            attributes,
            metric_sets,
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
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
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
        F: for<'a> FnMut(&'static str, Box<dyn Iterator<Item = (&'a MetricsField, u64)> + 'a>, &dyn AttributeSetHandler),
    {
        let mut reg = self.metric_registry.lock();
        reg.for_each_changed_field_iter_and_zero(f);
    }

    /// Generates a SemConvRegistry from the current MetricsRegistry.
    /// AttributeFields are deduplicated based on their key.
    pub fn generate_semconv_registry(&self) -> SemConvRegistry {
        self.metric_registry.lock().generate_semconv_registry()
    }
}