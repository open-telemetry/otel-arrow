// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! **Backend-agnostic, zero-copy view traits for OTLP Metrics.**
//!
//! ```text
//! MetricsData
//! └─── ResourceMetrics
//!   ├── Resource
//!   ├── SchemaURL
//!   └── ScopeMetrics
//!      ├── Scope
//!      ├── SchemaURL
//!      └── Metric
//!         ├── Name
//!         ├── Description
//!         ├── Unit
//!         └── data
//!            ├── Gauge
//!            ├── Sum
//!            ├── Histogram
//!            ├── ExponentialHistogram
//!            └── Summary
//! ```

use otel_arrow_rust::proto::opentelemetry::metrics::v1 as proto;

use crate::views::{
    common::{AttributeView, InstrumentationScopeView, SpanId, Str, TraceId},
    resource::ResourceView,
};

/// View for top level MetricsData
pub trait MetricsView {
    /// The `ResourcesMetricsView` type associated with this implementation.
    type ResourceMetrics<'res>: ResourceMetricsView
    where
        Self: 'res;

    /// The `ResourceMetrics` iterator type associated with this implementation.
    type ResourceMetricsIter<'res>: Iterator<Item = Self::ResourceMetrics<'res>>
    where
        Self: 'res;

    /// Iterator yielding borrowed references to ResourceMetrics wrappers.
    fn resources(&self) -> Self::ResourceMetricsIter<'_>;
}

/// View for ResourceMetrics
pub trait ResourceMetricsView {
    /// The `ResourceView` trait associated with this impl of the `ResourceMetricsView` trait.
    type Resource<'res>: ResourceView
    where
        Self: 'res;

    /// The `ScopeMetricsView` trait associated with this impl of the `ResourceMetricsView`.
    type ScopeMetrics<'scp>: ScopeMetricsView
    where
        Self: 'scp;

    /// The `ScopeMetrics` iterator type associated with this implementation.
    type ScopesIter<'scp>: Iterator<Item = Self::ScopeMetrics<'scp>>
    where
        Self: 'scp;

    /// Access the resource
    fn resource(&self) -> Option<Self::Resource<'_>>;

    /// Iterator yielding the `ScopeMetrics` that originate from this Resource.
    fn scopes(&self) -> Self::ScopesIter<'_>;

    /// The schema URL for the resource.
    fn schema_url(&self) -> Str<'_>;
}

/// View for ScopeMetrics
pub trait ScopeMetricsView {
    /// The `InstrumentationScopeView` trait associated with this implementation.
    type Scope<'scp>: InstrumentationScopeView
    where
        Self: 'scp;

    /// The `MetricsView` trait associated with this implementation.
    type Metric<'met>: MetricView
    where
        Self: 'met;

    /// The associated iterator type for this impl of the trait.
    type MetricIter<'met>: Iterator<Item = Self::Metric<'met>>
    where
        Self: 'met;

    /// Access the instrumentation scope for the metrics contained in this scope.
    fn scope(&self) -> Option<Self::Scope<'_>>;

    /// Iterator yielding the `Metrics` that originate from this resource.
    fn metrics(&self) -> Self::MetricIter<'_>;

    /// The schema URL for the resource.
    fn schema_url(&self) -> Str<'_>;
}

/// View for Metric
pub trait MetricView {
    /// The `DataView` type associated with this impl of the `MetricView` trait.
    type Data<'dat>: DataView<'dat>
    where
        Self: 'dat;

    /// The `AttributeView` type associated with this implementation.
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// Iterator type for yielding attributes.
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// Access the metric's name
    fn name(&self) -> Str<'_>;

    /// Access the metric's description
    fn description(&self) -> Str<'_>;

    /// Access the metric's unit
    fn unit(&self) -> Str<'_>;

    /// Access the metric's data
    fn data(&self) -> Option<Self::Data<'_>>;

    /// Access the metric's attributes
    fn metadata(&self) -> Self::AttributeIter<'_>;
}

