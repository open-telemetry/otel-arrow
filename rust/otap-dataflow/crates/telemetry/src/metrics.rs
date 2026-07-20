// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core multivariate metrics (aka metric set) traits and types + Metric Set Registry.
//!
//! This module intentionally contains no product-specific metrics definitions. Concrete metrics
//! live in their respective nodes/crates and implement the `MetricSetHandler` trait defined
//! here.

pub mod dispatcher;

use crate::attributes::{AttributeSetHandler, MeasurementAttributeSet};
use crate::descriptor::{
    Instrument, MeasurementAttributeDescriptor, MetricsDescriptor, MetricsField, Temporality,
};
use crate::entity::EntityRegistry;
use crate::instrument::MmscSnapshot;
use crate::registry::{EntityKey, MetricSetKey};
use crate::semconv::SemConvRegistry;
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// The default per-set cardinality budget used by the compile-time check emitted
/// by the `#[metric_set]` macro.
///
/// This mirrors the Rust OpenTelemetry SDK's default per-instrument cardinality
/// limit: once a single instrument exceeds it, overflow series collapse into a
/// single `otel.metric.overflow` series, silently losing fidelity. Because a
/// measurement metric set's worst-case cardinality (the product of its enum
/// attributes' variant counts) is known at compile time, the macro rejects at
/// build time any set whose product would exceed this budget.
pub const CARDINALITY_BUDGET: usize = 2000;

/// Compile-time cardinality guard used by generated `#[metric_set]` code.
///
/// Generated code for a measurement metric set evaluates
/// `check_cardinality(<D as MeasurementAttributeSet>::CARDINALITY)` in a `const`
/// item. The function panics in a `const` context when the cardinality exceeds
/// [`CARDINALITY_BUDGET`], which the compiler surfaces as a hard build error at
/// the metric-set declaration site; within budget it is a no-op.
#[track_caller]
pub const fn check_cardinality(cardinality: usize) {
    assert!(
        cardinality <= CARDINALITY_BUDGET,
        "metric set worst-case cardinality exceeds CARDINALITY_BUDGET; \
         reduce the number of measurement enum attributes or their variants"
    );
}

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
            descriptor: self.metrics.descriptor(),
            measurement_attributes: &[],
            bucket: 0,
            metrics: self.metrics.snapshot_values(),
        }
    }

    /// Takes the snapshot for terminal handoff and clears the metric set.
    ///
    /// This uses the same ownership-transfer semantics as
    /// [`MeasurementMetricSet::terminal_snapshots`]. Plain sets always return
    /// one snapshot because they have exactly one bucket.
    #[must_use]
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let snapshot = self.snapshot();
        self.clear_values();
        vec![snapshot]
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
    pub(crate) descriptor: &'static MetricsDescriptor,
    pub(crate) measurement_attributes: &'static [MeasurementAttributeDescriptor],
    /// Bucket index within the set. Always `0` for plain sets and for sets with
    /// only registration attributes; for measurement sets it selects the item whose
    /// enum-attribute combination decodes from this index (see
    /// [`MeasurementAttributeSet::bucket_index`]).
    pub(crate) bucket: usize,
    pub(crate) metrics: Vec<MetricValue>,
}

impl MetricSetSnapshot {
    /// Returns the metric set key that identifies this snapshot's source.
    #[must_use]
    pub fn key(&self) -> MetricSetKey {
        self.key
    }

