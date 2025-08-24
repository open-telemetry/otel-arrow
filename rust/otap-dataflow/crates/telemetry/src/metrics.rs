// SPDX-License-Identifier: Apache-2.0

//! Core multivariate metrics traits and types.
//!
//! This module intentionally contains no product-specific metrics definitions. Concrete metrics
//! live in their respective nodes/crates and implement the `MetricSetHandler` trait defined
//! here.

use crate::descriptor::MetricsDescriptor;
use crate::registry::MetricsKey;
use std::ops::{Deref, DerefMut};

/// A concrete set of metrics values grouped under a single descriptor/key.
#[derive(Clone)]
pub struct MetricSet<M: MetricSetHandler> {
    pub(crate) key: MetricsKey,
    pub(crate) metrics: M,
}

impl<M: MetricSetHandler> MetricSet<M> {
    /// Creates a snapshot of the current metrics values.
    pub fn snapshot(&self) -> MetricSetSnapshot {
        MetricSetSnapshot {
            key: self.key.clone(),
            metrics: self.metrics.snapshot_values(),
        }
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

/// Immutable snapshot of a metric set's current values.
pub struct MetricSetSnapshot {
    pub(crate) key: MetricsKey,
    pub(crate) metrics: Vec<u64>,
}

/// Handler trait implemented by generated metric set structs (see 'metric_set' proc macro).
pub trait MetricSetHandler {
    /// Returns the static descriptor describing this metric set (name + ordered fields).
    fn descriptor(&self) -> &'static MetricsDescriptor;
    /// Returns a snapshot of all metric field values in descriptor order.
    fn snapshot_values(&self) -> Vec<u64>;
    /// Resets all metric field values to zero.
    fn clear_values(&mut self);
    /// Returns true if at least one metric value is non-zero (fast path check).
    fn needs_flush(&self) -> bool;
}
