// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Log equivalence checking.

use crate::proto::opentelemetry::logs::v1::{LogsData, ResourceLogs, ScopeLogs};
use crate::testing::equiv::canonical::{
    assert_equivalent, canonicalize_any_value, canonicalize_vec,
};

/// Split a LogsData into individual singleton LogsData messages (one per log record).
fn logs_split_into_singletons(logs_data: &LogsData) -> Vec<LogsData> {
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

/// Canonicalize a presumed-singleton LogsData by sorting all
/// item-level fields that need canonical ordering.
fn logs_canonicalize_singleton_in_place(logs_data: &mut LogsData) {
    // Canonicalize resource attributes
    for resource_logs in &mut logs_data.resource_logs {
        if let Some(resource) = &mut resource_logs.resource {
            canonicalize_vec(&mut resource.attributes, |attr| {
                if let Some(value) = &mut attr.value {
                    canonicalize_any_value(value);
                }
            });
        }

        // Canonicalize scope attributes
        for scope_logs in &mut resource_logs.scope_logs {
            if let Some(scope) = &mut scope_logs.scope {
                canonicalize_vec(&mut scope.attributes, |attr| {
                    if let Some(value) = &mut attr.value {
                        canonicalize_any_value(value);
                    }
                });
            }

            // Canonicalize log record attributes
            for log_record in &mut scope_logs.log_records {
                canonicalize_vec(&mut log_record.attributes, |attr| {
                    if let Some(value) = &mut attr.value {
                        canonicalize_any_value(value);
                    }
                });

                // Canonicalize body
                if let Some(body) = &mut log_record.body {
                    canonicalize_any_value(body);
                }
            }
        }
    }
}

/// Assert that two collections of `LogsData` instances are equivalent.
pub fn assert_logs_equivalent(left: &[LogsData], right: &[LogsData]) {
    assert_equivalent(
        left,
        right,
        logs_split_into_singletons,
        logs_canonicalize_singleton_in_place,
        "LogsData",
    );
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

    #[test]
    fn test_logs_equivalent_unordered_attributes() {
        // Test that attributes in different orders are considered equivalent
        let request1 = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue::new("service", AnyValue::new_string("test")),
                        KeyValue::new("version", AnyValue::new_string("1.0")),
                        KeyValue::new("env", AnyValue::new_string("prod")),
                    ],
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
                        attributes: vec![
                            KeyValue::new("key1", AnyValue::new_string("value1")),
                            KeyValue::new("key2", AnyValue::new_int(42)),
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        // Same data but attributes in different order
        let request2 = LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue::new("env", AnyValue::new_string("prod")),
                        KeyValue::new("service", AnyValue::new_string("test")),
                        KeyValue::new("version", AnyValue::new_string("1.0")),
                    ],
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
                        attributes: vec![
                            KeyValue::new("key2", AnyValue::new_int(42)),
                            KeyValue::new("key1", AnyValue::new_string("value1")),
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        assert_logs_equivalent(&[request1], &[request2]);
    }
}