/// Enum representing the type of some DataType
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    /// Gauge represents the type of a scalar metric that always exports the "current value" for
    /// every data point.
    Gauge = 5,

    /// Sum represents the type of a scalar metric that is calculated as a sum of all reported
    /// measurements over a time interval.
    Sum = 7,

    /// Histogram represents the type of a metric that is calculated by aggregating as a Histogram
    /// of all reported measurements over a time interval.
    Histogram = 9,

    /// ExponentialHistogram represents the type of a metric that is calculated by aggregating as a
    /// ExponentialHistogram of all reported double measurements over a time interval.
    ExponentialHistogram = 10,

    /// Summary metric data are used to convey quantile summaries.
    Summary = 11,
}

impl From<&proto::metric::Data> for DataType {
    fn from(value: &proto::metric::Data) -> Self {
        use proto::metric::Data::*;
        match value {
            Gauge(_) => DataType::Gauge,
            Sum(_) => DataType::Sum,
            Histogram(_) => DataType::Histogram,
            ExponentialHistogram(_) => DataType::ExponentialHistogram,
            Summary(_) => DataType::Summary,
        }
    }
}

/// View for Data
pub trait DataView<'val> {
    /// The `NumberDataPointView` type associated with this implementation.
    type NumberDataPoint<'dp>: NumberDataPointView
    where
        Self: 'dp;

    /// An iterator type that yields instances of Self::NumberDataPoint.
    type NumberDataPointIter<'dp>: Iterator<Item = Self::NumberDataPoint<'dp>>
    where
        Self: 'dp;

    /// A type that wraps references to `Gauge`
    type Gauge<'gauge>: GaugeView
    where
        Self: 'gauge;

    /// A type that wraps references to `Sum`
    type Sum<'sum>: SumView
    where
        Self: 'sum;

    /// A type that wraps references to `Histogram`
    type Histogram<'histogram>: HistogramView
    where
        Self: 'histogram;

    /// A type that wraps references to `ExponentialHistogram`
    type ExponentialHistogram<'exp>: ExponentialHistogramView
    where
        Self: 'exp;

    /// A type that wraps references to `Summary`
    type Summary<'summary>: SummaryView
    where
        Self: 'summary;

    /// The type of this Data element
    fn value_type(&self) -> DataType;

    /// Get the Gauge value
    fn as_gauge(&self) -> Option<Self::Gauge<'_>>;

    /// Get the Sum value
    fn as_sum(&self) -> Option<Self::Sum<'_>>;

    /// Get the Histogram value
    fn as_histogram(&self) -> Option<Self::Histogram<'_>>;

    /// Get the ExpoenentialHistogram value
    fn as_exponential_histogram(&self) -> Option<Self::ExponentialHistogram<'_>>;

    /// Get the Summary value
    fn as_summary(&self) -> Option<Self::Summary<'_>>;
}

/// View for Gauge
pub trait GaugeView {
    /// The `NumberDataPointView` type associated with this implementation.
    type NumberDataPoint<'dp>: NumberDataPointView
    where
        Self: 'dp;

    /// An iterator type that yields instances of Self::NumberDataPoint.
    type NumberDataPointIter<'dp>: Iterator<Item = Self::NumberDataPoint<'dp>>
    where
        Self: 'dp;

    /// Access the Gauge's data points
    fn data_points(&self) -> Self::NumberDataPointIter<'_>;
}

/// View for Sum
pub trait SumView {
    /// The `NumberDataPointView` type associated with this implementation.
    type NumberDataPoint<'dp>: NumberDataPointView
    where
        Self: 'dp;

    /// An iterator type that yields instances of Self::NumberDataPoint.
    type NumberDataPointIter<'dp>: Iterator<Item = Self::NumberDataPoint<'dp>>
    where
        Self: 'dp;

    /// Access the Sum's data points
    fn data_points(&self) -> Self::NumberDataPointIter<'_>;

    /// Access the Sum's aggregation temporality
    fn aggregation_temporality(&self) -> AggregationTemporality;

