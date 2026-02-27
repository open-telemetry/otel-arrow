// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core multivariate metrics (aka metric set) traits and types + Metric Set Registry.
//!
//! This module intentionally contains no product-specific metrics definitions. Concrete metrics
//! live in their respective nodes/crates and implement the `MetricSetHandler` trait defined
//! here.

pub mod dispatcher;

use crate::attributes::AttributeSetHandler;
use crate::descriptor::{Instrument, MetricsDescriptor, MetricsField, Temporality};
use crate::entity::EntityRegistry;
use crate::instrument::MmscSnapshot;
use crate::registry::{EntityKey, MetricSetKey};
use crate::semconv::SemConvRegistry;
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// Numeric metric value — a scalar integer or float, or a pre-aggregated MMSC summary.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(variant_size_differences)] // Mmsc is 32 bytes vs 8 for scalars; acceptable for internal telemetry.
pub enum MetricValue {
    /// Unsigned 64-bit integer value.
    U64(u64),
    /// 64-bit floating point value.
    F64(f64),
    /// Pre-aggregated min/max/sum/count summary from an [`crate::instrument::Mmsc`] instrument.
    Mmsc(MmscSnapshot),
}

impl MetricValue {
    /// Returns `true` when the value is exactly zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        match self {
            MetricValue::U64(v) => v == 0,
            MetricValue::F64(v) => v == 0.0,
            MetricValue::Mmsc(s) => s.count == 0,
        }
    }

    /// Returns a zero value of the same variant.
    ///
    /// For `Mmsc`, the zero state uses sentinel values (`f64::MAX` for min,
    /// `f64::MIN` for max) so that subsequent merges work correctly.
    #[must_use]
    pub const fn zero_of_kind(self) -> Self {
        match self {
            MetricValue::U64(_) => MetricValue::U64(0),
            MetricValue::F64(_) => MetricValue::F64(0.0),
            MetricValue::Mmsc(_) => MetricValue::Mmsc(MmscSnapshot {
                min: f64::MAX,
                max: f64::MIN,
                sum: 0.0,
                count: 0,
            }),
        }
    }

    /// Adds another metric value to this one, converting between numeric kinds if needed.
    ///
    /// For scalars, this performs addition. For MMSC, this performs a
    /// merge (min of mins, max of maxes, sum of sums, count of counts).
    ///
    /// # Panics (debug only)
    /// Debug-asserts that both values are the same variant.
    pub const fn add_in_place(&mut self, other: MetricValue) {
        match other {
            MetricValue::U64(rhs) => match self {
                MetricValue::U64(lhs) => {
                    #[cfg(feature = "unchecked-arithmetic")]
                    {
                        *lhs = lhs.wrapping_add(rhs);
                    }
                    #[cfg(not(feature = "unchecked-arithmetic"))]
                    {
                        *lhs += rhs;
                    }
                }
                MetricValue::F64(lhs) => {
                    *lhs += rhs as f64;
                }
                MetricValue::Mmsc(_) => {
                    debug_assert!(false, "add_in_place: cannot add U64 to Mmsc");
                }
            },
            MetricValue::F64(rhs) => match self {
                MetricValue::U64(lhs) => {
                    *self = MetricValue::F64(*lhs as f64 + rhs);
                }
                MetricValue::F64(lhs) => {
                    *lhs += rhs;
                }
                MetricValue::Mmsc(_) => {
                    debug_assert!(false, "add_in_place: cannot add F64 to Mmsc");
                }
            },
            MetricValue::Mmsc(rhs) => match self {
                MetricValue::Mmsc(lhs) => {
                    lhs.min = lhs.min.min(rhs.min);
                    lhs.max = lhs.max.max(rhs.max);
                    lhs.sum += rhs.sum;
                    lhs.count += rhs.count;
                }
                _ => {
                    debug_assert!(false, "add_in_place: cannot add Mmsc to scalar");
                }
            },
        }
    }

    /// Resets the value to zero while keeping the numeric variant.
    pub const fn reset(&mut self) {
        *self = self.zero_of_kind();
    }

    /// Returns the floating-point representation of the value.
    ///
    /// This method is intended for **scalar** values only.
    /// For `Mmsc` variants, use [`MmscSnapshot`] fields directly.
    #[must_use]
    pub const fn to_f64(self) -> f64 {
        match self {
            MetricValue::U64(v) => v as f64,
            MetricValue::F64(v) => v,
            MetricValue::Mmsc(_) => {
                debug_assert!(false, "to_f64() called on Mmsc MetricValue");
                0.0
            }
        }
    }

    /// Converts the metric value to `u64`, lossy for floating-point values.
    ///
    /// This method is intended for **scalar** values only.
    /// For `Mmsc` variants, use [`MmscSnapshot`] fields directly.
    #[must_use]
    pub const fn to_u64_lossy(self) -> u64 {
        match self {
            MetricValue::U64(v) => v,
            MetricValue::F64(v) => v as u64,
            MetricValue::Mmsc(_) => {
                debug_assert!(false, "to_u64_lossy() called on Mmsc MetricValue");
                0
            }
        }
    }
}

