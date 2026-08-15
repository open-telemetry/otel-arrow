// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP JSON serialization for traces pdata views.
//!
//! The encoder walks resources, scopes, spans, events, links, and status values through the trace
//! view traits. It streams the OTLP protobuf JSON representation directly to serde, including the
//! required hexadecimal identifiers and numeric enum values, without intermediate materialization.

use super::common::{AttributeIterJson, HexId, ProtoU64, ResourceJson, ScopeJson, Utf8};
use super::{JsonEncodeError, write_json};
use otap_df_pdata_views::views::trace::{
    EventView, LinkView, ResourceSpansView, ScopeSpansView, SpanView, StatusView, TracesView,
};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::io::Write;

/// Writes one traces pdata view as a compact OTLP JSON document.
///
/// This function does not add a delimiter, bound output size, or roll back bytes already
/// accepted by the writer when serialization fails. Callers own those policies.
pub fn write_traces_json<T: TracesView, W: Write>(
    traces: &T,
    output: &mut W,
) -> Result<(), JsonEncodeError> {
    write_json(&TracesJson(traces), output)
}

struct TracesJson<'a, T: TracesView>(&'a T);

impl<T: TracesView> Serialize for TracesJson<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.0.resources().next().is_some() {
            map.serialize_entry("resourceSpans", &ResourceSpansList(self.0))?;
        }
        map.end()
    }
}

struct ResourceSpansList<'a, T: TracesView>(&'a T);

impl<T: TracesView> Serialize for ResourceSpansList<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for resource in self.0.resources() {
            sequence.serialize_element(&ResourceSpansJson(resource))?;
        }
        sequence.end()
    }
}

struct ResourceSpansJson<R: ResourceSpansView>(R);

impl<R: ResourceSpansView> Serialize for ResourceSpansJson<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(resource) = self.0.resource() {
            map.serialize_entry("resource", &ResourceJson(&resource))?;
        }
        if self.0.scopes().next().is_some() {
            map.serialize_entry("scopeSpans", &ScopeSpansList(&self.0))?;
        }
        if let Some(schema_url) = self.0.schema_url().filter(|value| !value.is_empty()) {
            map.serialize_entry("schemaUrl", &Utf8(schema_url))?;
        }
        map.end()
    }
}

struct ScopeSpansList<'a, R: ResourceSpansView>(&'a R);

impl<R: ResourceSpansView> Serialize for ScopeSpansList<'_, R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for scope in self.0.scopes() {
            sequence.serialize_element(&ScopeSpansJson(scope))?;
        }
        sequence.end()
    }
}

struct ScopeSpansJson<T: ScopeSpansView>(T);

impl<T: ScopeSpansView> Serialize for ScopeSpansJson<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(scope) = self.0.scope() {
            map.serialize_entry("scope", &ScopeJson(&scope))?;
        }
        if self.0.spans().next().is_some() {
            map.serialize_entry("spans", &SpanList(&self.0))?;
        }
        if let Some(schema_url) = self.0.schema_url().filter(|value| !value.is_empty()) {
            map.serialize_entry("schemaUrl", &Utf8(schema_url))?;
        }
        map.end()
    }
}

struct SpanList<'a, T: ScopeSpansView>(&'a T);

impl<T: ScopeSpansView> Serialize for SpanList<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for span in self.0.spans() {
            sequence.serialize_element(&SpanJson(span))?;
        }
        sequence.end()
    }
}

struct SpanJson<T: SpanView>(T);

