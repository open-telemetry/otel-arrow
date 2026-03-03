// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test fixture data for OTLP edge cases.
//!
//! This module provides pre-constructed test data for various edge cases
//! in OTLP telemetry data, including empty resources, scopes, attributes,
//! and other optional field combinations.

use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use crate::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use crate::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Gauge, Histogram, HistogramDataPoint, Metric, MetricsData,
    NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
};
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, TracesData, span::SpanKind, status::StatusCode,
};

//
// Logs Fixtures
//

/// Two scopes with two logs each, for testing tree structure.
#[must_use]
pub fn logs_with_full_resource_and_scope() -> LogsData {
    // 2025-01-15T10:30:00.000Z in nanoseconds
    const BASE_TIME: u64 = 1_736_937_000_000_000_000;
    const ONE_SEC: u64 = 1_000_000_000;

    LogsData::new(vec![ResourceLogs::new(
        Resource::build()
            .attributes(vec![KeyValue::new("res.id", AnyValue::new_string("self"))])
            .finish(),
        vec![
            ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope-alpha".to_string())
                    .version("1.0.0".to_string())
                    .attributes(vec![KeyValue::new(
                        "scopekey",
                        AnyValue::new_string("scopeval"),
                    )])
                    .finish(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(BASE_TIME)
                        .observed_time_unix_nano(BASE_TIME + 100_000_000)
                        .severity_number(SeverityNumber::Info as i32)
                        .event_name("event_1")
                        .body(AnyValue::new_string("first log in alpha"))
                        .finish(),
                    LogRecord::build()
                        .time_unix_nano(BASE_TIME + ONE_SEC)
                        .observed_time_unix_nano(BASE_TIME + ONE_SEC + 100_000_000)
                        .severity_number(SeverityNumber::Warn as i32)
                        .body(AnyValue::new_string("second log in alpha"))
                        .finish(),
                ],
            ),
            ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope-beta".to_string())
                    .version("2.0.0".to_string())
                    .finish(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(BASE_TIME + 2 * ONE_SEC)
                        .observed_time_unix_nano(BASE_TIME + 2 * ONE_SEC + 100_000_000)
                        .severity_number(SeverityNumber::Error as i32)
                        .severity_text("HOTHOT")
                        .body(AnyValue::new_string("first log in beta"))
                        .finish(),
                    LogRecord::build()
                        .time_unix_nano(BASE_TIME + 3 * ONE_SEC)
                        .observed_time_unix_nano(BASE_TIME + 3 * ONE_SEC + 100_000_000)
                        .severity_number(SeverityNumber::Debug as i32)
                        .event_name("event_2")
                        .attributes(vec![KeyValue::new(
                            "detail",
                            AnyValue::new_string("no body here"),
                        )])
                        .finish(),
                ],
            ),
        ],
    )])
}

/// Logs with no resource
#[must_use]
pub fn logs_with_no_resource() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::default(),
        vec![ScopeLogs::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(),
            vec![
                LogRecord::build()
                    .time_unix_nano(1000u64)
                    .observed_time_unix_nano(1100u64)
                    .severity_number(SeverityNumber::Info as i32)
                    .finish(),
            ],
        )],
    )])
}

/// One log with empty scope
#[must_use]
pub fn log_with_no_scope() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::build()
            .attributes(vec![KeyValue::new(
                "resource",
                AnyValue::new_string("value"),
            )])
            .finish(),
        vec![ScopeLogs::new(
            InstrumentationScope::default(),
            vec![
                LogRecord::build()
                    .time_unix_nano(1000u64)
                    .observed_time_unix_nano(1100u64)
                    .severity_number(SeverityNumber::Info as i32)
                    .finish(),
            ],
        )],
    )])
}

/// Logs with no resource, no scope
#[must_use]
pub fn logs_with_no_resource_no_scope() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::default(),
        vec![ScopeLogs::new(
            InstrumentationScope::default(),
            vec![
                LogRecord::build()
                    .attributes(vec![
                        KeyValue::new("lk1", AnyValue::new_string("attr")),
                        KeyValue::new("lk2", AnyValue::new_int(2)),
                    ])
                    .time_unix_nano(1000u64)
                    .observed_time_unix_nano(1100u64)
                    .severity_number(SeverityNumber::Info as i32)
                    .finish(),
            ],
        )],
    )])
}

/// Logs with resource and scope, no attributes
#[must_use]
pub fn logs_with_no_attributes() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::build().finish(),
        vec![ScopeLogs::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(),
            vec![
                LogRecord::build()
                    .time_unix_nano(1000u64)
                    .observed_time_unix_nano(1100u64)
                    .severity_number(SeverityNumber::Info as i32)
                    .finish(),
            ],
        )],
    )])
}

/// Completely empty logs data
#[must_use]
pub fn empty_logs() -> LogsData {
    LogsData::new(vec![])
}

/// Resource with no scope logs
#[must_use]
pub fn logs_with_empty_scope_logs() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(Resource::build().finish(), vec![])])
}

/// Scope with no log records
#[must_use]
pub fn logs_with_empty_log_records() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::build().finish(),
        vec![ScopeLogs::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            vec![],
        )],
    )])
}

/// LogRecord whose body is an empty string
#[must_use]
pub fn logs_with_body_empty_string() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::build().finish(),
        vec![ScopeLogs::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            vec![LogRecord::build().body(AnyValue::new_string("")).finish()],
        )],
    )])
}

