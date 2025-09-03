// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for proto message structs
//! from otlp metrics.proto.

use otel_arrow_rust::proto::opentelemetry::metrics::v1::{
    Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
    HistogramDataPoint, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    Summary, SummaryDataPoint, exponential_histogram_data_point::Buckets, metric::Data,
    summary_data_point::ValueAtQuantile,
};

use crate::otlp::proto::common::{
    KeyValueIter, ObjInstrumentationScope, ObjKeyValue, parse_span_id, parse_trace_id,
};
use crate::otlp::proto::resource::ObjResource;
use crate::otlp::proto::wrappers::{GenericIterator, GenericObj, Wraps};
use crate::views::common::{SpanId, Str, TraceId};
use crate::views::metrics::{
    BucketsView, DataPointFlags, DataType, DataView, ExemplarView,
    ExponentialHistogramDataPointView, ExponentialHistogramView, GaugeView, HistogramDataPointView,
    HistogramView, MetricView, MetricsView, NumberDataPointView, ResourceMetricsView,
    ScopeMetricsView, SumView, SummaryDataPointView, SummaryView, Value, ValueAtQuantileView,
};

/* ───────────────────────────── VIEW WRAPPERS (zero-alloc) ────────────── */

/// A wrapper for references to `ResourceMetrics`.
pub type ObjResourceMetrics<'a> = GenericObj<'a, ResourceMetrics>;

/// A wrapper for references to `ScopeMetrics`.
pub type ObjScopeMetrics<'a> = GenericObj<'a, ScopeMetrics>;

/// A wrapper for references to `Metric`.
pub type ObjMetric<'a> = GenericObj<'a, Metric>;

/// A wrapper for references to `Data`.
pub type ObjData<'a> = GenericObj<'a, Data>;

/// A wrapper for references to `Gauge`.
pub type ObjGauge<'a> = GenericObj<'a, Gauge>;

/// A wrapper for references to `Sum`.
pub type ObjSum<'a> = GenericObj<'a, Sum>;

/// A wrapper for references to `Histogram`.
pub type ObjHistogram<'a> = GenericObj<'a, Histogram>;

/// A wrapper for references to `ExponentialHistogram`
pub type ObjExponentialHistogram<'a> = GenericObj<'a, ExponentialHistogram>;

/// A wrapper for references to `Summary`.
pub type ObjSummary<'a> = GenericObj<'a, Summary>;

/// A wrapper for references to `NumberDataPoint`.
pub type ObjNumberDataPoint<'a> = GenericObj<'a, NumberDataPoint>;

/// A wrapper for references to `Exemplar`.
pub type ObjExemplar<'a> = GenericObj<'a, Exemplar>;

/// A wrapper for references to `SummaryDataPoint`.
pub type ObjSummaryDataPoint<'a> = GenericObj<'a, SummaryDataPoint>;

/// A wrapper for references to `HistogramDataPoint`.
pub type ObjHistogramDataPoint<'a> = GenericObj<'a, HistogramDataPoint>;

/// A wrapper for references to `ExponentialHistogramDataPoint`.
pub type ObjExponentialHistogramDataPoint<'a> = GenericObj<'a, ExponentialHistogramDataPoint>;

/// A wrapper for references to `Buckets`.
pub type ObjBuckets<'a> = GenericObj<'a, Buckets>;

/// A wrapper for references to `ValueAtQuantile`.
pub type ObjValueAtQuantile<'a> = GenericObj<'a, ValueAtQuantile>;

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

/// An iterator for `ObjResourceMetrics`; it consumes a slice iterator of `ResourceMetrics`.
pub type ResourceMetricsIter<'a> = GenericIterator<'a, ResourceMetrics, ObjResourceMetrics<'a>>;

/// An iterator for `ObjScopeMetrics`; it consumes a slice iterator of `ScopeMetrics`.
pub type ScopeMetricsIter<'a> = GenericIterator<'a, ScopeMetrics, ObjScopeMetrics<'a>>;

/// An iterator for `ObjMetric`; it consumes a slice iterator of `Metric`s.
pub type MetricIter<'a> = GenericIterator<'a, Metric, ObjMetric<'a>>;

/// An iterator for `ObjNumberDataPoint`; it consumes a slice iterator of `NumberDataPoint`s.
pub type NumberDataPointIter<'a> = GenericIterator<'a, NumberDataPoint, ObjNumberDataPoint<'a>>;

/// An iterator for `ObjExemplar`; it consumes a slice iterator of `Exemplar`s.
pub type ExemplarIter<'a> = GenericIterator<'a, Exemplar, ObjExemplar<'a>>;

/// An iterator for `ObjSummaryDataPoint`; it consumes a slice iterator of `SummaryDataPoint`.
pub type SummaryDataPointIter<'a> = GenericIterator<'a, SummaryDataPoint, ObjSummaryDataPoint<'a>>;

