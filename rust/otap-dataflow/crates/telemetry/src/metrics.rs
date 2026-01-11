// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core multivariate metrics traits and types.
//!
//! This module intentionally contains no product-specific metrics definitions. Concrete metrics
//! live in their respective nodes/crates and implement the `MetricSetHandler` trait defined
//! here.

pub mod dispatcher;

use crate::descriptor::MetricsDescriptor;
use crate::registry::{EntityKey, MetricSetKey};
use serde::Serialize;
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
