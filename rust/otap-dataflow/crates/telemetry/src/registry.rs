// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The telemetry registry component combining entity and metrics registries (see the ITS diagram
//! architecture in the main [README.md](../README.md) file of this crate.

use crate::attributes::AttributeSetHandler;
use crate::descriptor::MetricsDescriptor;
use crate::entity::{EntityRegistry, RegisterOutcome};
use crate::metrics::{
    MeasurementMetricSet, MeasurementMetricSetHandler, MetricExportBatch, MetricExportCheckpoint,
    MetricSet, MetricSetHandler, MetricSetRegistry, MetricSetUnregister, MetricValue,
    MetricsIterator, RegistrationMetricSetHandler,
};
use crate::otel_debug;
use crate::reporter::MetricsFlushHandle;
use crate::semconv::SemConvRegistry;
use parking_lot::Mutex;
use slotmap::new_key_type;
use std::fmt::Debug;
use std::sync::Arc;

new_key_type! {
    /// This key is used to identify a specific entity entry in the entity registry (slotmap index).
    pub struct EntityKey;

    /// This key is used to identify a specific metrics entry in the metric set registry (slotmap
    /// index).
    pub struct MetricSetKey;
}

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
    pub(crate) registry: Arc<Mutex<TelemetryRegistry>>,
    metrics_flusher: Arc<Mutex<Option<MetricsFlushHandle>>>,
}

/// An owned metrics export that is committed only after downstream delivery.
///
/// Dropping this value without calling [`Self::commit`] restores resettable
/// values to the registry, merging them with values accumulated while the
/// export was in flight. Current-value instruments retain their newest value
/// and are marked dirty so the next export retries them.
#[derive(Debug)]
#[must_use = "an uncommitted metrics export is rolled back when dropped"]
pub struct MetricExportTransaction {
    registry: TelemetryRegistryHandle,
    /// Kept in an `Option` so commit can take ownership and disarm [`Drop`].
    batch: Option<MetricExportBatch>,
    /// Parallel to `batch.metric_sets`; maps owned values back to registry entries.
    checkpoints: Vec<MetricExportCheckpoint>,
}

impl MetricExportTransaction {
    /// Returns the owned batch to encode and deliver.
    #[must_use]
    pub fn batch(&self) -> &MetricExportBatch {
        self.batch.as_ref().expect("export transaction has a batch")
    }

    /// Commits successful delivery and returns the exported batch.
    #[must_use]
    pub fn commit(mut self) -> MetricExportBatch {
        {
            let mut reg = self.registry.registry.lock();
            let TelemetryRegistry {
                entities, metrics, ..
            } = &mut *reg;
            metrics.commit_export_batch(entities, &self.checkpoints);
        }
        self.checkpoints.clear();
        self.batch.take().expect("export transaction has a batch")
    }
}

impl Drop for MetricExportTransaction {
    fn drop(&mut self) {
        if let Some(batch) = &self.batch {
            self.registry
                .registry
                .lock()
                .metrics
                .rollback_export_batch(batch, &self.checkpoints);
        }
    }
}

/// The main telemetry registry maintaining both entity and metric set registries.
#[derive(Debug, Default)]
pub(crate) struct TelemetryRegistry {
    pub(crate) entities: EntityRegistry,
    pub(crate) metrics: MetricSetRegistry,
    defer_dirty_metric_unregistration: bool,
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
            metrics_flusher: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn configure_metrics_collector(
        &self,
        flusher: Option<MetricsFlushHandle>,
        defer_dirty_unregistration: bool,
    ) {
        *self.metrics_flusher.lock() = flusher;
        self.registry.lock().defer_dirty_metric_unregistration = defer_dirty_unregistration;
    }

    /// Flushes snapshots accepted by the internal metrics collector, if one is configured.
    pub async fn flush_pending_metrics(&self) -> Result<(), crate::error::Error> {
        let flusher = self.metrics_flusher.lock().clone();
        match flusher {
            Some(flusher) => flusher.flush().await,
            None => Ok(()),
        }
    }

    /// Registers (or reuses) an entity for the provided attribute set.
    /// Logs the entity definition when a new entity is created.
    pub fn register_entity(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> EntityKey {
        let schema = attrs.schema_name();
        let definition = attrs.attributes_to_string();
        let outcome = self.registry.lock().entities.register(attrs);
        if let RegisterOutcome::Created(_) = outcome {
            // Log the entity definition.
            //
            // TODO(#1907): This could benefit from logging a human-readable form
            // of the entity that we refer to later in the logs, instead of logging
            // every key/value in every line of console_async output.
            otel_debug!("registry.define_entity", schema, definition);
        }
        outcome.key()
    }

    /// Unregisters an entity by key.
    #[must_use]
    pub fn unregister_entity(&self, entity_key: EntityKey) -> bool {
        self.registry.lock().entities.unregister(entity_key)
    }

    /// Returns the total number of registered entities.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.registry.lock().entities.len()
    }