/// An iterator for `ObjHistogramDataPoint`; it consumes a slice iterator of `HistogramDataPoint`.
pub type HistogramDataPointIter<'a> =
    GenericIterator<'a, HistogramDataPoint, ObjHistogramDataPoint<'a>>;

/// An iterator for `ObjExponentialHistogramDataPoint`; it consumes a slice iterator of
/// `ExponentialHistogramDataPoint`.
pub type ExponentialHistogramDataPointIter<'a> =
    GenericIterator<'a, ExponentialHistogramDataPoint, ObjExponentialHistogramDataPoint<'a>>;

/// An iterator for `ObjValueAtQuantile`; it consumes a slice iterator of `ValueAtQuantile`.
pub type ValueAtQuantileIter<'a> = GenericIterator<'a, ValueAtQuantile, ObjValueAtQuantile<'a>>;

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */

impl MetricsView for MetricsData {
    type ResourceMetrics<'res>
        = ObjResourceMetrics<'res>
    where
        Self: 'res;

    type ResourceMetricsIter<'res>
        = ResourceMetricsIter<'res>
    where
        Self: 'res;

    fn resources(&self) -> Self::ResourceMetricsIter<'_> {
        ResourceMetricsIter::new(self.resource_metrics.iter())
    }
}

impl ResourceMetricsView for ObjResourceMetrics<'_> {
    type Resource<'res>
        = ObjResource<'res>
    where
        Self: 'res;

    type ScopeMetrics<'scp>
        = ObjScopeMetrics<'scp>
    where
        Self: 'scp;

    type ScopesIter<'scp>
        = ScopeMetricsIter<'scp>
    where
        Self: 'scp;

    fn resource(&self) -> Option<Self::Resource<'_>> {
        self.inner.resource.as_ref().map(ObjResource::new)
    }

    fn scopes(&self) -> Self::ScopesIter<'_> {
        ScopeMetricsIter::new(self.inner.scope_metrics.iter())
    }

    fn schema_url(&self) -> Str<'_> {
        &self.inner.schema_url
    }
}

impl ScopeMetricsView for ObjScopeMetrics<'_> {
    type Scope<'scp>
        = ObjInstrumentationScope<'scp>
    where
        Self: 'scp;

    type Metric<'met>
        = ObjMetric<'met>
    where
        Self: 'met;

    type MetricIter<'met>
        = MetricIter<'met>
    where
        Self: 'met;

    fn scope(&self) -> Option<Self::Scope<'_>> {
        self.inner.scope.as_ref().map(ObjInstrumentationScope::new)
    }

    fn metrics(&self) -> Self::MetricIter<'_> {
        MetricIter::new(self.inner.metrics.iter())
    }

    fn schema_url(&self) -> Str<'_> {
        &self.inner.schema_url
    }
}

impl MetricView for ObjMetric<'_> {
    type Data<'dat>
        = ObjData<'dat>
    where
        Self: 'dat;

    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    fn name(&self) -> Str<'_> {
        &self.inner.name
    }

    fn description(&self) -> Str<'_> {
        &self.inner.description
    }

    fn unit(&self) -> Str<'_> {
        &self.inner.unit
    }

    fn data(&self) -> Option<Self::Data<'_>> {
        self.inner.data.as_ref().map(ObjData::new)
    }

    fn metadata(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.metadata.iter())
    }
}

impl DataView<'_> for ObjData<'_> {
    type NumberDataPoint<'dp>
        = ObjNumberDataPoint<'dp>
    where
        Self: 'dp;

    type NumberDataPointIter<'dp>
        = NumberDataPointIter<'dp>
    where
        Self: 'dp;

    type Gauge<'gauge>
        = ObjGauge<'gauge>
    where
        Self: 'gauge;

    type Sum<'sum>
        = ObjSum<'sum>
    where
        Self: 'sum;

    type Histogram<'histogram>
        = ObjHistogram<'histogram>
    where
        Self: 'histogram;

    type ExponentialHistogram<'exp>
        = ObjExponentialHistogram<'exp>
    where
        Self: 'exp;

    type Summary<'summary>
        = ObjSummary<'summary>
    where
        Self: 'summary;

    fn value_type(&self) -> DataType {
        self.inner.into()
    }

