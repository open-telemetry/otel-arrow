// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for the messages defined in metrics.proto

use std::cell::Cell;
use std::num::NonZeroUsize;

use crate::proto::consts::field_num::metrics::{
    EXEMPLAR_AS_DOUBLE, EXEMPLAR_AS_INT, EXEMPLAR_FILTERED_ATTRIBUTES, EXEMPLAR_SPAN_ID,
    EXEMPLAR_TIME_UNIX_NANO, EXEMPLAR_TRACE_ID, EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS,
    EXP_HISTOGRAM_BUCKET_OFFSET, EXP_HISTOGRAM_DP_ATTRIBUTES, EXP_HISTOGRAM_DP_COUNT,
    EXP_HISTOGRAM_DP_EXEMPLARS, EXP_HISTOGRAM_DP_FLAGS, EXP_HISTOGRAM_DP_MAX, EXP_HISTOGRAM_DP_MIN,
    EXP_HISTOGRAM_DP_NEGATIVE, EXP_HISTOGRAM_DP_POSITIVE, EXP_HISTOGRAM_DP_SCALE,
    EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO, EXP_HISTOGRAM_DP_SUM, EXP_HISTOGRAM_DP_TIME_UNIX_NANO,
    EXP_HISTOGRAM_DP_ZERO_COUNT, EXP_HISTOGRAM_DP_ZERO_THRESHOLD,
    EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY, EXPONENTIAL_HISTOGRAM_DATA_POINTS,
    GAUGE_DATA_POINTS, HISTOGRAM_AGGREGATION_TEMPORALITY, HISTOGRAM_DATA_POINTS,
    HISTOGRAM_DP_ATTRIBUTES, HISTOGRAM_DP_BUCKET_COUNTS, HISTOGRAM_DP_COUNT,
    HISTOGRAM_DP_EXEMPLARS, HISTOGRAM_DP_EXPLICIT_BOUNDS, HISTOGRAM_DP_FLAGS, HISTOGRAM_DP_MAX,
    HISTOGRAM_DP_MIN, HISTOGRAM_DP_START_TIME_UNIX_NANO, HISTOGRAM_DP_SUM,
    HISTOGRAM_DP_TIME_UNIX_NANO, METRIC_DESCRIPTION, METRIC_EXPONENTIAL_HISTOGRAM, METRIC_GAUGE,
    METRIC_HISTOGRAM, METRIC_METADATA, METRIC_NAME, METRIC_SUM, METRIC_SUMMARY, METRIC_UNIT,
    METRICS_DATA_RESOURCE_METRICS, NUMBER_DP_AS_DOUBLE, NUMBER_DP_AS_INT, NUMBER_DP_ATTRIBUTES,
    NUMBER_DP_EXEMPLARS, NUMBER_DP_FLAGS, NUMBER_DP_START_TIME_UNIX_NANO, NUMBER_DP_TIME_UNIX_NANO,
    RESOURCE_METRICS_RESOURCE, RESOURCE_METRICS_SCHEMA_URL, RESOURCE_METRICS_SCOPE_METRICS,
    SCOPE_METRICS_METRICS, SCOPE_METRICS_SCHEMA_URL, SCOPE_METRICS_SCOPE,
    SUM_AGGREGATION_TEMPORALITY, SUM_DATA_POINTS, SUM_IS_MONOTONIC, SUMMARY_DATA_POINTS,
    SUMMARY_DP_ATTRIBUTES, SUMMARY_DP_COUNT, SUMMARY_DP_FLAGS, SUMMARY_DP_QUANTILE_VALUES,
    SUMMARY_DP_START_TIME_UNIX_NANO, SUMMARY_DP_SUM, SUMMARY_DP_TIME_UNIX_NANO,
    VALUE_AT_QUANTILE_QUANTILE, VALUE_AT_QUANTILE_VALUE,
};
use crate::proto::consts::wire_types;
use crate::schema::{SpanId, TraceId};
use crate::views::common::Str;
use crate::views::metrics::{
    AggregationTemporality, BucketsView, DataPointFlags, DataType, DataView, ExemplarView,
    ExponentialHistogramDataPointView, ExponentialHistogramView, GaugeView, HistogramDataPointView,
    HistogramView, MetricView, MetricsView, NumberDataPointView, ResourceMetricsView,
    ScopeMetricsView, SumView, SummaryDataPointView, SummaryView, Value, ValueAtQuantileView,
};
use crate::views::otlp::bytes::common::{KeyValueIter, RawInstrumentationScope, RawKeyValue};
use crate::views::otlp::bytes::decode::{
    FieldRanges, ProtoBytesParser, RepeatedFieldEncodings, RepeatedFieldProtoBytesParser,
    RepeatedFixed64Iter, RepeatedVarintIter, decode_sint32, from_option_nonzero_range_to_primitive,
    read_len_delim, read_varint, to_nonzero_range,
};
use crate::views::otlp::bytes::resource::RawResource;

/// Implementation of [`MetricView`] backed by protobuf serialized `MetricsData` message
pub struct RawMetricsData<'a> {
    buf: &'a [u8],
}

impl<'a> RawMetricsData<'a> {
    /// Create a new [`RawMetricsData`]
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

/// Implementation of [`ResourceMetricsView`] backed by protobuf serialized `ResourceMetrics` message
pub struct RawResourceMetrics<'a> {
    byte_parser: ProtoBytesParser<'a, ResourceMetricsFieldRanges>,
}

/// Known field offsets within byte buffer for fields in `ResourceMetrics` message
pub struct ResourceMetricsFieldRanges {
    resource: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    schema_url: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_scope_metrics: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ResourceMetricsFieldRanges {
    fn new() -> Self {
        Self {
            resource: Cell::new(None),
            schema_url: Cell::new(None),
            first_scope_metrics: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            RESOURCE_METRICS_RESOURCE => self.resource.get(),
            RESOURCE_METRICS_SCHEMA_URL => self.schema_url.get(),
            RESOURCE_METRICS_SCOPE_METRICS => self.first_scope_metrics.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        if wire_type == wire_types::LEN {
            match field_num {
                RESOURCE_METRICS_RESOURCE => self.resource.set(Some(range)),
                RESOURCE_METRICS_SCHEMA_URL => self.schema_url.set(Some(range)),
                RESOURCE_METRICS_SCOPE_METRICS => {
                    if self.first_scope_metrics.get().is_none() {
                        self.first_scope_metrics.set(Some(range))
                    }
                }
                _ => { /* ignore */ }
            }
        }
    }
}

/// Implementation of [`ScopeMetricsView`] backed by protobuf serialized `ScopeMetrics` message
pub struct RawScopeMetrics<'a> {
    byte_parser: ProtoBytesParser<'a, ScopeMetricsFieldRanges>,
}

/// Known field offsets within byte buffer for fields in the `ScopeMetrics` message
pub struct ScopeMetricsFieldRanges {
    scope: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    schema_url: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_metric: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ScopeMetricsFieldRanges {
    fn new() -> Self {
        Self {
            scope: Cell::new(None),
            schema_url: Cell::new(None),
            first_metric: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SCOPE_METRICS_SCOPE => self.scope.get(),
            SCOPE_METRICS_SCHEMA_URL => self.schema_url.get(),
            SCOPE_METRICS_METRICS => self.first_metric.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        if wire_type == wire_types::LEN {
            match field_num {
                SCOPE_METRICS_SCOPE => self.scope.set(Some(range)),
                SCOPE_METRICS_SCHEMA_URL => self.schema_url.set(Some(range)),
                SCOPE_METRICS_METRICS => {
                    if self.first_metric.get().is_none() {
                        self.first_metric.set(Some(range))
                    }
                }
                _ => { /* ignore */ }
            }
        }
    }
}

/// Implementation of [`MetricView`] backed by protobuf serialized `Metric` message
pub struct RawMetric<'a> {
    byte_parser: ProtoBytesParser<'a, MetricFieldRanges>,
}

/// Known offsets within byte buffer for fields in the `Metric` message
pub struct MetricFieldRanges {
    name: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    description: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    unit: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_metadata: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,

