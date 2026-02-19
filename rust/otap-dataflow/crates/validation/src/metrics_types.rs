// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deserialization types for the admin telemetry metrics endpoint.

use otap_df_telemetry::attributes::AttributeValue;
use otap_df_telemetry::descriptor::{Instrument, MetricValueType, Temporality};
pub use otap_df_telemetry::metrics::MetricValue;
use serde::Deserialize;
use std::collections::HashMap;

/// Shape of /telemetry/metrics (format=json) response.
#[derive(Debug, Deserialize)]
pub struct MetricsSnapshot {
    /// Time the snapshot was captured, as an RFC 3339 string.
    pub timestamp: String,
    /// Collection of metric sets emitted by the running pipelines.
    pub metric_sets: Vec<MetricSetSnapshot>,
}

/// use to debug
impl std::fmt::Display for MetricsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "timestamp: {}", self.timestamp)?;
        for set in &self.metric_sets {
            writeln!(f, "metric_set: {}", set.name)?;
            writeln!(f, "  brief: {}", set.brief)?;
            writeln!(f, "  attributes:")?;
            for (k, v) in &set.attributes {
                writeln!(f, "    {k}: {}", format_attribute_value(v))?;
            }
            writeln!(f, "  metrics:")?;
            for m in &set.metrics {
                writeln!(
                    f,
                    "    {} [{}] instrument={:?} temporality={:?} value_type={:?} value={}",
                    m.name,
                    m.unit,
                    m.instrument,
                    m.temporality,
                    m.value_type,
                    format_metric_value(&m.value)
                )?;
            }
        }
        Ok(())
    }
}

fn format_attribute_value(value: &AttributeValue) -> String {
    match value {
        AttributeValue::String(v) => v.clone(),
        AttributeValue::Int(v) => v.to_string(),
        AttributeValue::UInt(v) => v.to_string(),
        AttributeValue::Double(v) => v.to_string(),
        AttributeValue::Boolean(v) => v.to_string(),
        AttributeValue::Map(_) => value.to_string_value(),
    }
}

fn format_metric_value(value: &MetricValue) -> String {
    match value {
        MetricValue::U64(v) => v.to_string(),
        MetricValue::F64(v) => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_telemetry::descriptor::{Instrument, MetricValueType, Temporality};

    #[test]
    fn format_attribute_and_metric_values_cover_variants() {
        assert_eq!(
            format_attribute_value(&AttributeValue::String("abc".into())),
            "abc"
        );
        assert_eq!(format_attribute_value(&AttributeValue::Int(-5)), "-5");
        assert_eq!(format_attribute_value(&AttributeValue::UInt(5)), "5");
        assert_eq!(format_attribute_value(&AttributeValue::Double(1.5)), "1.5");
        assert_eq!(
            format_attribute_value(&AttributeValue::Boolean(true)),
            "true"
        );

        assert_eq!(format_metric_value(&MetricValue::U64(42)), "42");
        assert_eq!(format_metric_value(&MetricValue::F64(4.1)), "4.1");
    }

    #[test]
    fn display_formats_snapshot_readably() {
        let snapshot = MetricsSnapshot {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            metric_sets: vec![MetricSetSnapshot {
                name: "fake_data_generator.receiver.metrics".into(),
                brief: "loadgen metrics".into(),
                attributes: HashMap::from([(
                    "role".into(),
                    AttributeValue::String("generator".into()),
                )]),
                metrics: vec![MetricDataPoint {
                    name: "logs.produced".into(),
                    unit: "{log}".into(),
                    brief: "produced logs".into(),
                    instrument: Instrument::Counter,
                    temporality: Some(Temporality::Cumulative),
                    value_type: MetricValueType::U64,
                    value: MetricValue::U64(123),
                }],
            }],
        };

        let rendered = format!("{snapshot}");
        assert!(rendered.contains("timestamp: 2024-01-01T00:00:00Z"));
        assert!(rendered.contains("metric_set: fake_data_generator.receiver.metrics"));
        assert!(rendered.contains("logs.produced [{log}]")); // unit shows up in brackets
        assert!(rendered.contains("value=123"));
    }
}

/// A single metric set emitted by the telemetry subsystem.
#[derive(Debug, Deserialize)]
pub struct MetricSetSnapshot {
    /// Unique identifier of the metric set (usually a component name).
    pub name: String,
    /// Short human-readable description of the set.
    pub brief: String,
    /// Attributes attached to this metric set.
    pub attributes: HashMap<String, AttributeValue>,
    /// Individual metric data points within the set.
    pub metrics: Vec<MetricDataPoint>,
}

/// A single recorded metric, including its metadata and value.
#[derive(Debug, Deserialize)]
pub struct MetricDataPoint {
    /// Metric name (e.g. `logs.produced`).
    pub name: String,
    /// Unit of measurement associated with the metric.
    pub unit: String,
    /// Short description of what the metric represents.
    pub brief: String,
    /// Instrument type used to record the metric.
    pub instrument: Instrument,
    #[serde(default)]
    /// Temporality of the metric if provided by the source.
    pub temporality: Option<Temporality>,
    /// Metric value encoding (e.g. integer or float).
    pub value_type: MetricValueType,
    /// The recorded value for this data point.
    pub value: MetricValue,
}