/// Multiple log records with no resource
#[must_use]
pub fn logs_multiple_records_no_resource() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::default(),
        vec![ScopeLogs::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            vec![
                LogRecord::build()
                    .time_unix_nano(1000u64)
                    .observed_time_unix_nano(1100u64)
                    .severity_number(SeverityNumber::Info as i32)
                    .finish(),
                LogRecord::build()
                    .time_unix_nano(2000u64)
                    .observed_time_unix_nano(2100u64)
                    .severity_number(SeverityNumber::Warn as i32)
                    .finish(),
                LogRecord::build()
                    .time_unix_nano(3000u64)
                    .observed_time_unix_nano(3100u64)
                    .severity_number(SeverityNumber::Error as i32)
                    .finish(),
            ],
        )],
    )])
}

/// Logs with scopes with no resource
#[must_use]
pub fn logs_multiple_scopes_no_resource() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::default(),
        vec![
            ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope1".to_string())
                    .finish(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(1000u64)
                        .observed_time_unix_nano(1100u64)
                        .severity_number(SeverityNumber::Info as i32)
                        .finish(),
                ],
            ),
            ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope2".to_string())
                    .finish(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(2000u64)
                        .observed_time_unix_nano(2100u64)
                        .severity_number(SeverityNumber::Warn as i32)
                        .finish(),
                ],
            ),
        ],
    )])
}

/// Logs with multiple resources with different content
#[must_use]
pub fn logs_multiple_resources_mixed_content() -> LogsData {
    LogsData::new(vec![
        ResourceLogs::new(
            Resource::default(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope1".to_string())
                    .finish(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(1000u64)
                        .observed_time_unix_nano(1100u64)
                        .severity_number(SeverityNumber::Info as i32)
                        .finish(),
                ],
            )],
        ),
        ResourceLogs::new(
            Resource::build().finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::default(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(2000u64)
                        .observed_time_unix_nano(2100u64)
                        .severity_number(SeverityNumber::Warn as i32)
                        .finish(),
                ],
            )],
        ),
    ])
}

/// Generate logs with varying attributes and properties that follow some semantic
/// conventions. This can be used to generate somewhat realistic set of records that
/// of various batch sizes that could be used to test transformations such as filtering
#[must_use]
pub fn logs_with_varying_attributes_and_properties(batch_size: usize) -> LogsData {
    let log_records = (0..batch_size)
        .map(|i| {
            // generate some log attributes that somewhat follow semantic conventions
            let attrs = vec![
                KeyValue::new(
                    "code.namespace",
                    AnyValue::new_string(match i % 3 {
                        0 => "main",
                        1 => "otap_dataflow_engine",
                        _ => "arrow::array",
                    }),
                ),
                KeyValue::new("code.line.number", AnyValue::new_int((i % 5) as i64)),
            ];

            // cycle through severity numbers
            // 5 = DEBUG, 9 = INFO, 13 = WARN, 17 = ERROR
            let severity_number =
                SeverityNumber::try_from(((i % 4) * 4 + 1) as i32).expect("valid severity_number");
            let severity_text = severity_number
                .as_str_name()
                .split("_") // Note: this splitting something like SEVERITY_NUMBER_INFO
                .nth(2)
                .expect("can parse severity_text");
            let event_name = format!("event {}", i);
            let time_unix_nano = i as u64;

            LogRecord::build()
                .attributes(attrs)
                .event_name(event_name)
                .severity_number(severity_number)
                .severity_text(severity_text)
                .time_unix_nano(time_unix_nano)
                .finish()
        })
        .collect::<Vec<_>>();

    LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

//
// Traces Fixtures
//

/// Traces with full resource and scope
#[must_use]
pub fn traces_with_full_resource_and_scope() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::build().finish(),
        vec![ScopeSpans::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(),
            vec![
                Span::build()
                    .trace_id(vec![1u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span1".to_string())
                    .kind(SpanKind::Internal)
                    .start_time_unix_nano(1000u64)
                    .end_time_unix_nano(2000u64)
                    .status(Status::default())
                    .finish(),
                Span::build()
                    .trace_id(vec![2u8; 16])
                    .span_id(vec![2u8; 8])
                    .name("span2".to_string())
                    .kind(SpanKind::Server)
                    .start_time_unix_nano(3000u64)
                    .end_time_unix_nano(4000u64)
                    .status(Status::default())
                    .finish(),
            ],
        )],
    )])
}

/// Traces with no resource
#[must_use]
pub fn traces_with_no_resource() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::default(),
        vec![ScopeSpans::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(),
            vec![
                Span::build()
                    .trace_id(vec![1u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span1".to_string())
                    .kind(SpanKind::Internal)
                    .start_time_unix_nano(1000u64)
                    .end_time_unix_nano(2000u64)
                    .finish(),
            ],
        )],
    )])
}

/// Traces with no scope
#[must_use]
pub fn traces_with_no_scope() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::build()
            .attributes(vec![KeyValue::new(
                "resource",
                AnyValue::new_string("value"),
            )])
            .finish(),
        vec![ScopeSpans::new(
            InstrumentationScope::default(),
            vec![
                Span::build()
                    .trace_id(vec![1u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span1".to_string())
                    .kind(SpanKind::Internal)
                    .start_time_unix_nano(1000u64)
                    .end_time_unix_nano(2000u64)
                    .finish(),
            ],
        )],
    )])
}

