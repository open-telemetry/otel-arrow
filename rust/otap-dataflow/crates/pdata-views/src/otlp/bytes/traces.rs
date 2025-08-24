// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for serialized OTLP protobuf
//! bytes for the messages defined in trace.proto

use std::{cell::Cell, num::NonZeroUsize};

use otel_arrow_rust::{
    proto::opentelemetry::trace::v1::{span::SpanKind, status::StatusCode},
    schema::{SpanId, TraceId},
};

use crate::otlp::bytes::common::{KeyValueIter, RawInstrumentationScope, RawKeyValue};
use crate::otlp::bytes::consts::field_num::traces::{
    RESOURCE_SPANS_RESOURCE, RESOURCE_SPANS_SCHEMA_URL, RESOURCE_SPANS_SCOPE_SPANS,
    SCOPE_SPANS_SCHEMA_URL, SCOPE_SPANS_SCOPE, SCOPE_SPANS_SPANS, SPAN_ATTRIBUTES,
    SPAN_DROPPED_ATTRIBUTES_COUNT, SPAN_DROPPED_EVENTS_COUNT, SPAN_DROPPED_LINKS_COUNT,
    SPAN_END_TIME_UNIX_NANO, SPAN_EVENT_ATTRIBUTES, SPAN_EVENT_DROPPED_ATTRIBUTES_COUNTS,
    SPAN_EVENT_NAME, SPAN_EVENT_TIME_UNIX_NANO, SPAN_EVENTS, SPAN_FLAGS, SPAN_KIND,
    SPAN_LINK_ATTRIBUTES, SPAN_LINK_DROPPED_ATTRIBUTES_COUNT, SPAN_LINK_FLAGS, SPAN_LINK_SPAN_ID,
    SPAN_LINK_TRACE_ID, SPAN_LINK_TRACE_STATE, SPAN_LINKS, SPAN_NAME, SPAN_PARENT_SPAN_ID,
    SPAN_SPAN_ID, SPAN_START_TIME_UNIX_NANO, SPAN_STATUS, SPAN_STATUS_CODE, SPAN_STATUS_MESSAGE,
    SPAN_TRACE_ID, SPAN_TRACE_STATE, TRACES_DATA_RESOURCE_SPANS,
};
use crate::otlp::bytes::consts::wire_types;
use crate::otlp::bytes::decode::{
    FieldRanges, ProtoBytesParser, RepeatedFieldProtoBytesParser,
    from_option_nonzero_range_to_primitive, read_dropped_count, read_len_delim, read_varint,
    to_nonzero_range,
};
use crate::otlp::bytes::resource::RawResource;
use crate::views::trace::{
    EventView, LinkView, ResourceSpansView, ScopeSpansView, SpanView, StatusView, TracesView,
};

/// Implementation of [`TracesView`] backed by protobuf serialized `TracesData` message
pub struct RawTraceData<'a> {
    buf: &'a [u8],
}

impl<'a> RawTraceData<'a> {
    /// Create a new [`RawTraceData`] instance
    #[must_use]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

/// Implementation of [`ResourceSpansView`] backed by protobuf serialized `ResourceSpans` message
pub struct RawResourceSpans<'a> {
    byte_parser: ProtoBytesParser<'a, ResourceSpansFieldRanges>,
}

/// Known field offsets within byte buffer for fields in `ResourceSpans` message
pub struct ResourceSpansFieldRanges {
    resource: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    schema_url: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_scope_spans: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ResourceSpansFieldRanges {
    fn new() -> Self {
        Self {
            resource: Cell::new(None),
            schema_url: Cell::new(None),
            first_scope_spans: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            RESOURCE_SPANS_RESOURCE => self.resource.get(),
            RESOURCE_SPANS_SCHEMA_URL => self.schema_url.get(),
            RESOURCE_SPANS_SCOPE_SPANS => self.first_scope_spans.get(),
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
                RESOURCE_SPANS_RESOURCE => self.resource.set(Some(range)),
                RESOURCE_SPANS_SCHEMA_URL => self.schema_url.set(Some(range)),
                RESOURCE_SPANS_SCOPE_SPANS => {
                    if self.first_scope_spans.get().is_none() {
                        self.first_scope_spans.set(Some(range));
                    }
                }
                _ => { /* ignore  */ }
            }
        }
    }
}