    /// Visits a single entity by key.
    pub fn visit_entity<F, T>(&self, key: EntityKey, f: F) -> Option<T>
    where
        F: FnOnce(&dyn AttributeSetHandler) -> T,
    {
        let reg = self.registry.lock();
        reg.entities.get(key).map(|attrs| f(attrs))
    }

    fn register_metric_set_for_new_entity<A, R>(
        &self,
        scope_attrs: A,
        register: impl FnOnce(&mut MetricSetRegistry, EntityKey) -> R,
    ) -> R
    where
        A: AttributeSetHandler + Send + Sync + 'static,
    {
        let mut registry = self.registry.lock();
        let entity_key = registry.entities.register(scope_attrs).key();
        register(&mut registry.metrics, entity_key)
    }

    fn register_metric_set_for_existing_entity<R>(
        &self,
        entity_key: EntityKey,
        register: impl FnOnce(&mut MetricSetRegistry, EntityKey) -> R,
    ) -> R {
        let mut registry = self.registry.lock();
        let retained = registry.entities.retain(entity_key);
        debug_assert!(retained, "entity key must be registered before metrics");
        register(&mut registry.metrics, entity_key)
    }

    /// Registers a metric set type with the given scope attributes and returns a `MetricSet`
    /// instance that can be used to report metrics for that type.
    pub fn register_metric_set<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> MetricSet<T> {
        // TODO: Note this code path is not logged the way entity registration
        // does for referring to in console logs. Will be needed to print metrics
        // to the console.
        self.register_metric_set_for_new_entity(attrs, MetricSetRegistry::register::<T>)
    }

    /// Registers a metric set type for an existing entity key.
    #[must_use]
    pub fn register_metric_set_for_entity<T: MetricSetHandler + Default + Debug + Send + Sync>(
        &self,
        entity_key: EntityKey,
    ) -> MetricSet<T> {
        self.register_metric_set_for_existing_entity(entity_key, MetricSetRegistry::register::<T>)
    }

    /// Internal registrar operation for registration-time item attributes.
    ///
    /// This is public only so engine contexts and generated metric-set `register(...)`
    /// methods (e.g. `MyMetrics::register`) can select an entity scope. Component code
    /// must use the generated `MyMetrics::register(...)` method.
    #[doc(hidden)]
    pub fn register_metric_set_with_registration_attributes<
        M: RegistrationMetricSetHandler + Debug + Send + Sync,
    >(
        &self,
        scope_attrs: impl AttributeSetHandler + Send + Sync + 'static,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MetricSet<M> {
        let registration_attributes = capture_registration_attributes(registration_attrs);
        self.register_metric_set_for_new_entity(scope_attrs, |metrics, entity_key| {
            metrics.register_with_registration_attributes(entity_key, registration_attributes)
        })
    }