/// Traces with neither resource nor scope data
#[must_use]
pub fn traces_with_no_resource_no_scope() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::default(),
        vec![ScopeSpans::new(
            InstrumentationScope::default(),
            vec![
                Span::build()
                    .attributes(vec![
                        KeyValue::new("sk1", AnyValue::new_string("attr")),
                        KeyValue::new("sk2", AnyValue::new_int(2)),
                    ])
                    .trace_id(vec![1u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span1".to_string())
                    .kind(SpanKind::Internal)
                    .start_time_unix_nano(1000u64)
                    .end_time_unix_nano(2000u64)
                    .finish(),
            ],
        )],
    )])
}

/// Traces with resource and scope but no attributes
#[must_use]
pub fn traces_with_no_attributes() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::build().finish(),
        vec![ScopeSpans::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(),
            vec![
                Span::build()
                    .trace_id(vec![1u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span1".to_string())
                    .kind(SpanKind::Internal)
                    .start_time_unix_nano(1000u64)
                    .end_time_unix_nano(2000u64)
                    .finish(),
            ],
        )],
    )])
}

/// Completely empty traces data
#[must_use]
pub fn empty_traces() -> TracesData {
    TracesData::new(vec![])
}

/// Resource with no scope spans
#[must_use]
pub fn traces_with_empty_scope_spans() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(Resource::build().finish(), vec![])])
}

/// Scope with no spans
#[must_use]
pub fn traces_with_empty_spans() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::build().finish(),
        vec![ScopeSpans::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            vec![],
        )],
    )])
}

/// Multiple spans with no resource
#[must_use]
pub fn traces_multiple_spans_no_resource() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::default(),
        vec![ScopeSpans::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            vec![
                Span::build()
                    .trace_id(vec![1u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span1".to_string())
                    .kind(SpanKind::Internal)
                    .start_time_unix_nano(1000u64)
                    .end_time_unix_nano(2000u64)
                    .finish(),
                Span::build()
                    .trace_id(vec![2u8; 16])
                    .span_id(vec![2u8; 8])
                    .name("span2".to_string())
                    .kind(SpanKind::Server)
                    .start_time_unix_nano(3000u64)
                    .end_time_unix_nano(4000u64)
                    .finish(),
                Span::build()
                    .trace_id(vec![3u8; 16])
                    .span_id(vec![3u8; 8])
                    .name("span3".to_string())
                    .kind(SpanKind::Client)
                    .start_time_unix_nano(5000u64)
                    .end_time_unix_nano(6000u64)
                    .finish(),
            ],
        )],
    )])
}

/// Multiple scopes with no resource
#[must_use]
pub fn traces_multiple_scopes_no_resource() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::default(),
        vec![
            ScopeSpans::new(
                InstrumentationScope::build()
                    .name("scope1".to_string())
                    .finish(),
                vec![
                    Span::build()
                        .trace_id(vec![1u8; 16])
                        .span_id(vec![1u8; 8])
                        .name("span1".to_string())
                        .kind(SpanKind::Internal)
                        .start_time_unix_nano(1000u64)
                        .end_time_unix_nano(2000u64)
                        .finish(),
                ],
            ),
            ScopeSpans::new(
                InstrumentationScope::build()
                    .name("scope2".to_string())
                    .finish(),
                vec![
                    Span::build()
                        .trace_id(vec![2u8; 16])
                        .span_id(vec![2u8; 8])
                        .name("span2".to_string())
                        .kind(SpanKind::Server)
                        .start_time_unix_nano(3000u64)
                        .end_time_unix_nano(4000u64)
                        .finish(),
                ],
            ),
        ],
    )])
}

/// Multiple resources with different content
#[must_use]
pub fn traces_multiple_resources_mixed_content() -> TracesData {
    TracesData::new(vec![
        ResourceSpans::new(
            Resource::default(),
            vec![ScopeSpans::new(
                InstrumentationScope::build()
                    .name("scope1".to_string())
                    .finish(),
                vec![
                    Span::build()
                        .trace_id(vec![1u8; 16])
                        .span_id(vec![1u8; 8])
                        .name("span1".to_string())
                        .kind(SpanKind::Internal)
                        .start_time_unix_nano(1000u64)
                        .end_time_unix_nano(2000u64)
                        .finish(),
                ],
            )],
        ),
        ResourceSpans::new(
            Resource::build().finish(),
            vec![ScopeSpans::new(
                InstrumentationScope::default(),
                vec![
                    Span::build()
                        .trace_id(vec![2u8; 16])
                        .span_id(vec![2u8; 8])
                        .name("span2".to_string())
                        .kind(SpanKind::Server)
                        .start_time_unix_nano(3000u64)
                        .end_time_unix_nano(4000u64)
                        .finish(),
                ],
            )],
        ),
    ])
}

//
// Metrics Fixtures
//

/// Metrics with full resource, scope, and data points
#[must_use]
pub fn metrics_sum_with_full_resource_and_scope() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(),
            vec![
                Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![
                            NumberDataPoint::build()
                                .time_unix_nano(1000u64)
                                .value_int(42i64)
                                .finish(),
                            NumberDataPoint::build()
                                .time_unix_nano(2000u64)
                                .value_int(100i64)
                                .finish(),
                        ],
                    ))
                    .finish(),
            ],
        )],
    )])
}

