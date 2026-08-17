// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP JSON serialization for metrics pdata views.
//!
//! The encoder traverses `MetricsView` hierarchies and dispatches each metric data variant to a
//! small serde adapter. Data points, exemplars, histogram buckets, and summaries are streamed from
//! borrowed view iterators without converting the input into an owned protobuf message.

use super::common::{
    AttributeIterJson, HexId, ProtoDouble, ProtoI64, ProtoU64, ResourceJson, ScopeJson, Utf8,
};
use super::{JsonEncodeError, write_json};
use otap_df_pdata_views::views::metrics::{
    AggregationTemporality, BucketsView, DataType, DataView, ExemplarView,
    ExponentialHistogramDataPointView, ExponentialHistogramView, GaugeView, HistogramDataPointView,
    HistogramView, MetricView, MetricsView, NumberDataPointView, ResourceMetricsView,
    ScopeMetricsView, SumView, SummaryDataPointView, SummaryView, Value, ValueAtQuantileView,
};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::io::Write;

/// Writes one metrics pdata view as a compact OTLP JSON document.
///
/// This function does not add a delimiter, bound output size, or roll back bytes already
/// accepted by the writer when serialization fails. Callers own those policies.
pub fn write_metrics_json<M: MetricsView, W: Write>(
    metrics: &M,
    output: &mut W,
) -> Result<(), JsonEncodeError> {
    write_json(&MetricsJson(metrics), output)
}

struct MetricsJson<'a, M: MetricsView>(&'a M);

impl<M: MetricsView> Serialize for MetricsJson<'_, M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.resources().next().is_some() {
            map.serialize_entry("resourceMetrics", &ResourceMetricsList(self.0))?;
        }
        map.end()
    }
}

struct ResourceMetricsList<'a, M: MetricsView>(&'a M);

impl<M: MetricsView> Serialize for ResourceMetricsList<'_, M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for resource in self.0.resources() {
            sequence.serialize_element(&ResourceMetricsJson(resource))?;
        }
        sequence.end()
    }
}

struct ResourceMetricsJson<R: ResourceMetricsView>(R);

impl<R: ResourceMetricsView> Serialize for ResourceMetricsJson<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(resource) = self.0.resource() {
            map.serialize_entry("resource", &ResourceJson(&resource))?;
        }
        if self.0.scopes().next().is_some() {
            map.serialize_entry("scopeMetrics", &ScopeMetricsList(&self.0))?;
        }
        if let Some(schema_url) = self.0.schema_url().filter(|value| !value.is_empty()) {
            map.serialize_entry("schemaUrl", &Utf8(schema_url))?;
        }
        map.end()
    }
}

struct ScopeMetricsList<'a, R: ResourceMetricsView>(&'a R);

impl<R: ResourceMetricsView> Serialize for ScopeMetricsList<'_, R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for scope in self.0.scopes() {
            sequence.serialize_element(&ScopeMetricsJson(scope))?;
        }
        sequence.end()
    }
}

struct ScopeMetricsJson<M: ScopeMetricsView>(M);

impl<M: ScopeMetricsView> Serialize for ScopeMetricsJson<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(scope) = self.0.scope() {
            map.serialize_entry("scope", &ScopeJson(&scope))?;
        }
        if self.0.metrics().next().is_some() {
            map.serialize_entry("metrics", &MetricList(&self.0))?;
        }
        let schema_url = self.0.schema_url();
        if !schema_url.is_empty() {
            map.serialize_entry("schemaUrl", &Utf8(schema_url))?;
        }
        map.end()
    }
}

struct MetricList<'a, M: ScopeMetricsView>(&'a M);

impl<M: ScopeMetricsView> Serialize for MetricList<'_, M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for metric in self.0.metrics() {
            sequence.serialize_element(&MetricJson(metric))?;
        }
        sequence.end()
    }
}

struct MetricJson<M: MetricView>(M);