    /// Internal registrar operation for an existing entity key.
    #[must_use]
    #[doc(hidden)]
    pub fn register_metric_set_with_registration_attributes_for_entity<
        M: RegistrationMetricSetHandler + Debug + Send + Sync,
    >(
        &self,
        entity_key: EntityKey,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MetricSet<M> {
        let registration_attributes = capture_registration_attributes(registration_attrs);
        self.register_metric_set_for_existing_entity(entity_key, |metrics, entity_key| {
            metrics.register_with_registration_attributes(entity_key, registration_attributes)
        })
    }

    /// Internal registrar operation for a measurement metric set.
    #[doc(hidden)]
    pub fn register_metric_set_with_measurement_attributes<
        M: MeasurementMetricSetHandler + Debug + Send + Sync,
    >(
        &self,
        scope_attrs: impl AttributeSetHandler + Send + Sync + 'static,
    ) -> MeasurementMetricSet<M> {
        self.register_metric_set_for_new_entity(scope_attrs, |metrics, entity_key| {
            metrics.register_with_measurement_attributes::<M>(entity_key)
        })
    }

    /// Internal registrar operation for a measurement metric set on an existing entity.
    #[must_use]
    #[doc(hidden)]
    pub fn register_metric_set_with_measurement_attributes_for_entity<
        M: MeasurementMetricSetHandler + Debug + Send + Sync,
    >(
        &self,
        entity_key: EntityKey,
    ) -> MeasurementMetricSet<M> {
        self.register_metric_set_for_existing_entity(entity_key, |metrics, entity_key| {
            metrics.register_with_measurement_attributes::<M>(entity_key)
        })
    }

    /// Internal registrar operation for registration and measurement attributes.
    #[must_use]
    #[doc(hidden)]
    pub fn register_metric_set_with_registration_and_measurement_attributes<
        M: RegistrationMetricSetHandler + MeasurementMetricSetHandler + Debug + Send + Sync,
    >(
        &self,
        scope_attrs: impl AttributeSetHandler + Send + Sync + 'static,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MeasurementMetricSet<M> {
        let registration_attributes = capture_registration_attributes(registration_attrs);
        self.register_metric_set_for_new_entity(scope_attrs, |metrics, entity_key| {
            metrics.register_with_registration_and_measurement_attributes::<M>(
                entity_key,
                registration_attributes,
            )
        })
    }

    /// Internal registrar operation for combined attributes on an existing entity.
    #[must_use]
    #[doc(hidden)]
    pub fn register_metric_set_with_registration_and_measurement_attributes_for_entity<
        M: RegistrationMetricSetHandler + MeasurementMetricSetHandler + Debug + Send + Sync,
    >(
        &self,
        entity_key: EntityKey,
        registration_attrs: &M::RegistrationAttributes,
    ) -> MeasurementMetricSet<M> {
        let registration_attributes = capture_registration_attributes(registration_attrs);
        self.register_metric_set_for_existing_entity(entity_key, |metrics, entity_key| {
            metrics.register_with_registration_and_measurement_attributes::<M>(
                entity_key,
                registration_attributes,
            )
        })
    }

    /// Unregisters a metric set by key.
    #[must_use]
    pub fn unregister_metric_set(&self, metrics_key: MetricSetKey) -> bool {
        let mut reg = self.registry.lock();
        let defer_dirty_unregistration = reg.defer_dirty_metric_unregistration;
        match reg
            .metrics
            .unregister(metrics_key, defer_dirty_unregistration)
        {
            Some(MetricSetUnregister::Removed(entity_key)) => {
                let _ = reg.entities.unregister(entity_key);
                true
            }
            Some(MetricSetUnregister::Deferred) => true,
            None => false,
        }
    }

    /// Adds a new metrics snapshot to the aggregator for the given key and bucket.
    pub fn accumulate_metric_set_snapshot(
        &self,
        metric_set_key: MetricSetKey,
        bucket: usize,
        metrics: &[MetricValue],
    ) {
        self.registry
            .lock()
            .metrics
            .accumulate_snapshot(metric_set_key, bucket, metrics);
    }

    /// Returns the total number of registered metric sets.
    #[must_use]
    pub fn metric_set_count(&self) -> usize {
        self.registry.lock().metrics.len()
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
    pub fn visit_metrics_and_reset_with_zeroes<F>(&self, mut f: F, keep_all_zeroes: bool)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        self.visit_metrics_and_reset_with_item_attrs(
            |desc, attrs, _dp, iter| f(desc, attrs, iter),
            keep_all_zeroes,
        );
    }

    /// Visits every non-empty item bucket, yielding the per-item
    /// enum/registration attributes (`&[(key, value)]`) in addition to scope attributes,
    /// then resets the visited bucket to zero.
    ///
    /// The item attributes are empty for plain metric sets; the primary
    /// consumer is the metrics dispatcher, which attaches them to the emitted
    /// OpenTelemetry data points.
    pub fn visit_metrics_and_reset_with_item_attrs<F>(&self, f: F, keep_all_zeroes: bool)
    where
        for<'a> F: FnMut(
            &'static MetricsDescriptor,
            &'a dyn AttributeSetHandler,
            &'a [(&'a str, &'a str)],
            MetricsIterator<'a>,
        ),
    {
        let mut reg = self.registry.lock();
        let TelemetryRegistry {
            entities, metrics, ..
        } = &mut *reg;
        metrics.visit_and_reset_with_item_attrs(entities, f, keep_all_zeroes);
    }

    /// Atomically drains the pending metrics export accumulator into an owned batch.
    ///
    /// Encoding and downstream waits can safely happen after this method returns;
    /// neither holds the registry mutex.
    #[must_use]
    pub fn drain_metric_export_batch(&self) -> MetricExportBatch {
        self.begin_metric_export_batch().commit()
    }