/// Implementation of the [`ScopeSpansView`] backed by protobuf serialized `ScopeSpans` message
pub struct RawScopeSpans<'a> {
    byte_parser: ProtoBytesParser<'a, ScopeSpansFieldRanges>,
}

/// Known field offsets within byte buffer for fields in `ScopeSpans` message
pub struct ScopeSpansFieldRanges {
    scope: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    schema_url: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_span: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for ScopeSpansFieldRanges {
    fn new() -> Self {
        Self {
            scope: Cell::new(None),
            schema_url: Cell::new(None),
            first_span: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SCOPE_SPANS_SCOPE => self.scope.get(),
            SCOPE_SPANS_SCHEMA_URL => self.schema_url.get(),
            SCOPE_SPANS_SPANS => self.first_span.get(),
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
                SCOPE_SPANS_SCOPE => self.scope.set(Some(range)),
                SCOPE_SPANS_SCHEMA_URL => self.schema_url.set(Some(range)),
                SCOPE_SPANS_SPANS => {
                    if self.first_span.get().is_none() {
                        self.first_span.set(Some(range));
                    }
                }
                _ => { /* ignore  */ }
            }
        }
    }
}

/// Implementation of [`SpanView`] backed by protobuf serialized `Span` message
pub struct RawSpan<'a> {
    bytes_parser: ProtoBytesParser<'a, SpanFieldRanges>,
}