    /// Access the Sum's monotonicity
    fn is_monotonic(&self) -> bool;
}

/// View for NumberDataPoint
pub trait NumberDataPointView {
    /// The `AttributeView` type associated with this implementation.
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// Iterator type for yielding attributes.
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// The `ExemplarView` type associated with this implementation.
    type Exemplar<'ex>: ExemplarView
    where
        Self: 'ex;

    /// Iteratory type for yielding Exemplar wrappers.
    type ExemplarIter<'ex>: Iterator<Item = Self::Exemplar<'ex>>
    where
        Self: 'ex;

    /// Access the start time
    fn start_time_unix_nano(&self) -> u64;

    /// Access the time
    fn time_unix_nano(&self) -> u64;

    /// Access data
    fn value(&self) -> Option<Value>;

    /// Access the point's attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access the point's exemplars
    fn exemplars(&self) -> Self::ExemplarIter<'_>;

    /// Get the flags
    fn flags(&self) -> DataPointFlags;
}

/// The value of the measurement that was recorded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    /// A double value
    Double(f64),

    /// An integer value
    Integer(i64),
}

impl Value {
    /// Fetch a double option.
    #[must_use]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            Self::Double(x) => Some(*x),
            _ => None,
        }
    }

    /// Fetch an integer option.
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(x) => Some(*x),
            _ => None,
        }
    }
}

impl From<&proto::exemplar::Value> for Value {
    fn from(value: &proto::exemplar::Value) -> Self {
        use proto::exemplar::Value::*;
        match value {
            AsDouble(d) => Value::Double(*d),
            AsInt(i) => Value::Integer(*i),
        }
    }
}

impl From<&proto::number_data_point::Value> for Value {
    fn from(value: &proto::number_data_point::Value) -> Self {
        use proto::number_data_point::Value::*;
        match value {
            AsDouble(d) => Value::Double(*d),
            AsInt(i) => Value::Integer(*i),
        }
    }
}

/// View for Exemplar
pub trait ExemplarView {
    /// The `AttributeView` type associated with this implementation.
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// Iterator type for yielding attributes.
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// Get the filtered attributes
    fn filtered_attributes(&self) -> Self::AttributeIter<'_>;

    /// Access the time
    fn time_unix_nano(&self) -> u64;

    /// Access data
    fn value(&self) -> Option<Value>;

    /// Access the span ID. Should return None if the underlying span ID is missing or if the
    /// backend representation of the span ID is invalid. Invalid span IDs include all-0s or a
    /// span ID or incorrect length (length != 8)
    fn span_id(&self) -> Option<&SpanId>;

    /// Access the trace ID. Should return None if the underlying trace ID is missing, or if the
    /// backend representation of the trace ID is invalid. Invalid trace IDs include all-0s or a
    /// trace ID of incorrect length (length != 16)
    fn trace_id(&self) -> Option<&TraceId>;
}

/// View for Summary
pub trait SummaryView {
    /// The `SummaryDataPointView` type associated with this implementation.
    type SummaryDataPoint<'dp>: SummaryDataPointView
    where
        Self: 'dp;

    /// Iterator type for yielding data points.
    type SummaryDataPointIter<'dp>: Iterator<Item = Self::SummaryDataPoint<'dp>>
    where
        Self: 'dp;

    /// Access the Summary's data points
    fn data_points(&self) -> Self::SummaryDataPointIter<'_>;
}

/// View for SummaryDataPoint
pub trait SummaryDataPointView {
    /// The `AttributeView` type associated with this implementation.
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// Iterator type for yielding attributes.
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// The `ValueAtQuantileView` type associated with this implementation.
    type ValueAtQuantile<'vaq>: ValueAtQuantileView
    where
        Self: 'vaq;

    /// Iterator type for yielding ValueAtQuantiles.
    type ValueAtQuantileIter<'vaq>: Iterator<Item = Self::ValueAtQuantile<'vaq>>
    where
        Self: 'vaq;

