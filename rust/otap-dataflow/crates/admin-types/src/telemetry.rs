// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared telemetry-scoped admin models.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Options for structured metrics requests.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsOptions {
    /// Reset metrics after reading.
    #[serde(default)]
    pub reset: bool,
    /// Keep all-zero metric sets.
    #[serde(default)]
    pub keep_all_zeroes: bool,
}

impl MetricsOptions {
    /// Converts these options into URL query pairs for SDK transports.
    ///
    /// Most callers do not need this directly because the built-in HTTP
    /// transport uses it automatically.
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("reset", self.reset.to_string()),
            ("keep_all_zeroes", self.keep_all_zeroes.to_string()),
        ]
    }
}

/// Verbose JSON metrics response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// RFC 3339 timestamp.
    pub timestamp: String,
    /// Metric sets with metadata.
    pub metric_sets: Vec<MetricSetWithMetadata>,
}

/// Compact JSON metrics response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactMetricsResponse {
    /// RFC 3339 timestamp.
    pub timestamp: String,
    /// Metric sets without field metadata.
    pub metric_sets: Vec<MetricSet>,
}

/// Metric set with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSetWithMetadata {
    /// Metric set name.
    pub name: String,
    /// Short description.
    pub brief: String,
    /// Attributes.
    pub attributes: BTreeMap<String, AttributeValue>,
    /// Individual metrics.
    pub metrics: Vec<MetricDataPoint>,
}

/// Metric set without field metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSet {
    /// Metric set name.
    pub name: String,
    /// Attributes.
    pub attributes: BTreeMap<String, AttributeValue>,
    /// Attributes that identify the metric data point bucket.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub data_point_attributes: BTreeMap<String, AttributeValue>,
    /// Metric values keyed by field name.
    pub metrics: BTreeMap<String, MetricValue>,
}

/// Metric data point with field metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// Metric descriptor fields.
    #[serde(flatten)]
    pub metadata: MetricsField,
    /// Attributes that identify this metric data point.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, AttributeValue>,
    /// Metric value.
    pub value: MetricValue,
}

/// Attribute value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    /// String value.
    String(String),
    /// Signed integer.
    Int(i64),
    /// Unsigned integer.
    UInt(u64),
    /// Floating-point value.
    Double(f64),
    /// Boolean value.
    Boolean(bool),
    /// Map value.
    Map(BTreeMap<String, AttributeValue>),
}

/// Metric value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(variant_size_differences)]
pub enum MetricValue {
    /// Unsigned integer.
    U64(u64),
    /// Floating-point value.
    F64(f64),
    /// Summary of a pre-aggregated distribution.
    Distribution(DistributionSummary),
}

/// Summary of a distribution metric.
///
/// The exact statistics are always present. Everything that depends on bucket
/// structure lives in [`details`](Self::details), which is absent for the
/// basic tier because that tier encodes no buckets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DistributionSummary {
    /// Minimum observed value.
    ///
    /// Defaulted because the admin serializer omits it, along with `max` and
    /// `sum`, for a distribution that observed nothing: they carry no meaning
    /// without an observation, and reporting them would invite a consumer to
    /// read a zero as a measurement. `count` needs no default, since it is
    /// written for every distribution including the empty one; a payload
    /// missing it is malformed, and failing to parse says so rather than
    /// quietly producing an empty distribution.
    #[serde(default)]
    pub min: f64,
    /// Maximum observed value.
    #[serde(default)]
    pub max: f64,
    /// Sum of observed values.
    #[serde(default)]
    pub sum: f64,
    /// Total observation count, including exact zeros.
    pub count: u64,
    /// Everything recoverable only from the encoded buckets.
    ///
    /// `None` for the basic tier, which keeps no buckets: it tracks no zero
    /// population -- a zero there is an ordinary observation reflected in
    /// `min` -- and can estimate no quantiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<DistributionDetails>,
}

/// The bucket-derived part of a [`DistributionSummary`].
///
/// Every field here is a byproduct of one walk over the encoded buckets, so
/// they are reported together rather than exposed as separate accessors that
/// would each pay for their own scan.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DistributionDetails {
    /// Number of observations excluded from the bucket range because they
    /// were exactly zero.
    pub zero_count: u64,
    /// Relative error bound that applies to the quantile estimates.
    pub relative_error: f64,
    /// Estimated median, within `relative_error` of the true value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p50: Option<f64>,
    /// Estimated 90th percentile, within `relative_error` of the true value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p90: Option<f64>,
    /// Estimated 99th percentile, within `relative_error` of the true value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p99: Option<f64>,
}

/// Metric field descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsField {
    /// Metric name.
    pub name: String,
    /// UCUM unit.
    pub unit: String,
    /// Short description.
    pub brief: String,
    /// Instrument kind.
    pub instrument: Instrument,
    /// Aggregation temporality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporality: Option<Temporality>,
    /// Numeric value representation.
    pub value_type: MetricValueType,
}

