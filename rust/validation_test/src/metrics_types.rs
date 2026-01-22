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

#[derive(Debug, Deserialize)]
pub struct MetricSetSnapshot {
    pub name: String,
    pub brief: String,
    pub attributes: HashMap<String, AttributeValue>,
    pub metrics: Vec<MetricDataPoint>,
}

#[derive(Debug, Deserialize)]
pub struct MetricDataPoint {
    #[serde(flatten)]
    pub metadata: MetricMetadata,
    pub value: MetricValue,
}

#[derive(Debug, Deserialize)]
pub struct MetricMetadata {
    pub name: String,
    pub unit: String,
    pub brief: String,
    pub instrument: Instrument,
    #[serde(default)]
    pub temporality: Option<Temporality>,
    pub value_type: MetricValueType,
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

#[derive(Debug, Deserialize)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    UInt(u64),
    Double(f64),
    Boolean(bool),
}