    /// Returns the descriptor for the metric set that produced this snapshot.
    #[must_use]
    pub const fn descriptor(&self) -> &'static MetricsDescriptor {
        self.descriptor
    }

    /// Returns the bucket index this snapshot targets (0 for non-measurement sets).
    #[must_use]
    pub fn bucket(&self) -> usize {
        self.bucket
    }

    /// Iterates over the measurement attributes decoded for this snapshot's bucket.
    ///
    /// Attributes are yielded in declaration order. Callers that need an
    /// order-independent identity can sort the returned key-value pairs.
    #[must_use]
    pub fn measurement_attributes(
        &self,
    ) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        let mut rem = self.bucket;
        self.measurement_attributes
            .iter()
            .filter_map(move |descriptor| {
                let radix = descriptor.variants.len();
                debug_assert!(
                    radix > 0,
                    "measurement attribute descriptor must have at least one variant"
                );
                if radix == 0 {
                    return None;
                }

                let value = descriptor.variants[rem % radix];
                rem /= radix;
                Some((descriptor.key, value))
            })
    }

    /// Returns the value of a measurement attribute for this snapshot's bucket.
    #[must_use]
    pub fn measurement_attribute_value(&self, key: &str) -> Option<&'static str> {
        self.measurement_attributes()
            .find_map(|(attribute_key, value)| (attribute_key == key).then_some(value))
    }

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

/// A [`MetricSetHandler`] that binds a set of measurement (per-item) enum
/// attributes, generated by `#[metric_set(measurement_attributes = ...)]`.
///
/// The associated [`MeasurementAttributes`](Self::MeasurementAttributes) type identifies
/// the [`MeasurementAttributeSet`] whose variants address the set's buckets.
pub trait MeasurementMetricSetHandler: MetricSetHandler + Default {
    /// The measurement attribute set whose combinations index this set's items.
    type MeasurementAttributes: MeasurementAttributeSet;
}

/// A [`MetricSetHandler`] that binds a set of registration-time attributes,
/// generated by `#[metric_set(registration_attributes = ...)]`.
pub trait RegistrationMetricSetHandler: MetricSetHandler + Default {
    /// The attribute set supplied at registration and attached to every
    /// item of this set.
    type RegistrationAttributes: AttributeSetHandler;
}

/// Implementation detail used by generated [`metric_set`](otap_df_telemetry_macros::metric_set)
/// `register` methods.
///
/// This trait is public so macro expansions can use it outside this crate.
/// Contexts implement it to select the owning entity scope; component code must
/// use the generated `MyMetrics::register(...)` method instead.
#[doc(hidden)]
pub trait MetricSetRegistrar {
    /// Registers a metric set without item attributes.
    fn register_metric_set<M: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
    ) -> MetricSet<M>;

    /// Registers a metric set with registration-time item attributes.
    fn register_registration_metric_set<M: RegistrationMetricSetHandler + Debug + Send + Sync>(
        &self,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MetricSet<M>;

    /// Registers a metric set with bounded per-measurement attributes.
    fn register_measurement_metric_set<M: MeasurementMetricSetHandler + Debug + Send + Sync>(
        &self,
    ) -> MeasurementMetricSet<M>;

    /// Registers a metric set with registration-time and per-measurement attributes.
    fn register_registration_and_measurement_metric_set<
        M: RegistrationMetricSetHandler + MeasurementMetricSetHandler + Debug + Send + Sync,
    >(
        &self,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MeasurementMetricSet<M>;
}

/// A registered measurement metric set: a dense array of per-bucket metric structs
/// addressed by a [`MeasurementAttributeSet`]'s mixed-radix bucket index.
///
/// Recording resolves a bucket by arithmetic (no hashing, no allocation) via
/// [`with`](Self::with), which returns a mutable view of the whole metric struct
/// for that attribute combination. A `touched` bitset tracks which buckets have
/// been written so only live items are reported.
pub struct MeasurementMetricSet<M: MeasurementMetricSetHandler> {
    pub(crate) key: MetricSetKey,
    pub(crate) entity_key: EntityKey,
    pub(crate) buckets: Vec<M>,
    pub(crate) touched: Vec<u64>,
}

impl<M: MeasurementMetricSetHandler + Debug> Debug for MeasurementMetricSet<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeasurementMetricSet")
            .field("key", &self.key)
            .field("entity_key", &self.entity_key)
            .field("buckets", &self.buckets.len())
            .finish()
    }
}