    // since this is a oneof, we also keep the field_num alongside the range.
    // That way when `get_field_range` is called, we can avoid returning the
    // data range for the wrong variant of the oneof field
    data: Cell<Option<((NonZeroUsize, NonZeroUsize), u64)>>,
}

impl FieldRanges for MetricFieldRanges {
    fn new() -> Self {
        Self {
            name: Cell::new(None),
            description: Cell::new(None),
            unit: Cell::new(None),
            data: Cell::new(None),
            first_metadata: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            METRIC_NAME => self.name.get(),
            METRIC_DESCRIPTION => self.description.get(),
            METRIC_UNIT => self.unit.get(),
            METRIC_GAUGE
            | METRIC_SUM
            | METRIC_HISTOGRAM
            | METRIC_EXPONENTIAL_HISTOGRAM
            | METRIC_SUMMARY => {
                let (range, actual_field_num) = self.data.get()?;
                (field_num == actual_field_num).then_some(range)
            }
            METRIC_METADATA => self.first_metadata.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        if wire_type == wire_types::LEN {
            let range = match to_nonzero_range(start, end) {
                Some(range) => range,
                None => return,
            };

            match field_num {
                METRIC_NAME => self.name.set(Some(range)),
                METRIC_DESCRIPTION => self.description.set(Some(range)),
                METRIC_UNIT => self.unit.set(Some(range)),
                METRIC_GAUGE
                | METRIC_SUM
                | METRIC_HISTOGRAM
                | METRIC_EXPONENTIAL_HISTOGRAM
                | METRIC_SUMMARY => self.data.set(Some((range, field_num))),
                METRIC_METADATA => {
                    if self.first_metadata.get().is_none() {
                        self.first_metadata.set(Some(range))
                    }
                }
                _ => { /* ignore */ }
            }
        }
    }
}

/// Implementation of [`DataView`] backed by one of the buffers in Metric's "data" oneof field.
pub struct RawData<'a> {
    field_num: u64,
    buf: &'a [u8],
}

/// Implementation of [`GaugeView`]
pub struct RawGauge<'a> {
    byte_parser: ProtoBytesParser<'a, GaugeFieldRanges>,
}

/// Known field ranges for fields on Gauge messages
pub struct GaugeFieldRanges {
    first_data_point: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for GaugeFieldRanges {
    fn new() -> Self {
        Self {
            first_data_point: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            GAUGE_DATA_POINTS => self.first_data_point.get(),
            _ => return None,
        };
        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        if wire_type == wire_types::LEN {
            let range = match to_nonzero_range(start, end) {
                Some(range) => range,
                None => return,
            };

            match field_num {
                GAUGE_DATA_POINTS => {
                    if self.first_data_point.get().is_none() {
                        self.first_data_point.set(Some(range))
                    }
                }
                _ => { /* ignore */ }
            }
        }
    }
}

/// Implementation of [`SumView`] backed by buffer containing proto serialized `Sum` Message
pub struct RawSum<'a> {
    byte_parser: ProtoBytesParser<'a, SumFieldRanges>,
}

/// Known field ranges for fields on `Sum` message
pub struct SumFieldRanges {
    is_monotonic: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    aggregation_temporality: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_data_point: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for SumFieldRanges {
    fn new() -> Self {
        Self {
            is_monotonic: Cell::new(None),
            aggregation_temporality: Cell::new(None),
            first_data_point: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SUM_AGGREGATION_TEMPORALITY => self.aggregation_temporality.get(),
            SUM_IS_MONOTONIC => self.is_monotonic.get(),
            SUM_DATA_POINTS => self.first_data_point.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            SUM_AGGREGATION_TEMPORALITY => {
                if wire_type == wire_types::VARINT {
                    self.aggregation_temporality.set(Some(range))
                }
            }
            SUM_IS_MONOTONIC => {
                if wire_type == wire_types::VARINT {
                    self.is_monotonic.set(Some(range))
                }
            }
            SUM_DATA_POINTS => {
                if wire_type == wire_types::LEN && self.first_data_point.get().is_none() {
                    self.first_data_point.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`HistogramView`] backed by byte buffer containing proto serialized
/// `Histogram` message
pub struct RawHistogram<'a> {
    byte_parser: ProtoBytesParser<'a, HistogramFieldRanges>,
}

/// Known field ranges for fields on `Histogram` message
pub struct HistogramFieldRanges {
    aggregation_temporality: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_data_point: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for HistogramFieldRanges {
    fn new() -> Self {
        Self {
            aggregation_temporality: Cell::new(None),
            first_data_point: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            HISTOGRAM_AGGREGATION_TEMPORALITY => self.aggregation_temporality.get(),
            HISTOGRAM_DATA_POINTS => self.first_data_point.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            HISTOGRAM_AGGREGATION_TEMPORALITY => {
                if wire_type == wire_types::VARINT {
                    self.aggregation_temporality.set(Some(range))
                }
            }
            HISTOGRAM_DATA_POINTS => {
                if wire_type == wire_types::LEN && self.first_data_point.get().is_none() {
                    self.first_data_point.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`ExponentialHistogramView`] backed by byte buffer containing proto
/// serialized `ExponentialHistogram` message
pub struct RawExpHistogram<'a> {
    byte_parser: ProtoBytesParser<'a, ExpHistogramFieldRanges>,
}

/// Known field ranges for fields on `ExponentialHistogram` message
struct ExpHistogramFieldRanges {
    aggregation_temporality: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_data_point: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ExpHistogramFieldRanges {
    fn new() -> Self {
        Self {
            aggregation_temporality: Cell::new(None),
            first_data_point: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY => self.aggregation_temporality.get(),
            EXPONENTIAL_HISTOGRAM_DATA_POINTS => self.first_data_point.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY => {
                if wire_type == wire_types::VARINT {
                    self.aggregation_temporality.set(Some(range))
                }
            }
            EXPONENTIAL_HISTOGRAM_DATA_POINTS => {
                if wire_type == wire_types::LEN && self.first_data_point.get().is_none() {
                    self.first_data_point.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`SummaryView`] backed by byte buffer containing Summary message
pub struct RawSummary<'a> {
    byte_parser: ProtoBytesParser<'a, SummaryFieldRanges>,
}

/// Known field ranges on Summary message
#[derive(Default)]
pub struct SummaryFieldRanges {
    first_data_point: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for SummaryFieldRanges {
    fn new() -> Self {
        Self::default()
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SUMMARY_DATA_POINTS => self.first_data_point.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            SUMMARY_DATA_POINTS => {
                if wire_type == wire_types::LEN && self.first_data_point.get().is_none() {
                    self.first_data_point.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`NumberDataPointView`] backed by buffer containing proto serialized
/// NumberDataPoint message
pub struct RawNumberDataPoint<'a> {
    byte_parser: ProtoBytesParser<'a, NumberDataPointFieldRanges>,
}

/// Known field ranges for fields in proto serialized NumberDataPoint message
pub struct NumberDataPointFieldRanges {
    start_time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    flags: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_exemplar: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,

    // since this is a oneof, we also keep the field_num alongside the range.
    // That way when `get_field_range` is called, we can avoid returning the
    // data range for the wrong variant of the oneof field
    value: Cell<Option<((NonZeroUsize, NonZeroUsize), u64)>>,
}

impl FieldRanges for NumberDataPointFieldRanges {
    fn new() -> Self {
        Self {
            start_time_unix_nano: Cell::new(None),
            time_unix_nano: Cell::new(None),
            value: Cell::new(None),
            flags: Cell::new(None),
            first_attribute: Cell::new(None),
            first_exemplar: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            NUMBER_DP_START_TIME_UNIX_NANO => self.start_time_unix_nano.get(),
            NUMBER_DP_TIME_UNIX_NANO => self.time_unix_nano.get(),
            NUMBER_DP_FLAGS => self.flags.get(),
            NUMBER_DP_AS_DOUBLE | NUMBER_DP_AS_INT => {
                let (range, actual_field_num) = self.value.get()?;
                (actual_field_num == field_num).then_some(range)
            }
            NUMBER_DP_ATTRIBUTES => self.first_attribute.get(),
            NUMBER_DP_EXEMPLARS => self.first_exemplar.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            NUMBER_DP_START_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.start_time_unix_nano.set(Some(range))
                }
            }
            NUMBER_DP_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.time_unix_nano.set(Some(range))
                }
            }

            NUMBER_DP_AS_DOUBLE | NUMBER_DP_AS_INT => {
                if wire_type == wire_types::FIXED64 {
                    self.value.set(Some((range, field_num)));
                }
            }
            NUMBER_DP_ATTRIBUTES => {
                if wire_type == wire_types::LEN && self.first_attribute.get().is_none() {
                    self.first_attribute.set(Some(range))
                }
            }
            NUMBER_DP_EXEMPLARS => {
                if wire_type == wire_types::LEN && self.first_exemplar.get().is_none() {
                    self.first_exemplar.set(Some(range))
                }
            }

            NUMBER_DP_FLAGS => {
                if wire_type == wire_types::VARINT {
                    self.flags.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`HistogramDataPointView`] backed by buffer containing proto serialized
/// HistogramDataPoint message
pub struct RawHistogramDataPoint<'a> {
    byte_parser: ProtoBytesParser<'a, HistogramDataPointFieldRanges>,
}

/// Known field ranges for fields in proto serialized HistogramDataPoint message
#[derive(Default)]
pub struct HistogramDataPointFieldRanges {
    start_time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    count: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    sum: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    flags: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    min: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    max: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attributes: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_exemplar: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,

    // these are repeated primitive fields that may use either packed or expanded encoding
    // so we store an extra flag which determines if it's packed encoding.
    first_explicit_bounds: Cell<Option<((NonZeroUsize, NonZeroUsize), bool)>>,
    first_bucket_counts: Cell<Option<((NonZeroUsize, NonZeroUsize), bool)>>,
}

impl FieldRanges for HistogramDataPointFieldRanges {
    fn new() -> Self {
        Self::default()
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            HISTOGRAM_DP_START_TIME_UNIX_NANO => self.start_time_unix_nano.get(),
            HISTOGRAM_DP_TIME_UNIX_NANO => self.time_unix_nano.get(),
            HISTOGRAM_DP_COUNT => self.count.get(),
            HISTOGRAM_DP_SUM => self.sum.get(),
            HISTOGRAM_DP_FLAGS => self.flags.get(),
            HISTOGRAM_DP_MIN => self.min.get(),
            HISTOGRAM_DP_MAX => self.max.get(),
            HISTOGRAM_DP_ATTRIBUTES => self.first_attributes.get(),
            HISTOGRAM_DP_BUCKET_COUNTS => self.first_bucket_counts.get().map(|(range, _)| range),
            HISTOGRAM_DP_EXPLICIT_BOUNDS => {
                self.first_explicit_bounds.get().map(|(range, _)| range)
            }
            HISTOGRAM_DP_EXEMPLARS => self.first_exemplar.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            HISTOGRAM_DP_START_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.start_time_unix_nano.set(Some(range));
                }
            }
            HISTOGRAM_DP_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.time_unix_nano.set(Some(range))
                }
            }
            HISTOGRAM_DP_COUNT => {
                if wire_type == wire_types::FIXED64 {
                    self.count.set(Some(range))
                }
            }
            HISTOGRAM_DP_SUM => {
                if wire_type == wire_types::FIXED64 {
                    self.sum.set(Some(range))
                }
            }
            HISTOGRAM_DP_FLAGS => {
                if wire_type == wire_types::VARINT {
                    self.flags.set(Some(range))
                }
            }
            HISTOGRAM_DP_EXPLICIT_BOUNDS => {
                if (wire_type == wire_types::LEN || wire_type == wire_types::FIXED64)
                    && self.first_explicit_bounds.get().is_none()
                {
                    let packed = wire_type == wire_types::LEN;
                    self.first_explicit_bounds.set(Some((range, packed)));
                }
            }
            HISTOGRAM_DP_BUCKET_COUNTS => {
                if (wire_type == wire_types::LEN || wire_type == wire_types::FIXED64)
                    && self.first_bucket_counts.get().is_none()
                {
                    let packed = wire_type == wire_types::LEN;
                    self.first_bucket_counts.set(Some((range, packed)));
                }
            }
            HISTOGRAM_DP_MIN => {
                if wire_type == wire_types::FIXED64 {
                    self.min.set(Some(range))
                }
            }
            HISTOGRAM_DP_MAX => {
                if wire_type == wire_types::FIXED64 {
                    self.max.set(Some(range))
                }
            }
            HISTOGRAM_DP_ATTRIBUTES => {
                if wire_type == wire_types::LEN && self.first_attributes.get().is_none() {
                    self.first_attributes.set(Some(range))
                }
            }
            HISTOGRAM_DP_EXEMPLARS => {
                if wire_type == wire_types::LEN && self.first_exemplar.get().is_none() {
                    self.first_exemplar.set(Some(range))
                }
            }

            _ => { /* ignore */ }
        }
    }
}

impl RepeatedFieldEncodings for HistogramDataPointFieldRanges {
    fn is_packed(&self, field_num: u64) -> bool {
        match field_num {
            HISTOGRAM_DP_BUCKET_COUNTS => self.first_bucket_counts.get().map(|(_, packed)| packed),

            HISTOGRAM_DP_EXPLICIT_BOUNDS => {
                self.first_explicit_bounds.get().map(|(_, packed)| packed)
            }
            _ => None,
        }
        .unwrap_or_default()
    }
}

/// Implementation of [`ExponentialHistogramDataPointView`] backed by buffer containing proto
/// serialized ExponentialHistogramDataPoint message
pub struct RawExpHistogramDatapoint<'a> {
    byte_parser: ProtoBytesParser<'a, ExpHistogramDataPointFieldRanges>,
}

/// Known field ranges for fields in proto serialized ExponentialHistogramDataPoint message
#[derive(Default)]

pub struct ExpHistogramDataPointFieldRanges {
    start_time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    count: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    sum: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    scale: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    zero_count: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    positive: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    negative: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    flags: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    min: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    max: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    zero_threshold: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attributes: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_exemplar: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ExpHistogramDataPointFieldRanges {
    fn new() -> Self {
        Self::default()
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO => self.start_time_unix_nano.get(),
            EXP_HISTOGRAM_DP_TIME_UNIX_NANO => self.time_unix_nano.get(),
            EXP_HISTOGRAM_DP_COUNT => self.count.get(),
            EXP_HISTOGRAM_DP_SUM => self.sum.get(),
            EXP_HISTOGRAM_DP_SCALE => self.scale.get(),
            EXP_HISTOGRAM_DP_ZERO_COUNT => self.zero_count.get(),
            EXP_HISTOGRAM_DP_POSITIVE => self.positive.get(),
            EXP_HISTOGRAM_DP_NEGATIVE => self.negative.get(),
            EXP_HISTOGRAM_DP_FLAGS => self.flags.get(),
            EXP_HISTOGRAM_DP_MIN => self.min.get(),
            EXP_HISTOGRAM_DP_MAX => self.max.get(),
            EXP_HISTOGRAM_DP_ZERO_THRESHOLD => self.zero_threshold.get(),
            EXP_HISTOGRAM_DP_ATTRIBUTES => self.first_attributes.get(),
            EXP_HISTOGRAM_DP_EXEMPLARS => self.first_exemplar.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.start_time_unix_nano.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.time_unix_nano.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_COUNT => {
                if wire_type == wire_types::FIXED64 {
                    self.count.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_SUM => {
                if wire_type == wire_types::FIXED64 {
                    self.sum.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_SCALE => {
                if wire_type == wire_types::VARINT {
                    self.scale.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_ZERO_COUNT => {
                if wire_type == wire_types::FIXED64 {
                    self.zero_count.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_NEGATIVE => {
                if wire_type == wire_types::LEN {
                    self.negative.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_POSITIVE => {
                if wire_type == wire_types::LEN {
                    self.positive.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_FLAGS => {
                if wire_type == wire_types::VARINT {
                    self.flags.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_MIN => {
                if wire_type == wire_types::FIXED64 {
                    self.min.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_MAX => {
                if wire_type == wire_types::FIXED64 {
                    self.max.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_ZERO_THRESHOLD => {
                if wire_type == wire_types::FIXED64 {
                    self.zero_threshold.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_ATTRIBUTES => {
                if wire_type == wire_types::LEN && self.first_attributes.get().is_none() {
                    self.first_attributes.set(Some(range))
                }
            }
            EXP_HISTOGRAM_DP_EXEMPLARS => {
                if wire_type == wire_types::LEN && self.first_exemplar.get().is_none() {
                    self.first_exemplar.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`BucketsView`] backed by buffer containing proto serialized Buckets message
pub struct RawBuckets<'a> {
    byte_parser: ProtoBytesParser<'a, BucketsFieldRanges>,
}

/// Known field ranges for fields on `Buckets` message
#[derive(Default)]
pub struct BucketsFieldRanges {
    offset: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,

    // this is repeated primitive fields that may use either packed or expanded encoding
    // so we store an extra flag which determines if it's packed encoding.
    first_bucket_count: Cell<Option<((NonZeroUsize, NonZeroUsize), bool)>>,
}

impl FieldRanges for BucketsFieldRanges {
    fn new() -> Self {
        Self::default()
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            EXP_HISTOGRAM_BUCKET_OFFSET => self.offset.get(),
            EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS => {
                self.first_bucket_count.get().map(|(range, _)| range)
            }
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            EXP_HISTOGRAM_BUCKET_OFFSET => {
                if wire_type == wire_types::VARINT {
                    self.offset.set(Some(range));
                }
            }
            EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS => {
                if (wire_type == wire_types::LEN || wire_type == wire_types::VARINT)
                    && self.first_bucket_count.get().is_none()
                {
                    let packed = wire_type == wire_types::LEN;
                    self.first_bucket_count.set(Some((range, packed)))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

impl RepeatedFieldEncodings for BucketsFieldRanges {
    fn is_packed(&self, field_num: u64) -> bool {
        match field_num {
            EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS => {
                self.first_bucket_count.get().map(|(_, packed)| packed)
            }
            _ => None,
        }
        .unwrap_or_default()
    }
}

/// Implementation of [`SummaryDataPointView`] backed by buffer containing proto serialized
/// SummaryDataPoint message
pub struct RawSummaryDataPoint<'a> {
    byte_parser: ProtoBytesParser<'a, SummaryDataPointFieldRanges>,
}

/// Known field ranges for for fields in `SummaryDataPoint` message
#[derive(Default)]
pub struct SummaryDataPointFieldRanges {
    start_time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    count: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    sum: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    flags: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_quantile_value: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for SummaryDataPointFieldRanges {
    fn new() -> Self {
        Self::default()
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SUMMARY_DP_START_TIME_UNIX_NANO => self.start_time_unix_nano.get(),
            SUMMARY_DP_TIME_UNIX_NANO => self.time_unix_nano.get(),
            SUMMARY_DP_COUNT => self.count.get(),
            SUMMARY_DP_SUM => self.sum.get(),
            SUMMARY_DP_FLAGS => self.flags.get(),
            SUMMARY_DP_ATTRIBUTES => self.first_attribute.get(),
            SUMMARY_DP_QUANTILE_VALUES => self.first_quantile_value.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            SUMMARY_DP_START_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.start_time_unix_nano.set(Some(range))
                }
            }
            SUMMARY_DP_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.time_unix_nano.set(Some(range))
                }
            }
            SUMMARY_DP_COUNT => {
                if wire_type == wire_types::FIXED64 {
                    self.count.set(Some(range))
                }
            }
            SUMMARY_DP_SUM => {
                if wire_type == wire_types::FIXED64 {
                    self.sum.set(Some(range))
                }
            }
            SUMMARY_DP_FLAGS => {
                if wire_type == wire_types::VARINT {
                    self.flags.set(Some(range))
                }
            }
            SUMMARY_DP_ATTRIBUTES => {
                if wire_type == wire_types::LEN && self.first_attribute.get().is_none() {
                    self.first_attribute.set(Some(range))
                }
            }
            SUMMARY_DP_QUANTILE_VALUES => {
                if wire_type == wire_types::LEN && self.first_quantile_value.get().is_none() {
                    self.first_quantile_value.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`ValueAtQuantileView`] backed by byte buffer containing proto serialized
/// `ValueAtQuantile` message
pub struct RawValueAtQuantile<'a> {
    byte_parser: ProtoBytesParser<'a, ValueAtQuantileFieldRanges>,
}

/// Known field ranges in buffer containing proto serialized ValueAtQuantile message
#[derive(Default)]
pub struct ValueAtQuantileFieldRanges {
    quantile: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    value: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ValueAtQuantileFieldRanges {
    fn new() -> Self {
        Self::default()
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            VALUE_AT_QUANTILE_QUANTILE => self.quantile.get(),
            VALUE_AT_QUANTILE_VALUE => self.value.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        if wire_type == wire_types::FIXED64 {
            let range = match to_nonzero_range(start, end) {
                Some(range) => range,
                None => return,
            };

            match field_num {
                VALUE_AT_QUANTILE_QUANTILE => self.quantile.set(Some(range)),
                VALUE_AT_QUANTILE_VALUE => self.value.set(Some(range)),
                _ => { /* ignore */ }
            }
        }
    }
}

/// Implementation of [`ExemplarView`] backed by buf containing proto serialized Exemplar message
pub struct RawExemplar<'a> {
    byte_parser: ProtoBytesParser<'a, ExemplarFieldRanges>,
}

/// Known field ranges in buffer containing proto serialized Exemplar message
#[derive(Default)]
pub struct ExemplarFieldRanges {
    time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_filtered_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    span_id: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    trace_id: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,

    // since this is a oneof, we also keep the field_num alongside the range.
    // That way when `get_field_range` is called, we can avoid returning the
    // data range for the wrong variant of the oneof field
    value: Cell<Option<((NonZeroUsize, NonZeroUsize), u64)>>,
}

impl FieldRanges for ExemplarFieldRanges {
    fn new() -> Self {
        Self::default()
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            EXEMPLAR_FILTERED_ATTRIBUTES => self.first_filtered_attribute.get(),
            EXEMPLAR_TIME_UNIX_NANO => self.time_unix_nano.get(),
            EXEMPLAR_SPAN_ID => self.span_id.get(),
            EXEMPLAR_TRACE_ID => self.trace_id.get(),
            EXEMPLAR_AS_DOUBLE | EXEMPLAR_AS_INT => {
                let (range, actual_field_num) = self.value.get()?;
                (actual_field_num == field_num).then_some(range)
            }
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            EXEMPLAR_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.time_unix_nano.set(Some(range))
                }
            }
            EXEMPLAR_SPAN_ID => {
                if wire_type == wire_types::LEN {
                    self.span_id.set(Some(range))
                }
            }
            EXEMPLAR_TRACE_ID => {
                if wire_type == wire_types::LEN {
                    self.trace_id.set(Some(range))
                }
            }
            EXEMPLAR_AS_DOUBLE | EXEMPLAR_AS_INT => {
                if wire_type == wire_types::FIXED64 {
                    self.value.set(Some((range, field_num)));
                }
            }
            EXEMPLAR_FILTERED_ATTRIBUTES => {
                if wire_type == wire_types::LEN && self.first_filtered_attribute.get().is_none() {
                    self.first_filtered_attribute.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

/// Iterator of ResourceMetrics - produces implementation of [`ResourceMetricsView`] from byte
/// array containing serialized `MetricsData` message
pub struct ResourceMetricsIter<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for ResourceMetricsIter<'a> {
    type Item = RawResourceMetrics<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buf.len() {
            let (tag, next_pos) = read_varint(self.buf, self.pos)?;
            self.pos = next_pos;
            let field = tag >> 3;
            let wire_type = tag & 7;
            if field == METRICS_DATA_RESOURCE_METRICS && wire_type == wire_types::LEN {
                let (slice, next_pos) = read_len_delim(self.buf, self.pos)?;
                self.pos = next_pos;
                return Some(RawResourceMetrics {
                    byte_parser: ProtoBytesParser::new(slice),
                });
            }
        }

        None
    }
}

/// Iterator of ScopeMetrics - produces an implementation of [`ScopeMetricsView`] from byte array
/// containing a serialized ResourceMetrics message
pub struct ScopeMetricsIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, ResourceMetricsFieldRanges>,
}

impl<'a> Iterator for ScopeMetricsIter<'a> {
    type Item = RawScopeMetrics<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;

        Some(RawScopeMetrics {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of Metric - produces implementation of [`MetricView`] from byte array containing a
/// serialized ScopeMetrics object
pub struct MetricIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, ScopeMetricsFieldRanges>,
}

impl<'a> Iterator for MetricIter<'a> {
    type Item = RawMetric<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;

        Some(RawMetric {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of NumberDataPoint - produces implementation of [`NumberDataPointView`] from byte
/// array containing a serialized metric data message
pub struct NumberDataPointIter<'a, T: FieldRanges> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, T>,
}

impl<'a, T> Iterator for NumberDataPointIter<'a, T>
where
    T: FieldRanges,
{
    type Item = RawNumberDataPoint<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;
        Some(RawNumberDataPoint {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of HistogramDatapoint - produces implementation of [`HistogramDataPointView`]
/// from byte array containing a serialized Histogram message
pub struct HistogramDataPointIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, HistogramFieldRanges>,
}

impl<'a> Iterator for HistogramDataPointIter<'a> {
    type Item = RawHistogramDataPoint<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;
        Some(RawHistogramDataPoint {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of HistogramDatapoint - produces implementation of
/// [`ExponentialHistogramDataPointView`] from byte array containing a serialized
/// ExponentialHistogram message
pub struct ExpHistogramDataPointIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, ExpHistogramFieldRanges>,
}

impl<'a> Iterator for ExpHistogramDataPointIter<'a> {
    type Item = RawExpHistogramDatapoint<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;
        Some(RawExpHistogramDatapoint {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of SummaryDataPoint - produces implementation of [`SummaryDataPointView`] from byte
/// array containing a serialized Summary message
pub struct SummaryDataPointIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, SummaryFieldRanges>,
}

impl<'a> Iterator for SummaryDataPointIter<'a> {
    type Item = RawSummaryDataPoint<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;
        Some(RawSummaryDataPoint {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of ValueAtQuantile - produces implementation of [`ValueAtQuantileView`] from byte
/// array containing a serialized SummaryDataPoint message
pub struct ValueAtQuantileIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, SummaryDataPointFieldRanges>,
}

impl<'a> Iterator for ValueAtQuantileIter<'a> {
    type Item = RawValueAtQuantile<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;
        Some(RawValueAtQuantile {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of Exemplars - produces implementation of [`ExemplarView`] from byte buffer containing
/// proto serialized data point
pub struct ExemplarIter<'a, T: FieldRanges> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, T>,
}

impl<'a, T> Iterator for ExemplarIter<'a, T>
where
    T: FieldRanges,
{
    type Item = RawExemplar<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;
        Some(RawExemplar {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */

impl MetricsView for RawMetricsData<'_> {
    type ResourceMetrics<'res>
        = RawResourceMetrics<'res>
    where
        Self: 'res;
    type ResourceMetricsIter<'res>
        = ResourceMetricsIter<'res>
    where
        Self: 'res;

    #[inline]
    fn resources(&self) -> Self::ResourceMetricsIter<'_> {
        ResourceMetricsIter {
            buf: self.buf,
            pos: 0,
        }
    }
}

impl ResourceMetricsView for RawResourceMetrics<'_> {
    type Resource<'res>
        = RawResource<'res>
    where
        Self: 'res;
    type ScopeMetrics<'scp>
        = RawScopeMetrics<'scp>
    where
        Self: 'scp;
    type ScopesIter<'scp>
        = ScopeMetricsIter<'scp>
    where
        Self: 'scp;

    #[inline]
    fn resource(&self) -> Option<Self::Resource<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(RESOURCE_METRICS_RESOURCE)?;
        Some(RawResource::new(ProtoBytesParser::new(slice)))
    }

    #[inline]
    fn schema_url(&self) -> Option<Str<'_>> {
        self.byte_parser
            .advance_to_find_field(RESOURCE_METRICS_SCHEMA_URL)
    }

    #[inline]
    fn scopes(&self) -> Self::ScopesIter<'_> {
        ScopeMetricsIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                RESOURCE_METRICS_SCOPE_METRICS,
                wire_types::LEN,
            ),
        }
    }
}

impl ScopeMetricsView for RawScopeMetrics<'_> {
    type Metric<'met>
        = RawMetric<'met>
    where
        Self: 'met;
    type MetricIter<'met>
        = MetricIter<'met>
    where
        Self: 'met;
    type Scope<'scp>
        = RawInstrumentationScope<'scp>
    where
        Self: 'scp;

    #[inline]
    fn schema_url(&self) -> Str<'_> {
        self.byte_parser
            .advance_to_find_field(SCOPE_METRICS_SCHEMA_URL)
            .unwrap_or_default()
    }

    #[inline]
    fn scope(&self) -> Option<Self::Scope<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(SCOPE_METRICS_SCOPE)?;
        Some(RawInstrumentationScope::new(ProtoBytesParser::new(slice)))
    }

    #[inline]
    fn metrics(&self) -> Self::MetricIter<'_> {
        MetricIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                SCOPE_METRICS_METRICS,
                wire_types::LEN,
            ),
        }
    }
}

impl MetricView for RawMetric<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;
    type AttributeIter<'att>
        = KeyValueIter<'att, MetricFieldRanges>
    where
        Self: 'att;

    type Data<'dat>
        = RawData<'dat>
    where
        Self: 'dat;

    fn name(&self) -> Str<'_> {
        self.byte_parser
            .advance_to_find_field(METRIC_NAME)
            .unwrap_or_default()
    }

    fn description(&self) -> Str<'_> {
        self.byte_parser
            .advance_to_find_field(METRIC_DESCRIPTION)
            .unwrap_or_default()
    }

    fn unit(&self) -> Str<'_> {
        self.byte_parser
            .advance_to_find_field(METRIC_UNIT)
            .unwrap_or_default()
    }

    fn data(&self) -> Option<Self::Data<'_>> {
        let (slice, field_num) = self.byte_parser.advance_to_find_oneof(&[
            METRIC_GAUGE,
            METRIC_SUM,
            METRIC_HISTOGRAM,
            METRIC_EXPONENTIAL_HISTOGRAM,
            METRIC_SUMMARY,
        ])?;

        Some(RawData {
            field_num,
            buf: slice,
        })
    }

    fn metadata(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.byte_parser,
            METRIC_METADATA,
            wire_types::LEN,
        ))
    }
}

impl DataView<'_> for RawData<'_> {
    type Gauge<'gauge>
        = RawGauge<'gauge>
    where
        Self: 'gauge;

    type Sum<'sum>
        = RawSum<'sum>
    where
        Self: 'sum;

    type Histogram<'histogram>
        = RawHistogram<'histogram>
    where
        Self: 'histogram;

    type ExponentialHistogram<'exp>
        = RawExpHistogram<'exp>
    where
        Self: 'exp;

    type Summary<'summary>
        = RawSummary<'summary>
    where
        Self: 'summary;

    fn value_type(&self) -> DataType {
        match self.field_num {
            METRIC_GAUGE => DataType::Gauge,
            METRIC_SUM => DataType::Sum,
            METRIC_HISTOGRAM => DataType::Histogram,
            METRIC_EXPONENTIAL_HISTOGRAM => DataType::ExponentialHistogram,
            METRIC_SUMMARY => DataType::Summary,
            _ => {
                // safety: we only initialize this with the field number after having parsed one of
                // the valid field numbers from the Metric's buffer
                unreachable!("RawData DataView initialized with invalid field num")
            }
        }
    }

    fn as_gauge(&self) -> Option<Self::Gauge<'_>> {
        (self.field_num == METRIC_GAUGE).then_some(RawGauge {
            byte_parser: ProtoBytesParser::new(self.buf),
        })
    }

    fn as_sum(&self) -> Option<Self::Sum<'_>> {
        (self.field_num == METRIC_SUM).then_some(RawSum {
            byte_parser: ProtoBytesParser::new(self.buf),
        })
    }

    fn as_histogram(&self) -> Option<Self::Histogram<'_>> {
        (self.field_num == METRIC_HISTOGRAM).then_some(RawHistogram {
            byte_parser: ProtoBytesParser::new(self.buf),
        })
    }

    fn as_exponential_histogram(&self) -> Option<Self::ExponentialHistogram<'_>> {
        (self.field_num == METRIC_EXPONENTIAL_HISTOGRAM).then_some(RawExpHistogram {
            byte_parser: ProtoBytesParser::new(self.buf),
        })
    }

    fn as_summary(&self) -> Option<Self::Summary<'_>> {
        (self.field_num == METRIC_SUMMARY).then_some(RawSummary {
            byte_parser: ProtoBytesParser::new(self.buf),
        })
    }
}

impl GaugeView for RawGauge<'_> {
    type NumberDataPoint<'dp>
        = RawNumberDataPoint<'dp>
    where
        Self: 'dp;

    type NumberDataPointIter<'dp>
        = NumberDataPointIter<'dp, GaugeFieldRanges>
    where
        Self: 'dp;

    fn data_points(&self) -> Self::NumberDataPointIter<'_> {
        NumberDataPointIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                GAUGE_DATA_POINTS,
                wire_types::LEN,
            ),
        }
    }
}

impl SumView for RawSum<'_> {
    type NumberDataPoint<'dp>
        = RawNumberDataPoint<'dp>
    where
        Self: 'dp;

    type NumberDataPointIter<'dp>
        = NumberDataPointIter<'dp, SumFieldRanges>
    where
        Self: 'dp;

    fn aggregation_temporality(&self) -> AggregationTemporality {
        let val = self
            .byte_parser
            .advance_to_find_field(SUM_AGGREGATION_TEMPORALITY)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| val)
            .unwrap_or_default();

        AggregationTemporality::from(val as u32)
    }

    fn is_monotonic(&self) -> bool {
        let val = self
            .byte_parser
            .advance_to_find_field(SUM_IS_MONOTONIC)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| val)
            .unwrap_or_default();

        val != 0
    }

    fn data_points(&self) -> Self::NumberDataPointIter<'_> {
        NumberDataPointIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                GAUGE_DATA_POINTS,
                wire_types::LEN,
            ),
        }
    }
}

impl HistogramView for RawHistogram<'_> {
    type HistogramDataPoint<'dp>
        = RawHistogramDataPoint<'dp>
    where
        Self: 'dp;
    type HistogramDataPointIter<'dp>
        = HistogramDataPointIter<'dp>
    where
        Self: 'dp;

    fn aggregation_temporality(&self) -> AggregationTemporality {
        let val = self
            .byte_parser
            .advance_to_find_field(HISTOGRAM_AGGREGATION_TEMPORALITY)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| val)
            .unwrap_or_default();

        AggregationTemporality::from(val as u32)
    }

    fn data_points(&self) -> Self::HistogramDataPointIter<'_> {
        HistogramDataPointIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                HISTOGRAM_DATA_POINTS,
                wire_types::LEN,
            ),
        }
    }
}

impl SummaryView for RawSummary<'_> {
    type SummaryDataPoint<'dp>
        = RawSummaryDataPoint<'dp>
    where
        Self: 'dp;
    type SummaryDataPointIter<'dp>
        = SummaryDataPointIter<'dp>
    where
        Self: 'dp;

    fn data_points(&self) -> Self::SummaryDataPointIter<'_> {
        SummaryDataPointIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                SUMMARY_DATA_POINTS,
                wire_types::LEN,
            ),
        }
    }
}

impl ExponentialHistogramView for RawExpHistogram<'_> {
    type ExponentialHistogramDataPoint<'edp>
        = RawExpHistogramDatapoint<'edp>
    where
        Self: 'edp;
    type ExponentialHistogramDataPointIter<'edp>
        = ExpHistogramDataPointIter<'edp>
    where
        Self: 'edp;

    fn aggregation_temporality(&self) -> AggregationTemporality {
        let val = self
            .byte_parser
            .advance_to_find_field(EXPONENTIAL_HISTOGRAM_AGGREGATION_TEMPORALITY)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| val)
            .unwrap_or_default();

        AggregationTemporality::from(val as u32)
    }

    fn data_points(&self) -> Self::ExponentialHistogramDataPointIter<'_> {
        ExpHistogramDataPointIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                EXPONENTIAL_HISTOGRAM_DATA_POINTS,
                wire_types::LEN,
            ),
        }
    }
}

