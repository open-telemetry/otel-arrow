// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics dispatcher that handles internal telemetry metrics.

use std::sync::Arc;

use opentelemetry::{global, metrics::Meter};
use tokio::time::{MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

use crate::{
    attributes::{AttributeSetHandler, AttributeValue},
    descriptor::{Instrument, MetricsField, Temporality},
    error::Error,
    instrument::MmscSnapshot,
    metrics::MetricValue,
    registry::TelemetryRegistryHandle,
};

/// Dispatcher for metrics that periodically flushes metrics to OpenTelemetry SDK.
pub struct MetricsDispatcher {
    metrics_handler: TelemetryRegistryHandle,
    reporting_interval: std::time::Duration,
}

impl MetricsDispatcher {
    /// Create a new metrics dispatcher
    #[must_use]
    pub const fn new(
        metrics_handler: TelemetryRegistryHandle,
        reporting_interval: std::time::Duration,
    ) -> Self {
        Self {
            metrics_handler,
            reporting_interval,
        }
    }

    /// Start the metrics dispatcher.
    pub async fn run_dispatch_loop(
        self: Arc<Self>,
        cancellation_token: CancellationToken,
    ) -> Result<(), Error> {
        let mut ticker = interval(self.reporting_interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    return Ok(())
                }
                _ = ticker.tick() => {
                    self.dispatch_metrics()?
                }
            }
        }
    }

    fn dispatch_metrics(&self) -> Result<(), Error> {
        self.metrics_handler
            .visit_metrics_and_reset(|descriptor, attributes, metrics_iter| {
                let meter = global::meter(descriptor.name);

                // There is no support for metric level attributes currently. Attaching resource level attributes to every metric.
                // TODO: Consider adding metric level attributes and not to add resource level attributes to every metric.
                let otel_attributes = Self::to_opentelemetry_attributes(attributes);

                for (field, value) in metrics_iter {
                    self.add_opentelemetry_metric(field, value, &otel_attributes, &meter);
                }
            });

        Ok(())
    }

    fn to_opentelemetry_attributes(
        attributes: &dyn AttributeSetHandler,
    ) -> Vec<opentelemetry::KeyValue> {
        let mut kvs = Vec::new();
        for (key, value) in attributes.iter_attributes() {
            kvs.push(Self::to_opentelemetry_key_value(key, value));
        }
        kvs
    }

    fn to_opentelemetry_key_value(key: &str, value: &AttributeValue) -> opentelemetry::KeyValue {
        let otel_value = match value {
            AttributeValue::String(s) => opentelemetry::Value::String(s.clone().into()),
            AttributeValue::Int(v) => opentelemetry::Value::I64(*v),
            AttributeValue::Double(v) => opentelemetry::Value::F64(*v),
            AttributeValue::UInt(v) => opentelemetry::Value::I64(*v as i64),
            AttributeValue::Boolean(v) => opentelemetry::Value::Bool(*v),
            AttributeValue::Map(_) => {
                // The OTel metrics SDK `Value` enum has no Map/kvlist variant
                // (only Bool, I64, F64, String, Array). The logs SDK has
                // `AnyValue::Map` but that's a different type. Encode as string
                // for metrics attribute compatibility.
                opentelemetry::Value::String(value.to_string_value().into())
            }
        };
        opentelemetry::KeyValue::new(key.to_string(), otel_value)
    }

    fn add_opentelemetry_metric(
        &self,
        field: &MetricsField,
        value: MetricValue,
        attributes: &[opentelemetry::KeyValue],
        meter: &Meter,
    ) {
        match field.instrument {
            Instrument::Counter => match field.temporality {
                Some(Temporality::Delta) => {
                    self.add_opentelemetry_counter(field, value, attributes, meter)
                }
                Some(Temporality::Cumulative) | None => {
                    debug_assert!(
                        field.temporality.is_some(),
                        "sum-like instrument must have a temporality"
                    );
                    // Cumulative counters are exported as gauges to avoid double-counting.
                    // Note for reviewers: This is a temporary workaround until we figure out a
                    // better way to handle cumulative sums via the Rust Client SDK.
                    self.add_opentelemetry_gauge(field, value, attributes, meter)
                }
            },
            Instrument::UpDownCounter => match field.temporality {
                Some(Temporality::Delta) => {
                    self.add_opentelemetry_up_down_counter(field, value, attributes, meter)
                }
                Some(Temporality::Cumulative) | None => {
                    debug_assert!(
                        field.temporality.is_some(),
                        "sum-like instrument must have a temporality"
                    );
                    // Cumulative up-down counters are exported as gauges to avoid double-counting.
                    // Note for reviewers: This is a temporary workaround until we figure out a
                    // better way to handle cumulative sums via the Rust Client SDK.
                    self.add_opentelemetry_gauge(field, value, attributes, meter)
                }
            },
            Instrument::Gauge => self.add_opentelemetry_gauge(field, value, attributes, meter),
            Instrument::Histogram => {
                // TODO: Handle snapshot values that represent pre-aggregated histograms when we add support for them in the MetricsSnapshot.
                self.add_opentelemetry_histogram(field, value, attributes, meter)
            }
            Instrument::Mmsc => {
                if let MetricValue::Mmsc(s) = value {
                    Self::record_synthetic_histogram(field, &s, attributes, meter);
                } else {
                    unreachable!("MMSC instrument must have Mmsc value");
                }
            }
        }
    }

    fn add_opentelemetry_counter(
        &self,
        field: &MetricsField,
        value: MetricValue,
        attributes: &[opentelemetry::KeyValue],
        meter: &Meter,
    ) {
        match value {
            MetricValue::U64(v) => {
                let counter = meter
                    .u64_counter(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .build();
                counter.add(v, attributes);
            }
            MetricValue::F64(v) => {
                let counter = meter
                    .f64_counter(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .build();
                counter.add(v, attributes);
            }
            MetricValue::Mmsc(_) => unreachable!("Counter instrument cannot have Mmsc value"),
        }
    }

    fn add_opentelemetry_gauge(
        &self,
        field: &MetricsField,
        value: MetricValue,
        attributes: &[opentelemetry::KeyValue],
        meter: &Meter,
    ) {
        match value {
            MetricValue::U64(v) => {
                let gauge = meter
                    .u64_gauge(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .build();
                gauge.record(v, attributes);
            }
            MetricValue::F64(v) => {
                let gauge = meter
                    .f64_gauge(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .build();
                gauge.record(v, attributes);
            }
            MetricValue::Mmsc(_) => unreachable!("Gauge instrument cannot have Mmsc value"),
        }
    }

    fn add_opentelemetry_histogram(
        &self,
        field: &MetricsField,
        value: MetricValue,
        attributes: &[opentelemetry::KeyValue],
        meter: &Meter,
    ) {
        match value {
            MetricValue::U64(v) => {
                let histogram = meter
                    .u64_histogram(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .build();
                histogram.record(v, attributes);
            }
            MetricValue::F64(v) => {
                let histogram = meter
                    .f64_histogram(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .build();
                histogram.record(v, attributes);
            }
            MetricValue::Mmsc(_) => unreachable!("Histogram instrument cannot have Mmsc value"),
        }
    }

    /// Records synthetic histogram observations from pre-aggregated MMSC values.
    ///
    /// This produces `count` calls to `histogram.record()` that yield
    /// the same min/max/sum/count on the exported `HistogramDataPoint`:
    /// - count == 0: no records
    /// - count == 1: record(sum)  (since min == max == sum)
    /// - count == 2: record(min), record(max)
    /// - count >= 3: record(min), record(max), then record a fill value
    ///   `(sum - min - max) / (count - 2)` for the remaining `count - 2` times.
    ///
    /// **Note:** The histogram is built with `.with_boundaries(vec![])` to
    /// disable bucket counting, so only min/max/sum/count are exported.
    fn record_synthetic_histogram(
        field: &MetricsField,
        s: &MmscSnapshot,
        attributes: &[opentelemetry::KeyValue],
        meter: &Meter,
    ) {
        if s.count == 0 {
            return;
        }
        let histogram = meter
            .f64_histogram(field.name)
            .with_description(field.brief)
            .with_unit(field.unit)
            .with_boundaries(vec![])
            .build();
        match s.count {
            1 => {
                histogram.record(s.sum, attributes);
            }
            2 => {
                histogram.record(s.min, attributes);
                histogram.record(s.max, attributes);
            }
            n => {
                histogram.record(s.min, attributes);
                histogram.record(s.max, attributes);
                let fill = (s.sum - s.min - s.max) / (n - 2) as f64;
                for _ in 0..(n - 2) {
                    histogram.record(fill, attributes);
                }
            }
        }
    }

    fn add_opentelemetry_up_down_counter(
        &self,
        field: &MetricsField,
        value: MetricValue,
        attributes: &[opentelemetry::KeyValue],
        meter: &Meter,
    ) {
        match value {
            MetricValue::U64(v) => {
                let up_down_counter = meter
                    .i64_up_down_counter(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .build();
                up_down_counter.add(v as i64, attributes);
            }
            MetricValue::F64(v) => {
                let up_down_counter = meter
                    .f64_up_down_counter(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .build();
                up_down_counter.add(v, attributes);
            }
            MetricValue::Mmsc(_) => unreachable!("UpDownCounter instrument cannot have Mmsc value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;
    use crate::descriptor::MetricValueType;
    use crate::{
        attributes::AttributeIterator,
        descriptor::{AttributeField, AttributeValueType, AttributesDescriptor},
    };

    #[test]
    fn test_to_opentelemetry_key_value() {
        let key = "test_key";
        let string_value = AttributeValue::String("test_value".to_string());
        let int_value = AttributeValue::Int(42);
        let double_value = AttributeValue::Double(PI);
        let uint_value = AttributeValue::UInt(100);
        let bool_value = AttributeValue::Boolean(true);
        let otel_string_kv = MetricsDispatcher::to_opentelemetry_key_value(key, &string_value);
        let otel_int_kv = MetricsDispatcher::to_opentelemetry_key_value(key, &int_value);
        let otel_double_kv = MetricsDispatcher::to_opentelemetry_key_value(key, &double_value);
        let otel_uint_kv = MetricsDispatcher::to_opentelemetry_key_value(key, &uint_value);
        let otel_bool_kv = MetricsDispatcher::to_opentelemetry_key_value(key, &bool_value);
        assert_eq!(otel_string_kv.key.as_str(), key);
        assert_eq!(otel_string_kv.value.as_str(), "test_value");
        assert_eq!(otel_int_kv.key.as_str(), key);
        if let opentelemetry::Value::I64(v) = otel_int_kv.value {
            assert_eq!(v, 42);
        } else {
            panic!("Expected I64 value");
        }
        assert_eq!(otel_double_kv.key.as_str(), key);
        if let opentelemetry::Value::F64(v) = otel_double_kv.value {
            assert!((v - PI).abs() < f64::EPSILON);
        } else {
            panic!("Expected F64 value");
        }
        assert_eq!(otel_uint_kv.key.as_str(), key);
        if let opentelemetry::Value::I64(v) = otel_uint_kv.value {
            assert_eq!(v, 100);
        } else {
            panic!("Expected I64 value");
        }
        assert_eq!(otel_bool_kv.key.as_str(), key);
        if let opentelemetry::Value::Bool(v) = otel_bool_kv.value {
            assert!(v);
        } else {
            panic!("Expected Bool value");
        }
    }

    #[test]
    fn test_dispatch_metrics_no_metrics() {
        let dispatcher = MetricsDispatcher::new(
            TelemetryRegistryHandle::new(),
            std::time::Duration::from_secs(10),
        );
        let result = dispatcher.dispatch_metrics();
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_opentelemetry_attributes() {
        struct MockAttributeSetHandler {
            values: Vec<AttributeValue>,
        }

        impl MockAttributeSetHandler {
            fn new() -> Self {
                Self {
                    values: vec![
                        AttributeValue::String("value1".to_string()),
                        AttributeValue::Int(42),
                    ],
                }
            }
        }

        impl AttributeSetHandler for MockAttributeSetHandler {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &AttributesDescriptor {
                    name: "mock",
                    fields: &[
                        AttributeField {
                            key: "key1",
                            brief: "brief1",
                            r#type: AttributeValueType::String,
                        },
                        AttributeField {
                            key: "key2",
                            brief: "brief2",
                            r#type: AttributeValueType::Int,
                        },
                    ],
                }
            }

            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }

            fn iter_attributes<'a>(&'a self) -> AttributeIterator<'a> {
                AttributeIterator::new(self.descriptor().fields, self.attribute_values())
            }
        }

        let attributes = MockAttributeSetHandler::new();
        let otel_attributes = MetricsDispatcher::to_opentelemetry_attributes(&attributes);
        assert_eq!(otel_attributes.len(), 2);
        assert!(
            otel_attributes
                .iter()
                .any(|kv| kv.key.as_str() == "key1" && kv.value.as_str() == "value1")
        );
        assert!(otel_attributes.iter().any(
            |kv| kv.key.as_str() == "key2" && matches!(kv.value, opentelemetry::Value::I64(42))
        ));
    }

    #[test]
    fn test_add_opentelemetry_counter() {
        let meter = global::meter("test_meter");
        let field = MetricsField {
            name: "test_counter",
            brief: "A test counter",
            unit: "1",
            instrument: Instrument::Counter,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::U64,
        };
        let value = MetricValue::U64(42);
        let attributes = vec![opentelemetry::KeyValue::new("key", "value")];
        let dispatcher = MetricsDispatcher::new(
            TelemetryRegistryHandle::new(),
            std::time::Duration::from_secs(10),
        );
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter);
        // Nothing to assert here. Test the function call and it should not panic.
    }

    #[test]
    fn test_add_opentelemetry_gauge() {
        let meter = global::meter("test_meter");
        let field = MetricsField {
            name: "test_gauge",
            brief: "A test gauge",
            unit: "1",
            instrument: Instrument::Gauge,
            temporality: None,
            value_type: MetricValueType::U64,
        };
        let value = MetricValue::U64(42);
        let attributes = vec![opentelemetry::KeyValue::new("key", "value")];
        let dispatcher = MetricsDispatcher::new(
            TelemetryRegistryHandle::new(),
            std::time::Duration::from_secs(10),
        );
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter);
        // Nothing to assert here. Test the function call and it should not panic.
    }

    #[test]
    fn test_add_opentelemetry_histogram() {
        let meter = global::meter("test_meter");
        let field = MetricsField {
            name: "test_histogram",
            brief: "A test histogram",
            unit: "1",
            instrument: Instrument::Histogram,
            temporality: None,
            value_type: MetricValueType::U64,
        };
        let value = MetricValue::U64(42);
        let attributes = vec![opentelemetry::KeyValue::new("key", "value")];
        let dispatcher = MetricsDispatcher::new(
            TelemetryRegistryHandle::new(),
            std::time::Duration::from_secs(10),
        );
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter);
        // Nothing to assert here. Test the function call and it should not panic.
    }

    #[test]
    fn test_add_opentelemetry_up_down_counter() {
        let meter = global::meter("test_meter");
        let field = MetricsField {
            name: "test_up_down_counter",
            brief: "A test up_down_counter",
            unit: "1",
            instrument: Instrument::UpDownCounter,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::U64,
        };
        let value = MetricValue::U64(42);
        let attributes = vec![opentelemetry::KeyValue::new("key", "value")];
        let dispatcher = MetricsDispatcher::new(
            TelemetryRegistryHandle::new(),
            std::time::Duration::from_secs(10),
        );
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter);
        // Nothing to assert here. Test the function call and it should not panic.
    }

    #[test]
    fn test_add_opentelemetry_mmsc() {
        let meter = global::meter("test_meter_mmsc");
        let field = MetricsField {
            name: "test_mmsc",
            brief: "A test MMSC instrument",
            unit: "ms",
            instrument: Instrument::Mmsc,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::F64,
        };
        let attributes = vec![opentelemetry::KeyValue::new("key", "value")];
        let dispatcher = MetricsDispatcher::new(
            TelemetryRegistryHandle::new(),
            std::time::Duration::from_secs(10),
        );

        // Test with count == 0 (should be a no-op)
        let value = MetricValue::Mmsc(MmscSnapshot {
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
            count: 0,
        });
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter);

        // Test with count == 1
        let value = MetricValue::Mmsc(MmscSnapshot {
            min: 5.0,
            max: 5.0,
            sum: 5.0,
            count: 1,
        });
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter);

        // Test with count == 2
        let value = MetricValue::Mmsc(MmscSnapshot {
            min: 3.0,
            max: 7.0,
            sum: 10.0,
            count: 2,
        });
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter);

        // Test with count >= 3
        let value = MetricValue::Mmsc(MmscSnapshot {
            min: 1.0,
            max: 10.0,
            sum: 25.0,
            count: 5,
        });
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter);
        // Nothing to assert here. Test the function call and it should not panic.
    }

    /// Verifies that MMSC instruments are exported as OTel SDK histograms
    /// with no bucket boundaries (only min/max/sum/count).
    ///
    /// Flow: Mmsc::record → snapshot → accumulate into registry →
    /// visit_metrics_and_reset → add_opentelemetry_metric →
    /// record_synthetic_histogram → OTel SDK histogram export.
    #[test]
    fn test_mmsc_exported_as_histogram_with_no_buckets() {
        use opentelemetry::metrics::MeterProvider as _;
        use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};
        use opentelemetry_sdk::metrics::{InMemoryMetricExporter, SdkMeterProvider};

        use crate::attributes::AttributeIterator;
        use crate::descriptor::{AttributeField, AttributeValueType, AttributesDescriptor};
        use crate::instrument::Mmsc;
        use crate::metrics::{MetricSetHandler, MetricsDescriptor};

        // --- Mock metric set with a single MMSC field --------------------------

        static MMSC_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
            name: "mmsc_e2e_test",
            metrics: &[MetricsField {
                name: "latency",
                unit: "ms",
                brief: "request latency",
                instrument: Instrument::Mmsc,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::F64,
            }],
        };

        #[derive(Debug, Default)]
        struct MmscMetricSet {
            latency: Mmsc,
        }

        impl MetricSetHandler for MmscMetricSet {
            fn descriptor(&self) -> &'static MetricsDescriptor {
                &MMSC_METRICS_DESCRIPTOR
            }
            fn snapshot_values(&self) -> Vec<MetricValue> {
                vec![MetricValue::from(self.latency.get())]
            }
            fn clear_values(&mut self) {
                self.latency.reset();
            }
            fn needs_flush(&self) -> bool {
                !MetricValue::from(self.latency.get()).is_zero()
            }
        }

        static MOCK_ATTRS_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
            name: "test_attrs",
            fields: &[AttributeField {
                key: "service",
                r#type: AttributeValueType::String,
                brief: "service name",
            }],
        };

        #[derive(Debug)]
        struct MockAttrs {
            values: Vec<AttributeValue>,
        }

        impl AttributeSetHandler for MockAttrs {
            fn descriptor(&self) -> &'static AttributesDescriptor {
                &MOCK_ATTRS_DESCRIPTOR
            }
            fn attribute_values(&self) -> &[AttributeValue] {
                &self.values
            }
            fn iter_attributes<'a>(&'a self) -> AttributeIterator<'a> {
                AttributeIterator::new(self.descriptor().fields, &self.values)
            }
        }

        // --- Set up OTel SDK with in-memory exporter ---------------------------

        let exporter = InMemoryMetricExporter::default();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();

        // --- Register, record, accumulate, dispatch ----------------------------

        let registry = TelemetryRegistryHandle::new();
        let mut metric_set = registry.register_metric_set::<MmscMetricSet>(MockAttrs {
            values: vec![AttributeValue::String("my-svc".into())],
        });

        // Simulate recording five latency observations.
        metric_set.latency.record(1.0);
        metric_set.latency.record(4.0);
        metric_set.latency.record(4.0);
        metric_set.latency.record(4.0);
        metric_set.latency.record(10.0);

        // Snapshot and accumulate into the registry.
        let snapshot = metric_set.snapshot();
        registry.accumulate_metric_set_snapshot(snapshot.key, &snapshot.metrics);

        // Dispatch: walk the registry and push to OTel SDK (via a
        // provider-specific meter instead of the global one).
        let dispatcher =
            MetricsDispatcher::new(registry.clone(), std::time::Duration::from_secs(1));
        registry.visit_metrics_and_reset(|descriptor, attributes, metrics_iter| {
            let meter = meter_provider.meter(descriptor.name);
            let otel_attributes = MetricsDispatcher::to_opentelemetry_attributes(attributes);
            for (field, value) in metrics_iter {
                dispatcher.add_opentelemetry_metric(field, value, &otel_attributes, &meter);
            }
        });

        // Flush to trigger export.
        meter_provider.force_flush().expect("force_flush failed");

        // --- Assert on the exported histogram ----------------------------------

        let finished = exporter
            .get_finished_metrics()
            .expect("get_finished_metrics failed");

        let mut found = false;
        for rm in &finished {
            for sm in rm.scope_metrics() {
                for metric in sm.metrics() {
                    if metric.name() == "latency" {
                        if let AggregatedMetrics::F64(MetricData::Histogram(h)) = metric.data() {
                            let dp = h.data_points().next().expect("no data point");

                            let bounds: Vec<f64> = dp.bounds().collect();
                            let bucket_counts: Vec<u64> = dp.bucket_counts().collect();

                            assert!(bounds.is_empty(), "expected no bucket boundaries");
                            assert!(
                                bucket_counts.is_empty(),
                                "no bucket counts when boundaries are empty"
                            );
                            assert_eq!(dp.min(), Some(1.0));
                            assert_eq!(dp.max(), Some(10.0));
                            assert!((dp.sum() - 23.0).abs() < f64::EPSILON);
                            assert_eq!(dp.count(), 5);

                            found = true;
                        }
                    }
                }
            }
        }
        assert!(
            found,
            "histogram metric 'latency' not found in exported data"
        );
    }
}