impl<T: SpanView> Serialize for SpanJson<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(trace_id) = self.0.trace_id() {
            map.serialize_entry("traceId", &HexId(trace_id))?;
        }
        if let Some(span_id) = self.0.span_id() {
            map.serialize_entry("spanId", &HexId(span_id))?;
        }
        if let Some(value) = self.0.trace_state().filter(|value| !value.is_empty()) {
            map.serialize_entry("traceState", &Utf8(value))?;
        }
        if let Some(parent_span_id) = self.0.parent_span_id() {
            map.serialize_entry("parentSpanId", &HexId(parent_span_id))?;
        }
        if let Some(flags) = self.0.flags().filter(|value| *value != 0) {
            map.serialize_entry("flags", &flags)?;
        }
        if let Some(value) = self.0.name().filter(|value| !value.is_empty()) {
            map.serialize_entry("name", &Utf8(value))?;
        }
        let kind = self.0.kind();
        if kind != 0 {
            map.serialize_entry("kind", &kind)?;
        }
        if let Some(value) = self.0.start_time_unix_nano().filter(|value| *value != 0) {
            map.serialize_entry("startTimeUnixNano", &ProtoU64(value))?;
        }
        if let Some(value) = self.0.end_time_unix_nano().filter(|value| *value != 0) {
            map.serialize_entry("endTimeUnixNano", &ProtoU64(value))?;
        }
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        let dropped_attributes = self.0.dropped_attributes_count();
        if dropped_attributes != 0 {
            map.serialize_entry("droppedAttributesCount", &dropped_attributes)?;
        }
        if self.0.events().next().is_some() {
            map.serialize_entry("events", &EventList(&self.0))?;
        }
        let dropped_events = self.0.dropped_events_count();
        if dropped_events != 0 {
            map.serialize_entry("droppedEventsCount", &dropped_events)?;
        }
        if self.0.links().next().is_some() {
            map.serialize_entry("links", &LinkList(&self.0))?;
        }
        let dropped_links = self.0.dropped_links_count();
        if dropped_links != 0 {
            map.serialize_entry("droppedLinksCount", &dropped_links)?;
        }
        if let Some(status) = self.0.status() {
            map.serialize_entry("status", &StatusJson(status))?;
        }
        map.end()
    }
}

struct EventList<'a, T: SpanView>(&'a T);

impl<T: SpanView> Serialize for EventList<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for event in self.0.events() {
            sequence.serialize_element(&EventJson(event))?;
        }
        sequence.end()
    }
}

struct EventJson<E: EventView>(E);

impl<E: EventView> Serialize for EventJson<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(value) = self.0.time_unix_nano().filter(|value| *value != 0) {
            map.serialize_entry("timeUnixNano", &ProtoU64(value))?;
        }
        if let Some(value) = self.0.name().filter(|value| !value.is_empty()) {
            map.serialize_entry("name", &Utf8(value))?;
        }
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        let dropped = self.0.dropped_attributes_count();
        if dropped != 0 {
            map.serialize_entry("droppedAttributesCount", &dropped)?;
        }
        map.end()
    }
}

struct LinkList<'a, T: SpanView>(&'a T);

impl<T: SpanView> Serialize for LinkList<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for link in self.0.links() {
            sequence.serialize_element(&LinkJson(link))?;
        }
        sequence.end()
    }
}

struct LinkJson<L: LinkView>(L);

impl<L: LinkView> Serialize for LinkJson<L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(trace_id) = self.0.trace_id() {
            map.serialize_entry("traceId", &HexId(trace_id))?;
        }
        if let Some(span_id) = self.0.span_id() {
            map.serialize_entry("spanId", &HexId(span_id))?;
        }
        if let Some(value) = self.0.trace_state().filter(|value| !value.is_empty()) {
            map.serialize_entry("traceState", &Utf8(value))?;
        }
        if self.0.attributes().next().is_some() {
            map.serialize_entry("attributes", &AttributeIterJson::new(self.0.attributes()))?;
        }
        let dropped = self.0.dropped_attributes_count();
        if dropped != 0 {
            map.serialize_entry("droppedAttributesCount", &dropped)?;
        }
        if let Some(flags) = self.0.flags().filter(|value| *value != 0) {
            map.serialize_entry("flags", &flags)?;
        }
        map.end()
    }
}

struct StatusJson<T: StatusView>(T);

impl<T: StatusView> Serialize for StatusJson<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if let Some(value) = self.0.message().filter(|value| !value.is_empty()) {
            map.serialize_entry("message", &Utf8(value))?;
        }
        let code = self.0.status_code();
        if code != 0 {
            map.serialize_entry("code", &code)?;
        }
        map.end()
    }
}
