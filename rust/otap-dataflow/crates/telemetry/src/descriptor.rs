// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric and Attribute descriptor types for metrics reflection.

use serde::Serialize;

/// The type of instrument used to record the metric. Must be one of the following variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    /// A sum-like instrument reporting deltas over an interval (to be accumulated).
    DeltaCounter,
    /// A sum-like instrument reporting a current (observed) value (to be replaced).
    ObserveCounter,
    /// A sum-like instrument reporting signed deltas over an interval (to be accumulated).
    DeltaUpDownCounter,
    /// A sum-like instrument reporting a current (observed) signed value (to be replaced).
    ObserveUpDownCounter,
    /// A value that can arbitrarily go up and down, used for temperature or current memory usage
    Gauge,
    /// Distribution of recorded values, used for latencies or request sizes
    Histogram,
}

/// Numeric representation used by a metric field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricValueType {
    /// Unsigned 64-bit integer.
    U64,
    /// 64-bit floating point.
    F64,
}

/// Metadata describing a single field inside a metrics struct.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct MetricsField {
    /// Canonical metric name (e.g., "bytes.rx"). Uniquely identifies the metric.
    pub name: &'static str,
    /// The unit in which the metric is measured matching
    /// [Unified Code for Units of Measure](https://unitsofmeasure.org/ucum.html).
    pub unit: &'static str,
    /// Short human readable description extracted from the doc comment of the field.
    pub brief: &'static str,
    /// The type of instrument used to record the metric.
    pub instrument: Instrument,
    /// The numeric representation for the metric values.
    pub value_type: MetricValueType,
}

/// Descriptor for a multivariate metrics.
#[derive(Debug, Serialize)]
pub struct MetricsDescriptor {
    /// Human-friendly group name.
    pub name: &'static str,
    /// Ordered field metadata.
    pub metrics: &'static [MetricsField],
}

/// Supported attribute value kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeValueType {
    /// String attribute value
    String,
    /// Integer attribute value
    Int,
    /// Double-precision floating-point attribute value
    Double,
    /// Boolean attribute value
    Boolean,
}

/// Metadata describing a single attribute field.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct AttributeField {
    /// Attribute key (canonical, may contain dots instead of underscores).
    pub key: &'static str,
    /// Short description extracted from doc comments.
    pub brief: &'static str,
    /// Value kind.
    pub r#type: AttributeValueType,
}

/// Descriptor for an attribute set.
#[derive(Debug)]
pub struct AttributesDescriptor {
    /// Human-friendly group name.
    pub name: &'static str,
    /// Ordered attribute field metadata.
    pub fields: &'static [AttributeField],
}
