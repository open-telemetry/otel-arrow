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
use slotmap::{new_key_type, SlotMap};
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

/// Lightweight iterator over non-zero metrics (no heap allocs).
pub struct NonZeroMetrics<'a> {
    fields: &'static [MetricsField],
    values: &'a [u64],
    idx: usize,
    len: usize,
}

impl<'a> NonZeroMetrics<'a> {
    #[inline]
    fn new(fields: &'static [MetricsField], values: &'a [u64]) -> Self {
        let len = values.len();
        debug_assert_eq!(
            fields.len(),
            len,
            "descriptor.fields and metric values length must match"
        );
        Self { fields, values, idx: 0, len }
    }
}

impl<'a> Iterator for NonZeroMetrics<'a> {
    type Item = (&'static MetricsField, u64);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY invariants (upheld by construction and the registry):
        // - self.idx < self.len is the loop guard; we read index `i` captured before increment.
        // - fields.len() == values.len() (asserted in debug in new()) => indexing `fields[i]` is valid.
        // - `values` is an immutable slice for the lifetime of the iterator; no concurrent mutation.
        while self.idx < self.values.len() {
            let i = self.idx;
            self.idx += 1;

            // Use unchecked indexing when the feature is enabled, otherwise use safe indexing
            let v = {
                #[cfg(feature = "unchecked-index")]
                {
                    // SAFETY: We know `i` is valid because:
                    // 1. `i` was captured from `self.idx` before incrementing
                    // 2. Loop condition ensures `self.idx < self.values.len()` when we enter
                    // 3. `values` slice is immutable for the iterator's lifetime
                    unsafe { *self.values.get_unchecked(i) }
                }
                #[cfg(not(feature = "unchecked-index"))]
                {
                    self.values[i]
                }
            };

            if v != 0 {
                let field = {
                    #[cfg(feature = "unchecked-index")]
                    {
                        // SAFETY: Same invariants as above apply to fields array
                        // fields.len() == values.len() is asserted in new()
                        unsafe { self.fields.get_unchecked(i) }
                    }
                    #[cfg(not(feature = "unchecked-index"))]
                    {
                        &self.fields[i]
                    }
                };
                return Some((field, v));
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Upper bound: remaining elements
        // Lower bound: unknown number of non-zeros.
        (0, Some(self.len.saturating_sub(self.idx)))
    }
}

// This iterator is "fused": once `next()` returns `None`, it will always return `None`.
// Rationale:
// - `idx` increases monotonically up to `len` and is never reset.
// - No internal state can make new items appear after exhaustion.
// Benefit:
// - Allows iterator adaptors (e.g. `chain`) to skip redundant checks after exhaustion,
//   and callers do not need to wrap with `iter.fuse()`.

// Note: This marker trait does not change behavior. It only encodes the guarantee.
impl<'a> core::iter::FusedIterator for NonZeroMetrics<'a> {}

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

        // ToDo remove this debug print in production code
        println!("{}", self.generate_semconv_registry().to_yaml());

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

    /// Visits only metric sets with at least one non-zero value, yields a zero-alloc iterator
    /// of (MetricsField, value), then resets the values to zero.
    pub(crate) fn visit_non_zero_metrics_and_reset<F>(&mut self, mut f: F)
    where
            for<'a> F: FnMut(
        &'static MetricsDescriptor,
        &'a dyn AttributeSetHandler,
        NonZeroMetrics<'a>,
    ),
    {
        for entry in self.metrics.values_mut() {
            let values = &mut entry.metric_values;
            if values.iter().any(|&v| v != 0) {
                let desc = entry.metrics_descriptor;
                let attrs = entry.attribute_values.as_ref();

                f(desc, attrs, NonZeroMetrics::new(desc.metrics, values));

                // Zero after reporting.
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
    /// Creates a new `MetricsRegistryHandle`.
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
    pub fn len(&self) -> usize {
        self.metric_registry.lock().len()
    }

    /// Visits only metric sets with at least one non-zero value, yields a zero-alloc iterator
    /// of (MetricsField, value), then resets the values to zero.
    pub fn visit_non_zero_metrics_and_reset<F>(&self, f: F)
    where
            for<'a> F: FnMut(
        &'static MetricsDescriptor,
        &'a dyn AttributeSetHandler,
        NonZeroMetrics<'a>,
    ),
    {
        let mut reg = self.metric_registry.lock();
        reg.visit_non_zero_metrics_and_reset(f);
    }

    /// Generates a SemConvRegistry from the current MetricsRegistry.
    /// AttributeFields are deduplicated based on their key.
    pub fn generate_semconv_registry(&self) -> SemConvRegistry {
        self.metric_registry.lock().generate_semconv_registry()
    }
}
