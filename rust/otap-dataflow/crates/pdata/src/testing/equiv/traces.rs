// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trace equivalence checking.

use crate::proto::opentelemetry::trace::v1::TracesData;
use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::proto::opentelemetry::trace::v1::Span;
use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::canonical::compare_attributes;

/// A key that uniquely identifies a span within a trace request.
///
/// This struct holds references to the parent structures (resource, scope) and the span itself.
/// The Ord implementation defines canonical ordering for comparison.
#[derive(Debug, Clone)]
struct SpanItemKey<'a> {
    resource: Option<&'a Resource>,
    resource_schema_url: &'a str,
    scope: Option<&'a InstrumentationScope>,
    scope_schema_url: &'a str,
    span: &'a Span,
}

impl<'a> PartialEq for SpanItemKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<'a> Eq for SpanItemKey<'a> {}

impl<'a> PartialOrd for SpanItemKey<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for SpanItemKey<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare resource
        match (self.resource, other.resource) {
            (Some(r1), Some(r2)) => {
                match compare_attributes(&r1.attributes, &r2.attributes) {
                    Ordering::Equal => {}
                    other => return other,
                }
            }
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => {}
        }

        // Compare resource_schema_url
        match self.resource_schema_url.cmp(other.resource_schema_url) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare scope
        match (self.scope, other.scope) {
            (Some(s1), Some(s2)) => {
                match s1.name.cmp(&s2.name) {
                    Ordering::Equal => {}
                    other => return other,
                }
                match s1.version.cmp(&s2.version) {
                    Ordering::Equal => {}
                    other => return other,
                }
                match compare_attributes(&s1.attributes, &s2.attributes) {
                    Ordering::Equal => {}
                    other => return other,
                }
                match s1.dropped_attributes_count.cmp(&s2.dropped_attributes_count) {
                    Ordering::Equal => {}
                    other => return other,
                }
            }
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => {}
        }

        // Compare scope_schema_url
        match self.scope_schema_url.cmp(other.scope_schema_url) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare span fields
        self.span
            .trace_id
            .cmp(&other.span.trace_id)
            .then_with(|| self.span.span_id.cmp(&other.span.span_id))
            .then_with(|| self.span.trace_state.cmp(&other.span.trace_state))
            .then_with(|| self.span.parent_span_id.cmp(&other.span.parent_span_id))
            .then_with(|| self.span.flags.cmp(&other.span.flags))
            .then_with(|| self.span.name.cmp(&other.span.name))
            .then_with(|| self.span.kind.cmp(&other.span.kind))
            .then_with(|| {
                self.span
                    .start_time_unix_nano
                    .cmp(&other.span.start_time_unix_nano)
            })
            .then_with(|| {
                self.span
                    .end_time_unix_nano
                    .cmp(&other.span.end_time_unix_nano)
            })
            .then_with(|| compare_attributes(&self.span.attributes, &other.span.attributes))
            .then_with(|| {
                self.span
                    .dropped_attributes_count
                    .cmp(&other.span.dropped_attributes_count)
            })
            .then_with(|| compare_span_events(&self.span.events, &other.span.events))
            .then_with(|| {
                self.span
                    .dropped_events_count
                    .cmp(&other.span.dropped_events_count)
            })
            .then_with(|| compare_span_links(&self.span.links, &other.span.links))
            .then_with(|| {
                self.span
                    .dropped_links_count
                    .cmp(&other.span.dropped_links_count)
            })
            .then_with(|| compare_span_status(&self.span.status, &other.span.status))
    }
}

/// Compare span events in canonical order (sorted by time, name, attributes)
fn compare_span_events(
    a: &[crate::proto::opentelemetry::trace::v1::span::Event],
    b: &[crate::proto::opentelemetry::trace::v1::span::Event],
) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| {
        for (ea, eb) in a.iter().zip(b.iter()) {
            let ord = ea
                .time_unix_nano
                .cmp(&eb.time_unix_nano)
                .then_with(|| ea.name.cmp(&eb.name))
                .then_with(|| compare_attributes(&ea.attributes, &eb.attributes))
                .then_with(|| ea.dropped_attributes_count.cmp(&eb.dropped_attributes_count));
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    })
}

