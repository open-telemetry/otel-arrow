// SPDX-License-Identifier: Apache-2.0

//! Core multivariate metrics traits and types.
//!
//! This module intentionally contains no product-specific metrics definitions. Concrete metrics
//! live in their respective nodes/crates and implement the `MultivariateMetrics` trait defined
//! here.

use crate::descriptor::{MetricsDescriptor, MetricsField};
use crate::registry::MetricsKey;
use std::ops::{Deref, DerefMut};

/// Type representing a set of multivariate metrics with a unique key.
#[derive(Clone)]
pub struct MvMetrics<M: MultivariateMetrics> {
    /// The metrics key for this set of metrics.
    pub(crate) key: MetricsKey,
    /// The actual multivariate metrics instance.
    pub(crate) metrics: M,
}

impl<M: MultivariateMetrics> MvMetrics<M> {
    /// Creates a snapshot of the current metrics values.
    pub fn snapshot(&self) -> MvMetricsSnapshot {
        MvMetricsSnapshot {
            key: self.key.clone(),
            metrics: self.metrics.to_vec(),
        }
    }
}

impl<M: MultivariateMetrics> Deref for MvMetrics<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.metrics
    }
}

impl<M: MultivariateMetrics> DerefMut for MvMetrics<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.metrics
    }
}

/// A snapshot of the current values of a set of multivariate metrics.
pub struct MvMetricsSnapshot {
    /// The metrics key for this set of metrics.
    pub(crate) key: MetricsKey,
    /// The actual multivariate metrics values.
    pub(crate) metrics: Vec<u64>,
}

/// Trait for types that can aggregate their metrics into a `MetricsRegistry`.
pub trait MultivariateMetrics {
    /// Returns the descriptor for this set of metrics.
    fn descriptor(&self) -> &'static MetricsDescriptor;

    /// Iterate over (descriptor_field, current_value) pairs in defined order.
    fn field_values(&self) -> Box<dyn Iterator<Item = (&'static MetricsField, u64)> + '_>;

    /// Returns the current metric values collected into a Vec<u64>.
    ///
    /// The order of values matches the order of fields in `descriptor().fields`.
    /// Implementors using the derive/attribute macro get an optimized implementation;
    /// the default uses `field_values()`.
    fn to_vec(&self) -> Vec<u64> {
        let mut out = Vec::with_capacity(self.descriptor().fields.len());
        for (_, v) in self.field_values() {
            out.push(v);
        }
        out
    }

    /// Resets all metrics to zero / default.
    fn zero(&mut self);

    /// Returns true if at least one metric has a non-zero value.
    fn has_non_zero(&self) -> bool {
        self.field_values().any(|(_, v)| v != 0)
    }
}