/// Metrics with no resource
#[must_use]
pub fn metrics_sum_with_no_resource() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::default(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(),
            vec![
                Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![
                            NumberDataPoint::build()
                                .time_unix_nano(1000u64)
                                .value_int(42i64)
                                .finish(),
                        ],
                    ))
                    .finish(),
            ],
        )],
    )])
}

/// Metrics with no scope
#[must_use]
pub fn metrics_sum_with_no_scope() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build()
            .attributes(vec![KeyValue::new(
                "resource",
                AnyValue::new_string("value"),
            )])
            .finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::default(),
            vec![
                Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![
                            NumberDataPoint::build()
                                .time_unix_nano(1000u64)
                                .value_int(42i64)
                                .finish(),
                        ],
                    ))
                    .finish(),
            ],
        )],
    )])
}

/// Metrics with neither resource nor scope
#[must_use]
pub fn metrics_sum_with_no_resource_no_scope() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::default(),
        vec![ScopeMetrics::new(
            InstrumentationScope::default(),
            vec![
                Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![
                            NumberDataPoint::build()
                                .attributes(vec![
                                    KeyValue::new("mk1", AnyValue::new_string("attr")),
                                    KeyValue::new("mk2", AnyValue::new_int(2)),
                                ])
                                .time_unix_nano(1000u64)
                                .value_int(42i64)
                                .finish(),
                        ],
                    ))
                    .finish(),
            ],
        )],
    )])
}

/// Sum metric with no data points
#[must_use]
pub fn metrics_sum_with_no_data_points() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(),
            vec![
                Metric::build()
                    .name("test.counter")
                    .data_sum(Sum::new(AggregationTemporality::Cumulative, true, vec![]))
                    .finish(),
            ],
        )],
    )])
}

/// Completely empty metrics data
#[must_use]
pub fn empty_metrics() -> MetricsData {
    MetricsData::new(vec![])
}

/// Resource with no scope metrics
#[must_use]
pub fn metrics_with_no_scope_metrics() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![],
    )])
}

/// Scope with no metrics
#[must_use]
pub fn metrics_with_no_metrics() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            vec![],
        )],
    )])
}

/// Multiple Sum metrics with no resource
#[must_use]
pub fn metrics_multiple_sums_no_resource() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::default(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            vec![
                Metric::build()
                    .name("test.counter1")
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![
                            NumberDataPoint::build()
                                .time_unix_nano(1000u64)
                                .value_int(42i64)
                                .finish(),
                        ],
                    ))
                    .finish(),
                Metric::build()
                    .name("test.counter2")
                    .data_sum(Sum::new(
                        AggregationTemporality::Delta,
                        false,
                        vec![
                            NumberDataPoint::build()
                                .time_unix_nano(2000u64)
                                .value_int(99i64)
                                .finish(),
                        ],
                    ))
                    .finish(),
            ],
        )],
    )])
}

/// Configuration for generating metrics with specific shapes.
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Number of data points per gauge metric (one gauge per entry)
    pub gauge_points: Vec<usize>,
    /// Number of data points per sum metric (one sum per entry)
    pub sum_points: Vec<usize>,
    /// Number of data points per histogram metric (one histogram per entry)
    pub histogram_points: Vec<usize>,
    /// Number of data points per summary metric (one summary per entry)
    pub summary_points: Vec<usize>,
    /// Whether to add varying attributes to data points
    pub vary_attributes: bool,
    /// Number of distinct resources (default 1).
    pub num_resources: usize,
    /// Number of scopes per resource (default 1).
    pub scopes_per_resource: usize,
    /// Number of attributes per resource (default 0).
    pub resource_attrs: usize,
    /// Number of attributes per scope (default 0).
    pub scope_attrs: usize,
    /// Number of metadata attributes per metric (default 0).
    pub metric_attrs: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            gauge_points: Vec::new(),
            sum_points: Vec::new(),
            histogram_points: Vec::new(),
            summary_points: Vec::new(),
            vary_attributes: false,
            num_resources: 1,
            scopes_per_resource: 1,
            resource_attrs: 0,
            scope_attrs: 0,
            metric_attrs: 0,
        }
    }
}

impl MetricsConfig {
    /// Create a new empty metrics configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add gauge metrics with specified point counts
    #[must_use]
    pub fn with_gauges(mut self, points: Vec<usize>) -> Self {
        self.gauge_points = points;
        self
    }

    /// Add sum metrics with specified point counts
    #[must_use]
    pub fn with_sums(mut self, points: Vec<usize>) -> Self {
        self.sum_points = points;
        self
    }

    /// Add histogram metrics with specified point counts
    #[must_use]
    pub fn with_histograms(mut self, points: Vec<usize>) -> Self {
        self.histogram_points = points;
        self
    }

    /// Add summary metrics with specified point counts
    #[must_use]
    pub fn with_summaries(mut self, points: Vec<usize>) -> Self {
        self.summary_points = points;
        self
    }

    /// Enable varying attributes on data points
    #[must_use]
    pub const fn with_varying_attributes(mut self, vary: bool) -> Self {
        self.vary_attributes = vary;
        self
    }

    /// Set the number of distinct resources.
    #[must_use]
    pub const fn with_resources(mut self, n: usize) -> Self {
        self.num_resources = n;
        self
    }