impl<M: MetricView> Serialize for MetricJson<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        let name = self.0.name();
        if !name.is_empty() {
            map.serialize_entry("name", &Utf8(name))?;
        }
        let description = self.0.description();
        if !description.is_empty() {
            map.serialize_entry("description", &Utf8(description))?;
        }
        let unit = self.0.unit();
        if !unit.is_empty() {
            map.serialize_entry("unit", &Utf8(unit))?;
        }
        if let Some(data) = self.0.data() {
            match data.value_type() {
                DataType::Gauge => {
                    if let Some(value) = data.as_gauge() {
                        map.serialize_entry("gauge", &GaugeJson(value))?;
                    }
                }
                DataType::Sum => {
                    if let Some(value) = data.as_sum() {
                        map.serialize_entry("sum", &SumJson(value))?;
                    }
                }
                DataType::Histogram => {
                    if let Some(value) = data.as_histogram() {
                        map.serialize_entry("histogram", &HistogramJson(value))?;
                    }
                }
                DataType::ExponentialHistogram => {
                    if let Some(value) = data.as_exponential_histogram() {
                        map.serialize_entry(
                            "exponentialHistogram",
                            &ExponentialHistogramJson(value),
                        )?;
                    }
                }
                DataType::Summary => {
                    if let Some(value) = data.as_summary() {
                        map.serialize_entry("summary", &SummaryJson(value))?;
                    }
                }
            }
        }
        if self.0.metadata().next().is_some() {
            map.serialize_entry("metadata", &AttributeIterJson::new(self.0.metadata()))?;
        }
        map.end()
    }
}

struct GaugeJson<G: GaugeView>(G);

impl<G: GaugeView> Serialize for GaugeJson<G> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.data_points().next().is_some() {
            map.serialize_entry("dataPoints", &GaugeDataPointList(&self.0))?;
        }
        map.end()
    }
}

struct SumJson<T: SumView>(T);

impl<T: SumView> Serialize for SumJson<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.data_points().next().is_some() {
            map.serialize_entry("dataPoints", &SumDataPointList(&self.0))?;
        }
        if let Some(temporality) = aggregation_temporality(self.0.aggregation_temporality()) {
            map.serialize_entry("aggregationTemporality", &temporality)?;
        }
        if self.0.is_monotonic() {
            map.serialize_entry("isMonotonic", &true)?;
        }
        map.end()
    }
}

struct GaugeDataPointList<'a, G: GaugeView>(&'a G);

impl<G: GaugeView> Serialize for GaugeDataPointList<'_, G> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for point in self.0.data_points() {
            sequence.serialize_element(&NumberDataPointJson(point))?;
        }
        sequence.end()
    }
}

struct SumDataPointList<'a, T: SumView>(&'a T);

impl<T: SumView> Serialize for SumDataPointList<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for point in self.0.data_points() {
            sequence.serialize_element(&NumberDataPointJson(point))?;
        }
        sequence.end()
    }
}

struct NumberDataPointJson<P: NumberDataPointView>(P);

impl<P: NumberDataPointView> Serialize for NumberDataPointJson<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        serialize_times(
            &mut map,
            self.0.start_time_unix_nano(),
            self.0.time_unix_nano(),
        )?;
        if let Some(value) = self.0.value() {
            serialize_value(&mut map, value)?;
        }
        if self.0.exemplars().next().is_some() {
            map.serialize_entry("exemplars", &ExemplarList::new(self.0.exemplars()))?;
        }
        let flags = self.0.flags().into_inner();
        if flags != 0 {
            map.serialize_entry("flags", &flags)?;
        }
        map.end()
    }
}

fn serialize_times<S: SerializeMap>(
    map: &mut S,
    start_time: u64,
    time: u64,
) -> Result<(), S::Error> {
    if start_time != 0 {
        map.serialize_entry("startTimeUnixNano", &ProtoU64(start_time))?;
    }
    if time != 0 {
        map.serialize_entry("timeUnixNano", &ProtoU64(time))?;
    }
    Ok(())
}

fn serialize_value<S: SerializeMap>(map: &mut S, value: Value) -> Result<(), S::Error> {
    match value {
        Value::Double(value) => map.serialize_entry("asDouble", &ProtoDouble(value)),
        Value::Integer(value) => map.serialize_entry("asInt", &ProtoI64(value)),
    }
}

struct ExemplarList<I>(RefCell<Option<I>>);

impl<I> ExemplarList<I> {
    const fn new(iter: I) -> Self {
        Self(RefCell::new(Some(iter)))
    }
}

impl<I, E> Serialize for ExemplarList<I>
where
    I: Iterator<Item = E>,
    E: ExemplarView,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        let mut iter = self
            .0
            .borrow_mut()
            .take()
            .expect("exemplar iterator must be serialized once");
        for exemplar in &mut iter {
            sequence.serialize_element(&ExemplarJson(exemplar))?;
        }
        sequence.end()
    }
}

struct ExemplarJson<E: ExemplarView>(E);