    #[cfg(test)]
    pub(crate) fn drain_metric_export_batch_at(&self, time_unix_nano: u64) -> MetricExportBatch {
        self.begin_metric_export_batch_at(time_unix_nano).commit()
    }

    /// Starts an export transaction that rolls back unless delivery is committed.
    ///
    /// Encoding and downstream delivery happen without holding the registry
    /// lock. Call [`MetricExportTransaction::commit`] only after delivery has
    /// succeeded; errors and future cancellation safely trigger rollback.
    pub fn begin_metric_export_batch(&self) -> MetricExportTransaction {
        self.begin_metric_export_batch_at(crate::metrics::unix_time_nanos())
    }

    /// Starts a transaction using an explicit collection time.
    ///
    /// The explicit timestamp keeps registry tests deterministic; production
    /// callers use [`Self::begin_metric_export_batch`].
    pub(crate) fn begin_metric_export_batch_at(
        &self,
        time_unix_nano: u64,
    ) -> MetricExportTransaction {
        let mut reg = self.registry.lock();
        let TelemetryRegistry {
            entities, metrics, ..
        } = &mut *reg;
        let (batch, checkpoints) = metrics.begin_export_batch(entities, time_unix_nano);
        MetricExportTransaction {
            registry: self.clone(),
            batch: Some(batch),
            checkpoints,
        }
    }

    /// Visits the admin metrics accumulator, then resets only that accumulator.
    ///
    /// This does not consume data pending for ITS or the OpenTelemetry SDK.
    pub fn visit_admin_metrics_and_reset<F>(&self, f: F)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        self.visit_admin_metrics_and_reset_with_zeroes(f, false);
    }

    /// Visits and resets the admin accumulator, optionally retaining zero-valued sets.
    pub fn visit_admin_metrics_and_reset_with_zeroes<F>(&self, mut f: F, keep_all_zeroes: bool)
    where
        for<'a> F:
            FnMut(&'static MetricsDescriptor, &'a dyn AttributeSetHandler, MetricsIterator<'a>),
    {
        self.visit_admin_metrics_and_reset_with_item_attrs(
            |desc, attrs, _item, iter| f(desc, attrs, iter),
            keep_all_zeroes,
        );
    }

    /// Visits and resets the admin accumulator with per-item attributes.
    pub fn visit_admin_metrics_and_reset_with_item_attrs<F>(&self, f: F, keep_all_zeroes: bool)
    where
        for<'a> F: FnMut(
            &'static MetricsDescriptor,
            &'a dyn AttributeSetHandler,
            &'a [(&'a str, &'a str)],
            MetricsIterator<'a>,
        ),
    {
        let mut reg = self.registry.lock();
        let TelemetryRegistry {
            entities, metrics, ..
        } = &mut *reg;
        metrics.visit_admin_metrics_and_reset(entities, f, keep_all_zeroes);
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
        self.visit_current_metrics_with_item_attrs(
            |desc, attrs, _dp, iter| f(desc, attrs, iter),
            keep_all_zeroes,
        );
    }

    /// Read-only variant of [`Self::visit_metrics_and_reset_with_item_attrs`]
    /// that does not reset bucket values.
    pub fn visit_current_metrics_with_item_attrs<F>(&self, f: F, keep_all_zeroes: bool)
    where
        for<'a> F: FnMut(
            &'static MetricsDescriptor,
            &'a dyn AttributeSetHandler,
            &'a [(&'a str, &'a str)],
            MetricsIterator<'a>,
        ),
    {
        let reg = self.registry.lock();
        reg.metrics
            .visit_current_with_item_attrs(&reg.entities, f, keep_all_zeroes);
    }
}

