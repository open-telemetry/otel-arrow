// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for the messages defined in metrics.proto

use std::cell::Cell;
use std::num::NonZeroUsize;

use otel_arrow_rust::proto::consts::field_num::metrics::{
    METRIC_DESCRIPTION, METRIC_EXPONENTIAL_HISTOGRAM, METRIC_GAUGE, METRIC_HISTOGRAM,
    METRIC_METADATA, METRIC_NAME, METRIC_SUM, METRIC_SUMMARY, METRIC_UNIT,
    METRICS_DATA_RESOURCE_METRICS, RESOURCE_METRICS_RESOURCE, RESOURCE_METRICS_SCHEMA_URL,
    RESOURCE_METRICS_SCOPE_METRICS, SCOPE_METRICS_METRICS, SCOPE_METRICS_SCHEMA_URL,
    SCOPE_METRICS_SCOPE,
};
use otel_arrow_rust::proto::consts::wire_types;

use crate::views::common::Str;
use crate::views::metrics::{MetricView, MetricsView, ResourceMetricsView, ScopeMetricsView};
use crate::views::otlp::bytes::common::{KeyValueIter, RawInstrumentationScope, RawKeyValue};
use crate::views::otlp::bytes::decode::{
    FieldRanges, ProtoBytesParser, RepeatedFieldProtoBytesParser,
    from_option_nonzero_range_to_primitive, read_len_delim, read_varint, to_nonzero_range,
};
use crate::views::otlp::bytes::resource::RawResource;
use crate::views::otlp::proto::metrics::ObjData;

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
                METRIC_GAUGE | METRIC_SUM | METRIC_HISTOGRAM | METRIC_EXPONENTIAL_HISTOGRAM => {
                    self.data.set(Some(range))
                }
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

// TODO field ranges etc. for additional metrics messages

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

    // TODO - this is a placeholder so the code compiles. We don't have any other implementations
    // of DataView, so just referencing the proto structs version for now.
    type Data<'dat>
        = ObjData<'dat>
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
        // TODO add implementation of metrics data
        None
    }

    fn metadata(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.byte_parser,
            METRIC_METADATA,
            wire_types::LEN,
        ))
    }
}
