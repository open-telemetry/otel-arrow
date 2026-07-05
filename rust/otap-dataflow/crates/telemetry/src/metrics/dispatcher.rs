// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics dispatcher that handles internal telemetry metrics.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use opentelemetry::metrics::ObservableCounter;
use opentelemetry::{InstrumentationScope, global, metrics::Meter, metrics::MeterProvider};
use parking_lot::Mutex;
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
    /// Observable-counter series backing cumulative sums, keyed by scope
    /// series key, then by metric name. Each series holds the shared value
    /// read by an SDK callback registered once per series; the SDK reader
    /// then performs cumulative/delta temporality conversion itself.
    ///
    /// The series currently live for the process lifetime. Naive eviction is
    /// unsafe because dropping these instrument handles does not unregister
    /// their SDK callbacks; see #3413 for lifecycle-aware pruning.
    cumulative_series: Mutex<HashMap<String, HashMap<&'static str, CumulativeCounterSeries>>>,
}

/// Backing state for one exported cumulative counter series.
///
/// The `value` slot is shared with an SDK observable-counter callback that
/// reports the latest total at every SDK collection. The instrument handle is
/// retained so the registration lives as long as the series.
enum CumulativeCounterSeries {
    /// Series for a `u64` cumulative counter.
    U64 {
        value: Arc<AtomicU64>,
        _instrument: ObservableCounter<u64>,
    },
    /// Series for an `f64` cumulative counter; the atomic stores the bit
    /// pattern of the current value.
    F64 {
        value: Arc<AtomicU64>,
        _instrument: ObservableCounter<f64>,
    },
}