/// Captures registration attributes as owned strings for storage on a metric set entry.
fn capture_registration_attributes(attrs: &dyn AttributeSetHandler) -> Vec<(String, String)> {
    attrs
        .iter_attributes()
        .map(|(k, v)| (k.to_string(), v.to_string_value()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{
        AttributeSetHandler, AttributeSetKeySchema, AttributeValue, MeasurementAttributeSet,
    };
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, Instrument,
        MeasurementAttributeDescriptor, MetricValueType, MetricsField, Temporality,
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
                values: vec![MetricValue::from(0u64), MetricValue::from(0u64)],
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
    struct MockMeasurementAttributes;

    impl MeasurementAttributeSet for MockMeasurementAttributes {
        const CARDINALITY: usize = 1;
        const DESCRIPTORS: &'static [MeasurementAttributeDescriptor] =
            &[MeasurementAttributeDescriptor {
                key: "outcome",
                variants: &["success"],
            }];

        fn bucket_index(&self) -> usize {
            0
        }
    }

    #[derive(Debug)]
    struct MockAttributeSet {
        // Store the attribute values as owned data that we can return references to
        attribute_values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(value: String) -> Self {
            let attribute_values = vec![AttributeValue::String(value)];
            Self { attribute_values }
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

    impl AttributeSetKeySchema for MockAttributeSet {
        const KEY_SCHEMA: &'static [crate::attributes::AttributeKeySchema] =
            &[crate::attributes::AttributeKeySchema::Key("test_key")];
    }

    impl RegistrationMetricSetHandler for MockMetricSet {
        type RegistrationAttributes = MockAttributeSet;
    }

    impl MeasurementMetricSetHandler for MockMetricSet {
        type MeasurementAttributes = MockMeasurementAttributes;
    }

    #[test]
    fn test_telemetry_registry_new() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        assert_eq!(telemetry_registry.metric_set_count(), 0);
        assert_eq!(telemetry_registry.entity_count(), 0);
    }

    #[test]
    fn test_telemetry_registry_clone_shared_state() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let telemetry_registry_clone = telemetry_registry.clone();

        let attrs = MockAttributeSet::new("test_value".to_string());
        let _metric_set: MetricSet<MockMetricSet> = telemetry_registry.register_metric_set(attrs);

        assert_eq!(telemetry_registry.metric_set_count(), 1);
        assert_eq!(telemetry_registry_clone.metric_set_count(), 1);
        assert_eq!(telemetry_registry.entity_count(), 1);
        assert_eq!(telemetry_registry_clone.entity_count(), 1);
    }

    #[test]
    fn test_dirty_unregistration_is_retained_until_one_final_export() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        telemetry_registry
            .registry
            .lock()
            .defer_dirty_metric_unregistration = true;
        let metric_set: MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("final".to_owned()));
        let metrics_key = metric_set.metric_set_key();

        telemetry_registry.accumulate_metric_set_snapshot(
            metrics_key,
            0,
            &[MetricValue::U64(13), MetricValue::U64(21)],
        );
        assert!(telemetry_registry.unregister_metric_set(metrics_key));
        assert_eq!(telemetry_registry.metric_set_count(), 1);
        assert_eq!(telemetry_registry.entity_count(), 1);

        let final_batch = telemetry_registry.drain_metric_export_batch_at(u64::MAX);
        assert_eq!(final_batch.metric_sets.len(), 1);
        assert_eq!(
            final_batch.metric_sets[0].values,
            vec![MetricValue::U64(13), MetricValue::U64(21)]
        );
        assert_eq!(telemetry_registry.metric_set_count(), 0);
        assert_eq!(telemetry_registry.entity_count(), 0);
        assert!(
            telemetry_registry
                .drain_metric_export_batch_at(u64::MAX)
                .is_empty(),
            "a deferred unregister must be exported exactly once"
        );
    }

    #[test]
    fn test_dirty_unregistration_does_not_leak_without_its_export() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let metric_set: MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("final".to_owned()));
        let metrics_key = metric_set.metric_set_key();

        telemetry_registry.accumulate_metric_set_snapshot(
            metrics_key,
            0,
            &[MetricValue::U64(13), MetricValue::U64(21)],
        );
        assert!(telemetry_registry.unregister_metric_set(metrics_key));
        assert_eq!(telemetry_registry.metric_set_count(), 0);
        assert_eq!(telemetry_registry.entity_count(), 0);
    }

    #[test]
    fn test_registration_attribute_metric_set_routes() {
        let registry = TelemetryRegistryHandle::new();
        let registration_attributes = MockAttributeSet::new("registration".to_string());

        let registered: MetricSet<MockMetricSet> = registry
            .register_metric_set_with_registration_attributes(
                MockAttributeSet::new("new-entity".to_string()),
                &registration_attributes,
            );
        let entity = registry.register_entity(MockAttributeSet::new("existing-entity".to_string()));
        let existing: MetricSet<MockMetricSet> = registry
            .register_metric_set_with_registration_attributes_for_entity(
                entity,
                &registration_attributes,
            );
        let combined: MeasurementMetricSet<MockMetricSet> = registry
            .register_metric_set_with_registration_and_measurement_attributes(
                MockAttributeSet::new("combined-entity".to_string()),
                &registration_attributes,
            );
        let combined_existing: MeasurementMetricSet<MockMetricSet> = registry
            .register_metric_set_with_registration_and_measurement_attributes_for_entity(
                entity,
                &registration_attributes,
            );

        assert_ne!(registered.entity_key(), existing.entity_key());
        assert_eq!(existing.entity_key(), entity);
        assert_ne!(combined.entity_key(), combined_existing.entity_key());
        assert_eq!(combined_existing.entity_key(), entity);
    }

    #[test]
    fn test_telemetry_registry_concurrent_access() {
        use std::thread;

        let telemetry_registry = TelemetryRegistryHandle::new();
        let mut handles = Vec::new();

        for i in 0u64..5 {
            let telemetry_registry_clone = telemetry_registry.clone();
            let thread_handle = thread::spawn(move || {
                let attrs = MockAttributeSet::new(format!("value_{i}"));
                let metric_set: MetricSet<MockMetricSet> =
                    telemetry_registry_clone.register_metric_set(attrs);
                let metrics_key = metric_set.key;

                telemetry_registry_clone.accumulate_metric_set_snapshot(
                    metrics_key,
                    0,
                    &[MetricValue::from(i * 10), MetricValue::from(i * 20)],
                );
            });
            handles.push(thread_handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(telemetry_registry.metric_set_count(), 5);
        assert_eq!(telemetry_registry.entity_count(), 5);
    }

    #[test]
    fn test_export_drain_does_not_consume_the_next_accumulation_window() {
        use std::thread;

        let telemetry_registry = TelemetryRegistryHandle::new();
        let metric_set: MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("test_value".to_owned()));
        let metrics_key = metric_set.metric_set_key();

        telemetry_registry.accumulate_metric_set_snapshot(
            metrics_key,
            0,
            &[MetricValue::U64(3), MetricValue::U64(5)],
        );
        let first_batch = telemetry_registry.drain_metric_export_batch_at(10);

        // Keep the owned first batch alive while another thread accumulates the
        // next window. Encoding or downstream backpressure must not hold the
        // registry lock or cause the new snapshot to join the drained batch.
        let next_registry = telemetry_registry.clone();
        thread::spawn(move || {
            next_registry.accumulate_metric_set_snapshot(
                metrics_key,
                0,
                &[MetricValue::U64(7), MetricValue::U64(11)],
            );
        })
        .join()
        .expect("next-window accumulation thread should complete");

        assert_eq!(first_batch.metric_sets.len(), 1);
        assert_eq!(
            first_batch.metric_sets[0].values,
            vec![MetricValue::U64(3), MetricValue::U64(5)]
        );

        let second_batch = telemetry_registry.drain_metric_export_batch_at(20);
        assert_eq!(second_batch.metric_sets.len(), 1);
        assert_eq!(
            second_batch.metric_sets[0].values,
            vec![MetricValue::U64(7), MetricValue::U64(11)]
        );
        assert!(
            telemetry_registry
                .drain_metric_export_batch_at(30)
                .is_empty(),
            "the second window must be exported exactly once"
        );
    }

    #[test]
    fn test_dropped_export_restores_values_and_defers_unregistration() {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let metric_set: MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("retry".to_owned()));
        let metrics_key = metric_set.metric_set_key();

        telemetry_registry.accumulate_metric_set_snapshot(
            metrics_key,
            0,
            &[MetricValue::U64(3), MetricValue::U64(5)],
        );
        let export = telemetry_registry.begin_metric_export_batch_at(u64::MAX);
        assert_eq!(
            export.batch().metric_sets[0].values,
            vec![MetricValue::U64(3), MetricValue::U64(5)]
        );

        // Values collected while delivery is in flight belong to the next
        // window, but must be merged with the failed window on rollback.
        telemetry_registry.accumulate_metric_set_snapshot(
            metrics_key,
            0,
            &[MetricValue::U64(7), MetricValue::U64(11)],
        );
        assert!(telemetry_registry.unregister_metric_set(metrics_key));
        assert_eq!(telemetry_registry.metric_set_count(), 1);
        drop(export);

        let retry = telemetry_registry.begin_metric_export_batch_at(u64::MAX);
        assert_eq!(
            retry.batch().metric_sets[0].values,
            vec![MetricValue::U64(10), MetricValue::U64(16)]
        );
        let _ = retry.commit();
        assert_eq!(telemetry_registry.metric_set_count(), 0);
        assert_eq!(telemetry_registry.entity_count(), 0);
    }
}
