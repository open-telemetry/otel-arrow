// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of the pdata View traits for proto message structs
//! from otlp trace.proto.

use otel_arrow_rust::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, TracesData,
    span::{Event, Link},
};

use crate::otlp::proto::common::{
    KeyValueIter, ObjInstrumentationScope, ObjKeyValue, parse_span_id, parse_trace_id, read_str,
};
use crate::otlp::proto::resource::ObjResource;
use crate::otlp::proto::wrappers::{GenericIterator, GenericObj};
use crate::views::common::{SpanId, Str, TraceId};
use crate::views::trace::{
    EventView, LinkView, ResourceSpansView, ScopeSpansView, SpanView, StatusView, TracesView,
};

/* ───────────────────────────── VIEW WRAPPERS (zero-alloc) ────────────── */

/// A wrapper for `ResourceSpans`.
pub type ObjResourceSpans<'a> = GenericObj<'a, ResourceSpans>;

/// A wrapper for `ScopeSpans`.
pub type ObjScopeSpans<'a> = GenericObj<'a, ScopeSpans>;

/// A wrapper for `Span`.
pub type ObjSpan<'a> = GenericObj<'a, Span>;

/// A wrapper for `Status`.
pub type ObjStatus<'a> = GenericObj<'a, Status>;

/// A wrapper for `Event`.
pub type ObjEvent<'a> = GenericObj<'a, Event>;

/// A wrapper for `Link`.
pub type ObjLink<'a> = GenericObj<'a, Link>;

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

/// An iterator for `ObjResourceSpans`; it consumes a slice iterator of `ResourceSpans`
pub type ResourceIter<'a> = GenericIterator<'a, ResourceSpans, ObjResourceSpans<'a>>;

/// An iterator for `ObjScopeSpans`; it consumes a slice iterator of `ScopeSpans`
pub type ScopeSpansIter<'a> = GenericIterator<'a, ScopeSpans, ObjScopeSpans<'a>>;

/// An iterator for `ObjSpan`; it consumes a slice iterator of `Span`
pub type SpanIter<'a> = GenericIterator<'a, Span, ObjSpan<'a>>;

/// An iterator for `ObjEvent`; it consumes a slice iterator of `Event`
pub type EventIter<'a> = GenericIterator<'a, Event, ObjEvent<'a>>;

/// An iterator for `ObjLink`; it consumes a slice iterator of `Link`
pub type LinkIter<'a> = GenericIterator<'a, Link, ObjLink<'a>>;

/* ───────────────────────────── TRAIT IMPLEMENTATIONS ─────────────────── */

impl TracesView for TracesData {
    type ResourceSpans<'a>
        = ObjResourceSpans<'a>
    where
        Self: 'a;

    type ResourcesIter<'a>
        = ResourceIter<'a>
    where
        Self: 'a;

    fn resources(&self) -> Self::ResourcesIter<'_> {
        ResourceIter::new(self.resource_spans.iter())
    }
}

impl ResourceSpansView for ObjResourceSpans<'_> {
    type Resource<'res>
        = ObjResource<'res>
    where
        Self: 'res;

    type ScopeSpans<'scp>
        = ObjScopeSpans<'scp>
    where
        Self: 'scp;

    type ScopesIter<'scp>
        = ScopeSpansIter<'scp>
    where
        Self: 'scp;

    fn resource(&self) -> Option<Self::Resource<'_>> {
        self.inner.resource.as_ref().map(ObjResource::new)
    }

    fn scopes(&self) -> Self::ScopesIter<'_> {
        ScopeSpansIter::new(self.inner.scope_spans.iter())
    }

    fn schema_url(&self) -> Option<Str<'_>> {
        read_str(&self.inner.schema_url)
    }
}

impl ScopeSpansView for ObjScopeSpans<'_> {
    type Scope<'scp>
        = ObjInstrumentationScope<'scp>
    where
        Self: 'scp;

    type Span<'sp>
        = ObjSpan<'sp>
    where
        Self: 'sp;

    type SpanIter<'sp>
        = SpanIter<'sp>
    where
        Self: 'sp;

    fn scope(&self) -> Option<Self::Scope<'_>> {
        self.inner.scope.as_ref().map(ObjInstrumentationScope::new)
    }

    fn spans(&self) -> Self::SpanIter<'_> {
        SpanIter::new(self.inner.spans.iter())
    }

    fn schema_url(&self) -> Option<Str<'_>> {
        read_str(&self.inner.schema_url)
    }
}