impl From<u64> for MetricValue {
    fn from(value: u64) -> Self {
        MetricValue::U64(value)
    }
}

impl From<f64> for MetricValue {
    fn from(value: f64) -> Self {
        MetricValue::F64(value)
    }
}

impl std::ops::AddAssign for MetricValue {
    fn add_assign(&mut self, rhs: Self) {
        self.add_in_place(rhs);
    }
}

impl From<MmscSnapshot> for MetricValue {
    fn from(value: MmscSnapshot) -> Self {
        MetricValue::Mmsc(value)
    }
}

/// A concrete set of metrics values grouped under a single descriptor/key.
#[derive(Clone)]
pub struct MetricSet<M: MetricSetHandler> {
    pub(crate) key: MetricSetKey,
    pub(crate) entity_key: EntityKey,
    pub(crate) metrics: M,
}

impl<M: MetricSetHandler> MetricSet<M> {
    /// Creates a snapshot of the current metrics values.
    pub fn snapshot(&self) -> MetricSetSnapshot {
        MetricSetSnapshot {
            key: self.key,
            metrics: self.metrics.snapshot_values(),
        }
    }

    /// Returns the entity key associated with this metric set.
    #[must_use]
    pub const fn entity_key(&self) -> EntityKey {
        self.entity_key
    }

    /// Returns the metrics key associated with this metric set.
    #[must_use]
    pub const fn metrics_key(&self) -> MetricSetKey {
        self.key
    }

    /// Returns the metric set key associated with this metric set.
    #[must_use]
    pub const fn metric_set_key(&self) -> MetricSetKey {
        self.key
    }
}

impl<M: MetricSetHandler> Deref for MetricSet<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.metrics
    }
}
impl<M: MetricSetHandler> DerefMut for MetricSet<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.metrics
    }
}

impl<M: MetricSetHandler> From<MetricSet<M>> for MetricSetSnapshot {
    fn from(val: MetricSet<M>) -> Self {
        val.snapshot()
    }
}

/// Immutable snapshot of a metric set's current values.
#[derive(Debug)]
pub struct MetricSetSnapshot {
    pub(crate) key: MetricSetKey,
    pub(crate) metrics: Vec<MetricValue>,
}

impl MetricSetSnapshot {
    /// get a reference to the metric values
    #[must_use]
    pub fn get_metrics(&self) -> &[MetricValue] {
        &self.metrics
    }
}

/// Handler trait implemented by generated metric set structs (see 'metric_set' proc macro).
pub trait MetricSetHandler {
    /// Returns the static descriptor describing this metric set (name + ordered fields).
    fn descriptor(&self) -> &'static MetricsDescriptor;
    /// Returns a snapshot of all metric field values in descriptor order.
    fn snapshot_values(&self) -> Vec<MetricValue>;
    /// Resets all metric field values to zero.
    fn clear_values(&mut self);
    /// Returns true if at least one metric value is non-zero (fast path check).
    fn needs_flush(&self) -> bool;
}