impl NumberDataPointView for RawNumberDataPoint<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, NumberDataPointFieldRanges>
    where
        Self: 'att;

    type Exemplar<'ex>
        = RawExemplar<'ex>
    where
        Self: 'ex;
    type ExemplarIter<'ex>
        = ExemplarIter<'ex, NumberDataPointFieldRanges>
    where
        Self: 'ex;

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.byte_parser,
            NUMBER_DP_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    fn start_time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(NUMBER_DP_START_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(NUMBER_DP_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn value(&self) -> Option<Value> {
        let (slice, field_num) = self
            .byte_parser
            .advance_to_find_oneof(&[NUMBER_DP_AS_DOUBLE, NUMBER_DP_AS_INT])?;

        match field_num {
            NUMBER_DP_AS_DOUBLE => {
                let double_bytes: [u8; 8] = slice.try_into().ok()?;
                Some(Value::Double(f64::from_le_bytes(double_bytes)))
            }
            NUMBER_DP_AS_INT => {
                let int_bytes: [u8; 8] = slice.try_into().ok()?;
                Some(Value::Integer(i64::from_le_bytes(int_bytes)))
            }
            _ => {
                // this shouldn't happen, as advance_to_find_oneof should return one of the passed
                // field_num, so just ignore it
                None
            }
        }
    }

    fn exemplars(&self) -> Self::ExemplarIter<'_> {
        ExemplarIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                NUMBER_DP_EXEMPLARS,
                wire_types::LEN,
            ),
        }
    }

    fn flags(&self) -> DataPointFlags {
        let flags = self
            .byte_parser
            .advance_to_find_field(NUMBER_DP_FLAGS)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| val as u32);

        DataPointFlags::new(flags.unwrap_or_default())
    }
}

