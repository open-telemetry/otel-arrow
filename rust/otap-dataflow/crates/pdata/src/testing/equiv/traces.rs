// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Trace equivalence checking.

use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Status, TracesData};
use crate::testing::equiv::canonical::{
    assert_equivalent, canonicalize_any_value, canonicalize_vec,
};

/// Split a TracesData into individual singleton TracesData messages (one per span).
fn traces_split_into_singletons(traces_data: &TracesData) -> Vec<TracesData> {
    let mut result = Vec::new();

    for resource_spans in &traces_data.resource_spans {
        for scope_spans in &resource_spans.scope_spans {
            for span in &scope_spans.spans {
                result.push(TracesData {
                    resource_spans: vec![ResourceSpans {
                        resource: resource_spans.resource.clone(),
                        scope_spans: vec![ScopeSpans {
                            scope: scope_spans.scope.clone(),
                            spans: vec![span.clone()],
                            schema_url: scope_spans.schema_url.clone(),
                        }],
                        schema_url: resource_spans.schema_url.clone(),
                    }],
                });
            }
        }
    }

    result
}

/// Canonicalize a singleton TracesData by sorting all fields that need canonical ordering.
fn traces_canonicalize_singleton_in_place(traces_data: &mut TracesData) {
    // Canonicalize resource attributes
    for resource_spans in &mut traces_data.resource_spans {
        if let Some(resource) = &mut resource_spans.resource {
            canonicalize_vec(&mut resource.attributes, |attr| {
                if let Some(value) = &mut attr.value {
                    canonicalize_any_value(value);
                }
            });
        }

        // Canonicalize scope attributes
        for scope_spans in &mut resource_spans.scope_spans {
            if let Some(scope) = &mut scope_spans.scope {
                canonicalize_vec(&mut scope.attributes, |attr| {
                    if let Some(value) = &mut attr.value {
                        canonicalize_any_value(value);
                    }
                });
            }

            // Canonicalize span fields
            for span in &mut scope_spans.spans {
                // When encoding OTLP -> OTAP -> OTLP, spans without
                // Status=None are round-trip encoded with a Status struct
                // containing all-default values (code=0, message="").
                //
                // See: encode/mod.rs encode_spans_otap_batch.
                //
                // TODO: For now, we normalize by removing message
                // values equal to the default.
                if let Some(status) = &span.status {
                    if *status == Status::default() {
                        span.status = None;
                    }
                }

                // Canonicalize span attributes
                canonicalize_vec(&mut span.attributes, |attr| {
                    if let Some(value) = &mut attr.value {
                        canonicalize_any_value(value);
                    }
                });

                // Canonicalize span events
                canonicalize_vec(&mut span.events, |event| {
                    canonicalize_vec(&mut event.attributes, |attr| {
                        if let Some(value) = &mut attr.value {
                            canonicalize_any_value(value);
                        }
                    });
                });

                // Canonicalize span links
                canonicalize_vec(&mut span.links, |link| {
                    canonicalize_vec(&mut link.attributes, |attr| {
                        if let Some(value) = &mut attr.value {
                            canonicalize_any_value(value);
                        }
                    });
                });
            }
        }
    }
}

