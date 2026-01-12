// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core multivariate metrics traits and types.
//!
//! This module intentionally contains no product-specific metrics definitions. Concrete metrics
//! live in their respective nodes/crates and implement the `MetricSetHandler` trait defined
//! here.

pub mod dispatcher;

use crate::attributes::AttributeSetHandler;
use crate::descriptor::{Instrument, MetricsDescriptor, MetricsField, Temporality};
use crate::entity::EntityRegistry;
use crate::registry::{EntityKey, MetricSetKey};
use crate::semconv::SemConvRegistry;
use serde::Serialize;
use slotmap::SlotMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// Numeric metric value (integer or floating-point).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(untagged)]
pub enum MetricValue {
    /// Unsigned 64-bit integer value.
    U64(u64),
    /// 64-bit floating point value.
    F64(f64),
}

impl MetricValue {
    /// Returns `true` when the value is exactly zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        match self {
            MetricValue::U64(v) => v == 0,
            MetricValue::F64(v) => v == 0.0,
        }
    }

    /// Returns a zero value of the same variant.
    #[must_use]
    pub const fn zero_of_kind(self) -> Self {
        match self {
            MetricValue::U64(_) => MetricValue::U64(0),
            MetricValue::F64(_) => MetricValue::F64(0.0),
        }
    }

    /// Adds another metric value to this one, converting between numeric kinds if needed.
    pub fn add_in_place(&mut self, other: MetricValue) {
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
            },
            MetricValue::F64(rhs) => match self {
                MetricValue::U64(lhs) => {
                    *self = MetricValue::F64(*lhs as f64 + rhs);
                }
                MetricValue::F64(lhs) => {
                    *lhs += rhs;
                }
            },
        }
    }

    /// Resets the value to zero while keeping the numeric variant.
    pub fn reset(&mut self) {
        *self = self.zero_of_kind();
    }

    /// Returns the floating-point representation of the value.
    #[must_use]
    pub fn to_f64(self) -> f64 {
        match self {
            MetricValue::U64(v) => v as f64,
            MetricValue::F64(v) => v,
        }
    }

    /// Converts the metric value to `u64`, lossy for floating-point values.
    #[must_use]
    pub fn to_u64_lossy(self) -> u64 {
        match self {
            MetricValue::U64(v) => v,
            MetricValue::F64(v) => v as u64,
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
    pub fn entity_key(&self) -> EntityKey {
        self.entity_key
    }

    /// Returns the metrics key associated with this metric set.
    #[must_use]
    pub fn metrics_key(&self) -> MetricSetKey {
        self.key
    }

    /// Returns the metric set key associated with this metric set.
    #[must_use]
    pub fn metric_set_key(&self) -> MetricSetKey {
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

    /// Merges a metrics snapshot (delta) into the registered instance keyed by `metrics_key`.
    pub(crate) fn accumulate_snapshot(
        &mut self,
        metrics_key: MetricSetKey,
        metrics_values: &[MetricValue],
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

    pub(crate) fn unregister(&mut self, metrics_key: MetricSetKey) -> bool {
        self.metrics.remove(metrics_key).is_some()
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