impl HistogramDataPointView for RawHistogramDataPoint<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, HistogramDataPointFieldRanges>
    where
        Self: 'att;

    type BucketCountIter<'bc>
        = RepeatedFixed64Iter<'bc, HistogramDataPointFieldRanges, u64>
    where
        Self: 'bc;

    type ExplicitBoundsIter<'eb>
        = RepeatedFixed64Iter<'eb, HistogramDataPointFieldRanges, f64>
    where
        Self: 'eb;

    type Exemplar<'ex>
        = RawExemplar<'ex>
    where
        Self: 'ex;
    type ExemplarIter<'ex>
        = ExemplarIter<'ex, HistogramDataPointFieldRanges>
    where
        Self: 'ex;

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.byte_parser,
            HISTOGRAM_DP_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    fn bucket_counts(&self) -> Self::BucketCountIter<'_> {
        RepeatedFixed64Iter::from_byte_parser(&self.byte_parser, HISTOGRAM_DP_BUCKET_COUNTS)
    }

    fn count(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(HISTOGRAM_DP_COUNT)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn exemplars(&self) -> Self::ExemplarIter<'_> {
        ExemplarIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                HISTOGRAM_DP_EXEMPLARS,
                wire_types::LEN,
            ),
        }
    }

    fn explicit_bounds(&self) -> Self::ExplicitBoundsIter<'_> {
        RepeatedFixed64Iter::from_byte_parser(&self.byte_parser, HISTOGRAM_DP_EXPLICIT_BOUNDS)
    }

    fn flags(&self) -> DataPointFlags {
        let flags = self
            .byte_parser
            .advance_to_find_field(HISTOGRAM_DP_FLAGS)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| val as u32);

        DataPointFlags::new(flags.unwrap_or_default())
    }

    fn max(&self) -> Option<f64> {
        self.byte_parser
            .advance_to_find_field(HISTOGRAM_DP_MAX)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
    }

    fn min(&self) -> Option<f64> {
        self.byte_parser
            .advance_to_find_field(HISTOGRAM_DP_MIN)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
    }

    fn start_time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(HISTOGRAM_DP_START_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn sum(&self) -> Option<f64> {
        self.byte_parser
            .advance_to_find_field(HISTOGRAM_DP_SUM)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
    }

    fn time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(HISTOGRAM_DP_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }
}