impl<E: ExemplarView> Serialize for ExemplarJson<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.filtered_attributes().next().is_some() {
            map.serialize_entry(
                "filteredAttributes",
                &AttributeIterJson::new(self.0.filtered_attributes()),
            )?;
        }
        let time = self.0.time_unix_nano();
        if time != 0 {
            map.serialize_entry("timeUnixNano", &ProtoU64(time))?;
        }
        if let Some(value) = self.0.value() {
            serialize_value(&mut map, value)?;
        }
        if let Some(span_id) = self.0.span_id() {
            map.serialize_entry("spanId", &HexId(span_id))?;
        }
        if let Some(trace_id) = self.0.trace_id() {
            map.serialize_entry("traceId", &HexId(trace_id))?;
        }
        map.end()
    }
}

struct HistogramJson<H: HistogramView>(H);

impl<H: HistogramView> Serialize for HistogramJson<H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.data_points().next().is_some() {
            map.serialize_entry("dataPoints", &HistogramDataPointList(&self.0))?;
        }
        if let Some(temporality) = aggregation_temporality(self.0.aggregation_temporality()) {
            map.serialize_entry("aggregationTemporality", &temporality)?;
        }
        map.end()
    }
}

struct HistogramDataPointList<'a, H: HistogramView>(&'a H);

impl<H: HistogramView> Serialize for HistogramDataPointList<'_, H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for point in self.0.data_points() {
            sequence.serialize_element(&HistogramDataPointJson(point))?;
        }
        sequence.end()
    }
}

struct HistogramDataPointJson<P: HistogramDataPointView>(P);

impl<P: HistogramDataPointView> Serialize for HistogramDataPointJson<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        serialize_times(
            &mut map,
            self.0.start_time_unix_nano(),
            self.0.time_unix_nano(),
        )?;
        let count = self.0.count();
        if count != 0 {
            map.serialize_entry("count", &ProtoU64(count))?;
        }
        if let Some(sum) = self.0.sum() {
            map.serialize_entry("sum", &ProtoDouble(sum))?;
        }
        if self.0.bucket_counts().next().is_some() {
            map.serialize_entry("bucketCounts", &ProtoU64List::new(self.0.bucket_counts()))?;
        }
        if self.0.explicit_bounds().next().is_some() {
            map.serialize_entry(
                "explicitBounds",
                &ProtoDoubleList::new(self.0.explicit_bounds()),
            )?;
        }
        if self.0.exemplars().next().is_some() {
            map.serialize_entry("exemplars", &ExemplarList::new(self.0.exemplars()))?;
        }
        let flags = self.0.flags().into_inner();
        if flags != 0 {
            map.serialize_entry("flags", &flags)?;
        }
        if let Some(min) = self.0.min() {
            map.serialize_entry("min", &ProtoDouble(min))?;
        }
        if let Some(max) = self.0.max() {
            map.serialize_entry("max", &ProtoDouble(max))?;
        }
        map.end()
    }
}

struct ExponentialHistogramJson<H: ExponentialHistogramView>(H);

impl<H: ExponentialHistogramView> Serialize for ExponentialHistogramJson<H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.data_points().next().is_some() {
            map.serialize_entry("dataPoints", &ExponentialHistogramDataPointList(&self.0))?;
        }
        if let Some(temporality) = aggregation_temporality(self.0.aggregation_temporality()) {
            map.serialize_entry("aggregationTemporality", &temporality)?;
        }
        map.end()
    }
}

struct ExponentialHistogramDataPointList<'a, H: ExponentialHistogramView>(&'a H);

impl<H: ExponentialHistogramView> Serialize for ExponentialHistogramDataPointList<'_, H> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for point in self.0.data_points() {
            sequence.serialize_element(&ExponentialHistogramDataPointJson(point))?;
        }
        sequence.end()
    }
}

struct ExponentialHistogramDataPointJson<P: ExponentialHistogramDataPointView>(P);

impl<P: ExponentialHistogramDataPointView> Serialize for ExponentialHistogramDataPointJson<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        serialize_times(
            &mut map,
            self.0.start_time_unix_nano(),
            self.0.time_unix_nano(),
        )?;
        let count = self.0.count();
        if count != 0 {
            map.serialize_entry("count", &ProtoU64(count))?;
        }
        if let Some(sum) = self.0.sum() {
            map.serialize_entry("sum", &ProtoDouble(sum))?;
        }
        let scale = self.0.scale();
        if scale != 0 {
            map.serialize_entry("scale", &scale)?;
        }
        let zero_count = self.0.zero_count();
        if zero_count != 0 {
            map.serialize_entry("zeroCount", &ProtoU64(zero_count))?;
        }
        if let Some(positive) = self.0.positive() {
            map.serialize_entry("positive", &BucketsJson(positive))?;
        }
        if let Some(negative) = self.0.negative() {
            map.serialize_entry("negative", &BucketsJson(negative))?;
        }
        let flags = self.0.flags().into_inner();
        if flags != 0 {
            map.serialize_entry("flags", &flags)?;
        }
        if self.0.exemplars().next().is_some() {
            map.serialize_entry("exemplars", &ExemplarList::new(self.0.exemplars()))?;
        }
        if let Some(min) = self.0.min() {
            map.serialize_entry("min", &ProtoDouble(min))?;
        }
        if let Some(max) = self.0.max() {
            map.serialize_entry("max", &ProtoDouble(max))?;
        }
        let zero_threshold = self.0.zero_threshold();
        if zero_threshold != 0.0 {
            map.serialize_entry("zeroThreshold", &ProtoDouble(zero_threshold))?;
        }
        map.end()
    }
}

