// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The telemetry registry component combining entity and metrics registries (see the ITS diagram
//! architecture in the main [README.md](../README.md) file of this crate.
//!
//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: Concrete metrics live in their respective crates; this registry aggregates them via
//! dynamic dispatch.

use crate::attributes::AttributeSetHandler;
use crate::descriptor::MetricsDescriptor;
use crate::entity::EntityRegistry;
use crate::metrics::{
    MetricSet, MetricSetHandler, MetricSetRegistry, MetricValue, MetricsIterator,
};
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
    pub fn register_entity(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> EntityKey {
        self.registry.lock().entities.register(attrs)
    }

    /// Unregisters an entity by key.
    #[must_use]
    pub fn unregister_entity(&self, entity_key: EntityKey) -> bool {
        self.registry.lock().entities.unregister(entity_key)
    }

    /// Returns the total number of registered entities.
    #[must_use]
    pub fn entity_len(&self) -> usize {
        self.registry.lock().entities.len()
    }

    /// Returns true if there are no registered entities.
    #[must_use]
    pub fn entity_is_empty(&self) -> bool {
        self.entity_len() == 0
    }

    /// Visits all registered entities.
    pub fn visit_entities<F>(&self, f: F)
    where
        F: FnMut(EntityKey, &dyn AttributeSetHandler),
    {
        self.registry.lock().entities.visit_entities(f);
    }

    /// Returns a shared attribute set handle for the given key.
    #[must_use]
    pub fn get_entity_attributes(
        &self,
        entity_key: EntityKey,
    ) -> Option<Arc<dyn AttributeSetHandler + Send + Sync>> {
        self.registry
            .lock()
            .entities
            .get_shared(entity_key)
            .map(|attrs| {
                let attrs: Arc<dyn AttributeSetHandler + Send + Sync> = attrs;
                attrs
            })
    }

    /// Registers a metric set type for the provided entity key.
    #[must_use]
    pub fn register_metric_set_with_entity<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        entity_key: EntityKey,
    ) -> MetricSet<T> {
        let mut registry = self.registry.lock();
        debug_assert!(
            registry.entities.get(entity_key).is_some(),
            "entity key must exist in registry"
        );
        registry.metrics.register(entity_key)
    }

    /// Registers a metric set type with the given static attributes and returns a `MetricSet`
    /// instance that can be used to report metrics for that type.
    pub fn register_metric_set<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> MetricSet<T> {
        let mut registry = self.registry.lock();
        let entity_key = registry.entities.register(attrs);
        registry.metrics.register(entity_key)
    }

    /// Unregisters a metric set by key.
    #[must_use]
    pub fn unregister_metric_set(&self, metrics_key: MetricSetKey) -> bool {
        self.registry.lock().metrics.unregister(metrics_key)
    }

    /// Adds a new metrics snapshot to the aggregator for the given key.
    pub fn accumulate_snapshot(&self, metrics_key: MetricSetKey, metrics: &[MetricValue]) {
        self.registry
            .lock()
            .metrics
            .accumulate_snapshot(metrics_key, metrics);
    }

    /// Returns the total number of registered metrics sets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.registry.lock().metrics.len()
    }

    /// Returns true if there are no registered metrics sets.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
                values: vec![MetricValue::U64(0), MetricValue::U64(0)],
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
            let attribute_values = vec![AttributeValue::String(value.clone())];
            Self { attribute_values }
        }
    }

    impl AttributeSetHandler for MockAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &MOCK_ATTRIBUTES_DESCRIPTOR
        }

        fn iter_attributes<'a>(&'a self) -> crate::attributes::AttributeIterator<'a> {
            crate::attributes::AttributeIterator::new(
                MOCK_ATTRIBUTES_DESCRIPTOR.fields,
                &self.attribute_values,
            )
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.attribute_values
        }
    }

    #[test]
    fn test_metrics_registry_new() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        assert_eq!(telemetry_registry.len(), 0);
    }

    #[test]
    fn test_entity_registry_register_dedupes() {
        let telemetry_registry = TelemetryRegistryHandle::new();

        let key1 = telemetry_registry.register_entity(MockAttributeSet::new("value".to_string()));
        let key2 = telemetry_registry.register_entity(MockAttributeSet::new("value".to_string()));

        assert_eq!(key1, key2);
        assert_eq!(telemetry_registry.entity_len(), 1);
    }

    #[test]
    fn test_entity_registry_dedupes_sorted_attributes() {
        static SORTED_DESCRIPTOR_A: AttributesDescriptor = AttributesDescriptor {
            name: "sorted_attrs_a",
            fields: &[
                AttributeField {
                    key: "alpha",
                    r#type: AttributeValueType::String,
                    brief: "alpha",
                },
                AttributeField {
                    key: "beta",
                    r#type: AttributeValueType::Int,
                    brief: "beta",
                },
            ],
        };

        static SORTED_DESCRIPTOR_B: AttributesDescriptor = AttributesDescriptor {
            name: "sorted_attrs_b",
            fields: &[
                AttributeField {
                    key: "beta",
                    r#type: AttributeValueType::Int,
                    brief: "beta",
                },
                AttributeField {
                    key: "alpha",
                    r#type: AttributeValueType::String,
                    brief: "alpha",
                },
            ],
        };

        #[derive(Debug)]
        struct OrderedAttributeSetA {
            values: Vec<AttributeValue>,
        }

        #[derive(Debug)]
        struct OrderedAttributeSetB {
            values: Vec<AttributeValue>,
        }

        impl AttributeSetHandler for OrderedAttributeSetA {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &SORTED_DESCRIPTOR_A
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        impl AttributeSetHandler for OrderedAttributeSetB {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &SORTED_DESCRIPTOR_B
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
        }

        let telemetry_registry = TelemetryRegistryHandle::new();

        let key1 = telemetry_registry.register_entity(OrderedAttributeSetA {
            values: vec![
                AttributeValue::String("value".to_string()),
                AttributeValue::Int(7),
            ],
        });
        let key2 = telemetry_registry.register_entity(OrderedAttributeSetB {
            values: vec![
                AttributeValue::Int(7),
                AttributeValue::String("value".to_string()),
            ],
        });

        assert_eq!(key1, key2);
        assert_eq!(telemetry_registry.entity_len(), 1);
    }

    #[test]
    fn test_entity_registry_get_attributes() {
        let telemetry_registry = TelemetryRegistryHandle::new();

        let key = telemetry_registry.register_entity(MockAttributeSet::new("value".to_string()));
        let attrs = telemetry_registry
            .get_entity_attributes(key)
            .expect("missing attributes");

        let collected: Vec<_> = attrs.iter_attributes().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, "test_key");
        assert_eq!(*collected[0].1, AttributeValue::String("value".to_string()));
    }

    #[test]
    fn test_entity_registry_unregister() {
        let telemetry_registry = TelemetryRegistryHandle::new();

        let key = telemetry_registry.register_entity(MockAttributeSet::new("value".to_string()));

        assert!(telemetry_registry.unregister_entity(key));
        assert_eq!(telemetry_registry.entity_len(), 0);
        assert!(telemetry_registry.get_entity_attributes(key).is_none());
        assert!(!telemetry_registry.unregister_entity(key));
    }

    #[test]
    fn test_metrics_registry_register_with_entity_key() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let entity_key =
            telemetry_registry.register_entity(MockAttributeSet::new("value".to_string()));

        let metric_set: MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set_with_entity(entity_key);
        assert_eq!(metric_set.entity_key(), entity_key);
    }

    #[test]
    fn test_metrics_registry_unregister() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);
        let metrics_key = metric_set.key;

        assert!(telemetry_registry.unregister_metric_set(metrics_key));
        assert_eq!(telemetry_registry.len(), 0);
        assert!(!telemetry_registry.unregister_metric_set(metrics_key));
    }

    #[test]
    fn test_metrics_registry_register() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let _metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);
        assert_eq!(telemetry_registry.len(), 1);
    }

    #[test]
    fn test_metrics_registry_multiple_registrations() {
        let telemetry_registry = TelemetryRegistryHandle::new();

        let attrs1 = MockAttributeSet::new("value1".to_string());
        let attrs2 = MockAttributeSet::new("value2".to_string());

        let _metric_set1: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs1);
        let _metric_set2: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs2);

        assert_eq!(telemetry_registry.len(), 2);
    }

    #[test]
    fn test_accumulate_snapshot_basic() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);
        let metrics_key = metric_set.key;

        // Accumulate some values
        telemetry_registry
            .accumulate_snapshot(metrics_key, &[MetricValue::U64(10), MetricValue::U64(20)]);
        telemetry_registry
            .accumulate_snapshot(metrics_key, &[MetricValue::U64(5), MetricValue::U64(15)]);

        // Values should be accumulated
        let mut accumulated_values = Vec::new();
        telemetry_registry.visit_metrics_and_reset(|_desc, _attrs, iter| {
            for (_field, value) in iter {
                accumulated_values.push(value);
            }
        });

        assert_eq!(
            accumulated_values,
            vec![MetricValue::U64(15), MetricValue::U64(35)]
        );
    }

    #[test]
    fn test_accumulate_snapshot_invalid_key() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let invalid_key = MetricSetKey::default();

        // This should not panic, just ignore the invalid key
        telemetry_registry
            .accumulate_snapshot(invalid_key, &[MetricValue::U64(10), MetricValue::U64(20)]);
        assert_eq!(telemetry_registry.len(), 0);
    }

    #[cfg(feature = "unchecked-arithmetic")]
    #[test]
    fn test_accumulate_snapshot_overflow_wrapping() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);
        let metrics_key = metric_set.key;

        // Test wrapping behavior with overflow
        telemetry_registry.accumulate_snapshot(
            metrics_key,
            &[MetricValue::U64(u64::MAX), MetricValue::U64(u64::MAX - 5)],
        );
        telemetry_registry
            .accumulate_snapshot(metrics_key, &[MetricValue::U64(10), MetricValue::U64(10)]);

        let mut accumulated_values = Vec::new();
        telemetry_registry.visit_metrics_and_reset(|_desc, _attrs, iter| {
            for (_field, value) in iter {
                accumulated_values.push(value);
            }
        });

        // Should wrap around: u64::MAX + 10 = 9, (u64::MAX - 5) + 10 = 4
        assert_eq!(
            accumulated_values,
            vec![MetricValue::U64(9), MetricValue::U64(4)]
        );
    }

    #[cfg(not(feature = "unchecked-arithmetic"))]
    #[test]
    #[should_panic]
    fn test_accumulate_snapshot_overflow_panic() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);
        let metrics_key = metric_set.key;

        // This should panic on overflow when unchecked-arithmetic is disabled
        telemetry_registry.accumulate_snapshot(metrics_key, &[MetricValue::U64(u64::MAX)]);
        telemetry_registry.accumulate_snapshot(metrics_key, &[MetricValue::U64(1)]);
    }

    #[test]
    fn test_visit_metrics_and_reset() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);
        let metrics_key = metric_set.key;

        // Add some metrics
        telemetry_registry
            .accumulate_snapshot(metrics_key, &[MetricValue::U64(100), MetricValue::U64(0)]);

        let mut visit_count = 0;
        let mut collected_values = Vec::new();

        telemetry_registry.visit_metrics_and_reset(|desc, _attrs, iter| {
            visit_count += 1;
            assert_eq!(desc.name, "test_metrics");

            for (field, value) in iter {
                collected_values.push((field.name, value));
            }
        });

        assert_eq!(visit_count, 1);
        assert_eq!(
            collected_values,
            vec![
                ("counter1", MetricValue::U64(100)),
                ("counter2", MetricValue::U64(0))
            ]
        );

        // After reset, should not visit again
        visit_count = 0;
        collected_values.clear();

        telemetry_registry.visit_metrics_and_reset(|_desc, _attrs, _iter| {
            visit_count += 1;
        });

        assert_eq!(visit_count, 0);
    }

    #[test]
    fn test_metrics_iterator() {
        let fields = &[
            MetricsField {
                name: "metric1",
                unit: "1",
                brief: "Test metric 1",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "metric2",
                unit: "1",
                brief: "Test metric 2",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
        ];

        let values = [
            MetricValue::U64(0),
            MetricValue::U64(5),
            MetricValue::U64(0),
            MetricValue::U64(10),
            MetricValue::U64(0),
        ];
        let mut iter = MetricsIterator::new(fields, &values[..2]);

        let item1 = iter.next().unwrap();
        assert_eq!(item1.0.name, "metric1");
        assert_eq!(item1.1, MetricValue::U64(0));

        let item2 = iter.next().unwrap();
        assert_eq!(item2.0.name, "metric2");
        assert_eq!(item2.1, MetricValue::U64(5));

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_metrics_iterator_size_hint() {
        let fields = &[MetricsField {
            name: "metric1",
            unit: "1",
            brief: "Test metric 1",
            instrument: Instrument::Counter,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::U64,
        }];

        let values = [MetricValue::U64(10)];
        let iter = MetricsIterator::new(fields, &values);

        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 1);
        assert_eq!(upper, Some(1));
    }

    #[test]
    fn test_metrics_iterator_fused() {
        let fields = &[MetricsField {
            name: "metric1",
            unit: "1",
            brief: "Test metric 1",
            instrument: Instrument::Counter,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::U64,
        }];

        let values = [MetricValue::U64(10)];
        let mut iter = MetricsIterator::new(fields, &values);

        // Consume the iterator
        let _first = iter.next();

        // Should consistently return None after exhaustion
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_metrics_registry_clone() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let telemetry_registry_clone = telemetry_registry.clone();

        let attrs = MockAttributeSet::new("test_value".to_string());
        let _metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);

        // Both handles should see the same registry
        assert_eq!(telemetry_registry.len(), 1);
        assert_eq!(telemetry_registry_clone.len(), 1);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let telemetry_registry = TelemetryRegistryHandle::new();
        let mut handles = Vec::new();

        // Spawn multiple threads to test concurrent access
        for i in 0..5 {
            let telemetry_registry_clone = telemetry_registry.clone();
            let thread_handle = thread::spawn(move || {
                let attrs = MockAttributeSet::new(format!("value_{i}"));
                let metric_set: MetricSet<MockMetricSet> =
                    telemetry_registry_clone.register_metric_set(attrs);
                let metrics_key = metric_set.key;

                // Accumulate some values
                telemetry_registry_clone.accumulate_snapshot(
                    metrics_key,
                    &[MetricValue::U64(i * 10), MetricValue::U64(i * 20)],
                );
            });
            handles.push(thread_handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(telemetry_registry.len(), 5);
    }

    #[test]
    fn test_accumulate_snapshot_gauge_replaces_counter_accumulates() {
        #[derive(Debug)]
        struct MockGaugeMetricSet {
            values: Vec<MetricValue>,
        }

        impl MockGaugeMetricSet {
            fn new() -> Self {
                Self {
                    values: vec![MetricValue::U64(0), MetricValue::U64(0)],
                }
            }
        }

        impl Default for MockGaugeMetricSet {
            fn default() -> Self {
                Self::new()
            }
        }

        static MOCK_GAUGE_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
            name: "test_gauge_metrics",
            metrics: &[
                MetricsField {
                    name: "gauge1",
                    unit: "1",
                    brief: "Test gauge 1",
                    instrument: Instrument::Gauge,
                    temporality: None,
                    value_type: MetricValueType::U64,
                },
                MetricsField {
                    name: "counter1",
                    unit: "1",
                    brief: "Test counter 1",
                    instrument: Instrument::Counter,
                    temporality: Some(Temporality::Delta),
                    value_type: MetricValueType::U64,
                },
            ],
        };

        impl MetricSetHandler for MockGaugeMetricSet {
            fn descriptor(&self) -> &'static MetricsDescriptor {
                &MOCK_GAUGE_METRICS_DESCRIPTOR
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

        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());
        let metric_set: MetricSet<MockGaugeMetricSet> =
            telemetry_registry.register_metric_set(attrs);
        let key = metric_set.key;

        // First snapshot sets gauge=5, counter+=10.
        telemetry_registry.accumulate_snapshot(key, &[MetricValue::U64(5), MetricValue::U64(10)]);
        // Second snapshot sets gauge=2 (replaces), counter+=3 (accumulates).
        telemetry_registry.accumulate_snapshot(key, &[MetricValue::U64(2), MetricValue::U64(3)]);

        let mut values = Vec::new();
        telemetry_registry.visit_current_metrics(|_desc, _attrs, iter| {
            for (_field, value) in iter {
                values.push(value);
            }
        });

        assert_eq!(values, vec![MetricValue::U64(2), MetricValue::U64(13)]);
    }

    #[test]
    fn test_accumulate_snapshot_observe_counter_replaces() {
        #[derive(Debug)]
        struct MockCumulativeCounterMetricSet {
            values: Vec<MetricValue>,
        }

        impl MockCumulativeCounterMetricSet {
            fn new() -> Self {
                Self {
                    values: vec![MetricValue::U64(0)],
                }
            }
        }

        impl Default for MockCumulativeCounterMetricSet {
            fn default() -> Self {
                Self::new()
            }
        }

        static MOCK_OBSERVED_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
            name: "test_observed_metrics",
            metrics: &[MetricsField {
                name: "counter1",
                unit: "1",
                brief: "Test counter 1",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Cumulative),
                value_type: MetricValueType::U64,
            }],
        };

        impl MetricSetHandler for MockCumulativeCounterMetricSet {
            fn descriptor(&self) -> &'static MetricsDescriptor {
                &MOCK_OBSERVED_METRICS_DESCRIPTOR
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

        let telemetry_registry = TelemetryRegistryHandle::new();
        let metric_set: MetricSet<MockCumulativeCounterMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr".to_string()));
        let metrics_key = metric_set.key;

        telemetry_registry.accumulate_snapshot(metrics_key, &[MetricValue::U64(10)]);
        telemetry_registry.accumulate_snapshot(metrics_key, &[MetricValue::U64(15)]);

        let mut collected = Vec::new();
        telemetry_registry.visit_metrics_and_reset(|_desc, _attrs, iter| {
            for (_field, value) in iter {
                collected.push(value);
            }
        });

        assert_eq!(collected, vec![MetricValue::U64(15)]);
    }
}
