// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The telemetry registry component combining entity and metrics registries (see the ITS diagram
//! architecture in the main [README.md](../README.md) file of this crate.

use crate::attributes::AttributeSetHandler;
use crate::descriptor::MetricsDescriptor;
use crate::entity::{EntityRegistry, RegisterOutcome};
use crate::metrics::{
    MetricSet, MetricSetHandler, MetricSetRegistry, MetricValue, MetricsIterator,
};
use crate::otel_debug;
use crate::semconv::SemConvRegistry;
use parking_lot::Mutex;
use slotmap::new_key_type;
use std::fmt::Debug;
use std::sync::Arc;

new_key_type! {
    /// This key is used to identify a specific entity entry in the entity registry (slotmap index).
    pub struct EntityKey;

    /// This key is used to identify a specific metrics entry in the metric set registry (slotmap
    /// index).
    pub struct MetricSetKey;
}

/// A sendable and cloneable handle on a telemetry registry.
///
/// # Performance Note
///
/// The mutexes used here ARE NOT on the hot path of metrics reporting. They are only used:
/// - when registering new metrics (which is a rare operation compared to reporting metrics),
/// - or when the consumer of the MPSC channel aggregates the received metrics into the registry
///   (which is not on the hot path either).
#[derive(Debug, Clone)]
pub struct TelemetryRegistryHandle {
    pub(crate) registry: Arc<Mutex<TelemetryRegistry>>,
}

/// The main telemetry registry maintaining both entity and metric set registries.
#[derive(Debug, Default)]
pub(crate) struct TelemetryRegistry {
    pub(crate) entities: EntityRegistry,
    pub(crate) metrics: MetricSetRegistry,
}