impl ExponentialHistogramDataPointView for RawExpHistogramDatapoint<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, ExpHistogramDataPointFieldRanges>
    where
        Self: 'att;

    type Buckets<'b>
        = RawBuckets<'b>
    where
        Self: 'b;

    type Exemplar<'ex>
        = RawExemplar<'ex>
    where
        Self: 'ex;
    type ExemplarIter<'ex>
        = ExemplarIter<'ex, ExpHistogramDataPointFieldRanges>
    where
        Self: 'ex;

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.byte_parser,
            EXP_HISTOGRAM_DP_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    fn count(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_COUNT)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn exemplars(&self) -> Self::ExemplarIter<'_> {
        ExemplarIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                EXP_HISTOGRAM_DP_EXEMPLARS,
                wire_types::LEN,
            ),
        }
    }

    fn flags(&self) -> DataPointFlags {
        let flags = self
            .byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_FLAGS)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| val as u32);

        DataPointFlags::new(flags.unwrap_or_default())
    }

    fn max(&self) -> Option<f64> {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_MAX)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
    }

    fn min(&self) -> Option<f64> {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_MIN)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
    }

    fn negative(&self) -> Option<Self::Buckets<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_NEGATIVE)?;
        Some(RawBuckets {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }

    fn positive(&self) -> Option<Self::Buckets<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_POSITIVE)?;
        Some(RawBuckets {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }

    fn scale(&self) -> i32 {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_SCALE)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| decode_sint32(val as i32))
            .unwrap_or_default()
    }

    fn start_time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_START_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn sum(&self) -> Option<f64> {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_SUM)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
    }

    fn time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn zero_count(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_ZERO_COUNT)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn zero_threshold(&self) -> f64 {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_DP_ZERO_THRESHOLD)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
            .unwrap_or_default()
    }
}

