// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! **Backend-agnostic, zero-copy view traits for OTLP Traces.**
//! ```text
//! TracesView
//! └─ ResourceSpansView
//!    │  resource::ResourceView
//!    └─ ScopeSpansView
//!       │  common::InstrumentationScopeView
//!       └─ SpanView
//!          ├─ common::AttributeView
//!          │  └─ common::AnyValueView
//!          ├─ EventView
//!          │  └─ common::AttributeView
//!          │     └─ common::AnyValueView
//!          ├─ LinkView
//!          │  └─ common::AttributeView
//!          │     └─ common::AnyValueView
//!          └─ StatusView
//! ```

use crate::views::{
    common::{AttributeView, InstrumentationScopeView, SpanId, Str, TraceId},
    resource::ResourceView,
};

/// View for top level TracesData
pub trait TracesView {
    /// The `ResourceSpansView` trait associated with this impl of the TracesView trait
    type ResourceSpans<'res>: ResourceSpansView
    where
        Self: 'res;

    /// The associated iterator type for this implementation of the trait. The iterator will yield
    /// borrowed references that must live as long as the input lifetime 'res
    type ResourcesIter<'res>: Iterator<Item = Self::ResourceSpans<'res>>
    where
        Self: 'res;

    /// Iterator yielding borrowed references for resources associated with this `LogsData`
    fn resources(&self) -> Self::ResourcesIter<'_>;
}

/// View for ResourceSpans
pub trait ResourceSpansView {
    /// The `ResourceView` trait associated with this impl of the `ResourceSpansView` trait.
    type Resource<'res>: ResourceView
    where
        Self: 'res;

    /// The `ScopeSpansView` trait associated with this impl of the `ResourceSpansView`.
    type ScopeSpans<'scp>: ScopeSpansView
    where
        Self: 'scp;

    /// The associated iterator type for this impl of the the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'scp
    type ScopesIter<'scp>: Iterator<Item = Self::ScopeSpans<'scp>>
    where
        Self: 'scp;

    /// Access the resource for the scopes contained in the `ScopeSpans`. If this returns `None` it
    /// means the resource info is unknown.
    fn resource(&self) -> Option<Self::Resource<'_>>;

    /// Iterator yielding the `ScopeSpans` that originate from this Resource.
    fn scopes(&self) -> Self::ScopesIter<'_>;

    /// The schema URL for the resource. If the schema is not known, this returns `None`
    fn schema_url(&self) -> Option<Str<'_>>;
}

/// View for ScopeSpans
pub trait ScopeSpansView {
    /// The `InstrumentationScopeView` trait associated with this impl of the `ScopeSpansView` trait.
    type Scope<'scp>: InstrumentationScopeView
    where
        Self: 'scp;

    /// The `SpanView` trait associated with this impl of the `ScopeSpansView` trait.
    type Span<'sp>: SpanView
    where
        Self: 'sp;

    /// The associated iterator type for this impl of the the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'sp
    type SpanIter<'sp>: Iterator<Item = Self::Span<'sp>>
    where
        Self: 'sp;

    /// Access the instrumentation scope for the spans contained in this scope. If this returns
    /// `None` it means the scope is unknown
    fn scope(&self) -> Option<Self::Scope<'_>>;

    /// Iterator yielding `Span`s
    fn spans(&self) -> Self::SpanIter<'_>;

    /// The schema URL. This schema URL applies to all spans returned by the `spans` iterator.
    /// This method returns `None` if the schema URL is not known.
    fn schema_url(&self) -> Option<Str<'_>>;
}

/// View for Spans
pub trait SpanView {
    /// The `AttributeView` type associated with this impl of the `SpanView` trait for
    /// accessing this Span's attributes
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// The associated attribute iterator type for this impl of the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'att
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// The `EventView` trait associated with this impl of the `SpanView` trait.
    type Event<'ev>: EventView
    where
        Self: 'ev;

    /// The associated iterator type for this impl of the the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'scp
    type EventsIter<'ev>: Iterator<Item = Self::Event<'ev>>
    where
        Self: 'ev;

    /// The `LinkView` trait associated with this impl of the `SpanView` trait.
    type Link<'ln>: LinkView
    where
        Self: 'ln;

    /// The associated iterator type for this impl of the the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'scp
    type LinksIter<'ln>: Iterator<Item = Self::Link<'ln>>
    where
        Self: 'ln;