/// Assert that two collections of `TracesData` instances are equivalent.
pub fn assert_traces_equivalent(left: &[TracesData], right: &[TracesData]) {
    assert_equivalent(
        left,
        right,
        traces_split_into_singletons,
        traces_canonicalize_singleton_in_place,
        "TracesData",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status,
        span::{Event, Link},
    };

    #[test]
    fn test_traces_equivalent_with_unordered_fields() {
        // Test that spans with attributes, events, and links in different orders are considered equivalent
        let request1 = TracesData {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue::new("service", AnyValue::new_string("test")),
                        KeyValue::new("version", AnyValue::new_string("1.0")),
                    ],
                    ..Default::default()
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "scope1".into(),
                        attributes: vec![
                            KeyValue::new("scope.key1", AnyValue::new_string("value1")),
                            KeyValue::new("scope.key2", AnyValue::new_int(42)),
                        ],
                        ..Default::default()
                    }),
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        name: "test-span".into(),
                        start_time_unix_nano: 1000,
                        end_time_unix_nano: 2000,
                        attributes: vec![
                            KeyValue::new("span.key1", AnyValue::new_string("value1")),
                            KeyValue::new("span.key2", AnyValue::new_int(100)),
                        ],
                        events: vec![
                            Event {
                                time_unix_nano: 1500,
                                name: "event1".into(),
                                attributes: vec![
                                    KeyValue::new("event.key1", AnyValue::new_string("ev1")),
                                    KeyValue::new("event.key2", AnyValue::new_bool(true)),
                                ],
                                ..Default::default()
                            },
                            Event {
                                time_unix_nano: 1600,
                                name: "event2".into(),
                                attributes: vec![KeyValue::new(
                                    "event.key3",
                                    AnyValue::new_int(99),
                                )],
                                ..Default::default()
                            },
                        ],
                        links: vec![
                            Link {
                                trace_id: vec![
                                    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
                                ],
                                span_id: vec![10, 11, 12, 13, 14, 15, 16, 17],
                                attributes: vec![KeyValue::new(
                                    "link.key1",
                                    AnyValue::new_string("link1"),
                                )],
                                ..Default::default()
                            },
                            Link {
                                trace_id: vec![
                                    20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
                                ],
                                span_id: vec![20, 21, 22, 23, 24, 25, 26, 27],
                                attributes: vec![KeyValue::new(
                                    "link.key2",
                                    AnyValue::new_int(200),
                                )],
                                ..Default::default()
                            },
                        ],
                        status: Some(Status {
                            code: 1,
                            message: "OK".into(),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        // Same data but with all Vec fields in different order
        let request2 = TracesData {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue::new("version", AnyValue::new_string("1.0")),
                        KeyValue::new("service", AnyValue::new_string("test")),
                    ],
                    ..Default::default()
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "scope1".into(),
                        attributes: vec![
                            KeyValue::new("scope.key2", AnyValue::new_int(42)),
                            KeyValue::new("scope.key1", AnyValue::new_string("value1")),
                        ],
                        ..Default::default()
                    }),
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        name: "test-span".into(),
                        start_time_unix_nano: 1000,
                        end_time_unix_nano: 2000,
                        attributes: vec![
                            KeyValue::new("span.key2", AnyValue::new_int(100)),
                            KeyValue::new("span.key1", AnyValue::new_string("value1")),
                        ],
                        events: vec![
                            Event {
                                time_unix_nano: 1600,
                                name: "event2".into(),
                                attributes: vec![KeyValue::new(
                                    "event.key3",
                                    AnyValue::new_int(99),
                                )],
                                ..Default::default()
                            },
                            Event {
                                time_unix_nano: 1500,
                                name: "event1".into(),
                                attributes: vec![
                                    KeyValue::new("event.key2", AnyValue::new_bool(true)),
                                    KeyValue::new("event.key1", AnyValue::new_string("ev1")),
                                ],
                                ..Default::default()
                            },
                        ],
                        links: vec![
                            Link {
                                trace_id: vec![
                                    20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
                                ],
                                span_id: vec![20, 21, 22, 23, 24, 25, 26, 27],
                                attributes: vec![KeyValue::new(
                                    "link.key2",
                                    AnyValue::new_int(200),
                                )],
                                ..Default::default()
                            },
                            Link {
                                trace_id: vec![
                                    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
                                ],
                                span_id: vec![10, 11, 12, 13, 14, 15, 16, 17],
                                attributes: vec![KeyValue::new(
                                    "link.key1",
                                    AnyValue::new_string("link1"),
                                )],
                                ..Default::default()
                            },
                        ],
                        status: Some(Status {
                            code: 1,
                            message: "OK".into(),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        assert_traces_equivalent(&[request1], &[request2]);
    }
}
