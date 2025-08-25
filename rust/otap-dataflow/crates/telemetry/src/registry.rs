// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: Concrete metrics live in their respective crates; this registry aggregates them via
//! dynamic dispatch.

use crate::attributes::AttributeSetHandler;
use crate::descriptor::MetricsDescriptor;
use crate::descriptor::MetricsField;
use crate::metrics::{MetricSet, MetricSetHandler};
use crate::semconv::SemConvRegistry;
use parking_lot::Mutex;
use slotmap::{SlotMap, new_key_type};
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

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
    #[must_use]
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

/// Lightweight iterator over metrics (no heap allocs).
pub struct MetricsIterator<'a> {
    fields: &'static [MetricsField],
    values: &'a [u64],
    idx: usize,
    len: usize,
}

impl<'a> MetricsIterator<'a> {
    #[inline]
    fn new(fields: &'static [MetricsField], values: &'a [u64]) -> Self {
        let len = values.len();
        debug_assert_eq!(
            fields.len(),
            len,
            "descriptor.fields and metric values length must match"
        );
        Self {
            fields,
            values,
            idx: 0,
            len,
        }
    }
}

impl<'a> Iterator for MetricsIterator<'a> {
    type Item = (&'static MetricsField, u64);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Single bound check: emit every metric (including zeros).
        if self.idx >= self.len {
            return None;
        }
        let i = self.idx;
        self.idx = i + 1;

        // SAFETY: `i < self.len` and `self.len == self.fields.len() == self.values.len()` by construction.
        let v = {
            #[cfg(feature = "unchecked-index")]
            unsafe {
                *self.values.get_unchecked(i)
            }
            #[cfg(not(feature = "unchecked-index"))]
            {
                self.values[i]
            }
        };

        let field = {
            #[cfg(feature = "unchecked-index")]
            unsafe {
                self.fields.get_unchecked(i)
            }
            #[cfg(not(feature = "unchecked-index"))]
            {
                &self.fields[i]
            }
        };

        Some((field, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Exact remaining length now that we yield all elements.
        let rem = self.len.saturating_sub(self.idx);
        (rem, Some(rem))
    }
}

impl<'a> ExactSizeIterator for MetricsIterator<'a> {}

// This iterator is "fused": once `next()` returns `None`, it will always return `None`.
// Rationale:
// - `idx` increases monotonically up to `len` and is never reset.
// - No internal state can make new items appear after exhaustion.
// Benefit:
// - Allows iterator adaptors (e.g. `chain`) to skip redundant checks after exhaustion,
//   and callers do not need to wrap with `iter.fuse()`.

// Note: This marker trait does not change behavior. It only encodes the guarantee.
impl<'a> core::iter::FusedIterator for MetricsIterator<'a> {}

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
    /// Registers a metric set type with the given static attributes and returns a `MetricSet`
    /// instance that can be used to report metrics for that type.
    fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &mut self,
        static_attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> MetricSet<T> {
        let metrics = T::default();
        let descriptor = metrics.descriptor();

        let metrics_key = self.metrics.insert(MetricsEntry::new(
            descriptor,
            static_attrs.descriptor(),
            metrics.snapshot_values(),
            Box::new(static_attrs),
        ));

        MetricSet {
            key: metrics_key,
            metrics,
        }
    }

    /// Merges a metrics snapshot (delta) into the registered instance keyed by `metrics_key`.
    fn accumulate_snapshot(&mut self, metrics_key: MetricsKey, metrics_values: &[u64]) {
        if let Some(entry) = self.metrics.get_mut(metrics_key) {
            entry
                .metric_values
                .iter_mut()
                .zip(metrics_values)
                .for_each(|(e, v)| {
                    #[cfg(feature = "unchecked-arithmetic")]
                    {
                        // SAFETY: Metric values are expected to be well-behaved and not overflow
                        // in typical telemetry scenarios. This is a performance optimization for
                        // hot path metric accumulation.
                        *e = e.wrapping_add(*v);
                    }
                    #[cfg(not(feature = "unchecked-arithmetic"))]
                    {
                        *e += v;
                    }
                });
        } else {
            // TODO: consider logging missing key
        }
    }

    /// Returns the total number of registered metrics sets.
    fn len(&self) -> usize {
        self.metrics.len()
    }

    /// Visits only metric sets, yields a zero-alloc iterator
    /// of (MetricsField, value), then resets the values to zero.
    pub(crate) fn visit_metrics_and_reset<F>(&mut self, mut f: F)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        for entry in self.metrics.values_mut() {
            let values = &mut entry.metric_values;
            if values.iter().any(|&v| v != 0) {
                let desc = entry.metrics_descriptor;
                let attrs = entry.attribute_values.as_ref();

                f(desc, attrs, MetricsIterator::new(desc.metrics, values));

                // Zero after reporting.
                values.iter_mut().for_each(|v| *v = 0);
            }
        }
    }

