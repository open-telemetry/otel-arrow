// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deserialization types for the admin telemetry metrics endpoint.

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
                writeln!(f, "    {k}: {v}")?;
            }
            writeln!(f, "  metrics:")?;
            for m in &set.metrics {
                writeln!(
                    f,
                    "    {} [{}] instrument={:?} temporality={:?} value_type={:?} value={}",
                    m.name, m.unit, m.instrument, m.temporality, m.value_type, m.value
                )?;
            }
        }
        Ok(())
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    Counter,
    UpDownCounter,
    Gauge,
    Histogram,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Temporality {
    Delta,
    Cumulative,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricValueType {
    U64,
    F64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    U64(u64),
    F64(f64),
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::U64(v) => write!(f, "{v}"),
            MetricValue::F64(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String { String: String },
    Int { Int: i64 },
    UInt { UInt: u64 },
    Double { Double: f64 },
    Boolean { Boolean: bool },
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::String { String: v } => write!(f, "{v}"),
            AttributeValue::Int { Int: v } => write!(f, "{v}"),
            AttributeValue::UInt { UInt: v } => write!(f, "{v}"),
            AttributeValue::Double { Double: v } => write!(f, "{v}"),
            AttributeValue::Boolean { Boolean: v } => write!(f, "{v}"),
        }
    }
}
