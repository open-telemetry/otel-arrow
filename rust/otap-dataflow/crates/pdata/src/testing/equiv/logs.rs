// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log equivalence checking.

use crate::proto::opentelemetry::logs::v1::{LogsData, ResourceLogs, ScopeLogs};
use prost::Message;
use std::collections::BTreeSet;

/// Split a LogsData into individual singleton LogsData messages (one per log record).
///
/// This flattens the structure so each output contains exactly one log record
/// with its complete context (resource, scope).
fn split_into_singletons(logs_data: &LogsData) -> Vec<LogsData> {
    let mut result = Vec::new();

    for resource_logs in &logs_data.resource_logs {
        for scope_logs in &resource_logs.scope_logs {
            for log_record in &scope_logs.log_records {
                result.push(LogsData {
                    resource_logs: vec![ResourceLogs {
                        resource: resource_logs.resource.clone(),
                        scope_logs: vec![ScopeLogs {
                            scope: scope_logs.scope.clone(),
                            log_records: vec![log_record.clone()],
                            schema_url: scope_logs.schema_url.clone(),
                        }],
                        schema_url: resource_logs.schema_url.clone(),
                    }],
                });
            }
        }
    }

    result
}

/// Canonicalize a singleton LogsData by sorting all fields that need canonical ordering.
///
/// This mutates the message in place to sort:
/// - Resource attributes
/// - Scope attributes
/// - Log record attributes
/// - Key-value lists in AnyValue (recursively)
fn canonicalize_singleton(logs_data: &mut LogsData) {
    use crate::proto::opentelemetry::common::v1::any_value;

    // Helper to recursively canonicalize AnyValue
    fn canonicalize_any_value(av: &mut crate::proto::opentelemetry::common::v1::AnyValue) {
        if let Some(value) = &mut av.value {
            match value {
                any_value::Value::ArrayValue(arr) => {
                    for v in &mut arr.values {
                        canonicalize_any_value(v);
                    }
                }
                any_value::Value::KvlistValue(kvlist) => {
                    // Sort by key
                    kvlist.values.sort_by(|a, b| a.key.cmp(&b.key));
                    // Recursively canonicalize values
                    for kv in &mut kvlist.values {
                        if let Some(v) = &mut kv.value {
                            canonicalize_any_value(v);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Canonicalize resource attributes
    for resource_logs in &mut logs_data.resource_logs {
        if let Some(resource) = &mut resource_logs.resource {
            resource.attributes.sort_by(|a, b| a.key.cmp(&b.key));
            for attr in &mut resource.attributes {
                if let Some(value) = &mut attr.value {
                    canonicalize_any_value(value);
                }
            }
        }

        // Canonicalize scope attributes
        for scope_logs in &mut resource_logs.scope_logs {
            if let Some(scope) = &mut scope_logs.scope {
                scope.attributes.sort_by(|a, b| a.key.cmp(&b.key));
                for attr in &mut scope.attributes {
                    if let Some(value) = &mut attr.value {
                        canonicalize_any_value(value);
                    }
                }
            }

            // Canonicalize log record attributes
            for log_record in &mut scope_logs.log_records {
                log_record.attributes.sort_by(|a, b| a.key.cmp(&b.key));
                for attr in &mut log_record.attributes {
                    if let Some(value) = &mut attr.value {
                        canonicalize_any_value(value);
                    }
                }

                // Canonicalize body
                if let Some(body) = &mut log_record.body {
                    canonicalize_any_value(body);
                }
            }
        }
    }
}

/// Assert that two collections of `LogsData` instances are equivalent.
///
/// This checks all structured fields and attributes, comparing them in canonical order
/// so that field order doesn't matter.
///
/// Approach:
/// 1. Split each LogsData into singleton messages (one log record per message)
/// 2. Clone and canonicalize each singleton (sort attributes, key-value lists, etc.)
/// 3. Encode each canonicalized singleton as bytes using Prost encoding
/// 4. Collect encoded bytes into BTreeSet<Vec<u8>>
/// 5. Compare sets
///
/// Using bytes as the comparison key works around the fact that Prost doesn't
/// generate Eq/Ord for messages with f64 fields (due to NaN). The encoding
/// is deterministic for canonicalized messages.
///
/// # Panics
/// Panics if the two collections are not equivalent.
pub fn assert_logs_equivalent(left: &[LogsData], right: &[LogsData]) {
    // Split into singletons from all LogsData in the slices
    let mut left_singletons: Vec<LogsData> = left
        .iter()
        .flat_map(split_into_singletons)
        .collect();
    let mut right_singletons: Vec<LogsData> = right
        .iter()
        .flat_map(split_into_singletons)
        .collect();
    
    // Canonicalize each singleton
    for singleton in &mut left_singletons {
        canonicalize_singleton(singleton);
    }
    for singleton in &mut right_singletons {
        canonicalize_singleton(singleton);
    }
    
    // Encode to bytes and collect into BTreeSets
    let left_set: BTreeSet<Vec<u8>> = left_singletons
        .into_iter()
        .map(|msg| {
            let mut buf = Vec::new();
            msg.encode(&mut buf).expect("encoding should not fail");
            buf
        })
        .collect();
    
    let right_set: BTreeSet<Vec<u8>> = right_singletons
        .into_iter()
        .map(|msg| {
            let mut buf = Vec::new();
            msg.encode(&mut buf).expect("encoding should not fail");
            buf
        })
        .collect();
    
    if left_set != right_set {
        // For debugging, decode the differences back to messages
        let only_left: Vec<_> = left_set
            .difference(&right_set)
            .filter_map(|bytes| LogsData::decode(bytes.as_slice()).ok())
            .collect();
        let only_right: Vec<_> = right_set
            .difference(&left_set)
            .filter_map(|bytes| LogsData::decode(bytes.as_slice()).ok())
            .collect();
        
        panic!(
            "LogsData not equivalent:\n\
             Only in left ({} items):\n{:#?}\n\
             Only in right ({} items):\n{:#?}",
            only_left.len(),
            only_left,
            only_right.len(),
            only_right
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::common::v1::InstrumentationScope;
    use crate::proto::opentelemetry::{
        common::v1::{AnyValue, KeyValue},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    };

    #[test]
    fn test_logs_equivalent_identical() {
        let request1 = LogsData {
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
        assert_logs_equivalent(&[request1], &[request2]);
    }

    #[test]
    fn test_logs_equivalent_restructured() {
        // One resource with two logs
        let request1 = LogsData {
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
        let request2 = LogsData {
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

        assert_logs_equivalent(&[request1], &[request2]);
    }
}