    /// Set the number of scopes per resource.
    #[must_use]
    pub const fn with_scopes_per_resource(mut self, n: usize) -> Self {
        self.scopes_per_resource = n;
        self
    }

    /// Set the number of attributes per resource.
    #[must_use]
    pub const fn with_resource_attrs(mut self, n: usize) -> Self {
        self.resource_attrs = n;
        self
    }

    /// Set the number of attributes per scope.
    #[must_use]
    pub const fn with_scope_attrs(mut self, n: usize) -> Self {
        self.scope_attrs = n;
        self
    }

    /// Set the number of metadata attributes per metric.
    #[must_use]
    pub const fn with_metric_attrs(mut self, n: usize) -> Self {
        self.metric_attrs = n;
        self
    }

    /// Calculate total data point count across all metrics
    #[must_use]
    pub fn total_points(&self) -> usize {
        self.gauge_points.iter().sum::<usize>()
            + self.sum_points.iter().sum::<usize>()
            + self.histogram_points.iter().sum::<usize>()
            + self.summary_points.iter().sum::<usize>()
    }

    /// Count total number of metrics
    #[must_use]
    pub const fn metric_count(&self) -> usize {
        self.gauge_points.len()
            + self.sum_points.len()
            + self.histogram_points.len()
            + self.summary_points.len()
    }
}

/// Configuration for generating logs with a specific number of log records.
#[derive(Debug, Clone)]
pub struct LogsConfig {
    /// Number of log records per scope.
    pub logs_per_scope: usize,
    /// Number of distinct resources (default 1).
    pub num_resources: usize,
    /// Number of scopes per resource (default 1).
    pub scopes_per_resource: usize,
    /// Number of attributes per resource (default 0).
    pub resource_attrs: usize,
    /// Number of attributes per scope (default 0).
    pub scope_attrs: usize,
    /// Number of attributes per log record (default 0).
    pub log_attrs: usize,
}

impl LogsConfig {
    /// Create a new `LogsConfig` with the given number of log records per scope.
    #[must_use]
    pub const fn new(logs_per_scope: usize) -> Self {
        Self {
            logs_per_scope,
            num_resources: 1,
            scopes_per_resource: 1,
            resource_attrs: 0,
            scope_attrs: 0,
            log_attrs: 0,
        }
    }

    /// Set the number of distinct resources.
    #[must_use]
    pub const fn with_resources(mut self, n: usize) -> Self {
        self.num_resources = n;
        self
    }

    /// Set the number of scopes per resource.
    #[must_use]
    pub const fn with_scopes_per_resource(mut self, n: usize) -> Self {
        self.scopes_per_resource = n;
        self
    }

    /// Set the number of attributes per resource.
    #[must_use]
    pub const fn with_resource_attrs(mut self, n: usize) -> Self {
        self.resource_attrs = n;
        self
    }

    /// Set the number of attributes per scope.
    #[must_use]
    pub const fn with_scope_attrs(mut self, n: usize) -> Self {
        self.scope_attrs = n;
        self
    }

    /// Set the number of attributes per log record.
    #[must_use]
    pub const fn with_log_attrs(mut self, n: usize) -> Self {
        self.log_attrs = n;
        self
    }
}

/// Configuration for generating traces with a specific number of spans.
#[derive(Debug, Clone)]
pub struct TracesConfig {
    /// Number of spans per scope.
    pub spans_per_scope: usize,
    /// Number of distinct resources (default 1).
    pub num_resources: usize,
    /// Number of scopes per resource (default 1).
    pub scopes_per_resource: usize,
    /// Number of attributes per resource (default 0).
    pub resource_attrs: usize,
    /// Number of attributes per scope (default 0).
    pub scope_attrs: usize,
    /// Number of attributes per span (default 0).
    pub span_attrs: usize,
}

impl TracesConfig {
    /// Create a new `TracesConfig` with the given number of spans per scope.
    #[must_use]
    pub const fn new(spans_per_scope: usize) -> Self {
        Self {
            spans_per_scope,
            num_resources: 1,
            scopes_per_resource: 1,
            resource_attrs: 0,
            scope_attrs: 0,
            span_attrs: 0,
        }
    }

    /// Set the number of distinct resources.
    #[must_use]
    pub const fn with_resources(mut self, n: usize) -> Self {
        self.num_resources = n;
        self
    }

    /// Set the number of scopes per resource.
    #[must_use]
    pub const fn with_scopes_per_resource(mut self, n: usize) -> Self {
        self.scopes_per_resource = n;
        self
    }

    /// Set the number of attributes per resource.
    #[must_use]
    pub const fn with_resource_attrs(mut self, n: usize) -> Self {
        self.resource_attrs = n;
        self
    }

    /// Set the number of attributes per scope.
    #[must_use]
    pub const fn with_scope_attrs(mut self, n: usize) -> Self {
        self.scope_attrs = n;
        self
    }

    /// Set the number of attributes per span.
    #[must_use]
    pub const fn with_span_attrs(mut self, n: usize) -> Self {
        self.span_attrs = n;
        self
    }
}

/// Generator for test data.
///
/// TODO: This is a placeholder, only varies timestamp_offset; add
/// more variation, use realistic schemas, deterministic randomness.
///
/// Note: see go/pkg/datagen for a Go package with similar goals.
///
/// Note: otap/batching_tests.rs uses these functions to exercise
/// itself by appending N copies of the messages returned below. Its
/// test coverage will improve with more variation here.
pub struct DataGenerator {
    limit: usize,
    count: usize,
    time_value: u64,
    metrics_config: Option<MetricsConfig>,
    logs_config: Option<LogsConfig>,
    traces_config: Option<TracesConfig>,
}

