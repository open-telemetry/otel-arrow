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
    AggregationTemporality, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    Sum,
};
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, TracesData, span::SpanKind,
};

// ============================================================================
// Logs Fixtures
// ============================================================================

/// Logs with full resource and scope - baseline test with all fields populated
#[must_use]
pub fn logs_with_full_resource_and_scope() -> LogsData {
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
                LogRecord::build()
                    .time_unix_nano(2000u64)
                    .observed_time_unix_nano(2100u64)
                    .severity_number(SeverityNumber::Warn as i32)
                    .finish(),
            ],
        )],
    )])
}

/// Logs with no resource - empty Resource
#[must_use]
pub fn logs_with_no_resource() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::default(), // Empty resource
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

/// Logs with no scope - empty InstrumentationScope
#[must_use]
pub fn logs_with_no_scope() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::build()
            .attributes(vec![KeyValue::new(
                "resource",
                AnyValue::new_string("value"),
            )])
            .finish(),
        vec![ScopeLogs::new(
            InstrumentationScope::default(), // Empty scope
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

/// Logs with neither resource nor scope data - minimal case
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

/// Logs with resource and scope but no attributes
#[must_use]
pub fn logs_with_no_attributes() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::build().finish(), // No attributes
        vec![ScopeLogs::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(), // No attributes
            vec![
                LogRecord::build()
                    .time_unix_nano(1000u64)
                    .observed_time_unix_nano(1100u64)
                    .severity_number(SeverityNumber::Info as i32)
                    // No attributes
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

/// Resource with no scope logs (no actual log records)
#[must_use]
pub fn logs_with_empty_scope_logs() -> LogsData {
    LogsData::new(vec![ResourceLogs::new(
        Resource::build().finish(),
        vec![], // No scope logs
    )])
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
            vec![], // No log records
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

/// Multiple scopes with no resource
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

/// Multiple resources with different content
#[must_use]
pub fn logs_multiple_resources_mixed_content() -> LogsData {
    LogsData::new(vec![
        ResourceLogs::new(
            Resource::default(), // Empty resource
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
            Resource::build().finish(), // Non-empty resource
            vec![ScopeLogs::new(
                InstrumentationScope::default(), // Empty scope
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

// ============================================================================
// Traces Fixtures
// ============================================================================

/// Traces with full resource and scope - baseline test with all fields populated
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

/// Traces with no resource - empty Resource
#[must_use]
pub fn traces_with_no_resource() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::default(), // Empty resource
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

/// Traces with no scope - empty InstrumentationScope
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
            InstrumentationScope::default(), // Empty scope
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

/// Traces with neither resource nor scope data - minimal case
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
        Resource::build().finish(), // No attributes
        vec![ScopeSpans::new(
            InstrumentationScope::build()
                .name("test-scope".to_string())
                .finish(), // No attributes
            vec![
                Span::build()
                    .trace_id(vec![1u8; 16])
                    .span_id(vec![1u8; 8])
                    .name("span1".to_string())
                    .kind(SpanKind::Internal)
                    .start_time_unix_nano(1000u64)
                    .end_time_unix_nano(2000u64)
                    // No attributes
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

/// Resource with no scope spans (no actual spans)
#[must_use]
pub fn traces_with_empty_scope_spans() -> TracesData {
    TracesData::new(vec![ResourceSpans::new(
        Resource::build().finish(),
        vec![], // No scope spans
    )])
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
            vec![], // No spans
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
            Resource::default(), // Empty resource
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
            Resource::build().finish(), // Non-empty resource
            vec![ScopeSpans::new(
                InstrumentationScope::default(), // Empty scope
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

// ============================================================================
// Metrics Fixtures
// ============================================================================

/// Metrics with full resource, scope, and data points - baseline test
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

/// Metrics with no resource - empty Resource
#[must_use]
pub fn metrics_sum_with_no_resource() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::default(), // Empty resource
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

/// Metrics with no scope - empty InstrumentationScope
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
            InstrumentationScope::default(), // Empty scope
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

/// Metrics with neither resource nor scope - minimal case
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
                    .data_sum(Sum::new(
                        AggregationTemporality::Cumulative,
                        true,
                        vec![], // No data points
                    ))
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

/// Resource with no scope metrics (no actual metrics)
#[must_use]
pub fn metrics_with_no_scope_metrics() -> MetricsData {
    MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![], // No scope metrics
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
            vec![], // No metrics
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
