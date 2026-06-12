// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observer context for embedders using the controller as a library.

use otap_df_state::store::ObservedStateHandle;
use otap_df_telemetry::registry::TelemetryRegistryHandle;

/// Context provided to observer callbacks during controller startup.
///
/// Gives embedders zero-overhead, in-process access to pipeline state
/// and internal metrics without requiring the HTTP admin server.
///
/// Both handles are cheaply cloneable (`Arc`-based) and safe to move
/// into background threads.
#[derive(Debug, Clone)]
pub struct EngineObserverContext {
    state: ObservedStateHandle,
    telemetry: TelemetryRegistryHandle,
}

impl EngineObserverContext {
    /// Creates a new observer context from the given handles.
    pub(crate) fn new(state: ObservedStateHandle, telemetry: TelemetryRegistryHandle) -> Self {
        Self { state, telemetry }
    }

    /// Returns a reference to the observed pipeline state handle.
    ///
    /// Use this to query pipeline liveness, readiness, and health status
    /// without going through the admin HTTP server.
    #[must_use]
    pub fn state_handle(&self) -> &ObservedStateHandle {
        &self.state
    }

    /// Returns a reference to the telemetry registry handle.
    ///
    /// Use this to read internal metrics snapshots (e.g. per-node throughput,
    /// queue depths, processing latencies) without enabling the admin HTTP
    /// server. The returned handle is shared and supports both read and write
    /// operations; writes can affect what other readers observe.
    #[must_use]
    pub fn telemetry_handle(&self) -> &TelemetryRegistryHandle {
        &self.telemetry
    }

    /// Consumes the context and returns both handles.
    #[must_use]
    pub fn into_parts(self) -> (ObservedStateHandle, TelemetryRegistryHandle) {
        (self.state, self.telemetry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::PipelineKey;
    use otap_df_config::observed_state::ObservedStateSettings;
    use otap_df_state::store::ObservedStateStore;
    use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};
    use otap_df_telemetry::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, Instrument, MetricValueType,
        MetricsDescriptor, MetricsField, Temporality,
    };
    use otap_df_telemetry::metrics::{MetricSetHandler, MetricValue};
    use std::borrow::Cow;

    // Mock metric types (same pattern as registry.rs tests)

    #[derive(Debug)]
    struct MockMetricSet {
        values: Vec<MetricValue>,
    }

    impl Default for MockMetricSet {
        fn default() -> Self {
            Self {
                values: vec![MetricValue::from(0u64), MetricValue::from(0u64)],
            }
        }
    }

    static MOCK_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "observer_test_metrics",
        metrics: &[
            MetricsField {
                name: "items_processed",
                unit: "1",
                brief: "Number of items processed",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "queue_depth",
                unit: "1",
                brief: "Current queue depth",
                instrument: Instrument::Gauge,
                temporality: None,
                value_type: MetricValueType::U64,
            },
        ],
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