/// Generate `count` synthetic KeyValue attributes with the given prefix.
fn generate_attrs(prefix: &str, count: usize) -> Vec<KeyValue> {
    (0..count)
        .map(|i| {
            KeyValue::new(
                format!("{prefix}.attr_{i}"),
                AnyValue::new_string(format!("value_{i}")),
            )
        })
        .collect()
}

impl DataGenerator {
    /// Generate N 'limit' number of items
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self {
            limit,
            count: 0,

            // One million nanoseconds past the UTC epoch.
            time_value: 1_000_000_000_000_000,
            metrics_config: None,
            logs_config: None,
            traces_config: None,
        }
    }

    /// Create a DataGenerator with a specific metrics configuration
    #[must_use]
    pub const fn with_metrics_config(config: MetricsConfig) -> Self {
        Self {
            limit: 0,
            count: 0,
            time_value: 1_000_000_000_000_000,
            metrics_config: Some(config),
            logs_config: None,
            traces_config: None,
        }
    }

    /// Create a DataGenerator with a specific logs configuration
    #[must_use]
    pub const fn with_logs_config(config: LogsConfig) -> Self {
        Self {
            limit: 0,
            count: 0,
            time_value: 1_000_000_000_000_000,
            metrics_config: None,
            logs_config: Some(config),
            traces_config: None,
        }
    }

    /// Create a DataGenerator with a specific traces configuration
    #[must_use]
    pub const fn with_traces_config(config: TracesConfig) -> Self {
        Self {
            limit: 0,
            count: 0,
            time_value: 1_000_000_000_000_000,
            metrics_config: None,
            logs_config: None,
            traces_config: Some(config),
        }
    }
}

impl DataGenerator {
    /// Return a unique test timestamp.
    const fn timestamp(&mut self) -> u64 {
        let val = self.time_value;
        // add one second
        self.time_value += 1_000_000_000;
        val
    }

    /// Consume N points.
    fn consume(&mut self, n: usize) -> usize {
        let take = n.min(self.limit - self.count);
        self.count += take;
        take
    }

