// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics dispatcher that handles internal telemetry metrics.

use std::sync::Arc;

use opentelemetry::{global, metrics::Meter};
use tokio::time::{MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

use crate::{
    attributes::{AttributeSetHandler, AttributeValue},
    descriptor::{Instrument, MetricsField},
    error::Error,
    registry::MetricsRegistryHandle,
};

/// Dispatcher for metrics that periodically flushes metrics to OpenTelemetry SDK.
pub struct MetricsDispatcher {
    metrics_handler: MetricsRegistryHandle,
    reporting_interval: std::time::Duration,
}

impl MetricsDispatcher {
    /// Create a new metrics dispatcher
    pub fn new(
        metrics_handler: MetricsRegistryHandle,
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
                    if let Err(e) = self.dispatch_metrics() {
                        return Err(e);
                    }
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
                let otel_attributes = self.to_opentelemetry_attributes(attributes);

                for (field, value) in metrics_iter {
                    self.add_opentelemetry_metric(field, value, &otel_attributes, &meter);
                }
            });

        Ok(())
    }

    fn to_opentelemetry_attributes(
        &self,
        attributes: &dyn AttributeSetHandler,
    ) -> Vec<opentelemetry::KeyValue> {
        let mut kvs = Vec::new();
        for (key, value) in attributes.iter_attributes() {
            kvs.push(self.to_opentelemetry_key_value(key, value));
        }
        kvs
    }

    fn to_opentelemetry_key_value(
        &self,
        key: &str,
        value: &AttributeValue,
    ) -> opentelemetry::KeyValue {
        let otel_value = match value {
            AttributeValue::String(s) => opentelemetry::Value::String(s.clone().into()),
            AttributeValue::Int(v) => opentelemetry::Value::I64(*v),
            AttributeValue::Double(v) => opentelemetry::Value::F64(*v),
            AttributeValue::UInt(v) => opentelemetry::Value::I64(*v as i64),
            AttributeValue::Boolean(v) => opentelemetry::Value::Bool(*v),
        };
        opentelemetry::KeyValue::new(key.to_string(), otel_value)
    }

    fn add_opentelemetry_metric(
        &self,
        field: &MetricsField,
        value: u64,
        attributes: &Vec<opentelemetry::KeyValue>,
        meter: &Meter,
    ) {
        match field.instrument {
            Instrument::Counter => self.add_opentelemetry_counter(field, value, attributes, meter),
            Instrument::Gauge => self.add_opentelemetry_gauge(field, value, attributes, meter),
            Instrument::Histogram => {
                self.add_opentelemetry_histogram(field, value, attributes, meter)
            }
            Instrument::UpDownCounter => {
                self.add_opentelemetry_up_down_counter(field, value, attributes, meter)
            }
        }
    }

    fn add_opentelemetry_counter(
        &self,
        field: &MetricsField,
        value: u64,
        attributes: &Vec<opentelemetry::KeyValue>,
        meter: &Meter,
    ) {
        let counter = meter
            .u64_counter(field.name)
            .with_description(field.brief)
            .with_unit(field.unit)
            .build();
        counter.add(value, &attributes);
    }

    fn add_opentelemetry_gauge(
        &self,
        field: &MetricsField,
        value: u64,
        attributes: &Vec<opentelemetry::KeyValue>,
        meter: &Meter,
    ) {
        let gauge = meter
            .u64_gauge(field.name)
            .with_description(field.brief)
            .with_unit(field.unit)
            .build();
        gauge.record(value, &attributes);
    }

    fn add_opentelemetry_histogram(
        &self,
        field: &MetricsField,
        value: u64,
        attributes: &Vec<opentelemetry::KeyValue>,
        meter: &Meter,
    ) {
        let histogram = meter
            .u64_histogram(field.name)
            .with_description(field.brief)
            .with_unit(field.unit)
            .build();
        histogram.record(value, &attributes);
    }

    fn add_opentelemetry_up_down_counter(
        &self,
        field: &MetricsField,
        value: u64,
        attributes: &Vec<opentelemetry::KeyValue>,
        meter: &Meter,
    ) {
        let up_down_counter = meter
            .i64_up_down_counter(field.name)
            .with_description(field.brief)
            .with_unit(field.unit)
            .build();
        up_down_counter.add(value as i64, &attributes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_opentelemetry_key_value() {
        let dispatcher = MetricsDispatcher::new(
            MetricsRegistryHandle::new(),
            std::time::Duration::from_secs(10),
        );
        let key = "test_key";
        let string_value = AttributeValue::String("test_value".to_string());
        let int_value = AttributeValue::Int(42);
        let double_value = AttributeValue::Double(3.14);
        let uint_value = AttributeValue::UInt(100);
        let bool_value = AttributeValue::Boolean(true);
        let otel_string_kv = dispatcher.to_opentelemetry_key_value(key, &string_value);
        let otel_int_kv = dispatcher.to_opentelemetry_key_value(key, &int_value);
        let otel_double_kv = dispatcher.to_opentelemetry_key_value(key, &double_value);
        let otel_uint_kv = dispatcher.to_opentelemetry_key_value(key, &uint_value);
        let otel_bool_kv = dispatcher.to_opentelemetry_key_value(key, &bool_value);
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
            assert!((v - 3.14).abs() < f64::EPSILON);
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
            assert_eq!(v, true);
        } else {
            panic!("Expected Bool value");
        }
    }

    #[test]
    fn test_dispatch_metrics_no_metrics() {
        let dispatcher = MetricsDispatcher::new(
            MetricsRegistryHandle::new(),
            std::time::Duration::from_secs(10),
        );
        let result = dispatcher.dispatch_metrics();
        assert!(result.is_ok());
    }
}