    static MOCK_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "observer_test_attributes",
        fields: &[AttributeField {
            key: "node.name",
            r#type: AttributeValueType::String,
            brief: "Node name",
        }],
    };

    #[derive(Debug)]
    struct MockAttributeSet {
        attribute_values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(name: &str) -> Self {
            Self {
                attribute_values: vec![AttributeValue::String(name.to_string())],
            }
        }
    }

    impl AttributeSetHandler for MockAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &MOCK_ATTRIBUTES_DESCRIPTOR
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.attribute_values
        }
    }

    // Helpers

    fn make_context_with_store() -> (EngineObserverContext, ObservedStateStore) {
        let registry = TelemetryRegistryHandle::new();
        let store = ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
        let ctx = EngineObserverContext::new(store.handle(), registry);
        (ctx, store)
    }

    fn make_context() -> EngineObserverContext {
        make_context_with_store().0
    }

    fn test_pipeline_key() -> PipelineKey {
        PipelineKey::new(Cow::Borrowed("test_group"), Cow::Borrowed("test_pipeline"))
    }

    // Basic accessor tests

    #[test]
    fn accessors_return_valid_handles() {
        let ctx = make_context();
        // state_handle should return an empty snapshot (no pipelines registered).
        assert!(ctx.state_handle().snapshot().is_empty());
        // telemetry_handle is accessible.
        let _telemetry = ctx.telemetry_handle();
    }

    #[test]
    fn clone_shares_underlying_state() {
        let ctx = make_context();
        let cloned = ctx.clone();
        // Both clones should see the same (empty) snapshot.
        assert_eq!(
            ctx.state_handle().snapshot().len(),
            cloned.state_handle().snapshot().len()
        );
    }

    #[test]
    fn into_parts_yields_both_handles() {
        let ctx = make_context();
        let (state, telemetry) = ctx.into_parts();
        assert!(state.snapshot().is_empty());
        let _telemetry = telemetry;
    }

    // ObservedStateHandle data tests

    #[test]
    fn state_handle_reflects_pipeline_data() {
        let (ctx, store) = make_context_with_store();
        let key = test_pipeline_key();

        // Populate the store with pipeline state.
        store.set_pipeline_active_generation(key.clone(), 1);
        store.set_pipeline_active_cores(key.clone(), vec![0, 1, 2]);
        store.set_pipeline_serving_generation(key.clone(), 0, 1);

        // The handle should see the populated pipeline.
        let snapshot = ctx.state_handle().snapshot();
        assert_eq!(snapshot.len(), 1);
        assert!(snapshot.contains_key(&key));

        let status = ctx.state_handle().pipeline_status(&key);
        assert!(status.is_some());
        let status = status.unwrap();
        assert_eq!(status.active_generation(), Some(1));
        // serving_generations tracks which generation each core is running.
        assert_eq!(status.serving_generations().len(), 1);
        assert_eq!(status.serving_generations().get(&0), Some(&1));
    }

    #[test]
    fn state_handle_shared_with_store() {
        let (ctx, store) = make_context_with_store();
        let key = test_pipeline_key();

        // Handle is created before any data is added.
        assert!(ctx.state_handle().snapshot().is_empty());

        // Mutate the store after the handle was created.
        store.set_pipeline_active_generation(key.clone(), 0);
        store.set_pipeline_active_cores(key.clone(), vec![0]);
        store.set_pipeline_serving_generation(key.clone(), 0, 0);

        // The handle should see the new data (Arc sharing).
        assert_eq!(ctx.state_handle().snapshot().len(), 1);
        assert!(ctx.state_handle().pipeline_status(&key).is_some());
    }

    // TelemetryRegistryHandle data tests

    #[test]
    fn telemetry_handle_register_and_read_metrics() {
        let ctx = make_context();
        let telemetry = ctx.telemetry_handle();

        // Register a metric set.
        let metric_set =
            telemetry.register_metric_set::<MockMetricSet>(MockAttributeSet::new("node_a"));
        assert_eq!(telemetry.metric_set_count(), 1);

        // Accumulate a snapshot with known values.
        let key = metric_set.metrics_key();
        telemetry.accumulate_metric_set_snapshot(
            key,
            &[MetricValue::from(42u64), MetricValue::from(7u64)],
        );

        // Read back via visit_current_metrics (non-destructive).
        let mut visited = false;
        telemetry.visit_current_metrics(|descriptor, _attrs, iter| {
            assert_eq!(descriptor.name, "observer_test_metrics");
            let values: Vec<_> = iter.collect();
            assert_eq!(values.len(), 2);
            assert_eq!(values[0].0.name, "items_processed");
            assert_eq!(values[1].0.name, "queue_depth");
            visited = true;
        });
        assert!(visited, "visit_current_metrics callback was not invoked");
    }

    #[test]
    fn telemetry_handle_shared_across_clones() {
        let ctx = make_context();
        let handle_a = ctx.telemetry_handle().clone();
        let handle_b = ctx.telemetry_handle().clone();

        // Register through one clone.
        let _metric_set =
            handle_a.register_metric_set::<MockMetricSet>(MockAttributeSet::new("shared_node"));

        // Visible through the other clone.
        assert_eq!(handle_b.metric_set_count(), 1);
        assert_eq!(handle_b.entity_count(), 1);
    }

    // Cross-thread test

    #[test]
    fn both_handles_usable_from_spawned_thread() {
        let (ctx, store) = make_context_with_store();
        let key = test_pipeline_key();

        // Populate state before spawning.
        store.set_pipeline_active_generation(key.clone(), 2);
        store.set_pipeline_active_cores(key.clone(), vec![0, 1]);
        store.set_pipeline_serving_generation(key.clone(), 0, 2);

        // Register metrics before spawning.
        let metric_set = ctx
            .telemetry_handle()
            .register_metric_set::<MockMetricSet>(MockAttributeSet::new("thread_node"));
        ctx.telemetry_handle().accumulate_metric_set_snapshot(
            metric_set.metrics_key(),
            &[MetricValue::from(100u64), MetricValue::from(5u64)],
        );

        // Clone the context and move it to another thread.
        let ctx_clone = ctx.clone();
        let key_clone = key.clone();
        let handle = std::thread::spawn(move || {
            // Verify state handle works from another thread.
            let snapshot = ctx_clone.state_handle().snapshot();
            assert_eq!(snapshot.len(), 1);
            let status = ctx_clone
                .state_handle()
                .pipeline_status(&key_clone)
                .unwrap();
            assert_eq!(status.active_generation(), Some(2));
            assert_eq!(status.serving_generations().len(), 1);

            // Verify telemetry handle works from another thread.
            assert_eq!(ctx_clone.telemetry_handle().metric_set_count(), 1);
            let mut visited = false;
            ctx_clone
                .telemetry_handle()
                .visit_current_metrics(|desc, _attrs, _iter| {
                    assert_eq!(desc.name, "observer_test_metrics");
                    visited = true;
                });
            assert!(visited);
        });
        handle.join().expect("spawned thread panicked");
    }
}