/// A registered metrics entry containing all necessary information for metrics aggregation.
pub struct MetricsEntry {
    /// The static descriptor describing the metrics structure
    pub metrics_descriptor: &'static MetricsDescriptor,
    /// Current snapshot values stored as a vector
    pub metric_values: Vec<MetricValue>,

    /// Entity key for the associated attribute set
    pub entity_key: EntityKey,
}

impl Debug for MetricsEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsEntry")
            .field("metrics_descriptor", &self.metrics_descriptor)
            .field("metric_values", &self.metric_values)
            .field("entity_key", &self.entity_key)
            .finish()
    }
}

impl MetricsEntry {
    /// Creates a new metrics entry
    #[must_use]
    pub const fn new(
        metrics_descriptor: &'static MetricsDescriptor,
        metric_values: Vec<MetricValue>,
        entity_key: EntityKey,
    ) -> Self {
        Self {
            metrics_descriptor,
            metric_values,
            entity_key,
        }
    }
}

/// Lightweight iterator over metrics (no heap allocs).
pub struct MetricsIterator<'a> {
    fields: &'static [MetricsField],
    values: &'a [MetricValue],
    idx: usize,
    len: usize,
}

impl<'a> MetricsIterator<'a> {
    #[inline]
    pub(crate) fn new(fields: &'static [MetricsField], values: &'a [MetricValue]) -> Self {
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
    type Item = (&'static MetricsField, MetricValue);

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
            #[allow(unsafe_code)]
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
            #[allow(unsafe_code)]
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

/// This iterator is "fused": once `next()` returns `None`, it will always return `None`.
/// Rationale:
/// - `idx` increases monotonically up to `len` and is never reset.
/// - No internal state can make new items appear after exhaustion.
///
/// Benefit:
/// - Allows iterator adaptors (e.g. `chain`) to skip redundant checks after exhaustion,
///   and callers do not need to wrap with `iter.fuse()`.
///
/// Note: This marker trait does not change behavior. It only encodes the guarantee.
impl<'a> core::iter::FusedIterator for MetricsIterator<'a> {}

/// A metrics registry that maintains aggregated metrics for different entity keys.
#[derive(Default)]
pub struct MetricSetRegistry {
    pub(crate) metrics: SlotMap<MetricSetKey, MetricsEntry>,
}

impl Debug for MetricSetRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricSetRegistry")
            .field("metrics_len", &self.metrics.len())
            .finish()
    }
}

impl MetricSetRegistry {
    /// Registers a metric set type for the given entity and returns a `MetricSet`
    /// instance that can be used to report metrics for that type.
    pub(crate) fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &mut self,
        entity_key: EntityKey,
    ) -> MetricSet<T> {
        let metrics = T::default();
        let descriptor = metrics.descriptor();

        let metrics_key = self.metrics.insert(MetricsEntry::new(
            descriptor,
            metrics.snapshot_values(),
            entity_key,
        ));

        MetricSet {
            key: metrics_key,
            entity_key,
            metrics,
        }
    }

    /// Merges a metrics snapshot into the registered instance keyed by `metrics_key`.
    pub(crate) fn accumulate_snapshot(
        &mut self,
        metrics_key: MetricSetKey,
        metrics_values: &[MetricValue], // snapshot values
    ) {
        if let Some(entry) = self.metrics.get_mut(metrics_key) {
            debug_assert_eq!(
                entry.metrics_descriptor.metrics.len(),
                metrics_values.len(),
                "descriptor.metrics and snapshot values length must match"
            );

            entry
                .metric_values
                .iter_mut()
                .zip(metrics_values)
                .zip(entry.metrics_descriptor.metrics.iter())
                .for_each(|((current, incoming), field)| match field.instrument {
                    Instrument::Gauge => {
                        // Gauges report absolute values; replace.
                        *current = *incoming;
                    }
                    Instrument::Histogram | Instrument::Mmsc => {
                        // Histograms and MMSC instruments report per-interval changes.
                        current.add_in_place(*incoming);
                    }
                    Instrument::Counter | Instrument::UpDownCounter => match field.temporality {
                        Some(Temporality::Delta) => {
                            // Delta sums report per-interval changes => accumulate.
                            current.add_in_place(*incoming);
                        }
                        Some(Temporality::Cumulative) => {
                            // Cumulative sums report the current value => replace.
                            *current = *incoming;
                        }
                        None => {
                            debug_assert!(false, "sum-like instrument must have a temporality");
                            // Prefer replacing to avoid runaway accumulation if misconfigured.
                            *current = *incoming;
                        }
                    },
                });
        } else {
            // TODO: consider logging missing key
        }
    }

