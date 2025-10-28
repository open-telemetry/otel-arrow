// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for the messages defined in metrics.proto

use std::cell::Cell;
use std::num::NonZeroUsize;

use otel_arrow_rust::proto::consts::field_num::metrics::{
    METRIC_DESCRIPTION, METRIC_EXPONENTIAL_HISTOGRAM, METRIC_GAUGE, METRIC_HISTOGRAM,
    METRIC_METADATA, METRIC_NAME, METRIC_SUM, METRIC_SUMMARY, METRIC_UNIT,
    RESOURCE_METRICS_RESOURCE, RESOURCE_METRICS_SCHEMA_URL, RESOURCE_METRICS_SCOPE_METRICS,
    SCOPE_METRICS_METRICS, SCOPE_METRICS_SCHEMA_URL, SCOPE_METRICS_SCOPE,
};
use otel_arrow_rust::proto::consts::wire_types;

use crate::views::metrics::{MetricView, ResourceMetricsView, ScopeMetricsView};
use crate::views::otlp::bytes::decode::{
    FieldRanges, ProtoBytesParser, from_option_nonzero_range_to_primitive, to_nonzero_range,
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
                RESOURCE_METRICS_SCHEMA_URL => self.resource.set(Some(range)),
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

/// Implementation of [`MetricView`] backed by protobuf serialized `ResourceMetrics` message
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
                SCOPE_METRICS_SCHEMA_URL => self.scope.set(Some(range)),
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

pub struct RawMetric<'a> {
    byte_parser: ProtoBytesParser<'a, MetricFieldRanges>,
}

/// Known offsets within byte buffer for fields in the `Metric` message
pub struct MetricFieldRanges {
    name: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    description: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    unit: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    data: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_metadata: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
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
            | METRIC_SUMMARY => self.data.get(),
            METRIC_METADATA => self.first_metadata.get(),
            _ => return None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {}
}
