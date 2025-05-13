// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;

pub mod traces {
    use super::*;
    use crate::pdata::{SpanID, TraceID};
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, status::StatusCode,
    };

    pub fn create_single_request() -> ExportTraceServiceRequest {
        let start_time = 1619712000000000000u64;
        let end_time = 1619712001000000000u64;
        let trace_id = TraceID::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let span_id = SpanID::new(&[1, 2, 3, 4, 5, 6, 7, 8]);

        let span = Span::build(trace_id, span_id, "test_span", start_time)
            .end_time_unix_nano(end_time)
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .status(Status::new("success", StatusCode::Ok))
            .finish();

        ExportTraceServiceRequest::new(vec![
            ResourceSpans::build(Resource::default())
                .scope_spans(vec![
                    ScopeSpans::build(InstrumentationScope::default())
                        .spans(vec![span])
                        .finish(),
                ])
                .finish(),
        ])
    }
}

pub mod metrics {
    use super::*;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;

    pub fn create_single_request() -> ExportMetricsServiceRequest {
        let timestamp = 1619712000000000000u64;

        let data_point = NumberDataPoint::build_double(timestamp + 1000000000, 42.0)
            .start_time_unix_nano(timestamp)
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .finish();

        let metric = Metric::build_gauge("test_gauge", Gauge::new(vec![data_point]))
            .description(format!("Test metric"))
            .unit("count")
            .finish();

        ExportMetricsServiceRequest::new(vec![
            ResourceMetrics::build(Resource::default())
                .scope_metrics(vec![
                    ScopeMetrics::build(InstrumentationScope::default())
                        .metrics(vec![metric])
                        .finish(),
                ])
                .finish(),
        ])
    }
}

pub mod logs {
    use super::*;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use crate::proto::opentelemetry::resource::v1::Resource;

    pub fn create_single_request() -> ExportLogsServiceRequest {
        let timestamp = 1619712000000000000u64;

        // TODO after we support event_name, set this to non-empty string
        // to ensure we correctly decode it
        // https://github.com/open-telemetry/otel-arrow/issues/422
        let event_name = "";

        let log_record = LogRecord::build(timestamp, SeverityNumber::Info, event_name)
            .severity_text("INFO")
            .body(AnyValue::new_string(format!("Test log message")))
            .attributes(vec![KeyValue::new(
                "test.attribute",
                AnyValue::new_string("test value"),
            )])
            .trace_id((0u8..16u8).into_iter().collect::<Vec<u8>>())
            .span_id((0u8..8u8).into_iter().collect::<Vec<u8>>())
            .finish();

        ExportLogsServiceRequest::new(vec![
            ResourceLogs::build(Resource::default())
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::default())
                        .log_records(vec![log_record])
                        .finish(),
                ])
                .finish(),
        ])
    }
}