/// Known field offsets within byte buffer for fields in `Span` message
pub struct SpanFieldRanges {
    scalar_fields: [Cell<Option<(NonZeroUsize, NonZeroUsize)>>; 17],
    first_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_event: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_link: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for SpanFieldRanges {
    fn new() -> Self {
        Self {
            scalar_fields: std::array::from_fn(|_| Cell::new(None)),
            first_attribute: Cell::new(None),
            first_event: Cell::new(None),
            first_link: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SPAN_ATTRIBUTES => self.first_attribute.get(),
            SPAN_EVENTS => self.first_event.get(),
            SPAN_LINKS => self.first_link.get(),
            _ => self
                .scalar_fields
                .get(field_num as usize)
                .and_then(|c| c.get()),
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        const WIRE_TYPES: [u64; 17] = [
            0,
            wire_types::LEN,     // trace_id = 1
            wire_types::LEN,     // span_id = 2
            wire_types::LEN,     // trace_state = 3
            wire_types::LEN,     // parent_span_id = 4
            wire_types::LEN,     // name = 5
            wire_types::VARINT,  // kind = 6
            wire_types::FIXED64, // start_time_unix_nano = 7
            wire_types::FIXED64, // end_time_unix_nano = 8
            wire_types::LEN,     // attributes = 9
            wire_types::VARINT,  // dropped_attributes_count = 10
            wire_types::LEN,     // events = 11,
            wire_types::VARINT,  // dropped_events_count = 12
            wire_types::LEN,     // links = 13
            wire_types::VARINT,  // dropped_links_count = 14,
            wire_types::LEN,     // status = 15
            wire_types::FIXED32, // flags = 16
        ];

        let range = match to_nonzero_range(start, end) {
            Some(range) => Some(range),
            None => return,
        };

        match field_num {
            SPAN_ATTRIBUTES => {
                if self.first_attribute.get().is_none() && wire_type == wire_types::LEN {
                    self.first_attribute.set(range)
                }
            }
            SPAN_EVENTS => {
                if self.first_event.get().is_none() && wire_type == wire_types::LEN {
                    self.first_event.set(range)
                }
            }
            SPAN_LINKS => {
                if self.first_link.get().is_none() && wire_type == wire_types::LEN {
                    self.first_link.set(range)
                }
            }
            _ => {
                let idx = field_num as usize;
                if wire_type == WIRE_TYPES[idx] {
                    self.scalar_fields[idx].set(range)
                }
            }
        }
    }
}

/// Implementation of [`EventView`] backed by protobuf serialized span `Event` message
pub struct RawSpanEvent<'a> {
    bytes_parser: ProtoBytesParser<'a, SpanEventFieldRanges>,
}

/// Known field offsets within byte buffer for fields in span `Event` message
pub struct SpanEventFieldRanges {
    time_unix_nano: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    name: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    dropped_attributes_count: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for SpanEventFieldRanges {
    fn new() -> Self {
        Self {
            time_unix_nano: Cell::new(None),
            name: Cell::new(None),
            dropped_attributes_count: Cell::new(None),
            first_attribute: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SPAN_EVENT_TIME_UNIX_NANO => self.time_unix_nano.get(),
            SPAN_EVENT_NAME => self.name.get(),
            SPAN_EVENT_DROPPED_ATTRIBUTES_COUNTS => self.dropped_attributes_count.get(),
            SPAN_EVENT_ATTRIBUTES => self.first_attribute.get(),
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
            SPAN_EVENT_TIME_UNIX_NANO => {
                if wire_type == wire_types::FIXED64 {
                    self.time_unix_nano.set(Some(range))
                }
            }
            SPAN_EVENT_NAME => {
                if wire_type == wire_types::LEN {
                    self.name.set(Some(range))
                }
            }
            SPAN_EVENT_DROPPED_ATTRIBUTES_COUNTS => {
                if wire_type == wire_types::VARINT {
                    self.dropped_attributes_count.set(Some(range))
                }
            }
            SPAN_EVENT_ATTRIBUTES => {
                if self.first_attribute.get().is_none() && wire_type == wire_types::LEN {
                    self.first_attribute.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`LinkView`] backed by protobuf serialized span `Link` message
pub struct RawSpanLink<'a> {
    bytes_parser: ProtoBytesParser<'a, SpanLinkFieldRanges>,
}

/// Known field offsets within byte buffer for fields in span `Link` message
pub struct SpanLinkFieldRanges {
    trace_id: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    span_id: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    trace_state: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    dropped_attributes_count: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    flags: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    first_attribute: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for SpanLinkFieldRanges {
    fn new() -> Self {
        Self {
            trace_id: Cell::new(None),
            span_id: Cell::new(None),
            trace_state: Cell::new(None),
            dropped_attributes_count: Cell::new(None),
            flags: Cell::new(None),
            first_attribute: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SPAN_LINK_TRACE_ID => self.trace_id.get(),
            SPAN_LINK_SPAN_ID => self.span_id.get(),
            SPAN_LINK_TRACE_STATE => self.trace_state.get(),
            SPAN_LINK_DROPPED_ATTRIBUTES_COUNT => self.dropped_attributes_count.get(),
            SPAN_LINK_FLAGS => self.flags.get(),
            SPAN_LINK_ATTRIBUTES => self.first_attribute.get(),
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
            SPAN_LINK_TRACE_ID => {
                if wire_type == wire_types::LEN {
                    self.trace_id.set(Some(range))
                }
            }
            SPAN_LINK_SPAN_ID => {
                if wire_type == wire_types::LEN {
                    self.span_id.set(Some(range))
                }
            }
            SPAN_LINK_TRACE_STATE => {
                if wire_type == wire_types::LEN {
                    self.trace_state.set(Some(range))
                }
            }
            SPAN_LINK_DROPPED_ATTRIBUTES_COUNT => {
                if wire_type == wire_types::VARINT {
                    self.dropped_attributes_count.set(Some(range))
                }
            }
            SPAN_LINK_FLAGS => {
                if wire_type == wire_types::FIXED32 {
                    self.flags.set(Some(range))
                }
            }
            SPAN_LINK_ATTRIBUTES => {
                if self.first_attribute.get().is_none() && wire_type == wire_types::LEN {
                    self.first_attribute.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/// Implementation of [`StatusView`] backed by protobuf serialized span `Status` message
pub struct RawSpanStatus<'a> {
    bytes_parser: ProtoBytesParser<'a, SpanStatusFieldRanges>,
}

struct SpanStatusFieldRanges {
    message: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
    code: Cell<Option<(NonZeroUsize, NonZeroUsize)>>,
}

impl FieldRanges for SpanStatusFieldRanges {
    fn new() -> Self {
        Self {
            message: Cell::new(None),
            code: Cell::new(None),
        }
    }

    fn get_field_range(&self, field_num: u64) -> Option<(usize, usize)> {
        let range = match field_num {
            SPAN_STATUS_MESSAGE => self.message.get(),
            SPAN_STATUS_CODE => self.code.get(),
            _ => None,
        };

        from_option_nonzero_range_to_primitive(range)
    }

    fn set_field_range(&self, field_num: u64, wire_type: u64, start: usize, end: usize) {
        let range = match to_nonzero_range(start, end) {
            Some(range) => range,
            None => return,
        };

        match field_num {
            SPAN_STATUS_MESSAGE => {
                if wire_type == wire_types::LEN {
                    self.message.set(Some(range))
                }
            }
            SPAN_STATUS_CODE => {
                if wire_type == wire_types::VARINT {
                    self.message.set(Some(range))
                }
            }
            _ => { /* ignore */ }
        }
    }
}

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

/// Iterator of ResourceSpans - produces implementation of `ResourceSpans` view from byte array
/// containing serialized `TracesData` message
pub struct ResourceSpansIter<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for ResourceSpansIter<'a> {
    type Item = RawResourceSpans<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.buf.len() {
            let (tag, next_pos) = read_varint(self.buf, self.pos)?;
            self.pos = next_pos;
            let field = tag >> 3;
            let wire_type = tag & 7;
            if field == TRACES_DATA_RESOURCE_SPANS && wire_type == wire_types::LEN {
                let (slice, next_pos) = read_len_delim(self.buf, self.pos)?;
                self.pos = next_pos;
                return Some(RawResourceSpans {
                    byte_parser: ProtoBytesParser::new(slice),
                });
            }
        }

        None
    }
}

/// Iterator of ScopeSpans - produces implementation of `ResourceSpans` view from byte array
/// containing a serialized ResourceSpans message
pub struct ScopeSpansIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, ResourceSpansFieldRanges>,
}

impl<'a> Iterator for ScopeSpansIter<'a> {
    type Item = RawScopeSpans<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;

        Some(RawScopeSpans {
            byte_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of Span - produces implementation of `Span` view from byte array containing a
/// serialized ScopeSpans message
pub struct SpanIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, ScopeSpansFieldRanges>,
}

impl<'a> Iterator for SpanIter<'a> {
    type Item = RawSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;

        Some(RawSpan {
            bytes_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of span Event - produces implementation of span `Event` view from byte array
/// serialized Span message
pub struct SpanEventIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, SpanFieldRanges>,
}

impl<'a> Iterator for SpanEventIter<'a> {
    type Item = RawSpanEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;

        Some(RawSpanEvent {
            bytes_parser: ProtoBytesParser::new(slice),
        })
    }
}

/// Iterator of span Link - produces implementation of span `Link` view from byte array
/// serialized Span message
pub struct SpanLinkIter<'a> {
    byte_parser: RepeatedFieldProtoBytesParser<'a, SpanFieldRanges>,
}

impl<'a> Iterator for SpanLinkIter<'a> {
    type Item = RawSpanLink<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.byte_parser.next()?;

        Some(RawSpanLink {
            bytes_parser: ProtoBytesParser::new(slice),
        })
    }
}

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */
impl TracesView for RawTraceData<'_> {
    type ResourceSpans<'res>
        = RawResourceSpans<'res>
    where
        Self: 'res;

    type ResourcesIter<'res>
        = ResourceSpansIter<'res>
    where
        Self: 'res;

    #[inline]
    fn resources(&self) -> Self::ResourcesIter<'_> {
        ResourceSpansIter {
            buf: self.buf,
            pos: 0,
        }
    }
}

impl ResourceSpansView for RawResourceSpans<'_> {
    type Resource<'res>
        = RawResource<'res>
    where
        Self: 'res;

    type ScopeSpans<'scp>
        = RawScopeSpans<'scp>
    where
        Self: 'scp;

    type ScopesIter<'scp>
        = ScopeSpansIter<'scp>
    where
        Self: 'scp;

    #[inline]
    fn resource(&self) -> Option<Self::Resource<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(RESOURCE_SPANS_RESOURCE)?;

        Some(RawResource::new(ProtoBytesParser::new(slice)))
    }

    #[inline]
    fn schema_url(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(RESOURCE_SPANS_SCHEMA_URL)?;

        std::str::from_utf8(slice).ok()
    }

    #[inline]
    fn scopes(&self) -> Self::ScopesIter<'_> {
        ScopeSpansIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                RESOURCE_SPANS_SCOPE_SPANS,
                wire_types::LEN,
            ),
        }
    }
}

impl ScopeSpansView for RawScopeSpans<'_> {
    type Span<'sp>
        = RawSpan<'sp>
    where
        Self: 'sp;