    /// Get the attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access the start time
    fn start_time_unix_nano(&self) -> u64;

    /// Access the time
    fn time_unix_nano(&self) -> u64;

    /// Get the count
    fn count(&self) -> u64;

    /// Get the sum
    fn sum(&self) -> f64;

    /// Get quantile values
    fn quantile_values(&self) -> Self::ValueAtQuantileIter<'_>;

    /// Get the flags
    fn flags(&self) -> DataPointFlags;
}

/// View for ValueAtQuantile
pub trait ValueAtQuantileView {
    /// The quantile of a distribution. Must be in the interval [0.0, 1.0].
    fn quantile(&self) -> f64;

    /// The value at the given quantile of a distribution.
    ///
    /// Quantile values must NOT be negative.
    fn value(&self) -> f64;
}

/// DataPointFlags
pub struct DataPointFlags(u32);

impl DataPointFlags {
    /// Make a new instance
    #[must_use]
    pub fn new(flags: u32) -> DataPointFlags {
        DataPointFlags(flags)
    }

    /// This DataPoint is valid but has no recorded value. This value SHOULD be used to reflect
    /// explicitly missing data in a series, as for an equivalent to the Prometheus "staleness
    /// marker".
    #[must_use]
    pub fn no_recorded_value(&self) -> bool {
        self.0 & 1 != 0
    }

    /// Return the raw flags value.
    #[must_use]
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

// FIXME: the `DataPointFlags` enum specifies `repr(i32)` but the field definition in the message
// types uses `u32`.

/// AggregationTemporality defines how a metric aggregator reports aggregated values. It describes
/// how those values relate to the time interval over which they are aggregated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregationTemporality {
    /// UNSPECIFIED is the default AggregationTemporality, it MUST not be used.
    Unspecified = 0,

    /// DELTA is an AggregationTemporality for a metric aggregator which reports changes since last
    /// report time. Successive metrics contain aggregation of values from continuous and
    /// non-overlapping intervals.
    Delta = 1,

    /// CUMULATIVE is an AggregationTemporality for a metric aggregator which reports changes since
    /// a fixed start time.
    Cumulative = 2,
}

impl From<proto::AggregationTemporality> for AggregationTemporality {
    fn from(value: proto::AggregationTemporality) -> Self {
        use AggregationTemporality::*;
        match value {
            proto::AggregationTemporality::Unspecified => Unspecified,
            proto::AggregationTemporality::Delta => Delta,
            proto::AggregationTemporality::Cumulative => Cumulative,
        }
    }
}

/// View for Histogram
pub trait HistogramView {
    /// The `HistogramDataPointView` type associated with this implementation.
    type HistogramDataPoint<'dp>: HistogramDataPointView
    where
        Self: 'dp;

    /// Iterator type yielding HistogramDataPoints.
    type HistogramDataPointIter<'dp>: Iterator<Item = Self::HistogramDataPoint<'dp>>
    where
        Self: 'dp;

    /// Access the Histogram's data points
    fn data_points(&self) -> Self::HistogramDataPointIter<'_>;

    /// Access the Histogram's aggregation temporality
    fn aggregation_temporality(&self) -> AggregationTemporality;
}

/// View for HistogramDataPoint
pub trait HistogramDataPointView {
    /// The `AttributeView` type associated with this implementation.
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// Iterator type for yielding attributes.
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// Iterator type for yielding bucket counts.
    type BucketCountIter<'bc>: Iterator<Item = &'bc u64>
    where
        Self: 'bc;

    /// Iterator type for yielding explicit bounds.
    type ExplicitBoundsIter<'eb>: Iterator<Item = &'eb f64>
    where
        Self: 'eb;

    /// The `ExemplarView` type associated with this implementation.
    type Exemplar<'ex>: ExemplarView
    where
        Self: 'ex;

    /// Iteratory type for yielding Exemplar wrappers.
    type ExemplarIter<'ex>: Iterator<Item = Self::Exemplar<'ex>>
    where
        Self: 'ex;