impl CumulativeCounterSeries {
    fn build(field: &MetricsField, initial: MetricValue, meter: &Meter) -> Self {
        match initial {
            MetricValue::U64(v) => {
                let value = Arc::new(AtomicU64::new(v));
                let observed = Arc::clone(&value);
                let instrument = meter
                    .u64_observable_counter(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .with_callback(move |observer| {
                        observer.observe(observed.load(Ordering::Relaxed), &[]);
                    })
                    .build();
                Self::U64 {
                    value,
                    _instrument: instrument,
                }
            }
            MetricValue::F64(v) => {
                let value = Arc::new(AtomicU64::new(v.to_bits()));
                let observed = Arc::clone(&value);
                let instrument = meter
                    .f64_observable_counter(field.name)
                    .with_description(field.brief)
                    .with_unit(field.unit)
                    .with_callback(move |observer| {
                        observer.observe(f64::from_bits(observed.load(Ordering::Relaxed)), &[]);
                    })
                    .build();
                Self::F64 {
                    value,
                    _instrument: instrument,
                }
            }
            MetricValue::Mmsc(_) => unreachable!("Counter instrument cannot have Mmsc value"),
        }
    }

    fn store(&self, value: MetricValue) {
        match (self, value) {
            (Self::U64 { value: slot, .. }, MetricValue::U64(v)) => {
                slot.store(v, Ordering::Relaxed);
            }
            (Self::F64 { value: slot, .. }, MetricValue::F64(v)) => {
                slot.store(v.to_bits(), Ordering::Relaxed);
            }
            _ => debug_assert!(
                false,
                "metric value type must not change between dispatches"
            ),
        }
    }
}

impl MetricsDispatcher {
    /// Create a new metrics dispatcher
    #[must_use]
    pub fn new(
        metrics_handler: TelemetryRegistryHandle,
        reporting_interval: std::time::Duration,
    ) -> Self {
        Self {
            metrics_handler,
            reporting_interval,
            cumulative_series: Mutex::new(HashMap::new()),
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
        self.dispatch_metrics_to(&*global::meter_provider())
    }

    /// Dispatches the accumulated metrics through `meter_provider`.
    ///
    /// Production callers pass the global meter provider; tests can inject a
    /// dedicated [`MeterProvider`] (e.g. backed by an in-memory exporter) to
    /// observe the emitted instrumentation scopes and data points.
    fn dispatch_metrics_to(&self, meter_provider: &dyn MeterProvider) -> Result<(), Error> {
        self.metrics_handler
            .visit_metrics_and_reset(|descriptor, attributes, metrics_iter| {
                let scope_attributes = Self::to_opentelemetry_attributes(attributes);
                let scope_key = Self::scope_series_key(descriptor.name, &scope_attributes);
                let meter = if scope_attributes.is_empty() {
                    meter_provider.meter(descriptor.name)
                } else {
                    meter_provider.meter_with_scope(
                        InstrumentationScope::builder(descriptor.name)
                            .with_attributes(scope_attributes)
                            .build(),
                    )
                };

                // Entity attributes live on the instrumentation scope so OTel Views can
                // target them via scope_attributes selectors; data points carry none.
                for (field, value) in metrics_iter {
                    self.add_opentelemetry_metric(field, value, &[], &meter, &scope_key);
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

    /// Builds a stable identifier for the series cache from the scope name and
    /// its entity attributes (which are fixed for a registered metric set).
    fn scope_series_key(scope_name: &str, attributes: &[opentelemetry::KeyValue]) -> String {
        use std::fmt::Write;
        let mut key = String::from(scope_name);
        for kv in attributes {
            let _ = write!(key, "\u{1f}{}\u{1f}{}", kv.key, kv.value);
        }
        key
    }

    fn add_opentelemetry_metric(
        &self,
        field: &MetricsField,
        value: MetricValue,
        attributes: &[opentelemetry::KeyValue],
        meter: &Meter,
        scope_key: &str,
    ) {
        match field.instrument {
            Instrument::Counter => match field.temporality {
                Some(Temporality::Delta) => {
                    self.add_opentelemetry_counter(field, value, attributes, meter)
                }
                Some(Temporality::Cumulative) => {
                    self.observe_cumulative_counter(field, value, meter, scope_key)
                }
                None => {
                    debug_assert!(false, "sum-like instrument must have a temporality");
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
                    // Cumulative up-down counters are still exported as gauges to avoid
                    // double-counting. They cannot reuse the observable-counter path
                    // below: the registry zeroes values after each visit, and unlike a
                    // monotonic total, a legitimate up-down value of zero cannot be
                    // distinguished from "no fresh observation this interval".
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

    /// Publishes the latest total of a cumulative counter through an SDK
    /// observable counter, so the configured reader performs the
    /// cumulative/delta temporality conversion.
    ///
    /// The observable instrument and its callback are created once per series
    /// and cached; later dispatches only update the shared value slot.
    ///
    /// Zero updates to existing series are skipped: the registry zeroes
    /// accumulated values after every visit, so a zero for an already-created
    /// monotonic series means "no fresh observation this interval". Initial
    /// zero values still create a series so readers can observe a legitimate
    /// zero starting total.
    fn observe_cumulative_counter(
        &self,
        field: &MetricsField,
        value: MetricValue,
        meter: &Meter,
        scope_key: &str,
    ) {
        let mut scopes = self.cumulative_series.lock();
        if let Some(series) = scopes.get(scope_key).and_then(|s| s.get(field.name)) {
            if !value.is_zero() {
                series.store(value);
            }
        } else {
            let series = CumulativeCounterSeries::build(field, value, meter);
            let _ = scopes
                .entry(scope_key.to_owned())
                .or_default()
                .insert(field.name, series);
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
    use crate::instrument::Mmsc;
    use crate::metrics::{MetricSetHandler, MetricsDescriptor};
    use crate::{
        attributes::AttributeIterator,
        descriptor::{AttributeField, AttributeValueType, AttributesDescriptor},
    };
    use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData, ResourceMetrics};
    use opentelemetry_sdk::metrics::{InMemoryMetricExporter, SdkMeterProvider};

    // --- End-to-end scope-attribute dispatch tests -----------------------------
    //
    // These drive the real `dispatch_metrics_to` code path against a dedicated
    // in-memory meter provider (no global state, parallel-safe) and assert the
    // key behavior of the resource/scope reorganization: entity attributes are
    // attached to the *instrumentation scope*, never duplicated onto data points.

    static SCOPE_ATTR_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "scope_attr_dispatch_test",
        metrics: &[MetricsField {
            name: "requests",
            unit: "{request}",
            brief: "request count",
            instrument: Instrument::Counter,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::U64,
        }],
    };

    #[derive(Debug, Default)]
    struct CounterMetricSet {
        requests: crate::instrument::Counter<u64>,
    }

    impl MetricSetHandler for CounterMetricSet {
        fn descriptor(&self) -> &'static MetricsDescriptor {
            &SCOPE_ATTR_METRICS_DESCRIPTOR
        }
        fn snapshot_values(&self) -> Vec<MetricValue> {
            vec![MetricValue::from(self.requests.get())]
        }
        fn clear_values(&mut self) {
            self.requests.reset();
        }
        fn needs_flush(&self) -> bool {
            !MetricValue::from(self.requests.get()).is_zero()
        }
    }

    // Carries a single `service` attribute -> exercises the scope-attribute branch.
    static WITH_ATTRS_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "scope_attrs_present",
        fields: &[AttributeField {
            key: "service",
            r#type: AttributeValueType::String,
            brief: "service name",
        }],
    };

    // Carries no attributes -> exercises the bare-scope branch.
    static NO_ATTRS_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "scope_attrs_absent",
        fields: &[],
    };

    #[derive(Debug)]
    struct ScopeAttrs {
        descriptor: &'static AttributesDescriptor,
        values: Vec<AttributeValue>,
    }

    impl AttributeSetHandler for ScopeAttrs {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            self.descriptor
        }
        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
        fn iter_attributes<'a>(&'a self) -> AttributeIterator<'a> {
            AttributeIterator::new(self.descriptor.fields, &self.values)
        }
    }

    /// Dispatches the dispatcher's accumulated metrics through a dedicated
    /// in-memory meter provider and returns the exported resource metrics.
    ///
    /// Shared harness for the end-to-end dispatch tests: it drives the real
    /// `dispatch_metrics_to` code path with no global state (parallel-safe).
    fn dispatch_to_in_memory(dispatcher: &MetricsDispatcher) -> Vec<ResourceMetrics> {
        let exporter = InMemoryMetricExporter::default();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();

        dispatcher
            .dispatch_metrics_to(&meter_provider)
            .expect("dispatch_metrics_to failed");

        meter_provider.force_flush().expect("force_flush failed");

        exporter
            .get_finished_metrics()
            .expect("get_finished_metrics failed")
    }

    /// Registers a counter metric set with `attrs`, records `count` increments,
    /// dispatches through the shared in-memory harness, and returns
    /// `(scope attributes, data-point attributes, summed value)` for the
    /// exported `requests` counter. Asserts exactly one data point was emitted.
    fn dispatch_and_collect(
        attrs: ScopeAttrs,
        count: u64,
    ) -> (
        Vec<opentelemetry::KeyValue>,
        Vec<opentelemetry::KeyValue>,
        u64,
    ) {
        let registry = TelemetryRegistryHandle::new();
        let mut metric_set = registry.register_metric_set::<CounterMetricSet>(attrs);
        metric_set.requests.add(count);

        let snapshot = metric_set.snapshot();
        registry.accumulate_metric_set_snapshot(snapshot.key, &snapshot.metrics);

        let dispatcher =
            MetricsDispatcher::new(registry.clone(), std::time::Duration::from_secs(1));
        let finished = dispatch_to_in_memory(&dispatcher);

        let mut result = None;
        for rm in &finished {
            for sm in rm.scope_metrics() {
                for metric in sm.metrics() {
                    if metric.name() == "requests" {
                        let scope_attrs = sm.scope().attributes().cloned().collect::<Vec<_>>();
                        let AggregatedMetrics::U64(MetricData::Sum(sum)) = metric.data() else {
                            panic!("expected a u64 Sum for the requests counter");
                        };
                        let data_points: Vec<_> = sum.data_points().collect();
                        assert_eq!(
                            data_points.len(),
                            1,
                            "expected exactly one data point, found {}",
                            data_points.len()
                        );
                        let dp = data_points[0];
                        let dp_attrs = dp.attributes().cloned().collect::<Vec<_>>();
                        assert!(result.is_none(), "requests metric exported more than once");
                        result = Some((scope_attrs, dp_attrs, dp.value()));
                    }
                }
            }
        }

        result.expect("requests metric not exported")
    }

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
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter, "test_scope");
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
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter, "test_scope");
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
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter, "test_scope");
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
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter, "test_scope");
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
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter, "test_scope");

        // Test with count == 1
        let value = MetricValue::Mmsc(MmscSnapshot {
            min: 5.0,
            max: 5.0,
            sum: 5.0,
            count: 1,
        });
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter, "test_scope");

        // Test with count == 2
        let value = MetricValue::Mmsc(MmscSnapshot {
            min: 3.0,
            max: 7.0,
            sum: 10.0,
            count: 2,
        });
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter, "test_scope");

        // Test with count >= 3
        let value = MetricValue::Mmsc(MmscSnapshot {
            min: 1.0,
            max: 10.0,
            sum: 25.0,
            count: 5,
        });
        dispatcher.add_opentelemetry_metric(&field, value, &attributes, &meter, "test_scope");
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

        // --- Register, record, accumulate, dispatch ----------------------------

        let registry = TelemetryRegistryHandle::new();
        let mut metric_set = registry.register_metric_set::<MmscMetricSet>(ScopeAttrs {
            descriptor: &WITH_ATTRS_DESCRIPTOR,
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

        // Dispatch through the real code path via the shared in-memory harness.
        let dispatcher =
            MetricsDispatcher::new(registry.clone(), std::time::Duration::from_secs(1));
        let finished = dispatch_to_in_memory(&dispatcher);

        // --- Assert on the exported histogram ----------------------------------

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

    // --- Cumulative counter dispatch tests ---------------------------------
    //
    // These cover the observable-counter path for cumulative sums: the
    // dispatcher publishes the latest total through an SDK async instrument,
    // and the reader performs the cumulative/delta temporality conversion.

    static CUMULATIVE_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "cumulative_dispatch_test",
        metrics: &[
            MetricsField {
                name: "events_total",
                unit: "{event}",
                brief: "lifetime event count",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Cumulative),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "ticks",
                unit: "{tick}",
                brief: "per-interval tick count",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
        ],
    };

    #[derive(Debug, Default)]
    struct CumulativeMetricSet {
        events_total: crate::instrument::ObserveCounter<u64>,
        ticks: crate::instrument::Counter<u64>,
    }

    impl MetricSetHandler for CumulativeMetricSet {
        fn descriptor(&self) -> &'static MetricsDescriptor {
            &CUMULATIVE_METRICS_DESCRIPTOR
        }
        fn snapshot_values(&self) -> Vec<MetricValue> {
            vec![
                MetricValue::from(self.events_total.get()),
                MetricValue::from(self.ticks.get()),
            ]
        }
        fn clear_values(&mut self) {
            // Mirrors the derive macro: delta counters reset, observe
            // counters keep their last observed total.
            self.ticks.reset();
        }
        fn needs_flush(&self) -> bool {
            true
        }
    }

    /// Returns `(value, is_monotonic)` of the single data point exported for
    /// the u64 sum named `name` in one collected batch, or `None` if the
    /// metric is absent from the batch.
    fn find_u64_sum(rm: &ResourceMetrics, name: &str) -> Option<(u64, bool)> {
        let mut found = None;
        for sm in rm.scope_metrics() {
            for metric in sm.metrics() {
                if metric.name() == name {
                    assert!(
                        found.is_none(),
                        "expected exactly one metric named '{name}'"
                    );
                    let AggregatedMetrics::U64(MetricData::Sum(sum)) = metric.data() else {
                        panic!("expected metric '{name}' to be exported as a u64 Sum");
                    };
                    let data_points: Vec<_> = sum.data_points().collect();
                    assert_eq!(data_points.len(), 1, "expected one data point for '{name}'");
                    found = Some((data_points[0].value(), sum.is_monotonic()));
                }
            }
        }
        found
    }

    /// Snapshots `metric_set` into the registry, dispatches through
    /// `meter_provider`, and returns the newly collected batch.
    fn accumulate_and_collect(
        registry: &TelemetryRegistryHandle,
        metric_set: &crate::metrics::MetricSet<CumulativeMetricSet>,
        dispatcher: &MetricsDispatcher,
        meter_provider: &SdkMeterProvider,
        exporter: &InMemoryMetricExporter,
    ) -> ResourceMetrics {
        let snapshot = metric_set.snapshot();
        registry.accumulate_metric_set_snapshot(snapshot.key, &snapshot.metrics);
        dispatcher
            .dispatch_metrics_to(meter_provider)
            .expect("dispatch_metrics_to failed");
        meter_provider.force_flush().expect("force_flush failed");
        exporter
            .get_finished_metrics()
            .expect("get_finished_metrics failed")
            .pop()
            .expect("no batch collected")
    }

    #[test]
    fn test_cumulative_counter_exported_as_monotonic_sum() {
        let registry = TelemetryRegistryHandle::new();
        let mut metric_set = registry.register_metric_set::<CumulativeMetricSet>(ScopeAttrs {
            descriptor: &NO_ATTRS_DESCRIPTOR,
            values: vec![],
        });
        let dispatcher =
            MetricsDispatcher::new(registry.clone(), std::time::Duration::from_secs(1));
        let exporter = InMemoryMetricExporter::default();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();

        metric_set.events_total.observe(10);
        metric_set.ticks.add(1);
        let batch = accumulate_and_collect(
            &registry,
            &metric_set,
            &dispatcher,
            &meter_provider,
            &exporter,
        );
        let (value, monotonic) = find_u64_sum(&batch, "events_total").expect("metric not exported");
        assert!(monotonic, "cumulative counter must export as monotonic sum");
        assert_eq!(value, 10);

        // A later, larger total replaces the previous one; a cumulative
        // reader must report 25, not a double-counted 35.
        metric_set.events_total.observe(25);
        metric_set.ticks.add(1);
        let batch = accumulate_and_collect(
            &registry,
            &metric_set,
            &dispatcher,
            &meter_provider,
            &exporter,
        );
        let (value, _) = find_u64_sum(&batch, "events_total").expect("metric not exported");
        assert_eq!(value, 25, "cumulative total must be replaced, not summed");
    }

    #[test]
    fn test_cumulative_counter_honors_delta_reader_temporality() {
        use opentelemetry_sdk::metrics::InMemoryMetricExporterBuilder;

        let registry = TelemetryRegistryHandle::new();
        let mut metric_set = registry.register_metric_set::<CumulativeMetricSet>(ScopeAttrs {
            descriptor: &NO_ATTRS_DESCRIPTOR,
            values: vec![],
        });
        let dispatcher =
            MetricsDispatcher::new(registry.clone(), std::time::Duration::from_secs(1));
        let exporter = InMemoryMetricExporterBuilder::new()
            .with_temporality(opentelemetry_sdk::metrics::Temporality::Delta)
            .build();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();

        metric_set.events_total.observe(10);
        let batch = accumulate_and_collect(
            &registry,
            &metric_set,
            &dispatcher,
            &meter_provider,
            &exporter,
        );
        let (value, _) = find_u64_sum(&batch, "events_total").expect("metric not exported");
        assert_eq!(
            value, 10,
            "first delta observation reports the initial total"
        );

        metric_set.events_total.observe(25);
        let batch = accumulate_and_collect(
            &registry,
            &metric_set,
            &dispatcher,
            &meter_provider,
            &exporter,
        );
        let (value, _) = find_u64_sum(&batch, "events_total").expect("metric not exported");
        assert_eq!(
            value, 15,
            "delta reader must receive the per-interval difference"
        );
    }

    #[test]
    fn test_cumulative_counter_creates_series_on_initial_zero() {
        let registry = TelemetryRegistryHandle::new();
        let mut metric_set = registry.register_metric_set::<CumulativeMetricSet>(ScopeAttrs {
            descriptor: &NO_ATTRS_DESCRIPTOR,
            values: vec![],
        });
        let dispatcher =
            MetricsDispatcher::new(registry.clone(), std::time::Duration::from_secs(1));
        let exporter = InMemoryMetricExporter::default();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();

        // The nonzero delta sibling forces a registry visit; the cumulative
        // counter's first observed total is legitimately zero and should still
        // create an observable series.
        metric_set.events_total.observe(0);
        metric_set.ticks.add(1);
        let batch = accumulate_and_collect(
            &registry,
            &metric_set,
            &dispatcher,
            &meter_provider,
            &exporter,
        );
        let (value, monotonic) = find_u64_sum(&batch, "events_total").expect("metric not exported");
        assert!(monotonic, "cumulative counter must export as monotonic sum");
        assert_eq!(value, 0);

        // A later nonzero value must update the existing series. The helper
        // asserts that exactly one metric and data point are exported.
        metric_set.events_total.observe(7);
        metric_set.ticks.add(1);
        let batch = accumulate_and_collect(
            &registry,
            &metric_set,
            &dispatcher,
            &meter_provider,
            &exporter,
        );
        let (value, _) = find_u64_sum(&batch, "events_total").expect("metric not exported");
        assert_eq!(value, 7);
    }

    #[test]
    fn test_cumulative_counter_ignores_zeroed_registry_values() {
        let registry = TelemetryRegistryHandle::new();
        let mut metric_set = registry.register_metric_set::<CumulativeMetricSet>(ScopeAttrs {
            descriptor: &NO_ATTRS_DESCRIPTOR,
            values: vec![],
        });
        let dispatcher =
            MetricsDispatcher::new(registry.clone(), std::time::Duration::from_secs(1));
        let exporter = InMemoryMetricExporter::default();
        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();

        metric_set.events_total.observe(10);
        metric_set.ticks.add(1);
        let batch = accumulate_and_collect(
            &registry,
            &metric_set,
            &dispatcher,
            &meter_provider,
            &exporter,
        );
        let (value, _) = find_u64_sum(&batch, "events_total").expect("metric not exported");
        assert_eq!(value, 10);

        // Simulate an interval where only the delta counter was reported:
        // the registry visit then yields a zero for the cumulative field,
        // which must not clobber the previously observed total.
        let snapshot_key = metric_set.snapshot().key;
        registry.accumulate_metric_set_snapshot(
            snapshot_key,
            &[MetricValue::U64(0), MetricValue::U64(1)],
        );
        dispatcher
            .dispatch_metrics_to(&meter_provider)
            .expect("dispatch_metrics_to failed");
        meter_provider.force_flush().expect("force_flush failed");
        let batch = exporter
            .get_finished_metrics()
            .expect("get_finished_metrics failed")
            .pop()
            .expect("no batch collected");
        let (value, _) = find_u64_sum(&batch, "events_total").expect("metric not exported");
        assert_eq!(value, 10, "zeroed registry value must not reset the total");
    }

    #[test]
    fn test_dispatch_metrics_puts_entity_attributes_on_scope() {
        let attrs = ScopeAttrs {
            descriptor: &WITH_ATTRS_DESCRIPTOR,
            values: vec![AttributeValue::String("my-svc".into())],
        };

        let (scope_attrs, dp_attrs, value) = dispatch_and_collect(attrs, 3);

        // The recorded value must survive the dispatch unchanged.
        assert_eq!(value, 3, "exported counter value mismatch");

        // Entity attributes must live on the instrumentation scope...
        assert_eq!(scope_attrs.len(), 1, "expected exactly one scope attribute");
        assert_eq!(scope_attrs[0].key.as_str(), "service");
        assert_eq!(scope_attrs[0].value.as_str(), "my-svc");

        // ...and must NOT be duplicated onto the data points.
        assert!(
            dp_attrs.is_empty(),
            "data points must not carry entity attributes, found: {dp_attrs:?}"
        );
    }

    #[test]
    fn test_dispatch_metrics_without_attributes_uses_bare_scope() {
        let attrs = ScopeAttrs {
            descriptor: &NO_ATTRS_DESCRIPTOR,
            values: vec![],
        };

        let (scope_attrs, dp_attrs, value) = dispatch_and_collect(attrs, 5);

        assert_eq!(value, 5, "exported counter value mismatch");
        assert!(
            scope_attrs.is_empty(),
            "scope must have no attributes when the entity has none, found: {scope_attrs:?}"
        );
        assert!(dp_attrs.is_empty(), "data points must carry no attributes");
    }
}