impl<M: MeasurementMetricSetHandler> MeasurementMetricSet<M> {
    pub(crate) fn new(key: MetricSetKey, entity_key: EntityKey) -> Self {
        let mut buckets = Vec::with_capacity(M::MeasurementAttributes::CARDINALITY);
        buckets.resize_with(M::MeasurementAttributes::CARDINALITY, M::default);
        Self {
            key,
            entity_key,
            buckets,
            touched: vec![0u64; M::MeasurementAttributes::CARDINALITY.div_ceil(64)],
        }
    }

    /// Returns a mutable view of the metric struct for the given attribute
    /// combination, marking its bucket as touched so it is reported.
    #[inline]
    pub fn with(&mut self, attrs: M::MeasurementAttributes) -> &mut M {
        let bucket = attrs.bucket_index();
        debug_assert!(bucket < self.buckets.len(), "bucket index out of range");
        self.touched[bucket / 64] |= 1u64 << (bucket % 64);
        &mut self.buckets[bucket]
    }

    /// Returns an existing bucket without marking it for reporting.
    /// Useful for testing.
    #[must_use]
    #[inline]
    pub fn get(&self, attrs: M::MeasurementAttributes) -> &M {
        let bucket = attrs.bucket_index();
        debug_assert!(bucket < self.buckets.len(), "bucket index out of range");
        &self.buckets[bucket]
    }

    /// Returns the metric set key associated with this measurement metric set.
    #[must_use]
    pub const fn metric_set_key(&self) -> MetricSetKey {
        self.key
    }

    /// Returns the entity key associated with this measurement metric set.
    #[must_use]
    pub const fn entity_key(&self) -> EntityKey {
        self.entity_key
    }

    #[inline]
    fn is_touched(&self, bucket: usize) -> bool {
        (self.touched[bucket / 64] >> (bucket % 64)) & 1 == 1
    }

    /// Produces one snapshot per touched bucket without clearing reported values.
    ///
    /// Empty touched buckets are cleared because they have no values to retry. A
    /// caller must invoke [`Self::clear_bucket`] only after it successfully sends
    /// a returned snapshot.
    ///
    /// Reporting is intentionally **event-driven**: only buckets recorded into
    /// since the last drain are exported. So an `always_flush` instrument (e.g.
    /// `Gauge`/`Observe*`) in a measurement set is exported only for intervals in
    /// which its combination was recorded, not every cycle. A plain (non-measurement)
    /// set is usually the better fit for continuously-sampled values.
    pub(crate) fn pending_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut out = Vec::new();
        for bucket in 0..self.buckets.len() {
            if self.is_touched(bucket) {
                if self.buckets[bucket].needs_flush() {
                    out.push(MetricSetSnapshot {
                        key: self.key,
                        descriptor: self.buckets[bucket].descriptor(),
                        measurement_attributes: M::MeasurementAttributes::DESCRIPTORS,
                        bucket,
                        metrics: self.buckets[bucket].snapshot_values(),
                    });
                } else {
                    self.clear_bucket(bucket);
                }
            }
        }
        out
    }

    /// Takes snapshots for all touched, non-empty buckets during terminal handoff.
    ///
    /// Unlike reporter-driven collection, terminal handoff transfers ownership of
    /// every returned snapshot. The corresponding buckets are therefore cleared
    /// immediately and cannot be returned again.
    ///
    /// This retains measurement sets' event-driven behavior: untouched and empty
    /// buckets are omitted rather than emitting every possible attribute
    /// combination.
    #[must_use]
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let snapshots = self.pending_snapshots();
        for snapshot in &snapshots {
            self.clear_bucket(snapshot.bucket());
        }
        snapshots
    }

    pub(crate) fn clear_bucket(&mut self, bucket: usize) {
        self.buckets[bucket].clear_values();
        self.touched[bucket / 64] &= !(1u64 << (bucket % 64));
    }
}