impl BucketsView for RawBuckets<'_> {
    type BucketCountIter<'bc>
        = RepeatedVarintIter<'bc, BucketsFieldRanges>
    where
        Self: 'bc;

    fn bucket_counts(&self) -> Self::BucketCountIter<'_> {
        RepeatedVarintIter::from_byte_parser(&self.byte_parser, EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS)
    }

    fn offset(&self) -> i32 {
        self.byte_parser
            .advance_to_find_field(EXP_HISTOGRAM_BUCKET_OFFSET)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| decode_sint32(val as i32))
            .unwrap_or_default()
    }
}

impl SummaryDataPointView for RawSummaryDataPoint<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, SummaryDataPointFieldRanges>
    where
        Self: 'att;

    type ValueAtQuantile<'vaq>
        = RawValueAtQuantile<'vaq>
    where
        Self: 'vaq;

    type ValueAtQuantileIter<'vaq>
        = ValueAtQuantileIter<'vaq>
    where
        Self: 'vaq;

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.byte_parser,
            SUMMARY_DP_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    fn count(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(SUMMARY_DP_COUNT)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn sum(&self) -> f64 {
        self.byte_parser
            .advance_to_find_field(SUMMARY_DP_SUM)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
            .unwrap_or_default()
    }

    fn quantile_values(&self) -> Self::ValueAtQuantileIter<'_> {
        ValueAtQuantileIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                SUMMARY_DP_QUANTILE_VALUES,
                wire_types::LEN,
            ),
        }
    }

    fn start_time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(SUMMARY_DP_START_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(SUMMARY_DP_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn flags(&self) -> DataPointFlags {
        let flags = self
            .byte_parser
            .advance_to_find_field(SUMMARY_DP_FLAGS)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(val, _)| val as u32);

        DataPointFlags::new(flags.unwrap_or_default())
    }
}