    fn as_gauge(&self) -> Option<Self::Gauge<'_>> {
        match self.inner {
            Data::Gauge(gauge) => Some(ObjGauge::new(gauge)),
            _ => None,
        }
    }

    fn as_sum(&self) -> Option<Self::Sum<'_>> {
        match self.inner {
            Data::Sum(sum) => Some(ObjSum::new(sum)),
            _ => None,
        }
    }

    fn as_histogram(&self) -> Option<Self::Histogram<'_>> {
        match self.inner {
            Data::Histogram(histogram) => Some(ObjHistogram::new(histogram)),
            _ => None,
        }
    }

    fn as_exponential_histogram(&self) -> Option<Self::ExponentialHistogram<'_>> {
        match self.inner {
            Data::ExponentialHistogram(exp) => Some(ObjExponentialHistogram::new(exp)),
            _ => None,
        }
    }

    fn as_summary(&self) -> Option<Self::Summary<'_>> {
        match self.inner {
            Data::Summary(summary) => Some(ObjSummary::new(summary)),
            _ => None,
        }
    }
}

impl GaugeView for ObjGauge<'_> {
    type NumberDataPoint<'dp>
        = ObjNumberDataPoint<'dp>
    where
        Self: 'dp;

    type NumberDataPointIter<'dp>
        = NumberDataPointIter<'dp>
    where
        Self: 'dp;

    fn data_points(&self) -> Self::NumberDataPointIter<'_> {
        NumberDataPointIter::new(self.inner.data_points.iter())
    }
}

impl NumberDataPointView for ObjNumberDataPoint<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    type Exemplar<'ex>
        = ObjExemplar<'ex>
    where
        Self: 'ex;

    type ExemplarIter<'ex>
        = ExemplarIter<'ex>
    where
        Self: 'ex;

    fn start_time_unix_nano(&self) -> u64 {
        self.inner.start_time_unix_nano
    }

    fn time_unix_nano(&self) -> u64 {
        self.inner.time_unix_nano
    }

    fn value(&self) -> Option<Value> {
        self.inner.value.as_ref().map(Value::from)
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn exemplars(&self) -> Self::ExemplarIter<'_> {
        ExemplarIter::new(self.inner.exemplars.iter())
    }

    fn flags(&self) -> DataPointFlags {
        DataPointFlags::new(self.inner.flags)
    }
}

impl ExemplarView for ObjExemplar<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    fn filtered_attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.filtered_attributes.iter())
    }

    fn time_unix_nano(&self) -> u64 {
        self.inner.time_unix_nano
    }

    fn value(&self) -> Option<Value> {
        self.inner.value.as_ref().map(Value::from)
    }

    fn span_id(&self) -> Option<&SpanId> {
        parse_span_id(&self.inner.span_id)
    }

    fn trace_id(&self) -> Option<&TraceId> {
        parse_trace_id(&self.inner.trace_id)
    }
}

impl SumView for ObjSum<'_> {
    type NumberDataPoint<'dp>
        = ObjNumberDataPoint<'dp>
    where
        Self: 'dp;

    type NumberDataPointIter<'dp>
        = NumberDataPointIter<'dp>
    where
        Self: 'dp;

    fn data_points(&self) -> Self::NumberDataPointIter<'_> {
        NumberDataPointIter::new(self.inner.data_points.iter())
    }

    fn aggregation_temporality(&self) -> crate::views::metrics::AggregationTemporality {
        self.inner.aggregation_temporality().into()
    }

    fn is_monotonic(&self) -> bool {
        self.inner.is_monotonic
    }
}

impl HistogramView for ObjHistogram<'_> {
    type HistogramDataPoint<'dp>
        = ObjHistogramDataPoint<'dp>
    where
        Self: 'dp;

    type HistogramDataPointIter<'dp>
        = HistogramDataPointIter<'dp>
    where
        Self: 'dp;

    fn data_points(&self) -> Self::HistogramDataPointIter<'_> {
        HistogramDataPointIter::new(self.inner.data_points.iter())
    }

    fn aggregation_temporality(&self) -> crate::views::metrics::AggregationTemporality {
        self.inner.aggregation_temporality().into()
    }
}

impl HistogramDataPointView for ObjHistogramDataPoint<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    type BucketCountIter<'bc>
        = std::slice::Iter<'bc, u64>
    where
        Self: 'bc;

    type ExplicitBoundsIter<'eb>
        = std::slice::Iter<'eb, f64>
    where
        Self: 'eb;

    type Exemplar<'ex>
        = ObjExemplar<'ex>
    where
        Self: 'ex;

    type ExemplarIter<'ex>
        = ExemplarIter<'ex>
    where
        Self: 'ex;

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn start_time_unix_nano(&self) -> u64 {
        self.inner.start_time_unix_nano
    }

    fn time_unix_nano(&self) -> u64 {
        self.inner.time_unix_nano
    }

    fn count(&self) -> u64 {
        self.inner.count
    }

    fn sum(&self) -> Option<f64> {
        self.inner.sum
    }

    fn bucket_counts(&self) -> Self::BucketCountIter<'_> {
        self.inner.bucket_counts.iter()
    }

    fn explicit_bounds(&self) -> Self::ExplicitBoundsIter<'_> {
        self.inner.explicit_bounds.iter()
    }

    fn exemplars(&self) -> Self::ExemplarIter<'_> {
        ExemplarIter::new(self.inner.exemplars.iter())
    }

    fn flags(&self) -> DataPointFlags {
        DataPointFlags::new(self.inner.flags)
    }

    fn min(&self) -> Option<f64> {
        self.inner.min
    }

    fn max(&self) -> Option<f64> {
        self.inner.max
    }
}

