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
use crate::views::common::{SpanId, Str, TraceId};
use crate::views::trace::{
    EventView, LinkView, ResourceSpansView, ScopeSpansView, SpanView, StatusView, TracesView,
};

/* ───────────────────────────── VIEW WRAPPERS (zero-alloc) ────────────── */

/// Lightweight wrapper around `ResourceSpans` that implements `ResourceSpansView`
#[derive(Clone, Copy)]
pub struct ObjResourceSpans<'a> {
    inner: &'a ResourceSpans,
}

/// Lightweight wrapper around `ScopeSpans` that implements `ScopeSpansView`
#[derive(Clone, Copy)]
pub struct ObjScopeSpans<'a> {
    inner: &'a ScopeSpans,
}

/// Lightweight wrapper around `Span` that implements `SpanView`
#[derive(Clone, Copy)]
pub struct ObjSpan<'a> {
    inner: &'a Span,
}

/// Lightweight wrapper around `Status` that implements `StatusView`
#[derive(Clone, Copy)]
pub struct ObjStatus<'a> {
    inner: &'a Status,
}

/// Lightweight wrapper around `Event` that implements `EventView`
#[derive(Clone, Copy)]
pub struct ObjEvent<'a> {
    inner: &'a Event,
}

/// Lightweight wrapper around `Link` that implements `LinkView`
#[derive(Clone, Copy)]
pub struct ObjLink<'a> {
    inner: &'a Link,
}

/* ───────────────────────────── ADAPTER ITERATORS ─────────────────────── */

// FIXME: surely these can all be consolidated into a single generic implementation....

/// Iterator of `ObjResourceSpans`. Used in the implementation of `TracesDataView` to get an iterator
/// of the resources contained in the trace data.
#[derive(Clone)]
pub struct ResourceIter<'a> {
    it: std::slice::Iter<'a, ResourceSpans>,
}

impl<'a> Iterator for ResourceIter<'a> {
    type Item = ObjResourceSpans<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|r| ObjResourceSpans { inner: r })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

/// Iterator of `ObjScopeSpans`. Used in the implementation of `ResourceSpansView` to get an iterator
/// of the scopes contained in the trace data.
#[derive(Clone)]
pub struct ScopeSpansIter<'a> {
    it: std::slice::Iter<'a, ScopeSpans>,
}

impl<'a> Iterator for ScopeSpansIter<'a> {
    type Item = ObjScopeSpans<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|r| ObjScopeSpans { inner: r })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

/// Iterator of `ObjSpan`. Used in the implementation of `ScopeSpansView` to get an iterator
/// of the scopes contained in the trace data.
#[derive(Clone)]
pub struct SpanIter<'a> {
    it: std::slice::Iter<'a, Span>,
}

impl<'a> Iterator for SpanIter<'a> {
    type Item = ObjSpan<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|r| ObjSpan { inner: r })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

/// Iterator of `ObjEvent`. Used in the implementation of `SpanView` to get an iterator
/// of the events contained in the trace data.
#[derive(Clone)]
pub struct EventIter<'a> {
    it: std::slice::Iter<'a, Event>,
}

impl<'a> Iterator for EventIter<'a> {
    type Item = ObjEvent<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|r| ObjEvent { inner: r })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

/// Iterator of `ObjLink`. Used in the implementation of `SpanView` to get an iterator
/// of the links contained in the trace data.
#[derive(Clone)]
pub struct LinkIter<'a> {
    it: std::slice::Iter<'a, Link>,
}

impl<'a> Iterator for LinkIter<'a> {
    type Item = ObjLink<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|r| ObjLink { inner: r })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

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
        ResourceIter {
            it: self.resource_spans.iter(),
        }
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
        ScopeSpansIter {
            it: self.inner.scope_spans.iter(),
        }
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
        SpanIter {
            it: self.inner.spans.iter(),
        }
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
        EventIter {
            it: self.inner.events.iter(),
        }
    }

    fn dropped_events_count(&self) -> u32 {
        self.inner.dropped_events_count
    }

    fn links(&self) -> Self::LinksIter<'_> {
        LinkIter {
            it: self.inner.links.iter(),
        }
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