    type SpanIter<'sp>
        = SpanIter<'sp>
    where
        Self: 'sp;

    type Scope<'scp>
        = RawInstrumentationScope<'scp>
    where
        Self: 'scp;

    #[inline]
    fn schema_url(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .byte_parser
            .advance_to_find_field(SCOPE_SPANS_SCHEMA_URL)?;
        std::str::from_utf8(slice).ok()
    }

    #[inline]
    fn scope(&self) -> Option<Self::Scope<'_>> {
        let slice = self.byte_parser.advance_to_find_field(SCOPE_SPANS_SCOPE)?;
        Some(RawInstrumentationScope::new(ProtoBytesParser::new(slice)))
    }

    #[inline]
    fn spans(&self) -> Self::SpanIter<'_> {
        SpanIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.byte_parser,
                SCOPE_SPANS_SPANS,
                wire_types::LEN,
            ),
        }
    }
}

impl SpanView for RawSpan<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, SpanFieldRanges>
    where
        Self: 'att;

    type Event<'ev>
        = RawSpanEvent<'ev>
    where
        Self: 'ev;

    type EventsIter<'ev>
        = SpanEventIter<'ev>
    where
        Self: 'ev;

    type Link<'ln>
        = RawSpanLink<'ln>
    where
        Self: 'ln;

    type LinksIter<'ln>
        = SpanLinkIter<'ln>
    where
        Self: 'ln;

    type Status<'st>
        = RawSpanStatus<'st>
    where
        Self: 'st;

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.bytes_parser,
            SPAN_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_DROPPED_ATTRIBUTES_COUNT);
        read_dropped_count(slice)
    }

    #[inline]
    fn dropped_events_count(&self) -> u32 {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_DROPPED_EVENTS_COUNT);
        read_dropped_count(slice)
    }

    #[inline]
    fn dropped_links_count(&self) -> u32 {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_DROPPED_LINKS_COUNT);
        read_dropped_count(slice)
    }

    #[inline]
    fn end_time_unix_nano(&self) -> Option<u64> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_END_TIME_UNIX_NANO)?;
        let byte_arr: [u8; 8] = slice.try_into().ok()?;
        Some(u64::from_le_bytes(byte_arr))
    }

    #[inline]
    fn events(&self) -> Self::EventsIter<'_> {
        SpanEventIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.bytes_parser,
                SPAN_EVENTS,
                wire_types::LEN,
            ),
        }
    }

    #[inline]
    fn flags(&self) -> Option<u32> {
        let slice = self.bytes_parser.advance_to_find_field(SPAN_FLAGS)?;
        let byte_arr: [u8; 4] = slice.try_into().ok()?;
        Some(u32::from_le_bytes(byte_arr))
    }

    #[inline]
    fn kind(&self) -> i32 {
        self.bytes_parser
            .advance_to_find_field(SPAN_KIND)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(var, _)| SpanKind::try_from(var as i32).unwrap_or(SpanKind::Unspecified))
            .unwrap_or(SpanKind::Unspecified) as i32
    }

    #[inline]
    fn links(&self) -> Self::LinksIter<'_> {
        SpanLinkIter {
            byte_parser: RepeatedFieldProtoBytesParser::from_byte_parser(
                &self.bytes_parser,
                SPAN_LINKS,
                wire_types::LEN,
            ),
        }
    }

    #[inline]
    fn name(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self.bytes_parser.advance_to_find_field(SPAN_NAME)?;
        std::str::from_utf8(slice).ok()
    }

    #[inline]
    fn parent_span_id(&self) -> Option<&SpanId> {
        self.bytes_parser
            .advance_to_find_field(SPAN_PARENT_SPAN_ID)
            .and_then(|slice| slice.try_into().ok())
    }

    #[inline]
    fn span_id(&self) -> Option<&SpanId> {
        self.bytes_parser
            .advance_to_find_field(SPAN_SPAN_ID)
            .and_then(|slice| slice.try_into().ok())
    }

    #[inline]
    fn start_time_unix_nano(&self) -> Option<u64> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_START_TIME_UNIX_NANO)?;
        let byte_arr: [u8; 8] = slice.try_into().ok()?;
        Some(u64::from_le_bytes(byte_arr))
    }

    #[inline]
    fn status(&self) -> Option<Self::Status<'_>> {
        let slice = self.bytes_parser.advance_to_find_field(SPAN_STATUS)?;
        Some(RawSpanStatus {
            bytes_parser: ProtoBytesParser::new(slice),
        })
    }

    #[inline]
    fn trace_id(&self) -> Option<&TraceId> {
        self.bytes_parser
            .advance_to_find_field(SPAN_TRACE_ID)
            .and_then(|slice| slice.try_into().ok())
    }

    #[inline]
    fn trace_state(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self.bytes_parser.advance_to_find_field(SPAN_TRACE_STATE)?;
        std::str::from_utf8(slice).ok()
    }
}