    pub(crate) fn unregister(&mut self, metrics_key: MetricSetKey) -> Option<EntityKey> {
        self.metrics
            .remove(metrics_key)
            .map(|entry| entry.entity_key)
    }

    /// Returns the total number of registered metrics sets.
    pub(crate) fn len(&self) -> usize {
        self.metrics.len()
    }

    /// Visits only metric sets, yields a zero-alloc iterator
    /// of (MetricsField, value), then resets the values to zero.
    pub(crate) fn visit_metrics_and_reset<F>(
        &mut self,
        entities: &EntityRegistry,
        mut f: F,
        keep_all_zeroes: bool,
    ) where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        for entry in self.metrics.values_mut() {
            let values = &mut entry.metric_values;
            if keep_all_zeroes || values.iter().any(|&v| !v.is_zero()) {
                let desc = entry.metrics_descriptor;
                if let Some(attrs) = entities.get(entry.entity_key) {
                    f(desc, attrs, MetricsIterator::new(desc.metrics, values));
                }

                // Zero after reporting.
                values.iter_mut().for_each(MetricValue::reset);
            }
        }
    }

    /// Generates a SemConvRegistry from the current MetricSetRegistry.
    /// AttributeFields are deduplicated based on their key.
    #[must_use]
    pub fn generate_semconv_registry(&self, entities: &EntityRegistry) -> SemConvRegistry {
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
            if let Some(entity) = entities.get(entry.entity_key) {
                for field in entity.descriptor().fields {
                    if unique_attributes.insert(field.key) {
                        attributes.push(field);
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, Instrument, MetricValueType,
        MetricsField, Temporality,
    };
    use crate::entity::EntityRegistry;
    use std::fmt::Debug;

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
        values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(value: String) -> Self {
            Self {
                values: vec![AttributeValue::String(value)],
            }
        }
    }

    impl AttributeSetHandler for MockAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &MOCK_ATTRIBUTES_DESCRIPTOR
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    fn register_entity(registry: &mut EntityRegistry, value: &str) -> EntityKey {
        // Note: tests do not distinguish outcomes, so this returns just the key().
        registry
            .register(MockAttributeSet::new(value.to_string()))
            .key()
    }

    #[test]
    fn test_register() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMetricSet> = metrics.register(entity_key);
        assert_eq!(metric_set.entity_key(), entity_key);
        assert_eq!(metrics.len(), 1);
    }

    #[test]
    fn test_unregister() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "test_value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        assert!(metrics.unregister(metrics_key).is_some());
        assert_eq!(metrics.len(), 0);
        assert!(metrics.unregister(metrics_key).is_none());
    }

    #[test]
    fn test_multiple_registrations() {
        let mut entities = EntityRegistry::default();
        let entity_key1 = register_entity(&mut entities, "value1");
        let entity_key2 = register_entity(&mut entities, "value2");
        let mut metrics = MetricSetRegistry::default();

        let _metric_set1: MetricSet<MockMetricSet> = metrics.register(entity_key1);
        let _metric_set2: MetricSet<MockMetricSet> = metrics.register(entity_key2);

        assert_eq!(metrics.len(), 2);
    }

    #[test]
    fn test_accumulate_snapshot_basic() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "test_value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(10), MetricValue::U64(20)]);
        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(5), MetricValue::U64(15)]);

        let mut accumulated_values = Vec::new();
        metrics.visit_metrics_and_reset(
            &entities,
            |_desc, _attrs, iter| {
                for (_field, value) in iter {
                    accumulated_values.push(value);
                }
            },
            false,
        );

        assert_eq!(
            accumulated_values,
            vec![MetricValue::U64(15), MetricValue::U64(35)]
        );
    }

    #[test]
    fn test_accumulate_snapshot_invalid_key() {
        let mut metrics = MetricSetRegistry::default();
        let invalid_key = MetricSetKey::default();

        metrics.accumulate_snapshot(invalid_key, &[MetricValue::U64(10), MetricValue::U64(20)]);
        assert_eq!(metrics.len(), 0);
    }

    #[cfg(feature = "unchecked-arithmetic")]
    #[test]
    fn test_accumulate_snapshot_overflow_wrapping() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "test_value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        metrics.accumulate_snapshot(
            metrics_key,
            &[MetricValue::U64(u64::MAX), MetricValue::U64(u64::MAX - 5)],
        );
        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(10), MetricValue::U64(10)]);

        let mut accumulated_values = Vec::new();
        metrics.visit_metrics_and_reset(
            &entities,
            |_desc, _attrs, iter| {
                for (_field, value) in iter {
                    accumulated_values.push(value);
                }
            },
            false,
        );

        assert_eq!(
            accumulated_values,
            vec![MetricValue::U64(9), MetricValue::U64(4)]
        );
    }

    #[cfg(not(feature = "unchecked-arithmetic"))]
    #[test]
    #[should_panic]
    fn test_accumulate_snapshot_overflow_panic() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "test_value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(u64::MAX)]);
        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(1)]);
    }

    #[test]
    fn test_visit_metrics_and_reset() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "test_value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(100), MetricValue::U64(0)]);

        let mut visit_count = 0;
        let mut collected_values = Vec::new();

        metrics.visit_metrics_and_reset(
            &entities,
            |desc, _attrs, iter| {
                visit_count += 1;
                assert_eq!(desc.name, "test_metrics");

                for (field, value) in iter {
                    collected_values.push((field.name, value));
                }
            },
            false,
        );

        assert_eq!(visit_count, 1);
        assert_eq!(
            collected_values,
            vec![
                ("counter1", MetricValue::U64(100)),
                ("counter2", MetricValue::U64(0))
            ]
        );

        visit_count = 0;
        collected_values.clear();

        metrics.visit_metrics_and_reset(
            &entities,
            |_desc, _attrs, _iter| {
                visit_count += 1;
            },
            false,
        );

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

        // Consume the single item
        assert!(iter.next().is_some());
        // After exhaustion, further calls must keep returning None (fused)
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
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

        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "test_value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockGaugeMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(5), MetricValue::U64(10)]);
        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(2), MetricValue::U64(3)]);

        let entry = metrics.metrics.get(metrics_key).expect("metric set entry");
        assert_eq!(
            entry.metric_values,
            vec![MetricValue::U64(2), MetricValue::U64(13)]
        );
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

        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "attr");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockCumulativeCounterMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(10)]);
        metrics.accumulate_snapshot(metrics_key, &[MetricValue::U64(15)]);

        let entry = metrics.metrics.get(metrics_key).expect("metric set entry");
        assert_eq!(entry.metric_values, vec![MetricValue::U64(15)]);
    }

    #[test]
    fn test_mmsc_snapshot_value_is_zero() {
        let zero = MetricValue::Mmsc(MmscSnapshot {
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
            count: 0,
        });
        assert!(zero.is_zero());

        let non_zero = MetricValue::Mmsc(MmscSnapshot {
            min: 1.0,
            max: 5.0,
            sum: 6.0,
            count: 2,
        });
        assert!(!non_zero.is_zero());
    }

    #[test]
    fn test_mmsc_snapshot_value_zero_of_kind() {
        let val = MetricValue::Mmsc(MmscSnapshot {
            min: 1.0,
            max: 5.0,
            sum: 6.0,
            count: 2,
        });
        let zero = val.zero_of_kind();
        assert!(zero.is_zero());
        match zero {
            MetricValue::Mmsc(s) => {
                assert_eq!(s.min, f64::MAX);
                assert_eq!(s.max, f64::MIN);
                assert_eq!(s.sum, 0.0);
                assert_eq!(s.count, 0);
            }
            _ => panic!("Expected Mmsc variant"),
        }
    }

    #[test]
    fn test_mmsc_snapshot_value_merge() {
        let mut a = MetricValue::Mmsc(MmscSnapshot {
            min: 2.0,
            max: 8.0,
            sum: 15.0,
            count: 3,
        });
        let b = MetricValue::Mmsc(MmscSnapshot {
            min: 1.0,
            max: 10.0,
            sum: 20.0,
            count: 4,
        });
        a.add_in_place(b);
        match a {
            MetricValue::Mmsc(s) => {
                assert_eq!(s.min, 1.0);
                assert_eq!(s.max, 10.0);
                assert_eq!(s.sum, 35.0);
                assert_eq!(s.count, 7);
            }
            _ => panic!("Expected Mmsc variant"),
        }
    }

    #[test]
    fn test_mmsc_snapshot_value_merge_zero_to_value() {
        // Merging into a zero/sentinel Mmsc should produce the incoming value
        let mut a = MetricValue::Mmsc(MmscSnapshot {
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
            count: 0,
        });
        let b = MetricValue::Mmsc(MmscSnapshot {
            min: 3.0,
            max: 7.0,
            sum: 10.0,
            count: 2,
        });
        a.add_in_place(b);
        match a {
            MetricValue::Mmsc(s) => {
                assert_eq!(s.min, 3.0);
                assert_eq!(s.max, 7.0);
                assert_eq!(s.sum, 10.0);
                assert_eq!(s.count, 2);
            }
            _ => panic!("Expected Mmsc variant"),
        }
    }

    #[test]
    fn test_mmsc_from_snapshot() {
        let snap = MmscSnapshot {
            min: 1.0,
            max: 10.0,
            sum: 25.0,
            count: 5,
        };
        let val = MetricValue::from(snap);
        assert_eq!(
            val,
            MetricValue::Mmsc(MmscSnapshot {
                min: 1.0,
                max: 10.0,
                sum: 25.0,
                count: 5,
            })
        );
    }

    #[test]
    fn test_accumulate_snapshot_mmsc() {
        #[derive(Debug)]
        struct MockMmscMetricSet {
            values: Vec<MetricValue>,
        }

        impl MockMmscMetricSet {
            fn new() -> Self {
                Self {
                    values: vec![MetricValue::Mmsc(MmscSnapshot {
                        min: f64::MAX,
                        max: f64::MIN,
                        sum: 0.0,
                        count: 0,
                    })],
                }
            }
        }

        impl Default for MockMmscMetricSet {
            fn default() -> Self {
                Self::new()
            }
        }

        static MOCK_MMSC_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
            name: "test_mmsc_metrics",
            metrics: &[MetricsField {
                name: "latency",
                unit: "ms",
                brief: "Test MMSC instrument",
                instrument: Instrument::Mmsc,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::F64,
            }],
        };

        impl MetricSetHandler for MockMmscMetricSet {
            fn descriptor(&self) -> &'static MetricsDescriptor {
                &MOCK_MMSC_METRICS_DESCRIPTOR
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

        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "test_value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMmscMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        // First snapshot: min=2, max=8, sum=15, count=3
        metrics.accumulate_snapshot(
            metrics_key,
            &[MetricValue::Mmsc(MmscSnapshot {
                min: 2.0,
                max: 8.0,
                sum: 15.0,
                count: 3,
            })],
        );

        // Second snapshot: min=1, max=10, sum=20, count=4
        metrics.accumulate_snapshot(
            metrics_key,
            &[MetricValue::Mmsc(MmscSnapshot {
                min: 1.0,
                max: 10.0,
                sum: 20.0,
                count: 4,
            })],
        );

        // Accumulated: min=1, max=10, sum=35, count=7
        let entry = metrics.metrics.get(metrics_key).expect("metric set entry");
        match entry.metric_values[0] {
            MetricValue::Mmsc(s) => {
                assert_eq!(s.min, 1.0);
                assert_eq!(s.max, 10.0);
                assert_eq!(s.sum, 35.0);
                assert_eq!(s.count, 7);
            }
            _ => panic!("Expected Mmsc variant"),
        }
    }
}