    /// Generate test OTLP logs data
    #[must_use]
    pub fn generate_logs(&mut self) -> LogsData {
        LogsData::new(vec![
            ResourceLogs::new(
                Resource::build().finish(),
                vec![
                    ScopeLogs::new(
                        InstrumentationScope::build()
                            .name("scope".to_string())
                            .finish(),
                        vec![
                            LogRecord::build()
                                .time_unix_nano(self.timestamp())
                                .observed_time_unix_nano(self.timestamp())
                                .severity_number(SeverityNumber::Info as i32)
                                .finish(),
                        ],
                    ),
                    ScopeLogs::new(
                        InstrumentationScope::build()
                            .name("scope2".to_string())
                            .finish(),
                        vec![
                            LogRecord::build()
                                .time_unix_nano(self.timestamp())
                                .observed_time_unix_nano(self.timestamp())
                                .severity_number(SeverityNumber::Error as i32)
                                .finish(),
                        ],
                    ),
                ],
            ),
            ResourceLogs::new(
                Resource::build().finish(),
                vec![ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("scope".to_string())
                        .finish(),
                    vec![
                        LogRecord::build()
                            .time_unix_nano(self.timestamp())
                            .observed_time_unix_nano(self.timestamp())
                            .severity_number(SeverityNumber::Info as i32)
                            .finish(),
                    ],
                )],
            ),
        ])
    }

    /// Generate test OTLP traces data
    #[must_use]
    pub fn generate_traces(&mut self) -> TracesData {
        TracesData::new(vec![ResourceSpans::new(
            Resource::build().finish(),
            vec![ScopeSpans::new(
                InstrumentationScope::build().finish(),
                vec![
                    Span::build()
                        .trace_id(vec![0u8; 16])
                        .span_id(vec![1u8; 8])
                        .name("span0".to_string())
                        .start_time_unix_nano(self.timestamp())
                        .end_time_unix_nano(self.timestamp())
                        .status(Status::new(StatusCode::Ok, "ok"))
                        .finish(),
                    Span::build()
                        .trace_id(vec![0u8; 16])
                        .span_id(vec![2u8; 8])
                        .name("span1".to_string())
                        .start_time_unix_nano(self.timestamp())
                        .end_time_unix_nano(self.timestamp())
                        .status(Status::new(StatusCode::Ok, "ok"))
                        .finish(),
                    Span::build()
                        .trace_id(vec![0u8; 16])
                        .span_id(vec![3u8; 8])
                        .name("span2".to_string())
                        .start_time_unix_nano(self.timestamp())
                        .end_time_unix_nano(self.timestamp())
                        .status(Status::new(StatusCode::Ok, "ok"))
                        .finish(),
                ],
            )],
        )])
    }

    /// Generate test OTLP metrics data at a timestamp offset
    #[must_use]
    pub fn generate_metrics(&mut self) -> MetricsData {
        // TODO: @@@
        MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().finish(),
                vec![
                    Metric::build()
                        .name("gauge1")
                        .description("First gauge")
                        .unit("1")
                        .data_gauge(Gauge::new(self.build_gauge_data(3)))
                        .finish(),
                    Metric::build()
                        .name("gauge2")
                        .description("Second gauge")
                        .unit("By")
                        .data_gauge(Gauge::new(self.build_gauge_data(2)))
                        .finish(),
                    Metric::build()
                        .name("sum1")
                        .description("A sum")
                        .unit("1")
                        .data_sum(Sum::new(
                            AggregationTemporality::Delta,
                            true,
                            self.build_sum_data(1),
                        ))
                        .finish(),
                ],
            )],
        )])
    }

    /// Generate test OTLP metrics data using the configured MetricsConfig
    #[must_use]
    pub fn generate_metrics_from_config(&mut self) -> MetricsData {
        let config = self
            .metrics_config
            .as_ref()
            .expect("metrics_config must be set")
            .clone();

        let mut metrics = Vec::new();
        let vary_attrs = config.vary_attributes;
        let metric_attrs = generate_attrs("metric", config.metric_attrs);

        // Generate gauge metrics
        for (idx, &point_count) in config.gauge_points.iter().enumerate() {
            metrics.push(
                Metric::build()
                    .name(format!("gauge_{}", idx))
                    .description(format!("Gauge metric {}", idx))
                    .unit("1")
                    .metadata(metric_attrs.clone())
                    .data_gauge(Gauge::new(
                        self.build_number_data_points(point_count, vary_attrs),
                    ))
                    .finish(),
            );
        }

        // Generate sum metrics
        for (idx, &point_count) in config.sum_points.iter().enumerate() {
            metrics.push(
                Metric::build()
                    .name(format!("sum_{}", idx))
                    .description(format!("Sum metric {}", idx))
                    .unit("1")
                    .metadata(metric_attrs.clone())
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        self.build_number_data_points(point_count, vary_attrs),
                    ))
                    .finish(),
            );
        }

        // Generate histogram metrics
        for (idx, &point_count) in config.histogram_points.iter().enumerate() {
            metrics.push(
                Metric::build()
                    .name(format!("histogram_{}", idx))
                    .description(format!("Histogram metric {}", idx))
                    .unit("s")
                    .metadata(metric_attrs.clone())
                    .data_histogram(Histogram::new(
                        AggregationTemporality::Delta,
                        self.build_histogram_data_points(point_count, vary_attrs),
                    ))
                    .finish(),
            );
        }

        // Generate summary metrics
        for (idx, &point_count) in config.summary_points.iter().enumerate() {
            metrics.push(
                Metric::build()
                    .name(format!("summary_{}", idx))
                    .description(format!("Summary metric {}", idx))
                    .unit("ms")
                    .metadata(metric_attrs.clone())
                    .data_summary(Summary::new(
                        self.build_summary_data_points(point_count, vary_attrs),
                    ))
                    .finish(),
            );
        }

        let scope_attrs = generate_attrs("scope", config.scope_attrs);
        let resource_attrs = generate_attrs("resource", config.resource_attrs);
        let resource_metrics: Vec<ResourceMetrics> = (0..config.num_resources)
            .map(|r| {
                let scope_metrics: Vec<ScopeMetrics> = (0..config.scopes_per_resource)
                    .map(|s| {
                        ScopeMetrics::new(
                            InstrumentationScope::build()
                                .name(format!("scope_{s}"))
                                .attributes(scope_attrs.clone())
                                .finish(),
                            metrics.clone(),
                        )
                    })
                    .collect();
                let mut attrs = vec![KeyValue::new("resource.index", AnyValue::new_int(r as i64))];
                attrs.extend(resource_attrs.iter().cloned());
                ResourceMetrics::new(Resource::build().attributes(attrs).finish(), scope_metrics)
            })
            .collect();

        MetricsData::new(resource_metrics)
    }

    /// Generate test OTLP logs data using the configured LogsConfig.
    #[must_use]
    pub fn generate_logs_from_config(&mut self) -> LogsData {
        let config = self
            .logs_config
            .as_ref()
            .expect("logs_config must be set")
            .clone();

        let log_attrs = generate_attrs("log", config.log_attrs);
        let scope_attrs = generate_attrs("scope", config.scope_attrs);
        let resource_attrs = generate_attrs("resource", config.resource_attrs);
        let resource_logs: Vec<ResourceLogs> = (0..config.num_resources)
            .map(|r| {
                let scope_logs: Vec<ScopeLogs> = (0..config.scopes_per_resource)
                    .map(|s| {
                        let logs: Vec<LogRecord> = (0..config.logs_per_scope)
                            .map(|_| {
                                LogRecord::build()
                                    .time_unix_nano(self.timestamp())
                                    .observed_time_unix_nano(self.timestamp())
                                    .severity_number(SeverityNumber::Info as i32)
                                    .attributes(log_attrs.clone())
                                    .finish()
                            })
                            .collect();
                        ScopeLogs::new(
                            InstrumentationScope::build()
                                .name(format!("scope_{s}"))
                                .attributes(scope_attrs.clone())
                                .finish(),
                            logs,
                        )
                    })
                    .collect();
                let mut attrs = vec![KeyValue::new("resource.index", AnyValue::new_int(r as i64))];
                attrs.extend(resource_attrs.iter().cloned());
                ResourceLogs::new(Resource::build().attributes(attrs).finish(), scope_logs)
            })
            .collect();

        LogsData::new(resource_logs)
    }

    /// Generate test OTLP traces data using the configured TracesConfig.
    #[must_use]
    pub fn generate_traces_from_config(&mut self) -> TracesData {
        let config = self
            .traces_config
            .as_ref()
            .expect("traces_config must be set")
            .clone();

        let span_attrs = generate_attrs("span", config.span_attrs);
        let scope_attrs = generate_attrs("scope", config.scope_attrs);
        let resource_attrs = generate_attrs("resource", config.resource_attrs);
        let mut span_counter: u64 = 0;
        let resource_spans: Vec<ResourceSpans> = (0..config.num_resources)
            .map(|r| {
                let scope_spans: Vec<ScopeSpans> = (0..config.scopes_per_resource)
                    .map(|s| {
                        let spans: Vec<Span> = (0..config.spans_per_scope)
                            .map(|_| {
                                span_counter += 1;
                                Span::build()
                                    .trace_id(vec![0u8; 16])
                                    .span_id({
                                        let mut id = [0u8; 8];
                                        let bytes = span_counter.to_be_bytes();
                                        id.copy_from_slice(&bytes);
                                        id.to_vec()
                                    })
                                    .name(format!("span_{span_counter}"))
                                    .start_time_unix_nano(self.timestamp())
                                    .end_time_unix_nano(self.timestamp())
                                    .status(Status::new(StatusCode::Ok, "ok"))
                                    .attributes(span_attrs.clone())
                                    .finish()
                            })
                            .collect();
                        ScopeSpans::new(
                            InstrumentationScope::build()
                                .name(format!("scope_{s}"))
                                .attributes(scope_attrs.clone())
                                .finish(),
                            spans,
                        )
                    })
                    .collect();
                let mut attrs = vec![KeyValue::new("resource.index", AnyValue::new_int(r as i64))];
                attrs.extend(resource_attrs.iter().cloned());
                ResourceSpans::new(Resource::build().attributes(attrs).finish(), scope_spans)
            })
            .collect();

        TracesData::new(resource_spans)
    }

    fn build_gauge_data(&mut self, n: usize) -> Vec<NumberDataPoint> {
        (0..self.consume(n))
            .map(|i| {
                NumberDataPoint::build()
                    .value_double(i as f64 * 10.0)
                    .time_unix_nano(self.timestamp())
                    // TODO: this will break a test
                    // .attributes(vec![KeyValue::new("G", AnyValue::new_int(i as i64))])
                    .finish()
            })
            .collect()
    }

    fn build_sum_data(&mut self, n: usize) -> Vec<NumberDataPoint> {
        (0..self.consume(n))
            .map(|i| {
                NumberDataPoint::build()
                    .value_double(i as f64 * 10.0)
                    .time_unix_nano(self.timestamp())
                    // TODO: this will break a test
                    // .value_int(i as i64 * 100)
                    // .start_time_unix_nano(self.timestamp())
                    // .attributes(vec![KeyValue::new(
                    //     "S",
                    //     AnyValue::new_string(format!("{i}")),
                    // )])
                    .finish()
            })
            .collect()
    }

    /// Build number data points (for gauge and sum metrics)
    fn build_number_data_points(&mut self, n: usize, vary_attrs: bool) -> Vec<NumberDataPoint> {
        (0..n)
            .map(|i| {
                let mut builder = NumberDataPoint::build()
                    .value_double((i as f64 + 1.0) * 10.0)
                    .time_unix_nano(self.timestamp());

                if vary_attrs {
                    builder = builder
                        .attributes(vec![KeyValue::new("point_id", AnyValue::new_int(i as i64))]);
                }

                builder.finish()
            })
            .collect()
    }

    /// Build histogram data points
    fn build_histogram_data_points(
        &mut self,
        n: usize,
        vary_attrs: bool,
    ) -> Vec<HistogramDataPoint> {
        (0..n)
            .map(|i| {
                let mut builder = HistogramDataPoint::build()
                    .time_unix_nano(self.timestamp())
                    .count(10 + i as u64)
                    .sum((100 + i * 10) as f64)
                    .bucket_counts(vec![1, 2, 3, 4])
                    .explicit_bounds(vec![0.0, 10.0, 50.0, 100.0]);

                if vary_attrs {
                    builder = builder
                        .attributes(vec![KeyValue::new("point_id", AnyValue::new_int(i as i64))]);
                }

                builder.finish()
            })
            .collect()
    }

    /// Build summary data points
    fn build_summary_data_points(&mut self, n: usize, vary_attrs: bool) -> Vec<SummaryDataPoint> {
        use crate::proto::opentelemetry::metrics::v1::summary_data_point::ValueAtQuantile;

        (0..n)
            .map(|i| {
                let mut builder = SummaryDataPoint::build()
                    .time_unix_nano(self.timestamp())
                    .count(20 + i as u64)
                    .sum((200 + i * 20) as f64)
                    .quantile_values(vec![
                        ValueAtQuantile {
                            quantile: 0.5,
                            value: 50.0 + i as f64,
                        },
                        ValueAtQuantile {
                            quantile: 0.95,
                            value: 95.0 + i as f64,
                        },
                    ]);

                if vary_attrs {
                    builder = builder
                        .attributes(vec![KeyValue::new("point_id", AnyValue::new_int(i as i64))]);
                }

                builder.finish()
            })
            .collect()
    }
}