    /// Generates a SemConvRegistry from the current MetricsRegistry.
    /// AttributeFields are deduplicated based on their key.
    #[must_use]
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

impl Default for MetricsRegistryHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistryHandle {
    /// Creates a new `MetricsRegistryHandle`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            metric_registry: Arc::new(Mutex::new(MetricsRegistry {
                metrics: SlotMap::default(),
            })),
        }
    }

    /// Registers a metric set type with the given static attributes and returns a `MetricSet`
    /// instance that can be used to report metrics for that type.
    pub fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> MetricSet<T> {
        self.metric_registry.lock().register(attrs)
    }

    /// Adds a new metrics snapshot to the aggregator for the given key.
    pub fn accumulate_snapshot(&self, metrics_key: MetricsKey, metrics: &[u64]) {
        self.metric_registry
            .lock()
            .accumulate_snapshot(metrics_key, metrics);
    }

    /// Returns the total number of registered metrics sets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.metric_registry.lock().len()
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
        let mut reg = self.metric_registry.lock();
        reg.visit_metrics_and_reset(f);
    }

    /// Generates a SemConvRegistry from the current MetricsRegistry.
    /// AttributeFields are deduplicated based on their key.
    #[must_use]
    pub fn generate_semconv_registry(&self) -> SemConvRegistry {
        self.metric_registry.lock().generate_semconv_registry()
    }

    /// Visits current metric sets without resetting them.
    /// This is useful for read-only access to metrics for HTTP endpoints.
    pub fn visit_current_metrics<F>(&self, mut f: F)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        let reg = self.metric_registry.lock();
        for entry in reg.metrics.values() {
            let values = &entry.metric_values;
            if values.iter().any(|&v| v != 0) {
                let desc = entry.metrics_descriptor;
                let attrs = entry.attribute_values.as_ref();

                f(desc, attrs, MetricsIterator::new(desc.metrics, values));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor, Instrument};
    use std::fmt::Debug;

    // Mock implementations for testing
    #[derive(Debug)]
    struct MockMetricSet {
        values: Vec<u64>,
    }

    impl MockMetricSet {
        fn new() -> Self {
            Self {
                values: vec![0, 0], // Initialize with 2 values to match MOCK_METRICS_DESCRIPTOR
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
            },
            MetricsField {
                name: "counter2",
                unit: "1",
                brief: "Test counter 2",
                instrument: Instrument::Counter,
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

        fn snapshot_values(&self) -> Vec<u64> {
            self.values.clone()
        }

        fn clear_values(&mut self) {
            self.values.iter_mut().for_each(|v| *v = 0);
        }

        fn needs_flush(&self) -> bool {
            self.values.iter().any(|&v| v != 0)
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
        let handle = MetricsRegistryHandle::new();
        assert_eq!(handle.len(), 0);
    }

    #[test]
    fn test_metrics_registry_register() {
        let handle = MetricsRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let _metric_set: MetricSet<MockMetricSet> = handle.register(attrs);
        assert_eq!(handle.len(), 1);
    }

    #[test]
    fn test_metrics_registry_multiple_registrations() {
        let handle = MetricsRegistryHandle::new();

        let attrs1 = MockAttributeSet::new("value1".to_string());
        let attrs2 = MockAttributeSet::new("value2".to_string());

        let _metric_set1: MetricSet<MockMetricSet> = handle.register(attrs1);
        let _metric_set2: MetricSet<MockMetricSet> = handle.register(attrs2);

        assert_eq!(handle.len(), 2);
    }

    #[test]
    fn test_accumulate_snapshot_basic() {
        let handle = MetricsRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = handle.register(attrs);
        let metrics_key = metric_set.key;

        // Accumulate some values
        handle.accumulate_snapshot(metrics_key, &[10, 20]);
        handle.accumulate_snapshot(metrics_key, &[5, 15]);

        // Values should be accumulated
        let mut accumulated_values = Vec::new();
        handle.visit_metrics_and_reset(|_desc, _attrs, iter| {
            for (_field, value) in iter {
                accumulated_values.push(value);
            }
        });

        assert_eq!(accumulated_values, vec![15, 35]);
    }

    #[test]
    fn test_accumulate_snapshot_invalid_key() {
        let handle = MetricsRegistryHandle::new();
        let invalid_key = MetricsKey::default();

        // This should not panic, just ignore the invalid key
        handle.accumulate_snapshot(invalid_key, &[10, 20]);
        assert_eq!(handle.len(), 0);
    }

    #[cfg(feature = "unchecked-arithmetic")]
    #[test]
    fn test_accumulate_snapshot_overflow_wrapping() {
        let handle = MetricsRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = handle.register(attrs);
        let metrics_key = metric_set.key;

        // Test wrapping behavior with overflow
        handle.accumulate_snapshot(metrics_key, &[u64::MAX, u64::MAX - 5]);
        handle.accumulate_snapshot(metrics_key, &[10, 10]);

        let mut accumulated_values = Vec::new();
        handle.visit_metrics_and_reset(|_desc, _attrs, iter| {
            for (_field, value) in iter {
                accumulated_values.push(value);
            }
        });

        // Should wrap around: u64::MAX + 10 = 9, (u64::MAX - 5) + 10 = 4
        assert_eq!(accumulated_values, vec![9, 4]);
    }

    #[cfg(not(feature = "unchecked-arithmetic"))]
    #[test]
    #[should_panic]
    fn test_accumulate_snapshot_overflow_panic() {
        let handle = MetricsRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = handle.register(attrs);
        let metrics_key = metric_set.key;

        // This should panic on overflow when unchecked-arithmetic is disabled
        handle.accumulate_snapshot(metrics_key, &[u64::MAX]);
        handle.accumulate_snapshot(metrics_key, &[1]);
    }

    #[test]
    fn test_visit_metrics_and_reset() {
        let handle = MetricsRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = handle.register(attrs);
        let metrics_key = metric_set.key;

        // Add some metrics
        handle.accumulate_snapshot(metrics_key, &[100, 0]);

        let mut visit_count = 0;
        let mut collected_values = Vec::new();

        handle.visit_metrics_and_reset(|desc, _attrs, iter| {
            visit_count += 1;
            assert_eq!(desc.name, "test_metrics");

            for (field, value) in iter {
                collected_values.push((field.name, value));
            }
        });

        assert_eq!(visit_count, 1);
        assert_eq!(collected_values, vec![("counter1", 100), ("counter2", 0)]);

        // After reset, should not visit again
        visit_count = 0;
        collected_values.clear();

        handle.visit_metrics_and_reset(|_desc, _attrs, _iter| {
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
            },
            MetricsField {
                name: "metric2",
                unit: "1",
                brief: "Test metric 2",
                instrument: Instrument::Counter,
            },
        ];

        let values = [0, 5, 0, 10, 0];
        let mut iter = MetricsIterator::new(fields, &values[..2]);

        let item1 = iter.next().unwrap();
        assert_eq!(item1.0.name, "metric1");
        assert_eq!(item1.1, 0);

        let item2 = iter.next().unwrap();
        assert_eq!(item2.0.name, "metric2");
        assert_eq!(item2.1, 5);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_metrics_iterator_size_hint() {
        let fields = &[MetricsField {
            name: "metric1",
            unit: "1",
            brief: "Test metric 1",
            instrument: Instrument::Counter,
        }];

        let values = [10];
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
        }];

        let values = [10];
        let mut iter = MetricsIterator::new(fields, &values);

        // Consume the iterator
        let _first = iter.next();

        // Should consistently return None after exhaustion
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_metrics_registry_clone() {
        let handle1 = MetricsRegistryHandle::new();
        let handle2 = handle1.clone();

        let attrs = MockAttributeSet::new("test_value".to_string());
        let _metric_set: MetricSet<MockMetricSet> = handle1.register(attrs);

        // Both handles should see the same registry
        assert_eq!(handle1.len(), 1);
        assert_eq!(handle2.len(), 1);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let handle = MetricsRegistryHandle::new();
        let mut handles = Vec::new();

        // Spawn multiple threads to test concurrent access
        for i in 0..5 {
            let handle_clone = handle.clone();
            let thread_handle = thread::spawn(move || {
                let attrs = MockAttributeSet::new(format!("value_{}", i));
                let metric_set: MetricSet<MockMetricSet> = handle_clone.register(attrs);
                let metrics_key = metric_set.key;

                // Accumulate some values
                handle_clone.accumulate_snapshot(metrics_key, &[i * 10, i * 20]);
            });
            handles.push(thread_handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(handle.len(), 5);
    }
}