/// Compare span links in canonical order (sorted by trace_id, span_id, etc.)
fn compare_span_links(
    a: &[crate::proto::opentelemetry::trace::v1::span::Link],
    b: &[crate::proto::opentelemetry::trace::v1::span::Link],
) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| {
        for (la, lb) in a.iter().zip(b.iter()) {
            let ord = la
                .trace_id
                .cmp(&lb.trace_id)
                .then_with(|| la.span_id.cmp(&lb.span_id))
                .then_with(|| la.trace_state.cmp(&lb.trace_state))
                .then_with(|| compare_attributes(&la.attributes, &lb.attributes))
                .then_with(|| la.dropped_attributes_count.cmp(&lb.dropped_attributes_count))
                .then_with(|| la.flags.cmp(&lb.flags));
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    })
}

/// Compare span status
fn compare_span_status(
    a: &Option<crate::proto::opentelemetry::trace::v1::Status>,
    b: &Option<crate::proto::opentelemetry::trace::v1::Status>,
) -> Ordering {
    match (a, b) {
        (Some(s1), Some(s2)) => s1
            .message
            .cmp(&s2.message)
            .then_with(|| s1.code.cmp(&s2.code)),
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Flatten trace hierarchy into an iterator of SpanItemKeys
fn iter_span_items(request: &TracesData) -> impl Iterator<Item = SpanItemKey<'_>> {
    request.resource_spans.iter().flat_map(|rs| {
        let resource = rs.resource.as_ref();
        let resource_schema_url = rs.schema_url.as_str();

        rs.scope_spans.iter().flat_map(move |ss| {
            let scope = ss.scope.as_ref();
            let scope_schema_url = ss.schema_url.as_str();

            ss.spans.iter().map(move |span| SpanItemKey {
                resource,
                resource_schema_url,
                scope,
                scope_schema_url,
                span,
            })
        })
    })
}

/// Assert that two trace requests are semantically equivalent.
///
/// This compares the flattened span items from both requests, treating them as sets.
/// Spans are compared in canonical order with attributes sorted by key.
pub fn assert_traces_equivalent(
    expected: &[TracesData],
    actual: &[TracesData],
) {
    let expected_items: BTreeSet<_> = expected.iter().flat_map(iter_span_items).collect();
    let actual_items: BTreeSet<_> = actual.iter().flat_map(iter_span_items).collect();

    let missing: Vec<_> = expected_items.difference(&actual_items).collect();
    let unexpected: Vec<_> = actual_items.difference(&expected_items).collect();

    if !missing.is_empty() || !unexpected.is_empty() {
        let mut msg = String::from("Trace requests are not equivalent:\n");

        if !missing.is_empty() {
            msg.push_str(&format!("Missing {} spans:\n", missing.len()));
            for item in missing.iter().take(5) {
                msg.push_str(&format!(
                    "  - span_id={:?}, name={}, trace_id={:?}\n",
                    item.span.span_id, item.span.name, item.span.trace_id
                ));
            }
            if missing.len() > 5 {
                msg.push_str(&format!("  ... and {} more\n", missing.len() - 5));
            }
        }

        if !unexpected.is_empty() {
            msg.push_str(&format!("Unexpected {} spans:\n", unexpected.len()));
            for item in unexpected.iter().take(5) {
                msg.push_str(&format!(
                    "  + span_id={:?}, name={}, trace_id={:?}\n",
                    item.span.span_id, item.span.name, item.span.trace_id
                ));
            }
            if unexpected.len() > 5 {
                msg.push_str(&format!("  ... and {} more\n", unexpected.len() - 5));
            }
        }

        panic!("{}", msg);
    }
}
