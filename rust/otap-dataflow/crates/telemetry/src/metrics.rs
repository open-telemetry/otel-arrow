// SPDX-License-Identifier: Apache-2.0

//! Core multivariate metrics traits and types.
//!
//! This module intentionally contains no product-specific metrics definitions. Concrete metrics
//! live in their respective nodes/crates and implement the `MultivariateMetrics` trait defined
//! here.

use crate::attributes::NodeStaticAttrs;
use crate::descriptor::{MetricsDescriptor, MetricsField};
use crate::error::Error;
use crate::registry::MetricsRegistry;
use std::any::Any;

/// Type representing a snapshot of multivariate metrics.
pub type MetricsSnapshot = Box<dyn MultivariateMetrics + Send + Sync>;

/// Trait for types that can aggregate their metrics into a `MetricsRegistry`.
pub trait MultivariateMetrics: Any + Send + Sync {
    /// Register the current multivariate metrics into the metrics registry.
    #[doc(hidden)]
    fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs);

    /// Returns the descriptor for this set of metrics.
    fn descriptor(&self) -> &'static MetricsDescriptor;

    /// Iterate over (descriptor_field, current_value) pairs in defined order.
    fn field_values(&self) -> Box<dyn Iterator<Item=(&'static MetricsField, u64)> + '_>;

    /// Merges the values from `other` into `self`.
    ///
    /// Implementations MUST assume `other` is of the same concrete type. Callers should ensure
    /// type compatibility using `descriptor()` or by trying a downcast.
    fn merge_from_same_kind(&mut self, other: &dyn MultivariateMetrics);

    /// Aggregates the metrics of this type into the provided registry (identified by a key that
    /// must have been set at registration time by the implementer).
    fn aggregate_into(&self, registry: &mut crate::registry::MetricsRegistryHandle) -> Result<(), Error>;

    /// Resets all metrics to zero / default.
    fn zero(&mut self);

    /// Returns true if at least one metric has a non-zero value.
    fn has_non_zero(&self) -> bool {
        self.field_values().any(|(_, v)| v != 0)
    }

    /// Downcast helper for dynamic dispatch.
    fn as_any(&self) -> &dyn Any;
    /// Downcast helper for dynamic dispatch (mutable).
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{MetricsDescriptor, MetricsField, MetricsKind};
    use crate::registry::{MetricsKey, MetricsRegistry, MetricsRegistryHandle};
    use crate::attributes::NodeStaticAttrs;

    #[derive(Clone, Default)]
    struct TestMetrics {
        key: Option<MetricsKey>,
        v: u64,
    }

    const TEST_DESC: MetricsDescriptor = MetricsDescriptor {
        name: "test.metrics",
        fields: &[MetricsField { name: "v", unit: "{unit}", kind: MetricsKind::Counter }],
    };

    impl MultivariateMetrics for TestMetrics {
        fn register_into(&mut self, registry: &mut MetricsRegistry, attrs: NodeStaticAttrs) {
            let key = registry.insert_default::<Self>(attrs);
            self.key = Some(key);
        }
        fn descriptor(&self) -> &'static MetricsDescriptor { &TEST_DESC }
        fn field_values(&self) -> Box<dyn Iterator<Item=(&'static MetricsField, u64)> + '_> {
            Box::new(TEST_DESC.fields.iter().zip([self.v].into_iter()).map(|(f, v)| (f, v)))
        }
        fn merge_from_same_kind(&mut self, other: &dyn MultivariateMetrics) {
            let other = other.as_any().downcast_ref::<Self>().unwrap();
            self.v += other.v;
        }
        fn aggregate_into(&self, registry: &mut MetricsRegistryHandle) -> Result<(), Error> {
            if let Some(key) = self.key { registry.add_metrics(key, self); Ok(()) } else { Err(Error::MetricsNotRegistered { descriptor: self.descriptor() }) }
        }
        fn zero(&mut self) { self.v = 0; }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    #[test]
    fn test_trait_basics() {
        let mut reg = MetricsRegistryHandle::new();
        let mut m = TestMetrics::default();
        reg.register(&mut m, NodeStaticAttrs { node_id: "n".into(), node_type: "t".into(), pipeline_id: "p".into(), core_id: 0, numa_node_id: 0, process_id: 0 });
        m.v = 10;
        m.aggregate_into(&mut reg).unwrap();
        assert_eq!(reg.len(), 1);
        // flush loop should see non-zero then zero them
        let mut seen = 0;
        reg.for_each_changed_and_zero(|mm, _| {
            for (_, val) in mm.field_values() { assert_eq!(val, 10); }
            seen += 1;
        });
        assert_eq!(seen, 1);
        // second pass: zeros
        seen = 0;
        reg.for_each_changed_and_zero(|_, _| { seen += 1; });
        assert_eq!(seen, 0);
    }
}