struct BucketsJson<B: BucketsView>(B);

impl<B: BucketsView> Serialize for BucketsJson<B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        let offset = self.0.offset();
        if offset != 0 {
            map.serialize_entry("offset", &offset)?;
        }
        if self.0.bucket_counts().next().is_some() {
            map.serialize_entry("bucketCounts", &ProtoU64List::new(self.0.bucket_counts()))?;
        }
        map.end()
    }
}

struct SummaryJson<T: SummaryView>(T);

impl<T: SummaryView> Serialize for SummaryJson<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.data_points().next().is_some() {
            map.serialize_entry("dataPoints", &SummaryDataPointList(&self.0))?;
        }
        map.end()
    }
}

struct SummaryDataPointList<'a, T: SummaryView>(&'a T);

impl<T: SummaryView> Serialize for SummaryDataPointList<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for point in self.0.data_points() {
            sequence.serialize_element(&SummaryDataPointJson(point))?;
        }
        sequence.end()
    }
}

struct SummaryDataPointJson<P: SummaryDataPointView>(P);

impl<P: SummaryDataPointView> Serialize for SummaryDataPointJson<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        serialize_times(
            &mut map,
            self.0.start_time_unix_nano(),
            self.0.time_unix_nano(),
        )?;
        let count = self.0.count();
        if count != 0 {
            map.serialize_entry("count", &ProtoU64(count))?;
        }
        let sum = self.0.sum();
        if sum != 0.0 {
            map.serialize_entry("sum", &ProtoDouble(sum))?;
        }
        if self.0.quantile_values().next().is_some() {
            map.serialize_entry("quantileValues", &QuantileValueList(&self.0))?;
        }
        let flags = self.0.flags().into_inner();
        if flags != 0 {
            map.serialize_entry("flags", &flags)?;
        }
        map.end()
    }
}

struct QuantileValueList<'a, P: SummaryDataPointView>(&'a P);

impl<P: SummaryDataPointView> Serialize for QuantileValueList<'_, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for value in self.0.quantile_values() {
            sequence.serialize_element(&QuantileValueJson(value))?;
        }
        sequence.end()
    }
}

struct QuantileValueJson<Q: ValueAtQuantileView>(Q);

impl<Q: ValueAtQuantileView> Serialize for QuantileValueJson<Q> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        let quantile = self.0.quantile();
        if quantile != 0.0 {
            map.serialize_entry("quantile", &ProtoDouble(quantile))?;
        }
        let value = self.0.value();
        if value != 0.0 {
            map.serialize_entry("value", &ProtoDouble(value))?;
        }
        map.end()
    }
}

struct ProtoU64List<I>(RefCell<Option<I>>);

impl<I> ProtoU64List<I> {
    const fn new(iter: I) -> Self {
        Self(RefCell::new(Some(iter)))
    }
}

impl<I: Iterator<Item = u64>> Serialize for ProtoU64List<I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        let mut iter = self
            .0
            .borrow_mut()
            .take()
            .expect("u64 iterator must be serialized once");
        for value in &mut iter {
            sequence.serialize_element(&ProtoU64(value))?;
        }
        sequence.end()
    }
}

struct ProtoDoubleList<I>(RefCell<Option<I>>);

impl<I> ProtoDoubleList<I> {
    const fn new(iter: I) -> Self {
        Self(RefCell::new(Some(iter)))
    }
}

impl<I: Iterator<Item = f64>> Serialize for ProtoDoubleList<I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        let mut iter = self
            .0
            .borrow_mut()
            .take()
            .expect("double iterator must be serialized once");
        for value in &mut iter {
            sequence.serialize_element(&ProtoDouble(value))?;
        }
        sequence.end()
    }
}

fn aggregation_temporality(value: AggregationTemporality) -> Option<i32> {
    match value {
        AggregationTemporality::Unspecified => None,
        AggregationTemporality::Delta => Some(1),
        AggregationTemporality::Cumulative => Some(2),
    }
}
