// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Type-safe metrics registry maintaining aggregated telemetry metrics.
//!
//! Note: Concrete metrics live in their respective crates; this registry aggregates them via
//! dynamic dispatch.

use crate::attributes::{AttributeSetHandler, AttributeValue};
use crate::descriptor::{
    AttributeValueType, AttributesDescriptor, Instrument, MetricsDescriptor, MetricsField,
    Temporality,
};
use crate::metrics::{MetricSet, MetricSetHandler, MetricValue};
use crate::semconv::SemConvRegistry;
use parking_lot::Mutex;
use slotmap::{SlotMap, new_key_type};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

new_key_type! {
    /// This key is used to identify a specific entity entry in the registry (slotmap index).
    pub struct EntityKey;
    /// This key is used to identify a specific metrics entry in the registry (slotmap index).
    pub struct MetricSetKey;
}

#[derive(Debug, Clone)]
struct EntityAttributeSet {
    descriptor: &'static AttributesDescriptor,
    values: Arc<[AttributeValue]>,
    sorted_indices: Arc<[usize]>,
}

impl AttributeSetHandler for EntityAttributeSet {
    fn descriptor(&self) -> &'static AttributesDescriptor {
        self.descriptor
    }

    fn attribute_values(&self) -> &[AttributeValue] {
        &self.values
    }
}

fn hash_attribute_value<H: Hasher>(value: &AttributeValue, state: &mut H) {
    match value {
        AttributeValue::String(v) => {
            0u8.hash(state);
            v.hash(state);
        }
        AttributeValue::Int(v) => {
            1u8.hash(state);
            v.hash(state);
        }
        AttributeValue::UInt(v) => {
            2u8.hash(state);
            v.hash(state);
        }
        AttributeValue::Double(v) => {
            3u8.hash(state);
            v.to_bits().hash(state);
        }
        AttributeValue::Boolean(v) => {
            4u8.hash(state);
            v.hash(state);
        }
    }
}

fn attribute_value_type_rank(value_type: AttributeValueType) -> u8 {
    match value_type {
        AttributeValueType::String => 0,
        AttributeValueType::Int => 1,
        AttributeValueType::Double => 2,
        AttributeValueType::Boolean => 3,
    }
}

fn attribute_value_equal(left: &AttributeValue, right: &AttributeValue) -> bool {
    match (left, right) {
        (AttributeValue::String(a), AttributeValue::String(b)) => a == b,
        (AttributeValue::Int(a), AttributeValue::Int(b)) => a == b,
        (AttributeValue::UInt(a), AttributeValue::UInt(b)) => a == b,
        (AttributeValue::Double(a), AttributeValue::Double(b)) => a.to_bits() == b.to_bits(),
        (AttributeValue::Boolean(a), AttributeValue::Boolean(b)) => a == b,
        _ => false,
    }
}

impl EntityAttributeSet {
    fn new(attrs: impl AttributeSetHandler) -> Self {
        let descriptor = attrs.descriptor();
        let values: Arc<[AttributeValue]> = attrs.attribute_values().to_vec().into();
        debug_assert_eq!(
            descriptor.fields.len(),
            values.len(),
            "descriptor.fields and attribute values length must match"
        );

        let mut indices: Vec<usize> = (0..descriptor.fields.len()).collect();
        indices.sort_by(|&left, &right| {
            let left_field = &descriptor.fields[left];
            let right_field = &descriptor.fields[right];
            match left_field.key.cmp(right_field.key) {
                std::cmp::Ordering::Equal => attribute_value_type_rank(left_field.r#type)
                    .cmp(&attribute_value_type_rank(right_field.r#type)),
                other => other,
            }
        });

        Self {
            descriptor,
            values,
            sorted_indices: indices.into(),
        }
    }
}

impl PartialEq for EntityAttributeSet {
    fn eq(&self, other: &Self) -> bool {
        if self.descriptor.fields.len() != self.values.len()
            || other.descriptor.fields.len() != other.values.len()
        {
            return false;
        }

        if self.sorted_indices.len() != other.sorted_indices.len() {
            return false;
        }

        self.sorted_indices
            .iter()
            .zip(other.sorted_indices.iter())
            .all(|(lhs_idx, rhs_idx)| {
                let lhs_field = &self.descriptor.fields[*lhs_idx];
                let rhs_field = &other.descriptor.fields[*rhs_idx];
                if lhs_field.key != rhs_field.key || lhs_field.r#type != rhs_field.r#type {
                    return false;
                }
                let lhs_value = &self.values[*lhs_idx];
                let rhs_value = &other.values[*rhs_idx];
                attribute_value_equal(lhs_value, rhs_value)
            })
    }
}

impl Eq for EntityAttributeSet {}

impl Hash for EntityAttributeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let descriptor_len = self.descriptor.fields.len();
        let values_len = self.values.len();
        descriptor_len.hash(state);
        values_len.hash(state);
        if descriptor_len != values_len {
            return;
        }

        self.sorted_indices.len().hash(state);
        for idx in self.sorted_indices.iter() {
            let field = &self.descriptor.fields[*idx];
            field.key.hash(state);
            attribute_value_type_rank(field.r#type).hash(state);
            let value = &self.values[*idx];
            hash_attribute_value(value, state);
        }
    }
}

/// A registry that maintains de-duplicated attribute sets for entities.
#[derive(Default)]
pub struct EntityRegistry {
    entities: SlotMap<EntityKey, Arc<EntityAttributeSet>>,
    entities_by_signature: HashMap<EntityAttributeSet, EntityKey>,
}

impl Debug for EntityRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityRegistry")
            .field("entities_len", &self.entities.len())
            .finish()
    }
}

