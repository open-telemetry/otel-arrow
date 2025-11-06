// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for the messages defined in metrics.proto

use std::cell::Cell;
use std::num::NonZeroUsize;

use crate::proto::consts::field_num::metrics::{
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
    SUM_AGGREGATION_TEMPORALITY, SUM_DATA_POINTS, SUM_IS_MONOTONIC,
};
use crate::proto::consts::wire_types;

use crate::views::common::Str;
use crate::views::metrics::{
    AggregationTemporality, DataPointFlags, DataType, DataView, GaugeView, HistogramDataPointView,
    HistogramView, MetricView, MetricsView, NumberDataPointView, ResourceMetricsView,
    ScopeMetricsView, SumView, Value,
};
use crate::views::otlp::bytes::common::{KeyValueIter, RawInstrumentationScope, RawKeyValue};
use crate::views::otlp::bytes::decode::{
    FieldRanges, PackedFixed64Iter, ProtoBytesParser, RepeatedFieldProtoBytesParser,
    from_option_nonzero_range_to_primitive, read_len_delim, read_varint, to_nonzero_range,
};
use crate::views::otlp::bytes::resource::RawResource;
use crate::views::otlp::proto::metrics::{
    ExemplarIter, NumberDataPointIter as ObjNumberDataPointIter, ObjExemplar,
    ObjExponentialHistogram, ObjNumberDataPoint, ObjSummary,
};

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

/// Known field ranges for fields on `Sum` message
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
    explicit_bounds: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    bucket_counts: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    min: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    max: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attributes: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_exemplar: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
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
            HISTOGRAM_DP_BUCKET_COUNTS => self.bucket_counts.get(),
            HISTOGRAM_DP_EXPLICIT_BOUNDS => self.explicit_bounds.get(),
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
                if wire_type == wire_types::LEN {
                    self.explicit_bounds.set(Some(range));
                }
            }
            HISTOGRAM_DP_BUCKET_COUNTS => {
                if wire_type == wire_types::LEN {
                    self.bucket_counts.set(Some(range));
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
    // TODO all the use of the Obj implementations here are placeholders
    type NumberDataPoint<'dp>
        = ObjNumberDataPoint<'dp>
    where
        Self: 'dp;
    type NumberDataPointIter<'dp>
        = ObjNumberDataPointIter<'dp>
    where
        Self: 'dp;

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
        = ObjExponentialHistogram<'exp>
    where
        Self: 'exp;

    type Summary<'summary>
        = ObjSummary<'summary>
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
        // TODO
        None
    }

    fn as_summary(&self) -> Option<Self::Summary<'_>> {
        // TODO
        None
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

impl NumberDataPointView for RawNumberDataPoint<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, NumberDataPointFieldRanges>
    where
        Self: 'att;

    // TODO using Obj Exemplars temporarily here until we've implemented an exemplar view
    // backed by proto bytes
    type Exemplar<'ex>
        = ObjExemplar<'ex>
    where
        Self: 'ex;
    type ExemplarIter<'ex>
        = ExemplarIter<'ex>
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
        // TODO exemplars
        ExemplarIter::new([].iter())
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
        = PackedFixed64Iter<'bc, u64>
    where
        Self: 'bc;

    type ExplicitBoundsIter<'eb>
        = PackedFixed64Iter<'eb, f64>
    where
        Self: 'eb;

    // TODO this is temporary
    type Exemplar<'ex>
        = ObjExemplar<'ex>
    where
        Self: 'ex;
    type ExemplarIter<'ex>
        = ExemplarIter<'ex>
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
        let buf = self
            .byte_parser
            .advance_to_find_field(HISTOGRAM_DP_BUCKET_COUNTS)
            .unwrap_or_default();
        PackedFixed64Iter::new(buf)
    }

    fn count(&self) -> u64 {
        self.byte_parser
            .advance_to_find_field(HISTOGRAM_DP_COUNT)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .unwrap_or_default()
    }

    fn exemplars(&self) -> Self::ExemplarIter<'_> {
        // TODO
        ExemplarIter::new([].iter())
    }

    fn explicit_bounds(&self) -> Self::ExplicitBoundsIter<'_> {
        let buf = self
            .byte_parser
            .advance_to_find_field(HISTOGRAM_DP_EXPLICIT_BOUNDS)
            .unwrap_or_default();

        PackedFixed64Iter::new(buf)
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::proto::opentelemetry::metrics::v1::{
        Metric, NumberDataPoint, Sum, metric::Data, number_data_point,
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
}
