// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric and Attribute descriptor types for metrics reflection.

use serde::{Deserialize, Serialize};

/// The type of instrument used to record the metric. Must be one of the following variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    /// A monotonic sum.
    Counter,
    /// A signed sum that can go up and down.
    UpDownCounter,
    /// A value that can arbitrarily go up and down, used for temperature or current memory usage
    Gauge,
    /// Pre-aggregated min/max/sum/count summary.
    ///
    /// Internally tracked as an `Mmsc` instrument; the OTLP bridge exports the
    /// aggregated snapshot as a bucketless OTel histogram.
    Mmsc,
    /// Exponential-histogram distribution (normal/detailed tiers).
    ///
    /// Tracked as a [`crate::instrument::DistributionValue`]; the OTLP bridge exports
    /// the aggregation as an OTel exponential-histogram point.
    ExponentialHistogram,
}

/// Aggregation temporality for sum-like instruments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Temporality {
    /// Each snapshot represents a delta over the reporting interval.
    Delta,
    /// Each snapshot represents the cumulative value at the time of reporting.
    Cumulative,
}

/// Numeric representation used by a metric field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Aggregation temporality (only meaningful for sum-like instruments).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporality: Option<Temporality>,
    /// The numeric representation for the metric values.
    pub value_type: MetricValueType,
}

impl MetricsField {
    /// Returns true when this field's stored value is an increment contributed
    /// by the reporting instrument, rather than an absolute reading.
    ///
    /// This is an input-side property and is independent of the aggregation
    /// temporality an exporter chooses. Incremental fields are combined into an
    /// accumulator with `add_in_place` (a merge, for distributions) and are
    /// cleared once the accumulator that owns them is drained; absolute fields
    /// (gauges, and sums whose instrument itself holds the running total) are
    /// replaced on collection and must survive a drain untouched.
    ///
    /// Distribution-shaped instruments are always incremental: the instrument
    /// holds only what was recorded since `clear_values`, so its snapshot is
    /// per-collection no matter how it is later exported. Delta and cumulative
    /// output are then a property of the accumulator being read -- the OTLP
    /// path drains and resets, while the admin path is read without reset for
    /// a cumulative Prometheus scrape -- not of this classification.
    ///
    /// Keeping accumulate, reset, and rollback on one predicate is what stops
    /// an instrument from being accumulated but never cleared, which silently
    /// re-exports the previous interval's observations.
    #[must_use]
    pub fn accumulates(&self) -> bool {
        match self.instrument {
            Instrument::Counter | Instrument::UpDownCounter => {
                self.temporality == Some(Temporality::Delta)
            }
            Instrument::Mmsc | Instrument::ExponentialHistogram => true,
            Instrument::Gauge => false,
        }
    }
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
    /// Map attribute value (key-value pairs)
    Map,
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

/// Descriptor for a single per-measurement enum attribute.
///
/// Measurement attributes vary per recorded item. Because the value space is a
/// closed `enum`, the ordered string forms of every variant are known at compile
/// time. The number of variants is the attribute's radix in the mixed-radix
/// bucket index used to address a metric set's items.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct MeasurementAttributeDescriptor {
    /// Attribute key (e.g. "signal").
    pub key: &'static str,
    /// Ordered string forms of the enum variants (declaration order).
    /// `variants.len()` is the attribute's radix.
    pub variants: &'static [&'static str],
}