impl EntityRegistry {
    /// Registers (or reuses) an entity for the provided attribute set and returns its key.
    fn register(&mut self, attrs: impl AttributeSetHandler) -> EntityKey {
        let entity = EntityAttributeSet::new(attrs);
        if let Some(existing) = self.entities_by_signature.get(&entity) {
            return *existing;
        }

        let attrs = Arc::new(entity.clone());

        let entity_key = self.entities.insert(attrs);
        let _ = self.entities_by_signature.insert(entity, entity_key);
        entity_key
    }

    fn unregister(&mut self, entity_key: EntityKey) -> bool {
        if let Some(attrs) = self.entities.remove(entity_key) {
            let _ = self.entities_by_signature.remove(attrs.as_ref());
            true
        } else {
            false
        }
    }

    fn len(&self) -> usize {
        self.entities.len()
    }

    fn get(&self, key: EntityKey) -> Option<&EntityAttributeSet> {
        self.entities.get(key).map(|attrs| attrs.as_ref())
    }

    fn get_shared(&self, key: EntityKey) -> Option<Arc<EntityAttributeSet>> {
        self.entities.get(key).cloned()
    }

    fn visit_entities<F>(&self, mut f: F)
    where
        F: FnMut(EntityKey, &dyn AttributeSetHandler),
    {
        for (key, attrs) in self.entities.iter() {
            f(key, attrs.as_ref());
        }
    }
}