    /// Get the attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access the start time
    fn start_time_unix_nano(&self) -> u64;

    /// Access the time
    fn time_unix_nano(&self) -> u64;

    /// Get the count
    fn count(&self) -> u64;

    /// Get the sum
    fn sum(&self) -> Option<f64>;

    /// Get the count values of the histogram for each bucket
    fn bucket_counts(&self) -> Self::BucketCountIter<'_>;

    /// Get buckets with explicitly defined bounds for values
    fn explicit_bounds(&self) -> Self::ExplicitBoundsIter<'_>;

    /// Access the point's exemplars
    fn exemplars(&self) -> Self::ExemplarIter<'_>;

    /// Get the flags
    fn flags(&self) -> DataPointFlags;

    /// Get the minimum value over (start_time, end_time)
    fn min(&self) -> Option<f64>;

    /// Get the maximum value over (start_time, end_time)
    fn max(&self) -> Option<f64>;
}

/// View for ExponentialHistogram
pub trait ExponentialHistogramView {
    /// The `ExponentialHistogramDataPointView` type associated with this implementation.
    type ExponentialHistogramDataPoint<'edp>: ExponentialHistogramDataPointView
    where
        Self: 'edp;

    /// Iterator type for yielding data points.
    type ExponentialHistogramDataPointIter<'edp>: Iterator<
        Item = Self::ExponentialHistogramDataPoint<'edp>,
    >
    where
        Self: 'edp;

    /// Access the Histogram's data points
    fn data_points(&self) -> Self::ExponentialHistogramDataPointIter<'_>;

    /// Access the Histogram's aggregation temporality
    fn aggregation_temporality(&self) -> AggregationTemporality;
}

/// View for ExponentialHistogramDataPoint
pub trait ExponentialHistogramDataPointView {
    /// The `AttributeView` type associated with this implementation.
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// Iterator type for yielding attributes.
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// The `BucketsView` type associated with this implementation.
    type Buckets<'b>: BucketsView
    where
        Self: 'b;

    /// The `ExemplarView` type associated with this implementation.
    type Exemplar<'ex>: ExemplarView
    where
        Self: 'ex;

    /// Iteratory type for yielding Exemplar wrappers.
    type ExemplarIter<'ex>: Iterator<Item = Self::Exemplar<'ex>>
    where
        Self: 'ex;

    /// Get the attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access the start time
    fn start_time_unix_nano(&self) -> u64;

    /// Access the time
    fn time_unix_nano(&self) -> u64;

    /// Get the count
    fn count(&self) -> u64;

    /// Get the sum
    fn sum(&self) -> Option<f64>;

    /// Get the histogram resolution
    fn scale(&self) -> i32;

    /// Get the count of values that are either exactly zero of within the region considered zero by
    /// the instrumentation.
    fn zero_count(&self) -> u64;

    /// Get the positive range of exponential bucket counts.
    fn positive(&self) -> Option<Self::Buckets<'_>>;

    /// Get the negative range of exponential bucket counts.
    fn negative(&self) -> Option<Self::Buckets<'_>>;

    /// Get the flags
    fn flags(&self) -> DataPointFlags;

    /// Access the point's exemplars
    fn exemplars(&self) -> Self::ExemplarIter<'_>;

    /// Get the minimum value over (start_time, end_time)
    fn min(&self) -> Option<f64>;

    /// Get the maximum value over (start_time, end_time)
    fn max(&self) -> Option<f64>;

    /// Get the ZeroThreshold
    fn zero_threshold(&self) -> f64;
}

/// View for Bucket
pub trait BucketsView {
    /// Iterator type for bucket counts.
    type BucketCountIter<'bc>: Iterator<Item = &'bc u64>
    where
        Self: 'bc;

    /// Get the bucket index of the first entry in the array.
    fn offset(&self) -> i32;

    /// Get the array of bucket counts.
    fn bucket_counts(&self) -> Self::BucketCountIter<'_>;
}