    /// The `StatusView` trait associated with this impl of the `SpanView` trait.
    type Status<'st>: StatusView
    where
        Self: 'st;

    /// Access the trace ID. Should return None if the underlying trace ID is missing, or if the
    /// backend representation of the trace ID is invalid. Invalid trace IDs include all-0s or a
    /// trace ID of incorrect length (length != 16)
    fn trace_id(&self) -> Option<&TraceId>;

    /// Access the span ID. Should return None if the underlying span ID is missing or if the
    /// backend representation of the span ID is invalid. Invalid span IDs include all-0s or a
    /// span ID or incorrect length (length != 8)
    fn span_id(&self) -> Option<&SpanId>;

    /// Access the trace state
    fn trace_state(&self) -> Option<Str<'_>>;

    /// Access the parent span ID. Should return None if the underlying span ID is missing or if the
    /// backend representation of the span ID is invalid. Invalid span IDs include all-0s or a span
    /// ID or incorrect length (length != 8)
    fn parent_span_id(&self) -> Option<&SpanId>;

    /// Access the span's flags
    // FIXME: why Option here?
    fn flags(&self) -> Option<u32>;

    /// Access the span's name
    fn name(&self) -> Option<Str<'_>>;

    // FIXME: is this the best way to handle enums like SpanKind and StatusCode? We're just passing
    // raw ints from the proto repr on theory that the consumer can cast them to the corresponding
    // prost enum so we don't have to establish a dependency on prost code.

    /// Access the Span kind
    fn kind(&self) -> i32;

    /// Access the start time of the Span
    fn start_time_unix_nano(&self) -> Option<u64>;

    /// Access the end time of the Span
    fn end_time_unix_nano(&self) -> Option<u64>;

    /// Access the span's attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access this span's dropped attributes. The value is 0 when no attributes were dropped.
    fn dropped_attributes_count(&self) -> u32;

    /// Iterator yielding `Events`s
    fn events(&self) -> Self::EventsIter<'_>;

    /// Access this span's dropped events count
    fn dropped_events_count(&self) -> u32;

    /// Iterator yielding `Links`s
    fn links(&self) -> Self::LinksIter<'_>;

    /// Access this span's dropped links count
    fn dropped_links_count(&self) -> u32;

    /// Access this span's Status
    fn status(&self) -> Option<Self::Status<'_>>;
}

/// View for an Event
pub trait EventView {
    /// The `AttributeView` type associated with this impl of the `EventView` trait for
    /// accessing this Event's attributes
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// The associated attribute iterator type for this impl of the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'att
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// Access this event's time
    fn time_unix_nano(&self) -> Option<u64>;

    /// Access this event's name
    fn name(&self) -> Option<Str<'_>>;

    /// Access the event's attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access this event's dropped attributes. The value is 0 when no attributes were dropped.
    fn dropped_attributes_count(&self) -> u32;
}

/// View for a Link
pub trait LinkView {
    /// The `AttributeView` type associated with this impl of the `LinkView` trait for
    /// accessing this Link's attributes
    type Attribute<'att>: AttributeView
    where
        Self: 'att;

    /// The associated attribute iterator type for this impl of the trait. The iterator will yield
    /// borrowed references that must live as long as the lifetime 'att
    type AttributeIter<'att>: Iterator<Item = Self::Attribute<'att>>
    where
        Self: 'att;

    /// Access the trace ID. Should return None if the underlying trace ID is missing, or if the
    /// backend representation of the trace ID is invalid. Invalid trace IDs include all-0s or a
    /// trace ID of incorrect length (length != 16)
    fn trace_id(&self) -> Option<&TraceId>;

    /// Access the span ID. Should return None if the underlying span ID is missing or if the
    /// backend representation of the span ID is invalid. Invalid span IDs include all-0s or a
    /// span ID or incorrect length (length != 8)
    fn span_id(&self) -> Option<&SpanId>;

    /// Access the link's trace state
    fn trace_state(&self) -> Option<Str<'_>>;

    /// Access the link's attributes
    fn attributes(&self) -> Self::AttributeIter<'_>;

    /// Access this event's dropped attributes. The value is 0 when no attributes were dropped.
    fn dropped_attributes_count(&self) -> u32;

    /// Access the link's flags
    fn flags(&self) -> Option<u32>;
}

/// View for a Span's Status
pub trait StatusView {
    /// Access the status message
    fn message(&self) -> Option<Str<'_>>;

    /// Access the numeric status code
    fn status_code(&self) -> i32;
}
