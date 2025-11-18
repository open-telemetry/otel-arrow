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
    AggregationTemporality, Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
    ScopeMetrics, Sum, metric::Data, number_data_point::Value as NumberValue,
};
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::proto::opentelemetry::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, TracesData, span::SpanKind, status::StatusCode,
};

//
// Logs Fixtures
//

/// Two logs with full resource and scope
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
    time_value: u64,
}

impl Default for DataGenerator {
    fn default() -> Self {
        Self {
            // One million nanoseconds past the UTC epoch.
            time_value: 1_000_000_000_000_000,
        }
    }
}

impl DataGenerator {
    /// Return a unique test timestamp.
    fn timestamp(&mut self) -> u64 {
        let val = self.time_value;
        // add one second
        self.time_value += 1_000_000_000;
        val
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
        MetricsData::new(vec![ResourceMetrics::new(
            Resource::build().finish(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().finish(),
                vec![
                    Metric {
                        name: "gauge1".into(),
                        description: "First gauge".into(),
                        unit: "1".into(),
                        metadata: vec![],
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint {
                                    time_unix_nano: self.timestamp(),
                                    value: Some(NumberValue::AsDouble(10.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    time_unix_nano: self.timestamp(),
                                    value: Some(NumberValue::AsDouble(20.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    time_unix_nano: self.timestamp(),
                                    value: Some(NumberValue::AsDouble(1200.0)),
                                    ..Default::default()
                                },
                            ],
                        })),
                    },
                    Metric {
                        name: "gauge2".into(),
                        description: "Second gauge".into(),
                        unit: "1".into(),
                        metadata: vec![],
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint {
                                    time_unix_nano: self.timestamp(),
                                    value: Some(NumberValue::AsDouble(30.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    time_unix_nano: self.timestamp(),
                                    value: Some(NumberValue::AsDouble(40.0)),
                                    ..Default::default()
                                },
                            ],
                        })),
                    },
                    Metric {
                        name: "sum1".into(),
                        description: "A sum".into(),
                        unit: "1".into(),
                        metadata: vec![],
                        data: Some(Data::Sum(Sum {
                            data_points: vec![NumberDataPoint {
                                time_unix_nano: self.timestamp(),
                                value: Some(NumberValue::AsDouble(50.0)),
                                ..Default::default()
                            }],
                            aggregation_temporality: 1,
                            is_monotonic: true,
                        })),
                    },
                ],
            )],
        )])
    }
}
