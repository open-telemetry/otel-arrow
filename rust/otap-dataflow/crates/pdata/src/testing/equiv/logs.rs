// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log equivalence checking.

use super::canonical::{compare_attributes, CanonicalValue};
use crate::proto::opentelemetry::{
    collector::logs::v1::ExportLogsServiceRequest,
    common::v1::InstrumentationScope,
    logs::v1::LogRecord,
    resource::v1::Resource,
};
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// A flattened, comparable representation of a single log record with all its context.
///
/// This holds references to the original structs and implements comparison by
/// comparing fields in canonical order (with sorted attributes).
#[derive(Debug, Clone)]
pub struct LogItemKey<'a> {
    pub resource: Option<&'a Resource>,
    pub resource_schema_url: &'a str,
    pub scope: Option<&'a InstrumentationScope>,
    pub scope_schema_url: &'a str,
    pub log_record: &'a LogRecord,
}

impl<'a> PartialEq for LogItemKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<'a> Eq for LogItemKey<'a> {}

impl<'a> PartialOrd for LogItemKey<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for LogItemKey<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare resource
        match (self.resource, other.resource) {
            (Some(r1), Some(r2)) => {
                match compare_attributes(&r1.attributes, &r2.attributes) {
                    Ordering::Equal => {}
                    other => return other,
                }
                match r1.dropped_attributes_count.cmp(&r2.dropped_attributes_count) {
                    Ordering::Equal => {}
                    other => return other,
                }
            }
            (None, None) => {}
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
        }
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
            (None, None) => {}
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
        }
        match self.scope_schema_url.cmp(other.scope_schema_url) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare log record
        let lr1 = self.log_record;
        let lr2 = other.log_record;

        match lr1.time_unix_nano.cmp(&lr2.time_unix_nano) {
            Ordering::Equal => {}
            other => return other,
        }
        match lr1.observed_time_unix_nano.cmp(&lr2.observed_time_unix_nano) {
            Ordering::Equal => {}
            other => return other,
        }
        match lr1.severity_number.cmp(&lr2.severity_number) {
            Ordering::Equal => {}
            other => return other,
        }
        match lr1.severity_text.cmp(&lr2.severity_text) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare body
        let body1 = CanonicalValue::from(lr1.body.as_ref());
        let body2 = CanonicalValue::from(lr2.body.as_ref());
        match body1.cmp(&body2) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare attributes (sorted)
        match compare_attributes(&lr1.attributes, &lr2.attributes) {
            Ordering::Equal => {}
            other => return other,
        }

        match lr1.dropped_attributes_count.cmp(&lr2.dropped_attributes_count) {
            Ordering::Equal => {}
            other => return other,
        }
        match lr1.flags.cmp(&lr2.flags) {
            Ordering::Equal => {}
            other => return other,
        }
        match lr1.trace_id.cmp(&lr2.trace_id) {
            Ordering::Equal => {}
            other => return other,
        }
        lr1.span_id.cmp(&lr2.span_id)
    }
}

/// Iterator over all log items in a request, flattened with their context.
pub fn iter_log_items(
    request: &ExportLogsServiceRequest,
) -> impl Iterator<Item = LogItemKey<'_>> + '_ {
    request.resource_logs.iter().flat_map(|rl| {
        let resource = rl.resource.as_ref();
        let resource_schema_url = rl.schema_url.as_str();
        rl.scope_logs.iter().flat_map(move |sl| {
            let scope = sl.scope.as_ref();
            let scope_schema_url = sl.schema_url.as_str();
            sl.log_records.iter().map(move |lr| LogItemKey {
                resource,
                resource_schema_url,
                scope,
                scope_schema_url,
                log_record: lr,
            })
        })
    })
}

/// Assert that two log requests are semantically equivalent.
///
/// This compares the set of log items, ignoring structural differences like
/// how resources and scopes are grouped.
pub fn assert_logs_equivalent(
    expected: &ExportLogsServiceRequest,
    actual: &ExportLogsServiceRequest,
) {
    let expected_items: BTreeSet<LogItemKey<'_>> = iter_log_items(expected).collect();
    let actual_items: BTreeSet<LogItemKey<'_>> = iter_log_items(actual).collect();

    let missing_expected: Vec<_> = expected_items.difference(&actual_items).collect();
    let missing_actual: Vec<_> = actual_items.difference(&expected_items).collect();

    if !missing_expected.is_empty() {
        eprintln!("Missing expected log items ({} items):", missing_expected.len());
        for (i, item) in missing_expected.iter().enumerate().take(5) {
            eprintln!("  [{}] {:?}", i, item);
        }
        if missing_expected.len() > 5 {
            eprintln!("  ... and {} more", missing_expected.len() - 5);
        }
    }

    if !missing_actual.is_empty() {
        eprintln!("Unexpected log items ({} items):", missing_actual.len());
        for (i, item) in missing_actual.iter().enumerate().take(5) {
            eprintln!("  [{}] {:?}", i, item);
        }
        if missing_actual.len() > 5 {
            eprintln!("  ... and {} more", missing_actual.len() - 5);
        }
    }

    assert!(
        missing_expected.is_empty() && missing_actual.is_empty(),
        "Log requests are not equivalent: {} missing expected, {} unexpected",
        missing_expected.len(),
        missing_actual.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::{
        common::v1::{AnyValue, KeyValue},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    };

    #[test]
    fn test_logs_equivalent_identical() {
        let request1 = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue::new("service", AnyValue::new_string("test"))],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    }),
                    log_records: vec![LogRecord {
                        time_unix_nano: 1000,
                        body: Some(AnyValue::new_string("log1")),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let request2 = request1.clone();
        assert_logs_equivalent(&request1, &request2);
    }

    #[test]
    fn test_logs_equivalent_restructured() {
        // One resource with two logs
        let request1 = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue::new("service", AnyValue::new_string("test"))],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    }),
                    log_records: vec![
                        LogRecord {
                            time_unix_nano: 1000,
                            body: Some(AnyValue::new_string("log1")),
                            ..Default::default()
                        },
                        LogRecord {
                            time_unix_nano: 2000,
                            body: Some(AnyValue::new_string("log2")),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        // Same logs split into two resources (same attributes)
        let request2 = ExportLogsServiceRequest {
            resource_logs: vec![
                ResourceLogs {
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("service", AnyValue::new_string("test"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "scope1".into(),
                            ..Default::default()
                        }),
                        log_records: vec![LogRecord {
                            time_unix_nano: 1000,
                            body: Some(AnyValue::new_string("log1")),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ResourceLogs {
                    resource: Some(Resource {
                        attributes: vec![KeyValue::new("service", AnyValue::new_string("test"))],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: Some(InstrumentationScope {
                            name: "scope1".into(),
                            ..Default::default()
                        }),
                        log_records: vec![LogRecord {
                            time_unix_nano: 2000,
                            body: Some(AnyValue::new_string("log2")),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
        };

        assert_logs_equivalent(&request1, &request2);
    }
}
