// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deserialization types for the admin telemetry metrics endpoint.

use otap_df_telemetry::attributes::AttributeValue;
use otap_df_telemetry::descriptor::{Instrument, MetricValueType, Temporality};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Shape of /telemetry/metrics (format=json) response.
#[derive(Debug, Serialize, Deserialize)]
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

/// A metric value as rendered by the admin telemetry endpoint.
///
/// This mirrors the JSON the admin endpoint emits rather than reusing the
/// engine's live `MetricValue`, which has no serde representation: a
/// distribution's canonical wire form is the OTLP exponential histogram, and
/// the JSON endpoint renders only its summary.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(variant_size_differences)]
pub enum MetricValue {
    /// Unsigned 64-bit scalar.
    U64(u64),
    /// 64-bit floating point scalar.
    F64(f64),
    /// The summary of a pre-aggregated distribution.
    Distribution(DistributionSummary),
}

impl MetricValue {
    /// Converts the value to `u64`, lossy for floating-point values.
    ///
    /// Distributions report their observation count.
    #[must_use]
    pub fn to_u64_lossy(&self) -> u64 {
        match self {
            MetricValue::U64(v) => *v,
            MetricValue::F64(v) => *v as u64,
            MetricValue::Distribution(d) => d.count,
        }
    }
}

impl From<u64> for MetricValue {
    fn from(value: u64) -> Self {
        MetricValue::U64(value)
    }
}

impl From<f64> for MetricValue {
    fn from(value: f64) -> Self {
        MetricValue::F64(value)
    }
}

/// Summary statistics of a distribution metric, as rendered by the admin
/// telemetry endpoint.
///
/// Re-exported from the admin API so the validation harness observes exactly
/// the fields the endpoint emits, including the `details` object carrying the
/// bucket-derived values (zero count, relative error, `p50`/`p90`/`p99`) that
/// only the exponential-histogram tiers populate.
pub use otap_df_admin_api::telemetry::{DistributionDetails, DistributionSummary};

fn format_metric_value(value: &MetricValue) -> String {
    match value {
        MetricValue::U64(v) => v.to_string(),
        MetricValue::F64(v) => v.to_string(),
        MetricValue::Distribution(d) => {
            let zero_count = d.details.map_or(0, |details| details.zero_count);
            format!(
                "min={} max={} sum={} count={} zero_count={}",
                d.min, d.max, d.sum, d.count, zero_count
            )
        }
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

        assert_eq!(format_metric_value(&MetricValue::from(42u64)), "42");
        assert_eq!(format_metric_value(&MetricValue::from(4.1f64)), "4.1");
    }

    #[test]
    fn display_formats_snapshot_readably() {
        let snapshot = MetricsSnapshot {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            metric_sets: vec![MetricSetSnapshot {
                name: "receiver.traffic_generator".into(),
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
                    value: MetricValue::from(123u64),
                }],
            }],
        };

        let rendered = format!("{snapshot}");
        assert!(rendered.contains("timestamp: 2024-01-01T00:00:00Z"));
        assert!(rendered.contains("metric_set: receiver.traffic_generator"));
        assert!(rendered.contains("logs.produced [{log}]")); // unit shows up in brackets
        assert!(rendered.contains("value=123"));
    }
}

/// A single metric set emitted by the telemetry subsystem.
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