/// A registered metrics entry containing all necessary information for metrics aggregation.
pub struct MetricsEntry {
    /// The static descriptor describing the metrics structure
    pub metrics_descriptor: &'static MetricsDescriptor,
    /// Current snapshot values stored as a vector.
    ///
    /// Length is `bucket_count * metrics_descriptor.metrics.len()`: the values
    /// for bucket `b` occupy the slice `[b * fields .. (b + 1) * fields]`. Plain
    /// sets have `bucket_count == 1` and this is exactly the field values.
    pub metric_values: Vec<MetricValue>,

    /// Entity key for the associated attribute set
    pub entity_key: EntityKey,

    /// Number of item buckets (1 for plain and registration-only sets, the
    /// [`MeasurementAttributeSet::CARDINALITY`] for measurement sets).
    pub bucket_count: usize,

    /// Per-item enum attribute descriptors used to decode a bucket index into
    /// item attributes at export time (empty for non-measurement sets).
    pub measurement_attributes: &'static [MeasurementAttributeDescriptor],

    /// Fixed (key, value) attributes attached to every item of this set,
    /// captured at registration (empty for sets without registration attributes).
    pub registration_attributes: Vec<(String, String)>,
}

impl Debug for MetricsEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsEntry")
            .field("metrics_descriptor", &self.metrics_descriptor)
            .field("metric_values", &self.metric_values)
            .field("entity_key", &self.entity_key)
            .field("bucket_count", &self.bucket_count)
            .finish()
    }
}