impl SpanView for ObjSpan<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    type Event<'ev>
        = ObjEvent<'ev>
    where
        Self: 'ev;

    type EventsIter<'ev>
        = EventIter<'ev>
    where
        Self: 'ev;

    type Link<'ln>
        = ObjLink<'ln>
    where
        Self: 'ln;

    type LinksIter<'ln>
        = LinkIter<'ln>
    where
        Self: 'ln;

    type Status<'st>
        = ObjStatus<'st>
    where
        Self: 'st;

    fn trace_id(&self) -> Option<&TraceId> {
        parse_trace_id(&self.inner.trace_id)
    }

    fn span_id(&self) -> Option<&SpanId> {
        parse_span_id(&self.inner.span_id)
    }

    fn trace_state(&self) -> Option<Str<'_>> {
        read_str(&self.inner.trace_state)
    }

    fn parent_span_id(&self) -> Option<&SpanId> {
        parse_span_id(&self.inner.parent_span_id)
    }

    fn flags(&self) -> Option<u32> {
        let flags = self.inner.flags;
        (flags != 0).then_some(flags)
    }

    fn name(&self) -> Option<Str<'_>> {
        read_str(&self.inner.name)
    }

    fn kind(&self) -> i32 {
        self.inner.kind
    }

    fn start_time_unix_nano(&self) -> Option<u64> {
        let time = self.inner.start_time_unix_nano;
        (time != 0).then_some(time)
    }

    fn end_time_unix_nano(&self) -> Option<u64> {
        let time = self.inner.end_time_unix_nano;
        (time != 0).then_some(time)
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn dropped_attributes_count(&self) -> u32 {
        self.inner.dropped_attributes_count
    }

    fn events(&self) -> Self::EventsIter<'_> {
        EventIter::new(self.inner.events.iter())
    }

    fn dropped_events_count(&self) -> u32 {
        self.inner.dropped_events_count
    }

    fn links(&self) -> Self::LinksIter<'_> {
        LinkIter::new(self.inner.links.iter())
    }

    fn dropped_links_count(&self) -> u32 {
        self.inner.dropped_links_count
    }

    fn status(&self) -> Option<Self::Status<'_>> {
        self.inner
            .status
            .as_ref()
            .map(|status| ObjStatus { inner: status })
    }
}

impl StatusView for ObjStatus<'_> {
    fn message(&self) -> Option<Str<'_>> {
        read_str(&self.inner.message)
    }

    fn status_code(&self) -> i32 {
        self.inner.code
    }
}

impl EventView for ObjEvent<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    fn time_unix_nano(&self) -> Option<u64> {
        let time = self.inner.time_unix_nano;
        (time != 0).then_some(time)
    }

    fn name(&self) -> Option<Str<'_>> {
        read_str(&self.inner.name)
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn dropped_attributes_count(&self) -> u32 {
        self.inner.dropped_attributes_count
    }
}

impl LinkView for ObjLink<'_> {
    type Attribute<'att>
        = ObjKeyValue<'att>
    where
        Self: 'att;

    type AttributeIter<'att>
        = KeyValueIter<'att>
    where
        Self: 'att;

    fn trace_id(&self) -> Option<&TraceId> {
        parse_trace_id(&self.inner.trace_id)
    }

    fn span_id(&self) -> Option<&SpanId> {
        parse_span_id(&self.inner.span_id)
    }

    fn trace_state(&self) -> Option<Str<'_>> {
        read_str(&self.inner.trace_state)
    }

    fn attributes(&self) -> Self::AttributeIter<'_> {
        KeyValueIter::new(self.inner.attributes.iter())
    }

    fn dropped_attributes_count(&self) -> u32 {
        self.inner.dropped_attributes_count
    }

    fn flags(&self) -> Option<u32> {
        let flags = self.inner.flags;
        (flags != 0).then_some(flags)
    }
}