impl Default for TelemetryRegistryHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryRegistryHandle {
    /// Creates a new `TelemetryRegistryHandle`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(TelemetryRegistry::default())),
        }
    }

    /// Registers (or reuses) an entity for the provided attribute set.
    /// Logs the entity definition when a new entity is created.
    pub fn register_entity(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> EntityKey {
        let schema = attrs.schema_name();
        let definition = attrs.attributes_to_string();
        let outcome = self.registry.lock().entities.register(attrs);
        if let RegisterOutcome::Created(_) = outcome {
            // Log the entity definition.
            //
            // TODO(#1907): This could benefit from logging a human-readable form
            // of the entity that we refer to later in the logs, instead of logging
            // every key/value in every line of console_async output.
            otel_debug!("registry.define_entity", schema, definition);
        }
        outcome.key()
    }

    /// Unregisters an entity by key.
    #[must_use]
    pub fn unregister_entity(&self, entity_key: EntityKey) -> bool {
        self.registry.lock().entities.unregister(entity_key)
    }

    /// Returns the total number of registered entities.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.registry.lock().entities.len()
    }

    /// Visits a single entity by key.
    pub fn visit_entity<F, T>(&self, key: EntityKey, f: F) -> Option<T>
    where
        F: FnOnce(&dyn AttributeSetHandler) -> T,
    {
        let reg = self.registry.lock();
        reg.entities.get(key).map(|attrs| f(attrs))
    }

    /// Registers a metric set type with the given static attributes and returns a `MetricSet`
    /// instance that can be used to report metrics for that type.
    pub fn register_metric_set<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> MetricSet<T> {
        // TODO: Note this code path is not logged the way entity registration
        // does for referring to in console logs. Will be needed to print metrics
        // to the console.
        let mut registry = self.registry.lock();
        let outcome = registry.entities.register(attrs);
        registry.metrics.register(outcome.key())
    }

    /// Registers a metric set type for an existing entity key.
    #[must_use]
    pub fn register_metric_set_for_entity<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        entity_key: EntityKey,
    ) -> MetricSet<T> {
        let mut registry = self.registry.lock();
        let retained = registry.entities.retain(entity_key);
        debug_assert!(retained, "entity key must be registered before metrics");
        registry.metrics.register(entity_key)
    }

    /// Unregisters a metric set by key.
    #[must_use]
    pub fn unregister_metric_set(&self, metrics_key: MetricSetKey) -> bool {
        let mut reg = self.registry.lock();
        if let Some(entity_key) = reg.metrics.unregister(metrics_key) {
            let _ = reg.entities.unregister(entity_key);
            true
        } else {
            false
        }
    }

    /// Adds a new metrics snapshot to the aggregator for the given key.
    pub fn accumulate_metric_set_snapshot(
        &self,
        metric_set_key: MetricSetKey,
        metrics: &[MetricValue],
    ) {
        self.registry
            .lock()
            .metrics
            .accumulate_snapshot(metric_set_key, metrics);
    }

    /// Returns the total number of registered metric sets.
    #[must_use]
    pub fn metric_set_count(&self) -> usize {
        self.registry.lock().metrics.len()
    }

    /// Visits metric sets, yields a zero-alloc iterator
    /// of (MetricsField, value), then resets the values to zero.
    pub fn visit_metrics_and_reset<F>(&self, f: F)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        self.visit_metrics_and_reset_with_zeroes(f, false);
    }

    /// Visits metric sets, yields a zero-alloc iterator
    /// of (MetricsField, value), then resets the values to zero.
    /// Retains zero-valued metrics if `keep_all_zeroes` is true.
    pub fn visit_metrics_and_reset_with_zeroes<F>(&self, f: F, keep_all_zeroes: bool)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        let mut reg = self.registry.lock();
        let TelemetryRegistry { entities, metrics } = &mut *reg;
        metrics.visit_metrics_and_reset(entities, f, keep_all_zeroes);
    }

    /// Generates a SemConvRegistry from the current MetricSetRegistry.
    /// AttributeFields are deduplicated based on their key.
    #[must_use]
    pub fn generate_semconv_registry(&self) -> SemConvRegistry {
        let reg = self.registry.lock();
        reg.metrics.generate_semconv_registry(&reg.entities)
    }

    /// Visits current metric sets without resetting them.
    /// This is useful for read-only access to metrics for HTTP endpoints.
    pub fn visit_current_metrics<F>(&self, f: F)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        self.visit_current_metrics_with_zeroes(f, false);
    }

    /// Visits current metric sets without resetting them, with optional zero retention.
    /// This is useful for read-only access to metrics for HTTP endpoints.
    pub fn visit_current_metrics_with_zeroes<F>(&self, mut f: F, keep_all_zeroes: bool)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        let reg = self.registry.lock();
        for entry in reg.metrics.metrics.values() {
            let values = &entry.metric_values;
            if keep_all_zeroes || values.iter().any(|&v| !v.is_zero()) {
                let desc = entry.metrics_descriptor;
                if let Some(attrs) = reg.entities.get(entry.entity_key) {
                    f(desc, attrs, MetricsIterator::new(desc.metrics, values));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, Instrument, MetricValueType,
        MetricsField, Temporality,
    };
    use std::fmt::Debug;

    // Mock implementations for testing
    #[derive(Debug)]
    struct MockMetricSet {
        values: Vec<MetricValue>,
    }

    impl MockMetricSet {
        fn new() -> Self {
            Self {
                values: vec![MetricValue::from(0u64), MetricValue::from(0u64)],
            }
        }
    }

    impl Default for MockMetricSet {
        fn default() -> Self {
            Self::new()
        }
    }

    static MOCK_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test_metrics",
        metrics: &[
            MetricsField {
                name: "counter1",
                unit: "1",
                brief: "Test counter 1",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "counter2",
                unit: "1",
                brief: "Test counter 2",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
        ],
    };

    static MOCK_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test_attributes",
        fields: &[AttributeField {
            key: "test_key",
            r#type: AttributeValueType::String,
            brief: "Test attribute",
        }],
    };

    impl MetricSetHandler for MockMetricSet {
        fn descriptor(&self) -> &'static MetricsDescriptor {
            &MOCK_METRICS_DESCRIPTOR
        }

        fn snapshot_values(&self) -> Vec<MetricValue> {
            self.values.clone()
        }

        fn clear_values(&mut self) {
            self.values.iter_mut().for_each(MetricValue::reset);
        }

        fn needs_flush(&self) -> bool {
            self.values.iter().any(|&v| !v.is_zero())
        }
    }

    #[derive(Debug)]
    struct MockAttributeSet {
        // Store the attribute values as owned data that we can return references to
        attribute_values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(value: String) -> Self {
            let attribute_values = vec![AttributeValue::String(value)];
            Self { attribute_values }
        }
    }

    impl AttributeSetHandler for MockAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &MOCK_ATTRIBUTES_DESCRIPTOR
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.attribute_values
        }
    }

    #[test]
    fn test_telemetry_registry_new() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        assert_eq!(telemetry_registry.metric_set_count(), 0);
        assert_eq!(telemetry_registry.entity_count(), 0);
    }

    #[test]
    fn test_telemetry_registry_clone_shared_state() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let telemetry_registry_clone = telemetry_registry.clone();

        let attrs = MockAttributeSet::new("test_value".to_string());
        let _metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);

        assert_eq!(telemetry_registry.metric_set_count(), 1);
        assert_eq!(telemetry_registry_clone.metric_set_count(), 1);
        assert_eq!(telemetry_registry.entity_count(), 1);
        assert_eq!(telemetry_registry_clone.entity_count(), 1);
    }

    #[test]
    fn test_telemetry_registry_concurrent_access() {
        use std::thread;

        let telemetry_registry = TelemetryRegistryHandle::new();
        let mut handles = Vec::new();

        for i in 0u64..5 {
            let telemetry_registry_clone = telemetry_registry.clone();
            let thread_handle = thread::spawn(move || {
                let attrs = MockAttributeSet::new(format!("value_{i}"));
                let metric_set: MetricSet<MockMetricSet> =
                    telemetry_registry_clone.register_metric_set(attrs);
                let metrics_key = metric_set.key;

                telemetry_registry_clone.accumulate_metric_set_snapshot(
                    metrics_key,
                    &[MetricValue::from(i * 10), MetricValue::from(i * 20)],
                );
            });
            handles.push(thread_handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(telemetry_registry.metric_set_count(), 5);
        assert_eq!(telemetry_registry.entity_count(), 5);
    }
}