impl MetricsEntry {
    /// Creates a new plain metrics entry (single bucket, no per-item attributes).
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
            bucket_count: 1,
            measurement_attributes: &[],
            registration_attributes: Vec::new(),
        }
    }

    /// Creates a metrics entry with registration-time attributes and `bucket_count` measurement
    /// buckets. The value vector is pre-sized to `bucket_count * fields` zeroed
    /// slots.
    #[must_use]
    pub fn new_with_item_attributes(
        metrics_descriptor: &'static MetricsDescriptor,
        zeroed_bucket: &[MetricValue],
        entity_key: EntityKey,
        bucket_count: usize,
        measurement_attributes: &'static [MeasurementAttributeDescriptor],
        registration_attributes: Vec<(String, String)>,
    ) -> Self {
        let mut metric_values = Vec::with_capacity(bucket_count * zeroed_bucket.len());
        for _ in 0..bucket_count {
            metric_values.extend_from_slice(zeroed_bucket);
        }
        Self {
            metrics_descriptor,
            metric_values,
            entity_key,
            bucket_count,
            measurement_attributes,
            registration_attributes,
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

    /// Registers a metric set carrying registration-time item attributes captured
    /// once at registration.
    pub(crate) fn register_with_registration_attributes<
        T: MetricSetHandler + Default + Debug + Send + Sync,
    >(
        &mut self,
        entity_key: EntityKey,
        registration_attributes: Vec<(String, String)>,
    ) -> MetricSet<T> {
        let metrics = T::default();
        let descriptor = metrics.descriptor();

        let metrics_key = self.metrics.insert(MetricsEntry::new_with_item_attributes(
            descriptor,
            &metrics.snapshot_values(),
            entity_key,
            1,
            &[],
            registration_attributes,
        ));

        MetricSet {
            key: metrics_key,
            entity_key,
            metrics,
        }
    }

    /// Registers a measurement metric set with one bucket per attribute combination.
    pub(crate) fn register_with_measurement_attributes<M>(
        &mut self,
        entity_key: EntityKey,
    ) -> MeasurementMetricSet<M>
    where
        M: MeasurementMetricSetHandler + Debug + Send + Sync,
    {
        let zeroed_bucket = M::default().snapshot_values();
        let descriptor = M::default().descriptor();

        let metrics_key = self.metrics.insert(MetricsEntry::new_with_item_attributes(
            descriptor,
            &zeroed_bucket,
            entity_key,
            M::MeasurementAttributes::CARDINALITY,
            M::MeasurementAttributes::DESCRIPTORS,
            Vec::new(),
        ));

        MeasurementMetricSet::new(metrics_key, entity_key)
    }

    /// Registers a metric set with registration-time attributes and one bucket per measurement
    /// attribute combination.
    pub(crate) fn register_with_registration_and_measurement_attributes<M>(
        &mut self,
        entity_key: EntityKey,
        registration_attributes: Vec<(String, String)>,
    ) -> MeasurementMetricSet<M>
    where
        M: MeasurementMetricSetHandler + Debug + Send + Sync,
    {
        let zeroed_bucket = M::default().snapshot_values();
        let descriptor = M::default().descriptor();

        let metrics_key = self.metrics.insert(MetricsEntry::new_with_item_attributes(
            descriptor,
            &zeroed_bucket,
            entity_key,
            M::MeasurementAttributes::CARDINALITY,
            M::MeasurementAttributes::DESCRIPTORS,
            registration_attributes,
        ));

        MeasurementMetricSet::new(metrics_key, entity_key)
    }

    /// Merges a metrics snapshot into the bucket `bucket` of the registered
    /// instance keyed by `metrics_key`.
    pub(crate) fn accumulate_snapshot(
        &mut self,
        metrics_key: MetricSetKey,
        bucket: usize,
        metrics_values: &[MetricValue], // snapshot values for a single bucket
    ) {
        if let Some(entry) = self.metrics.get_mut(metrics_key) {
            let fields_len = entry.metrics_descriptor.metrics.len();
            debug_assert_eq!(
                fields_len,
                metrics_values.len(),
                "descriptor.metrics and snapshot values length must match"
            );
            debug_assert!(bucket < entry.bucket_count, "bucket index out of range");
            let start = bucket * fields_len;
            let Some(bucket_slice) = entry.metric_values.get_mut(start..start + fields_len) else {
                debug_assert!(false, "bucket slice out of range");
                return;
            };

            bucket_slice
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

    /// Visits every non-empty item bucket of every metric set, yielding the
    /// per-item enum/registration attributes alongside a zero-alloc iterator of
    /// `(MetricsField, value)`, then resets the visited bucket to zero.
    pub(crate) fn visit_and_reset_with_item_attrs<F>(
        &mut self,
        entities: &EntityRegistry,
        mut f: F,
        keep_all_zeroes: bool,
    ) where
        for<'a> F: FnMut(
            &'static MetricsDescriptor,
            &'a dyn AttributeSetHandler,
            &'a [(&'a str, &'a str)],
            MetricsIterator<'a>,
        ),
    {
        for entry in self.metrics.values_mut() {
            let MetricsEntry {
                metrics_descriptor: desc,
                metric_values,
                entity_key,
                bucket_count,
                measurement_attributes,
                registration_attributes,
            } = entry;
            let Some(attrs) = entities.get(*entity_key) else {
                continue;
            };
            let fields_len = desc.metrics.len();
            let mut dp: Vec<(&str, &str)> = Vec::new();
            for bucket in 0..*bucket_count {
                let start = bucket * fields_len;
                let slice = &mut metric_values[start..start + fields_len];
                if keep_all_zeroes || slice.iter().any(|v| !v.is_zero()) {
                    decode_bucket_item_attrs(
                        measurement_attributes,
                        registration_attributes,
                        bucket,
                        &mut dp,
                    );
                    f(desc, attrs, &dp, MetricsIterator::new(desc.metrics, slice));
                    slice.iter_mut().for_each(MetricValue::reset);
                }
            }
        }
    }

    /// Read-only variant of [`Self::visit_and_reset_with_item_attrs`] that
    /// does not reset bucket values.
    pub(crate) fn visit_current_with_item_attrs<F>(
        &self,
        entities: &EntityRegistry,
        mut f: F,
        keep_all_zeroes: bool,
    ) where
        for<'a> F: FnMut(
            &'static MetricsDescriptor,
            &'a dyn AttributeSetHandler,
            &'a [(&'a str, &'a str)],
            MetricsIterator<'a>,
        ),
    {
        for entry in self.metrics.values() {
            let Some(attrs) = entities.get(entry.entity_key) else {
                continue;
            };
            let desc = entry.metrics_descriptor;
            let fields_len = desc.metrics.len();
            let mut dp: Vec<(&str, &str)> = Vec::new();
            for bucket in 0..entry.bucket_count {
                let start = bucket * fields_len;
                let slice = &entry.metric_values[start..start + fields_len];
                if keep_all_zeroes || slice.iter().any(|v| !v.is_zero()) {
                    decode_bucket_item_attrs(
                        entry.measurement_attributes,
                        &entry.registration_attributes,
                        bucket,
                        &mut dp,
                    );
                    f(desc, attrs, &dp, MetricsIterator::new(desc.metrics, slice));
                }
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

/// Decodes a dense mixed-radix `bucket` index into item attributes.
///
/// Registration attributes are emitted first (fixed key/value pairs), followed by the
/// measurement enum attributes. For the measurement axis the first declared attribute is
/// the low-order digit: `variant_index = (rem % radix); rem /= radix`.
fn decode_bucket_item_attrs<'a>(
    measurement: &'a [MeasurementAttributeDescriptor],
    registration_attrs: &'a [(String, String)],
    bucket: usize,
    out: &mut Vec<(&'a str, &'a str)>,
) {
    out.clear();
    for (k, v) in registration_attrs {
        out.push((k.as_str(), v.as_str()));
    }
    let mut rem = bucket;
    for d in measurement {
        debug_assert!(
            !d.variants.is_empty(),
            "measurement attribute descriptor must have at least one variant"
        );
        if d.variants.is_empty() {
            continue;
        }
        let radix = d.variants.len();
        let vidx = rem % radix;
        rem /= radix;
        out.push((d.key, d.variants[vidx]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, Instrument,
        MeasurementAttributeDescriptor, MetricValueType, MetricsField, Temporality,
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

    #[derive(Clone, Copy)]
    enum MockMeasurementAttributes {
        First,
        Second,
    }

    impl MeasurementAttributeSet for MockMeasurementAttributes {
        const CARDINALITY: usize = 4;
        const DESCRIPTORS: &'static [MeasurementAttributeDescriptor] = &[
            MeasurementAttributeDescriptor {
                key: "outcome",
                variants: &["first", "second"],
            },
            MeasurementAttributeDescriptor {
                key: "reason",
                variants: &["one", "two"],
            },
        ];

        fn bucket_index(&self) -> usize {
            match self {
                Self::First => 0,
                Self::Second => 1,
            }
        }
    }

    impl MeasurementMetricSetHandler for MockMetricSet {
        type MeasurementAttributes = MockMeasurementAttributes;
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
    fn test_metric_set_snapshot_carries_descriptor() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMetricSet> = metrics.register(entity_key);
        let snapshot = metric_set.snapshot();

        assert_eq!(snapshot.descriptor().name, "test_metrics");
        assert_eq!(snapshot.bucket(), 0);
        assert_eq!(snapshot.measurement_attribute_value("outcome"), None);
    }

    #[test]
    fn test_metric_set_terminal_snapshots_take_plain_bucket() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "value");
        let mut registry = MetricSetRegistry::default();
        let mut metrics: MetricSet<MockMetricSet> = registry.register(entity_key);
        metrics.values[0] = MetricValue::U64(7);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].get_metrics()[0], MetricValue::U64(7));
        assert_eq!(metrics.values[0], MetricValue::U64(0));
    }

    /// Scenario: A snapshot is emitted for a measurement bucket.
    /// Guarantees: The decoded measurement attributes are available for generic inspection.
    #[test]
    fn test_measurement_metric_set_get_and_snapshot_decode_attributes() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "value");
        let mut registry = MetricSetRegistry::default();
        let mut metrics: MeasurementMetricSet<MockMetricSet> =
            registry.register_with_measurement_attributes(entity_key);

        assert_eq!(
            metrics.get(MockMeasurementAttributes::First).values[0],
            MetricValue::U64(0)
        );
        metrics.with(MockMeasurementAttributes::Second).values[0] = MetricValue::U64(7);

        let snapshots = metrics.pending_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].descriptor().name, "test_metrics");
        assert_eq!(
            snapshots[0].measurement_attribute_value("outcome"),
            Some("second")
        );
        assert_eq!(
            snapshots[0].measurement_attributes().collect::<Vec<_>>(),
            vec![("outcome", "second"), ("reason", "one")]
        );
        assert_eq!(
            snapshots[0].get_metrics(),
            &[MetricValue::U64(7), MetricValue::U64(0)]
        );
    }

    #[test]
    fn test_measurement_metric_set_terminal_snapshots_take_touched_buckets() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "value");
        let mut registry = MetricSetRegistry::default();
        let mut metrics: MeasurementMetricSet<MockMetricSet> =
            registry.register_with_measurement_attributes(entity_key);

        metrics.with(MockMeasurementAttributes::Second).values[0] = MetricValue::U64(7);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].bucket(), 1);
        assert_eq!(
            snapshots[0].measurement_attribute_value("outcome"),
            Some("second")
        );
        assert_eq!(
            metrics.get(MockMeasurementAttributes::Second).values[0],
            MetricValue::U64(0)
        );
        assert!(metrics.terminal_snapshots().is_empty());
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

        metrics.accumulate_snapshot(
            metrics_key,
            0,
            &[MetricValue::U64(10), MetricValue::U64(20)],
        );
        metrics.accumulate_snapshot(metrics_key, 0, &[MetricValue::U64(5), MetricValue::U64(15)]);

        let mut accumulated_values = Vec::new();
        metrics.visit_and_reset_with_item_attrs(
            &entities,
            |_desc, _attrs, _dp, iter| {
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

        metrics.accumulate_snapshot(
            invalid_key,
            0,
            &[MetricValue::U64(10), MetricValue::U64(20)],
        );
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
            0,
            &[MetricValue::U64(u64::MAX), MetricValue::U64(u64::MAX - 5)],
        );
        metrics.accumulate_snapshot(
            metrics_key,
            0,
            &[MetricValue::U64(10), MetricValue::U64(10)],
        );

        let mut accumulated_values = Vec::new();
        metrics.visit_and_reset_with_item_attrs(
            &entities,
            |_desc, _attrs, _dp, iter| {
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

        metrics.accumulate_snapshot(metrics_key, 0, &[MetricValue::U64(u64::MAX)]);
        metrics.accumulate_snapshot(metrics_key, 0, &[MetricValue::U64(1)]);
    }

    #[test]
    fn test_visit_metrics_and_reset() {
        let mut entities = EntityRegistry::default();
        let entity_key = register_entity(&mut entities, "test_value");
        let mut metrics = MetricSetRegistry::default();

        let metric_set: MetricSet<MockMetricSet> = metrics.register(entity_key);
        let metrics_key = metric_set.key;

        metrics.accumulate_snapshot(
            metrics_key,
            0,
            &[MetricValue::U64(100), MetricValue::U64(0)],
        );

        let mut visit_count = 0;
        let mut collected_values = Vec::new();

        metrics.visit_and_reset_with_item_attrs(
            &entities,
            |desc, _attrs, _dp, iter| {
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

        metrics.visit_and_reset_with_item_attrs(
            &entities,
            |_desc, _attrs, _dp, _iter| {
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

        metrics.accumulate_snapshot(metrics_key, 0, &[MetricValue::U64(5), MetricValue::U64(10)]);
        metrics.accumulate_snapshot(metrics_key, 0, &[MetricValue::U64(2), MetricValue::U64(3)]);

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

        metrics.accumulate_snapshot(metrics_key, 0, &[MetricValue::U64(10)]);
        metrics.accumulate_snapshot(metrics_key, 0, &[MetricValue::U64(15)]);

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
            0,
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
            0,
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