/// A registered metrics entry containing all necessary information for metrics aggregation.
pub struct MetricsEntry {
    /// The static descriptor describing the metrics structure
    pub metrics_descriptor: &'static MetricsDescriptor,
    /// Current metric values stored as a vector
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
    pub fn new(
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
    fn new(fields: &'static [MetricsField], values: &'a [MetricValue]) -> Self {
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

// This iterator is "fused": once `next()` returns `None`, it will always return `None`.
// Rationale:
// - `idx` increases monotonically up to `len` and is never reset.
// - No internal state can make new items appear after exhaustion.
// Benefit:
// - Allows iterator adaptors (e.g. `chain`) to skip redundant checks after exhaustion,
//   and callers do not need to wrap with `iter.fuse()`.

// Note: This marker trait does not change behavior. It only encodes the guarantee.
impl<'a> core::iter::FusedIterator for MetricsIterator<'a> {}

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
    registry: Arc<Mutex<TelemetryRegistry>>,
}

/// A sendable and cloneable handle on a metric set registry.
#[derive(Debug, Clone)]
pub struct MetricSetRegistryHandle {
    registry: Arc<Mutex<TelemetryRegistry>>,
}

/// A sendable and cloneable handle on an entity registry.
#[derive(Debug, Clone)]
pub struct EntityRegistryHandle {
    registry: Arc<Mutex<TelemetryRegistry>>,
}

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
    fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(
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

    /// Merges a metrics snapshot (delta) into the registered instance keyed by `metrics_key`.
    fn accumulate_snapshot(&mut self, metrics_key: MetricSetKey, metrics_values: &[MetricValue]) {
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
                    Instrument::Histogram => {
                        // Histograms (currently represented as numeric aggregates) report per-interval changes.
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

    fn unregister(&mut self, metrics_key: MetricSetKey) -> bool {
        self.metrics.remove(metrics_key).is_some()
    }

    /// Returns the total number of registered metrics sets.
    fn len(&self) -> usize {
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

#[derive(Debug, Default)]
struct TelemetryRegistry {
    entities: EntityRegistry,
    metrics: MetricSetRegistry,
}

impl Default for EntityRegistryHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRegistryHandle {
    /// Creates a new `EntityRegistryHandle`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(TelemetryRegistry::default())),
        }
    }

    /// Returns a telemetry registry handle sharing this entity registry.
    #[must_use]
    pub fn telemetry_registry(&self) -> TelemetryRegistryHandle {
        TelemetryRegistryHandle {
            registry: self.registry.clone(),
        }
    }

    /// Returns a metric set registry handle sharing this entity registry.
    #[must_use]
    pub fn metric_set_registry(&self) -> MetricSetRegistryHandle {
        MetricSetRegistryHandle {
            registry: self.registry.clone(),
        }
    }

    /// Registers (or reuses) an entity for the provided attribute set.
    pub fn register(&self, attrs: impl AttributeSetHandler + Send + Sync + 'static) -> EntityKey {
        self.registry.lock().entities.register(attrs)
    }

    /// Unregisters an entity by key.
    #[must_use]
    pub fn unregister(&self, entity_key: EntityKey) -> bool {
        self.registry.lock().entities.unregister(entity_key)
    }

    /// Returns the total number of registered entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.registry.lock().entities.len()
    }

    /// Returns true if there are no registered entities.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    pub fn get_attributes(
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

    /// Returns a metric set registry handle sharing this telemetry registry.
    #[must_use]
    pub fn metric_set_registry(&self) -> MetricSetRegistryHandle {
        MetricSetRegistryHandle {
            registry: self.registry.clone(),
        }
    }

    /// Returns an entity registry handle sharing this telemetry registry.
    #[must_use]
    pub fn entity_registry(&self) -> EntityRegistryHandle {
        EntityRegistryHandle {
            registry: self.registry.clone(),
        }
    }

    /// Registers (or reuses) an entity for the provided attribute set.
    pub fn register_entity(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> EntityKey {
        self.entity_registry().register(attrs)
    }

    /// Unregisters an entity by key.
    #[must_use]
    pub fn unregister_entity(&self, entity_key: EntityKey) -> bool {
        self.entity_registry().unregister(entity_key)
    }

    /// Registers a metric set type for the provided entity key.
    #[must_use]
    pub fn register_with_entity<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        entity_key: EntityKey,
    ) -> MetricSet<T> {
        self.metric_set_registry().register_with_entity(entity_key)
    }

    /// Registers a metric set type with the given static attributes and returns a `MetricSet`
    /// instance that can be used to report metrics for that type.
    pub fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> MetricSet<T> {
        self.metric_set_registry().register(attrs)
    }

    /// Unregisters a metric set by key.
    #[must_use]
    pub fn unregister_metric_set(&self, metrics_key: MetricSetKey) -> bool {
        self.metric_set_registry().unregister(metrics_key)
    }

    /// Adds a new metrics snapshot to the aggregator for the given key.
    pub fn accumulate_snapshot(&self, metrics_key: MetricSetKey, metrics: &[MetricValue]) {
        self.metric_set_registry()
            .accumulate_snapshot(metrics_key, metrics);
    }

    /// Returns the total number of registered metrics sets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.metric_set_registry().len()
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
        self.metric_set_registry()
            .visit_metrics_and_reset_with_zeroes(f, keep_all_zeroes);
    }

    /// Generates a SemConvRegistry from the current MetricSetRegistry.
    /// AttributeFields are deduplicated based on their key.
    #[must_use]
    pub fn generate_semconv_registry(&self) -> SemConvRegistry {
        self.metric_set_registry().generate_semconv_registry()
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
    pub fn visit_current_metrics_with_zeroes<F>(&self, f: F, keep_all_zeroes: bool)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        self.metric_set_registry()
            .visit_current_metrics_with_zeroes(f, keep_all_zeroes);
    }
}

impl Default for MetricSetRegistryHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricSetRegistryHandle {
    /// Creates a new `MetricSetRegistryHandle`.
    #[must_use]
    pub fn new() -> Self {
        TelemetryRegistryHandle::new().metric_set_registry()
    }

    /// Returns an entity registry handle sharing this metric set registry.
    #[must_use]
    pub fn entity_registry(&self) -> EntityRegistryHandle {
        EntityRegistryHandle {
            registry: self.registry.clone(),
        }
    }

    /// Registers (or reuses) an entity for the provided attribute set.
    pub fn register_entity(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> EntityKey {
        self.entity_registry().register(attrs)
    }

    /// Registers a metric set type for the provided entity key.
    #[must_use]
    pub fn register_with_entity<T: MetricSetHandler + Default + Debug + Send + Sync>(
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
    pub fn register<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> MetricSet<T> {
        let mut registry = self.registry.lock();
        let entity_key = registry.entities.register(attrs);
        registry.metrics.register(entity_key)
    }

    /// Unregisters a metric set by key.
    #[must_use]
    pub fn unregister(&self, metrics_key: MetricSetKey) -> bool {
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
        metrics.visit_metrics_and_reset(&*entities, f, keep_all_zeroes);
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
        Temporality,
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
        let entity_registry = telemetry_registry.entity_registry();

        let key1 = entity_registry.register(MockAttributeSet::new("value".to_string()));
        let key2 = entity_registry.register(MockAttributeSet::new("value".to_string()));

        assert_eq!(key1, key2);
        assert_eq!(entity_registry.len(), 1);
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
        let entity_registry = telemetry_registry.entity_registry();

        let key1 = entity_registry.register(OrderedAttributeSetA {
            values: vec![
                AttributeValue::String("value".to_string()),
                AttributeValue::Int(7),
            ],
        });
        let key2 = entity_registry.register(OrderedAttributeSetB {
            values: vec![
                AttributeValue::Int(7),
                AttributeValue::String("value".to_string()),
            ],
        });

        assert_eq!(key1, key2);
        assert_eq!(entity_registry.len(), 1);
    }

    #[test]
    fn test_entity_registry_get_attributes() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let entity_registry = telemetry_registry.entity_registry();

        let key = entity_registry.register(MockAttributeSet::new("value".to_string()));
        let attrs = entity_registry
            .get_attributes(key)
            .expect("missing attributes");

        let collected: Vec<_> = attrs.iter_attributes().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, "test_key");
        assert_eq!(*collected[0].1, AttributeValue::String("value".to_string()));
    }

    #[test]
    fn test_entity_registry_unregister() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let entity_registry = telemetry_registry.entity_registry();

        let key = entity_registry.register(MockAttributeSet::new("value".to_string()));

        assert!(entity_registry.unregister(key));
        assert_eq!(entity_registry.len(), 0);
        assert!(entity_registry.get_attributes(key).is_none());
        assert!(!entity_registry.unregister(key));
    }

    #[test]
    fn test_metrics_registry_register_with_entity_key() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let entity_key = telemetry_registry
            .entity_registry()
            .register(MockAttributeSet::new("value".to_string()));

        let metric_set: MetricSet<MockMetricSet> =
            telemetry_registry.register_with_entity(entity_key);
        assert_eq!(metric_set.entity_key(), entity_key);
    }