impl EventView for RawSpanEvent<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, SpanEventFieldRanges>
    where
        Self: 'att;

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.bytes_parser,
            SPAN_EVENT_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_EVENT_DROPPED_ATTRIBUTES_COUNTS);
        read_dropped_count(slice)
    }

    #[inline]
    fn name(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self.bytes_parser.advance_to_find_field(SPAN_EVENT_NAME)?;
        std::str::from_utf8(slice).ok()
    }

    #[inline]
    fn time_unix_nano(&self) -> Option<u64> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_EVENT_TIME_UNIX_NANO)?;
        let byte_arr: [u8; 8] = slice.try_into().ok()?;
        Some(u64::from_le_bytes(byte_arr))
    }
}

impl LinkView for RawSpanLink<'_> {
    type Attribute<'att>
        = RawKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att, SpanLinkFieldRanges>
    where
        Self: 'att;

    #[inline]
    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(RepeatedFieldProtoBytesParser::from_byte_parser(
            &self.bytes_parser,
            SPAN_LINK_ATTRIBUTES,
            wire_types::LEN,
        ))
    }

    #[inline]
    fn dropped_attributes_count(&self) -> u32 {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_LINK_DROPPED_ATTRIBUTES_COUNT);
        read_dropped_count(slice)
    }

    #[inline]
    fn trace_id(&self) -> Option<&TraceId> {
        self.bytes_parser
            .advance_to_find_field(SPAN_LINK_TRACE_ID)
            .and_then(|slice| slice.try_into().ok())
    }

    #[inline]
    fn span_id(&self) -> Option<&SpanId> {
        self.bytes_parser
            .advance_to_find_field(SPAN_LINK_SPAN_ID)
            .and_then(|slice| slice.try_into().ok())
    }

    #[inline]
    fn trace_state(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_LINK_TRACE_STATE)?;
        std::str::from_utf8(slice).ok()
    }

    #[inline]
    fn flags(&self) -> Option<u32> {
        let slice = self.bytes_parser.advance_to_find_field(SPAN_LINK_FLAGS)?;
        let byte_arr: [u8; 4] = slice.try_into().ok()?;
        Some(u32::from_le_bytes(byte_arr))
    }
}

impl StatusView for RawSpanStatus<'_> {
    #[inline]
    fn status_code(&self) -> i32 {
        self.bytes_parser
            .advance_to_find_field(SPAN_STATUS_CODE)
            .and_then(|slice| read_varint(slice, 0))
            .map(|(var, _)| var as i32)
            .unwrap_or(StatusCode::Unset as i32)
    }

    #[inline]
    fn message(&self) -> Option<crate::views::common::Str<'_>> {
        let slice = self
            .bytes_parser
            .advance_to_find_field(SPAN_STATUS_MESSAGE)?;
        std::str::from_utf8(slice).ok()
    }
}