impl ValueAtQuantileView for RawValueAtQuantile<'_> {
    fn quantile(&self) -> f64 {
        self.byte_parser
            .advance_to_find_field(VALUE_AT_QUANTILE_QUANTILE)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
            .unwrap_or_default()
    }

    fn value(&self) -> f64 {
        self.byte_parser
            .advance_to_find_field(VALUE_AT_QUANTILE_VALUE)
            .and_then(|slice| slice.try_into().ok())
            .map(f64::from_le_bytes)
            .unwrap_or_default()
    }
}

impl ExemplarView for RawExemplar<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, ExemplarFieldRanges>
    where
        Self: 'att;

    fn filtered_attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.byte_parser,
            EXEMPLAR_FILTERED_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    fn span_id(&self) -> Option<&SpanId> {
        let slice = self.byte_parser.advance_to_find_field(EXEMPLAR_SPAN_ID);
        slice.and_then(|slice| slice.try_into().ok())
    }

    fn time_unix_nano(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(EXEMPLAR_TIME_UNIX_NANO)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn trace_id(&self) -> Option<&TraceId> {
        self.byte_parser
            .advance_to_find_field(EXEMPLAR_TRACE_ID)
            .and_then(|slice| slice.try_into().ok())
    }

    fn value(&self) -> Option<Value> {
        let (slice, field_num) = self
            .byte_parser
            .advance_to_find_oneof(&[EXEMPLAR_AS_DOUBLE, EXEMPLAR_AS_INT])?;

        match field_num {
            EXEMPLAR_AS_DOUBLE => {
                let double_bytes: [u8; 8] = slice.try_into().ok()?;
                Some(Value::Double(f64::from_le_bytes(double_bytes)))
            }
            EXEMPLAR_AS_INT => {
                let int_bytes: [u8; 8] = slice.try_into().ok()?;
                Some(Value::Integer(i64::from_le_bytes(int_bytes)))
            }
            _ => {
                // this shouldn't happen, as advance_to_find_oneof should return one of the passed
                // field_num, so just ignore it
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        otlp::ProtoBuffer,
        proto::opentelemetry::metrics::v1::{
            Metric, NumberDataPoint, Sum, metric::Data, number_data_point,
        },
    };
    use prost::Message;

    #[test]
    fn test_oneof_double_reads() {
        // this test is guarding against regressions of a bug where if multiple read calls
        // were made to some view implementations to read the field of a oneof, we'd sometimes
        // return the wrong type of value

        let metric = Metric {
            data: Some(Data::Sum(Sum::default())),
            ..Default::default()
        };
        let mut bytes = vec![];
        metric.encode(&mut bytes).unwrap();
        let metric_view = RawMetric {
            byte_parser: ProtoBytesParser::new(&bytes),
        };
        assert_eq!(metric_view.data().unwrap().value_type(), DataType::Sum);
        assert_eq!(metric_view.data().unwrap().value_type(), DataType::Sum);

        let number_dp = NumberDataPoint {
            value: Some(number_data_point::Value::AsInt(1)),
            ..Default::default()
        };
        let mut bytes = vec![];
        number_dp.encode(&mut bytes).unwrap();
        let number_dp_view = RawNumberDataPoint {
            byte_parser: ProtoBytesParser::new(&bytes),
        };
        assert_eq!(number_dp_view.value(), Some(Value::Integer(1)));
        assert_eq!(number_dp_view.value(), Some(Value::Integer(1)));
    }

    #[test]
    fn test_packed_and_expanded_decoding_hist_dp() {
        // In histogram data-points there are two fields that are repeated primitives which should
        // be "packed" encoded: bucket_counts and explicit_bounds. However, proto docs say we need
        // to support maybe reading these as packed & expanded, so we test for this

        let mut buffer = ProtoBuffer::new();

        // first write packed encoded
        buffer.encode_field_tag(HISTOGRAM_DP_BUCKET_COUNTS, wire_types::LEN);
        buffer.encode_varint(3 * 8); // 8 bytes per val (fixed64)
        buffer.extend_from_slice(&1u64.to_le_bytes());
        buffer.extend_from_slice(&2u64.to_le_bytes());
        buffer.extend_from_slice(&3u64.to_le_bytes());

        buffer.encode_field_tag(HISTOGRAM_DP_EXPLICIT_BOUNDS, wire_types::LEN);
        buffer.encode_varint(2 * 8); // 8 bytes per val (double)
        buffer.extend_from_slice(&4.0f64.to_le_bytes());
        buffer.extend_from_slice(&5.0f64.to_le_bytes());

        // check results
        let hist_dp_view = RawHistogramDataPoint {
            byte_parser: ProtoBytesParser::new(buffer.as_ref()),
        };
        let result_bucket_counts = hist_dp_view.bucket_counts().collect::<Vec<_>>();
        assert_eq!(result_bucket_counts, vec![1, 2, 3]);
        let result_explicit_bounds = hist_dp_view.explicit_bounds().collect::<Vec<_>>();
        assert_eq!(result_explicit_bounds, vec![4.0, 5.0]);

        // now write with expanded encoding
        buffer.clear();
        for i in [6u64, 7, 8] {
            buffer.encode_field_tag(HISTOGRAM_DP_BUCKET_COUNTS, wire_types::FIXED64);
            buffer.extend_from_slice(&i.to_le_bytes());
        }
        for i in [9f64, 0.0] {
            buffer.encode_field_tag(HISTOGRAM_DP_EXPLICIT_BOUNDS, wire_types::FIXED64);
            buffer.extend_from_slice(&i.to_le_bytes());
        }

        // check results
        let hist_dp_view = RawHistogramDataPoint {
            byte_parser: ProtoBytesParser::new(buffer.as_ref()),
        };
        let result_bucket_counts = hist_dp_view.bucket_counts().collect::<Vec<_>>();
        assert_eq!(result_bucket_counts, vec![6, 7, 8]);
        let result_explicit_bounds = hist_dp_view.explicit_bounds().collect::<Vec<_>>();
        assert_eq!(result_explicit_bounds, vec![9.0, 0.0]);

        // there's an extra edge-case where when using packed encoding, multiple key-value
        // pairs are allowed. test to ensure we handle this:
        buffer.clear();
        buffer.encode_field_tag(HISTOGRAM_DP_BUCKET_COUNTS, wire_types::LEN);
        buffer.encode_varint(2 * 8);
        buffer.extend_from_slice(&1u64.to_le_bytes());
        buffer.extend_from_slice(&2u64.to_le_bytes());
        buffer.encode_field_tag(HISTOGRAM_DP_BUCKET_COUNTS, wire_types::LEN);
        buffer.encode_varint(8);
        buffer.extend_from_slice(&3u64.to_le_bytes());

        buffer.encode_field_tag(HISTOGRAM_DP_EXPLICIT_BOUNDS, wire_types::LEN);
        buffer.encode_varint(8);
        buffer.extend_from_slice(&4.0f64.to_le_bytes());
        buffer.encode_field_tag(HISTOGRAM_DP_EXPLICIT_BOUNDS, wire_types::LEN);
        buffer.encode_varint(8);
        buffer.extend_from_slice(&5.0f64.to_le_bytes());
        let hist_dp_view = RawHistogramDataPoint {
            byte_parser: ProtoBytesParser::new(buffer.as_ref()),
        };
        let result_bucket_counts = hist_dp_view.bucket_counts().collect::<Vec<_>>();
        assert_eq!(result_bucket_counts, vec![1, 2, 3]);
        let result_explicit_bounds = hist_dp_view.explicit_bounds().collect::<Vec<_>>();
        assert_eq!(result_explicit_bounds, vec![4.0, 5.0]);
    }

    #[test]
    fn test_packed_and_expanded_decoding_exp_hist_dp_bucket() {
        // same as the above test case, but for the bucket_counts field on
        // ExponentialHistogramDataPoint Bucket's bucket_counts field which
        // contains a repeated uint64

        let mut buffer = ProtoBuffer::new();

        // test packed encoding
        buffer.encode_field_tag(EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, wire_types::LEN);
        buffer.encode_varint(3); // 3 x 1byte varints
        buffer.extend_from_slice(&[0x01, 0x02, 0x03]);

        let bucket_view = RawBuckets {
            byte_parser: ProtoBytesParser::new(buffer.as_ref()),
        };
        let bucket_counts = bucket_view.bucket_counts().collect::<Vec<_>>();
        assert_eq!(bucket_counts, vec![1, 2, 3]);

        // test expanded encoding
        buffer.clear();
        buffer.encode_field_tag(EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, wire_types::VARINT);
        buffer.encode_varint(1);
        buffer.encode_field_tag(EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, wire_types::VARINT);
        buffer.encode_varint(2);
        buffer.encode_field_tag(EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, wire_types::VARINT);
        buffer.encode_varint(3);
        let bucket_view = RawBuckets {
            byte_parser: ProtoBytesParser::new(buffer.as_ref()),
        };
        let bucket_counts = bucket_view.bucket_counts().collect::<Vec<_>>();
        assert_eq!(bucket_counts, vec![1, 2, 3]);

        // there's an extra edge-case where when using packed encoding, multiple key-value
        // pairs are allowed. test to ensure we handle this:
        buffer.clear();
        buffer.encode_field_tag(EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, wire_types::LEN);
        buffer.encode_varint(2);
        buffer.extend_from_slice(&[0x01, 0x02]);
        buffer.encode_field_tag(EXP_HISTOGRAM_BUCKET_BUCKET_COUNTS, wire_types::LEN);
        buffer.encode_varint(1);
        buffer.extend_from_slice(&[0x03]);
        let bucket_view = RawBuckets {
            byte_parser: ProtoBytesParser::new(buffer.as_ref()),
        };
        let bucket_counts = bucket_view.bucket_counts().collect::<Vec<_>>();
        assert_eq!(bucket_counts, vec![1, 2, 3]);
    }
}