/// Instrument type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    /// Counter.
    Counter,
    /// Up/down counter.
    UpDownCounter,
    /// Gauge.
    Gauge,
    /// Distribution keeping exact min/max/sum/count with no encoded buckets.
    Mmsc,
    /// Distribution keeping exponential-histogram buckets.
    ExponentialHistogram,
}

/// Aggregation temporality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Temporality {
    /// Delta.
    Delta,
    /// Cumulative.
    Cumulative,
}

/// Metric numeric representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricValueType {
    /// Unsigned integer.
    U64,
    /// Floating-point value.
    F64,
}

/// Logs query.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogsQuery {
    /// Return logs strictly newer than this sequence.
    #[serde(default)]
    pub after: Option<u64>,
    /// Maximum number of retained logs to return.
    #[serde(default)]
    pub limit: Option<usize>,
}

impl LogsQuery {
    /// Converts this query into URL query pairs for SDK transports.
    ///
    /// Most callers do not need this directly because the built-in HTTP
    /// transport uses it automatically.
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(after) = self.after {
            pairs.push(("after", after.to_string()));
        }
        if let Some(limit) = self.limit {
            pairs.push(("limit", limit.to_string()));
        }
        pairs
    }
}

/// Logs response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogsResponse {
    /// Oldest retained sequence number.
    pub oldest_seq: Option<u64>,
    /// Newest retained sequence number.
    pub newest_seq: Option<u64>,
    /// Sequence to request next.
    pub next_seq: u64,
    /// Sequence before which history was truncated.
    pub truncated_before_seq: Option<u64>,
    /// Count dropped on ingest.
    pub dropped_on_ingest: u64,
    /// Count dropped on retention.
    pub dropped_on_retention: u64,
    /// Retained bytes.
    pub retained_bytes: usize,
    /// Rendered log entries.
    pub logs: Vec<LogEntry>,
}

/// Log entry returned by the logs endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    /// Sequence number.
    pub seq: u64,
    /// RFC 3339 timestamp.
    pub timestamp: String,
    /// Log level.
    pub level: String,
    /// Log target.
    pub target: String,
    /// Event name.
    pub event_name: String,
    /// Source file.
    pub file: Option<String>,
    /// Source line.
    pub line: Option<u32>,
    /// Fully rendered log message.
    pub rendered: String,
    /// Resolved contexts.
    pub contexts: Vec<ResolvedLogContext>,
}

/// Resolved log context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedLogContext {
    /// Entity key.
    pub entity_key: String,
    /// Schema name.
    pub schema_name: Option<String>,
    /// Attributes.
    pub attributes: BTreeMap<String, AttributeValue>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::{Value, json};

    fn assert_roundtrip<T>(value: Value)
    where
        T: DeserializeOwned + Serialize,
    {
        let parsed: T = serde_json::from_value(value.clone()).expect("fixture should deserialize");
        let serialized = serde_json::to_value(parsed).expect("model should serialize");
        assert_eq!(serialized, value);
    }

    #[test]
    fn logs_response_roundtrips_current_wire_shape() {
        assert_roundtrip::<LogsResponse>(json!({
            "oldest_seq": 1,
            "newest_seq": 2,
            "next_seq": 3,
            "truncated_before_seq": null,
            "dropped_on_ingest": 0,
            "dropped_on_retention": 0,
            "retained_bytes": 128,
            "logs": [
                {
                    "seq": 2,
                    "timestamp": "2026-01-01T00:00:00Z",
                    "level": "INFO",
                    "target": "admin",
                    "event_name": "startup",
                    "file": "src/main.rs",
                    "line": 42,
                    "rendered": "started",
                    "contexts": [
                        {
                            "entity_key": "EntityKey(1)",
                            "schema_name": "pipeline.attrs",
                            "attributes": {
                                "node.id": { "String": "receiver" }
                            }
                        }
                    ]
                }
            ]
        }));
    }

    #[test]
    fn metrics_verbose_roundtrips_current_wire_shape() {
        assert_roundtrip::<MetricsResponse>(json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "metric_sets": [
                {
                    "name": "engine",
                    "brief": "engine",
                    "attributes": {
                        "node.id": { "String": "receiver" }
                    },
                    "metrics": [
                        {
                            "name": "items",
                            "unit": "{item}",
                            "brief": "items processed",
                            "instrument": "counter",
                            "temporality": "cumulative",
                            "value_type": "u64",
                            "attributes": {
                                "signal": { "String": "logs" }
                            },
                            "value": 5
                        }
                    ]
                }
            ]
        }));
    }

    #[test]
    fn metrics_compact_roundtrips_current_wire_shape() {
        assert_roundtrip::<CompactMetricsResponse>(json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "metric_sets": [
                {
                    "name": "engine",
                    "attributes": {
                        "node.id": { "String": "receiver" }
                    },
                    "data_point_attributes": {
                        "signal": { "String": "logs" }
                    },
                    "metrics": {
                        "items": 5
                    }
                }
            ]
        }));
    }

    #[test]
    fn metrics_options_roundtrip_current_wire_shape() {
        assert_roundtrip::<MetricsOptions>(json!({
            "reset": true,
            "keep_all_zeroes": true
        }));
    }
}