impl ValueAtQuantileView for ObjValueAtQuantile<'_> {
    fn quantile(&self) -> f64 {
        self.inner.quantile
    }

    fn value(&self) -> f64 {
        self.inner.value
    }
}

impl ExponentialHistogramView for ObjExponentialHistogram<'_> {
    type ExponentialHistogramDataPoint<'edp>
        = ObjExponentialHistogramDataPoint<'edp>
    where
        Self: 'edp;

    type ExponentialHistogramDataPointIter<'edp>
        = ExponentialHistogramDataPointIter<'edp>
    where
        Self: 'edp;

    fn data_points(&self) -> Self::ExponentialHistogramDataPointIter<'_> {
        ExponentialHistogramDataPointIter::new(self.inner.data_points.iter())
    }

    fn aggregation_temporality(&self) -> crate::views::metrics::AggregationTemporality {
        self.inner.aggregation_temporality().into()
    }
}

impl ExponentialHistogramDataPointView for ObjExponentialHistogramDataPoint<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    type Buckets<'b>
        = ObjBuckets<'b>
    where
        Self: 'b;

    type Exemplar<'ex>
        = ObjExemplar<'ex>
    where
        Self: 'ex;

    type ExemplarIter<'ex>
        = ExemplarIter<'ex>
    where
        Self: 'ex;

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn start_time_unix_nano(&self) -> u64 {
        self.inner.start_time_unix_nano
    }

    fn time_unix_nano(&self) -> u64 {
        self.inner.time_unix_nano
    }

    fn count(&self) -> u64 {
        self.inner.count
    }

    fn sum(&self) -> Option<f64> {
        self.inner.sum
    }

    fn scale(&self) -> i32 {
        self.inner.scale
    }

    fn zero_count(&self) -> u64 {
        self.inner.zero_count
    }

    fn positive(&self) -> Option<Self::Buckets<'_>> {
        self.inner.positive.as_ref().map(ObjBuckets::new)
    }

    fn negative(&self) -> Option<Self::Buckets<'_>> {
        self.inner.negative.as_ref().map(ObjBuckets::new)
    }

    fn flags(&self) -> DataPointFlags {
        DataPointFlags::new(self.inner.flags)
    }

    fn exemplars(&self) -> Self::ExemplarIter<'_> {
        ExemplarIter::new(self.inner.exemplars.iter())
    }

    fn min(&self) -> Option<f64> {
        self.inner.min
    }

    fn max(&self) -> Option<f64> {
        self.inner.max
    }

    fn zero_threshold(&self) -> f64 {
        self.inner.zero_threshold
    }
}

impl BucketsView for ObjBuckets<'_> {
    type BucketCountIter<'bc>
        = std::slice::Iter<'bc, u64>
    where
        Self: 'bc;

    fn offset(&self) -> i32 {
        self.inner.offset
    }

    fn bucket_counts(&self) -> Self::BucketCountIter<'_> {
        self.inner.bucket_counts.iter()
    }
}

impl SummaryView for ObjSummary<'_> {
    type SummaryDataPoint<'dp>
        = ObjSummaryDataPoint<'dp>
    where
        Self: 'dp;

    type SummaryDataPointIter<'dp>
        = SummaryDataPointIter<'dp>
    where
        Self: 'dp;

    fn data_points(&self) -> Self::SummaryDataPointIter<'_> {
        SummaryDataPointIter::new(self.inner.data_points.iter())
    }
}

impl SummaryDataPointView for ObjSummaryDataPoint<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    type ValueAtQuantile<'vaq>
        = ObjValueAtQuantile<'vaq>
    where
        Self: 'vaq;

    type ValueAtQuantileIter<'vaq>
        = ValueAtQuantileIter<'vaq>
    where
        Self: 'vaq;

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn start_time_unix_nano(&self) -> u64 {
        self.inner.start_time_unix_nano
    }

    fn time_unix_nano(&self) -> u64 {
        self.inner.time_unix_nano
    }

    fn count(&self) -> u64 {
        self.inner.count
    }

    fn sum(&self) -> f64 {
        self.inner.sum
    }

    fn quantile_values(&self) -> Self::ValueAtQuantileIter<'_> {
        ValueAtQuantileIter::new(self.inner.quantile_values.iter())
    }

    fn flags(&self) -> DataPointFlags {
        DataPointFlags::new(self.inner.flags)
    }
}
