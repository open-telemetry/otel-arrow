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
                self.add_opentelemetry_histogram(field, value, attributes, meter)
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
}
