// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deserialization types for the admin telemetry metrics endpoint.

use otap_df_telemetry::attributes::AttributeValue;
#[allow(unused_imports)]
use otap_df_telemetry::descriptor::{
    Instrument, MetricValueType, MetricsDescriptor, MetricsField, Temporality,
};
pub use otap_df_telemetry::metrics::MetricValue;
use serde::Deserialize;
use std::collections::HashMap;

/// Shape of /telemetry/metrics (format=json) response.
#[derive(Debug, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: String,
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
    }
}

fn format_metric_value(value: &MetricValue) -> String {
    match value {
        MetricValue::U64(v) => v.to_string(),
        MetricValue::F64(v) => v.to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct MetricSetSnapshot {
    pub name: String,
    pub brief: String,
    pub attributes: HashMap<String, AttributeValue>,
    pub metrics: Vec<MetricDataPoint>,
}

#[derive(Debug, Deserialize)]
pub struct MetricDataPoint {
    pub name: String,
    pub unit: String,
    pub brief: String,
    pub instrument: Instrument,
    #[serde(default)]
    pub temporality: Option<Temporality>,
    pub value_type: MetricValueType,
    pub value: MetricValue,
}