    #[test]
    fn test_metrics_registry_unregister() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register(attrs);
        let metrics_key = metric_set.key;

        assert!(telemetry_registry.unregister_metric_set(metrics_key));
        assert_eq!(telemetry_registry.len(), 0);
        assert!(!telemetry_registry.unregister_metric_set(metrics_key));
    }

    #[test]
    fn test_metrics_registry_register() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let _metric_set: MetricSet<MockMetricSet> = telemetry_registry.register(attrs);
        assert_eq!(telemetry_registry.len(), 1);
    }

    #[test]
    fn test_metrics_registry_multiple_registrations() {
        let telemetry_registry = TelemetryRegistryHandle::new();

        let attrs1 = MockAttributeSet::new("value1".to_string());
        let attrs2 = MockAttributeSet::new("value2".to_string());

        let _metric_set1: MetricSet<MockMetricSet> = telemetry_registry.register(attrs1);
        let _metric_set2: MetricSet<MockMetricSet> = telemetry_registry.register(attrs2);

        assert_eq!(telemetry_registry.len(), 2);
    }

    #[test]
    fn test_accumulate_snapshot_basic() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register(attrs);
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

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register(attrs);
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

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register(attrs);
        let metrics_key = metric_set.key;

        // This should panic on overflow when unchecked-arithmetic is disabled
        telemetry_registry.accumulate_snapshot(metrics_key, &[MetricValue::U64(u64::MAX)]);
        telemetry_registry.accumulate_snapshot(metrics_key, &[MetricValue::U64(1)]);
    }

    #[test]
    fn test_visit_metrics_and_reset() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let attrs = MockAttributeSet::new("test_value".to_string());

        let metric_set: MetricSet<MockMetricSet> = telemetry_registry.register(attrs);
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
        let _metric_set: MetricSet<MockMetricSet> = telemetry_registry.register(attrs);

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
                let metric_set: MetricSet<MockMetricSet> = telemetry_registry_clone.register(attrs);
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
        let metric_set: MetricSet<MockGaugeMetricSet> = telemetry_registry.register(attrs);
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
            telemetry_registry.register(MockAttributeSet::new("attr".to_string()));
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
